//! The garrisons and bonuses XL map committed under `fixtures/maps/` (E08 T116c, increment 12's
//! design note §16): SoD, 144 × 144 of grass with a sea on columns 132 to 143, cut by rock into
//! five rows of four regions (walls on rows 28, 57, 86 and 115, up to the sea, and on columns 35,
//! 71 and 107), every way between two regions a map garrison: the shipped maps' most of each class
//! on one map (`examples/original_data.rs`' census, survey `E08-garrisons-bonuses.md` §1.1), sixteen
//! horizontal garrisons (33), one in each wall row between two regions of a column, and eleven
//! vertical ones (219) in the wall columns, four ways of those left as rock; their owners in turn
//! red, blue and none, every fourth of a class the anti-magic one, every fifth empty, every seventh
//! not removable. The fifteen bonus visitables at the shipped maps' most of each on one map (§1.2):
//! the twelve on land, each in a 6 × 6 cell of its own drawn by a fixed stride through the land's
//! cells (no cell holding a wall or a hero), the sea's three in the sea's cells by a stride of their
//! own; eight Magic Springs and eleven Fountains of Fortune among them. Red's five heroes in the
//! north, blue's five in the south, one of each on the coast beside a boat; a monster on every
//! hundredth free land tile, counted row by row. Written by [`crate::writer`] as the teleporters XL
//! map (`teleporters_xl_map.rs`); played by the benchmarks `reach-garrisons-xl-144` (the walkers'
//! reachable-tiles queries, which cross their own side's garrisons and stop at the others') and
//! `fork-bonuses-xl-144` (`benches/adventure/`: the forks after a visit has emptied a Spring, with
//! the Fountains' values). The test checks that the committed file reads to the map written here
//! (it is rewritten when `H3_WRITE_FIXTURES` is set), imports each garrison with its owner, army and
//! removable flag and each of the fifteen at its count, and builds with the eleven Fountains.

use std::path::PathBuf;

use super::garrisons_bonuses::{GARRISONS, LAND_BONUSES, WATER_BONUSES, garrison, troops};
use super::import::SOD;
use super::sea_map::{boat, hero, lines as sea, plain, template};
use crate::model::RawMap;
use crate::writer as w;
use crate::{Limits, import, read};
use h3_adventure::{BoatSetup, Holds, Pos};
use h3_contracts::PlayerId;
use h3_game::GameState;

/// The committed file's name.
const NAME: &str = "e08-garrisons-bonuses-144.h3m";
/// The map's side.
const SIDE: u8 = 144;
/// The sea's first column; the sea runs to the map's east edge on every row.
const SEA: u8 = 132;
/// The rows of rock between the regions' rows (each up to the sea) and the columns of rock between
/// the regions' columns (each over the whole map).
const WALL_ROWS: [u8; 4] = [28, 57, 86, 115];
const WALL_COLUMNS: [u8; 3] = [35, 71, 107];
/// A cell's side: the bonus visitables stand one to a cell.
const CELL: u8 = 6;
/// The stride through the land's free cells (370, coprime with it), and through the sea's 48
/// cells (coprime with them).
const LAND_STRIDE: usize = 101;
const SEA_STRIDE: usize = 5;
/// A monster on every this many free land tiles, counted row by row.
const EVERY: usize = 100;

/// The shipped maps' most of each land bonus visitable on one map (the census, survey §1.2), by
/// its `LAND_BONUSES` line: the Swan Pond, the Faerie Ring, the Fountain of Fortune, the Fountain
/// of Youth, the Idol of Fortune, the Magic Spring, the Magic Well, the Oasis, the Rally Flag, the
/// Stables, the Temple and the Watering Hole (158 in all).
const LAND_COUNTS: [usize; 12] = [8, 12, 11, 13, 15, 8, 32, 4, 19, 10, 22, 4];
/// The same of the sea's three, by its `WATER_BONUSES` line: the Buoy, the Mermaids and the Sirens.
const WATER_COUNTS: [usize; 3] = [17, 9, 9];
/// The fifteen's types by key, the land's then the sea's, in their lines' order.
const BONUS_KEYS: [&str; 15] = [
    "base:swan_pond",
    "base:faerie_ring",
    "base:fountain_of_fortune",
    "base:fountain_of_youth",
    "base:idol_of_fortune",
    "base:magic_spring",
    "base:magic_well",
    "base:oasis",
    "base:rally_flag",
    "base:stables",
    "base:temple",
    "base:watering_hole",
    "base:buoy",
    "base:mermaids",
    "base:sirens",
];

