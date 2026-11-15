//! Tests of a visit of another team to a town its garrison hero holds (E09 T28b2; the town epic's
//! design note §54; survey `E09-towns.md` §9.1's first and third rows, §9.3): alone, the garrison
//! hero defends the town's siege with its own army, no merge, and as the winner keeps its
//! survivors in the garrison, else leaves the game for the capture; beside a visitor the
//! visitor's battle at the gate is no siege, and the town stands. T-5's garrison hero without a
//! stack: `super::super::rows`' `twn_quirk_11_…`.

use alloc::vec;
use alloc::vec::Vec;

use h3_adventure::{
    AdventureCommand, AdventureEvent, AdventureEventKind, BattleStart, Combatant, Defender, Pos,
    TownHeroMove, Viewers,
};
use h3_battle::Command;
use h3_contracts::{ArmyStack, HeroId};

use super::super::command::end_turn;
use super::{
    BLUE, BLUE_GATE, BLUE_HERO, HERO, OUTSIDE, RED, ROCK, T1, battle, captured, expected_setup,
    fight, garrison, play, siege_rules, siege_setup, siege_start, standard, step_onto_gate,
    stepped,
};
use crate::town::pool::HeroStatus;
use crate::{
    Actor, Answer, GameCommand, GameEvent, GameRuleset, GameState, HeroArmy, SkillOffer, apply,
};

/// Moves blue's visitor `hero` of its town 1 into the town's garrison, the garrison's four imps
/// joining its army (TWN-ROW-002), as blue's town screen would; its army after.
fn enter(rules: &GameRuleset, game: &mut GameState, hero: HeroId) -> Vec<ArmyStack> {
    let (adventure, _) = game.adventure_with_heroes();
    let parts = (rules.core(), rules.adventure());
    adventure
        .move_town_hero(
            parts,
            BLUE,
            (T1, 8),
            TownHeroMove::Enter,
            Viewers::NONE,
            &mut Vec::new(),
        )
        .unwrap();
    let held = adventure.town_hero(T1).unwrap();
    assert_eq!(held.id, hero);
    held.army.clone()
}

/// [`siege_setup`] with blue's hero 1 the visitor of its town 1 moved into its garrison (red's
/// hero outside with `red` for its army, or its own; without a fort): the game and the garrison
/// hero's army.
fn held_game(rules: &GameRuleset, red: Option<Vec<ArmyStack>>) -> (GameState, Vec<ArmyStack>) {
    let mut setup = siege_setup(rules, true, standard(false));
    if let Some(red) = red {
        setup.heroes[0].army = HeroArmy::Custom(red);
    }
    let mut game = GameState::new(rules, &setup, 7).unwrap();
    let army = enter(rules, &mut game, BLUE_HERO);
    assert!(garrison(&game).is_empty());
    (game, army)
}

/// The siege of blue's town 1 its garrison hero defends with `army`, as the visit gives it.
fn held_start(game: &GameState, army: Vec<ArmyStack>) -> BattleStart {
    let held = game.adventure().town_hero(T1).unwrap();
    BattleStart {
        defender: Defender::Hero(Combatant {
            hero: BLUE_HERO,
            owner: BLUE,
            army,
            formation: held.formation,
            tactics: held.tactics,
        }),
        ..siege_start(game, Some(T1))
    }
}

// covers: TWN-SIE-001, TWN-SIE-002, TWN-VIS-001, GAM-BAT-001
#[test]
fn a_garrison_hero_alone_defends_its_town_s_siege_with_its_own_army() {
    let rules = siege_rules();
    let (game, army) = held_game(&rules, None);
    let mut after = game.clone();
    let events = step_onto_gate(&rules, &mut after);
    // From the tile before, without a merge (no stack told): the siege against the garrison hero
    // with its army, built from the town as a visitor's (TWN-SIE-003 to -005).
    assert!(!stepped(&events), "{events:?}");
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, GameEvent::StackChanged { .. }))
    );
    let expected = expected_setup(
        &game,
        &rules,
        &held_start(&game, army.clone()),
        (ROCK, None),
    );
    let open = after.battle(RED).unwrap();
    assert_eq!(open.state(), &battle(&rules, &expected));
    assert_eq!(open.owners(), [Some(RED), Some(BLUE)]);
    assert_eq!(open.heroes(), [Some(HERO), Some(BLUE_HERO)]);
    assert_eq!(after.adventure().hero(HERO).unwrap().pos, OUTSIDE);
    assert_eq!(after.adventure().town_hero(T1).unwrap().army, army);
}

