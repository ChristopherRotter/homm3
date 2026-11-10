//! The teleporters XL map committed under `fixtures/maps/` (E08 T106c, increment 11's design note
//! §14): SoD, 144 × 144 of grass with a sea across rows 60 to 83, over an underground of
//! subterranean ground (the XL map's 41 472 tiles), holding the shipped maps' most of each
//! teleporter class on one map (`examples/original_data.rs`' census, survey `E08-teleporters.md`
//! §1): eight one-way entrances and eight exits over four channels (the most one-way channels on
//! one map; two entrances and two exits each), seventeen two-way monoliths over six channels (the
//! most), 26 Subterranean Gates (thirteen a level) and nineteen Whirlpools in the sea, each with the
//! original's template of its class and subclass (`Objects.txt`; the two-way channels 4 and 5 two
//! tiles wide, visited left of the anchor). Each land teleporter takes a cell of 12 × 12 tiles of
//! its own, the cells drawn by a fixed stride through both levels' land cells, so that each
//! channel's ends lie far apart (a one-way channel's exits on both levels); the Whirlpools take the
//! sea's cells by a stride of their own. Red's five heroes in the north and underground, blue's five in the south and
//! underground, one of each by the sea beside a boat; a monster on every hundredth free land tile,
//! counted row by row over both levels. Written by [`crate::writer`] as the sea XL map
//! (`sea_xl_map.rs`); played by the benchmark `reach-teleporters-xl-144` (`benches/adventure/`: the
//! walkers' reachable-tiles queries, which meet the entrances as stop nodes). The test checks that
//! the committed file reads to the map written here (it is rewritten when `H3_WRITE_FIXTURES` is
//! set), imports with every class typed and builds the channels, the gates' pairs and the
//! Whirlpools' tiles of this file's layout.

use std::path::PathBuf;

use super::play::FAITHFUL;
use super::sea_map::{boat, hero, plain, template};
use crate::model::RawMap;
use crate::writer as w;
use crate::{Limits, import, read};
use h3_adventure::{BoatSetup, Pos};
use h3_game::GameState;

/// The committed file's name.
const NAME: &str = "e08-teleporters-144.h3m";
/// The map's side.
const SIDE: u8 = 144;
/// A cell's side; a level holds `SIDE / CELL` × `SIDE / CELL` cells.
const CELL: u8 = 12;
/// The surface's sea: the cells' rows 5 and 6 (the map's rows 60 to 83), every column.
const SEA_ROWS: [u8; 2] = [5, 6];
/// The stride through the land cells (both levels' in order, 264; coprime with it), and through
/// the sea's 24 cells (coprime with them).
const LAND_STRIDE: usize = 101;
const SEA_STRIDE: usize = 5;
/// A monster on every this many free land tiles, counted row by row (the surface's first).
const EVERY: usize = 100;

