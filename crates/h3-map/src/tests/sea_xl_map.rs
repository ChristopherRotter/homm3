//! The sea XL map committed under `fixtures/maps/` (E08 T75b, design §15): SoD, 144 × 144, one
//! level of sea with sixteen islands; eight heroes, four red and four blue, each on its island's
//! east coast beside a boat with a source of sea points, and seven more boats beside other
//! islands (fifteen, as Myth and Legend's, the shipped maps' most: survey `E08-sea.md` §1); the
//! sea's four pickups and Favorable Winds between the islands. Written by [`crate::writer`] as the
//! sea map (`sea_map.rs`, whose templates and helpers it takes); played by the boats' benchmarks
//! (`benches/adventure/`: `reach-sea-xl-144`, the reachable-tiles query of the heroes in boats,
//! and `fork-sea-xl-144`). The tests check that the committed file reads to the map written here
//! and imports with its boats (it is rewritten when `H3_WRITE_FIXTURES` is set), and that the map
//! plays: its heroes board, sail and land, and a fork evolves as its parent.

use std::path::PathBuf;

use proptest::collection::vec;
use proptest::prelude::*;
use proptest::strategy::ValueTree;

use super::play::{FAITHFUL, Step, at_sea, options, resolve};
use super::sea_map::{
    ADMIRALS_HAT, CAPTAINS_HAT, HEAD, NAVIGATION, NECK, NECKLACE, boat, bottle, hero, lines, plain,
    template, wearing,
};
use crate::model::RawMap;
use crate::writer as w;
use crate::{Limits, import, read};
use h3_adventure::{AdventureEvent, AdventureEventKind};
use h3_game::{GameEvent, GameState, apply};

/// The committed file's name.
const NAME: &str = "e08-sea-144.h3m";
/// The map's side.
const SIDE: u8 = 144;
/// The islands' grid: one island in each cell of `CELL` × `CELL` tiles, `ISLAND` tiles square,
/// `SHORE` tiles from the cell's top left corner each way.
const CELL: u8 = 36;
const ISLAND: u8 = 5;
const SHORE: u8 = 15;
/// The heroes' islands by the grid's column and row, in the map's order: red's four, then blue's.
const HOMES: [(u8, u8); 8] = [
    (0, 0),
    (2, 0),
    (1, 1),
    (3, 1),
    (0, 2),
    (2, 2),
    (1, 3),
    (3, 3),
];
/// The heroes' types, in the map's order (SoD's 156: one of each class's first eight in turn).
const TYPES: [u8; 8] = [0, 17, 34, 51, 68, 85, 102, 119];

/// The middle tile of the east coast of island `(i, j)` (a hero's).
const fn coast((i, j): (u8, u8)) -> (u8, u8) {
    (CELL * i + SHORE + ISLAND - 1, CELL * j + SHORE + ISLAND / 2)
}

/// The water tile east of [`coast`] (a boat's).
const fn offshore(island: (u8, u8)) -> (u8, u8) {
    let (x, y) = coast(island);
    (x + 1, y)
}

/// Every island, row by row.
fn islands() -> impl Iterator<Item = (u8, u8)> {
    (0..SIDE / CELL).flat_map(|j| (0..SIDE / CELL).map(move |i| (i, j)))
}

