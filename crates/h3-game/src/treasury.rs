//! A player's treasury (ADR-0017 point 1, E08 T13; GAM-RES-001): an amount per resource kind of
//! the core ruleset, dense by [`ResourceId`] (the kinds stay data, ADR-0006). The game keeps it;
//! a context's command gains or costs resources through its outcome and never sees a treasury
//! (ADR-0017 point 10).

use alloc::vec::Vec;

use h3_adventure::MAX_GAIN;
use h3_content::ResourceId;
use h3_kernel::{CanonicalHash, StateHasher};

/// A player's resources: never negative, each at most 2³¹ − 1 (a gain past it saturates; the
/// original's 32-bit sum wraps negative and drops the resource to 0, `A-5`, a state-corrupting bug
/// fixed without a flag: survey `E08-objects.md` §7.1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Treasury {
    amounts: Vec<u32>,
}

impl Treasury {
    /// A treasury holding `amounts`, by resource in the core's order, each capped at
    /// [`MAX_GAIN`] (the compiled starting rows already are).
    // rule: GAM-RES-001
    pub(crate) fn new(amounts: &[u32]) -> Self {
        Self {
            amounts: amounts.iter().map(|&a| a.min(MAX_GAIN)).collect(),
        }
    }

    /// The amounts, by resource in the core's order.
    pub fn amounts(&self) -> &[u32] {
        &self.amounts
    }

    /// The amount of `resource` (0 for a resource the ruleset does not have).
    pub fn amount(&self, resource: ResourceId) -> u32 {
        self.amounts
            .get(usize::from(resource.raw()))
            .copied()
            .unwrap_or(0)
    }
}

impl CanonicalHash for Treasury {
    /// The amounts as their length and entries.
    fn hash_into(&self, h: &mut StateHasher) {
        self.amounts.hash_into(h);
    }
}