/// The original's templates (`Objects.txt`) of the objects placed here: the one-way entrances
/// (class 43) and exits (44) of channels 0 to 3, the two-way monoliths (45) of channels 0 to 5
/// (those of 0 to 3 blocked and visited on their anchor, of 4 and 5 blocked on their anchor and
/// the tile left of it, visited on that one), the Subterranean Gate (blocked on its anchor, the two
/// tiles left of it and the tile above the middle one, visited from the middle one), the Whirlpool
/// (its anchor's row and the row above, three columns ending at the anchor, all six visited) and
/// the gnolls.
mod lines {
    pub(super) const ENTRANCES: [&str; 4] = [
        "AVXmn1b0.def 011111111111111111111111111111111111111111111111 \
         100000000000000000000000000000000000000000000000 011111111 011111111 43 0 0 0",
        "AVXmn1r0.def 011111111111111111111111111111111111111111111111 \
         100000000000000000000000000000000000000000000000 011111111 011111111 43 1 0 0",
        "AVXmn1y0.def 011111111111111111111111111111111111111111111111 \
         100000000000000000000000000000000000000000000000 011111111 011111111 43 2 0 0",
        "AVXmn4i0.def 011111111111111111111111111111111111111111111111 \
         100000000000000000000000000000000000000000000000 011111111 011111111 43 3 0 0",
    ];
    pub(super) const EXITS: [&str; 4] = [
        "AVXmx1b0.def 011111111111111111111111111111111111111111111111 \
         100000000000000000000000000000000000000000000000 011111111 011111111 44 0 0 0",
        "AVXmx1r0.def 011111111111111111111111111111111111111111111111 \
         100000000000000000000000000000000000000000000000 011111111 011111111 44 1 0 0",
        "AVXmx1y0.def 011111111111111111111111111111111111111111111111 \
         100000000000000000000000000000000000000000000000 011111111 011111111 44 2 0 0",
        "AVXmn4o0.def 011111111111111111111111111111111111111111111111 \
         100000000000000000000000000000000000000000000000 011111111 011111111 44 3 0 0",
    ];
    pub(super) const TWO_WAY: [&str; 6] = [
        "AVXmn2g0.def 011111111111111111111111111111111111111111111111 \
         100000000000000000000000000000000000000000000000 011111111 011111111 45 0 0 0",
        "AVXmn2o0.def 011111111111111111111111111111111111111111111111 \
         100000000000000000000000000000000000000000000000 011111111 011111111 45 1 0 0",
        "AVXmn2p0.def 011111111111111111111111111111111111111111111111 \
         100000000000000000000000000000000000000000000000 011111111 011111111 45 2 0 0",
        "AVXmn4b0.def 011111111111111111111111111111111111111111111111 \
         100000000000000000000000000000000000000000000000 011111111 011111111 45 3 0 0",
        "AVXmn5b0.def 001111111111111111111111111111111111111111111111 \
         010000000000000000000000000000000000000000000000 011111111 011111111 45 4 0 0",
        "AVXmn6b0.def 001111111111111111111111111111111111111111111111 \
         010000000000000000000000000000000000000000000000 011111111 011111111 45 5 0 0",
    ];
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

/// The shipped maps' most on one map (the census, survey §1 as E08 T106c amended it): one-way
/// entrances and exits (each over [`ONE_WAY_CHANNELS`]), two-way monoliths (over
/// [`TWO_WAY_CHANNELS`]), Subterranean Gates and Whirlpools.
const ONE_WAY: usize = 8;
const ONE_WAY_CHANNELS: usize = 4;
const TWO_WAY: usize = 17;
const TWO_WAY_CHANNELS: usize = 6;
const GATES: usize = 26;
const WHIRLPOOLS: usize = 19;

/// A tile: column, row, level.
type Spot = (u8, u8, u8);

/// The heroes' tiles and owners, in the map's order (the surface's first): red's three in the
/// north, the last by the sea, blue's three in the south, the last by the sea; underground red's
/// two, then blue's.
const HEROES: [(Spot, u8); 10] = [
    ((13, 13, 0), 0),
    ((121, 25, 0), 0),
    ((61, 59, 0), 0),
    ((13, 121, 0), 1),
    ((130, 118, 0), 1),
    ((85, 84, 0), 1),
    ((25, 37, 1), 0),
    ((109, 13, 1), 0),
    ((37, 109, 1), 1),
    ((121, 133, 1), 1),
];
/// The heroes' types, in the map's order (SoD's, a class apart or more: none of the campaigns').
const TYPES: [u8; 10] = [0, 14, 28, 42, 56, 70, 84, 98, 112, 126];
/// The boats on the water beside the heroes by the sea (red's, then blue's).
const BOATS: [(u8, u8); 2] = [(61, 60), (85, 83)];

/// A teleporter's class: a one-way entrance or exit or a two-way monolith of a channel, a gate, a
/// Whirlpool.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Class {
    Entrance(usize),
    Exit(usize),
    TwoWay(usize),
    Gate,
    Whirlpool,
}

impl Class {
    fn line(self) -> &'static str {
        match self {
            Self::Entrance(channel) => lines::ENTRANCES[channel],
            Self::Exit(channel) => lines::EXITS[channel],
            Self::TwoWay(channel) => lines::TWO_WAY[channel],
            Self::Gate => lines::GATE,
            Self::Whirlpool => lines::WHIRLPOOL,
        }
    }

    /// The tiles it blocks or is visited on, and those it is visited on, as (columns, rows) left
    /// of and above its anchor.
    fn tiles(self) -> (&'static [(u8, u8)], &'static [(u8, u8)]) {
        match self {
            Self::Entrance(_) | Self::Exit(_) | Self::TwoWay(0..=3) => (&[(0, 0)], &[(0, 0)]),
            Self::TwoWay(_) => (&[(0, 0), (1, 0)], &[(1, 0)]),
            Self::Gate => (&[(0, 0), (1, 0), (2, 0), (1, 1)], &[(1, 0)]),
            Self::Whirlpool => (WHIRLPOOL_TILES, WHIRLPOOL_TILES),
        }
    }
}

const WHIRLPOOL_TILES: &[(u8, u8)] = &[(0, 0), (1, 0), (2, 0), (0, 1), (1, 1), (2, 1)];

/// The tiles `class`'s object anchored on `(x, y, z)` covers (`visited`: those it is visited on).
fn covered(class: Class, (x, y, z): Spot, visited: bool) -> impl Iterator<Item = Spot> {
    let (all, visits) = class.tiles();
    let offsets = if visited { visits } else { all };
    offsets.iter().map(move |&(dx, dy)| (x - dx, y - dy, z))
}

