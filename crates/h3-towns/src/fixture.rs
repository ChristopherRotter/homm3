//! The derivations' and the setup's test part (E09 T6b, T6c), built in code (a domain crate takes
//! no pack adapter as a dev-dependency): a core of three resources and three factions, a town part
//! with the Keep's full record (every role of increment 1, the Castle's and the Rampart's roles
//! side by side, map slots, four levels of creatures, a horde and its upgrade; increment 2's
//! requirements, costs and Shipyard, E09 T17b; increment 3's horde roles, Portal and week's
//! building, E09 T25a), the Marsh's small one (a silo that pays gold, one level of creatures, free
//! buildings without requirements, not a Restoration of Erathia record) and no record for the
//! Wild; the core's creatures by handle, with the Keep's and the Marsh's growths.

extern crate std;

use std::vec;
use std::vec::Vec;

use h3_content::{
    ContentKey, ContentPack, CreatureDef, CreatureInfoDef, CreatureTypeId, FactionDef, FactionId,
    PackManifest, ProfileDef, ResourceDef, ResourceId, Ruleset,
};
use h3_contracts::Moat;

use crate::building::{BuildingId, BuildingSet};
use crate::record::Town;
use crate::ruleset::{
    Building, BuildingRoles, CreatureLevel, DefenderBonus, Dwelling, GarrisonRow,
    GarrisonWeekParams, Horde, Reveal, TownParams, TownRecord, TownRuleset, WeekParams,
};
use crate::setup::SlotSet;

/// The core's resources, in its order.
pub(crate) const WOOD: ResourceId = ResourceId::from_raw(0);
pub(crate) const ORE: ResourceId = ResourceId::from_raw(1);
pub(crate) const GOLD: ResourceId = ResourceId::from_raw(2);

/// The core's factions, in its order: the Keep and the Marsh have records, the Wild none.
pub(crate) const KEEP: FactionId = FactionId::from_raw(0);
pub(crate) const MARSH: FactionId = FactionId::from_raw(1);
pub(crate) const WILD: FactionId = FactionId::from_raw(2);

/// The Keep's buildings by index.
pub(crate) const VILLAGE_HALL: usize = 0;
pub(crate) const TOWN_HALL: usize = 1;
pub(crate) const CITY_HALL: usize = 2;
pub(crate) const CAPITOL: usize = 3;
pub(crate) const FORT: usize = 4;
pub(crate) const CITADEL: usize = 5;
pub(crate) const CASTLE: usize = 6;
/// The Tavern: `tavern`, morale +1.
pub(crate) const TAVERN: usize = 7;
/// The Brotherhood of the Sword: `tavern`, morale +2, superseding the Tavern.
pub(crate) const BROTHERHOOD: usize = 8;
pub(crate) const SILO: usize = 9;
/// The Grail: `grail`, the defender's attack and defense +10 and mana +150, reveals all; every
/// week level 1's (`week_of`, E09 T25a).
pub(crate) const GRAIL: usize = 10;
/// The Lookout Tower: reveals 20.
pub(crate) const LOOKOUT: usize = 11;
/// The Fountain of Fortune: luck +2, superseding the Mystic Pond.
pub(crate) const FOUNTAIN: usize = 12;
/// The Mystic Pond: luck +1 (no original's pond has one: the fixture's, so that a superseded
/// luck building is read over the ever-built set).
pub(crate) const MYSTIC_POND: usize = 13;
pub(crate) const ESCAPE_TUNNEL: usize = 14;
/// The Blood Obelisk: the defender's attack +2.
pub(crate) const OBELISK: usize = 15;
/// The Glyphs of Fear: the defender's defense +2.
pub(crate) const GLYPHS: usize = 16;
pub(crate) const DWELLING_1: usize = 17;
/// The upgraded level-1 dwelling, superseding the dwelling.
pub(crate) const UPG_DWELLING_1: usize = 18;
pub(crate) const DWELLING_2: usize = 19;
/// The level-1 horde, superseding the level-1 dwelling: `horde = [1, false]`.
pub(crate) const HORDE_1: usize = 20;
/// The upgraded level-1 horde (no slot): forbidden with the horde, built once the horde and the
/// upgraded dwelling are, superseding both (and the dwelling): `horde = [1, true]`.
pub(crate) const UPG_HORDE_1: usize = 21;
/// The fixture's own shrine (no slot; no original's building is one): forbidden with the upgraded
/// horde and built once it is, so that both closures take two steps; automatic with the upgraded
/// horde without superseding it, so the hall never offers it in the upgraded horde's place. The
/// Portal of Summoning (E09 T25a).
pub(crate) const SHRINE: usize = 22;
/// The Shipyard: `shipyard`, requiring nothing (as the original's, E09 survey
/// `E09-construction.md` §6).
pub(crate) const SHIPYARD: usize = 23;

