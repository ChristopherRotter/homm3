//! The session core (E16 M1 T2): scripted clients play hosted battles through it. Identity is the
//! seat, a rejected command changes nothing, a viewer's bytes do not depend on what it may not
//! see, and the journal is the replay.

use std::boxed::Box;
use std::collections::BTreeMap;
use std::format;
use std::string::{String, ToString};
use std::vec;
use std::vec::Vec;

use h3_battle::{BattleSide, BattleState, LegalCommands};
use h3_content::{Ruleset, SpellId};
use h3_kernel::{Rng, StreamId};
use proptest::prelude::*;

use super::full;
use crate::RECORD_FORMAT_VERSION;
use crate::codec::{self, decode_server};
use crate::map;
use crate::session::{Bot, ConnId, Entry, GameConfig, Outbound, RestoreError, SeatKind, Session};
use crate::wire::{
    ClientMessage, Command, GoodbyeReason, Outcome, Phase, Query, RecordHeader, Refusal,
    RejectCode, Seat, ServerMessage, Target,
};

const A: ConnId = ConnId(1);
const B: ConnId = ConnId(2);
const TOKENS: [&str; 2] = ["attacker-secret", "defender-secret"];

/// The sim's random agent as the host's bot.
#[derive(Debug)]
struct RandomBot(Rng);

impl Bot for RandomBot {
    fn command(
        &mut self,
        state: &BattleState,
        ruleset: &Ruleset,
        side: BattleSide,
        legal: &LegalCommands,
    ) -> h3_battle::Command {
        h3_sim::rollout::choose(state, ruleset, side, legal, &mut self.0)
    }
}

/// A bot whose every choice the engine rejects.
#[derive(Debug)]
struct WrongBot;

impl Bot for WrongBot {
    fn command(
        &mut self,
        _: &BattleState,
        _: &Ruleset,
        _: BattleSide,
        _: &LegalCommands,
    ) -> h3_battle::Command {
        h3_battle::Command::Heal {
            target: h3_battle::StackId(200),
        }
    }
}

fn players() -> [SeatKind; 2] {
    TOKENS.map(|token| SeatKind::Player {
        token: token.to_string(),
    })
}

fn random_bot(seed: u64) -> SeatKind {
    SeatKind::Bot(Box::new(RandomBot(Rng::for_stream(seed, StreamId::AGENT))))
}

/// Game 1: the `full` scenario with `seed`, a checkpoint every 10 commands.
fn config(seed: u64) -> GameConfig {
    GameConfig {
        game: 1,
        setup: full().1,
        seed,
        checkpoint_every: 10,
        shadow: false,
        spectators: 2,
        watchable: true,
        record_bytes: 1 << 20,
    }
}

/// A session of the `full` scenario with `seed`.
fn host<'r>(r: &'r Ruleset, seed: u64, seats: [SeatKind; 2]) -> Session<'r> {
    Session::new(r, config(seed), seats).unwrap()
}

/// The entries the session made since they were last drained.
fn drained(s: &mut Session<'_>) -> Vec<Entry> {
    s.drain_entries().collect()
}

fn ruleset() -> Ruleset {
    full().0
}

/// What `conn` sends, and the frames that come out.
fn send(s: &mut Session<'_>, conn: ConnId, message: ClientMessage) -> Vec<Outbound> {
    let mut out = Vec::new();
    s.handle(conn, message, &mut out);
    out
}

fn join(s: &mut Session<'_>, conn: ConnId, seat: Seat, token: Option<&str>) -> Vec<Outbound> {
    let token = token.map(str::to_string);
    send(
        s,
        conn,
        ClientMessage::Join {
            game: 1,
            seat,
            token,
        },
    )
}

fn decode(frame: &str) -> ServerMessage {
    decode_server(frame.as_bytes(), usize::MAX).unwrap()
}

/// The messages for `conn`, in order.
fn to(out: &[Outbound], conn: ConnId) -> Vec<ServerMessage> {
    out.iter()
        .filter(|o| o.to == conn)
        .map(|o| decode(&o.frame))
        .collect()
}

/// The frames for `conn`, in order.
fn frames(out: &[Outbound], conn: ConnId) -> Vec<String> {
    out.iter()
        .filter(|o| o.to == conn)
        .map(|o| o.frame.clone())
        .collect()
}

/// A message's number, if it has one that counts.
fn number(message: &ServerMessage) -> Option<u32> {
    match message {
        ServerMessage::Snapshot { n, .. }
        | ServerMessage::YourTurn { n, .. }
        | ServerMessage::Accepted { n, .. }
        | ServerMessage::Rejected { n, .. }
        | ServerMessage::QueryResult { n, .. }
        | ServerMessage::Events { n, .. }
        | ServerMessage::GameEnded { n, .. }
        | ServerMessage::PauseAsked { n, .. } => Some(*n),
        _ => None,
    }
}

/// The connection told to act in `out`, with its legal commands.
fn turn(out: &[Outbound]) -> Option<(ConnId, Vec<Command>)> {
    out.iter().rev().find_map(|o| match decode(&o.frame) {
        ServerMessage::YourTurn { legal, .. } => Some((o.to, legal)),
        _ => None,
    })
}

/// Answers every `YourTurn` with a legal command that stays in the battle, chosen by `rng` among
/// the strikes if there are any (so that battles end), until no one is told to act or `limit`
/// commands were sent; every frame, `first` included.
fn play(s: &mut Session<'_>, first: Vec<Outbound>, rng: &mut Rng, limit: u32) -> Vec<Outbound> {
    let mut all = first;
    let mut pending = all.clone();
    for seq in 1..=limit {
        let Some((conn, legal)) = turn(&pending) else {
            break;
        };
        let strikes: Vec<Command> = legal
            .iter()
            .copied()
            .filter(|c| matches!(c, Command::Attack { .. } | Command::Shoot { .. }))
            .collect();
        let staying: Vec<Command> = if strikes.is_empty() {
            legal
                .into_iter()
                .filter(|c| !matches!(c, Command::Retreat {} | Command::Surrender {}))
                .collect()
        } else {
            strikes
        };
        let command = staying[rng.below(u32::try_from(staying.len()).unwrap()) as usize];
        pending = send(s, conn, ClientMessage::Command { seq, command });
        all.extend(pending.iter().cloned());
    }
    all
}

/// Both players joined; every frame so far.
fn started(s: &mut Session<'_>) -> Vec<Outbound> {
    let mut out = join(s, A, Seat::Attacker, None);
    out.extend(join(s, B, Seat::Defender, None));
    out
}

/// The session's journal as the sim's replay.
fn replay(r: &Ruleset, s: &Session<'_>) -> h3_sim::Replay {
    let journal = s.journal().clone();
    h3_sim::Replay {
        header: h3_sim::ReplayHeader::current(r),
        seed: journal.seed,
        setup: journal.setup,
        commands: journal.commands,
        checkpoint_every: journal.checkpoint_every,
        checkpoints: journal.checkpoints,
        final_hash: s.state_hash(),
        result: s.result(),
    }
}

#[test]
fn two_players_join_and_play_a_hosted_battle_to_its_end() {
    let r = ruleset();
    let mut s = host(&r, 5, players());
    let first = join(&mut s, A, Seat::Attacker, None);
    let manifest = map::manifest(&r);
    match &to(&first, A)[..] {
        [
            ServerMessage::Joined {
                game: 1,
                seat: Seat::Attacker,
                token: Some(token),
                manifest: sent,
            },
            ServerMessage::Snapshot { n: 1, observation },
        ] => {
            assert_eq!(token, TOKENS[0]);
            assert_eq!(*sent, manifest);
            assert_eq!(observation.viewer, crate::wire::Side::Attacker);
        }
        other => panic!("{other:?}"),
    }
    assert!(!s.started(), "a seat is still empty");
    let second = join(&mut s, B, Seat::Defender, None);
    assert!(s.started());
    assert!(matches!(
        &to(&second, B)[..2],
        [ServerMessage::Joined { token: Some(t), .. }, ServerMessage::Snapshot { n: 1, .. }]
            if t == TOKENS[1]
    ));
    let (actor, _) = turn(&second).unwrap();
    let mut all = first;
    all.extend(play(&mut s, second, &mut Rng::from_seed(9), 5000));
    let result = s.result().expect("the battle is over");
    assert!(actor == A || actor == B);
    for conn in [A, B] {
        let messages = to(&all, conn);
        // The numbers count the viewer's messages without a gap.
        let numbers: Vec<u32> = messages.iter().filter_map(number).collect();
        assert_eq!(
            numbers,
            (1..=u32::try_from(numbers.len()).unwrap()).collect::<Vec<_>>()
        );
        assert!(matches!(
            messages.last(),
            Some(ServerMessage::GameEnded { result: r, .. }) if *r == map::outcome(result)
        ));
    }
    // Every accepted command was acknowledged to its seat, and journaled.
    let accepted = all
        .iter()
        .filter(|o| matches!(decode(&o.frame), ServerMessage::Accepted { .. }))
        .count();
    assert_eq!(accepted, s.journal().commands.len());
    // The journal is the replay: it re-runs to the same hashes and result.
    let replay = replay(&r, &s);
    assert_eq!(replay.checkpoints.len(), replay.commands.len() / 10);
    assert_eq!(replay.verify(&r), Ok(()));
}

