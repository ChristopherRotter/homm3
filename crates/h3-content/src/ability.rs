//! Creature abilities: the closed set of typed behaviour primitives a creature type can have
//! (ADR-0012 §1, ADR-0006 Tier 3).
//!
//! Abilities complement the creature flags (`CreatureFlag`, the yes/no properties the original
//! executable stores as bits): an ability carries parameters or describes behaviour the original
//! hard-codes per creature. Packs name abilities with their parameters; the compiler validates
//! them and resolves creature references. Rule code reads them where the rule applies (E03).
//!
//! [`Ability`] is generic over how creature types (`C`), spells (`S`) and spell schools (`L`) are
//! referenced: [`ContentKey`]s in packs ([`AbilityDef`]), handles once compiled
//! ([`CompiledAbility`]).
//!
//! [`ContentKey`]: crate::ContentKey

use h3_kernel::{CanonicalHash, StateHasher};

use crate::heroes::{SchoolId, SpellId};
use crate::key::ContentKey;
use crate::ruleset::CreatureTypeId;

/// Which stacks a battle-wide aura affects.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AuraTarget {
    /// The stacks of the bearer's side.
    Allies,
    /// The stacks of the other side.
    Enemies,
}

/// What an area shot does to the stacks around its target (the target itself is always hit).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AreaShot {
    /// Fireball (Magog): spares fire-immune stacks around the target.
    Fireball,
    /// Death cloud (Lich): spares stacks that are not alive (undead, golems, elementals, …)
    /// around the target.
    DeathCloud,
}

