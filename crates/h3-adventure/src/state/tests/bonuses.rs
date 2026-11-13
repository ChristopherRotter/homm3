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

// covers: ADV-BON-005, ADV-BON-006, HER-ADV-003, ADV-MOV-002
#[test]
fn the_days_and_the_weeks_marks_drop_at_their_steps_and_the_stables_raise_the_land_points() {
    let rules = bonus_rules();
    let ty = |k| rules.1.object_type_by_key(k).unwrap();
    let (well, stables, buoy) = (ty("t:well"), ty("t:stables"), ty("t:buoy"));
    let mut state = jump_state(&rules, (1, None), &[], &[]);
    let base = state.hero(HERO).unwrap().max_movement;
    let timed = |state: &AdventureState| state.hero(HERO).unwrap().timed.clone();
    // A mark once per type, kept sorted (the visits come with T115b).
    let hero = state.hero_mut(HERO).unwrap();
    assert!(add_timed(&mut hero.timed, stables));
    assert!(add_timed(&mut hero.timed, well));
    assert!(!add_timed(&mut hero.timed, well));
    let mut sorted = vec![well, stables];
    sorted.sort_unstable();
    assert_eq!(hero.timed, sorted);
    assert_eq!(week_land_points(&rules.1, &hero.timed), 400);
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
    assert_eq!(state.hero(HERO).unwrap().max_movement, base);
    // One record alone (the town lane's garrison hero): a kind of no lifetime and a type the
    // part lacks are kept and count nothing.
    let stranger = ObjectTypeId::from_raw(u16::MAX);
    let mut record = vec![buoy, well, stables, stranger];
    record.sort_unstable();
    drop_timed(&rules.1, &mut record, MarkLifetime::Week);
    drop_timed(&rules.1, &mut record, MarkLifetime::Day);
    let mut kept = vec![buoy, stranger];
    kept.sort_unstable();
    assert_eq!(record, kept);
    assert_eq!(week_land_points(&rules.1, &record), 0);
    // The week's points saturate.
    let twice = [stables, stables];
    assert_eq!(week_land_points(&rules.1, &twice), 800);
}

// covers: GAM-BAT-014
#[test]
fn a_battles_end_clears_the_bonuses_of_its_heroes_still_on_the_map() {
    let rules = bonus_rules();
    let well = rules.1.object_type_by_key("t:well").unwrap();
    let other = HeroId::new(2);
    let mut state = jump_state(&rules, (1, None), &[(2, 1, p(9, 1))], &[]);
    for id in [HERO, other] {
        let hero = state.hero_mut(id).unwrap();
        hero.bonuses.add(1, 1);
        hero.bonuses.add_once(once(0, -3, 0));
        add_timed(&mut hero.timed, well);
    }
    let kept = state.hero(other).unwrap().clone();
    // Its heroes on the map lose them; a hero not on the map is nothing; the timed marks stay.
    state.clear_battle_bonuses(&[HERO, HeroId::new(9)]);
    let hero = state.hero(HERO).unwrap();
    assert_eq!(
        (&hero.bonuses, &hero.timed[..]),
        (&HeroBonuses::NONE, &[well][..])
    );
    assert_eq!(state.hero(other), Some(&kept));
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
    let hero = state.hero_mut(HERO).unwrap();
    hero.bonuses.add(1, 2);
    hero.bonuses.add_once(once(13, 0, 0));
    hero.bonuses.add_once(once(0, -3, 0));
    add_timed(&mut hero.timed, well);
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
    let hero = state.hero_mut(HERO).unwrap();
    hero.bonuses.add_once(once(1, 0, -2));
    hero.bonuses.add_once(once(13, 0, 1));
    add_timed(&mut hero.timed, stables);
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
