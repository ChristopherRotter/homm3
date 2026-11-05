//! The dwellings map committed under `fixtures/maps/` (E08 T96a, increment 10's design note §14):
//! two players; a Guardhouse, a Barracks, the Dragon Cliffs (guarded), the Golem Factory and the
//! Elemental Conflux, neutral and owned at load by each player; random dwellings of 216, 217 and
//! 218, the Angel Portal among their outcomes; two Refugee Camps; a hero with seven stacks; gold
//! and no crystal. Written by [`crate::writer`] as the economy map (`economy_map.rs`); the golden
//! replays recorded on it play it (random play over 22 days and the scripted ones,
//! `crates/h3-cli/tests/adventure_replays.rs`; T96a's later parts). The test checks that the committed file reads to
//! the map written here and imports with every class, its owners and the random dwellings' holds,
//! and that the game's build resolves each random dwelling into a dwelling of its set and levels
//! with its type's footprint at the moved anchor; it rewrites the file when `H3_WRITE_FIXTURES` is
//! set (a change invalidates the replays that name the map by content hash, ADV-Q2).

use std::path::PathBuf;

use super::dwellings::LINES;
use super::economy_map::owned;
use super::play::FAITHFUL;
use super::sea_map::{hero, plain};
use crate::import::codes;
use crate::writer as w;
use crate::{Limits, import, read};
use h3_adventure::{Holds, ObjectKind, Pos};
use h3_contracts::PlayerId;
use h3_game::GameState;

/// The committed file's name.
const NAME: &str = "e08-dwellings-36.h3m";

/// The original's Barracks (17.58, Castle's level 4, Swordsmen; `Objects.txt`), visited on its
/// position; the other dwellings' lines are `dwellings.rs`' [`LINES`].
const BARRACKS: &str = "AVGswor0.def 001111111111111111111111111111111111111111111111 \
     100000000000000000000000000000000000000000000000 111111111 011111111 17 58 0 0";

/// Creature codes: Halberdiers, Archangels, and the seven Rampart stacks of the full army
/// (Centaurs, Centaur Captains, Dwarves, Wood Elves, Pegasi, Dendroid Guards, Unicorns), none a
/// creature of the map's dwellings.
const HALBERDIER: u16 = 1;
const ARCHANGEL: u16 = 13;
const FULL_ARMY: [(u16, u16); 7] = [
    (14, 10),
    (15, 5),
    (16, 8),
    (18, 6),
    (20, 4),
    (22, 3),
    (24, 2),
];

/// The heroes' tiles, by hero id (the map's order): red's Orrin, Valeska and Thorgrim, then blue's
/// Adela and Rion.
pub(super) const HEROES: [(u8, u8); 5] = [(5, 9), (5, 6), (16, 3), (6, 30), (26, 30)];
/// The dwellings' positions (their entrances: the Elemental Conflux's one column left, the others
/// on it): the Guardhouse, the Barracks, the Dragon Cliffs, red's Golem Factory, blue's Elemental
/// Conflux.
pub(super) const DWELLINGS: [(u8, u8); 5] = [(13, 4), (13, 9), (8, 4), (18, 6), (10, 27)];
/// The random dwellings' positions (each entered on it or one column left; the Angel Portal on
/// it): the neutral 216 of Castle, Rampart or Tower, levels 1 to 7; red's 217 of level 3, Castle
/// or Tower; blue's 218 of Inferno, levels 1 to 4.
pub(super) const RANDOM: [(u8, u8); 3] = [(18, 27), (20, 11), (23, 27)];
/// The Refugee Camps, visited on their positions: red's side, blue's side.
pub(super) const CAMPS: [(u8, u8); 2] = [(9, 12), (14, 31)];

/// A hero of `hero_type` on `at` with `army` (its type's own when empty).
fn with_army(at: (u8, u8), owner: u8, hero_type: u8, army: &[(u16, u16)]) -> w::MapObject {
    let mut hero = hero(at, owner, hero_type);
    if !army.is_empty() {
        let mut slots = [None; w::ARMY_SLOTS];
        for (slot, &stack) in slots.iter_mut().zip(army) {
            *slot = Some(stack);
        }
        hero.army = Some(slots);
    }
    w::MapObject::Hero(hero)
}