/// One ability. `C`, `S` and `L` are how creature types, spells and spell schools are referenced.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ability<C, S, L> {
    /// Deals `percent` % more base damage to `target` (hate, opposite elementals).
    Hate {
        /// The hated creature type.
        target: C,
        /// Extra damage in percent of the base damage.
        percent: u16,
    },
    /// Deals `percent_per_hex` % more base damage per hex travelled before a melee attack.
    Jousting {
        /// Extra damage per hex, percent.
        percent_per_hex: u16,
    },
    /// Attackers get no jousting bonus against this creature.
    ChargeImmunity,
    /// The target's defense counts only `100 - percent` % (rounded down) against this creature.
    DefenseReduction {
        /// Ignored share of the target's defense, percent (1..=100).
        percent: u8,
    },
    /// Chance per attack to deal +100 % base damage.
    DeathBlow {
        /// Chance in percent (1..=100).
        chance_percent: u8,
    },
    /// Heals (and resurrects) itself by `percent` % of the damage dealt to living targets.
    LifeDrain {
        /// Healed share of the damage, percent.
        percent: u16,
    },
    /// The top creature regains all lost hit points at the start of the stack's turn.
    Regeneration,
    /// After attacking or retaliating, each creature has `chance_percent` % to kill one living
    /// enemy creature, at most `ceil(count · chance_percent / 100)` kills.
    DeathStare {
        /// Chance per creature in percent (1..=100).
        chance_percent: u8,
    },
    /// Retaliates `1 + count` times per round.
    ExtraRetaliations {
        /// Additional retaliations per round (≥ 1).
        count: u8,
    },
    /// Retaliates against every attack.
    UnlimitedRetaliations,
    /// Returns to its starting hex after a melee attack.
    StrikeAndReturn,
    /// Its melee attack hits every adjacent enemy stack.
    AttacksAllAdjacent,
    /// Its melee attack hits the target and the two hexes beside it towards the attacker.
    ThreeHeadedAttack,
    /// Moves by teleporting: like flying, the path does not matter.
    Teleports,
    /// Changes the morale of every stack of the affected side for the whole battle.
    MoraleAura {
        /// Morale change.
        amount: i8,
        /// Whose morale.
        affects: AuraTarget,
    },
    /// Changes the luck of every stack of the affected side for the whole battle.
    LuckAura {
        /// Luck change.
        amount: i8,
        /// Whose luck.
        affects: AuraTarget,
    },
    /// Its morale is at least `value`.
    MinimumMorale {
        /// Lowest morale.
        value: i8,
    },
    /// Its luck is at least `value`.
    MinimumLuck {
        /// Lowest luck.
        value: i8,
    },
    /// Its shots have no distance penalty.
    NoDistancePenalty,
    /// Its shots also hit the stacks around the target.
    AreaShot {
        /// Who is spared around the target.
        effect: AreaShot,
    },
    /// War machine: on its turn it heals the top creature of an allied stack by a uniform
    /// `min_hp..=max_hp` hit points (First Aid Tent; BAT-WAR-003).
    FirstAid {
        /// Least hit points healed (≥ 1).
        min_hp: u16,
        /// Most hit points healed (≥ `min_hp`).
        max_hp: u16,
    },
    /// War machine: while it lives, its side's shots use no ammunition; it never acts (Ammo
    /// Cart; BAT-WAR-004).
    AmmoSupply,
    /// Immune to spells of level `max_level` and below (dragons, Magic Elemental; BAT-MAG-005).
    SpellLevelImmunity {
        /// Highest level it is immune to (1..=5; 5: every spell).
        max_level: u8,
    },
    /// Immune to the spells of a school (fire immunity; BAT-MAG-005).
    SchoolImmunity {
        /// The school.
        school: L,
    },
    /// Immune to one spell (one entry per spell; elementals; BAT-MAG-005).
    SpellImmunity {
        /// The spell.
        spell: S,
    },
    /// Takes `percent` % more damage from one spell (one entry per spell; elementals;
    /// BAT-MAG-004).
    SpellVulnerability {
        /// The spell.
        spell: S,
        /// Extra damage, percent of the damage (1..=1000).
        percent: u16,
    },
    /// Damage from spells reduced by `percent` % (Golems; BAT-MAG-004).
    SpellDamageReduction {
        /// Reduction, percent (1..=100).
        percent: u8,
    },
    /// Resists hostile spells with `percent` % chance (Dwarves, Crystal Dragon; BAT-MAG-008).
    MagicResistance {
        /// Chance, percent (1..=100).
        percent: u8,
    },
    /// Adjacent allied stacks (not itself) resist hostile spells, the chance to be affected
    /// multiplied by `100 − percent` % once however many such neighbours (Unicorns; BAT-MAG-008).
    MagicResistanceAura {
        /// Resistance, percent (1..=100).
        percent: u8,
    },
    /// After each of its melee strikes (attacks, retaliations, second strikes; never shots),
    /// casts `spell` on the struck stack as a creature's cast without a hero (Unicorns' Blind,
    /// Black Knights' Curse, the flies' Dispel Helpful Spells and Weakness, BAT-MAG-040; the
    /// Dendroids' Bind and the Rust Dragon's Acid Breath, BAT-MAG-045, BAT-MAG-046). The battle
    /// applies a creature's strike abilities in list order: list the original's pending spells
    /// (its 0x440500 table) before the striker's own effects (0x440BC0: the Thunderbird's bolt, the
    /// flies, the Rust Dragon's extra damage), as the harvested packs do (`installation.md` §7.20).
    SpellAfterStrike {
        /// The spell.
        spell: S,
        /// Chance per strike in percent (1..=100; 100: no draw).
        chance_percent: u8,
        /// The mastery it is cast at (0..=3).
        mastery: u8,
        /// The rounds its timed effects last (≥ 1; the original's power argument, 3).
        rounds: u8,
        /// Only on a stack of a living creature (flag `alive`).
        living: bool,
        /// Not on a stack the spell already affects.
        unless_active: bool,
        /// Only on a stack whose defense is above 0 (Acid Breath; v21).
        defended: bool,
    },
    /// After each of its melee strikes, deals `per_creature` damage per creature of its stack to
    /// the struck stack as `spell` without a hero (the Thunderbird's thunderbolt as Lightning
    /// Bolt; BAT-MAG-041).
    SpellDamageAfterStrike {
        /// The damage spell whose tests and target factors apply.
        spell: S,
        /// Chance per strike in percent (1..=100).
        chance_percent: u8,
        /// Damage per creature (≥ 1).
        per_creature: u16,
    },
    /// After each of its melee strikes, deals `percent` % of one roll of its damage range per
    /// creature of its stack to the struck stack, without a test (the Rust Dragon's extra damage;
    /// BAT-MAG-047, v21).
    DamageAfterStrike {
        /// Chance per strike in percent (1..=100).
        chance_percent: u8,
        /// The share of the roll per creature, percent (1..=100).
        percent: u8,
    },
    /// Casts `spell`, aimed at a stack, as its stack's action, as many times a battle as its
    /// creature's `casts`: a creature's cast at `mastery` with `power` (the Ogre Mage's Bloodlust,
    /// the AB elementals' Protections at Advanced with power 6; BAT-MAG-048, v22).
    SpellCaster {
        /// The spell (it must take a target stack at `mastery`).
        spell: S,
        /// The mastery it is cast at (0..=3).
        mastery: u8,
        /// The caster's power: the rounds of its timed effects and its value's power (≥ 1).
        power: u8,
    },
}