/// The shipped maps' most garrisons of each class on one map (the census, survey §1.1).
const HORIZONTAL: usize = 16;
const VERTICAL: usize = 11;

/// The heroes' tiles and owners, in the map's order: red's in the north (the first beside a
/// Magic Spring, the second and third by their garrisons, the fifth on the coast), then blue's in
/// the south as well.
const HEROES: [((u8, u8), u8); 10] = [
    ((15, 15), 0),
    ((51, 21), 0),
    ((87, 21), 0),
    ((57, 45), 0),
    ((131, 10), 0),
    ((15, 129), 1),
    ((51, 123), 1),
    ((87, 123), 1),
    ((57, 99), 1),
    ((131, 133), 1),
];
/// The heroes' types, in the map's order (SoD's, a class apart or more: none of the campaigns').
const TYPES: [u8; 10] = [0, 14, 28, 42, 56, 70, 84, 98, 112, 126];
/// The boats on the water beside the heroes on the coast (red's, then blue's).
const BOATS: [(u8, u8); 2] = [(SEA, 10), (SEA, 133)];

/// Creature codes of the garrisons' armies, in turn: Pikemen, Archers, Griffins, Swordsmen, Monks.
const CREATURES: [u16; 5] = [0, 2, 4, 6, 8];

/// A garrison as placed: its entrance (the middle of its three tiles on the wall), whether it is
/// vertical, its `GARRISONS` line, its owner, its army (one stack, or none) and its removable flag.
#[derive(Clone, Copy, Debug)]
struct Placed {
    entrance: (u8, u8),
    vertical: bool,
    line: usize,
    owner: Option<u8>,
    army: Option<(u16, u16)>,
    removable: bool,
}

impl Placed {
    /// Its anchor: one right of a horizontal one's entrance, one below a vertical one's.
    fn anchor(&self) -> (u8, u8) {
        let (x, y) = self.entrance;
        if self.vertical {
            (x, y + 1)
        } else {
            (x + 1, y)
        }
    }

    /// The three tiles it covers on the wall.
    fn tiles(&self) -> [(u8, u8); 3] {
        let (x, y) = self.entrance;
        if self.vertical {
            [(x, y - 1), (x, y), (x, y + 1)]
        } else {
            [(x - 1, y), (x, y), (x + 1, y)]
        }
    }

    fn stacks(&self) -> Vec<(u16, u16)> {
        self.army.into_iter().collect()
    }
}

/// The regions' columns and rows as (first, last) tiles: between the walls, the last column's up
/// to the sea.
fn spans(walls: &[u8], last: u8) -> Vec<(u8, u8)> {
    let mut spans = Vec::new();
    let mut first = 0;
    for &wall in walls {
        spans.push((first, wall - 1));
        first = wall + 1;
    }
    spans.push((first, last));
    spans
}

