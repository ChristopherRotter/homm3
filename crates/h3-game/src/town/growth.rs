//! The towns' growth in the game's day (TWN-GRO-001, TWN-GRO-003, TWN-GRO-004; E09 T26a1, the town
//! epic's design note §41 to §43, §47; survey `E09-growth.md` §1 to §3): each town's
//! [`GrowthInputs`] from the adventure's owners, the owners' dwellings on the map and the heroes'
//! Legion ([`inputs`]), then on a week's first day every town's week ([`grow_week`]) and on a
//! month's first day its month ([`grow_month`]), each written into the store where it changes.
//! Every week is a normal one until the week's kind is drawn (TWN-GRO-002, E09 T26a2); the counts
//! are the owner's and shown with E09 T26b, so nothing is told yet.

use alloc::vec;
use alloc::vec::Vec;

use h3_adventure::{AdventureState, ObjectKind};
use h3_content::{CreatureTypeId, Ruleset};
use h3_contracts::{PlayerId, TownId};
use h3_towns::{
    GrowthInputs, MAX_CREATURE_LEVELS, MonthGrowth, Town, TownRuleset, WeekKind, index_creature,
};

use crate::{GameRuleset, GameState, MonthKind};

/// What a player's towns read of the player (TWN-GRO-001's inputs; design §41): the share of its
/// heroes' Statue of Legion and its dwellings on the map by their first creature. *Agent's call*
/// (design §41): built once per player and step, not per town.
struct Owner {
    /// The highest [`h3_heroes::owner_town_growth`] of the player's heroes on the map (garrison
    /// heroes: E09 T28b).
    share_bp: u16,
    /// By creature handle: how many of the player's dwellings on the map have it as their kind's
    /// first creature (the original's `town_dwelling_count_add` 0x5BFE50; a Refugee Camp is none).
    firsts: Vec<u16>,
}

impl Owner {
    /// `player`'s share and dwellings in `state`.
    fn of(state: &GameState, rules: &GameRuleset, player: PlayerId) -> Self {
        let (core, adventure) = (rules.core(), state.adventure());
        let share_bp = adventure
            .heroes()
            .iter()
            .filter(|hero| hero.owner == player)
            .filter_map(|hero| state.heroes().get(usize::from(hero.id.raw())))
            .map(|details| h3_heroes::owner_town_growth(details, core))
            .max()
            .unwrap_or(0);
        let mut firsts = vec![0_u16; core.creature_count()];
        for creature in owned_dwellings(adventure, rules, player) {
            if let Some(count) = firsts.get_mut(usize::from(creature.raw())) {
                *count = count.saturating_add(1);
            }
        }
        Self { share_bp, firsts }
    }

    /// How many of the player's dwellings count at a town's index of `creature` (none for an index
    /// without one), with `base` the level's base creature at an upgrade's index (a base dwelling
    /// counts at its upgrade's index too, survey §1 step 6).
    fn dwellings(&self, creature: Option<CreatureTypeId>, base: Option<CreatureTypeId>) -> u16 {
        let count = |c: CreatureTypeId| self.firsts.get(usize::from(c.raw())).copied().unwrap_or(0);
        creature.map_or(0, |c| count(c).saturating_add(base.map_or(0, count)))
    }
}

/// The first creature of each dwelling `player` owns on the map, in object order (the static
/// map's dwellings, ADV-DWL-001: a random one is listed resolved).
fn owned_dwellings<'a>(
    adventure: &'a AdventureState,
    rules: &'a GameRuleset,
    player: PlayerId,
) -> impl Iterator<Item = CreatureTypeId> + 'a {
    let map = adventure.map();
    map.dwellings()
        .iter()
        .filter(move |&&index| adventure.owner(index) == Some(player))
        .filter_map(move |&index| map.objects()[index].object_type)
        .filter_map(|id| rules.adventure().object_type(id))
        .filter_map(|ty| match &ty.kind {
            ObjectKind::Dwelling { creatures, .. } => creatures.first().copied(),
            _ => None,
        })
}

