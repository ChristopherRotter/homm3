//! Tests of the adventure pack format (E08 T4).

use std::error::Error;

use h3_adventure::{KEY_COLOURS, ObjectKind, Pickup};
use h3_content::ResourceId;

use super::*;

const PROFILE: &str = "base:faithful";

fn manifest(id: &str) -> PackManifest {
    PackManifest {
        id: id.into(),
        version: "1".into(),
        depends: vec![],
    }
}

fn parse(text: &str) -> Result<AdventurePack, AdventurePackError> {
    parse_adventure(&manifest("p"), text)
}

fn cost(value: u16) -> NonZeroU16 {
    NonZeroU16::new(value).unwrap()
}

fn key_of(text: &str) -> ContentKey {
    ContentKey::parse(text).unwrap()
}

// covers: GAM-SET-001
#[test]
fn the_original_starting_armies_fit_the_draw() {
    let (core, part) = compile(&ORIGINAL_PACKS, PROFILE).unwrap();
    assert_eq!(part.params().starting_stack_chances(), [88, 25]);
    // The draw reads a hero type's starting slots as its first stack, its war machines, then the
    // other stacks: the original's hero table has three slots and every war machine in the
    // second (Heroes3.exe 0x679DD0 + 0x24; E08 T6b). The packs keep that shape.
    let mut machines = 0;
    for hero in core.hero_types() {
        let slots = (hero.army.len(), hero.war_machines.len());
        assert!(
            slots.0 >= 1 && slots.0 + slots.1 <= 3,
            "{}: {slots:?}",
            hero.key.as_str()
        );
        if slots.1 > 0 {
            machines += 1;
            assert_eq!(slots, (2, 1), "{}", hero.key.as_str());
        }
    }
    assert_eq!(machines, 9);
}

// covers: ADV-MOV-001, ADV-MOV-003, ADV-MOV-004, ADV-MOV-007, ADV-FOG-001
#[test]
fn the_base_pack_holds_the_sourced_values() {
    let (core, part) = compile(&ORIGINAL_PACKS[..1], PROFILE).unwrap();
    let params = part.params();
    assert_eq!(
        params.land_points_table(),
        [
            1300, 1360, 1430, 1500, 1560, 1630, 1700, 1760, 1830, 1900, 1960, 2000
        ]
    );
    assert_eq!(params.land_points(20), 2000);
    assert_eq!(params.native_cost(), cost(100));
    assert_eq!(params.scouting_radius(), 5);
    // GAM-BAT-001 (E08 T19a): underground on subterranean, on the coast on sand and the shore.
    let battle = params.battle_terrains().unwrap();
    let terrain = |k| core.terrain_by_key(k).unwrap();
    assert_eq!(
        (
            battle.underground,
            battle.coast,
            battle.coast_battlefield.as_str()
        ),
        (
            terrain("base:subterranean"),
            terrain("base:desert"),
            "base:shore"
        )
    );
    let passages: Vec<(&str, Passage)> = part
        .terrains()
        .iter()
        .map(|t| (t.key.as_str(), t.passage))
        .collect();
    let land = |c| Passage::Land(cost(c));
    assert_eq!(
        passages,
        [
            ("base:dirt", land(100)),
            ("base:desert", land(150)),
            ("base:grass", land(100)),
            ("base:snow", land(150)),
            ("base:swamp", land(175)),
            ("base:rough", land(125)),
            ("base:subterranean", land(100)),
            ("base:lava", land(100)),
            ("base:water", Passage::Water(cost(100))),
            ("base:rock", Passage::Impassable),
        ]
    );
    assert_eq!(part.terrains().len(), core.terrains().len());
    // E08 T29: the straight step by Pathfinding level 0 to 3 (the original's step_cost_table
    // 0x63E510: sand and snow 125, 100, 100; swamp 150, 125, 100; rough 100 at every level).
    let by_level: Vec<(&str, [Option<u16>; 4])> = core
        .terrains()
        .iter()
        .map(|info| {
            let t = terrain(info.key.as_str());
            let at = |level| part.step_cost(t, level).map(NonZeroU16::get);
            (info.key.as_str(), [at(0), at(1), at(2), at(3)])
        })
        .collect();
    let all = |c| [Some(c); 4];
    assert_eq!(
        by_level,
        [
            ("base:dirt", all(100)),
            ("base:desert", [Some(150), Some(125), Some(100), Some(100)]),
            ("base:grass", all(100)),
            ("base:snow", [Some(150), Some(125), Some(100), Some(100)]),
            ("base:swamp", [Some(175), Some(150), Some(125), Some(100)]),
            ("base:rough", [Some(125), Some(100), Some(100), Some(100)]),
            ("base:subterranean", all(100)),
            ("base:lava", all(100)),
            ("base:water", all(100)),
            ("base:rock", [None; 4]),
        ]
    );
    let roads: Vec<(&str, u16)> = part
        .roads()
        .iter()
        .map(|r| (r.key.as_str(), r.cost.get()))
        .collect();
    assert_eq!(
        roads,
        [
            ("base:dirt_road", 75),
            ("base:gravel_road", 65),
            ("base:cobblestone_road", 50)
        ]
    );
    let rivers: Vec<&str> = part.rivers().iter().map(|r| r.key.as_str()).collect();
    assert_eq!(
        rivers,
        [
            "base:clear_river",
            "base:icy_river",
            "base:muddy_river",
            "base:lava_river"
        ]
    );
}

