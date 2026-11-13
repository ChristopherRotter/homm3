//! The Portal of Summoning's draws in the game (TWN-GRO-007; E09 T26b, the town epic's design note
//! §45, decisions 71 and 82; survey `E09-growth.md` §5, the original's `portal_of_summoning_week`
//! 0x5BD750): a town's Portal holds one of its owner's dwellings' creatures, drawn with
//! [`h3_towns::summon`] each week, when it is built, when its owner flags a dwelling while it holds
//! none and at a capture that finds it holding none, each from its own stream ([`Draws`]). A
//! neutral town's is never drawn (the original's week skips it); the owner's view shows it
//! (TWN-VEW-004), told with the change that drew it.

use alloc::vec::Vec;

use h3_adventure::AdventureState;
use h3_contracts::{PlayerId, TownId};
use h3_kernel::Rng;
use h3_towns::Towns;

use super::growth::owned_dwellings;
use super::view::Told;
use crate::command::Sink;
use crate::{GameRuleset, GameState};

/// The stream a town's draw takes (E09 T26b, design §44, §45; ADR-0017 point 4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Draws {
    /// The towns' stream (`StreamId::TOWNS`, E09 decision 75): the week's draws, a town event's
    /// build.
    Towns,
    /// A player's stream: its command's build, its visit's flag, its capture.
    Player(PlayerId),
}

/// Draws the Portal of `town` in `towns` with `draws` (TWN-GRO-007): for a town with an owner and
/// a standing Portal ([`h3_towns::has_portal`]) [`h3_towns::summon`] with the owner's dwellings'
/// creature lists in object order ([`owned_dwellings`]), written where it changes; nothing, and no
/// draw, for any other town.
// rule: TWN-GRO-007
fn draw(
    adventure: &AdventureState,
    towns: &mut Towns,
    rules: &GameRuleset,
    town: TownId,
    draws: &mut Rng,
) {
    let part = rules.towns();
    // The Portal first: most towns have none, and the owner's lookup is a search.
    let Some(record) = towns
        .get(town)
        .filter(|record| h3_towns::has_portal(record, part))
    else {
        return;
    };
    let map = adventure.map();
    let Some(owner) = map
        .town_object(town)
        .and_then(|index| adventure.owner(index))
    else {
        return;
    };
    let offer: Vec<_> = owned_dwellings(adventure, rules, owner).collect();
    let after = h3_towns::summon(record, part, rules.core(), &offer, draws);
    if after != *record {
        towns.write(town, after);
    }
}

/// [`draw`] in `state`'s towns from the stream `draws` names.
fn draw_in(state: &mut GameState, rules: &GameRuleset, town: TownId, draws: Draws) {
    let (adventure, towns, stream) = state.town_draws(draws);
    draw(adventure, towns, rules, town, stream);
}

/// The towns of `state`'s map, in `TownId` order.
fn towns_of(state: &GameState) -> impl Iterator<Item = TownId> + use<> {
    let count = state.adventure().map().towns().len();
    (0..=u16::MAX).take(count).map(TownId::new)
}

/// The week's draws (TWN-GRO-007; design §45, survey §5: `new_day_week_events`' last loop), on a
/// week's first day after the neutral garrisons' week (`super::garrison::garrison_weeks`): every
/// town with an owner and a standing Portal in `TownId` order, from the towns' stream. Told with
/// the towns' day (`super::start_day`).
// rule: TWN-GRO-007
pub(crate) fn weekly(state: &mut GameState, rules: &GameRuleset) {
    for town in towns_of(state) {
        draw_in(state, rules, town, Draws::Towns);
    }
}

/// The setup's draws (TWN-GRO-007; design §45): each owned town's Portal that the setup built, in
/// `TownId` order, from the setup stream `draws` after every other placeholder (the game's build,
/// before the first observation, so nothing is told).
// rule: TWN-GRO-007
pub(crate) fn at_setup(
    adventure: &AdventureState,
    towns: &mut Towns,
    rules: &GameRuleset,
    draws: &mut Rng,
) {
    let count = adventure.map().towns().len();
    for town in (0..=u16::MAX).take(count).map(TownId::new) {
        draw(adventure, towns, rules, town, draws);
    }
}

/// A build's draw (TWN-GRO-007; design §45): a build that made `town`'s Portal stand (`had`:
/// whether one stood before it) draws it, from `draws` (a command's: its player's stream, after
/// the checks and the payment; a town event's: the towns' stream). The caller tells the views.
// rule: TWN-GRO-007
pub(crate) fn at_build(
    state: &mut GameState,
    rules: &GameRuleset,
    town: TownId,
    had: bool,
    draws: Draws,
) {
    if !had {
        draw_in(state, rules, town, draws);
    }
}

/// Whether `town` in `state` has a standing Portal that holds nothing.
fn empty_portal(state: &GameState, rules: &GameRuleset, town: TownId) -> bool {
    state.towns().get(town).is_some_and(|record| {
        record.portal().is_none() && h3_towns::has_portal(record, rules.towns())
    })
}

/// A capture's draw (TWN-GRO-007; design §45, §51, E09 decision 82): a captured town whose
/// standing Portal holds nothing is drawn for its new owner `player`, from its stream after the
/// capture's own draws (the original keeps the Portal at a change of owner and a human's capture
/// opens the town screen, which draws at once). The caller, the capture, tells the views.
// rule: TWN-GRO-007
pub(crate) fn at_capture(
    state: &mut GameState,
    rules: &GameRuleset,
    town: TownId,
    player: PlayerId,
) {
    if empty_portal(state, rules, town) {
        draw_in(state, rules, town, Draws::Player(player));
    }
}

/// A flag's draws (TWN-GRO-007; design §45, E09 decision 71): after a visit of `player`'s that
/// flagged a dwelling (`h3_adventure::AdventureOutcome::flagged_dwelling`), each of its towns
/// whose standing Portal holds nothing, in `TownId` order, from its stream after the visit's
/// draws; each view the draws changed told to its viewer ([`Told`]: the owner's, TWN-VEW-004).
// rule: TWN-GRO-007, TWN-VEW-004
pub(crate) fn on_flag<S: Sink>(
    state: &mut GameState,
    rules: &GameRuleset,
    player: PlayerId,
    sink: &mut S,
) {
    let adventure = state.adventure();
    let map = adventure.map();
    let empty: Vec<TownId> = towns_of(state)
        .filter(|&town| {
            map.town_object(town)
                .is_some_and(|index| adventure.owner(index) == Some(player))
        })
        .filter(|&town| empty_portal(state, rules, town))
        .collect();
    if empty.is_empty() {
        return;
    }
    let told = Told::before::<S>(state, rules, &empty, &[]);
    for &town in &empty {
        draw_in(state, rules, town, Draws::Player(player));
    }
    told.tell(state, rules, sink);
}
