//! The builds map committed under `fixtures/maps/` (E09 T19a, design note `docs/epics/E09-design.md`
//! §32; increment 2's own map, decision 59): two players, four towns with custom sets and the
//! town events. Red's main Castle "Harbour" stands by the sea (its Shipyard's boat tile is water,
//! TWN-SET-011) with four events of red's (a day-1 build of the Marketplace and the Blacksmith, an
//! every-day and a weekly gift, a creature event); red's Castle "Keep" inland (its Shipyard
//! forbidden for want of water), every requirement of the Capitol and an upgraded dwelling (with
//! the dwellings it requires, without its horde) built, an empty garrison, a day-1 event of red's (the Shipyard, skipped, and the Resource Silo) and an
//! every-day gift of blue's; blue's main Conflux "Nexus" with a Capitol, its Altar of Water
//! forbidden (a requirement of the Altar of Earth, so that one is never possible); blue's Tower
//! "Spire" with every requirement of the Capitol and a day-2 event of blue's building it (T-8: two
//! Capitols). Red's Orrin idles in the south-west, blue's Yog stands a short walk from the Keep.
//! Written by [`crate::writer`] as the towns map (`towns_map.rs`); the golden replays `builds-*`
//! play it (`crates/h3-cli/tests/adventure_replays.rs`). The tests check that the committed file
//! reads to the map written here, imports with every town's faction, owner, buildings and events,
//! and builds: the water's Shipyards, the day-1 events fired with their builds and gains; then
//! each refusal of a build once on red's and blue's first turns, none changing the state hash
//! (TWN-BLD-001 to -004), and the day-2 Capitol beside blue's other one. It rewrites the file when
//! `H3_WRITE_FIXTURES` is set (a change invalidates the replays that name the map by content
//! hash, ADV-Q2).

use std::path::PathBuf;

use super::play::FAITHFUL;
use super::towns_map::{at, entrance, town, with_army};
use crate::import::codes;
use crate::writer as w;
use crate::{Limits, import, read};
use h3_adventure::{AdventureCommand, Pos};
use h3_content::ResourceId;
use h3_contracts::{HeroId, PlayerId, TownId};
use h3_game::{
    Actor, BuildingsSetup, GameCommand, GameCommandError, GameState, MainTown, NoopSink,
    TownFaction, apply, legal_moves,
};
use h3_towns::{BuildingId, SlotSet, TownCommand, TownCommandError, TownEventSetup};

/// The committed file's name.
const NAME: &str = "e09-builds-36.h3m";

/// Faction codes (`TOWNTYPE.TXT`, the writer's subclasses).
const CASTLE: u32 = 0;
const TOWER: u32 = 2;
const CONFLUX: u32 = 8;

/// Creature codes: Pikemen, Gorgons.
const PIKEMAN: u16 = 0;
const GORGON: u16 = 102;

/// Building bits (`docs/formats/h3m.md`, "Town buildings").
const CITY_HALL: u8 = 1;
const CAPITOL: u8 = 2;
const FORT: u8 = 3;
const CASTLE_BUILDING: u8 = 5;
const TAVERN: u8 = 6;
const BLACKSMITH: u8 = 7;
const MARKETPLACE: u8 = 8;
const SILO: u8 = 9;
const MAGE_GUILD_1: u8 = 11;
const SHIPYARD: u8 = 16;
/// Level 1's dwelling (a Conflux's Magic Lantern), level 2's (its Altar of Air), level 3's (its
/// Altar of Water) and level 3's upgraded one (a Castle's Upg. Griffin Tower).
const DWELLING_1: u8 = 22;
const DWELLING_2: u8 = 25;
const DWELLING_3: u8 = 28;
const UPGRADED_3: u8 = 29;
/// Level 4's dwelling (a Castle's Barracks, which the Griffin Tower requires).
const DWELLING_4: u8 = 31;

