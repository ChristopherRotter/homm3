//! Increment 12's garrisons on the map (the bonus kinds' visits since E08 T115b: `visitables.rs`).
//! Since E08 T114a1 (design note §5 to §7,
//! §14) the garrisons: a garrison of the mover's team crossed on the way, any other a
//! destination; another's or a neutral one fought at once, unsplit, an empty one flagged; the
//! battle's end keeping the survivors or flagging it after the levels; the anti-magic battle.
//! Since E08 T114a2 (design note §13) the exchange: opened at a team's garrison where a move ends
//! or by a revisit, after an empty one's flag and after a won battle's levels; its army commands
//! by the one owner rule.

use std::sync::Arc;

use super::revisits::{revisit, revisitable};
use super::teleporters::{BLUE, HERO, RED, Rules, p, stack, walk};
use super::*;
use crate::army::{ArmyCommand, ArmyCommandError, ArmyRef, ArmySlot};
use crate::command::{BattleStart, Defender, Exchange};
use crate::ruleset::{BONUS_DAYS, ObjectKindDef, ObjectTypeDef, Visit};
use crate::testing::{
    GNOLL, PIKEMAN, WOLF, hero_at, hero_details, key, movement_rules_with, started, stream, world,
};

/// [`movement_rules_with`] (faithful) with one type of each of increment 12's seven kinds: the
/// garrison `t:garrison` (anti-magic), entered and from above, a bonus `t:buoy` visited from a
/// neighbour, and `t:fountain`, `t:well`, `t:spring`, `t:stables` and `t:sirens` entered.
pub(super) fn bonus_rules() -> Rules {
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
    fixed(rules, (k, at), owner, army, true)
}

/// [`garrison`] with its removable flag `removable`.
fn fixed(
    rules: &Rules,
    (k, at): (&str, Pos),
    owner: Option<PlayerId>,
    army: &[ArmyStack],
    removable: bool,
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
            removable,
        },
        owner,
    }
}

/// The exchange of red's hero [`HERO`] with the garrison at object index 0 on [`ENTRANCE`], of
/// type `k`.
fn exchange(rules: &Rules, k: &str) -> AdventureOutcome {
    AdventureOutcome {
        exchange: Some(Exchange {
            hero: HERO,
            garrison: 0,
            object_type: rules.1.object_type_by_key(k).unwrap(),
            anchor: ENTRANCE,
        }),
        ..AdventureOutcome::default()
    }
}

