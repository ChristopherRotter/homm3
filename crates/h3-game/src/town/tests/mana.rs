//! Tests of the spell points a town gives its heroes (E09 T49a1, design note `E09-design.md` §92,
//! §93; TWN-GLD-008, -009): the night's refill of a town with a guild ever built (its visitor, then
//! its garrison hero, to the maximum, never lowered, silent) and the Mana Vortex (twice the
//! maximum, once a week a town: the visit's end before its learning, every build after its
//! learning, the garrison hero first; a town's hire to the hired hero alone; the cleared flag told
//! to the owner, the week's flags in the week's views). On [`super::learn`]'s game: red's T0 on
//! (1, 1) and blue's T1 on (3, 0), each built by its setup's slots only, `t:town`'s Mana Vortex at
//! map slot 18; the part's factor 2.

use alloc::vec::Vec;

use h3_adventure::{AdventureCommand, Pos};
use h3_contracts::{HeroId, PlayerId, TownId};
use h3_heroes::HeroChange;
use h3_towns::{TownCommand, TownHeroChange};

use super::command::{BLUE, RED, T0, T1, end_turn, run};
use super::learn::{
    BLUE_HERO, ENTRANCE, GUILD_SLOT, RED_HERO, T1_ENTRANCE, TAVERN_SLOT, VORTEX_SLOT, building,
    game, level_1, next_day, rules, setup, step, taught, town_command,
};
use crate::{GameCommand, GameEvent, GameRuleset, GameState};

/// `hero`'s maximum spell points (HER-MAN-001).
fn max(rules: &GameRuleset, game: &GameState, hero: HeroId) -> u16 {
    h3_heroes::max_mana(game.hero(hero).unwrap(), rules.core())
}

fn mana(game: &GameState, hero: HeroId) -> u16 {
    game.hero(hero).unwrap().mana
}

/// The spell points `events` set, as (owner, hero, points) in order.
fn given(events: &[GameEvent]) -> Vec<(PlayerId, HeroId, u16)> {
    events
        .iter()
        .filter_map(|e| match *e {
            GameEvent::HeroChanged {
                player,
                hero,
                change: HeroChange::SetSpellPoints(points),
            } => Some((player, hero, points)),
            _ => None,
        })
        .collect()
}

/// The week flags `events` tell, as (viewer, town, flag) in order.
fn flags(events: &[GameEvent]) -> Vec<(PlayerId, TownId, bool)> {
    events
        .iter()
        .filter_map(|e| match *e {
            GameEvent::TownChanged {
                player,
                town,
                buildings: Some(buildings),
                ..
            } => Some((player, town, buildings.vortex)),
            _ => None,
        })
        .collect()
}

/// Whether `town`'s Mana Vortex may still be taken this week.
fn flag(game: &GameState, town: TownId) -> bool {
    game.towns().get(town).unwrap().vortex()
}

fn revisit() -> GameCommand {
    GameCommand::Adventure(AdventureCommand::Revisit { hero: RED_HERO })
}

fn enter() -> GameCommand {
    town_command(TownCommand::Heroes {
        town: T0,
        change: TownHeroChange::Enter,
    })
}