/// The Keep's buildings' map slots by index (the Mystic Pond, the upgraded horde and the shrine
/// have none): not their indices, so that a slot and a building's index are told apart.
const KEEP_SLOTS: [(usize, u8); 21] = [
    (SHIPYARD, 6),
    (VILLAGE_HALL, 10),
    (TOWN_HALL, 11),
    (CITY_HALL, 12),
    (CAPITOL, 13),
    (FORT, 7),
    (CITADEL, 8),
    (CASTLE, 9),
    (TAVERN, 5),
    (BROTHERHOOD, 22),
    (SILO, 15),
    (GRAIL, 26),
    (LOOKOUT, 21),
    (FOUNTAIN, 20),
    (ESCAPE_TUNNEL, 17),
    (OBELISK, 23),
    (GLYPHS, 24),
    (DWELLING_1, 30),
    (UPG_DWELLING_1, 37),
    (DWELLING_2, 31),
    (HORDE_1, 18),
];

/// The Keep's requirements (E09 design §24), each the closure as the compiler gives it: the halls'
/// chain to the Capitol through the Tavern and the Castle, the forts', the dwellings' from the
/// Fort, the hordes' from their dwellings, the upgraded horde's from the upgraded dwelling alone
/// (the original's, survey `E09-construction.md` §6); the Village Hall, the Grail, the Tavern, the
/// Mystic Pond, the Fort and the Shipyard require nothing.
const KEEP_REQUIRES: [(usize, &[usize]); 18] = [
    (TOWN_HALL, &[TAVERN]),
    (CITY_HALL, &[TOWN_HALL, TAVERN]),
    (
        CAPITOL,
        &[CITY_HALL, TOWN_HALL, TAVERN, CASTLE, CITADEL, FORT],
    ),
    (CITADEL, &[FORT]),
    (CASTLE, &[CITADEL, FORT]),
    (BROTHERHOOD, &[TAVERN]),
    (SILO, &[TAVERN]),
    (LOOKOUT, &[FORT]),
    (FOUNTAIN, &[MYSTIC_POND]),
    (ESCAPE_TUNNEL, &[FORT]),
    (OBELISK, &[FORT]),
    (GLYPHS, &[FORT]),
    (DWELLING_1, &[FORT]),
    (UPG_DWELLING_1, &[DWELLING_1, FORT]),
    (DWELLING_2, &[DWELLING_1, FORT]),
    (HORDE_1, &[DWELLING_1, FORT]),
    (UPG_HORDE_1, &[UPG_DWELLING_1, DWELLING_1, FORT]),
    (SHRINE, &[UPG_HORDE_1, UPG_DWELLING_1, DWELLING_1, FORT]),
];

