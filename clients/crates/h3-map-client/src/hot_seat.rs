//! The hot seat's session in-process (E24 M5 T3a; the user's cut of T3 at its start: the session
//! headless here, the window's End Turn and the view's switch T3b's): the game the shell began
//! ([`Game`]) hosted by `h3-protocol`'s whole-game session (`GameSession::start`: the host holds
//! every seat at once), a [`Seat`] per player of the map, each human's by its colour's hot-seat
//! name.
//!
//! - **The frames folded:** every frame the session sends a player is read (`ServerMessage`) and
//!   folded into that seat's own view, as a client keeps its `GameObservation`: a
//!   `GameSnapshot` the view, a `GameEvents` folded event by event
//!   (`GameObservation::fold`), a `GameReveal` after them (`GameObservation::reveal`), a
//!   `GameTurn` the commands the seat may give now (none since the last `GameEvents` else: the
//!   session tells every player each accepted command's events, then each that may act its turn),
//!   a `GameClock` the seat's clock. Each seat's messages are numbered from 1 without a gap; a gap
//!   is an error. A battle's own `Snapshot` is numbered but not folded: the battle is its battle
//!   seat's (below), the view's `battle` stays none and its events change nothing but the
//!   battle's end (`GameObservation::fold`); at the end of a human's own battle its view is the
//!   engine's projection again (`BattleEnded` carries no survivors: SRV-STUB-9).
//! - **The turn:** the human whose turn runs and who may act ([`HotSeat::turn`]; in its battle its
//!   legal commands are the battle's); its End Turn ([`HotSeat::end_turn`]) is played as its
//!   command (`GameSession::play`, numbered by the host). **A computer colour's turn ends at
//!   once** (`MCL-STUB-5`, the user at M5 T0, lifted by E31's increment 2): whenever a computer
//!   may act after a command, the host ends its turn for it, until a human's turn runs.
//! - **A human's battle** (E24 M5 T6a, design §5.33: a walk's or an answer's battle against a
//!   monster, guards or a neutral): from the `GameEvents` whose `BattleStarted` opens a battle with
//!   a side of the human's, the host keeps that player's messages; [`HotSeat::battle`] gives a
//!   battle client's seat (`h3_battle_client::game_seat::GameSeat`) fed with them, and each host
//!   pass ([`HotSeat::serve`]) plays what the seat sent and hands it what came since. The unowned
//!   side's turns are the system actor's, chosen by the original's battle AI (all-seeing,
//!   `h3_battle_ai::choose`, as `h3_sim::computer`'s) from a stream of the host's own, after every
//!   command, so that whenever the host returns a human may act (or the game is over). A battle
//!   between two players' sides (another human's or a computer colour's hero, town or garrison
//!   attacked) is refused with an error (E24 M5 T6c: `CLT-STUB-5`, `MCL-STUB-3`).
//! - **The clock:** the turn's duration (`turndur.txt`'s record, [`clock`]) the session's turn
//!   timer; the host reads its clock into the session ([`HotSeat::elapse`], before the session's
//!   [`HotSeat::deadline`]), and a turn whose time ran out is ended by the session's system actor,
//!   its frames folded as a command's. The window's clock drives it from T3b on.
//! - **The records:** each seat's record so far ([`HotSeat::record`], `GameSession::record`: the
//!   whole game one record), which the record player reads and plays (`h3-map-view record`).
//!
//! *Provisional* (T3a, an agent decision): the names, the frames read back from their text, the
//! clock's level-up time and battle allowance (none, the session's default: the session times a
//! battle against a monster on the turn's timer; MCL-Q75 (b), the user at T6a: the original's
//! pause for every battle, built by play's host, `crate::play`, which holds the clock it reads in
//! from a battle's hand-over to its return), an error for a computer that may act but not end its
//! turn. T6a's (agent decisions): the battle seat's own numbers played as the player's (the
//! in-process session only echoes a number: the host's own sequence stays apart, so a record's
//! `GameAccepted` numbers are two sequences), the battle AI's stream (the game's seed's
//! `StreamId::AGENT`, the kernel's stream for bots, `h3_sim::computer`'s system actor's), the
//! messages kept from the battle's start until its return (one battle at a time: a later one's
//! start replaces them), a duel's refusal after the engine opened it (play ends then, as before
//! T6a every battle did).

use std::ffi::OsStr;
use std::path::Path;

use h3_battle_client::game_seat::{GameSeat, ToHost};
use h3_contracts::PlayerId;
use h3_game::GameRuleset;
use h3_game::Viewer;
use h3_kernel::{Rng, StreamId};
use h3_map_view::engine::{self, GameRecord, Live};
use h3_protocol::codec;
use h3_protocol::map::game::manifest;
use h3_protocol::map::game::observe;
use h3_protocol::record::MAX_LINE;
use h3_protocol::session::game::{
    Allowance, Clock, CommandBounds, GameOptions, GameOutbound, GameSession, Spectators,
};
use h3_protocol::wire::game::{
    Decision, GameAnswer, GameClock, GameCommand, GameEvent, GameObservation, GameQuery,
    GameRecordHeader, GameReject, MapIdentity,
};
use h3_protocol::wire::{ServerMessage, WireU64};

use crate::play::Game;

/// The original's turn durations by `turndur.txt`'s record, in minutes, 0 unlimited
/// (`turn_duration_minutes` 0x641AE8, read in E16 M5 T15: `docs/epics/E16-timer-survey.md`).
pub const TURN_MINUTES: [u8; 11] = [1, 2, 4, 6, 8, 10, 15, 20, 25, 30, 0];

/// What the game's seed is mixed with for the client's own picks in its views (E24 M5 T4b): a
/// stream apart from the engine's, the same for every view of the game.
const PICKS: u64 = 0x6D61_702D_7069_636B;

/// The turn duration's record that is unlimited (`turndur.txt`'s last).
pub const UNLIMITED: u8 = 10;

/// The game's number in its records (`GameOptions::game`: the in-process host hosts one).
const GAME: u32 = 0;

/// The most system commands a host's call plays for the unowned battle side before it gives up
/// (E24 M5 T6a, *provisional*): a battle's side ends its turn within far fewer.
const SIDE_STEPS: usize = 4_096;

/// The session's clock of the turn's duration `duration` (`turndur.txt`'s record, 0 to 10): its
/// minutes the turn timer's limit, none for [`UNLIMITED`]; no level-up time, the default battle
/// allowance (*provisional*: the server's model; MCL-Q75 (b), the user at E24 M5 T6a, wants no
/// battle clock, so play's host holds the clock for every battle, `crate::play`, and a battle
/// between two players' sides, the allowance's, is refused until T6c).
///
/// # Errors
/// A record past [`UNLIMITED`].
pub fn clock(duration: u8) -> Result<Clock, String> {
    let minutes = TURN_MINUTES
        .get(usize::from(duration))
        .ok_or_else(|| format!("the turn's duration {duration}: none (0 to {UNLIMITED})"))?;
    Ok(Clock {
        turn_ms: (*minutes > 0).then(|| u64::from(*minutes) * 60_000),
        level_up_ms: None,
        battle: Allowance::DEFAULT,
    })
}

