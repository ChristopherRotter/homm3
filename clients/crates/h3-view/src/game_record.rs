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
#[derive(Debug, Default)]
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
        let manifest = manifest(&self.header.manifest);
        let mut done = Vec::new();
        let mut open: Option<GameBattle> = None;
        // A start is the snapshot's right after its `GameEvents` only.
        let mut pending: Option<Start> = None;
        for message in &self.messages {
            let start = pending.take();
            match message {
                ServerMessage::GameEvents { n, events } => {
                    let mut told = Vec::new();
                    for event in events {
                        match event {
                            GameEvent::BattleStarted { heroes, tile } => {
                                if let Some(battle) = open.take() {
                                    done.push(battle);
                                }
                                pending = Some(Start {
                                    heroes: *heroes,
                                    tile: Some(*tile),
                                    ..Start::default()
                                });
                            }
                            GameEvent::Battle { event } => match (&open, &mut pending) {
                                (Some(_), _) => told.push(event.clone()),
                                (None, Some(start)) => start.events.push(event.clone()),
                                (None, None) => {}
                            },
                            GameEvent::BattleEnded { result, .. } => {
                                if let Some(mut battle) = open.take() {
                                    battle.tell(core::mem::take(&mut told));
                                    battle.end(*n, *result);
                                    done.push(battle);
                                } else if let Some(start) = &mut pending {
                                    start.ended = Some((*n, *result));
                                }
                            }
                            _ => {}
                        }
                    }
                    if let Some(battle) = &mut open {
                        battle.tell(told);
                    }
                }
                ServerMessage::Snapshot { n, observation } => match &mut open {
                    Some(battle) => battle.push(|n| ServerMessage::Snapshot {
                        n,
                        observation: observation.clone(),
                    }),
                    None => {
                        let start = start.unwrap_or_default();
                        let mut battle = GameBattle::open(self, &manifest, observation, &start, *n);
                        battle.tell(start.events);
                        match start.ended {
                            Some((n, result)) => {
                                battle.end(n, result);
                                done.push(battle);
                            }
                            None => open = Some(battle),
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
                    if let Some(battle) = open.as_mut().filter(|_| !legal.is_empty()) {
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
                    if let Some(battle) = &mut open {
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
                    if let Some(battle) = &mut open {
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
        done.extend(open);
        done
    }
}

impl GameBattle {
    /// The battle `observation` opens for the player of `game` at the game's message `opened`:
    /// its record's header and snapshot.
    fn open(
        game: &GameRecord,
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
            game: game.header.game,
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
