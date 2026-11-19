use super::*;
use alloc::collections::BTreeMap;

use h3_map::Limits;
use h3_map::writer::{self, AnyObject, MapObject, ObjectTemplate, TownBuildings};

use crate::art::MapArt;

const PLAIN: &str = r#"
size = 36
terrain = "grass"
heroes = [{ at = [34, 34], class = 2 }, { at = [34, 1], owner = 1 }]
view = [0, 0, 0]
"#;

/// A shipped map's stand-in: the writer's 36 × 36 dirt map with a lava patch at (2, 2)-(3, 3), a
/// cobblestone road on row 5 and a muddy river on (7, 7), picture 4, mirrored.
fn source() -> RawMap {
    let mut map = Situation::parse(PLAIN)
        .unwrap()
        .build(&BTreeMap::new(), &[])
        .unwrap()
        .0;
    map.terrain = Terrain::Dirt;
    map.patches = vec![Patch {
        terrain: Terrain::Lava,
        from: (2, 2),
        to: (3, 3),
    }];
    map.roads = vec![Road {
        kind: RoadKind::Cobblestone,
        row: 5,
        from: 1,
        to: 4,
    }];
    let river = GivenTile {
        level: 0,
        tile: (7, 7),
        terrain: Terrain::Dirt,
        picture: 3,
        river: Some((RiverKind::Muddy, 4)),
        road: None,
        flags: 0x0C,
    };
    let extras = Extras {
        tiles: vec![river],
        ..Extras::default()
    };
    let bytes = writer::write_with(&map, &extras).unwrap();
    h3_map::parse(&bytes, &Limits::default()).unwrap()
}

fn sources() -> BTreeMap<String, RawMap> {
    BTreeMap::from([("Shipped.h3m".to_string(), source())])
}

/// The art of a situation's map.
fn art(text: &str) -> MapArt {
    let situation = Situation::parse(text).unwrap();
    let (map, extras) = situation.build(&sources(), &[]).unwrap();
    let bytes = writer::write_with(&map, &extras).unwrap();
    MapArt::from_raw(&h3_map::parse(&bytes, &Limits::default()).unwrap()).unwrap()
}

#[test]
fn a_plain_map_with_two_heroes() {
    let situation = Situation::parse(PLAIN).unwrap();
    assert!(situation.sources().is_empty());
    assert!(situation.hidden.is_empty());
    assert_eq!(situation.explored, None);
    // The start's explored tiles kept (E24 M4 T3a); no other choice.
    let start = Situation::parse(&format!("{PLAIN}explored = \"start\"\n")).unwrap();
    assert_eq!(start.explored, Some(Explored::Start));
    assert!(Situation::parse(&format!("{PLAIN}explored = \"all\"\n")).is_err());
    let (map, extras) = situation.build(&BTreeMap::new(), &[]).unwrap();
    assert_eq!((map.size, map.terrain), (36, Terrain::Grass));
    assert!(
        map.players[..2]
            .iter()
            .all(|p| p.as_ref().is_some_and(|p| p.human && p.computer))
    );
    assert!(map.players[2..].iter().all(Option::is_none));
    let heroes: Vec<(u8, (u8, u8))> = map
        .objects
        .iter()
        .filter_map(|o| match o {
            MapObject::Hero(h) => Some((h.owner, h.tile)),
            _ => None,
        })
        .collect();
    assert_eq!(heroes, [(0, (34, 34)), (1, (34, 1))]);
    assert_eq!(extras, Extras::default());
    let art = art(PLAIN);
    assert_eq!(art.tile(5, 5, 0).unwrap().terrain.code, 2);
}

#[test]
fn an_underground_of_terrains() {
    // E24 M3 T6b2 (design §4.14): the writer's second level, its terrain and patches, no objects.
    let text = r#"
size = 36
terrain = "grass"
underground = { terrain = "subterranean", patches = [{ terrain = "lava", from = [2, 2], to = [3, 3] }] }
heroes = [{ at = [34, 34], class = 2 }, { at = [34, 1], owner = 1 }]
view = [0, 0, 0]
"#;
    let (map, _) = Situation::parse(text)
        .unwrap()
        .build(&BTreeMap::new(), &[])
        .unwrap();
    let under = map.underground.unwrap();
    assert_eq!(
        (under.terrain, under.patches.len()),
        (Terrain::Subterranean, 1)
    );
    assert!(under.objects.is_empty());
    let art = art(text);
    assert_eq!(art.levels, 2);
    assert_eq!(art.tile(2, 3, 1).unwrap().terrain.code, 7);
    assert_eq!(art.tile(5, 5, 1).unwrap().terrain.code, 6);
    assert_eq!(art.tile(5, 5, 0).unwrap().terrain.code, 2);
    // Without one, one level.
    assert_eq!(self::art(PLAIN).levels, 1);
}