// covers: ADV-OBJ-002, ADV-PIK-001, ADV-PIK-002, ADV-PIK-003, ADV-VIS-001, ADV-VIS-002
// covers: ADV-VIS-003, ADV-VIS-004
#[test]
fn the_base_pack_types_the_objects_of_increment_2() {
    let (core, part) = compile(&ORIGINAL_PACKS[..1], PROFILE).unwrap();
    let resource = |key: &str| {
        let index = core
            .resources()
            .iter()
            .position(|r| r.key.as_str() == key)
            .unwrap();
        ResourceId::from_index(index).unwrap()
    };
    let ty = |key: &str| {
        part.object_type(part.object_type_by_key(key).unwrap())
            .unwrap()
    };
    let piles = [
        ("wood", 5, 10, 1),
        ("mercury", 3, 6, 1),
        ("ore", 5, 10, 1),
        ("sulfur", 3, 6, 1),
        ("crystal", 3, 6, 1),
        ("gems", 3, 6, 1),
        ("gold", 5, 10, 100),
    ];
    for (name, low, high, multiplier) in piles {
        let pile = ty(&format!("base:pile_{name}"));
        assert_eq!((pile.visit, pile.from_above), (Visit::Neighbour, true));
        assert!(pile.takes_amount());
        assert_eq!(
            pile.kind,
            ObjectKind::Pickup(Pickup::Resource {
                resource: resource(&format!("base:{name}")),
                amount: [low, high],
                multiplier: NonZeroU32::new(multiplier).unwrap(),
            })
        );
    }
    let random = ty("base:random_resource");
    let ObjectKind::Pickup(Pickup::OneOf(choices)) = &random.kind else {
        panic!("{:?}", random.kind);
    };
    let choices: Vec<&str> = choices
        .iter()
        .map(|&c| part.object_type(c).unwrap().key.as_str())
        .collect();
    assert_eq!(choices.len(), 7);
    assert!(choices.iter().zip(piles).all(|(c, p)| c.ends_with(p.0)));
    assert!(random.takes_amount());
    let campfire = ty("base:campfire");
    assert!(!campfire.takes_amount());
    assert_eq!(
        campfire.kind,
        ObjectKind::Pickup(Pickup::Paired {
            amount: [4, 6],
            resources: ["wood", "mercury", "ore", "sulfur", "crystal", "gems"]
                .map(|r| resource(&format!("base:{r}")))
                .to_vec(),
            paired: resource("base:gold"),
            multiplier: NonZeroU32::new(100).unwrap(),
        })
    );
    let terrain = |key: &str| core.terrain_by_key(key).unwrap();
    let gold = vec![(resource("base:gold"), 1000)];
    for (kind, terrains) in [
        ("water", vec!["base:water"]),
        (
            "land",
            vec![
                "base:dirt",
                "base:desert",
                "base:grass",
                "base:snow",
                "base:swamp",
                "base:rough",
                "base:lava",
            ],
        ),
        ("subterranean", vec!["base:subterranean"]),
    ] {
        let cartographer = ty(&format!("base:cartographer_{kind}"));
        assert_eq!(
            (cartographer.visit, cartographer.from_above),
            (Visit::Enter, false)
        );
        assert_eq!(
            cartographer.kind,
            ObjectKind::RevealTerrain {
                terrains: terrains.into_iter().map(terrain).collect(),
                price: gold.clone(),
            }
        );
    }
    for key in ["base:redwood_observatory", "base:pillar_of_fire"] {
        assert_eq!(ty(key).kind, ObjectKind::RevealArea { radius: 20 });
        assert_eq!(ty(key).visit, Visit::Enter);
    }
    let colours = [
        "light_blue",
        "green",
        "red",
        "dark_blue",
        "brown",
        "purple",
        "white",
        "black",
    ];
    for (key, colour) in (0..KEY_COLOURS).zip(colours) {
        let tent = ty(&format!("base:keymaster_{colour}"));
        assert_eq!(tent.kind, ObjectKind::KeyTent { key });
        assert_eq!((tent.visit, tent.from_above), (Visit::Enter, false));
        let guard = ty(&format!("base:border_guard_{colour}"));
        assert_eq!(guard.kind, ObjectKind::KeyGuard { key });
        assert_eq!((guard.visit, guard.from_above), (Visit::Neighbour, true));
    }
    assert_eq!(part.object_types().len(), 7 + 1 + 1 + 3 + 2 + 8 + 8 + 9 + 2);
    assert!(part.object_type_by_key("base:border_gate_red").is_none());
}

