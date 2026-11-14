//! The bonus visitables map committed under `fixtures/maps/` (E08 T116a, increment 12's design
//! note §16): SoD, 36 × 36 of grass with a strait of sea on columns 24 to 29, red in the north,
//! blue in the south. Each of the fifteen bonus visitables: the twelve land ones spread over both
//! halves, two Magic Springs and two Fountains of Fortune among them, the Stables beside red's
//! hero with Cavaliers; the sea's three (the Buoy, the Sirens, the Mermaids) on the strait, an
//! empty boat by each player's coast and a stack of pikemen on the water south of the Sirens.
//! Written by [`crate::writer`] with the original's templates; the golden replays play it
//! (`crates/h3-cli/tests/adventure_replays.rs`). The test checks that the committed file reads to
//! the map written here (it is rewritten when `H3_WRITE_FIXTURES` is set), imports with each of
//! the fifteen typed, and builds with its two Fountains.

use std::path::PathBuf;

use super::garrisons_bonuses::{LAND_BONUSES, WATER_BONUSES, army};
use super::import::SOD;
use super::sea_map::{boat, hero, lines as sea, plain, template};
use crate::model::RawMap;
use crate::writer as w;
use crate::{Limits, import, read};
use h3_adventure::{BoatSetup, Pos};
use h3_game::GameState;

/// The committed file's name.
const NAME: &str = "e08-bonuses-36.h3m";

/// The heroes' tiles, by hero id (the map's order): red's Orrin (0) by red's boat, Valeska (1)
/// below the Stables, Edric (2) beside her, Sorsha (3) and Christian (4) on either side below the
/// first Magic Spring; blue's Rion (5) by blue's boat, Adela (6) below the second Spring, Cuthbert
/// (7) between the Magic Well and the Fountain of Youth.
pub(super) const HEROES: [(u8, u8); 8] = [
    (23, 6),
    (8, 5),
    (9, 5),
    (6, 11),
    (3, 11),
    (23, 28),
    (6, 27),
    (16, 25),
];
/// The heroes' owners and types, by hero id.
const HERO_TYPES: [(u8, u8); 8] = [
    (0, 0),
    (0, 1),
    (0, 2),
    (0, 5),
    (0, 6),
    (1, 8),
    (1, 9),
    (1, 10),
];
/// The empty boats' water tiles (both of kind 0): Orrin's and Rion's.
pub(super) const BOATS: [(u8, u8); 2] = [(24, 6), (24, 28)];

/// The land bonus visitables in the map's order (the objects' order after the heroes): the
/// anchor and the `LAND_BONUSES` line (0 the Swan Pond, 1 the Faerie Ring, 2 the Fountain of
/// Fortune, 3 the Fountain of Youth, 4 the Idol of Fortune, 5 the Magic Spring, 6 the Magic Well,
/// 7 the Oasis, 8 the Rally Flag, 9 the Stables, 10 the Temple, 11 the Watering Hole). Entered on
/// the anchor but the Swan Pond, the Faerie Ring and the Temple (one left of it), the Spring and
/// the Oasis (the anchor and one left of it) and the Watering Hole (its four tiles).
pub(super) const LAND: [((u8, u8), usize); 14] = [
    // Red's half.
    ((8, 4), 9),
    ((5, 10), 5),
    ((14, 3), 2),
    ((17, 6), 4),
    ((12, 9), 10),
    ((18, 12), 0),
    ((10, 14), 1),
    ((21, 15), 7),
    // Blue's half.
    ((5, 26), 5),
    ((14, 32), 2),
    ((10, 22), 3),
    ((17, 24), 6),
    ((12, 28), 8),
    ((21, 31), 11),
];
/// The sea's three on the strait, after the land ones: the Buoy (visited from any neighbour),
/// the Sirens and the Mermaids (each three tiles ending at the anchor, visited on the middle one
/// from beside or below), by their `WATER_BONUSES` line.
pub(super) const WATER: [((u8, u8), usize); 3] = [((27, 3), 0), ((27, 9), 2), ((27, 22), 1)];
/// The pikemen on the water south of the Sirens, and their count.
pub(super) const PIKEMEN: ((u8, u8), u16) = ((27, 14), 6);

/// Creature codes: Pikemen, Archers, Swordsmen, Cavaliers.
const PIKEMAN: u16 = 0;
const ARCHER: u16 = 2;
const SWORDSMAN: u16 = 6;
const CAVALIER: u16 = 10;

/// The map's heroes, by hero id: Orrin (0) with twenty pikemen, ten archers and five swordsmen
/// (the Sirens' losses in three slots); Edric (2) with five Cavaliers and ten pikemen; the others
/// their type's own.
fn heroes() -> Vec<w::MapObject> {
    HEROES
        .iter()
        .zip(HERO_TYPES)
        .enumerate()
        .map(|(id, (&tile, (owner, hero_type)))| {
            let mut h = hero(tile, owner, hero_type);
            match id {
                0 => h.army = Some(army(&[(PIKEMAN, 20), (ARCHER, 10), (SWORDSMAN, 5)])),
                2 => h.army = Some(army(&[(CAVALIER, 5), (PIKEMAN, 10)])),
                _ => {}
            }
            w::MapObject::Hero(h)
        })
        .collect()
}

