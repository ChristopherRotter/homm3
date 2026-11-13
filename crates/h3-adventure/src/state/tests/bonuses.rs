//! The battle bonuses and the timed marks on the hero on the map (E08 T115a; increment 12's
//! design note §12, §14, §15): the once-marks' gates and the saturating sums, the battle's clear,
//! the day's and the week's drops, the week's land points in the day's maximum, and what a
//! command tells the owner of them.

use h3_heroes::{BattleBonuses, VisitBonus};

use super::garrisons::bonus_rules;
use super::teleporters::{BLUE, HERO, RED, jump_state, p};
use super::*;
use crate::command::OwnHeroes;
use crate::ruleset::MarkLifetime;
use crate::testing::stream;

const fn once(mark: u8, morale: i8, luck: i8) -> VisitBonus {
    VisitBonus { mark, morale, luck }
}

// covers: HER-ADV-008
#[test]
fn the_bonuses_add_up_saturating_and_a_once_mark_gates_its_bonus() {
    let mut b = HeroBonuses::NONE;
    assert!(b.is_empty() && !b.has_mark(0));
    assert_eq!(b, HeroBonuses::default());
    assert_eq!(b.battle(), BattleBonuses::NONE);
    // The Warrior's Tomb's −3 under mark 0: a second Tomb adds nothing, whatever its amounts
    // (E7-11).
    assert!(b.add_once(once(0, -3, 0)));
    assert!(!b.add_once(once(0, -3, 0)));
    assert!(!b.add_once(once(0, 1, 1)));
    // A looted Pyramid's under mark 1 and the Sirens' 13 of no amount, kept sorted by mark with
    // their amounts (increment 12's design note §12.2).
    assert!(b.add_once(once(13, 0, 0)));
    assert!(b.add_once(once(1, 0, -2)));
    assert_eq!(b.once, [once(0, -3, 0), once(1, 0, -2), once(13, 0, 0)]);
    assert!(b.has_mark(13) && !b.has_mark(2) && !b.is_empty());
    // A box's and an event's add up without a mark, saturating both ways (A-9's class).
    b.add(100, 127);
    b.add(100, 127);
    assert_eq!((b.morale, b.luck), (127, 127));
    let mut low = HeroBonuses::NONE;
    low.add(-100, -100);
    low.add(-100, -100);
    assert_eq!((low.morale, low.luck), (-128, -128));
    low.add(0, 0);
    assert!(!low.is_empty());
    let mut lucky = HeroBonuses::NONE;
    lucky.add(0, 1);
    assert!(!lucky.is_empty());
    assert_eq!(
        b.battle(),
        BattleBonuses {
            morale: 127,
            luck: 127,
            once: &b.once,
        }
    );
    // No fixed count: every mark a byte names, sorted whatever the order of the visits.
    let mut all = HeroBonuses::NONE;
    for mark in (0..=u8::MAX).rev() {
        assert!(all.add_once(once(mark, 1, 0)));
    }
    assert_eq!(all.once.len(), 256);
    assert!(all.once.windows(2).all(|w| w[0].mark < w[1].mark));
}

// covers: HER-ADV-008, GAM-BAT-014
#[test]
fn a_heros_visit_marks_hold_nothing_until_a_visit_and_nothing_again_once_empty() {
    let rules = bonus_rules();
    let well = rules.1.object_type_by_key("t:well").unwrap();
    let mut marks = VisitMarks::NONE;
    assert!(marks.is_empty() && marks.bonuses().is_empty() && marks.timed().is_empty());
    assert_eq!(marks, VisitMarks::default());
    // Nothing added writes nothing.
    marks.add(0, 0);
    marks.clear_bonuses();
    marks.drop_timed(&rules.1, MarkLifetime::Day);
    assert!(marks.is_empty());
    // A mark held gates its bonus; the block's bonuses are `HeroBonuses`'.
    assert!(marks.add_once(once(0, -3, 0)));
    assert!(!marks.add_once(once(0, 1, 1)));
    marks.add(1, 2);
    assert!(marks.add_timed(well) && !marks.add_timed(well));
    assert_eq!(marks.bonuses().once, [once(0, -3, 0)]);
    assert_eq!((marks.bonuses().morale, marks.bonuses().luck), (1, 2));
    assert_eq!(marks.timed(), [well]);
    // A fork shares the block; a write copies it.
    let fork = marks.clone();
    marks.clear_bonuses();
    assert!(marks.bonuses().is_empty() && !marks.is_empty());
    assert_eq!(fork.bonuses().once, [once(0, -3, 0)]);
    // Empty again once both are: equal to a record that never held anything.
    marks.drop_timed(&rules.1, MarkLifetime::Day);
    assert_eq!(marks, VisitMarks::NONE);
    let mut other = VisitMarks::NONE;
    other.add(1, 0);
    other.add(-1, 0);
    assert_eq!(other, VisitMarks::NONE);
}

