//! The published language: every message on the wire, in both directions (ADR-0018 points 4–7;
//! the catalogue in `docs/epics/E16-design.md` §2).
//!
//! These are the protocol's own types, never the engine's: [`crate::map`] translates, with
//! exhaustive matches, so a new engine variant needs its wire variant here (ADR-0018 point 5,
//! amended 2026-10-10). Serde's
//! default, externally tagged enum form with struct variants and `deny_unknown_fields`; no
//! `tag`, `untagged` or `flatten`, so the same types also encode in a format that is not
//! self-describing. Integers only; a 64-bit value travels as a string ([`WireU64`]); content
//! is named by the ruleset's dense handles, whose keys the [`Manifest`] carries. Stacks and
//! hexes are the engine's indices (hex `x + y × width`, the width in [`Field`]).
//!
//! Provisional (E16 M1 review): the message schema inside ADR-0018.

use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// A 64-bit value on the wire: a decimal string, since JavaScript clients lose precision above
/// 2^53 (ADR-0018 point 4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct WireU64(pub u64);

impl Serialize for WireU64 {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for WireU64 {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        String::deserialize(deserializer)
            .and_then(|text| text.parse().map(Self).map_err(D::Error::custom))
    }
}

impl fmt::Display for WireU64 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// A side of the battle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Side {
    /// The attacker (the left side).
    Attacker,
    /// The defender (the right side).
    Defender,
}

/// What a connection asks to be in a game, and what it holds once joined.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Seat {
    /// The attacker's seat.
    Attacker,
    /// The defender's seat.
    Defender,
    /// A spectator: no seat, what is public to both sides (E16 M2 T4, the battle's spectator
    /// viewer, SRV-Q4).
    Spectator,
}

/// What a spell or a command is aimed at.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Target {
    /// Nothing: a spell without a target, or no second target.
    None {},
    /// A stack, by its id.
    Stack {
        /// The stack's id.
        stack: u8,
    },
    /// A hex, by its index.
    Hex {
        /// The hex's index.
        hex: u8,
    },
}

/// A battle command, in both directions: a client sends one, the server lists the legal ones and
/// echoes the one it applied. It names no side: the actor is the connection's seat (ADR-0018
/// point 6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Command {
    /// Walk to a hex.
    Move {
        /// The destination hex.
        to: u8,
    },
    /// Walk to `from` (or stay) and strike `target` in melee.
    Attack {
        /// The enemy stack.
        target: u8,
        /// The head position to strike from.
        from: u8,
    },
    /// Shoot at `target`.
    Shoot {
        /// The enemy stack.
        target: u8,
        /// The target's hex aimed at: its head, or (area shots) its tail.
        at: u8,
    },
    /// Postpone the turn to the wait phase.
    Wait {},
    /// End the turn with the Defend bonus.
    Defend {},
    /// Heal an allied stack with the commanded First Aid Tent.
    Heal {
        /// The allied stack.
        target: u8,
    },
    /// End the tactics phase.
    EndTactics {},
    /// Retreat from the battle.
    Retreat {},
    /// Surrender and pay the price.
    Surrender {},
    /// The hero casts a spell; the active stack's turn goes on.
    Cast {
        /// The spell's handle.
        spell: u16,
        /// What it is aimed at.
        target: Target,
        /// Its second target (Teleport's destination, Sacrifice's victim), else `None`.
        second: Target,
    },
    /// The active stack casts its own spell as its action.
    CreatureCast {
        /// What it is aimed at.
        target: Target,
    },
}

/// How a battle ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Outcome {
    /// The side won: the other has no living stacks.
    Victory {
        /// The winner.
        side: Side,
    },
    /// Neither side won.
    Draw {},
    /// The side retreated; the other wins.
    Retreat {
        /// The side that retreated.
        side: Side,
    },
    /// The side surrendered; the other wins.
    Surrender {
        /// The side that surrendered.
        side: Side,
    },
    /// The side's player stayed away beyond the ranked game's grace (E16 M4 T3a): the other wins,
    /// the battle left where it stood. Only in `GameEnded`.
    Forfeit {
        /// The side that forfeited.
        side: Side,
    },
    /// Both seats of the ranked game were empty when one's grace ran out (E16 M4 T3a): the game
    /// is void, a no-show each. Only in `GameEnded`.
    NoShow {},
}

/// Why a command was rejected: the engine's errors one to one, then the session's own codes.
/// The text is the client's. A rejected command changed nothing.
#[allow(missing_docs)] // the engine's error names; their meaning is in `h3_battle::CommandError`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RejectCode {
    BattleOver,
    NotYourTurn,
    AlreadyWaited,
    InvalidHex,
    Unreachable,
    NoSuchTarget,
    NotAnEnemy,
    NotAdjacent,
    MustShoot,
    CannotShoot,
    NotOnTarget,
    NoMelee,
    CannotHeal,
    NotAPatient,
    InTacticsPhase,
    NoTacticsPhase,
    OutsideZone,
    CannotRetreat,
    CannotSurrender,
    CannotAffordSurrender,
    NotInSpellBook,
    NotACombatSpell,
    AlreadyCast,
    NotCastable,
    NotEnoughMana,
    InvalidSpellTarget,
    CannotCast,
    CastingForbidden,
    SpellLevelBarred,
    /// The connection holds no seat that may act (a spectator, or not joined).
    NotSeated,
    /// The game has not started (a seat is still empty).
    NotStarted,
    /// The game is not over (the replay comes after it).
    NotOver,
    /// The game's journal did not re-run to the game as played: its replay is withheld (E16 M2
    /// T3).
    Unverified,
    /// Only the game's host may do this (E16 M2 T4c); a ranked game has none (E16 M4 T3a).
    NotHost,
    /// The game has started: its configuration is fixed (E16 M2 T4c).
    AlreadyStarted,
}

