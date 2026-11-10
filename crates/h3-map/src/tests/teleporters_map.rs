//! The teleporters map committed under `fixtures/maps/` (E08 T106a, increment 11's design note
//! §14): SoD, 36 × 36 of grass with the sea on columns 24 to 35, over an underground of
//! subterranean ground; red in the north, blue in the south. A one-way channel of one entrance and
//! three exits (channel 0, two exits on the surface, one underground), a one-way channel of one
//! entrance and a lone exit (channel 1: its battles, its "no candidate" and its "leads nowhere"
//! are deterministic), a two-way channel of three (channel 0, one member underground, one guarded
//! by a monster beside it), two gate pairs and an underground gate without a pair, three
//! Whirlpools in the sea with three boats (two by red's coast, one by blue's), the Sea Captain's
//! Hat on one hero, an army of several stacks with a tie, a lone creature, a Windmill and a
//! Guardhouse to revisit. Written by [`crate::writer`] with the original's templates; the golden
//! replays play it (`crates/h3-cli/tests/adventure_replays.rs`). The test checks that the
//! committed file reads to the map written here (it is rewritten when `H3_WRITE_FIXTURES` is
//! set), imports with every class typed and builds the channels, the gates' pairs and the
//! Whirlpools' list this file's layout says.

use std::path::PathBuf;

use super::play::FAITHFUL;
use super::sea_map::{CAPTAINS_HAT, HEAD, boat, hero, plain, wearing};
use crate::model::RawMap;
use crate::writer as w;
use crate::{Limits, import, read};
use h3_adventure::{BoatSetup, Pos};
use h3_game::GameState;

/// The committed file's name.
const NAME: &str = "e08-teleporters-36.h3m";

/// The original's templates (`Objects.txt`) of the objects placed here. A monolith is blocked and
/// visited on its anchor; a gate covers its anchor, the two tiles left of it and the one above
/// the middle one, and is visited from the middle one; a Whirlpool covers its anchor's row and the
/// row above, three columns ending at the anchor, all six visited; a Guardhouse is visited on its
/// anchor, a Windmill one left of it.
mod lines {
    pub(super) const ENTRANCE_0: &str = "AVXmn1b0.def \
        011111111111111111111111111111111111111111111111 \
        100000000000000000000000000000000000000000000000 011111111 011111111 43 0 0 0";
    pub(super) const ENTRANCE_1: &str = "AVXmn1r0.def \
        011111111111111111111111111111111111111111111111 \
        100000000000000000000000000000000000000000000000 011111111 011111111 43 1 0 0";
    pub(super) const EXIT_0: &str = "AVXmx1b0.def \
        011111111111111111111111111111111111111111111111 \
        100000000000000000000000000000000000000000000000 011111111 011111111 44 0 0 0";
    pub(super) const EXIT_1: &str = "AVXmx1r0.def \
        011111111111111111111111111111111111111111111111 \
        100000000000000000000000000000000000000000000000 011111111 011111111 44 1 0 0";
    pub(super) const TWO_WAY_0: &str = "AVXmn2g0.def \
        011111111111111111111111111111111111111111111111 \
        100000000000000000000000000000000000000000000000 011111111 011111111 45 0 0 0";
    pub(super) const GATE: &str = "AvTCave.def \
        000111111011111111111111111111111111111111111111 \
        010000000000000000000000000000000000000000000000 011111111 011111111 103 0 0 0";
    pub(super) const WHIRLPOOL: &str = "AVXwhrl0.def \
        000111110001111111111111111111111111111111111111 \
        111000001110000000000000000000000000000000000000 100000000 100000000 111 0 0 1";
    pub(super) const GNOLLS: &str = "AVWgnll0.def \
        011111111111111111111111111111111111111111111111 \
        100000000000000000000000000000000000000000000000 111111111 000000001 54 98 2 0";
}

/// The heroes' tiles and levels, by hero id (the map's order, the surface's first): red's Orrin
/// (0), Sylvia (1), Sorsha (2), Edric (3), Lord Haart (4) and Christian (5); blue's Rion (6),
/// Adela (7) and Cuthbert (8); underground red's Valeska (9) and Tyris (10), blue's Ingham (11).
pub(super) const HEROES: [(u8, u8, u8); 12] = [
    (18, 3, 0),
    (23, 7, 0),
    (23, 11, 0),
    (8, 4, 0),
    (8, 9, 0),
    (6, 15, 0),
    (3, 25, 0),
    (7, 31, 0),
    (23, 27, 0),
    (11, 8, 1),
    (11, 10, 1),
    (10, 24, 1),
];
/// The heroes' owners and types, by hero id.
const HERO_TYPES: [(u8, u8); 12] = [
    (0, 0),
    (0, 3),
    (0, 5),
    (0, 2),
    (0, 4),
    (0, 6),
    (1, 8),
    (1, 9),
    (1, 10),
    (0, 1),
    (0, 7),
    (1, 12),
];
/// The empty boats' water tiles (all of kind 0): Sylvia's, Sorsha's and Cuthbert's.
pub(super) const BOATS: [(u8, u8); 3] = [(24, 7), (24, 11), (24, 27)];