/// The garrisons in the map's order: the horizontal ones, a wall row in turn, one in the middle of
/// each region's column; then the vertical ones, a region's row in turn, one in the middle of
/// each wall column but every fourth such way from the third (left as rock). The `n`th garrison
/// is red's, blue's or neutral by `n` modulo 3, empty when `n` is 4 modulo 5, not removable when
/// it is 6 modulo 7; the `m`th of a class is the anti-magic one when `m` is 3 modulo 4.
fn garrisons() -> Vec<Placed> {
    let columns = spans(&WALL_COLUMNS, SEA - 1);
    let rows = spans(&WALL_ROWS, SIDE - 1);
    let middle = |(first, last): (u8, u8)| first + (last - first) / 2;
    let horizontal = (WALL_ROWS.iter())
        .flat_map(|&y| columns.iter().map(move |&span| ((middle(span), y), false)));
    let vertical = (rows
        .iter()
        .flat_map(|&span| WALL_COLUMNS.iter().map(move |&x| ((x, middle(span)), true))))
    .enumerate()
    .filter(|(way, _)| way % 4 != 2)
    .map(|(_, placed)| placed);
    let mut of_class = [0; 2];
    (horizontal.chain(vertical).enumerate())
        .map(|(n, (entrance, vertical))| {
            let m = of_class[usize::from(vertical)];
            of_class[usize::from(vertical)] += 1;
            let count = u16::try_from(1 + n % 6).unwrap();
            Placed {
                entrance,
                vertical,
                line: 2 * usize::from(vertical) + usize::from(m % 4 == 3),
                owner: [Some(0), Some(1), None][n % 3],
                army: (n % 5 != 4).then_some((CREATURES[n % 5], count)),
                removable: n % 7 != 6,
            }
        })
        .collect()
}

/// A tile's cell: its column and row of cells.
fn cell_of((x, y): (u8, u8)) -> (u8, u8) {
    (x / CELL, y / CELL)
}

/// A bonus visitable placed in a cell: anchored 4 right of and 3 below the cell's corner, so
/// that its tiles (up to three left of and one above the anchor) leave the cell's edges free.
fn anchor_in((i, j): (u8, u8)) -> (u8, u8) {
    (CELL * i + 4, CELL * j + 3)
}

/// The land's free cells, row by row: none holding a wall's tile, the sea's or a hero's.
fn land_cells() -> Vec<(u8, u8)> {
    let cells = SIDE / CELL;
    let walls = |(i, j): (u8, u8)| {
        WALL_ROWS.iter().any(|&y| cell_of((0, y)).1 == j)
            || WALL_COLUMNS.iter().any(|&x| cell_of((x, 0)).0 == i)
    };
    (0..cells)
        .flat_map(|j| (0..cells).map(move |i| (i, j)))
        .filter(|&cell| CELL * cell.0 + CELL <= SEA && !walls(cell))
        .filter(|&cell| HEROES.iter().all(|&(tile, _)| cell_of(tile) != cell))
        .collect()
}

/// The sea's cells, row by row.
fn sea_cells() -> Vec<(u8, u8)> {
    let cells = SIDE / CELL;
    (0..cells)
        .flat_map(|j| (SEA / CELL..cells).map(move |i| (i, j)))
        .collect()
}

/// The bonus visitables in the map's order, each its line (the land's `LAND_BONUSES`, then the
/// sea's `WATER_BONUSES`) and its anchor: the land's by their lines in turn, each at its count,
/// the `k`th in the free land cell `k` × [`LAND_STRIDE`] modulo their number; then the sea's
/// the same way along [`SEA_STRIDE`].
fn bonuses() -> Vec<(&'static str, (u8, u8))> {
    let mut bonuses = placed(&LAND_BONUSES, &LAND_COUNTS, &land_cells(), LAND_STRIDE);
    bonuses.extend(placed(
        &WATER_BONUSES,
        &WATER_COUNTS,
        &sea_cells(),
        SEA_STRIDE,
    ));
    bonuses
}