/// The flags of a strike.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Strike {
    /// A retaliation.
    pub retaliation: bool,
    /// A shot.
    pub ranged: bool,
    /// A hit beside the main target (a breath, a cloud, a cleave).
    pub secondary: bool,
    /// The Death Blow doubled it.
    pub death_blow: bool,
    /// Good luck raised it.
    pub lucky: bool,
    /// An ability doubled it.
    pub double_damage: bool,
}

/// What a spell's battlefield obstacle does.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum ObstacleKind {
    /// Blocks its hexes (Force Field).
    Wall {},
    /// Burns a stack in its hexes (Fire Wall).
    Fire {},
    /// Stops a walker; hidden from the other side (Quicksand).
    Pit {},
    /// Goes off under an enemy; hidden from the other side (Land Mine).
    Mine {},
}

/// One battle event, already projected for its viewer (ADR-0018 point 7). The fields are the
/// engine's (`h3_battle::EventKind`), with stacks and hexes as indices and content as handles.
#[allow(missing_docs)] // the variants and fields are documented on `h3_battle::EventKind`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Event {
    TacticsStarted {
        side: Side,
        columns: u8,
    },
    TacticsEnded {},
    RoundStarted {
        round: u16,
    },
    TurnStarted {
        stack: u8,
    },
    Moved {
        stack: u8,
        from: u8,
        to: u8,
    },
    Waited {
        stack: u8,
    },
    Defended {
        stack: u8,
    },
    Struck {
        attacker: u8,
        target: u8,
        damage: WireU64,
        kills: u32,
        flags: Strike,
    },
    Reflected {
        stack: u8,
        damage: WireU64,
        kills: u32,
    },
    LifeDrained {
        stack: u8,
        hit_points: WireU64,
        resurrected: u32,
    },
    DeathStared {
        attacker: u8,
        target: u8,
        kills: u32,
    },
    Regenerated {
        stack: u8,
        hit_points: u32,
    },
    Healed {
        healer: u8,
        target: u8,
        hit_points: u32,
    },
    GoodMorale {
        stack: u8,
    },
    BadMorale {
        stack: u8,
    },
    Frightened {
        stack: u8,
    },
    TurnPassed {
        stack: u8,
    },
    SpellCast {
        side: Side,
        spell: u16,
        target: Target,
        second: Target,
    },
    StackCast {
        stack: u8,
        spell: u16,
        target: Target,
    },
    PeriodicCast {
        stack: u8,
        spell: u16,
    },
    OpeningSpell {
        side: Side,
        spell: u16,
    },
    ManaChanneled {
        side: Side,
        mana: u16,
    },
    ManaDrained {
        stack: u8,
        side: Side,
        mana: u16,
    },
    CreatureRaise {
        stack: u8,
        target: u8,
    },
    RoseFromCorpse {
        stack: u8,
        corpse: u8,
        creature: u16,
        count: u32,
        hex: u8,
    },
    SpellDrawn {
        stack: u8,
        spell: u16,
    },
    SpellResisted {
        target: u8,
    },
    SpellRedirected {
        from: u8,
        to: u8,
    },
    StrikeSpell {
        striker: u8,
        target: u8,
        spell: u16,
    },
    StrikeDamage {
        striker: u8,
        target: u8,
        damage: WireU64,
        kills: u32,
    },
    Poisoned {
        stack: u8,
        hit_points: u32,
    },
    SpellDamage {
        target: u8,
        value: WireU64,
        caster: WireU64,
        damage: WireU64,
        kills: u32,
    },
    SpellHealed {
        target: u8,
        hit_points: u32,
    },
    EffectApplied {
        target: u8,
        spell: u16,
        rounds: Option<u16>,
    },
    EffectEnded {
        target: u8,
        spell: u16,
    },
    ObstaclePlaced {
        side: Side,
        spell: u16,
        hexes: Vec<u8>,
    },
    ObstacleRemoved {
        spell: u16,
        hexes: Vec<u8>,
    },
    NaturalObstacleRemoved {
        anchor: u8,
        hexes: Vec<u8>,
    },
    ObstacleRevealed {
        side: Side,
        spell: u16,
        hexes: Vec<u8>,
    },
    ObstacleTriggered {
        stack: u8,
        spell: u16,
        hex: u8,
    },
    SpellRaised {
        target: u8,
        hit_points: WireU64,
        creatures: u32,
        temporary: bool,
    },
    Sacrificed {
        victim: u8,
        creatures: u32,
    },
    Reborn {
        stack: u8,
        creatures: u32,
    },
    StackCreated {
        stack: u8,
        creature: u16,
        count: u32,
        hex: u8,
        original: Option<u8>,
    },
    TurnedSummoned {
        stack: u8,
        rounds: u16,
    },
    Vanished {
        stack: u8,
        creatures: u32,
    },
    Teleported {
        stack: u8,
        from: u8,
        to: u8,
    },
    BattleEnded {
        result: Outcome,
    },
}

/// How a stack's damage rolls under Bless or Curse.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum DamageRoll {
    /// Rolled at random.
    Random {},
    /// Blessed: the maximum, raised by `value` percent.
    Maximum {
        /// The blessing's value.
        value: u16,
    },
    /// Cursed: the minimum, lowered by `value` percent.
    Minimum {
        /// The curse's value.
        value: u16,
    },
}