/// Channel 0's one-way entrance, on the surface beside Edric.
pub(super) const ENTRANCE_0: (u8, u8, u8) = (10, 4, 0);
/// Channel 0's one-way exits, in the walk's order (level, row, column).
pub(super) const EXITS_0: [(u8, u8, u8); 3] = [(16, 18, 0), (12, 30, 0), (4, 14, 1)];
/// Channel 1's one-way entrance, between Orrin and Sylvia, and its lone exit, between Rion and
/// Adela.
pub(super) const ENTRANCE_1: (u8, u8, u8) = (20, 5, 0);
pub(super) const EXIT_1: (u8, u8, u8) = (5, 28, 0);
/// Channel 0's two-way monoliths, in the walk's order: beside Lord Haart, the guarded one, the
/// underground one.
pub(super) const TWO_WAY_0: [(u8, u8, u8); 3] = [(10, 9, 0), (16, 12, 0), (20, 20, 1)];
/// The gnolls beside the guarded two-way monolith, and their count.
pub(super) const GNOLLS: ((u8, u8, u8), u16) = ((17, 12, 0), 4);
/// The gates' anchors: on the surface, then underground (the pairs by their entries, one left of
/// the anchors: (13, 6) with (13, 6), (7, 21) with (7, 21); the underground (21, 6) left over).
pub(super) const GATES: [(u8, u8, u8); 5] =
    [(14, 6, 0), (8, 21, 0), (14, 6, 1), (22, 6, 1), (8, 21, 1)];
/// The Whirlpools' anchors, each covering its row and the one above, three columns ending there.
pub(super) const WHIRLPOOLS: [(u8, u8); 3] = [(30, 5), (30, 17), (30, 29)];
/// The neutral Guardhouse (visited on its anchor) and the Windmill (anchored one right of the
/// tile it is visited from).
pub(super) const GUARDHOUSE: (u8, u8) = (9, 15);
pub(super) const WINDMILL: (u8, u8) = (13, 16);

/// Experience a little below level 2 (1 000), so that Orrin's first battle gives him a level-up.
const NEAR_LEVEL_2: u32 = 900;
/// Creature codes: Pikemen, Angels, Archangels.
const PIKEMAN: u16 = 0;
const ANGEL: u16 = 12;
const ARCHANGEL: u16 = 13;

fn army(stacks: &[(u16, u16)]) -> [Option<(u16, u16)>; w::ARMY_SLOTS] {
    let mut army = [None; w::ARMY_SLOTS];
    for (slot, &stack) in army.iter_mut().zip(stacks) {
        *slot = Some(stack);
    }
    army
}

/// An object standing on its anchor `(x, y)`, the level dropped (the caller puts it on its
/// level's list).
fn on(line: &str, (x, y, _): (u8, u8, u8)) -> w::MapObject {
    plain(line, (x, y))
}

/// The map's heroes on `level`, by hero id:
///
/// - Orrin (0) with ten archangels and 900 experience; Sylvia (1) with the Sea Captain's Hat, her
///   type's army, beside her boat; Sorsha (2) beside hers with three angels and two stacks of ten
///   pikemen (the Whirlpool's weakest: the second slot, tied with the third); Edric (3) beside
///   channel 0's entrance, Lord Haart (4) beside the first two-way monolith, Christian (5) by the
///   Guardhouse and the Windmill; each but Orrin, Sylvia and Sorsha their type's own.
/// - Blue's Rion (6), his type's army, and Adela (7) with five angels, each beside channel 1's
///   exit; Cuthbert (8) with one pikeman beside blue's boat.
/// - Underground: red's Valeska (9) and Tyris (10) by the first gate's pair, blue's Ingham (11)
///   by the second's.
fn heroes(level: u8) -> Vec<w::MapObject> {
    HEROES
        .iter()
        .zip(HERO_TYPES)
        .enumerate()
        .filter(|(_, (tile, _))| tile.2 == level)
        .map(|(id, (&(x, y, _), (owner, hero_type)))| {
            let mut h = hero((x, y), owner, hero_type);
            match id {
                0 => {
                    h.army = Some(army(&[(ARCHANGEL, 10)]));
                    h.experience = Some(NEAR_LEVEL_2);
                }
                1 => wearing(&mut h, HEAD, CAPTAINS_HAT, Vec::new()),
                2 => h.army = Some(army(&[(ANGEL, 3), (PIKEMAN, 10), (PIKEMAN, 10)])),
                7 => h.army = Some(army(&[(ANGEL, 5)])),
                8 => h.army = Some(army(&[(PIKEMAN, 1)])),
                _ => {}
            }
            w::MapObject::Hero(h)
        })
        .collect()
}

