//! What a spell does in battle (E04b, ADR-0014 point 1): the target its caster names, the stacks
//! the cast reaches (an area around a hex, a chain from a stack, or every stack), the conditions
//! a stack must meet, and its effects from a closed set of kinds (pack format v4, E04b T2a; timed
//! changes and mass versions since format v5, E04b T3a).
//!
//! Spells are data over these kinds: each kind is one battle rule (`BAT-MAG-…`), so a new spell of
//! a known kind is a data change. Kinds join with the task that implements them: damage in T2;
//! timed changes of a stack's values in T3a; dispel and heal in T3b; and so on. A spell without
//! effects cannot be cast.

use h3_kernel::{CanonicalHash, StateHasher};

use crate::effect::FULL_BP;
use crate::flags::CreatureFlags;

/// Most effects of one spell.
pub const MAX_SPELL_EFFECTS: usize = 4;
/// Largest distance from the target hex an area may reach.
pub const MAX_AREA_RADIUS: u8 = 16;
/// Most stacks a chain may strike.
pub const MAX_CHAIN_TARGETS: u8 = 16;

/// What the caster names when casting (below Expert for spells with a mass version; T3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum SpellTarget {
    /// Nothing: the spell reaches every stack that meets its conditions (Armageddon).
    #[default]
    None,
    /// A stack (Magic Arrow; Chain Lightning's first stack).
    Stack,
    /// A hex: the spell reaches the stacks on its area (Fireball).
    Hex,
}

impl SpellTarget {
    /// All values, in code order.
    pub const ALL: [Self; 3] = [Self::None, Self::Stack, Self::Hex];

    /// Pack name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Stack => "stack",
            Self::Hex => "hex",
        }
    }

    /// Looks a value up by its pack name.
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|t| t.name() == name)
    }
}

/// Whose stacks a spell may reach, seen from the caster (VCMI's "smart" targeting: Magic Arrow
/// only enemies).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum TargetSide {
    /// Stacks of either side.
    #[default]
    Any,
    /// The enemy's stacks.
    Enemy,
    /// The caster's own stacks.
    Own,
}

impl TargetSide {
    /// All values, in code order.
    pub const ALL: [Self; 3] = [Self::Any, Self::Enemy, Self::Own];

    /// Pack name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Any => "any",
            Self::Enemy => "enemy",
            Self::Own => "own",
        }
    }

    /// Looks a value up by its pack name.
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|s| s.name() == name)
    }
}

/// The hexes a spell aimed at a hex reaches: those at a distance of `inner` to `radius[mastery]`
/// from the target hex (Fireball 0 to 1, Frost Ring 1 to 1, Inferno 0 to 2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpellArea {
    /// Smallest distance reached (1 leaves the target hex out).
    pub inner: u8,
    /// Largest distance reached, by mastery: none, basic, advanced, expert.
    pub radius: [u8; 4],
}

/// How a spell aimed at a stack jumps on to further stacks (Chain Lightning).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpellChain {
    /// Stacks struck in all, the first included, by mastery.
    pub targets: [u8; 4],
    /// Each further stack takes this share of the previous stack's value, basis points.
    pub factor_bp: u16,
}

