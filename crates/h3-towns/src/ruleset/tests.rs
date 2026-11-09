//! Tests of the town part's compiler (E09 T5a1), on core rulesets built in code (a domain crate
//! takes no pack adapter as a dev-dependency): every check refused by a test, the hash pinned.

extern crate std;

use std::string::{String, ToString};
use std::vec;
use std::vec::Vec;

use h3_content::{
    ContentKey, ContentPack, CreatureDef, CreatureInfoDef, FactionDef, PackManifest, ProfileDef,
    ResourceDef, Ruleset,
};

use super::*;

fn key(text: &str) -> ContentKey {
    ContentKey::parse(text).unwrap()
}

fn manifest(id: &str, depends: &[&str]) -> PackManifest {
    PackManifest {
        id: id.into(),
        version: "1".into(),
        depends: depends.iter().map(|d| (*d).to_string()).collect(),
    }
}

fn faction(k: &str) -> FactionDef {
    FactionDef {
        key: key(k),
        name: "Faction".into(),
        native_terrain: None,
    }
}

fn resource(k: &str) -> ResourceDef {
    ResourceDef {
        key: key(k),
        name: "Resource".into(),
    }
}

fn creature(k: &str, faction: Option<&str>, level: u8, upgrades: &[&str]) -> CreatureDef {
    CreatureDef {
        key: key(k),
        speed: 5,
        info: CreatureInfoDef {
            faction: faction.map(key),
            level,
            upgrades: upgrades.iter().map(|u| key(u)).collect(),
            ..CreatureInfoDef::default()
        },
    }
}

/// Core packs: `t` (the resources `t:wood` and `t:gold`, the factions `t:keep` with `t:pike`
/// upgrading to `t:halberd` at level 1 and `t:archer` at level 2, and `t:wild` with `t:imp` and
/// `t:gog`; the neutral `t:wisp`; the profile), `u` (depends on `t`: the resource `u:gem`, the
/// faction `u:marsh` with `u:frog` and `u:toad`), `v` (no dependencies: the faction `v:dune` with
/// `v:a` and `v:b`) and `w` (depends on `t`, nothing defined).
fn core() -> Ruleset {
    let t = ContentPack {
        resources: vec![resource("t:wood"), resource("t:gold")],
        factions: vec![faction("t:keep"), faction("t:wild")],
        creatures: vec![
            creature("t:pike", Some("t:keep"), 1, &["t:halberd"]),
            creature("t:halberd", Some("t:keep"), 1, &[]),
            creature("t:archer", Some("t:keep"), 2, &[]),
            creature("t:imp", Some("t:wild"), 1, &[]),
            creature("t:gog", Some("t:wild"), 2, &[]),
            creature("t:wisp", None, 1, &[]),
        ],
        profiles: vec![ProfileDef {
            key: key("t:faithful"),
            quirks: vec![],
        }],
        ..ContentPack::new(manifest("t", &[]))
    };
    let u = ContentPack {
        resources: vec![resource("u:gem")],
        factions: vec![faction("u:marsh")],
        creatures: vec![
            creature("u:frog", Some("u:marsh"), 1, &[]),
            creature("u:toad", Some("u:marsh"), 2, &[]),
        ],
        ..ContentPack::new(manifest("u", &["t"]))
    };
    let v = ContentPack {
        factions: vec![faction("v:dune")],
        creatures: vec![
            creature("v:a", Some("v:dune"), 1, &[]),
            creature("v:b", Some("v:dune"), 2, &[]),
        ],
        ..ContentPack::new(manifest("v", &[]))
    };
    let w = ContentPack::new(manifest("w", &["t"]));
    Ruleset::compile(&[t, u, v, w], "t:faithful").unwrap()
}

fn params() -> TownParamsDef {
    TownParamsDef {
        gold: key("t:gold"),
        hall_gold: [500, 1000, 2000, 4000],
        grail_gold: 5000,
        reveal_radius: 5,
        level_2_dwelling_chance: 30,
        neutral_garrison: [
            GarrisonRow {
                chance: 33,
                count: [8, 15],
            },
            GarrisonRow {
                chance: 33,
                count: [5, 7],
            },
            GarrisonRow {
                chance: 20,
                count: [3, 5],
            },
            GarrisonRow {
                chance: 13,
                count: [1, 3],
            },
        ],
        capitol_demoted_to: "city_hall".into(),
        visions_factor: 3,
        visions_min_radius: 2,
    }
}

fn building(k: &str, roles: BuildingRoles) -> BuildingDef {
    BuildingDef {
        key: k.into(),
        name: "Building".into(),
        slot: None,
        supersedes: vec![],
        forbidden_with: None,
        auto_with: vec![],
        requires: vec![],
        cost: vec![],
        roles,
    }
}