/// The objects other than the heroes, in the map's order.
fn objects() -> Vec<w::MapObject> {
    let mut objects: Vec<w::MapObject> = LAND
        .iter()
        .map(|&(at, line)| plain(LAND_BONUSES[line], at))
        .collect();
    objects.extend(
        WATER
            .iter()
            .map(|&(at, line)| plain(WATER_BONUSES[line], at)),
    );
    let (at, count) = PIKEMEN;
    objects.push(w::MapObject::Monster(w::Monster {
        tile: at,
        template: template(sea::PIKEMEN),
        count,
        disposition: 4,
        never_flees: true,
        fixed_count: true,
        treasure: None,
    }));
    objects.extend(BOATS.map(|at| boat(0, at)));
    objects
}

fn map() -> w::Map {
    let mut players: [Option<w::Player>; w::PLAYERS] = Default::default();
    for player in &mut players[..2] {
        *player = Some(w::Player {
            human: true,
            computer: true,
            towns: 1,
        });
    }
    let mut all = heroes();
    all.extend(objects());
    w::Map {
        name: "Bonuses".into(),
        description: "E08 T116a: the fifteen bonus visitables".into(),
        size: 36,
        terrain: w::Terrain::Grass,
        patches: vec![w::Patch {
            terrain: w::Terrain::Water,
            from: (24, 0),
            to: (29, 35),
        }],
        roads: Vec::new(),
        players,
        objects: all,
        underground: None,
        level_limit: 0,
        banned_skills: Vec::new(),
        events: Vec::new(),
        teams: None,
    }
}

/// The committed file, checked to read to the map written here (rewritten first when
/// `H3_WRITE_FIXTURES` is set).
fn committed() -> RawMap {
    let bytes = w::write(&map()).unwrap();
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/maps")
        .join(NAME);
    if std::env::var_os("H3_WRITE_FIXTURES").is_some() {
        std::fs::write(&path, w::gzip(&bytes)).unwrap();
    }
    let file = std::fs::read(&path).unwrap();
    let written = read(&w::gzip(&bytes), &Limits::default()).unwrap();
    let committed = read(&file, &Limits::default()).unwrap();
    assert!(
        committed == written,
        "{NAME} is not the map written here: H3_WRITE_FIXTURES=1 rewrites it"
    );
    committed
}

fn at((x, y): (u8, u8)) -> Pos {
    Pos::new(x.into(), y.into(), 0)
}

/// The objects' types by key, in the map's order (the boats are the map's boats, no objects).
const TYPES: [&str; 18] = [
    "base:stables",
    "base:magic_spring",
    "base:fountain_of_fortune",
    "base:idol_of_fortune",
    "base:temple",
    "base:swan_pond",
    "base:faerie_ring",
    "base:oasis",
    "base:magic_spring",
    "base:fountain_of_fortune",
    "base:fountain_of_youth",
    "base:magic_well",
    "base:rally_flag",
    "base:watering_hole",
    "base:buoy",
    "base:sirens",
    "base:mermaids",
    "base:monster",
];

// covers: ADV-BON-001
#[test]
fn the_bonuses_map_is_committed_as_written_and_imports_the_fifteen() {
    let rules = &*SOD;
    let setup = import(&committed(), rules).unwrap();
    let heroes: Vec<(Pos, usize)> = setup
        .heroes
        .iter()
        .map(|h| (h.pos, h.owner.index()))
        .collect();
    let expected: Vec<(Pos, usize)> = HEROES
        .iter()
        .zip(HERO_TYPES)
        .map(|(&tile, (owner, _))| (at(tile), usize::from(owner)))
        .collect();
    assert_eq!(heroes, expected);
    assert_eq!(
        setup.map.boats,
        BOATS.map(|tile| BoatSetup {
            tile: at(tile),
            kind: 0
        })
    );
    let part = rules.adventure();
    let placed: Vec<(&str, Pos)> = (setup.map.objects.iter())
        .map(|o| {
            let key = part
                .object_type(o.object_type.unwrap())
                .unwrap()
                .key
                .as_str();
            (key, o.anchor)
        })
        .collect();
    let anchors = (LAND.iter().chain(&WATER))
        .map(|&(tile, _)| tile)
        .chain([PIKEMEN.0]);
    let expected: Vec<(&str, Pos)> = TYPES.into_iter().zip(anchors.map(at)).collect();
    assert_eq!(placed, expected);
    let game = GameState::new(rules, &setup, 1).unwrap();
    // Both Fountains of Fortune, in object order: their values are drawn by their place here.
    assert_eq!(game.adventure().map().fountains(), [2, 9]);
}
