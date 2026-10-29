//! The experiment writer's sea objects (E08 T67: batch E8's maps; survey
//! `docs/rules/surveys/E08-sea.md` §5, CLT-Q5): empty boats, the sea's four pickups, a hole,
//! Favorable Winds, a hero and a monster on water read back as written, and the map imports: since
//! E08 T72 with the boats as the map's boats, the pickups, the hole and Favorable Winds typed, and a
//! hero on water refused (D-44).

use super::import::SOD;
use super::limits;
use crate::model::{Body, RawMap, Text};
use crate::writer as w;
use crate::{ImportError, Unsupported};
use crate::{import, read};
use h3_adventure::{BoatSetup, Pos, TileMask};

/// The game's `Objects.txt` lines of the classes written here (E08 T67): the three boats,
/// Flotsam, the Sea Chest, the Shipwreck Survivor, the Ocean Bottle, a grass hole and Favorable
/// Winds' 3 × 3 picture (`AVXfw5`, its `.msk` 3 × 3).
const LINES: [&str; 9] = [
    "AVXboat0.def 101111111111111111111111111111111111111111111111 \
     010000000000000000000000000000000000000000000000 100000000 100000000 8 0 0 0",
    "AVXboat1.def 101111111111111111111111111111111111111111111111 \
     010000000000000000000000000000000000000000000000 100000000 100000000 8 1 0 0",
    "AVXboat2.def 101111111111111111111111111111111111111111111111 \
     010000000000000000000000000000000000000000000000 100000000 100000000 8 2 0 0",
    "AVAflot0.def 011111111111111111111111111111111111111111111111 \
     100000000000000000000000000000000000000000000000 100000000 100000000 29 0 0 0",
    "AVXccht0.def 011111111111111111111111111111111111111111111111 \
     100000000000000000000000000000000000000000000000 100000000 100000000 82 0 0 0",
    "AVAsurv0.def 011111111111111111111111111111111111111111111111 \
     100000000000000000000000000000000000000000000000 100000000 100000000 86 0 0 0",
    "AVXbttl0.def 011111111111111111111111111111111111111111111111 \
     100000000000000000000000000000000000000000000000 100000000 100000000 59 0 0 0",
    "AVLholg0.def 111111111111111111111111111111111111111111111111 \
     000000000000000000000000000000000000000000000000 111111111 000000100 124 0 0 1",
    "AVXfw5.def 111111111111111111111111111111111111111111111111 \
     000000000000000000000000000000000000000000000000 100000000 100000000 225 0 0 1",
];

/// A Pikeman's wandering-monster template (`Objects.txt`).
const PIKEMAN: &str = "AvWPike.def 011111111111111111111111111111111111111111111111 \
                       100000000000000000000000000000000000000000000000 111111111 000000100 \
                       54 0 2 0";

fn template(line: &str) -> w::ObjectTemplate {
    w::ObjectTemplate::from_objects_txt(line).unwrap()
}

fn hero(tile: (u8, u8), hero_type: u8) -> w::MapObject {
    w::MapObject::Hero(w::Hero {
        tile,
        owner: 0,
        hero_type,
        class: 0,
        name: None,
        experience: None,
        skills: Some(vec![(5, 2)]),
        army: Some([Some((1, 5)), None, None, None, None, None, None]),
        tight_formation: false,
        artifacts: None,
        spells: None,
        primary: None,
    })
}

/// A 36 × 36 grass map whose columns 12 to 35 are water: a hero on the shore at (10, 4) and one
/// on water at (20, 4); the three boats on the coast's water (their visitable tile one column left
/// of the position, as a hero's), the pickups, two bottles (with a text and without), a hole on the
/// shore, Favorable Winds on water and a Pikeman on water.
fn sea_map() -> w::Map {
    let [
        boat0,
        boat1,
        boat2,
        flotsam,
        chest,
        survivor,
        bottle,
        hole,
        winds,
    ] = LINES.map(template);
    let plain = |tile, template| w::MapObject::Plain(w::Plain { tile, template });
    let sign = |tile, text: &str| {
        w::MapObject::Reward(w::RewardObject {
            tile,
            template: bottle.clone(),
            body: w::RewardBody::Sign(text.into()),
        })
    };
    let mut players: [Option<w::Player>; w::PLAYERS] = Default::default();
    players[0] = Some(w::Player {
        human: true,
        computer: true,
        towns: 1,
    });
    w::Map {
        name: "Sea".into(),
        description: "E08 T67".into(),
        size: 36,
        terrain: w::Terrain::Grass,
        patches: vec![w::Patch {
            terrain: w::Terrain::Water,
            from: (12, 0),
            to: (35, 35),
        }],
        roads: vec![],
        players,
        objects: vec![
            w::MapObject::Ground(w::Ground {
                tile: (24, 14),
                template: winds,
            }),
            hero((10, 4), 0),
            hero((20, 4), 1),
            plain((13, 6), boat0),
            plain((13, 8), boat1),
            plain((13, 10), boat2),
            plain((14, 12), flotsam),
            plain((16, 12), chest),
            plain((18, 12), survivor),
            sign((20, 12), "Ahoy"),
            sign((22, 12), ""),
            plain((10, 16), hole),
            w::MapObject::Monster(w::Monster {
                tile: (20, 20),
                template: template(PIKEMAN),
                count: 10,
                disposition: 2,
                never_flees: true,
                fixed_count: false,
                treasure: None,
            }),
        ],
        underground: None,
        level_limit: 0,
        banned_skills: Vec::new(),
    }
}

