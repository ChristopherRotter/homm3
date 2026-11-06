//! The experiment writer's teleporters (E08 T99a: batch E11's maps; survey
//! `docs/rules/surveys/E08-teleporters.md` §2, increment 11's design note §16): the one-way
//! monoliths' entrances and exits (43, 44), the two-way monoliths (45) by subclass, the
//! Subterranean Gate (103) and the Whirlpool (111), on either level, read back as written without a
//! body; an empty boat on a Whirlpool's tile; and the import as it reads them until E08 T103:
//! blockers without a type, the boat the map's.

use super::import::SOD;
use super::limits;
use crate::import::codes;
use crate::model::{Body, RawMap};
use crate::writer as w;
use crate::{import, read};
use h3_adventure::{BoatSetup, Pos, TileMask};

/// The game's `Objects.txt` lines of the classes written here: a one-way entrance and exit of
/// channel 0 (blue), two-way monoliths of channels 0 (green; the anchor visited) and 7 (SoD's;
/// visited one left of the anchor), the gate, the Whirlpool and a boat.
const LINES: [&str; 7] = [
    "AVXmn1b0.def 011111111111111111111111111111111111111111111111 \
     100000000000000000000000000000000000000000000000 011111111 011111111 43 0 0 0",
    "AVXmx1b0.def 011111111111111111111111111111111111111111111111 \
     100000000000000000000000000000000000000000000000 011111111 011111111 44 0 0 0",
    "AVXmn2g0.def 011111111111111111111111111111111111111111111111 \
     100000000000000000000000000000000000000000000000 011111111 011111111 45 0 0 0",
    "AVXmn8b0.def 001111111111111111111111111111111111111111111111 \
     010000000000000000000000000000000000000000000000 011111111 011111111 45 7 0 0",
    "AvTCave.def 000111111011111111111111111111111111111111111111 \
     010000000000000000000000000000000000000000000000 011111111 011111111 103 0 0 0",
    "AVXwhrl0.def 000111110001111111111111111111111111111111111111 \
     111000001110000000000000000000000000000000000000 100000000 100000000 111 0 0 1",
    "AVXboat0.def 101111111111111111111111111111111111111111111111 \
     010000000000000000000000000000000000000000000000 100000000 100000000 8 0 0 0",
];

fn template(line: &str) -> w::ObjectTemplate {
    w::ObjectTemplate::from_objects_txt(line).unwrap()
}

/// A 36 × 36 map of two levels, grass above and subterranean below, columns 20 to 35 water on
/// both: a hero at (2, 2); on the surface a one-way entrance at (4, 4) and its exit at (4, 8), the
/// two-way monoliths at (8, 4) and (8, 8), a gate at (12, 4) (its entrance (11, 4)), a Whirlpool at
/// (24, 10) (its six tiles 22 to 24 × 9 and 10) and after it an empty boat whose tile (24, 10) is
/// the Whirlpool's anchor; underground a second exit at (4, 4), the two monoliths' pairs at (8, 4)
/// and (8, 8), a gate at (12, 6) and a Whirlpool at (24, 20).
fn teleporters_map() -> w::Map {
    let [entrance, exit, two_way, two_way_7, gate, whirlpool, boat] = LINES.map(template);
    let plain = |tile, template: &w::ObjectTemplate| {
        w::MapObject::Plain(w::Plain {
            tile,
            template: template.clone(),
        })
    };
    let mut players: [Option<w::Player>; w::PLAYERS] = Default::default();
    players[0] = Some(w::Player {
        human: true,
        computer: true,
        towns: 1,
    });
    let water = w::Patch {
        terrain: w::Terrain::Water,
        from: (20, 0),
        to: (35, 35),
    };
    w::Map {
        name: "Teleporters".into(),
        description: "E08 T99a".into(),
        size: 36,
        terrain: w::Terrain::Grass,
        patches: vec![water],
        roads: vec![],
        players,
        objects: vec![
            w::MapObject::Hero(w::Hero {
                tile: (2, 2),
                owner: 0,
                hero_type: 0,
                class: 0,
                name: None,
                experience: None,
                skills: None,
                army: Some([Some((1, 5)), None, None, None, None, None, None]),
                tight_formation: false,
                artifacts: None,
                spells: None,
                primary: None,
            }),
            plain((4, 4), &entrance),
            plain((4, 8), &exit),
            plain((8, 4), &two_way),
            plain((8, 8), &two_way_7),
            plain((12, 4), &gate),
            plain((24, 10), &whirlpool),
            plain((25, 10), &boat),
        ],
        underground: Some(w::Underground {
            terrain: w::Terrain::Subterranean,
            patches: vec![water],
            objects: vec![
                plain((4, 4), &exit),
                plain((8, 4), &two_way),
                plain((8, 8), &two_way_7),
                plain((12, 6), &gate),
                plain((24, 20), &whirlpool),
            ],
        }),
        level_limit: 0,
        banned_skills: Vec::new(),
        events: Vec::new(),
        teams: None,
    }
}

