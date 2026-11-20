//! The creature banks XL map committed under `fixtures/maps/` (E08 T125b1, increment 13's design
//! note §17): SoD, 144 × 144 of grass with the sea on columns 96 to 143. The census's most banks on
//! one map (106, as *Thousand Islands*; survey `E08-banks.md` §1), every kind, in the census's
//! shares: the nine land kinds' 78 each in a 6 × 6 cell of its own drawn by a fixed stride through
//! the land's cells (no cell holding a hero), fourteen Shipwrecks along the coast (each visited
//! from the land beside it, an island of 3 × 3 two columns beyond it, so that a flight from the
//! coast to the island crosses it), six more and the eight Derelict Ships in the open sea by a
//! stride of their own. Red's five heroes in the north, blue's five in the south: on each side a
//! flyer on the coast in front of a Shipwreck and a hero on the coast beside a boat; a monster on
//! every hundredth free land tile, counted row by row. Written by [`crate::writer`] as the
//! garrisons and bonuses XL map (`garrisons_bonuses_xl_map.rs`); to be played by the benchmarks
//! `reach-banks-xl-144` and `build-banks-xl-144` (T125b2). The test checks that the committed file
//! reads to the map written here (it is rewritten when `H3_WRITE_FIXTURES` is set), imports each
//! bank typed at its anchor, the coast's Shipwrecks with their entrances beside the land, and
//! builds with every bank's guards drawn and each flyer's path to its island across the Shipwreck.

use std::path::PathBuf;

use super::banks::{LAND_BANKS, WATER_BANKS};
use super::import::SOD;
use super::sea_map::{boat, hero, lines as sea, plain, template, wearing};
use crate::model::RawMap;
use crate::writer as w;
use crate::{Limits, import, read};
use h3_adventure::{BoatSetup, Pos};
use h3_game::GameState;

/// The committed file's name.
const NAME: &str = "e08-banks-144.h3m";
/// The map's side.
const SIDE: u8 = 144;
/// The sea's first column; the sea runs to the map's east edge on every row.
const SEA: u8 = 96;
/// A cell's side: the banks stand one to a cell.
const CELL: u8 = 6;
/// The stride through the land's free cells (374, coprime with it), and through the open sea's
/// 168 cells (those not touching the coast; coprime with them).
const LAND_STRIDE: usize = 101;
const SEA_STRIDE: usize = 5;
/// A monster on every this many free land tiles, counted row by row.
const EVERY: usize = 100;

/// The land kinds' counts, by their `LAND_BANKS` line (the census's shares of 106, survey §1):
/// the Cyclops Stockpile, the Dwarven Treasury, the Griffin Conservatory, the Imp Cache, the
/// Medusa Stores, the Naga Bank, the Dragon Fly Hive, the Crypt (of dirt, grass and swamp, line
/// 8) and Dragon Utopia (78 in all).
const LAND_COUNTS: [(usize, usize); 9] = [
    (0, 8),
    (1, 12),
    (2, 6),
    (3, 10),
    (4, 9),
    (5, 7),
    (6, 7),
    (8, 14),
    (10, 5),
];
/// The Shipwrecks along the coast: one every ten rows from row 5, anchored one column into the
/// sea, so that the entrance (the anchor's west neighbour, `WATER_BANKS` line 1) is the coast's
/// water tile.
const COAST_WRECKS: u8 = 14;
/// The islands beyond the coast's Shipwrecks: their first column and side.
const ISLAND: u8 = SEA + 3;
const ISLAND_SIDE: u8 = 3;
/// The open sea's banks by their `WATER_BANKS` line and count: six Shipwrecks, eight Derelict
/// Ships (twenty Shipwrecks and eight Derelict Ships in all).
const SEA_COUNTS: [(usize, usize); 2] = [(1, 6), (0, 8)];
/// The kinds' types by key, in `LAND_COUNTS`' order, then the Shipwreck and the Derelict Ship.
const KEYS: [&str; 11] = [
    "base:cyclops_stockpile",
    "base:dwarven_treasury",
    "base:griffin_conservatory",
    "base:imp_cache",
    "base:medusa_stores",
    "base:naga_bank",
    "base:dragon_fly_hive",
    "base:crypt",
    "base:dragon_utopia",
    "base:shipwreck",
    "base:derelict_ship",
];

