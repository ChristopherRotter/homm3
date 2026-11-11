use std::path::{Path, PathBuf};

use h3_contracts::{BattleSide, PlayerId};
use h3_game::{GameRuleset, GameSetup};
use h3_protocol::session::game::{GameOptions, GameOutbound, GameSession};
use h3_protocol::wire::game::{GameEvent, GameReject, MapIdentity};
use h3_protocol::wire::{Outcome, WireU64};

use super::*;

/// The player whose battle the element plays.
const PLAYER: PlayerId = PlayerId::new(0);

/// The repository's root.
fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

/// A test host's whole game (E17 M7 T3b2): an adventure replay fixture's game hosted in-process
/// by `h3_protocol::session::game`, as a map client hosts its game, its adventure commands played
/// up to the replay's first battle command; from then the battle's player's side is the seat's,
/// the other side the system actor's (a monster's), which the host plays by the plain choice.
pub(crate) struct Host {
    rules: &'static GameRuleset,
    session: GameSession<'static>,
    /// The player's messages not handed to the seat yet.
    inbox: Vec<ServerMessage>,
    /// The game's `BattleEnded` as the player was told it: its message's number and result.
    pub(crate) ended: Option<(u32, Outcome)>,
    /// How many of the seat's commands the game accepted, and refused.
    pub(crate) accepted: usize,
    pub(crate) refused: usize,
}

impl Host {
    /// `sea-deck-monster-s1`'s game (a hero's battle with a monster on a ship's deck) at the
    /// replay's first battle command: the battle open for the player.
    pub(crate) fn sea_monster() -> Self {
        let path = repo().join("fixtures/replays/adventure/sea-deck-monster-s1.toml");
        let replay = h3_files::adventure::parse_replay(&std::fs::read_to_string(path).unwrap())
            .unwrap();
        let ids: Vec<String> = (replay.header.packs.iter())
            .map(|(id, _)| id.clone())
            .collect();
        let rules =
            h3_files::adventure::load_rules(&repo().join("packs"), &ids, &replay.header.profile)
                .unwrap();
        // The session borrows its rules for the test's whole run.
        let rules: &'static GameRuleset = Box::leak(Box::new(rules));
        let (bytes, map) = h3_files::adventure::read_map(&repo(), &replay.map.path).unwrap();
        let raw = h3_map::read(&bytes, &h3_map::Limits::default()).unwrap();
        let setup = GameSetup {
            difficulty: replay.difficulty,
            ..h3_map::import(&raw, rules).unwrap()
        };
        let identity = MapIdentity {
            content_hash: WireU64(map.content_hash),
            import_version: map.import_version,
        };
        let options = GameOptions {
            game: 7,
            checkpoint_every: replay.checkpoint_every,
            record_bytes: usize::MAX,
        };
        let manifest = h3_protocol::map::game::manifest(rules, identity);
        let mut out = Vec::new();
        let session =
            GameSession::start(rules, &setup, replay.seed, manifest, options, &mut out).unwrap();
        let mut host = Self {
            rules,
            session,
            inbox: Vec::new(),
            ended: None,
            accepted: 0,
            refused: 0,
        };
        host.deliver(&out);
        let mut seq = 0;
        for (actor, command) in &replay.commands {
            if matches!(command, h3_game::GameCommand::Battle(_)) {
                break;
            }
            assert_eq!(*actor, h3_game::Actor::Player(PLAYER));
            seq += 1;
            let wire = h3_protocol::map::game::command(command);
            out.clear();
            host.session.play(PLAYER, seq, wire, &mut out).unwrap();
            host.deliver(&out);
        }
        host
    }

    /// The player's record's header: what its seat is made with.
    pub(crate) fn header(&self) -> GameRecordHeader {
        GameRecordHeader {
            format: h3_protocol::RECORD_FORMAT_VERSION,
            game: 7,
            player: PLAYER.raw(),
            manifest: self.session.manifest().clone(),
            first: 1,
        }
    }

    /// The player's frames among `out`, kept for the seat; the battle's end noted.
    fn deliver(&mut self, out: &[GameOutbound]) {
        for frame in out.iter().filter(|o| o.to == PLAYER) {
            let message: ServerMessage =
                h3_protocol::codec::decode(frame.frame.as_bytes(), usize::MAX).unwrap();
            if let ServerMessage::GameEvents { n, events } = &message {
                for event in events {
                    if let GameEvent::BattleEnded { result, .. } = event {
                        self.ended = Some((*n, *result));
                    }
                }
            }
            self.inbox.push(message);
        }
    }

    /// The player's battle side, its battle open.
    fn side(&self) -> Option<BattleSide> {
        let battle = self.session.state().battle(PLAYER)?;
        let index = battle.owners().iter().position(|&o| o == Some(PLAYER))?;
        Some(if index == 0 {
            BattleSide::Attacker
        } else {
            BattleSide::Defender
        })
    }

