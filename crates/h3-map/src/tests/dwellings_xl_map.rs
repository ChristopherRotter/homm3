//! The dwellings XL map committed under `fixtures/maps/` (E08 T96b2, increment 10's design note
//! §14): SoD, 144 × 144 with an underground (41 472 tiles), as the economy XL (`economy_xl_map.rs`)
//! without its sea; as many dwellings of class 17 and of class 20, typed random dwellings and
//! Refugee Camps as the shipped maps hold on one map (99, 14, 30 and 9: the census's "largest"
//! lines, `examples/original_data.rs`), every fourth dwelling owned at load, and a wandering
//! monster on every hundredth free tile. Written by [`crate::writer`] with the original's templates
//! (`Objects.txt`); played by the dwellings' benchmarks (`benches/adventure/`:
//! `day-start-dwellings-xl-144`, the `EndTurn` that starts day 8, the refill and the camps' draws;
//! `fork-dwellings-xl-144`, forks after a week of random play). The tests check that the committed
//! file reads to the map written here and imports with every class and its owners, that the game's
//! build resolves every random dwelling into a dwelling entered from the map, and that its week
//! starts refill every dwelling and draw every camp (it is rewritten when `H3_WRITE_FIXTURES` is
//! set; a change invalidates what names the map by content hash, ADV-Q2).

use std::path::PathBuf;

use super::dwellings::LINES;
use super::economy_map::owned;
use super::economy_xl_map::{
    CELL, HEROES, MONSTER_EVERY, Placing, SIDE, TYPES, interleave, ring, surface_share, wandering,
};
use super::play::{FAITHFUL, check_dwellings_week};
use super::sea_map::{hero, plain};
use crate::model::RawMap;
use crate::writer as w;
use crate::{Limits, import, read};
use h3_adventure::{Holds, ObjectKind};
use h3_contracts::PlayerId;
use h3_game::{Actor, GameCommand, GameEvent, GameState, apply};

/// The committed file's name.
const NAME: &str = "e08-dwellings-144.h3m";

/// The class-17 dwellings the map takes in turn: Castle's, Necropolis' and Dungeon's levels 1 to 7
/// (`Objects.txt`'s lines; the Angel Portal's the one with a free cell left of its anchor).
const CLASS_17: [&str; 21] = [
    "AVGpike0.def 001111111011111111111111111111111111111111111111 \
     100000000000000000000000000000000000000000000000 111111111 011111111 17 56 0 0",
    "AVGcros0.def 001111111111111111111111111111111111111111111111 \
     010000000000000000000000000000000000000000000000 111111111 011111111 17 57 0 0",
    "AVGgrff0.def 001111111111111111111111111111111111111111111111 \
     100000000000000000000000000000000000000000000000 111111111 011111111 17 25 0 0",
    "AVGswor0.def 001111111111111111111111111111111111111111111111 \
     100000000000000000000000000000000000000000000000 111111111 011111111 17 58 0 0",
    "AVGmonk0.def 001111111111111111111111111111111111111111111111 \
     100000000000000000000000000000000000000000000000 111111111 011111111 17 35 0 0",
    "AVGcavl0.def 001111111111111111111111111111111111111111111111 \
     100000000000000000000000000000000000000000000000 111111111 011111111 17 5 0 0",
    "AVGangl0.def 100111111111111111111111111111111111111111111111 \
     010000000000000000000000000000000000000000000000 111111111 011111111 17 8 0 0",
    "AVGskel0.def 001111111111111111111111111111111111111111111111 \
     100000000000000000000000000000000000000000000000 111111111 011111111 17 54 0 0",
    "AVGzomb0.def 001111111111111111111111111111111111111111111111 \
     100000000000000000000000000000000000000000000000 111111111 011111111 17 55 0 0",
    "AVGwght0.def 001111111111111111111111111111111111111111111111 \
     010000000000000000000000000000000000000000000000 111111111 011111111 17 48 0 0",
    "AVGvamp0.def 001111111111111111111111111111111111111111111111 \
     010000000000000000000000000000000000000000000000 111111111 011111111 17 53 0 0",
    "AVGlich0.def 001111111111111111111111111111111111111111111111 \
     100000000000000000000000000000000000000000000000 111111111 011111111 17 52 0 0",
    "AVGbkni0.def 001111111111111111111111111111111111111111111111 \
     100000000000000000000000000000000000000000000000 111111111 011111111 17 3 0 0",
    "AVGbone0.def 001111111111111111111111111111111111111111111111 \
     010000000000000000000000000000000000000000000000 111111111 011111111 17 4 0 0",
    "AVGtrog0.def 001111111111111111111111111111111111111111111111 \
     100000000000000000000000000000000000000000000000 111111111 011111111 17 46 0 0",
    "AVGharp0.def 001111111111111111111111111111111111111111111111 \
     100000000000000000000000000000000000000000000000 111111111 011111111 17 26 0 0",
    "AVGbhld0.def 001111111111111111111111111111111111111111111111 \
     010000000000000000000000000000000000000000000000 111111111 011111111 17 2 0 0",
    "AVGmdsa0.def 001111111111111111111111111111111111111111111111 \
     100000000000000000000000000000000000000000000000 111111111 011111111 17 33 0 0",
    "AVGmino0.def 001111111111111111111111111111111111111111111111 \
     010000000000000000000000000000000000000000000000 111111111 011111111 17 34 0 0",
    "AVGmant0.def 001111111111111111111111111111111111111111111111 \
     100000000000000000000000000000000000000000000000 111111111 011111111 17 32 0 0",
    "AVGrdrg0.def 001111111111111111111111111111111111111111111111 \
     010000000000000000000000000000000000000000000000 111111111 011111111 17 41 0 0",
];
/// The random dwellings by level (217, subclass 0 to 6) and by faction (218, subclass 0 to 8): the
/// lines differ in their picture and subclass alone.
const RANDOM_BY_LEVEL: &str = "avrcgen{n}.def 001111110011111111111111111111111111111111111111 \
     110000000000000000000000000000000000000000000000 111111111 011111111 217 {s} 0 0";