/// An ability as written in a pack (references are keys).
pub type AbilityDef = Ability<ContentKey, ContentKey, ContentKey>;

/// A compiled ability (references are handles).
pub type CompiledAbility = Ability<CreatureTypeId, SpellId, SchoolId>;

impl<C, S, L> Ability<C, S, L> {
    /// Every ability name, in code order (the order of [`Ability::code`]).
    pub const NAMES: [&'static str; 33] = [
        "hate",
        "jousting",
        "charge_immunity",
        "defense_reduction",
        "death_blow",
        "life_drain",
        "regeneration",
        "death_stare",
        "extra_retaliations",
        "unlimited_retaliations",
        "strike_and_return",
        "attacks_all_adjacent",
        "three_headed_attack",
        "teleports",
        "morale_aura",
        "luck_aura",
        "minimum_morale",
        "minimum_luck",
        "no_distance_penalty",
        "area_shot",
        "first_aid",
        "ammo_supply",
        "spell_level_immunity",
        "school_immunity",
        "spell_immunity",
        "spell_vulnerability",
        "spell_damage_reduction",
        "magic_resistance",
        "magic_resistance_aura",
        "spell_after_strike",
        "spell_damage_after_strike",
        "damage_after_strike",
        "spell_caster",
    ];

    /// Stable numeric code of the ability kind (canonical hashing, duplicate checks).
    pub const fn code(&self) -> u8 {
        match self {
            Self::Hate { .. } => 0,
            Self::Jousting { .. } => 1,
            Self::ChargeImmunity => 2,
            Self::DefenseReduction { .. } => 3,
            Self::DeathBlow { .. } => 4,
            Self::LifeDrain { .. } => 5,
            Self::Regeneration => 6,
            Self::DeathStare { .. } => 7,
            Self::ExtraRetaliations { .. } => 8,
            Self::UnlimitedRetaliations => 9,
            Self::StrikeAndReturn => 10,
            Self::AttacksAllAdjacent => 11,
            Self::ThreeHeadedAttack => 12,
            Self::Teleports => 13,
            Self::MoraleAura { .. } => 14,
            Self::LuckAura { .. } => 15,
            Self::MinimumMorale { .. } => 16,
            Self::MinimumLuck { .. } => 17,
            Self::NoDistancePenalty => 18,
            Self::AreaShot { .. } => 19,
            Self::FirstAid { .. } => 20,
            Self::AmmoSupply => 21,
            Self::SpellLevelImmunity { .. } => 22,
            Self::SchoolImmunity { .. } => 23,
            Self::SpellImmunity { .. } => 24,
            Self::SpellVulnerability { .. } => 25,
            Self::SpellDamageReduction { .. } => 26,
            Self::MagicResistance { .. } => 27,
            Self::MagicResistanceAura { .. } => 28,
            Self::SpellAfterStrike { .. } => 29,
            Self::SpellDamageAfterStrike { .. } => 30,
            Self::DamageAfterStrike { .. } => 31,
            Self::SpellCaster { .. } => 32,
        }
    }

    /// For morale and luck auras, a key that is equal exactly for equal aura values (kind,
    /// amount, affected side): auras with equal keys do not stack (BAT-MOR-001, BAT-LCK-001).
    pub const fn aura_key(&self) -> Option<u32> {
        let (amount, affects) = match self {
            Self::MoraleAura { amount, affects } | Self::LuckAura { amount, affects } => {
                (*amount, *affects)
            }
            _ => return None,
        };
        let side = match affects {
            AuraTarget::Allies => 0,
            AuraTarget::Enemies => 1,
        };
        Some((self.code() as u32) << 16 | side << 8 | amount as u8 as u32)
    }