/// `def` requiring the buildings `keys` (direct).
fn needs(def: BuildingDef, keys: &[&str]) -> BuildingDef {
    BuildingDef {
        requires: keys.iter().map(|&k| k.into()).collect(),
        ..def
    }
}

fn superseding(k: &str, supersedes: &str, roles: BuildingRoles) -> BuildingDef {
    BuildingDef {
        supersedes: vec![supersedes.into()],
        ..building(k, roles)
    }
}

fn hall(level: u8) -> BuildingRoles {
    BuildingRoles {
        hall: Some(level),
        ..BuildingRoles::default()
    }
}

fn fort(level: u8) -> BuildingRoles {
    BuildingRoles {
        fort: Some(level),
        ..BuildingRoles::default()
    }
}

fn dwelling(level: u8, upgraded: bool) -> BuildingRoles {
    BuildingRoles {
        dwelling: Some(Dwelling { level, upgraded }),
        ..BuildingRoles::default()
    }
}

/// A valid record of `faction` with these creatures: the four halls and three forts, each
/// superseding and requiring the one below (the Town Hall requiring the Tavern, the Capitol the
/// Castle too), the Tavern, the silo, the Grail, the two dwellings (the second requiring the
/// first), the Lookout Tower, a horde requiring the level-2 dwelling and its upgrade requiring
/// it, forbidden with it and built with it and the level-2 dwelling, and the Shipyard requiring
/// the Tavern; the Town Hall, the City Hall and the Shipyard cost resources (E09 T16a).
fn town(faction: &str, creatures: &[(&str, Option<&str>)], moat: Moat) -> TownDef {
    let buildings = vec![
        BuildingDef {
            slot: Some(0),
            ..building("village_hall", hall(1))
        },
        BuildingDef {
            slot: Some(1),
            cost: vec![(key("t:gold"), 2500)],
            ..needs(
                superseding("town_hall", "village_hall", hall(2)),
                &["tavern"],
            )
        },
        BuildingDef {
            cost: vec![(key("t:gold"), 5000), (key("t:wood"), 5)],
            ..needs(
                superseding("city_hall", "town_hall", hall(3)),
                &["town_hall"],
            )
        },
        needs(
            superseding("capitol", "city_hall", hall(4)),
            &["city_hall", "castle"],
        ),
        building("fort", fort(1)),
        needs(superseding("citadel", "fort", fort(2)), &["fort"]),
        needs(superseding("castle", "citadel", fort(3)), &["citadel"]),
        building(
            "tavern",
            BuildingRoles {
                tavern: true,
                morale: 1,
                ..BuildingRoles::default()
            },
        ),
        building(
            "silo",
            BuildingRoles {
                silo: true,
                ..BuildingRoles::default()
            },
        ),
        building(
            "grail",
            BuildingRoles {
                grail: true,
                luck: -1,
                escape_tunnel: true,
                defender: DefenderBonus {
                    attack: 1,
                    defense: 2,
                    power: 3,
                    mana: 150,
                },
                reveal: Some(Reveal::All),
                ..BuildingRoles::default()
            },
        ),
        building("dwelling_1", dwelling(1, false)),
        needs(building("dwelling_2", dwelling(2, false)), &["dwelling_1"]),
        building(
            "lookout",
            BuildingRoles {
                reveal: Some(Reveal::Radius(20)),
                ..BuildingRoles::default()
            },
        ),
        needs(building("horde", BuildingRoles::default()), &["dwelling_2"]),
        BuildingDef {
            forbidden_with: Some("horde".into()),
            auto_with: vec!["horde".into(), "dwelling_2".into()],
            ..needs(building("horde_up", BuildingRoles::default()), &["horde"])
        },
        BuildingDef {
            cost: vec![(key("t:wood"), 20), (key("t:gold"), 2000)],
            ..needs(
                building(
                    "shipyard",
                    BuildingRoles {
                        shipyard: true,
                        ..BuildingRoles::default()
                    },
                ),
                &["tavern"],
            )
        },
    ];
    TownDef {
        faction: key(faction),
        moat,
        siege_obstacles: false,
        silo: vec![(key("t:wood"), 1)],
        creatures: creatures
            .iter()
            .map(|(base, upgrade)| CreatureLevelDef {
                base: key(base),
                upgrade: upgrade.map(key),
            })
            .collect(),
        buildings,
    }
}