const RANDOM_BY_FACTION: &str = "avrcgn0{s}.def 001111110011111111111111111111111111111111111111 \
     110000000000000000000000000000000000000000000000 111111111 011111111 218 {s} 0 0";

/// The shipped maps' most on one map (E08 T96b2's census, `examples/original_data.rs`): dwellings
/// of class 17 (`Pandora's Box `), of class 20 (`Golems Aplenty Allied`), typed random dwellings
/// (`Thousand Islands (allies)`) and Refugee Camps (`Battle of the Sexes Allied`).
const DWELLINGS_17: usize = 99;
const DWELLINGS_20: usize = 14;
const RANDOM: usize = 30;
const CAMPS: usize = 9;
/// The random dwellings of 216 and of 218; the rest are of 217, about the shipped maps' share (79
/// and 1 of the 206 typed).
const RANDOM_216: usize = 12;
const RANDOM_218: usize = 3;
/// 216's faction sets (bit 0 Castle … bit 8 Conflux: all nine, Castle to Tower, Inferno and
/// Conflux) and level code ranges (the shipped maps' commonest), each in turn.
const SETS_216: [u16; 3] = [0x1FF, 0b111, 0x108];
const LEVELS_216: [(u8, u8); 6] = [(0, 6), (0, 4), (0, 3), (0, 5), (4, 6), (0, 2)];
/// 218's factions (Inferno, Dungeon, Conflux) and level code ranges.
const RANDOM_218_BODIES: [(u8, (u8, u8)); RANDOM_218] = [(3, (0, 2)), (5, (0, 3)), (8, (0, 6))];

/// A line of `Objects.txt` built from `pattern` with `{n}` and `{s}` replaced.
fn line(pattern: &str, n: usize, s: usize) -> String {
    pattern
        .replace("{n}", &n.to_string())
        .replace("{s}", &s.to_string())
}

