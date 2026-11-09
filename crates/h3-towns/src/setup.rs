//! A town's setup at the game's build (E09 increment 1's design note §5): what the map gives a
//! town ([`TownSetup`]: its faction, its buildings by the map editor's slots, its garrison, its
//! name and events) and the pure functions `h3-game` calls with the setup stream once it has
//! resolved the faction (design §14, E09 T7b2): the buildings ([`town_buildings_setup`],
//! TWN-SET-003), the garrison ([`town_garrison`], TWN-SET-004) and the whole town
//! ([`town_setup`]). The original's `town_setup` (0x5C0670) and `town_buildings_setup` (0x5C08C0;
//! E09 survey §5).

use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

use h3_content::{CreatureTypeId, FactionId, ResourceId};
use h3_contracts::{ArmyStack, PlayerId};
use h3_kernel::Rng;

use crate::building::{BuildingId, BuildingSet, MAX_BUILDINGS};
use crate::record::{Town, TownEvent, TownStatic};
use crate::ruleset::{
    BUILDING_SLOTS, Building, Dwelling, MAX_CREATURE_LEVELS, TownParams, TownRecord, TownRuleset,
};

#[cfg(test)]
mod tests;

/// A set of the map editor's building slots, a bit per slot below [`BUILDING_SLOTS`] (the h3m's
/// 41 bits of a town's built or forbidden buildings; E09 survey §5.3). A slot names a building
/// only through a faction's record: the building whose [`Building::slot`] it is
/// ([`SlotSet::buildings`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub struct SlotSet(u64);

impl SlotSet {
    /// No slot.
    pub const EMPTY: Self = Self(0);

    /// Every bit set below [`BUILDING_SLOTS`].
    const MASK: u64 = (1 << BUILDING_SLOTS) - 1;

    /// The set of the slots whose bits `bits` sets, bit `i` for slot `i` (the h3m's order); the
    /// bits from [`BUILDING_SLOTS`] up are dropped.
    pub const fn from_bits(bits: u64) -> Self {
        Self(bits & Self::MASK)
    }

    /// The set's bits, bit `i` for slot `i`.
    pub const fn bits(self) -> u64 {
        self.0
    }

    /// Whether the set holds `slot` (never a slot from [`BUILDING_SLOTS`] up).
    pub const fn contains(self, slot: u8) -> bool {
        slot < BUILDING_SLOTS && self.0 & (1 << slot) != 0
    }

    /// The set with `slot` added; a slot from [`BUILDING_SLOTS`] up is no slot, the set is
    /// unchanged.
    #[must_use]
    pub const fn with(self, slot: u8) -> Self {
        if slot < BUILDING_SLOTS {
            Self(self.0 | (1 << slot))
        } else {
            self
        }
    }

    /// The buildings of `record` (a faction's town record) at the set's slots: each building
    /// whose slot the set holds; a slot without one of the faction's buildings gives nothing.
    pub fn buildings(self, record: &TownRecord) -> BuildingSet {
        indexed(record)
            .filter(|(_, b)| b.slot.is_some_and(|slot| self.contains(slot)))
            .fold(BuildingSet::EMPTY, |set, (id, _)| set.with(id))
    }
}

/// A town's faction as the map gives it; the game resolves it before the setup (design §14, E09
/// T7b2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TownFaction {
    /// A fixed faction, which has a town record in the town part.
    Fixed(FactionId),
    /// A random town's faction, drawn by the game.
    Random {
        /// The player colour (0 to 7) whose faction it takes, the SoD alignment byte's; `None`
        /// for none (the byte's `0xFF`).
        alignment: Option<u8>,
    },
}

/// A town's buildings as the map gives them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildingsSetup {
    /// The standard set (the h3m's custom-buildings flag clear): the Village Hall, a Fort with
    /// the map's flag, the start's Tavern and dwellings (TWN-SET-003).
    Standard {
        /// Whether the town has a Fort (the h3m's has-fort flag).
        fort: bool,
    },
    /// A custom set: the slots the map builds and those it forbids.
    Custom {
        /// The slots built.
        built: SlotSet,
        /// The slots forbidden.
        forbidden: SlotSet,
    },
}

/// A stack of a town's custom garrison as the map gives it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GarrisonStack {
    /// Its army slot (0-based).
    pub slot: u8,
    /// Its creature.
    pub creature: GarrisonCreature,
    /// Its count; 0 leaves the slot empty (the original's).
    pub count: u32,
}