/// A timed effect on a stack. What the stack's effects change together (its protections, its
/// status, …) follows from its effects and the spells' data (BAT-MAG-011); the values a client
/// shows without the rules are on the stack.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Effect {
    /// The spell's handle.
    pub spell: u16,
    /// Rounds left; `None` for the rest of the battle.
    pub rounds: Option<u16>,
    /// The first cast's mastery (0 none to 3 expert).
    pub mastery: u8,
    /// The spell's effect at that mastery.
    pub value: u16,
    /// What it added to the stack's attack (negative: took from it).
    pub attack: i16,
    /// What it added to the stack's defense.
    pub defense: i16,
    /// What it added to the stack's speed.
    pub speed: i16,
}

/// One stack as its viewer sees it (the public stack values; the sim's observation encoding in
/// named fields).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Stack {
    /// The stack's id (its index).
    pub id: u8,
    /// Its owner.
    pub side: Side,
    /// Who commands it now (the other side while it is hypnotized).
    pub controller: Side,
    /// Its army slot; 255 for a stack created in battle.
    pub slot: u8,
    /// Its creature's handle.
    pub creature: u16,
    /// Its head hex.
    pub hex: u8,
    /// Whether it has creatures left.
    pub alive: bool,
    /// Creatures in it.
    pub count: u32,
    /// Creatures at the battle's start, or when the battle created it (the resurrection limit).
    pub base_count: u32,
    /// The top creature's hit points.
    pub top_hp: u32,
    /// A creature's full hit points.
    pub max_hp: u32,
    /// Attack now, with the timed effects (without the melee or ranged bonus).
    pub attack: i32,
    /// Defense now, with the timed effects.
    pub defense: i32,
    /// The damage range's low end.
    pub damage_min: u16,
    /// The damage range's high end.
    pub damage_max: u16,
    /// Speed now, with the timed effects.
    pub speed: u8,
    /// Morale, before what its timed effects change (Mirth, Sorrow: in `effects`).
    pub morale: i8,
    /// Luck, before what its timed effects change (Fortune, Misfortune: in `effects`).
    pub luck: i8,
    /// Shots left.
    pub shots: u16,
    /// Retaliations per round (255: unlimited).
    pub retaliations: u8,
    /// Retaliations left this round (255: unlimited).
    pub retaliations_left: u8,
    /// Hit points regenerated at its turn's start.
    pub regeneration: u16,
    /// It has acted this round.
    pub acted: bool,
    /// It has waited this round.
    pub waited: bool,
    /// It has retaliated this round.
    pub retaliated: bool,
    /// It defends.
    pub defending: bool,
    /// It had good morale this round.
    pub had_morale: bool,
    /// Its turn has started this round.
    pub started: bool,
    /// It takes two hexes.
    pub two_hex: bool,
    /// It flies.
    pub flies: bool,
    /// It shoots.
    pub shooter: bool,
    /// It is a war machine.
    pub war_machine: bool,
    /// Its shots have no distance penalty.
    pub no_distance_penalty: bool,
    /// It shoots while an enemy stands next to it.
    pub shoots_when_adjacent: bool,
    /// It fights on its native terrain.
    pub native: bool,
    /// It was created in the battle.
    pub created: bool,
    /// It is a clone.
    pub clone: bool,
    /// It was raised from a corpse in the battle (a Pit Lord's Demons).
    pub raised: bool,
    /// A clone's rounds left (0: not a clone).
    pub clone_rounds: u16,
    /// For a clone its original, for a stack with a living clone that clone; a crowded original
    /// that became its own clone links to itself (quirk BAT-QUIRK-7).
    pub clone_link: Option<u8>,
    /// Its creatures raised for this battle only.
    pub temporary: u32,
    /// Its corpse: 0 while it lives or until its death is marked, then the order of its death
    /// among the battle's corpses; 65 535 if it left none.
    pub corpse: u16,
    /// The stacks whose strike bound it, by id, ascending.
    pub bound_by: Vec<u8>,
    /// The tenths of its hit points Poison took for good.
    pub poison: u8,
    /// A creature's hit points at the battle's start if Poison or Age changed them (0: unchanged).
    pub start_hp: u32,
    /// Its casts left.
    pub casts_left: u8,
    /// How many spells the active stack's cast could draw for it (its viewer's helpful caster).
    pub cast_candidates: u8,
    /// The melee attack bonus of its timed effects.
    pub melee_attack_bonus: u8,
    /// The ranged attack bonus of its timed effects.
    pub ranged_attack_bonus: u8,
    /// How its damage rolls (Bless, Curse).
    pub damage_roll: DamageRoll,
    /// Its speed share in percent (100: not slowed).
    pub speed_percent: u8,
    /// Its timed effects, by spell handle.
    pub effects: Vec<Effect>,
}

/// The public part of a hero leading a side.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Hero {
    /// The hero type's handle.
    pub hero_type: u16,
    /// Attack skill.
    pub attack: u8,
    /// Defense skill.
    pub defense: u8,
    /// Spell power.
    pub power: u8,
    /// Knowledge.
    pub knowledge: u8,
    /// Mana now.
    pub mana: u16,
    /// Mana at most.
    pub max_mana: u16,
    /// Casts this round.
    pub casts: u8,
    /// It commands its Ballista.
    pub commands_ballista: bool,
    /// It commands its First Aid Tent.
    pub commands_tent: bool,
    /// It carries a spell book.
    pub spell_book: bool,
}

/// The commanded Ballista.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Ballista {
    /// Shots per turn.
    pub shots: u8,
    /// Its double-damage chance in basis points.
    pub double_damage_chance_bp: u16,
}

/// A spell in the hero's book.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BookSpell {
    /// The spell's handle.
    pub spell: u16,
    /// The hero's mastery of it (0 none to 3 expert).
    pub mastery: u8,
}

