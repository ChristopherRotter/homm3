//! The town map committed under `fixtures/maps/` (E09 T9a, design note `docs/epics/E09-design.md`
//! §18): two players and eight towns of every kind the import reads. Red's main town, a Castle
//! with a custom set (a Capitol, a Resource Silo, the Grail building, a Tavern), named, generates
//! red's starting hero; red's random town takes red's faction, its garrison of random stacks; a
//! neutral Inferno whose garrison is drawn, a neutral Necropolis explicitly empty; a neutral
//! random town aligned to blue and one drawn; blue's Tower, whose visitor stood right of its
//! entrance (TWN-SET-010), and blue's main town, a Stronghold with a Capitol, empty. Red's heroes:
//! ten archangels, five halberdiers, a Knight with Rogues whose Visions reach blue's Tower, a
//! random hero. Written by [`crate::writer`] as the dwellings map (`dwellings_map.rs`); the golden
//! replays `towns-*` play it (`crates/h3-cli/tests/adventure_replays.rs`). The test checks that
//! the committed file reads to the map written here, imports with every town's faction, owner,
//! buildings, garrison and name, the starts' main towns and the heroes, and builds: the random
//! towns' factions, the main town's hero on its entrance, the random stacks drawn of their levels,
//! the Visions' view of blue's Tower. It rewrites the file when `H3_WRITE_FIXTURES` is set (a
//! change invalidates the replays that name the map by content hash, ADV-Q2).

use std::path::PathBuf;

use super::play::FAITHFUL;
use super::sea_map::hero;
use crate::import::codes;
use crate::writer as w;
use crate::{Limits, import, read};
use h3_adventure::Pos;
use h3_contracts::{HeroId, PlayerId, TownId};
use h3_game::{
    BuildingsSetup, FactionChoice, GameState, GarrisonCreature, GarrisonStack, MainTown,
    OWNER_LEVEL, TownFaction, Viewer, observe,
};

/// The committed file's name.
const NAME: &str = "e09-towns-36.h3m";

/// `Objects.txt`'s town pictures (class 98 by faction in `TOWNTYPE.TXT`'s order, then the random
/// town's, 77); every town's line has the same masks ([`MASKS`]).
const PICTURES: [&str; 10] = [
    "AVCcasx0", "AVCramx0", "AVCtowx0", "AVCinfx0", "AVCnecx0", "AVCdunx0", "AVCstrx0", "AVCftrx0",
    "avchforx", "AVCranx0",
];
/// A town's masks in `Objects.txt`: five tiles of its row and of the row above blocked, three of
/// the row above that, the entrance two columns left of its position; every land terrain.
const MASKS: &str = "000001110000011110001111111111111111111111111111 \
                     001000000000000000000000000000000000000000000000 011111111 011111111";

/// Faction codes (`TOWNTYPE.TXT`, the writer's subclasses).
const CASTLE: u32 = 0;
const TOWER: u32 = 2;
const INFERNO: u32 = 3;
const NECROPOLIS: u32 = 4;
const STRONGHOLD: u32 = 6;

/// Creature codes: Halberdiers, Archangels, Gremlins, Stone Gargoyles, Gorgons (a Fortress
/// creature in a Stronghold hero's army), Rogues.
const HALBERDIER: u16 = 1;
const ARCHANGEL: u16 = 13;
const GREMLIN: u16 = 28;
const GARGOYLE: u16 = 30;
const GORGON: u16 = 102;
const ROGUE: u16 = 143;

/// The heroes' tiles as placed, by hero id (the map's order): red's Orrin, Valeska, Sorsha (the
/// Rogues) and the random hero, then blue's Yog (right of the Tower's entrance) and Gurnisson.
pub(super) const HEROES: [(u8, u8); 6] = [(7, 8), (11, 8), (27, 12), (15, 8), (30, 10), (27, 30)];
/// The towns' positions by `TownId` (each entered two columns left of it): red's main Castle,
/// red's random town, the neutral Inferno and Necropolis, the neutral random towns aligned to blue
/// and drawn, blue's Tower and blue's main Stronghold.
pub(super) const TOWNS: [(u8, u8); 8] = [
    (9, 5),
    (9, 12),
    (20, 5),
    (14, 18),
    (24, 18),
    (18, 27),
    (31, 10),
    (31, 22),
];

/// Building bits (`docs/formats/h3m.md`, "Town buildings").
const TOWN_HALL: u8 = 0;
const CITY_HALL: u8 = 1;
const CAPITOL: u8 = 2;
const FORT: u8 = 3;
const TAVERN: u8 = 6;
const SILO: u8 = 9;
const GRAIL: u8 = 17;