/// The one timed event: on day 1 both players lose every crystal (100, more than any difficulty
/// gives), so the Dragon Cliffs' Green Dragons are refused for the treasury.
fn events() -> Vec<w::TimedEvent> {
    vec![w::TimedEvent {
        name: "No crystal".into(),
        message: "The crystal is gone.".into(),
        resources: [0, 0, 0, 0, -100, 0, 0],
        players: 0b11,
        human: true,
        computer: true,
        first_day: 0,
        interval: 0,
    }]
}

/// The map: SoD, 36 × 36 of grass, no underground, no sea; red in the north, blue in the south;
/// the difficulty's gold and, after day 1's event ([`events`]), no crystal.
///
/// - Red's Orrin (hero 0) on (5, 9) with ten archangels (the guards' battle won), Valeska
///   (hero 1) on (5, 6) with five halberdiers (the battle lost), Thorgrim (hero 2) on (16, 3) with
///   seven stacks of Rampart creatures (D-52: the free creatures and the dialog refused for room).
/// - Blue's Adela (hero 3) on (6, 30) and Rion (hero 4) on (26, 30), their types' armies.
/// - The dwellings ([`DWELLINGS`]): the neutral Guardhouse on (13, 4) (Pikemen, level 1: free) and
///   Barracks on (13, 9) (Swordsmen: a dialog of one creature); the neutral Dragon Cliffs on
///   (8, 4), guarded (Green Dragons: crystal); red's Golem Factory on (18, 6) and blue's Elemental
///   Conflux on (10, 27) (entered from (9, 27)), four creatures each.
/// - The random dwellings ([`RANDOM`]): the neutral 216 on (18, 27) (the Angel Portal among its
///   outcomes, its anchor moved to (19, 27)), red's 217 of level 3 on (20, 11), blue's 218 of
///   Inferno on (23, 27).
/// - The Refugee Camps ([`CAMPS`]) on (9, 12) and (14, 31).
fn map() -> w::Map {
    use w::OwnedBody::{Owner, RandomDwelling};
    let mut players: [Option<w::Player>; w::PLAYERS] = Default::default();
    for player in &mut players[..2] {
        *player = Some(w::Player {
            human: true,
            computer: true,
            towns: 1,
        });
    }
    let [orrin, valeska, thorgrim, adela, rion] = HEROES;
    let mut objects = vec![
        with_army(orrin, 0, 0, &[(ARCHANGEL, 10)]),
        with_army(valeska, 0, 1, &[(HALBERDIER, 5)]),
        with_army(thorgrim, 0, 20, &FULL_ARMY),
        with_army(adela, 1, 9, &[]),
        with_army(rion, 1, 8, &[]),
    ];
    let [pike, dragon, golem, conflux, camp, any, level_3, inferno] = LINES;
    let [guardhouse, barracks, cliffs, factory, confluence] = DWELLINGS;
    objects.extend([
        owned(pike, guardhouse, Owner(None)),
        owned(BARRACKS, barracks, Owner(None)),
        owned(dragon, cliffs, Owner(None)),
        owned(golem, factory, Owner(Some(0))),
        owned(conflux, confluence, Owner(Some(1))),
    ]);
    let bodies = [
        (any, None, Some(0b111), Some((0, 6))),
        (level_3, Some(0), Some(0b101), None),
        (inferno, Some(1), None, Some((0, 3))),
    ];
    for ((line, owner, factions, levels), at) in bodies.into_iter().zip(RANDOM) {
        let body = RandomDwelling {
            owner,
            factions,
            levels,
        };
        objects.push(owned(line, at, body));
    }
    objects.extend(CAMPS.map(|at| plain(camp, at)));
    w::Map {
        name: "Dwellings".into(),
        description: "E08 T96a: dwellings, random dwellings and Refugee Camps".into(),
        size: 36,
        terrain: w::Terrain::Grass,
        patches: Vec::new(),
        roads: Vec::new(),
        players,
        objects,
        underground: None,
        level_limit: 0,
        banned_skills: Vec::new(),
        events: events(),
        teams: None,
    }
}

/// Increment 10's object types on the map, in its order after the heroes.
const CLASSES: [&str; 10] = [
    "base:guardhouse",
    "base:barracks",
    "base:dragon_cliffs",
    "base:golem_factory",
    "base:elemental_conflux",
    "base:random_dwelling",
    "base:random_dwelling",
    "base:random_dwelling",
    "base:refugee_camp",
    "base:refugee_camp",
];

/// The random dwellings' holds as the import makes them: faction codes and a level range.
const HOLDS: [(&[usize], (u8, u8)); 3] = [(&[0, 1, 2], (1, 7)), (&[0, 2], (3, 3)), (&[3], (1, 4))];

