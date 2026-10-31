//! Tests of the adventure pack format (E08 T4).

use std::error::Error;

use h3_adventure::{
    Chest, Find, FindContent, Gift, Gives, KEY_COLOURS, ObjectKind, Pickup, Pyramid, Turn,
};
use h3_content::{ResourceId, SkillId};

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

// covers: ADV-PIK-014, ADV-PIK-015, ADV-PIK-016, HER-ADV-009, GAM-BAT-001
#[test]
fn the_packs_type_the_sea_s_objects() {
    // E08 T72 (increment 8's design note §5): the sea's parameters, its four pickups, the hole and
    // Favorable Winds.
    let (core, part) = compile(&ORIGINAL_PACKS, PROFILE).unwrap();
    let params = part.params();
    assert_eq!(params.sea_points(), &[1500, 2250, 3000, 3750]);
    assert_eq!(params.sea_ground(), [2, 3]);
    // E08 T83b1: the flag's reveal radius.
    assert_eq!(params.flag_radius(), 3);
    let battle = params.battle_terrains().unwrap();
    assert_eq!(
        (battle.sea, battle.deck.as_str(), battle.boats.as_str()),
        (
            core.terrain_by_key("base:water").unwrap(),
            "base:deck",
            "base:boats"
        )
    );
    let ty = |key: &str| {
        part.object_type(part.object_type_by_key(key).unwrap())
            .unwrap()
    };
    let resource = |key: &str| {
        core.resources()
            .iter()
            .position(|r| r.key.as_str() == key)
            .and_then(ResourceId::from_index)
            .unwrap()
    };
    let (wood, gold) = (resource("base:wood"), resource("base:gold"));
    let fixed = |resources: Vec<(ResourceId, u32)>, artifact| FindContent::Fixed {
        resources,
        artifact,
    };
    for key in [
        "base:flotsam",
        "base:sea_chest",
        "base:shipwreck_survivor",
        "base:ocean_bottle",
    ] {
        assert_eq!(
            (ty(key).visit, ty(key).from_above),
            (Visit::Neighbour, true),
            "{key}"
        );
    }
    assert_eq!(
        ty("base:flotsam").kind,
        ObjectKind::Find(Find {
            draws: vec![
                (25, FindContent::Nothing),
                (25, fixed(vec![(wood, 5)], None)),
                (25, fixed(vec![(wood, 5), (gold, 200)], None)),
                (25, fixed(vec![(wood, 10), (gold, 500)], None)),
            ],
            full_backpack: None,
            question: false,
            bonus: None,
            leaves: true,
        })
    );
    assert_eq!(
        ty("base:sea_chest").kind,
        ObjectKind::Find(Find {
            draws: vec![
                (20, FindContent::Nothing),
                (70, fixed(vec![(gold, 1500)], None)),
                (
                    10,
                    fixed(vec![(gold, 1000)], Some(vec![ArtifactClass::Treasure]))
                ),
            ],
            full_backpack: Some(Turn::Give(vec![(gold, 1500)])),
            question: false,
            bonus: None,
            leaves: true,
        })
    );
    let class = |c| FindContent::Artifact(vec![c]);
    assert_eq!(
        ty("base:shipwreck_survivor").kind,
        ObjectKind::Find(Find {
            draws: vec![
                (55, class(ArtifactClass::Treasure)),
                (20, class(ArtifactClass::Minor)),
                (20, class(ArtifactClass::Major)),
                (5, class(ArtifactClass::Relic)),
            ],
            full_backpack: Some(Turn::Lost),
            question: false,
            bonus: None,
            leaves: true,
        })
    );
    assert_eq!(
        ty("base:ocean_bottle").kind,
        ObjectKind::Message { leaves: true }
    );
    for (key, kind) in [
        ("base:hole", ObjectKind::Hole),
        ("sod:favorable_winds", ObjectKind::SeaGround),
    ] {
        let ty = ty(key);
        assert_eq!(
            (&ty.kind, ty.visit, ty.from_above),
            (&kind, Visit::Enter, true)
        );
    }
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
    // E08 T61a: increment 7's 31 types after them; E08 T72: the sea's four pickups and the hole;
    // E08 T83a: increment 9's thirteen (seven mines, the abandoned mine, the Lighthouse, the
    // Shipyard and the three weekly objects).
    assert_eq!(
        part.object_types().len(),
        7 + 1 + 1 + 3 + 2 + 8 + 8 + 9 + 2 + 31 + 5 + 13
    );
    assert!(part.object_type_by_key("base:border_gate_red").is_none());
}