/// A custom set of `bits` built, none forbidden.
fn custom(bits: &[u8]) -> w::TownBuildings {
    let mut built = [0; 6];
    for &bit in bits {
        built[usize::from(bit / 8)] |= 1 << (bit % 8);
    }
    w::TownBuildings::Custom {
        built,
        forbidden: [0; 6],
    }
}

/// A town on `at` of `faction` (`None`: random), its template from `Objects.txt`.
pub(super) fn town(at: (u8, u8), faction: Option<u32>, owner: Option<u8>) -> w::AnyTown {
    let (picture, class, subclass) = match faction {
        Some(faction) => (PICTURES[faction as usize], w::TOWN_OBJECT, faction),
        None => (PICTURES[9], w::RANDOM_TOWN_OBJECT, 0),
    };
    let line = format!("{picture}.def {MASKS} {class} {subclass} 1 0");
    w::AnyTown {
        tile: at,
        template: w::ObjectTemplate::from_objects_txt(&line).unwrap(),
        owner,
        alignment: None,
        main: None,
        name: None,
        garrison: None,
        tight_formation: false,
        buildings: w::TownBuildings::Standard { fort: true },
        events: Vec::new(),
    }
}

/// Seven slots of `stacks`, in order.
fn slots(stacks: &[(u16, u16)]) -> [Option<(u16, u16)>; w::ARMY_SLOTS] {
    let mut army = [None; w::ARMY_SLOTS];
    for (slot, &stack) in army.iter_mut().zip(stacks) {
        *slot = Some(stack);
    }
    army
}

/// A hero of `hero_type` on `at` with `army` (its type's own when empty).
pub(super) fn with_army(at: (u8, u8), owner: u8, hero_type: u8, army: &[(u16, u16)]) -> w::Hero {
    let mut hero = hero(at, owner, hero_type);
    if !army.is_empty() {
        hero.army = Some(slots(army));
    }
    hero
}

