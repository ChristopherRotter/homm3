//! The bonus visitables in the game (E08 T115b; increment 12's design note §8, §9, §15): the
//! calendar's day of the week passed to the adventure's visits, the Sirens' losses given as the
//! hero's experience by the battle part's hit points.

use h3_adventure::{AdventureEvent, AdventureEventKind, Holds, Viewers, Weekday};
use h3_heroes::{HeroChange, VisitBonus};

use super::decisions::{RED, after_steps, move_hero, play};
use super::rewards::{AT, HERO, changed, game, reward_at};
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