// covers: TWN-GLD-008, GAM-CAL-002
#[test]
fn a_night_raises_a_town_s_visitor_and_garrison_hero_to_their_maximum_silently() {
    let rules = rules();
    // Red's hero in T0's garrison; blue's T1's visitor (the guild in both, no owner tested).
    let mut setup = setup(&rules, &[GUILD_SLOT]);
    setup.heroes[1].pos = T1_ENTRANCE;
    let mut game = game(&rules, &setup);
    run(&rules, &mut game, RED, enter()).unwrap();
    assert_eq!(game.adventure().town_hero(T0).unwrap().id, RED_HERO);
    let [red_max, blue_max] = [RED_HERO, BLUE_HERO].map(|hero| max(&rules, &game, hero));
    game.set_mana(RED_HERO, 0);
    game.set_mana(BLUE_HERO, 1);
    // Nothing within the day: a turn's end is no night.
    end_turn(&rules, &mut game, RED);
    assert_eq!((mana(&game, RED_HERO), mana(&game, BLUE_HERO)), (0, 1));
    // The night: both at their maximum, told nothing.
    let events = run(&rules, &mut game, BLUE, GameCommand::EndTurn).unwrap();
    assert_eq!(given(&events), []);
    assert_eq!(
        (mana(&game, RED_HERO), mana(&game, BLUE_HERO)),
        (red_max, blue_max)
    );
    // Above the maximum the points stay.
    game.set_mana(RED_HERO, red_max + 7);
    next_day(&rules, &mut game);
    assert_eq!(mana(&game, RED_HERO), red_max + 7);
}

// covers: TWN-GLD-008
#[test]
fn a_town_without_a_guild_refills_nothing_and_a_hero_outside_a_town_neither() {
    let rules = rules();
    let mut setup = setup(&rules, &[TAVERN_SLOT]);
    setup.heroes[1].pos = T1_ENTRANCE;
    let mut game = game(&rules, &setup);
    let red_max = max(&rules, &game, RED_HERO);
    game.set_mana(RED_HERO, 0);
    next_day(&rules, &mut game);
    let regained = mana(&game, RED_HERO);
    assert!(regained < red_max, "{regained} of {red_max}");
    // With the guild, a hero beside the town is not refilled either.
    let mut setup = self::setup(&rules, &[GUILD_SLOT]);
    setup.heroes[0].pos = Pos::new(0, 1, 0);
    let mut game = self::game(&rules, &setup);
    game.set_mana(RED_HERO, 0);
    next_day(&rules, &mut game);
    assert_eq!(mana(&game, RED_HERO), regained);
}

// covers: TWN-GLD-009, TWN-VIS-001, TWN-VEW-004, GAM-CAL-002
#[test]
fn the_visit_s_end_gives_the_vortex_before_the_learning_once_a_week() {
    let rules = rules();
    let mut setup = setup(&rules, &[GUILD_SLOT, VORTEX_SLOT]);
    setup.heroes[0].pos = Pos::new(0, 1, 0);
    let mut game = game(&rules, &setup);
    let [slot, _] = level_1(&game, T0);
    let twice = 2 * max(&rules, &game, RED_HERO);
    assert!(flag(&game, T0) && flag(&game, T1));
    // The step's visit: twice the maximum, then the cleared flag told to red, then the learning.
    let events = step(&rules, &mut game, ENTRANCE);
    assert_eq!(given(&events), [(RED, RED_HERO, twice)]);
    assert_eq!(taught(&events), [(RED, RED_HERO, slot)]);
    let at = |kind: fn(&GameEvent) -> bool| events.iter().position(kind).unwrap();
    let set = at(|e| {
        matches!(
            e,
            GameEvent::HeroChanged {
                change: HeroChange::SetSpellPoints(_),
                ..
            }
        )
    });
    let told = events
        .iter()
        .rposition(|e| matches!(e, GameEvent::TownChanged { .. }))
        .unwrap();
    let learned = at(|e| {
        matches!(
            e,
            GameEvent::HeroChanged {
                change: HeroChange::LearnSpell(_),
                ..
            }
        )
    });
    assert!(set < told && told < learned, "{events:?}");
    assert_eq!(flags(&events[told..=told]), [(RED, T0, false)]);
    assert_eq!(mana(&game, RED_HERO), twice);
    assert!(!flag(&game, T0) && flag(&game, T1));
    // Once a week a town: drained, a revisit and the nights give nothing until the week's start.
    game.set_mana(RED_HERO, 0);
    assert_eq!(given(&run(&rules, &mut game, RED, revisit()).unwrap()), []);
    let mut week = Vec::new();
    for _ in 2..=8 {
        assert!(!flag(&game, T0));
        week = next_day(&rules, &mut game);
    }
    // Day 8, the week's first: the flag set again, told to red in the week's views.
    assert!(flag(&game, T0));
    assert!(flags(&week).contains(&(RED, T0, true)), "{week:?}");
    assert!(!flags(&week).iter().any(|&(viewer, _, _)| viewer == BLUE));
    // At or above twice the maximum the flag stays for the next.
    game.set_mana(RED_HERO, twice);
    assert_eq!(given(&run(&rules, &mut game, RED, revisit()).unwrap()), []);
    assert!(flag(&game, T0));
    game.set_mana(RED_HERO, twice - 1);
    let events = run(&rules, &mut game, RED, revisit()).unwrap();
    assert_eq!(given(&events), [(RED, RED_HERO, twice)]);
    assert!(!flag(&game, T0));
}

