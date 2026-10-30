use super::*;
use alloc::string::ToString;
use alloc::vec;

use h3_map::Limits;
use h3_map::writer::{
    self, Ground, Hero, Map, MapObject, ObjectTemplate, Patch, Player, Road, RoadKind, Terrain,
    Town, TownBuildings,
};

/// A template of the writer's with `def`, `class` and the masks: one blocked cell at the anchor,
/// visited one column left of it.
pub(crate) fn template(def: &str, class: u32, subclass: u32, priority: u8) -> ObjectTemplate {
    ObjectTemplate {
        def: def.to_string(),
        passable: [0xff, 0xff, 0xff, 0xff, 0xff, 0x7f],
        visitable: [0, 0, 0, 0, 0, 0x40],
        terrains: 0x1ff,
        menu: 0x1ff,
        class,
        subclass,
        group: 0,
        priority,
    }
}

/// A 36 × 36 grass map with a pool of water at the top-left, a dirt road on row 5 from column 3
/// to 6, and its objects: a red hero of class 2 at (10, 10), a special ground (flat) at (20, 20),
/// a blue town at (30, 30).
pub(crate) fn map() -> Map {
    let mut players = [None, None, None, None, None, None, None, None];
    for p in &mut players[..2] {
        *p = Some(Player {
            human: true,
            computer: true,
            towns: 0x1ff,
        });
    }
    Map {
        name: "art".to_string(),
        description: String::new(),
        size: 36,
        terrain: Terrain::Grass,
        patches: vec![Patch {
            terrain: Terrain::Water,
            from: (0, 0),
            to: (2, 2),
        }],
        roads: vec![Road {
            kind: RoadKind::Dirt,
            row: 5,
            from: 3,
            to: 6,
        }],
        players,
        objects: vec![
            MapObject::Hero(Hero {
                tile: (10, 10),
                owner: 0,
                hero_type: 16,
                class: 2,
                name: None,
                experience: None,
                skills: None,
                army: None,
                tight_formation: false,
                artifacts: None,
                spells: None,
                primary: None,
            }),
            MapObject::Ground(Ground {
                tile: (20, 20),
                template: template("cursed.def", writer::GROUND_OBJECTS[0], 0, 1),
            }),
            MapObject::Town(Town {
                tile: (30, 30),
                template: template("avccast0.def", writer::TOWN_OBJECT, 0, 0),
                owner: Some(1),
                garrison: None,
                tight_formation: false,
                buildings: TownBuildings::Standard { fort: true },
            }),
        ],
        underground: None,
        level_limit: 0,
        banned_skills: vec![],
        events: Vec::new(),
    }
}

/// The raw model of a writer's map.
pub(crate) fn raw(map: &Map) -> RawMap {
    h3_map::parse(&writer::write(map).unwrap(), &Limits::default()).unwrap()
}

#[test]
fn reads_the_tiles_pictures_and_mirrors() {
    let mut raw = raw(&map());
    // A river the writer cannot write yet (E24 T3a): code 3, picture 7, mirrored both ways.
    let t = &mut raw.terrain.tiles[4 * 36 + 8];
    t.river = 3;
    t.river_dir = 7;
    t.flags |= 0x0C;
    let art = MapArt::from_raw(&raw).unwrap();
    assert_eq!((art.size, art.levels, art.tiles.len()), (36, 1, 36 * 36));
    let pool = art.tile(1, 1, 0).unwrap();
    assert_eq!(pool.terrain.code, 8);
    assert_eq!((pool.river, pool.road), (None, None));
    // The road's west end is picture 15, its east end 15 mirrored (the writer's).
    let west = art.tile(3, 5, 0).unwrap().road.unwrap();
    let east = art.tile(6, 5, 0).unwrap().road.unwrap();
    assert_eq!((west.code, west.picture, west.mirror_x), (1, 15, false));
    assert_eq!(
        (east.picture, east.mirror_x, east.mirror_y),
        (15, true, false)
    );
    let river = art.tile(8, 4, 0).unwrap().river.unwrap();
    assert_eq!(
        river,
        Piece {
            code: 3,
            picture: 7,
            mirror_x: true,
            mirror_y: true
        }
    );
    // Beside the pool: the coast flag kept as stored.
    assert_eq!(art.tile(3, 1, 0).unwrap().flags & 0x40, 0x40);
    let grass = art.tile(10, 20, 0).unwrap().terrain;
    assert_eq!(
        (grass.code, grass.mirror_x, grass.mirror_y),
        (2, false, false)
    );
    let mirrored = &mut raw.terrain.tiles[0];
    mirrored.flags = 0x03;
    let art = MapArt::from_raw(&raw).unwrap();
    let corner = art.tile(0, 0, 0).unwrap().terrain;
    assert!(corner.mirror_x && corner.mirror_y);
}

#[test]
fn a_tile_off_the_map_is_none() {
    let art = MapArt::from_raw(&raw(&map())).unwrap();
    for (x, y, z) in [(-1, 0, 0), (0, -1, 0), (36, 0, 0), (0, 36, 0), (0, 0, 1)] {
        assert_eq!(art.tile(x, y, z), None, "({x}, {y}, {z})");
    }
    assert!(art.tile(35, 35, 0).is_some());
}