/// A single spell's damage bonus of a hero (a specialty).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpellBonus {
    /// The spell's handle.
    pub spell: u16,
    /// More damage from the spell, in basis points.
    pub damage_bp: u32,
    /// A specialty's growth per hero level, in basis points.
    pub per_level_bp: u32,
    /// The hero's level for the specialty.
    pub level: u8,
}

/// What a hero adds to the immunity of the stacks it controls.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Defences {
    /// The spells they are never affected by, by handle.
    pub spells: Vec<u16>,
    /// The highest level of the spells they are never affected by (Dispel excepted).
    pub up_to_level: Option<u8>,
    /// They are immune to mind spells.
    pub mind: bool,
}

/// A spell a hero casts at its side's first activation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OpeningSpell {
    /// The spell's handle.
    pub spell: u16,
    /// Its mastery (0 none to 3 expert).
    pub mastery: u8,
    /// Its power, the duration of its effects.
    pub power: u16,
}

/// The private part of the viewer's own hero (ADR-0013 point 9), its hidden magic among it
/// (BAT-Q26).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OwnHero {
    /// Tactics level.
    pub tactics: u8,
    /// Ranged damage bonus in basis points.
    pub ranged_bonus_bp: u32,
    /// Melee damage bonus in basis points.
    pub melee_bonus_bp: u32,
    /// Damage reduction in basis points.
    pub damage_reduction_bp: u32,
    /// The commanded Ballista, if any.
    pub ballista: Option<Ballista>,
    /// The commanded tent's heal bonus in basis points, if it commands one.
    pub tent_heal_bonus_bp: Option<u32>,
    /// The surrender discount in basis points.
    pub surrender_discount_bp: u16,
    /// The gold it can pay.
    pub funds: u32,
    /// The price of surrendering now, if the side may.
    pub surrender_price: Option<u32>,
    /// Its spell book with the masteries, by spell handle.
    pub spell_book: Vec<BookSpell>,
    /// More damage from every spell, in basis points.
    pub spell_damage_bp: u32,
    /// More damage from the spells of each school, in basis points, by school handle.
    pub school_damage_bp: Vec<u32>,
    /// The damage bonuses of single spells.
    pub spell_bonuses: Vec<SpellBonus>,
    /// The resistance of its side's stacks to hostile spells, in basis points.
    pub magic_resistance_bp: u32,
    /// Rounds added to the duration of its timed spells.
    pub duration_bonus: u16,
    /// It forbids every cast in the battle.
    pub no_casting: bool,
    /// The highest spell level it allows in the battle, if it limits them.
    pub level_limit: Option<u8>,
    /// What it adds to the immunity of the stacks it controls.
    pub defences: Defences,
    /// It negates immunities and resistance.
    pub negates: bool,
    /// Its opening spells, in their order.
    pub opening: Vec<OpeningSpell>,
    /// Its side's first activation, when it casts them, is still to come.
    pub opening_due: bool,
    /// Its Eagle Eye's chance to pick a spell at a cast of the enemy hero, in percent.
    pub eagle_chance: u8,
    /// The spells its Eagle Eye may pick, by handle.
    pub eagle_spells: Vec<u16>,
    /// The spells its Eagle Eye picked so far, by handle.
    pub learned: Vec<u16>,
}

/// One side's public state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SideView {
    /// It may retreat.
    pub may_retreat: bool,
    /// It may surrender at all (whether the viewer can pay is in [`OwnHero`]).
    pub may_surrender: bool,
    /// Round starts since its periodic casters cast (at most 255).
    pub periodic_rounds: u8,
    /// The creature's handle its Summon Elemental casts are bound to, once it cast one.
    pub summoned: Option<u16>,
    /// The change of its hero's spell costs from the creatures in the armies (0 without a hero).
    pub cost_change: i8,
    /// Its hero's public part, if a hero leads it.
    pub hero: Option<Hero>,
}

/// A running tactics phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Tactics {
    /// The side that places its stacks.
    pub side: Side,
    /// Its zone in columns.
    pub columns: u8,
}

/// A natural obstacle a spell can remove.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RemovableObstacle {
    /// Its anchor hex.
    pub anchor: u8,
    /// The hexes it blocks.
    pub hexes: Vec<u8>,
}

/// A spell's obstacle the viewer sees.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpellObstacle {
    /// The spell's handle.
    pub spell: u16,
    /// What it does.
    pub kind: ObstacleKind,
    /// The side that placed it.
    pub side: Side,
    /// Its hexes.
    pub hexes: Vec<u8>,
    /// Rounds left.
    pub rounds: u8,
    /// Its damage, for the burning and exploding kinds.
    pub value: WireU64,
    /// Whether the other side sees it.
    pub revealed: bool,
}

