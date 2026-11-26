//! Tests of the town part's compiler (E09 T5a1), on core rulesets built in code (a domain crate
//! takes no pack adapter as a dev-dependency): every check refused by a test, the hash pinned.

extern crate std;

use std::string::{String, ToString};
use std::vec;
use std::vec::Vec;

use h3_content::{
    ArtifactDef, ArtifactSlotDef, ContentKey, ContentPack, CreatureDef, CreatureInfoDef,
    FactionDef, HeroClassDef, PackManifest, ProfileDef, ResourceDef, Ruleset, SecondarySkillDef,
    SpellCategory, SpellDef, SpellSchoolDef,
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

fn class(k: &str, faction: &str) -> HeroClassDef {
    HeroClassDef {
        key: key(k),
        name: "Class".into(),
        faction: key(faction),
        primary: [1, 1, 1, 1],
        primary_gain: [[25, 25, 25, 25]; 2],
        skill_weights: vec![],
        offer_intervals: [4, 4],
    }
}

fn spell(k: &str, category: SpellCategory, level: u8, granted_by_name_only: bool) -> SpellDef {
    SpellDef {
        key: key(k),
        name: "Spell".into(),
        category,
        level,
        schools: if category == SpellCategory::Ability {
            vec![]
        } else {
            vec![key("t:air")]
        },
        cost: [0; 4],
        power: 0,
        effect: [0; 4],
        granted_by_name_only,
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
/// `t:gog`; the neutral `t:wisp`; the hero classes `t:knight` of `t:keep` and `t:shaman` of
/// `t:wild`; the spells `t:bolt` (combat, level 1), `t:fly` (adventure, level 2), the creature
/// ability `t:howl` and `t:titan` (granted by name only, level 5), each hero spell of the school
/// `t:air`; the war machine `t:tent` (a creature and its artifact), the artifact `t:ring` (no war
/// machine), the skills `t:fire` and `t:earth` (E09 T56); the profile), `u` (depends on `t`: the resource `u:gem`, the faction `u:marsh` with `u:frog` and `u:toad`, its hero class
/// `u:seer`, the spell `u:mist`), `v` (no dependencies: the faction `v:dune` with `v:a` and `v:b`)
/// and `w` (depends on `t`, nothing defined).
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
            CreatureDef {
                key: key("t:tent"),
                speed: 0,
                info: CreatureInfoDef {
                    flags: vec!["war_machine".into()],
                    ..CreatureInfoDef::default()
                },
            },
        ],
        skills: ["t:fire", "t:earth"]
            .map(|k| SecondarySkillDef {
                key: key(k),
                name: "Skill".into(),
                levels: [vec![], vec![], vec![]],
            })
            .into(),
        artifact_slots: vec![ArtifactSlotDef {
            key: key("t:machine"),
            name: "Machine".into(),
            count: 1,
            backpack: false,
        }],
        artifacts: [("t:tent", Some(key("t:tent"))), ("t:ring", None)]
            .map(|(k, war_machine)| ArtifactDef {
                key: key(k),
                name: "Artifact".into(),
                class: ArtifactClass::Treasure,
                cost: vec![],
                slot: Some(key("t:machine")),
                war_machine,
                needs_skill: None,
                effects: vec![],
                parts: vec![],
            })
            .into(),
        hero_classes: vec![class("t:knight", "t:keep"), class("t:shaman", "t:wild")],
        spell_schools: vec![SpellSchoolDef {
            key: key("t:air"),
            name: "Air".into(),
            protection_rank: 0,
        }],
        spells: vec![
            spell("t:bolt", SpellCategory::Combat, 1, false),
            spell("t:fly", SpellCategory::Adventure, 2, false),
            spell("t:howl", SpellCategory::Ability, 1, false),
            spell("t:titan", SpellCategory::Combat, 5, true),
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
        hero_classes: vec![class("u:seer", "u:marsh")],
        spells: vec![spell("u:mist", SpellCategory::Combat, 1, false)],
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
        most_heroes: 8,
        hire_price: vec![(key("t:gold"), 2500)],
        guild_resources: [vec![key("t:gold")], vec![key("t:wood")], vec![]],
        guild_slots: [5, 4, 3, 2, 1],
        spell_book_price: vec![(key("t:gold"), 500)],
        vortex_factor: 2,
        merchants_stock: vec![
            ArtifactClass::Treasure,
            ArtifactClass::Minor,
            ArtifactClass::Major,
        ],
        boat_price: vec![(key("t:gold"), 1000), (key("t:wood"), 10)],
        boat_type: 1,
        university_skills: vec![key("t:fire"), key("t:earth")],
        university_price: vec![(key("t:gold"), 2000)],
        transformer_into: key("t:wisp"),
        transformer_dragons_into: key("t:archer"),
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
        war_machine: None,
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

fn horde(level: u8, upgraded: bool) -> BuildingRoles {
    BuildingRoles {
        horde: Some(Horde { level, upgraded }),
        ..BuildingRoles::default()
    }
}

/// A valid record of `faction` with these creatures: the four halls and three forts, each
/// superseding and requiring the one below (the Town Hall requiring the Tavern, the Capitol the
/// Castle too), the Tavern, the silo, the Grail, the two dwellings (the second requiring the
/// first), the Lookout Tower, a horde requiring the level-2 dwelling and its upgrade requiring
/// it, forbidden with it and built with it and the level-2 dwelling, and the Shipyard requiring
/// the Tavern; the Town Hall, the City Hall and the Shipyard cost resources (E09 T16a); the horde
/// of level 2, the Lookout the Portal of Summoning, the Grail every week level 1's (E09 T25a); the
/// guild weights `t:bolt` 60 and `t:fly` 40 (E09 T46).
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
                week_of: Some(1),
                ..BuildingRoles::default()
            },
        ),
        building("dwelling_1", dwelling(1, false)),
        needs(building("dwelling_2", dwelling(2, false)), &["dwelling_1"]),
        building(
            "lookout",
            BuildingRoles {
                reveal: Some(Reveal::Radius(20)),
                portal: true,
                ..BuildingRoles::default()
            },
        ),
        needs(building("horde", horde(2, false)), &["dwelling_2"]),
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
        roe: true,
        tavern: vec![(key("t:knight"), 6), (key("t:shaman"), 5)],
        guild: vec![(key("t:bolt"), 60), (key("t:fly"), 40)],
    }
}