#[test]
fn patches_roads_copies_and_tiles_in_order() {
    let drawn = art(r#"
size = 36
terrain = "sand"
heroes = [{ at = [34, 34], class = 2 }, { at = [34, 1], owner = 1 }]
view = [0, 0, 0]
hidden = [[2, 3, 4, 5]]
patches = [{ terrain = "snow", from = [20, 20], to = [22, 22] }]
roads = [{ kind = "gravel", row = 30, from = 2, to = 5 }]
copy = [{ map = "Shipped.h3m", from = [2, 2], to = [7, 7], at = [10, 10] }]
tiles = [
    { at = [12, 12], terrain = "swamp", picture = 9, flags = 2 },
    { at = [1, 1], terrain = "water", picture = 20, river = ["icy", 2], road = ["dirt", 6] },
]
"#);
    assert_eq!(drawn.tile(21, 21, 0).unwrap().terrain.code, 3);
    assert_eq!(drawn.tile(3, 30, 0).unwrap().road.unwrap().code, 2);
    // The copy: the lava patch at (10, 10)-(11, 11), the road's row 5 at row 13, the river at
    // (15, 15) with its picture and mirrors; the given swamp tile over the copy.
    assert_eq!(drawn.tile(11, 11, 0).unwrap().terrain.code, 7);
    let road = drawn.tile(10, 13, 0).unwrap().road.unwrap();
    assert_eq!(road.code, 3);
    let river = drawn.tile(15, 15, 0).unwrap().river.unwrap();
    assert_eq!(
        (river.code, river.picture, river.mirror_x, river.mirror_y),
        (3, 4, true, true)
    );
    let swamp = drawn.tile(12, 12, 0).unwrap().terrain;
    assert_eq!((swamp.code, swamp.picture, swamp.mirror_y), (4, 9, true));
    let one = drawn.tile(1, 1, 0).unwrap();
    assert_eq!(
        (
            one.terrain.code,
            one.river.unwrap().code,
            one.road.unwrap().picture
        ),
        (8, 2, 6)
    );
    // Every name of each kind.
    for (name, code) in [
        ("dirt", 0),
        ("sand", 1),
        ("grass", 2),
        ("snow", 3),
        ("swamp", 4),
        ("rough", 5),
        ("subterranean", 6),
        ("lava", 7),
        ("water", 8),
        ("rock", 9),
    ] {
        let text = alloc::format!(
            "{PLAIN}tiles = [{{ at = [0, 0], terrain = \"{name}\", picture = 0, river = [\"clear\", 1], road = [\"cobblestone\", 1] }}]"
        );
        assert_eq!(
            art(&text).tile(0, 0, 0).unwrap().terrain.code,
            code,
            "{name}"
        );
    }
    for (name, code) in [("clear", 1), ("icy", 2), ("muddy", 3), ("lava", 4)] {
        let text = alloc::format!(
            "{PLAIN}tiles = [{{ at = [0, 0], terrain = \"dirt\", picture = 0, river = [\"{name}\", 1] }}]"
        );
        assert_eq!(art(&text).tile(0, 0, 0).unwrap().river.unwrap().code, code);
    }
    for (name, code) in [("dirt", 1), ("gravel", 2), ("cobblestone", 3)] {
        let text = alloc::format!(
            "{PLAIN}tiles = [{{ at = [0, 0], terrain = \"dirt\", picture = 0, road = [\"{name}\", 1] }}]"
        );
        assert_eq!(art(&text).tile(0, 0, 0).unwrap().road.unwrap().code, code);
    }
}

#[test]
fn the_sources_and_the_hidden_rectangles() {
    let situation = Situation::parse(&alloc::format!(
        "{PLAIN}hidden = [[1, 2, 3, 4], [5, 5, 5, 5]]\ncopy = [{{ map = \"A.h3m\", from = [0, 0], to = [1, 1], at = [0, 0] }}, {{ map = \"B.h3m\", from = [0, 0], to = [0, 0], at = [5, 5], level = 1 }}]"
    ))
    .unwrap();
    let names: Vec<String> = situation.sources().into_iter().collect();
    assert_eq!(names, ["A.h3m", "B.h3m"]);
    assert_eq!(situation.hidden, [[1, 2, 3, 4], [5, 5, 5, 5]]);
    assert_eq!(
        situation.build(&BTreeMap::new(), &[]).unwrap_err(),
        "A.h3m: not read"
    );
}

#[test]
fn refuses_what_it_cannot_build() {
    let with = |copy: &str| {
        let text = alloc::format!("{PLAIN}copy = [{copy}]");
        Situation::parse(&text).unwrap().build(&sources(), &[])
    };
    // Outside the source, reversed, past the situation's side, on a level the source lacks.
    for copy in [
        r#"{ map = "Shipped.h3m", from = [30, 30], to = [36, 31], at = [0, 0] }"#,
        r#"{ map = "Shipped.h3m", from = [5, 5], to = [4, 6], at = [0, 0] }"#,
        r#"{ map = "Shipped.h3m", from = [0, 0], to = [3, 3], at = [33, 0] }"#,
        r#"{ map = "Shipped.h3m", from = [0, 0], to = [3, 3], at = [0, 33] }"#,
        r#"{ map = "Shipped.h3m", from = [0, 0], to = [0, 0], at = [0, 0], level = 1 }"#,
    ] {
        assert!(with(copy).unwrap_err().contains("does not fit"), "{copy}");
    }
    // A source's code with no name, and a source whose tiles do not fill it.
    let bad = |edit: fn(&mut RawMap)| {
        let mut raw = source();
        edit(&mut raw);
        let situation = Situation::parse(&alloc::format!(
            "{PLAIN}copy = [{{ map = \"Shipped.h3m\", from = [0, 0], to = [0, 0], at = [0, 0] }}]"
        ))
        .unwrap();
        situation
            .build(&BTreeMap::from([("Shipped.h3m".to_string(), raw)]), &[])
            .unwrap_err()
    };
    assert_eq!(
        bad(|r| r.terrain.tiles[0].terrain = 10),
        "Shipped.h3m: (0, 0): terrain code 10"
    );
    assert_eq!(
        bad(|r| r.terrain.tiles[0].river = 5),
        "Shipped.h3m: (0, 0): river code 5"
    );
    assert_eq!(
        bad(|r| r.terrain.tiles[0].road = 4),
        "Shipped.h3m: (0, 0): road code 4"
    );
    assert_eq!(
        bad(|r| r.terrain.tiles.clear()),
        "Shipped.h3m: (0, 0): no tile code 0"
    );
    // The view on the surface only; the parser's own errors.
    assert_eq!(
        Situation::parse(&PLAIN.replace("view = [0, 0, 0]", "view = [0, 0, 1]")).unwrap_err(),
        "a situation's view is on the surface (level 0)"
    );
    // The view's corner within the map's side of the map (M2 T6a, the reviewer's R1-8).
    for (view, ok) in [
        ("[-36, 36, 0]", true),
        ("[-37, 0, 0]", false),
        ("[0, 37, 0]", false),
    ] {
        let parsed = Situation::parse(&PLAIN.replace("[0, 0, 0]", view));
        assert_eq!(parsed.is_ok(), ok, "{view}");
    }
    assert_eq!(
        Situation::parse(&PLAIN.replace("view = [0, 0, 0]", "view = [2147483640, 0, 0]"))
            .unwrap_err(),
        "the view's corner [2147483640, 0] lies more than the map's side (36) off the map"
    );
    assert!(Situation::parse("size = 36").is_err());
}

/// The source with objects after its two heroes: a tree at (3, 3), a pile at (4, 4).
fn source_with_objects() -> RawMap {
    let mut map = Situation::parse(PLAIN)
        .unwrap()
        .build(&BTreeMap::new(), &[])
        .unwrap()
        .0;
    map.terrain = Terrain::Dirt;
    let object = |def: &str, class, tile| AnyObject {
        level: 0,
        tile,
        template: crate::art::tests::template(def, class, 0, 0),
        owner: None,
    };
    let extras = Extras {
        tiles: vec![],
        objects: vec![
            object("tree.def", 155, (3, 3)),
            object("pile.def", 79, (4, 4)),
        ],
        ..Extras::default()
    };
    let bytes = writer::write_with(&map, &extras).unwrap();
    h3_map::parse(&bytes, &Limits::default()).unwrap()
}

#[test]
fn a_copy_brings_its_objects_but_heroes() {
    // The rectangle (2, 0)-(35, 4) holds Blue's hero at (34, 1), the tree at (3, 3) and the pile
    // at (4, 4); the pile's template index is broken, so it is left out with the hero.
    let mut raw = source_with_objects();
    raw.objects[3].template = 99;
    let text = r#"
size = 36
terrain = "grass"
copy = [{ map = "Shipped.h3m", from = [2, 0], to = [35, 4], at = [0, 10], objects = true }]
heroes = [{ at = [34, 34], class = 2 }, { at = [34, 1], owner = 1 }]
view = [0, 0, 0]
"#;
    let situation = Situation::parse(text).unwrap();
    let (_, extras) = situation
        .build(&BTreeMap::from([("Shipped.h3m".to_string(), raw)]), &[])
        .unwrap();
    assert_eq!(extras.objects.len(), 1);
    let tree = &extras.objects[0];
    assert_eq!(
        (tree.tile, tree.template.def.as_str()),
        ((1, 13), "tree.def")
    );
    assert_eq!((tree.template.class, tree.owner), (155, None));
    // Without `objects` a copy brings none; a copy elsewhere none either.
    let text = text.replace(", objects = true", "");
    let sources = BTreeMap::from([("Shipped.h3m".to_string(), source_with_objects())]);
    let (_, extras) = Situation::parse(&text)
        .unwrap()
        .build(&sources, &[])
        .unwrap();
    assert!(extras.objects.is_empty());
    let text = text.replace("at = [0, 10] }", "at = [0, 10], objects = true }");
    let text = text.replace("from = [2, 0]", "from = [5, 0]");
    let (_, extras) = Situation::parse(&text)
        .unwrap()
        .build(&sources, &[])
        .unwrap();
    assert!(extras.objects.is_empty());
}

#[test]
fn objects_given_by_their_templates_def() {
    let line = |def: &str, class, subclass| {
        ObjectTemplate::from_objects_txt(&format!(
            "{def} {} {} 111111111 111111111 {class} {subclass} 0 0",
            "1".repeat(48),
            "0".repeat(48)
        ))
        .unwrap()
    };
    let templates = [
        line("A.def", 1, 0),
        line("a.def", 2, 0),
        line("a.def", 2, 5),
    ];
    let text = |objects: &str| {
        format!(
            "size = 36\nterrain = \"grass\"\nheroes = [{{ at = [34, 34], class = 2 }}, {{ at = [34, 1], owner = 1 }}]\nview = [0, 0, 0]\n{objects}"
        )
    };
    let given = text(
        r#"objects = [
    { def = "a.DEF", at = [3, 4] },
    { def = "a.def", class = 2, at = [5, 6] },
    { def = "a.def", subclass = 5, at = [40, 39] },
]"#,
    );
    let situation = Situation::parse(&given).unwrap();
    assert!(situation.needs_templates());
    assert!(!Situation::parse(PLAIN).unwrap().needs_templates());
    let (_, extras) = situation.build(&BTreeMap::new(), &templates).unwrap();
    let seen: Vec<_> = extras
        .objects
        .iter()
        .map(|o| (o.tile, o.template.class, o.template.subclass))
        .collect();
    assert_eq!(seen, [((3, 4), 1, 0), ((5, 6), 2, 0), ((40, 39), 2, 5)]);
    let missing = text(r#"objects = [{ def = "a.def", class = 3, at = [1, 1] }]"#);
    assert_eq!(
        Situation::parse(&missing)
            .unwrap()
            .build(&BTreeMap::new(), &templates)
            .unwrap_err(),
        "a.def: no such template in Objects.txt"
    );
}

/// E24 M3 T6a2b: a Seer's Hut's and a Quest Guard's quests by the writer's missions, a hero's and
/// a monster's by their identifiers (the heroes first, then the extras' objects); a hero or a
/// monster missing.
#[test]
fn quests_of_seers_and_guards() {
    let line = |def: &str, class| {
        ObjectTemplate::from_objects_txt(&format!(
            "{def} {} {} 111111111 111111111 {class} 0 0 0",
            "1".repeat(48),
            "0".repeat(48)
        ))
        .unwrap()
    };
    let templates = [
        line("seer.def", 83),
        line("guard.def", 215),
        line("pike.def", 54),
    ];
    let text = |objects: &str| {
        format!(
            "size = 36\nterrain = \"grass\"\nheroes = [{{ at = [34, 34], class = 2 }}, {{ at = [34, 1], owner = 1 }}]\nview = [0, 0, 0]\n{objects}"
        )
    };
    let given = text(
        r#"objects = [
    { def = "seer.def", at = [2, 2], quest = { level = 10 } },
    { def = "seer.def", at = [4, 2], quest = { skills = [1, 2, 0, 3] } },
    { def = "seer.def", at = [6, 2], quest = { hero = [34, 1] } },
    { def = "seer.def", at = [8, 2], quest = { monster = [30, 30] } },
    { def = "seer.def", at = [10, 2], quest = { artifacts = [7, 8] } },
    { def = "guard.def", at = [2, 4], quest = { creatures = [[0, 10], [13, 1]] } },
    { def = "guard.def", at = [4, 4], quest = { resources = [0, 0, 0, 0, 0, 5, 1000] } },
    { def = "guard.def", at = [6, 4], quest = { be_hero = 3 } },
    { def = "guard.def", at = [8, 4], quest = { player = 1 } },
    { def = "guard.def", at = [10, 4] },
    { def = "pike.def", at = [30, 30] },
]"#,
    );
    let (map, extras) = Situation::parse(&given)
        .unwrap()
        .build(&BTreeMap::new(), &templates)
        .unwrap();
    assert_eq!(map.objects.len(), 2);
    let quests: Vec<_> = extras.quests.iter().map(|(&i, m)| (i, m.clone())).collect();
    assert_eq!(
        quests,
        [
            (0, Mission::Level(10)),
            (1, Mission::PrimarySkills([1, 2, 0, 3])),
            (2, Mission::KillHero(2)),
            (3, Mission::KillCreature(13)),
            (4, Mission::Artifacts(vec![7, 8])),
            (
                5,
                Mission::Army(vec![
                    Stack {
                        creature: 0,
                        count: 10
                    },
                    Stack {
                        creature: 13,
                        count: 1
                    }
                ])
            ),
            (6, Mission::Resources([0, 0, 0, 0, 0, 5, 1000])),
            (7, Mission::Hero(3)),
            (8, Mission::Player(1)),
        ]
    );
    let refused = |objects: &str| {
        Situation::parse(&text(objects))
            .unwrap()
            .build(&BTreeMap::new(), &templates)
            .unwrap_err()
    };
    assert_eq!(
        refused(r#"objects = [{ def = "seer.def", at = [2, 2], quest = { hero = [1, 1] } }]"#),
        "the quest at [2, 2]: no hero on [1, 1]"
    );
    assert_eq!(
        refused(r#"objects = [{ def = "seer.def", at = [2, 2], quest = { monster = [2, 2] } }]"#),
        "the quest at [2, 2]: no monster anchored on [2, 2]"
    );
}

#[test]
fn owned_objects_and_towns_by_their_buildings() {
    let line = |def: &str, class, subclass| {
        ObjectTemplate::from_objects_txt(&format!(
            "{def} {} {} 111111111 111111111 {class} {subclass} 0 0",
            "1".repeat(48),
            "0".repeat(47) + "1"
        ))
        .unwrap()
    };
    let templates = [
        line("mine.def", 53, 0),
        line("avccast0.def", 98, 0),
        line("avcramp0.def", 98, 1),
    ];
    let text = r#"size = 36
terrain = "grass"
heroes = [{ at = [34, 34], class = 2 }, { at = [34, 1], owner = 1 }]
view = [0, 0, 0]
objects = [{ def = "mine.def", at = [3, 4], owner = 1 }, { def = "mine.def", at = [6, 4] }]
towns = [
    { at = [10, 10], faction = 1, owner = 0, buildings = "village" },
    { at = [20, 10], faction = 0, buildings = "fort" },
    { at = [10, 20], faction = 0, buildings = "citadel" },
    { at = [20, 20], faction = 1, owner = 1, buildings = "castle" },
    { at = [30, 20], faction = 0, buildings = "capitol" },
    { at = [10, 30], faction = 0, buildings = "townhall" },
    { at = [20, 30], faction = 1, buildings = "cityhall" },
]"#;
    let situation = Situation::parse(text).unwrap();
    assert!(situation.needs_templates());
    let (map, extras) = situation.build(&BTreeMap::new(), &templates).unwrap();
    let owners: Vec<_> = extras.objects.iter().map(|o| o.owner).collect();
    assert_eq!(owners, [Some(1), None]);
    // The towns after the heroes, each of its faction's template.
    let towns: Vec<_> = map
        .objects
        .iter()
        .filter_map(|o| match o {
            MapObject::Town(t) => Some((t.tile, t.template.subclass, t.owner, t.buildings)),
            _ => None,
        })
        .collect();
    let custom = |byte: u8| TownBuildings::Custom {
        built: [byte, 0, 0, 0, 0, 0],
        forbidden: [0; 6],
    };
    assert_eq!(
        towns,
        [
            (
                (10, 10),
                1,
                Some(0),
                TownBuildings::Standard { fort: false }
            ),
            ((20, 10), 0, None, TownBuildings::Standard { fort: true }),
            ((10, 20), 0, None, custom(0x18)),
            ((20, 20), 1, Some(1), custom(0x38)),
            ((30, 20), 0, None, custom(0x3F)),
            // E24 M3 T5b2: the Town Hall and a Fort; the City Hall too.
            ((10, 30), 0, None, custom(0b1001)),
            ((20, 30), 1, None, custom(0b1011)),
        ]
    );
    assert!(matches!(map.objects[2], MapObject::Town(_)));
    // The map writes and reads back; a faction without a template is refused.
    let raw = h3_map::parse(
        &writer::write_with(&map, &extras).unwrap(),
        &Limits::default(),
    );
    assert!(raw.is_ok());
    let none = text.replace("faction = 1, owner = 0", "faction = 2, owner = 0");
    assert_eq!(
        Situation::parse(&none)
            .unwrap()
            .build(&BTreeMap::new(), &templates)
            .unwrap_err(),
        "town at [10, 10]: no template of faction 2"
    );
}

#[test]
fn heroes_of_every_colour_and_class_and_boats() {
    let boat = |kind| {
        ObjectTemplate::from_objects_txt(&format!(
            "AVXboat{kind}.def 1011{} 01{} 100000000 100000000 8 {kind} 0 0",
            "1".repeat(44),
            "0".repeat(46)
        ))
        .unwrap()
    };
    let templates = [boat(0), boat(1), boat(2)];
    let text = r#"
size = 36
terrain = "water"
heroes = [
    { at = [1, 1], class = 4, facing = 3, walk = [[2, 1], [3, 1]] },
    { at = [5, 1], owner = 7, class = 4 },
    { at = [9, 1], owner = 3, class = 16 },
    { at = [13, 1], owner = 3, class = 17 },
]
boats = [{ at = [3, 1], kind = 1, facing = 5 }, { at = [20, 4], kind = 2 }]
view = [0, 0, 0]
"#;
    let situation = Situation::parse(text).unwrap();
    assert!(situation.needs_templates());
    assert_eq!(situation.heroes[0].walk, [[2, 1], [3, 1]]);
    assert_eq!(
        (situation.heroes[0].facing, situation.boats[0].facing),
        (Some(3), Some(5))
    );
    let (map, extras) = situation.build(&BTreeMap::new(), &templates).unwrap();
    // Red, Green and Pink play; each hero of its class's next free type.
    let playing: Vec<bool> = map.players.iter().map(Option::is_some).collect();
    assert_eq!(
        playing,
        [true, false, false, true, false, false, false, true]
    );
    let heroes: Vec<(u8, u8, u8)> = map
        .objects
        .iter()
        .filter_map(|o| match o {
            MapObject::Hero(h) => Some((h.owner, h.hero_type, h.class)),
            _ => None,
        })
        .collect();
    assert_eq!(heroes, [(0, 32, 4), (7, 33, 4), (3, 128, 16), (3, 136, 17)]);
    // The boats after the objects, each its type's template anchored one column right.
    let boats: Vec<((u8, u8), u32)> = extras
        .objects
        .iter()
        .map(|o| (o.tile, o.template.subclass))
        .collect();
    assert_eq!(boats, [((4, 1), 1), ((21, 4), 2)]);
    let bytes = writer::write_with(&map, &extras).unwrap();
    assert!(h3_map::parse(&bytes, &Limits::default()).is_ok());
}

#[test]
fn a_heros_spells_in_its_worn_book() {
    // E24 M4 T2d3: the spells given, the book worn in slot 17 its artifacts alone; none, the
    // type's; one past 69 or given twice refused.
    let text = |spells: &str| {
        format!(
            "size = 36\nterrain = \"grass\"\nheroes = [{{ at = [4, 4], class = 4{spells} }}, \
             {{ at = [9, 9], owner = 1, class = 4 }}]\nview = [0, 0, 0]\n"
        )
    };
    let hero = |spells: &str| {
        let (map, _) = Situation::parse(&text(spells))
            .unwrap()
            .build(&BTreeMap::new(), &[])
            .unwrap();
        map.objects
            .into_iter()
            .find_map(|o| match o {
                MapObject::Hero(h) => Some(h),
                _ => None,
            })
            .unwrap()
    };
    let caster = hero(", spells = [0, 9, 69]");
    let mut bits = [0u8; 9];
    (bits[0], bits[1], bits[8]) = (1, 2, 0x20);
    assert_eq!(caster.spells, Some(bits));
    let worn = caster.artifacts.unwrap().worn;
    assert_eq!(worn.iter().flatten().count(), 1);
    assert_eq!(worn[17], Some(0));
    let plain = hero("");
    assert_eq!((plain.spells, plain.artifacts), (None, None));
    for bad in [", spells = [70]", ", spells = [9, 9]"] {
        let refused = Situation::parse(&text(bad))
            .unwrap()
            .build(&BTreeMap::new(), &[])
            .unwrap_err();
        assert!(refused.contains("past 69 or given twice"), "{refused}");
    }
}

#[test]
fn the_hero_types_of_a_class() {
    assert_eq!(hero_type(0, 0), Some(0));
    assert_eq!(hero_type(15, 7), Some(127));
    assert_eq!(hero_type(16, 0), Some(128));
    assert_eq!(hero_type(17, 7), Some(143));
    assert_eq!(hero_type(17, 8), None);
    assert_eq!(hero_type(18, 0), None);
}

#[test]
fn refuses_heroes_and_boats_it_cannot_write() {
    let boat0 = ObjectTemplate::from_objects_txt(&format!(
        "AVXboat0.def {} {} 100000000 100000000 8 0 0 0",
        "1".repeat(48),
        "0".repeat(48)
    ))
    .unwrap();
    let build = |heroes: &str, boats: &str| {
        let text = format!(
            "size = 36\nterrain = \"water\"\nheroes = [{heroes}]\nboats = [{boats}]\nview = [0, 0, 0]\n"
        );
        Situation::parse(&text)
            .unwrap()
            .build(&BTreeMap::new(), core::slice::from_ref(&boat0))
            .map(|_| ())
    };
    let two = "{ at = [1, 1] }, { at = [3, 3], owner = 1 }";
    assert_eq!(build(two, "{ at = [5, 5], kind = 0 }"), Ok(()));
    // A Red hero's path and a boat's flag colour (E24 M1 T5b2), the harness's alone.
    let shown = "{ at = [1, 1], path = [4, 2] }, { at = [3, 3], owner = 1 }";
    assert_eq!(
        build(shown, "{ at = [5, 5], kind = 0, colour = 7 }"),
        Ok(())
    );
    for (heroes, boats, error) in [
        (
            "{ at = [1, 1], owner = 1, path = [2, 2] }, { at = [3, 3] }",
            "",
            "hero at [1, 1]: only Red's heroes walk, show a path or have anything written",
        ),
        (
            "{ at = [1, 1], path = [2, 2] }, { at = [3, 3], path = [4, 4] }",
            "",
            "hero at [3, 3]: a second path shown",
        ),
        (
            two,
            "{ at = [5, 5], kind = 0, colour = 8 }",
            "boat at [5, 5]: no template of type 0 or a facing or colour past 7",
        ),
        (
            "{ at = [1, 1] }",
            "",
            "a situation needs a hero of Red's and of another colour",
        ),
        (
            "{ at = [1, 1], owner = 1 }, { at = [3, 3], owner = 2 }",
            "",
            "a situation needs a hero of Red's and of another colour",
        ),
        (
            "{ at = [1, 1], owner = 8 }",
            "",
            "hero at [1, 1]: owner 8 or facing 0 past 7",
        ),
        (
            "{ at = [1, 1], facing = 8 }",
            "",
            "hero at [1, 1]: owner 0 or facing 8 past 7",
        ),
        (
            "{ at = [1, 1], class = 18 }",
            "",
            "hero at [1, 1]: class 18 is no hero class (0-17)",
        ),
        (
            "{ at = [1, 1], owner = 1, walk = [[2, 2]] }",
            "",
            "hero at [1, 1]: only Red's heroes walk, show a path or have anything written",
        ),
        (
            two,
            "{ at = [5, 5], kind = 1 }",
            "boat at [5, 5]: no template of type 1 or a facing or colour past 7",
        ),
        (
            "{ at = [1, 1] }, { at = [3, 3], owner = 1, points = 5 }",
            "",
            "hero at [3, 3]: only Red's heroes walk, show a path or have anything written",
        ),
        (
            "{ at = [1, 1] }, { at = [3, 3], owner = 1, mana = 5 }",
            "",
            "hero at [3, 3]: only Red's heroes walk, show a path or have anything written",
        ),
        // T7b2 (F5-6): the status window's facts are Red's alone too.
        (
            "{ at = [1, 1] }, { at = [3, 3], owner = 1, morale = 1 }",
            "",
            "hero at [3, 3]: only Red's heroes walk, show a path or have anything written",
        ),
        (
            "{ at = [1, 1] }, { at = [3, 3], owner = 1, army = [[0, 1]] }",
            "",
            "hero at [3, 3]: only Red's heroes walk, show a path or have anything written",
        ),
        (
            two,
            "{ at = [5, 5], kind = 0, facing = 9 }",
            "boat at [5, 5]: no template of type 0 or a facing or colour past 7",
        ),
    ] {
        assert_eq!(
            build(heroes, boats),
            Err(error.to_string()),
            "{heroes} {boats}"
        );
    }
    // A ninth hero of a colour (each of another class), a ninth of a class (of two colours).
    let nine = |hero: &dyn Fn(usize) -> String| {
        let list: Vec<String> = (0..9)
            .map(|i| format!("{{ at = [{}, 1]{} }}", 2 * i, hero(i)))
            .collect();
        format!("{}, {{ at = [1, 9], owner = 1 }}", list.join(", "))
    };
    assert_eq!(
        build(&nine(&|i| format!(", class = {i}")), ""),
        Err("hero at [16, 1]: a ninth of colour 0".to_string())
    );
    assert_eq!(
        build(&nine(&|i| format!(", class = 5, owner = {}", i % 2)), ""),
        Err("hero at [16, 1]: a ninth of class 5".to_string())
    );
}

#[test]
fn the_lists_writes_and_the_selection() {
    // E24 M3 T4a: the harness writes Red's heroes' points and mana and Red's towns' day, then
    // selects Red's hero on its tile (after its walk) or Red's town on its anchor.
    let castle = ObjectTemplate::from_objects_txt(&format!(
        "avccast0.def {} {} 111111111 111111111 98 0 0 0",
        "1".repeat(48),
        "0".repeat(47) + "1"
    ))
    .unwrap();
    let build = |rest: &str| {
        let text = format!(
            "size = 36\nterrain = \"grass\"\nview = [0, 0, 0]\nheroes = [{{ at = [1, 1], walk = \
             [[2, 1]], points = 2800, mana = 0 }}, {{ at = [5, 5] }}, {{ at = [9, 9], owner = 1 \
             }}]\ntowns = [{{ at = [20, 20], faction = 0, owner = 0, buildings = \"fort\", built = \
             true }}, {{ at = [30, 20], faction = 0, owner = 1, buildings = \"fort\" }}]\n{rest}"
        );
        Situation::parse(&text)
            .unwrap()
            .build(&BTreeMap::new(), core::slice::from_ref(&castle))
            .map(|_| ())
    };
    for good in [
        "select = { hero = [2, 1] }",
        "select = { hero = [5, 5] }",
        "select = { town = [20, 20] }",
    ] {
        assert_eq!(build(good), Ok(()), "{good}");
    }
    for (bad, error) in [
        ("", "a write into the lists without a selection"),
        (
            "select = { hero = [1, 1] }",
            "the selection Some(Hero([1, 1])) is none of Red's",
        ),
        (
            "select = { hero = [9, 9] }",
            "the selection Some(Hero([9, 9])) is none of Red's",
        ),
        (
            "select = { town = [30, 20] }",
            "the selection Some(Town([30, 20])) is none of Red's",
        ),
    ] {
        assert_eq!(build(bad), Err(error.to_string()), "{bad}");
    }
    // A selection without writes; a day written in another colour's town.
    let plain = Situation::parse(
        "size = 36\nterrain = \"grass\"\nview = [0, 0, 0]\nheroes = [{ at = [1, 1] }, { at = \
         [9, 9], owner = 1 }]\nselect = { hero = [1, 1] }\n",
    )
    .unwrap();
    assert!(plain.build(&BTreeMap::new(), &[]).is_ok());
    let theirs = Situation::parse(
        "size = 36\nterrain = \"grass\"\nview = [0, 0, 0]\nheroes = [{ at = [1, 1] }, { at = \
         [9, 9], owner = 1 }]\ntowns = [{ at = [30, 20], faction = 0, owner = 1, buildings = \
         \"fort\", built = true }]\n",
    )
    .unwrap();
    assert_eq!(
        theirs.build(&BTreeMap::new(), core::slice::from_ref(&castle)),
        Err("town at [30, 20]: only Red's towns have their day written".to_string())
    );
    // T5a: the resources and the date with a hero's selection, a day of the game's.
    let bar = "resources = [0, 1, 2, 3, 4, 5, 6]";
    for good in [
        format!("select = {{ hero = [2, 1] }}\n{bar}"),
        "select = { hero = [5, 5] }\ndate = [12, 4, 7]".to_string(),
        "select = { hero = [5, 5] }\ndate = [1, 1, 1]".to_string(),
    ] {
        assert_eq!(build(&good), Ok(()), "{good}");
    }
    let without = "the resources or the date without a hero's selection";
    for (bad, error) in [
        (format!("select = {{ town = [20, 20] }}\n{bar}"), without),
        (
            "select = { town = [20, 20] }\ndate = [1, 1, 1]".to_string(),
            without,
        ),
        (
            "select = { hero = [5, 5] }\ndate = [0, 1, 1]".to_string(),
            "[0, 1, 1]",
        ),
        (
            "select = { hero = [5, 5] }\ndate = [1, 0, 1]".to_string(),
            "[1, 0, 1]",
        ),
        (
            "select = { hero = [5, 5] }\ndate = [1, 5, 1]".to_string(),
            "[1, 5, 1]",
        ),
        (
            "select = { hero = [5, 5] }\ndate = [1, 1, 0]".to_string(),
            "[1, 1, 0]",
        ),
        (
            "select = { hero = [5, 5] }\ndate = [1, 1, 8]".to_string(),
            "[1, 1, 8]",
        ),
    ] {
        let error = if error.starts_with('[') {
            format!("the date Some({error}) is no day of the game's")
        } else {
            error.to_string()
        };
        assert_eq!(build(&bad), Err(error), "{bad}");
    }
    let plain_bar = Situation::parse(&format!(
        "size = 36\nterrain = \"grass\"\nview = [0, 0, 0]\nheroes = [{{ at = [1, 1] }}, {{ at = \
         [9, 9], owner = 1 }}]\n{bar}\n"
    ))
    .unwrap();
    assert_eq!(
        plain_bar.build(&BTreeMap::new(), &[]).map(|_| ()),
        Err(without.to_string())
    );
}

#[test]
fn the_status_windows_writes() {
    // E24 M3 T5b1 (design §4.8): a hero's skills, morale, luck and army; a town's garrison and
    // garrison hero, Red's; seven slots at most; a silo with chosen buildings, the map's
    // building 9.
    let castle = ObjectTemplate::from_objects_txt(&format!(
        "avccast0.def {} {} 111111111 111111111 98 0 0 0",
        "1".repeat(48),
        "0".repeat(47) + "1"
    ))
    .unwrap();
    let build = |hero: &str, town: &str, select: &str| {
        let text = format!(
            "size = 36\nterrain = \"grass\"\nview = [0, 0, 0]\nheroes = [{{ at = [1, 1]{hero} }}, \
             {{ at = [9, 9], owner = 1 }}]\ntowns = [{{ at = [20, 20], faction = 0, owner = 0, \
             buildings = \"castle\"{town} }}, {{ at = [30, 20], faction = 0, owner = 1, \
             buildings = \"fort\" }}]\n{select}"
        );
        Situation::parse(&text)
            .unwrap()
            .build(&BTreeMap::new(), core::slice::from_ref(&castle))
    };
    let hero = ", skills = [120, -5, 0, -1], morale = 3, luck = -3, army = [[1, 2], [3, 4]]";
    let hero_selected = "select = { hero = [1, 1] }";
    assert!(build(hero, "", hero_selected).is_ok());
    let town = ", silo = true, garrison = [[1, 99999]], garrison_hero = [1, 1]";
    let town_selected = "select = { town = [20, 20] }";
    let (built, _) = build("", town, town_selected).unwrap();
    // The silo among the castle's chosen buildings (Fort, Citadel, Castle: bits 3 to 5).
    assert!(built.objects.iter().any(|o| matches!(
        o,
        MapObject::Town(t) if t.buildings == TownBuildings::Custom {
            built: [0b11_1000, 0b10, 0, 0, 0, 0],
            forbidden: [0; 6],
        }
    )));
    for (hero, town, select, error) in [
        (hero, "", "", "a write into the lists without a selection"),
        (
            "",
            ", garrison = [[1, 1]]",
            "",
            "a write into the lists without a selection",
        ),
        (
            ", army = [[1, 1], [1, 1], [1, 1], [1, 1], [1, 1], [1, 1], [1, 1], [1, 1]]",
            "",
            hero_selected,
            "an army of more than seven slots",
        ),
        (
            "",
            ", garrison_hero = [9, 9]",
            town_selected,
            "town at [20, 20]: its garrison hero is none of Red's",
        ),
    ] {
        assert_eq!(
            build(hero, town, select).map(|_| ()),
            Err(error.to_string()),
            "{hero} {town}"
        );
    }
    let theirs = Situation::parse(
        "size = 36\nterrain = \"grass\"\nview = [0, 0, 0]\nheroes = [{ at = [1, 1] }, { at = \
         [9, 9], owner = 1 }]\ntowns = [{ at = [30, 20], faction = 0, owner = 1, buildings = \
         \"fort\", garrison = [[1, 1]] }]\n",
    )
    .unwrap();
    assert_eq!(
        theirs
            .build(&BTreeMap::new(), core::slice::from_ref(&castle))
            .map(|_| ()),
        Err("town at [30, 20]: only Red's towns have their garrison written".to_string())
    );
    let standard = theirs_with_silo();
    assert_eq!(
        standard
            .build(&BTreeMap::new(), core::slice::from_ref(&castle))
            .map(|_| ()),
        Err("town at [20, 20]: a silo with the standard buildings".to_string())
    );
    // E24 M3 T5b2 (design §4.9): the kind asked, the date's or the kingdom's; the teams, each 0
    // to 7.
    let asked = |extra: &str| build("", "", &format!("{extra}\n")).map(|_| ());
    assert_eq!(
        asked("status = 1\nteams = [0, 1, 0, 2, 3, 4, 5, 7]"),
        Ok(())
    );
    assert_eq!(asked("status = 2"), Ok(()));
    assert_eq!(
        asked("status = 3"),
        Err("the status window's kind 3: not 1 or 2".to_string())
    );
    assert_eq!(
        asked("teams = [0, 1, 0, 2, 3, 4, 5, 8]"),
        Err("the teams [0, 1, 0, 2, 3, 4, 5, 8]: one past 7".to_string())
    );
    // E24 M3 T6a2a (design §4.11): the visit marks, each of an object given, once.
    let pond = "objects = [{ def = \"avccast0.def\", at = [5, 5] }]";
    assert_eq!(
        asked(&format!(
            "{pond}\nvisits = [{{ at = [5, 5], known = true, visited = false }}]"
        )),
        Ok(())
    );
    assert_eq!(
        asked(&format!("{pond}\nvisits = [{{ at = [6, 5] }}]")),
        Err("the visits at [6, 5]: no object anchored there".to_string())
    );
    assert_eq!(
        asked(&format!(
            "{pond}\nvisits = [{{ at = [5, 5] }}, {{ at = [5, 5] }}]"
        )),
        Err("the visits at [5, 5]: given twice".to_string())
    );
}

fn theirs_with_silo() -> Situation {
    Situation::parse(
        "size = 36\nterrain = \"grass\"\nview = [0, 0, 0]\nheroes = [{ at = [1, 1] }, { at = \
         [9, 9], owner = 1 }]\ntowns = [{ at = [20, 20], faction = 0, owner = 0, buildings = \
         \"fort\", silo = true }]\n",
    )
    .unwrap()
}

#[test]
fn a_shipped_map_loaded_whole_is_the_harnesss() {
    // E24 M1 T6b (the reviewer's R2-12): not the writer's to build.
    let text = alloc::format!("map = \"Hoard.h3m\"\n{PLAIN}");
    assert_eq!(
        Situation::parse(&text).unwrap_err(),
        "it names a shipped map whole (`map`): the harness loads it"
    );
    // Not TOML, and TOML that is no situation.
    assert!(Situation::parse("size = ").is_err());
    let unknown = alloc::format!("{PLAIN}colour = 3\n");
    assert!(Situation::parse(&unknown).unwrap_err().contains("colour"));
}

/// E24 M4 T2a1 (design §5.3): the objects with a body among the map's placed objects, after the
/// heroes, each on its class (a pile's amount, a Shrine's spell, a Witch Hut's skills, an event's
/// rewards), the rest the extras'; a hero's experience the writer's; a body its class does not
/// take, two bodies, a skill past 27 and bodies with quests refused.
#[test]
fn objects_with_their_bodies() {
    let line = |def: &str, class, subclass| {
        ObjectTemplate::from_objects_txt(&format!(
            "{def} {} {} 111111111 111111111 {class} {subclass} 0 0",
            "1".repeat(48),
            "0".repeat(48)
        ))
        .unwrap()
    };
    let templates = [
        line("pile.def", 79, 6),
        line("shrine.def", 88, 0),
        line("hut.def", 113, 0),
        line("event.def", 26, 0),
        line("plain.def", 112, 0),
    ];
    let text = |objects: &str| {
        format!(
            "size = 36\nterrain = \"grass\"\nheroes = [{{ at = [4, 4], class = 1, experience = 3200 }}, {{ at = [34, 1], owner = 1 }}]\nview = [0, 0, 0]\n{objects}"
        )
    };
    let given = text(
        r#"objects = [
    { def = "pile.def", at = [5, 4], amount = 700 },
    { def = "plain.def", at = [9, 9] },
    { def = "shrine.def", at = [5, 6], spell = 53 },
    { def = "hut.def", at = [6, 8], skills = [2, 27] },
    { def = "event.def", at = [5, 10], rewards = { luck = 1, mana = 10, resources = [0, 0, 0, 0, 0, 0, 500], skills = [[2, 1]], creatures = [[1, 5]] } },
]"#,
    );
    let (map, extras) = Situation::parse(&given)
        .unwrap()
        .build(&BTreeMap::new(), &templates)
        .unwrap();
    assert_eq!(
        extras.objects.iter().map(|o| o.tile).collect::<Vec<_>>(),
        [(9, 9)]
    );
    let MapObject::Hero(red) = &map.objects[0] else {
        panic!("Red's hero first");
    };
    assert_eq!(red.experience, Some(3200));
    let MapObject::Pile(pile) = &map.objects[2] else {
        panic!("the pile after the heroes");
    };
    assert_eq!(
        (pile.tile, pile.amount, pile.guards.is_none()),
        ((5, 4), 700, true)
    );
    let bodies: Vec<_> = map.objects[3..]
        .iter()
        .map(|o| match o {
            MapObject::Reward(r) => (r.tile, r.body.clone()),
            _ => panic!("a reward object"),
        })
        .collect();
    let mut rewards = Rewards {
        mana: 10,
        luck: 1,
        ..Rewards::default()
    };
    rewards.resources[6] = 500;
    rewards.skills = vec![(2, 1)];
    rewards.creatures = vec![Stack {
        creature: 1,
        count: 5,
    }];
    assert_eq!(
        bodies,
        [
            ((5, 6), RewardBody::Shrine(Some(53))),
            ((6, 8), RewardBody::WitchHut(Some([4, 0, 0, 8]))),
            (
                (5, 10),
                RewardBody::Event(Event {
                    message: None,
                    rewards,
                    players: 0xFF,
                    computer: false,
                    removed_after_visit: true,
                })
            ),
        ]
    );
    let refused = |objects: &str| {
        Situation::parse(&text(objects))
            .unwrap()
            .build(&BTreeMap::new(), &templates)
            .unwrap_err()
    };
    assert_eq!(
        refused(r#"objects = [{ def = "plain.def", at = [5, 4], amount = 7 }]"#),
        "plain.def at [5, 4]: a body its class 112 does not take, or more than one"
    );
    assert_eq!(
        refused(r#"objects = [{ def = "shrine.def", at = [5, 4], spell = 1, amount = 7 }]"#),
        "shrine.def at [5, 4]: a body its class 88 does not take, or more than one"
    );
    assert_eq!(
        refused(r#"objects = [{ def = "hut.def", at = [5, 4], skills = [28] }]"#),
        "hut.def at [5, 4]: skill 28 past 27"
    );
    assert_eq!(
        refused(
            r#"objects = [{ def = "pile.def", at = [5, 4], amount = 7 }, { def = "plain.def", at = [7, 7], quest = { level = 2 } }]"#
        ),
        "a situation with both objects' bodies and quests"
    );
}

/// E24 M4 T2b1 (design §5.4): a monster's body, a pile's and an artifact's guards with their
/// text, Pandora's rewards, each on its class; guards of no stack, of eight, of a count of 0, on
/// a monster or a Shrine refused, a monster's count or disposition out of range refused; the
/// visits' words in their bits' ranges, a hero's Diplomacy at most Expert.
#[test]
fn questions_bodies_and_words() {
    let line = |def: &str, class, subclass| {
        ObjectTemplate::from_objects_txt(&format!(
            "{def} {} {} 111111111 111111111 {class} {subclass} 0 0",
            "1".repeat(48),
            "0".repeat(48)
        ))
        .unwrap()
    };
    let templates = [
        line("pile.def", 79, 6),
        line("art.def", 5, 7),
        line("box.def", 6, 0),
        line("peas.def", 54, 139),
        line("shrine.def", 88, 0),
        line("hut.def", 113, 0),
        line("event.def", 26, 0),
        line("chest.def", 101, 0),
    ];
    let text = |rest: &str| {
        format!(
            "size = 36\nterrain = \"grass\"\nheroes = [{{ at = [4, 4], class = 1, diplomacy = 2 }}, {{ at = [34, 1], owner = 1 }}]\nview = [0, 0, 0]\n{rest}"
        )
    };
    let given = text(
        r#"objects = [
    { def = "pile.def", at = [5, 4], guards = { army = [[0, 10]], text = "Fight?" } },
    { def = "art.def", at = [5, 6], guards = { army = [[0, 10], [1, 2]] } },
    { def = "box.def", at = [5, 8], rewards = { resources = [0, 0, 0, 0, 0, 0, 1500] } },
    { def = "peas.def", at = [5, 10], monster = { count = 10, disposition = 0, never_flees = true } },
    { def = "chest.def", at = [7, 7] },
]
visits = [{ at = [5, 10], aggression = -4 }, { at = [7, 7], gold = 1000 }]"#,
    );
    let situation = Situation::parse(&given).unwrap();
    assert_eq!(situation.heroes[0].diplomacy, Some(2));
    assert_eq!(
        (situation.visits[0].aggression, situation.visits[1].gold),
        (Some(-4), Some(1000))
    );
    let (map, _) = situation.build(&BTreeMap::new(), &templates).unwrap();
    let MapObject::Pile(pile) = &map.objects[2] else {
        panic!("the pile after the heroes");
    };
    let mut army = [None; 7];
    army[0] = Some((0, 10));
    assert_eq!(
        (pile.amount, pile.guards.clone()),
        (
            0,
            Some(Guards {
                army,
                question: "Fight?".into()
            })
        )
    );
    let MapObject::Reward(artifact) = &map.objects[3] else {
        panic!("the artifact");
    };
    army[1] = Some((1, 2));
    assert_eq!(
        artifact.body,
        RewardBody::Artifact(Some(Message {
            text: String::new(),
            guards: Some(army),
        }))
    );
    let MapObject::Reward(pandora) = &map.objects[4] else {
        panic!("Pandora's Box");
    };
    let mut rewards = Rewards::default();
    rewards.resources[6] = 1500;
    assert_eq!(
        pandora.body,
        RewardBody::Pandora {
            message: None,
            rewards
        }
    );
    let MapObject::Monster(monster) = &map.objects[5] else {
        panic!("the monster");
    };
    assert_eq!(
        (
            monster.tile,
            monster.count,
            monster.disposition,
            monster.never_flees
        ),
        ((5, 10), 10, 0, true)
    );
    assert!(!monster.fixed_count && monster.treasure.is_none());
    let refused = |rest: &str| {
        Situation::parse(&text(rest))
            .and_then(|s| s.build(&BTreeMap::new(), &templates).map(|_| ()))
            .unwrap_err()
    };
    let class = |def: &str, class: u32| {
        format!("{def} at [5, 4]: a body its class {class} does not take, or more than one")
    };
    assert_eq!(
        refused(r#"objects = [{ def = "pile.def", at = [5, 4], guards = { army = [] } }]"#),
        "guards of 0 stacks: one to seven"
    );
    assert_eq!(
        refused(
            r#"objects = [{ def = "pile.def", at = [5, 4], guards = { army = [[0, 1], [0, 1], [0, 1], [0, 1], [0, 1], [0, 1], [0, 1], [0, 1]] } }]"#
        ),
        "guards of 8 stacks: one to seven"
    );
    assert_eq!(
        refused(r#"objects = [{ def = "art.def", at = [5, 4], guards = { army = [[3, 0]] } }]"#),
        "guards of creature 3: a count of 0"
    );
    for (def, body, c) in [
        (
            "peas.def",
            "monster = { count = 1, disposition = 0 }, guards = { army = [[0, 1]] }",
            54,
        ),
        ("shrine.def", "spell = 1, guards = { army = [[0, 1]] }", 88),
        ("hut.def", "skills = [1], guards = { army = [[0, 1]] }", 113),
        (
            "event.def",
            "rewards = { mana = 1 }, guards = { army = [[0, 1]] }",
            26,
        ),
        (
            "box.def",
            "rewards = { mana = 1 }, guards = { army = [[0, 1]] }",
            6,
        ),
        ("art.def", "amount = 3", 5),
    ] {
        assert_eq!(
            refused(&format!(
                r#"objects = [{{ def = "{def}", at = [5, 4], {body} }}]"#
            )),
            class(def, c)
        );
    }
    for monster in [
        "count = 0, disposition = 0",
        "count = 4096, disposition = 0",
        "count = 1, disposition = 5",
    ] {
        assert!(
            refused(&format!(
                r#"objects = [{{ def = "peas.def", at = [5, 4], monster = {{ {monster} }} }}]"#
            ))
            .starts_with("peas.def at [5, 4]: a monster of ")
        );
    }
    let word = |visit: &str| {
        refused(&format!(
            "objects = [{{ def = \"chest.def\", at = [5, 4] }}]\nvisits = [{{ at = [5, 4], {visit} }}]"
        ))
    };
    for visit in [
        "aggression = 16",
        "aggression = -17",
        "gold = 750",
        "gold = 0",
        "gold = 8000",
        "price = 3",
    ] {
        assert!(
            word(visit)
                .ends_with("an aggression, a chest's gold or a Tree's price its word cannot hold"),
            "{visit}"
        );
    }
    let expert = Situation::parse(&text("").replace("diplomacy = 2", "diplomacy = 4"))
        .unwrap()
        .build(&BTreeMap::new(), &templates)
        .unwrap_err();
    assert_eq!(expert, "the hero at [4, 4]: Diplomacy past Expert");
    // A level-up's offers (E24 M4 T2c): the secondary skills as given, checked.
    let known = |skills: &str| text("").replace("diplomacy = 2", &format!("secondary = {skills}"));
    let eight = known("[[0, 3], [1, 3], [2, 3], [3, 3], [4, 3], [5, 3], [6, 3], [7, 2]]");
    assert_eq!(
        Situation::parse(&eight).unwrap().heroes[0].secondary[7],
        [7, 2]
    );
    for bad in [
        "[[0, 3], [1, 3], [2, 3], [3, 3], [4, 3], [5, 3], [6, 3], [7, 2], [8, 1]]",
        "[[28, 1]]",
        "[[0, 0]]",
        "[[0, 4]]",
        "[[0, 1], [0, 2]]",
    ] {
        let refused = Situation::parse(&known(bad))
            .unwrap()
            .build(&BTreeMap::new(), &templates)
            .unwrap_err();
        assert_eq!(
            refused,
            "the hero at [4, 4]: secondary skills past eight, unknown, twice or at no level",
            "{bad}"
        );
    }
}