/// The packs: `t` the parameters, the records of `t:keep` (harmless, its upgraded level-1
/// dwelling too) and `t:wild` (absent) and the Visions creature `t:wisp`, `u` the record of
/// `u:marsh` (harmless), overrides of `t:keep` (damaging) and of its own record (natural
/// obstacles) and the Visions creatures `u:toad` and `u:frog`, `v` and `w` empty.
fn packs() -> Vec<TownPack> {
    let mut keep = town(
        "t:keep",
        &[("t:pike", Some("t:halberd")), ("t:archer", None)],
        Moat::Harmless,
    );
    keep.buildings
        .push(building("dwelling_1_up", dwelling(1, true)));
    let t = TownPack {
        params: Some(params()),
        towns: vec![
            keep,
            town("t:wild", &[("t:imp", None), ("t:gog", None)], Moat::Absent),
        ],
        visions: vec![key("t:wisp")],
        ..TownPack::new(manifest("t", &[]))
    };
    let u = TownPack {
        towns: vec![town(
            "u:marsh",
            &[("u:frog", None), ("u:toad", None)],
            Moat::Harmless,
        )],
        overrides: vec![
            TownOverride {
                faction: key("t:keep"),
                patch: TownPatch {
                    moat: Some(Moat::Damaging),
                    siege_obstacles: Some(true),
                    silo: Some(vec![(key("u:gem"), 2), (key("t:wood"), 1)]),
                },
            },
            TownOverride {
                faction: key("u:marsh"),
                patch: TownPatch {
                    siege_obstacles: Some(true),
                    ..TownPatch::default()
                },
            },
        ],
        visions: vec![key("u:toad"), key("u:frog")],
        ..TownPack::new(manifest("u", &["t"]))
    };
    vec![
        t,
        u,
        TownPack::new(manifest("v", &[])),
        TownPack::new(manifest("w", &["t"])),
    ]
}

fn compiled() -> (Ruleset, TownRuleset) {
    let core = core();
    let part = TownRuleset::compile(&core, &packs()).unwrap();
    (core, part)
}

#[test]
fn the_records_compile_in_faction_order_with_the_overrides_applied() {
    let (core, part) = compiled();
    assert_eq!(part.core_hash(), core.content_hash());
    let factions: Vec<_> = part.towns().iter().map(|t| t.faction).collect();
    let by_key = |k: &str| core.faction_by_key(k).unwrap();
    assert_eq!(
        factions,
        [by_key("t:keep"), by_key("t:wild"), by_key("u:marsh")]
    );
    let keep = part.town(by_key("t:keep")).unwrap();
    assert_eq!(keep.moat, Moat::Damaging);
    assert!(keep.siege_obstacles);
    let resource = |k: &str| {
        let i = core.resources().iter().position(|r| r.key == key(k));
        ResourceId::from_index(i.unwrap()).unwrap()
    };
    assert_eq!(keep.silo, [(resource("t:wood"), 1), (resource("u:gem"), 2)]);
    let creature = |k: &str| core.creature_by_key(k).unwrap();
    assert_eq!(
        keep.creatures,
        [
            CreatureLevel {
                base: creature("t:pike"),
                upgrade: Some(creature("t:halberd")),
            },
            CreatureLevel {
                base: creature("t:archer"),
                upgrade: None,
            },
        ]
    );
    let wild = part.town(by_key("t:wild")).unwrap();
    assert_eq!((wild.moat, wild.siege_obstacles), (Moat::Absent, false));
    let marsh = part.town(by_key("u:marsh")).unwrap();
    assert_eq!((marsh.moat, marsh.siege_obstacles), (Moat::Harmless, true));
    assert!(part.town(by_key("v:dune")).is_none());
    let p = part.params().unwrap();
    assert_eq!(p.gold, resource("t:gold"));
    assert_eq!(
        (p.hall_gold, p.grail_gold, p.reveal_radius),
        ([500, 1000, 2000, 4000], 5000, 5)
    );
    assert_eq!(p.level_2_dwelling_chance, 30);
    assert_eq!(p.neutral_garrison[3].count, [1, 3]);
    assert_eq!(p.capitol_demoted_to, "city_hall");
}

#[test]
fn the_buildings_close_what_they_supersede() {
    let (core, part) = compiled();
    let keep = part.town(core.faction_by_key("t:keep").unwrap()).unwrap();
    let id = |k: &str| keep.building_by_key(k).unwrap();
    assert!(keep.building_by_key("moat").is_none());
    let capitol = keep.building(id("capitol")).unwrap();
    assert_eq!(capitol.key, "capitol");
    let below = ["village_hall", "town_hall", "city_hall"];
    assert!(below.iter().all(|k| capitol.supersedes.contains(id(k))));
    assert!(!capitol.supersedes.contains(id("capitol")));
    assert_eq!(capitol.supersedes.bits().count_ones(), 3);
    let village = keep.building(id("village_hall")).unwrap();
    assert_eq!(
        (village.supersedes, village.slot),
        (BuildingSet::EMPTY, Some(0))
    );
    let horde_up = keep.building(id("horde_up")).unwrap();
    assert_eq!(horde_up.forbidden_with, Some(id("horde")));
    assert_eq!(
        horde_up.auto_with,
        BuildingSet::EMPTY.with(id("horde")).with(id("dwelling_2"))
    );
    assert_eq!(
        keep.building(id("horde")).unwrap().auto_with,
        BuildingSet::EMPTY
    );
    assert_eq!(keep.capitol_demoted_to, id("city_hall"));
    let grail = keep.building(id("grail")).unwrap();
    assert_eq!(grail.roles.reveal, Some(Reveal::All));
    assert_eq!(grail.roles.defender.mana, 150);
    let past = BuildingId::from_index(keep.buildings.len()).unwrap();
    assert!(keep.building(past).is_none());
}