/// The packs: `t` the parameters, the records of `t:keep` (harmless, its upgraded level-1
/// dwelling and horde too) and `t:wild` (absent) and the Visions creature `t:wisp`, `u` the
/// record of `u:marsh` (harmless, not a Restoration of Erathia record, its class `u:seer` in its
/// tavern, its spell `u:mist` in its guild), overrides of `t:keep` (damaging, `u:seer` joining its tavern) and of its own record
/// (natural obstacles) and the Visions creatures `u:toad` and `u:frog`, `v` and `w` empty. The
/// trades (E09 T56): the keep's `market` (the Marketplace, the Artifact Merchants, the Freelancer's
/// Guild) and `school` (the University, the Transformer), each selling the war machine `t:tent`;
/// the Transformer's dragons `t:gog` (pack `t`), `u:toad` and `u:frog` (pack `u`).
fn packs() -> Vec<TownPack> {
    let mut keep = town(
        "t:keep",
        &[("t:pike", Some("t:halberd")), ("t:archer", None)],
        Moat::Harmless,
    );
    keep.buildings
        .push(building("dwelling_1_up", dwelling(1, true)));
    keep.buildings.push(building("horde_1_up", horde(1, true)));
    // The trades (E09 T56): one building of each role, two selling the war machine.
    keep.buildings.push(BuildingDef {
        war_machine: Some(key("t:tent")),
        ..building(
            "market",
            BuildingRoles {
                marketplace: true,
                artifact_merchants: true,
                freelancers: true,
                ..BuildingRoles::default()
            },
        )
    });
    keep.buildings.push(BuildingDef {
        war_machine: Some(key("t:tent")),
        ..building(
            "school",
            BuildingRoles {
                university: true,
                transformer: true,
                ..BuildingRoles::default()
            },
        )
    });
    let mut marsh = town(
        "u:marsh",
        &[("u:frog", None), ("u:toad", None)],
        Moat::Harmless,
    );
    marsh.roe = false;
    marsh.tavern.push((key("u:seer"), 5));
    marsh.guild.push((key("u:mist"), 25));
    let t = TownPack {
        params: Some(params()),
        towns: vec![
            keep,
            town("t:wild", &[("t:imp", None), ("t:gog", None)], Moat::Absent),
        ],
        visions: vec![key("t:wisp")],
        transformer_dragons: vec![key("t:gog")],
        ..TownPack::new(manifest("t", &[]))
    };
    let u = TownPack {
        towns: vec![marsh],
        overrides: vec![
            TownOverride {
                faction: key("t:keep"),
                patch: TownPatch {
                    moat: Some(Moat::Damaging),
                    siege_obstacles: Some(true),
                    silo: Some(vec![(key("u:gem"), 2), (key("t:wood"), 1)]),
                    tavern: Some(vec![(key("u:seer"), 4), (key("t:shaman"), 6)]),
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
        transformer_dragons: vec![key("u:toad"), key("u:frog")],
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
    // Increment 3's (E09 T25a): the growth's parameters, each record's `roe`.
    assert_eq!(p.week, params().week);
    assert_eq!(p.garrison_week, params().garrison_week);
    assert!(keep.roe && wild.roe && !marsh.roe);
    // Increment 4's (E09 T36): the hire's price, the tavern weights by the core's class index
    // (`t:knight`, `t:shaman`, `u:seer`), the override's classes changed or joined.
    assert_eq!(p.hire_price, [(resource("t:gold"), 2500)]);
    assert_eq!(keep.tavern, [6, 6, 4]);
    assert_eq!(wild.tavern, [6, 5, 0]);
    assert_eq!(marsh.tavern, [6, 5, 5]);
}

#[test]
fn the_growth_roles_compile_as_given() {
    let (core, part) = compiled();
    let keep = part.town(core.faction_by_key("t:keep").unwrap()).unwrap();
    let roles = |k: &str| {
        keep.building(keep.building_by_key(k).unwrap())
            .unwrap()
            .roles
    };
    assert_eq!(
        roles("horde").horde,
        Some(Horde {
            level: 2,
            upgraded: false
        })
    );
    assert_eq!(
        roles("horde_1_up").horde,
        Some(Horde {
            level: 1,
            upgraded: true
        })
    );
    assert_eq!(roles("horde_up").horde, None);
    assert!(roles("lookout").portal && !roles("grail").portal);
    assert_eq!(
        (roles("grail").week_of, roles("lookout").week_of),
        (Some(1), None)
    );
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
    // The tenth encoding (v10): pinned, so a change of what it covers bumps the version.
    assert_eq!(part.content_hash(), 0x08b1_fe5c_b1f7_09a6);
    assert_eq!(
        TownRuleset::empty(&core).content_hash(),
        0xa82b_d7e1_f29d_c67d
    );
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
    // The fifth (v5): the roles `horde`, `portal`, `week_of`, a record's `roe`, the growth's
    // parameters.
    let changes: [fn(&mut Vec<TownPack>); 13] = [
        |p| keep_building(p, "horde").roles.horde = None,
        |p| {
            keep_building(p, "horde_1_up").roles.horde = Some(Horde {
                level: 1,
                upgraded: false,
            });
        },
        |p| keep_building(p, "lookout").roles.portal = false,
        |p| keep_building(p, "grail").roles.week_of = Some(2),
        |p| keep(p).roe = false,
        |p| p[0].params.as_mut().unwrap().week.creature_chance = [1, 5],
        |p| p[0].params.as_mut().unwrap().week.creature_bonus = 4,
        |p| p[0].params.as_mut().unwrap().garrison_week.odds = [40, 81],
        |p| p[0].params.as_mut().unwrap().garrison_week.out_of = 100,
        |p| {
            p[0].params.as_mut().unwrap().garrison_week.weights.pop();
        },
        |p| p[0].params.as_mut().unwrap().garrison_week.weights[0] = 1,
        |p| p[0].params.as_mut().unwrap().garrison_week.weeks = 7,
        |p| p[0].params.as_mut().unwrap().garrison_week.upgrade = 6,
    ];
    for change in changes {
        let mut changed = packs();
        change(&mut changed);
        let other = TownRuleset::compile(&core, &changed).unwrap();
        assert_ne!(other.content_hash(), part.content_hash());
    }
    // The sixth (v6): the most heroes on the map.
    let mut changed = packs();
    changed[0].params.as_mut().unwrap().most_heroes = 7;
    let other = TownRuleset::compile(&core, &changed).unwrap();
    assert_ne!(other.content_hash(), part.content_hash());
    // The seventh (v7): the hire's price and the tavern weights.
    let changes: [fn(&mut Vec<TownPack>); 4] = [
        |p| p[0].params.as_mut().unwrap().hire_price[0].1 = 2000,
        |p| {
            p[0].params
                .as_mut()
                .unwrap()
                .hire_price
                .push((key("t:wood"), 1))
        },
        |p| keep(p).tavern[0].1 = 5,
        |p| drop(p[0].towns[1].tavern.pop()),
    ];
    for change in changes {
        let mut changed = packs();
        change(&mut changed);
        let other = TownRuleset::compile(&core, &changed).unwrap();
        assert_ne!(other.content_hash(), part.content_hash());
    }
    // The eighth (v8, E09 T39b1): the Thieves' Guild's resource rows, a resource moved between two.
    let mut changed = packs();
    let rows = &mut changed[0].params.as_mut().unwrap().guild_resources;
    rows[2] = core::mem::take(&mut rows[1]);
    let other = TownRuleset::compile(&core, &changed).unwrap();
    assert_ne!(other.content_hash(), part.content_hash());
    // The ninth (v9, E09 T46): the guild's slots, the book's price, the Vortex's factor, the guild
    // weights and the roles `guild`, `library`, `aurora`, `mana_vortex`.
    let changes: [fn(&mut Vec<TownPack>); 10] = [
        |p| p[0].params.as_mut().unwrap().guild_slots[4] = 2,
        |p| p[0].params.as_mut().unwrap().spell_book_price[0].1 = 400,
        |p| p[0].params.as_mut().unwrap().vortex_factor = 3,
        |p| keep(p).guild[0].1 = 59,
        |p| drop(p[0].towns[1].guild.pop()),
        |p| keep(p).guild.push((key("t:titan"), 1)),
        |p| keep_building(p, "lookout").roles.guild = Some(1),
        |p| keep_building(p, "lookout").roles.library = true,
        |p| keep_building(p, "grail").roles.aurora = true,
        |p| keep_building(p, "silo").roles.mana_vortex = true,
    ];
    for change in changes {
        let mut changed = packs();
        change(&mut changed);
        let other = TownRuleset::compile(&core, &changed).unwrap();
        assert_ne!(other.content_hash(), part.content_hash());
    }
    // The tenth (v10, E09 T56): the trades' parameters, roles and war machines.
    let changes: [fn(&mut Vec<TownPack>); 17] = [
        |p| {
            p[0].params.as_mut().unwrap().merchants_stock.pop();
        },
        |p| p[0].params.as_mut().unwrap().merchants_stock[0] = ArtifactClass::Relic,
        |p| p[0].params.as_mut().unwrap().boat_price[1].1 = 11,
        |p| p[0].params.as_mut().unwrap().boat_type = 2,
        |p| p[0].params.as_mut().unwrap().university_skills.reverse(),
        |p| p[0].params.as_mut().unwrap().university_price[0].1 = 1,
        |p| p[0].params.as_mut().unwrap().transformer_into = key("t:imp"),
        |p| p[0].params.as_mut().unwrap().transformer_dragons_into = key("t:imp"),
        |p| drop(p[1].transformer_dragons.pop()),
        |p| keep_building(p, "market").roles.marketplace = false,
        |p| keep_building(p, "market").roles.artifact_merchants = false,
        |p| keep_building(p, "market").roles.freelancers = false,
        |p| keep_building(p, "school").roles.university = false,
        |p| keep_building(p, "school").roles.transformer = false,
        |p| keep_building(p, "school").war_machine = None,
        |p| keep_building(p, "silo").war_machine = Some(key("t:tent")),
        |p| {
            keep_building(p, "market").roles.marketplace = false;
            keep_building(p, "tavern").roles.marketplace = true;
        },
    ];
    for change in changes {
        let mut changed = packs();
        change(&mut changed);
        let other = TownRuleset::compile(&core, &changed).unwrap();
        assert_ne!(other.content_hash(), part.content_hash());
    }
}

// covers: TWN-TRD-001
#[test]
fn the_trades_compile_as_given() {
    let (core, part) = compiled();
    let params = part.params().unwrap();
    let creature = |k: &str| core.creature_by_key(k).unwrap();
    let resource = |i| ResourceId::from_index(i).unwrap();
    assert_eq!(
        params.merchants_stock,
        [
            ArtifactClass::Treasure,
            ArtifactClass::Minor,
            ArtifactClass::Major
        ]
    );
    // Prices in the core's resource order (wood, gold), the skills in the pack's.
    assert_eq!(params.boat_price, [(resource(0), 10), (resource(1), 1000)]);
    assert_eq!(params.boat_type, 1);
    let skills = ["t:fire", "t:earth"].map(|k| core.skill_by_key(k).unwrap());
    assert_eq!(params.university_skills, skills);
    assert_eq!(params.university_price, [(resource(1), 2000)]);
    assert_eq!(params.transformer_into, creature("t:wisp"));
    assert_eq!(params.transformer_dragons_into, creature("t:archer"));
    assert_eq!(
        params.transformer_dragons,
        [creature("t:gog"), creature("u:frog"), creature("u:toad")]
    );
    let record = part.town(core.faction_by_key("t:keep").unwrap()).unwrap();
    let id = |k: &str| record.building_by_key(k).unwrap();
    let market = record.building(id("market")).unwrap();
    let school = record.building(id("school")).unwrap();
    let r = (market.roles, school.roles);
    assert!(r.0.marketplace && r.0.artifact_merchants && r.0.freelancers);
    assert!(!r.0.university && !r.0.transformer && r.1.university && r.1.transformer);
    let tent = core.artifact_by_key("t:tent");
    assert_eq!((market.war_machine, school.war_machine), (tent, tent));
    assert_eq!(record.building(id("silo")).unwrap().war_machine, None);
}

#[test]
fn the_trades_are_checked() {
    use TownCompileError as E;
    let param = |change: fn(&mut TownParamsDef)| refused(|p| change(p[0].params.as_mut().unwrap()));
    assert_eq!(
        param(|d| d.merchants_stock = vec![ArtifactClass::Minor; 9]),
        E::InvalidParams("the merchants' stock holds more than 8 entries")
    );
    assert_eq!(
        param(|d| d.merchants_stock.push(ArtifactClass::Special)),
        E::InvalidParams("the merchants' stock names the special class")
    );
    let boat = E::InvalidParams("the boat's price gives each resource once, at least 1");
    assert_eq!(param(|d| d.boat_price[0].1 = 0), boat);
    assert_eq!(param(|d| d.boat_price[1].0 = key("t:gold")), boat);
    assert_eq!(
        param(|d| d.university_skills.push(key("t:fire"))),
        E::InvalidParams("a University's skill is given twice")
    );
    assert_eq!(
        param(|d| d.university_skills.clear()),
        E::InvalidParams("the University offers no skill")
    );
    assert_eq!(
        param(|d| d.university_skills[0] = key("t:nope")),
        E::UnknownSkill("t:nope".into())
    );
    assert_eq!(
        param(|d| d.university_price.push((key("t:wood"), 0))),
        E::InvalidParams("the University's price gives each resource once, at least 1")
    );
    assert_eq!(
        param(|d| d.transformer_into = key("t:nope")),
        E::UnknownCreature("t:nope".into())
    );
    assert_eq!(
        param(|d| d.transformer_dragons_into = key("u:frog")),
        E::ReferenceOutsideDependencies {
            key: "u:frog".into(),
            pack: "t".into(),
        }
    );
    assert_eq!(
        refused(|p| p[3].transformer_dragons.push(key("t:gog"))),
        E::InvalidParams("a Transformer dragon is named twice")
    );
    assert_eq!(
        refused(|p| p[3].transformer_dragons.push(key("t:nope"))),
        E::UnknownCreature("t:nope".into())
    );
    // Dragons without the parameters (a pack set whose only content is them).
    let core = core();
    let mut alone: Vec<TownPack> = packs()
        .into_iter()
        .map(|p| TownPack::new(p.manifest))
        .collect();
    alone[3].transformer_dragons.push(key("t:gog"));
    assert_eq!(
        TownRuleset::compile(&core, &alone).unwrap_err(),
        E::MissingParams
    );
    // A building's war machine: the core's, with a war machine creature.
    assert_eq!(
        refused(|p| keep_building(p, "silo").war_machine = Some(key("t:nope"))),
        E::UnknownArtifact("t:nope".into())
    );
    assert_eq!(
        refused(|p| keep_building(p, "silo").war_machine = Some(key("t:ring"))),
        building_error(
            "silo",
            "its war machine's artifact names no war machine creature"
        )
    );
    // At most one building of each trade role a record.
    let twice = town_error(
        "t:keep",
        "has more than one Marketplace, Artifact Merchants, Freelancer's Guild, University or \
         Transformer",
    );
    let changes: [fn(&mut BuildingRoles); 5] = [
        |r| r.marketplace = true,
        |r| r.artifact_merchants = true,
        |r| r.freelancers = true,
        |r| r.university = true,
        |r| r.transformer = true,
    ];
    for change in changes {
        assert_eq!(
            refused(|p| change(&mut keep_building(p, "tavern").roles)),
            twice
        );
    }
}

// covers: TWN-GLD-001
#[test]
fn the_guild_weights_are_the_cores_hero_spells_by_index() {
    let (core, part) = compiled();
    let spell = |k: &str| core.spell_by_key(k).unwrap();
    let faction = |k: &str| core.faction_by_key(k).unwrap();
    let record = part.town(faction("t:keep")).unwrap();
    assert_eq!(record.guild.len(), core.spells().len());
    let weights =
        ["t:bolt", "t:fly", "t:howl", "t:titan", "u:mist"].map(|k| record.guild_weight(spell(k)));
    assert_eq!(weights, [60, 40, 0, 0, 0]);
    let marsh = part.town(faction("u:marsh")).unwrap();
    assert_eq!(marsh.guild_weight(spell("u:mist")), 25);
    // A spell past the core's weighs 0.
    assert_eq!(record.guild_weight(SpellId::from_index(99).unwrap()), 0);
    // A spell granted by name only may be listed; at the bounds 1 and 10 000.
    let mut listed = packs();
    keep(&mut listed).guild = vec![(key("t:titan"), 1), (key("t:bolt"), 10_000)];
    let part = TownRuleset::compile(&core, &listed).unwrap();
    let record = part.town(faction("t:keep")).unwrap();
    assert_eq!(
        (
            record.guild_weight(spell("t:titan")),
            record.guild_weight(spell("t:bolt")),
            record.guild_weight(spell("t:fly"))
        ),
        (1, 10_000, 0)
    );
    // The refusals.
    use TownCompileError as E;
    let reason = |change: fn(&mut TownDef)| refused(|p| change(keep(p)));
    assert_eq!(
        reason(|t| t.guild[0].0 = key("t:nope")),
        E::UnknownSpell("t:nope".into())
    );
    assert_eq!(
        reason(|t| t.guild.push((key("u:mist"), 1))),
        E::ReferenceOutsideDependencies {
            key: "u:mist".into(),
            pack: "t".into(),
        }
    );
    assert_eq!(
        reason(|t| t.guild.push((key("t:howl"), 1))),
        town_error("t:keep", "a guild weight's spell is a creature ability")
    );
    let bounds = town_error("t:keep", "a guild weight is 1 to 10 000");
    assert_eq!(reason(|t| t.guild[1].1 = 0), bounds);
    assert_eq!(reason(|t| t.guild[1].1 = 10_001), bounds);
    assert_eq!(
        reason(|t| t.guild.push((key("t:bolt"), 5))),
        town_error("t:keep", "a guild weight's spell is given twice")
    );
}

// covers: TWN-GLD-001
#[test]
fn the_guilds_parameters_are_checked() {
    let (core, part) = compiled();
    let params = part.params().unwrap();
    let gold = core.resources().iter().position(|r| r.key == key("t:gold"));
    let gold = ResourceId::from_index(gold.unwrap()).unwrap();
    assert_eq!(params.guild_slots, [5, 4, 3, 2, 1]);
    assert_eq!(params.spell_book_price, [(gold, 500)]);
    assert_eq!(params.vortex_factor, 2);
    use TownCompileError as E;
    let param = |change: fn(&mut TownParamsDef)| refused(|p| change(p[0].params.as_mut().unwrap()));
    let slots = E::InvalidParams("a guild level's slots are 1 to 5");
    assert_eq!(param(|d| d.guild_slots[0] = 0), slots);
    assert_eq!(param(|d| d.guild_slots[4] = 6), slots);
    let price = E::InvalidParams("the spell book's price gives each resource once, at least 1");
    assert_eq!(param(|d| d.spell_book_price[0].1 = 0), price);
    assert_eq!(
        param(|d| d.spell_book_price.push((key("t:gold"), 1))),
        price
    );
    assert_eq!(
        param(|d| d.spell_book_price[0].0 = key("t:nope")),
        E::UnknownResource("t:nope".into())
    );
    let factor = E::InvalidParams("the Mana Vortex's factor is 1 to 10");
    assert_eq!(param(|d| d.vortex_factor = 0), factor);
    assert_eq!(param(|d| d.vortex_factor = 11), factor);
    // At the bounds, a free book: compiles, the price in the core's order.
    let mut bounds = packs();
    let p = bounds[0].params.as_mut().unwrap();
    p.guild_slots = [1, 5, 5, 5, 1];
    p.vortex_factor = 10;
    p.spell_book_price.clear();
    let part = TownRuleset::compile(&core, &bounds).unwrap();
    assert!(part.params().unwrap().spell_book_price.is_empty());
    let mut both = packs();
    both[0].params.as_mut().unwrap().spell_book_price =
        vec![(key("t:gold"), 5), (key("t:wood"), 1)];
    let part = TownRuleset::compile(&core, &both).unwrap();
    let wood = ResourceId::from_index(0).unwrap();
    assert_eq!(
        part.params().unwrap().spell_book_price,
        [(wood, 1), (gold, 5)]
    );
}

// covers: TWN-GLD-001
#[test]
fn the_guilds_roles_are_checked() {
    let guild = |level: u8| BuildingRoles {
        guild: Some(level),
        ..BuildingRoles::default()
    };
    let with_guild = |t: &mut TownDef| {
        t.buildings.push(building("mage_guild_1", guild(1)));
        t.buildings
            .push(superseding("mage_guild_2", "mage_guild_1", guild(2)));
        t.buildings.push(building(
            "library",
            BuildingRoles {
                library: true,
                ..BuildingRoles::default()
            },
        ));
        t.buildings.push(building(
            "vortex",
            BuildingRoles {
                mana_vortex: true,
                ..BuildingRoles::default()
            },
        ));
        let aurora = &mut t
            .buildings
            .iter_mut()
            .find(|b| b.key == "grail")
            .unwrap()
            .roles;
        aurora.aurora = true;
    };
    let mut packs_with = packs();
    with_guild(keep(&mut packs_with));
    let core = core();
    let part = TownRuleset::compile(&core, &packs_with).unwrap();
    let keep_record = part.town(core.faction_by_key("t:keep").unwrap()).unwrap();
    let roles = |k: &str| {
        keep_record
            .building(keep_record.building_by_key(k).unwrap())
            .unwrap()
            .roles
    };
    assert_eq!(
        (roles("mage_guild_1").guild, roles("mage_guild_2").guild),
        (Some(1), Some(2))
    );
    assert!(roles("library").library && roles("vortex").mana_vortex && roles("grail").aurora);
    assert!(!roles("tavern").library && roles("tavern").guild.is_none());
    // A level outside 1 to 5.
    let level = |raw: u8| {
        refused(move |p| {
            with_guild(keep(p));
            keep_building(p, "mage_guild_2").roles.guild = Some(raw);
        })
    };
    assert_eq!(
        level(0),
        building_error("mage_guild_2", "a guild level is 1 to 5")
    );
    assert_eq!(
        level(6),
        building_error("mage_guild_2", "a guild level is 1 to 5")
    );
    assert!(
        TownRuleset::compile(&core, &{
            let mut p = packs_with.clone();
            keep_building(&mut p, "mage_guild_2").roles.guild = Some(5);
            p
        })
        .is_ok()
    );
    // A level twice, a second Library, Aurora or Vortex.
    let reason = |change: fn(&mut TownDef)| {
        refused(|p| {
            with_guild(keep(p));
            change(keep(p));
        })
    };
    assert_eq!(
        reason(|t| t.buildings.push(building(
            "guild_again",
            BuildingRoles {
                guild: Some(1),
                ..BuildingRoles::default()
            }
        ))),
        town_error("t:keep", "has two buildings of a guild level")
    );
    let one = town_error(
        "t:keep",
        "has more than one Library, Aurora Borealis or Mana Vortex",
    );
    let second = |change: fn(&mut BuildingRoles)| {
        refused(move |p| {
            with_guild(keep(p));
            change(&mut keep_building(p, "tavern").roles);
        })
    };
    assert_eq!(second(|r| r.library = true), one);
    assert_eq!(second(|r| r.aurora = true), one);
    assert_eq!(second(|r| r.mana_vortex = true), one);
}

// covers: TWN-TAV-008
#[test]
fn the_guilds_resource_rows_are_the_cores_each_in_at_most_one_row() {
    let (core, part) = compiled();
    let id = |k: &str| {
        let index = core.resources().iter().position(|r| r.key == key(k));
        ResourceId::from_index(index.unwrap()).unwrap()
    };
    let rows = &part.params().unwrap().guild_resources;
    assert_eq!(rows, &[vec![id("t:gold")], vec![id("t:wood")], vec![]]);
    // Each row in the core's order, whatever the pack's.
    let mut packs = packs();
    packs[0].params.as_mut().unwrap().guild_resources =
        [vec![], vec![key("t:gold"), key("t:wood")], vec![]];
    let part = TownRuleset::compile(&core, &packs).unwrap();
    let rows = &part.params().unwrap().guild_resources;
    assert_eq!(rows[1], [id("t:wood"), id("t:gold")]);
    let param = |change: fn(&mut TownParamsDef)| refused(|p| change(p[0].params.as_mut().unwrap()));
    assert_eq!(
        param(|d| d.guild_resources[2].push(key("t:nope"))),
        TownCompileError::UnknownResource("t:nope".into())
    );
    assert_eq!(
        param(|d| d.guild_resources[2].push(key("t:gold"))),
        TownCompileError::InvalidParams("a resource of the Thieves' Guild's rows is given twice")
    );
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
    // Increment 4's (E09 T36): a tavern weight's class and the hire's price.
    assert_eq!(
        refused(|p| keep(p).tavern[0].0 = key("t:nope")),
        E::UnknownHeroClass("t:nope".into())
    );
    assert_eq!(
        refused(|p| keep(p).tavern.push((key("u:seer"), 1))),
        outside("u:seer", "t")
    );
    assert_eq!(
        refused(|p| p[0].params.as_mut().unwrap().hire_price[0].0 = key("t:nope")),
        E::UnknownResource("t:nope".into())
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
    let chance = E::InvalidParams("the week's creature chance needs 1 <= numerator <= denominator");
    assert_eq!(param(|d| d.week.creature_chance = [0, 4]), chance);
    assert_eq!(param(|d| d.week.creature_chance = [5, 4]), chance);
    let odds = E::InvalidParams("the garrison week's odds need 0 <= odds <= out_of, out_of >= 1");
    assert_eq!(param(|d| d.garrison_week.odds = [40, 102]), odds);
    assert_eq!(param(|d| d.garrison_week.odds = [102, 80]), odds);
    assert_eq!(
        param(|d| {
            d.garrison_week.odds = [0, 0];
            d.garrison_week.out_of = 0;
        }),
        odds
    );
    let weights = E::InvalidParams("the garrison week has one to six level weights");
    assert_eq!(param(|d| d.garrison_week.weights.clear()), weights);
    assert_eq!(param(|d| d.garrison_week.weights.push(1)), weights);
    assert_eq!(
        param(|d| d.garrison_week.upgrade = 101),
        E::InvalidParams("the garrison week's upgrade chance is above 100")
    );
    assert_eq!(
        param(|d| d.most_heroes = 0),
        E::InvalidParams("the most heroes on the map needs at least 1")
    );
    let price = E::InvalidParams("the hire's price gives each resource once, at least 1");
    assert_eq!(param(|d| d.hire_price[0].1 = 0), price);
    assert_eq!(param(|d| d.hire_price.push((key("t:gold"), 1))), price);
    // A free hire compiles.
    let mut free = packs();
    free[0].params.as_mut().unwrap().hire_price.clear();
    assert!(TownRuleset::compile(&core(), &free).is_ok());
    // At the bounds: a chance of 1 in 1, both odds the denominator, one weight, a sure upgrade.
    let mut at_bounds = packs();
    let p = at_bounds[0].params.as_mut().unwrap();
    p.week.creature_chance = [1, 1];
    p.garrison_week = GarrisonWeekParams {
        odds: [101, 101],
        out_of: 101,
        weights: vec![0],
        weeks: 0,
        upgrade: 100,
    };
    assert!(TownRuleset::compile(&core(), &at_bounds).is_ok());
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
    // Increment 4's (E09 T36): the tavern weights, each class once (the override's too), not all
    // 0 after the overrides (the Keep's 0 weights are filled by `u`'s override).
    let twice = town_error("t:keep", "a tavern weight's class is given twice");
    assert_eq!(reason(|t| t.tavern.push((key("t:knight"), 1))), twice);
    assert_eq!(
        refused(|p| {
            let patch = &mut p[1].overrides[0].patch;
            patch.tavern.as_mut().unwrap().push((key("u:seer"), 1));
        }),
        twice
    );
    let nothing = town_error("t:wild", "the tavern weights sum to 0");
    assert_eq!(refused(|p| p[0].towns[1].tavern.clear()), nothing);
    assert_eq!(
        refused(|p| p[0].towns[1].tavern.iter_mut().for_each(|w| w.1 = 0)),
        nothing
    );
    let mut filled = packs();
    keep(&mut filled).tavern.clear();
    assert!(TownRuleset::compile(&core(), &filled).is_ok());
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
    // Increment 3's (E09 T25a): each index's horde once (a level's base and upgrade apart), one
    // Portal of Summoning, one week's building.
    assert_eq!(
        reason(|t| t.buildings.push(building("horde_b", horde(2, false)))),
        town_error("t:keep", "has a horde of one index twice")
    );
    let one = town_error(
        "t:keep",
        "has more than one Portal of Summoning or week's building",
    );
    let second = |roles: BuildingRoles| {
        move |t: &mut TownDef| {
            t.buildings.push(building("second", roles));
        }
    };
    let portal = BuildingRoles {
        portal: true,
        ..BuildingRoles::default()
    };
    let week = BuildingRoles {
        week_of: Some(2),
        ..BuildingRoles::default()
    };
    assert_eq!(refused(|p| second(portal)(keep(p))), one);
    assert_eq!(refused(|p| second(week)(keep(p))), one);
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
    // Increment 3's (E09 T25a): a horde's level and upgrade, a week's level.
    let hordes = "a horde's level, or its upgrade, is not the faction's";
    for (level, upgraded) in [(0, false), (3, false), (2, true)] {
        let error = refused(|p| {
            keep_building(p, "horde").roles.horde = Some(Horde { level, upgraded });
        });
        assert_eq!(error, building_error("horde", hordes));
    }
    for level in [0, 3] {
        let error = refused(|p| keep_building(p, "grail").roles.week_of = Some(level));
        assert_eq!(
            error,
            building_error("grail", "a week's level is not the faction's")
        );
    }
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
            E::UnknownHeroClass("p:x".into()),
            "unknown hero class `p:x`",
        ),
        (E::UnknownSpell("p:x".into()), "unknown spell `p:x`"),
        (E::UnknownArtifact("p:x".into()), "unknown artifact `p:x`"),
        (E::UnknownSkill("p:x".into()), "unknown skill `p:x`"),
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
