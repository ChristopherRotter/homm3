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
//!   is an error. A battle's own `Snapshot` is not folded (no battle opens before M5 T6, "the
//!   battles in play").
//! - **The turn:** the human whose turn runs and who may act ([`HotSeat::turn`]); its End Turn
//!   ([`HotSeat::end_turn`]) is played as its command (`GameSession::play`, numbered by the host).
//!   **A computer colour's turn ends at once** (`MCL-STUB-5`, the user at M5 T0, lifted by E31's
//!   increment 2): whenever a computer may act, the host ends its turn for it, until a human's
//!   turn runs.
//! - **The clock:** the turn's duration (`turndur.txt`'s record, [`clock`]) the session's turn
//!   timer; the host reads its clock into the session ([`HotSeat::elapse`], before the session's
//!   [`HotSeat::deadline`]), and a turn whose time ran out is ended by the session's system actor,
//!   its frames folded as a command's. The window's clock drives it from T3b on.
//! - **The records:** each seat's record so far ([`HotSeat::record`], `GameSession::record`: the
//!   whole game one record), which the record player reads and plays (`h3-map-view record`).
//!
//! *Provisional* (T3a, an agent decision): the names, the frames read back from their text, the
//! clock's level-up time and battle allowance (none, the session's default: T5 reads the
//! original's), an error for a computer that may act but not end its turn.

use std::ffi::OsStr;
use std::path::Path;

use h3_contracts::PlayerId;
use h3_game::GameRuleset;
use h3_map_view::engine::{self, GameRecord, Live};
use h3_protocol::codec;
use h3_protocol::map::game::manifest;
use h3_protocol::record::MAX_LINE;
use h3_protocol::session::game::{
    Allowance, Clock, CommandBounds, GameOptions, GameOutbound, GameSession, Spectators,
};
use h3_protocol::wire::game::{GameClock, GameCommand, GameObservation, MapIdentity};
use h3_protocol::wire::{ServerMessage, WireU64};

use crate::play::Game;

/// The original's turn durations by `turndur.txt`'s record, in minutes, 0 unlimited
/// (`turn_duration_minutes` 0x641AE8, read in E16 M5 T15: `docs/epics/E16-timer-survey.md`).
pub const TURN_MINUTES: [u8; 11] = [1, 2, 4, 6, 8, 10, 15, 20, 25, 30, 0];

/// The turn duration's record that is unlimited (`turndur.txt`'s last).
pub const UNLIMITED: u8 = 10;

/// The session's clock of the turn's duration `duration` (`turndur.txt`'s record, 0 to 10): its
/// minutes the turn timer's limit, none for [`UNLIMITED`]; no level-up time, the default battle
/// allowance (*provisional*: M5 T5 reads the original's).
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
    /// Its view, every frame it was told folded in; none before its first snapshot.
    pub view: Option<GameObservation>,
    /// The commands it may give now, as its last `GameTurn` listed them; none when it may not act.
    pub legal: Option<Vec<GameCommand>>,
    /// Its clock as last told, if it was.
    pub clock: Option<GameClock>,
    /// The number of its last message.
    n: u32,
}

impl Seat {
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
            // A command's answers change no view; a battle's `Snapshot` waits for M5 T6.
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

/// The game hosted in-process, a seat per player.
pub struct HotSeat<'r> {
    session: GameSession<'r>,
    seats: Vec<Seat>,
    /// The host's last command's number (the in-process host's are only echoed).
    seq: u32,
}

impl std::fmt::Debug for HotSeat<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HotSeat")
            .field("seats", &self.seats)
            .field("seq", &self.seq)
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
            game: 0,
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
            seats.push(Seat {
                player,
                colour: setup.colour,
                human: setup.human,
                name,
                view: None,
                legal: None,
                clock: None,
                n: 0,
            });
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
            seats,
            seq: 0,
        };
        hot.deliver(out)?;
        hot.computers()?;
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
    /// another player's holding the game, E10's end).
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
        self.computers()?;
        Ok(self.turn())
    }

    /// The host's clock reads `now`, in milliseconds (`GameSession::elapse`): a turn whose time
    /// ran out ended by the session, its frames folded; the computers' turns ended after it.
    ///
    /// # Errors
    /// A frame does not read or fold; a computer may not end its turn.
    pub fn elapse(&mut self, now: u64) -> Result<(), String> {
        let mut out = Vec::new();
        self.session.elapse(now, &mut out);
        self.deliver(out)?;
        self.computers()
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
        let Some(left) = left else {
            return Ok((record, Live::default()));
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

    /// The session's frames read and folded into their players' seats, in order.
    ///
    /// # Errors
    /// A frame does not read; it is not a player of the game's; it does not fold.
    fn deliver(&mut self, out: Vec<GameOutbound>) -> Result<(), String> {
        for GameOutbound { to, frame } in out {
            let message = codec::decode_server(frame.as_bytes(), MAX_LINE)
                .map_err(|e| format!("player {}: a frame does not read: {e:?}", to.index()))?;
            let seat = self
                .seats
                .get_mut(to.index())
                .ok_or_else(|| format!("a frame for player {}, no seat", to.index()))?;
            seat.fold(message)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use h3_game::Viewer;
    use h3_protocol::map::game::observe;
    use h3_protocol::wire::game::GameEvent;

    use super::*;
    use crate::play::begin;
    use crate::play::tests::{MapFile, TWO};

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
}
