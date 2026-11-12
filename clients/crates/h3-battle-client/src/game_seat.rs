//! A whole game's battle played live through its host's game (E17 M7 T3b2; design §13.4): the
//! element's side of a host's whole game (an in-process `h3_protocol::session::game::GameSession`,
//! the map client's), as a seat on a server is the window's side of a server ([`crate::online`]).
//!
//! - **In:** the host hands the seat its player's messages as they come ([`GameSeat::receive`]),
//!   from the game's start or from the `GameEvents` whose `BattleStarted` begins the battle. The
//!   seat follows the last battle that opened in them (E17 M7 T4, R3-3: the battles done before
//!   it are not kept), its record growing as `h3_view::game_record` reads the game's messages (the
//!   battle's `Snapshot`, its events inside `GameEvents`, the player's turns in `GameTurn`,
//!   `GameAccepted`, `GameRejected`, until `BattleEnded`). Loaded (`Ground::load_game`, which takes
//!   the seat: one seat for each battle a host shows), it stays on that battle: past the message
//!   right after its end (its last snapshot, once the game sends it) it reads nothing more.
//! - **Out:** what the battle sends goes to the host ([`GameSeat::take_outgoing`]): its human
//!   side's commands, as the player's `GameCommand::Battle` numbered by the seat (the host plays
//!   each as that player's with its number, which the game's answer echoes), and each turn's
//!   queries of its casts' targets, which the host answers ([`GameSeat::answer`]: an in-process
//!   host from its battle's state, `h3_protocol::map::query`; the whole game's protocol has no
//!   query yet). A turn is played once every query it asked was answered, as a server's seat's.
//! - **Waiting:** none. The host's loop hands the messages over and takes what goes out between
//!   the element's passes; the battle takes what came at each pass ([`Remote::poll`]).
//!
//! An adapter beside the window's battle. *Provisional*: the names, the queries the host
//! answers, the seat's own numbers (from 1, its commands' and its queries' one sequence), the
//! battle followed (the last to open in what the host hands over until it is loaded, then that
//! one: E17 M7 T4, an agent decision; T3b2's followed the first).

use std::collections::BTreeMap;

use h3_protocol::wire::game::{GameCommand, GameRecordHeader};
use h3_protocol::wire::{Command, Query, Seat, ServerMessage, Target};
use h3_view::game_record::{Battles, GameBattle, InGame};
use h3_view::record::Record;

use crate::battle::Remote;

/// What the seat sends its host ([`GameSeat::take_outgoing`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToHost {
    /// The player's `command` for its battle's turn, numbered `seq`: the host plays it as the
    /// player's (`GameSession::play`), whose `GameAccepted` or `GameRejected` echoes the number.
    Command {
        /// The seat's number of it.
        seq: u32,
        /// The command (`GameCommand::Battle`).
        command: GameCommand,
    },
    /// The targets the game's rules allow the player's battle side for `query` now, which the
    /// host answers with [`GameSeat::answer`] and `seq`.
    Query {
        /// The seat's number of it.
        seq: u32,
        /// What is asked.
        query: Query,
    },
}

/// The element's side of a host's whole game (the module's text).
#[derive(Debug, Clone)]
pub struct GameSeat {
    /// The battles the player's messages show it, read as they came.
    battles: Battles,
    /// The seat's last number given (its commands' and queries' one sequence).
    seq: u32,
    /// What waits for the host.
    outgoing: Vec<ToHost>,
    /// A command was sent for the turn and not answered yet: its number.
    asked: Option<u32>,
    /// The turn the queries were asked for (the battle record's length then), and each query's
    /// spell (none: the active stack's own cast) with its number.
    queried: Option<usize>,
    queries: Vec<(Option<u16>, u32)>,
    /// The host's answers, by the query's number.
    answers: BTreeMap<u32, Vec<Target>>,
    /// The battle record's length at the last poll.
    polled: usize,
    /// The battle loaded ([`GameSeat::follow`]), by the order they opened from 0: the seat stays
    /// on it; none, the last opened.
    follows: Option<usize>,
    /// The message right after the battle loaded ended was read: nothing more is.
    closed: bool,
}

impl GameSeat {
    /// A seat in the game whose player's record `header` heads (its manifest: the battle's
    /// keys and packs), no message read yet.
    #[must_use]
    pub fn new(header: &GameRecordHeader) -> Self {
        Self {
            battles: Battles::new(header),
            seq: 0,
            outgoing: Vec::new(),
            asked: None,
            queried: None,
            queries: Vec::new(),
            answers: BTreeMap::new(),
            polled: 0,
            follows: None,
            closed: false,
        }
    }

