//! The battle hero (ADR-0013 point 4): everything a battle needs to know about the hero leading a
//! side, derived before the seam (`h3_heroes::derive_battle_hero` from a hero's details; a
//! scenario may also give it directly). The battle takes it as given and checks only what its
//! rules need to run soundly, not whether a real hero could have these values.
//!
//! The magic (E04b) is derived since E04b T2a (HER-BAT-011, HER-BAT-012); the battle reads it from
//! E04b T2b on. Since E04b T8a the masteries come per spell, derived once in the Heroes context
//! like the available spells themselves (BAT-Q19, ADR-0013 point 2).

use alloc::vec::Vec;
use core::num::NonZeroU32;

use h3_content::{
    Affects, CreatureSelector, CreatureStat, CreatureTypeId, FactionId, HeroTypeId,
    MAX_ALIGNMENT_FACTIONS, SchoolId, SpellId, SpellScope,
};

/// Most entries in [`BattleHero::effects`].
pub const MAX_BATTLE_EFFECTS: usize = 64;

/// The hero leading one side of a battle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BattleHero {
    /// The hero type: identity for observations and replays; no battle rule reads it.
    pub hero_type: HeroTypeId,
    /// Effective attack (base, artifacts and specialty): added to the attack of the side's stacks.
    pub attack: u8,
    /// Effective defense: added to the defense of the side's stacks.
    pub defense: u8,
    /// Effective spell power (E04b).
    pub power: u8,
    /// Effective knowledge; explains the maximum mana (E04b).
    pub knowledge: u8,
    /// Tactics level, 0 (none) to 3 (Expert); the battle compares both sides' levels (T8).
    pub tactics: u8,
    /// Damage bonuses of the side's stacks (T6).
    pub damage: DamageBonuses,
    /// Which war machines the hero commands, and how (T7).
    pub war_machines: MachineControl,
    /// Discount on the price of a surrender, basis points (at most 10 000; T9).
    pub surrender_discount_bp: u16,
    /// Every other battle effect of the hero's skills, artifacts and specialty, resolved (at most
    /// [`MAX_BATTLE_EFFECTS`]; morale and luck since E04a T5b, `NoRetreat` since T9a, stack
    /// statistics and traits since T10a and T10b, alignment groups since T10c).
    pub effects: Vec<BattleEffect>,
    /// Spell book, mana and schools (E04b; carried and ignored before).
    pub magic: BattleMagic,
}

impl BattleHero {
    /// A hero of `hero_type` with every value neutral: primary skills 0, no Tactics, no bonuses,
    /// no war-machine control, no effects, no magic.
    pub const fn new(hero_type: HeroTypeId) -> Self {
        Self {
            hero_type,
            attack: 0,
            defense: 0,
            power: 0,
            knowledge: 0,
            tactics: 0,
            damage: DamageBonuses {
                ranged: Rate::bp(0),
                melee: Rate::bp(0),
                reduction: Rate::bp(0),
            },
            war_machines: MachineControl {
                ballista: None,
                tent: None,
                catapult: None,
            },
            surrender_discount_bp: 0,
            effects: Vec::new(),
            magic: BattleMagic {
                spell_book: None,
                masteries: Vec::new(),
                mana: 0,
                max_mana: 0,
                eagle_eye: None,
            },
        }
    }
}

/// A rate of the hero's as it crosses the seam (OPEN-QUESTIONS Q45): basis points, and the
/// single-precision value the original computes for it when the derivation follows quirk Q-0005
/// (a skill specialist's growth, artifacts added to a skill's value).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Rate {
    /// The rate in basis points (exact; a specialist's growth rounded down).
    pub bp: u32,
    /// The original's single-precision value, the bit pattern of a positive IEEE 754 binary32
    /// value (`h3_kernel::arith::X87::from_f32_bits`); `None`: the single-precision value of
    /// `bp`.
    pub single: Option<NonZeroU32>,
}

impl Rate {
    /// The rate of `bp` basis points, without a single-precision value of its own.
    pub const fn bp(bp: u32) -> Self {
        Self { bp, single: None }
    }
}

/// Damage bonuses of a hero's side (T6; survey `E04-skills.md` §3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DamageBonuses {
    /// Added to the bonus sum of the side's shots, the Ballista's included (Archery). May exceed
    /// 10 000 basis points.
    pub ranged: Rate,
    /// Added to the bonus sum of the side's melee attacks and retaliations (Offense). May exceed
    /// 10 000 basis points.
    pub melee: Rate,
    /// One factor `1 − reduction` on every physical strike against the side's stacks
    /// (Armorer); at most 10 000 basis points.
    pub reduction: Rate,
}

/// The war machines a hero commands (T7). Without control a machine acts on its own (E03).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MachineControl {
    /// The side commands its Ballista (Artillery).
    pub ballista: Option<BallistaControl>,
    /// The side commands its First Aid Tent (First Aid).
    pub tent: Option<TentControl>,
    /// The side commands its Catapult (Ballistics). Reserved for sieges (E04c): the value cannot
    /// be built before then, so it is always `None`.
    pub catapult: Option<CatapultControl>,
}

