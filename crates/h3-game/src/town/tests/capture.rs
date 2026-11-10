//! Tests of the step onto a town through the game and of the capture the game composes (E09 T8a2;
//! the town epic's design note §9 and §11): an own town entered, an undefended one captured with
//! its reveal and the Capitol's limit told where the town is seen, a defended one besieged (since
//! E09 T8b2a), the legal moves; the capture's rule of the day's mark (since E09 T18b1).

use alloc::vec::Vec;

use h3_adventure::{AdventureCommand, AdventureEventKind, Pos, Viewers};
use h3_contracts::{HeroId, PlayerId, TownId};
use h3_towns::{BuildingId, BuildingsSetup, SlotSet, TownFaction};

use super::{
    CAPITOL_SLOT, LOOKOUT_RADIUS, LOOKOUT_SLOT, faction, telling, towns_rules, towns_rules_on,
    towns_setup,
};
use crate::{
    Actor, GameCommand, GameCommandError, GameEvent, GameRuleset, GameSetup, GameState, Viewer,
    apply, legal_moves, observe, project,
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
        // explored at the start); no change of its picture; then each viewer's view of the town
        // (E09 T18b2a): red's, its new owner's, and blue's.
        let seen = |player| observe(&game, &rules, Viewer::Player(player)).towns[1].clone();
        let mut told = telling(RED, &seen(RED));
        told.extend(telling(BLUE, &seen(BLUE)));
        assert_eq!(events[3..], told, "{events:?}");
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
    assert!(matches!(
        kind(&events[1]),
        Some(AdventureEventKind::TownCaptured { town: T1, .. })
    ));
    // Told to each viewer whose view it changed (E09 T18b2a): red, its new owner, and blue, which
    // sees it, its Capitol gone.
    let seen = |player| observe(&game, &rules, Viewer::Player(player)).towns[1].clone();
    let (red, blue) = (seen(RED), seen(BLUE));
    assert!(!red.capitol && !blue.capitol);
    let mut told = telling(RED, &red);
    told.extend(telling(BLUE, &blue));
    assert_eq!(events[3..], told);
    // Its Capitol out of the ever-built set, its City Hall in; red's own keeps its Capitol.
    let towns = game.towns();
    let built = towns.get(T1).unwrap().built();
    assert!(!built.contains(capitol) && built.contains(city_hall));
    assert!(!towns.capitol(T1, rules.towns()));
    assert!(towns.capitol(T0, rules.towns()));
    // Each viewer told alone.
    for event in &events[3..] {
        let to = |player| project(event, Viewer::Player(player)) == Some(*event);
        assert_ne!(to(RED), to(BLUE), "{event:?}");
    }
}

/// Town 1's entrance on the 48 × 48 map of the capture far from blue.
const FAR_GATE: Pos = Pos::new(40, 40, 0);

// covers: TWN-CAP-001, TWN-CAP-002
#[test]
fn a_capture_reveals_the_towns_radius_and_tells_its_change_where_it_is_seen() {
    // [`at`]'s neutral town 1 on (40, 40) of a 48 × 48 map with its Capitol and a Lookout, red's
    // hero below it on (40, 41), red's town 0 on (1, 1) with its Capitol; blue's hero on (2, 2).
    let rules = towns_rules_on(false, 48);
    let mut setup = at(&rules, Pos::new(40, 41, 0), true);
    let tile = setup.map.tiles[0];
    (setup.map.width, setup.map.height) = (48, 48);
    setup.map.tiles = [tile; 48 * 48].to_vec();
    setup.map.objects[1].anchor = FAR_GATE;
    setup.towns[0].buildings = with_capitol();
    setup.towns[1].buildings = BuildingsSetup::Custom {
        built: SlotSet::EMPTY.with(CAPITOL_SLOT).with(LOOKOUT_SLOT),
        forbidden: SlotSet::EMPTY,
    };
    let mut game = GameState::new(&rules, &setup, 7).unwrap();
    let tiles = || (0..48).flat_map(|y| (0..48).map(move |x| Pos::new(x, y, 0)));
    let explored = |game: &GameState| -> Vec<bool> {
        tiles()
            .map(|pos| game.adventure().is_explored(RED, pos))
            .collect()
    };
    // ADV-FOG-002's disc of `radius` around the entrance (the tiles within `r² + r`).
    let disc = |radius: u32, pos: Pos| {
        let (dx, dy) = (pos.x.abs_diff(FAR_GATE.x), pos.y.abs_diff(FAR_GATE.y));
        u32::from(dx).pow(2) + u32::from(dy).pow(2) <= radius * radius + radius
    };
    let before = explored(&game);
    assert!(!game.adventure().is_explored(BLUE, FAR_GATE));
    let events = step(&rules, &mut game, FAR_GATE).unwrap();
    // The town's reveal, `h3_towns::reveal`: the Lookout's radius, past the step's own (the
    // hero's scouting radius 5, no Scouting) and the part's 5.
    let radius = u32::from(LOOKOUT_RADIUS);
    let new = tiles()
        .zip(&before)
        .filter(|&(pos, &seen)| !seen && disc(radius, pos) && !disc(5, pos))
        .count();
    assert!(new > 0);
    assert!(matches!(
        kind(&events[2]),
        Some(AdventureEventKind::TilesRevealed { count, .. }) if count == u32::try_from(new).unwrap()
    ));
    for ((pos, seen), now) in tiles().zip(before).zip(explored(&game)) {
        assert_eq!(now, seen || disc(radius, pos), "{pos:?}");
    }
    // Blue, which does not see the town, is told neither the capture nor its change.
    let alone = Viewers::NONE.with(RED);
    let GameEvent::Adventure(captured) = events[1] else {
        panic!("{events:?}");
    };
    assert_eq!(captured.seen_by, alone);
    let red = &observe(&game, &rules, Viewer::Player(RED)).towns;
    let town = red.iter().find(|t| t.town == T1).unwrap();
    assert!(!town.capitol);
    assert_eq!(events[3..], telling(RED, town));
    for event in [&events[1]].into_iter().chain(&events[3..]) {
        assert_eq!(project(event, Viewer::Player(RED)), Some(*event));
        assert_eq!(project(event, Viewer::Player(BLUE)), None);
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
        // The step's, `TownCaptured`, `TilesRevealed`, then each viewer's view (E09 T18b2a), its
        // Capitol kept.
        assert_eq!(game.adventure().owner(1), Some(RED));
        assert!(game.towns().capitol(T1, rules.towns()));
        let capitols: Vec<bool> = events[3..]
            .iter()
            .filter_map(|e| match *e {
                GameEvent::TownChanged { capitol, .. } => Some(capitol),
                _ => None,
            })
            .collect();
        assert_eq!(capitols, [true, true], "{events:?}");
    }
}