/// The heroes' tiles and owners, in the map's order: red's in the north (the third a flyer on the
/// coast in front of a Shipwreck, the fifth on the coast beside a boat), then blue's in the south
/// as well.
const HEROES: [((u8, u8), u8); 10] = [
    ((15, 15), 0),
    ((51, 21), 0),
    ((95, 35), 0),
    ((57, 45), 0),
    ((95, 10), 0),
    ((15, 129), 1),
    ((51, 123), 1),
    ((95, 115), 1),
    ((57, 99), 1),
    ((95, 130), 1),
];
/// The heroes' types, in the map's order (SoD's, a class apart or more: none of the campaigns').
const TYPES: [u8; 10] = [0, 14, 28, 42, 56, 70, 84, 98, 112, 126];
/// The flyers among them (Angel Wings), by hero id.
const FLYERS: [usize; 2] = [2, 7];
/// The boats on the water beside the heroes on the coast (red's, then blue's).
const BOATS: [(u8, u8); 2] = [(SEA, 10), (SEA, 130)];
/// Angel Wings (`ARTRAITS.TXT` order) and its slot, the shoulders (`ArtSlots.txt` order).
const WINGS: u16 = 72;
const SHOULDERS: usize = 1;

/// A tile's cell: its column and row of cells.
fn cell_of((x, y): (u8, u8)) -> (u8, u8) {
    (x / CELL, y / CELL)
}

/// A bank placed in a cell: anchored 5 right of and 3 below the cell's corner, so that its tiles
/// (up to five left of and two above the anchor, Dragon Utopia's) stay in the cell, its entrance
/// on the anchor's row with the cell's two rows below it free.
fn anchor_in((i, j): (u8, u8)) -> (u8, u8) {
    (CELL * i + 5, CELL * j + 3)
}

/// The land's free cells, row by row: none holding a hero's tile.
fn land_cells() -> Vec<(u8, u8)> {
    let cells = SIDE / CELL;
    (0..cells)
        .flat_map(|j| (0..SEA / CELL).map(move |i| (i, j)))
        .filter(|&cell| HEROES.iter().all(|&(tile, _)| cell_of(tile) != cell))
        .collect()
}

/// The open sea's cells, row by row: the sea's but its first column of cells (the coast's).
fn sea_cells() -> Vec<(u8, u8)> {
    let cells = SIDE / CELL;
    (0..cells)
        .flat_map(|j| (SEA / CELL + 1..cells).map(move |i| (i, j)))
        .collect()
}