// covers: ADV-BON-001, HER-ADV-003, ADV-MOV-002
#[test]
fn the_days_and_the_weeks_marks_drop_at_their_steps_and_the_stables_raise_the_land_points() {
    let rules = bonus_rules();
    let ty = |k| rules.1.object_type_by_key(k).unwrap();
    let (well, stables, buoy) = (ty("t:well"), ty("t:stables"), ty("t:buoy"));
    let mut state = jump_state(&rules, (1, None), &[], &[]);
    let base = state.hero(HERO).unwrap().max_movement;
    let fresh = state.clone();
    let timed = |state: &AdventureState| state.visit_marks(HERO).timed().to_vec();
    // A mark once per type, kept sorted (the visits come with T115b).
    assert!(state.change_marks(HERO, |m| m.add_timed(stables)));
    assert!(state.change_marks(HERO, |m| m.add_timed(well)));
    assert!(!state.change_marks(HERO, |m| m.add_timed(well)));
    let mut sorted = vec![well, stables];
    sorted.sort_unstable();
    assert_eq!(timed(&state), sorted);
    assert_eq!(state.visit_marks(HERO).week_land_points(&rules.1), 400);
    // The turn's maximum on land holds the Stables' 400 (the week's land points, last).
    state.start_turn(&rules.0, &rules.1, &[], RED);
    let hero = state.hero(HERO).unwrap();
    assert_eq!((hero.max_movement, hero.movement), (base + 400, base + 400));
    // The day's step drops the Well's mark, not the Stables'.
    state.start_day(&rules.0, &rules.1, &[]);
    assert_eq!(timed(&state), [stables]);
    // The week's step drops the Stables': the week's first turn has no 400.
    state.start_week(&rules.0, &rules.1, &mut stream(), &mut Vec::new());
    assert_eq!(timed(&state), []);
    state.start_turn(&rules.0, &rules.1, &[], RED);
    // The same steps without the marks: no block left.
    let mut again = fresh;
    again.start_turn(&rules.0, &rules.1, &[], RED);
    again.start_day(&rules.0, &rules.1, &[]);
    again.start_week(&rules.0, &rules.1, &mut stream(), &mut Vec::new());
    again.start_turn(&rules.0, &rules.1, &[], RED);
    assert_eq!(state, again);
    assert_eq!(state.hero(HERO).unwrap().max_movement, base);
    // One record alone (the town lane's garrison hero): a kind of no lifetime and a type the
    // part lacks are kept and count nothing.
    let stranger = ObjectTypeId::from_raw(u16::MAX);
    let mut record = VisitMarks::NONE;
    for ty in [buoy, well, stables, stranger] {
        record.add_timed(ty);
    }
    record.drop_timed(&rules.1, MarkLifetime::Week);
    record.drop_timed(&rules.1, MarkLifetime::Day);
    let mut kept = vec![buoy, stranger];
    kept.sort_unstable();
    assert_eq!(record.timed(), kept);
    assert_eq!(record.week_land_points(&rules.1), 0);
}

// covers: GAM-BAT-014
#[test]
fn a_battles_end_clears_the_bonuses_of_its_heroes_still_on_the_map() {
    let rules = bonus_rules();
    let well = rules.1.object_type_by_key("t:well").unwrap();
    let other = HeroId::new(2);
    let mut state = jump_state(&rules, (1, None), &[(2, 1, p(9, 1))], &[]);
    let fresh = state.clone();
    for id in [HERO, other] {
        state.give_bonus(id, 1, 1);
        assert!(state.give_once(id, once(0, -3, 0)));
        assert!(!state.give_once(id, once(0, 1, 0)));
        state.change_marks(id, |m| m.add_timed(well));
    }
    // A hero not on the map gets nothing.
    let stranger = HeroId::new(9);
    state.give_bonus(stranger, 1, 1);
    assert!(!state.give_once(stranger, once(1, 0, 1)));
    assert!(state.visit_marks(stranger).is_empty());
    // A fork shares the block; a write copies it.
    let fork = state.clone();
    let kept = state.visit_marks(other).clone();
    // Its heroes on the map lose them; a hero not on the map is nothing; the timed marks stay.
    state.clear_battle_bonuses(&[HERO, stranger]);
    let marks = state.visit_marks(HERO);
    assert_eq!(
        (marks.bonuses(), marks.timed()),
        (&HeroBonuses::NONE, &[well][..])
    );
    assert_eq!(state.visit_marks(other), &kept);
    assert_eq!(fork.visit_marks(HERO).bonuses().once, [once(0, -3, 0)]);
    // A hero that leaves the map leaves with them; the block goes once none holds any.
    state.end_battle(&rules.0, &[(HERO, None)], &mut Vec::new());
    assert!(state.visit_marks(HERO).is_empty());
    state.leave_marks(other);
    state.leave_marks(other);
    let mut expected = fresh;
    expected.end_battle(&rules.0, &[(HERO, None)], &mut Vec::new());
    assert_eq!(state, expected);
}

