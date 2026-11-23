//! Tests of the prisoners (E08 T141a; increment 15's design note §15, §16; survey `E08-spells.md`
//! §2): each built into the store as a placed hero in no list (GAM-SET-001, ADV-PRI-001), its
//! type its own; a visit frees it below the most heroes, finds the player full, or empties a
//! Prison whose prisoner is no prisoner (ADV-PRI-002); each player's list starts with its placed
//! heroes by type, then its main town's hero (GAM-SET-004, GAM-HER-003).

use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

use h3_adventure::{AdventureEventKind, Pos, Resolved};
use h3_content::HeroTypeId;
use h3_contracts::{HeroId, PlayerId};

use crate::GameRuleset;
use crate::command::{Actor, GameCommand, GameEvent, apply};
use crate::observe::{Viewer, observe};
use crate::setup::{GamePrisonerSetup, GameSetup, GameSetupError, HeroArmy, HeroDetailsSetup};
use crate::state::GameState;
use crate::tests::{held, imprison};
use crate::town::pool::HeroStatus;
use crate::town::tests::{towns_rules, towns_setup};

const RED: PlayerId = PlayerId::new(0);
const BLUE: PlayerId = PlayerId::new(1);
/// The prisoner: after the two placed heroes.
const PRISONER: HeroId = HeroId::new(2);
/// The Prison's tile: red's hero 0 reaches it from the side, from (1, 3) (a Prison is not
/// visited from above).
const PRISON: Pos = Pos::new(0, 3, 0);
/// Where the visitor stands.
const BESIDE: Pos = Pos::new(1, 3, 0);

/// The hero type of key `k`.
fn hero_type(rules: &GameRuleset, k: &str) -> HeroTypeId {
    rules.core().hero_type_by_key(k).unwrap()
}

/// The towns' setup with a Prison on [`PRISON`] holding a sage with its starting army and 5 000
/// experience.
fn prison_setup(rules: &GameRuleset) -> GameSetup {
    let mut setup = towns_setup(rules);
    let sage = GamePrisonerSetup {
        details: HeroDetailsSetup {
            experience: Some(5000),
            ..HeroDetailsSetup::default()
        },
        ..GamePrisonerSetup::of_type(hero_type(rules, "t:sage"))
    };
    imprison(rules, &mut setup, (0, 3), sage);
    setup
}

/// Red's hero 0 walks from (0, 0) down to [`BESIDE`] and visits the Prison: the command's
/// events.
fn visit(state: &mut GameState, rules: &GameRuleset) -> Vec<GameEvent> {
    let command = GameCommand::Adventure(h3_adventure::AdventureCommand::MoveHero {
        hero: HeroId::new(0),
        path: vec![Pos::new(0, 1, 0), Pos::new(0, 2, 0), BESIDE, PRISON],
    });
    let mut events = Vec::new();
    apply(state, rules, Actor::Player(RED), command, &mut events).unwrap();
    events
}

/// The events of `events` that are no adventure event (a movement's change is one).
fn game_events(events: &[GameEvent]) -> Vec<&GameEvent> {
    events
        .iter()
        .filter(|e| !matches!(e, GameEvent::Adventure(_)))
        .collect()
}

// covers: GAM-SET-001, GAM-SET-002, ADV-PRI-001
#[test]
fn the_build_stores_each_prisoner_as_a_placed_hero_in_no_list() {
    let rules = towns_rules();
    let game = GameState::new(&rules, &prison_setup(&rules), 3).unwrap();
    let sage = hero_type(&rules, "t:sage");
    // In the store after the two placed heroes, its experience's levels drawn at load.
    let details = game.hero(PRISONER).unwrap();
    assert_eq!((details.hero_type, details.experience), (sage, 5000));
    assert!(details.level > 1, "{}", details.level);
    // Its type a prisoner's, its record its own.
    assert_eq!(game.pool().status(sage), Some(HeroStatus::Prisoner));
    assert_eq!(game.pool().record(sage), Some(PRISONER));
    // Held by the Prison with its type's starting army (one Imp), off the map, in no list.
    let prison = &game.adventure().map().objects()[2];
    let Resolved::Prisoner { hero, army, .. } = &prison.resolved else {
        panic!("{:?}", prison.resolved);
    };
    assert_eq!((*hero, army.len(), army[0].count), (PRISONER, 1, 1));
    assert!(game.adventure().hero(PRISONER).is_none());
    assert_eq!(game.adventure().hero_list(RED), [HeroId::new(0)]);
    assert_eq!(game.adventure().hero_list(BLUE), [HeroId::new(1)]);
    let seen = observe(&game, &rules, Viewer::Player(RED));
    assert_eq!(seen.hero_list, [HeroId::new(0)]);
    assert!(!seen.heroes.iter().any(|info| info.id == PRISONER));
}