/// The Keep's costs in the core's resource order (wood, ore, gold): every building but the Village
/// Hall and the Grail, which are free.
const KEEP_COSTS: [(usize, &[(ResourceId, u32)]); 22] = [
    (TOWN_HALL, &[(GOLD, 2500)]),
    (CITY_HALL, &[(WOOD, 5), (GOLD, 5000)]),
    (CAPITOL, &[(GOLD, 10_000)]),
    (FORT, &[(WOOD, 20), (ORE, 20), (GOLD, 5000)]),
    (CITADEL, &[(ORE, 5), (GOLD, 2500)]),
    (CASTLE, &[(WOOD, 10), (ORE, 10), (GOLD, 5000)]),
    (TAVERN, &[(WOOD, 5), (GOLD, 500)]),
    (BROTHERHOOD, &[(GOLD, 500)]),
    (SILO, &[(ORE, 5), (GOLD, 5000)]),
    (LOOKOUT, &[(WOOD, 5), (GOLD, 1000)]),
    (FOUNTAIN, &[(GOLD, 1500)]),
    (MYSTIC_POND, &[(GOLD, 2000)]),
    (ESCAPE_TUNNEL, &[(GOLD, 2000)]),
    (OBELISK, &[(GOLD, 1000)]),
    (GLYPHS, &[(GOLD, 1000)]),
    (DWELLING_1, &[(ORE, 10), (GOLD, 500)]),
    (UPG_DWELLING_1, &[(ORE, 5), (GOLD, 1000)]),
    (DWELLING_2, &[(WOOD, 5), (ORE, 5), (GOLD, 1000)]),
    (HORDE_1, &[(GOLD, 1000)]),
    (UPG_HORDE_1, &[(GOLD, 1000)]),
    (SHRINE, &[(GOLD, 100)]),
    (SHIPYARD, &[(WOOD, 20), (GOLD, 2000)]),
];

/// A map slot no building of the Keep's has.
pub(crate) const NO_BUILDING_SLOT: u8 = 40;

/// The creature of handle `raw`: the Keep's level `n` is `10 n`, its upgrade `10 n + 1` (none at
/// level 3); the Marsh's one level 50, without an upgrade.
pub(crate) const fn creature(raw: u16) -> CreatureTypeId {
    CreatureTypeId::from_raw(raw)
}

/// The core's creatures' handles: `0` to `CREATURES - 1`.
const CREATURES: u16 = 52;

/// The core's weekly growth and horde growth of the creature of handle `raw` (E09 T25a): the
/// Keep's level 1 14 (its horde 8, for the base and the upgrade), level 2 9, level 3 7, level 4 4
/// (each upgrade as its base), the Marsh's 6; every other creature 0.
pub(crate) const fn growths(raw: u16) -> (u16, u16) {
    match raw {
        10 | 11 => (14, 8),
        20 | 21 => (9, 0),
        30 => (7, 0),
        40 | 41 => (4, 0),
        50 => (6, 0),
        _ => (0, 0),
    }
}

/// The set of the Keep's buildings at `indices`.
pub(crate) fn set(indices: &[usize]) -> BuildingSet {
    indices.iter().fold(BuildingSet::EMPTY, |set, &i| {
        set.with(BuildingId::from_index(i).unwrap())
    })
}

/// The slots of the Keep's buildings at `indices` (each has one).
pub(crate) fn slots(indices: &[usize]) -> SlotSet {
    indices.iter().fold(SlotSet::EMPTY, |set, &i| {
        set.with(KEEP_SLOTS.iter().find(|&&(b, _)| b == i).unwrap().1)
    })
}

/// A town of `faction` with the buildings at `built` ever built, none forbidden.
pub(crate) fn town_of(faction: FactionId, built: &[usize]) -> Town {
    Town::new(faction, set(built), BuildingSet::EMPTY)
}

/// A Keep town with the buildings at `built` ever built, none forbidden.
pub(crate) fn town(built: &[usize]) -> Town {
    town_of(KEEP, built)
}

fn key(text: &str) -> ContentKey {
    ContentKey::parse(text).unwrap()
}

/// The core: the resources `t:wood`, `t:ore`, `t:gold`, the factions `t:keep`, `t:marsh`,
/// `t:wild` and the creatures `t:c0` to `t:c51` (handles 0 to 51), with [`growths`]; its profile
/// enables no quirk flag.
pub(crate) fn core() -> Ruleset {
    core_with(&[])
}