/// How a timed effect changes a stack while it lasts (E04b T3a; the original's code,
/// `docs/formats/installation.md` §7.9). `v` is the spell's `effect[mastery]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TimedChange {
    /// Speed + v, not on war machines (Haste, Prayer).
    Speed,
    /// Speed × v %, rounded down, at least 1 (Slow).
    SpeedPercent,
    /// Attack + v (Prayer).
    Attack,
    /// Attack − v, at most down to 0 (Weakness).
    AttackLoss,
    /// Defense + v (Stone Skin, Prayer).
    Defense,
    /// Defense − v, at most down to 0, never given back (Disrupting Ray).
    DefenseLoss,
    /// Attack + v on melee attacks (Bloodlust).
    MeleeAttack,
    /// Attack + v on ranged attacks (Precision).
    RangedAttack,
    /// Every creature deals its maximum damage + v (Bless).
    MaximumDamage,
    /// Every creature deals its minimum damage − v, at least 1 (Curse).
    MinimumDamage,
    /// Morale + v (Mirth).
    Morale,
    /// Morale − v (Sorrow).
    MoraleLoss,
    /// Luck + v (Fortune).
    Luck,
    /// Luck − v (Misfortune).
    LuckLoss,
    /// Damage of melee strikes taken × v %, single precision (Shield; BAT-MAG-012).
    MeleeProtection,
    /// Damage of shots taken × v %, single precision (Air Shield; BAT-MAG-012).
    RangedProtection,
    /// Damage of spells of this spell's schools taken × v % (the Protections; BAT-MAG-013).
    SchoolProtection,
    /// Attack + v against creatures of a king level the first cast's mastery reaches: king 1
    /// always, king 2 from Advanced, king 3 at Expert (Slayer; BAT-MAG-014).
    Slayer,
    /// Retaliations per round + v; the first cast adds v to those left at once, the end gives
    /// nothing back (Counterstrike; BAT-MAG-015).
    Retaliations,
    /// Melee strikes on the stack reflect v % of their first-stage damage onto the striker as
    /// this spell's damage (Fire Shield; BAT-MAG-018, v8).
    FireShield,
    /// Attack + its defense × v %, defense 0 when struck (Frenzy; BAT-MAG-019, v8).
    Frenzy,
    /// Immune to every spell below level v that is not a creature ability, whatever its
    /// polarity; the first cast ends the stack's negative effects below level v (Anti-Magic;
    /// BAT-MAG-021, v10).
    SpellImmunity,
    /// A single-target spell aimed at the stack from the other side goes, with a chance of v %, to
    /// a random living stack of the caster's side instead (Magic Mirror; BAT-MAG-022, v10).
    MagicMirror,
}

impl TimedChange {
    /// All values, in code order.
    pub const ALL: [Self; 23] = [
        Self::Speed,
        Self::SpeedPercent,
        Self::Attack,
        Self::AttackLoss,
        Self::Defense,
        Self::DefenseLoss,
        Self::MeleeAttack,
        Self::RangedAttack,
        Self::MaximumDamage,
        Self::MinimumDamage,
        Self::Morale,
        Self::MoraleLoss,
        Self::Luck,
        Self::LuckLoss,
        Self::MeleeProtection,
        Self::RangedProtection,
        Self::SchoolProtection,
        Self::Slayer,
        Self::Retaliations,
        Self::FireShield,
        Self::Frenzy,
        Self::SpellImmunity,
        Self::MagicMirror,
    ];

    /// Pack name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Speed => "speed",
            Self::SpeedPercent => "speed_percent",
            Self::Attack => "attack",
            Self::AttackLoss => "attack_loss",
            Self::Defense => "defense",
            Self::DefenseLoss => "defense_loss",
            Self::MeleeAttack => "melee_attack",
            Self::RangedAttack => "ranged_attack",
            Self::MaximumDamage => "maximum_damage",
            Self::MinimumDamage => "minimum_damage",
            Self::Morale => "morale",
            Self::MoraleLoss => "morale_loss",
            Self::Luck => "luck",
            Self::LuckLoss => "luck_loss",
            Self::MeleeProtection => "melee_protection",
            Self::RangedProtection => "ranged_protection",
            Self::SchoolProtection => "school_protection",
            Self::Slayer => "slayer",
            Self::Retaliations => "retaliations",
            Self::FireShield => "fire_shield",
            Self::Frenzy => "frenzy",
            Self::SpellImmunity => "spell_immunity",
            Self::MagicMirror => "magic_mirror",
        }
    }

    /// Looks a value up by its pack name.
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|c| c.name() == name)
    }
}