fn written() -> RawMap {
    read(&w::gzip(&w::write(&teleporters_map()).unwrap()), &limits()).unwrap()
}

#[test]
fn the_teleporters_read_back_as_written() {
    let m = written();
    let placed: Vec<(u32, u32, (u8, u8, u8))> = m
        .objects
        .iter()
        .map(|o| {
            let t = &m.templates[usize::try_from(o.template).unwrap()];
            (t.class, t.subclass, (o.pos.x, o.pos.y, o.pos.z))
        })
        .collect();
    assert_eq!(
        placed,
        [
            (34, 0, (3, 2, 0)),
            (43, 0, (4, 4, 0)),
            (44, 0, (4, 8, 0)),
            (45, 0, (8, 4, 0)),
            (45, 7, (8, 8, 0)),
            (103, 0, (12, 4, 0)),
            (111, 0, (24, 10, 0)),
            (8, 0, (25, 10, 0)),
            (44, 0, (4, 4, 1)),
            (45, 0, (8, 4, 1)),
            (45, 7, (8, 8, 1)),
            (103, 0, (12, 6, 1)),
            (111, 0, (24, 20, 1)),
        ]
    );
    assert!(m.objects[1..].iter().all(|o| o.body == Body::None));
    // The templates as the game's: a monolith of channels 0 to 3 visited on its anchor, one of 4
    // to 7 one left of it with two blocked, the gate one left with four blocked, the Whirlpool's
    // six tiles all blocked and visitable, on water only.
    let masks = |i: usize| {
        let t = &m.templates[usize::try_from(m.objects[i].template).unwrap()];
        (
            t.passable[4],
            t.passable[5],
            t.visitable[4],
            t.visitable[5],
            t.terrains,
        )
    };
    assert_eq!(masks(1), (0xff, 0x7f, 0x00, 0x80, 0x0ff));
    assert_eq!(masks(4), (0xff, 0x3f, 0x00, 0x40, 0x0ff));
    assert_eq!(masks(5), (0xbf, 0x1f, 0x00, 0x40, 0x0ff));
    assert_eq!(masks(6), (0x1f, 0x1f, 0xe0, 0xe0, 0x100));
}

// covers: ADV-OBJ-002
#[test]
fn the_teleporters_import_as_blockers_until_their_kinds() {
    // The import's class table gives the five classes no type yet (E08 T103a adds the kinds).
    for (class, subclasses) in [(43, 8), (44, 8), (45, 8), (103, 1), (111, 1)] {
        for subclass in 0..subclasses {
            assert_eq!(
                codes::object_type(class, subclass),
                None,
                "{class}.{subclass}"
            );
        }
    }
    // The boat on the Whirlpool's anchor tile is the map's boat, no object; the setup refuses it
    // while the Whirlpool is a blocker (a boat on a tile an object blocks; no shipped map holds a
    // boat on a teleporter, survey §1). Batch E11 needs it only in the original.
    let setup = import(&written(), &SOD).unwrap();
    assert_eq!(
        setup.map.boats,
        [BoatSetup {
            tile: Pos::new(24, 10, 0),
            kind: 0
        }]
    );
    assert_eq!(
        setup.validate(&SOD),
        Err(h3_game::GameSetupError::Adventure(
            h3_adventure::SetupError::Boat(0)
        ))
    );
    // Without it the map validates: each teleporter a blocker on its level with its picture's
    // tiles.
    let mut map = teleporters_map();
    map.objects.pop();
    let m = read(&w::gzip(&w::write(&map).unwrap()), &limits()).unwrap();
    let setup = import(&m, &SOD).unwrap();
    setup.validate(&SOD).unwrap();
    assert!(setup.map.boats.is_empty());
    let objects: Vec<(Pos, Option<_>, TileMask)> = setup
        .map
        .objects
        .iter()
        .map(|o| (o.anchor, o.object_type, o.visitable))
        .collect();
    let at = |x, y, z| Pos::new(x, y, z);
    let (anchor, two_left) = (TileMask::from_bits(1), TileMask::from_bits(2));
    let six = TileMask::from_bits(0x0707);
    assert_eq!(
        objects,
        [
            (at(4, 4, 0), None, anchor),
            (at(4, 8, 0), None, anchor),
            (at(8, 4, 0), None, anchor),
            (at(8, 8, 0), None, two_left),
            (at(12, 4, 0), None, two_left),
            (at(24, 10, 0), None, six),
            (at(4, 4, 1), None, anchor),
            (at(8, 4, 1), None, anchor),
            (at(8, 8, 1), None, two_left),
            (at(12, 6, 1), None, two_left),
            (at(24, 20, 1), None, six),
        ]
    );
}
