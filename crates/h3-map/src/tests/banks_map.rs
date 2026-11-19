//! The creature banks map committed under `fixtures/maps/` (E08 T125b1, increment 13's design note
//! §17): SoD, 36 × 36 of grass, dirt on columns 0 to 12, the sea on columns 27 to 35 with an island
//! of grass on columns 29 to 31, rows 13 to 17. The eleven kinds beside the two players' heroes:
//! the land ones on dirt and on grass (two Crypts among them), a Shipwreck on the water beside the
//! coast (its entrance one column off the land, the island beyond it), a Derelict Ship off the
//! coast and an empty boat by red's hero on the coast. Red's heroes each by the banks of a scripted
//! replay: one with a full army and 900 experience below the Griffin Conservatory, a weak one and a
//! strong one by the Imp Cache, one between the two Crypts and the Shipwreck, one by the boat, a
//! flyer on the coast in front of the Shipwreck; blue's one below the Dragon Fly Hive and beside
//! the second Crypt, which red's other hero declines too, and two in the south. Written by
//! [`crate::writer`] with the original's templates; the golden replays play it
//! (`crates/h3-cli/tests/adventure_replays.rs`). The test checks that the committed file reads to
//! the map written here (it is rewritten when `H3_WRITE_FIXTURES` is set), imports with each bank
//! typed at its anchor, and builds with every bank's guards drawn.

use std::path::PathBuf;

use super::banks::{LAND_BANKS, WATER_BANKS};
use super::garrisons_bonuses::army;
use super::import::SOD;
use super::sea_map::{boat, hero, plain, wearing};
use crate::model::RawMap;
use crate::writer as w;
use crate::{Limits, import, read};
use h3_adventure::{BoatSetup, Pos};
use h3_game::GameState;

/// The committed file's name.
const NAME: &str = "e08-banks-36.h3m";

/// The heroes' tiles, by hero id (the map's order): red's Orrin (0) below the Griffin
/// Conservatory's entrance, Valeska (1) two below the Dragon Fly Hive's, Edric (2) below the Imp
/// Cache's and Sylvia (3) beside him, Sorsha (4) between the Crypts and the Shipwreck, Christian
/// (5) on the coast by the boat, Tyris (6) on the coast in front of the Shipwreck; blue's Rion (7)
/// below Valeska, left of the second Crypt, Adela (8) and Cuthbert (9) in the south.
pub(super) const HEROES: [(u8, u8); 10] = [
    (8, 9),
    (20, 17),
    (14, 9),
    (15, 9),
    (24, 15),
    (26, 27),
    (26, 15),
    (19, 20),
    (6, 28),
    (14, 28),
];
/// The heroes' owners and types, by hero id.
const HERO_TYPES: [(u8, u8); 10] = [
    (0, 0),
    (0, 1),
    (0, 2),
    (0, 3),
    (0, 5),
    (0, 6),
    (0, 7),
    (1, 8),
    (1, 9),
    (1, 10),
];
/// The empty boat's water tile (kind 0), right of Christian.
pub(super) const BOAT: (u8, u8) = (27, 27);

/// The banks in the map's order (the objects' order after the heroes): the anchor and the line,
/// `Ok` of `LAND_BANKS` (0 the Cyclops Stockpile, 1 the Dwarven Treasury, 2 the Griffin
/// Conservatory, 3 the Imp Cache, 4 the Medusa Stores, 5 the Naga Bank, 6 the Dragon Fly Hive, 8
/// the Crypt of dirt, grass and swamp, 10 Dragon Utopia) or `Err` of `WATER_BANKS` (0 the Derelict
/// Ship, 1 the Shipwreck entered west of its anchor). Entered on the anchor the Griffin
/// Conservatory and the Imp Cache, the others one left of it; the Shipwreck visited from its
/// entrance's neighbours.
pub(super) const BANKS: [((u8, u8), Result<usize, usize>); 12] = [
    // In the west: on dirt but the Imp Cache and the Naga Bank (grass) and Dragon Utopia (across
    // both, columns 9 to 14).
    ((8, 8), Ok(2)),
    ((14, 8), Ok(3)),
    ((4, 14), Ok(0)),
    ((10, 14), Ok(4)),
    ((16, 14), Ok(5)),
    ((6, 20), Ok(1)),
    ((14, 22), Ok(10)),
    // In the east, on grass.
    ((25, 10), Ok(8)),
    ((21, 15), Ok(6)),
    ((25, 20), Ok(8)),
    // On the water.
    ((28, 15), Err(1)),
    ((31, 25), Err(0)),
];