// covers: ADV-OBJ-002
// covers: ADV-OBJ-002
#[test]
fn the_base_pack_types_the_economy_of_increment_9() {
    // E08 T83a (increment 9's design note §5): the thirteen types, the calendar, the growth and
    // the months as the survey read them; every type entered, none from above (ADV-Q39).
    let (core, part) = compile(&ORIGINAL_PACKS[..1], PROFILE).unwrap();
    let ty = |key: &str| {
        part.object_type(part.object_type_by_key(&format!("base:{key}")).unwrap())
            .unwrap()
    };
    let resource = |key: &str| {
        let index = core
            .resources()
            .iter()
            .position(|r| r.key.as_str() == format!("base:{key}"))
            .unwrap();
        ResourceId::from_index(index).unwrap()
    };
    let subterranean = core.terrain_by_key("base:subterranean");
    let mines = [
        ("sawmill", "wood", 2, None),
        ("alchemists_lab", "mercury", 1, None),
        ("ore_pit", "ore", 2, None),
        ("sulfur_dune", "sulfur", 1, None),
        ("crystal_cavern", "crystal", 1, subterranean),
        ("gem_pond", "gems", 1, None),
        ("gold_mine", "gold", 1000, subterranean),
    ];
    for (key, produced, daily, battle_terrain) in mines {
        assert_eq!(
            ty(key).kind,
            ObjectKind::Mine {
                resource: resource(produced),
                daily,
                battle_terrain,
            },
            "{key}"
        );
        assert_eq!(part.mine_daily(resource(produced)), Some(daily));
    }
    assert_eq!(
        ty("abandoned_mine").kind,
        ObjectKind::AbandonedMine {
            guards: core.creature_by_key("base:troglodyte").unwrap(),
            count: [100, 200],
            battle_terrain: subterranean,
        }
    );
    assert_eq!(
        ty("lighthouse").kind,
        ObjectKind::Lighthouse { sea_points: 500 }
    );
    let ObjectKind::Shipyard {
        price,
        boat_type,
        offsets,
    } = &ty("shipyard").kind
    else {
        panic!("not a Shipyard");
    };
    assert_eq!(
        (price.as_slice(), *boat_type),
        (
            [(resource("wood"), 10), (resource("gold"), 1000)].as_slice(),
            1
        )
    );
    assert_eq!(
        offsets.as_slice(),
        [
            (-2, 0),
            (2, 0),
            (-2, 1),
            (2, 1),
            (-1, 1),
            (1, 1),
            (0, 1),
            (-2, -1),
            (2, -1),
            (-1, -1),
            (1, -1),
            (0, -1)
        ]
    );
    let rare = ["mercury", "ore", "sulfur", "crystal", "gems"].map(resource);
    assert_eq!(
        ty("windmill").kind,
        ObjectKind::Windmill {
            resources: rare.to_vec(),
            amount: [3, 6],
        }
    );
    assert_eq!(
        ty("water_wheel").kind,
        ObjectKind::WaterWheel {
            resource: resource("gold"),
            first: 500,
            later: 1000,
        }
    );
    assert_eq!(
        ty("mystical_garden").kind,
        ObjectKind::Garden {
            gifts: vec![(resource("gems"), 5), (resource("gold"), 500)],
        }
    );
    let keys = mines.map(|(key, ..)| key);
    let others = [
        "abandoned_mine",
        "lighthouse",
        "shipyard",
        "windmill",
        "water_wheel",
        "mystical_garden",
    ];
    for key in keys.iter().chain(&others) {
        assert_eq!(
            (ty(key).visit, ty(key).from_above),
            (Visit::Enter, false),
            "{key}"
        );
        assert_eq!(ty(key).kind.ownable(), !ty(key).kind.weekly(), "{key}");
    }
    let params = part.params();
    assert_eq!(
        (
            params.calendar().week_days.get(),
            params.calendar().month_weeks.get()
        ),
        (7, 4)
    );
    let monsters = params.monsters().unwrap();
    assert_eq!(
        (
            monsters.growth_bp(),
            monsters.growth_unit(),
            monsters.growth_cap()
        ),
        (1000, 16, 4000)
    );
    let months = params.months().unwrap();
    let creature = |key: &str| core.creature_by_key(&format!("base:{key}")).unwrap();
    let twelve = [
        "serpent_fly",
        "griffin",
        "centaur",
        "gremlin",
        "hobgoblin",
        "gog",
        "troglodyte",
        "harpy",
        "lizardman",
        "pegasus",
        "wolf_rider",
        "wight",
    ]
    .map(creature);
    assert_eq!(months.odds, [5, 4, 1]);
    assert_eq!(months.creatures, twelve);
    assert_eq!(
        months.monster,
        part.object_type_by_key("base:monster").unwrap()
    );
    assert_eq!(
        (
            months.spawn_one_in.get(),
            months.spawn_factor.get(),
            months.spawn_aggression
        ),
        (200, 2, [1, 10])
    );
}