#[test]
fn the_buildings_close_what_they_require_and_list_their_costs() {
    // covers: TWN-BLD-006, TWN-BLD-007
    let (core, part) = compiled();
    for record in part.towns() {
        let id = |k: &str| record.building_by_key(k).unwrap();
        let of = |k: &str| record.building(id(k)).unwrap();
        let set = |keys: &[&str]| {
            keys.iter()
                .fold(BuildingSet::EMPTY, |set, &k| set.with(id(k)))
        };
        // The closure: the Capitol needs the halls below it, the Tavern through the Town Hall,
        // and the forts through the Castle; never itself.
        assert_eq!(
            of("capitol").requires,
            set(&[
                "city_hall",
                "town_hall",
                "tavern",
                "castle",
                "citadel",
                "fort"
            ])
        );
        assert_eq!(
            of("horde_up").requires,
            set(&["horde", "dwelling_2", "dwelling_1"])
        );
        assert_eq!(of("shipyard").requires, set(&["tavern"]));
        assert_eq!(of("village_hall").requires, BuildingSet::EMPTY);
        assert!(of("shipyard").roles.shipyard);
        assert!(!of("tavern").roles.shipyard);
        // The costs in the core's resource order (wood before gold), whatever the order given.
        let resource = |k: &str| {
            let index = core.resources().iter().position(|r| r.key.as_str() == k);
            index.and_then(ResourceId::from_index).unwrap()
        };
        let (wood, gold) = (resource("t:wood"), resource("t:gold"));
        assert_eq!(of("town_hall").cost, [(gold, 2500)]);
        assert_eq!(of("city_hall").cost, [(wood, 5), (gold, 5000)]);
        assert_eq!(of("shipyard").cost, [(wood, 20), (gold, 2000)]);
        assert!(of("village_hall").cost.is_empty());
    }
}

#[test]
fn packs_without_towns_compile_to_the_empty_part() {
    let core = core();
    let empty: Vec<_> = ["t", "u", "v", "w"]
        .iter()
        .map(|id| {
            let depends: &[&str] = if matches!(*id, "u" | "w") {
                &["t"]
            } else {
                &[]
            };
            TownPack::new(manifest(id, depends))
        })
        .collect();
    let part = TownRuleset::compile(&core, &empty).unwrap();
    assert_eq!(part, TownRuleset::empty(&core));
    assert!(part.towns().is_empty() && part.params().is_none());
    assert_eq!(part.core_hash(), core.content_hash());
    // Visions creatures need the parameters, as records do.
    let mut visions = empty.clone();
    visions[1].visions.push(key("u:frog"));
    assert_eq!(
        TownRuleset::compile(&core, &visions).unwrap_err(),
        TownCompileError::MissingParams
    );
    // The parameters alone, without a record, are kept, with the Visions creatures.
    let mut with_params = visions;
    with_params[0].params = Some(params());
    let part = TownRuleset::compile(&core, &with_params).unwrap();
    assert!(part.towns().is_empty());
    let frog = core.creature_by_key("u:frog").unwrap();
    assert_eq!(part.params().unwrap().visions_creatures, [frog]);
    assert_ne!(
        part.content_hash(),
        TownRuleset::empty(&core).content_hash()
    );
}