fn written() -> RawMap {
    read(&w::gzip(&w::write(&sea_map()).unwrap()), &limits()).unwrap()
}

/// The terrain code of the tile at `(x, y)` on the surface.
fn terrain(m: &RawMap, (x, y): (u8, u8)) -> u8 {
    m.terrain.tiles[usize::from(y) * m.terrain.size + usize::from(x)].terrain
}

#[test]
fn the_sea_s_objects_read_back_as_written() {
    let m = written();
    let placed: Vec<(u32, u32, (u8, u8))> = m
        .objects
        .iter()
        .map(|o| {
            let t = &m.templates[usize::try_from(o.template).unwrap()];
            (t.class, t.subclass, (o.pos.x, o.pos.y))
        })
        .collect();
    assert_eq!(
        placed,
        [
            (225, 0, (24, 14)),
            (34, 0, (11, 4)),
            (34, 0, (21, 4)),
            (8, 0, (13, 6)),
            (8, 1, (13, 8)),
            (8, 2, (13, 10)),
            (29, 0, (14, 12)),
            (82, 0, (16, 12)),
            (86, 0, (18, 12)),
            (59, 0, (20, 12)),
            (59, 0, (22, 12)),
            (124, 0, (10, 16)),
            (54, 0, (20, 20)),
        ]
    );
    let bodies: Vec<&Body> = m.objects.iter().map(|o| &o.body).collect();
    assert_eq!(*bodies[9], Body::Sign(Text(b"Ahoy".to_vec())));
    assert_eq!(*bodies[10], Body::Sign(Text(Vec::new())));
    for i in [0, 3, 4, 5, 6, 7, 8, 11] {
        assert_eq!(*bodies[i], Body::None, "object {i}");
    }
    // Water under the second hero, each boat's visitable tile (one column left of its position),
    // the pickups, the bottles, the monster and the winds' corner; grass under the hole and the
    // first hero.
    const WATER: u8 = w::Terrain::Water as u8;
    for tile in [
        (20, 4),
        (12, 6),
        (12, 8),
        (12, 10),
        (14, 12),
        (20, 12),
        (20, 20),
        (24, 14),
    ] {
        assert_eq!(terrain(&m, tile), WATER, "{tile:?}");
    }
    for tile in [(10, 4), (10, 16)] {
        assert_eq!(terrain(&m, tile), w::Terrain::Grass as u8, "{tile:?}");
    }
    // The boats' templates as the game's: blocked and visited one column left of the position.
    let boat = &m.templates[usize::try_from(m.objects[3].template).unwrap()];
    assert_eq!(
        (boat.passable[5], boat.visitable[5], boat.terrains),
        (0xbf, 0x40, 0x100)
    );
}

// covers: ADV-SEA-001, ADV-SEA-007, ADV-OBJ-002
#[test]
fn the_sea_map_imports() {
    // The hero on water is refused (ADV-SEA-007; D-44, decisions 163, 178).
    assert_eq!(
        import(&written(), &SOD),
        Err(ImportError::Unsupported(Unsupported::HeroOnWater(2)))
    );
    // Without it the map imports and validates (E08 T72): the boats are the map's boats on their
    // visitable tiles with their types, no objects; the pickups, the bottles, the hole and the
    // winds typed, the winds on their picture's 3 × 3 tiles and the hole on its tile.
    let mut map = sea_map();
    map.objects.remove(2);
    let m = read(&w::gzip(&w::write(&map).unwrap()), &limits()).unwrap();
    let setup = import(&m, &SOD).unwrap();
    setup.validate(&SOD).unwrap();
    assert_eq!(setup.heroes.len(), 1);
    let at = |x, y| Pos::new(x, y, 0);
    assert_eq!(
        setup.map.boats,
        [(at(12, 6), 0), (at(12, 8), 1), (at(12, 10), 2)]
            .map(|(tile, kind)| BoatSetup { tile, kind })
    );
    let part = SOD.adventure();
    let keys: Vec<Option<&str>> = setup
        .map
        .objects
        .iter()
        .map(|o| {
            o.object_type
                .and_then(|ty| part.object_type(ty))
                .map(|ty| ty.key.as_str())
        })
        .collect();
    assert_eq!(
        keys,
        [
            Some("sod:favorable_winds"),
            Some("base:flotsam"),
            Some("base:sea_chest"),
            Some("base:shipwreck_survivor"),
            Some("base:ocean_bottle"),
            Some("base:ocean_bottle"),
            Some("base:hole"),
            Some("base:monster"),
        ]
    );
    let drawn: Vec<TileMask> = setup.map.objects.iter().map(|o| o.drawn).collect();
    assert_eq!(drawn[0], TileMask::from_bits(0x07_0707));
    assert_eq!(drawn[6], TileMask::from_bits(1));
    assert!(drawn[1..6].iter().all(|d| *d == TileMask::default()));
}