// covers: GAM-SET-001
#[test]
fn a_prisoners_type_is_the_rulesets_and_no_other_heros() {
    let rules = towns_rules();
    let refused = |edit: &dyn Fn(&mut GameSetup)| {
        let mut setup = prison_setup(&rules);
        edit(&mut setup);
        setup.validate(&rules).err()
    };
    let unknown = HeroTypeId::from_raw(999);
    let placed = towns_setup(&rules).heroes[0].hero_type.unwrap();
    assert_eq!(
        refused(&|s| s.prisoners[0].hero_type = unknown),
        Some(GameSetupError::PrisonerType(0))
    );
    assert_eq!(
        refused(&|s| s.prisoners[0].hero_type = placed),
        Some(GameSetupError::PrisonerType(0))
    );
    let twice = |s: &mut GameSetup| {
        let again = s.prisoners[0].clone();
        imprison(&rules, s, (3, 1), again);
    };
    assert_eq!(refused(&twice), Some(GameSetupError::PrisonerType(1)));
    assert_eq!(
        GameSetupError::PrisonerType(1).to_string(),
        "prisoner 1's hero type is unknown or taken"
    );
    // A custom army, as a placed hero's.
    let mut setup = prison_setup(&rules);
    setup.prisoners[0] = held(hero_type(&rules, "t:sage"));
    assert!(matches!(setup.prisoners[0].army, HeroArmy::Custom(_)));
    GameState::new(&rules, &setup, 3).unwrap();
}

// covers: ADV-PRI-002, ADV-PRI-003, GAM-HER-003
#[test]
fn a_visit_frees_the_prisoner_last_in_the_visitors_owners_list() {
    let rules = towns_rules();
    let mut game = GameState::new(&rules, &prison_setup(&rules), 3).unwrap();
    let sage = hero_type(&rules, "t:sage");
    let events = visit(&mut game, &rules);
    // The Prison's removal, then the freeing with its owner's view, then the view's parts.
    let removed = events.iter().position(|e| {
        matches!(
            e,
            GameEvent::Adventure(a) if matches!(a.kind, AdventureEventKind::ObjectRemoved { .. })
        )
    });
    let freed = events
        .iter()
        .position(|e| matches!(e, GameEvent::PrisonerFreed { .. }));
    assert!(removed.unwrap() < freed.unwrap(), "{events:?}");
    let GameEvent::PrisonerFreed {
        player,
        hero,
        hero_type,
        pos,
        seen_by,
        view,
    } = events[freed.unwrap()]
    else {
        unreachable!()
    };
    assert_eq!(
        (player, hero, hero_type, pos),
        (RED, PRISONER, sage, PRISON)
    );
    assert!(seen_by.contains(RED));
    let view = view.unwrap();
    assert_eq!(
        (view.id, view.hero_type, view.experience),
        (PRISONER, sage, 5000)
    );
    let told = game_events(&events);
    assert!(matches!(
        told[1],
        GameEvent::PrisonerPartSeen { hero: PRISONER, .. }
    ));
    assert!(
        told[1..].iter().all(|e| matches!(
            e,
            GameEvent::PrisonerPartSeen { .. } | GameEvent::HiredPartSeen { .. }
        )),
        "{told:?}"
    );
    // On the Prison's tile with a full day, last in red's list; its type red's.
    let on_map = game.adventure().hero(PRISONER).unwrap();
    assert_eq!((on_map.owner, on_map.pos), (RED, PRISON));
    assert_eq!(on_map.movement, on_map.max_movement);
    assert!(on_map.movement > 0);
    assert_eq!(game.adventure().hero_list(RED), [HeroId::new(0), PRISONER]);
    assert_eq!(game.pool().status(sage), Some(HeroStatus::Taken(0)));
    assert_eq!(game.adventure().hero(HeroId::new(0)).unwrap().pos, BESIDE);
    // The owner's observation lists it; the command's points tell it.
    let seen = observe(&game, &rules, Viewer::Player(RED));
    assert_eq!(seen.hero_list, [HeroId::new(0), PRISONER]);
    assert!(events.iter().any(|e| matches!(
        e,
        GameEvent::Adventure(a)
            if matches!(a.kind, AdventureEventKind::MovementChanged { hero: PRISONER, .. })
    )));
    // Another player sees no view of it, nor its parts.
    let projected: Vec<_> = events
        .iter()
        .filter_map(|e| crate::observe::project(e, Viewer::Player(BLUE)))
        .collect();
    for event in &projected {
        if let GameEvent::PrisonerFreed { view, .. } = event {
            assert!(view.is_none());
        }
        assert!(!matches!(
            event,
            GameEvent::PrisonerPartSeen { .. } | GameEvent::HiredPartSeen { .. }
        ));
    }
}

