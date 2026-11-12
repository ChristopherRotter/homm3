//! A town's own record (ADR-0028 point 2; E09 increment 1's design note §4): [`Town`], what the
//! rules change (its faction, the buildings ever built and forbidden, the day's mark; increment
//! 3's available creatures and Portal of Summoning, design §40), and
//! [`TownStatic`], what the map gives and nothing changes (its name, its events). The store ([`crate::Towns`]) holds
//! both by `h3_contracts::TownId`.

use alloc::string::String;
use alloc::vec::Vec;

use h3_content::{CreatureTypeId, FactionId, ResourceId};
use h3_contracts::PlayerId;

use crate::building::BuildingSet;
use crate::ruleset::{MAX_CREATURE_LEVELS, TownRecord};

/// A town's creature indexes (E09 design §40): index `level - 1` the level's creature, `level - 1 +
/// 7` its upgrade (the original's +0x16).
pub const CREATURE_INDEXES: usize = 2 * MAX_CREATURE_LEVELS;

/// The most creatures of an index a town holds available, and the most a Portal of Summoning
/// holds (E09 design §40, `TWN-QUIRK-8`): every count saturates here, where the original's 16-bit
/// adds and doubles wrap (T-10, fixed).
pub const MAX_AVAILABLE: u16 = 32_767;

/// The creatures a Portal of Summoning holds (TWN-GRO-007; the original's +0x3C and +0x40).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Summoned {
    /// The creature, one of a dwelling's on the map.
    pub creature: CreatureTypeId,
    /// How many, at most [`MAX_AVAILABLE`].
    pub count: u16,
}

/// A town's record: its faction and the buildings ever built and forbidden, each a set over its
/// faction's town record's buildings ([`TownRecord::buildings`]), whether it was built in today
/// (the day's mark, E09 design §26), its available creatures by index and its Portal of
/// Summoning's (E09 design §40).
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
    built_today: bool,
    available: [u16; CREATURE_INDEXES],
    portal: Option<Summoned>,
}

impl Town {
    /// A town of `faction` with the buildings `built` ever built and `forbidden` forbidden, each
    /// a set over the faction's town record (the build's setup gives them, E09 T6c), not built in
    /// today, without available creatures, its Portal empty.
    pub const fn new(faction: FactionId, built: BuildingSet, forbidden: BuildingSet) -> Self {
        Self {
            faction,
            built,
            forbidden,
            built_today: false,
            available: [0; CREATURE_INDEXES],
            portal: None,
        }
    }

    /// The town with the buildings `built` ever built and the day's mark `built_today`, the rest
    /// kept (the store's writes, E09 design §26).
    pub(crate) const fn rebuilt(&self, built: BuildingSet, built_today: bool) -> Self {
        Self {
            built,
            built_today,
            ..*self
        }
    }

    /// The town with the buildings `built` ever built and `forbidden` forbidden, the rest kept (a
    /// Shipyard without water, [`crate::without_water`], E09 T18a).
    pub(crate) const fn reset(&self, built: BuildingSet, forbidden: BuildingSet) -> Self {
        Self {
            built,
            forbidden,
            ..*self
        }
    }

    /// The town with `count` creatures available at `index`, at most [`MAX_AVAILABLE`] (an index
    /// past [`CREATURE_INDEXES`]: the town unchanged), the rest kept (E09 design §40).
    #[must_use]
    pub fn with_available(&self, index: usize, count: u16) -> Self {
        let mut town = self.clone();
        if let Some(slot) = town.available.get_mut(index) {
            *slot = count.min(MAX_AVAILABLE);
        }
        town
    }

