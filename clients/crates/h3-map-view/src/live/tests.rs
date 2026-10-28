use super::*;

use h3_map::model::{RandomDwelling, Template, Text};

use crate::art::tests::{map, raw};

/// The committed tables' towns' pictures (`clients/map/`).
fn towns() -> TownArt {
    let text = include_str!("../../../../map/assets/base.toml");
    tables::load(&[tables::parse(text).unwrap()], "base")
        .unwrap()
        .map
        .towns
}

fn pos(x: u8, y: u8) -> Pos {
    Pos { x, y, z: 0 }
}

#[test]
fn a_maps_start_its_objects_owners_and_heroes() {
    let raw = raw(&map());
    let art = MapArt::from_raw(&raw).unwrap();
    let live = Live::at_start(&raw, &art, 1, &towns());
    assert_eq!(live.viewer, 1);
    let owners: Vec<Option<u8>> = live.objects.iter().map(|o| o.owner).collect();
    assert_eq!(owners, [Some(0), None, Some(1)]);
    // Nothing random drawn (the town's picture by its buildings: below).
    assert!(
        live.objects[..2]
            .iter()
            .all(|o| o.standing && o.drawn.is_none() && o.phase == 0)
    );
    // The hero stands one column left of its anchor, on its template's visitable cell.
    assert_eq!(
        live.heroes,
        [LiveHero {
            at: pos(10, 10),
            class: 2,
            owner: 0,
            facing: Facing::Right,
            boat: None,
        }]
    );
    assert!(live.boats.is_empty());
    assert!(live.explored.explored(35, 35, 0));
    assert_eq!(live.moment, Moment::default());
}

/// A boat's template: class 8, visited at its anchor.
fn boat(kind: u32) -> Template {
    Template {
        def: Text(b"avxboat0.def".to_vec()),
        passable: [0xff; 6],
        visitable: [0, 0, 0, 0, 0, 0x80],
        class: BOAT_CLASS,
        subclass: kind,
        ..Template::default()
    }
}

#[test]
fn a_maps_boats_and_the_heroes_it_leaves_out() {
    let mut raw = raw(&map());
    // Object 1 becomes a boat of type 2, object 2 one of no type the pictures know.
    raw.templates.push(boat(2));
    raw.templates.push(boat(300));
    raw.objects[1].template = 3;
    raw.objects[2].template = 4;
    let art = MapArt::from_raw(&raw).unwrap();
    let live = Live::at_start(&raw, &art, 0, &towns());
    assert_eq!(
        live.boats,
        [LiveBoat {
            at: pos(20, 20),
            kind: 2,
            facing: Facing::Right
        }]
    );
    // A hero without an owner, of a class past a byte, or whose cell lies off the map's
    // coordinates is left out; so is one without a visitable cell.
    let Body::Hero(hero) = &mut raw.objects[0].body else {
        panic!("object 0 is the hero");
    };
    hero.owner = 0xFF;
    let art = MapArt::from_raw(&raw).unwrap();
    assert!(Live::at_start(&raw, &art, 0, &towns()).heroes.is_empty());
    let mut big = raw.clone();
    let Body::Hero(hero) = &mut big.objects[0].body else {
        panic!("object 0 is the hero");
    };
    hero.owner = 3;
    big.templates[0].subclass = 256;
    let art = MapArt::from_raw(&big).unwrap();
    assert!(Live::at_start(&big, &art, 0, &towns()).heroes.is_empty());
    big.templates[0].subclass = 5;
    big.objects[0].pos.x = 0;
    let art = MapArt::from_raw(&big).unwrap();
    assert!(Live::at_start(&big, &art, 0, &towns()).heroes.is_empty());
    big.objects[0].pos = pos(5, 0);
    big.templates[0].visitable = [0; 6];
    let art = MapArt::from_raw(&big).unwrap();
    assert!(Live::at_start(&big, &art, 0, &towns()).heroes.is_empty());
    big.templates[0].visitable = [0, 0, 0, 0, 0x40, 0];
    big.objects[0].pos = pos(5, 0);
    let art = MapArt::from_raw(&big).unwrap();
    assert!(Live::at_start(&big, &art, 0, &towns()).heroes.is_empty());
    big.objects[0].pos = pos(5, 3);
    let art = MapArt::from_raw(&big).unwrap();
    let heroes = Live::at_start(&big, &art, 0, &towns()).heroes;
    assert_eq!(
        heroes
            .iter()
            .map(|h| (h.at, h.class, h.owner))
            .collect::<Vec<_>>(),
        [(pos(4, 2), 5, 3)]
    );
}