// covers: TWN-GLD-009, TWN-BLD-005
#[test]
fn a_build_gives_the_vortex_after_its_learning_to_the_garrison_hero_first() {
    let rules = rules();
    // Red's hero in T0's garrison, a hired visitor (no Vortex standing yet: the hire gives none).
    let mut game = game(&rules, &setup(&rules, &[GUILD_SLOT, TAVERN_SLOT]));
    run(&rules, &mut game, RED, enter()).unwrap();
    let hire = town_command(TownCommand::Hire { town: T0, slot: 0 });
    let events = run(&rules, &mut game, RED, hire).unwrap();
    assert_eq!(given(&events), []);
    let hired = game.adventure().town_visitor(T0).unwrap();
    let build = town_command(TownCommand::Build {
        town: T0,
        building: building(&rules, "vortex"),
    });
    let [garrison, visitor] = [RED_HERO, hired].map(|hero| 2 * max(&rules, &game, hero));
    // The Vortex's own build: the garrison hero takes it, the visitor nothing.
    let mut first = game.clone();
    let events = run(&rules, &mut first, RED, build.clone()).unwrap();
    assert_eq!(given(&events), [(RED, RED_HERO, garrison)]);
    let at = |kind: fn(&GameEvent) -> bool| events.iter().position(kind).unwrap();
    let built = at(|e| matches!(e, GameEvent::TownBuilt { .. }));
    let set = at(|e| matches!(e, GameEvent::HeroChanged { .. }));
    assert!(built < set, "{events:?}");
    // The build's views, then the cleared flag's.
    assert_eq!(flags(&events[set..]), [(RED, T0, false)]);
    assert!(!flag(&first, T0));
    // A garrison hero at twice its maximum leaves it to the visitor.
    game.set_mana(RED_HERO, garrison);
    let events = run(&rules, &mut game, RED, build).unwrap();
    assert_eq!(given(&events), [(RED, hired, visitor)]);
    assert_eq!(mana(&game, RED_HERO), garrison);
}

// covers: TWN-GLD-009, TWN-TAV-003, TWN-ROW-002
#[test]
fn a_hire_gives_the_vortex_to_the_hired_hero_alone_a_move_between_the_bars_nothing() {
    let rules = rules();
    let mut game = game(
        &rules,
        &setup(&rules, &[GUILD_SLOT, TAVERN_SLOT, VORTEX_SLOT]),
    );
    // Into the garrison, drained: a move between the bars gives nothing.
    game.set_mana(RED_HERO, 0);
    let events = run(&rules, &mut game, RED, enter()).unwrap();
    assert_eq!(given(&events), []);
    assert!(flag(&game, T0));
    let hire = town_command(TownCommand::Hire { town: T0, slot: 0 });
    let events = run(&rules, &mut game, RED, hire).unwrap();
    let hired = game.adventure().town_visitor(T0).unwrap();
    let twice = 2 * max(&rules, &game, hired);
    assert_eq!(given(&events), [(RED, hired, twice)]);
    assert_eq!(mana(&game, RED_HERO), 0);
    assert!(!flag(&game, T0));
}