#[test]
fn an_accepted_command_is_acknowledged_then_told_to_every_viewer() {
    let r = ruleset();
    let mut s = host(&r, 5, players());
    let out = started(&mut s);
    let (actor, legal) = turn(&out).unwrap();
    let other = if actor == A { B } else { A };
    let command = legal[0];
    let out = send(&mut s, actor, ClientMessage::Command { seq: 41, command });
    let mine = to(&out, actor);
    assert!(matches!(
        mine[0],
        ServerMessage::Accepted { seq: 41, command: c, .. } if c == command
    ));
    assert!(matches!(mine[1], ServerMessage::Events { .. }));
    assert!(matches!(to(&out, other)[0], ServerMessage::Events { .. }));
    assert_eq!(s.journal().commands.len(), 1);
}

#[test]
fn a_connection_without_a_seat_is_no_actor() {
    let r = ruleset();
    let mut s = host(&r, 5, players());
    let messages = [
        ClientMessage::Command {
            seq: 1,
            command: Command::Defend {},
        },
        ClientMessage::Query {
            seq: 2,
            query: Query::CastTargets { spell: 0 },
        },
        ClientMessage::FetchReplay { seq: 3 },
        ClientMessage::FetchRecord { seq: 4 },
    ];
    for (message, seq) in messages.into_iter().zip(1..) {
        let out = send(&mut s, A, message);
        assert_eq!(
            to(&out, A),
            [ServerMessage::Rejected {
                n: 0,
                seq,
                code: RejectCode::NotSeated
            }]
        );
    }
    // A seat before the start may not act.
    join(&mut s, A, Seat::Attacker, None);
    let hash = s.state_hash();
    let out = send(
        &mut s,
        A,
        ClientMessage::Command {
            seq: 5,
            command: Command::EndTactics {},
        },
    );
    assert_eq!(
        to(&out, A),
        [ServerMessage::Rejected {
            n: 2,
            seq: 5,
            code: RejectCode::NotStarted
        }]
    );
    assert_eq!(s.state_hash(), hash);
    assert!(s.journal().commands.is_empty());
    // The hello and the ping are the connection's, not the game's.
    for message in [
        ClientMessage::Hello {
            protocol: 2,
            client: String::new(),
            name: String::new(),
        },
        ClientMessage::Ping { nonce: 1 },
    ] {
        assert!(send(&mut s, A, message).is_empty());
    }
}

#[test]
fn joins_are_refused_with_their_reason() {
    let r = ruleset();
    let mut s = host(
        &r,
        5,
        [random_bot(1), players().into_iter().nth(1).unwrap()],
    );
    let refused = |out: Vec<Outbound>, conn| match &to(&out, conn)[..] {
        [ServerMessage::Refused { reason }] => *reason,
        other => panic!("{other:?}"),
    };
    let wrong_game = send(
        &mut s,
        A,
        ClientMessage::Join {
            game: 2,
            seat: Seat::Defender,
            token: None,
        },
    );
    assert_eq!(refused(wrong_game, A), Refusal::NoSuchGame);
    // Spectators up to the game's limit (two here); a spectator takes no seat.
    for conn in [ConnId(5), ConnId(6)] {
        assert!(matches!(
            to(&join(&mut s, conn, Seat::Spectator, None), conn)[..],
            [
                ServerMessage::Joined {
                    seat: Seat::Spectator,
                    token: None,
                    ..
                },
                ServerMessage::Snapshot { n: 1, .. }
            ]
        ));
    }
    let late = ConnId(7);
    assert_eq!(
        refused(join(&mut s, late, Seat::Spectator, None), late),
        Refusal::NoSpectators
    );
    for seat in [Seat::Defender, Seat::Spectator] {
        assert_eq!(
            refused(join(&mut s, ConnId(5), seat, None), ConnId(5)),
            Refusal::AlreadyJoined
        );
    }
    assert_eq!(
        refused(join(&mut s, A, Seat::Attacker, None), A),
        Refusal::SeatTaken,
        "the bot's seat"
    );
    // A wrong token, of another length or of the same: refused.
    for token in ["", "defender-secreT", "defender-secret-"] {
        assert_eq!(
            refused(join(&mut s, A, Seat::Defender, Some(token)), A),
            Refusal::BadToken
        );
    }
    join(&mut s, A, Seat::Defender, None);
    assert_eq!(
        refused(join(&mut s, A, Seat::Defender, None), A),
        Refusal::AlreadyJoined
    );
    assert_eq!(
        refused(join(&mut s, B, Seat::Defender, None), B),
        Refusal::SeatTaken
    );
    // Without a token, a seat once taken stays taken while its holder is away.
    s.disconnected(A);
    assert_eq!(
        refused(join(&mut s, B, Seat::Defender, None), B),
        Refusal::SeatTaken
    );
}

#[test]
fn a_seat_is_retaken_with_its_token_and_the_record_spans_the_gap() {
    let r = ruleset();
    let mut s = host(&r, 5, players());
    let out = started(&mut s);
    let (actor, legal) = turn(&out).unwrap();
    let (other, other_seat, other_token) = if actor == A {
        (B, Seat::Defender, TOKENS[1])
    } else {
        (A, Seat::Attacker, TOKENS[0])
    };
    // The other seat drops; the actor's command is still told to it, in its record only.
    s.disconnected(other);
    let out = send(
        &mut s,
        actor,
        ClientMessage::Command {
            seq: 1,
            command: legal[0],
        },
    );
    assert!(to(&out, other).is_empty());
    // It comes back on a new connection with its token: a fresh snapshot, numbers going on.
    let back = ConnId(3);
    let out = join(&mut s, back, other_seat, Some(other_token));
    let messages = to(&out, back);
    assert!(matches!(messages[0], ServerMessage::Joined { .. }));
    let ServerMessage::Snapshot { n, .. } = messages[1] else {
        panic!("{messages:?}")
    };
    assert_eq!(n, 3, "Snapshot 1, then Events 2 while away");
    let ServerMessage::Record {
        record,
        n: last,
        seq: 7,
    } = to(
        &send(&mut s, back, ClientMessage::FetchRecord { seq: 7 }),
        back,
    )
    .remove(0)
    else {
        panic!()
    };
    assert_eq!(record.lines().count(), 1 + last as usize);
    // A connection that re-takes a held seat replaces its holder.
    let again = ConnId(4);
    let out = join(&mut s, again, other_seat, Some(other_token));
    assert_eq!(
        to(&out, back),
        [ServerMessage::Goodbye {
            reason: GoodbyeReason::Replaced
        }]
    );
    assert!(out.iter().any(|o| o.to == back && o.close));
    assert!(matches!(to(&out, again)[0], ServerMessage::Joined { .. }));
    // The replaced connection is no actor any more.
    let wait = ClientMessage::Command {
        seq: 8,
        command: Command::Wait {},
    };
    assert_eq!(
        to(&send(&mut s, back, wait), back),
        [ServerMessage::Rejected {
            n: 0,
            seq: 8,
            code: RejectCode::NotSeated
        }]
    );
    // Leaving frees the seat for its token.
    let out = send(&mut s, again, ClientMessage::Leave {});
    assert_eq!(
        out,
        [Outbound {
            to: again,
            frame: r#"{"Goodbye":{"reason":"Left"}}"#.to_string(),
            close: true
        }]
    );
    let out = join(&mut s, ConnId(5), other_seat, Some(other_token));
    assert!(matches!(
        to(&out, ConnId(5))[0],
        ServerMessage::Joined { .. }
    ));
}

#[test]
fn a_seat_retaken_on_its_turn_is_told_to_act() {
    let r = ruleset();
    let mut s = host(&r, 5, players());
    let out = started(&mut s);
    let (actor, _) = turn(&out).unwrap();
    let seat = if actor == A {
        Seat::Attacker
    } else {
        Seat::Defender
    };
    let token = TOKENS[usize::from(actor == B)];
    s.disconnected(actor);
    let out = join(&mut s, ConnId(9), seat, Some(token));
    assert!(matches!(
        to(&out, ConnId(9))[..],
        [
            ServerMessage::Joined { .. },
            ServerMessage::Snapshot { .. },
            ServerMessage::YourTurn { .. }
        ]
    ));
}

#[test]
fn a_bot_plays_its_seat_and_the_journal_verifies() {
    let r = ruleset();
    for bot in [random_bot(3), SeatKind::Bot(Box::new(WrongBot))] {
        let mut s = host(&r, 8, [players().into_iter().next().unwrap(), bot]);
        let out = join(&mut s, A, Seat::Attacker, None);
        assert!(s.started(), "the bot's seat is held");
        let all = play(&mut s, out, &mut Rng::from_seed(4), 5000);
        assert!(s.result().is_some());
        let commands = &s.journal().commands;
        assert!(
            commands
                .iter()
                .any(|(side, _)| *side == BattleSide::Defender)
        );
        let accepted = to(&all, A)
            .iter()
            .filter(|m| matches!(m, ServerMessage::Accepted { .. }))
            .count();
        let attacker = commands
            .iter()
            .filter(|(side, _)| *side == BattleSide::Attacker)
            .count();
        assert_eq!(
            accepted, attacker,
            "the bot's commands are told as events only"
        );
        assert_eq!(replay(&r, &s).verify(&r), Ok(()));
    }
}

