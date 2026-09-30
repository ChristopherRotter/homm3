//! Events: facts emitted while applying a command (ADR-0004 §5). The state, not the event log, is
//! the source of truth. Every event carries a visibility scope; [`crate::project`] filters events
//! per viewer.

use alloc::vec::Vec;

use h3_contracts::BattleSide;

use crate::geometry::Hex;
use crate::state::StackId;

/// Who may see an event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Visibility {
    /// Both sides (and spectators).
    Public,
    /// Only one side.
    Side(BattleSide),
}

/// What happened.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventKind {
    /// A new round began (BAT-TRN-003).
    RoundStarted {
        /// The new round number.
        round: u16,
    },
    /// A stack's turn began.
    TurnStarted {
        /// The active stack.
        stack: StackId,
    },
    /// A stack moved (BAT-MOV-001).
    Moved {
        /// The stack.
        stack: StackId,
        /// Where it started.
        from: Hex,
        /// Where it ended.
        to: Hex,
    },
    /// A stack waited (BAT-TRN-002).
    Waited {
        /// The stack.
        stack: StackId,
    },
    /// A stack defended (BAT-TRN-004).
    Defended {
        /// The stack.
        stack: StackId,
    },
    /// A stack struck another (BAT-ATK-001, BAT-RET-001).
    Struck {
        /// The striking stack.
        attacker: StackId,
        /// The stack that was hit.
        target: StackId,
        /// Damage dealt (BAT-DMG-002).
        damage: u64,
        /// Creatures killed (BAT-DMG-003).
        kills: u32,
        /// Whether this strike is a retaliation.
        retaliation: bool,
    },
    /// The battle ended (BAT-END-001).
    BattleEnded {
        /// The winner.
        winner: BattleSide,
    },
}

/// An event with its visibility scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Event {
    /// Who may see it.
    pub visibility: Visibility,
    /// What happened.
    pub kind: EventKind,
}

impl Event {
    /// A public event.
    pub const fn public(kind: EventKind) -> Self {
        Self {
            visibility: Visibility::Public,
            kind,
        }
    }
}

/// Receiver of events. Simulation uses [`NoopSink`], which compiles to nothing.
pub trait Sink {
    /// Receives one event.
    fn emit(&mut self, event: Event);
}

/// A sink that drops every event.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoopSink;

impl Sink for NoopSink {
    #[inline]
    fn emit(&mut self, _event: Event) {}
}

impl Sink for Vec<Event> {
    fn emit(&mut self, event: Event) {
        self.push(event);
    }
}
