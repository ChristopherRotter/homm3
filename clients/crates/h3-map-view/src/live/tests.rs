use super::*;
use alloc::string::ToString;

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
    // E24 M3 T2a (design §4.1): the art map's Red hero and Blue town; their portrait (the hero's
    // type, 16) and the town's faction and fort (T4a, §4.5).
    let mut raw = raw(&map());
    let art = MapArt::from_raw(&raw).unwrap();
    let kingdom = |raw: &RawMap, viewer| Live::at_start(raw, &art, viewer, &towns()).kingdom;
    let awake = KingdomHero {
        asleep: false,
        movable: true,
        portrait: 16,
        points: 0,
        mana: 0,
    };
    let red = kingdom(&raw, 0);
    assert_eq!(
        red,
        Kingdom {
            heroes: vec![awake],
            towns: vec![],
            selected: Selected::Hero(0),
            tops: [0, 0],
            quests: false,
            // T5a: no resources known, the first day.
            resources: [0; 7],
            date: Date {
                month: 1,
                week: 1,
                day: 1,
            },
            // T5b1 (design §4.8): the selected hero's window, named by its type, nothing else
            // known.
            status: Some(Status::Hero(HeroStatus {
                hero: 16,
                own_name: false,
                skills: [0; 4],
                morale: 0,
                luck: 0,
                army: [None; 7],
            })),
            selection: None,
        }
    );
    let blue = kingdom(&raw, 1);
    let castle = KingdomTown {
        faction: 0,
        fort: true,
        built: false,
    };
    assert_eq!((blue.heroes.len(), &blue.towns), (0, &vec![castle]));
    assert_eq!(blue.selected, Selected::Town(0));
    // The selected town's window: its faction's first name, its Fort standing; nothing of the
    // viewer's, nothing selected, no window.
    assert_eq!(
        blue.status,
        Some(Status::Town(TownStatus {
            name: 0,
            buildings: 1 << 7,
            garrison: false,
            army: [None; 7],
            silo: Vec::new(),
            income: 0,
        }))
    );
    // A colour with neither heroes nor towns: the date's window (the chooser's kind 0 without a
    // selection, E24 M3 T5b2) at its first frame.
    assert_eq!(
        kingdom(&raw, 5).status,
        Some(Status::Date(DateStatus { frame: 0 }))
    );
    // A portrait the map gives; a town without a fort.
    let hero = raw
        .objects
        .iter()
        .position(|o| matches!(o.body, Body::Hero(_)))
        .unwrap();
    if let Body::Hero(h) = &mut raw.objects[hero].body {
        h.portrait = Some(130);
    }
    let town = raw
        .objects
        .iter()
        .position(|o| matches!(o.body, Body::Town(_)))
        .unwrap();
    if let Body::Town(t) = &mut raw.objects[town].body {
        t.buildings = Buildings::Standard { fort: false };
    }
    assert_eq!(kingdom(&raw, 0).heroes[0].portrait, 130);
    assert!(!kingdom(&raw, 1).towns[0].fort);
    let raw = self::raw(&map());
    // Nothing of a colour's: the kingdom's defaults but the date's window (E24 M3 T5b2).
    assert_eq!(
        kingdom(&raw, 2),
        Kingdom {
            status: Some(Status::Date(DateStatus { frame: 0 })),
            ..Kingdom::default()
        }
    );
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
                ..awake
            },
            KingdomHero {
                movable: false,
                ..awake
            },
        ],
        ..red
    };
    assert!(!tired.next_hero());
}

