//! Increment 12's kinds on the map (E08 T113a, increment 12's design note §19's round 5): each
//! bonus kind's tile is refused as any unbuilt visit's (ADV-OBJ-003), entered or visited from a
//! neighbour, and the reach ends before it, until T115b. Since E08 T114a1 (design note §5 to §7,
//! §14) the garrisons: a garrison of the mover's team crossed on the way, any other a
//! destination; another's or a neutral one fought at once, unsplit, an empty one flagged; the
//! battle's end keeping the survivors or flagging it after the levels; the anti-magic battle.

use std::sync::Arc;

use super::teleporters::{BLUE, HERO, ONTO, RED, Rules, jump_state, p, stack, walk};
use super::*;
use crate::command::{BattleStart, Defender};
use crate::ruleset::{BONUS_DAYS, ObjectKindDef, ObjectTypeDef, Visit};
use crate::testing::{
    GNOLL, PIKEMAN, WOLF, hero_at, hero_details, key, movement_rules_with, started, stream, world,
};

/// [`movement_rules_with`] (faithful) with one type of each of increment 12's seven kinds: the
/// garrison `t:garrison` (anti-magic), entered and from above, a bonus `t:buoy` visited from a
/// neighbour, and `t:fountain`, `t:well`, `t:spring`, `t:stables` and `t:sirens` entered.
fn bonus_rules() -> Rules {
    movement_rules_with(
        true,
        32,
        |_| {},
        |part| {
            let ty = |k: &str, visit, from_above, kind| ObjectTypeDef {
                key: key(k),
                name: "Object".into(),
                visit,
                from_above,
                kind_mark: true,
                kind,
            };
            part.object_types.extend([
                ty(
                    "t:garrison",
                    Visit::Enter,
                    true,
                    ObjectKindDef::Garrison { anti_magic: true },
                ),
                ty(
                    "t:buoy",
                    Visit::Neighbour,
                    true,
                    ObjectKindDef::Bonus {
                        mark: 2,
                        days: [(1, 0); BONUS_DAYS],
                        points: 0,
                        drain: false,
                    },
                ),
                ty(
                    "t:fountain",
                    Visit::Enter,
                    false,
                    ObjectKindDef::FountainOfFortune {
                        mark: 5,
                        values: vec![-1, 1, 2, 3],
                    },
                ),
                ty("t:well", Visit::Enter, false, ObjectKindDef::MagicWell),
                ty("t:spring", Visit::Enter, false, ObjectKindDef::MagicSpring),
                ty(
                    "t:stables",
                    Visit::Enter,
                    false,
                    ObjectKindDef::Stables {
                        points: 400,
                        from: key("t:pikeman"),
                        to: key("t:gnoll"),
                    },
                ),
                ty(
                    "t:sirens",
                    Visit::Neighbour,
                    false,
                    ObjectKindDef::Sirens {
                        mark: 13,
                        keep: (7, 10),
                    },
                ),
            ]);
        },
    )
}

// covers: ADV-BON-001
#[test]
fn a_step_onto_a_bonus_visitable_is_refused_until_its_visit() {
    let rules = bonus_rules();
    let heroes = [Arc::new(hero_details(&[]))];
    for k in [
        "t:buoy",
        "t:fountain",
        "t:well",
        "t:spring",
        "t:stables",
        "t:sirens",
    ] {
        // The object on (1, 1), visited from it; the hero on (0, 1) beside it.
        let mut state = jump_state(&rules, (1, None), &[], &[(k, ONTO[0])]);
        assert_eq!(
            walk(&mut state, &rules, &ONTO[..1], &mut stream()).err(),
            Some(AdventureCommandError::Visit { step: 0 }),
            "{k}"
        );
        // The reach ends before it: no path onto or through its tile.
        let reach = state.reach(&rules.0, &rules.1, &heroes, HERO).unwrap();
        assert_eq!(reach.path(ONTO[0]), None, "{k}");
        assert!(reach.path(p(2, 1)).is_some(), "{k}");
    }
}