/// A player's seat: who plays it, and what it was told folded into its own view.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Seat {
    /// The player.
    pub player: PlayerId,
    /// Its colour, 0 red to 7 pink.
    pub colour: u8,
    /// Whether a human plays it (else the host ends its turns: `MCL-STUB-5`).
    pub human: bool,
    /// A human's name in the hot seat (its colour's), if the shell gave one.
    pub name: Option<String>,
    /// Its view, every frame it was told folded in; none before its first snapshot. At the end
    /// of a battle of its the engine's projection again (E24 M5 T6a: SRV-STUB-9's survivors).
    pub view: Option<GameObservation>,
    /// The commands it may give now, as its last `GameTurn` listed them; none when it may not act.
    pub legal: Option<Vec<GameCommand>>,
    /// Its clock as last told, if it was.
    pub clock: Option<GameClock>,
    /// The number of its last message.
    n: u32,
}

impl Seat {
    /// `player`'s seat of `colour`, a human's (named so) or a computer's, told nothing yet.
    #[must_use]
    pub fn new(player: PlayerId, colour: u8, human: bool, name: Option<String>) -> Self {
        Self {
            player,
            colour,
            human,
            name,
            view: None,
            legal: None,
            clock: None,
            n: 0,
        }
    }

    /// The frame `message` folded in.
    ///
    /// # Errors
    /// Its number does not follow the last one's.
    fn fold(&mut self, message: ServerMessage) -> Result<(), String> {
        let n = number(&message);
        if let Some(n) = n
            && n != self.n + 1
        {
            return Err(format!(
                "colour {}: message {n} after {}",
                self.colour, self.n
            ));
        }
        self.n = n.unwrap_or(self.n);
        match message {
            ServerMessage::GameSnapshot { observation, .. } => self.view = Some(observation),
            ServerMessage::GameEvents { events, .. } => {
                // Each accepted command's events go to every player, then its turn to each that
                // may act: what it may give is its next `GameTurn`'s.
                self.legal = None;
                let unready = unready(self.colour);
                let view = self.view.as_mut().ok_or(unready)?;
                for event in &events {
                    view.fold(event);
                }
            }
            ServerMessage::GameReveal { reveal, .. } => {
                let unready = unready(self.colour);
                self.view.as_mut().ok_or(unready)?.reveal(&reveal);
            }
            ServerMessage::GameTurn { legal, .. } => self.legal = Some(legal),
            ServerMessage::GameClock { clock, .. } => self.clock = Some(clock),
            // A command's answers change no view; a battle's `Snapshot` is its battle seat's
            // (E24 M5 T6a, `HotSeat::battle`): the adventure view's `battle` stays none.
            _ => {}
        }
        Ok(())
    }
}

impl Seat {
    /// The number of the last message it was told (0 before its snapshot).
    #[must_use]
    pub fn told(&self) -> u32 {
        self.n
    }
}

/// Where a human's view was left as the turn passed from it (E24 M5 T3b): its seat's last
/// message's number then, its view then, and its heroes' facings as the window last drew them (by
/// hero id, the original's order of eight: the view names none, MCL-Q58). At the seat's next turn
/// its view is set up again from there ([`HotSeat::since`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Left {
    /// The seat's last message's number.
    pub n: u32,
    /// Its view.
    pub view: GameObservation,
    /// Its heroes' facings, by hero id.
    pub facings: Vec<(u16, u8)>,
}

/// A frame folded before the seat of `colour` got its first snapshot.
fn unready(colour: u8) -> String {
    format!("colour {colour}: told the game before its snapshot")
}

/// A message's number in the player's sequence, if it has one.
fn number(message: &ServerMessage) -> Option<u32> {
    match message {
        ServerMessage::GameSnapshot { n, .. }
        | ServerMessage::GameEvents { n, .. }
        | ServerMessage::GameReveal { n, .. }
        | ServerMessage::GameTurn { n, .. }
        | ServerMessage::GameAccepted { n, .. }
        | ServerMessage::GameRejected { n, .. }
        | ServerMessage::GameClock { n, .. }
        | ServerMessage::Snapshot { n, .. }
        | ServerMessage::GamePauseAsked { n, .. }
        | ServerMessage::GameSpectators { n, .. } => Some(*n),
        _ => None,
    }
}

/// Whether `message` begins a battle (its `GameEvents` hold a `BattleStarted`).
fn begins(message: &ServerMessage) -> bool {
    matches!(message, ServerMessage::GameEvents { events, .. }
        if events.iter().any(|e| matches!(e, GameEvent::BattleStarted { .. })))
}

/// Whether `message` ends a battle (its `GameEvents` hold a `BattleEnded`).
fn ends(message: &ServerMessage) -> bool {
    matches!(message, ServerMessage::GameEvents { events, .. }
        if events.iter().any(|e| matches!(e, GameEvent::BattleEnded { .. })))
}

/// A human's battle the host holds (E24 M5 T6a): the player's messages from the `GameEvents`
/// whose `BattleStarted` began it, and how many of them the last seat was handed.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Fight {
    player: PlayerId,
    messages: Vec<ServerMessage>,
    handed: usize,
}

/// The game hosted in-process, a seat per player.
pub struct HotSeat<'r> {
    session: GameSession<'r>,
    rules: &'r GameRuleset,
    /// The seed of the client's own picks in its views (E24 M5 T4b, `h3_map_view::picks`).
    picks: u64,
    seats: Vec<Seat>,
    /// The host's last command's number (the in-process host's are only echoed; a battle seat's
    /// commands keep the seat's own, E24 M5 T6a).
    seq: u32,
    /// The battle of a human's the host holds, from its start until its return (E24 M5 T6a).
    fight: Option<Fight>,
    /// The original's battle AI's stream for the unowned battle side (E24 M5 T6a, *provisional*):
    /// the game's seed's `StreamId::AGENT`, apart from the engine's streams and the views' picks.
    sides: Rng,
}

impl std::fmt::Debug for HotSeat<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HotSeat")
            .field("seats", &self.seats)
            .field("seq", &self.seq)
            .field("fight", &self.fight)
            .finish_non_exhaustive()
    }
}

