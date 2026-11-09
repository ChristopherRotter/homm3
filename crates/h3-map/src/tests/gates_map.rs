//! The gates XL map committed under `fixtures/maps/` (E08 T104, increment 11's design note §5,
//! §14): SoD, 144 × 144 of grass over an underground of subterranean ground (the XL map's 41 472
//! tiles), a Subterranean Gate (the original's template, `AvTCave`: four tiles, visited one left of
//! its anchor) on every fifth tile of both levels, about 8 000, the gates' pairing's worst case
//! (quadratic in the gates, ADV-TEL-002); red's and blue's heroes on each level, as decision 133's
//! full maps (`full_maps.rs`). Written by [`crate::writer`]; built by the benchmark
//! `build-gates-xl-144` (`benches/adventure/`: the game's build). The test checks that the
//! committed file reads to the map written here (it is rewritten when `H3_WRITE_FIXTURES` is set),
//! imports and builds with every gate paired to the one below it.

use std::path::PathBuf;

use crate::model::RawMap;
use crate::writer as w;
use crate::{Limits, import, read};
use h3_adventure::{ObjectKind, Pos};
use h3_game::GameState;

use super::play::FAITHFUL;

/// The committed file's name.
const NAME: &str = "e08-gates-144.h3m";
/// The map's side.
const SIDE: u8 = 144;
/// A gate on every this many tiles of a level, counted row by row.
const EVERY: usize = 5;
/// The heroes' tiles, on both levels: red's in the north-west, blue's in the south-east.
const HEROES: [(u8, u8); 2] = [(8, 8), (135, 135)];
/// The gate's template (`Objects.txt`, as `teleporters.rs` carries it): blocked on its anchor, the
/// two tiles left of it and the tile above the middle one, visited from the middle one.
const GATE: &str = "AvTCave.def 000111111011111111111111111111111111111111111111 \
    010000000000000000000000000000000000000000000000 011111111 011111111 103 0 0 0";

/// The tiles a gate anchored on `(x, y)` covers.
fn covered((x, y): (u8, u8)) -> [(u8, u8); 4] {
    [(x, y), (x - 1, y), (x - 2, y), (x - 1, y - 1)]
}

/// The gates' anchors on a level, row by row: every fifth tile whose gate lies on the map whole
/// and covers no hero's tile.
fn anchors() -> impl Iterator<Item = (u8, u8)> {
    let tiles = (0..SIDE).flat_map(|y| (0..SIDE).map(move |x| (x, y)));
    tiles
        .enumerate()
        .filter(|&(index, (x, y))| index % EVERY == 0 && x >= 2 && y >= 1)
        .map(|(_, tile)| tile)
        .filter(|&tile| !covered(tile).iter().any(|t| HEROES.contains(t)))
}

/// One level's objects: red's and blue's hero (of these types), then the gates.
fn level(types: [u8; 2]) -> Vec<w::MapObject> {
    let gate = w::ObjectTemplate::from_objects_txt(GATE).unwrap();
    let heroes = HEROES
        .iter()
        .zip(types)
        .zip(0..)
        .map(|((&tile, hero_type), owner)| {
            w::MapObject::Hero(w::Hero {
                tile,
                owner,
                hero_type,
                class: hero_type / 8,
                name: None,
                experience: None,
                skills: None,
                army: None,
                tight_formation: false,
                artifacts: None,
                spells: None,
                primary: None,
            })
        });
    let gates = anchors().map(|tile| {
        w::MapObject::Plain(w::Plain {
            tile,
            template: gate.clone(),
        })
    });
    heroes.chain(gates).collect()
}

/// The map: SoD, 144 × 144 of grass over an underground of subterranean ground; red and blue
/// with a hero on each level, a gate on every fifth tile.
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
        name: "Gates everywhere".into(),
        description: "E08 T104: a Subterranean Gate on every fifth tile of both levels".into(),
        size: SIDE,
        terrain: w::Terrain::Grass,
        patches: Vec::new(),
        roads: Vec::new(),
        players,
        objects: level([0, 25]),
        underground: Some(w::Underground {
            terrain: w::Terrain::Subterranean,
            patches: Vec::new(),
            objects: level([9, 34]),
        }),
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

// covers: ADV-TEL-002
#[test]
fn the_gates_map_is_committed_as_written_and_pairs_every_gate_below_it() {
    let rules = &*FAITHFUL;
    let setup = import(&committed(), rules).unwrap();
    let per_level = anchors().count();
    let gates = setup
        .map
        .objects
        .iter()
        .filter(|o| {
            o.object_type
                .and_then(|id| rules.adventure().object_type(id))
                .is_some_and(|ty| ty.kind == ObjectKind::SubterraneanGate)
        })
        .count();
    assert_eq!(gates, 2 * per_level);
    assert!((8_000..8_300).contains(&gates), "{gates} gates");
    assert_eq!(setup.heroes.len(), 4);
    let game = GameState::new(rules, &setup, 1).unwrap();
    let channels = game.adventure().map().channels();
    assert_eq!(channels.gates().len(), gates);
    // Each surface gate's entry, in the walk's order, finds the one below it at distance 0
    // unpaired: the underground's are only ever taken from above.
    let below = |tile: Pos| Pos::new(tile.x, tile.y, 1 - tile.level);
    assert!(
        channels
            .gates()
            .iter()
            .all(|gate| gate.pair == Some(below(gate.tile)))
    );
}