#[test]
fn the_hash_is_pinned_and_follows_the_content() {
    let (core, part) = compiled();
    let (_, again) = compiled();
    assert_eq!(part.content_hash(), again.content_hash());
    assert_eq!(part.content_hash(), hash_of(&part));
    // The fourth encoding (v4): pinned, so a change of what it covers bumps the version.
    assert_eq!(part.content_hash(), 0xcb0d02cf9d6eb150);
    assert_eq!(TownRuleset::empty(&core).content_hash(), 0x985b2cc3d2245173);
    let mut changed = packs();
    changed[0].towns[0].buildings[2].name = "Hall".into();
    let other = TownRuleset::compile(&core, &changed).unwrap();
    assert_ne!(other.content_hash(), part.content_hash());
    let mut automatic = packs();
    keep_building(&mut automatic, "horde_up").auto_with.pop();
    let other = TownRuleset::compile(&core, &automatic).unwrap();
    assert_ne!(other.content_hash(), part.content_hash());
    // The third (v3): Visions' factor, least radius and creatures.
    let changes: [fn(&mut Vec<TownPack>); 3] = [
        |p| p[0].params.as_mut().unwrap().visions_factor = 2,
        |p| p[0].params.as_mut().unwrap().visions_min_radius = 3,
        |p| drop(p[1].visions.pop()),
    ];
    for change in changes {
        let mut changed = packs();
        change(&mut changed);
        let other = TownRuleset::compile(&core, &changed).unwrap();
        assert_ne!(other.content_hash(), part.content_hash());
    }
    // The fourth (v4): a building's requirements, its cost and the role `shipyard`.
    let changes: [fn(&mut Vec<TownPack>); 4] = [
        |p| drop(keep_building(p, "capitol").requires.pop()),
        |p| keep_building(p, "town_hall").cost[0].1 = 2000,
        |p| drop(keep_building(p, "city_hall").cost.pop()),
        |p| keep_building(p, "shipyard").roles.shipyard = false,
    ];
    for change in changes {
        let mut changed = packs();
        change(&mut changed);
        let other = TownRuleset::compile(&core, &changed).unwrap();
        assert_ne!(other.content_hash(), part.content_hash());
    }
}

// covers: TWN-VEW-003
#[test]
fn the_visions_parameters_take_every_packs_creatures_in_handle_order() {
    let (core, part) = compiled();
    let params = part.params().unwrap();
    assert_eq!((params.visions_factor, params.visions_min_radius), (3, 2));
    let creature = |k: &str| core.creature_by_key(k).unwrap();
    assert_eq!(
        params.visions_creatures,
        [creature("t:wisp"), creature("u:frog"), creature("u:toad")]
    );
}

/// Compiles the packs after `change`.
fn refused(change: impl FnOnce(&mut Vec<TownPack>)) -> TownCompileError {
    let mut packs = packs();
    change(&mut packs);
    TownRuleset::compile(&core(), &packs).unwrap_err()
}

fn town_error(faction: &str, reason: &'static str) -> TownCompileError {
    TownCompileError::InvalidTown {
        faction: faction.into(),
        reason,
    }
}

fn building_error(building: &str, reason: &'static str) -> TownCompileError {
    TownCompileError::InvalidBuilding {
        faction: "t:keep".into(),
        building: building.into(),
        reason,
    }
}

/// The record of `t:keep` in pack `t`.
fn keep(packs: &mut [TownPack]) -> &mut TownDef {
    &mut packs[0].towns[0]
}

/// The building `k` of `t:keep`.
fn keep_building<'a>(packs: &'a mut [TownPack], k: &str) -> &'a mut BuildingDef {
    keep(packs)
        .buildings
        .iter_mut()
        .find(|b| b.key == k)
        .unwrap()
}

#[test]
fn the_pack_set_is_checked() {
    use TownCompileError as E;
    assert_eq!(refused(|p| drop(p.remove(1))), E::MissingPack("u".into()));
    assert_eq!(
        refused(|p| p[2].manifest.version = "2".into()),
        E::UnknownPack("v@2".into())
    );
    assert_eq!(
        refused(|p| p.push(TownPack::new(manifest("v", &[])))),
        E::DuplicatePack("v".into())
    );
    assert_eq!(
        refused(|p| p[1].manifest.depends = vec!["nope".into()]),
        E::UnknownDependency {
            pack: "u".into(),
            dependency: "nope".into(),
        }
    );
}