#[test]
fn a_game_of_two_bots_never_starts() {
    let r = ruleset();
    let mut s = host(&r, 8, [random_bot(1), random_bot(2)]);
    let out = join(&mut s, A, Seat::Attacker, None);
    assert_eq!(
        to(&out, A),
        [ServerMessage::Refused {
            reason: Refusal::SeatTaken
        }]
    );
    assert!(!s.started());
}

#[test]
fn the_record_is_the_viewers_numbered_messages() {
    let r = ruleset();
    let mut s = host(&r, 5, players());
    let first = started(&mut s);
    let all = play(&mut s, first, &mut Rng::from_seed(2), 30);
    for (conn, seat) in [(A, Seat::Attacker), (B, Seat::Defender)] {
        let fetched = send(&mut s, conn, ClientMessage::FetchRecord { seq: 3 });
        let [ServerMessage::Record { n, seq: 3, record }] = &to(&fetched, conn)[..] else {
            panic!()
        };
        let mut lines = record.lines();
        let header: RecordHeader = serde_json::from_str(lines.next().unwrap()).unwrap();
        assert_eq!(header.format, RECORD_FORMAT_VERSION);
        assert_eq!((header.game, header.seat), (1, seat));
        assert_eq!(header.manifest, map::manifest(&r));
        assert_eq!(header.first, 1, "every message, within the bound");
        let numbered: Vec<String> = frames(&all, conn)
            .into_iter()
            .filter(|f| number(&decode(f)).is_some())
            .collect();
        assert_eq!(lines.map(str::to_string).collect::<Vec<_>>(), numbered);
        assert_eq!(*n, u32::try_from(numbered.len()).unwrap());
        // The answer is not recorded itself.
        let again = send(&mut s, conn, ClientMessage::FetchRecord { seq: 4 });
        assert!(
            matches!(&to(&again, conn)[..], [ServerMessage::Record { record: same, .. }] if same == record)
        );
    }
}

#[test]
fn the_replay_reaches_the_seats_only_after_the_game() {
    let r = ruleset();
    let mut s = host(&r, 5, players());
    let out = started(&mut s);
    s.publish_replay("early".to_string());
    let fetch = send(&mut s, A, ClientMessage::FetchReplay { seq: 6 });
    assert!(matches!(
        to(&fetch, A)[..],
        [ServerMessage::Rejected {
            seq: 6,
            code: RejectCode::NotOver,
            ..
        }]
    ));
    play(&mut s, out, &mut Rng::from_seed(1), 5000);
    assert!(s.result().is_some());
    s.publish_replay("[header]".to_string());
    let fetch = send(&mut s, B, ClientMessage::FetchReplay { seq: 7 });
    assert!(matches!(
        &to(&fetch, B)[..],
        [ServerMessage::Replay { seq: 7, replay, .. }] if replay == "[header]"
    ));
}

#[test]
fn queries_are_answered_by_the_servers_rules() {
    let r = ruleset();
    let mut s = host(&r, 5, players());
    started(&mut s);
    let spell = r.spell_by_key("synthetic:spark").unwrap().raw();
    let query = Query::CastTargets { spell };
    let out = send(&mut s, A, ClientMessage::Query { seq: 2, query });
    let [
        ServerMessage::QueryResult {
            seq: 2, targets, ..
        },
    ] = &to(&out, A)[..]
    else {
        panic!()
    };
    let (_, setup) = full();
    let state = BattleState::new(&r, &setup, 5).unwrap();
    assert_eq!(
        *targets,
        map::query(&state, &r, BattleSide::Attacker, query)
    );
}

#[test]
fn the_manifest_names_what_the_replay_header_names() {
    let r = ruleset();
    let m = map::manifest(&r);
    let h = h3_sim::ReplayHeader::current(&r);
    assert_eq!(m.protocol, crate::PROTOCOL_VERSION);
    assert_eq!(m.engine, h.engine_version);
    let packs: Vec<(String, String)> = m
        .packs
        .iter()
        .map(|p| (p.id.clone(), p.version.clone()))
        .collect();
    assert_eq!(packs, h.packs);
    assert_eq!((m.profile, m.ruleset_hash.0), (h.profile, h.ruleset_hash));
    assert_eq!(
        (m.rng.name, m.rng.version),
        (h.rng_algorithm, h.rng_version)
    );
    assert_eq!(
        (m.hash.name, m.hash.version),
        (h.hash_algorithm, h.hash_version)
    );
    assert_eq!(m.battle_state_hash_version, h.battle_state_hash_version);
    assert_eq!(m.field, map::field(&r));
    assert_eq!(m.keys, map::key_tables(&r));
    // An offer names what the manifest names, before anyone joins.
    let offer = map::offer("pvp", "full", &r, vec![Seat::Defender]);
    let m = map::manifest(&r);
    assert_eq!(
        (offer.id.as_str(), offer.scenario.as_str()),
        ("pvp", "full")
    );
    assert_eq!(
        (offer.packs, offer.profile, offer.ruleset_hash),
        (m.packs, m.profile, m.ruleset_hash)
    );
    assert_eq!(offer.bots, [Seat::Defender]);
}

#[test]
fn the_lobby_hears_where_the_game_is() {
    let r = ruleset();
    let mut s = host(&r, 5, players());
    assert_eq!(s.phase(), Phase::Waiting);
    assert_eq!(s.open_seats(), [Seat::Attacker, Seat::Defender]);
    // The lobby's messages are not the game's.
    for message in [
        ClientMessage::ListGames { seq: 1 },
        ClientMessage::CreateGame {
            seq: 2,
            offer: "pvp".to_string(),
            seat: Seat::Attacker,
            spectators: None,
        },
        ClientMessage::ListReplays { seq: 3, from: 0 },
        ClientMessage::FetchArchived {
            seq: 4,
            replay: "pvp-game1-0".to_string(),
        },
    ] {
        assert!(send(&mut s, A, message).is_empty());
    }
    join(&mut s, A, Seat::Attacker, None);
    assert_eq!(s.open_seats(), [Seat::Defender]);
    // A seat whose holder left needs its token: it is not open again.
    s.disconnected(A);
    assert_eq!(s.open_seats(), [Seat::Defender]);
    let out = join(&mut s, A, Seat::Attacker, Some(TOKENS[0]));
    let out = [out, join(&mut s, B, Seat::Defender, None)].concat();
    assert_eq!((s.phase(), s.open_seats()), (Phase::Running, vec![]));
    play(&mut s, out, &mut Rng::from_seed(3), 400);
    assert_eq!(s.phase(), Phase::Over);
    // A bot's seat is never open.
    let [_, defender] = players();
    let bot = host(&r, 5, [random_bot(1), defender]);
    assert_eq!(bot.open_seats(), [Seat::Defender]);
}

#[test]
fn journals_without_checkpoints_verify() {
    let r = ruleset();
    let (_, setup) = full();
    let config = GameConfig {
        game: 1,
        setup,
        seed: 3,
        checkpoint_every: 0,
        shadow: false,
        spectators: 2,
        watchable: true,
        record_bytes: 1 << 20,
    };
    let mut s = Session::new(&r, config, players()).unwrap();
    let out = started(&mut s);
    play(&mut s, out, &mut Rng::from_seed(3), 40);
    assert!(s.journal().checkpoints.is_empty());
    assert_eq!(replay(&r, &s).verify(&r), Ok(()));
}

#[test]
fn a_setup_the_engine_refuses_hosts_no_game() {
    let r = ruleset();
    let (_, mut setup) = full();
    setup.attacker.stacks.clear();
    let config = GameConfig {
        game: 1,
        setup,
        seed: 3,
        checkpoint_every: 0,
        shadow: false,
        spectators: 2,
        watchable: true,
        record_bytes: 1 << 20,
    };
    assert!(Session::new(&r, config.clone(), players()).is_err());
    assert!(matches!(
        Session::restore(&r, config, players(), &[]),
        Err(RestoreError::Setup(_))
    ));
}

/// Two players took their seats and sent `commands` commands; every entry, drained as it came.
fn journaled(r: &Ruleset, commands: u32) -> (Session<'_>, Vec<Entry>) {
    let mut s = host(r, 5, players());
    let first = join(&mut s, A, Seat::Attacker, None);
    let mut entries = drained(&mut s);
    assert_eq!(entries, [Entry::Taken(BattleSide::Attacker)]);
    let mut out = first;
    out.extend(join(&mut s, B, Seat::Defender, None));
    assert_eq!(
        drained(&mut s),
        [
            Entry::Taken(BattleSide::Defender),
            Entry::Spectators(true),
            Entry::Started,
        ]
    );
    entries.extend([
        Entry::Taken(BattleSide::Defender),
        Entry::Spectators(true),
        Entry::Started,
    ]);
    play(&mut s, out, &mut Rng::from_seed(9), commands);
    entries.extend(drained(&mut s));
    (s, entries)
}