/// How a commanded Ballista shoots.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BallistaControl {
    /// Shots per turn.
    pub shots: u8,
    /// Chance of each shot to deal double base damage, basis points (at most 10 000).
    pub double_damage_chance_bp: u16,
}

/// How a commanded First Aid Tent heals.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TentControl {
    /// Increase of the tent's largest heal, a share of the tent's own maximum (a First Aid
    /// specialist's growth can take it far beyond 10 000 basis points).
    pub heal_bonus: Rate,
}

/// How a commanded Catapult shoots: reserved for sieges (E04c), which define its contents.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct CatapultControl {}

/// A hero's magic in battle (E04b): present and checked from E04a on, used from E04b on.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BattleMagic {
    /// The spells the hero can cast, if it has a spell book: its available spells
    /// (`h3_heroes::available_spells`, HER-SPL-002), each once.
    pub spell_book: Option<Vec<SpellId>>,
    /// The hero's mastery of each spell it casts above none (`h3_heroes::spell_masteries`,
    /// HER-SPL-003; E04b T8a, BAT-Q19), each spell once, 1 (Basic) to 3 (Expert) on ordinary
    /// ground; a battlefield's raise is the battle's (E04c). It covers every spell, not only the
    /// book's: the original's mastery is a function of the hero for any spell (Sacrifice's test
    /// reads Resurrection's, BAT-QUIRK-5).
    pub masteries: Vec<(SpellId, u8)>,
    /// Spell points at the start of the battle (may exceed the maximum, e.g. after a Mana
    /// Vortex).
    pub mana: u16,
    /// Maximum spell points (informative).
    pub max_mana: u16,
    /// The hero's Eagle Eye, if it has the skill (`h3_heroes`, HER-SPL-004; E04b T8d,
    /// BAT-MAG-069).
    pub eagle_eye: Option<EagleEye>,
}

/// What the battle needs of a hero's Eagle Eye (E04b T8d, BAT-Q23): at each cast of the enemy
/// hero of one of `spells`, the battle rolls `chance` and reports the spells it hit
/// ([`crate::HeroOutcome::learned`]); who learns them is the orchestrator's (BAT-Q24).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EagleEye {
    /// The chance to learn a spell at one cast, at most 1 (HER-SPL-004).
    pub chance: Rate,
    /// The spells it may learn: those the hero has not learned, of a level at most its Eagle Eye
    /// level + 1, by spell handle, each once.
    pub spells: Vec<SpellId>,
}

/// How a [`BattleEffect::StackStat`] changes a statistic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatOp {
    /// Adds the amount.
    Add,
    /// Adds the amount in basis points of the creature's own value, rounded up (creature
    /// specialties).
    PercentBp,
    /// Adds the amount in basis points of the creature's own value, rounded down (Elixir of
    /// Life).
    PercentBpDown,
}

/// Up to [`MAX_ALIGNMENT_FACTIONS`] factions in a fixed array, so that [`BattleEffect`] stays
/// `Copy` (user decision, OPEN-QUESTIONS Q36).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FactionGroup {
    /// Number of factions in use.
    len: u8,
    /// The factions, then default handles.
    ids: [FactionId; MAX_ALIGNMENT_FACTIONS],
}

impl FactionGroup {
    /// A group of `factions`, or `None` if they are more than [`MAX_ALIGNMENT_FACTIONS`].
    pub fn new(factions: &[FactionId]) -> Option<Self> {
        let mut ids = [FactionId::default(); MAX_ALIGNMENT_FACTIONS];
        ids.get_mut(..factions.len())?.copy_from_slice(factions);
        Some(Self {
            // At most `MAX_ALIGNMENT_FACTIONS`, which fits.
            len: u8::try_from(factions.len()).unwrap_or(u8::MAX),
            ids,
        })
    }

    /// The factions, in the order given.
    pub fn factions(&self) -> &[FactionId] {
        self.ids.get(..usize::from(self.len)).unwrap_or_default()
    }
}

