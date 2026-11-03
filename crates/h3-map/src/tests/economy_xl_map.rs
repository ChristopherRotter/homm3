//! The economy XL map committed under `fixtures/maps/` (E08 T86b2, increment 9's design note §15,
//! §19): SoD, 144 × 144 with an underground (41 472 tiles); as many mines, abandoned mines,
//! Lighthouses, Shipyards and weekly objects as the largest shipped map holds of each class
//! (survey `E08-economy.md` §1: a class's most on one map; the weekly objects' 18, 24 and 11) and
//! a wandering monster on every hundredth free tile. Written by [`crate::writer`] with the economy
//! map's templates (`economy_map.rs`) and the underground's; played by the day start's benchmarks
//! (`benches/adventure/`: `day-start-xl-144`, the `EndTurn` that starts day 8;
//! `month-start-xl-144`, the one that starts day 29 in a creature month; `fork-economy-xl-144`,
//! forks after a week's writes). The tests check that the committed file reads to the map written
//! here and imports with every class and its owners (it is rewritten when `H3_WRITE_FIXTURES` is
//! set; a change invalidates what names the map by content hash, ADV-Q2), and that its day starts
//! run: four weeks' growth, then a creature month's monsters.

use std::path::PathBuf;

use super::economy_map::{lines, owned};
use super::play::FAITHFUL;
use super::sea_map::{hero, plain, template};
use crate::model::RawMap;
use crate::writer as w;
use crate::{Limits, import, read};
use h3_adventure::{AdventureEvent, AdventureEventKind};
use h3_contracts::PlayerId;
use h3_game::{Actor, GameCommand, GameEvent, GameState, MonthKind, apply};

/// The committed file's name.
const NAME: &str = "e08-economy-144.h3m";
/// The map's side.
const SIDE: u8 = 144;
/// The objects' lattice: one object in a cell of `CELL` × `CELL` tiles, its position on the
/// cell's tile (5, 5), so its picture (at most four columns by three rows) and a free ring around
/// it stay inside the cell.
const CELL: u8 = 8;
/// The surface's last land column: the sea fills the columns east of it, the Lighthouses and the
/// Shipyards stand on it. The surface's lattice ends four columns west of the coast's objects.
const COAST: u8 = 127;
const SURFACE_COLUMNS: u8 = 15;
/// The heroes' tiles on each level (a cell's tile (1, 1), off every object's ring): red's two in
/// the west, blue's two in the east.
const HEROES: [(u8, u8); 4] = [(9, 9), (9, 73), (105, 73), (105, 137)];
/// The heroes' types by level (the map's order: the surface's four, then the underground's; SoD's
/// 156, one of each class's first eight in turn).
const TYPES: [[u8; 4]; 2] = [[0, 17, 68, 85], [34, 51, 102, 119]];
/// A wandering monster stands on every `MONSTER_EVERY`-th free land tile (one no object's ring and
/// no hero's two tiles cover), in level, row and column order.
const MONSTER_EVERY: usize = 100;

/// The shipped maps' most of each mine on one map (survey §1), by subclass (sawmill … gold mine).
const MINES: [usize; 7] = [26, 14, 20, 13, 13, 12, 18];
/// The underground's mine templates by subclass (the surface's are `lines::MINES`): the same masks,
/// the subterranean pictures.
const UNDERGROUND_MINES: [&str; 7] = [
    "AVMsawl0.def 000011110011111111111111111111111111111111111111 \
     010000000000000000000000000000000000000000000000 111111111 011000000 53 0 0 0",
    "AVMalch0.def 000111111111111111111111111111111111111111111111 \
     010000000000000000000000000000000000000000000000 111111111 001110000 53 1 0 0",
    "AVMorsb0.def 000111110001111111111111111111111111111111111111 \
     010000000000000000000000000000000000000000000000 111111111 001000000 53 2 0 1",
    "AVMsulf0.def 000111111111111111111111111111111111111111111111 \
     010000000000000000000000000000000000000000000000 111111111 011000000 53 3 0 0",
    "AVMcrsu0.def 000111111111111111111111111111111111111111111111 \
     010000000000000000000000000000000000000000000000 111111111 001000000 53 4 0 0",
    "AVMgerf0.def 000111110001111111111111111111111111111111111111 \
     010000000000000000000000000000000000000000000000 111111111 001100010 53 5 0 1",
    "AVMgosb0.def 000111111111111111111111111111111111111111111111 \
     010000000000000000000000000000000000000000000000 111111111 001000000 53 6 0 0",
];
/// The abandoned mines: four of class 53 (the RoE picture on grass) and ten of SoD's class 220
/// (the most of each on a map, survey §1), half of those underground.
const ABANDONED_53: usize = 4;
const ABANDONED_220: usize = 10;
const ABANDONED_GRASS: &str = "AvMAbMG.def 000111111111111111111111111111111111111111111111 \
    010000000000000000000000000000000000000000000000 111111111 000000100 53 7 0 0";