fn at((x, y): (u8, u8)) -> Pos {
    Pos::new(x.into(), y.into(), 0)
}

fn holds((set, levels): (&[usize], (u8, u8))) -> Holds {
    let core = FAITHFUL.core();
    let factions = set
        .iter()
        .map(|&code| core.faction_by_key(codes::FACTIONS[code]).unwrap())
        .collect();
    Holds::RandomDwelling { factions, levels }
}

// covers: ADV-DWL-001, ADV-DWL-005, ADV-RFG-001, ADV-OWN-002
#[test]
fn the_dwellings_map_is_committed_as_written() {
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
    let setup = import(&committed, &FAITHFUL).unwrap();
    setup.validate(&FAITHFUL).unwrap();
    let heroes: Vec<(Pos, usize)> = setup
        .heroes
        .iter()
        .map(|h| (h.pos, h.owner.index()))
        .collect();
    assert_eq!(
        heroes,
        HEROES
            .iter()
            .zip([0, 0, 0, 1, 1])
            .map(|(&tile, owner)| (at(tile), owner))
            .collect::<Vec<_>>()
    );
    let adventure = FAITHFUL.adventure();
    let types: Vec<_> = setup.map.objects.iter().map(|o| o.object_type).collect();
    let expected: Vec<_> = CLASSES
        .iter()
        .map(|key| Some(adventure.object_type_by_key(key).unwrap()))
        .collect();
    assert_eq!(types, expected);
    // Red's Golem Factory and 217, blue's Elemental Conflux and 218 owned at load.
    let owners: Vec<_> = setup.map.objects.iter().map(|o| o.owner).collect();
    let (red, blue) = (Some(PlayerId::new(0)), Some(PlayerId::new(1)));
    let mut expected = vec![None; owners.len()];
    (expected[3], expected[4], expected[6], expected[7]) = (red, blue, red, blue);
    assert_eq!(owners, expected);
    let held: Vec<_> = setup.map.objects[5..8]
        .iter()
        .map(|o| o.holds.clone())
        .collect();
    assert_eq!(held, HOLDS.map(holds));
    assert_eq!(setup.events.len(), 1);
    // The game built with seed 1 (the scripted replays'): the Dragon Cliffs guarded by three
    // Green Dragons, the Guardhouse and the Barracks not; red's treasury after day 1's event (its
    // turn started in the build): the difficulty's gold, no crystal (`RESTYPES.TXT` order).
    let state = GameState::new(&FAITHFUL, &setup, 1).unwrap();
    let guards = |index| -> Vec<_> {
        let guards = state.adventure().dwelling_guards(index);
        guards.iter().map(|s| (s.creature, s.count)).collect()
    };
    let green = FAITHFUL
        .core()
        .creature_by_key("base:green_dragon")
        .unwrap();
    assert_eq!(guards(2), [(green, 3)]);
    assert!(guards(0).is_empty() && guards(1).is_empty());
    let treasury = state.treasury(PlayerId::new(0)).unwrap();
    assert_eq!((treasury[4], treasury[6]), (0, 20_000));
    // Each random dwelling a dwelling of its set and levels, its type's footprint at the anchor
    // moved by the type's shift.
    let objects = state.adventure().map().objects();
    for ((index, anchor), (set, (low, high))) in (5..8).zip(RANDOM).zip(HOLDS) {
        let object = &objects[index];
        let outcomes: Vec<_> = set
            .iter()
            .flat_map(|&code| {
                let faction = FAITHFUL.core().faction_by_key(codes::FACTIONS[code]);
                (1..).zip(adventure.random_dwelling(faction.unwrap()).unwrap())
            })
            .filter(|(level, _)| (low..=high).contains(level))
            .map(|(_, &ty)| Some(ty))
            .collect();
        assert!(outcomes.contains(&object.object_type), "{index}");
        let ty = adventure.object_type(object.object_type.unwrap()).unwrap();
        let ObjectKind::Dwelling {
            blocked,
            visitable,
            shift,
            ..
        } = ty.kind
        else {
            panic!("{index} is no dwelling");
        };
        assert_eq!((object.blocked, object.visitable), (blocked, visitable));
        let moved = Pos::new((anchor.0 + shift).into(), anchor.1.into(), 0);
        assert_eq!(object.anchor, moved);
        assert_eq!(object.holds, Holds::Nothing);
    }
}
