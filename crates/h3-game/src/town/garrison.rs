//! A neutral town's garrison week in the game (TWN-GRO-006; E09 T26b, the town epic's design note
//! §44; survey `E09-growth.md` §4, the original's `neutral_town_garrison_week` 0x4BF570 from
//! `new_day_week_events`): on a week's first day, after the adventure's week step, each town
//! without an owner may draw a stack of its faction's creatures into its garrison out of its
//! available creatures ([`h3_towns::garrison_week`]), from the towns' stream; the garrison is
//! written into the adventure and the town into the store, in one application. Told with the
//! towns' day (`super::start_day`): a garrison's change reaches every viewer whose view of the town
//! it moves (TWN-VEW-002).

use h3_contracts::TownId;

use super::Draws;
use crate::{GameRuleset, GameState};

/// The neutral towns' garrison week (TWN-GRO-006; design §44), `weeks` the weeks since the start
/// (the day less one, divided by the week's days: 1 on day 8): for each town without an owner in
/// `TownId` order, [`h3_towns::garrison_week`] with its garrison as it stands, the adventure
/// part's army slots and a stack's most creatures (ADV-ARM-007: the room is the army's), from the
/// towns' stream ([`Draws::Towns`], E09 decision 75); a week that adds a stack writes the
/// garrison ([`h3_adventure::AdventureState::set_town_garrison`]) and the town, its counts
/// lowered. An owned town's garrison never grows (none of its draws are made).
// rule: TWN-GRO-006
pub(crate) fn garrison_weeks(state: &mut GameState, rules: &GameRuleset, weeks: u32) {
    let (core, part) = (rules.core(), rules.towns());
    let params = rules.adventure().params();
    let army = (params.army_slots(), params.max_stack_count());
    let (adventure, towns, draws) = state.town_draws(Draws::Towns);
    let count = adventure.map().towns().len();
    for town in (0..=u16::MAX).take(count).map(TownId::new) {
        let week = adventure
            .map()
            .town_object(town)
            .filter(|&index| adventure.owner(index).is_none())
            .zip(towns.get(town))
            .and_then(|(index, record)| {
                let garrison = adventure.town_garrison(index).unwrap_or_default();
                h3_towns::garrison_week(record, part, core, weeks, garrison, army, draws)
            });
        if let Some(week) = week {
            adventure.set_town_garrison(town, week.stacks);
            towns.write(town, week.town);
        }
    }
}