impl<'r> HotSeat<'r> {
    /// `game` started under `rules` (`GameSession::start`: every seat the host's, the whole game
    /// one record, the turn's duration its clock), the map named by its bytes' hash, each human's
    /// seat its colour's name; the computers' turns ended until a human's runs.
    ///
    /// # Errors
    /// The map file does not read; the turn's duration is none; the engine starts no game of the
    /// setup; no player is human; a frame does not read or fold; a computer may not end its turn.
    pub fn start(game: &Game, rules: &'r GameRuleset) -> Result<Self, String> {
        let file = &game.map;
        let failed = |e: &dyn std::fmt::Display| format!("{}: {e}", file.display());
        if !game.setup.players.iter().any(|p| p.human) {
            return Err(failed(&"no player is human"));
        }
        let name = file
            .file_name()
            .and_then(OsStr::to_str)
            .ok_or_else(|| failed(&"not a map file's name"))?;
        let root = file.parent().unwrap_or_else(|| Path::new("."));
        let (_, map) = h3_files::adventure::read_map(root, name).map_err(|e| failed(&e))?;
        let identity = MapIdentity {
            content_hash: WireU64(map.content_hash),
            import_version: map.import_version,
        };
        let options = GameOptions {
            game: GAME,
            checkpoint_every: 0,
            record_bytes: usize::MAX,
            clock: clock(game.duration).map_err(|e| failed(&e))?,
            commands: CommandBounds::NONE,
            spectators: Spectators::NONE,
        };
        let mut seats = Vec::with_capacity(game.setup.players.len());
        for (index, setup) in game.setup.players.iter().enumerate() {
            let player = PlayerId::new(u8::try_from(index).map_err(|e| failed(&e))?);
            let name = game
                .names
                .get(usize::from(setup.colour))
                .cloned()
                .flatten()
                .filter(|_| setup.human);
            seats.push(Seat::new(player, setup.colour, setup.human, name));
        }
        let mut out = Vec::new();
        let session = GameSession::start(
            rules,
            &game.setup,
            game.seed,
            manifest(rules, identity),
            options,
            &mut out,
        )
        .map_err(|e| failed(&format_args!("the game does not start: {e}")))?;
        let mut hot = Self {
            session,
            rules,
            picks: game.seed ^ PICKS,
            seats,
            seq: 0,
            fight: None,
            sides: Rng::for_stream(game.seed, StreamId::AGENT),
        };
        hot.deliver(out)?;
        hot.settle()?;
        Ok(hot)
    }

    /// The seats, by player.
    #[must_use]
    pub fn seats(&self) -> &[Seat] {
        &self.seats
    }

    /// `player`'s seat, if it is a player of the game.
    #[must_use]
    pub fn seat(&self, player: PlayerId) -> Option<&Seat> {
        self.seats.get(player.index())
    }

    /// The human whose turn runs and who may act now; none while no human may (a level-up of
    /// another player's holding the game, E10's end). In its battle (E24 M5 T6a) the human too,
    /// its legal commands the battle's: the host plays the other side at once.
    #[must_use]
    pub fn turn(&self) -> Option<PlayerId> {
        let state = self.session.state();
        self.seats
            .iter()
            .find(|s| s.human && s.legal.is_some() && state.is_acting(s.player))
            .map(|s| s.player)
    }

    /// The human in turn ends its turn (`GameCommand::EndTurn`, its command); the computers'
    /// turns ended after it; the human whose turn runs then.
    ///
    /// # Errors
    /// No human's turn runs; the game rejects the End Turn (its open decision); a frame does not
    /// read or fold; a computer may not end its turn.
    pub fn end_turn(&mut self) -> Result<Option<PlayerId>, String> {
        let player = self.turn().ok_or("no human's turn runs")?;
        self.command(player, GameCommand::EndTurn {})?;
        self.settle()?;
        Ok(self.turn())
    }

    /// The human in turn's `command` (E24 M5 T4a: a walk), as the host vouches for it: the game's
    /// rejection if it refused it, its frames folded either way (a rejection is told to the
    /// player too). A battle it began (E24 M5 T6a) is held for [`HotSeat::battle`], the unowned
    /// side's turns played until the human's side is to act; the computers' turns ended after it.
    ///
    /// # Errors
    /// No human's turn runs; a frame does not read or fold; the command began a battle between
    /// two players' sides (E24 M5 T6c); a computer may not end its turn.
    pub fn play(&mut self, command: GameCommand) -> Result<Option<GameReject>, String> {
        let player = self.turn().ok_or("no human's turn runs")?;
        self.seq += 1;
        let mut out = Vec::new();
        let played = self.session.play(player, self.seq, command, &mut out);
        self.deliver(out)?;
        self.settle()?;
        Ok(played.err())
    }

    /// The human whose battle the host holds (E24 M5 T6a), from the command that began it until
    /// its return ([`HotSeat::returned`]); none without one.
    #[must_use]
    pub fn fighting(&self) -> Option<PlayerId> {
        self.fight.as_ref().map(|f| f.player)
    }

    /// The battle client's seat for the human's battle the host holds (E24 M5 T6a, design §5.33):
    /// made from the player's record's header and fed every message the host kept since the
    /// battle's start, so that it opened (`GameSeat::opened`: the window loads it,
    /// `h3_battle_client::battle::Ground::load_game`); none without a battle held. A host makes
    /// one for each battle it shows; a later call makes another from the start.
    #[must_use]
    pub fn battle(&mut self) -> Option<GameSeat> {
        let header = self.header(self.fighting()?);
        let fight = self.fight.as_mut()?;
        let mut seat = GameSeat::new(&header);
        for message in &fight.messages {
            seat.receive(message);
        }
        fight.handed = fight.messages.len();
        seat.opened().then_some(seat)
    }

    /// A host pass for the battle's `seat` (E24 M5 T6a, as `h3_battle_client`'s test host's): each
    /// command it sent played as its player's, with the seat's own number (the in-process session
    /// only echoes it: the host's numbers stay apart); each query answered from the battle's
    /// state (`h3_protocol::map::query`: the targets the rules allow the player's side); the
    /// unowned side's turns played; the computers' turns ended once the battle's end passed the
    /// turn on; the player's messages since the last pass handed to the seat. The engine's
    /// rejections of the seat's commands (told to the seat too), in order.
    ///
    /// # Errors
    /// No battle held; a frame does not read or fold; the unowned side's turns do not end; a
    /// computer may not end its turn.
    pub fn serve(&mut self, seat: &mut GameSeat) -> Result<Vec<GameReject>, String> {
        let player = self.fighting().ok_or("no human's battle is held")?;
        let mut refused = Vec::new();
        for sent in seat.take_outgoing() {
            match sent {
                ToHost::Command { seq, command } => {
                    let mut out = Vec::new();
                    let played = self.session.play(player, seq, command, &mut out);
                    self.deliver(out)?;
                    refused.extend(played.err());
                }
                ToHost::Query { seq, query } => {
                    let state = self.session.state();
                    let Some(battle) = state.battle(player) else {
                        continue;
                    };
                    let Some(side) = battle.side_of(player) else {
                        continue;
                    };
                    let targets =
                        h3_protocol::map::query(battle.state(), self.rules.battle(), side, query);
                    seat.answer(seq, targets);
                }
            }
        }
        self.settle()?;
        if let Some(fight) = self.fight.as_mut() {
            for message in fight.messages.get(fight.handed..).unwrap_or_default() {
                seat.receive(message);
            }
            fight.handed = fight.messages.len();
        }
        Ok(refused)
    }