/// The battle as one viewer, a side or a spectator, sees it (`h3_battle::observe_as`, ADR-0018
/// point 7).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Observation {
    /// The side this observation is for; the attacker for a spectator's.
    pub viewer: Side,
    /// A spectator's observation (E16 M2 T4): what is public to both sides, no side's private
    /// part.
    pub spectator: bool,
    /// The round (0 in the tactics phase).
    pub round: u16,
    /// The running tactics phase.
    pub tactics: Option<Tactics>,
    /// The round's wait phase has begun: only stacks that waited are left to act.
    pub wait_phase: bool,
    /// The stack whose turn it is.
    pub active: Option<u8>,
    /// The side of the last stack that took a turn.
    pub last_side_acted: Option<Side>,
    /// How the battle ended, once it is over.
    pub result: Option<Outcome>,
    /// The spell the active stack drew, if it is a random caster the viewer commands.
    pub drawn: Option<u16>,
    /// The battlefield's terrain handle.
    pub terrain: Option<u16>,
    /// The attacker's and the defender's public state.
    pub sides: [SideView; 2],
    /// The private part of the viewer's own hero.
    pub own_hero: Option<OwnHero>,
    /// Every stack, by id.
    pub stacks: Vec<Stack>,
    /// The hexes natural obstacles block.
    pub obstacles: Vec<u8>,
    /// The anchor hexes that belong to their obstacles.
    pub obstacle_anchors: Vec<u8>,
    /// The natural obstacles a spell can remove.
    pub removable_obstacles: Vec<RemovableObstacle>,
    /// The hexes a removal of a natural obstacle cleared: they lead to no obstacle any more.
    pub obstacles_cleared: Vec<u8>,
    /// The spell obstacles the viewer sees, in the order they were placed.
    pub spell_obstacles: Vec<SpellObstacle>,
}

/// A content pack the game is made of.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PackRef {
    /// The pack's id.
    pub id: String,
    /// The pack's version.
    pub version: String,
}

/// An algorithm and its version.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Algorithm {
    /// Its id.
    pub name: String,
    /// Its version.
    pub version: u32,
}

/// The battlefield's size.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Field {
    /// Hexes per row.
    pub width: u8,
    /// Rows.
    pub height: u8,
}

/// The content keys by handle: entry `i` is the key of handle `i` (`base:pikeman`), so clients
/// find assets by key.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KeyTables {
    /// Creature types.
    pub creatures: Vec<String>,
    /// Spells.
    pub spells: Vec<String>,
    /// Secondary skills.
    pub skills: Vec<String>,
    /// Hero types.
    pub hero_types: Vec<String>,
    /// Artifact types.
    pub artifacts: Vec<String>,
    /// Factions.
    pub factions: Vec<String>,
    /// Terrains.
    pub terrains: Vec<String>,
}

/// What a game is made of, sent at join (ADR-0018 point 10): the replay header's fields, the
/// field's size and the key tables. A client previews with its own ruleset only when its hash
/// equals `ruleset_hash`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    /// [`crate::PROTOCOL_VERSION`].
    pub protocol: u32,
    /// The engine's version (informational).
    pub engine: String,
    /// The packs, in load order.
    pub packs: Vec<PackRef>,
    /// The profile's key.
    pub profile: String,
    /// The compiled ruleset's content hash.
    pub ruleset_hash: WireU64,
    /// The random number generator.
    pub rng: Algorithm,
    /// The state hash.
    pub hash: Algorithm,
    /// The battle state hash's encoding version.
    pub battle_state_hash_version: u32,
    /// The battlefield's size.
    pub field: Field,
    /// The content keys by handle.
    pub keys: KeyTables,
}

/// A game configuration the server offers (E16 M2 T1): what a game created from it is made of,
/// shown before anyone joins. The scenario is the server's; a client names the offer by `id`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Offer {
    /// The offer's id, as `CreateGame` names it.
    pub id: String,
    /// The scenario's id.
    pub scenario: String,
    /// The packs, in load order.
    pub packs: Vec<PackRef>,
    /// The profile's key.
    pub profile: String,
    /// The compiled ruleset's content hash (the manifest's).
    pub ruleset_hash: WireU64,
    /// The seats the server's bot plays.
    pub bots: Vec<Seat>,
}

/// Where a game is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Phase {
    /// Not every seat was held at once yet.
    Waiting,
    /// Started and not over.
    Running,
    /// Over.
    Over,
    /// Paused (E16 M2 T3): a seat's player resumes it by joining with the seat's token, or as the
    /// seat's account (E16 M3 T3).
    Paused,
}

/// A seat that belongs to an account, as the lobby shows it (E16 M3 T3).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Seated {
    /// The seat.
    pub seat: Seat,
    /// The account's display name.
    pub display: String,
}

/// One hosted game in the lobby's list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GameInfo {
    /// The game's id, as `Join` names it.
    pub game: u32,
    /// The offer it was created from.
    pub offer: String,
    /// Where it is.
    pub phase: Phase,
    /// The seats a newcomer may take without a token.
    pub open: Vec<Seat>,
    /// Whether it takes spectators (E16 M2 T4c): its host's setting, on a server that takes them
    /// at all; a game watched by as many as the server allows refuses more.
    pub spectators: bool,
    /// The seats that belong to accounts, with their display names (E16 M3 T3); a seat an
    /// anonymous player or a bot holds is not named.
    pub seated: Vec<Seated>,
    /// The ladder whose matchmaking created it (E16 M4 T2); `None` for a game a client created.
    pub ladder: Option<String>,
}

/// A ladder as `ListLadders` shows it (E16 M4 T2; ADR-0019 point 4).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LadderInfo {
    /// The ladder's id, as `Queue` names it.
    pub id: String,
    /// Its name.
    pub name: String,
    /// Its current season.
    pub season: u32,
    /// How many players wait in its queue, matched or not.
    pub waiting: u32,
}

/// Why a player is no longer in a ladder's queue without having asked (E16 M4 T2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnqueueReason {
    /// The player did not accept its match in time.
    Expired,
    /// The server drains (E16 M2 T5a): it makes no new match.
    Draining,
    /// Both accepted, but the server has as many open games as it allows.
    Full,
}

/// A question about the battle that only the server's rules answer (ADR-0018 point 10).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Query {
    /// The targets the seat's hero may cast `spell` at now.
    CastTargets {
        /// The spell's handle.
        spell: u16,
    },
    /// The second targets of `spell` cast at `target` (Teleport, Sacrifice).
    SecondTargets {
        /// The spell's handle.
        spell: u16,
        /// The first target.
        target: Target,
    },
}