/// One effect of a spell on each stack it reaches. The value of a cast is
/// `power × power factor + effect[mastery]` (the spell's data row), changed per target and caster
/// by the kind's rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SpellEffect {
    /// Deals the value as damage (BAT-MAG rules, E04b T2).
    Damage,
    /// Changes the stack's values while the effect lasts (BAT-MAG-009 to 011, E04b T3a).
    Timed(TimedChange),
    /// Ends the stack's timed effects: those of the spell's `dispels`, or, without that list,
    /// every one but those of its `spares`; never a permanent one (BAT-MAG-016, E04b T3b1b).
    Dispel,
    /// Heals the stack's top creature by the value, raising none (BAT-MAG-017, E04b T3b1b).
    Heal,
}

impl SpellEffect {
    /// Pack name of the kind.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Damage => "damage",
            Self::Timed(_) => "timed",
            Self::Dispel => "dispel",
            Self::Heal => "heal",
        }
    }

    /// The timed change, if this is one.
    pub const fn timed(self) -> Option<TimedChange> {
        match self {
            Self::Timed(c) => Some(c),
            Self::Damage | Self::Dispel | Self::Heal => None,
        }
    }

    /// Canonical code (hash): the kind, then the change.
    fn code(self) -> [u8; 2] {
        match self {
            Self::Damage => [0, 0],
            Self::Timed(c) => [1, c as u8],
            Self::Dispel => [2, 0],
            Self::Heal => [3, 0],
        }
    }
}

/// How long the timed effects of a spell last.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum EffectDuration {
    /// The caster's power plus its duration bonuses, in rounds (BAT-MAG-010).
    #[default]
    Rounds,
    /// The rest of the battle (Disrupting Ray).
    Permanent,
    /// Through the stack's next turn: 1 round on the active stack, else 2, counted down at the
    /// stack's own turn start instead of the round start (Frenzy; BAT-MAG-019, v8).
    OwnTurn,
}

/// Creature levels a [`SpecialistBonus::CreatureLevel`] table covers (1 to 7).
pub const SPECIALIST_LEVELS: usize = 7;

/// What a hero specialised in a spell adds to the spell's value on a stack, when the spell has a
/// rule of its own (`docs/formats/installation.md` §7.12; v9). Without one, the hero's
/// `spell_specialty` rate scales the value (damage, Cure, Bless).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpecialistBonus {
    /// An amount by the stack's creature level, 1 to 7 (higher levels: the last).
    CreatureLevel([u8; SPECIALIST_LEVELS]),
    /// A fixed amount.
    Fixed(u16),
    /// The value becomes this total, whatever the mastery.
    Total(u16),
}

impl SpecialistBonus {
    /// The bonus to the value `t` on a stack of creature level `level` (at least 1).
    pub fn bonus(self, level: u8, t: u64) -> i64 {
        match self {
            Self::CreatureLevel(values) => {
                let i = usize::from(level.max(1) - 1).min(SPECIALIST_LEVELS - 1);
                i64::from(values[i])
            }
            Self::Fixed(amount) => i64::from(amount),
            Self::Total(total) => i64::from(total) - i64::try_from(t).unwrap_or(i64::MAX),
        }
    }
}

impl CanonicalHash for SpecialistBonus {
    /// Kind (0 creature level, 1 fixed, 2 total) and its values.
    fn hash_into(&self, h: &mut StateHasher) {
        match *self {
            Self::CreatureLevel(values) => {
                h.write_u8(0);
                h.write_bytes(&values);
            }
            Self::Fixed(amount) => {
                h.write_u8(1);
                h.write_u16(amount);
            }
            Self::Total(total) => {
                h.write_u8(2);
                h.write_u16(total);
            }
        }
    }
}

/// Highest mastery (Expert).
pub const MAX_MASTERY: u8 = 3;

/// Pack names of the masteries that can start a mass version, from Basic (1) to Expert (3).
pub const MASTERY_NAMES: [&str; 3] = ["basic", "advanced", "expert"];

/// Which stacks a spell may affect, by creature flags (Destroy Undead: only `undead`; Death
/// Ripple: no `undead`, no `war_machine`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TargetConditions {
    /// A stack needs every one of these flags.
    pub requires: CreatureFlags,
    /// A stack may have none of these flags.
    pub excludes: CreatureFlags,
}