#[test]
fn the_owners_every_body_gives() {
    let mut raw = raw(&map());
    let bodies = [
        (Body::Mine { owner: 4 }, Some(4)),
        (Body::Owned { owner: 7 }, Some(7)),
        (Body::Owned { owner: 8 }, None),
        (Body::Owned { owner: 0xFF }, None),
        (Body::Owned { owner: 0x1_0000 }, None),
        (
            Body::Garrison {
                owner: 2,
                army: Vec::new(),
                removable: true,
            },
            Some(2),
        ),
        (
            Body::RandomDwelling(RandomDwelling {
                owner: 6,
                ..RandomDwelling::default()
            }),
            Some(6),
        ),
        (
            Body::HeroPlaceholder {
                owner: 5,
                hero_type: 0xFF,
                power_rank: Some(1),
            },
            Some(5),
        ),
        (Body::None, None),
    ];
    let art = MapArt::from_raw(&raw).unwrap();
    for (body, owner) in bodies {
        raw.objects[1].body = body.clone();
        let live = Live::at_start(&raw, &art, 0, &towns());
        assert_eq!(live.objects[1].owner, owner, "{body:?}");
    }
}

#[test]
fn the_facings_numbers_and_mirrors() {
    for (n, f) in Facing::ALL.iter().enumerate() {
        assert_eq!(usize::from(f.number()), n);
        assert_eq!(Facing::from_number(f.number()), Some(*f));
        assert_eq!(f.mirrored(), n > 4);
    }
    assert_eq!(Facing::from_number(8), None);
}

#[test]
fn the_explored_tiles() {
    let mut explored = Explored::all(4, 2, false);
    assert!(!explored.explored(1, 2, 1));
    explored.set(1, 2, 1, true);
    assert!(explored.explored(1, 2, 1));
    assert!(!explored.explored(1, 2, 0));
    // Off the map: never explored, and setting it changes nothing.
    let before = explored.clone();
    for (x, y, z) in [(-1, 0, 0), (0, -1, 0), (4, 0, 0), (0, 4, 0), (0, 0, 2)] {
        explored.set(x, y, z, true);
        assert!(!explored.explored(x, y, z), "({x}, {y}, {z})");
    }
    assert_eq!(explored, before);
    assert!(Explored::all(4, 2, true).explored(3, 3, 1));
}

#[test]
fn the_moment_at_a_time() {
    // 1000 ms: five 200-ms cycle steps, five whole 180-ms frame steps.
    assert_eq!(
        Moment::at(1000, 200, 180),
        Moment {
            cycles: 5,
            river_cycles: 5,
            frames: 5
        }
    );
    assert_eq!(
        Moment::at(1079, 200, 180),
        Moment {
            cycles: 5,
            river_cycles: 5,
            frames: 5
        }
    );
    assert_eq!(Moment::at(0, 200, 180), Moment::default());
    // A step of 0 never comes.
    assert_eq!(Moment::at(5000, 0, 0), Moment::default());
}

#[test]
fn every_town_its_picture_by_its_buildings_at_the_start() {
    use h3_map::model::TownBody;
    // The art map's Blue town of Castle (its template `avccast0.def`) has the standard set with a
    // Fort: the fort's picture.
    let mut raw = raw(&map());
    let art = MapArt::from_raw(&raw).unwrap();
    let town = 2;
    let drawn = |raw: &RawMap| {
        Live::at_start(raw, &art, 0, &towns()).objects[town]
            .drawn
            .clone()
    };
    assert_eq!(
        drawn(&raw),
        Some(Drawn {
            def: "AVCcasx0.def".into(),
            own: None,
            class: TOWN_CLASS,
            subclass: 0,
        })
    );
    let Body::Town(body) = &raw.objects[town].body else {
        panic!("the art map's third object is its town");
    };
    let body: TownBody = (**body).clone();
    let with = |buildings: Buildings| {
        Body::Town(alloc::boxed::Box::new(TownBody {
            buildings,
            ..body.clone()
        }))
    };
    let custom = |byte: u8| Buildings::Custom {
        built: [byte, 0, 0, 0, 0, 0],
        forbidden: [0; 6],
    };
    // Without a Fort: the village's picture, the template's own (no copy drawn differently).
    raw.objects[town].body = with(Buildings::Standard { fort: false });
    assert_eq!(drawn(&raw), None);
    // The map's buildings 3, 4, 5 (Fort, Citadel, Castle): the fort's; 2 (Capitol): the
    // capitol's; the rest (Town Hall, 0) the village's.
    for (byte, def) in [
        (1 << 3, Some("AVCcasx0.def")),
        (1 << 4, Some("AVCcasx0.def")),
        (1 << 5, Some("AVCcasx0.def")),
        (1 << 2 | 1 << 3, Some("AVCcasz0.def")),
        (1, None),
    ] {
        raw.objects[town].body = with(custom(byte));
        assert_eq!(
            drawn(&raw).map(|d| d.def),
            def.map(String::from),
            "{byte:#x}"
        );
    }
    // A random town (class 77) waits for its draw; a body that is not a town's draws none.
    let mut random = raw.clone();
    random.templates[usize::try_from(random.objects[town].template).unwrap()].class = 77;
    random.objects[town].body = with(custom(1 << 3));
    let art77 = MapArt::from_raw(&random).unwrap();
    assert_eq!(
        Live::at_start(&random, &art77, 0, &towns()).objects[town].drawn,
        None
    );
    raw.objects[town].body = Body::None;
    assert_eq!(drawn(&raw), None);
}

