//! Tests of Dimension Door (E08 T143b; increment 15's design note §10.5, §11.2; ADV-SPL-007 to
//! ADV-SPL-009): the refusals in the original's order, the window, the free tiles, the arrival
//! without a guard's battle or a visit and with its reveal, a boat taken along, the points and
//! the casts of the day, the wrong kind under `ADV-QUIRK-55` and the hidden targets under
//! `ADV-QUIRK-51` with their reveal. Batch E15's rows as sources (batch 15b): E15-5 (four casts at
//! Expert for 200 points each, the fifth refused), E15-6 (one point left: cast, 0 left; then
//! refused), E15-7 (water from land fails, charged: A-52), E15-8 (a hidden free tile a target, a
//! hidden tree none: A-53), E15-16 (the original's tenth row, fixed: A-54).

use alloc::sync::Arc;
use core::num::NonZeroU16;

use h3_content::{SpellCategory, SpellDef, SpellId, SpellSchoolDef};
use h3_heroes::HeroDetails;

use super::*;
use crate::command::{AdventureEvent, AdventureEventKind, Viewers};
use crate::ruleset::{AdventureSpell, SpellParams};
use crate::testing::{PEASANT, hero_at, hero_details, key, movement_rules_with, started, world};

type Rules = (Ruleset, AdventureRuleset);

const RED: PlayerId = PlayerId::new(0);
const BLUE: PlayerId = PlayerId::new(1);
const HERO: HeroId = HeroId::new(1);
/// The core's test spell: Dimension Door's casts by mastery (1, 2, 3, 4) as the original's.
const DOOR: SpellId = SpellId::from_raw(0);
/// The points the tests give the caster before a cast.
const POINTS: u32 = 1500;

fn p(x: u16, y: u16) -> Pos {
    Pos::new(x, y, 0)
}

/// The movement rules (`faithful` with both of the door's flags, or none) with the door and
/// `[params.spells]`.
fn rules(faithful: bool) -> Rules {
    movement_rules_with(
        faithful,
        16,
        |core| {
            core.spell_schools = vec![SpellSchoolDef {
                key: key("t:air"),
                name: "Air".into(),
                protection_rank: 0,
            }];
            core.spells = vec![SpellDef {
                key: key("t:door"),
                name: "Spell".into(),
                category: SpellCategory::Adventure,
                level: 5,
                schools: vec![key("t:air")],
                cost: [25, 20, 20, 20],
                power: 0,
                effect: [1, 2, 3, 4],
                granted_by_name_only: false,
            }];
        },
        |part| {
            part.params.as_mut().unwrap().spells = Some(SpellParams {
                door_points: [300, 300, 300, 200],
                target_range: [9, 8],
                boats_most: NonZeroU16::new(64).unwrap(),
            });
        },
    )
}

fn store() -> Vec<Arc<HeroDetails>> {
    (0..3).map(|_| Arc::new(hero_details(&[]))).collect()
}

/// The door's map, 12 × 10: red's hero 1 on (0, 0) on land, blue's hero 2 on (9, 8). Explored by
/// red (to dx² + dy² ≤ 30): (1, 0) an object's blocked tile, (2, 0) its visitable tile, (3, 0)
/// rock, (4, 0) water, (5, 0) an empty boat, (1, 1) a monster, (2, 2) free grass beside it.
/// Hidden from red: (7, 0) water, (8, 0) an object's blocked tile, (9, 0) free grass, blue's
/// hero's tile and (8, 7) beside it, which blue sees. (10, 0) and (0, 9) lie outside the window
/// (10 columns, 9 rows away).
fn door_world(rules: &Rules) -> AdventureState {
    let mut rows = vec!["gBV#~bg~Bggg", "gMgggggggggg"];
    rows.extend(["gggggggggggg"; 8]);
    let heroes = vec![
        hero_at(1, 0, p(0, 0), &[PEASANT]),
        hero_at(2, 1, p(9, 8), &[PEASANT]),
    ];
    let mut state = started(rules, &world(&rows, 1, heroes)).unwrap();
    state.heroes[0].movement = POINTS;
    for (pos, seen) in [(p(5, 0), true), (p(2, 2), true), (p(7, 0), false)] {
        assert_eq!(explored(&state, RED, pos), seen, "{pos:?}");
    }
    for pos in [p(8, 0), p(9, 0), p(9, 8), p(8, 7)] {
        assert!(!explored(&state, RED, pos), "{pos:?}");
    }
    state
}

