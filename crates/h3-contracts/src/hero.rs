//! The battle hero (ADR-0013 point 4): everything a battle needs to know about the hero leading a
//! side, derived before the seam (`h3_heroes::derive_battle_hero` from a hero's details; a
//! scenario may also give it directly). The battle takes it as given and checks only what its
//! rules need to run soundly, not whether a real hero could have these values.
//!
//! Fields that later tasks give a meaning stay at their neutral value until then: the damage
//! bonuses (E04a T6), war-machine control (T7), the surrender discount (T9), the effects other
//! than morale and luck (T10) and the magic (E04b).

use alloc::vec::Vec;

use h3_content::{Affects, CreatureSelector, CreatureStat, CreatureTypeId, HeroTypeId, SpellId};

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
    /// [`MAX_BATTLE_EFFECTS`]; morale and luck since E04a T5b, the others with T10).
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
                ranged_bp: 0,
                melee_bp: 0,
                reduction_bp: 0,
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
                mana: 0,
                max_mana: 0,
                schools: [0; 4],
            },
        }
    }
}

/// Damage bonuses of a hero's side, in basis points (T6; survey `E04-skills.md` §3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DamageBonuses {
    /// Added to the bonus sum of the side's shots, the Ballista's included (Archery). May exceed
    /// 10 000.
    pub ranged_bp: u16,
    /// Added to the bonus sum of the side's melee attacks and retaliations (Offense). May exceed
    /// 10 000.
    pub melee_bp: u16,
    /// One factor `(10 000 − reduction) / 10 000` on every physical strike against the side's
    /// stacks (Armorer); at most 10 000.
    pub reduction_bp: u16,
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
    /// Increase of the tent's largest heal, basis points of the tent's own maximum (a First Aid
    /// specialist's growth can take it far beyond 10 000).
    pub heal_bonus_bp: u32,
}

/// How a commanded Catapult shoots: reserved for sieges (E04c), which define its contents.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct CatapultControl {}

/// A hero's magic in battle (E04b): present and checked from E04a on, used from E04b on.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BattleMagic {
    /// The spells the hero can cast, if it has a spell book.
    pub spell_book: Option<Vec<SpellId>>,
    /// Spell points at the start of the battle (may exceed the maximum, e.g. after a Mana
    /// Vortex).
    pub mana: u16,
    /// Maximum spell points (informative).
    pub max_mana: u16,
    /// Mastery of the air, earth, fire and water schools, 0 (none) to 3 (Expert).
    pub schools: [u8; 4],
}

/// How a [`BattleEffect::StackStat`] changes a statistic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatOp {
    /// Adds the amount.
    Add,
    /// Adds the amount in basis points of the creature's own value, rounded up.
    PercentBp,
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
}