/// The map's lattice objects by group (the class-17 dwellings, the class-20 ones, the random
/// dwellings, the Refugee Camps), each group split into the surface's larger half and the
/// underground's rest; every fourth dwelling of the map in that order owned at load, red's and
/// blue's in turn.
fn groups() -> [(Vec<Placing>, Vec<Placing>); 4] {
    use w::OwnedBody::{Owner, RandomDwelling};
    let mut dwellings = 0;
    let mut owner = move || {
        let m: usize = dwellings;
        dwellings += 1;
        m.is_multiple_of(4)
            .then(|| u8::try_from(m / 4 % 2).unwrap())
    };
    let class_17: Vec<Placing> = (0..DWELLINGS_17)
        .map(|n| {
            let (line, owner) = (CLASS_17[n % CLASS_17.len()], owner());
            Box::new(move |at| owned(line, at, Owner(owner))) as Placing
        })
        .collect();
    let [golem, conflux] = [LINES[2], LINES[3]];
    let class_20: Vec<Placing> = (0..DWELLINGS_20)
        .map(|n| {
            let (line, owner) = (if n.is_multiple_of(2) { golem } else { conflux }, owner());
            Box::new(move |at| owned(line, at, Owner(owner))) as Placing
        })
        .collect();
    let random: Vec<Placing> = (0..RANDOM)
        .map(|n| {
            let owner = owner();
            let (line, factions, levels) = if n < RANDOM_216 {
                let set = SETS_216[n % SETS_216.len()];
                let levels = LEVELS_216[n % LEVELS_216.len()];
                (LINES[5].to_owned(), Some(set), Some(levels))
            } else if n < RANDOM - RANDOM_218 {
                // 217 by level code 0 to 6 in turn, all nine factions or one in turn.
                let k = n - RANDOM_216;
                let set = if k.is_multiple_of(2) {
                    0x1FF
                } else {
                    1 << (k / 2 % 9)
                };
                (line(RANDOM_BY_LEVEL, k % 7 + 1, k % 7), Some(set), None)
            } else {
                let (faction, levels) = RANDOM_218_BODIES[n - (RANDOM - RANDOM_218)];
                let line = line(RANDOM_BY_FACTION, 0, usize::from(faction));
                (line, None, Some(levels))
            };
            Box::new(move |at| {
                let body = RandomDwelling {
                    owner,
                    factions,
                    levels,
                };
                owned(&line, at, body)
            }) as Placing
        })
        .collect();
    let camp = LINES[4];
    let camps: Vec<Placing> = (0..CAMPS)
        .map(|_| Box::new(move |at| plain(camp, at)) as Placing)
        .collect();
    [class_17, class_20, random, camps].map(|mut group| {
        let underground = group.split_off(surface_share(group.len()));
        (group, underground)
    })
}