    /// The town with its Portal of Summoning holding `portal` (its count at most
    /// [`MAX_AVAILABLE`]; none: empty), the rest kept (E09 design §40).
    #[must_use]
    pub fn with_portal(&self, portal: Option<Summoned>) -> Self {
        Self {
            portal: portal.map(|p| Summoned {
                count: p.count.min(MAX_AVAILABLE),
                ..p
            }),
            ..self.clone()
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

    /// Whether the town was built in today (the original's +0x2 with the values 0 and 1 only, E09
    /// decision 43): set by a build of its owner's, cleared at each day's start and by a capture
    /// from a team without a human by one with one, never by a town event (E09 design §26, §30).
    pub const fn built_today(&self) -> bool {
        self.built_today
    }

    /// The creatures available at `index` (index `level - 1` the level's creature, `level - 1 + 7`
    /// its upgrade, [`CREATURE_INDEXES`]); 0 past them. At most [`MAX_AVAILABLE`].
    pub fn available(&self, index: usize) -> u16 {
        self.available.get(index).copied().unwrap_or(0)
    }

    /// The creatures available by index.
    pub const fn available_all(&self) -> &[u16; CREATURE_INDEXES] {
        &self.available
    }

    /// What its Portal of Summoning holds, if anything (TWN-GRO-007).
    pub const fn portal(&self) -> Option<Summoned> {
        self.portal
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
            requires: BuildingSet::EMPTY,
            cost: Vec::new(),
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
            roe: true,
        }
    }

    #[test]
    fn a_town_keeps_its_faction_and_sets() {
        let town = Town::new(FactionId::from_raw(2), set(&[0, 1]), set(&[5]));
        assert_eq!(town.faction(), FactionId::from_raw(2));
        assert_eq!((town.built(), town.forbidden()), (set(&[0, 1]), set(&[5])));
        assert!(!town.built_today());
        // A rebuilt town keeps its faction and forbidden set.
        let rebuilt = town.rebuilt(set(&[0, 1, 4]), true);
        assert_eq!(rebuilt.faction(), FactionId::from_raw(2));
        assert_eq!(
            (rebuilt.built(), rebuilt.forbidden()),
            (set(&[0, 1, 4]), set(&[5]))
        );
        assert!(rebuilt.built_today());
    }

    // TWN-QUIRK-8's regression test (T-10 fixed: a count saturates).
    #[test]
    fn twn_quirk_8_a_town_s_counts_saturate_at_the_bound() {
        let town = Town::new(FactionId::from_raw(2), set(&[0]), set(&[5]));
        assert_eq!(*town.available_all(), [0; CREATURE_INDEXES]);
        assert_eq!(
            (town.available(0), town.available(99), town.portal()),
            (0, 0, None)
        );
        // Each index apart, saturating at the bound (T-10 fixed); past the indexes nothing.
        let counted = town
            .with_available(0, 7)
            .with_available(13, u16::MAX)
            .with_available(14, 5);
        assert_eq!(
            (counted.available(0), counted.available(13)),
            (7, MAX_AVAILABLE)
        );
        assert_eq!(counted.available_all()[1..13], [0; 12]);
        assert_eq!(
            counted.with_available(0, MAX_AVAILABLE).available(0),
            MAX_AVAILABLE
        );
        let imp = CreatureTypeId::from_raw(4);
        let summoned = counted.with_portal(Some(Summoned {
            creature: imp,
            count: 40_000,
        }));
        assert_eq!(
            summoned.portal(),
            Some(Summoned {
                creature: imp,
                count: MAX_AVAILABLE
            })
        );
        assert_eq!(summoned.with_portal(None).portal(), None);
        // The store's writes keep them, and the faction, sets and mark.
        let rebuilt = summoned
            .rebuilt(set(&[0, 1]), true)
            .reset(set(&[0, 2]), set(&[3]));
        assert_eq!(
            (rebuilt.available(0), rebuilt.portal()),
            (7, summoned.portal())
        );
        assert_eq!(
            (
                rebuilt.faction(),
                rebuilt.built(),
                rebuilt.forbidden(),
                rebuilt.built_today()
            ),
            (FactionId::from_raw(2), set(&[0, 2]), set(&[3]), true)
        );
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
