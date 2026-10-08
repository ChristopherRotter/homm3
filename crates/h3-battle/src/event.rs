//! Events: facts emitted while applying a command (ADR-0004 §5). The state, not the event log, is
//! the source of truth. Every event carries a visibility scope; [`crate::project`] filters events
//! per viewer.

use alloc::vec::Vec;

use h3_content::{CreatureTypeId, SpellId};
use h3_contracts::{BattleResult, BattleSide};

use crate::geometry::{Hex, HexSet};
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
    /// The Fire Shields of the stacks a melee attack struck dealt damage back to the striker
    /// (BAT-MAG-018).
    Reflected {
        /// The striker.
        stack: StackId,
        /// Damage dealt (capped at its remaining health when applied).
        damage: u64,
        /// Creatures killed.
        kills: u32,
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
    /// A blind stack's turn passes without an action (BAT-MAG-024).
    TurnPassed {
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
        /// Its second target (Teleport's destination, Sacrifice's victim), else none.
        second: CastTarget,
    },
    /// A stack cast a spell as its action (BAT-MAG-048; public): one of its casts is used.
    StackCast {
        /// The casting stack.
        stack: StackId,
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
    /// A Magic Mirror sent a spell aimed at one stack to another (BAT-MAG-022); the spell then
    /// goes on as if aimed at the new stack.
    SpellRedirected {
        /// The stack the spell was aimed at.
        from: StackId,
        /// The stack it went to.
        to: StackId,
    },
    /// A stack's ability cast a spell on the stack it struck in melee, as a creature's cast
    /// without a hero (BAT-MAG-040, BAT-MAG-041); the spell's own events follow.
    StrikeSpell {
        /// The striking stack.
        striker: StackId,
        /// The struck stack.
        target: StackId,
        /// The spell.
        spell: SpellId,
    },
    /// A stack's ability dealt extra damage to the stack it struck in melee, without a test (the
    /// Rust Dragon, BAT-MAG-047).
    StrikeDamage {
        /// The striking stack.
        striker: StackId,
        /// The struck stack.
        target: StackId,
        /// Damage dealt, before capping at the stack's remaining health.
        damage: u64,
        /// Creatures killed.
        kills: u32,
    },
    /// Poison took another tenth of a stack's hit points at a round start (BAT-MAG-044).
    Poisoned {
        /// The stack.
        stack: StackId,
        /// Its hit points per creature now.
        hit_points: u32,
    },
    /// A spell damaged a stack (BAT-MAG-004). The three amounts let a client show what the
    /// original's combat log shows: the caster's extra damage (`caster − value`) and the
    /// target's extra damage or reduction (`damage` against `caster`; batch 2a R5).
    SpellDamage {
        /// The stack.
        target: StackId,
        /// The spell's value for this stack: power × power factor + effect, halved at every
        /// jump of a chain.
        value: u64,
        /// The value after the caster's bonuses (the Orbs, Sorcery, the spell's own bonuses).
        caster: u64,
        /// Damage dealt, after the target's extra damage or reduction, before capping at the
        /// stack's remaining health.
        damage: u64,
        /// Creatures killed.
        kills: u32,
    },
    /// A spell healed a stack's top creature (Cure, BAT-MAG-017).
    SpellHealed {
        /// The stack.
        target: StackId,
        /// Hit points regained.
        hit_points: u32,
    },
    /// A timed spell took effect on a stack, or renewed its effect (BAT-MAG-009).
    EffectApplied {
        /// The stack.
        target: StackId,
        /// The spell.
        spell: SpellId,
        /// Rounds left, the current one included; `None` for the rest of the battle.
        rounds: Option<u16>,
    },
    /// A timed effect ended: its last round passed, a counter spell or a dispel ended it
    /// (BAT-MAG-009, BAT-MAG-010, BAT-MAG-016). Effects of a stack that dies end without an
    /// event.
    EffectEnded {
        /// The stack.
        target: StackId,
        /// The spell.
        spell: SpellId,
    },
    /// A spell placed an obstacle (BAT-MAG-027); a hidden one is reported only to the sides that
    /// see it (BAT-MAG-031).
    ObstaclePlaced {
        /// The side whose hero cast it.
        side: BattleSide,
        /// The spell.
        spell: SpellId,
        /// Its hexes.
        hexes: HexSet,
    },
    /// A spell obstacle went: its last round passed or a spell removed it (BAT-MAG-027,
    /// BAT-MAG-030); a hidden one is reported only to the sides that saw it (BAT-MAG-031).
    ObstacleRemoved {
        /// The spell that placed it.
        spell: SpellId,
        /// Its hexes.
        hexes: HexSet,
    },
    /// A spell removed a natural obstacle (BAT-MAG-034): its hexes are free (public).
    NaturalObstacleRemoved {
        /// Its anchor hex.
        anchor: Hex,
        /// The hexes it blocked.
        hexes: HexSet,
    },
    /// A hidden spell obstacle became visible to both sides: a stack that did not see it ran
    /// into it (Quicksand, BAT-MAG-031/032).
    ObstacleRevealed {
        /// The side whose hero cast it.
        side: BattleSide,
        /// The spell.
        spell: SpellId,
        /// Its hexes.
        hexes: HexSet,
    },
    /// A stack entered a hex of a spell obstacle that acts on it, or acted on one (Fire Wall,
    /// BAT-MAG-028); what it does follows (`SpellDamage`). A walker that entered a pit stops there
    /// (Quicksand, BAT-MAG-032).
    ObstacleTriggered {
        /// The stack.
        stack: StackId,
        /// The spell that placed the obstacle.
        spell: SpellId,
        /// The hex.
        hex: Hex,
    },
    /// A spell raised creatures of a stack, a dead one brought back with its hexes (Resurrection,
    /// Animate Dead, BAT-MAG-035; Sacrifice, BAT-MAG-036).
    SpellRaised {
        /// The stack.
        target: StackId,
        /// Hit points regained.
        hit_points: u64,
        /// Creatures brought back.
        creatures: u32,
        /// Whether they stay for this battle only (BAT-MAG-035).
        temporary: bool,
    },
    /// A stack was sacrificed: all its creatures died and it leaves no corpse (BAT-MAG-036).
    Sacrificed {
        /// The stack.
        victim: StackId,
        /// Its creatures.
        creatures: u32,
    },
    /// A spell created a stack (BAT-MAG-037): a clone of `original` (Clone, BAT-MAG-038) or a
    /// summoned one (Summon Elemental, BAT-MAG-039).
    StackCreated {
        /// The new stack (a dead created stack's id may be reused).
        stack: StackId,
        /// Its creature.
        creature: CreatureTypeId,
        /// Its creatures.
        count: u32,
        /// Its position.
        hex: Hex,
        /// The stack it copies, for a clone.
        original: Option<StackId>,
    },
    /// A clone vanished: its rounds ran out or its original died (BAT-MAG-038).
    Vanished {
        /// The clone.
        stack: StackId,
        /// Its creatures.
        creatures: u32,
    },
    /// A spell moved a stack (Teleport, BAT-MAG-029).
    Teleported {
        /// The stack.
        stack: StackId,
        /// Its position before.
        from: Hex,
        /// Its position now.
        to: Hex,
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