#[test]
fn references_are_checked() {
    use TownCompileError as E;
    let outside = |key: &str, pack: &str| E::ReferenceOutsideDependencies {
        key: key.into(),
        pack: pack.into(),
    };
    // A record of `v` for a faction of `t`, which `v` does not depend on.
    assert_eq!(
        refused(|p| p[2].towns.push(town("t:wild", &[], Moat::Absent))),
        outside("t:wild", "v")
    );
    // `w` depends on `t` but not on `u`, which defined the record it overrides.
    assert_eq!(
        refused(|p| {
            let o = p[1].overrides.remove(0);
            p[0].towns.remove(0);
            p[1].towns.push(town(
                "t:keep",
                &[("t:pike", None), ("t:archer", None)],
                Moat::Absent,
            ));
            p[3].overrides.push(o);
        }),
        outside("t:keep", "w")
    );
    assert_eq!(
        refused(|p| keep(p).faction = key("t:nope")),
        E::UnknownFaction("t:nope".into())
    );
    assert_eq!(
        refused(|p| p[1].overrides[0].faction = key("v:dune")),
        outside("v:dune", "u")
    );
    assert_eq!(
        refused(|p| keep(p).silo[0].0 = key("t:nope")),
        E::UnknownResource("t:nope".into())
    );
    assert_eq!(
        refused(|p| p[0].params.as_mut().unwrap().gold = key("u:gem")),
        outside("u:gem", "t")
    );
    assert_eq!(
        refused(|p| keep(p).creatures[1].base = key("t:nope")),
        E::UnknownCreature("t:nope".into())
    );
    assert_eq!(
        refused(|p| keep(p).creatures[0].upgrade = Some(key("t:nope"))),
        E::UnknownCreature("t:nope".into())
    );
    assert_eq!(
        refused(|p| p[0].visions.push(key("t:nope"))),
        E::UnknownCreature("t:nope".into())
    );
    assert_eq!(
        refused(|p| p[2].visions.push(key("t:wisp"))),
        outside("t:wisp", "v")
    );
    assert_eq!(
        refused(|p| p[0].towns.push(town("t:keep", &[], Moat::Absent))),
        town_error("t:keep", "needs one to seven creature levels")
    );
    assert_eq!(
        refused(|p| {
            let marsh = p[1].towns.remove(0);
            p[1].towns.push(TownDef {
                faction: key("t:wild"),
                creatures: town("t:wild", &[("t:imp", None), ("t:gog", None)], Moat::Absent)
                    .creatures,
                ..marsh
            });
        }),
        E::DuplicateTown("t:wild".into())
    );
    assert_eq!(
        refused(|p| {
            p[0].towns.pop();
            p[1].overrides[0].faction = key("t:wild");
        }),
        E::UnknownTown("t:wild".into())
    );
}

#[test]
fn the_parameters_are_checked() {
    use TownCompileError as E;
    assert_eq!(refused(|p| p[0].params = None), E::MissingParams);
    assert_eq!(
        refused(|p| p[3].params = Some(params())),
        E::DuplicateParams("w".into())
    );
    let param = |change: fn(&mut TownParamsDef)| refused(|p| change(p[0].params.as_mut().unwrap()));
    assert_eq!(
        param(|d| d.level_2_dwelling_chance = 101),
        E::InvalidParams("the level-2 dwelling's chance is above 100")
    );
    assert_eq!(
        param(|d| d.neutral_garrison[2].chance = 101),
        E::InvalidParams("a garrison row's chance is above 100")
    );
    let count = E::InvalidParams("a garrison row's count needs 1 <= least <= most");
    assert_eq!(param(|d| d.neutral_garrison[0].count = [0, 1]), count);
    assert_eq!(param(|d| d.neutral_garrison[1].count = [5, 3]), count);
    assert_eq!(
        refused(|p| p[3].visions.push(key("t:wisp"))),
        E::InvalidParams("a Visions creature is named twice")
    );
    assert_eq!(
        param(|d| d.capitol_demoted_to = "town_hall".into()),
        town_error(
            "t:keep",
            "the demoted Capitol's key is not its hall-3 building"
        )
    );
}

#[test]
fn a_record_is_checked() {
    let reason = |change: fn(&mut TownDef)| refused(|p| change(keep(p)));
    let levels = town_error("t:keep", "needs one to seven creature levels");
    assert_eq!(reason(|t| t.creatures.clear()), levels);
    assert_eq!(
        reason(|t| t.creatures = vec![t.creatures[1].clone(); 8]),
        levels
    );
    let not_the_level = town_error(
        "t:keep",
        "a level's creature is not the faction's of that level",
    );
    assert_eq!(
        reason(|t| t.creatures[0].base = key("t:imp")),
        not_the_level
    );
    assert_eq!(
        reason(|t| t.creatures[0].base = key("t:archer")),
        not_the_level
    );
    assert_eq!(
        reason(|t| t.creatures[1].upgrade = Some(key("t:pike"))),
        town_error("t:keep", "a level's upgrade is not its base's")
    );
    let silo = town_error("t:keep", "the silo gives each resource once, at least 1");
    assert_eq!(reason(|t| t.silo[0].1 = 0), silo);
    assert_eq!(reason(|t| t.silo.push((key("t:wood"), 2))), silo);
    // The override's silo is checked too.
    assert_eq!(
        refused(|p| {
            let patch = &mut p[1].overrides[0].patch;
            patch.silo.as_mut().unwrap()[0].1 = 0;
        }),
        silo
    );
    assert_eq!(
        reason(|t| {
            let extra =
                (0..50).map(|i| building(&std::format!("extra_{i}"), BuildingRoles::default()));
            t.buildings.extend(extra);
        }),
        town_error("t:keep", "has more than 64 buildings")
    );
}