/// [`movement_rules_with`] (faithful) with a garrison `t:garrison` and an anti-magic one
/// `t:anti_garrison`, both entered and from above (ADV-GAR-001); the part's flag radius 3.
fn garrison_rules() -> Rules {
    movement_rules_with(
        true,
        32,
        |_| {},
        |part| {
            let ty = |k: &str, anti_magic| ObjectTypeDef {
                key: key(k),
                name: "Garrison".into(),
                visit: Visit::Enter,
                from_above: true,
                kind_mark: false,
                kind: ObjectKindDef::Garrison { anti_magic },
            };
            part.object_types
                .extend([ty("t:garrison", false), ty("t:anti_garrison", true)]);
        },
    )
}

/// A corridor of grass between rock, so that a garrison on it bars the way.
const CORRIDOR: [&str; 3] = ["############", "gggggggggggg", "############"];

/// The garrisons' entrance in these tests, the third step right of red's hero on (0, 1).
const ENTRANCE: Pos = Pos::new(3, 1, 0);

/// A garrison of type `k` on `at`, blocking and entered there, `owner`'s or neutral, holding
/// `army`, removable.
fn garrison(
    rules: &Rules,
    k: &str,
    at: Pos,
    owner: Option<PlayerId>,
    army: &[ArmyStack],
) -> ObjectSetup {
    ObjectSetup {
        scenery: Scenery::default(),
        anchor: at,
        blocked: TileMask::from_bits(1),
        visitable: TileMask::from_bits(1),
        object_type: rules.1.object_type_by_key(k),
        amount: None,
        drawn: TileMask::default(),
        monster: None,
        guards: None,
        holds: Holds::Garrison {
            army: army.to_vec(),
            removable: true,
        },
        owner,
    }
}

/// The game of the [`CORRIDOR`] with red's hero [`HERO`] on (0, 1), blue's hero 2 on (11, 1),
/// and `objects` (the first at object index 0).
fn corridor(rules: &Rules, objects: Vec<ObjectSetup>) -> AdventureState {
    let heroes = vec![
        hero_at(1, 0, p(0, 1), &[PIKEMAN]),
        hero_at(2, 1, p(11, 1), &[PIKEMAN]),
    ];
    let mut setup = world(&CORRIDOR, 1, heroes);
    setup.map.objects.extend(objects);
    started(rules, &setup).unwrap()
}

/// The corridor's tiles from (1, 1) to (`to`, 1).
fn east(to: u16) -> Vec<Pos> {
    (1..=to).map(|x| p(x, 1)).collect()
}

/// The events but the walk's steps.
fn after_steps(events: &[AdventureEvent]) -> Vec<AdventureEventKind> {
    events
        .iter()
        .filter(|e| !matches!(e.kind, AdventureEventKind::HeroStepped { .. }))
        .map(|e| e.kind)
        .collect()
}

const GNOLLS: [ArmyStack; 1] = [stack(2, GNOLL, 30)];

