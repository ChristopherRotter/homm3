//! A whole game's record (E17 M7 T3b1; record format 4, E16 M5 T5b, `h3_protocol::session::game`):
//! its first line a [`GameRecordHeader`] (the game's manifest and the player), every further line
//! one numbered server message the player was sent. The battles it shows the player come out each
//! as the battle's own [`Record`], as a seat's record of that battle would hold it, which the
//! battle client plays as any record: a host's game handed to the battle and back (the map client
//! lane's E24 M4 T5).
//!
//! A battle opens for the player with the battle's own `Snapshot` (SRV-Q27 (1)), right after the
//! `GameEvents` whose `BattleStarted` began it; the battle's events after `BattleStarted` there are
//! its start's. From then on its events come inside `GameEvents` (`GameEvent::Battle`), the
//! player's turns in `GameTurn`, its commands' acceptance in `GameAccepted` and refusal in
//! `GameRejected` (`GameCommand::Battle`, `GameReject::Battle`), until `BattleEnded` tells its
//! result. The battle's record holds, numbered afresh from 1: the snapshot; the start's events;
//! each of the player's turns (`YourTurn`, the turn's battle commands), acceptances and refusals;
//! each message's battle events (`Events`; a message without any adds none); a later snapshot of
//! the battle; and `GameEnded` with the result. What a seat's record of the battle holds and this
//! one cannot: the battle as it ended (`SRV-STUB-14`: the game sends no last snapshot yet), so the
//! seat's own hero's Eagle Eye picks.
//!
//! [`Battles`] reads the messages one at a time (E17 M7 T3b2): a host's game played live hands the
//! battle client its player's messages as they come.
//!
//! *Provisional* (E17 M7 T3b1): the names, the battles in the order they open, the numbers
//! afresh, the record's seat the snapshot's viewer, a `BattleStarted` while a battle is open
//! ending that one unended (the engine opens one battle at a time).

use alloc::vec;
use alloc::vec::Vec;

use h3_protocol::codec;
use h3_protocol::wire::game::{
    GameCommand, GameEvent, GameManifest, GameRecordHeader, GameReject, Pos,
};
use h3_protocol::wire::{
    Event, Manifest, Observation, Outcome, RecordHeader, Seat, ServerMessage, Side,
};
use h3_protocol::{PROTOCOL_VERSION, RECORD_FORMAT_VERSION};

use crate::record::{MAX_LINE, Record, RecordError, decode};

/// A whole game's record, read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameRecord {
    /// What the game is made of and whose record it is.
    pub header: GameRecordHeader,
    /// The player's messages, in order.
    pub messages: Vec<ServerMessage>,
}

/// A battle of a whole game as its record shows it to the player.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameBattle {
    /// The battle's own record, as a seat's of it would hold it (the module's documentation).
    pub record: Record,
    /// Where the battle stands in the game.
    pub at: InGame,
}

/// Where a battle stands in its game (E17 M7 T3b1): what a host needs to switch to it and back.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InGame {
    /// Its heroes by side, attacker first (`BattleStarted`; none: the record does not hold its
    /// start).
    pub heroes: [Option<u16>; 2],
    /// Its tile on the adventure map (none: the record does not hold its start).
    pub tile: Option<Pos>,
    /// The game record's number of the battle's snapshot: where it opened for the player.
    pub opened: u32,
    /// The game record's number of the `GameEvents` that ended it, and how it ended; none: the
    /// record ends before.
    pub ended: Option<(u32, Outcome)>,
}

/// Whether `text` is a whole game's record: its first line not blank reads as a
/// [`GameRecordHeader`] (a battle's own record's first line, a `RecordHeader`, does not: both
/// deny unknown fields).
#[must_use]
pub fn is_game(text: &str) -> bool {
    let first = text.lines().find(|line| !line.trim().is_empty());
    first.is_some_and(|line| codec::decode::<GameRecordHeader>(line.as_bytes(), MAX_LINE).is_ok())
}