const fn door(mastery: u8, target: Pos) -> Cast {
    Cast {
        spell: DOOR,
        effect: AdventureSpell::DimensionDoor,
        mastery,
        target: Some(target),
    }
}

/// What red's hero 1's cast told and did.
type Made = (Result<(), CastFailure>, Vec<AdventureEvent>);

/// Red's hero 1's cast `cast`, checked, then made as the game makes it; nothing is drawn.
fn cast(state: &mut AdventureState, rules: &Rules, cast: Cast) -> Result<Made, CastError> {
    let store = store();
    state.check_cast((&rules.0, &rules.1), &store, (RED, HERO), cast)?;
    let mut draws = Rng::from_seed(7);
    let unused = draws.clone();
    let mut events = Vec::new();
    let made = state.apply_cast(
        (&rules.0, &rules.1, &store),
        (RED, HERO),
        cast,
        &mut draws,
        &mut events,
    );
    assert_eq!(draws, unused, "the door draws nothing");
    Ok((made, events))
}

fn explored(state: &AdventureState, player: PlayerId, pos: Pos) -> bool {
    state.explored(player).unwrap().get(pos)
}

/// The hero's tile and points left, and its casts of the day.
fn caster(state: &AdventureState) -> (Pos, u32, u8) {
    let hero = state.hero(HERO).unwrap();
    (hero.pos, hero.movement, state.day_spells(HERO).doors)
}

/// The door's arrival as told: red's hero from `from` to `to`, `seen` by the viewers of both ends.
fn teleported(from: Pos, to: Pos, seen: Viewers) -> AdventureEvent {
    AdventureEvent {
        kind: AdventureEventKind::HeroTeleported {
            hero: HERO,
            from: Some(from),
            to: Some(to),
            seen_from: seen,
            seen_to: seen,
        },
        seen_by: seen,
    }
}

// covers: ADV-SPL-007, ADV-SPL-008
#[test]
fn the_door_is_refused_without_points_casts_or_a_target_in_its_window() {
    // GENRLTXT 126 (the points not above 0), then 339 (the day's casts at the mastery's count,
    // checked before the target), then the window 9 × 8 (`ADV-QUIRK-56` fixed: no tenth row), on
    // the hero's level and the map. Free: nothing changes.
    for faithful in [true, false] {
        let rules = rules(faithful);
        let mut state = door_world(&rules);
        let before = state.clone();
        state.heroes[0].movement = 0;
        assert_eq!(
            cast(&mut state, &rules, door(3, p(10, 0))),
            Err(CastError::NoPoints)
        );
        state.heroes[0].movement = 1;
        for mastery in 0..4 {
            let spells = DaySpells {
                doors: mastery + 1,
                ..DaySpells::NONE
            };
            state.set_day_spells(HERO, spells);
            assert_eq!(
                cast(&mut state, &rules, door(mastery, p(10, 0))),
                Err(CastError::NoCastsLeft)
            );
        }
        let mut state = before.clone();
        for target in [p(10, 0), p(0, 9), Pos::new(2, 2, 1)] {
            assert_eq!(
                cast(&mut state, &rules, door(3, target)),
                Err(CastError::OutOfWindow),
                "{target:?}"
            );
        }
        let mut targetless = door(3, p(2, 2));
        targetless.target = None;
        assert_eq!(
            cast(&mut state, &rules, targetless),
            Err(CastError::OutOfWindow)
        );
        assert_eq!(state, before);
        // Off the map, inside the window.
        state.heroes[0].pos = p(11, 9);
        assert_eq!(
            cast(&mut state, &rules, door(3, p(12, 9))),
            Err(CastError::OutOfWindow)
        );
    }
}