// covers: ADV-MON-001, ADV-MON-002, ADV-MON-003, ADV-GRD-001
#[test]
fn the_packs_type_the_monsters_and_the_grounds() {
    let (core, part) = compile(&ORIGINAL_PACKS, PROFILE).unwrap();
    let ty = |key: &str| {
        part.object_type(part.object_type_by_key(key).unwrap())
            .unwrap()
    };
    let monster = ty("base:monster");
    assert_eq!(monster.kind, ObjectKind::Monster { levels: None });
    assert_eq!(
        (monster.visit, monster.from_above),
        (Visit::Neighbour, true)
    );
    assert_eq!(
        ty("base:random_monster").kind,
        ObjectKind::Monster {
            levels: Some([1, 7])
        }
    );
    for level in 1..=7 {
        let random = ty(&format!("base:random_monster_{level}"));
        assert_eq!(
            random.kind,
            ObjectKind::Monster {
                levels: Some([level, level])
            }
        );
        assert_eq!((random.visit, random.from_above), (Visit::Neighbour, true));
    }
    let monsters = part.params().monsters().unwrap();
    let aggression: Vec<_> = (0..6).map(|d| monsters.aggression(d)).collect();
    assert_eq!(
        aggression,
        [
            Some([-4, -4]),
            Some([1, 7]),
            Some([1, 10]),
            Some([4, 10]),
            Some([10, 10]),
            None
        ]
    );
    // The wiki's words: few 1-4, several 5-9 ... zounds 500-999, legion from 1 000.
    let words: Vec<u8> = [1, 4, 5, 9, 10, 49, 50, 99, 100, 999, 1000, 4095]
        .map(|count| monsters.count_word(count))
        .to_vec();
    assert_eq!(words, [0, 0, 1, 1, 2, 3, 4, 4, 5, 7, 8, 8]);
    let grounds = [
        ("base:cursed_ground", "base:cursed_ground"),
        ("base:magic_plains", "base:magic_plains"),
        ("sod:clover_field", "sod:clover_field"),
        ("sod:cursed_ground", "base:cursed_ground"),
        ("sod:evil_fog", "sod:evil_fog"),
        ("sod:fiery_fields", "sod:fiery_fields"),
        ("sod:holy_ground", "sod:holy_ground"),
        ("sod:lucid_pools", "sod:lucid_pools"),
        ("sod:magic_clouds", "sod:magic_clouds"),
        ("sod:magic_plains", "base:magic_plains"),
        ("sod:rocklands", "sod:rocklands"),
    ];
    for (key, battlefield) in grounds {
        assert_eq!(
            ty(key).kind,
            ObjectKind::Ground {
                battlefield: ContentKey::parse(battlefield).unwrap(),
                no_native: key.ends_with("cursed_ground"),
            }
        );
    }
    // The random monsters' creatures: every creature of the levels but the war machines and
    // the expansions' six (ids 132 to 137); the elementals by the executable's levels in
    // faithful (BAT-QUIRK-19), by their base creatures' otherwise.
    let keys = |core: &Ruleset, part: &AdventureRuleset, levels| {
        part.random_monsters(core, levels)
            .map(|id| core.creature_key(id).unwrap().as_str().to_owned())
            .collect::<Vec<_>>()
    };
    let all = keys(&core, &part, [1, 7]);
    assert_eq!(all.len(), 118 + 23 - 6);
    for key in [
        "ab:azure_dragon",
        "ab:enchanter",
        "ab:sharpshooter",
        "base:ballista",
    ] {
        assert!(!all.iter().any(|k| k == key), "{key}");
    }
    assert!(all.iter().any(|k| k == "ab:pixie"));
    assert!(
        keys(&core, &part, [4, 4])
            .iter()
            .any(|k| k == "ab:ice_elemental")
    );
    let (fixed_core, fixed) = compile(&ORIGINAL_PACKS, "base:strict-fixed").unwrap();
    assert!(
        keys(&fixed_core, &fixed, [3, 3])
            .iter()
            .any(|k| k == "ab:ice_elemental")
    );
    // RoE alone draws among its own 118.
    let (roe_core, roe) = compile(&ORIGINAL_PACKS[..1], PROFILE).unwrap();
    assert_eq!(keys(&roe_core, &roe, [1, 7]).len(), 118);
    assert_eq!(keys(&roe_core, &roe, [7, 7]).len(), 16);
}