    /// The human's battle handed back (E24 M5 T6a): the host keeps its messages no longer; the
    /// human whose turn runs then (the battle's end may have passed the turn on: a player left
    /// without a hero or a town). *Provisional*: a battle still open stays the engine's (its
    /// human's legal commands the battle's), with no seat to play it.
    pub fn returned(&mut self) -> Option<PlayerId> {
        self.fight = None;
        self.turn()
    }

    /// `player`'s record's header (as `GameSession::record` writes it: the game's, the manifest,
    /// every message from 1).
    fn header(&self, player: PlayerId) -> GameRecordHeader {
        GameRecordHeader {
            format: h3_protocol::RECORD_FORMAT_VERSION,
            game: GAME,
            player: player.raw(),
            manifest: self.session.manifest().clone(),
            first: 1,
        }
    }

    /// `player`'s first open decision but a battle as the engine projects it now (E24 M5 T4b,
    /// design §5.27: a visit's question's facts, a chest's gold and experience, a price), none
    /// without one. *Provisional*: the in-process host reads the engine's projection (`observe`),
    /// since the fold of the seat's frames keeps no offer's or choice's view (`SRV-STUB-5`).
    #[must_use]
    pub fn decision(&self, player: PlayerId) -> Option<Decision> {
        observe(self.session.state(), self.rules, Viewer::Player(player)).decision
    }

    /// `player`'s `query` of the running game (`GameSession::query`, E24 M5 T4a: a hero's moves);
    /// none while the game does not answer (not started, paused).
    #[must_use]
    pub fn query(&self, player: PlayerId, query: GameQuery) -> Option<GameAnswer> {
        self.session.query(player, query).ok()
    }

    /// The host's clock reads `now`, in milliseconds (`GameSession::elapse`): a turn whose time
    /// ran out ended by the session, its frames folded; the unowned battle side's turns played
    /// (E24 M5 T6a: a human's side the session played for it) and the computers' turns ended
    /// after it.
    ///
    /// # Errors
    /// A frame does not read or fold; the unowned side's turns do not end; a computer may not end
    /// its turn.
    pub fn elapse(&mut self, now: u64) -> Result<(), String> {
        let mut out = Vec::new();
        self.session.elapse(now, &mut out);
        self.deliver(out)?;
        self.settle()
    }

    /// When the host's clock should next read (`GameSession::deadline`); none while no timer
    /// runs.
    #[must_use]
    pub fn deadline(&self) -> Option<u64> {
        self.session.deadline()
    }

    /// `player`'s record so far (`GameSession::record`), none for a player outside the game.
    #[must_use]
    pub fn record(&self, player: PlayerId) -> Option<String> {
        self.session.record(player)
    }

    /// `player`'s view left now with its heroes' `facings` (E24 M5 T3b, [`Left`]); none for a
    /// player outside the game or before its snapshot.
    #[must_use]
    pub fn leave(&self, player: PlayerId, facings: Vec<(u16, u8)>) -> Option<Left> {
        let seat = self.seat(player)?;
        Some(Left {
            n: seat.n,
            view: seat.view.clone()?,
            facings,
        })
    }

    /// `player`'s record since its view was left at `left` (E24 M5 T3b), as the map client's live
    /// start reads it (`h3_map_view::engine::live`): its first message a snapshot of the view
    /// then (numbered as the message it was left at, the record's `first`), then the record's
    /// later messages; its whole record without one. With the facings the host saw (none: every
    /// hero faces right, as at a new game).
    ///
    /// # Errors
    /// No record of `player`; it does not read.
    pub fn since(
        &self,
        player: PlayerId,
        left: Option<&Left>,
    ) -> Result<(GameRecord, Live), String> {
        let text = self
            .record(player)
            .ok_or_else(|| format!("no record of player {}", player.index()))?;
        let mut record =
            engine::parse(&text).map_err(|e| format!("player {}: {e}", player.index()))?;
        let picks = Some(self.picks);
        let Some(left) = left else {
            return Ok((
                record,
                Live {
                    facings: Vec::new(),
                    picks,
                },
            ));
        };
        record
            .messages
            .retain(|m| number(m).is_some_and(|n| n > left.n));
        record.messages.insert(
            0,
            ServerMessage::GameSnapshot {
                n: left.n,
                observation: left.view.clone(),
            },
        );
        record.header.first = left.n;
        let live = Live {
            facings: left.facings.clone(),
            picks,
        };
        Ok((record, live))
    }

    /// `player`'s `command`, as the host vouches for it, played and its frames folded.
    ///
    /// # Errors
    /// The game's rejection; a frame does not read or fold.
    fn command(&mut self, player: PlayerId, command: GameCommand) -> Result<(), String> {
        self.seq += 1;
        let mut out = Vec::new();
        let played = self.session.play(player, self.seq, command, &mut out);
        // A rejection is told to the player too: its frames folded first.
        self.deliver(out)?;
        played.map_err(|code| {
            let colour = self.seats[player.index()].colour;
            format!("colour {colour}: the game rejects its command: {code:?}")
        })
    }

    /// What follows any command (E24 M5 T6a): a battle between two players' sides refused, the
    /// unowned battle side's turns played, the computers' turns ended.
    ///
    /// # Errors
    /// [`HotSeat::duels`]', [`HotSeat::sides`]', [`HotSeat::computers`]'.
    fn settle(&mut self) -> Result<(), String> {
        self.duels()?;
        self.sides()?;
        self.computers()
    }

    /// A battle between two players' sides open (E24 M5 T6c's: a human's hero attacking another
    /// player's hero, town or garrison; no seat plays the other side here, a computer's
    /// idling, `MCL-STUB-5`) refused.
    ///
    /// # Errors
    /// Such a battle is open.
    fn duels(&self) -> Result<(), String> {
        let state = self.session.state();
        for seat in &self.seats {
            // stub: CLT-STUB-5 (a battle between two players' sides, E24 M5 T6c; MCL-STUB-3)
            let Some(battle) = state.battle(seat.player) else {
                continue;
            };
            if let [Some(attacker), Some(defender)] = battle.owners() {
                let colour = |p: PlayerId| self.seat(p).map_or(p.raw(), |s| s.colour);
                return Err(format!(
                    "colour {} attacks colour {}: a battle between two players' sides is not \
                     played yet (E24 M5 T6c: CLT-STUB-5, MCL-STUB-3)",
                    colour(attacker),
                    colour(defender)
                ));
            }
        }
        Ok(())
    }