/// [`core`] with its profile enabling the quirk flags named in `quirks` (E09 T25b1).
pub(crate) fn core_with(quirks: &[&str]) -> Ruleset {
    let resource = |k: &str| ResourceDef {
        key: key(k),
        name: "Resource".into(),
    };
    let faction = |k: &str| FactionDef {
        key: key(k),
        name: "Faction".into(),
        native_terrain: None,
    };
    let creature = |raw: u16| {
        let (growth, horde_growth) = growths(raw);
        CreatureDef {
            key: key(&std::format!("t:c{raw}")),
            speed: 1,
            info: CreatureInfoDef {
                growth,
                horde_growth,
                ..CreatureInfoDef::default()
            },
        }
    };
    let pack = ContentPack {
        resources: vec![resource("t:wood"), resource("t:ore"), resource("t:gold")],
        factions: vec![faction("t:keep"), faction("t:marsh"), faction("t:wild")],
        creatures: (0..CREATURES).map(creature).collect(),
        profiles: vec![ProfileDef {
            key: key("t:faithful"),
            quirks: quirks.iter().map(|q| ((*q).into(), true)).collect(),
        }],
        ..ContentPack::new(PackManifest {
            id: "t".into(),
            version: "1".into(),
            depends: vec![],
        })
    };
    Ruleset::compile(&[pack], "t:faithful").unwrap()
}

fn building(key: &str, supersedes: &[usize], roles: BuildingRoles) -> Building {
    Building {
        key: key.into(),
        name: key.into(),
        slot: None,
        supersedes: set(supersedes),
        forbidden_with: None,
        auto_with: BuildingSet::EMPTY,
        requires: BuildingSet::EMPTY,
        cost: Vec::new(),
        roles,
    }
}

fn roles(change: impl FnOnce(&mut BuildingRoles)) -> BuildingRoles {
    let mut roles = BuildingRoles::default();
    change(&mut roles);
    roles
}

/// The Keep's record: the buildings above at their slots, a damaging moat, the natural obstacles,
/// a silo of 1 wood and 1 ore, four levels of creatures, its demoted Capitol the City Hall.
fn keep() -> TownRecord {
    let mut buildings: Vec<Building> = vec![
        building("village_hall", &[], roles(|r| r.hall = Some(1))),
        building("town_hall", &[0], roles(|r| r.hall = Some(2))),
        building("city_hall", &[0, 1], roles(|r| r.hall = Some(3))),
        building("capitol", &[0, 1, 2], roles(|r| r.hall = Some(4))),
        building("fort", &[], roles(|r| r.fort = Some(1))),
        building("citadel", &[4], roles(|r| r.fort = Some(2))),
        building("castle", &[4, 5], roles(|r| r.fort = Some(3))),
        building(
            "tavern",
            &[],
            roles(|r| {
                r.tavern = true;
                r.morale = 1;
            }),
        ),
        building(
            "brotherhood",
            &[TAVERN],
            roles(|r| {
                r.tavern = true;
                r.morale = 2;
            }),
        ),
        building("resource_silo", &[], roles(|r| r.silo = true)),
        building(
            "grail",
            &[],
            roles(|r| {
                r.grail = true;
                r.defender = DefenderBonus {
                    attack: 10,
                    defense: 10,
                    power: 0,
                    mana: 150,
                };
                r.reveal = Some(Reveal::All);
                r.week_of = Some(1);
            }),
        ),
        building(
            "lookout",
            &[],
            roles(|r| r.reveal = Some(Reveal::Radius(20))),
        ),
        building("fountain", &[MYSTIC_POND], roles(|r| r.luck = 2)),
        building("mystic_pond", &[], roles(|r| r.luck = 1)),
        building("escape_tunnel", &[], roles(|r| r.escape_tunnel = true)),
        building("obelisk", &[], roles(|r| r.defender.attack = 2)),
        building("glyphs", &[], roles(|r| r.defender.defense = 2)),
        building(
            "dwelling_1",
            &[],
            roles(|r| {
                r.dwelling = Some(Dwelling {
                    level: 1,
                    upgraded: false,
                });
            }),
        ),
        building(
            "upg_dwelling_1",
            &[DWELLING_1],
            roles(|r| {
                r.dwelling = Some(Dwelling {
                    level: 1,
                    upgraded: true,
                });
            }),
        ),
        building(
            "dwelling_2",
            &[],
            roles(|r| {
                r.dwelling = Some(Dwelling {
                    level: 2,
                    upgraded: false,
                });
            }),
        ),
        building(
            "horde_1",
            &[DWELLING_1],
            roles(|r| r.horde = Some(horde(false))),
        ),
        Building {
            forbidden_with: BuildingId::from_index(HORDE_1),
            auto_with: set(&[HORDE_1, UPG_DWELLING_1]),
            ..building(
                "upg_horde_1",
                &[HORDE_1, UPG_DWELLING_1, DWELLING_1],
                roles(|r| r.horde = Some(horde(true))),
            )
        },
        Building {
            forbidden_with: BuildingId::from_index(UPG_HORDE_1),
            auto_with: set(&[UPG_HORDE_1]),
            ..building("shrine", &[], roles(|r| r.portal = true))
        },
        building("shipyard", &[], roles(|r| r.shipyard = true)),
    ];
    for (index, slot) in KEEP_SLOTS {
        buildings[index].slot = Some(slot);
    }
    for (index, requires) in KEEP_REQUIRES {
        buildings[index].requires = set(requires);
    }
    for (index, cost) in KEEP_COSTS {
        buildings[index].cost = cost.to_vec();
    }
    let level = |base: u16, upgrade: Option<u16>| CreatureLevel {
        base: creature(base),
        upgrade: upgrade.map(creature),
    };
    TownRecord {
        faction: KEEP,
        moat: Moat::Damaging,
        siege_obstacles: true,
        silo: vec![(WOOD, 1), (ORE, 1)],
        creatures: vec![
            level(10, Some(11)),
            level(20, Some(21)),
            level(30, None),
            level(40, Some(41)),
        ],
        buildings,
        capitol_demoted_to: BuildingId::from_index(CITY_HALL).unwrap(),
        roe: true,
    }
}