// covers: ADV-VIS-005
#[test]
fn the_expansions_add_the_border_gates() {
    let (_, base) = compile(&ORIGINAL_PACKS[..1], PROFILE).unwrap();
    let (ab_core, ab) = compile(&ORIGINAL_PACKS[..2], PROFILE).unwrap();
    let (sod_core, sod) = compile(&ORIGINAL_PACKS, PROFILE).unwrap();
    assert_ne!(ab.content_hash(), base.content_hash());
    assert_ne!(sod.content_hash(), ab.content_hash());
    assert_eq!(ab.object_types().len(), base.object_types().len() + 8);
    assert_eq!(
        ab.object_types()[..base.object_types().len()],
        *base.object_types()
    );
    // SoD adds its nine special grounds (E08 T20a2a).
    assert_eq!(sod.object_types().len(), ab.object_types().len() + 9);
    let gate = ab
        .object_type(ab.object_type_by_key("ab:border_gate_black").unwrap())
        .unwrap();
    assert_eq!(gate.kind, ObjectKind::KeyGate { key: 7 });
    assert_eq!((gate.visit, gate.from_above), (Visit::Neighbour, true));
    // The parts still belong to their own core rulesets.
    assert_eq!(ab.core_hash(), ab_core.content_hash());
    assert_eq!(sod.core_hash(), sod_core.content_hash());
    assert_ne!(ab.core_hash(), sod.core_hash());
}

// covers: GAM-RES-002
#[test]
fn the_base_pack_holds_the_starting_resources() {
    let (core, part) = compile(&ORIGINAL_PACKS, PROFILE).unwrap();
    let order: Vec<&str> = core.resources().iter().map(|r| r.key.as_str()).collect();
    assert_eq!(
        order,
        [
            "base:wood",
            "base:mercury",
            "base:ore",
            "base:sulfur",
            "base:crystal",
            "base:gems",
            "base:gold"
        ]
    );
    // Survey E08-objects.md §7.2: wood and ore, each rare resource, gold; human, then computer.
    let row = |wood_ore: u32, rare: u32, gold: u32| {
        vec![wood_ore, rare, wood_ore, rare, rare, rare, gold]
    };
    let rows: Vec<(Vec<u32>, Vec<u32>)> = part
        .starting_resources()
        .iter()
        .map(|r| (r.human().to_vec(), r.computer().to_vec()))
        .collect();
    assert_eq!(
        rows,
        [
            (row(30, 15, 30_000), row(5, 2, 5_000)),
            (row(20, 10, 20_000), row(10, 4, 7_500)),
            (row(15, 7, 15_000), row(15, 7, 10_000)),
            (row(10, 4, 10_000), row(15, 7, 10_000)),
            (row(0, 0, 0), row(15, 7, 10_000)),
        ]
    );
}

#[test]
fn the_synthetic_packs_compile() {
    let (_, part) = compile(
        &[
            (h3_pack::SYNTHETIC_EXT_CORE, SYNTHETIC_EXT_ADVENTURE),
            (h3_pack::SYNTHETIC_CORE, SYNTHETIC_ADVENTURE),
        ],
        "synthetic:faithful",
    )
    .unwrap();
    let roads: Vec<&str> = part.roads().iter().map(|r| r.key.as_str()).collect();
    assert_eq!(roads, ["synthetic:trail", "synthetic-ext:causeway"]);
    assert_eq!(part.river_by_key("synthetic-ext:creek").unwrap().raw(), 1);
    assert_eq!(part.params().land_points(9), 1700);
}