    /// A pass of the host's loop: what the seat sent played as the player's and its queries
    /// answered from the battle's state; the system's battle turns played (the plain choice);
    /// the player's messages handed to the seat.
    pub(crate) fn serve(&mut self, seat: &mut GameSeat) {
        let mut out = Vec::new();
        for sent in seat.take_outgoing() {
            match sent {
                ToHost::Command { seq, command } => match self.session.play(PLAYER, seq, command, &mut out) {
                    Ok(()) => self.accepted += 1,
                    Err(_) => self.refused += 1,
                },
                ToHost::Query { seq, query } => {
                    let (Some(battle), Some(side)) =
                        (self.session.state().battle(PLAYER), self.side())
                    else {
                        continue;
                    };
                    let targets =
                        h3_protocol::map::query(battle.state(), self.rules.battle(), side, query);
                    seat.answer(seq, targets);
                }
            }
        }
        let width = self.session.manifest().field.width;
        loop {
            let legal: Vec<Command> = (self.session.system_legal().into_iter())
                .filter_map(|c| match c {
                    GameCommand::Battle { command } => Some(command),
                    _ => None,
                })
                .collect();
            let Some(command) = crate::connect::plain(&legal, Seat::Defender, width) else {
                break;
            };
            self.session
                .system(GameCommand::Battle { command }, &mut out)
                .unwrap();
        }
        self.deliver(&out);
        for message in self.inbox.drain(..) {
            seat.receive(&message);
        }
    }

    /// The player's record as the game kept it, read by `h3_view::game_record`.
    pub(crate) fn battles(&self) -> Vec<h3_view::game_record::GameBattle> {
        let text = self.session.record(PLAYER).unwrap();
        h3_view::game_record::parse(&text).unwrap().battles()
    }
}

#[test]
fn a_host_s_game_s_battle_is_played_through_the_seat_to_its_end() {
    // E17 M7 T3b2: the seat follows the battle the host's game opens for its player, asks the
    // host for the turn's targets before it gives the turn, sends the player's commands (here
    // the plain choice of each turn, as a test's player), and its record grows as the game's
    // record shows the battle to the player: the same, message for message, to its end.
    let mut host = Host::sea_monster();
    let mut seat = GameSeat::new(&host.header());
    assert!(!seat.opened());
    assert_eq!((seat.seat(), seat.in_game()), (Seat::Spectator, None));
    assert!(seat.send(Command::Wait {}).is_err());
    host.serve(&mut seat);
    assert!(seat.opened());
    assert_eq!(seat.seat(), Seat::Attacker);
    // The turn waits for its queries' answers, which the next pass brings.
    assert_eq!(seat.poll(0), Ok(true));
    assert_eq!(seat.poll(0), Ok(false));
    assert!(seat.turn().is_none());
    let width = host.session.manifest().field.width;
    let mut refused_once = false;
    for _ in 0..500 {
        host.serve(&mut seat);
        seat.poll(0).unwrap();
        if seat.over() {
            break;
        }
        let Some((legal, _)) = seat.turn() else {
            continue;
        };
        let command = crate::connect::plain(legal, Seat::Attacker, width).unwrap();
        seat.send(command).unwrap();
        assert!(seat.turn().is_none());
        if !refused_once {
            // A refusal the battle's record holds nothing of (another code than the battle's)
            // frees the turn too; the host never saw the command.
            refused_once = true;
            let Some(ToHost::Command { seq, .. }) = seat.take_outgoing().pop() else {
                panic!("no command sent");
            };
            seat.receive(&ServerMessage::GameRejected {
                n: 0,
                seq,
                code: GameReject::NoBattle {},
            });
            assert!(seat.turn().is_some());
            seat.send(command).unwrap();
        }
    }
    assert!(seat.over(), "the battle did not end");
    assert!(host.accepted > 1 && host.refused == 0);
    let (n, result) = host.ended.unwrap();
    assert_eq!(seat.in_game().and_then(|at| at.ended), Some((n, result)));
    assert_eq!(seat.turn(), None);
    // An answer to no query of the turn's is dropped.
    seat.answer(9_999, Vec::new());
    assert!(!seat.answers.contains_key(&9_999));
    let battles = host.battles();
    assert_eq!(battles.len(), 1);
    assert_eq!(seat.record(), Some(&battles[0].record));
    assert_eq!(seat.in_game(), Some(battles[0].at));
}

#[test]
fn a_host_s_window_plays_its_game_s_battle_live_and_gets_its_outcome() {
    // E17 M7 T3b2: the element over the seat (`Ground::load_game`), its human side the player's
    // pressed by the unattended presses (the seat's plain choice), the host's game the other
    // side and the answers; the result window at the battle's end, its OK closing the screen,
    // the outcome the game told handed back with where the battle stands in the game.
    let Some(source) = crate::battle::tests::staged("seq-end.toml", [false, false]) else {
        return;
    };
    let mut host = Host::sea_monster();
    let mut seat = GameSeat::new(&host.header());
    assert!(crate::battle::Ground::load_game(&source, seat.clone())
        .unwrap_err()
        .contains("not opened"));
    host.serve(&mut seat);
    let ground = crate::battle::Ground::load_game(&source, seat).unwrap();
    assert_eq!(ground.content(), Some(h3_view::remote::Content::Same));
    assert!(ground.game_seat().is_some() && ground.online().is_none());
    let serve = || host.serve(&mut ground.game_seat().unwrap().borrow_mut());
    let (hosted, shown) = crate::window::tests::hosted_to_close_with(&ground, &source, serve);
    assert!(shown > 1);
    let (n, result) = host.ended.unwrap();
    assert_eq!(hosted.outcome(), Some(result));
    assert_eq!(ground.in_game().and_then(|at| at.ended), Some((n, result)));
    assert!(host.accepted > 1 && host.refused == 0);
}