    /// The unowned battle side's turns while it is to act (E24 M5 T6a: a monster's, guards', a
    /// neutral's), each the system actor's command (`GameSession::system`) the original's battle
    /// AI chooses seeing everything (`h3_battle_ai::choose`, as `h3_sim::computer`'s system
    /// actor) from the host's stream; a choice the engine refuses replaced by the first of the
    /// side's legal commands it takes (as the session's own system steps), its frames folded.
    ///
    /// # Errors
    /// The engine takes none of the side's commands; the side still acts after
    /// [`SIDE_STEPS`]; a frame does not read or fold.
    fn sides(&mut self) -> Result<(), String> {
        for _ in 0..SIDE_STEPS {
            let state = self.session.state();
            let Some(battle) = state.unowned_battle() else {
                return Ok(());
            };
            let Some(side) = battle.state().side_to_act() else {
                return Ok(());
            };
            let choice =
                h3_battle_ai::choose(battle.state(), self.rules.battle(), side, &mut self.sides);
            let choice = h3_protocol::map::game::command(&h3_game::GameCommand::Battle(choice));
            let mut out = Vec::new();
            let mut candidates = core::iter::once(choice).chain(self.session.system_legal());
            let played = candidates.any(|command| self.session.system(command, &mut out).is_ok());
            self.deliver(out)?;
            if !played {
                return Err("the unowned battle side's commands are all refused".into());
            }
        }
        Err(format!(
            "the unowned battle side still acts after {SIDE_STEPS} commands"
        ))
    }

    /// Each computer that may act has its turn ended (`MCL-STUB-5`), until none may.
    ///
    /// # Errors
    /// A computer may act but not end its turn (an open decision of its); the game rejects it; a
    /// frame does not read or fold.
    fn computers(&mut self) -> Result<(), String> {
        // Each End Turn passes the turn on: within one round a human's turn runs again.
        for _ in 0..=self.seats.len() {
            // stub: MCL-STUB-5 (a computer colour's turn ended at once: E31's increment 2)
            let Some(seat) = self.seats.iter().find(|s| !s.human && s.legal.is_some()) else {
                return Ok(());
            };
            let (player, colour) = (seat.player, seat.colour);
            let end = GameCommand::EndTurn {};
            if !seat.legal.as_ref().is_some_and(|l| l.contains(&end)) {
                return Err(format!(
                    "colour {colour}: a computer that may act but not end its turn (MCL-STUB-5)"
                ));
            }
            self.command(player, end)?;
        }
        Err("the computers' turns do not end".into())
    }

    /// The session's frames read and folded into their players' seats, in order; a human's
    /// battle's kept from its start (E24 M5 T6a, [`HotSeat::keep`]).
    ///
    /// # Errors
    /// A frame does not read; it is not a player of the game's; it does not fold.
    fn deliver(&mut self, out: Vec<GameOutbound>) -> Result<(), String> {
        let mut ended = None;
        for GameOutbound { to, frame } in out {
            let message = codec::decode_server(frame.as_bytes(), MAX_LINE)
                .map_err(|e| format!("player {}: a frame does not read: {e:?}", to.index()))?;
            let seat = self.seats.get(to.index());
            let seat = seat.ok_or_else(|| format!("a frame for player {}, no seat", to.index()))?;
            if seat.human {
                self.keep(to, &message);
                if ends(&message) && self.fighting() == Some(to) {
                    ended = Some(to);
                }
            }
            self.seats[to.index()].fold(message)?;
        }
        // The battle's end (E24 M5 T6a, *provisional*, the in-process host's): its human's view is
        // the engine's projection again, since `BattleEnded` carries no hero's survivors and the
        // fold keeps the armies as they were (SRV-STUB-9); the view left at the turn's end then
        // starts the next turn right.
        if let Some(player) = ended {
            let view = observe(self.session.state(), self.rules, Viewer::Player(player));
            self.seats[player.index()].view = Some(view);
        }
        Ok(())
    }