#[test]
fn parses_every_field() {
    let pack = parse(
        r#"
        not_random_monsters = ["imp", "other:gog"]

        [params]
        native_cost = 90
        land_points = [1000, 1100]
        scouting_radius = 3
        max_map_size = 72
        max_levels = 1
        max_players = 4
        army_slots = 5
        starting_stack_chances = [80, 20]
        fly_cost = [140, 140, 120, 100]
        water_walk_cost = [150, 140, 125, 110]

        [params.monsters]
        aggression = [[-4, -4], [1, 7]]
        count_words = [5, 10]
        price_resource = "coin"

        [[terrain]]
        terrain = "plain"
        cost = 110
        pathfinding = [100, 100, 90]

        [[terrain]]
        terrain = "other:sea"
        passage = "water"
        cost = 120

        [[terrain]]
        terrain = "cliff"
        passage = "impassable"

        [[road]]
        id = "path"
        name = "Path"
        cost = 80

        [[river]]
        id = "stream"
        name = "Stream"

        [[difficulty]]
        human = { coin = 10, "other:gem" = 2 }
        computer = {}

        [[object_type]]
        id = "heap"
        name = "Heap"
        visit = "neighbour"
        from_above = true
        pickup = { resource = "coin", amount = [2, 4] }

        [[object_type]]
        id = "big_heap"
        name = "Big heap"
        visit = "neighbour"
        from_above = false
        pickup = { resource = "other:gem", amount = [1, 1], multiplier = 10 }

        [[object_type]]
        id = "any_heap"
        name = "Any heap"
        visit = "neighbour"
        from_above = true
        pickup_one_of = ["heap", "other:heap"]

        [[object_type]]
        id = "fire"
        name = "Fire"
        visit = "neighbour"
        from_above = true
        pickup_paired = { amount = [1, 2], resources = ["coin", "shard"], paired = "gem" }

        [[object_type]]
        id = "mapper"
        name = "Mapper"
        visit = "enter"
        from_above = false
        reveal_terrain = { terrains = ["plain", "other:sea"], price = { coin = 5, "other:gem" = 1 } }

        [[object_type]]
        id = "free_mapper"
        name = "Free mapper"
        visit = "enter"
        from_above = false
        reveal_terrain = { terrains = ["plain"] }

        [[object_type]]
        id = "tower"
        name = "Tower"
        visit = "enter"
        from_above = false
        reveal_area = { radius = 9 }

        [[object_type]]
        id = "tent"
        name = "Tent"
        visit = "enter"
        from_above = false
        key_tent = { key = 1 }

        [[object_type]]
        id = "guard"
        name = "Guard"
        visit = "neighbour"
        from_above = true
        key_guard = { key = 2 }

        [[object_type]]
        id = "gate"
        name = "Gate"
        visit = "neighbour"
        from_above = true
        key_gate = { key = 3 }

        [[object_type]]
        id = "monster"
        name = "Monster"
        visit = "neighbour"
        from_above = true
        monster = {}

        [[object_type]]
        id = "random_monster"
        name = "Random monster"
        visit = "neighbour"
        from_above = true
        monster = { levels = [1, 7] }

        [[object_type]]
        id = "cursed_ground"
        name = "Cursed ground"
        visit = "enter"
        from_above = true
        ground = { battlefield = "other:cursed", no_native = true }

        [[terrain_walker]]
        creature = "nomad"
        terrain = "other:sand"
        counts_as = "dirt"
        "#,
    )
    .unwrap();
    assert_eq!(pack.manifest, manifest("p"));
    let terrains: Vec<(&str, Passage)> = pack
        .terrains
        .iter()
        .map(|t| (t.terrain.as_str(), t.passage))
        .collect();
    assert_eq!(
        terrains,
        [
            ("p:plain", Passage::Land(cost(110))),
            ("other:sea", Passage::Water(cost(120))),
            ("p:cliff", Passage::Impassable),
        ]
    );
    let pathfinding: Vec<_> = pack.terrains.iter().map(|t| t.pathfinding).collect();
    assert_eq!(
        pathfinding,
        [Some([cost(100), cost(100), cost(90)]), None, None]
    );
    assert_eq!(pack.roads[0].key.as_str(), "p:path");
    assert_eq!(
        (pack.roads[0].name.as_str(), pack.roads[0].cost),
        ("Path", cost(80))
    );
    assert_eq!(pack.rivers[0].key.as_str(), "p:stream");
    assert_eq!(pack.rivers[0].name, "Stream");
    let k = |text: &str| ContentKey::parse(text).unwrap();
    let n = |v: u32| NonZeroU32::new(v).unwrap();
    let kinds: Vec<(&str, &str, Visit, bool, &ObjectKindDef)> = pack
        .object_types
        .iter()
        .map(|t| {
            (
                t.key.as_str(),
                t.name.as_str(),
                t.visit,
                t.from_above,
                &t.kind,
            )
        })
        .collect();
    assert_eq!(
        kinds,
        [
            (
                "p:heap",
                "Heap",
                Visit::Neighbour,
                true,
                &ObjectKindDef::Pickup(PickupDef::Resource {
                    resource: k("p:coin"),
                    amount: [2, 4],
                    multiplier: n(1),
                })
            ),
            (
                "p:big_heap",
                "Big heap",
                Visit::Neighbour,
                false,
                &ObjectKindDef::Pickup(PickupDef::Resource {
                    resource: k("other:gem"),
                    amount: [1, 1],
                    multiplier: n(10),
                })
            ),
            (
                "p:any_heap",
                "Any heap",
                Visit::Neighbour,
                true,
                &ObjectKindDef::Pickup(PickupDef::OneOf(vec![k("p:heap"), k("other:heap")]))
            ),
            (
                "p:fire",
                "Fire",
                Visit::Neighbour,
                true,
                &ObjectKindDef::Pickup(PickupDef::Paired {
                    amount: [1, 2],
                    resources: vec![k("p:coin"), k("p:shard")],
                    paired: k("p:gem"),
                    multiplier: n(1),
                })
            ),
            (
                "p:mapper",
                "Mapper",
                Visit::Enter,
                false,
                &ObjectKindDef::RevealTerrain {
                    terrains: vec![k("p:plain"), k("other:sea")],
                    price: vec![(k("p:coin"), 5), (k("other:gem"), 1)],
                }
            ),
            (
                "p:free_mapper",
                "Free mapper",
                Visit::Enter,
                false,
                &ObjectKindDef::RevealTerrain {
                    terrains: vec![k("p:plain")],
                    price: vec![],
                }
            ),
            (
                "p:tower",
                "Tower",
                Visit::Enter,
                false,
                &ObjectKindDef::RevealArea { radius: 9 }
            ),
            (
                "p:tent",
                "Tent",
                Visit::Enter,
                false,
                &ObjectKindDef::KeyTent { key: 1 }
            ),
            (
                "p:guard",
                "Guard",
                Visit::Neighbour,
                true,
                &ObjectKindDef::KeyGuard { key: 2 }
            ),
            (
                "p:gate",
                "Gate",
                Visit::Neighbour,
                true,
                &ObjectKindDef::KeyGate { key: 3 }
            ),
            (
                "p:monster",
                "Monster",
                Visit::Neighbour,
                true,
                &ObjectKindDef::Monster { levels: None }
            ),
            (
                "p:random_monster",
                "Random monster",
                Visit::Neighbour,
                true,
                &ObjectKindDef::Monster {
                    levels: Some([1, 7])
                }
            ),
            (
                "p:cursed_ground",
                "Cursed ground",
                Visit::Enter,
                true,
                &ObjectKindDef::Ground {
                    battlefield: k("other:cursed"),
                    no_native: true,
                }
            ),
        ]
    );
    assert_eq!(pack.not_random_monsters, [k("p:imp"), k("other:gog")]);
    assert_eq!(
        pack.params,
        Some(AdventureParamsDef {
            native_cost: cost(90),
            land_points: vec![1000, 1100],
            scouting_radius: 3,
            max_map_size: cost(72),
            max_levels: NonZeroU8::new(1).unwrap(),
            max_players: NonZeroU8::new(4).unwrap(),
            army_slots: NonZeroU8::new(5).unwrap(),
            starting_stack_chances: vec![80, 20],
            battle: None,
            monsters: Some(MonsterParamsDef {
                aggression: vec![[-4, -4], [1, 7]],
                count_words: vec![5, 10],
                price_resource: k("p:coin"),
            }),
            fly_cost: [cost(140), cost(140), cost(120), cost(100)],
            water_walk_cost: [cost(150), cost(140), cost(125), cost(110)],
        })
    );
    assert_eq!(
        pack.terrain_walkers,
        [TerrainWalkerDef {
            creature: k("p:nomad"),
            terrain: k("other:sand"),
            counts_as: k("p:dirt"),
        }]
    );
    assert_eq!(
        pack.starting_resources,
        [StartingResourcesDef {
            human: vec![
                (ContentKey::parse("p:coin").unwrap(), 10),
                (ContentKey::parse("other:gem").unwrap(), 2)
            ],
            computer: vec![],
        }]
    );
    assert_eq!(parse("").unwrap(), AdventurePack::new(manifest("p")));
}