const ABANDONED_220_GRASS: &str = "AVXamGr.def \
    000111111111111111111111111111111111111111111111 \
    010000000000000000000000000000000000000000000000 111111111 000000100 220 7 0 0";
const ABANDONED_220_UNDERGROUND: &str = "AVXamSu.def \
    000111111111111111111111111111111111111111111111 \
    010000000000000000000000000000000000000000000000 111111111 001000000 220 7 0 0";
/// An abandoned mine's sets (bit 0 wood … bit 6 gold): all but wood (most shipped maps'), then
/// gold only, in turn.
const SETS: [u8; 2] = [0b111_1110, 0b100_0000];
/// The Lighthouses and Shipyards on the coast (survey §1's most: 16 and 11).
const LIGHTHOUSES: usize = 16;
const SHIPYARDS: usize = 11;
/// The weekly objects' most on a map (survey §1): Windmills (half of them underground), Water
/// Wheels and Mystical Gardens (on the surface: their pictures are of its terrains).
const WINDMILLS: usize = 18;
const WATER_WHEELS: usize = 24;
const GARDENS: usize = 11;

/// The object kinds the map places, for the counts the tests expect by type key.
const KEYS: [(&str, usize); 13] = [
    ("base:sawmill", MINES[0]),
    ("base:alchemists_lab", MINES[1]),
    ("base:ore_pit", MINES[2]),
    ("base:sulfur_dune", MINES[3]),
    ("base:crystal_cavern", MINES[4]),
    ("base:gem_pond", MINES[5]),
    ("base:gold_mine", MINES[6]),
    ("base:abandoned_mine", ABANDONED_53 + ABANDONED_220),
    ("base:lighthouse", LIGHTHOUSES),
    ("base:shipyard", SHIPYARDS),
    ("base:windmill", WINDMILLS),
    ("base:water_wheel", WATER_WHEELS),
    ("base:mystical_garden", GARDENS),
];

/// The first `n` of `count` items for the surface: the larger half.
const fn surface_share(count: usize) -> usize {
    count.div_ceil(2)
}

/// One object of a level's lattice before its cell is chosen.
type Placing = Box<dyn Fn((u8, u8)) -> w::MapObject>;

/// Takes one item of each group in turn until all are taken, so that each kind spreads over the
/// lattice.
fn interleave(mut groups: Vec<Vec<Placing>>) -> Vec<Placing> {
    for group in &mut groups {
        group.reverse();
    }
    let mut out = Vec::new();
    while groups.iter().any(|g| !g.is_empty()) {
        for group in &mut groups {
            out.extend(group.pop());
        }
    }
    out
}

/// A level's lattice objects, each kind interleaved with the others: the mines (every fourth of
/// the map's owned at load, red's and blue's in turn), the abandoned mines, the weekly objects.
fn lattice(underground: bool, mines_before: &mut usize) -> Vec<Placing> {
    let mut groups: Vec<Vec<Placing>> = Vec::new();
    for (subclass, &count) in MINES.iter().enumerate() {
        let share = surface_share(count);
        let (n, line) = if underground {
            (count - share, UNDERGROUND_MINES[subclass])
        } else {
            (share, lines::MINES[subclass])
        };
        groups.push(
            (0..n)
                .map(|_| {
                    let m = *mines_before;
                    *mines_before += 1;
                    let owner = m
                        .is_multiple_of(4)
                        .then(|| u8::try_from(m / 4 % 2).unwrap());
                    Box::new(move |at| owned(line, at, w::OwnedBody::Owner(owner))) as Placing
                })
                .collect(),
        );
    }
    let abandoned: Vec<&str> = if underground {
        vec![ABANDONED_220_UNDERGROUND; ABANDONED_220 - surface_share(ABANDONED_220)]
    } else {
        let mut lines = vec![ABANDONED_GRASS; ABANDONED_53];
        lines.extend(vec![ABANDONED_220_GRASS; surface_share(ABANDONED_220)]);
        lines
    };
    let offset = if underground {
        ABANDONED_53 + surface_share(ABANDONED_220)
    } else {
        0
    };
    groups.push(
        abandoned
            .into_iter()
            .enumerate()
            .map(|(n, line)| {
                let set = SETS[(offset + n) % SETS.len()];
                Box::new(move |at| owned(line, at, w::OwnedBody::Resources(set))) as Placing
            })
            .collect(),
    );
    let weekly = |line: &'static str, count: usize| -> Vec<Placing> {
        (0..count)
            .map(|_| Box::new(move |at| plain(line, at)) as Placing)
            .collect()
    };
    let windmills = surface_share(WINDMILLS);
    if underground {
        groups.push(weekly(lines::WINDMILL, WINDMILLS - windmills));
    } else {
        groups.push(weekly(lines::WINDMILL, windmills));
        groups.push(weekly(lines::WATER_WHEEL, WATER_WHEELS));
        groups.push(weekly(lines::GARDEN, GARDENS));
    }
    interleave(groups)
}