/// A custom garrison stack's creature.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GarrisonCreature {
    /// A creature of the core ruleset.
    Creature(CreatureTypeId),
    /// A random stack (the h3m's codes from −2 down): the town's faction's creature of `level`.
    Random {
        /// The level, 1 to 7.
        level: u8,
        /// Whether it is the level's upgrade (the code odd).
        upgraded: bool,
    },
}

/// A town event as the map gives it: [`TownEvent`]'s fields, its buildings by the map editor's
/// slots (resolved to the town's faction's buildings by [`town_setup`]).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TownEventSetup {
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
    /// The slots of the buildings it builds.
    pub buildings: SlotSet,
    /// The creatures it adds to the town's dwellings by level, from level 1.
    pub creatures: [u16; MAX_CREATURE_LEVELS],
}

/// A town as the map gives it (in the game's setup, design §13): what [`town_setup`] makes a
/// [`Town`] of once the game has resolved its faction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TownSetup {
    /// Its faction, fixed or random.
    pub faction: TownFaction,
    /// Its buildings.
    pub buildings: BuildingsSetup,
    /// Its custom garrison, in the map's order (each slot once); `None` for the map's default
    /// (drawn for a neutral town, TWN-SET-004).
    pub garrison: Option<Vec<GarrisonStack>>,
    /// Its name, if the map gives one.
    pub name: Option<String>,
    /// Its events, in the map's order.
    pub events: Vec<TownEventSetup>,
}

/// A town as the build makes it ([`town_setup`]): its record, its static part and its garrison,
/// which the game writes into the adventure (design §8).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuiltTown {
    /// Its record.
    pub town: Town,
    /// Its name and events, the events' buildings its faction's.
    pub town_static: TownStatic,
    /// Its garrison, each stack at least 1 creature, slots distinct.
    pub garrison: Vec<ArmyStack>,
}

/// Why a town cannot be set up.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TownSetupError {
    /// The town part has no town record of the faction (or, a part built without the compiler,
    /// no parameters).
    NoTownRecord(FactionId),
}

impl fmt::Display for TownSetupError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoTownRecord(faction) => write!(
                f,
                "faction {} has no town record in the town part",
                faction.raw()
            ),
        }
    }
}

impl core::error::Error for TownSetupError {}

/// The record's buildings with their ids, in index order.
pub(crate) fn indexed(record: &TownRecord) -> impl Iterator<Item = (BuildingId, &Building)> {
    (0..MAX_BUILDINGS)
        .filter_map(BuildingId::from_index)
        .zip(&record.buildings)
}

/// The record's first building, in index order, for which `pick` holds.
fn first(record: &TownRecord, pick: impl Fn(&Building) -> bool) -> Option<BuildingId> {
    indexed(record).find(|(_, b)| pick(b)).map(|(id, _)| id)
}

/// `id` with every building it supersedes (the compiler's closure).
pub(crate) fn closed(record: &TownRecord, id: BuildingId) -> BuildingSet {
    record
        .building(id)
        .map_or(BuildingSet::EMPTY, |b| b.supersedes)
        .with(id)
}

/// `forbidden` with every building whose `forbidden_with` is forbidden, to a fixpoint (the
/// compiler refuses a cycle).
fn close_forbidden(record: &TownRecord, mut forbidden: BuildingSet) -> BuildingSet {
    loop {
        let next = indexed(record)
            .filter(|(_, b)| {
                b.forbidden_with
                    .is_some_and(|with| forbidden.contains(with))
            })
            .fold(forbidden, |set, (id, _)| set.with(id));
        if next == forbidden {
            return forbidden;
        }
        forbidden = next;
    }
}

/// The hordes' upgrade (E09 survey §16.2, decision 29): each building not forbidden whose
/// non-empty `auto_with` is all ever built becomes ever built with what it supersedes (less the
/// forbidden), to a fixpoint. A build's ([`crate::built_with`]) tests no forbidden set (E09
/// design §27).
pub(crate) fn upgrade_hordes(
    record: &TownRecord,
    mut built: BuildingSet,
    forbidden: BuildingSet,
) -> BuildingSet {
    loop {
        let next = indexed(record)
            .filter(|&(id, b)| {
                !b.auto_with.is_empty()
                    && b.auto_with.difference(built).is_empty()
                    && !forbidden.contains(id)
            })
            .fold(built, |set, (id, b)| {
                set.union(b.supersedes.difference(forbidden).with(id))
            });
        if next == built {
            return built;
        }
        built = next;
    }
}