#[test]
fn a_maps_start_its_kingdom_and_buttons() {
    // E24 M3 T2a (design §4.1): the art map's Red hero and Blue town.
    let raw = raw(&map());
    let art = MapArt::from_raw(&raw).unwrap();
    let kingdom = |viewer| Live::at_start(&raw, &art, viewer, &towns()).kingdom;
    let awake = KingdomHero {
        asleep: false,
        movable: true,
    };
    let red = kingdom(0);
    assert_eq!(
        red,
        Kingdom {
            heroes: vec![awake],
            towns: 0,
            selected: Selected::Hero(0),
            tops: [0, 0],
            quests: false,
        }
    );
    let blue = kingdom(1);
    assert_eq!((blue.heroes.len(), blue.towns), (0, 1));
    assert_eq!(blue.selected, Selected::Town(0));
    assert_eq!(kingdom(2), Kingdom::default());
    assert!(Live::at_start(&raw, &art, 0, &towns()).buttons.is_none());
    // Move hero by a green arrow of the selected hero's path, next hero by an awake hero able to
    // move.
    let arrow = |arrow| Arrow {
        at: pos(1, 1),
        arrow,
    };
    let buttons = |kingdom: &Kingdom, path: &[Arrow]| Buttons::of(kingdom, path);
    assert_eq!(
        buttons(&red, &[arrow(30), arrow(25)]),
        Buttons {
            move_hero: true,
            next_hero: true,
        }
    );
    assert!(!buttons(&red, &[arrow(26)]).move_hero);
    assert!(!buttons(&red, &[arrow(0)]).move_hero);
    assert_eq!(
        buttons(&blue, &[arrow(1)]),
        Buttons {
            move_hero: false,
            next_hero: false,
        }
    );
    let tired = Kingdom {
        heroes: vec![
            KingdomHero {
                asleep: true,
                movable: true,
            },
            KingdomHero {
                asleep: false,
                movable: false,
            },
        ],
        ..red
    };
    assert!(!tired.next_hero());
}

#[test]
fn a_live_parts_kingdom_read() {
    // E24 M3 T2a: `h3mem.py`'s `Mem.kingdom` as the harness writes it.
    let read = |text: &str| {
        let table: toml::Table = toml::from_str(&alloc::format!("kingdom = {text}")).unwrap();
        Kingdom::read(&table["kingdom"])
    };
    let all =
        "{ heroes = [[0, 1], [1, 0]], towns = 2, selected = [1, 1], tops = [0, 1], quests = 1 }";
    assert_eq!(
        read(all),
        Some(Kingdom {
            heroes: vec![
                KingdomHero {
                    asleep: false,
                    movable: true,
                },
                KingdomHero {
                    asleep: true,
                    movable: false,
                },
            ],
            towns: 2,
            selected: Selected::Hero(1),
            tops: [0, 1],
            quests: true,
        })
    );
    let selected = |s: &str| read(&all.replace("selected = [1, 1]", s)).map(|k| k.selected);
    assert_eq!(selected("selected = [0, 9]"), Some(Selected::None));
    assert_eq!(selected("selected = [2, 1]"), Some(Selected::Town(1)));
    for wrong in [
        "selected = [1, 2]",
        "selected = [2, 2]",
        "selected = [3, 0]",
        "selected = [1]",
        "selected = [-1, 0]",
    ] {
        assert_eq!(selected(wrong), None, "{wrong}");
    }
    for (from, to) in [
        ("[[0, 1], [1, 0]]", "[[0, 2]]"),
        ("[[0, 1], [1, 0]]", "[[0]]"),
        ("[[0, 1], [1, 0]]", "[[0, \"1\"]]"),
        ("[[0, 1], [1, 0]]", "3"),
        ("towns = 2", "towns = -1"),
        ("tops = [0, 1]", "tops = [0]"),
        ("tops = [0, 1]", "tops = [0, -1]"),
        ("quests = 1", "quests = 2"),
        ("quests = 1", "missing = 1"),
    ] {
        assert_eq!(read(&all.replace(from, to)), None, "{to}");
    }
    let table: toml::Table = toml::from_str("kingdom = 1").unwrap();
    assert_eq!(Kingdom::read(&table["kingdom"]), None);
}
