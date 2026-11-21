//! The trades XL map committed under `fixtures/maps/` (E08 T135b1, increment 14's design note
//! §18): SoD, 144 × 144 of grass over an underground of subterranean ground (41 472 tiles), as the
//! dwellings XL (`dwellings_xl_map.rs`): the economy XL's heroes, one object to a cell of each
//! level's lattice, a wandering monster on every hundredth free tile. As many of each class of
//! increment 14 as the shipped maps hold on one map (survey `docs/rules/surveys/E08-trades.md` §1:
//! 7 Altars of Sacrifice, 6 Black Markets, 11 Hill Forts, 9 Trading Posts of class 99 and 2 of
//! class 221, 7 War Machine Factories, 7 Freelancer's Guilds; 49), the kinds interleaved over the
//! lattice: the Hill Forts and the Trading Posts on the surface (their pictures' terrains: grass,
//! a snow patch under each snow Trading Post), the larger half of the others there and the rest
//! underground. No bans: the six Black Markets draw their 42 from the whole pool. Written by
//! [`crate::writer`] with the original's templates; to be played by the benchmark
//! `build-trades-xl-144` (T135b2: the game's build with the Black Markets' draws). The test checks
//! that the committed file reads to the map written here (it is rewritten when
//! `H3_WRITE_FIXTURES` is set), imports every class at its count by level, and builds with each
//! Black Market's seven drawn.

use std::path::PathBuf;

use super::economy_xl_map::{
    CELL, HEROES, MONSTER_EVERY, SIDE, TYPES, ring, surface_share, wandering,
};
use super::import::SOD;
use super::sea_map::{hero, plain};
use super::trades::PICTURES;
use crate::model::RawMap;
use crate::writer as w;
use crate::{Limits, import, read};
use h3_content::ArtifactClass;
use h3_game::GameState;

/// The committed file's name.
const NAME: &str = "e08-trades-144.h3m";

/// [`PICTURES`]' lines the map places.
const ALTAR: usize = 0;
const BLACK_MARKET: usize = 1;
const HILL_FORT_GRASS: usize = 3;
const POST: usize = 4;
const POST_SNOW: usize = 6;
const FACTORY: usize = 7;
const GUILD: usize = 8;

/// The census's most of each class on one map (survey §1), by line: those on either level, the
/// larger half on the surface ([`surface_share`]), and those of the surface alone.
const BOTH_LEVELS: [(usize, usize); 4] = [(ALTAR, 7), (BLACK_MARKET, 6), (FACTORY, 7), (GUILD, 7)];
const SURFACE_ONLY: [(usize, usize); 3] = [(HILL_FORT_GRASS, 11), (POST, 9), (POST_SNOW, 2)];

/// The object kinds by type key and the count the tests expect of each.
const KEYS: [(&str, usize); 6] = [
    ("base:altar_of_sacrifice", 7),
    ("base:black_market", 6),
    ("base:war_machine_factory", 7),
    ("ab:freelancers_guild", 7),
    ("base:hill_fort", 11),
    ("base:trading_post", 11),
];

/// A level's lines, each kind spread over the lattice: one of each kind in turn until all are
/// taken.
fn lines(underground: bool) -> Vec<usize> {
    let mut groups: Vec<Vec<usize>> = BOTH_LEVELS
        .iter()
        .map(|&(line, count)| {
            let surface = surface_share(count);
            let here = if underground {
                count - surface
            } else {
                surface
            };
            vec![line; here]
        })
        .collect();
    if !underground {
        groups.extend(SURFACE_ONLY.iter().map(|&(line, count)| vec![line; count]));
    }
    let mut out = Vec::new();
    while groups.iter().any(|g| !g.is_empty()) {
        for group in &mut groups {
            out.extend(group.pop());
        }
    }
    out
}

/// A level's objects by line and anchor: the `n`th of `k` on the lattice's cell `n` × cells / `k`,
/// on the cell's tile (5, 5), so that its picture (three tiles of a row at most) and a free ring
/// stay inside the cell, the entrance's row below free.
fn placed(underground: bool) -> Vec<(usize, (u8, u8))> {
    let cells: Vec<(u8, u8)> = (0..SIDE / CELL)
        .flat_map(|j| (0..SIDE / CELL).map(move |i| (CELL * i + 5, CELL * j + 5)))
        .filter(|&(x, y)| {
            // No cell holds a hero (the economy XL's heroes stand on a cell's tile (1, 1)).
            HEROES
                .iter()
                .all(|&(hx, hy)| (hx / CELL, hy / CELL) != (x / CELL, y / CELL))
        })
        .collect();
    let lines = lines(underground);
    (lines.iter().enumerate())
        .map(|(n, &line)| (line, cells[n * cells.len() / lines.len()]))
        .collect()
}

/// One level's objects: its four heroes, its trades, then its wandering monsters on every
/// [`MONSTER_EVERY`]th free tile (`monsters` counts the map's so far).
fn level(underground: bool, monsters: &mut usize) -> Vec<w::MapObject> {
    let types = TYPES[usize::from(underground)];
    let mut objects: Vec<w::MapObject> = HEROES
        .iter()
        .zip(types)
        .zip([0, 0, 1, 1])
        .map(|((&at, hero_type), owner)| w::MapObject::Hero(hero(at, owner, hero_type)))
        .collect();
    let mut covered = vec![false; usize::from(SIDE) * usize::from(SIDE)];
    let index = |(x, y): (u8, u8)| usize::from(y) * usize::from(SIDE) + usize::from(x);
    for (line, at) in placed(underground) {
        objects.push(plain(PICTURES[line], at));
        for tile in ring(at) {
            covered[index(tile)] = true;
        }
    }
    for &(x, y) in &HEROES {
        for dy in 0..=4 {
            for dx in 0..=4 {
                covered[index((x + dx - 2, y + dy - 2))] = true;
            }
        }
    }
    let mut free: usize = 0;
    for y in 0..SIDE {
        for x in 0..SIDE {
            if covered[index((x, y))] {
                continue;
            }
            free += 1;
            if free.is_multiple_of(MONSTER_EVERY) {
                objects.push(wandering(*monsters, (x, y)));
                *monsters += 1;
            }
        }
    }
    objects
}