    /// The name used in packs.
    pub fn name(&self) -> &'static str {
        Self::NAMES[usize::from(self.code())]
    }

    /// The same ability with its creature, spell and school references mapped (fails if a
    /// mapping fails).
    pub fn try_map<C2, S2, L2, E>(
        &self,
        creature: impl FnOnce(&C) -> Result<C2, E>,
        spell: impl FnOnce(&S) -> Result<S2, E>,
        school: impl FnOnce(&L) -> Result<L2, E>,
    ) -> Result<Ability<C2, S2, L2>, E> {
        Ok(match self {
            Self::Hate { target, percent } => Ability::Hate {
                target: creature(target)?,
                percent: *percent,
            },
            Self::Jousting { percent_per_hex } => Ability::Jousting {
                percent_per_hex: *percent_per_hex,
            },
            Self::ChargeImmunity => Ability::ChargeImmunity,
            Self::DefenseReduction { percent } => Ability::DefenseReduction { percent: *percent },
            Self::DeathBlow { chance_percent } => Ability::DeathBlow {
                chance_percent: *chance_percent,
            },
            Self::LifeDrain { percent } => Ability::LifeDrain { percent: *percent },
            Self::Regeneration => Ability::Regeneration,
            Self::DeathStare { chance_percent } => Ability::DeathStare {
                chance_percent: *chance_percent,
            },
            Self::ExtraRetaliations { count } => Ability::ExtraRetaliations { count: *count },
            Self::UnlimitedRetaliations => Ability::UnlimitedRetaliations,
            Self::StrikeAndReturn => Ability::StrikeAndReturn,
            Self::AttacksAllAdjacent => Ability::AttacksAllAdjacent,
            Self::ThreeHeadedAttack => Ability::ThreeHeadedAttack,
            Self::Teleports => Ability::Teleports,
            Self::MoraleAura { amount, affects } => Ability::MoraleAura {
                amount: *amount,
                affects: *affects,
            },
            Self::LuckAura { amount, affects } => Ability::LuckAura {
                amount: *amount,
                affects: *affects,
            },
            Self::MinimumMorale { value } => Ability::MinimumMorale { value: *value },
            Self::MinimumLuck { value } => Ability::MinimumLuck { value: *value },
            Self::NoDistancePenalty => Ability::NoDistancePenalty,
            Self::AreaShot { effect } => Ability::AreaShot { effect: *effect },
            Self::FirstAid { min_hp, max_hp } => Ability::FirstAid {
                min_hp: *min_hp,
                max_hp: *max_hp,
            },
            Self::AmmoSupply => Ability::AmmoSupply,
            Self::SpellLevelImmunity { max_level } => Ability::SpellLevelImmunity {
                max_level: *max_level,
            },
            Self::SchoolImmunity { school: l } => Ability::SchoolImmunity { school: school(l)? },
            Self::SpellImmunity { spell: x } => Ability::SpellImmunity { spell: spell(x)? },
            Self::SpellVulnerability { spell: x, percent } => Ability::SpellVulnerability {
                spell: spell(x)?,
                percent: *percent,
            },
            Self::SpellDamageReduction { percent } => {
                Ability::SpellDamageReduction { percent: *percent }
            }
            Self::MagicResistance { percent } => Ability::MagicResistance { percent: *percent },
            Self::MagicResistanceAura { percent } => {
                Ability::MagicResistanceAura { percent: *percent }
            }
            Self::SpellAfterStrike {
                spell: x,
                chance_percent,
                mastery,
                rounds,
                living,
                unless_active,
                defended,
            } => Ability::SpellAfterStrike {
                spell: spell(x)?,
                chance_percent: *chance_percent,
                mastery: *mastery,
                rounds: *rounds,
                living: *living,
                unless_active: *unless_active,
                defended: *defended,
            },
            Self::SpellDamageAfterStrike {
                spell: x,
                chance_percent,
                per_creature,
            } => Ability::SpellDamageAfterStrike {
                spell: spell(x)?,
                chance_percent: *chance_percent,
                per_creature: *per_creature,
            },
            Self::DamageAfterStrike {
                chance_percent,
                percent,
            } => Ability::DamageAfterStrike {
                chance_percent: *chance_percent,
                percent: *percent,
            },
            Self::SpellCaster {
                spell: x,
                mastery,
                power,
            } => Ability::SpellCaster {
                spell: spell(x)?,
                mastery: *mastery,
                power: *power,
            },
        })
    }

    /// Why the parameters are invalid, if they are (the compiler's per-ability check).
    pub fn parameter_error(&self) -> Option<&'static str> {
        match self {
            Self::Hate { percent: 0, .. } => Some("hate percent must be at least 1"),
            Self::Jousting { percent_per_hex: 0 } => {
                Some("jousting percent_per_hex must be at least 1")
            }
            Self::DefenseReduction { percent } if !(1..=100).contains(percent) => {
                Some("defense_reduction percent must be 1..=100")
            }
            Self::DeathBlow { chance_percent }
            | Self::DeathStare { chance_percent }
            | Self::SpellAfterStrike { chance_percent, .. }
            | Self::SpellDamageAfterStrike { chance_percent, .. }
            | Self::DamageAfterStrike { chance_percent, .. }
                if !(1..=100).contains(chance_percent) =>
            {
                Some("chance_percent must be 1..=100")
            }
            Self::LifeDrain { percent } if !(1..=1000).contains(percent) => {
                Some("life_drain percent must be 1..=1000")
            }
            Self::ExtraRetaliations { count: 0 } => {
                Some("extra_retaliations count must be at least 1")
            }
            Self::MoraleAura { amount: 0, .. } | Self::LuckAura { amount: 0, .. } => {
                Some("aura amount must not be 0")
            }
            Self::FirstAid { min_hp, max_hp } if *min_hp == 0 || max_hp < min_hp => {
                Some("first_aid needs 1 <= min_hp <= max_hp")
            }
            Self::SpellAfterStrike { mastery, .. } if *mastery > 3 => {
                Some("spell_after_strike mastery must be 0..=3")
            }
            Self::SpellAfterStrike { rounds: 0, .. } => {
                Some("spell_after_strike rounds must be at least 1")
            }
            Self::SpellCaster { mastery, .. } if *mastery > 3 => {
                Some("spell_caster mastery must be 0..=3")
            }
            Self::SpellCaster { power: 0, .. } => Some("spell_caster power must be at least 1"),
            Self::SpellDamageAfterStrike {
                per_creature: 0, ..
            } => Some("spell_damage_after_strike per_creature must be at least 1"),
            Self::SpellLevelImmunity { max_level } if !(1..=5).contains(max_level) => {
                Some("spell_level_immunity max_level must be 1..=5")
            }
            Self::SpellVulnerability { percent, .. } if !(1..=1000).contains(percent) => {
                Some("spell_vulnerability percent must be 1..=1000")
            }
            Self::SpellDamageReduction { percent }
            | Self::MagicResistance { percent }
            | Self::MagicResistanceAura { percent }
            | Self::DamageAfterStrike { percent, .. }
                if !(1..=100).contains(percent) =>
            {
                Some("percent must be 1..=100")
            }
            _ => None,
        }
    }
}

