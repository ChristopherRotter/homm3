//! Creature abilities: the closed set of typed behaviour primitives a creature type can have
//! (ADR-0012 §1, ADR-0006 Tier 3).
//!
//! Abilities complement the creature flags (`CreatureFlag`, the yes/no properties the original
//! executable stores as bits): an ability carries parameters or describes behaviour the original
//! hard-codes per creature. Packs name abilities with their parameters; the compiler validates
//! them and resolves creature references. Rule code reads them where the rule applies (E03).
//!
//! [`Ability`] is generic over how creature types are referenced: [`ContentKey`] in packs
//! ([`AbilityDef`]), [`CreatureTypeId`] once compiled.
//!
//! [`ContentKey`]: crate::ContentKey
//! [`CreatureTypeId`]: crate::CreatureTypeId

use h3_kernel::{CanonicalHash, StateHasher};

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

/// One ability. `C` is how creature types are referenced.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ability<C> {
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
}

/// An ability as written in a pack (creature references are keys).
pub type AbilityDef = Ability<ContentKey>;

/// A compiled ability (creature references are handles).
pub type CompiledAbility = Ability<CreatureTypeId>;

impl<C> Ability<C> {
    /// Every ability name, in code order (the order of [`Ability::code`]).
    pub const NAMES: [&'static str; 22] = [
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

    /// The same ability with creature references mapped by `f` (fails if `f` fails).
    pub fn try_map<D, E>(&self, f: impl FnOnce(&C) -> Result<D, E>) -> Result<Ability<D>, E> {
        Ok(match self {
            Self::Hate { target, percent } => Ability::Hate {
                target: f(target)?,
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
            Self::DeathBlow { chance_percent } | Self::DeathStare { chance_percent }
                if !(1..=100).contains(chance_percent) =>
            {
                Some("chance_percent must be 1..=100")
            }
            Self::LifeDrain { percent: 0 } => Some("life_drain percent must be at least 1"),
            Self::ExtraRetaliations { count: 0 } => {
                Some("extra_retaliations count must be at least 1")
            }
            Self::MoraleAura { amount: 0, .. } | Self::LuckAura { amount: 0, .. } => {
                Some("aura amount must not be 0")
            }
            Self::FirstAid { min_hp, max_hp } if *min_hp == 0 || max_hp < min_hp => {
                Some("first_aid needs 1 <= min_hp <= max_hp")
            }
            _ => None,
        }
    }
}

const fn signed(v: i8) -> u8 {
    v.to_le_bytes()[0]
}

impl CanonicalHash for Ability<CreatureTypeId> {
    /// Encoding: kind code (u8), then the parameters in field order: creature handles as their
    /// handle encoding, `u8`/`u16` as such, `i8` as its two's-complement byte, `affects`
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
/// kind (several `hate` entries are allowed, for different targets), and mutually exclusive
/// kinds. Returns the reason for the first problem.
pub(crate) fn list_error<C: PartialEq>(list: &[Ability<C>]) -> Option<&'static str> {
    if let Some(e) = list.iter().find_map(Ability::parameter_error) {
        return Some(e);
    }
    for (i, a) in list.iter().enumerate() {
        let earlier = list.get(..i).unwrap_or(&[]);
        let duplicate = earlier.iter().any(|b| match (a, b) {
            (Ability::Hate { target: x, .. }, Ability::Hate { target: y, .. }) => x == y,
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

    type A = Ability<u8>;

    fn all() -> [A; 22] {
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
        ]
    }

    #[test]
    fn codes_names_and_mapping() {
        for (i, a) in all().iter().enumerate() {
            assert_eq!(usize::from(a.code()), i);
            assert_eq!(a.name(), A::NAMES[i]);
            assert_eq!(a.parameter_error(), None, "{a:?}");
            let mapped = a.try_map(|t| Ok::<u16, ()>(u16::from(*t) + 1)).unwrap();
            assert_eq!(mapped.code(), a.code());
            let back = mapped.try_map(|t| u8::try_from(*t - 1)).unwrap();
            assert_eq!(&back, a);
        }
        let hate = A::Hate {
            target: 1,
            percent: 50,
        };
        assert_eq!(hate.try_map(|_| Err::<u8, _>("nope")), Err("nope"));
    }

    #[test]
    fn hash_encoding() {
        use h3_kernel::hash::hash_of;
        let id = CreatureTypeId::from_raw(7);
        let mut seen = alloc::collections::BTreeSet::new();
        for a in all() {
            let compiled = a.try_map(|_| Ok::<_, ()>(id)).unwrap();
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
        ];
        for a in bad {
            assert!(a.parameter_error().is_some(), "{a:?}");
            assert!(list_error(&[a]).is_some());
        }
    }

    #[test]
    fn list_rules() {
        assert_eq!(list_error::<u8>(&[]), None);
        assert_eq!(list_error(&all()[..8]), None);
        let hate = |target| A::Hate {
            target,
            percent: 50,
        };
        assert_eq!(list_error(&[hate(1), hate(2)]), None);
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