// covers: ADV-PRI-002
#[test]
fn a_player_at_the_most_heroes_frees_nobody() {
    // The shared test rules bundle no town part: their most heroes on the map is 0.
    let rules = crate::tests::rules();
    let mut setup = crate::tests::setup();
    let loner = rules.core().hero_type_by_key("t:loner").unwrap();
    imprison(&rules, &mut setup, (0, 3), held(loner));
    let mut game = GameState::new(&rules, &setup, 3).unwrap();
    let before = game.clone();
    let events = visit(&mut game, &rules);
    assert_eq!(
        game_events(&events),
        [&GameEvent::PrisonFull {
            player: RED,
            hero: HeroId::new(0),
        }]
    );
    assert!(game.adventure().hero(PRISONER).is_none());
    assert!(!game.adventure().is_removed(2));
    assert_eq!(game.pool(), before.pool());
    assert_eq!(game.adventure().hero_list(RED), [HeroId::new(0)]);
}

// covers: ADV-PRI-002
#[test]
fn a_prison_whose_prisoner_is_no_prisoner_is_emptied() {
    let rules = towns_rules();
    let mut game = GameState::new(&rules, &prison_setup(&rules), 3).unwrap();
    let sage = hero_type(&rules, "t:sage");
    // Only a campaign's carried hero rewrites a prisoner's status: here the pool is changed.
    alloc::sync::Arc::make_mut(&mut game.pool).free_prisoner(sage, 1);
    let events = visit(&mut game, &rules);
    assert_eq!(
        game_events(&events),
        [&GameEvent::PrisonEmptied {
            player: RED,
            hero: HeroId::new(0),
        }]
    );
    assert!(game.adventure().is_removed(2));
    assert!(game.adventure().hero(PRISONER).is_none());
    assert_eq!(game.pool().status(sage), Some(HeroStatus::Taken(1)));
}

// covers: GAM-SET-004, GAM-HER-003
#[test]
fn a_players_start_list_takes_its_placed_heroes_by_type_then_its_main_towns_hero() {
    let rules = towns_rules();
    let mut setup = towns_setup(&rules);
    // Red's two placed heroes: hero 0 a sage, hero 1 of the first type (by id the other way).
    setup.heroes[0].hero_type = Some(hero_type(&rules, "t:sage"));
    setup.heroes[1].owner = RED;
    let first = HeroTypeId::from_raw(0);
    setup.heroes[1].hero_type = Some(first);
    assert!(first < hero_type(&rules, "t:sage"));
    if let Some(main) = setup.players[0].start.main_town.as_mut() {
        main.generates_hero = true;
    }
    // The main town's hero drawn from the gunner alone (its starting army fits).
    setup.allowed_heroes = vec![hero_type(&rules, "t:gunner")];
    let game = GameState::new(&rules, &setup, 3).unwrap();
    let generated = HeroId::new(2);
    assert_eq!(game.adventure().hero(generated).map(|h| h.owner), Some(RED));
    assert_eq!(
        game.adventure().hero_list(RED),
        [HeroId::new(1), HeroId::new(0), generated]
    );
    assert_eq!(
        observe(&game, &rules, Viewer::Player(RED)).hero_list,
        [HeroId::new(1), HeroId::new(0), generated]
    );
}