/// Each of `lines` at its count, in turn, the `k`th in the cell `k` × `stride` modulo their
/// number of `cells` (all distinct: the stride is coprime with it, and there are cells enough).
fn placed(
    lines: &[&'static str],
    counts: &[usize],
    cells: &[(u8, u8)],
    stride: usize,
) -> Vec<(&'static str, (u8, u8))> {
    (lines.iter().zip(counts))
        .flat_map(|(&line, &count)| (0..count).map(move |_| line))
        .enumerate()
        .map(|(k, line)| (line, anchor_in(cells[k * stride % cells.len()])))
        .collect()
}

/// Whether `(x, y)` is land: off the walls (but for a garrison's gap) and the sea.
fn land((x, y): (u8, u8), garrisons: &[Placed]) -> bool {
    let gap = garrisons.iter().any(|g| g.tiles().contains(&(x, y)));
    x < SEA && (gap || !(WALL_ROWS.contains(&y) || WALL_COLUMNS.contains(&x)))
}

/// The tiles a bonus visitable of `line` anchored on `anchor` covers: its template's blocked and
/// visited tiles (bit `7 − c` of byte `5 − r` the tile `c` columns left of and `r` rows above
/// the anchor).
fn bonus_tiles(line: &str, anchor: (u8, u8)) -> Vec<(u8, u8)> {
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
/// taken); a tile is free when no garrison or bonus visitable covers it and no hero stands on it
/// or beside it.
fn monsters(garrisons: &[Placed], bonuses: &[(&str, (u8, u8))]) -> Vec<(u8, u8)> {
    let index = |(x, y): (u8, u8)| usize::from(y) * usize::from(SIDE) + usize::from(x);
    let mut taken = vec![false; usize::from(SIDE) * usize::from(SIDE)];
    for g in garrisons {
        for tile in g.tiles() {
            taken[index(tile)] = true;
        }
    }
    for &(line, anchor) in bonuses {
        for tile in bonus_tiles(line, anchor) {
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
        .flat_map(|y| (0..SIDE).map(move |x| (x, y)))
        .filter(|&tile| land(tile, garrisons) && !taken[index(tile)])
        .step_by(EVERY)
        .collect()
}

/// The rock: each wall row up to the sea and each wall column over the map, then the grass of
/// each garrison's three tiles, then the sea.
fn patches(garrisons: &[Placed]) -> Vec<w::Patch> {
    let patch = |terrain, from, to| w::Patch { terrain, from, to };
    let mut patches: Vec<w::Patch> = (WALL_ROWS.iter())
        .map(|&y| patch(w::Terrain::Rock, (0, y), (SEA - 1, y)))
        .chain((WALL_COLUMNS.iter()).map(|&x| patch(w::Terrain::Rock, (x, 0), (x, SIDE - 1))))
        .collect();
    patches.extend(garrisons.iter().map(|g| {
        let tiles = g.tiles();
        patch(w::Terrain::Grass, tiles[0], tiles[2])
    }));
    patches.push(patch(w::Terrain::Water, (SEA, 0), (SIDE - 1, SIDE - 1)));
    patches
}

/// The map's objects: the heroes, the garrisons, the bonus visitables, the boats, then the
/// monsters (a count of 1 to 12 gnolls in turn, savage, never fleeing, not growing).
fn objects(garrisons: &[Placed], bonuses: &[(&'static str, (u8, u8))]) -> Vec<w::MapObject> {
    let mut objects: Vec<w::MapObject> = (HEROES.iter().zip(TYPES))
        .map(|(&(tile, owner), hero_type)| w::MapObject::Hero(hero(tile, owner, hero_type)))
        .collect();
    objects.extend(garrisons.iter().map(|g| {
        garrison(
            g.anchor(),
            GARRISONS[g.line],
            g.owner,
            &g.stacks(),
            g.removable,
        )
    }));
    objects.extend(bonuses.iter().map(|&(line, at)| plain(line, at)));
    objects.extend(BOATS.map(|at| boat(0, at)));
    let gnolls = template(sea::GNOLLS);
    objects.extend(
        monsters(garrisons, bonuses)
            .into_iter()
            .enumerate()
            .map(|(n, tile)| {
                w::MapObject::Monster(w::Monster {
                    tile,
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

/// The map: SoD, 144 × 144 of grass with the walls and the sea; red and blue with five heroes
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
    let garrisons = garrisons();
    let bonuses = bonuses();
    w::Map {
        name: "Garrisons and bonuses XL".into(),
        description: "E08 T116c: the shipped maps' most garrisons and bonus visitables".into(),
        size: SIDE,
        terrain: w::Terrain::Grass,
        patches: patches(&garrisons),
        roads: Vec::new(),
        players,
        objects: objects(&garrisons, &bonuses),
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

/// The garrisons' types by `GARRISONS` line.
const GARRISON_KEYS: [&str; 4] = [
    "base:garrison",
    "base:anti_magic_garrison",
    "ab:garrison_vertical",
    "ab:anti_magic_garrison_vertical",
];

// covers: ADV-GAR-001, ADV-BON-001
#[test]
fn the_garrisons_bonuses_xl_map_is_committed_as_written_and_imports_its_seventeen_classes() {
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
    // Every object typed: the garrisons, the fifteen, then the monsters.
    let garrisons = garrisons();
    let bonuses = bonuses();
    let monsters = monsters(&garrisons, &bonuses);
    let objects = &setup.map.objects;
    assert_eq!(
        objects.len(),
        garrisons.len() + bonuses.len() + monsters.len()
    );
    assert!(objects.iter().all(|o| o.object_type.is_some()));
    assert!(
        (150..200).contains(&monsters.len()),
        "{} monsters",
        monsters.len()
    );
    // The garrisons: the census's most of each class, each with its owner, army and flag.
    let vertical = garrisons.iter().filter(|g| g.vertical).count();
    assert_eq!(
        (garrisons.len() - vertical, vertical),
        (HORIZONTAL, VERTICAL)
    );
    let placed: Vec<_> = (objects.iter().take(garrisons.len()))
        .map(|o| (key(o), o.anchor, o.owner, o.holds.clone()))
        .collect();
    let expected: Vec<_> = (garrisons.iter())
        .map(|g| {
            let stacks: Vec<(u16, u32)> = g.stacks().iter().map(|&(c, n)| (c, n.into())).collect();
            let holds = Holds::Garrison {
                army: troops(&stacks),
                removable: g.removable,
            };
            (
                GARRISON_KEYS[g.line],
                at(g.anchor()),
                g.owner.map(PlayerId::new),
                holds,
            )
        })
        .collect();
    assert_eq!(placed, expected);
    // Each wall's garrisons stand in its gaps: every way between two regions but the four of rock.
    assert!(garrisons.iter().all(|g| {
        let (x, y) = g.entrance;
        if g.vertical {
            WALL_COLUMNS.contains(&x)
        } else {
            WALL_ROWS.contains(&y)
        }
    }));
    // The fifteen at their counts, each at its anchor.
    let counts = LAND_COUNTS.iter().chain(&WATER_COUNTS);
    let bonus_objects = &objects[garrisons.len()..garrisons.len() + bonuses.len()];
    for (&wanted, count) in BONUS_KEYS.iter().zip(counts) {
        let found = bonus_objects.iter().filter(|o| key(o) == wanted).count();
        assert_eq!(found, *count, "{wanted}");
    }
    let anchors: Vec<Pos> = bonus_objects.iter().map(|o| o.anchor).collect();
    let expected: Vec<Pos> = bonuses.iter().map(|&(_, anchor)| at(anchor)).collect();
    assert_eq!(anchors, expected);
    // Red's first hero has a Magic Spring in its region within a few steps.
    let springs = (bonuses.iter()).filter(|&&(line, _)| line == LAND_BONUSES[5]);
    let (hx, hy) = HEROES[0].0;
    assert!(
        springs
            .into_iter()
            .any(|&(_, (x, y))| x.abs_diff(hx).max(y.abs_diff(hy)) <= 8)
    );
    let game = GameState::new(rules, &setup, 1).unwrap();
    assert_eq!(game.adventure().map().fountains().len(), LAND_COUNTS[2]);
}