#[test]
fn the_entries_are_the_journal_and_restore_the_game() {
    let r = ruleset();
    let (mut live, entries) = journaled(&r, 25);
    // The commands in order, each tenth followed by the state's hash.
    let journal = live.journal().clone();
    let mut expected = vec![
        Entry::Taken(BattleSide::Attacker),
        Entry::Taken(BattleSide::Defender),
        Entry::Spectators(true),
        Entry::Started,
    ];
    let mut checkpoints = journal.checkpoints.iter();
    for (i, &(side, command)) in journal.commands.iter().enumerate() {
        expected.push(Entry::Command(side, command));
        if (i + 1) % 10 == 0 {
            expected.push(Entry::Checkpoint(*checkpoints.next().unwrap()));
        }
    }
    assert_eq!(entries, expected);
    assert_eq!((journal.commands.len(), journal.checkpoints.len()), (25, 2));
    // The restored game is the live one, its seats taken and empty.
    let mut restored = Session::restore(&r, config(5), players(), &entries).unwrap();
    assert_eq!(drained(&mut restored), [], "the journal lacks nothing");
    assert_eq!(restored.state_hash(), live.state_hash());
    assert_eq!(restored.journal(), &journal);
    assert_eq!(restored.phase(), Phase::Running);
    assert_eq!(restored.open_seats(), []);
    let command = ClientMessage::Command {
        seq: 7,
        command: Command::Wait {},
    };
    assert_eq!(
        to(&send(&mut restored, A, command), A),
        [ServerMessage::Rejected {
            n: 0,
            seq: 7,
            code: RejectCode::NotSeated
        }],
        "no seat is held"
    );
    assert_eq!(
        to(&join(&mut restored, A, Seat::Attacker, None), A),
        [ServerMessage::Refused {
            reason: Refusal::SeatTaken
        }]
    );
    // Re-taken with the tokens: what the live game gives, but numbered afresh.
    let unnumbered = |messages: Vec<ServerMessage>| -> Vec<ServerMessage> {
        let mut messages = messages;
        for m in &mut messages {
            if let ServerMessage::Snapshot { n, .. } | ServerMessage::YourTurn { n, .. } = m {
                *n = 0;
            }
        }
        messages
    };
    for (seat, token) in [(Seat::Attacker, TOKENS[0]), (Seat::Defender, TOKENS[1])] {
        let back = to(&join(&mut restored, A, seat, Some(token)), A);
        let again = to(&join(&mut live, ConnId(9), seat, Some(token)), ConnId(9));
        assert!(matches!(back[1], ServerMessage::Snapshot { n: 1, .. }));
        assert_eq!(unnumbered(back), unnumbered(again));
        send(&mut restored, A, ClientMessage::Leave {});
        live.disconnected(ConnId(9));
    }
    assert_eq!(drained(&mut restored), [], "a re-take journals nothing");
}

#[test]
fn a_journal_that_is_not_its_rerun_restores_nothing() {
    let r = ruleset();
    let (_, entries) = journaled(&r, 25);
    let restore = |entries: &[Entry]| Session::restore(&r, config(5), players(), entries).err();
    let checkpoint = entries
        .iter()
        .position(|e| matches!(e, Entry::Checkpoint(_)))
        .unwrap();
    // A flipped hash.
    let mut flipped = entries.clone();
    flipped[checkpoint] = Entry::Checkpoint(1);
    assert_eq!(
        restore(&flipped),
        Some(RestoreError::Diverged { at: checkpoint })
    );
    // A checkpoint out of place, or one where none is due.
    let mut early = entries.clone();
    early.swap(checkpoint - 1, checkpoint);
    assert_eq!(
        restore(&early),
        Some(RestoreError::Diverged { at: checkpoint - 1 })
    );
    let mut extra = entries.clone();
    extra.push(Entry::Checkpoint(0));
    assert_eq!(
        restore(&extra),
        Some(RestoreError::Diverged { at: entries.len() })
    );
    // A command the engine refuses.
    let mut wrong = entries.clone();
    wrong[3] = Entry::Command(
        BattleSide::Defender,
        h3_battle::Command::Heal {
            target: h3_battle::StackId(200),
        },
    );
    assert!(matches!(
        restore(&wrong),
        Some(RestoreError::Rejected { at: 3, .. })
    ));
}

#[test]
fn a_journal_cut_short_is_completed() {
    let r = ruleset();
    // Cut between a command and its checkpoint: the checkpoint is the first entry drained.
    let (_, entries) = journaled(&r, 10);
    let (last, cut) = entries.split_last().unwrap();
    assert!(matches!(last, Entry::Checkpoint(_)));
    let mut restored = Session::restore(&r, config(5), players(), cut).unwrap();
    assert_eq!(drained(&mut restored), std::slice::from_ref(last));
    // Cut before the bot's answer: the restore lets the bot act.
    let bot = || [players().into_iter().next().unwrap(), random_bot(3)];
    let mut s = host(&r, 8, bot());
    let out = join(&mut s, A, Seat::Attacker, None);
    play(&mut s, out, &mut Rng::from_seed(4), 12);
    let entries = drained(&mut s);
    let answer = entries
        .iter()
        .rposition(|e| matches!(e, Entry::Command(BattleSide::Defender, _)))
        .unwrap();
    let mut restored = Session::restore(&r, config(8), bot(), &entries[..answer]).unwrap();
    assert!(matches!(
        drained(&mut restored).first(),
        Some(Entry::Command(BattleSide::Defender, _))
    ));
    let back = to(&join(&mut restored, A, Seat::Attacker, Some(TOKENS[0])), A);
    assert!(
        restored.result().is_some() || matches!(back.last(), Some(ServerMessage::YourTurn { .. })),
        "the player's turn, or the end: {back:?}"
    );
}

#[test]
fn a_waiting_game_restores_its_taken_seats() {
    let r = ruleset();
    let taken = [Entry::Taken(BattleSide::Attacker)];
    let mut s = Session::restore(&r, config(5), players(), &taken).unwrap();
    assert_eq!(
        (s.phase(), s.open_seats()),
        (Phase::Waiting, vec![Seat::Defender])
    );
    assert!(matches!(
        to(&join(&mut s, A, Seat::Attacker, None), A)[..],
        [ServerMessage::Refused {
            reason: Refusal::SeatTaken
        }]
    ));
    join(&mut s, A, Seat::Attacker, Some(TOKENS[0]));
    join(&mut s, B, Seat::Defender, None);
    assert!(s.started());
    assert_eq!(
        drained(&mut s),
        [
            Entry::Taken(BattleSide::Defender),
            Entry::Spectators(true),
            Entry::Started,
        ]
    );
}

/// The token `conn`'s `Joined` in `out` carries, or why its join was refused.
fn joined_token(out: &[Outbound], conn: ConnId) -> Result<Option<String>, Refusal> {
    match to(out, conn).into_iter().next() {
        Some(ServerMessage::Joined { token, .. }) => Ok(token),
        Some(ServerMessage::Refused { reason }) => Err(reason),
        other => panic!("{other:?}"),
    }
}

#[test]
fn a_logged_in_connections_seat_belongs_to_its_account() {
    let r = ruleset();
    let (c, d, e, f) = (ConnId(3), ConnId(4), ConnId(5), ConnId(6));
    let mut s = host(&r, 5, players());
    // The attacker's seat is taken logged in: it is the account's, and no token is handed out.
    s.logged_in(A, "alice".to_string());
    assert_eq!(
        joined_token(&join(&mut s, A, Seat::Attacker, None), A),
        Ok(None)
    );
    assert_eq!(
        joined_token(&join(&mut s, B, Seat::Defender, None), B),
        Ok(Some(TOKENS[1].to_string())),
        "an anonymous seat keeps its token"
    );
    assert!(s.started());
    assert_eq!(s.accounts(), [Some("alice"), None]);
    let mut entries = drained(&mut s);
    assert_eq!(
        entries[..2],
        [
            Entry::TakenBy(BattleSide::Attacker, "alice".to_string()),
            Entry::Taken(BattleSide::Defender),
        ]
    );
    // No token takes the account's seat, nor another account.
    s.disconnected(A);
    let refused = |s: &mut Session<'_>, conn, seat, token| {
        joined_token(&join(s, conn, seat, token), conn).unwrap_err()
    };
    assert_eq!(
        refused(&mut s, c, Seat::Attacker, Some(TOKENS[0])),
        Refusal::SeatTaken
    );
    assert_eq!(refused(&mut s, c, Seat::Attacker, None), Refusal::SeatTaken);
    s.logged_in(d, "bob".to_string());
    assert_eq!(refused(&mut s, d, Seat::Attacker, None), Refusal::SeatTaken);
    // Any connection logged in as the account re-takes it, replacing one that holds it.
    s.logged_in(e, "alice".to_string());
    assert_eq!(
        joined_token(&join(&mut s, e, Seat::Attacker, None), e),
        Ok(None)
    );
    s.logged_in(f, "alice".to_string());
    let out = join(&mut s, f, Seat::Attacker, Some("wrong"));
    assert_eq!(joined_token(&out, f), Ok(None), "its token does not matter");
    assert!(matches!(
        to(&out, e)[..],
        [ServerMessage::Goodbye {
            reason: GoodbyeReason::Replaced
        }]
    ));
    // The anonymous seat stays the token's, also for a logged-in connection.
    assert_eq!(refused(&mut s, d, Seat::Defender, None), Refusal::SeatTaken);
    assert_eq!(
        joined_token(&join(&mut s, d, Seat::Defender, Some(TOKENS[1])), d),
        Ok(Some(TOKENS[1].to_string()))
    );
    assert_eq!(s.accounts(), [Some("alice"), None]);
    // A connection's login ends with it.
    s.disconnected(f);
    assert_eq!(refused(&mut s, f, Seat::Attacker, None), Refusal::SeatTaken);
    // A restore binds the seats again.
    entries.extend(drained(&mut s));
    let mut restored = Session::restore(&r, config(5), players(), &entries).unwrap();
    assert_eq!(restored.accounts(), [Some("alice"), None]);
    restored.logged_in(A, "alice".to_string());
    assert_eq!(
        joined_token(&join(&mut restored, A, Seat::Attacker, None), A),
        Ok(None)
    );
    assert_eq!(
        refused(&mut restored, B, Seat::Attacker, Some(TOKENS[0])),
        Refusal::SeatTaken
    );
}