const fn signed(v: i8) -> u8 {
    v.to_le_bytes()[0]
}

impl CanonicalHash for CompiledAbility {
    /// Encoding: kind code (u8), then the parameters in field order: creature, spell and school
    /// handles as their handle encoding, `u8`/`u16` as such, `i8` as its two's-complement byte, `affects`
    /// (allies 0, enemies 1) and `effect` (fireball 0, death cloud 1) as one byte.
    fn hash_into(&self, h: &mut StateHasher) {
        h.write_u8(self.code());
        match *self {
            Self::Hate { target, percent } => {
                target.hash_into(h);
                h.write_u16(percent);
            }
            Self::Jousting { percent_per_hex } => h.write_u16(percent_per_hex),
            Self::DefenseReduction { percent } => h.write_u8(percent),
            Self::DeathBlow { chance_percent } | Self::DeathStare { chance_percent } => {
                h.write_u8(chance_percent);
            }
            Self::LifeDrain { percent } => h.write_u16(percent),
            Self::ExtraRetaliations { count } => h.write_u8(count),
            Self::MoraleAura { amount, affects } | Self::LuckAura { amount, affects } => {
                h.write_u8(signed(amount));
                h.write_u8(match affects {
                    AuraTarget::Allies => 0,
                    AuraTarget::Enemies => 1,
                });
            }
            Self::MinimumMorale { value } | Self::MinimumLuck { value } => {
                h.write_u8(signed(value))
            }
            Self::AreaShot { effect } => h.write_u8(match effect {
                AreaShot::Fireball => 0,
                AreaShot::DeathCloud => 1,
            }),
            Self::FirstAid { min_hp, max_hp } => {
                h.write_u16(min_hp);
                h.write_u16(max_hp);
            }
            Self::SpellLevelImmunity { max_level } => h.write_u8(max_level),
            Self::SchoolImmunity { school } => school.hash_into(h),
            Self::SpellImmunity { spell } => spell.hash_into(h),
            Self::SpellVulnerability { spell, percent } => {
                spell.hash_into(h);
                h.write_u16(percent);
            }
            Self::SpellDamageReduction { percent }
            | Self::MagicResistance { percent }
            | Self::MagicResistanceAura { percent } => h.write_u8(percent),
            Self::SpellAfterStrike {
                spell,
                chance_percent,
                mastery,
                rounds,
                living,
                unless_active,
                defended,
            } => {
                spell.hash_into(h);
                h.write_bytes(&[chance_percent, mastery, rounds]);
                h.write_bool(living);
                h.write_bool(unless_active);
                h.write_bool(defended);
            }
            Self::SpellDamageAfterStrike {
                spell,
                chance_percent,
                per_creature,
            } => {
                spell.hash_into(h);
                h.write_u8(chance_percent);
                h.write_u16(per_creature);
            }
            Self::DamageAfterStrike {
                chance_percent,
                percent,
            } => h.write_bytes(&[chance_percent, percent]),
            Self::SpellCaster {
                spell,
                mastery,
                power,
            } => {
                spell.hash_into(h);
                h.write_bytes(&[mastery, power]);
            }
            Self::ChargeImmunity
            | Self::Regeneration
            | Self::UnlimitedRetaliations
            | Self::StrikeAndReturn
            | Self::AttacksAllAdjacent
            | Self::ThreeHeadedAttack
            | Self::Teleports
            | Self::NoDistancePenalty
            | Self::AmmoSupply => {}
        }
    }
}