/// Creature codes (`CRTRAITS.TXT` order): Halberdiers, Marksmen, Royal Griffins, Swordsmen,
/// Crusaders, Zealots, Champions, Archangels.
const HALBERDIER: u16 = 1;
const MARKSMAN: u16 = 3;
const ROYAL_GRIFFIN: u16 = 5;
const SWORDSMAN: u16 = 6;
const CRUSADER: u16 = 7;
const ZEALOT: u16 = 9;
const CHAMPION: u16 = 11;
const ARCHANGEL: u16 = 13;
/// Angel Wings (`ARTRAITS.TXT` order) and its slot, the shoulders (`ArtSlots.txt` order).
const WINGS: u16 = 72;
const SHOULDERS: usize = 1;

/// The map's heroes, by hero id: Orrin (0) with a full army of seven Castle stacks, no Angels
/// (the Conservatory's reward finds no room) and 900 experience; Edric (2) with ten swordsmen;
/// Sylvia (3) with ten Archangels, Sorsha (4) with twenty-five, Christian (5) with fifteen, Adela
/// (8) with ten; Tyris (6) wearing Angel Wings; the others their type's own.
fn heroes() -> Vec<w::MapObject> {
    HEROES
        .iter()
        .zip(HERO_TYPES)
        .enumerate()
        .map(|(id, (&tile, (owner, hero_type)))| {
            let mut h = hero(tile, owner, hero_type);
            match id {
                0 => {
                    h.army = Some(army(&[
                        (ARCHANGEL, 10),
                        (CHAMPION, 10),
                        (CRUSADER, 15),
                        (ZEALOT, 15),
                        (ROYAL_GRIFFIN, 15),
                        (MARKSMAN, 20),
                        (HALBERDIER, 30),
                    ]));
                    h.experience = Some(900);
                }
                2 => h.army = Some(army(&[(SWORDSMAN, 10)])),
                3 | 8 => h.army = Some(army(&[(ARCHANGEL, 10)])),
                4 => h.army = Some(army(&[(ARCHANGEL, 25)])),
                5 => h.army = Some(army(&[(ARCHANGEL, 15)])),
                6 => wearing(&mut h, SHOULDERS, WINGS, Vec::new()),
                _ => {}
            }
            w::MapObject::Hero(h)
        })
        .collect()
}

/// The objects other than the heroes, in the map's order: the banks, then the boat.
fn objects() -> Vec<w::MapObject> {
    let mut objects: Vec<w::MapObject> = BANKS
        .iter()
        .map(|&(at, line)| {
            let line = match line {
                Ok(land) => LAND_BANKS[land],
                Err(water) => WATER_BANKS[water],
            };
            plain(line, at)
        })
        .collect();
    objects.push(boat(0, BOAT));
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
    let patch = |terrain, from, to| w::Patch { terrain, from, to };
    w::Map {
        name: "Banks".into(),
        description: "E08 T125b1: the eleven creature banks".into(),
        size: 36,
        terrain: w::Terrain::Grass,
        patches: vec![
            patch(w::Terrain::Dirt, (0, 0), (12, 35)),
            patch(w::Terrain::Water, (27, 0), (35, 35)),
            patch(w::Terrain::Grass, (29, 13), (31, 17)),
        ],
        roads: Vec::new(),
        players,
        objects: all,
        underground: None,
        level_limit: 0,
        banned_skills: Vec::new(),
        banned_artifacts: Vec::new(),
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

/// The banks' types by key, in the map's order.
const TYPES: [&str; 12] = [
    "base:griffin_conservatory",
    "base:imp_cache",
    "base:cyclops_stockpile",
    "base:medusa_stores",
    "base:naga_bank",
    "base:dwarven_treasury",
    "base:dragon_utopia",
    "base:crypt",
    "base:dragon_fly_hive",
    "base:crypt",
    "base:shipwreck",
    "base:derelict_ship",
];

// covers: ADV-BNK-001, ADV-BNK-002
#[test]
fn the_banks_map_is_committed_as_written_and_imports_the_eleven() {
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
        [BoatSetup {
            tile: at(BOAT),
            kind: 0
        }]
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
    let expected: Vec<(&str, Pos)> = TYPES
        .into_iter()
        .zip(BANKS.iter().map(|&(tile, _)| at(tile)))
        .collect();
    assert_eq!(placed, expected);
    // Every bank's guards drawn by the build (ADV-BNK-002): some stand at each.
    let game = GameState::new(rules, &setup, 1).unwrap();
    for index in 0..BANKS.len() {
        assert!(!game.adventure().guards_left(index).is_empty(), "{index}");
    }
}