/// The level-1 horde, upgraded or not.
const fn horde(upgraded: bool) -> Horde {
    Horde { level: 1, upgraded }
}

/// The Marsh's record: its Village Hall (0), a silo (1) that pays 250 gold and a Citadel (2), no
/// slot, a harmless moat, no natural obstacles, one level of creatures.
fn marsh() -> TownRecord {
    TownRecord {
        faction: MARSH,
        moat: Moat::Harmless,
        siege_obstacles: false,
        silo: vec![(GOLD, 250)],
        creatures: vec![CreatureLevel {
            base: creature(50),
            upgrade: None,
        }],
        buildings: vec![
            building("village_hall", &[], roles(|r| r.hall = Some(1))),
            building("resource_silo", &[], roles(|r| r.silo = true)),
            building("citadel", &[], roles(|r| r.fort = Some(2))),
        ],
        capitol_demoted_to: BuildingId::from_index(0).unwrap(),
        roe: false,
    }
}

/// The parameters: the original's halls and Grail in gold, a reveal radius of 5.
fn params() -> TownParams {
    TownParams {
        gold: GOLD,
        hall_gold: [500, 1000, 2000, 4000],
        grail_gold: 5000,
        reveal_radius: 5,
        level_2_dwelling_chance: 30,
        neutral_garrison: [GarrisonRow {
            chance: 33,
            count: [1, 3],
        }; 4],
        capitol_demoted_to: "city_hall".into(),
        visions_factor: 3,
        visions_min_radius: 3,
        visions_creatures: Vec::new(),
        week: WeekParams {
            creature_chance: [1, 4],
            creature_bonus: 5,
        },
        garrison_week: GarrisonWeekParams {
            odds: [40, 80],
            out_of: 101,
            weights: vec![2, 3, 4, 5, 4, 3],
            weeks: 8,
            upgrade: 5,
        },
    }
}

/// The part: the Keep's and the Marsh's records, the parameters.
pub(crate) fn part() -> TownRuleset {
    part_with(|_| {})
}

/// The part with its parameters changed by `change`.
pub(crate) fn part_with(change: impl FnOnce(&mut TownParams)) -> TownRuleset {
    let mut params = params();
    change(&mut params);
    TownRuleset::new(&core(), Some(params), vec![keep(), marsh()])
}

/// The part with the Keep's record changed by `change`.
pub(crate) fn part_with_keep(change: impl FnOnce(&mut TownRecord)) -> TownRuleset {
    let mut keep = keep();
    change(&mut keep);
    TownRuleset::new(&core(), Some(params()), vec![keep, marsh()])
}

/// The part without towns (no parameters, no record).
pub(crate) fn empty_part() -> TownRuleset {
    TownRuleset::empty(&core())
}