/// The teleporters in the map's order, each with its anchor: the one-way entrances, the exits and
/// the two-way monoliths, their channels in turn, then the gates, a level in turn, each in a land
/// cell of its own (the next one free along [`LAND_STRIDE`], on the gate's level), anchored 6
/// tiles right of and below the cell's corner; then the Whirlpools, each in a sea cell of its own
/// along [`SEA_STRIDE`], anchored as well.
fn layout() -> Vec<(Class, Spot)> {
    let cells = SIDE / CELL;
    let land: Vec<(u8, u8, u8)> = (0..2)
        .flat_map(|z| (0..cells).flat_map(move |j| (0..cells).map(move |i| (i, j, z))))
        .filter(|&(_, j, z)| z == 1 || !SEA_ROWS.contains(&j))
        .collect();
    let mut taken = vec![false; land.len()];
    let mut cursor = 0;
    let mut next = |level: Option<u8>| loop {
        let cell = cursor * LAND_STRIDE % land.len();
        cursor += 1;
        let (i, j, z) = land[cell];
        if !taken[cell] && level.is_none_or(|level| level == z) {
            taken[cell] = true;
            break (CELL * i + 6, CELL * j + 6, z);
        }
    };
    let classes = (0..ONE_WAY)
        .map(|n| (Class::Entrance(n % ONE_WAY_CHANNELS), None))
        .chain((0..ONE_WAY).map(|n| (Class::Exit(n % ONE_WAY_CHANNELS), None)))
        .chain((0..TWO_WAY).map(|n| (Class::TwoWay(n % TWO_WAY_CHANNELS), None)))
        .chain((0..GATES).map(|n| (Class::Gate, Some(u8::try_from(n % 2).unwrap()))));
    let mut placed: Vec<(Class, Spot)> =
        classes.map(|(class, level)| (class, next(level))).collect();
    let sea = SEA_ROWS.len() * usize::from(cells);
    placed.extend((0..WHIRLPOOLS).map(|n| {
        let cell = u8::try_from(n * SEA_STRIDE % sea).unwrap();
        let (i, j) = (cell % cells, SEA_ROWS[0] + cell / cells);
        (Class::Whirlpool, (CELL * i + 6, CELL * j + 6, 0))
    }));
    placed
}

/// Whether `(x, y, z)` is land: underground or off the surface's sea.
fn land((_, y, z): Spot) -> bool {
    z == 1 || !SEA_ROWS.contains(&(y / CELL))
}

/// The monsters' tiles: every [`EVERY`]th free tile of land, counted row by row over the surface,
/// then the underground (the first one taken); a tile is free when no teleporter covers it and no
/// hero stands on it or beside it.
fn monsters(teleporters: &[(Class, Spot)]) -> Vec<Spot> {
    let mut taken = vec![false; 2 * usize::from(SIDE) * usize::from(SIDE)];
    let index = |(x, y, z): Spot| {
        (usize::from(z) * usize::from(SIDE) + usize::from(y)) * usize::from(SIDE) + usize::from(x)
    };
    for &(class, anchor) in teleporters {
        for tile in covered(class, anchor, false) {
            taken[index(tile)] = true;
        }
    }
    for &((x, y, z), _) in &HEROES {
        for dy in 0..3 {
            for dx in 0..3 {
                if let (Some(tx), Some(ty)) = ((x + dx).checked_sub(1), (y + dy).checked_sub(1)) {
                    if tx < SIDE && ty < SIDE {
                        taken[index((tx, ty, z))] = true;
                    }
                }
            }
        }
    }
    (0..2)
        .flat_map(|z| (0..SIDE).flat_map(move |y| (0..SIDE).map(move |x| (x, y, z))))
        .filter(|&tile| land(tile) && !taken[index(tile)])
        .step_by(EVERY)
        .collect()
}

/// One level's objects: its heroes, its teleporters in the map's order, then (the surface) the
/// boats, then its monsters (a count of 1 to 12 gnolls in turn, savage, never fleeing, not
/// growing).
fn level(z: u8, teleporters: &[(Class, Spot)], monsters: &[Spot]) -> Vec<w::MapObject> {
    let mut objects: Vec<w::MapObject> = (HEROES.iter().zip(TYPES))
        .filter(|(((_, _, level), _), _)| *level == z)
        .map(|(&((x, y, _), owner), hero_type)| w::MapObject::Hero(hero((x, y), owner, hero_type)))
        .collect();
    objects.extend(
        (teleporters.iter())
            .filter(|(_, (_, _, level))| *level == z)
            .map(|&(class, (x, y, _))| plain(class.line(), (x, y))),
    );
    if z == 0 {
        objects.extend(BOATS.map(|at| boat(0, at)));
    }
    let gnolls = template(lines::GNOLLS);
    objects.extend(
        (monsters.iter().enumerate())
            .filter(|(_, (_, _, level))| *level == z)
            .map(|(n, &(x, y, _))| {
                w::MapObject::Monster(w::Monster {
                    tile: (x, y),
                    template: gnolls.clone(),
                    count: 1 + u16::try_from(n % 12).unwrap(),
                    disposition: 4,
                    never_flees: true,
                    fixed_count: true,
                    treasure: None,
                })
            }),
    );
    objects
}