/// The towns' positions by `TownId` (each entered two columns left of it): red's Harbour by the
/// sea and Keep inland, blue's Nexus and Spire.
const TOWNS: [(u8, u8); 4] = [(10, 29), (10, 12), (30, 8), (30, 20)];
/// The heroes' tiles: red's Orrin, blue's Yog.
const HEROES: [(u8, u8); 2] = [(4, 20), (14, 14)];
/// The sea: the rows from this one down (the Harbour's entrance two rows above it).
const SEA: u8 = 31;

/// A bit set of `bits` (a custom set's or an event's).
fn bits(bits: &[u8]) -> [u8; 6] {
    let mut set = [0; 6];
    for &bit in bits {
        set[usize::from(bit / 8)] |= 1 << (bit % 8);
    }
    set
}

/// A custom set of `built` built and `forbidden` forbidden.
fn custom(built: &[u8], forbidden: &[u8]) -> w::TownBuildings {
    w::TownBuildings::Custom {
        built: bits(built),
        forbidden: bits(forbidden),
    }
}

/// An event of the players `players` (bits by colour), human and computer, from the file's
/// `first_day` (0: day 1) every `interval` days (0: once), giving `resources`.
fn event(players: u8, first_day: u16, interval: u16, resources: [i32; 7]) -> w::TownEvent {
    w::TownEvent {
        event: w::TimedEvent {
            name: String::new(),
            message: String::new(),
            resources,
            players,
            human: true,
            computer: true,
            first_day,
            interval,
        },
        buildings: [0; 6],
        creatures: [0; 7],
    }
}

/// Red's and blue's bits in an event's players.
const RED: u8 = 1;
const BLUE: u8 = 2;

/// Resources in `RESTYPES.TXT`'s order: wood, mercury, ore, sulfur, crystal, gems, gold.
const fn gift(wood: i32, ore: i32, gold: i32) -> [i32; 7] {
    [wood, 0, ore, 0, 0, 0, gold]
}