/// One level's objects: its four heroes, its lattice objects (spread over the cells of `CELL` ×
/// `CELL` tiles, each on its cell's tile (5, 5)), then its wandering monsters (`monsters` counts
/// the map's so far) on every `MONSTER_EVERY`-th tile no object's ring and no hero's two tiles
/// cover.
fn level(underground: bool, lattice: &[Placing], monsters: &mut usize) -> Vec<w::MapObject> {
    let types = TYPES[usize::from(underground)];
    let mut objects: Vec<w::MapObject> = HEROES
        .iter()
        .zip(types)
        .zip([0, 0, 1, 1])
        .map(|((&at, hero_type), owner)| w::MapObject::Hero(hero(at, owner, hero_type)))
        .collect();
    let cells: Vec<(u8, u8)> = (0..SIDE / CELL)
        .flat_map(|j| (0..SIDE / CELL).map(move |i| (CELL * i + 5, CELL * j + 5)))
        .collect();
    let mut covered = vec![false; usize::from(SIDE) * usize::from(SIDE)];
    let index = |(x, y): (u8, u8)| usize::from(y) * usize::from(SIDE) + usize::from(x);
    for (n, place) in lattice.iter().enumerate() {
        let at = cells[n * cells.len() / lattice.len()];
        objects.push(place(at));
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

/// The map: SoD, 144 × 144 of grass over an underground of subterranean ground; red and blue with
/// two heroes on each level (the economy XL's); on each level's lattice its share of each group
/// ([`groups`]: the larger half on the surface), the groups interleaved; a wandering monster on
/// every hundredth free tile. Returns the map and its count of monsters.
fn map() -> (w::Map, usize) {
    let mut players: [Option<w::Player>; w::PLAYERS] = Default::default();
    for player in &mut players[..2] {
        *player = Some(w::Player {
            human: true,
            computer: true,
            towns: 1,
        });
    }
    let (surface, underground): (Vec<_>, Vec<_>) = groups().into_iter().unzip();
    let mut monsters = 0;
    let surface = level(false, &interleave(surface), &mut monsters);
    let underground = level(true, &interleave(underground), &mut monsters);
    let map = w::Map {
        name: "Dwellings XL".into(),
        description: "E08 T96b2: the dwellings' benchmarks".into(),
        size: SIDE,
        terrain: w::Terrain::Grass,
        patches: Vec::new(),
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

// covers: ADV-DWL-001, ADV-DWL-002, ADV-DWL-005, ADV-RFG-001, ADV-OWN-002
#[test]
fn the_dwellings_xl_map_is_committed_as_written() {
    let (raw, monsters) = committed();
    let rules = &*FAITHFUL;
    let setup = import(&raw, rules).unwrap();
    setup.validate(rules).unwrap();
    assert_eq!(setup.heroes.len(), 8);
    let part = rules.adventure();
    let count = |key: &str| {
        let id = part.object_type_by_key(key).unwrap();
        let objects = setup.map.objects.iter();
        objects.filter(|o| o.object_type == Some(id)).count()
    };
    let dwellings = setup
        .map
        .objects
        .iter()
        .filter_map(|o| part.object_type(o.object_type?))
        .filter(|ty| matches!(ty.kind, ObjectKind::Dwelling { .. }))
        .count();
    assert_eq!(dwellings, DWELLINGS_17 + DWELLINGS_20);
    let class_20 = count("base:golem_factory") + count("base:elemental_conflux");
    assert_eq!(class_20, DWELLINGS_20);
    assert_eq!(count("base:random_dwelling"), RANDOM);
    assert_eq!(count("base:refugee_camp"), CAMPS);
    assert_eq!(count("base:monster"), monsters);
    assert!((300..450).contains(&monsters), "{monsters}");
    // Every fourth dwelling owned at load; every random dwelling holds its set and range.
    let owned = setup.map.objects.iter().filter(|o| o.owner.is_some());
    let all = DWELLINGS_17 + DWELLINGS_20 + RANDOM;
    assert_eq!(owned.count(), all.div_ceil(4));
    let holds = setup.map.objects.iter();
    let holds = holds.filter(|o| matches!(o.holds, Holds::RandomDwelling { .. }));
    assert_eq!(holds.count(), RANDOM);
    // The game built: every random dwelling resolved into a dwelling entered from a tile of the
    // map, each dwelling full and each camp drawn.
    let state = GameState::new(rules, &setup, 1).unwrap();
    let adventure = state.adventure();
    let map = adventure.map();
    assert_eq!((map.dwellings().len(), map.camps().len()), (all, CAMPS));
    for &index in map.dwellings() {
        let object = &map.objects()[index];
        let ty = part.object_type(object.object_type.unwrap()).unwrap();
        assert!(matches!(ty.kind, ObjectKind::Dwelling { .. }), "{index}");
        assert_eq!(object.holds, Holds::Nothing, "{index}");
        assert!(object.entrance(map).is_some(), "{index}");
    }
    check_dwellings_week(rules, &state).unwrap();
    assert!(adventure.check_dwellings() && adventure.check_camps(rules.core(), part));
}

/// The map's week starts run (design §9, §14): ending every turn to day 15, each week's step
/// refills every dwelling and draws every camp anew, and the state's checks hold after each.
// covers: ADV-DWL-003, ADV-RFG-001, GAM-CAL-002
#[test]
fn the_dwellings_xl_map_starts_weeks() {
    let (raw, _) = committed();
    let rules = &*FAITHFUL;
    let setup = import(&raw, rules).unwrap();
    let mut state = GameState::new(rules, &setup, 1).unwrap();
    let draws = |state: &GameState| -> Vec<_> {
        let adventure = state.adventure();
        let camps = adventure.map().camps().iter();
        camps.map(|&index| adventure.camp(index)).collect()
    };
    let mut drawn = vec![draws(&state)];
    let mut weeks = 0;
    while state.day() < 15 {
        let mut events = Vec::new();
        for player in [0, 1].map(PlayerId::new) {
            let command = GameCommand::EndTurn;
            apply(
                &mut state,
                rules,
                Actor::Player(player),
                command,
                &mut events,
            )
            .unwrap();
        }
        if events
            .iter()
            .any(|e| matches!(e, GameEvent::WeekStarted { .. }))
        {
            weeks += 1;
            check_dwellings_week(rules, &state).unwrap();
            let adventure = state.adventure();
            assert!(
                adventure.check_dwellings()
                    && adventure.check_camps(rules.core(), rules.adventure())
            );
            drawn.push(draws(&state));
        }
    }
    assert_eq!(weeks, 2);
    // The camps drawn anew each week: no two weeks draw the same nine.
    assert!(drawn[0] != drawn[1] && drawn[1] != drawn[2], "{drawn:?}");
}
