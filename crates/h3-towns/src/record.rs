//! A town's own record (ADR-0028 point 2; E09 increment 1's design note §4): [`Town`], what the
//! rules change (its faction, the buildings ever built and forbidden), and [`TownStatic`], what
//! the map gives and nothing changes (its name, its events). The store ([`crate::Towns`]) holds
//! both by `h3_contracts::TownId`.

use alloc::string::String;
use alloc::vec::Vec;

use h3_content::{FactionId, ResourceId};
use h3_contracts::PlayerId;

use crate::building::BuildingSet;
use crate::ruleset::{MAX_CREATURE_LEVELS, TownRecord};

/// A town's record: its faction and the buildings ever built and forbidden, each a set over its
/// faction's town record's buildings ([`TownRecord::buildings`]).
///
/// `built` is **every building ever built**, the superseded ones too (the original's every-built
/// set +0x158; E09 survey §16.5); the buildings **standing** are derived from it
/// ([`Town::standing`], the original's built set +0x150), never stored (E09 decision 29). The
/// rules read each building role over one of the two sets (`h3-pack-towns`' format doc).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Town {
    faction: FactionId,
    built: BuildingSet,
    forbidden: BuildingSet,
}

impl Town {
    /// A town of `faction` with the buildings `built` ever built and `forbidden` forbidden, each
    /// a set over the faction's town record (the build's setup gives them, E09 T6c).
    pub const fn new(faction: FactionId, built: BuildingSet, forbidden: BuildingSet) -> Self {
        Self {
            faction,
            built,
            forbidden,
        }
    }

    /// Its faction, which has a town record in the town part.
    pub const fn faction(&self) -> FactionId {
        self.faction
    }

    /// Every building ever built, the superseded ones too.
    pub const fn built(&self) -> BuildingSet {
        self.built
    }

    /// The buildings forbidden.
    pub const fn forbidden(&self) -> BuildingSet {
        self.forbidden
    }

    /// The buildings standing: those ever built less every building an ever-built one supersedes
    /// (each [`crate::Building::supersedes`] is closed already), read from `record`, its
    /// faction's town record (E09 survey §16.5: a Capitol leaves the Village, Town and City
    /// Halls ever built but not standing; an upgraded dwelling, its dwelling).
    pub fn standing(&self, record: &TownRecord) -> BuildingSet {
        let superseded = self
            .built
            .iter()
            .filter_map(|id| record.building(id))
            .fold(BuildingSet::EMPTY, |set, building| {
                set.union(building.supersedes)
            });
        self.built.difference(superseded)
    }
}

/// What the map gives a town that no rule changes (E09 increment 1's design note §4): its name and
/// its events, kept for the increments that read them. Shared by every fork.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TownStatic {
    /// Its name, if the map gives one (an unnamed town's name is the client's: the original's
    /// lists are text; ADR-0028 point 2).
    pub name: Option<String>,
    /// Its events in the map's order, which increment 2 applies (none in increment 1).
    pub events: Vec<TownEvent>,
}

/// A town's event as the map gives it (the h3m's town event: a timed event with buildings and
/// creatures; `docs/formats/h3m.md`), kept for increment 2, which applies it: the timed part as
/// the game's timed events have it (`h3_game::TimedEventSetup`'s fields), the buildings resolved
/// to the town's faction. Its message is named by its town and its index in the town's list (the
/// text is the client's).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TownEvent {
    /// The players it fires for: players of the game, in id order, each once.
    pub players: Vec<PlayerId>,
    /// Whether it fires for a human player.
    pub human: bool,
    /// Whether it fires for a player that is not human.
    pub computer: bool,
    /// The first day it fires, from 1.
    pub first_day: u16,
    /// It fires again every this many days; 0 once.
    pub interval: u16,
    /// What it gives (a negative amount takes): distinct resources, none 0.
    pub resources: Vec<(ResourceId, i32)>,
    /// The buildings it builds, of the town's faction's record (a map slot without one of the
    /// faction's: none).
    pub buildings: BuildingSet,
    /// The creatures it adds to the town's dwellings by level, from level 1.
    pub creatures: [u16; MAX_CREATURE_LEVELS],
}

#[cfg(test)]
mod tests {
    extern crate std;

    use std::vec;

    use h3_contracts::Moat;

    use super::*;
    use crate::building::BuildingId;
    use crate::ruleset::{Building, BuildingRoles};

    fn id(index: usize) -> BuildingId {
        BuildingId::from_index(index).unwrap()
    }

    fn set(indices: &[usize]) -> BuildingSet {
        indices
            .iter()
            .fold(BuildingSet::EMPTY, |set, &i| set.with(id(i)))
    }

    fn building(key: &str, supersedes: &[usize]) -> Building {
        Building {
            key: key.into(),
            name: key.into(),
            slot: None,
            supersedes: set(supersedes),
            forbidden_with: None,
            auto_with: BuildingSet::EMPTY,
            roles: BuildingRoles::default(),
        }
    }

    /// A faction's record: the Village Hall (0), the Town Hall (1), the City Hall (2), the
    /// Capitol (3), each superseding those below it (closed), a dwelling (4) and its upgrade (5).
    fn record() -> TownRecord {
        TownRecord {
            faction: FactionId::from_raw(2),
            moat: Moat::Harmless,
            siege_obstacles: false,
            silo: vec![],
            creatures: vec![],
            buildings: vec![
                building("village_hall", &[]),
                building("town_hall", &[0]),
                building("city_hall", &[0, 1]),
                building("capitol", &[0, 1, 2]),
                building("dwelling_1", &[]),
                building("upg_dwelling_1", &[4]),
            ],
            capitol_demoted_to: id(2),
        }
    }

    #[test]
    fn a_town_keeps_its_faction_and_sets() {
        let town = Town::new(FactionId::from_raw(2), set(&[0, 1]), set(&[5]));
        assert_eq!(town.faction(), FactionId::from_raw(2));
        assert_eq!((town.built(), town.forbidden()), (set(&[0, 1]), set(&[5])));
    }

    #[test]
    fn the_buildings_a_built_one_supersedes_do_not_stand() {
        let record = record();
        let capitol = Town::new(record.faction, set(&[0, 1, 2, 3, 4]), BuildingSet::EMPTY);
        assert_eq!(capitol.standing(&record), set(&[3, 4]));
        let upgraded = Town::new(record.faction, set(&[0, 4, 5]), BuildingSet::EMPTY);
        assert_eq!(upgraded.standing(&record), set(&[0, 5]));
        // A superseding building stands alone even when what it replaces was never built (a
        // custom set's), and a building the record lacks supersedes nothing.
        let alone = Town::new(record.faction, set(&[2, 63]), BuildingSet::EMPTY);
        assert_eq!(alone.standing(&record), set(&[2, 63]));
        let none = Town::new(record.faction, BuildingSet::EMPTY, BuildingSet::EMPTY);
        assert!(none.standing(&record).is_empty());
    }
}