#[test]
fn the_roles_across_a_record_are_checked() {
    let reason = |change: fn(&mut TownDef)| refused(|p| change(keep(p)));
    let halls = town_error("t:keep", "needs one building of each hall level 1 to 4");
    assert_eq!(reason(|t| t.buildings[3].roles.hall = Some(3)), halls);
    assert_eq!(
        reason(|t| t.buildings.push(building("hall_2", hall(1)))),
        halls
    );
    assert_eq!(
        reason(|t| t.buildings[6].roles.fort = None),
        town_error("t:keep", "needs one building of each fort level 1 to 3")
    );
    let grail = town_error("t:keep", "needs one Grail building and one Resource Silo");
    assert_eq!(reason(|t| t.buildings[9].roles.grail = false), grail);
    assert_eq!(reason(|t| t.buildings[13].roles.silo = true), grail);
    let changes = town_error(
        "t:keep",
        "has more than 8 buildings with a morale or luck change",
    );
    let lucky = |t: &mut TownDef, n: usize| {
        for i in 0..n {
            let roles = BuildingRoles {
                morale: i8::from(i % 2 == 0),
                luck: i8::from(i % 2 == 1),
                ..BuildingRoles::default()
            };
            t.buildings
                .push(building(&std::format!("shrine_{i}"), roles));
        }
    };
    let morale_buildings = keep(&mut packs())
        .buildings
        .iter()
        .filter(|b| b.roles.morale != 0 || b.roles.luck != 0)
        .count();
    let mut packs_at_most = packs();
    lucky(
        keep(&mut packs_at_most),
        h3_contracts::MAX_TOWN_BONUSES - morale_buildings,
    );
    let core = core();
    assert!(TownRuleset::compile(&core, &packs_at_most).is_ok());
    let mut packs_more = packs();
    lucky(
        keep(&mut packs_more),
        h3_contracts::MAX_TOWN_BONUSES + 1 - morale_buildings,
    );
    assert_eq!(TownRuleset::compile(&core, &packs_more), Err(changes));
    assert_eq!(
        reason(|t| t
            .buildings
            .push(building("dwelling_1b", dwelling(1, false)))),
        town_error("t:keep", "has a dwelling twice")
    );
    assert_eq!(
        reason(|t| {
            let mut second = t
                .buildings
                .iter()
                .find(|b| b.roles.shipyard)
                .unwrap()
                .clone();
            second.key = "shipyard_2".into();
            t.buildings.push(second);
        }),
        town_error("t:keep", "has more than one Shipyard")
    );
    let dwellings = town_error("t:keep", "needs the level-1 and level-2 dwellings");
    assert_eq!(reason(|t| t.buildings[11].roles.dwelling = None), dwellings);
    assert_eq!(reason(|t| t.buildings[10].roles.dwelling = None), dwellings);
}

#[test]
fn a_building_is_checked() {
    let reason =
        |k: &'static str, change: fn(&mut BuildingDef)| refused(|p| change(keep_building(p, k)));
    let key_reason = "a key is `[a-z0-9_]+`";
    assert_eq!(
        reason("horde", |b| b.key = "Horde".into()),
        building_error("Horde", key_reason)
    );
    assert_eq!(
        reason("horde", |b| b.key = String::new()),
        building_error("", key_reason)
    );
    assert_eq!(
        reason("horde_up", |b| b.key = "lookout".into()),
        building_error("lookout", "a key is the faction's only one")
    );
    let slot = "a slot is below 41 and the faction's only one";
    assert_eq!(
        reason("fort", |b| b.slot = Some(41)),
        building_error("fort", slot)
    );
    assert_eq!(
        reason("fort", |b| b.slot = Some(1)),
        building_error("fort", slot)
    );
    let halls = "a hall level is 1 to 4";
    assert_eq!(
        reason("lookout", |b| b.roles.hall = Some(5)),
        building_error("lookout", halls)
    );
    assert_eq!(
        reason("lookout", |b| b.roles.hall = Some(0)),
        building_error("lookout", halls)
    );
    assert_eq!(
        reason("lookout", |b| b.roles.fort = Some(4)),
        building_error("lookout", "a fort level is 1 to 3")
    );
    let dwelling = "a dwelling's level, or its upgrade, is not the faction's";
    for level in [0, 3] {
        let error = refused(|p| {
            keep_building(p, "horde").roles.dwelling = Some(Dwelling {
                level,
                upgraded: false,
            });
        });
        assert_eq!(error, building_error("horde", dwelling));
    }
    assert_eq!(
        reason("horde", |b| b.roles.dwelling = Some(Dwelling {
            level: 2,
            upgraded: true
        })),
        building_error("horde", dwelling)
    );
    let mood = "morale and luck are -3 to 3";
    assert_eq!(
        reason("tavern", |b| b.roles.morale = 4),
        building_error("tavern", mood)
    );
    assert_eq!(
        reason("tavern", |b| b.roles.luck = -4),
        building_error("tavern", mood)
    );
}