/// A message from a client.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum ClientMessage {
    /// The first message: the client's protocol version and names.
    Hello {
        /// [`crate::PROTOCOL_VERSION`] of the client.
        protocol: u32,
        /// The client program's name (informational).
        client: String,
        /// The player's display name.
        name: String,
    },
    /// The server's offers and games (E16 M2 T1).
    ListGames {
        /// The client's sequence number.
        seq: u32,
    },
    /// Create a game of one of the server's offers and take `seat` in it (E16 M2 T1); the
    /// answer is `GameCreated`, then the game's `Joined`, or `Refused`.
    CreateGame {
        /// The client's sequence number.
        seq: u32,
        /// The offer's id.
        offer: String,
        /// The seat the creator takes.
        seat: Seat,
        /// Whether the game takes spectators (E16 M2 T4c); `None`, or left out: it does. Its host
        /// may change it until the game starts (`SetSpectators`).
        #[serde(default)]
        spectators: Option<bool>,
    },
    /// Take a seat in a game, or re-take it after a disconnect: with its token, or as the account
    /// it belongs to (E16 M3 T3: a seat a logged-in connection took is its account's).
    Join {
        /// The game.
        game: u32,
        /// The seat asked for.
        seat: Seat,
        /// The seat's token, to re-take a seat that belongs to no account.
        token: Option<String>,
    },
    /// A battle command; `seq` is the client's own counter, echoed in the answer.
    Command {
        /// The client's sequence number.
        seq: u32,
        /// The command.
        command: Command,
    },
    /// A question for the server's rules.
    Query {
        /// The client's sequence number.
        seq: u32,
        /// The question.
        query: Query,
    },
    /// Ask for the replay (participants, after the game).
    FetchReplay {
        /// The client's sequence number.
        seq: u32,
    },
    /// Ask for this viewer's record.
    FetchRecord {
        /// The client's sequence number.
        seq: u32,
    },
    /// Leave the game and close the connection.
    Leave {},
    /// Keep the connection alive.
    Ping {
        /// Echoed in the pong.
        nonce: u32,
    },
    /// Ask to pause the running game (E16 M2 T3): the host's ask (the seat its creator took)
    /// pauses it, another seat's is told to every viewer (`PauseAsked`).
    Pause {
        /// The client's sequence number.
        seq: u32,
    },
    /// Open the game to spectators, or close it (E16 M2 T4c): the host's, until the game starts;
    /// closing it ends the spectators' watch. The answer is `SpectatorsSet`, or `Rejected`.
    SetSpectators {
        /// The client's sequence number.
        seq: u32,
        /// Whether the game takes spectators.
        open: bool,
    },
    /// List the server's archive of finished games (E16 M2 T5d, SRV-Q15): a page of its replays,
    /// newest first, from the `from`-th; answered `Replays`. Needs no game.
    ListReplays {
        /// The client's sequence number.
        seq: u32,
        /// How many of the newest to skip.
        from: u32,
    },
    /// Download a replay of the archive by its id (E16 M2 T5d): answered `Replay` (numbered 0), or
    /// `Refused { NoSuchReplay }`. Needs no game.
    FetchArchived {
        /// The client's sequence number.
        seq: u32,
        /// The replay's id, as `Replays` lists it.
        replay: String,
    },
    /// Log in to an account, or register one with a display name (E16 M3 T1; ADR-0019 point 1):
    /// the account's public key and its signature over [`crate::account::login_message`] (this
    /// server's id and this connection's challenge from the welcome). Answered `Account`, or
    /// `Refused`.
    Login {
        /// The client's sequence number.
        seq: u32,
        /// The login name ([`crate::account::is_login_name`]): it salted the key's derivation.
        login: String,
        /// The public key, 32 bytes as lowercase hex.
        key: String,
        /// The signature, 64 bytes as lowercase hex.
        signature: String,
        /// The display name to register the account with; `None` to log in to it.
        display: Option<String>,
    },
    /// Re-key the logged-in account after a password change (E16 M3 T1): the new public key,
    /// and the signatures of the old key and of the new over
    /// [`crate::account::rekey_message`]. Answered `Account`, or `Refused`.
    Rekey {
        /// The client's sequence number.
        seq: u32,
        /// The new public key, 32 bytes as lowercase hex.
        key: String,
        /// The signature by the account's key so far, 64 bytes as lowercase hex.
        by_old: String,
        /// The signature by the new key, 64 bytes as lowercase hex.
        by_new: String,
    },
    /// Change the logged-in account's display name on this server (E16 M3 T1). Answered
    /// `Account`, or `Refused`.
    Rename {
        /// The client's sequence number.
        seq: u32,
        /// The new display name ([`crate::account::is_display_name`]).
        display: String,
    },
    /// Whether the replays of the logged-in account's finished games may be listed in the
    /// archive (E16 M3 T1; ADR-0019 point 2). Answered `Account`.
    SetListed {
        /// The client's sequence number.
        seq: u32,
        /// Whether they may.
        listed: bool,
    },
    /// The server's ladders (E16 M4 T2): answered `Ladders`. Needs no game.
    ListLadders {
        /// The client's sequence number.
        seq: u32,
    },
    /// Wait in a ladder's queue for a match, as the logged-in account (E16 M4 T2; ADR-0019
    /// point 5): answered `Queued`, or `Refused`. An account waits in one queue at a time.
    Queue {
        /// The client's sequence number.
        seq: u32,
        /// The ladder's id.
        ladder: String,
    },
    /// Leave the queue, or decline the match found: answered `Dequeued`, or `Refused`.
    Dequeue {
        /// The client's sequence number.
        seq: u32,
    },
    /// Accept the match found (`MatchFound`): once the other player accepts too, both hear
    /// `MatchReady`; until then nothing answers. `Refused { NoMatch }` without a match.
    AcceptMatch {
        /// The client's sequence number.
        seq: u32,
    },
    /// Decline the match found: answered `Dequeued` (the player leaves the queue), or `Refused`.
    DeclineMatch {
        /// The client's sequence number.
        seq: u32,
    },
}