    /// The human `player`'s `message` kept for its battle's seat (E24 M5 T6a): from the
    /// `GameEvents` whose `BattleStarted` opens a battle with a side of its (a battle it sees
    /// another's fight is not), every later message of its until the battle's return; a later
    /// battle's start begins the kept messages again. The state is the one after the command whose
    /// frames these are: in this host no command both opens a battle and ends it.
    fn keep(&mut self, player: PlayerId, message: &ServerMessage) {
        let state = self.session.state();
        let own = || {
            state
                .battle(player)
                .is_some_and(|b| b.side_of(player).is_some())
        };
        if begins(message) && own() {
            self.fight = Some(Fight {
                player,
                messages: Vec::new(),
                handed: 0,
            });
        }
        if let Some(fight) = self.fight.as_mut().filter(|f| f.player == player) {
            fight.messages.push(message.clone());
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use h3_battle_client::battle::Remote;
    use h3_protocol::wire::Command;
    use h3_protocol::wire::game::{AdventureCommand, Pos as WirePos};

    use super::*;
    use crate::play::begin;
    use crate::play::tests::{MapFile, PEASANTS, TWO, peasants};

    /// A test's human side in its battle: a shot, else an attack, else its first legal command.
    pub(crate) fn strike(legal: &[Command]) -> Command {
        let first = |f: fn(&Command) -> bool| legal.iter().find(|c| f(c)).copied();
        first(|c| matches!(c, Command::Shoot { .. }))
            .or_else(|| first(|c| matches!(c, Command::Attack { .. })))
            .or_else(|| legal.first().copied())
            .unwrap()
    }

    /// A test's human side that defends (else its first legal command): the other side wins.
    fn defend(legal: &[Command]) -> Command {
        let defend = Command::Defend {};
        if legal.contains(&defend) {
            defend
        } else {
            legal[0]
        }
    }

    /// The battle `seat` follows played to its end as a window plays it (E24 M5 T6a): a host
    /// pass (`serve`, the engine refusing none of the seat's commands), the seat polled, its turn
    /// played by `choose`; the passes it took.
    pub(crate) fn fight(
        seat: &mut GameSeat,
        mut serve: impl FnMut(&mut GameSeat) -> Vec<GameReject>,
        choose: fn(&[Command]) -> Command,
    ) -> usize {
        for pass in 1..2_000 {
            let refused = serve(seat);
            assert!(refused.is_empty(), "{refused:?}");
            seat.poll(0).unwrap();
            if seat.over() {
                return pass;
            }
            if let Some((legal, _)) = seat.turn() {
                let command = choose(legal);
                seat.send(command).unwrap();
            }
        }
        panic!("the battle did not end");
    }

    /// `player`'s first hero in its seat's view: its id and tile.
    pub(crate) fn hero(hot: &HotSeat<'_>, player: PlayerId) -> (u16, WirePos) {
        let view = hot.seat(player).unwrap().view.as_ref().unwrap();
        let id = view.hero_list[0];
        let h = view.adventure.heroes.iter().find(|h| h.id == id).unwrap();
        (h.id, h.pos)
    }

    /// The human in turn's hero `hero` walked along `path` (`MoveHero`).
    pub(crate) fn walk(hero: u16, path: Vec<WirePos>) -> GameCommand {
        GameCommand::Adventure {
            command: AdventureCommand::MoveHero { hero, path },
        }
    }

    /// The monsters in `player`'s view: their tiles.
    fn monsters(hot: &HotSeat<'_>, player: PlayerId) -> Vec<WirePos> {
        let view = hot.seat(player).unwrap().view.as_ref().unwrap();
        view.adventure
            .objects
            .iter()
            .filter(|o| o.monster.is_some())
            .map(|o| o.anchor)
            .collect()
    }

    impl HotSeat<'_> {
        /// The game's state (the tests' measure of the seats' views).
        fn state(&self) -> &h3_game::GameState {
            self.session.state()
        }
    }

    /// Each seat's folded view against the engine's `observe` for its player.
    fn views_are_the_engine_s(hot: &HotSeat<'_>, rules: &GameRuleset) {
        for seat in hot.seats() {
            let observed = observe(hot.state(), rules, Viewer::Player(seat.player));
            assert_eq!(
                seat.view.as_ref(),
                Some(&observed),
                "colour {}",
                seat.colour
            );
        }
    }

    #[test]
    fn the_turn_s_duration_is_the_original_s_minutes() {
        let minutes = |d| clock(d).unwrap().turn_ms.map(|ms| ms / 60_000);
        assert_eq!(minutes(0), Some(1));
        assert_eq!(minutes(6), Some(15));
        assert_eq!(minutes(9), Some(30));
        assert_eq!(clock(UNLIMITED).unwrap(), Clock::UNLIMITED);
        let e = clock(11).unwrap_err();
        assert_eq!(e, "the turn's duration 11: none (0 to 10)");
        let limited = clock(2).unwrap();
        assert_eq!(
            (limited.level_up_ms, limited.battle),
            (None, Allowance::DEFAULT)
        );
    }

    #[test]
    fn two_humans_pass_the_turn_each_seat_its_own_view() {
        let rules = h3_battle_client::maps::rules().unwrap();
        let file = MapFile::new("seats", TWO);
        let mut game = begin(&file.0, None, (4, UNLIMITED), &rules).unwrap();
        game.names = [Some("Ann".into()), Some("Bo".into()), Some("Cy".into())]
            .into_iter()
            .chain(core::iter::repeat(None))
            .take(8)
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();
        let mut hot = HotSeat::start(&game, &rules).unwrap();
        let names: Vec<_> = hot.seats().iter().map(|s| s.name.as_deref()).collect();
        assert_eq!(names, [Some("Ann"), Some("Bo")]);
        let (red, blue) = (PlayerId::new(0), PlayerId::new(1));
        assert_eq!(hot.turn(), Some(red));
        assert!(hot.seat(red).unwrap().legal.is_some());
        assert_eq!(hot.seat(blue).unwrap().legal, None);
        views_are_the_engine_s(&hot, &rules);
        // Red's End Turn: Blue's turn the same day, each view the engine's still.
        assert_eq!(hot.end_turn().unwrap(), Some(blue));
        assert_eq!(hot.seat(red).unwrap().legal, None);
        views_are_the_engine_s(&hot, &rules);
        // Blue's: Red's on the next day.
        assert_eq!(hot.end_turn().unwrap(), Some(red));
        for seat in hot.seats() {
            let view = seat.view.as_ref().unwrap();
            assert_eq!((view.day, view.viewer), (2, seat.player.raw()));
        }
        assert_eq!(hot.seat(PlayerId::new(2)), None);
        // No clock runs: none is told, none to read.
        assert_eq!((hot.deadline(), hot.seat(red).unwrap().clock), (None, None));
    }

    #[test]
    fn a_human_walks_through_the_session_and_asks_its_moves() {
        use h3_protocol::wire::game::{AdventureCommand, Pos as WirePos};
        let rules = h3_battle_client::maps::rules().unwrap();
        let file = MapFile::new("walks", TWO);
        let game = begin(&file.0, None, (4, UNLIMITED), &rules).unwrap();
        let mut hot = HotSeat::start(&game, &rules).unwrap();
        let (red, blue) = (PlayerId::new(0), PlayerId::new(1));
        let hero = |player| {
            let view = hot.seat(player).unwrap().view.clone().unwrap();
            let h = view
                .adventure
                .heroes
                .iter()
                .find(|h| h.owner == player.raw());
            h.map(|h| (h.id, h.pos)).unwrap()
        };
        let ((ours, at), (theirs, _)) = (hero(red), hero(blue));
        let moves =
            |hot: &HotSeat<'_>, player, hero| match hot.query(player, GameQuery::Moves { hero }) {
                Some(GameAnswer::Moves { moves, .. }) => moves,
                _ => None,
            };
        // Red's hero's moves; none of Blue's hero for Red.
        let reach = moves(&hot, red, ours).unwrap();
        assert_eq!(reach.origin, at);
        assert!(moves(&hot, red, theirs).is_none());
        // A step east taken: its frames folded into Red's view.
        let east = WirePos { x: at.x + 1, ..at };
        let walk = |path| GameCommand::Adventure {
            command: AdventureCommand::MoveHero { hero: ours, path },
        };
        assert_eq!(hot.play(walk(vec![east])).unwrap(), None);
        let view = hot.seat(red).unwrap().view.clone().unwrap();
        assert!(
            view.adventure
                .heroes
                .iter()
                .any(|h| (h.id, h.pos) == (ours, east))
        );
        // A path of no neighbours refused, nothing changed but the told rejection.
        let far = WirePos { x: at.x + 9, ..at };
        assert!(hot.play(walk(vec![far])).unwrap().is_some());
        assert_eq!(hot.seat(red).unwrap().view.as_ref(), Some(&view));
        views_are_the_engine_s(&hot, &rules);
        // Blue's turn: Red's hero has no moves now.
        assert_eq!(hot.end_turn().unwrap(), Some(blue));
        assert!(moves(&hot, red, ours).is_none());
    }

    #[test]
    fn a_computer_s_turn_ends_at_once() {
        let rules = h3_battle_client::maps::rules().unwrap();
        let file = MapFile::new("computer", TWO);
        // Red the computer's: its turn ended at the start, Blue's the human's.
        let game = begin(&file.0, Some(&[1]), (4, UNLIMITED), &rules).unwrap();
        let mut hot = HotSeat::start(&game, &rules).unwrap();
        let (red, blue) = (PlayerId::new(0), PlayerId::new(1));
        assert_eq!(hot.turn(), Some(blue));
        let seat = hot.seat(red).unwrap();
        assert!(!seat.human && seat.legal.is_none() && seat.name.is_none());
        // Blue's End Turn: the day passes, Red's turn ended at once, Blue's again.
        assert_eq!(hot.end_turn().unwrap(), Some(blue));
        assert_eq!(hot.seat(blue).unwrap().view.as_ref().unwrap().day, 2);
        // Red's record holds its turns ended (its own End Turns accepted).
        let record = hot.record(red).unwrap();
        assert_eq!(record.matches("GameAccepted").count(), 2, "{record}");
    }