#[test]
fn the_base_pack_types_the_rewards_of_increment_7() {
    // E08 T61a (increment 7's design note §8): the classes' kinds and parameters as the survey read
    // them.
    let (core, part) = compile(&ORIGINAL_PACKS[..1], PROFILE).unwrap();
    let ty = |key: &str| {
        part.object_type(part.object_type_by_key(key).unwrap())
            .unwrap()
    };
    let resource = |key: &str| {
        let index = core
            .resources()
            .iter()
            .position(|r| r.key.as_str() == key)
            .unwrap();
        ResourceId::from_index(index).unwrap()
    };
    let skill = |key: &str| core.skill_by_key(&format!("base:{key}")).unwrap();
    let gold = resource("base:gold");
    let neighbour = [
        "treasure_chest",
        "artifact",
        "random_artifact",
        "random_treasure_artifact",
        "random_minor_artifact",
        "random_major_artifact",
        "random_relic_artifact",
        "spell_scroll",
        "scholar",
        "corpse",
        "pandoras_box",
        "event",
    ];
    let entered = [
        "arena",
        "marletto_tower",
        "garden_of_revelation",
        "library_of_enlightenment",
        "mercenary_camp",
        "star_axis",
        "school_of_magic",
        "school_of_war",
        "learning_stone",
        "tree_of_knowledge",
        "shrine_of_magic_incantation",
        "shrine_of_magic_gesture",
        "shrine_of_magic_thought",
        "witch_hut",
        "university",
        "wagon",
        "warriors_tomb",
        "lean_to",
        "pyramid",
    ];
    assert_eq!(neighbour.len() + entered.len(), 31);
    for id in neighbour {
        let t = ty(&format!("base:{id}"));
        assert_eq!((t.visit, t.from_above), (Visit::Neighbour, true), "{id}");
    }
    for id in entered {
        let t = ty(&format!("base:{id}"));
        assert_eq!((t.visit, t.from_above), (Visit::Enter, false), "{id}");
    }
    assert_eq!(
        ty("base:treasure_chest").kind,
        ObjectKind::Chest(Chest {
            resource: gold,
            gold: vec![(32, 1000), (32, 1500), (31, 2000)],
            artifact: vec![ArtifactClass::Treasure],
            full_backpack: 1000,
            experience_offset: 500,
        })
    );
    assert_eq!(
        ty("base:artifact").kind,
        ObjectKind::Artifact { classes: None }
    );
    let classes = [
        (
            "random_artifact",
            &[
                ArtifactClass::Treasure,
                ArtifactClass::Minor,
                ArtifactClass::Major,
            ][..],
        ),
        ("random_treasure_artifact", &[ArtifactClass::Treasure]),
        ("random_minor_artifact", &[ArtifactClass::Minor]),
        ("random_major_artifact", &[ArtifactClass::Major]),
        ("random_relic_artifact", &[ArtifactClass::Relic]),
    ];
    for (id, list) in classes {
        assert_eq!(
            ty(&format!("base:{id}")).kind,
            ObjectKind::Artifact {
                classes: Some(list.to_vec())
            }
        );
    }
    let gift = |gives, prices: Vec<Vec<(ResourceId, u32)>>, admits, leave| {
        ObjectKind::Gift(Gift {
            gives,
            prices,
            admits,
            leave,
        })
    };
    let primary = |gains| gift(Gives::Primary(gains), vec![], None, false);
    assert_eq!(ty("base:marletto_tower").kind, primary([0, 1, 0, 0]));
    assert_eq!(ty("base:garden_of_revelation").kind, primary([0, 0, 0, 1]));
    assert_eq!(ty("base:mercenary_camp").kind, primary([1, 0, 0, 0]));
    assert_eq!(ty("base:star_axis").kind, primary([0, 0, 1, 0]));
    assert_eq!(
        ty("base:library_of_enlightenment").kind,
        gift(Gives::Primary([2; 4]), vec![], Some(10), false)
    );
    let choice = |skills, amount| Gives::Choice { skills, amount };
    use h3_adventure::PrimarySkill::{Attack, Defense, Knowledge, Power};
    assert_eq!(
        ty("base:arena").kind,
        gift(choice([Attack, Defense], 2), vec![], None, false)
    );
    assert_eq!(
        ty("base:school_of_magic").kind,
        gift(
            choice([Power, Knowledge], 1),
            vec![vec![(gold, 1000)]],
            None,
            true
        )
    );
    assert_eq!(
        ty("base:school_of_war").kind,
        gift(
            choice([Attack, Defense], 1),
            vec![vec![(gold, 1000)]],
            None,
            true
        )
    );
    assert_eq!(
        ty("base:learning_stone").kind,
        gift(Gives::Experience(1000), vec![], None, false)
    );
    assert_eq!(
        ty("base:tree_of_knowledge").kind,
        gift(
            Gives::Level,
            vec![
                vec![],
                vec![(gold, 2000)],
                vec![(resource("base:gems"), 10)]
            ],
            None,
            false
        )
    );
    for (level, id) in (1..).zip(["incantation", "gesture", "thought"]) {
        assert_eq!(
            ty(&format!("base:shrine_of_magic_{id}")).kind,
            ObjectKind::Shrine { level }
        );
    }
    // A-21 (`ADV-QUIRK-21`): 0xEFBF is skills 0 to 15 but Leadership (6) and Necromancy (12).
    let ObjectKind::WitchHut { roe, roe_fixed } = &ty("base:witch_hut").kind else {
        panic!("not a Witch Hut");
    };
    let mask: Vec<_> = (0..16_u16)
        .filter(|bit| 0xEFBF_u16 >> bit & 1 == 1)
        .map(|bit| SkillId::from_index(usize::from(bit)).unwrap())
        .collect();
    assert_eq!(*roe, mask);
    let all_but: Vec<_> = (0..28)
        .map(|i| SkillId::from_index(i).unwrap())
        .filter(|&s| s != skill("leadership") && s != skill("necromancy"))
        .collect();
    assert_eq!(*roe_fixed, all_but);
    assert_eq!(ty("base:scholar").kind, ObjectKind::Scholar);
    assert_eq!(
        ty("base:university").kind,
        ObjectKind::University {
            offers: 4,
            price: vec![(gold, 2000)]
        }
    );
    let six = ["wood", "mercury", "ore", "sulfur", "crystal", "gems"]
        .map(|r| resource(&format!("base:{r}")))
        .to_vec();
    let small = vec![ArtifactClass::Treasure, ArtifactClass::Minor];
    assert_eq!(
        ty("base:corpse").kind,
        ObjectKind::Find(Find {
            draws: vec![
                (20, FindContent::Artifact(small.clone())),
                (80, FindContent::Nothing)
            ],
            full_backpack: Some(Turn::Give(vec![(gold, 1000)])),
            question: false,
            bonus: None,
            leaves: false,
        })
    );
    assert_eq!(
        ty("base:wagon").kind,
        ObjectKind::Find(Find {
            draws: vec![
                (10, FindContent::Nothing),
                (40, FindContent::Artifact(small)),
                (
                    50,
                    FindContent::Resources {
                        amount: [2, 5],
                        resources: six.clone()
                    }
                ),
            ],
            full_backpack: Some(Turn::Drawn),
            question: false,
            bonus: None,
            leaves: false,
        })
    );
    let ObjectKind::Find(tomb) = &ty("base:warriors_tomb").kind else {
        panic!("not a find");
    };
    let chances: Vec<_> = tomb.draws.iter().map(|(c, _)| *c).collect();
    assert_eq!(chances, [30, 50, 15, 5]);
    assert_eq!(
        (&tomb.full_backpack, tomb.question, tomb.bonus),
        (
            &Some(Turn::Stays),
            true,
            Some(OnceBonus {
                morale: -3,
                luck: 0,
                mark: 0
            })
        )
    );
    assert_eq!(
        ty("base:lean_to").kind,
        ObjectKind::Find(Find {
            draws: vec![(
                100,
                FindContent::Resources {
                    amount: [1, 5],
                    resources: six
                }
            )],
            full_backpack: None,
            question: false,
            bonus: None,
            leaves: false,
        })
    );
    assert_eq!(ty("base:pandoras_box").kind, ObjectKind::PandoraBox);
    assert_eq!(ty("base:event").kind, ObjectKind::Event);
    let creature = |key: &str| core.creature_by_key(&format!("base:{key}")).unwrap();
    assert_eq!(
        ty("base:pyramid").kind,
        ObjectKind::Pyramid(Pyramid {
            split: (creature("gold_golem"), 40),
            fixed: (creature("diamond_golem"), vec![10, 10]),
            level: 5,
            looted: OnceBonus {
                morale: 0,
                luck: -2,
                mark: 1
            },
        })
    );
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
    // SoD adds its nine special grounds (E08 T20a2a) and Favorable Winds (E08 T72).
    assert_eq!(sod.object_types().len(), ab.object_types().len() + 10);
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
        max_stack_count = 5000
        starting_stack_chances = [80, 20]
        fly_cost = [140, 140, 120, 100]
        water_walk_cost = [150, 140, 125, 110]
        sea_points = [1500, 2250, 3000, 3750]
        sea_ground = [2, 3]
        flag_radius = 3

        [params.monsters]
        aggression = [[-4, -4], [1, 7]]
        count_words = [5, 10]
        price_resource = "coin"
        growth_bp = 1000
        growth_unit = 16
        growth_cap = 4000

        [params.calendar]
        week_days = 7
        month_weeks = 4

        [params.months]
        odds = [5, 4, 1]
        creatures = ["imp", "other:gog"]
        monster = "monster"
        spawn_one_in = 200
        spawn_factor = 2
        spawn_aggression = [1, 10]

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
            max_stack_count: NonZeroU32::new(5000).unwrap(),
            starting_stack_chances: vec![80, 20],
            battle: None,
            monsters: Some(MonsterParamsDef {
                aggression: vec![[-4, -4], [1, 7]],
                count_words: vec![5, 10],
                price_resource: k("p:coin"),
                growth_bp: 1000,
                growth_unit: NonZeroU16::new(16).unwrap(),
                growth_cap: NonZeroU32::new(4000).unwrap(),
            }),
            fly_cost: [cost(140), cost(140), cost(120), cost(100)],
            water_walk_cost: [cost(150), cost(140), cost(125), cost(110)],
            sea_points: [1500, 2250, 3000, 3750],
            sea_ground: [2, 3],
            flag_radius: 3,
            calendar: CalendarParams {
                week_days: NonZeroU8::new(7).unwrap(),
                month_weeks: NonZeroU8::new(4).unwrap(),
            },
            months: Some(MonthParamsDef {
                odds: [5, 4, 1],
                creatures: vec![k("p:imp"), k("other:gog")],
                monster: k("p:monster"),
                spawn_one_in: NonZeroU16::new(200).unwrap(),
                spawn_factor: NonZeroU8::new(2).unwrap(),
                spawn_aggression: [1, 10],
            }),
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

// E08 T83a: increment 9's seven kind tables, their references resolved in the pack's namespace.
#[test]
fn parses_the_economy_kinds() {
    let kind = |table: &str| {
        let text = format!(
            "[[object_type]]\nid = \"o\"\nname = \"O\"\nvisit = \"enter\"\nfrom_above = false\n{table}"
        );
        parse(&text).unwrap().object_types.remove(0).kind
    };
    let k = key_of;
    assert_eq!(
        kind("mine = { resource = \"coin\", daily = 2 }"),
        ObjectKindDef::Mine {
            resource: k("p:coin"),
            daily: 2,
            battle_terrain: None,
        }
    );
    assert_eq!(
        kind("mine = { resource = \"other:gem\", daily = 1, battle_terrain = \"cave\" }"),
        ObjectKindDef::Mine {
            resource: k("other:gem"),
            daily: 1,
            battle_terrain: Some(k("p:cave")),
        }
    );
    assert_eq!(
        kind(
            "abandoned_mine = { guards = \"imp\", count = [100, 200], battle_terrain = \"cave\" }"
        ),
        ObjectKindDef::AbandonedMine {
            guards: k("p:imp"),
            count: [100, 200],
            battle_terrain: Some(k("p:cave")),
        }
    );
    assert_eq!(
        kind("abandoned_mine = { guards = \"imp\", count = [1, 2] }"),
        ObjectKindDef::AbandonedMine {
            guards: k("p:imp"),
            count: [1, 2],
            battle_terrain: None,
        }
    );
    assert_eq!(
        kind("lighthouse = { sea_points = 500 }"),
        ObjectKindDef::Lighthouse { sea_points: 500 }
    );
    assert_eq!(
        kind(
            "shipyard = { price = { coin = 10, \"other:gem\" = 1 }, boat_type = 1, offsets = [[-2, 0], [0, -1]] }"
        ),
        ObjectKindDef::Shipyard {
            price: vec![(k("p:coin"), 10), (k("other:gem"), 1)],
            boat_type: 1,
            offsets: vec![(-2, 0), (0, -1)],
        }
    );
    assert_eq!(
        kind("windmill = { resources = [\"coin\", \"other:gem\"], amount = [3, 6] }"),
        ObjectKindDef::Windmill {
            resources: vec![k("p:coin"), k("other:gem")],
            amount: [3, 6],
        }
    );
    assert_eq!(
        kind("water_wheel = { resource = \"coin\", first = 500, later = 1000 }"),
        ObjectKindDef::WaterWheel {
            resource: k("p:coin"),
            first: 500,
            later: 1000,
        }
    );
    assert_eq!(
        kind(
            "garden = { gifts = [{ resource = \"other:gem\", amount = 5 }, { resource = \"coin\", amount = 500 }] }"
        ),
        ObjectKindDef::Garden {
            gifts: vec![(k("other:gem"), 5), (k("p:coin"), 500)],
        }
    );
}

// E08 T61a: increment 7's kind tables, their references resolved in the pack's namespace.
#[test]
fn parses_the_reward_kinds() {
    let kind = |table: &str| {
        let text = format!(
            "[[object_type]]\nid = \"o\"\nname = \"O\"\nvisit = \"enter\"\nfrom_above = false\n{table}"
        );
        parse(&text).unwrap().object_types.remove(0).kind
    };
    let k = key_of;
    assert_eq!(
        kind(
            "chest = { resource = \"coin\", gold = [[90, 10]], artifact = [\"relic\", \"special\"], full_backpack = 7, experience_offset = 5 }"
        ),
        ObjectKindDef::Chest(ChestDef {
            resource: k("p:coin"),
            gold: vec![(90, 10)],
            artifact: vec![ArtifactClass::Relic, ArtifactClass::Special],
            full_backpack: 7,
            experience_offset: 5,
        })
    );
    assert_eq!(
        kind(
            "chest = { resource = \"coin\", gold = [[100, 10]], full_backpack = 7, experience_offset = 5 }"
        ),
        ObjectKindDef::Chest(ChestDef {
            resource: k("p:coin"),
            gold: vec![(100, 10)],
            artifact: vec![],
            full_backpack: 7,
            experience_offset: 5,
        })
    );
    assert_eq!(
        kind("artifact = {}"),
        ObjectKindDef::Artifact { classes: None }
    );
    assert_eq!(
        kind("artifact = { classes = [\"treasure\", \"minor\", \"major\"] }"),
        ObjectKindDef::Artifact {
            classes: Some(vec![
                ArtifactClass::Treasure,
                ArtifactClass::Minor,
                ArtifactClass::Major
            ])
        }
    );
    assert_eq!(kind("scroll = {}"), ObjectKindDef::Scroll);
    assert_eq!(
        kind("gift = { gives = { primary = [1, 2, 3, 4] } }"),
        ObjectKindDef::Gift(GiftDef {
            gives: Gives::Primary([1, 2, 3, 4]),
            prices: vec![],
            admits: None,
            leave: false,
        })
    );
    assert_eq!(
        kind(
            "gift = { gives = { choice = { skills = [\"power\", \"knowledge\"], amount = 3 } }, prices = [{}, { coin = 2, \"other:gem\" = 1 }], admits = 9, leave = true }"
        ),
        ObjectKindDef::Gift(GiftDef {
            gives: Gives::Choice {
                skills: [PrimarySkill::Power, PrimarySkill::Knowledge],
                amount: 3
            },
            prices: vec![vec![], vec![(k("p:coin"), 2), (k("other:gem"), 1)]],
            admits: Some(9),
            leave: true,
        })
    );
    let gives = |table: &str| match kind(&format!("gift = {{ gives = {table} }}")) {
        ObjectKindDef::Gift(gift) => gift.gives,
        other => panic!("{other:?}"),
    };
    assert_eq!(gives("{ experience = 12 }"), Gives::Experience(12));
    assert_eq!(gives("\"level\""), Gives::Level);
    assert_eq!(
        gives("{ choice = { skills = [\"attack\", \"defense\"], amount = 1 } }"),
        Gives::Choice {
            skills: [PrimarySkill::Attack, PrimarySkill::Defense],
            amount: 1
        }
    );
    assert_eq!(
        kind("shrine = { level = 2 }"),
        ObjectKindDef::Shrine { level: 2 }
    );
    assert_eq!(
        kind("witch_hut = { roe = [\"a\"], roe_fixed = [\"a\", \"other:b\"] }"),
        ObjectKindDef::WitchHut {
            roe: vec![k("p:a")],
            roe_fixed: vec![k("p:a"), k("other:b")],
        }
    );
    assert_eq!(kind("scholar = {}"), ObjectKindDef::Scholar);
    assert_eq!(
        kind("university = { offers = 4, price = { coin = 20 } }"),
        ObjectKindDef::University {
            offers: 4,
            price: vec![(k("p:coin"), 20)],
        }
    );
    assert_eq!(
        kind(
            "find = { draws = [{ chance = 10, holds = \"nothing\" }, { chance = 40, holds = { artifact = [\"minor\"] } }, { chance = 50, holds = { resources = { amount = [2, 5], resources = [\"wood\"] } } }], full_backpack = { give = { coin = 3 } }, question = true, bonus = { morale = -3, luck = 1, mark = 0 } }"
        ),
        ObjectKindDef::Find(FindDef {
            draws: vec![
                (10, FindContentDef::Nothing),
                (40, FindContentDef::Artifact(vec![ArtifactClass::Minor])),
                (
                    50,
                    FindContentDef::Resources {
                        amount: [2, 5],
                        resources: vec![k("p:wood")],
                    }
                ),
            ],
            full_backpack: Some(TurnDef::Give(vec![(k("p:coin"), 3)])),
            question: true,
            bonus: Some(OnceBonus {
                morale: -3,
                luck: 1,
                mark: 0
            }),
            leaves: false,
        })
    );
    // E08 T72: a fixed entry with and without an artifact, the lost turn, leaving.
    assert_eq!(
        kind(
            "find = { draws = [{ chance = 90, holds = { fixed = { resources = { coin = 15 } } } }, { chance = 10, holds = { fixed = { resources = { coin = 10, wood = 1 }, artifact = [\"treasure\"] } } }], full_backpack = \"lost\", leaves = true }"
        ),
        ObjectKindDef::Find(FindDef {
            draws: vec![
                (
                    90,
                    FindContentDef::Fixed {
                        resources: vec![(k("p:coin"), 15)],
                        artifact: None,
                    }
                ),
                (
                    10,
                    FindContentDef::Fixed {
                        resources: vec![(k("p:coin"), 10), (k("p:wood"), 1)],
                        artifact: Some(vec![ArtifactClass::Treasure]),
                    }
                ),
            ],
            full_backpack: Some(TurnDef::Lost),
            question: false,
            bonus: None,
            leaves: true,
        })
    );
    assert_eq!(
        kind("message = { leaves = true }"),
        ObjectKindDef::Message { leaves: true }
    );
    assert_eq!(
        kind("message = {}"),
        ObjectKindDef::Message { leaves: false }
    );
    assert_eq!(kind("sea_ground = {}"), ObjectKindDef::SeaGround);
    assert_eq!(kind("hole = {}"), ObjectKindDef::Hole);
    let turn = |table: &str| match kind(&format!(
        "find = {{ draws = [{{ chance = 100, holds = \"nothing\" }}]{table} }}"
    )) {
        ObjectKindDef::Find(find) => (find.full_backpack, find.question, find.bonus),
        other => panic!("{other:?}"),
    };
    assert_eq!(turn(""), (None, false, None));
    assert_eq!(turn(", full_backpack = \"drawn\"").0, Some(TurnDef::Drawn));
    assert_eq!(turn(", full_backpack = \"stays\"").0, Some(TurnDef::Stays));
    assert_eq!(kind("pandora_box = {}"), ObjectKindDef::PandoraBox);
    assert_eq!(kind("event = {}"), ObjectKindDef::Event);
    assert_eq!(
        kind(
            "pyramid = { split = { creature = \"golem\", count = 40 }, fixed = { creature = \"other:gem_golem\", stacks = [10, 10] }, level = 5, looted = { luck = -2, mark = 1 } }"
        ),
        ObjectKindDef::Pyramid(PyramidDef {
            split: (k("p:golem"), 40),
            fixed: (k("other:gem_golem"), vec![10, 10]),
            level: 5,
            looted: OnceBonus {
                morale: 0,
                luck: -2,
                mark: 1
            },
        })
    );
    // Malformed tables: unknown fields, an unknown class or skill name, two ways to give.
    let syntax = |table: &str| {
        let text = format!(
            "[[object_type]]\nid = \"o\"\nname = \"O\"\nvisit = \"enter\"\nfrom_above = false\n{table}"
        );
        matches!(parse(&text), Err(AdventurePackError::Syntax(_)))
    };
    for table in [
        "scroll = { spell = 1 }",
        "artifact = { classes = [\"legendary\"] }",
        "gift = { gives = { primary = [1, 0, 0, 0], experience = 5 } }",
        "gift = { gives = { choice = { skills = [\"luck\", \"attack\"], amount = 1 } } }",
        "gift = { gives = \"levels\" }",
        "find = { draws = [{ chance = 100, holds = \"everything\" }] }",
        "find = { draws = [], full_backpack = \"gone\" }",
        "find = { draws = [{ chance = 100, holds = { fixed = { resources = {}, artifacts = [] } } }] }",
        "message = { text = \"Ahoy\" }",
        "hole = { depth = 1 }",
        "find = { draws = [], bonus = { morale = 1 } }",
        "pyramid = { split = { creature = \"g\", count = 1 }, fixed = { creature = \"g\", stacks = [1] }, level = 1, looted = { mark = 1 }, extra = 1 }",
    ] {
        assert!(syntax(table), "{table}");
    }
    // A bad key in every reference.
    let key = |table: &str| {
        let text = format!(
            "[[object_type]]\nid = \"o\"\nname = \"O\"\nvisit = \"enter\"\nfrom_above = false\n{table}"
        );
        matches!(parse(&text), Err(AdventurePackError::Key(_)))
    };
    for table in [
        "chest = { resource = \"Bad\", gold = [[1, 1]], full_backpack = 1, experience_offset = 0 }",
        "gift = { gives = \"level\", prices = [{ Bad = 1 }] }",
        "witch_hut = { roe = [\"Bad\"], roe_fixed = [] }",
        "witch_hut = { roe = [], roe_fixed = [\"Bad\"] }",
        "university = { offers = 1, price = { Bad = 1 } }",
        "find = { draws = [{ chance = 100, holds = { resources = { amount = [1, 1], resources = [\"Bad\"] } } }] }",
        "find = { draws = [], full_backpack = { give = { Bad = 1 } } }",
        "find = { draws = [{ chance = 100, holds = { fixed = { resources = { Bad = 1 } } } }] }",
        "pyramid = { split = { creature = \"Bad\", count = 1 }, fixed = { creature = \"g\", stacks = [1] }, level = 1, looted = { mark = 1 } }",
        "pyramid = { split = { creature = \"g\", count = 1 }, fixed = { creature = \"Bad\", stacks = [1] }, level = 1, looted = { mark = 1 } }",
    ] {
        assert!(key(table), "{table}");
    }
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
    let bounds = "native_cost = 100\nland_points = [1]\nscouting_radius = 1\nmax_map_size = 1\n\
                  max_stack_count = 4095";
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
    let prices = "fly_cost = [1, 1, 1, 1]\nwater_walk_cost = [1, 1, 1, 1]\nsea_points = [1, 1, 1, 1]\nsea_ground = [1, 1]\nflag_radius = 1\ncalendar = { week_days = 7, month_weeks = 4 }";
    assert!(syntax(&format!(
        "[params]\n{bounds}\n{slots} = 1\nstarting_stack_chances = []\nfly_cost = [1, 1, 1]"
    )));
    assert!(syntax(&format!(
        "[params]\n{bounds}\n{slots} = 1\nstarting_stack_chances = []\n{prices}\nwater_walk_cost = [0, 1, 1, 1]"
    )));
    let full = format!("[params]\n{bounds}\n{slots} = 1\nstarting_stack_chances = []\n{prices}");
    // E08 T50: a stack's most creatures is above 0 (ADV-ARM-007).
    assert!(syntax(
        &full.replace("max_stack_count = 4095", "max_stack_count = 0")
    ));
    // E08 T72: four sea points, two parts of the sea ground's share, all needed.
    assert!(syntax(
        &full.replace("sea_points = [1, 1, 1, 1]", "sea_points = [1, 1, 1]")
    ));
    assert!(syntax(
        &full.replace("sea_ground = [1, 1]", "sea_ground = [1]")
    ));
    assert!(syntax(&full.replace("\nsea_ground = [1, 1]", "")));
    // E08 T83b1: the flag radius is needed and fits a byte.
    assert!(syntax(&full.replace("\nflag_radius = 1", "")));
    assert!(syntax(
        &full.replace("flag_radius = 1", "flag_radius = 256")
    ));
    let battle = |fields: &str| format!("{full}\n[params.battle]\n{fields}");
    let fine = "underground = \"u\"\ncoast = \"c\"\ncoast_battlefield = \"f\"\nsea = \"w\"\ndeck = \"d\"\nboats = \"b\"";
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
    assert_eq!(
        (parsed.sea, parsed.deck, parsed.boats),
        (key_of("p:w"), key_of("p:d"), key_of("p:b"))
    );
    assert!(syntax(&battle("underground = \"u\"\ncoast = \"c\"")));
    assert!(syntax(&battle(&format!("{fine}\nships = \"s\""))));
    for field in [
        "underground",
        "coast",
        "coast_battlefield",
        "sea",
        "deck",
        "boats",
    ] {
        let bad = fine.replace(&format!("{field} = \""), &format!("{field} = \"Bad"));
        assert!(key(&battle(&bad)), "{field}");
    }
    assert!(syntax("[[difficulty]]\nhuman = {}"));
    assert!(syntax("[[difficulty]]\nhuman = { a = -1 }\ncomputer = {}"));
    // E08 T83a: the calendar required, its counts above 0; the months' fields and references;
    // the growth's fields with the monsters'; increment 9's kinds' references and fields.
    assert!(syntax(
        &full.replace("\ncalendar = { week_days = 7, month_weeks = 4 }", "")
    ));
    assert!(syntax(&full.replace("week_days = 7", "week_days = 0")));
    assert!(syntax(&full.replace("month_weeks = 4", "month_weeks = 0")));
    let months = |fields: &str| format!("{full}\n[params.months]\n{fields}");
    let fine = "odds = [5, 4, 1]\ncreatures = [\"c\"]\nmonster = \"m\"\nspawn_one_in = 200\nspawn_factor = 2\nspawn_aggression = [1, 10]";
    assert!(
        parse(&months(fine))
            .unwrap()
            .params
            .unwrap()
            .months
            .is_some()
    );
    assert!(syntax(&months(
        &fine.replace("spawn_one_in = 200", "spawn_one_in = 0")
    )));
    assert!(syntax(&months(
        &fine.replace("spawn_factor = 2", "spawn_factor = 0")
    )));
    assert!(syntax(&months(
        &fine.replace("odds = [5, 4, 1]", "odds = [5, 4]")
    )));
    assert!(syntax(&months("odds = [5, 4, 1]")));
    assert!(key(&months(&fine.replace("[\"c\"]", "[\"Bad\"]"))));
    assert!(key(&months(&fine.replace("\"m\"", "\"Bad\""))));
    let growth = "aggression = []\ncount_words = []\nprice_resource = \"g\"\ngrowth_bp = 1000\ngrowth_unit = 16\ngrowth_cap = 4000";
    let monsters = |fields: &str| format!("{full}\n[params.monsters]\n{fields}");
    assert!(
        parse(&monsters(growth))
            .unwrap()
            .params
            .unwrap()
            .monsters
            .is_some()
    );
    assert!(syntax(&monsters(&growth.replace("\ngrowth_bp = 1000", ""))));
    assert!(syntax(&monsters(
        &growth.replace("growth_unit = 16", "growth_unit = 0")
    )));
    assert!(syntax(&monsters(
        &growth.replace("growth_cap = 4000", "growth_cap = 0")
    )));
    for kind in [
        "mine = { resource = \"Bad\", daily = 1 }",
        "mine = { resource = \"a\", daily = 1, battle_terrain = \"Bad\" }",
        "abandoned_mine = { guards = \"Bad\", count = [1, 2] }",
        "abandoned_mine = { guards = \"a\", count = [1, 2], battle_terrain = \"Bad\" }",
        "shipyard = { price = { Bad = 1 }, boat_type = 1, offsets = [] }",
        "windmill = { resources = [\"Bad\"], amount = [1, 2] }",
        "water_wheel = { resource = \"Bad\", first = 1, later = 1 }",
        "garden = { gifts = [{ resource = \"Bad\", amount = 1 }] }",
    ] {
        assert!(key(&object(kind)), "{kind}");
    }
    for kind in [
        "mine = { resource = \"a\" }",
        "lighthouse = { sea_points = -1 }",
        "shipyard = { price = {}, boat_type = 256, offsets = [] }",
        "shipyard = { price = {}, boat_type = 1, offsets = [[1]] }",
        "shipyard = { price = {}, boat_type = 1, offsets = [[128, 0]] }",
        "windmill = { resources = [], amount = [1] }",
        "garden = { gifts = [{ resource = \"a\" }] }",
    ] {
        assert!(syntax(&object(kind)), "{kind}");
    }
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