/// The map: SoD, 36 × 36 of grass, no underground, no sea; red (Castle) in the west, blue
/// (Stronghold) in the east.
///
/// - Red's Orrin (hero 0) on (7, 8) with ten archangels (a siege won), Valeska (hero 1) on (11, 8)
///   with five halberdiers, Sorsha (hero 2) on (27, 12) with eight Rogues and Power 1 (Visions'
///   radius 3: blue's Tower's entrance (29, 10) at a squared distance of 8), a random hero (hero 3)
///   on (15, 8).
/// - Blue's Yog (hero 4) on (30, 10), right of the Tower's entrance, with fifteen Gorgons (moved
///   onto the entrance, the Tower's visitor), Gurnisson (hero 5) on (27, 30), his type's army.
/// - The towns ([`TOWNS`]): red's main Castle "Haven" on (9, 5) (the Town Hall, City Hall and
///   Capitol, a Fort, a Tavern, a Resource Silo and the Grail building; it generates red's
///   starting hero), red's random town on (9, 12) (a level 1 random stack of 20 and an upgraded
///   level 2 one of 5), the neutral Inferno on (20, 5) (the editor's garrison: drawn), the neutral
///   Necropolis on (14, 18) (an empty garrison, no Fort), the neutral random towns on (24, 18)
///   (aligned to blue) and (18, 27), blue's Tower on (31, 10) (30 Gremlins, 10 Stone Gargoyles)
///   and blue's main Stronghold on (31, 22) (the halls to the Capitol and a Fort, an empty
///   garrison; no hero generated).
fn map() -> w::Map {
    let mut players: [Option<w::Player>; w::PLAYERS] = Default::default();
    players[0] = Some(w::Player {
        human: true,
        computer: true,
        towns: 1 << CASTLE,
    });
    players[1] = Some(w::Player {
        human: true,
        computer: true,
        towns: 1 << STRONGHOLD,
    });
    let [orrin, valeska, sorsha, random, yog, gurnisson] = HEROES;
    let mut rogue = with_army(sorsha, 0, 5, &[(ROGUE, 8)]);
    rogue.primary = Some([2, 2, 1, 1]);
    let mut objects = vec![
        w::MapObject::Hero(with_army(orrin, 0, 0, &[(ARCHANGEL, 10)])),
        w::MapObject::Hero(with_army(valeska, 0, 1, &[(HALBERDIER, 5)])),
        w::MapObject::Hero(rogue),
        w::MapObject::RandomHero(w::RandomHero {
            tile: random,
            owner: 0,
            experience: None,
            army: None,
            tight_formation: false,
        }),
        w::MapObject::Hero(with_army(yog, 1, 96, &[(GORGON, 15)])),
        w::MapObject::Hero(with_army(gurnisson, 1, 97, &[])),
    ];
    let [
        haven,
        red_random,
        inferno,
        necropolis,
        aligned,
        drawn,
        tower,
        stronghold,
    ] = TOWNS;
    let random_stack =
        |level, upgraded, count| w::random_stack(level, upgraded).map(|c| (c, count));
    let towns = [
        w::AnyTown {
            main: Some(w::MainTown {
                generates_hero: true,
            }),
            name: Some("Haven".into()),
            buildings: custom(&[TOWN_HALL, CITY_HALL, CAPITOL, FORT, TAVERN, SILO, GRAIL]),
            ..town(haven, Some(CASTLE), Some(0))
        },
        w::AnyTown {
            garrison: Some([
                random_stack(1, false, 20),
                random_stack(2, true, 5),
                None,
                None,
                None,
                None,
                None,
            ]),
            buildings: w::TownBuildings::Standard { fort: false },
            ..town(red_random, None, Some(0))
        },
        town(inferno, Some(INFERNO), None),
        w::AnyTown {
            garrison: Some([None; w::ARMY_SLOTS]),
            buildings: w::TownBuildings::Standard { fort: false },
            ..town(necropolis, Some(NECROPOLIS), None)
        },
        w::AnyTown {
            alignment: Some(1),
            ..town(aligned, None, None)
        },
        town(drawn, None, None),
        w::AnyTown {
            garrison: Some(slots(&[(GREMLIN, 30), (GARGOYLE, 10)])),
            ..town(tower, Some(TOWER), Some(1))
        },
        w::AnyTown {
            main: Some(w::MainTown {
                generates_hero: false,
            }),
            garrison: Some([None; w::ARMY_SLOTS]),
            buildings: custom(&[TOWN_HALL, CITY_HALL, CAPITOL, FORT]),
            ..town(stronghold, Some(STRONGHOLD), Some(1))
        },
    ];
    objects.extend(towns.map(w::MapObject::AnyTown));
    w::Map {
        name: "Towns".into(),
        description: "E09 T9a: towns of every kind, main towns, a random hero, a Rogue".into(),
        size: 36,
        terrain: w::Terrain::Grass,
        patches: Vec::new(),
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

pub(super) fn at((x, y): (u8, u8)) -> Pos {
    Pos::new(x.into(), y.into(), 0)
}

/// A town's entrance: two columns left of its position.
pub(super) fn entrance((x, y): (u8, u8)) -> Pos {
    at((x - 2, y))
}

// covers: TWN-SET-001, TWN-SET-002, TWN-SET-007, TWN-SET-009, TWN-SET-010, TWN-VEW-002, TWN-VEW-003
#[test]
fn the_towns_map_is_committed_as_written() {
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
    let core = FAITHFUL.core();
    let faction = |code: u32| core.faction_by_key(codes::FACTIONS[code as usize]).unwrap();
    let (red, blue) = (PlayerId::new(0), PlayerId::new(1));
    // The starts: each player's one faction, its main town.
    let starts: Vec<_> = setup.players.iter().map(|p| p.start).collect();
    assert_eq!(
        starts
            .iter()
            .map(|s| (s.choice, s.main_town, s.random_hero))
            .collect::<Vec<_>>(),
        [
            (
                Some(FactionChoice::Fixed(faction(CASTLE))),
                Some(MainTown {
                    town: TownId::new(0),
                    generates_hero: true,
                }),
                false,
            ),
            (
                Some(FactionChoice::Fixed(faction(STRONGHOLD))),
                Some(MainTown {
                    town: TownId::new(7),
                    generates_hero: false,
                }),
                false,
            ),
        ]
    );
    // The heroes: Yog moved onto the Tower's entrance, the random hero without a type.
    let heroes: Vec<_> = setup
        .heroes
        .iter()
        .map(|h| (h.pos, h.owner, h.hero_type.is_some()))
        .collect();
    let mut expected: Vec<_> = HEROES
        .iter()
        .zip([red, red, red, red, blue, blue])
        .map(|(&tile, owner)| (at(tile), owner, true))
        .collect();
    expected[3].2 = false;
    expected[4].0 = entrance(TOWNS[6]);
    assert_eq!(heroes, expected);
    // The towns: their factions, buildings, garrisons and names as stored; their owners.
    let factions: Vec<_> = setup.towns.iter().map(|t| t.faction).collect();
    let random = |alignment| TownFaction::Random { alignment };
    assert_eq!(
        factions,
        [
            TownFaction::Fixed(faction(CASTLE)),
            random(None),
            TownFaction::Fixed(faction(INFERNO)),
            TownFaction::Fixed(faction(NECROPOLIS)),
            random(Some(1)),
            random(None),
            TownFaction::Fixed(faction(TOWER)),
            TownFaction::Fixed(faction(STRONGHOLD)),
        ]
    );
    let standard: Vec<_> = setup
        .towns
        .iter()
        .map(|t| match t.buildings {
            BuildingsSetup::Standard { fort } => Some(fort),
            BuildingsSetup::Custom { .. } => None,
        })
        .collect();
    let fort = Some(true);
    let no_fort = Some(false);
    assert_eq!(
        standard,
        [None, no_fort, fort, no_fort, fort, fort, fort, None]
    );
    let stack = |slot, level, upgraded, count| GarrisonStack {
        slot,
        creature: GarrisonCreature::Random { level, upgraded },
        count,
    };
    assert_eq!(
        setup.towns[1].garrison,
        Some(vec![stack(0, 1, false, 20), stack(1, 2, true, 5)])
    );
    let garrisons: Vec<_> = setup
        .towns
        .iter()
        .map(|t| t.garrison.as_ref().map(Vec::len))
        .collect();
    assert_eq!(
        garrisons,
        [None, Some(2), None, Some(0), None, None, Some(2), Some(0)]
    );
    assert_eq!(setup.towns[0].name.as_deref(), Some("Haven"));
    let adventure = FAITHFUL.adventure();
    let town_type = adventure.object_type_by_key("base:town");
    let towns: Vec<_> = setup
        .map
        .objects
        .iter()
        .filter(|o| o.object_type == town_type)
        .map(|o| (o.anchor, o.owner))
        .collect();
    let owners = [
        Some(red),
        Some(red),
        None,
        None,
        None,
        None,
        Some(blue),
        Some(blue),
    ];
    assert_eq!(
        towns,
        TOWNS
            .iter()
            .zip(owners)
            .map(|(&tile, owner)| (at(tile), owner))
            .collect::<Vec<_>>()
    );
    // The game built with seed 1 (the scripted replays'): red's random town red's Castle, the
    // town aligned to blue a Stronghold, the drawn one of a faction with a town record.
    let state = GameState::new(&FAITHFUL, &setup, 1).unwrap();
    let built: Vec<_> = (0..8)
        .map(|id| state.towns().get(TownId::new(id)).unwrap().faction())
        .collect();
    assert_eq!(built[..2], [faction(CASTLE), faction(CASTLE)]);
    assert_eq!(built[4], faction(STRONGHOLD));
    assert!(FAITHFUL.towns().town(built[5]).is_some());
    // Red's main town's hero on its entrance (its visitor), of a Castle class, after the setup's
    // six heroes; the random hero of a Castle class too (TWN-SET-007).
    let castle_class = |id: u16| {
        let ty = state.hero(HeroId::new(id)).unwrap().hero_type;
        let class = core.hero_type(ty).unwrap().class;
        core.hero_class(class).unwrap().faction == faction(CASTLE)
    };
    let generated = state.adventure().hero(HeroId::new(6)).unwrap();
    assert_eq!((generated.owner, generated.pos), (red, entrance(TOWNS[0])));
    assert!(castle_class(6) && castle_class(3));
    // The garrisons: red's random stacks drawn of their levels, the Inferno's drawn, the
    // Necropolis' and the Stronghold's empty, the Tower's as written.
    let index = |id: usize| state.adventure().map().towns()[id];
    let garrison = |id| state.adventure().town_garrison(index(id)).unwrap();
    let levels: Vec<_> = garrison(1)
        .iter()
        .map(|s| (s.slot, core.creature(s.creature).unwrap().level, s.count))
        .collect();
    assert_eq!(levels, [(0, 1, 20), (1, 2, 5)]);
    assert!(!garrison(2).is_empty());
    assert!(garrison(3).is_empty() && garrison(7).is_empty());
    assert_eq!(garrison(6).len(), 2);
    // Red sees blue's Tower at the top level by Sorsha's Rogues (TWN-VEW-003), and the neutral
    // Inferno, beyond the Visions' reach, at the level of red's two Taverns (Haven's and the
    // random town's standard one; TWN-VEW-002).
    let seen = observe(&state, &FAITHFUL, Viewer::Player(red));
    let level = |id: u16| {
        seen.towns
            .iter()
            .find(|t| t.town == TownId::new(id))
            .map(|t| (t.garrison.level, t.owner_facts.is_some()))
    };
    assert_eq!(level(6), Some((OWNER_LEVEL, true)));
    assert_eq!(level(2), Some((2, false)));
}