#[test]
fn reads_the_templates_and_objects_in_the_maps_order() {
    let art = MapArt::from_raw(&raw(&map())).unwrap();
    let defs: Vec<&str> = art.templates.iter().map(|t| t.def.as_str()).collect();
    assert_eq!(defs, ["ah02_e.def", "cursed.def", "avccast0.def"]);
    let hero = &art.templates[0];
    assert_eq!((hero.class, hero.subclass, hero.flat()), (34, 2, false));
    assert!(art.templates[1].flat());
    let anchors: Vec<(u8, u8, usize)> = art
        .objects
        .iter()
        .map(|o| (o.anchor.x, o.anchor.y, o.template))
        .collect();
    assert_eq!(anchors, [(11, 10, 0), (20, 20, 1), (30, 30, 2)]);
}

#[test]
fn the_maps_own_hero_names() {
    // E24 M3 T5b1 (design §4.8): a hero object's name, else a disposed hero's, the first of a type;
    // an empty one none; Windows-1252 decoded.
    let mut raw = raw(&map());
    assert_eq!(MapArt::from_raw(&raw).unwrap().hero_names, BTreeMap::new());
    let Body::Hero(hero) = &mut raw.objects[0].body else {
        panic!("the art map's first object is its hero");
    };
    let own = hero.hero_type;
    hero.name = Some(model::Text(vec![b'O', 0xE9, 0x80]));
    let disposed = |hero_type, name: &[u8]| model::DisposedHero {
        hero_type,
        portrait: 0,
        name: model::Text(name.to_vec()),
        players: 0xFF,
    };
    raw.disposed_heroes = vec![
        disposed(own, b"Later"),
        disposed(7, b"Seven"),
        disposed(9, b""),
    ];
    let names = MapArt::from_raw(&raw).unwrap().hero_names;
    assert_eq!(
        names,
        BTreeMap::from([
            (own, "O\u{e9}\u{20ac}".to_string()),
            (7, "Seven".to_string())
        ])
    );
}

#[test]
fn reads_a_cell_of_the_masks() {
    let t = TemplateArt::from_writer(&template("x.def", 1, 0, 0));
    assert!(!t.passable(0, 0));
    assert!(t.passable(1, 0) && t.passable(7, 5));
    assert!(t.visitable(1, 0));
    assert!(!t.visitable(0, 0) && !t.visitable(1, 1));
    // Outside the masks: passable, not visitable.
    for (dx, dy) in [(8, 0), (0, 6)] {
        assert!(t.passable(dx, dy) && !t.visitable(dx, dy));
    }
}

#[test]
fn the_highest_picture_by_code() {
    let mut raw = raw(&map());
    raw.terrain.tiles[100].river = 2;
    raw.terrain.tiles[100].river_dir = 9;
    raw.terrain.tiles[101].river = 2;
    raw.terrain.tiles[101].river_dir = 4;
    let art = MapArt::from_raw(&raw).unwrap();
    assert_eq!(art.pictures(Layer::River), [(2, 9)]);
    assert_eq!(art.pictures(Layer::Road), [(1, 15)]);
    let terrains = art.pictures(Layer::Terrain);
    let codes: Vec<u8> = terrains.iter().map(|&(c, _)| c).collect();
    assert_eq!(codes, [2, 8]);
    // The writer's interior pictures: grass 49 + 0 … 7, water 21 + 0 … 7.
    assert_eq!(terrains, [(2, 56), (8, 28)]);
}

#[test]
fn refuses_what_the_raw_model_leaves_open() {
    let good = raw(&map());
    let mut short = good.clone();
    short.terrain.tiles.pop();
    let err = MapArt::from_raw(&short).unwrap_err();
    assert_eq!(
        err,
        ArtError::Tiles {
            expected: 1296,
            found: 1295
        }
    );
    assert_eq!(err.to_string(), "the terrain has 1295 tiles, not 1296");
    for name in [&b""[..], b"a b.def", b"\xe4.def"] {
        let mut bad = good.clone();
        bad.templates[1].def.0 = name.to_vec();
        let err = MapArt::from_raw(&bad).unwrap_err();
        assert_eq!(err, ArtError::Name(1));
        assert_eq!(
            err.to_string(),
            "template 1: its DEF's name is not printable ASCII"
        );
    }
    let mut missing = good;
    missing.objects[2].template = 3;
    let err = MapArt::from_raw(&missing).unwrap_err();
    assert_eq!(err, ArtError::Template(2));
    assert_eq!(err.to_string(), "object 2: no such template");
}

impl TemplateArt {
    /// The art of a writer's template.
    pub(crate) fn from_writer(t: &ObjectTemplate) -> Self {
        Self {
            def: t.def.clone(),
            passable: t.passable,
            visitable: t.visitable,
            class: t.class,
            subclass: t.subclass,
            priority: t.priority,
        }
    }
}

#[test]
fn the_coast_water_of_a_shore_picture_or_land_of_the_coast_bit() {
    let tile = |code: u8, picture: u8, flags: u8| TileArt {
        terrain: Piece {
            code,
            picture,
            mirror_x: false,
            mirror_y: false,
        },
        river: None,
        road: None,
        flags,
    };
    assert!(tile(WATER, 19, 0).coast());
    assert!(!tile(WATER, 20, COAST_BIT).coast());
    assert!(tile(2, 30, COAST_BIT).coast());
    assert!(!tile(2, 0, 0).coast());
}