#[test]
fn an_accounts_seat_is_its_accounts_alone() {
    use crate::session::refusal;
    let (token, alice) = (TOKENS[0], Some("alice"));
    // A seat that belongs to an account: its account, whatever the token.
    assert_eq!(refusal((token, true, alice), None, alice), None);
    assert_eq!(refusal((token, true, alice), Some("x"), alice), None);
    for login in [None, Some("bob")] {
        assert_eq!(
            refusal((token, true, alice), Some(token), login),
            Some(Refusal::SeatTaken)
        );
    }
    // Another: without a token only the first time, else with its token.
    assert_eq!(refusal((token, false, None), None, alice), None);
    assert_eq!(
        refusal((token, true, None), None, alice),
        Some(Refusal::SeatTaken)
    );
    assert_eq!(refusal((token, true, None), Some(token), None), None);
    assert_eq!(
        refusal((token, false, None), Some("x"), None),
        Some(Refusal::BadToken)
    );
}

#[test]
fn the_host_pauses_and_a_seat_resumes_with_its_token() {
    let r = ruleset();
    let (mut s, mut entries) = journaled(&r, 12);
    // The other seat's ask is told to every viewer; the game goes on.
    let asked = send(&mut s, B, ClientMessage::Pause { seq: 3 });
    for conn in [A, B] {
        assert!(matches!(
            to(&asked, conn)[..],
            [ServerMessage::PauseAsked {
                seat: Seat::Defender,
                ..
            }]
        ));
    }
    assert_eq!((drained(&mut s), s.paused()), (vec![], false));
    // The host's (the attacker took its seat first) pauses it; each holder hears it.
    s.disconnected(B);
    let paused = send(&mut s, A, ClientMessage::Pause { seq: 4 });
    assert_eq!(
        paused,
        [Outbound {
            to: A,
            frame: codec::encode_text(&ServerMessage::Goodbye {
                reason: GoodbyeReason::Paused
            }),
            close: true,
        }]
    );
    assert_eq!((s.paused(), s.phase()), (true, Phase::Paused));
    let pause = drained(&mut s);
    assert_eq!(pause, [Entry::Paused]);
    entries.extend(pause);
    // The journal restores the paused game; a join with a seat's token resumes it.
    let mut restored = Session::restore(&r, config(5), players(), &entries).unwrap();
    assert_eq!(
        (restored.paused(), restored.phase(), restored.state_hash()),
        (true, Phase::Paused, s.state_hash())
    );
    assert!(matches!(
        to(&join(&mut restored, B, Seat::Defender, None), B)[..],
        [ServerMessage::Refused {
            reason: Refusal::SeatTaken
        }]
    ));
    assert!(restored.paused(), "a refused join resumes nothing");
    let back = to(&join(&mut restored, B, Seat::Defender, Some(TOKENS[1])), B);
    assert!(matches!(
        back[..2],
        [ServerMessage::Joined { .. }, ServerMessage::Snapshot { .. }]
    ));
    assert_eq!(
        (restored.paused(), restored.phase()),
        (false, Phase::Running)
    );
    let resumed = drained(&mut restored);
    assert_eq!(resumed, [Entry::Resumed]);
    entries.extend(resumed);
    let again = Session::restore(&r, config(5), players(), &entries).unwrap();
    assert_eq!((again.paused(), again.phase()), (false, Phase::Running));
}

#[test]
fn a_pause_is_refused_outside_a_running_game() {
    let r = ruleset();
    let mut s = host(&r, 5, players());
    let pause =
        |s: &mut Session<'_>, conn, seq| to(&send(s, conn, ClientMessage::Pause { seq }), conn);
    assert_eq!(
        pause(&mut s, A, 1),
        [ServerMessage::Rejected {
            n: 0,
            seq: 1,
            code: RejectCode::NotSeated
        }]
    );
    join(&mut s, A, Seat::Attacker, None);
    assert!(matches!(
        pause(&mut s, A, 2)[..],
        [ServerMessage::Rejected {
            seq: 2,
            code: RejectCode::NotStarted,
            ..
        }]
    ));
    let out = join(&mut s, B, Seat::Defender, None);
    play(&mut s, out, &mut Rng::from_seed(3), 400);
    assert!(s.result().is_some());
    assert!(matches!(
        pause(&mut s, A, 3)[..],
        [ServerMessage::Rejected {
            seq: 3,
            code: RejectCode::BattleOver,
            ..
        }]
    ));
    assert!(!drained(&mut s).contains(&Entry::Paused));
    // A game that did not verify withholds its replay.
    s.withhold_replay();
    assert!(matches!(
        to(&send(&mut s, B, ClientMessage::FetchReplay { seq: 4 }), B)[..],
        [ServerMessage::Rejected {
            seq: 4,
            code: RejectCode::Unverified,
            ..
        }]
    ));
}

#[test]
fn a_spectator_watches_what_is_public_and_never_acts() {
    let r = ruleset();
    let mut s = host(&r, 5, players());
    let out = started(&mut s);
    let out = play(&mut s, out, &mut Rng::from_seed(9), 12);
    // A spectator who comes late gets the spectator's snapshot, no token.
    let c = ConnId(5);
    match &to(&join(&mut s, c, Seat::Spectator, None), c)[..] {
        [
            ServerMessage::Joined {
                seat: Seat::Spectator,
                token: None,
                ..
            },
            ServerMessage::Snapshot { n: 1, observation },
        ] => assert!(observation.spectator && observation.own_hero.is_none()),
        other => panic!("{other:?}"),
    }
    // The events, never a turn.
    let (actor, command) = staying(&out);
    let next = send(&mut s, actor, ClientMessage::Command { seq: 90, command });
    let watched = to(&next, c);
    assert!(matches!(watched[0], ServerMessage::Events { n: 2, .. }));
    assert!(
        watched
            .iter()
            .all(|m| matches!(m, ServerMessage::Events { .. }))
    );
    // What it sends for a seat is rejected, numbered: it is a viewer.
    let asks = [
        ClientMessage::Command {
            seq: 1,
            command: Command::Wait {},
        },
        ClientMessage::Query {
            seq: 2,
            query: Query::CastTargets { spell: 0 },
        },
        ClientMessage::FetchReplay { seq: 3 },
        ClientMessage::Pause { seq: 4 },
    ];
    let mut n = u32::try_from(watched.len()).unwrap() + 1;
    for (message, seq) in asks.into_iter().zip(1..) {
        n += 1;
        let code = RejectCode::NotSeated;
        assert_eq!(
            to(&send(&mut s, c, message), c),
            [ServerMessage::Rejected { n, seq, code }]
        );
    }
    let fetched = to(&send(&mut s, c, ClientMessage::FetchRecord { seq: 5 }), c);
    let [
        ServerMessage::Record {
            n: last, record, ..
        },
    ] = &fetched[..]
    else {
        panic!("{fetched:?}")
    };
    let header: RecordHeader = serde_json::from_str(record.lines().next().unwrap()).unwrap();
    assert_eq!((header.seat, *last), (Seat::Spectator, n));
    assert_eq!(record.lines().count(), n as usize + 1);
    // Another seat's ask to pause reaches it; up to the limit of two, a place is freed by a
    // leave; the host's pause closes it.
    let asked = send(&mut s, B, ClientMessage::Pause { seq: 6 });
    assert!(matches!(
        to(&asked, c)[..],
        [ServerMessage::PauseAsked {
            seat: Seat::Defender,
            ..
        }]
    ));
    let (d, e) = (ConnId(6), ConnId(7));
    join(&mut s, d, Seat::Spectator, None);
    let full = to(&join(&mut s, e, Seat::Spectator, None), e);
    assert!(matches!(
        full[..],
        [ServerMessage::Refused {
            reason: Refusal::NoSpectators
        }]
    ));
    send(&mut s, c, ClientMessage::Leave {});
    join(&mut s, e, Seat::Spectator, None);
    let paused = send(&mut s, A, ClientMessage::Pause { seq: 7 });
    for conn in [d, e] {
        assert_eq!(
            to(&paused, conn),
            [ServerMessage::Goodbye {
                reason: GoodbyeReason::Paused
            }]
        );
    }
    assert_eq!(
        drained(&mut s).last(),
        Some(&Entry::Paused),
        "nothing of them"
    );
    // A spectator sees the end.
    let mut s = host(&r, 5, players());
    let mut out = join(&mut s, c, Seat::Spectator, None);
    out.extend(started(&mut s));
    let out = play(&mut s, out, &mut Rng::from_seed(3), 400);
    assert!(matches!(
        to(&out, c).last(),
        Some(ServerMessage::GameEnded { .. })
    ));
}