/// Checks a creature's whole ability list: per-ability parameters, at most one ability of each
/// kind (several `hate`, `school_immunity`, `spell_immunity`, `spell_vulnerability` and
/// `spell_after_strike` entries are allowed, for different targets, schools and spells), and
/// mutually exclusive kinds.
/// Returns the reason for the first problem.
pub(crate) fn list_error<C: PartialEq, S: PartialEq, L: PartialEq>(
    list: &[Ability<C, S, L>],
) -> Option<&'static str> {
    if let Some(e) = list.iter().find_map(Ability::parameter_error) {
        return Some(e);
    }
    for (i, a) in list.iter().enumerate() {
        let earlier = list.get(..i).unwrap_or(&[]);
        let duplicate = earlier.iter().any(|b| match (a, b) {
            (Ability::Hate { target: x, .. }, Ability::Hate { target: y, .. }) => x == y,
            (Ability::SchoolImmunity { school: x }, Ability::SchoolImmunity { school: y }) => {
                x == y
            }
            (Ability::SpellImmunity { spell: x }, Ability::SpellImmunity { spell: y })
            | (
                Ability::SpellVulnerability { spell: x, .. },
                Ability::SpellVulnerability { spell: y, .. },
            )
            | (
                Ability::SpellAfterStrike { spell: x, .. },
                Ability::SpellAfterStrike { spell: y, .. },
            ) => x == y,
            _ => a.code() == b.code(),
        });
        if duplicate {
            return Some("an ability is listed twice");
        }
    }
    let has = |code: u8| list.iter().any(|a| a.code() == code);
    if has(8) && has(9) {
        return Some("extra_retaliations and unlimited_retaliations exclude each other");
    }
    if has(11) && has(12) {
        return Some("attacks_all_adjacent and three_headed_attack exclude each other");
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    type A = Ability<u8, u8, u8>;

    fn all() -> [A; 33] {
        [
            A::Hate {
                target: 3,
                percent: 50,
            },
            A::Jousting { percent_per_hex: 5 },
            A::ChargeImmunity,
            A::DefenseReduction { percent: 40 },
            A::DeathBlow { chance_percent: 20 },
            A::LifeDrain { percent: 100 },
            A::Regeneration,
            A::DeathStare { chance_percent: 10 },
            A::ExtraRetaliations { count: 1 },
            A::UnlimitedRetaliations,
            A::StrikeAndReturn,
            A::AttacksAllAdjacent,
            A::ThreeHeadedAttack,
            A::Teleports,
            A::MoraleAura {
                amount: 1,
                affects: AuraTarget::Allies,
            },
            A::LuckAura {
                amount: -1,
                affects: AuraTarget::Enemies,
            },
            A::MinimumMorale { value: 1 },
            A::MinimumLuck { value: 1 },
            A::NoDistancePenalty,
            A::AreaShot {
                effect: AreaShot::DeathCloud,
            },
            A::FirstAid {
                min_hp: 1,
                max_hp: 25,
            },
            A::AmmoSupply,
            A::SpellLevelImmunity { max_level: 3 },
            A::SchoolImmunity { school: 2 },
            A::SpellImmunity { spell: 4 },
            A::SpellVulnerability {
                spell: 5,
                percent: 100,
            },
            A::SpellDamageReduction { percent: 50 },
            A::MagicResistance { percent: 20 },
            A::MagicResistanceAura { percent: 20 },
            A::SpellAfterStrike {
                spell: 6,
                chance_percent: 20,
                mastery: 0,
                rounds: 3,
                living: true,
                unless_active: false,
                defended: false,
            },
            A::SpellDamageAfterStrike {
                spell: 7,
                chance_percent: 20,
                per_creature: 10,
            },
            A::DamageAfterStrike {
                chance_percent: 20,
                percent: 50,
            },
            A::SpellCaster {
                spell: 8,
                mastery: 2,
                power: 6,
            },
        ]
    }

    #[test]
    fn codes_names_and_mapping() {
        for (i, a) in all().iter().enumerate() {
            assert_eq!(usize::from(a.code()), i);
            assert_eq!(a.name(), A::NAMES[i]);
            assert_eq!(a.parameter_error(), None, "{a:?}");
            let up = |t: &u8| Ok::<u16, ()>(u16::from(*t) + 1);
            let mapped = a.try_map(up, up, up).unwrap();
            assert_eq!(mapped.code(), a.code());
            let down = |t: &u16| u8::try_from(*t - 1);
            let back = mapped.try_map(down, down, down).unwrap();
            assert_eq!(&back, a);
        }
        let hate = A::Hate {
            target: 1,
            percent: 50,
        };
        let fail = |_: &u8| Err::<u8, _>("nope");
        let ok = |t: &u8| Ok::<u8, &str>(*t);
        assert_eq!(hate.try_map(fail, ok, ok), Err("nope"));
        assert_eq!(
            A::SpellImmunity { spell: 1 }.try_map(ok, fail, ok),
            Err("nope")
        );
        assert_eq!(
            A::SpellVulnerability {
                spell: 1,
                percent: 100
            }
            .try_map(ok, fail, ok),
            Err("nope")
        );
        assert_eq!(
            A::SchoolImmunity { school: 1 }.try_map(ok, ok, fail),
            Err("nope")
        );
        for a in &all()[29..31] {
            assert_eq!(a.try_map(ok, fail, ok), Err("nope"));
        }
        assert_eq!(
            A::SchoolImmunity { school: 1 }.try_map(fail, fail, ok),
            Ok(A::SchoolImmunity { school: 1 })
        );
    }

    #[test]
    fn hash_encoding() {
        use h3_kernel::hash::hash_of;
        let id = CreatureTypeId::from_raw(7);
        let mut seen = alloc::collections::BTreeSet::new();
        for a in all() {
            let compiled = a
                .try_map(
                    |_| Ok::<_, ()>(id),
                    |&s| Ok(SpellId::from_raw(s.into())),
                    |&l| Ok(SchoolId::from_raw(l.into())),
                )
                .unwrap();
            assert!(seen.insert(hash_of(&compiled)), "{compiled:?}");
        }
        let fireball = CompiledAbility::AreaShot {
            effect: AreaShot::Fireball,
        };
        assert!(seen.insert(hash_of(&fireball)));
        let allies_luck = CompiledAbility::LuckAura {
            amount: -1,
            affects: AuraTarget::Allies,
        };
        assert!(seen.insert(hash_of(&allies_luck)));
        let mut h = StateHasher::new();
        h.write_u8(16);
        h.write_u8(0xff);
        assert_eq!(
            hash_of(&CompiledAbility::MinimumMorale { value: -1 }),
            h.finish()
        );
        let mut h = StateHasher::new();
        h.write_u8(0);
        id.hash_into(&mut h);
        h.write_u16(50);
        let hate = CompiledAbility::Hate {
            target: id,
            percent: 50,
        };
        assert_eq!(hash_of(&hate), h.finish());
    }

    #[test]
    fn parameter_errors() {
        let bad = [
            A::Hate {
                target: 0,
                percent: 0,
            },
            A::Jousting { percent_per_hex: 0 },
            A::DefenseReduction { percent: 0 },
            A::DefenseReduction { percent: 101 },
            A::DeathBlow { chance_percent: 0 },
            A::DeathStare {
                chance_percent: 101,
            },
            A::LifeDrain { percent: 0 },
            A::LifeDrain { percent: 1001 },
            A::ExtraRetaliations { count: 0 },
            A::MoraleAura {
                amount: 0,
                affects: AuraTarget::Allies,
            },
            A::LuckAura {
                amount: 0,
                affects: AuraTarget::Enemies,
            },
            A::FirstAid {
                min_hp: 0,
                max_hp: 25,
            },
            A::FirstAid {
                min_hp: 5,
                max_hp: 4,
            },
            A::SpellLevelImmunity { max_level: 0 },
            A::SpellLevelImmunity { max_level: 6 },
            A::SpellVulnerability {
                spell: 0,
                percent: 0,
            },
            A::SpellVulnerability {
                spell: 0,
                percent: 1001,
            },
            A::SpellDamageReduction { percent: 0 },
            A::MagicResistance { percent: 101 },
            A::MagicResistanceAura { percent: 0 },
            A::SpellAfterStrike {
                spell: 0,
                chance_percent: 0,
                mastery: 0,
                rounds: 3,
                living: false,
                unless_active: false,
                defended: false,
            },
            A::SpellAfterStrike {
                spell: 0,
                chance_percent: 100,
                mastery: 4,
                rounds: 3,
                living: false,
                unless_active: false,
                defended: false,
            },
            A::SpellAfterStrike {
                spell: 0,
                chance_percent: 100,
                mastery: 3,
                rounds: 0,
                living: false,
                unless_active: false,
                defended: false,
            },
            A::SpellDamageAfterStrike {
                spell: 0,
                chance_percent: 101,
                per_creature: 10,
            },
            A::SpellDamageAfterStrike {
                spell: 0,
                chance_percent: 20,
                per_creature: 0,
            },
            A::DamageAfterStrike {
                chance_percent: 0,
                percent: 50,
            },
            A::DamageAfterStrike {
                chance_percent: 20,
                percent: 101,
            },
            A::SpellCaster {
                spell: 0,
                mastery: 4,
                power: 6,
            },
            A::SpellCaster {
                spell: 0,
                mastery: 2,
                power: 0,
            },
        ];
        for a in bad {
            assert!(a.parameter_error().is_some(), "{a:?}");
            assert!(list_error(&[a]).is_some());
        }
    }

    #[test]
    fn list_rules() {
        assert_eq!(list_error::<u8, u8, u8>(&[]), None);
        assert_eq!(list_error(&all()[..8]), None);
        let hate = |target| A::Hate {
            target,
            percent: 50,
        };
        assert_eq!(list_error(&[hate(1), hate(2)]), None);
        // Immunities and vulnerabilities to different spells and schools may stack.
        let immune = |spell| A::SpellImmunity { spell };
        let weak = |spell| A::SpellVulnerability {
            spell,
            percent: 100,
        };
        let school = |school| A::SchoolImmunity { school };
        assert_eq!(
            list_error(&[immune(1), immune(2), weak(1), weak(2), school(1), school(2)]),
            None
        );
        // The Dragon Fly's two spells after a strike.
        let strike = |spell| A::SpellAfterStrike {
            spell,
            chance_percent: 100,
            mastery: 0,
            rounds: 3,
            living: false,
            unless_active: false,
            defended: false,
        };
        assert_eq!(list_error(&[strike(1), strike(2)]), None);
        for twice in [
            [immune(1), immune(1)],
            [weak(3), weak(3)],
            [school(1), school(1)],
            [strike(4), strike(4)],
        ] {
            assert_eq!(list_error(&twice), Some("an ability is listed twice"));
        }
        assert_eq!(
            list_error(&[hate(1), hate(1)]),
            Some("an ability is listed twice")
        );
        assert_eq!(
            list_error(&[A::Teleports, A::ChargeImmunity, A::Teleports]),
            Some("an ability is listed twice")
        );
        assert!(
            list_error(&[A::ExtraRetaliations { count: 1 }, A::UnlimitedRetaliations])
                .unwrap()
                .contains("exclude")
        );
        assert!(
            list_error(&[A::ThreeHeadedAttack, A::AttacksAllAdjacent])
                .unwrap()
                .contains("exclude")
        );
    }
}
