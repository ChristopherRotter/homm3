//! Tests of the step onto a town through the game and of the capture the game composes (E09 T8a2;
//! the town epic's design note §9 and §11): an own town entered, an undefended one captured with
//! its reveal and the Capitol's limit, a defended one refused, the legal moves.

use alloc::vec::Vec;

use h3_adventure::{AdventureCommand, AdventureCommandError, AdventureEventKind, Pos};
use h3_contracts::{HeroId, PlayerId, TownId};
use h3_towns::{BuildingsSetup, SlotSet, TownFaction};

use super::{CAPITOL_SLOT, faction, towns_rules, towns_setup};
use crate::{
    Actor, GameCommand, GameCommandError, GameEvent, GameRuleset, GameSetup, GameState, Viewer,
    apply, legal_moves, project,
};

const RED: PlayerId = PlayerId::new(0);
const BLUE: PlayerId = PlayerId::new(1);
const T0: TownId = TownId::new(0);
const T1: TownId = TownId::new(1);
const HERO: HeroId = HeroId::new(0);
/// [`towns_setup`]'s red town 0's entrance, its object 0.
const RED_GATE: Pos = Pos::new(1, 1, 0);
/// [`towns_setup`]'s blue town 1's entrance, its object 1.
const BLUE_GATE: Pos = Pos::new(3, 0, 0);

/// [`towns_setup`] with red's hero on `at`; town 1 neutral, of `t:town` and holding an empty
/// garrison (undefended) when `neutral`.
fn at(rules: &GameRuleset, at: Pos, neutral: bool) -> GameSetup {
    let mut setup = towns_setup(rules);
    setup.heroes[0].pos = at;
    if neutral {
        setup.map.objects[1].owner = None;
        setup.towns[1] = super::town(TownFaction::Fixed(faction(rules, "t:town")));
        setup.towns[1].garrison = Some(Vec::new());
    }
    setup
}

/// The town's custom set with its Capitol built (and the Village Hall, always).
fn with_capitol() -> BuildingsSetup {
    BuildingsSetup::Custom {
        built: SlotSet::EMPTY.with(CAPITOL_SLOT),
        forbidden: SlotSet::EMPTY,
    }
}

/// Red's hero stepped onto `to`: the game's events, or its error with the state unchanged.
fn step(
    rules: &GameRuleset,
    game: &mut GameState,
    to: Pos,
) -> Result<Vec<GameEvent>, GameCommandError> {
    let before = game.state_hash();
    let command = GameCommand::Adventure(AdventureCommand::MoveHero {
        hero: HERO,
        path: [to].to_vec(),
    });
    let mut events = Vec::new();
    let result = apply(game, rules, Actor::Player(RED), command, &mut events);
    if result.is_err() {
        assert!(events.is_empty());
        assert_eq!(game.state_hash(), before);
    }
    result.map(|()| events)
}

/// The adventure event's kind, if `event` is one.
fn kind(event: &GameEvent) -> Option<AdventureEventKind> {
    match event {
        GameEvent::Adventure(event) => Some(event.kind),
        _ => None,
    }
}

// covers: TWN-VIS-001, TWN-CAP-001
#[test]
fn an_undefended_town_of_another_player_or_none_is_captured_by_the_step() {
    let rules = towns_rules();
    for neutral in [true, false] {
        let mut game = GameState::new(&rules, &at(&rules, Pos::new(3, 1, 0), neutral), 7).unwrap();
        let owner = (!neutral).then_some(BLUE);
        assert_eq!(game.adventure().owner(1), owner);
        let events = step(&rules, &mut game, BLUE_GATE).unwrap();
        // The step's, then the capture's: the owner, then its reveal (the 4 × 4 map was all
        // explored at the start); no change of its picture.
        assert_eq!(events.len(), 3, "{events:?}");
        assert!(matches!(
            kind(&events[0]),
            Some(AdventureEventKind::HeroStepped { hero: HERO, .. })
        ));
        assert_eq!(
            kind(&events[1]),
            Some(AdventureEventKind::TownCaptured {
                town: T1,
                anchor: BLUE_GATE,
                owner: RED,
            })
        );
        assert!(matches!(
            kind(&events[2]),
            Some(AdventureEventKind::TilesRevealed { count: 0, .. })
        ));
        let adventure = game.adventure();
        assert_eq!(adventure.owner(1), Some(RED));
        assert_eq!(adventure.town_garrison(1), Some(&[][..]));
        assert_eq!(adventure.town_visitor(T1), Some(HERO));
        // Red's own town stays red's.
        assert_eq!(adventure.owner(0), Some(RED));
    }
}