/// Reads a whole game's record's text; blank lines are skipped.
///
/// # Errors
/// The [`RecordError`] of the first line that does not read; a format or protocol not this
/// client's.
pub fn parse(text: &str) -> Result<GameRecord, RecordError> {
    let mut lines = text
        .lines()
        .enumerate()
        .filter(|(_, line)| !line.trim().is_empty());
    let (_, first) = lines.next().ok_or(RecordError::Empty)?;
    let header: GameRecordHeader = decode(1, first)?;
    if header.format != RECORD_FORMAT_VERSION {
        return Err(RecordError::Format(header.format));
    }
    if header.manifest.protocol != PROTOCOL_VERSION {
        return Err(RecordError::Protocol(header.manifest.protocol));
    }
    let messages = lines
        .map(|(i, line)| decode(i + 1, line))
        .collect::<Result<_, _>>()?;
    Ok(GameRecord { header, messages })
}

/// The battle's manifest of a whole game's: its battle part's identity and key tables.
#[must_use]
pub fn manifest(game: &GameManifest) -> Manifest {
    Manifest {
        protocol: game.protocol,
        engine: game.engine.clone(),
        packs: game.packs.clone(),
        profile: game.profile.clone(),
        ruleset_hash: game.ruleset_hash,
        battle_ruleset_hash: game.battle_ruleset_hash,
        rng: game.rng.clone(),
        hash: game.hash.clone(),
        battle_state_hash_version: game.battle_state_hash_version,
        field: game.field,
        keys: game.keys.clone(),
    }
}

/// A battle's start as a `GameEvents` message tells it: its heroes, its tile, the battle's events
/// after it, and its end if the same message holds it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct Start {
    heroes: [Option<u16>; 2],
    tile: Option<Pos>,
    events: Vec<Event>,
    ended: Option<(u32, Outcome)>,
}

impl GameRecord {
    /// Every battle the record shows the player, in the order they open (the module's
    /// documentation); the last one still open where the record ends.
    #[must_use]
    pub fn battles(&self) -> Vec<GameBattle> {
        let mut battles = Battles::new(&self.header);
        for message in &self.messages {
            battles.read(message);
        }
        battles.finish()
    }
}

/// The battles a whole game's messages show its player, read one message at a time (E17 M7
/// T3b2: a host's game played live hands the battle its player's messages as they come;
/// [`GameRecord::battles`] reads a record's all). Each battle opens, grows and ends as the
/// module's documentation says.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Battles {
    /// The game's id, each battle's record's.
    game: u32,
    /// The battle's manifest of the game's ([`manifest`]).
    manifest: Manifest,
    /// The battles ended or left unended by the next one's start, in the order they opened.
    done: Vec<GameBattle>,
    /// The battle open now.
    open: Option<GameBattle>,
    /// The start a `GameEvents` told: the snapshot right after it opens its battle.
    pending: Option<Start>,
}

impl Battles {
    /// No battle read yet of the game whose record `header` heads.
    #[must_use]
    pub fn new(header: &GameRecordHeader) -> Self {
        Self {
            game: header.game,
            manifest: manifest(&header.manifest),
            done: Vec::new(),
            open: None,
            pending: None,
        }
    }

    /// How many battles opened so far: those done and the one open.
    #[must_use]
    pub fn opened(&self) -> usize {
        self.done.len() + usize::from(self.open.is_some())
    }

    /// The battle opened `index`-th (from 0, in the order they open), done or open.
    #[must_use]
    pub fn get(&self, index: usize) -> Option<&GameBattle> {
        self.done
            .get(index)
            .or_else(|| self.open.as_ref().filter(|_| index == self.done.len()))
    }

    /// The battle open now, if any.
    #[must_use]
    pub const fn open(&self) -> Option<&GameBattle> {
        self.open.as_ref()
    }

    /// Every battle read, in the order they opened; the last one still open where the messages
    /// end.
    #[must_use]
    pub fn finish(self) -> Vec<GameBattle> {
        let mut done = self.done;
        done.extend(self.open);
        done
    }