impl TargetConditions {
    /// Whether a creature with `flags` meets the conditions.
    pub const fn admits(self, flags: CreatureFlags) -> bool {
        flags.bits() & self.requires.bits() == self.requires.bits()
            && flags.bits() & self.excludes.bits() == 0
    }
}

/// How a spell reaches stacks in battle and what it does to them.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SpellMagic {
    /// What the caster names.
    pub target: SpellTarget,
    /// The area around a hex target, if the target is a hex.
    pub area: Option<SpellArea>,
    /// The chain from a stack target, if any.
    pub chain: Option<SpellChain>,
    /// Whose stacks it may reach (below [`Self::any_side_from`]).
    pub side: TargetSide,
    /// The lowest mastery (1..=3) from which it may reach stacks of either side (Dispel: from
    /// Advanced; v7).
    pub any_side_from: Option<u8>,
    /// A stack it is aimed at needs an active timed effect (Dispel below Expert; v7).
    pub needs_effect: bool,
    /// No creature is immune to it and none resists it: no immunity, magic resistance or
    /// resistance aura applies (Dispel, `installation.md` §7.10; v7); nor do Anti-Magic and
    /// Magic Mirror (§7.13; v10).
    pub ignores_immunity: bool,
    /// The stacks it may affect.
    pub conditions: TargetConditions,
    /// What it does to each stack reached, distinct, in pack order (none: not castable yet).
    pub effects: alloc::vec::Vec<SpellEffect>,
    /// A mind spell (the executable's flag `0x400`): creatures with flag `mind_immune` are immune
    /// (v5).
    pub mind: bool,
    /// The lowest mastery (1..=3) at which the spell has a mass version: it then takes no target
    /// and reaches every stack it may reach (the flag `0x40`: Expert; v5).
    pub mass_from: Option<u8>,
    /// How long its timed effects last (v5).
    pub duration: EffectDuration,
    /// A cast on a stack it already affects applies its changes again (Disrupting Ray; v5).
    pub cumulative: bool,
    /// What a specialist in the spell adds to its value by a rule of its own (v9).
    pub specialist: Option<SpecialistBonus>,
}

impl SpellMagic {
    /// What the caster names at `mastery`: nothing for the mass version.
    pub fn target_at(&self, mastery: u8) -> SpellTarget {
        match self.mass_from {
            Some(from) if mastery >= from => SpellTarget::None,
            _ => self.target,
        }
    }

    /// Whose stacks it may reach at `mastery`.
    pub fn side_at(&self, mastery: u8) -> TargetSide {
        match self.any_side_from {
            Some(from) if mastery >= from => TargetSide::Any,
            _ => self.side,
        }
    }

    /// Whether any effect is a timed change.
    pub fn is_timed(&self) -> bool {
        self.effects.iter().any(|e| e.timed().is_some())
    }

    /// Whether it has a dispel effect.
    pub fn dispels(&self) -> bool {
        self.effects.contains(&SpellEffect::Dispel)
    }
}