/// The map: SoD, 144 × 144 of grass with the sea's rows, over an underground of subterranean
/// ground; red and blue with five heroes each.
fn map() -> w::Map {
    let mut players: [Option<w::Player>; w::PLAYERS] = Default::default();
    for player in &mut players[..2] {
        *player = Some(w::Player {
            human: true,
            computer: true,
            towns: 1,
        });
    }
    let teleporters = layout();
    let monsters = monsters(&teleporters);
    let rows = SEA_ROWS.map(|j| CELL * j);
    w::Map {
        name: "Teleporters XL".into(),
        description: "E08 T106c: the shipped maps' most teleporters of each class".into(),
        size: SIDE,
        terrain: w::Terrain::Grass,
        patches: vec![w::Patch {
            terrain: w::Terrain::Water,
            from: (0, rows[0]),
            to: (SIDE - 1, rows[1] + CELL - 1),
        }],
        roads: Vec::new(),
        players,
        objects: level(0, &teleporters, &monsters),
        underground: Some(w::Underground {
            terrain: w::Terrain::Subterranean,
            patches: Vec::new(),
            objects: level(1, &teleporters, &monsters),
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

fn at((x, y, z): Spot) -> Pos {
    Pos::new(x.into(), y.into(), z)
}

/// The visited tiles of the layout's teleporters `of` gives, in the walk's order (level, row,
/// column).
fn visited(layout: &[(Class, Spot)], of: impl Fn(Class) -> bool) -> Vec<Pos> {
    let mut tiles: Vec<Pos> = (layout.iter())
        .filter(|(class, _)| of(*class))
        .flat_map(|&(class, anchor)| covered(class, anchor, true))
        .map(at)
        .collect();
    tiles.sort_by_key(|pos| (pos.level, pos.y, pos.x));
    tiles
}

// covers: ADV-TEL-001, ADV-TEL-002
#[test]
fn the_teleporters_xl_map_is_committed_as_written_and_builds_its_channels() {
    let rules = &*FAITHFUL;
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
        BOATS.map(|(x, y)| BoatSetup {
            tile: at((x, y, 0)),
            kind: 0
        })
    );
    // Every object typed: the teleporters, then the monsters.
    let layout = layout();
    let monsters = monsters(&layout);
    assert_eq!(setup.map.objects.len(), layout.len() + monsters.len());
    assert!(setup.map.objects.iter().all(|o| o.object_type.is_some()));
    assert!(
        (350..450).contains(&monsters.len()),
        "{} monsters",
        monsters.len()
    );
    let game = GameState::new(rules, &setup, 1).unwrap();
    let channels = game.adventure().map().channels();
    let one_way: Vec<(u8, Vec<Pos>)> = (channels.one_way_channels())
        .map(|(channel, exits)| (channel, exits.to_vec()))
        .collect();
    let expected: Vec<(u8, Vec<Pos>)> = (0..ONE_WAY_CHANNELS)
        .map(|c| {
            (
                u8::try_from(c).unwrap(),
                visited(&layout, |k| k == Class::Exit(c)),
            )
        })
        .collect();
    assert_eq!(one_way, expected);
    let two_way: Vec<(u8, Vec<Pos>)> = (channels.two_way_channels())
        .map(|(channel, members)| (channel, members.to_vec()))
        .collect();
    let expected: Vec<(u8, Vec<Pos>)> = (0..TWO_WAY_CHANNELS)
        .map(|c| {
            (
                u8::try_from(c).unwrap(),
                visited(&layout, |k| k == Class::TwoWay(c)),
            )
        })
        .collect();
    assert_eq!(two_way, expected);
    // Each one-way channel's exits on both levels (the stride's spread).
    for (_, exits) in &one_way {
        assert!(exits.iter().any(|p| p.level == 0) && exits.iter().any(|p| p.level == 1));
    }
    // Every gate paired, thirteen pairs; the Whirlpools' 114 tiles.
    let gates: Vec<Pos> = channels.gates().iter().map(|gate| gate.tile).collect();
    assert_eq!(gates, visited(&layout, |k| k == Class::Gate));
    assert!(channels.gates().iter().all(|gate| gate.pair.is_some()));
    let whirlpools: Vec<Pos> = channels.whirlpools().iter().map(|w| w.tile).collect();
    assert_eq!(whirlpools, visited(&layout, |k| k == Class::Whirlpool));
    assert_eq!(whirlpools.len(), 6 * WHIRLPOOLS);
}