    /// What the player's next `message` adds to its battles.
    pub fn read(&mut self, message: &ServerMessage) {
        let start = self.pending.take();
        match message {
            ServerMessage::GameEvents { n, events } => self.events(*n, events),
            ServerMessage::Snapshot { n, observation } => match &mut self.open {
                Some(battle) => battle.push(|n| ServerMessage::Snapshot {
                    n,
                    observation: observation.clone(),
                }),
                None => {
                    let start = start.unwrap_or_default();
                    let mut battle =
                        GameBattle::open(self.game, &self.manifest, observation, &start, *n);
                    battle.tell(start.events);
                    match start.ended {
                        Some((n, result)) => {
                            battle.end(n, result);
                            self.done.push(battle);
                        }
                        None => self.open = Some(battle),
                    }
                }
            },
            ServerMessage::GameTurn {
                legal, castable, ..
            } => {
                let legal: Vec<_> = (legal.iter())
                    .filter_map(|c| match c {
                        GameCommand::Battle { command } => Some(*command),
                        _ => None,
                    })
                    .collect();
                if let Some(battle) = self.open.as_mut().filter(|_| !legal.is_empty()) {
                    battle.push(|n| ServerMessage::YourTurn {
                        n,
                        legal,
                        castable: castable.clone(),
                    });
                }
            }
            ServerMessage::GameAccepted {
                seq,
                command: GameCommand::Battle { command },
                ..
            } => {
                if let Some(battle) = &mut self.open {
                    battle.push(|n| ServerMessage::Accepted {
                        n,
                        seq: *seq,
                        command: *command,
                    });
                }
            }
            ServerMessage::GameRejected {
                seq,
                code: GameReject::Battle { code },
                ..
            } => {
                if let Some(battle) = &mut self.open {
                    battle.push(|n| ServerMessage::Rejected {
                        n,
                        seq: *seq,
                        code: *code,
                    });
                }
            }
            _ => {}
        }
    }

    /// The game's message `n`'s `events`: a battle's start, its events, its end.
    fn events(&mut self, n: u32, events: &[GameEvent]) {
        let mut told = Vec::new();
        for event in events {
            match event {
                GameEvent::BattleStarted { heroes, tile } => {
                    if let Some(battle) = self.open.take() {
                        self.done.push(battle);
                    }
                    self.pending = Some(Start {
                        heroes: *heroes,
                        tile: Some(*tile),
                        ..Start::default()
                    });
                }
                GameEvent::Battle { event } => match (&self.open, &mut self.pending) {
                    (Some(_), _) => told.push(event.clone()),
                    (None, Some(start)) => start.events.push(event.clone()),
                    (None, None) => {}
                },
                GameEvent::BattleEnded { result, .. } => {
                    if let Some(mut battle) = self.open.take() {
                        battle.tell(core::mem::take(&mut told));
                        battle.end(n, *result);
                        self.done.push(battle);
                    } else if let Some(start) = &mut self.pending {
                        start.ended = Some((n, *result));
                    }
                }
                _ => {}
            }
        }
        if let Some(battle) = &mut self.open {
            battle.tell(told);
        }
    }
}

impl GameBattle {
    /// The battle `observation` opens for the player of the game `game` at the game's message
    /// `opened`: its record's header and snapshot.
    fn open(
        game: u32,
        manifest: &Manifest,
        observation: &Observation,
        start: &Start,
        opened: u32,
    ) -> Self {
        let seat = match observation.viewer {
            _ if observation.spectator => Seat::Spectator,
            Side::Attacker => Seat::Attacker,
            Side::Defender => Seat::Defender,
        };
        let header = RecordHeader {
            format: RECORD_FORMAT_VERSION,
            game,
            seat,
            manifest: manifest.clone(),
            first: 1,
        };
        let snapshot = ServerMessage::Snapshot {
            n: 1,
            observation: observation.clone(),
        };
        Self {
            record: Record {
                header,
                messages: vec![snapshot],
            },
            at: InGame {
                heroes: start.heroes,
                tile: start.tile,
                opened,
                ended: None,
            },
        }
    }

    /// Appends the message `make` makes with the record's next number.
    fn push(&mut self, make: impl FnOnce(u32) -> ServerMessage) {
        let n = u32::try_from(self.record.messages.len()).map_or(u32::MAX, |len| len + 1);
        self.record.messages.push(make(n));
    }

    /// Appends the battle's `events` as one `Events` message, if there are any.
    fn tell(&mut self, events: Vec<Event>) {
        if !events.is_empty() {
            self.push(|n| ServerMessage::Events { n, events });
        }
    }

    /// The battle ended at the game's message `at` with `result`.
    fn end(&mut self, at: u32, result: Outcome) {
        self.push(|n| ServerMessage::GameEnded { n, result });
        self.at.ended = Some((at, result));
    }
}

#[cfg(test)]
mod tests;