// covers: ADV-SPL-008
#[test]
fn the_door_is_refused_on_a_tile_that_is_not_free() {
    // The dialog's missing cursor, on explored tiles: an object's blocked tile, its visitable
    // tile, rock, an empty boat, a monster, the caster's own tile (another hero's: the hidden
    // targets' test).
    for faithful in [true, false] {
        let rules = rules(faithful);
        let mut state = door_world(&rules);
        let before = state.clone();
        for target in [p(1, 0), p(2, 0), p(3, 0), p(5, 0), p(1, 1), p(0, 0)] {
            assert_eq!(
                cast(&mut state, &rules, door(3, target)),
                Err(CastError::NotFree),
                "{target:?}"
            );
        }
        assert_eq!(state, before);
    }
}

// covers: ADV-SPL-009, ADV-SPL-007
#[test]
fn the_door_relocates_the_hero_without_a_battle_or_a_visit() {
    // Beside the monster on (1, 1): no battle (the outcome holds none, the arrival's), the
    // tiles around the target explored by the scouting radius, told to red; 300 points at None,
    // the day's one cast, then refused (339); at Expert 200 points, four casts; at least 0 left.
    let rules = rules(true);
    let mut state = door_world(&rules);
    let made = cast(&mut state, &rules, door(0, p(2, 2))).unwrap();
    let red = Viewers::NONE.with(RED);
    assert_eq!(made, (Ok(()), vec![teleported(p(0, 0), p(2, 2), red)]));
    assert_eq!(caster(&state), (p(2, 2), POINTS - 300, 1));
    assert!(explored(&state, RED, p(7, 2)) && !explored(&state, RED, p(8, 2)));
    assert_eq!(
        cast(&mut state, &rules, door(0, p(4, 4))),
        Err(CastError::NoCastsLeft)
    );
    for (n, target) in [p(0, 4), p(0, 7), p(2, 2)].into_iter().enumerate() {
        let from = caster(&state).0;
        let made = cast(&mut state, &rules, door(3, target)).unwrap();
        assert_eq!(made, (Ok(()), vec![teleported(from, target, red)]));
        let left = (POINTS - 300).saturating_sub(200 * (u32::try_from(n).unwrap() + 1));
        assert_eq!(caster(&state), (target, left, u8::try_from(n).unwrap() + 2));
    }
    assert_eq!(caster(&state).1, 600);
    assert_eq!(
        cast(&mut state, &rules, door(3, p(9, 9))),
        Err(CastError::NoCastsLeft)
    );
    // A hero with fewer points than the cost arrives with 0.
    let mut state = door_world(&rules);
    state.heroes[0].movement = 1;
    let made = cast(&mut state, &rules, door(3, p(2, 2))).unwrap();
    assert_eq!(made.0, Ok(()));
    assert_eq!(caster(&state), (p(2, 2), 0, 1));
    // Blue sees the caster land beside its hero on (9, 8), not leave (0, 0).
    let mut state = door_world(&rules);
    let made = cast(&mut state, &rules, door(3, p(8, 7))).unwrap();
    let AdventureEventKind::HeroTeleported {
        seen_from, seen_to, ..
    } = made.1[0].kind
    else {
        panic!("{made:?}");
    };
    assert!(seen_to.contains(BLUE) && !seen_from.contains(BLUE));
}

