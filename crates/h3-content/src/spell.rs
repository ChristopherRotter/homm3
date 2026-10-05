//! What a spell does in battle (E04b, ADR-0014 point 1): the target its caster names, the stacks
//! the cast reaches (an area around a hex, a chain from a stack, or every stack), the conditions
//! a stack must meet, and its effects from a closed set of kinds (pack format v4, E04b T2a).
//!
//! Spells are data over these kinds: each kind is one battle rule (`BAT-MAG-…`), so a new spell of
//! a known kind is a data change. Kinds join with the task that implements them: damage in T2;
//! timed effects, dispel and heal in T3; and so on. A spell without effects cannot be cast.

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

/// One effect of a spell on each stack it reaches. The value of a cast is
/// `power × power factor + effect[mastery]` (the spell's data row), changed per target and caster
/// by the kind's rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SpellEffect {
    /// Deals the value as damage (BAT-MAG rules, E04b T2).
    Damage,
}

impl SpellEffect {
    /// All values, in code order.
    pub const ALL: [Self; 1] = [Self::Damage];

    /// Pack name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Damage => "damage",
        }
    }

    /// Looks a value up by its pack name.
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|e| e.name() == name)
    }
}

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
    /// The stacks it may affect.
    pub conditions: TargetConditions,
    /// What it does to each stack reached, distinct, in pack order (none: not castable yet).
    pub effects: alloc::vec::Vec<SpellEffect>,
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
        } else {
            None
        }
    }
}

impl CanonicalHash for SpellMagic {
    /// Target, area (presence, inner, radii), chain (presence, targets, factor), condition bits,
    /// effects (count, codes).
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
        h.write_u32(self.conditions.requires.bits());
        h.write_u32(self.conditions.excludes.bits());
        h.write_len(self.effects.len());
        for e in &self.effects {
            h.write_u8(*e as u8);
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
        for e in SpellEffect::ALL {
            assert_eq!(SpellEffect::from_name(e.name()), Some(e));
        }
        assert_eq!(SpellTarget::from_name("obstacle"), None);
        assert_eq!(SpellEffect::from_name("heal"), None);
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
        for (m, castable, reason) in bad {
            let error = m.error(castable).unwrap_or_default();
            assert!(error.contains(reason), "{m:?}: {error}");
        }
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
            conditions: TargetConditions::default(),
            effects: vec![SpellEffect::Damage],
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
        ];
        for v in &variants {
            assert_ne!(digest(v), digest(&base), "{v:?}");
        }
    }
}