/// The map: SoD, 36 × 36 of grass with the sea's rows from [`SEA`] down, no underground; red
/// (Castle) in the west, blue (Conflux) in the east.
///
/// - Red's Orrin (hero 0) on (4, 20) with ten pikemen; blue's Yog (hero 1) on (14, 14) with five
///   Gorgons, a short walk from the Keep's entrance (8, 12).
/// - The towns ([`TOWNS`]):
///   - red's main Castle "Harbour" on (10, 29), entered from (8, 29), its boat tile (7, 31) on the
///     sea: a Fort and a Tavern; events (all red's, from day 1): 0 once, the Marketplace and the
///     Blacksmith; 1 every day, 3 wood, 3 ore, 1 000 gold; 2 every week, 5 000 gold; 3 on day 3,
///     ten and five creatures of levels 1 and 2 (no dwelling stands: lost, TWN-EVT-004);
///   - red's Castle "Keep" on (10, 12): the City Hall, the Castle, the Tavern, the Blacksmith, the
///     Marketplace and the Mage Guild's first level (the Capitol's requirements), the Guardhouse,
///     the Barracks and the Upg. Griffin Tower (an upgraded dwelling without its horde), an empty
///     garrison; events: 0 blue's
///     every day from day 1, 500 gold; 1 red's once on day 1, the Shipyard (no water: skipped,
///     TWN-EVT-003) and the Resource Silo;
///   - blue's main Conflux "Nexus" on (30, 8): the Capitol, a Fort, a Tavern, the Marketplace,
///     the Blacksmith, the Magic Lantern and the Altar of Air; the Altar of Water forbidden;
///   - blue's Tower "Spire" on (30, 20): the Capitol's requirements; event 0 blue's once
///     on day 2, the Capitol.
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
        towns: 1 << CONFLUX,
    });
    let [orrin, yog] = HEROES;
    let mut objects = vec![
        w::MapObject::Hero(with_army(orrin, 0, 0, &[(PIKEMAN, 10)])),
        w::MapObject::Hero(with_army(yog, 1, 96, &[(GORGON, 5)])),
    ];
    let [harbour, keep, nexus, spire] = TOWNS;
    let capitol_requirements = [
        CITY_HALL,
        CASTLE_BUILDING,
        TAVERN,
        BLACKSMITH,
        MARKETPLACE,
        MAGE_GUILD_1,
    ];
    let keep_set: Vec<u8> = capitol_requirements
        .iter()
        .copied()
        .chain([DWELLING_1, DWELLING_4, UPGRADED_3])
        .collect();
    let towns = [
        w::AnyTown {
            main: Some(w::MainTown {
                generates_hero: false,
            }),
            name: Some("Harbour".into()),
            buildings: custom(&[FORT, TAVERN], &[]),
            events: vec![
                w::TownEvent {
                    buildings: bits(&[MARKETPLACE, BLACKSMITH]),
                    ..event(RED, 0, 0, [0; 7])
                },
                event(RED, 0, 1, gift(3, 3, 1000)),
                event(RED, 0, 7, gift(0, 0, 5000)),
                w::TownEvent {
                    creatures: [10, 5, 0, 0, 0, 0, 0],
                    ..event(RED, 2, 0, [0; 7])
                },
            ],
            ..town(harbour, Some(CASTLE), Some(0))
        },
        w::AnyTown {
            name: Some("Keep".into()),
            garrison: Some([None; w::ARMY_SLOTS]),
            buildings: custom(&keep_set, &[]),
            events: vec![
                event(BLUE, 0, 1, gift(0, 0, 500)),
                w::TownEvent {
                    buildings: bits(&[SHIPYARD, SILO]),
                    ..event(RED, 0, 0, [0; 7])
                },
            ],
            ..town(keep, Some(CASTLE), Some(0))
        },
        w::AnyTown {
            main: Some(w::MainTown {
                generates_hero: false,
            }),
            name: Some("Nexus".into()),
            buildings: custom(
                &[
                    CAPITOL,
                    FORT,
                    TAVERN,
                    MARKETPLACE,
                    BLACKSMITH,
                    DWELLING_1,
                    DWELLING_2,
                ],
                &[DWELLING_3],
            ),
            ..town(nexus, Some(CONFLUX), Some(1))
        },
        w::AnyTown {
            name: Some("Spire".into()),
            buildings: custom(&capitol_requirements, &[]),
            events: vec![w::TownEvent {
                buildings: bits(&[CAPITOL]),
                ..event(BLUE, 1, 0, [0; 7])
            }],
            ..town(spire, Some(TOWER), Some(1))
        },
    ];
    objects.extend(towns.map(w::MapObject::AnyTown));
    w::Map {
        name: "Builds".into(),
        description: "E09 T19a: builds, the Shipyard's water, the town events".into(),
        size: 36,
        terrain: w::Terrain::Grass,
        patches: vec![w::Patch {
            terrain: w::Terrain::Water,
            from: (0, SEA),
            to: (35, 35),
        }],
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

/// The committed map, read.
fn committed() -> crate::RawMap {
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

/// The base pack's resource of `key` (`RESTYPES.TXT`'s keys, the import's reading).
fn resource(key: &str) -> ResourceId {
    let resources = FAITHFUL.core().resources();
    resources
        .iter()
        .position(|r| r.key.as_str() == key)
        .and_then(ResourceId::from_index)
        .unwrap()
}

/// The slots `slots` as a set.
fn slot_set(slots: &[u8]) -> SlotSet {
    slots
        .iter()
        .fold(SlotSet::EMPTY, |set, &slot| set.with(slot))
}

/// The building of `key` in the town record of the faction of `code` (`TOWNTYPE.TXT`'s order).
fn building(code: u32, key: &str) -> BuildingId {
    let faction = FAITHFUL
        .core()
        .faction_by_key(codes::FACTIONS[code as usize])
        .unwrap();
    let record = FAITHFUL.towns().town(faction).unwrap();
    record.building_by_key(key).unwrap()
}

/// The keys of the buildings `town` of `state` ever built (`forbidden`: forbids), in index order.
fn keys(state: &GameState, town: u16, forbidden: bool) -> Vec<String> {
    let record = state.towns().get(TownId::new(town)).unwrap();
    let part = FAITHFUL.towns().town(record.faction()).unwrap();
    let set = if forbidden {
        record.forbidden()
    } else {
        record.built()
    };
    set.iter()
        .map(|id| part.building(id).unwrap().key.clone())
        .collect()
}

/// `player`'s build of `building` in `town`.
fn build(
    state: &mut GameState,
    player: u8,
    town: u16,
    building: BuildingId,
) -> Result<(), GameCommandError> {
    let command = TownCommand::Build {
        town: TownId::new(town),
        building,
    };
    let actor = Actor::Player(PlayerId::new(player));
    apply(
        state,
        &FAITHFUL,
        actor,
        GameCommand::Town(command),
        &mut NoopSink,
    )
}

/// `player` ends its turn.
fn end_turn(state: &mut GameState, player: u8) {
    let actor = Actor::Player(PlayerId::new(player));
    apply(state, &FAITHFUL, actor, GameCommand::EndTurn, &mut NoopSink).unwrap();
}

// covers: TWN-SET-001, TWN-SET-011, TWN-EVT-001, TWN-EVT-002, TWN-EVT-003
#[test]
fn the_builds_map_is_committed_as_written() {
    let committed = committed();
    let setup = import(&committed, &FAITHFUL).unwrap();
    setup.validate(&FAITHFUL).unwrap();
    let core = FAITHFUL.core();
    let faction = |code: u32| core.faction_by_key(codes::FACTIONS[code as usize]).unwrap();
    let (red, blue) = (PlayerId::new(0), PlayerId::new(1));
    // The starts: each player's main town.
    let mains: Vec<_> = setup.players.iter().map(|p| p.start.main_town).collect();
    let main = |id| {
        Some(MainTown {
            town: TownId::new(id),
            generates_hero: false,
        })
    };
    assert_eq!(mains, [main(0), main(2)]);
    // The towns: their factions, owners, custom sets and events as stored.
    let factions: Vec<_> = setup.towns.iter().map(|t| t.faction).collect();
    let fixed = |code| TownFaction::Fixed(faction(code));
    assert_eq!(
        factions,
        [fixed(CASTLE), fixed(CASTLE), fixed(CONFLUX), fixed(TOWER)]
    );
    let adventure = FAITHFUL.adventure();
    let town_type = adventure.object_type_by_key("base:town");
    let owners: Vec<_> = setup
        .map
        .objects
        .iter()
        .filter(|o| o.object_type == town_type)
        .map(|o| (o.anchor, o.owner))
        .collect();
    let tiles = TOWNS.map(at);
    assert_eq!(
        owners,
        [
            (tiles[0], Some(red)),
            (tiles[1], Some(red)),
            (tiles[2], Some(blue)),
            (tiles[3], Some(blue)),
        ]
    );
    let sets: Vec<_> = setup
        .towns
        .iter()
        .map(|t| match t.buildings {
            BuildingsSetup::Custom { built, forbidden } => (built, forbidden),
            BuildingsSetup::Standard { .. } => panic!("a custom set"),
        })
        .collect();
    let requirements = [
        CITY_HALL,
        CASTLE_BUILDING,
        TAVERN,
        BLACKSMITH,
        MARKETPLACE,
        MAGE_GUILD_1,
    ];
    let with_dwelling: Vec<u8> = requirements
        .iter()
        .copied()
        .chain([DWELLING_1, DWELLING_4, UPGRADED_3])
        .collect();
    assert_eq!(
        sets,
        [
            (slot_set(&[FORT, TAVERN]), SlotSet::EMPTY),
            (slot_set(&with_dwelling), SlotSet::EMPTY),
            (
                slot_set(&[
                    CAPITOL,
                    FORT,
                    TAVERN,
                    MARKETPLACE,
                    BLACKSMITH,
                    DWELLING_1,
                    DWELLING_2
                ]),
                slot_set(&[DWELLING_3]),
            ),
            (slot_set(&requirements), SlotSet::EMPTY),
        ]
    );
    let names: Vec<_> = setup.towns.iter().map(|t| t.name.as_deref()).collect();
    assert_eq!(
        names,
        [Some("Harbour"), Some("Keep"), Some("Nexus"), Some("Spire")]
    );
    let (wood, ore, gold) = (
        resource("base:wood"),
        resource("base:ore"),
        resource("base:gold"),
    );
    let event = |players: Vec<PlayerId>, first_day, interval, resources| TownEventSetup {
        players,
        human: true,
        computer: true,
        first_day,
        interval,
        resources,
        buildings: SlotSet::EMPTY,
        creatures: [0; 7],
    };
    assert_eq!(
        setup.towns[0].events,
        [
            TownEventSetup {
                buildings: slot_set(&[MARKETPLACE, BLACKSMITH]),
                ..event(vec![red], 1, 0, Vec::new())
            },
            event(vec![red], 1, 1, vec![(wood, 3), (ore, 3), (gold, 1000)]),
            event(vec![red], 1, 7, vec![(gold, 5000)]),
            TownEventSetup {
                creatures: [10, 5, 0, 0, 0, 0, 0],
                ..event(vec![red], 3, 0, Vec::new())
            },
        ]
    );
    assert_eq!(
        setup.towns[1].events,
        [
            event(vec![blue], 1, 1, vec![(gold, 500)]),
            TownEventSetup {
                buildings: slot_set(&[SHIPYARD, SILO]),
                ..event(vec![red], 1, 0, Vec::new())
            },
        ]
    );
    assert!(setup.towns[2].events.is_empty());
    assert_eq!(
        setup.towns[3].events,
        [TownEventSetup {
            buildings: slot_set(&[CAPITOL]),
            ..event(vec![blue], 2, 0, Vec::new())
        }]
    );
    // The game built with seed 1 (the scripted replays'): only the Harbour's boat tile is water,
    // so the three others forbid their Shipyards (the Tower has none, TWN-SET-011); red's day-1
    // events fired as the game was built, the Harbour's first three and the Keep's second, with
    // their builds (the Keep's Shipyard skipped, TWN-EVT-003) and gains (TWN-EVT-002): red's
    // treasury the difficulty's 20 wood, 20 ore and 20 000 gold, 3 wood, 3 ore and 6 000 gold
    // more; blue's none.
    let state = GameState::new(&FAITHFUL, &setup, 1).unwrap();
    assert_eq!(
        state.town_events_at_start(red),
        [
            (TownId::new(0), 0),
            (TownId::new(0), 1),
            (TownId::new(0), 2),
            (TownId::new(1), 1)
        ]
    );
    assert!(state.town_events_at_start(blue).is_empty());
    assert_eq!(
        state.treasury(red),
        Some(&[23, 10, 23, 10, 10, 10, 26_000][..])
    );
    assert_eq!(
        state.treasury(blue),
        Some(&[20, 10, 20, 10, 10, 10, 20_000][..])
    );
    let built = |town| keys(&state, town, false);
    let forbidden = |town| keys(&state, town, true);
    assert_eq!(
        built(0),
        [
            "tavern",
            "fort",
            "village_hall",
            "marketplace",
            "blacksmith"
        ]
    );
    assert!(forbidden(0).is_empty());
    assert!(built(1).contains(&"resource_silo".to_owned()));
    assert!(!built(1).contains(&"shipyard".to_owned()));
    assert_eq!(forbidden(1), ["shipyard"]);
    assert_eq!(forbidden(2), ["shipyard", "altar_of_water"]);
    assert!(forbidden(3).is_empty());
    let spot = |id| state.adventure().map().town_boat_spot(TownId::new(id));
    assert_eq!(
        (spot(0), spot(1), spot(2), spot(3)),
        (Some(Pos::new(7, SEA.into(), 0)), None, None, None)
    );
}

/// Blue's Yog's path to `to` in `state` (blue's turn), within what blue has explored.
fn yog_path(state: &GameState, to: Pos) -> Vec<Pos> {
    let blue = Actor::Player(PlayerId::new(1));
    let reach = legal_moves(state, &FAITHFUL, blue, HeroId::new(1)).unwrap();
    reach.path(to).unwrap()
}

/// Blue's Yog's walk to the Keep's entrance on day 1 (blue's turn): to (9, 13), where the
/// explored area ends, then onto the entrance, which captures the Keep undefended.
fn yog_takes_the_keep(state: &mut GameState) -> [Vec<Pos>; 2] {
    let blue = Actor::Player(PlayerId::new(1));
    let walk = |state: &mut GameState, to| {
        let path = yog_path(state, to);
        let command = AdventureCommand::MoveHero {
            hero: HeroId::new(1),
            path: path.clone(),
        };
        apply(
            state,
            &FAITHFUL,
            blue,
            GameCommand::Adventure(command),
            &mut NoopSink,
        )
        .unwrap();
        path
    };
    let first = walk(state, at((9, 13)));
    [first, walk(state, entrance(TOWNS[1]))]
}

// covers: TWN-BLD-001, TWN-BLD-002, TWN-BLD-003, TWN-BLD-004, TWN-EVT-003, TWN-CAP-001
#[test]
fn each_refused_build_on_the_builds_map_changes_nothing() {
    // E09 T19a (design note §32): every refusal kind once, on red's and blue's first turns, each
    // leaving the state hash as it was; a golden replay holds no refused command, so this test
    // keeps them (the replays `builds-*` the accepted builds).
    let setup = import(&committed(), &FAITHFUL).unwrap();
    let mut state = GameState::new(&FAITHFUL, &setup, 1).unwrap();
    let refused = |state: &mut GameState, (player, town), building, error| {
        let hash = state.state_hash();
        assert_eq!(build(state, player, town, building), Err(error), "{town}");
        assert_eq!(state.state_hash(), hash);
    };
    let (castle, tower, conflux) = (
        |key| building(CASTLE, key),
        |key| building(TOWER, key),
        |key| building(CONFLUX, key),
    );
    let town = GameCommandError::Town;
    let (harbour, keep, nexus, spire) = ((0, 0), (0, 1), (0, 2), (1, 3));
    let tavern = castle("tavern");
    // The game's: blue out of its turn, a town not in the store, another player's.
    refused(
        &mut state,
        (1, 2),
        tavern,
        GameCommandError::NotYourTurn(PlayerId::new(1)),
    );
    refused(
        &mut state,
        (0, 4),
        tavern,
        town(TownCommandError::UnknownTown),
    );
    refused(&mut state, nexus, tavern, town(TownCommandError::NotOwner));
    // The hall's, in its order: not of the record, ever built, forbidden (no water), the Grail's,
    // a requirement never built, the horde offered as its upgrade.
    let past = BuildingId::from_index(63).unwrap();
    refused(
        &mut state,
        harbour,
        past,
        town(TownCommandError::UnknownBuilding),
    );
    refused(&mut state, harbour, tavern, town(TownCommandError::Built));
    let shipyard = castle("shipyard");
    refused(
        &mut state,
        keep,
        shipyard,
        town(TownCommandError::Forbidden),
    );
    let colossus = castle("colossus");
    refused(&mut state, harbour, colossus, town(TownCommandError::Grail));
    let city_hall = castle("city_hall");
    refused(
        &mut state,
        harbour,
        city_hall,
        town(TownCommandError::Requirements),
    );
    let upgraded = castle("upg_griffin_bastion");
    refused(
        &mut state,
        keep,
        castle("griffin_bastion"),
        town(TownCommandError::OfferedAs(upgraded)),
    );
    // The Harbour's Shipyard (its water) takes 20 of red's 23 wood: a second build there today
    // is refused by the mark, the Keep's Mage Guild's second level (5 wood) by the treasury.
    build(&mut state, 0, 0, shipyard).unwrap();
    refused(
        &mut state,
        harbour,
        castle("lighthouse"),
        town(TownCommandError::BuiltToday),
    );
    refused(
        &mut state,
        keep,
        castle("mage_guild_level_2"),
        GameCommandError::CannotPay,
    );
    // Blue's Spire has every requirement of the Capitol, but blue holds the Nexus' (TWN-BLD-002).
    end_turn(&mut state, 0);
    let spire_capitol = tower("capitol");
    refused(
        &mut state,
        spire,
        spire_capitol,
        town(TownCommandError::Capitol),
    );
    assert!(
        !state
            .town_offer(&FAITHFUL, PlayerId::new(1), TownId::new(3))
            .unwrap()
            .possible
            .contains(spire_capitol)
    );
    // The Nexus forbids its Altar of Water, so the Altar of Earth it requires is never possible.
    let offer = state
        .town_offer(&FAITHFUL, PlayerId::new(1), TownId::new(2))
        .unwrap();
    assert!(!offer.possible.contains(conflux("altar_of_earth")));
    assert!(offer.possible.contains(conflux("altar_of_fire")));
    // On day 2 the Spire's event builds its Capitol at blue's turn start, beside the Nexus' (T-8:
    // no hall's test, TWN-EVT-003): blue holds two.
    end_turn(&mut state, 1);
    end_turn(&mut state, 0);
    let capitols = |state: &GameState| {
        [2, 3].map(|id| state.towns().capitol(TownId::new(id), FAITHFUL.towns()))
    };
    assert_eq!(capitols(&state), [true, true]);
    assert_eq!(
        state.town_events_at_start(PlayerId::new(1)),
        Vec::<(TownId, u16)>::new()
    );
}

// covers: TWN-BLD-004, TWN-CAP-001, TWN-CAP-002, TWN-EVT-001
#[test]
fn a_town_built_in_today_and_taken_keeps_its_mark() {
    // The golden replay `builds-capture-s1`: red builds the Keep's Capitol on day 1 (red holds
    // none), blue's Yog takes the Keep undefended the same day. Both players are human, so the
    // mark is kept (TWN-BLD-004) and blue cannot build there today; blue holds the Nexus' Capitol,
    // so the Keep's falls to its City Hall (TWN-CAP-002). On day 2 the Keep's event of blue's
    // fires for its new owner (TWN-EVT-001), after the map's events and in `TownId` order, and
    // blue builds there.
    let setup = import(&committed(), &FAITHFUL).unwrap();
    let mut state = GameState::new(&FAITHFUL, &setup, 1).unwrap();
    let part = FAITHFUL.towns();
    let keep = TownId::new(1);
    build(&mut state, 0, 1, building(CASTLE, "capitol")).unwrap();
    assert!(state.towns().capitol(keep, part));
    end_turn(&mut state, 0);
    let [first, second] = yog_takes_the_keep(&mut state);
    assert_eq!(
        (first.last(), second.as_slice()),
        (Some(&at((9, 13))), &[entrance(TOWNS[1])][..])
    );
    let adventure = state.adventure();
    let object = adventure.map().town_object(keep).unwrap();
    assert_eq!(adventure.owner(object), Some(PlayerId::new(1)));
    let town = state.towns().get(keep).unwrap();
    assert!(town.built_today() && !state.towns().capitol(keep, part));
    let guild = building(CASTLE, "mage_guild_level_2");
    let hash = state.state_hash();
    assert_eq!(
        build(&mut state, 1, 1, guild),
        Err(GameCommandError::Town(TownCommandError::BuiltToday))
    );
    assert_eq!(state.state_hash(), hash);
    end_turn(&mut state, 1);
    let mut told = Vec::new();
    let red = Actor::Player(PlayerId::new(0));
    apply(&mut state, &FAITHFUL, red, GameCommand::EndTurn, &mut told).unwrap();
    let fired: Vec<_> = told
        .iter()
        .filter_map(|e| match e {
            h3_game::GameEvent::TownEventFired {
                player,
                town,
                event,
            } => Some((player.raw(), town.raw(), *event)),
            _ => None,
        })
        .collect();
    // The Keep's event 0, then the Spire's Capitol (`TownId` order).
    assert_eq!(fired, [(1, 1, 0), (1, 3, 0)]);
    build(&mut state, 1, 1, guild).unwrap();
}
