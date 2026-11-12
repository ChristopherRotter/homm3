//! The Portal of Summoning's draw (E09 increment 3, design note §45; survey `E09-growth.md` §5, the
//! original's `portal_of_summoning_week` 0x5BD750, read whole): a town's Portal holds one of its
//! owner's dwellings' creatures, its count the creature's growth, set at each draw (TWN-GRO-007). A
//! pure function over the draws given; the game draws each week, at the Portal's build, when its
//! owner flags a dwelling while it is empty and at a capture that finds it empty (decisions 71 and
//! 82), each from its own stream (E09 T26b).

use h3_content::{CreatureTypeId, Ruleset};
use h3_kernel::Rng;

use crate::derive::Roles;
use crate::record::{Summoned, Town};
use crate::ruleset::TownRuleset;

/// The index `below(n)` draws among `n` items (`n` at least 1).
fn pick(n: usize, draws: &mut Rng) -> usize {
    let n = u32::try_from(n).unwrap_or(u32::MAX);
    usize::try_from(draws.below(n)).unwrap_or(0)
}

/// `town` after its Portal of Summoning's draw (TWN-GRO-007; E09 design §45, survey
/// `E09-growth.md` §5, the original's `portal_of_summoning_week` 0x5BD750), with its faction's
/// record in `part`, the creatures' growths in `core` and `offer` its owner's dwellings' creature
/// lists on the map, in object order (never a camp; the game's), from `draws`:
///
/// - a town without a standing building of role `portal` is unchanged, with no draw;
/// - without a dwelling the Portal is emptied, with no draw;
/// - else a dwelling `below(n)`, then one of its creatures `below(k)`: the Portal holds that
///   creature, its count the creature's core growth, set, not added (what was not hired is gone);
///   a dwelling without creatures leaves the Portal as it was.
///
/// No castle, horde, Grail, week, month or Plague touches the count (TD-16). The owner is the
/// game's: it never draws in a neutral town (the original's week skips it).
// rule: TWN-GRO-007
pub fn summon(
    town: &Town,
    part: &TownRuleset,
    core: &Ruleset,
    offer: &[&[CreatureTypeId]],
    draws: &mut Rng,
) -> Town {
    if !Roles::of(town, part)
        .standing_roles()
        .any(|(_, r)| r.portal)
    {
        return town.clone();
    }
    if offer.is_empty() {
        return town.with_portal(None);
    }
    let dwelling = offer[pick(offer.len(), draws)];
    if dwelling.is_empty() {
        return town.clone();
    }
    let creature = dwelling[pick(dwelling.len(), draws)];
    let count = core.creature(creature).map_or(0, |c| c.growth);
    town.with_portal(Some(Summoned { creature, count }))
}

#[cfg(test)]
mod tests;