#[test]
fn rejects_malformed_files() {
    let syntax = |text: &str| matches!(parse(text), Err(AdventurePackError::Syntax(_)));
    assert!(syntax("[[terrain]]\nterrain = \"a\"\ncost = 1\nspeed = 2"));
    assert!(syntax("[[road]]\nid = \"a\"\nname = \"A\"\ncost = 0"));
    assert!(syntax("[[terrain]]\nterrain = \"a\"\npassage = \"lava\""));
    assert!(syntax(
        "[params]\nnative_cost = 100\nland_points = [1]\nscouting_radius = 300"
    ));
    let bounds = "native_cost = 100\nland_points = [1]\nscouting_radius = 1\nmax_map_size = 1";
    let slots = "max_levels = 1\nmax_players = 1\narmy_slots";
    assert!(syntax(&format!(
        "[params]\n{bounds}\n{slots} = 0\nstarting_stack_chances = []"
    )));
    assert!(syntax(&format!(
        "[params]\n{bounds}\n{slots} = 1\nstarting_stack_chances = [256]"
    )));
    assert!(syntax(&format!("[params]\n{bounds}\n{slots} = 1")));
    assert!(syntax(&format!(
        "[params]\n{bounds}\nmax_levels = 1\nmax_players = 1"
    )));
    assert!(syntax("[unknown]"));
    let cost_error = |text: &str| parse(text).unwrap_err();
    for text in [
        "[[terrain]]\nterrain = \"a\"",
        "[[terrain]]\nterrain = \"a\"\npassage = \"water\"",
        "[[terrain]]\nterrain = \"a\"\npassage = \"impassable\"\ncost = 100",
    ] {
        assert_eq!(
            cost_error(text),
            AdventurePackError::TerrainCost("a".into())
        );
    }
    let key = |text: &str| matches!(parse(text), Err(AdventurePackError::Key(_)));
    assert!(key("[[terrain]]\nterrain = \"Bad:a\"\ncost = 100"));
    assert!(key("[[road]]\nid = \"Bad\"\nname = \"A\"\ncost = 1"));
    assert!(key("[[river]]\nid = \"Bad\"\nname = \"A\""));
    let object = |kind: &str| {
        format!(
            "[[object_type]]\nid = \"o\"\nname = \"O\"\nvisit = \"enter\"\nfrom_above = false\n{kind}"
        )
    };
    assert!(syntax(&object(
        "reveal_area = { radius = 1 }\nvisit_twice = true"
    )));
    assert!(syntax(&object("key_tent = { key = 256 }")));
    assert!(syntax(&object(
        "pickup = { resource = \"a\", amount = [1, 2], multiplier = 0 }"
    )));
    for kind in ["", "key_tent = { key = 0 }\nkey_gate = { key = 0 }"] {
        assert_eq!(
            parse(&object(kind)).unwrap_err(),
            AdventurePackError::ObjectKind("o".into())
        );
    }
    for kind in [
        "pickup = { resource = \"Bad\", amount = [1, 2] }",
        "pickup_one_of = [\"Bad\"]",
        "pickup_paired = { amount = [1, 2], resources = [\"Bad\"], paired = \"a\" }",
        "pickup_paired = { amount = [1, 2], resources = [\"a\"], paired = \"Bad\" }",
        "reveal_terrain = { terrains = [\"Bad\"] }",
        "reveal_terrain = { terrains = [\"a\"], price = { Bad = 1 } }",
    ] {
        assert!(key(&object(kind)), "{kind}");
    }
    assert!(key(
        &object("key_tent = { key = 0 }").replace("id = \"o\"", "id = \"Bad\"")
    ));
    assert!(key("[[difficulty]]\nhuman = { Bad = 1 }\ncomputer = {}"));
    assert!(key("[[difficulty]]\nhuman = {}\ncomputer = { Bad = 1 }"));
    // E08 T40: a terrain walker's three references; its fields, a ground's `no_native` a bool.
    let walker = |c: &str, t: &str, a: &str| {
        format!("[[terrain_walker]]\ncreature = \"{c}\"\nterrain = \"{t}\"\ncounts_as = \"{a}\"")
    };
    assert!(key(&walker("Bad", "a", "b")));
    assert!(key(&walker("c", "Bad", "b")));
    assert!(key(&walker("c", "a", "Bad")));
    assert!(syntax(&format!("{}\nspeed = 1", walker("c", "a", "b"))));
    assert!(syntax(
        "[[terrain_walker]]\ncreature = \"c\"\nterrain = \"a\""
    ));
    assert!(syntax(&object(
        "ground = { battlefield = \"f\", no_native = 1 }"
    )));
    let prices = "fly_cost = [1, 1, 1, 1]\nwater_walk_cost = [1, 1, 1, 1]";
    assert!(syntax(&format!(
        "[params]\n{bounds}\n{slots} = 1\nstarting_stack_chances = []\nfly_cost = [1, 1, 1]"
    )));
    assert!(syntax(&format!(
        "[params]\n{bounds}\n{slots} = 1\nstarting_stack_chances = []\n{prices}\nwater_walk_cost = [0, 1, 1, 1]"
    )));
    let full = format!("[params]\n{bounds}\n{slots} = 1\nstarting_stack_chances = []\n{prices}");
    let battle = |fields: &str| format!("{full}\n[params.battle]\n{fields}");
    let fine = "underground = \"u\"\ncoast = \"c\"\ncoast_battlefield = \"f\"";
    let parsed = parse(&battle(fine))
        .unwrap()
        .params
        .unwrap()
        .battle
        .unwrap();
    assert_eq!(
        (parsed.underground, parsed.coast, parsed.coast_battlefield),
        (key_of("p:u"), key_of("p:c"), key_of("p:f"))
    );
    assert!(syntax(&battle("underground = \"u\"\ncoast = \"c\"")));
    assert!(syntax(&battle(&format!("{fine}\nboats = \"b\""))));
    for field in ["underground", "coast", "coast_battlefield"] {
        let bad = fine.replace(&format!("{field} = \""), &format!("{field} = \"Bad"));
        assert!(key(&battle(&bad)), "{field}");
    }
    assert!(syntax("[[difficulty]]\nhuman = {}"));
    assert!(syntax("[[difficulty]]\nhuman = { a = -1 }\ncomputer = {}"));
}