// covers: ADV-FOG-006
#[test]
fn a_command_tells_the_owner_the_marks_dropped_the_sums_then_the_marks_gained() {
    let rules = bonus_rules();
    let ty = |k| rules.1.object_type_by_key(k).unwrap();
    let (well, stables) = (ty("t:well"), ty("t:stables"));
    let other = HeroId::new(2);
    let mut state = jump_state(&rules, (1, None), &[(2, 1, p(9, 1))], &[]);
    let told = |before: &OwnHeroes, state: &AdventureState| {
        let mut events = Vec::new();
        before.tell(state, &mut events);
        events
    };
    // Nothing changed: nothing told.
    assert_eq!(told(&state.own_heroes(), &state), []);
    // Gains: the sums, then each mark gained in order, to the owner alone.
    let before = state.own_heroes();
    state.give_bonus(HERO, 1, 2);
    state.give_once(HERO, once(13, 0, 0));
    state.give_once(HERO, once(0, -3, 0));
    state.change_marks(HERO, |m| m.add_timed(well));
    let red = Viewers::NONE.with(RED);
    let event = |kind| AdventureEvent { kind, seen_by: red };
    let held = |bonus, held| {
        event(AdventureEventKind::OnceMarkChanged {
            hero: HERO,
            bonus,
            held,
        })
    };
    let timed = |object_type, held| {
        event(AdventureEventKind::TimedMarkChanged {
            hero: HERO,
            object_type,
            held,
        })
    };
    let sums = |morale, luck| {
        event(AdventureEventKind::BonusChanged {
            hero: HERO,
            morale,
            luck,
        })
    };
    let gained = [
        sums(1, 2),
        held(once(0, -3, 0), true),
        held(once(13, 0, 0), true),
        timed(well, true),
    ];
    assert_eq!(told(&before, &state), gained);
    // Drops first, then the sums, then the gains: a battle's clear and a day's drop, then a
    // Pyramid's mark and the Stables' (a mark of other amounts counts as another).
    let before = state.own_heroes();
    state.clear_battle_bonuses(&[HERO]);
    state.start_day(&rules.0, &rules.1, &[]);
    state.give_once(HERO, once(1, 0, -2));
    state.give_once(HERO, once(13, 0, 1));
    state.change_marks(HERO, |m| m.add_timed(stables));
    assert_eq!(
        told(&before, &state),
        [
            held(once(0, -3, 0), false),
            held(once(13, 0, 0), false),
            timed(well, false),
            sums(0, 0),
            held(once(1, 0, -2), true),
            held(once(13, 0, 1), true),
            timed(stables, true),
        ]
    );
    // A hero that was not on the map is told its movement, and its bonuses against none: Blue's
    // hero without any, nothing more.
    let movement = |id: HeroId, seen_by| {
        let hero = state.hero(id).unwrap();
        AdventureEvent {
            kind: AdventureEventKind::MovementChanged {
                hero: id,
                movement: hero.movement,
                max_movement: hero.max_movement,
            },
            seen_by,
        }
    };
    let mut expected = vec![movement(HERO, red)];
    expected.extend([
        held(once(1, 0, -2), true),
        held(once(13, 0, 1), true),
        timed(stables, true),
    ]);
    expected.push(movement(other, Viewers::NONE.with(BLUE)));
    assert_eq!(told(&OwnHeroes::default(), &state), expected);
}

// covers: HER-ADV-008
#[test]
fn the_state_hash_covers_each_heros_bonuses_their_amounts_and_its_timed_marks() {
    let rules = bonus_rules();
    let well = rules.1.object_type_by_key("t:well").unwrap();
    let plain = jump_state(&rules, (1, None), &[], &[]);
    let changes: [fn(&mut AdventureState, ObjectTypeId); 5] = [
        |s, _| s.give_bonus(HERO, 1, 0),
        |s, _| s.give_bonus(HERO, 0, 1),
        |s, _| {
            s.give_once(HERO, once(0, -3, 0));
        },
        |s, _| {
            s.give_once(HERO, once(0, -2, 0));
        },
        |s, well| {
            s.change_marks(HERO, |m| m.add_timed(well));
        },
    ];
    let mut hashes = vec![hash_of(&plain)];
    for change in changes {
        let mut changed = plain.clone();
        change(&mut changed, well);
        assert_ne!(changed, plain);
        hashes.push(hash_of(&changed));
    }
    let mut unique = hashes.clone();
    unique.sort_unstable();
    unique.dedup();
    assert_eq!(unique.len(), hashes.len(), "{hashes:?}");
}
