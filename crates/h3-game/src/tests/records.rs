//! Tests of what the game applies of Pandora's Box and the Pyramid (E08 T62b1; ADR-0017 point
//! 10): the box's question, its record's gains, its losses floored at 0, the hero's half with its
//! experience, the creatures without room in one army dialog at the record's slots, a won guarded
//! box's record; the Pyramid's question, its battle against two creatures split as the original's
//! and ordered, a win's looting and a looted one's luck; a level-up drawn once.

use std::boxed::Box;

use h3_adventure::{Guards, Holds, OfferKind, RewardRecord, TileRand, split_with_second};
use h3_contracts::BattleSide;
use h3_heroes::{HeroChange, HeroReward};

use super::decisions::{ACCEPT, DECLINE, GOLD, RED, play};
use super::monsters::valued;
use super::rewards::{AT, HERO, changed, game, gold, reward_at, visit, won};
use super::*;

const GOG: CreatureTypeId = CreatureTypeId::from_raw(1);

/// A box at [`AT`] holding `record`.
fn boxed(rules: &GameRuleset, record: RewardRecord) -> GameState {
    game(
        rules,
        reward_at(rules, "t:box", Holds::Record(Box::new(record))),
    )
}

/// `count` of `creature` in `slot`.
fn stack(slot: u8, creature: CreatureTypeId, count: u32) -> ArmyStack {
    ArmyStack {
        slot,
        creature,
        count,
    }
}

// covers: ADV-PIK-013, ADV-RWD-001, GAM-RES-001, GAM-DEC-001
#[test]
fn a_boxs_question_opens_and_its_record_reaches_the_treasury_and_the_hero() {
    let rules = rules();
    let record = RewardRecord {
        hero: HeroReward {
            experience: 1000,
            primary: [1, 0, 0, 0],
            ..HeroReward::default()
        },
        resources: vec![(GOLD, 500)],
        ..RewardRecord::default()
    };
    let mut state = boxed(&rules, record);
    visit(&mut state, &rules);
    assert_eq!(state.decision(RED).unwrap().kind, OfferKind::OpenBox);
    // Declined: the box stays; the open question was hashed with its kind.
    let mut declined = state.clone();
    play(&mut declined, &rules, RED, DECLINE).unwrap();
    assert_ne!(state.state_hash(), declined.state_hash());
    assert!(!declined.adventure().is_removed(0));
    // Accepted: the gold gained, the primary skill, then the experience last with its level.
    let attack = state.hero(HERO).unwrap().primary[0];
    let events = play(&mut state, &rules, RED, ACCEPT).unwrap();
    assert!(events.contains(&GameEvent::ResourceGained {
        player: RED,
        resource: GOLD,
        amount: 500,
    }));
    let gained = |e: &GameEvent| matches!(e, GameEvent::HeroChanged { .. });
    let changes: Vec<&GameEvent> = events.iter().filter(|e| gained(e)).collect();
    assert_eq!(*changes[0], changed(HeroChange::GainPrimary([1, 0, 0, 0])));
    assert!(matches!(
        changes[1],
        GameEvent::HeroChanged {
            change: HeroChange::GainExperience(_),
            ..
        }
    ));
    assert_eq!(gold(&state), 2500);
    let hero = state.hero(HERO).unwrap();
    assert_eq!(hero.primary[0], attack + 1);
    assert!(hero.level == 2 || state.waiting_for() == Some(RED));
    assert!(state.adventure().is_removed(0));
}

// covers: ADV-RWD-001, GAM-RES-001
#[test]
fn a_records_loss_stops_at_zero() {
    let rules = rules();
    let record = RewardRecord {
        resources: vec![(GOLD, -2500)],
        ..RewardRecord::default()
    };
    let mut state = boxed(&rules, record);
    visit(&mut state, &rules);
    let events = play(&mut state, &rules, RED, ACCEPT).unwrap();
    assert!(events.contains(&GameEvent::ResourceLost {
        player: RED,
        resource: GOLD,
        amount: 2000,
    }));
    assert_eq!(gold(&state), 0);
    // Seen by its owner alone.
    let lost = events
        .iter()
        .find(|e| matches!(e, GameEvent::ResourceLost { .. }))
        .unwrap();
    assert_eq!(project(lost, Viewer::Player(RED)), Some(*lost));
    assert_eq!(project(lost, Viewer::Player(PlayerId::new(1))), None);
}