// covers: TWN-CAP-002
#[test]
fn a_captured_capitol_is_demoted_when_its_new_owner_holds_another() {
    let rules = towns_rules();
    let castle = faction(&rules, "t:town");
    let record = rules.towns().town(castle).unwrap();
    let [city_hall, capitol] = ["b2", "b3"].map(|k| record.building_by_key(k).unwrap());
    let mut setup = at(&rules, Pos::new(3, 1, 0), false);
    setup.towns[0].buildings = with_capitol();
    setup.towns[1].buildings = with_capitol();
    let mut game = GameState::new(&rules, &setup, 7).unwrap();
    // The start never applies the limit: two Capitols.
    for town in [T0, T1] {
        assert!(game.towns().capitol(town, rules.towns()));
    }
    let events = step(&rules, &mut game, BLUE_GATE).unwrap();
    assert_eq!(events.len(), 4, "{events:?}");
    assert!(matches!(
        kind(&events[1]),
        Some(AdventureEventKind::TownCaptured { town: T1, .. })
    ));
    assert_eq!(
        events[3],
        GameEvent::TownChanged {
            town: T1,
            capitol: false,
        }
    );
    // Its Capitol out of the ever-built set, its City Hall in; red's own keeps its Capitol.
    let towns = game.towns();
    let built = towns.get(T1).unwrap().built();
    assert!(!built.contains(capitol) && built.contains(city_hall));
    assert!(!towns.capitol(T1, rules.towns()));
    assert!(towns.capitol(T0, rules.towns()));
    // Public, as the town's picture (until E09 T8c).
    for player in [RED, BLUE] {
        assert_eq!(project(&events[3], Viewer::Player(player)), Some(events[3]));
    }
}

// covers: TWN-CAP-002
#[test]
fn a_captured_capitol_stays_when_its_new_owner_holds_no_other() {
    let rules = towns_rules();
    // Red's own town without a Capitol; then with one but neutral (red holds it no more).
    let mut plain = at(&rules, Pos::new(3, 1, 0), false);
    plain.towns[1].buildings = with_capitol();
    let mut lost = plain.clone();
    lost.towns[0].buildings = with_capitol();
    lost.towns[0].garrison = Some(Vec::new());
    lost.map.objects[0].owner = None;
    lost.players[0].start.main_town = None;
    for setup in [plain, lost] {
        let mut game = GameState::new(&rules, &setup, 7).unwrap();
        let events = step(&rules, &mut game, BLUE_GATE).unwrap();
        // The step's, `TownCaptured`, `TilesRevealed`: no `TownChanged`.
        assert_eq!(events.len(), 3, "{events:?}");
        assert_eq!(game.adventure().owner(1), Some(RED));
        assert!(game.towns().capitol(T1, rules.towns()));
    }
}

// covers: TWN-CAP-001
#[test]
fn a_capture_of_a_town_the_store_lacks_changes_nothing() {
    let rules = towns_rules();
    let game = GameState::new(&rules, &towns_setup(&rules), 7).unwrap();
    let (mut adventure, mut towns) = (game.adventure().clone(), game.towns().clone());
    let mut events = Vec::new();
    let lacked = (TownId::new(2), RED);
    crate::town::capture(&mut adventure, &mut towns, &rules, lacked, &mut events);
    assert!(events.is_empty());
    assert_eq!((&adventure, &towns), (game.adventure(), game.towns()));
}

// covers: TWN-VIS-001
#[test]
fn an_own_town_is_entered_without_a_capture() {
    let rules = towns_rules();
    let mut game = GameState::new(&rules, &at(&rules, Pos::new(1, 2, 0), false), 7).unwrap();
    let towns = game.towns().clone();
    let events = step(&rules, &mut game, RED_GATE).unwrap();
    assert_eq!(events.len(), 1, "{events:?}");
    assert!(matches!(
        kind(&events[0]),
        Some(AdventureEventKind::HeroStepped { hero: HERO, .. })
    ));
    assert_eq!(game.adventure().owner(0), Some(RED));
    assert_eq!(game.adventure().town_visitor(T0), Some(HERO));
    assert_eq!(game.towns(), &towns);
}

// covers: TWN-VIS-001
#[test]
fn a_defended_town_is_refused_until_the_siege() {
    // `TWN-STUB-7`: a neutral town with its drawn garrison.
    let rules = towns_rules();
    let mut setup = at(&rules, Pos::new(3, 1, 0), true);
    setup.towns[1].garrison = None;
    let mut game = GameState::new(&rules, &setup, 7).unwrap();
    assert!(!game.adventure().town_garrison(1).unwrap().is_empty());
    assert_eq!(
        step(&rules, &mut game, BLUE_GATE),
        Err(GameCommandError::Adventure(AdventureCommandError::Visit {
            step: 0
        }))
    );
    assert_eq!(game.adventure().owner(1), None);
}

// covers: TWN-VIS-001
#[test]
fn the_legal_moves_enter_an_own_or_undefended_town_not_a_defended_one() {
    let rules = towns_rules();
    let actor = Actor::Player(RED);
    let mut defended = at(&rules, Pos::new(2, 1, 0), true);
    defended.towns[1].garrison = None;
    for (setup, open) in [
        (at(&rules, Pos::new(2, 1, 0), false), true),
        (at(&rules, Pos::new(2, 1, 0), true), true),
        (defended, false),
    ] {
        let game = GameState::new(&rules, &setup, 7).unwrap();
        let moves = legal_moves(&game, &rules, actor, HERO).unwrap();
        assert_eq!(moves.path(RED_GATE), Some([RED_GATE].to_vec()));
        assert_eq!(moves.path(BLUE_GATE).is_some(), open);
    }
}