#[test]
fn the_host_opens_and_closes_its_game_to_spectators_until_it_starts() {
    let r = ruleset();
    let set = |seq, open| ClientMessage::SetSpectators { seq, open };
    let (c, d) = (ConnId(5), ConnId(6));
    let mut s = host(&r, 5, players());
    let mut entries = drained(&mut s);
    join(&mut s, A, Seat::Attacker, None);
    join(&mut s, c, Seat::Spectator, None);
    assert!(s.takes_spectators() && s.watchable());
    // Closing ends the spectators' watch and refuses the next ones; it is an entry.
    let closed = send(&mut s, A, set(1, false));
    assert_eq!(
        to(&closed, c),
        [ServerMessage::Goodbye {
            reason: GoodbyeReason::SpectatorsClosed
        }]
    );
    assert!(matches!(
        to(&closed, A)[..],
        [ServerMessage::SpectatorsSet {
            seq: 1,
            open: false,
            ..
        }]
    ));
    entries.extend(drained(&mut s));
    assert_eq!(
        entries,
        [Entry::Taken(BattleSide::Attacker)],
        "a change before the start is no entry"
    );
    assert!(!s.takes_spectators() && !s.watchable());
    let refused = [ServerMessage::Refused {
        reason: Refusal::NoSpectators,
    }];
    assert_eq!(to(&join(&mut s, d, Seat::Spectator, None), d), refused);
    // The same setting again is answered.
    let again = send(&mut s, A, set(2, false));
    assert!(matches!(
        to(&again, A)[..],
        [ServerMessage::SpectatorsSet {
            seq: 2,
            open: false,
            ..
        }]
    ));
    // Opened again, the game takes them.
    send(&mut s, A, set(3, true));
    assert!(matches!(
        to(&join(&mut s, d, Seat::Spectator, None), d)[0],
        ServerMessage::Joined { .. }
    ));
    // A spectator holds no seat.
    assert!(matches!(
        to(&send(&mut s, d, set(4, false)), d)[..],
        [ServerMessage::Rejected {
            seq: 4,
            code: RejectCode::NotSeated,
            ..
        }]
    ));
    // Closed before the start; then the configuration is fixed.
    send(&mut s, A, set(5, false));
    join(&mut s, B, Seat::Defender, None);
    entries.extend(drained(&mut s));
    assert_eq!(s.phase(), Phase::Running);
    assert_eq!(
        entries,
        [
            Entry::Taken(BattleSide::Attacker),
            Entry::Taken(BattleSide::Defender),
            Entry::Spectators(false),
            Entry::Started,
        ],
        "the setting the game started with is an entry"
    );
    for (conn, seq) in [(A, 6), (B, 7)] {
        assert!(matches!(
            to(&send(&mut s, conn, set(seq, true)), conn)[..],
            [ServerMessage::Rejected {
                code: RejectCode::AlreadyStarted,
                ..
            }]
        ));
    }
    assert!(!s.takes_spectators());
    // A restart restores the setting from the entries.
    let restored = Session::restore(&r, config(5), players(), &entries).unwrap();
    assert!(!restored.takes_spectators());
    // A newcomer in the other seat of a waiting game whose host left is not the host.
    let mut s = host(&r, 5, players());
    join(&mut s, A, Seat::Attacker, None);
    s.disconnected(A);
    join(&mut s, B, Seat::Defender, None);
    assert_eq!(s.phase(), Phase::Waiting);
    assert!(matches!(
        to(&send(&mut s, B, set(8, false)), B)[..],
        [ServerMessage::Rejected {
            seq: 8,
            code: RejectCode::NotHost,
            ..
        }]
    ));
    // A game created closed, one on a server without spectators and a paused one take none.
    let closed = GameConfig {
        watchable: false,
        ..config(5)
    };
    let mut s = Session::new(&r, closed, players()).unwrap();
    assert!(!s.takes_spectators());
    assert_eq!(to(&join(&mut s, c, Seat::Spectator, None), c), refused);
    let none = GameConfig {
        spectators: 0,
        ..config(5)
    };
    assert!(
        !Session::new(&r, none, players())
            .unwrap()
            .takes_spectators()
    );
    let mut s = host(&r, 5, players());
    started(&mut s);
    send(&mut s, A, ClientMessage::Pause { seq: 9 });
    assert!(s.paused() && !s.takes_spectators());
}

#[test]
fn the_host_suspends_a_running_game_whose_seats_are_empty() {
    let r = ruleset();
    let c = ConnId(5);
    let mut s = host(&r, 5, players());
    let mut out = Vec::new();
    // Waiting, then running with its seats held: nothing to suspend.
    join(&mut s, A, Seat::Attacker, None);
    assert!(s.seated() && !s.suspend(&mut out));
    started(&mut s);
    join(&mut s, c, Seat::Spectator, None);
    s.disconnected(A);
    assert!(s.seated() && !s.suspend(&mut out));
    // Its seats empty: suspended as a pause, the spectator told; then it runs no more.
    s.disconnected(B);
    drained(&mut s);
    assert!(!s.seated() && s.suspend(&mut out));
    assert_eq!(
        to(&out, c),
        [ServerMessage::Goodbye {
            reason: GoodbyeReason::Paused
        }]
    );
    assert_eq!(drained(&mut s), [Entry::Paused]);
    assert!(s.paused() && !s.suspend(&mut out));
    // A seat's join with its token resumes it.
    join(&mut s, A, Seat::Attacker, Some(TOKENS[0]));
    assert_eq!(s.phase(), Phase::Running);
}

#[test]
fn a_connection_holds_its_seat_or_its_watch_and_nothing_after_a_refused_join() {
    let r = ruleset();
    let (c, d) = (ConnId(5), ConnId(6));
    let mut s = host(&r, 5, players());
    assert!(!s.holds(A));
    join(&mut s, A, Seat::Attacker, None);
    join(&mut s, c, Seat::Spectator, None);
    assert!(s.holds(A) && s.holds(c));
    // The attacker's seat is taken: the join is refused, and the connection holds nothing.
    let refused = join(&mut s, d, Seat::Attacker, None);
    assert!(matches!(
        to(&refused, d)[..],
        [ServerMessage::Refused {
            reason: Refusal::SeatTaken
        }]
    ));
    assert!(!s.holds(d));
    s.disconnected(c);
    s.disconnected(A);
    assert!(!s.holds(c) && !s.holds(A));
}

/// The record `conn` fetches: its header, its lines and its last number.
fn fetched_record(s: &mut Session<'_>, conn: ConnId) -> (RecordHeader, Vec<String>, u32) {
    let fetched = send(s, conn, ClientMessage::FetchRecord { seq: 99 });
    let [ServerMessage::Record { n, record, .. }] = &to(&fetched, conn)[..] else {
        panic!("{fetched:?}")
    };
    let mut lines = record.lines().map(str::to_string);
    let header = serde_json::from_str(&lines.next().unwrap()).unwrap();
    (header, lines.collect(), *n)
}

/// E16 M2 T5c: a viewer's record keeps its latest messages within its bound in bytes, the oldest
/// gone first, numbered on without a gap; a message larger than the bound is kept alone.
#[test]
fn a_record_keeps_the_latest_messages_within_its_bound() {
    let r = ruleset();
    let bound = 4096;
    let small = GameConfig {
        record_bytes: bound,
        ..config(5)
    };
    let mut s = Session::new(&r, small.clone(), players()).unwrap();
    started(&mut s);
    // Queries without end: every answer is numbered and recorded.
    for seq in 1..200 {
        let query = Query::CastTargets { spell: 0 };
        send(&mut s, A, ClientMessage::Query { seq, query });
    }
    let (header, lines, n) = fetched_record(&mut s, A);
    assert!(header.first > 1, "the oldest went");
    assert_eq!(header.first as usize + lines.len() - 1, n as usize);
    for (k, line) in (header.first..).zip(&lines) {
        assert_eq!(number(&decode(line)), Some(k));
    }
    let held: usize = lines.iter().map(|l| l.len() + 1).sum();
    assert!(held <= bound, "{held}");
    assert!(
        held + lines[0].len() + 1 > bound,
        "no more went than had to"
    );
    // The other seat's record is its own.
    assert_eq!(fetched_record(&mut s, B).0.first, 1);
    // A bound below one message: the newest alone.
    let tiny = GameConfig {
        record_bytes: 1,
        ..small
    };
    let mut s = Session::new(&r, tiny, players()).unwrap();
    started(&mut s);
    let (header, lines, n) = fetched_record(&mut s, A);
    assert_eq!((header.first, lines.len()), (n, 1));
    // A spectator's record has the game's bound too.
    join(&mut s, ConnId(5), Seat::Spectator, None);
    let (header, lines, n) = fetched_record(&mut s, ConnId(5));
    assert_eq!((header.first, lines.len(), n), (1, 1, 1));
}

/// Game 1 of the `full` scenario with `seed` and a shadow instance.
fn shadowed(seed: u64) -> GameConfig {
    GameConfig {
        shadow: true,
        ..config(seed)
    }
}

/// `command` from `conn` to both sessions, which must send the same and be in the same state
/// after it: what they sent.
fn both(
    live: &mut Session<'_>,
    plain: &mut Session<'_>,
    conn: ConnId,
    command: Command,
) -> Vec<Outbound> {
    let message = ClientMessage::Command { seq: 50, command };
    let sent = send(live, conn, message.clone());
    assert_eq!(sent, send(plain, conn, message));
    assert_eq!(live.journal(), plain.journal());
    assert_eq!(live.state_hash(), plain.state_hash());
    sent
}