#[test]
fn compile_reports_each_stage() {
    assert!(matches!(
        compile(&[("[pack]", "")], PROFILE),
        Err(AdventurePackError::Core(PackError::Syntax(_)))
    ));
    assert!(matches!(
        compile(&ORIGINAL_PACKS, "base:unknown"),
        Err(AdventurePackError::Core(PackError::Compile(_)))
    ));
    assert!(matches!(
        compile(&[(h3_pack::BASE_CORE, "[unknown]")], PROFILE),
        Err(AdventurePackError::Syntax(_))
    ));
    assert_eq!(
        compile(&[(h3_pack::BASE_CORE, "")], PROFILE).unwrap_err(),
        AdventurePackError::Compile(AdventureCompileError::MissingTerrain("base:dirt".into()))
    );
}

#[test]
fn errors_explain_themselves() {
    let key = ContentKey::parse("Bad").unwrap_err();
    let cases = [
        (
            AdventurePackError::Syntax("eof".into()),
            "syntax: eof".to_owned(),
        ),
        (AdventurePackError::Key(key.clone()), key.to_string()),
        (
            AdventurePackError::TerrainCost("a".into()),
            "terrain `a`: land and water need a cost, an impassable terrain has none".into(),
        ),
        (
            AdventurePackError::ObjectKind("o".into()),
            "object type `o` needs exactly one kind table".into(),
        ),
        (
            AdventurePackError::Core(PackError::Syntax("eof".into())),
            "core packs: syntax: eof".into(),
        ),
        (
            AdventurePackError::Compile(AdventureCompileError::MissingParams),
            "no pack provides the adventure parameters".into(),
        ),
    ];
    for (error, text) in cases {
        assert_eq!(error.to_string(), text);
        assert!(error.source().is_none());
    }
    assert_eq!(
        AdventurePackError::from(key.clone()),
        AdventurePackError::Key(key)
    );
}