#[test]
fn supersedes_forbidden_with_and_auto_with_are_checked() {
    let reason =
        |k: &'static str, change: fn(&mut BuildingDef)| refused(|p| change(keep_building(p, k)));
    assert_eq!(
        reason("fort", |b| b.supersedes.push("moat".into())),
        building_error("fort", "it supersedes a building the faction lacks")
    );
    assert_eq!(
        reason("horde", |b| b.forbidden_with = Some("moat".into())),
        building_error("horde", "it is forbidden with a building the faction lacks")
    );
    assert_eq!(
        reason("village_hall", |b| b.supersedes.push("capitol".into())),
        building_error("village_hall", "it supersedes itself (a cycle)")
    );
    assert_eq!(
        reason("horde_up", |b| b.auto_with.push("moat".into())),
        building_error("horde_up", "it is built with a building the faction lacks")
    );
    assert_eq!(
        reason("horde_up", |b| b.auto_with.push("horde_up".into())),
        building_error("horde_up", "it is built with itself")
    );
    let cycle = "it is forbidden with itself (a cycle)";
    assert_eq!(
        reason("horde", |b| b.forbidden_with = Some("horde".into())),
        building_error("horde", cycle)
    );
    // The silo leads into the cycle of the horde and its upgrade: the horde is named.
    assert_eq!(
        refused(|p| {
            keep_building(p, "silo").forbidden_with = Some("horde".into());
            keep_building(p, "horde").forbidden_with = Some("horde_up".into());
        }),
        building_error("horde", cycle)
    );
}

#[test]
fn requires_and_cost_are_checked() {
    use TownCompileError as E;
    let reason =
        |k: &'static str, change: fn(&mut BuildingDef)| refused(|p| change(keep_building(p, k)));
    assert_eq!(
        reason("fort", |b| b.requires.push("moat".into())),
        building_error("fort", "it requires a building the faction lacks")
    );
    assert_eq!(
        reason("fort", |b| b.requires.push("fort".into())),
        building_error("fort", "it requires itself")
    );
    // The fort through the Castle and the Citadel; the first of the cycle is named.
    assert_eq!(
        reason("fort", |b| b.requires.push("castle".into())),
        building_error("fort", "it requires itself through others (a cycle)")
    );
    assert_eq!(
        reason("tavern", |b| b.cost.push((key("t:nope"), 1))),
        E::UnknownResource("t:nope".into())
    );
    assert_eq!(
        reason("tavern", |b| b.cost.push((key("u:gem"), 1))),
        E::ReferenceOutsideDependencies {
            key: "u:gem".into(),
            pack: "t".into(),
        }
    );
    let cost = "a cost names each resource once, at least 1";
    assert_eq!(
        reason("town_hall", |b| b.cost[0].1 = 0),
        building_error("town_hall", cost)
    );
    assert_eq!(
        reason("city_hall", |b| b.cost.push((key("t:wood"), 1))),
        building_error("city_hall", cost)
    );
}

#[test]
fn every_error_has_a_message() {
    use TownCompileError as E;
    let cases = [
        (E::MissingPack("p".into()), "pack `p` has no town part"),
        (
            E::UnknownPack("p@1".into()),
            "town part of `p@1`, which the core lacks",
        ),
        (E::DuplicatePack("p".into()), "pack `p` has two town parts"),
        (
            E::UnknownDependency {
                pack: "p".into(),
                dependency: "q".into(),
            },
            "pack `p` depends on `q`, which is not loaded",
        ),
        (
            E::ReferenceOutsideDependencies {
                key: "q:x".into(),
                pack: "p".into(),
            },
            "pack `p` references `q:x` outside its dependencies",
        ),
        (E::UnknownFaction("p:x".into()), "unknown faction `p:x`"),
        (E::UnknownResource("p:x".into()), "unknown resource `p:x`"),
        (E::UnknownCreature("p:x".into()), "unknown creature `p:x`"),
        (
            E::DuplicateTown("p:x".into()),
            "faction `p:x` has two town records",
        ),
        (
            E::UnknownTown("p:x".into()),
            "faction `p:x` has no town record to override",
        ),
        (E::MissingParams, "no pack provides the town parameters"),
        (
            E::DuplicateParams("p".into()),
            "pack `p` provides the town parameters a second time",
        ),
        (E::InvalidParams("why"), "town parameters: why"),
        (town_error("p:x", "why"), "town record of `p:x`: why"),
        (building_error("b", "why"), "building `b` of `t:keep`: why"),
    ];
    for (error, message) in cases {
        assert_eq!(error.to_string(), message);
    }
}