/// The start's Tavern: the record's first building of role `tavern` that supersedes no other
/// (the Tavern, not the Castle's Brotherhood of the Sword).
fn tavern(record: &TownRecord) -> Option<BuildingId> {
    let taverns = indexed(record)
        .filter(|(_, b)| b.roles.tavern)
        .fold(BuildingSet::EMPTY, |set, (id, _)| set.with(id));
    first(record, |b| {
        b.roles.tavern && b.supersedes.difference(taverns) == b.supersedes
    })
}

/// A town's buildings at the start (E09 survey §5.3, design §5), of `record`'s faction: its
/// buildings ever built and forbidden.
///
/// The record's Village Hall (its first building of `hall` 1) is ever built, whatever the set.
/// A custom set's forbidden buildings are its forbidden slots' with each building whose
/// `forbidden_with` is forbidden; its built ones each built slot's building with what it
/// supersedes, less the forbidden. A standard set builds a Fort (its first of `fort` 1) with
/// `fort`; an owned town's (`owned`) its Tavern and, only with a Fort, its level-1 dwelling and,
/// when one draw `1..=100` from `draws` is at most `params`' `level_2_dwelling_chance`, its
/// level-2 one (no draw without a Fort); a neutral town's both dwellings, Fort or not, without a
/// draw; each with what it supersedes. Then the hordes' upgrade: a building (not forbidden) whose
/// `auto_with` is all ever built is ever built too, with what it supersedes, so that a horde and
/// its level's upgraded dwelling stand as the upgraded horde alone (decision 29). The standing set
/// is derived ([`Town::standing`]), so a custom set's horde never leaves its dwelling standing
/// (the original's does: `TWN-QUIRK-5`, fixed).
// rule: TWN-SET-003
pub fn town_buildings_setup(
    record: &TownRecord,
    params: &TownParams,
    setup: &BuildingsSetup,
    owned: bool,
    draws: &mut Rng,
) -> Town {
    let hall = first(record, |b| b.roles.hall == Some(1))
        .map_or(BuildingSet::EMPTY, |id| closed(record, id));
    // The original's forbidden from the start whatever the set: the Grail building on a map whose
    // victory is to build the Grail in another town (0x5F2160, increment 2). The Shipyard without
    // a boat's spot by the entrance (0x5C0C90) is the game's, once the static map fixed the spots
    // ([`without_water`], E09 T18a).
    // stub: TWN-STUB-2
    let start_forbidden = BuildingSet::EMPTY;
    let (picked, forbidden) = match *setup {
        BuildingsSetup::Custom { built, forbidden } => (
            built
                .buildings(record)
                .iter()
                .fold(BuildingSet::EMPTY, |set, id| set.union(closed(record, id))),
            close_forbidden(record, start_forbidden.union(forbidden.buildings(record))),
        ),
        BuildingsSetup::Standard { fort } => {
            let level = |level| Dwelling {
                level,
                upgraded: false,
            };
            let (tavern, dwellings) = if !owned {
                (None, [Some(level(1)), Some(level(2))])
            } else if fort {
                let second =
                    draws.range_inclusive(1, 100) <= u32::from(params.level_2_dwelling_chance);
                (tavern(record), [Some(level(1)), second.then_some(level(2))])
            } else {
                (tavern(record), [None, None])
            };
            let fort = if fort {
                first(record, |b| b.roles.fort == Some(1))
            } else {
                None
            };
            let dwellings = dwellings
                .into_iter()
                .flatten()
                .map(|d| first(record, |b| b.roles.dwelling == Some(d)));
            let picked = [fort, tavern]
                .into_iter()
                .chain(dwellings)
                .flatten()
                .fold(BuildingSet::EMPTY, |set, id| set.union(closed(record, id)));
            (picked, start_forbidden)
        }
    };
    let built = upgrade_hordes(record, hall.union(picked.difference(forbidden)), forbidden);
    Town::new(record.faction, built, forbidden)
}

/// `town` without water (E09 T18a, design §28; TWN-SET-011): the town whose spot the static map
/// left empty, its record's buildings of role `shipyard` forbidden and out of the ever-built set,
/// its faction, its other buildings (one that requires the Shipyard among them: a custom set's
/// Lighthouse stays, as the original's) and its day's mark kept; a faction without a record or a
/// Shipyard, the town unchanged. The original forbids the Shipyard before the map's sets
/// (`town_buildings_setup` 0x5C08C0's step 2, survey `E09-towns.md` §5.3), which then build it
/// nowhere: the same sets, since no building of a record supersedes, is forbidden with or is
/// built automatically with the Shipyard. The game calls it at its build, after the adventure
/// fixed the spots and before the store is made; no draw.
// rule: TWN-SET-011
pub fn without_water(town: &Town, part: &TownRuleset) -> Town {
    let shipyards = part
        .town(town.faction())
        .map_or(BuildingSet::EMPTY, |record| {
            indexed(record)
                .filter(|(_, b)| b.roles.shipyard)
                .fold(BuildingSet::EMPTY, |set, (id, _)| set.with(id))
        });
    town.reset(
        town.built().difference(shipyards),
        town.forbidden().union(shipyards),
    )
}