/// The objects on `level` other than the heroes, in the map's order.
fn objects(level: u8) -> Vec<w::MapObject> {
    let here = |&tile: &(u8, u8, u8)| tile.2 == level;
    let mut objects = Vec::new();
    objects.extend(
        [ENTRANCE_0]
            .iter()
            .filter(|t| here(t))
            .map(|&t| on(lines::ENTRANCE_0, t)),
    );
    objects.extend(
        EXITS_0
            .iter()
            .filter(|t| here(t))
            .map(|&t| on(lines::EXIT_0, t)),
    );
    objects.extend(
        [ENTRANCE_1]
            .iter()
            .filter(|t| here(t))
            .map(|&t| on(lines::ENTRANCE_1, t)),
    );
    objects.extend(
        [EXIT_1]
            .iter()
            .filter(|t| here(t))
            .map(|&t| on(lines::EXIT_1, t)),
    );
    objects.extend(
        TWO_WAY_0
            .iter()
            .filter(|t| here(t))
            .map(|&t| on(lines::TWO_WAY_0, t)),
    );
    objects.extend(
        GATES
            .iter()
            .filter(|t| here(t))
            .map(|&t| on(lines::GATE, t)),
    );
    if level == 0 {
        let (at, count) = GNOLLS;
        objects.push(w::MapObject::Monster(w::Monster {
            tile: (at.0, at.1),
            template: super::sea_map::template(lines::GNOLLS),
            count,
            disposition: 4,
            never_flees: true,
            fixed_count: true,
            treasure: None,
        }));
        objects.extend(WHIRLPOOLS.map(|at| plain(lines::WHIRLPOOL, at)));
        objects.extend(BOATS.map(|at| boat(0, at)));
        objects.push(w::MapObject::Owned(w::OwnedObject {
            tile: GUARDHOUSE,
            template: super::sea_map::template(super::dwellings::LINES[0]),
            body: w::OwnedBody::Owner(None),
        }));
        objects.push(plain(super::economy_map::lines::WINDMILL, WINDMILL));
    }
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
    let level = |z| heroes(z).into_iter().chain(objects(z)).collect();
    w::Map {
        name: "Teleporters".into(),
        description: "E08 T106a: monoliths, gates and Whirlpools".into(),
        size: 36,
        terrain: w::Terrain::Grass,
        patches: vec![w::Patch {
            terrain: w::Terrain::Water,
            from: (24, 0),
            to: (35, 35),
        }],
        roads: Vec::new(),
        players,
        objects: level(0),
        underground: Some(w::Underground {
            terrain: w::Terrain::Subterranean,
            patches: Vec::new(),
            objects: level(1),
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

fn at((x, y, z): (u8, u8, u8)) -> Pos {
    Pos::new(x.into(), y.into(), z)
}

/// The object types on the map, by key (the boats are the map's boats, no objects).
const TYPES: [&str; 9] = [
    "base:monolith_entrance_0",
    "base:monolith_entrance_1",
    "base:monolith_exit_0",
    "base:monolith_exit_1",
    "base:monolith_two_way_0",
    "base:subterranean_gate",
    "base:whirlpool",
    "base:guardhouse",
    "base:windmill",
];

// covers: ADV-TEL-001, ADV-TEL-002
#[test]
fn the_teleporters_map_is_committed_as_written_and_builds_its_channels() {
    let rules = &*FAITHFUL;
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
        BOATS.map(|(x, y)| BoatSetup {
            tile: at((x, y, 0)),
            kind: 0
        })
    );
    let adventure = rules.adventure();
    for key in TYPES {
        let kind = adventure.object_type_by_key(key).unwrap();
        assert!(
            setup
                .map
                .objects
                .iter()
                .any(|o| o.object_type == Some(kind)),
            "{key}"
        );
    }
    let game = GameState::new(rules, &setup, 1).unwrap();
    let channels = game.adventure().map().channels();
    assert_eq!(channels.exits(0), EXITS_0.map(at));
    assert_eq!(channels.exits(1), [at(EXIT_1)]);
    assert_eq!(channels.two_way(0), TWO_WAY_0.map(at));
    assert_eq!(channels.one_way_channels().count(), 2);
    assert_eq!(channels.two_way_channels().count(), 1);
    // The gates' entries in the walk's order, each with its pair: the two surface gates take the
    // ones below them, the underground (21, 6) is left over.
    let entry = |(x, y, z): (u8, u8, u8)| at((x - 1, y, z));
    let [first, second, first_pair, lone, second_pair] = GATES.map(entry);
    let gates: Vec<(Pos, Option<Pos>)> = (channels.gates().iter())
        .map(|gate| (gate.tile, gate.pair))
        .collect();
    assert_eq!(
        gates,
        [
            (first, Some(first_pair)),
            (second, Some(second_pair)),
            (first_pair, Some(first)),
            (lone, None),
            (second_pair, Some(second)),
        ]
    );
    // Every Whirlpool's six tiles, none under a boat.
    let mut tiles: Vec<Pos> = (WHIRLPOOLS.iter())
        .flat_map(|&(x, y)| {
            [y - 1, y]
                .into_iter()
                .flat_map(move |row| (x - 2..=x).map(move |column| at((column, row, 0))))
        })
        .collect();
    tiles.sort_by_key(|pos| (pos.level, pos.y, pos.x));
    let listed: Vec<Pos> = channels.whirlpools().iter().map(|w| w.tile).collect();
    assert_eq!(listed, tiles);
}