/// A snow patch under each snow Trading Post: its ring.
fn snow() -> Vec<w::Patch> {
    placed(false)
        .into_iter()
        .filter(|&(line, _)| line == POST_SNOW)
        .map(|(_, (x, y))| w::Patch {
            terrain: w::Terrain::Snow,
            from: (x - 4, y - 3),
            to: (x + 1, y + 1),
        })
        .collect()
}

/// The map: SoD, 144 × 144 of grass over subterranean ground; red and blue with two heroes on
/// each level. Returns the map and its count of monsters.
fn map() -> (w::Map, usize) {
    let mut players: [Option<w::Player>; w::PLAYERS] = Default::default();
    for player in &mut players[..2] {
        *player = Some(w::Player {
            human: true,
            computer: true,
            towns: 1,
        });
    }
    let mut monsters = 0;
    let surface = level(false, &mut monsters);
    let underground = level(true, &mut monsters);
    let map = w::Map {
        name: "Trades XL".into(),
        description: "E08 T135b1: the shipped maps' most trades and services".into(),
        size: SIDE,
        terrain: w::Terrain::Grass,
        patches: snow(),
        roads: Vec::new(),
        players,
        objects: surface,
        underground: Some(w::Underground {
            terrain: w::Terrain::Subterranean,
            patches: Vec::new(),
            objects: underground,
        }),
        level_limit: 0,
        banned_skills: Vec::new(),
        banned_artifacts: Vec::new(),
        banned_spells: Vec::new(),
        events: Vec::new(),
        teams: None,
    };
    (map, monsters)
}

/// The committed file, checked to read to the map written here (rewritten first when
/// `H3_WRITE_FIXTURES` is set), and the map's count of monsters.
fn committed() -> (RawMap, usize) {
    let (map, monsters) = map();
    let bytes = w::write(&map).unwrap();
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
    (committed, monsters)
}

// covers: ADV-MKT-001, ADV-MKT-006
#[test]
fn the_trades_xl_map_is_committed_as_written_and_imports_the_census() {
    let rules = &*SOD;
    let (raw, monsters) = committed();
    let setup = import(&raw, rules).unwrap();
    assert_eq!(setup.heroes.len(), 2 * HEROES.len());
    let part = rules.adventure();
    let key = |o: &h3_adventure::ObjectSetup| {
        part.object_type(o.object_type.unwrap())
            .unwrap()
            .key
            .as_str()
    };
    // The trades and the monsters, every object typed.
    let objects = &setup.map.objects;
    let trades: usize = KEYS.iter().map(|&(_, n)| n).sum();
    assert_eq!(trades, 49);
    assert_eq!(objects.len(), trades + monsters);
    assert!(objects.iter().all(|o| o.object_type.is_some()));
    assert!((350..450).contains(&monsters), "{monsters} monsters");
    let monster = |o: &&h3_adventure::ObjectSetup| key(o) == "base:monster";
    assert_eq!(objects.iter().filter(monster).count(), monsters);
    // The census's most of each class, those of either level by its share, each at its anchor.
    for (wanted, count) in KEYS {
        let found = objects.iter().filter(|o| key(o) == wanted).count();
        assert_eq!(found, count, "{wanted}");
    }
    let trade_objects: Vec<_> = objects.iter().filter(|o| !monster(o)).collect();
    let expected: Vec<(u16, u16, u8)> = [false, true]
        .into_iter()
        .flat_map(|underground| {
            placed(underground)
                .into_iter()
                .map(move |(_, (x, y))| (x.into(), y.into(), u8::from(underground)))
        })
        .collect();
    let anchors: Vec<(u16, u16, u8)> = (trade_objects.iter())
        .map(|o| (o.anchor.x, o.anchor.y, o.anchor.level))
        .collect();
    assert_eq!(anchors, expected);
    let below = |wanted: &str| {
        (trade_objects.iter())
            .filter(|o| key(o) == wanted && o.anchor.level == 1)
            .count()
    };
    assert_eq!(
        [
            below("base:altar_of_sacrifice"),
            below("base:black_market"),
            below("base:war_machine_factory"),
            below("ab:freelancers_guild"),
            below("base:hill_fort"),
            below("base:trading_post"),
        ],
        [3, 3, 3, 3, 0, 0]
    );
    // Each Black Market draws three treasures, three minors and one major (ADV-MKT-006).
    let game = GameState::new(rules, &setup, 1).unwrap();
    let core = rules.core();
    let mut markets = 0;
    for (index, object) in objects.iter().enumerate() {
        if key(object) != "base:black_market" {
            continue;
        }
        markets += 1;
        let stock = game.adventure().map().objects()[index].resolved.stock();
        let classes: Vec<ArtifactClass> = (stock.iter())
            .map(|&a| core.artifact(a).unwrap().class)
            .collect();
        let of = |class| classes.iter().filter(|&&c| c == class).count();
        assert_eq!(
            (
                of(ArtifactClass::Treasure),
                of(ArtifactClass::Minor),
                of(ArtifactClass::Major)
            ),
            (3, 3, 1),
            "{index}"
        );
    }
    assert_eq!(markets, 6);
}