/// A town's garrison at the start (E09 survey §5.2, design §5), of `record`'s faction.
///
/// A custom garrison (`Some`) as the map gives it, in its order: a random stack is the record's
/// creature of its level, the level's upgrade when `upgraded` (the base at a level without one);
/// a stack of count 0, or a random one at a level the record lacks, leaves its slot empty; no
/// draw, so a custom empty garrison stays empty (TD-5). Without one (`None`), an owned town's is
/// empty; a neutral town's is drawn from `draws`: for each of `params`' `neutral_garrison` rows,
/// levels 1 to 4 in order (only the levels the record has), one draw `1..=100` at most the row's
/// chance adds a stack of the level's base creature, its count drawn in the row's bounds, in the
/// first free slot (from 0).
// rule: TWN-SET-004
pub fn town_garrison(
    record: &TownRecord,
    params: &TownParams,
    garrison: Option<&[GarrisonStack]>,
    owned: bool,
    draws: &mut Rng,
) -> Vec<ArmyStack> {
    match garrison {
        Some(stacks) => stacks
            .iter()
            .filter(|stack| stack.count > 0)
            .filter_map(|stack| {
                let creature = match stack.creature {
                    GarrisonCreature::Creature(creature) => Some(creature),
                    GarrisonCreature::Random { level, upgraded } => usize::from(level)
                        .checked_sub(1)
                        .and_then(|index| record.creatures.get(index))
                        .map(|l| {
                            if upgraded {
                                l.upgrade.unwrap_or(l.base)
                            } else {
                                l.base
                            }
                        }),
                };
                creature.map(|creature| ArmyStack {
                    slot: stack.slot,
                    creature,
                    count: stack.count,
                })
            })
            .collect(),
        None if owned => Vec::new(),
        None => {
            let mut army = Vec::new();
            for (row, level) in params.neutral_garrison.iter().zip(&record.creatures) {
                if draws.range_inclusive(1, 100) <= u32::from(row.chance) {
                    let [least, most] = row.count.map(u32::from);
                    let count = draws.range_inclusive(least, most);
                    // The first free slot: the stacks so far fill the slots from 0.
                    let slot = u8::try_from(army.len()).unwrap_or(u8::MAX);
                    army.push(ArmyStack {
                        slot,
                        creature: level.base,
                        count,
                    });
                }
            }
            army
        }
    }
}

/// A town event of `record`'s faction: the setup's, its slots resolved to the faction's buildings.
fn town_event(record: &TownRecord, event: &TownEventSetup) -> TownEvent {
    TownEvent {
        players: event.players.clone(),
        human: event.human,
        computer: event.computer,
        first_day: event.first_day,
        interval: event.interval,
        resources: event.resources.clone(),
        buildings: event.buildings.buildings(record),
        creatures: event.creatures,
    }
}

/// A town at the game's build (E09 survey §5.1, design §5): `setup`, its faction resolved to
/// `faction` by the game, owned by a player when `owned`, with `part`'s record of the faction and
/// its parameters, drawing from `draws` (the setup stream): the garrison's draws first
/// ([`town_garrison`]), then the buildings' ([`town_buildings_setup`]), the original's order;
/// its name and its events, their slots resolved to the faction's buildings.
///
/// # Errors
///
/// [`TownSetupError::NoTownRecord`] when `part` has no record of `faction` (or no parameters);
/// nothing is drawn then.
pub fn town_setup(
    setup: &TownSetup,
    faction: FactionId,
    owned: bool,
    part: &TownRuleset,
    draws: &mut Rng,
) -> Result<BuiltTown, TownSetupError> {
    let (record, params) = part
        .town(faction)
        .zip(part.params())
        .ok_or(TownSetupError::NoTownRecord(faction))?;
    let garrison = town_garrison(record, params, setup.garrison.as_deref(), owned, draws);
    let town = town_buildings_setup(record, params, &setup.buildings, owned, draws);
    let town_static = TownStatic {
        name: setup.name.clone(),
        events: setup
            .events
            .iter()
            .map(|event| town_event(record, event))
            .collect(),
    };
    Ok(BuiltTown {
        town,
        town_static,
        garrison,
    })
}