/// Each line at its count, in turn, the `k`th in the cell `k` × `stride` modulo their number of
/// `cells` (all distinct: the stride is coprime with it, and there are cells enough).
fn placed(
    lines: &[&'static str],
    counts: &[(usize, usize)],
    cells: &[(u8, u8)],
    stride: usize,
) -> Vec<(&'static str, (u8, u8))> {
    (counts.iter())
        .flat_map(|&(line, count)| (0..count).map(move |_| lines[line]))
        .enumerate()
        .map(|(k, line)| (line, anchor_in(cells[k * stride % cells.len()])))
        .collect()
}

/// The coast's Shipwrecks' anchors: one column into the sea, one every ten rows from row 5.
fn coast_wrecks() -> Vec<(u8, u8)> {
    (0..COAST_WRECKS).map(|k| (SEA + 1, 5 + 10 * k)).collect()
}

/// The banks in the map's order, each its line and its anchor: the land's, the coast's
/// Shipwrecks, then the open sea's.
fn banks() -> Vec<(&'static str, (u8, u8))> {
    let mut banks = placed(&LAND_BANKS, &LAND_COUNTS, &land_cells(), LAND_STRIDE);
    banks.extend(coast_wrecks().into_iter().map(|at| (WATER_BANKS[1], at)));
    banks.extend(placed(&WATER_BANKS, &SEA_COUNTS, &sea_cells(), SEA_STRIDE));
    banks
}

/// The tiles a bank of `line` anchored on `anchor` covers: its template's blocked and visited
/// tiles (bit `7 − c` of byte `5 − r` the tile `c` columns left of and `r` rows above the anchor).
fn bank_tiles(line: &str, anchor: (u8, u8)) -> Vec<(u8, u8)> {
    let template = template(line);
    let (x, y) = anchor;
    (0..6u8)
        .flat_map(|row| (0..8u8).map(move |column| (column, row)))
        .filter(|&(column, row)| {
            let (byte, bit) = (usize::from(5 - row), 7 - column);
            template.passable[byte] >> bit & 1 == 0 || template.visitable[byte] >> bit & 1 == 1
        })
        .map(|(column, row)| (x - column, y - row))
        .collect()
}

/// The monsters' tiles: every [`EVERY`]th free land tile, counted row by row (the first one
/// taken); a tile is free when no bank covers it and no hero stands on it or beside it.
fn monsters(banks: &[(&str, (u8, u8))]) -> Vec<(u8, u8)> {
    let index = |(x, y): (u8, u8)| usize::from(y) * usize::from(SIDE) + usize::from(x);
    let mut taken = vec![false; usize::from(SIDE) * usize::from(SIDE)];
    for &(line, anchor) in banks {
        for tile in bank_tiles(line, anchor) {
            taken[index(tile)] = true;
        }
    }
    // No hero stands on the map's edge.
    for &((x, y), _) in &HEROES {
        for ty in y - 1..=y + 1 {
            for tx in x - 1..=x + 1 {
                taken[index((tx, ty))] = true;
            }
        }
    }
    (0..SIDE)
        .flat_map(|y| (0..SEA).map(move |x| (x, y)))
        .filter(|&tile| !taken[index(tile)])
        .step_by(EVERY)
        .collect()
}

/// The map's objects: the heroes (the flyers wearing Angel Wings), the banks, the boats, then the
/// monsters (a count of 1 to 12 gnolls in turn, savage, never fleeing, not growing).
fn objects(banks: &[(&'static str, (u8, u8))]) -> Vec<w::MapObject> {
    let mut objects: Vec<w::MapObject> = (HEROES.iter().zip(TYPES).enumerate())
        .map(|(id, (&(tile, owner), hero_type))| {
            let mut h = hero(tile, owner, hero_type);
            if FLYERS.contains(&id) {
                wearing(&mut h, SHOULDERS, WINGS, Vec::new());
            }
            w::MapObject::Hero(h)
        })
        .collect();
    objects.extend(banks.iter().map(|&(line, at)| plain(line, at)));
    objects.extend(BOATS.map(|at| boat(0, at)));
    let gnolls = template(sea::GNOLLS);
    objects.extend(monsters(banks).into_iter().enumerate().map(|(n, tile)| {
        w::MapObject::Monster(w::Monster {
            tile,
            template: gnolls.clone(),
            count: 1 + u16::try_from(n % 12).unwrap(),
            disposition: 4,
            never_flees: true,
            fixed_count: true,
            treasure: None,
        })
    }));
    objects
}

/// The sea, then an island of grass beyond each of the coast's Shipwrecks, its middle row the
/// Shipwreck's.
fn patches() -> Vec<w::Patch> {
    let patch = |terrain, from, to| w::Patch { terrain, from, to };
    let mut patches = vec![patch(w::Terrain::Water, (SEA, 0), (SIDE - 1, SIDE - 1))];
    patches.extend(coast_wrecks().into_iter().map(|(_, y)| {
        let last = ISLAND + ISLAND_SIDE - 1;
        patch(w::Terrain::Grass, (ISLAND, y - 1), (last, y + 1))
    }));
    patches
}

/// The map: SoD, 144 × 144 of grass with the sea and its islands; red and blue with five heroes
/// each.
fn map() -> w::Map {
    let mut players: [Option<w::Player>; w::PLAYERS] = Default::default();
    for player in &mut players[..2] {
        *player = Some(w::Player {
            human: true,
            computer: true,
            towns: 1,
        });
    }
    w::Map {
        name: "Banks XL".into(),
        description: "E08 T125b1: the shipped maps' most creature banks".into(),
        size: SIDE,
        terrain: w::Terrain::Grass,
        patches: patches(),
        roads: Vec::new(),
        players,
        objects: objects(&banks()),
        underground: None,
        level_limit: 0,
        banned_skills: Vec::new(),
        banned_artifacts: Vec::new(),
        banned_spells: Vec::new(),
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

// covers: ADV-BNK-001, ADV-BNK-002
#[test]
fn the_banks_xl_map_is_committed_as_written_and_imports_its_banks() {
    let rules = &*SOD;
    let setup = import(&committed(), rules).unwrap();
    let heroes: Vec<(Pos, usize)> = (setup.heroes.iter())
        .map(|h| (h.pos, h.owner.index()))
        .collect();
    let expected: Vec<(Pos, usize)> = (HEROES.iter())
        .map(|&(tile, owner)| (at(tile), usize::from(owner)))
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
    let key = |o: &h3_adventure::ObjectSetup| {
        part.object_type(o.object_type.unwrap())
            .unwrap()
            .key
            .as_str()
    };
    // Every object typed: the banks, then the monsters.
    let banks = banks();
    let monsters = monsters(&banks);
    let objects = &setup.map.objects;
    assert_eq!(objects.len(), banks.len() + monsters.len());
    assert!(objects.iter().all(|o| o.object_type.is_some()));
    assert!(
        (100..150).contains(&monsters.len()),
        "{} monsters",
        monsters.len()
    );
    // The census's most on one map, every kind at its share, each at its anchor.
    assert_eq!(banks.len(), 106);
    assert_eq!(land_cells().len(), 374);
    let bank_objects = &objects[..banks.len()];
    let land: usize = LAND_COUNTS.iter().map(|&(_, n)| n).sum();
    let counts = (LAND_COUNTS.iter().map(|&(_, n)| n))
        .chain([usize::from(COAST_WRECKS) + SEA_COUNTS[0].1, SEA_COUNTS[1].1]);
    for (&wanted, count) in KEYS.iter().zip(counts) {
        let found = bank_objects.iter().filter(|o| key(o) == wanted).count();
        assert_eq!(found, count, "{wanted}");
    }
    let anchors: Vec<Pos> = bank_objects.iter().map(|o| o.anchor).collect();
    let expected: Vec<Pos> = banks.iter().map(|&(_, anchor)| at(anchor)).collect();
    assert_eq!(anchors, expected);
    // The coast's Shipwrecks: each entrance (west of the anchor) the water tile beside the
    // coast's land.
    let game = GameState::new(rules, &setup, 1).unwrap();
    let map = game.adventure().map();
    let coast = &bank_objects[land..land + usize::from(COAST_WRECKS)];
    for wreck in coast {
        assert_eq!(key(wreck), "base:shipwreck");
        assert_eq!(wreck.visitable.offsets().collect::<Vec<_>>(), [(1, 0)]);
        let shore = Pos::new(wreck.anchor.x - 2, wreck.anchor.y, 0);
        assert!(map.is_coast(shore), "{shore:?}");
    }
    // Each flyer reaches the island beyond the Shipwreck in front of it, its path across the
    // Shipwreck's two tiles.
    for (flyer, y) in [(2, 35), (7, 115)] {
        let hero = game.adventure().heroes()[flyer].id;
        let reach = (game.adventure())
            .reach(rules.core(), part, game.heroes(), hero)
            .unwrap();
        let path = reach.path(at((ISLAND, y))).unwrap();
        let crossed = |x: u8| path.contains(&at((x, y)));
        assert!(crossed(SEA) && crossed(SEA + 1), "{flyer} {y}: {path:?}");
    }
    // Every bank's guards drawn by the build (ADV-BNK-002).
    for index in 0..banks.len() {
        assert!(!game.adventure().guards_left(index).is_empty(), "{index}");
    }
}