/// Why a join, a game's creation or an account's request was refused; the connection stays open.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Refusal {
    /// No game has this id.
    NoSuchGame,
    /// The seat is taken: a bot's, one taken before (re-taken with its token), or one that belongs
    /// to another account (E16 M3 T3).
    SeatTaken,
    /// The token does not belong to the seat.
    BadToken,
    /// The connection is in a game already.
    AlreadyJoined,
    /// The game takes no more spectators (its limit), or none here: a game's creation, a paused
    /// game.
    NoSpectators,
    /// The server offers nothing under this id.
    NoSuchOffer,
    /// The server, or the client's address, has as many open games as it allows.
    TooManyGames,
    /// The server is draining (E16 M2 T5a): it takes no new game while the open ones finish.
    Draining,
    /// The archive holds no replay under this id (E16 M2 T5d), or no more.
    NoSuchReplay,
    /// The login does not hold (E16 M3 T1): a key or a signature that does not read or verify,
    /// or a login name that belongs to another key; all alike.
    BadLogin,
    /// A login name or a display name breaks its rules (E16 M3 T1).
    BadName,
    /// Another account has the display name on this server (E16 M3 T1).
    NameTaken,
    /// The request needs a logged-in account (E16 M3 T1), and so does joining or creating a game
    /// on a server that takes no anonymous players.
    NotLoggedIn,
    /// The connection is logged in already (E16 M3 T1).
    AlreadyLoggedIn,
    /// The server has no ladder under this id (E16 M4 T2).
    NoSuchLadder,
    /// The account waits in a queue already, or for a match's answers (E16 M4 T2).
    AlreadyQueued,
    /// The account's ranked game on this ladder is not over (E16 M4 T2; ADR-0019 point 8).
    InRankedGame,
    /// The account waits in no queue (E16 M4 T2).
    NotQueued,
    /// The account has no match to answer (E16 M4 T2).
    NoMatch,
}

/// Why the server closes the connection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GoodbyeReason {
    /// The client speaks another protocol version.
    Protocol,
    /// A message did not decode.
    Malformed,
    /// A message exceeded the frame size.
    TooLarge,
    /// Too many messages per second.
    RateLimited,
    /// No message within the idle timeout.
    IdleTimeout,
    /// The client did not read its messages fast enough.
    TooSlow,
    /// The client left.
    Left,
    /// Another connection took the seat with its token, or as its account (E16 M3 T3).
    Replaced,
    /// The server shuts down.
    Shutdown,
    /// The game paused (E16 M2 T3); a seat's player resumes it by joining with the token.
    Paused,
    /// The host closed the game to spectators (E16 M2 T4c).
    SpectatorsClosed,
    /// The account re-keyed on another connection (E16 M3 T3): a connection logged in with its
    /// former key closes; the player logs in again with the new one.
    Rekeyed,
    /// The game reached the server's limit of commands without an end (E16 M3 T4a): it is void,
    /// with no result and no replay.
    CommandLimit,
    /// The game had not started and none of its seats is held any more (E16 M3 T4a): it is gone,
    /// and its spectators with it.
    Abandoned,
}