// covers: ADV-GAR-004
#[test]
fn a_garrison_of_the_mover_s_team_is_crossed_and_every_other_is_a_destination() {
    let rules = garrison_rules();
    let details = [Arc::new(hero_details(&[]))];
    // An untyped blocker on the rock (object index 1): no map garrison.
    let blocker = ObjectSetup {
        object_type: None,
        holds: Holds::Nothing,
        ..garrison(&rules, "t:garrison", p(6, 0), None, &[])
    };
    let own = garrison(&rules, "t:garrison", ENTRANCE, Some(RED), &GNOLLS);
    let mut state = corridor(&rules, vec![own, blocker.clone()]);
    assert_eq!(state.garrison_army(0), GNOLLS);
    assert_eq!(state.garrison_army(1), []);
    // D-59: red's own garrison is a tile like any other, on the way: the path and the move go on
    // through it, with no visit (its army and owner unchanged, nothing told but the steps).
    let reach = state.reach(&rules.0, &rules.1, &details, HERO).unwrap();
    assert_eq!(reach.path(p(6, 1)), Some(east(6)));
    let before = state.clone();
    let (outcome, events) = walk(&mut state, &rules, &east(6), &mut stream()).unwrap();
    assert_eq!(outcome, AdventureOutcome::default());
    assert_eq!(after_steps(&events), []);
    assert_eq!(state.hero(HERO).unwrap().pos, p(6, 1));
    assert_eq!(
        (state.owner(0), state.garrison_army(0)),
        (Some(RED), &GNOLLS[..])
    );
    // A move that ends on it visits it (its exchange: T114a2, `ADV-STUB-17`); a border gate its
    // key's holder passes is never visited.
    let mut ending = before.clone();
    let (outcome, events) = walk(&mut ending, &rules, &east(3), &mut stream()).unwrap();
    assert_eq!(
        (outcome, after_steps(&events)),
        (AdventureOutcome::default(), vec![])
    );
    assert_eq!(ending.hero(HERO).unwrap().pos, ENTRANCE);
    let visit = ending.visit_at(&rules.1, RED, ENTRANCE).unwrap();
    assert!(visit.pass && visit.visited_at_end());
    let gate = StepVisit {
        effect: Effect::Gate(0),
        ..visit
    };
    assert!(!gate.visited_at_end());
    // Another team's or a neutral one, armed or empty, ends every path: the reach reaches its
    // entrance and goes no further, and a move past it is refused.
    for (owner, army) in [
        (Some(BLUE), &GNOLLS[..]),
        (None, &GNOLLS[..]),
        (None, &[][..]),
        (Some(BLUE), &[][..]),
    ] {
        let other = garrison(&rules, "t:garrison", ENTRANCE, owner, army);
        let mut state = corridor(&rules, vec![other, blocker.clone()]);
        assert!(!state.visit_at(&rules.1, RED, ENTRANCE).unwrap().pass);
        let reach = state.reach(&rules.0, &rules.1, &details, HERO).unwrap();
        assert_eq!(reach.path(ENTRANCE), Some(east(3)), "{owner:?}");
        assert_eq!(reach.path(p(6, 1)), None, "{owner:?}");
        assert_eq!(
            walk(&mut state, &rules, &east(6), &mut stream()).err(),
            Some(AdventureCommandError::PastVisit { step: 3 })
        );
    }
}

// covers: ADV-GAR-002, GAM-BAT-001
#[test]
fn another_s_or_a_neutral_garrison_holding_creatures_is_fought_at_once_unsplit() {
    let rules = garrison_rules();
    let army = [stack(0, WOLF, 4), stack(2, GNOLL, 30)];
    for (owner, defender) in [
        // A neutral single stack defends unsplit, as guards, not as a monster (E12-6).
        (
            None,
            Defender::Guards {
                army: GNOLLS.to_vec(),
            },
        ),
        (
            Some(BLUE),
            Defender::Garrison {
                owner: BLUE,
                army: army.to_vec(),
            },
        ),
    ] {
        let held = if owner.is_some() {
            &army[..]
        } else {
            &GNOLLS[..]
        };
        let mut state = corridor(
            &rules,
            vec![garrison(&rules, "t:garrison", ENTRANCE, owner, held)],
        );
        let before = state.clone();
        let (outcome, events) = walk(&mut state, &rules, &east(3), &mut stream()).unwrap();
        // Entered: the hero stands on the entrance and the battle starts there, no question.
        let hero = state.hero(HERO).unwrap();
        assert_eq!(hero.pos, ENTRANCE);
        let expected: BattleStart = state.battle_at(&rules.1, hero, defender, ENTRANCE);
        assert_eq!(
            outcome,
            AdventureOutcome {
                battle: Some(expected),
                ..AdventureOutcome::default()
            }
        );
        assert_eq!(after_steps(&events), []);
        // Nothing changes before the battle's end.
        assert_eq!((state.owner(0), state.garrison_army(0)), (owner, held));
        assert_eq!(state.garrison_army(0), before.garrison_army(0));
    }
}