/// The tiles an object on `at` and the ring around its picture cover (at most four columns left
/// and three rows above its position, one beyond).
fn ring((x, y): (u8, u8)) -> impl Iterator<Item = (u8, u8)> {
    let xs = x.saturating_sub(4)..=x.saturating_add(1).min(SIDE - 1);
    let ys = y.saturating_sub(3)..=y.saturating_add(1).min(SIDE - 1);
    ys.flat_map(move |y| xs.clone().map(move |x| (x, y)))
}

/// A wandering monster on `at`, the `n`-th of the map's: gnolls and pikemen in turn, 5 to 45 of
/// them, each disposition in turn, one in seven never fleeing, one in 25 never growing.
fn wandering(n: usize, at: (u8, u8)) -> w::MapObject {
    w::MapObject::Monster(w::Monster {
        tile: at,
        template: template(if n.is_multiple_of(2) {
            lines::GNOLLS
        } else {
            lines::PIKEMEN
        }),
        count: u16::try_from(5 + n * 13 % 41).unwrap(),
        disposition: u8::try_from(n % 5).unwrap(),
        never_flees: n.is_multiple_of(7),
        fixed_count: n % 25 == 24,
        treasure: None,
    })
}

/// One level's objects: its four heroes, its lattice objects, the coast's (the surface's), then
/// its wandering monsters (`monsters` counts the map's so far).
fn level(underground: bool, mines_before: &mut usize, monsters: &mut usize) -> Vec<w::MapObject> {
    let types = TYPES[usize::from(underground)];
    let mut objects: Vec<w::MapObject> = HEROES
        .iter()
        .zip(types)
        .zip([0, 0, 1, 1])
        .map(|((&at, hero_type), owner)| w::MapObject::Hero(hero(at, owner, hero_type)))
        .collect();
    let columns = if underground {
        SIDE / CELL
    } else {
        SURFACE_COLUMNS
    };
    let cells: Vec<(u8, u8)> = (0..SIDE / CELL)
        .flat_map(|j| (0..columns).map(move |i| (CELL * i + 5, CELL * j + 5)))
        .collect();
    let placing = lattice(underground, mines_before);
    let mut taken: Vec<(u8, u8)> = Vec::new();
    for (n, place) in placing.iter().enumerate() {
        let at = cells[n * cells.len() / placing.len()];
        objects.push(place(at));
        taken.push(at);
    }
    if !underground {
        // The coast: every fifth row from row 2, a Lighthouse and a Shipyard in turn until the
        // Shipyards are placed, then Lighthouses; the first Shipyard red's at load.
        let (mut lighthouses, mut shipyards) = (0, 0);
        for k in 0..LIGHTHOUSES + SHIPYARDS {
            let at = (COAST, u8::try_from(2 + 5 * k).unwrap());
            if k % 2 == 1 && shipyards < SHIPYARDS {
                let owner = (shipyards == 0).then_some(0);
                objects.push(owned(lines::SHIPYARD, at, w::OwnedBody::Owner(owner)));
                shipyards += 1;
            } else {
                objects.push(owned(lines::LIGHTHOUSE, at, w::OwnedBody::Owner(None)));
                lighthouses += 1;
            }
            taken.push(at);
        }
        assert_eq!((lighthouses, shipyards), (LIGHTHOUSES, SHIPYARDS));
    }
    let mut covered = vec![false; usize::from(SIDE) * usize::from(SIDE)];
    let index = |(x, y): (u8, u8)| usize::from(y) * usize::from(SIDE) + usize::from(x);
    for &at in &taken {
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
    let land = if underground { SIDE } else { COAST + 1 };
    let mut free: usize = 0;
    for y in 0..SIDE {
        for x in 0..land {
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

/// The map: SoD, 144 × 144, grass with the sea east of column 127 over an underground of
/// subterranean ground; red and blue with two heroes on each level (`HEROES`); on each level's
/// lattice its share of the mines (the larger half of each on the surface; every fourth owned at
/// load, red's and blue's in turn), of the abandoned mines (class 53's four on the surface, class
/// 220's ten half and half; all but wood or gold only in turn) and of the weekly objects (the
/// Windmills half and half, the Water Wheels and the Gardens on the surface); on the coast, every
/// fifth row, the Lighthouses and the Shipyards (each Shipyard's boat tile the water two columns
/// right of its entrance; the first red's at load); a wandering monster on every hundredth free
/// tile. Returns the map and its count of monsters.
fn map() -> (w::Map, usize) {
    let mut players: [Option<w::Player>; w::PLAYERS] = Default::default();
    for player in &mut players[..2] {
        *player = Some(w::Player {
            human: true,
            computer: true,
            towns: 1,
        });
    }
    let (mut mines, mut monsters) = (0, 0);
    let surface = level(false, &mut mines, &mut monsters);
    let underground = level(true, &mut mines, &mut monsters);
    assert_eq!(mines, MINES.iter().sum::<usize>());
    let map = w::Map {
        name: "Economy XL".into(),
        description: "E08 T86b2: the day start's benchmarks".into(),
        size: SIDE,
        terrain: w::Terrain::Grass,
        patches: vec![w::Patch {
            terrain: w::Terrain::Water,
            from: (COAST + 1, 0),
            to: (SIDE - 1, SIDE - 1),
        }],
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

// covers: ADV-OWN-002, ADV-SHY-001
#[test]
fn the_economy_xl_map_is_committed_as_written() {
    let (raw, monsters) = committed();
    let setup = import(&raw, &FAITHFUL).unwrap();
    setup.validate(&FAITHFUL).unwrap();
    assert_eq!(setup.heroes.len(), 8);
    let adventure = FAITHFUL.adventure();
    let count = |key: &str| {
        let id = adventure.object_type_by_key(key).unwrap();
        setup
            .map
            .objects
            .iter()
            .filter(|o| o.object_type == Some(id))
            .count()
    };
    for (key, expected) in KEYS {
        assert_eq!(count(key), expected, "{key}");
    }
    assert_eq!(count("base:monster"), monsters);
    assert!((300..400).contains(&monsters), "{monsters}");
    // Every fourth mine and the first Shipyard owned at load.
    let owned = setup
        .map
        .objects
        .iter()
        .filter(|o| o.owner.is_some())
        .count();
    assert_eq!(owned, MINES.iter().sum::<usize>().div_ceil(4) + 1);
    // Every Shipyard's boat tile on the water east of the coast.
    let state = GameState::new(&FAITHFUL, &setup, 1).unwrap();
    let shipyard = adventure.object_type_by_key("base:shipyard").unwrap();
    let objects = state.adventure().map().objects();
    let tiles: Vec<_> = objects
        .iter()
        .filter(|o| o.object_type == Some(shipyard))
        .map(|o| o.resolved.boat_tile().map(|t| (t.x, t.level)))
        .collect();
    assert_eq!(tiles, vec![Some((u16::from(COAST) + 1, 0)); SHIPYARDS]);
}

/// The game's seed whose day 29 starts a creature month when every player only ends its turns.
const MONTH_SEED: u64 = 2;

/// The map's day starts run (design §19): ending every turn to day 29, each week's step grows the
/// monsters and refills the weekly objects, each day pays the owned mines, and day 29 starts a
/// creature month whose walk adds monsters on free land tiles; the state's checks hold after.
// covers: ADV-MON-013, ADV-MON-014, GAM-CAL-002
#[test]
fn the_economy_xl_map_starts_weeks_and_months() {
    let (raw, _) = committed();
    let rules = &*FAITHFUL;
    let setup = import(&raw, rules).unwrap();
    let mut state = GameState::new(rules, &setup, MONTH_SEED).unwrap();
    let mut events = Vec::new();
    while state.day() < 29 {
        for player in [0, 1].map(PlayerId::new) {
            apply(
                &mut state,
                rules,
                Actor::Player(player),
                GameCommand::EndTurn,
                &mut events,
            )
            .unwrap();
        }
    }
    let weeks = events
        .iter()
        .filter(|e| matches!(e, GameEvent::WeekStarted { .. }))
        .count();
    assert_eq!(weeks, 4);
    let adventure = |kind: fn(&AdventureEventKind) -> bool| {
        events
            .iter()
            .filter(
                |e| matches!(e, GameEvent::Adventure(AdventureEvent { kind: k, .. }) if kind(k)),
            )
            .count()
    };
    assert!(adventure(|k| matches!(k, AdventureEventKind::MonsterChanged { .. })) > 0);
    let added = adventure(|k| matches!(k, AdventureEventKind::MonsterAdded { .. }));
    assert!(
        matches!(state.month(), MonthKind::Creature(_)),
        "{:?}",
        state.month()
    );
    assert_eq!(added, state.adventure().added_objects().len());
    assert!(added > 50, "{added}");
    let part = rules.adventure();
    let world = state.adventure();
    assert!(world.check_added(part) && world.check_weekly(part) && world.check_owners(part));
}