/// Every town's growth inputs (TWN-GRO-001; design §41), by `TownId`: a neutral town's
/// [`GrowthInputs::NEUTRAL`]; an owned town's its owner's share, its visitor's Legion
/// ([`h3_heroes::town_growth`]; the garrison hero's with E09 T28b) and per index its owner's
/// dwellings whose first creature is the index's, a base creature's at its upgrade's index too.
/// Computed once a week, before the towns' week, and kept for the month of the same day (nothing
/// between them changes an owner, a hero's artifacts or a visitor).
// rule: TWN-GRO-001
pub(crate) fn inputs(state: &GameState, rules: &GameRuleset) -> Vec<GrowthInputs> {
    let (core, part, adventure) = (rules.core(), rules.towns(), state.adventure());
    let mut owners: Vec<Option<Owner>> = Vec::new();
    let towns = adventure.map().towns();
    (0..=u16::MAX)
        .map(TownId::new)
        .zip(towns)
        .map(|(town, &index)| {
            let (Some(player), Some(record)) = (
                adventure.owner(index),
                state.towns().get(town).and_then(|t| part.town(t.faction())),
            ) else {
                return GrowthInputs::NEUTRAL;
            };
            if owners.len() <= player.index() {
                owners.resize_with(player.index() + 1, || None);
            }
            let owner =
                owners[player.index()].get_or_insert_with(|| Owner::of(state, rules, player));
            let legion = adventure
                .town_visitor(town)
                .and_then(|hero| state.heroes().get(usize::from(hero.raw())))
                .map_or([0; MAX_CREATURE_LEVELS], |details| {
                    h3_heroes::town_growth(details, core)
                });
            let dwellings = core::array::from_fn(|at| {
                let base = at
                    .checked_sub(MAX_CREATURE_LEVELS)
                    .and_then(|base| index_creature(record, base));
                owner.dwellings(index_creature(record, at), base)
            });
            GrowthInputs {
                owned: true,
                share_bp: owner.share_bp,
                legion,
                dwellings,
            }
        })
        .collect()
}

/// The towns' week (TWN-GRO-003; design §42, survey §2: the original's `town_week_growth` 0x5BFDD0
/// from `new_day_week_events`), on a week's first day after [`crate::GameEvent::WeekStarted`] and
/// before the adventure's week step: every town in `TownId` order grows its week with its
/// `inputs` ([`h3_towns::week_growth`]; a normal week until TWN-GRO-002 draws the kind, E09
/// T26a2).
// rule: TWN-GRO-003
pub(crate) fn grow_week(state: &mut GameState, rules: &GameRuleset, inputs: &[GrowthInputs]) {
    rewrite(state, rules, inputs, |town, part, core, inputs| {
        h3_towns::week_growth(town, part, core, &WeekKind::Normal, inputs)
    });
}

/// The towns' month (TWN-GRO-004; design §43, survey §3: the original's `new_month` 0x4C8F40 at its
/// towns), on a month's first day after [`crate::GameEvent::MonthStarted`] and before the
/// adventure's month step: every town in `TownId` order takes `month` with its `inputs`
/// ([`h3_towns::month`], the game's [`MonthKind`] as the town crate's [`MonthGrowth`]; the
/// month's first week a normal one until E09 T26a2). A normal month writes nothing.
// rule: TWN-GRO-004
pub(crate) fn grow_month(
    state: &mut GameState,
    rules: &GameRuleset,
    month: MonthKind,
    inputs: &[GrowthInputs],
) {
    let growth = match month {
        MonthKind::Normal => return,
        MonthKind::Creature(creature) => MonthGrowth::Creature(creature),
        MonthKind::Plague => MonthGrowth::Plague,
    };
    rewrite(state, rules, inputs, |town, part, core, inputs| {
        h3_towns::month(town, part, core, growth, &WeekKind::Normal, inputs)
    });
}

/// Writes each town, in `TownId` order with its `inputs`, as `step` makes it, only where it
/// changes (a fork's store is copied at its first changed town alone).
fn rewrite(
    state: &mut GameState,
    rules: &GameRuleset,
    inputs: &[GrowthInputs],
    step: impl Fn(&Town, &TownRuleset, &Ruleset, &GrowthInputs) -> Town,
) {
    let (core, part) = (rules.core(), rules.towns());
    let changed: Vec<(TownId, Town)> = (0..=u16::MAX)
        .map(TownId::new)
        .zip(inputs)
        .filter_map(|(town, inputs)| {
            let before = state.towns().get(town)?;
            let after = step(before, part, core, inputs);
            (after != *before).then_some((town, after))
        })
        .collect();
    for (town, after) in changed {
        state.towns_mut().write(town, after);
    }
}
