use super::*;

use h3_map::model::{RandomDwelling, Template, Text};

use crate::art::tests::{map, raw};

fn pos(x: u8, y: u8) -> Pos {
    Pos { x, y, z: 0 }
}

#[test]
fn a_maps_start_its_objects_owners_and_heroes() {
    let raw = raw(&map());
    let art = MapArt::from_raw(&raw).unwrap();
    let live = Live::at_start(&raw, &art, 1);
    assert_eq!(live.viewer, 1);
    let owners: Vec<Option<u8>> = live.objects.iter().map(|o| o.owner).collect();
    assert_eq!(owners, [Some(0), None, Some(1)]);
    assert!(
        live.objects
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
    let live = Live::at_start(&raw, &art, 0);
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
    assert!(Live::at_start(&raw, &art, 0).heroes.is_empty());
    let mut big = raw.clone();
    let Body::Hero(hero) = &mut big.objects[0].body else {
        panic!("object 0 is the hero");
    };
    hero.owner = 3;
    big.templates[0].subclass = 256;
    let art = MapArt::from_raw(&big).unwrap();
    assert!(Live::at_start(&big, &art, 0).heroes.is_empty());
    big.templates[0].subclass = 5;
    big.objects[0].pos.x = 0;
    let art = MapArt::from_raw(&big).unwrap();
    assert!(Live::at_start(&big, &art, 0).heroes.is_empty());
    big.objects[0].pos = pos(5, 0);
    big.templates[0].visitable = [0; 6];
    let art = MapArt::from_raw(&big).unwrap();
    assert!(Live::at_start(&big, &art, 0).heroes.is_empty());
    big.templates[0].visitable = [0, 0, 0, 0, 0x40, 0];
    big.objects[0].pos = pos(5, 0);
    let art = MapArt::from_raw(&big).unwrap();
    assert!(Live::at_start(&big, &art, 0).heroes.is_empty());
    big.objects[0].pos = pos(5, 3);
    let art = MapArt::from_raw(&big).unwrap();
    let heroes = Live::at_start(&big, &art, 0).heroes;
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
        let live = Live::at_start(&raw, &art, 0);
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
            frames: 5
        }
    );
    assert_eq!(
        Moment::at(1079, 200, 180),
        Moment {
            cycles: 5,
            frames: 5
        }
    );
    assert_eq!(Moment::at(0, 200, 180), Moment::default());
    // A step of 0 never comes.
    assert_eq!(Moment::at(5000, 0, 0), Moment::default());
}