#[test]
fn a_live_parts_kingdom_read() {
    // E24 M3 T2a: `h3mem.py`'s `Mem.kingdom` as the harness writes it (T5a: with the resources
    // and the date).
    let read = |text: &str| {
        let table: toml::Table = toml::from_str(&alloc::format!("kingdom = {text}")).unwrap();
        Kingdom::read(&table["kingdom"])
    };
    let all = "{ heroes = [[0, 1, 3, 1500, 20], [1, 0, 162, 0, -1]], towns = [[8, 1, 0], [0, 0, \
               1]], selected = [1, 1], tops = [0, 1], quests = 1, resources = [0, 1, 2, 3, 4, 5, \
               -6], date = [12, 4, 7] }";
    let tail = "resources = [0, 0, 0, 0, 0, 0, 0], date = [1, 1, 1]";
    let two = Kingdom {
        status: None,
        selection: None,
        heroes: vec![
            KingdomHero {
                asleep: false,
                movable: true,
                portrait: 3,
                points: 1500,
                mana: 20,
            },
            KingdomHero {
                asleep: true,
                movable: false,
                portrait: 162,
                points: 0,
                mana: -1,
            },
        ],
        towns: vec![
            KingdomTown {
                faction: 8,
                fort: true,
                built: false,
            },
            KingdomTown {
                faction: 0,
                fort: false,
                built: true,
            },
        ],
        selected: Selected::Hero(1),
        tops: [0, 1],
        quests: true,
        resources: [0, 1, 2, 3, 4, 5, -6],
        date: Date {
            month: 12,
            week: 4,
            day: 7,
        },
    };
    assert_eq!(read(all), Some(two));
    // The form before T4a (E24 M3 T4b: no longer read).
    let before = alloc::format!(
        "{{ heroes = [[0, 1], [1, 0]], towns = 2, selected = [2, 1], tops = [0, 1], quests = 1, \
         {tail} }}"
    );
    assert_eq!(read(&before), None);
    let short_hero = alloc::format!(
        "{{ heroes = [[0, 1]], towns = [], selected = [0, 0], tops = [0, 0], quests = 0, {tail} }}"
    );
    assert_eq!(read(&short_hero), None);
    let flag = alloc::format!(
        "{{ heroes = [], towns = [], selected = [0, 0], tops = [0, 0], quests = \"no\", {tail} }}"
    );
    assert_eq!(read(&flag), None);
    let empty = alloc::format!(
        "{{ heroes = [], towns = [], selected = [0, 0], tops = [0, 0], quests = 0, {tail} }}"
    );
    assert_eq!(read(&empty), Some(Kingdom::default()));
    for towns in ["2", "[3]", "[[0, 1]]"] {
        let text = alloc::format!(
            "{{ heroes = [], towns = {towns}, selected = [0, 0], tops = [0, 0], quests = 0, \
             {tail} }}"
        );
        assert_eq!(read(&text), None, "{towns}");
    }
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
    let heroes = "[[0, 1, 3, 1500, 20], [1, 0, 162, 0, -1]]";
    let towns = "[[8, 1, 0], [0, 0, 1]]";
    for (from, to) in [
        (heroes, "[[0, 2]]"),
        (heroes, "[[0]]"),
        (heroes, "[[0, \"1\"]]"),
        (heroes, "3"),
        (heroes, "[3]"),
        (heroes, "[[0, 1, 256, 0, 0]]"),
        (heroes, "[[0, 1, 0, 2147483648, 0]]"),
        (heroes, "[[0, 1, 0, 0, 2147483648]]"),
        (heroes, "[[0, 1, 0, 0]]"),
        (heroes, "[[0, 1, 0, 0, 0, 0]]"),
        (towns, "-1"),
        (towns, "[[9, 2, 0]]"),
        (towns, "[[9, 0, 2]]"),
        (towns, "[[256, 0, 0]]"),
        (towns, "[[0, 0]]"),
        (towns, "[3]"),
        (towns, "\"two\""),
        ("tops = [0, 1]", "tops = [0]"),
        ("tops = [0, 1]", "tops = [0, -1]"),
        ("quests = 1", "quests = 2"),
        ("quests = 1", "missing = 1"),
        (heroes, "[[2, 1, 0, 0, 0]]"),
        (heroes, "[[0, 2, 0, 0, 0]]"),
        (heroes, "[[2, 1]]"),
        ("heroes = ", "people = "),
        ("towns = ", "cities = "),
        ("selected = [1, 1]", "chosen = [1, 1]"),
        ("selected = [1, 1]", "selected = 1"),
        ("selected = [1, 1]", "selected = [1, -1]"),
        ("tops = [0, 1]", "top = [0, 1]"),
        ("tops = [0, 1]", "tops = [-1, 1]"),
        // T5a: seven amounts of 32 bits; three parts of the date of 16 bits each.
        ("resources = ", "wealth = "),
        ("-6]", "-6, 7]"),
        ("[0, 1, 2, 3, 4, 5, ", "[0, 1, 2, 3, 4, "),
        ("-6]", "-2147483649]"),
        ("-6]", "\"six\"]"),
        ("resources = [", "resources = 3, x = ["),
        ("date = ", "day = "),
        ("[12, 4, 7]", "[12, 4]"),
        ("[12, 4, 7]", "[12, 4, 7, 1]"),
        ("[12, 4, 7]", "[12, 4, 65536]"),
        ("[12, 4, 7]", "[12, -1, 7]"),
        ("[12, 4, 7]", "[-12, 4, 7]"),
        ("[12, 4, 7]", "12"),
    ] {
        assert_eq!(read(&all.replace(from, to)), None, "{to}");
    }
    let table: toml::Table = toml::from_str("kingdom = 1").unwrap();
    assert_eq!(Kingdom::read(&table["kingdom"]), None);
}