// covers: TWN-CAP-001
#[test]
fn a_capture_of_a_town_the_store_lacks_changes_nothing() {
    let rules = towns_rules();
    let mut game = GameState::new(&rules, &towns_setup(&rules), 7).unwrap();
    let before = game.clone();
    let mut events = Vec::new();
    let lacked = (TownId::new(2), RED);
    crate::town::capture(&mut game, &rules, lacked, &mut events);
    assert!(events.is_empty());
    assert_eq!(game.state_hash(), before.state_hash());
    assert_eq!(
        (game.adventure(), game.towns()),
        (before.adventure(), before.towns())
    );
}

// covers: TWN-BLD-004
#[test]
fn a_capture_keeps_the_day_s_mark_unless_a_human_takes_it_from_no_human() {
    // E09 decision 43 (survey `E09-construction.md` §3, `town_change_owner` 0x4C63B8): red takes
    // town 1, built in today, from blue or from no owner; the mark is cleared only when the old
    // owner is not human (or none) and red is.
    let rules = towns_rules();
    let fort = BuildingId::from_index(4).unwrap();
    let cases = [
        // (town 1 neutral, blue human, red human) → the mark kept
        (false, true, true, true),
        (false, false, true, false),
        (true, true, true, false),
        (false, true, false, true),
        (false, false, false, true),
    ];
    for (neutral, blue_human, red_human, kept) in cases {
        let mut setup = towns_setup(&rules);
        if neutral {
            setup.map.objects[1].owner = None;
            setup.towns[1] = super::town(TownFaction::Fixed(faction(&rules, "t:town")));
        }
        setup.players[0].human = red_human;
        setup.players[1].human = blue_human;
        let mut game = GameState::new(&rules, &setup, 7).unwrap();
        // Town 1 built in today (its Fort already stands: the build only sets the mark).
        game.towns_mut()
            .build(T1, fort, rules.towns(), true)
            .unwrap();
        let mut events = Vec::new();
        crate::town::capture(&mut game, &rules, (T1, RED), &mut events);
        assert_eq!(game.adventure().owner(1), Some(RED));
        let case = (neutral, blue_human, red_human);
        assert_eq!(
            game.towns().get(T1).unwrap().built_today(),
            kept,
            "{case:?}"
        );
        // The mark is public: each viewer is told it as it stands (E09 T18b2a, TWN-VEW-004).
        let marks: Vec<bool> = events
            .iter()
            .filter_map(|e| match *e {
                GameEvent::TownChanged { built_today, .. } => Some(built_today),
                _ => None,
            })
            .collect();
        assert_eq!(marks, [kept, kept], "{case:?}");
    }
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

// covers: TWN-VIS-001, TWN-SIE-001
#[test]
fn a_defended_town_is_besieged_not_captured() {
    // A neutral town with its drawn garrison: the step opens the siege against the neutral seat
    // (E09 T8b2a; the siege's own tests in `siege.rs`).
    let rules = towns_rules();
    let mut setup = at(&rules, Pos::new(3, 1, 0), true);
    setup.towns[1].garrison = None;
    let mut game = GameState::new(&rules, &setup, 7).unwrap();
    assert!(!game.adventure().town_garrison(1).unwrap().is_empty());
    step(&rules, &mut game, BLUE_GATE).unwrap();
    let battle = game.battle(RED).unwrap();
    assert_eq!(battle.owners(), [Some(RED), None]);
    assert_eq!(battle.tile(), BLUE_GATE);
    assert_eq!(game.adventure().owner(1), None);
    assert_eq!(game.adventure().hero(HERO).unwrap().pos, Pos::new(3, 1, 0));
}

// covers: TWN-VIS-001
#[test]
fn the_legal_moves_reach_a_towns_entrance_defended_or_not() {
    // An own or undefended town entered, a defended one visited from the tile before: each a
    // destination.
    let rules = towns_rules();
    let actor = Actor::Player(RED);
    let mut defended = at(&rules, Pos::new(2, 1, 0), true);
    defended.towns[1].garrison = None;
    for setup in [
        at(&rules, Pos::new(2, 1, 0), false),
        at(&rules, Pos::new(2, 1, 0), true),
        defended,
    ] {
        let game = GameState::new(&rules, &setup, 7).unwrap();
        let moves = legal_moves(&game, &rules, actor, HERO).unwrap();
        assert_eq!(moves.path(RED_GATE), Some([RED_GATE].to_vec()));
        assert_eq!(moves.path(BLUE_GATE), Some([BLUE_GATE].to_vec()));
    }
}
