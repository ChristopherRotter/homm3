//! The bonus visitables in the game (E08 T115b; increment 12's design note §8, §9, §15): the
//! calendar's day of the week passed to the adventure's visits, the Sirens' losses given as the
//! hero's experience by the battle part's hit points.

use h3_adventure::{AdventureEvent, AdventureEventKind, Holds, Viewers, Weekday};
use h3_heroes::{HeroChange, VisitBonus};

use super::decisions::{ACCEPT, RED, after_steps, move_hero, play};
use super::rewards::{AT, HERO, bonuses_of, changed, game, reward_at};
use super::*;

/// Every player ends its turn: the next day starts.
fn next_day(state: &mut GameState, rules: &GameRuleset) {
    for player in [RED, PlayerId::new(1)] {
        play(state, rules, player, GameCommand::EndTurn).unwrap();
    }
}

// covers: GAM-CAL-002, ADV-BON-002
#[test]
fn the_calendars_day_of_the_week_decides_a_bonus_visit() {
    let rules = rules();
    let mut state = game(&rules, reward_at(&rules, "t:idol", Holds::Nothing));
    assert_eq!(state.weekday(&rules), Weekday::FIRST);
    next_day(&mut state, &rules);
    assert_eq!((state.day(), state.weekday(&rules).get()), (2, 2));
    // Day 2: the Idol's morale (its even days), under its mark.
    let events = after_steps(play(&mut state, &rules, RED, move_hero(0, &[AT])).unwrap());
    let morale = VisitBonus {
        mark: 4,
        morale: 1,
        luck: 0,
    };
    assert_eq!(state.adventure().visit_marks(HERO).bonuses().once, [morale]);
    let told = GameEvent::Adventure(AdventureEvent {
        kind: AdventureEventKind::OnceMarkChanged {
            hero: HERO,
            bonus: morale,
            held: true,
        },
        seen_by: Viewers::NONE.with(RED),
    });
    assert!(events.contains(&told), "{events:?}");
    // The week's days run 1 to 7, then 1 again.
    for _ in 2..8 {
        next_day(&mut state, &rules);
    }
    assert_eq!((state.day(), state.weekday(&rules)), (8, Weekday::FIRST));
}

// covers: ADV-BON-008, HER-EXP-001
#[test]
fn the_sirens_losses_are_the_heros_experience_by_their_hit_points() {
    let rules = rules();
    let mut state = game(&rules, reward_at(&rules, "t:sirens", Holds::Nothing));
    let before = state.hero(HERO).unwrap().experience;
    // Nine Imps keep 6 (trunc(9 × 0.7)); the three lost of 4 hit points each: 12 (no Learning).
    let events = after_steps(play(&mut state, &rules, RED, move_hero(0, &[AT])).unwrap());
    let imps = state.adventure().hero(HERO).unwrap().army.clone();
    assert_eq!(imps.iter().map(|s| s.count).collect::<Vec<_>>(), [6]);
    let lost = GameEvent::Adventure(AdventureEvent {
        kind: AdventureEventKind::SirensLoss {
            hero: HERO,
            slot: 1,
            creature: IMP,
            count: 3,
        },
        seen_by: Viewers::NONE.with(RED),
    });
    let gained = changed(HeroChange::GainExperience(12));
    let at = |e: &GameEvent| events.iter().position(|x| x == e);
    assert!(
        at(&lost) < at(&gained) && at(&gained).is_some(),
        "{events:?}"
    );
    assert_eq!(state.hero(HERO).unwrap().experience, before + 12);
    // Again (the mark held): nothing lost, no experience.
    let again = after_steps(play(&mut state, &rules, RED, move_hero(0, &[AT])).unwrap());
    assert!(
        !again
            .iter()
            .any(|e| matches!(e, GameEvent::HeroChanged { .. }))
    );
    assert_eq!(state.hero(HERO).unwrap().experience, before + 12);
}