impl SpellMagic {
    /// Why the targeting is invalid, if it is (`castable` is false for adventure spells, which
    /// have no battle effects).
    pub(crate) fn error(&self, castable: bool) -> Option<&'static str> {
        let c = self.conditions;
        if !castable && !self.effects.is_empty() {
            Some("adventure spells have no battle effects")
        } else if self.effects.len() > MAX_SPELL_EFFECTS {
            Some("more than 4 effects")
        } else if self
            .effects
            .iter()
            .enumerate()
            .any(|(i, e)| self.effects.get(..i).unwrap_or_default().contains(e))
        {
            Some("an effect is listed twice")
        } else if self.effects.is_empty() && *self != Self::default() {
            Some("targeting without effects")
        } else if self.area.is_some() && self.target != SpellTarget::Hex {
            Some("an area needs target hex")
        } else if self
            .area
            .is_some_and(|a| a.radius.iter().any(|&r| r < a.inner || r > MAX_AREA_RADIUS))
        {
            Some("area radius must be inner..=16")
        } else if self.chain.is_some() && self.target != SpellTarget::Stack {
            Some("a chain needs target stack")
        } else if self.chain.is_some_and(|ch| {
            ch.factor_bp > FULL_BP
                || ch
                    .targets
                    .iter()
                    .any(|&t| !(1..=MAX_CHAIN_TARGETS).contains(&t))
        }) {
            Some("a chain needs 1..=16 targets and factor_bp <= 10000")
        } else if c.requires.bits() & c.excludes.bits() != 0 {
            Some("a flag is both required and excluded")
        } else if self
            .mass_from
            .is_some_and(|m| self.target != SpellTarget::Stack || !(1..=MAX_MASTERY).contains(&m))
        {
            Some("a mass version needs target stack and a mastery 1..=3")
        } else if (self.cumulative || self.duration != EffectDuration::Rounds) && !self.is_timed() {
            Some("duration and cumulative need a timed effect")
        } else if self
            .any_side_from
            .is_some_and(|m| self.side == TargetSide::Any || !(1..=MAX_MASTERY).contains(&m))
        {
            Some("any_side needs a side and a mastery 1..=3")
        } else if self.needs_effect && self.target != SpellTarget::Stack {
            Some("needs_effect needs target stack")
        } else {
            None
        }
    }
}

