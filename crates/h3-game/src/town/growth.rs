//! The towns' growth in the game's day (TWN-GRO-001, TWN-GRO-003, TWN-GRO-004; E09 T26a1, the town
//! epic's design note §41 to §43, §47; survey `E09-growth.md` §1 to §3): each town's
//! [`GrowthInputs`] from the adventure's owners, the owners' dwellings on the map and the heroes'
//! Legion ([`inputs`]), then on a week's first day every town's week ([`grow_week`]) and on a
//! month's first day its month ([`grow_month`]), each written into the store where it changes.
//! Since E09 T26a2 the week's kind (TWN-GRO-002); the counts are the owner's (TWN-VEW-004, E09
//! T26b), told at the towns' day (`super::start_day`).

use alloc::vec;
use alloc::vec::Vec;

use h3_adventure::{AdventureState, ObjectKind};
use h3_content::{CreatureTypeId, Ruleset};
use h3_contracts::{PlayerId, TownId};
use h3_towns::{
    CREATURE_INDEXES, GrowthInputs, MAX_CREATURE_LEVELS, MonthGrowth, Town, TownRuleset,
    index_creature,
};

use crate::{GameRuleset, GameState, MonthKind};

/// What a player's towns read of the player (TWN-GRO-001's inputs; design §41): the share of its
/// heroes' Statue of Legion and its dwellings on the map by their first creature. *Agent's call*
/// (design §41): built once per player and step, not per town.
struct Owner {
    /// The highest [`h3_heroes::owner_town_growth`] of the player's heroes on the map and its
    /// garrison heroes (E09 T28b1; the original's `player_hero_wears_artifact` 0x4BACB0: the
    /// player's list of heroes, then its towns' +0xC heroes).
    share_bp: u16,
    /// By creature handle: how many of the player's dwellings on the map have it as their kind's
    /// first creature (the original's `town_dwelling_count_add` 0x5BFE50; a Refugee Camp is none);
    /// empty when the player owns none.
    firsts: Vec<u16>,
}

impl Owner {
    /// `player`'s share and dwellings in `state`.
    fn of(state: &GameState, rules: &GameRuleset, player: PlayerId) -> Self {
        let (core, adventure) = (rules.core(), state.adventure());
        let garrisons = adventure.town_heroes().iter().map(|(_, hero)| hero);
        let share_bp = adventure
            .heroes()
            .iter()
            .chain(garrisons)
            .filter(|hero| hero.owner == player)
            .filter_map(|hero| state.heroes().get(usize::from(hero.id.raw())))
            .map(|details| h3_heroes::owner_town_growth(details, core))
            .max()
            .unwrap_or(0);
        let mut firsts = Vec::new();
        let dwellings = owned_dwellings(adventure, rules, player);
        for creature in dwellings.filter_map(|creatures| creatures.first().copied()) {
            if firsts.is_empty() {
                firsts = vec![0_u16; core.creature_count()];
            }
            let count = firsts.get_mut(usize::from(creature.raw()));
            count
                .into_iter()
                .for_each(|count| *count = count.saturating_add(1));
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

/// The creatures of each creature dwelling `player` owns on the map, in its kind's order, the
/// dwellings in object order (among the static map's ownable objects; a random dwelling is listed
/// resolved, ADV-DWL-005; a Refugee Camp is none): the growth's inputs by their first creature
/// (TWN-GRO-001), the Portal of Summoning's offer (TWN-GRO-007, E09 T26b).
pub(crate) fn owned_dwellings<'a, 'r>(
    adventure: &'a AdventureState,
    rules: &'r GameRuleset,
    player: PlayerId,
) -> impl Iterator<Item = &'r [CreatureTypeId]> + use<'a, 'r> {
    let map = adventure.map();
    map.ownables()
        .iter()
        .filter(move |&&index| adventure.owner(index) == Some(player))
        .filter_map(move |&index| map.objects()[index].object_type)
        .filter_map(|id| rules.adventure().object_type(id))
        .filter_map(|ty| match &ty.kind {
            ObjectKind::Dwelling { creatures, .. } => Some(creatures.as_slice()),
            _ => None,
        })
}

/// Every town's growth inputs (TWN-GRO-001; design §41), by `TownId`: a neutral town's
/// [`GrowthInputs::NEUTRAL`]; an owned town's its owner's share (over its heroes on the map and
/// its garrison heroes), its visitor's Legion ([`h3_heroes::town_growth`]) plus since E09 T28b1
/// its garrison hero's, each hero once, summed by level saturating (design §41, §53;
/// TWN-ROW-005), and per index its owner's dwellings whose first creature is the index's, a base
/// creature's at its upgrade's index too. Computed once a week, before the towns' week, and kept
/// for the month of the same day (nothing between them changes an owner, a hero's artifacts, a
/// visitor or a garrison hero).
// rule: TWN-GRO-001, TWN-ROW-005
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
            let garrison = adventure.town_hero(town).map(|hero| hero.id);
            let mut legion = [0_u16; MAX_CREATURE_LEVELS];
            let heroes = [adventure.town_visitor(town), garrison];
            let details = heroes
                .into_iter()
                .flatten()
                .filter_map(|hero| state.heroes().get(usize::from(hero.raw())));
            for pieces in details.map(|details| h3_heroes::town_growth(details, core)) {
                for (sum, piece) in legion.iter_mut().zip(pieces) {
                    *sum = sum.saturating_add(piece);
                }
            }
            let dwellings = if owner.firsts.is_empty() {
                [0; CREATURE_INDEXES]
            } else {
                core::array::from_fn(|at| {
                    let base = at
                        .checked_sub(MAX_CREATURE_LEVELS)
                        .and_then(|base| index_creature(record, base));
                    owner.dwellings(index_creature(record, at), base)
                })
            };
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
/// `inputs` and the week's kind ([`h3_towns::week_growth`], [`GameState::week`]: TWN-GRO-002,
/// drawn before [`crate::GameEvent::WeekStarted`] since E09 T26a2).
// rule: TWN-GRO-003
pub(crate) fn grow_week(state: &mut GameState, rules: &GameRuleset, inputs: &[GrowthInputs]) {
    let week = state.week();
    rewrite(state, rules, inputs, |town, part, core, inputs| {
        h3_towns::week_growth(town, part, core, &week, inputs)
    });
}

/// The towns' month (TWN-GRO-004; design §43, survey §3: the original's `new_month` 0x4C8F40 at its
/// towns), on a month's first day after [`crate::GameEvent::MonthStarted`] and before the
/// adventure's month step: every town in `TownId` order takes `month` with its `inputs`
/// ([`h3_towns::month`], the game's [`MonthKind`] as the town crate's [`MonthGrowth`], with the
/// month's first week's kind: after a building's week the month's creature doubles nothing). A
/// normal month writes nothing.
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
    let week = state.week();
    rewrite(state, rules, inputs, |town, part, core, inputs| {
        h3_towns::month(town, part, core, growth, &week, inputs)
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
            let before = state.towns().get(town);
            before.map(|before| (town, before, step(before, part, core, inputs)))
        })
        .filter(|(_, before, after)| after != *before)
        .map(|(town, _, after)| (town, after))
        .collect();
    for (town, after) in changed {
        state.towns_mut().write(town, after);
    }
}