/// One resolved effect of a hero in battle (ADR-0013 point 3). Later tasks add variants (T10,
/// E04b); the battle checks them like the rest of the setup.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BattleEffect {
    /// Changes the morale of the affected stacks for the whole battle (Leadership).
    Morale {
        /// Morale change.
        amount: i8,
        /// Whose stacks.
        affects: Affects,
    },
    /// Changes the luck of the affected stacks for the whole battle (Luck).
    Luck {
        /// Luck change.
        amount: i8,
        /// Whose stacks.
        affects: Affects,
    },
    /// Caps the morale of the affected stacks for the whole battle (Spirit of Oppression).
    MoraleAtMost {
        /// Highest morale.
        value: i8,
        /// Whose stacks.
        affects: Affects,
    },
    /// Caps the luck of the affected stacks for the whole battle (Hourglass of the Evil Hour).
    LuckAtMost {
        /// Highest luck.
        value: i8,
        /// Whose stacks.
        affects: Affects,
    },
    /// Changes a statistic of the hero's selected stacks for the whole battle (creature
    /// specialties, stat artifacts).
    StackStat {
        /// Which creatures.
        creatures: CreatureSelector<CreatureTypeId>,
        /// Which statistic.
        stat: CreatureStat,
        /// How the amount applies.
        op: StatOp,
        /// Amount (an addition, or basis points with [`StatOp::PercentBp`]).
        amount: i16,
    },
    /// The top creature of each selected stack of the hero regains up to `hit_points` lost hit
    /// points at the start of its turn (Elixir of Life).
    Regeneration {
        /// Which creatures.
        creatures: CreatureSelector<CreatureTypeId>,
        /// Most hit points regained per turn.
        hit_points: u16,
    },
    /// The hero's selected creatures shoot without range penalty (Golden Bow).
    NoRangePenalty {
        /// Which creatures.
        creatures: CreatureSelector<CreatureTypeId>,
    },
    /// The hero's selected creatures may shoot while an enemy stands next to them (Bow of the
    /// Sharpshooter).
    ShootWhenAdjacent {
        /// Which creatures.
        creatures: CreatureSelector<CreatureTypeId>,
    },
    /// Neither side may retreat or surrender (Shackles of War).
    NoRetreat,
    /// Creatures of these factions count as one alignment for the army-alignment morale of the
    /// hero's side (Angelic Alliance; BAT-MOR-002).
    AlignmentGroup {
        /// The factions.
        factions: FactionGroup,
    },
    /// More damage from the hero's spells of a scope (Sorcery, the Orbs, Ciele's specialty;
    /// E04b): one factor per kind of scope (HER-BAT-012).
    SpellDamage {
        /// Which spells.
        scope: SpellScope<SchoolId, SpellId>,
        /// The bonus.
        rate: Rate,
    },
    /// The effect of the hero's casts of `spell` on a stack grows by `bp_per_level` basis
    /// points for each whole multiple of the stack's creature level in the hero's `level` (a
    /// spell specialty, E04b; HER-BAT-012, BAT-MAG-004): the original divides the levels in
    /// whole numbers before it multiplies.
    SpellSpecialty {
        /// The spell.
        spell: SpellId,
        /// Growth per hero level, basis points.
        bp_per_level: u16,
        /// The hero's level.
        level: u8,
    },
    /// The side's stacks resist hostile spells: their creatures' share a spell affects shrinks
    /// by `rate` (the Resistance skill and artifacts, one effect per source in the derivation's
    /// order; E04b T2d; HER-BAT-013, BAT-MAG-008).
    MagicResistance {
        /// The resistance.
        rate: Rate,
    },
    /// The hero's timed spells last `rounds` rounds longer (the Conjuring artifacts, one effect
    /// per source; E04b T3a; HER-BAT-014, BAT-MAG-010).
    SpellDuration {
        /// Extra rounds.
        rounds: u8,
    },
    /// Neither side casts while this hero is in the battle: no hero's spell, no creature's cast
    /// or raise, no periodic cast (the Orb of Inhibition; E04b T8b1; HER-BAT-015, BAT-MAG-061).
    NoCasting,
    /// While this hero is in the battle no spell above `max_level` is cast by a hero or affects a
    /// stack, whoever casts it (Recanter's Cloak: 2; E04b T8b1; HER-BAT-015, BAT-MAG-062).
    SpellLevelLimit {
        /// The highest spell level allowed.
        max_level: u8,
    },
    /// The stacks the hero controls are never affected by `spell`, whoever casts it, before
    /// every other test but the spell level limit (a Pendant, the Sphere of Permanence for Dispel
    /// and Dispel Helpful Spells, Armageddon's Blade for Armageddon; E04b T8b2; HER-BAT-016,
    /// BAT-MAG-063).
    SpellImmunity {
        /// The spell.
        spell: SpellId,
    },
    /// The stacks the hero controls are never affected by a spell up to `max_level`, whoever
    /// casts it and whatever its polarity, Dispel excepted (Power of the Dragon Father: 4; E04b
    /// T8b2; HER-BAT-016, BAT-MAG-064).
    SpellLevelImmunity {
        /// The highest level.
        max_level: u8,
    },
    /// The stacks the hero controls are immune to mind spells unless the immunities are negated
    /// (Badge of Courage; E04b T8b2; HER-BAT-016, BAT-MAG-065).
    MindImmunity,
    /// Spells cast by the hero's side, and spells at the stacks it controls, ignore the
    /// creatures' immunities and magic resistance and the heroes' resistance (the Orb of
    /// Vulnerability; E04b T8b2; HER-BAT-016, BAT-MAG-066).
    NegateImmunities,
    /// At the first activation of a stack its side controls outside the tactics phase, the side
    /// casts `spell` at `mastery` with `power` (its duration) as its hero's cast that costs
    /// nothing and is not the round's cast, if it affects some stack (Angelic Alliance's Prayer,
    /// Armor of the Damned's four; E04b T8c; HER-BAT-017, BAT-MAG-068). The effects of a hero
    /// cast in their order.
    OpeningSpell {
        /// The spell.
        spell: SpellId,
        /// Its mastery, 0 (none) to 3 (Expert).
        mastery: u8,
        /// Its power: its effects' duration in rounds.
        power: u16,
    },
}