    #[test]
    fn a_turn_whose_time_runs_out_ends() {
        let rules = h3_battle_client::maps::rules().unwrap();
        let file = MapFile::new("clock", TWO);
        // A minute's turn.
        let game = begin(&file.0, None, (4, 0), &rules).unwrap();
        let mut hot = HotSeat::start(&game, &rules).unwrap();
        let (red, blue) = (PlayerId::new(0), PlayerId::new(1));
        hot.elapse(1_000).unwrap();
        let timer = hot.seat(red).unwrap().clock.and_then(|c| c.turn);
        assert!(timer.is_some(), "{:?}", hot.seat(red).unwrap().clock);
        let deadline = hot.deadline().unwrap();
        assert!(deadline > 1_000 && deadline <= 61_000, "{deadline}");
        hot.elapse(30_000).unwrap();
        assert_eq!(hot.turn(), Some(red));
        // A minute after the first reading: the session ends Red's turn, Blue's runs.
        hot.elapse(61_000).unwrap();
        assert_eq!(hot.turn(), Some(blue));
        views_are_the_engine_s(&hot, &rules);
    }

    #[test]
    fn no_human_starts_no_game() {
        let rules = h3_battle_client::maps::rules().unwrap();
        let file = MapFile::new("nobody", TWO);
        let mut game = begin(&file.0, None, (4, UNLIMITED), &rules).unwrap();
        for player in &mut game.setup.players {
            player.human = false;
        }
        let e = HotSeat::start(&game, &rules).err().unwrap();
        assert!(e.ends_with("no player is human"), "{e}");
        game.setup.players[0].human = true;
        game.duration = 11;
        let e = HotSeat::start(&game, &rules).err().unwrap();
        assert!(e.ends_with("the turn's duration 11: none (0 to 10)"), "{e}");
    }

    #[test]
    fn a_seat_s_messages_follow_each_other() {
        let mut seat = Seat {
            player: PlayerId::new(0),
            colour: 3,
            human: true,
            name: None,
            view: None,
            legal: Some(Vec::new()),
            clock: None,
            n: 0,
        };
        let events = |n| ServerMessage::GameEvents {
            n,
            events: vec![GameEvent::TurnEnded { player: 0 }],
        };
        let e = seat.fold(events(2)).unwrap_err();
        assert_eq!(e, "colour 3: message 2 after 0");
        let e = seat.fold(events(1)).unwrap_err();
        assert_eq!(e, "colour 3: told the game before its snapshot");
        // A message without a number in the player's sequence passes.
        let goodbye = ServerMessage::Goodbye {
            reason: h3_protocol::wire::GoodbyeReason::Paused,
        };
        assert_eq!(number(&goodbye), None);
    }

    #[test]
    fn a_view_left_is_set_up_again_from_where_it_was_left() {
        // E24 M5 T3b: Red's view left after its End Turn (its facings kept); at its next turn its
        // record since then begins at a snapshot of that view, numbered as the message it was
        // left at, and its next turn's start is the live start's first turn.
        let rules = h3_battle_client::maps::rules().unwrap();
        let file = MapFile::new("left", TWO);
        let bytes = std::fs::read(&file.0).unwrap();
        let game = begin(&file.0, None, (4, UNLIMITED), &rules).unwrap();
        let mut hot = HotSeat::start(&game, &rules).unwrap();
        let (red, blue) = (PlayerId::new(0), PlayerId::new(1));
        assert_eq!(hot.end_turn().unwrap(), Some(blue));
        let hero = hot.seat(red).unwrap().view.as_ref().unwrap().hero_list[0];
        let left = hot.leave(red, vec![(hero, 6)]).unwrap();
        assert_eq!(left.n, hot.seat(red).unwrap().told());
        assert_eq!(hot.leave(PlayerId::new(5), Vec::new()), None);
        // Nothing told since: the record since is the snapshot alone.
        let (since, live) = hot.since(red, Some(&left)).unwrap();
        assert_eq!(since.header.first, left.n);
        assert_eq!(since.messages.len(), 1);
        assert_eq!(live.facings, [(hero, 6)]);
        // Blue's End Turn: Red's day 2 after the snapshot, as its whole record has it.
        assert_eq!(hot.end_turn().unwrap(), Some(red));
        let (since, live) = hot.since(red, Some(&left)).unwrap();
        let ServerMessage::GameSnapshot { n, observation } = &since.messages[0] else {
            panic!("no snapshot first");
        };
        assert_eq!((*n, observation), (left.n, &left.view));
        let (whole, _) = hot.since(red, None).unwrap();
        assert_eq!(
            whole.messages[whole.messages.len() - since.messages.len() + 1..],
            since.messages[1..]
        );
        let loaded = engine::live(since, &bytes, &rules, live).unwrap();
        let art = h3_map_view::art::MapArt::from_raw(&loaded.raw).unwrap();
        let played = engine::played(&loaded, &art, &rules).unwrap();
        assert!(played.waits);
        assert_eq!(played.turns.len(), 1);
        let whole = engine::check(whole, &bytes, &rules).unwrap();
        let all = engine::played(&whole, &art, &rules).unwrap();
        assert_eq!(all.turns.last(), played.turns.first());
    }

    #[test]
    fn a_human_s_battle_with_a_monster_is_played_through_its_seat() {
        // E24 M5 T6a (design §5.33): Red's walk onto savage Peasants opens a battle the host
        // holds; its seat, fed from the battle's start, opened as the attacker; Red's turn runs
        // on, its legal commands the battle's; played to its end through the seat (the other side
        // the battle AI's), the seat's record is the battle the player's record shows; the
        // monster gone from the view, which is the engine's; Blue's computer turn ended after
        // Red's End Turn as before.
        let rules = h3_battle_client::maps::rules().unwrap();
        let file = MapFile::with("monster", &peasants("[[2, 5]]", 15), &[PEASANTS]);
        let game = begin(&file.0, Some(&[0]), (4, UNLIMITED), &rules).unwrap();
        let mut hot = HotSeat::start(&game, &rules).unwrap();
        let red = PlayerId::new(0);
        let (ours, at) = hero(&hot, red);
        let north = WirePos { y: at.y - 1, ..at };
        assert_eq!(monsters(&hot, red), [north]);
        assert_eq!((hot.fighting(), hot.battle().is_some()), (None, false));
        assert_eq!(hot.play(walk(ours, vec![north])).unwrap(), None);
        assert_eq!(hot.fighting(), Some(red));
        let mut seat = hot.battle().unwrap();
        assert!(seat.opened());
        assert_eq!(seat.seat(), h3_protocol::wire::Seat::Attacker);
        // The seat's header is the record's.
        let record = engine::parse(&hot.record(red).unwrap()).unwrap();
        assert_eq!(record.header, hot.header(red));
        assert_eq!(hot.turn(), Some(red));
        let legal = hot.seat(red).unwrap().legal.clone().unwrap();
        assert!(
            legal
                .iter()
                .all(|c| matches!(c, GameCommand::Battle { .. }))
        );
        // No End Turn in a battle.
        assert!(hot.end_turn().is_err());
        let passes = fight(&mut seat, |s| hot.serve(s).unwrap(), strike);
        assert!(passes > 1);
        let text = hot.record(red).unwrap();
        let battles = h3_view::game_record::parse(&text).unwrap().battles();
        assert_eq!(battles.len(), 1);
        assert_eq!(seat.record(), Some(&battles[0].record));
        assert!(battles[0].at.ended.is_some(), "{:?}", battles[0].at);
        assert!(text.contains("BattleEnded"));
        // The battle won: the monster gone, the hero where it stood; nothing left open.
        assert_eq!(monsters(&hot, red), []);
        assert_eq!(hero(&hot, red), (ours, at));
        assert_eq!(hot.decision(red), None);
        views_are_the_engine_s(&hot, &rules);
        assert_eq!(hot.returned(), Some(red));
        assert_eq!((hot.fighting(), hot.battle().is_some()), (None, false));
        assert!(hot.serve(&mut seat).is_err());
        // Red's End Turn: Blue's computer turn ended at once, Red's day 2.
        assert_eq!(hot.end_turn().unwrap(), Some(red));
        assert_eq!(hot.seat(red).unwrap().view.as_ref().unwrap().day, 2);
        let blue = hot.record(PlayerId::new(1)).unwrap();
        assert_eq!(blue.matches("GameAccepted").count(), 1);
    }

