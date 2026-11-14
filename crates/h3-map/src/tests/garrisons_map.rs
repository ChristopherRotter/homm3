//! The garrisons map committed under `fixtures/maps/` (E08 T116a, increment 12's design note §16):
//! SoD, 36 × 36 of grass cut into four quarters by rock, red in the north, blue in the south, and
//! every way between two quarters a map garrison: across the row of rock between the halves red's
//! armed garrison (33.0), blue's armed one (33.0), a neutral armed anti-magic one (33.1) and a
//! neutral empty one (33.0); across the column of rock between the west and the east red's
//! non-removable armed vertical garrison (219.0) in the north and blue's non-removable armed
//! anti-magic vertical one (219.1) in the south. A hero of each player stands beside each
//! garrison; red's and blue's level-1 heroes beside the neutral armed one and red's armed one
//! carry experience a little below level 2, so their first battle brings a level-up. Written by
//! [`crate::writer`] with the original's templates; the golden replays play it
//! (`crates/h3-cli/tests/adventure_replays.rs`). The test checks that the committed file reads to
//! the map written here (it is rewritten when `H3_WRITE_FIXTURES` is set), imports each garrison
//! with its owner, army and removable flag, and builds.

use std::path::PathBuf;

use super::garrisons_bonuses::{GARRISONS, army, garrison, troops};
use super::import::SOD;
use super::sea_map::hero;
use crate::model::RawMap;
use crate::writer as w;
use crate::{Limits, import, read};
use h3_adventure::{Holds, Pos};
use h3_contracts::PlayerId;
use h3_game::GameState;

/// The committed file's name.
const NAME: &str = "e08-garrisons-36.h3m";

/// The heroes' tiles, by hero id (the map's order): red's Orrin (0) above red's garrison, Ingham
/// (1) above the neutral armed one, Edric (2) above blue's, Sylvia (3) west of red's vertical
/// one; blue's Rion (4) below red's garrison, Adela (5) below blue's, Cuthbert (6) below the
/// neutral empty one, Adelaide (7) east of blue's vertical one.
pub(super) const HEROES: [(u8, u8); 8] = [
    (5, 16),
    (21, 16),
    (13, 16),
    (17, 8),
    (5, 18),
    (13, 18),
    (29, 18),
    (19, 26),
];
/// The heroes' owners and types, by hero id.
const HERO_TYPES: [(u8, u8); 8] = [
    (0, 0),
    (0, 12),
    (0, 2),
    (0, 3),
    (1, 8),
    (1, 9),
    (1, 10),
    (1, 11),
];

/// The row of rock between the halves and the column between the west and the east.
const WALL_ROW: u8 = 17;
const WALL_COLUMN: u8 = 18;

/// A garrison: its anchor, its `GARRISONS` line (0 horizontal, 1 horizontal anti-magic, 2
/// vertical, 3 vertical anti-magic), its owner, its army (creature code, count) and its
/// removable flag.
pub(super) struct Placed {
    pub(super) anchor: (u8, u8),
    pub(super) line: usize,
    pub(super) owner: Option<u8>,
    pub(super) army: &'static [(u16, u16)],
    pub(super) removable: bool,
}

/// Creature codes: Pikemen, Archers, Griffins, Swordsmen, Monks, Angels, Archangels.
const PIKEMAN: u16 = 0;
const ARCHER: u16 = 2;
const GRIFFIN: u16 = 4;
const SWORDSMAN: u16 = 6;
const MONK: u16 = 8;
const ANGEL: u16 = 12;
const ARCHANGEL: u16 = 13;

/// The garrisons in the map's order (the objects' order after the heroes): each horizontal one
/// covers its anchor and the two tiles left of it on the row of rock, its entrance the middle
/// one, entered from the three tiles above and the three below; each vertical one covers its
/// anchor and the two tiles above it on the column of rock, its entrance the middle one, entered
/// from the three tiles left and the three right.
pub(super) const GARRISONS_PLACED: [Placed; 6] = [
    // Red's armed horizontal garrison, entrance (5, 17), between the west quarters.
    Placed {
        anchor: (6, WALL_ROW),
        line: 0,
        owner: Some(0),
        army: &[(PIKEMAN, 10), (ARCHER, 5)],
        removable: true,
    },
    // Blue's armed horizontal garrison, entrance (13, 17), between the west quarters.
    Placed {
        anchor: (14, WALL_ROW),
        line: 0,
        owner: Some(1),
        army: &[(GRIFFIN, 4)],
        removable: true,
    },
    // A neutral armed anti-magic horizontal garrison, entrance (21, 17), between the east ones.
    Placed {
        anchor: (22, WALL_ROW),
        line: 1,
        owner: None,
        army: &[(PIKEMAN, 20), (SWORDSMAN, 4)],
        removable: true,
    },
    // A neutral empty horizontal garrison, entrance (29, 17), between the east quarters.
    Placed {
        anchor: (30, WALL_ROW),
        line: 0,
        owner: None,
        army: &[],
        removable: true,
    },
    // Red's armed non-removable vertical garrison, entrance (18, 8), between the north quarters.
    Placed {
        anchor: (WALL_COLUMN, 9),
        line: 2,
        owner: Some(0),
        army: &[(GRIFFIN, 3)],
        removable: false,
    },
    // Blue's armed non-removable anti-magic vertical garrison, entrance (18, 26), between the
    // south quarters.
    Placed {
        anchor: (WALL_COLUMN, 27),
        line: 3,
        owner: Some(1),
        army: &[(MONK, 3), (ARCHER, 6)],
        removable: false,
    },
];