// covers: ADV-BON-002, HER-ADV-008
#[test]
fn each_once_mark_reaches_the_battle_hero_as_its_own_visits_source() {
    let rules = rules();
    // A Warrior's Tomb at (0, 1), visited from the hero's (0, 0), and the Idol on [`AT`].
    let beside = Pos::new(0, 1, 0);
    let mut tomb = reward_at(&rules, "t:tomb", Holds::Nothing);
    tomb.anchor = beside;
    let mut setup = setup();
    setup.map.objects = vec![reward_at(&rules, "t:idol", Holds::Nothing), tomb];
    let mut state = GameState::new(&rules, &setup, 7).unwrap();
    // Day 1, the Tomb's question accepted: -3 morale under its mark 0.
    play(&mut state, &rules, RED, move_hero(0, &[beside])).unwrap();
    play(&mut state, &rules, RED, ACCEPT).unwrap();
    // Day 2, the Idol: +1 morale under its mark 4 (its even days), the Tomb's still held.
    next_day(&mut state, &rules);
    play(&mut state, &rules, RED, move_hero(0, &[AT])).unwrap();
    let once = |mark, morale| VisitBonus {
        mark,
        morale,
        luck: 0,
    };
    let bonuses = bonuses_of(&state);
    assert_eq!(bonuses.once, [once(0, -3), once(4, 1)]);
    assert_eq!((bonuses.morale, bonuses.luck), (0, 0));
    // The battle hero holds each as its own line, the visit's source by its mark with its amount,
    // never summed into one, and no temporary line (the unmarked sums 0; ADR-0013 point 15.1).
    let morale = |amount, mark| h3_contracts::BattleEffect::Morale {
        amount,
        affects: h3_content::Affects::Own,
        source: h3_contracts::EffectSource::Visit { mark },
    };
    let effects = state.battle_hero(rules.core(), HERO).unwrap().effects;
    let bonus_lines: Vec<_> = (effects.iter())
        .filter(|e| match e {
            h3_contracts::BattleEffect::Morale { source, .. }
            | h3_contracts::BattleEffect::Luck { source, .. } => matches!(
                source,
                h3_contracts::EffectSource::Visit { .. } | h3_contracts::EffectSource::Temporary
            ),
            _ => false,
        })
        .cloned()
        .collect();
    assert_eq!(bonus_lines, [morale(-3, 0), morale(1, 4)]);
}

// covers: ADV-BON-008, HER-EXP-001, GAM-DEC-002
#[test]
fn the_sirens_experience_grows_with_learning_and_brings_its_level() {
    let rules = rules();
    let lore = rules.core().skill_by_key("t:lore").unwrap();
    let mut setup = setup();
    setup.map.objects = vec![reward_at(&rules, "t:sirens", Holds::Nothing)];
    // Ninety Imps; Expert Lore (+15 %) and 900 points, 100 under level 2's 1 000.
    setup.heroes[0].army = HeroArmy::Custom(vec![ArmyStack {
        slot: 1,
        creature: IMP,
        count: 90,
    }]);
    setup.heroes[0].details = HeroDetailsSetup {
        experience: Some(900),
        skills: Some(vec![(lore, 3)]),
        ..HeroDetailsSetup::default()
    };
    let mut state = GameState::new(&rules, &setup, 7).unwrap();
    let hero = state.hero(HERO).unwrap().clone();
    assert_eq!((hero.experience, hero.level), (900, 1));
    let events = after_steps(play(&mut state, &rules, RED, move_hero(0, &[AT])).unwrap());
    // Ninety keep 62 (D-64): 28 lost of 4 hit points, 112, grown by 15 % and rounded down: 128
    // (HER-EXP-001's form, the game's `gained_experience`), past 1 000.
    let army = &state.adventure().hero(HERO).unwrap().army;
    assert_eq!(army.iter().map(|s| s.count).collect::<Vec<_>>(), [62]);
    let base = 28 * rules.battle().creature(IMP).unwrap().hit_points;
    assert_eq!(base, 112);
    assert_eq!(h3_heroes::gained_experience(&hero, rules.core(), base), 128);
    let gained = changed(HeroChange::GainExperience(128));
    let at = |found: &dyn Fn(&GameEvent) -> bool| events.iter().position(found);
    let experience = at(&|e| *e == gained).unwrap();
    assert_eq!(state.hero(HERO).unwrap().experience, 900 + 128);
    // Its level after it: gained, or its level-up opened (two offers) for red.
    let level = at(&|e| match *e {
        GameEvent::LevelGained {
            player,
            hero,
            level,
            ..
        }
        | GameEvent::LevelUpOpened {
            player,
            hero,
            level,
            ..
        } => (player, hero, level) == (RED, HERO, 2),
        _ => false,
    });
    assert!(level.is_some_and(|level| level > experience), "{events:?}");
    assert!(state.hero(HERO).unwrap().level == 2 || state.waiting_for() == Some(RED));
}
