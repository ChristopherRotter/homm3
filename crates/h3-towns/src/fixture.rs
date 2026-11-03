//! The derivations' test part (E09 T6b), built in code (a domain crate takes no pack adapter as a
//! dev-dependency): a core of three resources and three factions, a town part with the Keep's full
//! record (every role of increment 1, the Castle's and the Rampart's roles side by side), the
//! Marsh's small one (a silo that pays gold) and no record for the Wild.

extern crate std;

use std::vec;
use std::vec::Vec;

use h3_content::{
    ContentKey, ContentPack, FactionDef, FactionId, PackManifest, ProfileDef, ResourceDef,
    ResourceId, Ruleset,
};
use h3_contracts::Moat;

use crate::building::{BuildingId, BuildingSet};
use crate::record::Town;
use crate::ruleset::{
    Building, BuildingRoles, DefenderBonus, Dwelling, GarrisonRow, Reveal, TownParams, TownRecord,
    TownRuleset,
};

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
/// The Grail: `grail`, the defender's attack and defense +10 and mana +150, reveals all.
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

/// The set of the Keep's buildings at `indices`.
pub(crate) fn set(indices: &[usize]) -> BuildingSet {
    indices.iter().fold(BuildingSet::EMPTY, |set, &i| {
        set.with(BuildingId::from_index(i).unwrap())
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

/// The core: the resources `t:wood`, `t:ore`, `t:gold` and the factions `t:keep`, `t:marsh`,
/// `t:wild`.
pub(crate) fn core() -> Ruleset {
    let resource = |k: &str| ResourceDef {
        key: key(k),
        name: "Resource".into(),
    };
    let faction = |k: &str| FactionDef {
        key: key(k),
        name: "Faction".into(),
        native_terrain: None,
    };
    let pack = ContentPack {
        resources: vec![resource("t:wood"), resource("t:ore"), resource("t:gold")],
        factions: vec![faction("t:keep"), faction("t:marsh"), faction("t:wild")],
        profiles: vec![ProfileDef {
            key: key("t:faithful"),
            quirks: vec![],
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
        roles,
    }
}

fn roles(change: impl FnOnce(&mut BuildingRoles)) -> BuildingRoles {
    let mut roles = BuildingRoles::default();
    change(&mut roles);
    roles
}

/// The Keep's record: the buildings above, a damaging moat, the natural obstacles, a silo of 1
/// wood and 1 ore, its demoted Capitol the City Hall.
fn keep() -> TownRecord {
    let buildings: Vec<Building> = vec![
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
    ];
    TownRecord {
        faction: KEEP,
        moat: Moat::Damaging,
        siege_obstacles: true,
        silo: vec![(WOOD, 1), (ORE, 1)],
        creatures: vec![],
        buildings,
        capitol_demoted_to: BuildingId::from_index(CITY_HALL).unwrap(),
    }
}

/// The Marsh's record: its Village Hall (0) and a silo (1) that pays 250 gold, a harmless moat,
/// no natural obstacles.
fn marsh() -> TownRecord {
    TownRecord {
        faction: MARSH,
        moat: Moat::Harmless,
        siege_obstacles: false,
        silo: vec![(GOLD, 250)],
        creatures: vec![],
        buildings: vec![
            building("village_hall", &[], roles(|r| r.hall = Some(1))),
            building("resource_silo", &[], roles(|r| r.silo = true)),
        ],
        capitol_demoted_to: BuildingId::from_index(0).unwrap(),
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
    }
}

/// The part: the Keep's and the Marsh's records, the parameters.
pub(crate) fn part() -> TownRuleset {
    TownRuleset::new(&core(), Some(params()), vec![keep(), marsh()])
}

/// The part without towns (no parameters, no record).
pub(crate) fn empty_part() -> TownRuleset {
    TownRuleset::empty(&core())
}