    /// The player's next message from the host.
    pub fn receive(&mut self, message: &ServerMessage) {
        // Loaded, the seat reads its battle's messages up to the one right after its end (its
        // last snapshot), no later battle (E17 M7 T4, R3-3, *provisional*); else it keeps the
        // battle opened last alone.
        let read = match self.follows {
            Some(_) if self.over() => !core::mem::replace(&mut self.closed, true),
            _ => true,
        };
        if read {
            self.battles.read(message);
        }
        if self.follows.is_none() {
            self.battles.keep_last();
        }
        // The command answered (a refusal by another code than the battle's too: the record
        // holds none of it) or the battle over: the next turn may be played.
        let answered = match message {
            ServerMessage::GameAccepted { seq, .. } | ServerMessage::GameRejected { seq, .. } => {
                self.asked == Some(*seq)
            }
            _ => false,
        };
        if answered || self.over() {
            self.asked = None;
        }
    }

    /// Whether the battle followed opened: the host may load the battle then
    /// (`crate::battle::Ground::load_game`). A seat not loaded follows the battle opened last
    /// (E17 M7 T4, R3-3): fed on past that battle's end, it still has it until another opens.
    #[must_use]
    pub fn opened(&self) -> bool {
        self.followed().is_some()
    }

    /// Where the battle followed stands in the game (none before it opened).
    #[must_use]
    pub fn in_game(&self) -> Option<InGame> {
        self.followed().map(|b| b.at)
    }

    /// The battle the seat follows: the one loaded, else the one opened last.
    fn followed(&self) -> Option<&GameBattle> {
        match self.follows {
            Some(index) => self.battles.get(index),
            None => self.battles.last(),
        }
    }

    /// The battle followed now stays the seat's (E17 M7 T4, R3-3): `Ground::load_game` loads it.
    pub(crate) fn follow(&mut self) {
        self.follows = self.battles.opened().checked_sub(1);
    }

    /// What waits for the host, in the order the battle gave it.
    pub fn take_outgoing(&mut self) -> Vec<ToHost> {
        core::mem::take(&mut self.outgoing)
    }

    /// The host's answer to the query numbered `seq`: the `targets` the game's rules allow. An
    /// answer to no query of the turn's is dropped.
    pub fn answer(&mut self, seq: u32, targets: Vec<Target>) {
        if self.queries.iter().any(|&(_, s)| s == seq) {
            self.answers.insert(seq, targets);
        }
    }

    /// The seat's next number.
    fn next_seq(&mut self) -> u32 {
        self.seq = self.seq.wrapping_add(1);
        self.seq
    }

    /// Asks the host for the targets of the turn's casts once per turn: each spell the hero may
    /// cast, and the active stack's own cast (as a server's seat asks the server's rules).
    fn ask_targets(&mut self) {
        let Some(record) = self.record() else {
            return;
        };
        let Some((_, castable)) = h3_view::remote::turn(record) else {
            return;
        };
        let len = record.messages.len();
        let start = (record.messages.iter())
            .rposition(|m| matches!(m, ServerMessage::YourTurn { .. }))
            .unwrap_or(len);
        if self.queried.is_some_and(|q| q > start) {
            return;
        }
        let asks: Vec<_> = (castable.iter())
            .map(|&spell| (Some(spell), Query::CastTargets { spell }))
            .chain(core::iter::once((None, Query::CreatureCastTargets {})))
            .collect();
        self.queries.clear();
        self.answers.clear();
        for (spell, query) in asks {
            let seq = self.next_seq();
            self.outgoing.push(ToHost::Query { seq, query });
            self.queries.push((spell, seq));
        }
        self.queried = Some(len);
    }
}

impl Remote for GameSeat {
    /// Whether the battle's record grew since the last poll; the turn's queries go out then.
    fn poll(&mut self, _now: u64) -> Result<bool, String> {
        let len = self.record().map_or(0, |r| r.messages.len());
        let grew = len > self.polled;
        self.polled = len;
        self.ask_targets();
        Ok(grew)
    }

    fn record(&self) -> Option<&Record> {
        self.followed().map(|b| &b.record)
    }

    /// The battle's turn the game asks the player to play, once every target the turn asked for
    /// came, while no command is sent for it.
    fn turn(&self) -> Option<(&[Command], &[u16])> {
        if self.asked.is_some() || self.queried.is_none() {
            return None;
        }
        let answered = (self.queries.iter()).all(|(_, seq)| self.answers.contains_key(seq));
        h3_view::remote::turn(self.record()?).filter(|_| answered)
    }

    fn targets(&self, spell: Option<u16>) -> Option<&[Target]> {
        let &(_, seq) = self.queries.iter().find(|(s, _)| *s == spell)?;
        self.answers.get(&seq).map(Vec::as_slice)
    }

    fn send(&mut self, command: Command) -> Result<(), String> {
        if self.battles.open().is_none() {
            return Err("a command while no battle is open".into());
        }
        let seq = self.next_seq();
        self.outgoing.push(ToHost::Command {
            seq,
            command: GameCommand::Battle { command },
        });
        self.asked = Some(seq);
        Ok(())
    }

    fn seat(&self) -> Seat {
        self.record().map_or(Seat::Spectator, |r| r.header.seat)
    }

    fn over(&self) -> bool {
        self.record().is_some_and(|r| {
            r.messages
                .iter()
                .any(|m| matches!(m, ServerMessage::GameEnded { .. }))
        })
    }
}

#[cfg(test)]
mod tests;