// covers: ADV-SPL-009
#[test]
fn a_hero_in_a_boat_takes_it_through_the_door() {
    // Red's hero aboard the boat on (5, 0) of the door's map: a water target takes hero and boat
    // (its record keeps the boarded tile, the hero's tile is the boat's), an explored land one is
    // the wrong kind.
    for faithful in [true, false] {
        let rules = rules(faithful);
        let mut state = door_world(&rules);
        state.put_aboard(HERO, 0);
        let before = state.clone();
        let made = cast(&mut state, &rules, door(3, p(4, 0))).unwrap();
        assert_eq!(made.0, Ok(()));
        let hero = state.hero(HERO).unwrap();
        assert_eq!((hero.pos, hero.boat), (p(4, 0), Some(0)));
        assert!(state.check_boats(&rules.1) && !state.is_empty_boat(p(5, 0)));
        let mut state = before;
        let wrong = cast(&mut state, &rules, door(3, p(2, 2)));
        if faithful {
            assert_eq!(wrong.unwrap().0, Err(CastFailure::Target));
        } else {
            assert_eq!(wrong, Err(CastError::WrongKind));
        }
    }
}

// covers: ADV-SPL-009
#[test]
fn a_wrong_kind_fails_charged_under_adv_quirk_55_or_is_refused_without_it() {
    // E15-14 (faithful): water from land fails after the dialog (GENRLTXT 71), the points and
    // the cast spent, the hero where it was, nothing told by the adventure. Off (strict-fixed):
    // an explored target of the wrong kind is not offered.
    let rules_on = rules(true);
    let mut state = door_world(&rules_on);
    let made = cast(&mut state, &rules_on, door(3, p(4, 0))).unwrap();
    assert_eq!(made, (Err(CastFailure::Target), vec![]));
    assert_eq!(caster(&state), (p(0, 0), POINTS - 200, 1));

    let rules_off = rules(false);
    let mut state = door_world(&rules_off);
    let before = state.clone();
    assert_eq!(
        cast(&mut state, &rules_off, door(3, p(4, 0))),
        Err(CastError::WrongKind)
    );
    assert_eq!(state, before);
}

// covers: ADV-SPL-008, ADV-SPL-009
#[test]
fn a_hidden_target_reads_the_truth_under_adv_quirk_51_or_is_revealed_without_it() {
    // Faithful: a hidden blocked tile or hero's tile is no target (the cursor tells, A-53); a
    // hidden free one of the wrong kind fails, charged, nothing revealed; a hidden free one of the
    // hero's kind is the arrival. Off (strict-fixed): every hidden tile of the window a target; a
    // blocked one, a hero's or one of the wrong kind fails, charged, that tile alone revealed
    // (decision 399); a free one is the arrival.
    let rules_on = rules(true);
    let mut state = door_world(&rules_on);
    for target in [p(8, 0), p(9, 8)] {
        assert_eq!(
            cast(&mut state, &rules_on, door(3, target)),
            Err(CastError::NotFree)
        );
    }
    let made = cast(&mut state, &rules_on, door(3, p(7, 0))).unwrap();
    assert_eq!(made, (Err(CastFailure::Target), vec![]));
    assert!(!explored(&state, RED, p(7, 0)));
    let made = cast(&mut state, &rules_on, door(3, p(9, 0))).unwrap();
    let red = Viewers::NONE.with(RED);
    assert_eq!(made, (Ok(()), vec![teleported(p(0, 0), p(9, 0), red)]));
    assert_eq!(caster(&state), (p(9, 0), POINTS - 400, 2));

    let rules_off = rules(false);
    let mut state = door_world(&rules_off);
    for (n, target) in [p(8, 0), p(7, 0), p(9, 8)].into_iter().enumerate() {
        let made = cast(&mut state, &rules_off, door(3, target)).unwrap();
        assert_eq!(made, (Err(CastFailure::Target), vec![]), "{target:?}");
        assert!(explored(&state, RED, target), "{target:?}");
        assert_eq!(caster(&state).2, u8::try_from(n).unwrap() + 1);
    }
    assert!(!explored(&state, RED, p(9, 0)) && !explored(&state, RED, p(8, 1)));
    assert_eq!(caster(&state), (p(0, 0), POINTS - 600, 3));
    // Now seen, the blocked tile is refused as any the player sees.
    assert_eq!(
        cast(&mut state, &rules_off, door(3, p(8, 0))),
        Err(CastError::NotFree)
    );
    let made = cast(&mut state, &rules_off, door(3, p(9, 0))).unwrap();
    assert_eq!(made, (Ok(()), vec![teleported(p(0, 0), p(9, 0), red)]));
    assert_eq!(caster(&state), (p(9, 0), POINTS - 800, 4));
}

