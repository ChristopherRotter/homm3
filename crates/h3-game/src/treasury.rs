//! The players' treasuries (ADR-0017 point 1, E08 T13; GAM-RES-001): each player's amount of every
//! resource kind of the core ruleset, dense by [`ResourceId`] (the kinds stay data, ADR-0006). The
//! game keeps them; a context's command gains or costs resources through its outcome and never
//! sees a treasury (ADR-0017 point 10).
//!
//! All treasuries are one block behind an `Arc`, as the adventure's per-player layers are
//! (ADR-0016 point 1): a fork of a game shares it for a reference count, where a block per player
//! cost a fork an allocation per player (+14 % on the XL fork benchmark, E08 T13b); a gain copies
//! it once (`Arc::make_mut`). Nothing sees the sharing: equality, `Debug` and the hash see the
//! amounts only.

use alloc::sync::Arc;
use alloc::vec::Vec;

use h3_adventure::MAX_GAIN;
use h3_content::ResourceId;
use h3_contracts::PlayerId;
use h3_kernel::{CanonicalHash, StateHasher};

/// Every player's resources: never negative, each at most 2³¹ − 1 (a gain past it saturates; the
/// original's 32-bit sum wraps negative and drops the resource to 0, `A-5`, a state-corrupting bug
/// fixed without a flag: survey `E08-objects.md` §7.1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Treasuries {
    /// The resource kinds a treasury holds.
    kinds: usize,
    /// The amounts, player by player, `kinds` each.
    amounts: Arc<Vec<u32>>,
}

impl Treasuries {
    /// `players` treasuries, each holding `start`, by resource in the core's order, each amount
    /// capped at [`MAX_GAIN`] (the compiled starting rows already are).
    // rule: GAM-RES-001
    pub(crate) fn new(players: usize, start: &[u32]) -> Self {
        let start: Vec<u32> = start.iter().map(|&a| a.min(MAX_GAIN)).collect();
        Self {
            kinds: start.len(),
            amounts: Arc::new(start.repeat(players)),
        }
    }

    /// Where `player`'s amount of the resource at `index` is kept, if both exist.
    fn slot(&self, player: PlayerId, index: usize) -> Option<usize> {
        (index < self.kinds)
            .then(|| {
                player
                    .index()
                    .saturating_mul(self.kinds)
                    .saturating_add(index)
            })
            .filter(|&at| at < self.amounts.len())
    }

    /// `player`'s amounts by resource, if the player has a treasury.
    pub(crate) fn of(&self, player: PlayerId) -> Option<&[u32]> {
        let start = player.index().saturating_mul(self.kinds);
        self.amounts.get(start..start.saturating_add(self.kinds))
    }

    /// Adds `amount` of `resource` to `player`'s treasury, saturating at [`MAX_GAIN`]
    /// (GAM-RES-001; `A-5` fixed): the amount actually added (0 for a resource the ruleset does
    /// not have or a player without a treasury, which no rule gives).
    // rule: GAM-RES-001
    pub(crate) fn gain(&mut self, player: PlayerId, resource: ResourceId, amount: u32) -> u32 {
        self.slot(player, usize::from(resource.raw()))
            .map_or(0, |at| {
                let held = &mut Arc::make_mut(&mut self.amounts)[at];
                let before = *held;
                *held = held.saturating_add(amount).min(MAX_GAIN);
                held.saturating_sub(before)
            })
    }
}

impl CanonicalHash for Treasuries {
    /// The resource kinds, then every player's amounts as length and entries.
    fn hash_into(&self, h: &mut StateHasher) {
        h.write_len(self.kinds);
        self.amounts.hash_into(h);
    }
}