/// [`exchange`] after the visit flagged the garrison (E08 T114b: its views changed, ADV-GAR-006).
fn flagged(rules: &Rules, k: &str) -> AdventureOutcome {
    AdventureOutcome {
        garrison_flagged: true,
        ..exchange(rules, k)
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
    // A move that ends on it visits it: its exchange (ADV-GAR-003), nothing else told; a border
    // gate its key's holder passes is never visited.
    let mut ending = before.clone();
    let (outcome, events) = walk(&mut ending, &rules, &east(3), &mut stream()).unwrap();
    assert_eq!(
        (outcome, after_steps(&events)),
        (exchange(&rules, "t:garrison"), vec![])
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
        // The flag and its reveal around the entrance, then the exchange (ADV-GAR-003).
        assert_eq!(outcome, flagged(&rules, "t:garrison"));
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
        let mut gone = fought.clone();
        let given = fought.give_won(
            (&rules.0, &rules.1, &details[..]),
            (HERO, ENTRANCE),
            &mut events,
        );
        // Then the exchange (ADV-GAR-003), for a winner still on the map.
        assert_eq!(given, flagged(&rules, "t:garrison"));
        assert_eq!(fought.owner(0), Some(RED));
        assert_eq!(
            events[0].kind,
            AdventureEventKind::ObjectFlagged {
                object_type: ty,
                anchor: ENTRANCE,
                owner: RED,
            }
        );
        gone.heroes.retain(|h| h.id != HERO);
        let given = gone.give_won(
            (&rules.0, &rules.1, &details[..]),
            (HERO, ENTRANCE),
            &mut Vec::new(),
        );
        assert_eq!(given, AdventureOutcome::default());
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

// covers: ADV-GAR-003, ADV-GAR-004, ADV-TEL-008
#[test]
fn a_passed_garrison_of_the_hero_s_team_is_revisited_into_its_exchange() {
    let rules = garrison_rules();
    let own = garrison(&rules, "t:garrison", ENTRANCE, Some(RED), &GNOLLS);
    let mut state = corridor(&rules, vec![own]);
    // On the way to (5, 1) the hero passes it; standing on (5, 1) nothing is revisited.
    walk(&mut state, &rules, &east(5), &mut stream()).unwrap();
    assert_eq!(revisitable(&state, &rules, RED), []);
    // Standing on its entrance (a walk back), the revisit is its exchange (the original's space
    // key over the hero's own garrison), with no step and no points spent.
    let back = [p(4, 1), ENTRANCE];
    let (outcome, _) = walk(&mut state, &rules, &back, &mut stream()).unwrap();
    assert_eq!(outcome, exchange(&rules, "t:garrison"));
    assert_eq!(revisitable(&state, &rules, RED), [HERO]);
    let points = state.hero(HERO).unwrap().movement;
    let (outcome, events) = revisit(&mut state, &rules, (RED, HERO), &mut stream()).unwrap();
    assert_eq!((outcome, events), (exchange(&rules, "t:garrison"), vec![]));
    assert_eq!(state.hero(HERO).unwrap().movement, points);
}

/// A slot of `army`.
const fn at(army: ArmyRef, slot: u8) -> ArmySlot {
    ArmySlot { army, slot }
}

/// Red's hero's army.
const RED_ARMY: ArmyRef = ArmyRef::Hero(HERO);

/// The garrison at object index 0.
const GARRISON: ArmyRef = ArmyRef::MapGarrison(0);

/// Red's army command `command` on `state`, the hero's army and the garrison's after it (or the
/// error, the state unchanged).
fn manage(
    state: &mut AdventureState,
    rules: &Rules,
    command: ArmyCommand,
) -> Result<(Vec<ArmyStack>, Vec<ArmyStack>), ArmyCommandError> {
    let before = state.clone();
    let result = state.manage_army(&rules.1, RED, &command, None);
    if result.is_err() {
        assert_eq!(*state, before);
    }
    result.map(|()| {
        let hero = state.hero(HERO).unwrap().army.clone();
        (hero, state.garrison_army(0).to_vec())
    })
}

// covers: ADV-GAR-003, ADV-ARM-001, ADV-ARM-002, ADV-ARM-003, ADV-ARM-004
#[test]
fn an_own_removable_garrison_takes_the_army_dialog_s_full_rules() {
    let rules = garrison_rules();
    let own = garrison(&rules, "t:garrison", ENTRANCE, Some(RED), &GNOLLS);
    let mut state = corridor(&rules, vec![own]);
    let before = state.clone();
    // Its stack into the hero's army (a move), back by a split, merged, swapped, dismissed: the
    // garrison may be emptied, the state hash moves with its army.
    let hero = |army: &[(u8, CreatureTypeId, u32)]| -> Vec<ArmyStack> {
        army.iter().map(|&(slot, c, n)| stack(slot, c, n)).collect()
    };
    let moved = ArmyCommand::Move {
        from: at(GARRISON, 2),
        to: at(RED_ARMY, 3),
    };
    assert_eq!(
        manage(&mut state, &rules, moved),
        Ok((hero(&[(0, PIKEMAN, 1), (3, GNOLL, 30)]), vec![]))
    );
    assert_ne!(state.state_hash(), before.state_hash());
    let split = ArmyCommand::Split {
        from: at(RED_ARMY, 3),
        to: at(GARRISON, 6),
        count: 10,
    };
    assert_eq!(
        manage(&mut state, &rules, split),
        Ok((
            hero(&[(0, PIKEMAN, 1), (3, GNOLL, 20)]),
            hero(&[(6, GNOLL, 10)])
        ))
    );
    let merged = ArmyCommand::Move {
        from: at(GARRISON, 6),
        to: at(RED_ARMY, 3),
    };
    assert_eq!(
        manage(&mut state, &rules, merged),
        Ok((hero(&[(0, PIKEMAN, 1), (3, GNOLL, 30)]), vec![]))
    );
    let back = ArmyCommand::Move {
        from: at(RED_ARMY, 3),
        to: at(GARRISON, 0),
    };
    manage(&mut state, &rules, back).unwrap();
    let swapped = ArmyCommand::Move {
        from: at(RED_ARMY, 0),
        to: at(GARRISON, 0),
    };
    assert_eq!(
        manage(&mut state, &rules, swapped),
        Ok((hero(&[(0, GNOLL, 30)]), hero(&[(0, PIKEMAN, 1)])))
    );
    let dismissed = ArmyCommand::Dismiss {
        at: at(GARRISON, 0),
    };
    assert_eq!(
        manage(&mut state, &rules, dismissed),
        Ok((hero(&[(0, GNOLL, 30)]), vec![]))
    );
    // The hero's last stack never leaves it (ADV-ARM-002).
    let last = ArmyCommand::Move {
        from: at(RED_ARMY, 0),
        to: at(GARRISON, 1),
    };
    assert_eq!(
        manage(&mut state, &rules, last),
        Err(ArmyCommandError::LastStack)
    );
    // The listing and the split's range follow the same rules.
    let listed = state.army_commands(&rules.1, RED, (RED_ARMY, Some(GARRISON)), None);
    assert!(listed.iter().all(|c| matches!(c, ArmyCommand::Move { .. })));
    assert_eq!(listed.len(), 6, "{listed:?}");
    let range = state.split_range(&rules.1, RED, (at(RED_ARMY, 0), at(GARRISON, 4)), None);
    assert_eq!(range, Some(1..=29));
}

// covers: ADV-GAR-003
#[test]
fn a_bar_not_the_player_s_only_takes_stacks_into_its_empty_slots() {
    let rules = garrison_rules();
    let army = [stack(0, PIKEMAN, 5), stack(2, GNOLL, 30)];
    let refusals = |state: &mut AdventureState| {
        let commands = [
            // Nothing leaves it: a move, a split, a dismissal, a move within it.
            ArmyCommand::Move {
                from: at(GARRISON, 2),
                to: at(RED_ARMY, 4),
            },
            ArmyCommand::Split {
                from: at(GARRISON, 2),
                to: at(RED_ARMY, 4),
                count: 1,
            },
            ArmyCommand::Dismiss {
                at: at(GARRISON, 2),
            },
            ArmyCommand::Move {
                from: at(GARRISON, 2),
                to: at(GARRISON, 5),
            },
        ];
        for command in commands {
            assert_eq!(
                manage(state, &rules, command),
                Err(ArmyCommandError::NotOwn),
                "{command:?}"
            );
        }
        // Into it only an empty slot: no merge, no swap, no split onto a stack there.
        let commands = [
            ArmyCommand::Move {
                from: at(RED_ARMY, 0),
                to: at(GARRISON, 0),
            },
            ArmyCommand::Move {
                from: at(RED_ARMY, 0),
                to: at(GARRISON, 2),
            },
            ArmyCommand::Split {
                from: at(RED_ARMY, 0),
                to: at(GARRISON, 0),
                count: 1,
            },
        ];
        for command in commands {
            assert_eq!(
                manage(state, &rules, command),
                Err(ArmyCommandError::OnlyEmpty),
                "{command:?}"
            );
        }
        // A slot past the army's is still the plan's.
        let past = ArmyCommand::Move {
            from: at(RED_ARMY, 0),
            to: at(GARRISON, 7),
        };
        assert_eq!(
            manage(state, &rules, past),
            Err(ArmyCommandError::NoSlot(7))
        );
    };
    // An own non-removable garrison (its bar owner nobody, D-61), another player's and a
    // neutral one.
    for (owner, removable) in [(Some(RED), false), (Some(BLUE), true), (None, true)] {
        let held = fixed(&rules, ("t:garrison", ENTRANCE), owner, &army, removable);
        let mut state = corridor(&rules, vec![held]);
        state.heroes[0].army = vec![stack(0, PIKEMAN, 4), stack(1, WOLF, 2)];
        refusals(&mut state);
        // A move and a split into an empty slot stay; they are never taken back.
        let moved = ArmyCommand::Move {
            from: at(RED_ARMY, 1),
            to: at(GARRISON, 6),
        };
        let (hero, garrison) = manage(&mut state, &rules, moved).unwrap();
        assert_eq!(hero, [stack(0, PIKEMAN, 4)]);
        assert_eq!(garrison, [army[0], army[1], stack(6, WOLF, 2)]);
        let split = ArmyCommand::Split {
            from: at(RED_ARMY, 0),
            to: at(GARRISON, 4),
            count: 3,
        };
        let (hero, garrison) = manage(&mut state, &rules, split).unwrap();
        assert_eq!(hero, [stack(0, PIKEMAN, 1)]);
        assert_eq!(garrison[2], stack(4, PIKEMAN, 3));
        let back = ArmyCommand::Move {
            from: at(GARRISON, 6),
            to: at(RED_ARMY, 1),
        };
        assert_eq!(
            manage(&mut state, &rules, back),
            Err(ArmyCommandError::NotOwn)
        );
        // The listing: the hero's moves into the garrison's empty slots alone (its last stack
        // stays), none of the garrison's own; no split from it or onto its stacks.
        assert_eq!(
            state.army_commands(&rules.1, RED, (GARRISON, Some(RED_ARMY)), None),
            []
        );
        let listed = state.army_commands(&rules.1, RED, (RED_ARMY, Some(GARRISON)), None);
        assert!(listed.iter().all(|c| matches!(
            c,
            ArmyCommand::Move { from, to } if from.army == RED_ARMY
                && (to.army == RED_ARMY || [1, 3, 5].contains(&to.slot))
        )));
        let range = |from, to| state.split_range(&rules.1, RED, (from, to), None);
        assert_eq!(range(at(GARRISON, 2), at(RED_ARMY, 3)), None);
        assert_eq!(range(at(RED_ARMY, 0), at(GARRISON, 0)), None);
        let mut more = state.clone();
        more.heroes[0].army = vec![stack(0, PIKEMAN, 4)];
        let range = more.split_range(&rules.1, RED, (at(RED_ARMY, 0), at(GARRISON, 1)), None);
        assert_eq!(range, Some(1..=3));
    }
}

// covers: ADV-GAR-003
#[test]
fn an_army_command_names_a_map_garrison_by_its_object_index() {
    let rules = garrison_rules();
    let own = garrison(&rules, "t:garrison", ENTRANCE, Some(RED), &GNOLLS);
    let blocker = ObjectSetup {
        object_type: None,
        holds: Holds::Nothing,
        ..garrison(&rules, "t:garrison", p(6, 0), None, &[])
    };
    let mut state = corridor(&rules, vec![own, blocker]);
    // The removable flag is the map's; none for a blocker or past the objects.
    let removable: Vec<Option<bool>> = (0..3).map(|i| state.garrison_removable(i)).collect();
    assert_eq!(removable, [Some(true), None, None]);
    let fixed = fixed(&rules, ("t:garrison", ENTRANCE), Some(RED), &GNOLLS, false);
    assert_eq!(
        corridor(&rules, vec![fixed]).garrison_removable(0),
        Some(false)
    );
    for index in [1, 2, u32::MAX] {
        let command = ArmyCommand::Move {
            from: at(RED_ARMY, 0),
            to: at(ArmyRef::MapGarrison(index), 0),
        };
        assert_eq!(
            manage(&mut state, &rules, command),
            Err(ArmyCommandError::NoGarrison)
        );
        let range = state.split_range(
            &rules.1,
            RED,
            (at(GARRISON, 2), at(ArmyRef::MapGarrison(index), 0)),
            None,
        );
        assert_eq!(range, None);
    }
    for error in [
        ArmyCommandError::NoGarrison,
        ArmyCommandError::NotOwn,
        ArmyCommandError::OnlyEmpty,
    ] {
        assert!(!error.to_string().is_empty());
    }
}