/// The targets [`AdventureState::cast_targets`] gives red's hero 1 for `cast` (E08 T144a1):
/// exactly the window's tiles `check_cast` accepts, in row order.
fn listed(state: &AdventureState, rules: &Rules, cast: Cast) -> Vec<Pos> {
    let store = store();
    let at = |target| Cast {
        target: Some(target),
        ..cast
    };
    let accepted: Vec<Pos> = state
        .cast_window(&rules.1, state.hero(HERO).unwrap().pos)
        .filter(|&t| {
            let checked = state.check_cast((&rules.0, &rules.1), &store, (RED, HERO), at(t));
            checked.is_ok()
        })
        .collect();
    let unset = Cast {
        target: None,
        ..cast
    };
    let mut targets = Vec::new();
    state.cast_targets((&rules.0, &rules.1), (RED, HERO), unset, |t| {
        targets.push(t)
    });
    assert_eq!(targets, accepted);
    targets
}

// covers: ADV-SPL-004, ADV-SPL-008
#[test]
fn the_listed_targets_are_the_window_s_tiles_the_test_accepts() {
    let scuttle = Cast {
        effect: AdventureSpell::ScuttleBoat,
        ..door(3, p(0, 0))
    };
    for faithful in [true, false] {
        let rules = rules(faithful);
        let mut state = door_world(&rules);
        let doors = listed(&state, &rules, door(3, p(0, 0)));
        assert!(doors.contains(&p(2, 2)) && doors.contains(&p(9, 0)));
        for outside in [p(0, 0), p(10, 0), p(0, 9)] {
            assert!(!doors.contains(&outside), "{outside:?}");
        }
        // A hidden blocked tile: read under `ADV-QUIRK-51`, a target without it.
        assert_eq!(doors.contains(&p(8, 0)), !faithful);
        assert!(listed(&state, &rules, scuttle).contains(&p(5, 0)));
        // Without points: no door, the scuttle's targets as before.
        state.heroes[0].movement = 0;
        assert!(listed(&state, &rules, door(3, p(0, 0))).is_empty());
        assert!(!listed(&state, &rules, scuttle).is_empty());
    }
    // Another player's hero, a spell without a target: none.
    let rules = rules(true);
    let state = door_world(&rules);
    let parts = (&rules.0, &rules.1);
    let fly = Cast {
        effect: AdventureSpell::Fly,
        ..scuttle
    };
    for (player, cast) in [(BLUE, scuttle), (RED, fly)] {
        let mut targets = Vec::new();
        state.cast_targets(parts, (player, HERO), cast, |t| targets.push(t));
        assert!(targets.is_empty(), "{player:?}");
    }
}

#[test]
fn the_door_s_errors_explain_themselves() {
    let cases = [
        (CastError::NoPoints, "the hero has no movement points left"),
        (
            CastError::NoCastsLeft,
            "the hero cast the spell today as often as its mastery allows",
        ),
        (CastError::NotFree, "the target is blocked or an entrance"),
        (
            CastError::WrongKind,
            "the target is land from a boat or water from land",
        ),
    ];
    for (error, text) in cases {
        assert_eq!(error.to_string(), text);
    }
}
