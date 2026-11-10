//! The hall's offer and the legal builds (E09 T18b2b, the town epic's design note §29): what a
//! town's owner may build now, as [`crate::legal_commands`] lists it ([`legal_builds`]) and as a
//! client's hall shows it ([`GameState::town_offer`]). Both read the hall's tests
//! ([`h3_towns::buildable`], [`h3_towns::possible`]) with whether the owner holds another town
//! with a standing Capitol ([`holds_capitol`]), and the owner's treasury; the command's own checks
//! are `town::command`'s, so a listed build is one `apply` accepts.

use h3_contracts::{PlayerId, TownId};
use h3_towns::{BuildingSet, TownCommand};

use super::holds_capitol;
use crate::command::GameCommand;
use crate::{GameRuleset, GameState};

/// A town's hall as its owner sees it (E09 T18b2b, design §29, the agent's call), each set over
/// the town's faction's record (`h3_towns::TownRecord`), each holding the next.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TownOffer {
    /// The buildings it may still build some day, ever built or not ([`h3_towns::possible`]: the
    /// hall's "not now" against "never").
    pub possible: BuildingSet,
    /// The buildings the hall offers now ([`h3_towns::buildable`]): none while the town was built
    /// in today (TWN-BLD-004).
    pub buildable: BuildingSet,
    /// Those of `buildable` whose price the owner's treasury holds now (GAM-RES-001): the builds
    /// [`crate::legal_commands`] lists for the town.
    pub payable: BuildingSet,
}

impl GameState {
    /// The hall of `town` as `player` sees it (E09 T18b2b, design §29): `None` unless `player`
    /// owns the town (the adventure's owner of its object; the owner's buildings are its own to
    /// see, TWN-VEW-004). In its turn or not, as the original's town list opens the hall at any
    /// time (survey `E09-construction.md` §1.1); a build is still `apply`'s to accept.
    // rule: TWN-BLD-002, TWN-BLD-003, TWN-BLD-004
    pub fn town_offer(
        &self,
        rules: &GameRuleset,
        player: PlayerId,
        town: TownId,
    ) -> Option<TownOffer> {
        let adventure = self.adventure();
        let owner = adventure
            .map()
            .town_object(town)
            .and_then(|index| adventure.owner(index));
        let record = self.towns().get(town).filter(|_| owner == Some(player))?;
        let part = rules.towns();
        let holds = holds_capitol(adventure, self.towns(), part, player, town);
        let buildable = h3_towns::buildable(record, part, holds);
        Some(TownOffer {
            possible: h3_towns::possible(record, part, holds),
            buildable,
            payable: payable(self, rules, player, town, buildable),
        })
    }
}

/// The builds `player` may give now, for [`crate::legal_commands`] in `player`'s turn without a
/// decision (E09 T18b2b, design §29; TWN-BLD-001): for each town it owns, in `TownId` order (the
/// static map's towns in object order), each building of the hall's offer whose price its treasury
/// holds, in the record's order. A town built in today offers none (TWN-BLD-004) and costs no
/// Capitol test; a map without towns walks nothing.
// rule: TWN-BLD-001, TWN-BLD-004
pub(crate) fn legal_builds<'a>(
    state: &'a GameState,
    rules: &'a GameRuleset,
    player: PlayerId,
) -> impl Iterator<Item = GameCommand> + 'a {
    let adventure = state.adventure();
    let map = adventure.map();
    let part = rules.towns();
    map.towns()
        .iter()
        .filter(move |&&index| adventure.owner(index) == Some(player))
        .filter_map(move |&index| map.town_id(index))
        .flat_map(move |town| {
            let buildable = state
                .towns()
                .get(town)
                .filter(|record| !record.built_today())
                .map_or(BuildingSet::EMPTY, |record| {
                    let holds = holds_capitol(adventure, state.towns(), part, player, town);
                    h3_towns::buildable(record, part, holds)
                });
            payable(state, rules, player, town, buildable)
                .iter()
                .map(move |building| GameCommand::Town(TownCommand::Build { town, building }))
        })
}

/// The buildings of `buildable` (a set over `town`'s faction's record) whose price `player`'s
/// treasury holds (GAM-RES-001; the part's `Building.txt` costs, TWN-BLD-006).
fn payable(
    state: &GameState,
    rules: &GameRuleset,
    player: PlayerId,
    town: TownId,
    buildable: BuildingSet,
) -> BuildingSet {
    let record = state
        .towns()
        .get(town)
        .and_then(|town| rules.towns().town(town.faction()));
    buildable
        .iter()
        .filter(|&id| {
            record
                .and_then(|record| record.building(id))
                .is_some_and(|building| state.can_pay(player, &building.cost))
        })
        .fold(BuildingSet::EMPTY, BuildingSet::with)
}