    #[test]
    fn a_human_s_hero_lost_in_its_battle_leaves_the_map() {
        // E24 M5 T6a: a lone Pikeman defending against fifty Peasants: the hero lost leaves the
        // view, Red's turn runs on with its other hero, the Peasants stay.
        let rules = h3_battle_client::maps::rules().unwrap();
        let file = MapFile::with("lost", &peasants("[[0, 1]]", 50), &[PEASANTS]);
        let game = begin(&file.0, None, (4, UNLIMITED), &rules).unwrap();
        let mut hot = HotSeat::start(&game, &rules).unwrap();
        let red = PlayerId::new(0);
        let (ours, at) = hero(&hot, red);
        let north = WirePos { y: at.y - 1, ..at };
        assert_eq!(hot.play(walk(ours, vec![north])).unwrap(), None);
        let mut seat = hot.battle().unwrap();
        fight(&mut seat, |s| hot.serve(s).unwrap(), defend);
        let view = hot.seat(red).unwrap().view.as_ref().unwrap();
        assert!(!view.hero_list.contains(&ours), "{:?}", view.hero_list);
        assert!(view.adventure.heroes.iter().all(|h| h.id != ours));
        assert_eq!(monsters(&hot, red), [north]);
        assert_eq!(hot.returned(), Some(red));
        views_are_the_engine_s(&hot, &rules);
        assert_eq!(hot.end_turn().unwrap(), Some(PlayerId::new(1)));
    }

    #[test]
    fn a_battle_s_losses_show_in_the_view_after_it() {
        // E24 M5 T6a: twenty Pikemen beat ninety Peasants and lose some; `BattleEnded` carries no
        // survivors (SRV-STUB-9), and the host's view of Red is the engine's after the battle.
        let rules = h3_battle_client::maps::rules().unwrap();
        let file = MapFile::with("losses", &peasants("[[0, 20]]", 90), &[PEASANTS]);
        let game = begin(&file.0, None, (4, UNLIMITED), &rules).unwrap();
        let mut hot = HotSeat::start(&game, &rules).unwrap();
        let red = PlayerId::new(0);
        let (ours, at) = hero(&hot, red);
        let north = WirePos { y: at.y - 1, ..at };
        assert_eq!(hot.play(walk(ours, vec![north])).unwrap(), None);
        let mut seat = hot.battle().unwrap();
        fight(&mut seat, |s| hot.serve(s).unwrap(), strike);
        assert_eq!(monsters(&hot, red), []);
        views_are_the_engine_s(&hot, &rules);
        let view = hot.seat(red).unwrap().view.as_ref().unwrap();
        let army = view.adventure.heroes.iter().find(|h| h.id == ours);
        let own = army.and_then(|h| h.own.as_ref()).unwrap();
        // The seed's battle: some Pikemen fell.
        assert!(own.army.iter().all(|s| s.count < 20), "{:?}", own.army);
    }

    #[test]
    fn a_battle_won_may_leave_a_level_up_open() {
        // E24 M5 T6a: a thousand Peasants beaten by twenty Archangels: the hero's experience
        // reaches a level, whose offer is the open decision after the battle (Red's turn holds,
        // its legal commands the level-up's answers).
        let rules = h3_battle_client::maps::rules().unwrap();
        let file = MapFile::with("level", &peasants("[[13, 20]]", 1_000), &[PEASANTS]);
        let game = begin(&file.0, None, (4, UNLIMITED), &rules).unwrap();
        let mut hot = HotSeat::start(&game, &rules).unwrap();
        let red = PlayerId::new(0);
        let (ours, at) = hero(&hot, red);
        let north = WirePos { y: at.y - 1, ..at };
        assert_eq!(hot.play(walk(ours, vec![north])).unwrap(), None);
        let mut seat = hot.battle().unwrap();
        fight(&mut seat, |s| hot.serve(s).unwrap(), strike);
        assert_eq!(monsters(&hot, red), []);
        assert_eq!(hot.turn(), Some(red));
        assert_eq!(hot.session.state().waiting_for(), Some(red));
        let legal = hot.seat(red).unwrap().legal.clone().unwrap();
        assert!(
            legal
                .iter()
                .all(|c| matches!(c, GameCommand::Answer { .. })),
            "{legal:?}"
        );
        assert_eq!(hot.returned(), Some(red));
    }

    #[test]
    fn a_battle_between_two_players_is_refused() {
        // E24 M5 T6a: Red's hero attacking Blue's (a computer's, then a human's): the engine opens
        // the battle, the host refuses it (E24 M5 T6c's).
        let rules = h3_battle_client::maps::rules().unwrap();
        let file = MapFile::new(
            "duel",
            "size = 36\nterrain = \"grass\"\n\
             heroes = [{ at = [3, 3] }, { at = [4, 3], owner = 1 }]\nview = [0, 0, 0]\n",
        );
        for humans in [Some(&[0][..]), None] {
            let game = begin(&file.0, humans, (4, UNLIMITED), &rules).unwrap();
            let mut hot = HotSeat::start(&game, &rules).unwrap();
            let (ours, at) = hero(&hot, PlayerId::new(0));
            let east = WirePos { x: at.x + 1, ..at };
            let e = hot.play(walk(ours, vec![east])).unwrap_err();
            assert_eq!(
                e,
                "colour 0 attacks colour 1: a battle between two players' sides is not played \
                 yet (E24 M5 T6c: CLT-STUB-5, MCL-STUB-3)"
            );
        }
    }
}