#[test]
fn a_live_parts_status_read() {
    // E24 M3 T5b1 (design §4.8): `h3map.py`'s `status_toml`.
    let read = |text: &str| {
        let table: toml::Table = toml::from_str(&alloc::format!("status = {text}")).unwrap();
        Status::read(&table["status"])
    };
    let army = "army = [[42, 22], [-1, 0], [-1, 0], [-1, 0], [-1, 0], [-1, 0], [149, 123456]]";
    let slots = [
        Some(Stack {
            creature: 42,
            count: 22,
        }),
        None,
        None,
        None,
        None,
        None,
        Some(Stack {
            creature: 149,
            count: 123_456,
        }),
    ];
    let hero = alloc::format!(
        "{{ kind = 3, hero = 48, own_name = 1, skills = [2, -2, 1, 127], morale = -3, luck = 3, \
         {army} }}"
    );
    assert_eq!(
        read(&hero),
        Some(Status::Hero(HeroStatus {
            hero: 48,
            own_name: true,
            skills: [2, -2, 1, 127],
            morale: -3,
            luck: 3,
            army: slots,
        }))
    );
    let town = alloc::format!(
        "{{ kind = 4, name = 37, buildings = [0, 7, 63], garrison = 1, {army}, silo = [0, 2], \
         income = 1500 }}"
    );
    assert_eq!(
        read(&town),
        Some(Status::Town(TownStatus {
            name: 37,
            buildings: 1 | 1 << 7 | 1 << 63,
            garrison: true,
            army: slots,
            silo: vec![0, 2],
            income: 1500,
        }))
    );
    // The date's and the kingdom's (E24 M3 T5b2), their numbers; another kind by its number.
    let date = "{ kind = 1, frame = 9 }";
    assert_eq!(read(date), Some(Status::Date(DateStatus { frame: 9 })));
    let realm = "{ kind = 2, halls = [0, 2, 1, -1], allies = [0, 2], enemies = [7] }";
    let kingdom_status = KingdomStatus {
        halls: [0, 2, 1, -1],
        allies: vec![0, 2],
        enemies: vec![7],
    };
    assert_eq!(read(realm), Some(Status::Kingdom(kingdom_status.clone())));
    assert_eq!(read("{ kind = 5 }"), Some(Status::Other(5)));
    for (status, kind) in [
        (Status::Date(DateStatus { frame: 0 }), 1),
        (Status::Kingdom(kingdom_status), 2),
        (read(&hero).unwrap(), 3),
        (read(&town).unwrap(), 4),
        (Status::Other(6), 6),
    ] {
        assert_eq!(status.kind(), kind);
    }
    // Refused: a wrong kind or field, a slot or a list of the wrong length, a building past 63,
    // a garrison not 0 or 1, a number past its type; a colour past 7, four halls but not.
    for wrong in [
        "{ kind = 1, frame = -1 }".to_string(),
        realm.replace("[0, 2, 1, -1]", "[0, 2, 1]"),
        realm.replace("[0, 2, 1, -1]", "[0, 2, 1, 3000000000]"),
        realm.replace("[0, 2]", "[0, 8]"),
        realm.replace("[7]", "[-1]"),
        realm.replace("[7]", "7"),
        "{ kind = -1 }".to_string(),
        "{ kind = 3 }".to_string(),
        "[3]".to_string(),
        hero.replace("[2, -2, 1, 127]", "[2, -2, 1]"),
        hero.replace("127]", "128]"),
        hero.replace("own_name = 1", "own_name = 2"),
        hero.replace("[[42, 22], ", "[[42], "),
        hero.replace("[[42, 22], [-1, 0], ", "[[-1, 0], "),
        hero.replace("[149, 123456]", "[70000, 1]"),
        town.replace("63]", "64]"),
        town.replace("garrison = 1", "garrison = 2"),
        town.replace("silo = [0, 2]", "silo = [0, 256]"),
        town.replace("name = 37", "name = -1"),
        hero.replace("[149, 123456]", "[149, 3000000000]"),
        hero.replace("[149, 123456]", "[\"x\", 1]"),
        hero.replace("[149, 123456]", "[149, \"x\"]"),
        hero.replace("[149, 123456]", "149"),
        hero.replace("hero = 48", "hero = 300"),
        town.replace("[0, 7, 63]", "[0, \"x\"]"),
        town.replace("income = 1500", "income = 3000000000"),
    ] {
        assert_eq!(read(&wrong), None, "{wrong}");
    }
    // Every field missing or a text: refused; the flags 0 too.
    let fields = |text: &str| -> toml::Table {
        let table: toml::Table = toml::from_str(&alloc::format!("s = {text}")).unwrap();
        table["s"].as_table().unwrap().clone()
    };
    let flags = ["own_name", "garrison"];
    for whole in [fields(&hero), fields(&town), fields(date), fields(realm)] {
        for key in whole.keys() {
            let mut without = whole.clone();
            without.remove(key);
            assert_eq!(Status::read(&toml::Value::Table(without)), None, "no {key}");
            let mut text = whole.clone();
            text.insert(key.clone(), toml::Value::String("x".into()));
            assert_eq!(
                Status::read(&toml::Value::Table(text)),
                None,
                "{key} a text"
            );
        }
        for key in flags.iter().filter(|k| whole.contains_key(**k)) {
            let mut zero = whole.clone();
            zero.insert((*key).to_string(), toml::Value::Integer(0));
            assert!(Status::read(&toml::Value::Table(zero)).is_some(), "{key} 0");
        }
    }
}