/// The seat told to act in `out`, with a legal command that stays in the battle.
fn staying(out: &[Outbound]) -> (ConnId, Command) {
    let (conn, legal) = turn(out).unwrap();
    let command = legal
        .into_iter()
        .find(|c| !matches!(c, Command::Retreat {} | Command::Surrender {}))
        .unwrap();
    (conn, command)
}

#[test]
fn the_shadow_instance_rebuilds_a_faulty_battle_from_the_journal() {
    let r = ruleset();
    // With the shadow instance, a game plays as without.
    let mut plain = host(&r, 5, players());
    let out = started(&mut plain);
    play(&mut plain, out, &mut Rng::from_seed(9), 30);
    let mut s = Session::new(&r, shadowed(5), players()).unwrap();
    let out = started(&mut s);
    let out = play(&mut s, out, &mut Rng::from_seed(9), 30);
    assert_eq!(s.journal(), plain.journal());
    // A command from the seat not to act is rejected by both battles alike.
    let (actor, command) = staying(&out);
    let other = if actor == A { B } else { A };
    both(&mut s, &mut plain, other, Command::Wait {});
    assert_eq!(s.drain_mismatches().count(), 0);
    // A fault in the live battle: the next command finds it, both battles are rebuilt from the
    // journal and the command is applied to the truth; the viewers notice nothing.
    let elsewhere = |seed| BattleState::new(&r, &config(5).setup, seed).unwrap();
    s.corrupt(elsewhere(6));
    let sent = both(&mut s, &mut plain, actor, command);
    let step = s.journal().commands.len();
    assert_eq!(s.drain_mismatches().collect::<Vec<_>>(), [step]);
    // A fault in the shadow instance: found and rebuilt the same way.
    let (actor, command) = staying(&sent);
    s.corrupt_shadow(&elsewhere(7));
    both(&mut s, &mut plain, actor, command);
    assert_eq!(s.drain_mismatches().collect::<Vec<_>>(), [step + 1]);
    // The restore re-runs through both battles.
    let mut entries = vec![
        Entry::Taken(BattleSide::Attacker),
        Entry::Taken(BattleSide::Defender),
        Entry::Spectators(true),
        Entry::Started,
    ];
    entries.extend(
        s.journal()
            .commands
            .iter()
            .map(|&(side, c)| Entry::Command(side, c)),
    );
    let restored = Session::restore(
        &r,
        GameConfig {
            checkpoint_every: 0,
            ..shadowed(5)
        },
        players(),
        &entries,
    );
    assert_eq!(restored.unwrap().state_hash(), s.state_hash());
}

/// The hidden obstacles' spell of the defender's hero (a Quicksand) and of the attacker's (a Land
/// Mine), by side.
fn hidden_spell(r: &Ruleset, side: BattleSide) -> u16 {
    let key = match side {
        BattleSide::Attacker => "synthetic:snare",
        BattleSide::Defender => "synthetic:mire",
    };
    r.spell_by_key(key).map(SpellId::raw).unwrap()
}

/// What each viewer of [`hidden_cast`] received.
struct Seen {
    /// The caster's frames.
    caster: Vec<String>,
    /// The other side's, across its drop and return.
    other: Vec<String>,
    /// A spectator's from the start, then a spectator's who joined after the cast.
    spectators: Vec<String>,
    /// The caster's side.
    side: BattleSide,
}

/// A spectator watches, then both players join a game with `seed`; the tactics phase ends; the
/// side to act then places its hero's hidden obstacles; the other side's connection drops and
/// comes back, and another spectator joins. Every frame of each viewer.
fn hidden_cast(r: &Ruleset, seed: u64) -> Seen {
    let mut s = host(r, seed, players());
    let (early, late) = (ConnId(8), ConnId(9));
    let mut all = join(&mut s, early, Seat::Spectator, None);
    all.extend(started(&mut s));
    let (conn, legal) = turn(&all).unwrap();
    if legal.contains(&Command::EndTactics {}) {
        all.extend(send(
            &mut s,
            conn,
            ClientMessage::Command {
                seq: 1,
                command: Command::EndTactics {},
            },
        ));
    }
    let (caster, _) = turn(&all).unwrap();
    let side = if caster == A {
        BattleSide::Attacker
    } else {
        BattleSide::Defender
    };
    let cast = Command::Cast {
        spell: hidden_spell(r, side),
        target: Target::None {},
        second: Target::None {},
    };
    let out = send(
        &mut s,
        caster,
        ClientMessage::Command {
            seq: 2,
            command: cast,
        },
    );
    assert!(
        matches!(to(&out, caster)[0], ServerMessage::Accepted { .. }),
        "{:?}",
        to(&out, caster)
    );
    all.extend(out);
    let viewer = if caster == A { B } else { A };
    let (seat, token) = if viewer == A {
        (Seat::Attacker, TOKENS[0])
    } else {
        (Seat::Defender, TOKENS[1])
    };
    s.disconnected(viewer);
    let back = join(&mut s, ConnId(7), seat, Some(token));
    let mut other = frames(&all, viewer);
    other.extend(frames(&back, ConnId(7)));
    let mut spectators = frames(&all, early);
    spectators.extend(frames(&join(&mut s, late, Seat::Spectator, None), late));
    Seen {
        caster: frames(&all, caster),
        other,
        spectators,
        side,
    }
}