/// The map: SoD, 144 × 144 of water with sixteen grass islands, no underground.
///
/// - The heroes (red's four, then blue's) on their islands' east coasts, each with Navigation (basic,
///   advanced, expert in turn) and, in turn, the Admiral's Hat (free boarding), the Necklace of
///   Ocean Guidance, the Sea Captain's Hat or nothing; each hero's boat on the water east of it.
/// - Seven more boats east of the other islands but the last; the boats' types 0, 1, 2 in turn.
/// - In each cell's sea north-east of its island, one of the four pickups in turn (Flotsam, a Sea
///   Chest, the Shipwreck Survivor, a bottle with a message); in every other cell's south-west,
///   Favorable Winds (`AVXfw5`, 3 × 3).
fn map() -> w::Map {
    let mut players: [Option<w::Player>; w::PLAYERS] = Default::default();
    for player in &mut players[..2] {
        *player = Some(w::Player {
            human: true,
            computer: true,
            towns: 1,
        });
    }
    let patches = islands()
        .map(|(i, j)| {
            let from = (CELL * i + SHORE, CELL * j + SHORE);
            w::Patch {
                terrain: w::Terrain::Grass,
                from,
                to: (from.0 + ISLAND - 1, from.1 + ISLAND - 1),
            }
        })
        .collect();
    let mut objects = Vec::new();
    for (n, (&home, &hero_type)) in (0u8..).zip(HOMES.iter().zip(&TYPES)) {
        let mut h = hero(coast(home), n / 4, hero_type);
        h.skills = Some(vec![(NAVIGATION, 1 + n % 3)]);
        match n % 4 {
            0 => wearing(&mut h, HEAD, ADMIRALS_HAT, Vec::new()),
            1 => wearing(&mut h, NECK, NECKLACE, Vec::new()),
            2 => wearing(&mut h, HEAD, CAPTAINS_HAT, Vec::new()),
            _ => {}
        }
        objects.push(w::MapObject::Hero(h));
    }
    let others = islands().filter(|island| !HOMES.contains(island)).take(7);
    for (n, island) in HOMES.into_iter().chain(others).enumerate() {
        objects.push(boat(n % 3, offshore(island)));
    }
    for (n, (i, j)) in islands().enumerate() {
        let at = (CELL * i + 30, CELL * j + 6);
        objects.push(match n % 4 {
            0 => plain(lines::FLOTSAM, at),
            1 => plain(lines::SEA_CHEST, at),
            2 => plain(lines::SURVIVOR, at),
            _ => bottle(at, "Land ho"),
        });
        if n % 2 == 0 {
            objects.push(w::MapObject::Ground(w::Ground {
                tile: (CELL * i + 6, CELL * j + 30),
                template: template(lines::WINDS),
            }));
        }
    }
    w::Map {
        name: "Sea XL".into(),
        description: "E08 T75b: the boats' benchmarks".into(),
        size: SIDE,
        terrain: w::Terrain::Water,
        patches,
        roads: Vec::new(),
        players,
        objects,
        underground: None,
        level_limit: 0,
        banned_skills: Vec::new(),
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

#[test]
fn the_sea_xl_map_is_committed_as_written() {
    let setup = import(&committed(), &FAITHFUL).unwrap();
    let heroes: Vec<_> = setup.heroes.iter().map(|h| (h.pos.x, h.pos.y)).collect();
    let expected: Vec<_> = HOMES
        .iter()
        .map(|&home| coast(home))
        .map(|(x, y)| (u16::from(x), u16::from(y)))
        .collect();
    assert_eq!(heroes, expected);
    assert_eq!(setup.map.boats.len(), 15);
    let kinds: Vec<u8> = setup.map.boats.iter().map(|b| b.kind).collect();
    assert_eq!(kinds, [0, 1, 2].repeat(5));
    // The sixteen pickups and the eight winds, typed.
    assert_eq!(setup.map.objects.len(), 16 + 8);
    assert!(setup.map.objects.iter().all(|o| o.object_type.is_some()));
}

/// The sea XL map plays (E08 T75b): 400 steps of random play that prefer a boarding or a move of
/// a hero in a boat board, sail and land; then a fork playing on evolves as its parent does under
/// the same steps, and leaves the parent as it was.
// covers: ADV-SEA-001, ADV-SEA-002, ADV-SEA-003
#[test]
fn the_sea_xl_map_plays_and_forks() {
    let mut runner = proptest::test_runner::TestRunner::deterministic();
    let steps = vec(
        prop_oneof![
            6 => any::<u16>().prop_map(Step::Legal),
            1 => Just(Step::EndTurn),
        ],
        460,
    );
    let rules = &*FAITHFUL;
    let setup = import(&committed(), rules).unwrap();
    let mut state = GameState::new(rules, &setup, 1).unwrap();
    let steps = steps.new_tree(&mut runner).unwrap().current();
    let (played, rest) = steps.split_at(400);
    let (mut boardings, mut landings) = (0, 0);
    let play = |state: &mut GameState, step: &Step| {
        let legal = at_sea(state, options(state, rules));
        let (actor, command) = resolve(state, rules, &legal, step);
        let mut events = Vec::new();
        let _ = apply(state, rules, actor, command, &mut events);
        events
    };
    for step in played {
        for event in play(&mut state, step) {
            if let GameEvent::Adventure(AdventureEvent { kind, .. }) = event {
                boardings += usize::from(matches!(kind, AdventureEventKind::Boarded { .. }));
                landings += usize::from(matches!(kind, AdventureEventKind::Landed { .. }));
            }
        }
        assert!(state.adventure().check_boats(rules.adventure()));
    }
    assert!(boardings >= 4 && landings >= 1, "{boardings} {landings}");
    let parent = state.clone();
    let mut fork = state.clone();
    for step in rest {
        play(&mut state, step);
        play(&mut fork, step);
        assert_eq!(state, fork);
    }
    assert_eq!(state.state_hash(), fork.state_hash());
    assert_ne!(parent.state_hash(), state.state_hash(), "nothing played");
    let mut again = parent.clone();
    for step in rest {
        play(&mut again, step);
    }
    assert_eq!(again, state, "the parent changed");
}
