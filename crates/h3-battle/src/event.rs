//! Events: facts emitted while applying a command (ADR-0004 §5). The state, not the event log, is
//! the source of truth. Every event carries a visibility scope; [`crate::project`] filters events
//! per viewer.

use alloc::vec::Vec;

use h3_content::SpellId;
use h3_contracts::{BattleResult, BattleSide};

use crate::geometry::Hex;
use crate::magic::CastTarget;
use crate::state::StackId;

/// Who may see an event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Visibility {
    /// Both sides (and spectators).
    Public,
    /// Only one side.
    Side(BattleSide),
}

/// How a strike happened (BAT-ATK-001/003, BAT-RET-001, BAT-RNG-001/004, BAT-DMG-007,
/// BAT-LCK-002, BAT-WAR-006).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct StrikeFlags(u8);

impl StrikeFlags {
    /// A plain melee strike.
    pub const NONE: Self = Self(0);
    /// A retaliation.
    pub const RETALIATION: Self = Self(1);
    /// A ranged strike (a shot).
    pub const RANGED: Self = Self(1 << 1);
    /// A secondary target of a strike that hits several stacks (area shots, …).
    pub const SECONDARY: Self = Self(1 << 2);
    /// A death blow: +100 % base damage (BAT-DMG-007).
    pub const DEATH_BLOW: Self = Self(1 << 3);
    /// A lucky strike: +100 % base damage (BAT-LCK-002).
    pub const LUCKY: Self = Self(1 << 4);
    /// A double-damage shot of a war machine its hero commands (Artillery): +100 % base damage
    /// (BAT-WAR-006).
    pub const DOUBLE_DAMAGE: Self = Self(1 << 5);

    /// Whether all flags of `other` are set.
    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    /// The flags with `other` added.
    #[must_use]
    pub const fn with(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    /// The raw bits.
    pub const fn bits(self) -> u8 {
        self.0
    }
}

/// What happened.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventKind {
    /// The tactics phase began (round 0, BAT-TAC-001).
    TacticsStarted {
        /// The side that places its stacks.
        side: BattleSide,
        /// Width of its zone in columns.
        columns: u8,
    },
    /// The tactics phase ended; round 1 starts (BAT-TAC-002).
    TacticsEnded,
    /// A new round began (BAT-TRN-003).
    RoundStarted {
        /// The new round number.
        round: u16,
    },
    /// A stack's turn began (in the tactics phase: the stack to place, BAT-TAC-002).
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
    /// A stack struck another (BAT-ATK-001, BAT-RET-001, BAT-RNG-001).
    Struck {
        /// The striking stack.
        attacker: StackId,
        /// The stack that was hit.
        target: StackId,
        /// Damage dealt (BAT-DMG-002), before capping at the target's remaining health (VCMI
        /// reports the capped value; the hit points actually lost are at most the target's
        /// total health).
        damage: u64,
        /// Creatures killed (BAT-DMG-003).
        kills: u32,
        /// How the strike happened.
        flags: StrikeFlags,
    },
    /// A stack healed and resurrected itself from the damage it dealt (BAT-DMG-008).
    LifeDrained {
        /// The draining stack.
        stack: StackId,
        /// Hit points regained.
        hit_points: u64,
        /// Creatures brought back.
        resurrected: u32,
    },
    /// A death stare killed creatures outright (BAT-DMG-009).
    DeathStared {
        /// The staring stack.
        attacker: StackId,
        /// The stack that lost creatures.
        target: StackId,
        /// Creatures killed.
        kills: u32,
    },
    /// A stack's top creature regenerated (BAT-TRN-005).
    Regenerated {
        /// The stack.
        stack: StackId,
        /// Hit points regained.
        hit_points: u32,
    },
    /// A First Aid Tent healed a stack's top creature (BAT-WAR-003).
    Healed {
        /// The healing war machine.
        healer: StackId,
        /// The healed stack.
        target: StackId,
        /// Hit points regained.
        hit_points: u32,
    },
    /// Good morale: the stack that just acted gets another turn (BAT-MOR-003).
    GoodMorale {
        /// The stack.
        stack: StackId,
    },
    /// Bad morale: the stack freezes and loses its turn (BAT-MOR-004).
    BadMorale {
        /// The stack.
        stack: StackId,
    },
    /// A side's hero cast a spell (BAT-MAG-001; public, ADR-0014 point 8).
    SpellCast {
        /// The caster's side.
        side: BattleSide,
        /// The spell.
        spell: SpellId,
        /// What it was aimed at.
        target: CastTarget,
    },
    /// A stack resisted a spell (BAT-MAG-008); if it was the stack the spell was aimed at, the
    /// whole spell is resisted (BAT-MAG-007).
    SpellResisted {
        /// The stack.
        target: StackId,
    },
    /// A spell damaged a stack (BAT-MAG-004).
    SpellDamage {
        /// The stack.
        target: StackId,
        /// Damage dealt, before capping at the stack's remaining health.
        damage: u64,
        /// Creatures killed.
        kills: u32,
    },
    /// The battle ended (BAT-END-001/002).
    BattleEnded {
        /// A victory of one side, or a draw.
        result: BattleResult,
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