#[test]
fn a_hidden_quicksand_leaves_the_other_viewers_bytes_unchanged() {
    let r = ruleset();
    let (a, b) = (hidden_cast(&r, 11), hidden_cast(&r, 12));
    assert_eq!(
        a.side,
        BattleSide::Defender,
        "the defender places a Quicksand"
    );
    assert_ne!(
        a.caster, b.caster,
        "the pits lie elsewhere, as the caster sees"
    );
    assert_eq!(a.other, b.other);
    assert_eq!(a.spectators, b.spectators, "spectators see neither");
    let snapshot = |frame: &String| match decode(frame) {
        ServerMessage::Snapshot { observation, .. } => Some(observation),
        _ => None,
    };
    let watched: Vec<_> = a.spectators.iter().filter_map(snapshot).collect();
    assert_eq!(watched.len(), 2);
    assert!(watched.iter().all(|o| o.spectator && o.own_hero.is_none()));
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// Observation hygiene over the wire (ADR-0018 point 7): wherever the opponent's hidden
    /// obstacles lie, the other viewer and the spectators receive the same bytes.
    #[test]
    fn hidden_obstacles_never_reach_the_other_viewers_bytes(seeds: [u64; 2]) {
        let r = ruleset();
        let (first, second) = (hidden_cast(&r, seeds[0]), hidden_cast(&r, seeds[1]));
        prop_assert_eq!(first.other, second.other);
        prop_assert_eq!(first.spectators, second.spectators);
    }

    /// The seats and the numbers hold under any messages from any connections (ADR-0018 points
    /// 6, 7 and 9; the M1 review; spectators since E16 M2 T4): a connection gets `Joined` only
    /// while it holds no seat and watches nothing, so it is never two viewers; a spectator gets no
    /// token and never a turn; numbered messages reach only a viewer's connection, and each is
    /// that viewer's record line of that number, the lines numbered without a gap; any other
    /// connection gets only unnumbered replies.
    #[test]
    fn seats_and_numbers_hold_under_any_messages(
        acts in proptest::collection::vec(arb_act(), 0..60),
    ) {
        let r = ruleset();
        let mut s = host(&r, 5, players());
        let tokens = [TOKENS[0], TOKENS[1], "wrong"];
        let mut holders: [Option<ConnId>; 2] = [None, None];
        // A spectator's viewer, numbered from 2 at each join (its record starts afresh).
        let mut watchers: BTreeMap<ConnId, usize> = BTreeMap::new();
        let mut next = 2;
        let mut legal: [Vec<Command>; 2] = [Vec::new(), Vec::new()];
        let mut numbered: Vec<(usize, u32, String)> = Vec::new();
        let mut entries = Vec::new();
        for (seq, (conn, act)) in (1u32..).zip(acts) {
            let conn = ConnId(conn);
            let held = holders.iter().position(|&h| h == Some(conn));
            let message = match act {
                Act::Join(seat, token) => ClientMessage::Join {
                    game: 1,
                    seat,
                    token: token.map(|t| tokens[t].to_string()),
                },
                Act::Leave => ClientMessage::Leave {},
                Act::Disconnect => {
                    s.disconnected(conn);
                    holders = holders.map(|h| h.filter(|&h| h != conn));
                    watchers.remove(&conn);
                    continue;
                }
                Act::Query(spell) => ClientMessage::Query {
                    seq,
                    query: Query::CastTargets { spell },
                },
                Act::FetchRecord => ClientMessage::FetchRecord { seq },
                Act::FetchReplay => ClientMessage::FetchReplay { seq },
                Act::Pause => ClientMessage::Pause { seq },
                Act::Play(k) => match held.map(|i| &legal[i]).filter(|l| !l.is_empty()) {
                    Some(list) => ClientMessage::Command {
                        seq,
                        command: list[k % list.len()],
                    },
                    None => continue,
                },
                Act::Command(command) => ClientMessage::Command { seq, command },
            };
            for o in send(&mut s, conn, message) {
                let m = decode(&o.frame);
                let viewer = holders
                    .iter()
                    .position(|&h| h == Some(o.to))
                    .or_else(|| watchers.get(&o.to).copied());
                match (&m, number(&m)) {
                    (ServerMessage::Joined { seat, token, .. }, _) => {
                        prop_assert_eq!(viewer, None, "Joined only to no viewer");
                        if *seat == Seat::Spectator {
                            prop_assert_eq!(token, &None);
                            watchers.insert(o.to, next);
                            next += 1;
                        } else {
                            holders[usize::from(*seat != Seat::Attacker)] = Some(o.to);
                        }
                    }
                    (ServerMessage::Goodbye { .. }, _) => {
                        holders = holders.map(|h| h.filter(|&h| h != o.to));
                        watchers.remove(&o.to);
                    }
                    (_, Some(0)) => prop_assert_eq!(viewer, None, "number 0 only to no viewer"),
                    (_, Some(n)) => {
                        prop_assert!(viewer.is_some(), "numbered to no viewer: {:?}", m);
                        let i = viewer.unwrap_or_default();
                        if let ServerMessage::YourTurn { legal: list, .. } = &m {
                            prop_assert!(i < 2, "a spectator's turn");
                            legal[i].clone_from(list);
                        }
                        numbered.push((i, n, o.frame.clone()));
                    }
                    (_, None) => {}
                }
            }
            entries.extend(s.drain_entries());
        }
        // The entries restore the game as it is (E16 M2 T2).
        let mut restored = Session::restore(&r, config(5), players(), &entries)
            .map_err(|e| TestCaseError::fail(format!("{e:?}")))?;
        prop_assert_eq!(restored.drain_entries().count(), 0);
        prop_assert_eq!(restored.state_hash(), s.state_hash());
        prop_assert_eq!(restored.journal(), s.journal());
        prop_assert_eq!(restored.phase(), s.phase());
        prop_assert_eq!(restored.open_seats(), s.open_seats());
        // Each viewer's record: the spectators' first, then the seats', re-taken.
        let mut records: Vec<(ConnId, usize, Seat)> = watchers
            .iter()
            .map(|(&conn, &i)| (conn, i, Seat::Spectator))
            .collect();
        for (i, seat) in [Seat::Attacker, Seat::Defender].into_iter().enumerate() {
            let conn = ConnId(10 + i as u64);
            join(&mut s, conn, seat, Some(TOKENS[i]));
            records.push((conn, i, seat));
        }
        for (conn, i, seat) in records {
            let fetched = to(&send(&mut s, conn, ClientMessage::FetchRecord { seq: 0 }), conn);
            let Some(ServerMessage::Record { record, n, .. }) = fetched.first() else {
                return Err(TestCaseError::fail(format!("{fetched:?}")));
            };
            let mut lines = record.lines();
            let header: RecordHeader = serde_json::from_str(lines.next().unwrap_or_default())
                .map_err(|e| TestCaseError::fail(format!("{e}")))?;
            prop_assert_eq!(header.seat, seat);
            prop_assert_eq!(header.first, 1);
            let lines: Vec<&str> = lines.collect();
            prop_assert_eq!(lines.len(), *n as usize);
            for (k, line) in lines.iter().enumerate() {
                prop_assert_eq!(number(&decode(line)), Some(k as u32 + 1));
            }
            for (_, n, frame) in numbered.iter().filter(|(j, ..)| *j == i) {
                prop_assert_eq!(lines[*n as usize - 1], frame.as_str());
            }
        }
    }

    /// A command from either seat at any point either is accepted and journaled, or is rejected
    /// and changes neither the battle nor the journal (no entry either), and the other seat hears
    /// nothing of it; the seat not to act is always told it is not its turn.
    #[test]
    fn a_rejected_command_changes_nothing(
        steps in 0u32..30,
        play_seed: u64,
        command in arb_command(),
        wrong_seat: bool,
    ) {
        let r = ruleset();
        let mut s = host(&r, 5, players());
        let out = started(&mut s);
        let out = play(&mut s, out, &mut Rng::from_seed(play_seed), steps);
        let Some((actor, _)) = turn(&out) else {
            return Ok(());
        };
        let other = if actor == A { B } else { A };
        let conn = if wrong_seat { other } else { actor };
        let (hash, journal) = (s.state_hash(), s.journal().clone());
        s.drain_entries();
        let out = send(&mut s, conn, ClientMessage::Command { seq: 99, command });
        match to(&out, conn).remove(0) {
            ServerMessage::Accepted { seq: 99, .. } => {
                prop_assert!(!wrong_seat);
                prop_assert_eq!(s.journal().commands.len(), journal.commands.len() + 1);
                prop_assert!(matches!(s.drain_entries().next(), Some(Entry::Command(..))), "journaled");
            }
            ServerMessage::Rejected { seq: 99, code, .. } => {
                if wrong_seat {
                    prop_assert_eq!(code, RejectCode::NotYourTurn);
                }
                prop_assert_eq!(s.state_hash(), hash);
                prop_assert_eq!(s.journal(), &journal);
                prop_assert_eq!(s.drain_entries().count(), 0);
                prop_assert_eq!(out.len(), 1, "only the sender hears of it");
            }
            other => prop_assert!(false, "{:?}", other),
        }
    }
}

/// What a connection does in `seats_and_numbers_hold_under_any_messages`.
#[derive(Debug, Clone)]
enum Act {
    /// `Join` a seat with no token, the attacker's (0), the defender's (1) or a wrong one (2).
    Join(Seat, Option<usize>),
    Leave,
    /// The connection is gone (the host calls `disconnected`).
    Disconnect,
    Query(u16),
    FetchRecord,
    FetchReplay,
    Pause,
    /// The `k`-th legal command of the last turn the connection's seat was given.
    Play(usize),
    Command(Command),
}

/// An act of one of four connections.
fn arb_act() -> impl Strategy<Value = (u64, Act)> {
    let seat = prop_oneof![
        Just(Seat::Attacker),
        Just(Seat::Defender),
        Just(Seat::Spectator)
    ];
    let act = prop_oneof![
        3 => (seat, proptest::option::of(0usize..3)).prop_map(|(seat, t)| Act::Join(seat, t)),
        1 => Just(Act::Leave),
        1 => Just(Act::Disconnect),
        1 => (0u16..60).prop_map(Act::Query),
        1 => Just(Act::FetchRecord),
        1 => Just(Act::FetchReplay),
        1 => Just(Act::Pause),
        6 => any::<usize>().prop_map(Act::Play),
        2 => arb_command().prop_map(Act::Command),
    ];
    (1u64..=4, act)
}

#[test]
fn a_seats_token_reaches_only_its_holder() {
    let r = ruleset();
    let mut s = host(&r, 5, players());
    let out = started(&mut s);
    let out = play(&mut s, out, &mut Rng::from_seed(3), 400);
    assert!(s.result().is_some());
    for (conn, own, other) in [(A, TOKENS[0], TOKENS[1]), (B, TOKENS[1], TOKENS[0])] {
        let mut seen = frames(&out, conn);
        let fetched = send(&mut s, conn, ClientMessage::FetchRecord { seq: 900 });
        seen.extend(fetched.into_iter().map(|o| o.frame));
        assert!(
            seen.iter().all(|f| !f.contains(other)),
            "the other seat's token"
        );
        assert_eq!(
            seen.iter().filter(|f| f.contains(own)).count(),
            1,
            "its own only in Joined"
        );
    }
}

/// Any wire command, with values that are often in range.
fn arb_command() -> impl Strategy<Value = Command> {
    let target = prop_oneof![
        Just(Target::None {}),
        (0u8..24).prop_map(|stack| Target::Stack { stack }),
        (0u8..190).prop_map(|hex| Target::Hex { hex }),
    ];
    prop_oneof![
        (0u8..190).prop_map(|to| Command::Move { to }),
        (0u8..24, 0u8..190).prop_map(|(target, from)| Command::Attack { target, from }),
        (0u8..24, 0u8..190).prop_map(|(target, at)| Command::Shoot { target, at }),
        Just(Command::Wait {}),
        Just(Command::Defend {}),
        (0u8..24).prop_map(|target| Command::Heal { target }),
        Just(Command::EndTactics {}),
        Just(Command::Retreat {}),
        Just(Command::Surrender {}),
        (0u16..60, target.clone(), target.clone()).prop_map(|(spell, target, second)| {
            Command::Cast {
                spell,
                target,
                second,
            }
        }),
        target.prop_map(|target| Command::CreatureCast { target }),
    ]
}

#[test]
fn outcomes_reach_every_viewer() {
    // A surrender or retreat ends the game for both viewers too.
    let r = ruleset();
    let mut s = host(&r, 5, players());
    let out = started(&mut s);
    let (actor, legal) = turn(&out).unwrap();
    let mut out = out;
    if legal.contains(&Command::EndTactics {}) {
        out = send(
            &mut s,
            actor,
            ClientMessage::Command {
                seq: 1,
                command: Command::EndTactics {},
            },
        );
    }
    let (actor, _) = turn(&out).unwrap();
    let out = send(
        &mut s,
        actor,
        ClientMessage::Command {
            seq: 2,
            command: Command::Retreat {},
        },
    );
    let result = s.result().map(map::outcome);
    assert!(
        matches!(result, Some(Outcome::Retreat { .. })),
        "{result:?}"
    );
    for conn in [A, B] {
        assert!(matches!(
            to(&out, conn).last(),
            Some(ServerMessage::GameEnded { result: r, .. }) if Some(*r) == result
        ));
    }
}