impl CanonicalHash for SpellMagic {
    /// Target, area (presence, inner, radii), chain (presence, targets, factor), side (v15),
    /// condition bits, effects (count, codes: kind and change), mind, mass (0: none), duration,
    /// cumulative (v17); any side (0: never), needs effect, ignores immunity (v19); the
    /// specialist bonus (presence, then the bonus; v20).
    fn hash_into(&self, h: &mut StateHasher) {
        h.write_u8(self.target as u8);
        h.write_bool(self.area.is_some());
        if let Some(a) = self.area {
            h.write_u8(a.inner);
            h.write_bytes(&a.radius);
        }
        h.write_bool(self.chain.is_some());
        if let Some(ch) = self.chain {
            h.write_bytes(&ch.targets);
            h.write_u16(ch.factor_bp);
        }
        h.write_u8(self.side as u8);
        h.write_u32(self.conditions.requires.bits());
        h.write_u32(self.conditions.excludes.bits());
        h.write_len(self.effects.len());
        for e in &self.effects {
            h.write_bytes(&e.code());
        }
        h.write_bool(self.mind);
        h.write_u8(self.mass_from.unwrap_or(0));
        h.write_u8(self.duration as u8);
        h.write_bool(self.cumulative);
        h.write_u8(self.any_side_from.unwrap_or(0));
        h.write_bool(self.needs_effect);
        h.write_bool(self.ignores_immunity);
        h.write_bool(self.specialist.is_some());
        if let Some(b) = self.specialist {
            b.hash_into(h);
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use std::vec;

    use super::*;
    use crate::flags::CreatureFlag;

    #[test]
    fn names_round_trip() {
        for t in SpellTarget::ALL {
            assert_eq!(SpellTarget::from_name(t.name()), Some(t));
        }
        for c in TimedChange::ALL {
            assert_eq!(TimedChange::from_name(c.name()), Some(c));
            assert_eq!(SpellEffect::Timed(c).name(), "timed");
            assert_eq!(SpellEffect::Timed(c).timed(), Some(c));
        }
        assert_eq!(SpellEffect::Damage.name(), "damage");
        assert_eq!(SpellEffect::Damage.timed(), None);
        assert_eq!(SpellEffect::Dispel.name(), "dispel");
        assert_eq!(SpellEffect::Heal.name(), "heal");
        assert_eq!(SpellEffect::Dispel.timed(), None);
        assert_eq!(SpellEffect::Heal.timed(), None);
        assert_eq!(TimedChange::from_name("heal"), None);
        for s in TargetSide::ALL {
            assert_eq!(TargetSide::from_name(s.name()), Some(s));
        }
        assert_eq!(TargetSide::from_name("both"), None);
        assert_eq!(SpellTarget::from_name("obstacle"), None);
    }

    #[test]
    fn mass_versions_take_no_target() {
        let haste = SpellMagic {
            target: SpellTarget::Stack,
            mass_from: Some(3),
            effects: vec![SpellEffect::Timed(TimedChange::Speed)],
            ..SpellMagic::default()
        };
        assert_eq!(haste.target_at(2), SpellTarget::Stack);
        assert_eq!(haste.target_at(3), SpellTarget::None);
        let single = SpellMagic {
            mass_from: None,
            ..haste.clone()
        };
        assert_eq!(single.target_at(3), SpellTarget::Stack);
        assert!(haste.is_timed());
        assert!(!SpellMagic::default().is_timed());
        assert!(!haste.dispels());
    }

    #[test]
    fn the_side_may_widen_by_mastery() {
        let dispel = SpellMagic {
            target: SpellTarget::Stack,
            side: TargetSide::Own,
            any_side_from: Some(2),
            effects: vec![SpellEffect::Dispel],
            ..SpellMagic::default()
        };
        assert_eq!(dispel.side_at(1), TargetSide::Own);
        assert_eq!(dispel.side_at(2), TargetSide::Any);
        assert_eq!(dispel.side_at(3), TargetSide::Any);
        assert!(dispel.dispels());
        let fixed = SpellMagic {
            any_side_from: None,
            ..dispel.clone()
        };
        assert_eq!(fixed.side_at(3), TargetSide::Own);
        assert_eq!(dispel.error(true), None);
    }

    #[test]
    fn conditions_admit_by_flags() {
        let undead = CreatureFlags::default().with(CreatureFlag::Undead);
        let machine = CreatureFlags::default().with(CreatureFlag::WarMachine);
        let only_undead = TargetConditions {
            requires: undead,
            excludes: CreatureFlags::default(),
        };
        let no_undead = TargetConditions {
            requires: CreatureFlags::default(),
            excludes: undead.with(CreatureFlag::WarMachine),
        };
        assert!(only_undead.admits(undead.with(CreatureFlag::Flying)));
        assert!(!only_undead.admits(machine));
        assert!(no_undead.admits(CreatureFlags::default()));
        assert!(!no_undead.admits(undead) && !no_undead.admits(machine));
        assert!(TargetConditions::default().admits(undead));
    }

    #[test]
    fn targeting_is_validated() {
        let damage = |target| SpellMagic {
            target,
            effects: vec![SpellEffect::Damage],
            ..SpellMagic::default()
        };
        let area = SpellArea {
            inner: 1,
            radius: [1; 4],
        };
        let chain = SpellChain {
            targets: [4, 4, 5, 5],
            factor_bp: 5000,
        };
        let ok = [
            SpellMagic::default(),
            damage(SpellTarget::None),
            SpellMagic {
                area: Some(area),
                ..damage(SpellTarget::Hex)
            },
            SpellMagic {
                chain: Some(chain),
                ..damage(SpellTarget::Stack)
            },
        ];
        for m in &ok {
            assert_eq!(m.error(true), None, "{m:?}");
        }
        assert_eq!(SpellMagic::default().error(false), None);
        let undead = CreatureFlags::default().with(CreatureFlag::Undead);
        let bad = [
            (damage(SpellTarget::Stack), false, "adventure spells"),
            (
                SpellMagic {
                    effects: vec![SpellEffect::Damage; 5],
                    ..SpellMagic::default()
                },
                true,
                "more than 4",
            ),
            (
                SpellMagic {
                    effects: vec![SpellEffect::Damage; 2],
                    ..SpellMagic::default()
                },
                true,
                "listed twice",
            ),
            (
                SpellMagic {
                    target: SpellTarget::Stack,
                    ..SpellMagic::default()
                },
                true,
                "without effects",
            ),
            (
                SpellMagic {
                    area: Some(area),
                    ..damage(SpellTarget::Stack)
                },
                true,
                "needs target hex",
            ),
            (
                SpellMagic {
                    area: Some(SpellArea {
                        inner: 2,
                        radius: [2, 2, 1, 2],
                    }),
                    ..damage(SpellTarget::Hex)
                },
                true,
                "inner..=16",
            ),
            (
                SpellMagic {
                    area: Some(SpellArea {
                        inner: 0,
                        radius: [17; 4],
                    }),
                    ..damage(SpellTarget::Hex)
                },
                true,
                "inner..=16",
            ),
            (
                SpellMagic {
                    chain: Some(chain),
                    ..damage(SpellTarget::Hex)
                },
                true,
                "needs target stack",
            ),
            (
                SpellMagic {
                    chain: Some(SpellChain {
                        targets: [0, 4, 4, 4],
                        ..chain
                    }),
                    ..damage(SpellTarget::Stack)
                },
                true,
                "1..=16 targets",
            ),
            (
                SpellMagic {
                    chain: Some(SpellChain {
                        targets: [17; 4],
                        ..chain
                    }),
                    ..damage(SpellTarget::Stack)
                },
                true,
                "1..=16 targets",
            ),
            (
                SpellMagic {
                    chain: Some(SpellChain {
                        factor_bp: 10_001,
                        ..chain
                    }),
                    ..damage(SpellTarget::Stack)
                },
                true,
                "1..=16 targets",
            ),
            (
                SpellMagic {
                    conditions: TargetConditions {
                        requires: undead,
                        excludes: undead,
                    },
                    ..damage(SpellTarget::None)
                },
                true,
                "both required and excluded",
            ),
        ];
        let timed = |change| SpellMagic {
            target: SpellTarget::Stack,
            effects: vec![SpellEffect::Timed(change)],
            ..SpellMagic::default()
        };
        let ray = SpellMagic {
            duration: EffectDuration::Permanent,
            cumulative: true,
            ..timed(TimedChange::DefenseLoss)
        };
        assert_eq!(ray.error(true), None);
        assert_eq!(
            SpellMagic {
                mass_from: Some(3),
                mind: true,
                ..timed(TimedChange::Morale)
            }
            .error(true),
            None
        );
        let other = |side, any_side_from, needs_effect| SpellMagic {
            target: SpellTarget::Stack,
            side,
            any_side_from,
            needs_effect,
            effects: vec![SpellEffect::Dispel],
            ..SpellMagic::default()
        };
        assert_eq!(other(TargetSide::Own, Some(2), true).error(true), None);
        let bad = bad.into_iter().chain([
            (
                other(TargetSide::Any, Some(2), false),
                true,
                "any_side needs",
            ),
            (
                other(TargetSide::Own, Some(0), false),
                true,
                "any_side needs",
            ),
            (
                other(TargetSide::Own, Some(4), false),
                true,
                "any_side needs",
            ),
            (
                SpellMagic {
                    target: SpellTarget::None,
                    ..other(TargetSide::Own, None, true)
                },
                true,
                "needs_effect needs",
            ),
            (
                SpellMagic {
                    target: SpellTarget::Hex,
                    mass_from: Some(3),
                    ..timed(TimedChange::Speed)
                },
                true,
                "mass version",
            ),
            (
                SpellMagic {
                    mass_from: Some(0),
                    ..timed(TimedChange::Speed)
                },
                true,
                "mass version",
            ),
            (
                SpellMagic {
                    cumulative: true,
                    ..damage(SpellTarget::Stack)
                },
                true,
                "need a timed effect",
            ),
            (
                SpellMagic {
                    specialist: Some(SpecialistBonus::Fixed(2)),
                    ..SpellMagic::default()
                },
                true,
                "targeting without effects",
            ),
            (
                SpellMagic {
                    duration: EffectDuration::Permanent,
                    ..damage(SpellTarget::Stack)
                },
                true,
                "need a timed effect",
            ),
        ]);
        for (m, castable, reason) in bad {
            let error = m.error(castable).unwrap_or_default();
            assert!(error.contains(reason), "{m:?}: {error}");
        }
    }

    #[test]
    fn specialist_bonuses_by_kind() {
        let table = SpecialistBonus::CreatureLevel([3, 3, 2, 2, 1, 1, 0]);
        assert_eq!(table.bonus(1, 6), 3);
        assert_eq!(table.bonus(4, 6), 2);
        assert_eq!(table.bonus(7, 6), 0);
        // Level 0 counts as 1, levels above 7 as 7.
        assert_eq!(table.bonus(0, 6), 3);
        assert_eq!(table.bonus(9, 6), 0);
        assert_eq!(SpecialistBonus::Fixed(2).bonus(3, 5), 2);
        assert_eq!(SpecialistBonus::Total(3).bonus(3, 1), 2);
        assert_eq!(SpecialistBonus::Total(3).bonus(3, 5), -2);
    }

    #[test]
    fn hash_covers_every_field() {
        let digest = h3_kernel::hash::hash_of::<SpellMagic>;
        let base = SpellMagic {
            target: SpellTarget::Hex,
            area: Some(SpellArea {
                inner: 0,
                radius: [1; 4],
            }),
            chain: None,
            side: TargetSide::Any,
            conditions: TargetConditions::default(),
            effects: vec![SpellEffect::Damage],
            ..SpellMagic::default()
        };
        let undead = CreatureFlags::default().with(CreatureFlag::Undead);
        let variants = [
            SpellMagic {
                target: SpellTarget::Stack,
                area: None,
                chain: Some(SpellChain {
                    targets: [1; 4],
                    factor_bp: 0,
                }),
                ..base.clone()
            },
            SpellMagic {
                area: Some(SpellArea {
                    inner: 1,
                    radius: [1; 4],
                }),
                ..base.clone()
            },
            SpellMagic {
                conditions: TargetConditions {
                    requires: undead,
                    excludes: CreatureFlags::default(),
                },
                ..base.clone()
            },
            SpellMagic {
                conditions: TargetConditions {
                    requires: CreatureFlags::default(),
                    excludes: undead,
                },
                ..base.clone()
            },
            SpellMagic {
                effects: vec![],
                ..base.clone()
            },
            SpellMagic {
                side: TargetSide::Enemy,
                ..base.clone()
            },
            SpellMagic {
                effects: vec![SpellEffect::Timed(TimedChange::Speed)],
                ..base.clone()
            },
            SpellMagic {
                mind: true,
                ..base.clone()
            },
            SpellMagic {
                mass_from: Some(3),
                ..base.clone()
            },
            SpellMagic {
                duration: EffectDuration::Permanent,
                ..base.clone()
            },
            SpellMagic {
                cumulative: true,
                ..base.clone()
            },
            SpellMagic {
                any_side_from: Some(2),
                ..base.clone()
            },
            SpellMagic {
                needs_effect: true,
                ..base.clone()
            },
            SpellMagic {
                ignores_immunity: true,
                ..base.clone()
            },
            SpellMagic {
                effects: vec![SpellEffect::Dispel],
                ..base.clone()
            },
            SpellMagic {
                effects: vec![SpellEffect::Heal],
                ..base.clone()
            },
            SpellMagic {
                specialist: Some(SpecialistBonus::CreatureLevel([3, 3, 2, 2, 1, 1, 0])),
                ..base.clone()
            },
            SpellMagic {
                specialist: Some(SpecialistBonus::CreatureLevel([4, 3, 2, 1, 0, 0, 0])),
                ..base.clone()
            },
            SpellMagic {
                specialist: Some(SpecialistBonus::Fixed(2)),
                ..base.clone()
            },
            SpellMagic {
                specialist: Some(SpecialistBonus::Total(2)),
                ..base.clone()
            },
        ];
        let timed = |c| SpellMagic {
            effects: vec![SpellEffect::Timed(c)],
            ..base.clone()
        };
        assert_ne!(
            digest(&timed(TimedChange::Speed)),
            digest(&timed(TimedChange::SpeedPercent))
        );
        for v in &variants {
            assert_ne!(digest(v), digest(&base), "{v:?}");
        }
    }
}