// covers: TWN-SIE-001, TWN-CAP-003, TWN-SET-005, TWN-VEW-002
#[test]
fn a_garrison_hero_keeps_its_town_as_the_winner_and_leaves_the_game_beaten() {
    let rules = siege_rules();
    // Red retreats at once: the garrison hero wins, its army and its town kept.
    let (mut game, army) = held_game(&rules, None);
    let mut events = step_onto_gate(&rules, &mut game);
    events.extend(play(&mut game, &rules, |actor, listed| {
        if actor == Actor::Player(RED) {
            GameCommand::Battle(Command::Retreat)
        } else {
            fight(actor, listed)
        }
    }));
    assert!(game.battle(RED).is_none() && !captured(&events));
    let adventure = game.adventure();
    assert!(adventure.hero(HERO).is_none());
    assert_eq!(adventure.town_hero(T1).unwrap().army, army);
    assert_eq!(adventure.owner(1), Some(BLUE));
    // Red's horde of imps beats it: it leaves the game, told to blue, its type free; the capture
    // after red's levels, red's hero outside, the town empty.
    let imp = rules.core().creature_by_key("t:imp").unwrap();
    let horde = ArmyStack {
        slot: 1,
        creature: imp,
        count: 4000,
    };
    let (mut game, _) = held_game(&rules, Some(vec![horde]));
    let hero_type = game.hero(BLUE_HERO).unwrap().hero_type;
    let mut events = step_onto_gate(&rules, &mut game);
    events.extend(play(&mut game, &rules, fight));
    while game.level_up(RED).is_some() {
        let answer = GameCommand::Answer(Answer::LevelUp(SkillOffer::First));
        apply(&mut game, &rules, Actor::Player(RED), answer, &mut events).unwrap();
    }
    let removed = GameEvent::Adventure(AdventureEvent {
        kind: AdventureEventKind::HeroRemoved {
            hero: BLUE_HERO,
            pos: BLUE_GATE,
        },
        seen_by: Viewers::NONE.with(BLUE),
    });
    let at = |e: &GameEvent| events.iter().position(|x| x == e);
    assert!(at(&removed).is_some(), "{events:?}");
    // Blue is told its town's view without the garrison hero before the capture.
    let told = events.iter().position(|e| {
        matches!(
            e,
            GameEvent::TownChanged {
                player: BLUE,
                town: T1,
                garrison_hero: false,
                level: 3,
                ..
            }
        )
    });
    let capture = events
        .iter()
        .position(|e| captured(core::slice::from_ref(e)));
    assert!(
        told.is_some() && capture.is_some() && told < capture,
        "{events:?}"
    );
    let adventure = game.adventure();
    assert!(adventure.town_hero(T1).is_none());
    assert_eq!(adventure.owner(1), Some(RED));
    assert_eq!(adventure.hero(HERO).unwrap().pos, OUTSIDE);
    assert!(garrison(&game).is_empty());
    assert_eq!(game.hero_pool().status(hero_type), Some(HeroStatus::Free));
}

// covers: TWN-SIE-001, TWN-VIS-002, GAM-BAT-001
#[test]
fn beside_a_garrison_hero_its_visitor_fights_at_the_gate_and_the_town_stands() {
    let rules = siege_rules();
    let mut setup = siege_setup(&rules, true, standard(false));
    // Blue's second hero, hero 2, beside the town.
    let mut second = setup.heroes[1].clone();
    second.pos = Pos::new(2, 0, 0);
    setup.heroes.push(second);
    let mut game = GameState::new(&rules, &setup, 7).unwrap();
    let second = HeroId::new(2);
    let army = enter(&rules, &mut game, BLUE_HERO);
    end_turn(&rules, &mut game, RED);
    let step = GameCommand::Adventure(AdventureCommand::MoveHero {
        hero: second,
        path: vec![BLUE_GATE],
    });
    apply(
        &mut game,
        &rules,
        Actor::Player(BLUE),
        step,
        &mut Vec::new(),
    )
    .unwrap();
    assert_eq!(game.adventure().town_visitor(T1), Some(second));
    end_turn(&rules, &mut game, BLUE);
    // Red's step: from the tile before, the battle against the visitor alone, no siege.
    let mut events = step_onto_gate(&rules, &mut game);
    assert!(!stepped(&events));
    let open = game.battle(RED).unwrap();
    assert_eq!(open.heroes(), [Some(HERO), Some(second)]);
    assert!(h3_battle::computer::siege(open.state()).is_none());
    // Red's win takes the visitor off the map; the town stands with its garrison hero, red's
    // hero outside: its next visit meets the garrison hero.
    events.extend(play(&mut game, &rules, fight));
    while game.level_up(RED).is_some() {
        let answer = GameCommand::Answer(Answer::LevelUp(SkillOffer::First));
        apply(&mut game, &rules, Actor::Player(RED), answer, &mut events).unwrap();
    }
    assert!(!captured(&events), "{events:?}");
    let adventure = game.adventure();
    assert!(adventure.hero(second).is_none());
    assert_eq!(adventure.hero(HERO).unwrap().pos, OUTSIDE);
    assert_eq!(adventure.town_hero(T1).unwrap().army, army);
    assert_eq!(adventure.owner(1), Some(BLUE));
}
