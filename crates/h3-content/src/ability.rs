//! A creature's abilities outside battle (pack format v50, E08 increment 9; ADR-0015 amendment
//! R01 point 1, E08 decision 201): a closed core kind beside the creature's growth and flags,
//! read by the game's day (ADR-0017 point 10), never by the battle, whose abilities are
//! `h3-content-battle`'s (ADR-0012).

use h3_kernel::{CanonicalHash, StateHasher};

use crate::effect::MAX_DAILY_AMOUNT;
use crate::key::ContentKey;
use crate::ruleset::ResourceId;

/// Most abilities outside battle of one creature type.
pub const MAX_CREATURE_ABILITIES: usize = 4;

/// One ability of a creature type outside battle. `R` is how resources are referenced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CreatureAbility<R> {
    /// On a week's first day, `amount` of `resource` to the player of a hero whose army holds the
    /// creature, once a type however many stacks and heroes (ADR-0017 point 10, GAM-RES-005; the
    /// Crystal Dragons' 3 crystal: `player_daily_income` 0x4B8C32 tests creature 133; VCMI's
    /// `SPECIAL_CRYSTAL_GENERATION`).
    WeeklyResource {
        /// The resource.
        resource: R,
        /// Amount a week (1 to [`MAX_DAILY_AMOUNT`]).
        amount: u32,
    },
}

/// An ability as written in a pack (the resource is a key).
pub type CreatureAbilityDef = CreatureAbility<ContentKey>;
/// An ability with its resource resolved.
pub type CompiledCreatureAbility = CreatureAbility<ResourceId>;

impl<R> CreatureAbility<R> {
    /// Pack names, indexed by [`CreatureAbility::code`].
    pub const NAMES: [&'static str; 1] = ["weekly_resource"];

    /// Stable code of the variant (canonical hash, duplicate checks).
    pub const fn code(&self) -> u8 {
        match self {
            Self::WeeklyResource { .. } => 0,
        }
    }

    /// Pack name of the variant.
    pub fn name(&self) -> &'static str {
        // `code` is always below `NAMES.len()`.
        Self::NAMES
            .get(usize::from(self.code()))
            .copied()
            .unwrap_or_default()
    }

    /// Maps the resource reference.
    pub fn try_map<R2, E>(
        &self,
        resource: impl FnOnce(&R) -> Result<R2, E>,
    ) -> Result<CreatureAbility<R2>, E> {
        Ok(match self {
            Self::WeeklyResource {
                resource: r,
                amount,
            } => CreatureAbility::WeeklyResource {
                resource: resource(r)?,
                amount: *amount,
            },
        })
    }

    /// Why the parameters are invalid, if they are.
    pub fn parameter_error(&self) -> Option<&'static str> {
        match self {
            Self::WeeklyResource { amount, .. } if !(1..=MAX_DAILY_AMOUNT).contains(amount) => {
                Some("weekly_resource amount must be 1..=2147483647")
            }
            Self::WeeklyResource { .. } => None,
        }
    }
}

impl<R: PartialEq> CreatureAbility<R> {
    /// Whether two abilities may not both appear in one list: weekly resources of one resource.
    fn collides(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self::WeeklyResource { resource: a, .. },
                Self::WeeklyResource { resource: b, .. },
            ) => a == b,
        }
    }
}

/// Why a creature's ability list is invalid, if it is: too long, invalid parameters or an
/// ability listed twice.
pub(crate) fn list_error<R: PartialEq>(list: &[CreatureAbility<R>]) -> Option<&'static str> {
    if list.len() > MAX_CREATURE_ABILITIES {
        return Some("at most 4 abilities outside battle");
    }
    if let Some(e) = list.iter().find_map(CreatureAbility::parameter_error) {
        return Some(e);
    }
    for (i, a) in list.iter().enumerate() {
        if list.get(..i).unwrap_or(&[]).iter().any(|b| a.collides(b)) {
            return Some("an ability outside battle is listed twice");
        }
    }
    None
}

impl CanonicalHash for CompiledCreatureAbility {
    fn hash_into(&self, h: &mut StateHasher) {
        h.write_u8(self.code());
        match self {
            Self::WeeklyResource { resource, amount } => {
                resource.hash_into(h);
                h.write_u32(*amount);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use h3_kernel::Handle;

    use super::*;

    type A = CreatureAbility<u8>;

    fn weekly(resource: u8, amount: u32) -> A {
        A::WeeklyResource { resource, amount }
    }

    #[test]
    fn names_follow_codes() {
        let a = weekly(4, 3);
        assert_eq!(a.code(), 0);
        assert_eq!(a.name(), "weekly_resource");
        assert_eq!(a.name(), A::NAMES[0]);
    }

    #[test]
    fn lists_are_checked() {
        assert_eq!(list_error(&[weekly(4, 3), weekly(5, 1)]), None);
        assert_eq!(list_error::<u8>(&[]), None);
        assert_eq!(
            list_error(&[weekly(4, 3), weekly(4, 1)]),
            Some("an ability outside battle is listed twice")
        );
        for amount in [0, MAX_DAILY_AMOUNT + 1] {
            assert_eq!(
                list_error(&[weekly(4, amount)]),
                Some("weekly_resource amount must be 1..=2147483647")
            );
        }
        assert_eq!(list_error(&[weekly(4, MAX_DAILY_AMOUNT)]), None);
        let five: std::vec::Vec<A> = (0..5).map(|r| weekly(r, 1)).collect();
        assert_eq!(
            list_error(&five),
            Some("at most 4 abilities outside battle")
        );
    }

    #[test]
    fn try_map_maps_the_resource_and_the_hash_sees_every_field() {
        let a = weekly(4, 3);
        assert_eq!(
            a.try_map(|r| Ok::<u16, ()>(u16::from(*r) + 100)),
            Ok(CreatureAbility::WeeklyResource {
                resource: 104,
                amount: 3
            })
        );
        assert_eq!(a.try_map(|_| Err::<u16, ()>(())), Err(()));
        let c = |r, amount| CompiledCreatureAbility::WeeklyResource {
            resource: Handle::from_raw(r),
            amount,
        };
        let hash = h3_kernel::hash::hash_of(&c(4, 3));
        assert_ne!(hash, h3_kernel::hash::hash_of(&c(5, 3)));
        assert_ne!(hash, h3_kernel::hash::hash_of(&c(4, 2)));
    }
}