// covers: ADV-RWD-001, GAM-DEC-004
#[test]
fn a_records_creatures_without_room_open_one_army_dialog_at_their_slots() {
    let rules = rules();
    // Red's hero's two slots full of imps: the record's imps join, its gogs find no room.
    let mut setup = setup();
    setup.heroes[0].army = HeroArmy::Custom(vec![stack(0, IMP, 1), stack(1, IMP, 9)]);
    let record = RewardRecord {
        creatures: vec![stack(0, IMP, 2), stack(1, GOG, 3)],
        ..RewardRecord::default()
    };
    setup.map.objects = vec![reward_at(&rules, "t:box", Holds::Record(Box::new(record)))];
    let mut state = GameState::new(&rules, &setup, 7).unwrap();
    visit(&mut state, &rules);
    let events = play(&mut state, &rules, RED, ACCEPT).unwrap();
    assert!(events.contains(&GameEvent::ArmyDialogOpened {
        player: RED,
        hero: HERO,
        stack: stack(1, GOG, 3),
    }));
    assert_eq!(
        state.army_dialog(RED),
        Some((HERO, &[stack(1, GOG, 3)][..]))
    );
    let army = &state.adventure().hero(HERO).unwrap().army;
    assert_eq!(army[0], stack(0, IMP, 3));
    // Closed: what is left is lost.
    let events = play(&mut state, &rules, RED, DECLINE).unwrap();
    assert!(events.contains(&GameEvent::GroupLost {
        player: RED,
        hero: HERO,
        stack: stack(1, GOG, 3),
    }));
}

// covers: ADV-PIK-013, ADV-PIK-004
#[test]
fn a_won_guarded_boxs_record_reaches_the_treasury() {
    let rules = rules();
    let guarded = || {
        let record = RewardRecord {
            resources: vec![(GOLD, 300)],
            ..RewardRecord::default()
        };
        let object = h3_adventure::ObjectSetup {
            guards: Some(Guards {
                army: vec![stack(0, IMP, 2)],
                question: false,
            }),
            ..reward_at(&rules, "t:box", Holds::Record(Box::new(record)))
        };
        game(&rules, object)
    };
    // Accepted: the battle at once, no second question.
    let mut state = guarded();
    visit(&mut state, &rules);
    let events = play(&mut state, &rules, RED, ACCEPT).unwrap();
    let started = |e: &GameEvent| matches!(e, GameEvent::BattleStarted { .. });
    assert!(events.iter().any(started));
    assert!(state.battle(RED).is_some() && state.decision(RED).is_none());
    // Won: the record, the box gone.
    let mut state = guarded();
    let sides = ([Some(RED), None], [Some(HERO), None], AT);
    let mut events = Vec::new();
    state.end_battle(&rules, sides, &won(), &mut events);
    assert!(events.contains(&GameEvent::ResourceGained {
        player: RED,
        resource: GOLD,
        amount: 300,
    }));
    assert_eq!(gold(&state), 2300);
    assert!(state.adventure().is_removed(0));
}

// covers: ADV-TCH-008, ADV-MON-012, ADV-QUIRK-6, HER-ADV-008
#[test]
fn the_pyramid_fights_two_creatures_split_by_its_tile_and_a_looted_one_costs_luck() {
    let rules = valued("t:split");
    let mut state = game(&rules, reward_at(&rules, "t:pyramid", Holds::Nothing));
    visit(&mut state, &rules);
    assert_eq!(state.decision(RED).unwrap().kind, OfferKind::SearchPyramid);
    let mut declined = state.clone();
    play(&mut declined, &rules, RED, DECLINE).unwrap();
    assert_ne!(state.state_hash(), declined.state_hash());
    let events = play(&mut state, &rules, RED, ACCEPT).unwrap();
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::BattleStarted { .. }))
    );
    assert!(state.adventure().knows(RED, 0));
    // Under ADV-QUIRK-6 the split with the tile's draw, the gogs' stack placed by the order.
    let army = state.adventure().hero(HERO).unwrap().army.clone();
    let mut tile = TileRand::new(AT);
    let expected: Vec<(u8, CreatureTypeId, u32)> =
        split_with_second(rules.core(), &army, (IMP, 4), (GOG, &[2]), || {
            tile.percent()
        })
        .iter()
        .map(|s| (s.slot, s.creature, s.count))
        .collect();
    let battle = state.battle(RED).unwrap();
    let defending: Vec<(u8, CreatureTypeId, u32)> = battle
        .state()
        .stacks()
        .iter()
        .filter(|s| s.side == BattleSide::Defender)
        .map(|s| (s.slot, s.creature, s.count))
        .collect();
    assert_eq!(defending, expected);
    assert!(expected.iter().any(|&(_, c, n)| (c, n) == (GOG, 2)));
    // Won: looted; asked again, its luck once by its mark.
    let mut state = game(&rules, reward_at(&rules, "t:pyramid", Holds::Nothing));
    let sides = ([Some(RED), None], [Some(HERO), None], AT);
    state.end_battle(&rules, sides, &won(), &mut Vec::new());
    assert!(state.adventure().is_emptied(0) && !state.adventure().is_removed(0));
    visit(&mut state, &rules);
    let events = play(&mut state, &rules, RED, ACCEPT).unwrap();
    let luck = HeroChange::Bonus {
        morale: 0,
        luck: -2,
        mark: Some(1),
    };
    assert!(events.contains(&changed(luck)));
    assert_eq!(state.hero(HERO).unwrap().bonuses.luck, -2);
}