/// A garrison's entrance: one left of a horizontal one's anchor, one above a vertical one's.
pub(super) fn entrance(placed: &Placed) -> (u8, u8) {
    let (x, y) = placed.anchor;
    if placed.line < 2 {
        (x - 1, y)
    } else {
        (x, y - 1)
    }
}

/// Experience a little below level 2 (1 000), so that a first battle gives a level-up.
const NEAR_LEVEL_2: u32 = 900;

/// The map's heroes, by hero id: Orrin (0) with eight pikemen, four archers and two griffins;
/// Ingham (1) with ten archangels and 900 experience; Edric (2) with one pikeman; Sylvia (3)
/// with three swordsmen; Rion (4) with five angels and 900 experience; the others their type's
/// own.
fn heroes() -> Vec<w::MapObject> {
    HEROES
        .iter()
        .zip(HERO_TYPES)
        .enumerate()
        .map(|(id, (&tile, (owner, hero_type)))| {
            let mut h = hero(tile, owner, hero_type);
            match id {
                0 => h.army = Some(army(&[(PIKEMAN, 8), (ARCHER, 4), (GRIFFIN, 2)])),
                1 => {
                    h.army = Some(army(&[(ARCHANGEL, 10)]));
                    h.experience = Some(NEAR_LEVEL_2);
                }
                2 => h.army = Some(army(&[(PIKEMAN, 1)])),
                3 => h.army = Some(army(&[(SWORDSMAN, 3)])),
                4 => {
                    h.army = Some(army(&[(ANGEL, 5)]));
                    h.experience = Some(NEAR_LEVEL_2);
                }
                _ => {}
            }
            w::MapObject::Hero(h)
        })
        .collect()
}

/// The rock: the row between the halves and the column between the west and the east, each
/// left open only where a garrison stands.
fn rock() -> Vec<w::Patch> {
    let rock = |from, to| w::Patch {
        terrain: w::Terrain::Rock,
        from,
        to,
    };
    let row = WALL_ROW;
    let column = WALL_COLUMN;
    vec![
        rock((0, row), (3, row)),
        rock((7, row), (11, row)),
        rock((15, row), (19, row)),
        rock((23, row), (27, row)),
        rock((31, row), (35, row)),
        rock((column, 0), (column, 6)),
        rock((column, 10), (column, 16)),
        rock((column, 18), (column, 24)),
        rock((column, 28), (column, 35)),
    ]
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
    let mut objects = heroes();
    objects.extend(
        GARRISONS_PLACED
            .iter()
            .map(|g| garrison(g.anchor, GARRISONS[g.line], g.owner, g.army, g.removable)),
    );
    w::Map {
        name: "Garrisons".into(),
        description: "E08 T116a: owned, neutral, empty, non-removable and anti-magic garrisons"
            .into(),
        size: 36,
        terrain: w::Terrain::Grass,
        patches: rock(),
        roads: Vec::new(),
        players,
        objects,
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

/// The garrisons' types by key, in the map's order.
const TYPES: [&str; 6] = [
    "base:garrison",
    "base:garrison",
    "base:anti_magic_garrison",
    "base:garrison",
    "ab:garrison_vertical",
    "ab:anti_magic_garrison_vertical",
];

// covers: ADV-GAR-001
#[test]
fn the_garrisons_map_is_committed_as_written_and_imports_its_garrisons() {
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
    let part = rules.adventure();
    let placed: Vec<_> = (setup.map.objects.iter())
        .map(|o| {
            let key = part
                .object_type(o.object_type.unwrap())
                .unwrap()
                .key
                .as_str();
            (key, o.anchor, o.owner, o.holds.clone())
        })
        .collect();
    let expected: Vec<_> = GARRISONS_PLACED
        .iter()
        .zip(TYPES)
        .map(|(g, key)| {
            let stacks: Vec<(u16, u32)> = g.army.iter().map(|&(c, n)| (c, n.into())).collect();
            let holds = Holds::Garrison {
                army: troops(&stacks),
                removable: g.removable,
            };
            (key, at(g.anchor), g.owner.map(PlayerId::new), holds)
        })
        .collect();
    assert_eq!(placed, expected);
    // The entrances are where the garrisons are visited, each on grass between two rocks.
    let entrances: Vec<(u8, u8)> = GARRISONS_PLACED.iter().map(entrance).collect();
    assert_eq!(
        entrances,
        [(5, 17), (13, 17), (21, 17), (29, 17), (18, 8), (18, 26)]
    );
    GameState::new(rules, &setup, 1).unwrap();
}