// covers: ADV-GAR-002, ADV-OWN-001
#[test]
fn an_empty_garrison_of_another_team_is_flagged_with_its_reveal() {
    let rules = garrison_rules();
    let ty = rules.1.object_type_by_key("t:garrison").unwrap();
    for owner in [None, Some(BLUE)] {
        let mut state = corridor(
            &rules,
            vec![garrison(&rules, "t:garrison", ENTRANCE, owner, &[])],
        );
        let (outcome, events) = walk(&mut state, &rules, &east(3), &mut stream()).unwrap();
        // The flag and its reveal around the entrance; the exchange comes with T114a2.
        assert_eq!(outcome, AdventureOutcome::default());
        assert_eq!(state.owner(0), Some(RED));
        let told = after_steps(&events);
        assert_eq!(
            told[0],
            AdventureEventKind::ObjectFlagged {
                object_type: ty,
                anchor: ENTRANCE,
                owner: RED,
            }
        );
        assert!(
            matches!(told[1], AdventureEventKind::TilesRevealed { object_type, .. } if object_type == ty)
        );
        assert_eq!(told.len(), 2);
        assert!(state.check_owners(&rules.1));
    }
}

// covers: ADV-GAR-002
#[test]
fn a_garrison_battle_s_end_keeps_the_survivors_or_flags_it_after_the_levels() {
    let rules = garrison_rules();
    let details = [Arc::new(hero_details(&[]))];
    let ty = rules.1.object_type_by_key("t:garrison").unwrap();
    for owner in [None, Some(BLUE)] {
        let mut fought = corridor(
            &rules,
            vec![garrison(&rules, "t:garrison", ENTRANCE, owner, &GNOLLS)],
        );
        walk(&mut fought, &rules, &east(3), &mut stream()).unwrap();
        let end = |state: &mut AdventureState, survivors: &[ArmyStack], won| {
            let mut events = Vec::new();
            let rules = (&rules.0, &rules.1, &details[..]);
            let outcome =
                state.end_unowned_battle(rules, (HERO, ENTRANCE), (survivors, won), &mut events);
            (outcome, events)
        };
        // Lost (or a retreat): the defender's survivors stay as its army, by slot; its owner kept.
        let mut lost = fought.clone();
        let survivors = [stack(2, GNOLL, 12)];
        let (outcome, events) = end(&mut lost, &survivors, false);
        assert_eq!((outcome, events), (AdventureOutcome::default(), vec![]));
        assert_eq!(
            (lost.owner(0), lost.garrison_army(0)),
            (owner, &survivors[..])
        );
        assert_ne!(lost.state_hash(), fought.state_hash());
        // A draw leaves none: its next visitor flags it.
        let mut draw = fought.clone();
        end(&mut draw, &[], false);
        assert_eq!((draw.owner(0), draw.garrison_army(0)), (owner, &[][..]));
        // Won: the army gone, the flag after the winner's levels (the visit's rest).
        let (outcome, events) = end(&mut fought, &[], true);
        assert_eq!(
            outcome,
            AdventureOutcome {
                reward_after: Some((HERO, ENTRANCE)),
                ..AdventureOutcome::default()
            }
        );
        assert_eq!(events, []);
        assert_eq!((fought.owner(0), fought.garrison_army(0)), (owner, &[][..]));
        let mut events = Vec::new();
        let given = fought.give_won(
            (&rules.0, &rules.1, &details[..]),
            (HERO, ENTRANCE),
            &mut events,
        );
        assert_eq!(given, AdventureOutcome::default());
        assert_eq!(fought.owner(0), Some(RED));
        assert_eq!(
            events[0].kind,
            AdventureEventKind::ObjectFlagged {
                object_type: ty,
                anchor: ENTRANCE,
                owner: RED,
            }
        );
    }
}

// covers: ADV-GAR-005
#[test]
fn a_battle_on_an_anti_magic_garrison_s_entrance_is_fought_without_magic() {
    let rules = garrison_rules();
    let state = corridor(
        &rules,
        vec![
            garrison(&rules, "t:anti_garrison", ENTRANCE, None, &GNOLLS),
            garrison(&rules, "t:garrison", p(7, 1), None, &GNOLLS),
        ],
    );
    // Its entrance alone: the garrison's own battle and a hero's standing in it; not the tile
    // beside it (a hero attacking out of it fights with magic) nor a plain garrison's entrance.
    assert!(state.anti_magic(&rules.1, ENTRANCE));
    assert!(!state.anti_magic(&rules.1, p(2, 1)));
    assert!(!state.anti_magic(&rules.1, p(7, 1)));
    assert!(!state.anti_magic(&rules.1, p(5, 1)));
}
