use super::*;
use alloc::collections::BTreeMap;

use h3_map::Limits;
use h3_map::writer::{self, AnyObject};

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
