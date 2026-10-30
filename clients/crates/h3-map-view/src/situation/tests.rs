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
        objects: vec![],
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
            "hero at [1, 1]: only Red's heroes walk, show a path or have points or mana written",
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
            "hero at [1, 1]: only Red's heroes walk, show a path or have points or mana written",
        ),
        (
            two,
            "{ at = [5, 5], kind = 1 }",
            "boat at [5, 5]: no template of type 1 or a facing or colour past 7",
        ),
        (
            "{ at = [1, 1] }, { at = [3, 3], owner = 1, points = 5 }",
            "",
            "hero at [3, 3]: only Red's heroes walk, show a path or have points or mana written",
        ),
        (
            "{ at = [1, 1] }, { at = [3, 3], owner = 1, mana = 5 }",
            "",
            "hero at [3, 3]: only Red's heroes walk, show a path or have points or mana written",
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