/// A message from the server. `n` counts the messages a viewer received from its game, so a gap
/// shows a lost message and nothing about other viewers (ADR-0018 point 7).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum ServerMessage {
    /// The answer to a hello in the server's protocol version.
    Welcome {
        /// [`crate::PROTOCOL_VERSION`] of the server.
        protocol: u32,
        /// The server program's name and version (informational).
        server: String,
        /// The server's id (E16 M3 T1): the address its clients dial. A client signs a login
        /// only if it is the address it dialed.
        id: String,
        /// This connection's challenge for a login (E16 M3 T1), 32 bytes as lowercase hex.
        challenge: String,
    },
    /// The connection closes.
    Goodbye {
        /// Why.
        reason: GoodbyeReason,
    },
    /// The connection holds a seat now.
    Joined {
        /// The game.
        game: u32,
        /// The seat it holds.
        seat: Seat,
        /// The token that re-takes the seat after a disconnect: none for a spectator, nor for a
        /// seat that belongs to an account (E16 M3 T3), which its account re-takes.
        token: Option<String>,
        /// What the game is made of.
        manifest: Manifest,
    },
    /// The join was refused.
    Refused {
        /// Why.
        reason: Refusal,
    },
    /// The whole battle as the viewer sees it (after a join, and on request).
    Snapshot {
        /// The viewer's message number.
        n: u32,
        /// The observation.
        observation: Observation,
    },
    /// The seat may act: its legal commands and the spells its hero may cast.
    YourTurn {
        /// The viewer's message number.
        n: u32,
        /// The legal commands (casts are listed by spell in `castable`).
        legal: Vec<Command>,
        /// The handles of the spells the hero may cast now.
        castable: Vec<u16>,
    },
    /// The command was applied, as echoed.
    Accepted {
        /// The viewer's message number.
        n: u32,
        /// The client's sequence number.
        seq: u32,
        /// The command applied.
        command: Command,
    },
    /// The command (or the request) was rejected; nothing changed.
    Rejected {
        /// The viewer's message number (0 for a connection that holds no seat: it is no viewer).
        n: u32,
        /// The client's sequence number.
        seq: u32,
        /// Why.
        code: RejectCode,
    },
    /// The answer to a query.
    QueryResult {
        /// The viewer's message number.
        n: u32,
        /// The client's sequence number.
        seq: u32,
        /// The targets.
        targets: Vec<Target>,
    },
    /// What happened, as the viewer may see it.
    Events {
        /// The viewer's message number.
        n: u32,
        /// The events, in order.
        events: Vec<Event>,
    },
    /// The battle is over.
    GameEnded {
        /// The viewer's message number.
        n: u32,
        /// How it ended.
        result: Outcome,
    },
    /// The replay: the CLI's replay file (participants, after the game; ADR-0018 point 8).
    Replay {
        /// The number of the viewer's last numbered message (this one takes none).
        n: u32,
        /// The client's sequence number.
        seq: u32,
        /// The replay file's text.
        replay: String,
    },
    /// The viewer's record: its own message log, not re-runnable. Its first line is a
    /// [`RecordHeader`], then one line per numbered message the viewer was sent, in order, from
    /// message [`RecordHeader::first`] to message `n`: the record keeps a viewer's latest messages
    /// within a bound in bytes (E16 M2 T5c), its oldest go first.
    Record {
        /// The number of the viewer's last numbered message (this one takes none).
        n: u32,
        /// The client's sequence number.
        seq: u32,
        /// The record's text.
        record: String,
    },
    /// The answer to a ping.
    Pong {
        /// The ping's nonce.
        nonce: u32,
    },
    /// The server's offers and its games (E16 M2 T1).
    Games {
        /// The client's sequence number.
        seq: u32,
        /// What a game may be created from.
        offers: Vec<Offer>,
        /// The hosted games, by id.
        games: Vec<GameInfo>,
    },
    /// A game was created; the game's `Joined` follows.
    GameCreated {
        /// The client's sequence number.
        seq: u32,
        /// The new game's id.
        game: u32,
    },
    /// A seat asked to pause the game (E16 M2 T3); the host's ask pauses it.
    PauseAsked {
        /// The viewer's message number.
        n: u32,
        /// The seat that asked.
        seat: Seat,
    },
    /// The host's `SetSpectators` is done (E16 M2 T4c): whether the game takes spectators now.
    SpectatorsSet {
        /// The viewer's message number.
        n: u32,
        /// The request's sequence number.
        seq: u32,
        /// Whether the game takes spectators.
        open: bool,
    },
    /// A page of the archive (E16 M2 T5d): `ListReplays`'s answer, newest first.
    Replays {
        /// The client's sequence number.
        seq: u32,
        /// How many replays the archive holds.
        total: u32,
        /// The page.
        replays: Vec<ReplayInfo>,
    },
    /// The logged-in account as it stands (E16 M3 T1): the answer to `Login`, `Rekey`, `Rename`
    /// and `SetListed`. The login name and the key are the client's own already.
    Account {
        /// The request's sequence number.
        seq: u32,
        /// Its display name on this server.
        display: String,
        /// Whether its finished games' replays may be listed.
        listed: bool,
    },
    /// The server's ladders (E16 M4 T2): `ListLadders`'s answer.
    Ladders {
        /// The client's sequence number.
        seq: u32,
        /// The ladders, by id.
        ladders: Vec<LadderInfo>,
    },
    /// The account waits in the ladder's queue (E16 M4 T2).
    Queued {
        /// The client's sequence number.
        seq: u32,
        /// The ladder's id.
        ladder: String,
    },
    /// The account left its queue, as it asked (`Dequeue`, `DeclineMatch`; E16 M4 T2).
    Dequeued {
        /// The client's sequence number.
        seq: u32,
    },
    /// A match (E16 M4 T2): accept it (`AcceptMatch`) within `accept_ms`, or the player leaves the
    /// queue.
    MatchFound {
        /// The ladder's id.
        ladder: String,
        /// How long the player has to accept, in milliseconds.
        accept_ms: u64,
    },
    /// The match will not be played: the other player declined, did not accept in time or went
    /// away (E16 M4 T2). The player waits in the queue again, its waiting time kept.
    MatchCancelled {},
    /// Both accepted (E16 M4 T2): the game is created, the seat the account's; take it with
    /// `Join` (no token).
    MatchReady {
        /// The ladder's id.
        ladder: String,
        /// The game.
        game: u32,
        /// The account's seat.
        seat: Seat,
    },
    /// The player is out of its queue without having asked (E16 M4 T2).
    Unqueued {
        /// Why.
        reason: UnqueueReason,
    },
}

/// One finished game in the archive (E16 M2 T5d).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReplayInfo {
    /// Its id, as `FetchArchived` names it.
    pub id: String,
    /// The offer the game was created from.
    pub offer: String,
    /// The game's id while it ran (ids may repeat across restarts; the replay's id does not).
    pub game: u32,
}

/// The first line of a record (the `Record` message; `docs/epics/E16-design.md` §6): what the
/// game is made of and whose record it is. Every further line is one numbered server message.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecordHeader {
    /// [`crate::RECORD_FORMAT_VERSION`].
    pub format: u32,
    /// The game.
    pub game: u32,
    /// The seat whose messages these are.
    pub seat: Seat,
    /// What the game is made of.
    pub manifest: Manifest,
    /// The number of the record's first message (E16 M2 T5c, format 2): 1 while the record holds
    /// every message, higher once its bound let the oldest go; one more than the last number when
    /// it holds none.
    pub first: u32,
}
