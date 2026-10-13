//! Tests of the tactics phase (BAT-TAC-001/002).

use std::vec;
use std::vec::Vec;

use h3_content_battle::BattleRuleset;
use h3_contracts::{BallistaControl, BattleHero, BattleSide, TentControl};

use super::support::*;
use crate::{
    BattleSetup, BattleState, Command, CommandError, Event, EventKind, Hex, LegalCommands,
    StackFlags, StackId, TacticsPhase, legal_commands, observe,
};

/// The test hero with Tactics at `level`.
fn tactician(r: &BattleRuleset, level: u8) -> BattleHero {
    let mut h = hero(r);
    h.tactics = level;
    h
}

/// A setup whose sides are led by heroes with these Tactics levels (`None`: no hero).
fn led(
    r: &BattleRuleset,
    attacker: &[Spec],
    defender: &[Spec],
    levels: [Option<u8>; 2],
) -> BattleSetup {
    BattleSetup {
        heroes: levels.map(|l| l.map(|l| tactician(r, l))),
        ..setup(r, attacker, defender)
    }
}

fn start(r: &BattleRuleset, s: &BattleSetup) -> (BattleState, Vec<Event>) {
    let mut events = Vec::new();
    let state = BattleState::start(r, s, 1, &mut events).unwrap();
    (state, events)
}

fn kinds(events: &[Event]) -> Vec<EventKind> {
    events.iter().map(|e| e.kind).collect()
}

fn active(s: &BattleState) -> u8 {
    s.active().unwrap().0
}

fn moves(s: &BattleState, r: &BattleRuleset) -> Vec<Hex> {
    let mut legal = LegalCommands::new();
    legal_commands(s, r, s.side_to_act().unwrap(), &mut legal);
    legal
        .iter()
        .filter_map(|c| match c {
            Command::Move { to } => Some(*to),
            _ => None,
        })
        .collect()
}

// covers: BAT-TAC-001
#[test]
fn the_higher_tactics_level_gets_a_zone_by_the_difference() {
    use BattleSide::{Attacker, Defender};
    let r = ruleset();
    let cases = [
        ([Some(1), None], Some((Attacker, 3))),
        ([Some(2), None], Some((Attacker, 5))),
        ([Some(3), None], Some((Attacker, 7))),
        ([None, Some(3)], Some((Defender, 7))),
        // A level negates the same level of the enemy's (wiki, Tactics).
        ([Some(3), Some(1)], Some((Attacker, 5))),
        ([Some(1), Some(3)], Some((Defender, 5))),
        ([Some(3), Some(2)], Some((Attacker, 3))),
        ([Some(2), Some(2)], None),
        ([Some(0), None], None),
        ([None, None], None),
    ];
    for (levels, expected) in cases {
        let (s, events) = start(&r, &led(&r, &[(0, "fixed", 5)], &[(0, "fixed", 5)], levels));
        let phase = expected.map(|(side, columns)| TacticsPhase { side, columns });
        assert_eq!(s.tactics(), phase, "{levels:?}");
        assert_eq!(s.round(), if phase.is_some() { 0 } else { 1 });
        assert_eq!(
            matches!(
                events.first().map(|e| e.kind),
                Some(EventKind::TacticsStarted { .. })
            ),
            phase.is_some()
        );
    }
    // The widths are ruleset data.
    let narrow = ruleset_with("t:faithful", |b| b.tactics_zone_columns = vec![2, 4, 6]);
    let s = BattleState::new(
        &narrow,
        &led(
            &narrow,
            &[(0, "fixed", 5)],
            &[(0, "fixed", 5)],
            [Some(1), None],
        ),
        1,
    )
    .unwrap();
    assert_eq!(s.tactics().unwrap().columns, 2);
}

// covers: BAT-TAC-001
#[test]
fn the_zone_is_the_sides_nearest_raw_columns() {
    let r = ruleset();
    let f = field(&r);
    // Batch 1b R2 (Expert Tactics against a monster): the zone's rightmost hex is the 7th of the
    // top row and of the second row, counted from the row's leftmost hex.
    let expert = TacticsPhase {
        side: BattleSide::Attacker,
        columns: 7,
    };
    for y in [0, 1, 5, 10] {
        assert!(expert.contains(f, at(&r, 1, y)) && expert.contains(f, at(&r, 7, y)));
        assert!(!expert.contains(f, at(&r, 0, y)) && !expert.contains(f, at(&r, 8, y)));
    }
    // The defender's zone mirrors it from the last standable column.
    let defender = TacticsPhase {
        side: BattleSide::Defender,
        columns: 3,
    };
    for y in [0, 1] {
        assert!(defender.contains(f, at(&r, 13, y)) && defender.contains(f, at(&r, 15, y)));
        assert!(!defender.contains(f, at(&r, 12, y)) && !defender.contains(f, at(&r, 16, y)));
    }
}

// covers: BAT-TAC-001, BAT-TAC-002
#[test]
fn the_phase_starts_in_round_0_and_is_public() {
    let r = ruleset();
    let s = led(
        &r,
        &[(0, "slow", 5), (1, "fast", 5), (2, "fixed", 5)],
        &[(0, "fixed", 5)],
        [Some(2), None],
    );
    let (state, events) = start(&r, &s);
    let phase = TacticsPhase {
        side: BattleSide::Attacker,
        columns: 5,
    };
    assert_eq!(
        kinds(&events),
        [
            EventKind::TacticsStarted {
                side: BattleSide::Attacker,
                columns: 5
            },
            EventKind::TurnStarted { stack: StackId(1) },
        ]
    );
    assert!(
        events
            .iter()
            .all(|e| e.visibility == crate::Visibility::Public)
    );
    assert_eq!(
        (state.round(), state.side_to_act()),
        (0, Some(BattleSide::Attacker))
    );
    for viewer in BattleSide::BOTH {
        let o = observe(&state, &r, viewer);
        assert_eq!((o.round, o.tactics), (0, Some(phase)));
    }
    // The enemy cannot act in the phase.
    let mut copy = state.clone();
    let err = crate::apply(
        &mut copy,
        &r,
        BattleSide::Defender,
        Command::EndTactics,
        &mut crate::NoopSink,
    );
    assert_eq!(err, Err(CommandError::NotYourTurn));
    assert_eq!(copy, state);
}

// covers: BAT-TAC-002
#[test]
fn stacks_are_handed_over_in_rounds_and_next_creature_waits() {
    let r = ruleset();
    let mut s = BattleSetup {
        war_machines: [vec![r.creature_by_key("t:ballista").unwrap()], vec![]],
        ..led(
            &r,
            &[
                (0, "slow", 5),
                (1, "fast", 5),
                (2, "fixed", 5),
                (3, "wisp", 5),
            ],
            &[(0, "fixed", 5)],
            [Some(3), None],
        )
    };
    s.heroes[0].as_mut().unwrap().war_machines.ballista = Some(BallistaControl {
        shots: 1,
        double_damage_chance_bp: 0,
    });
    let (mut state, _) = start(&r, &s);
    // Speed 9 (slot 1, then slot 3), 5, 2; the Ballista (stack 4) is never handed over. Wait
    // ("next creature") makes each stack wait; the waiting stacks come back slowest first, and
    // Wait on a stack that has waited ends its turn; then the next round of the phase starts
    // with the fastest (batch 2a R7b).
    let mut seen = vec![active(&state)];
    for _ in 0..8 {
        let before = StackId(active(&state));
        let waited = state
            .stack(before)
            .unwrap()
            .flags
            .contains(StackFlags::WAITED);
        let events = act(&mut state, &r, Command::Wait).unwrap();
        let next = EventKind::TurnStarted {
            stack: state.active().unwrap(),
        };
        let expected = if waited {
            vec![next]
        } else {
            vec![EventKind::Waited { stack: before }, next]
        };
        assert_eq!(kinds(&events), expected);
        seen.push(active(&state));
    }
    assert_eq!(seen, [1, 3, 2, 0, 0, 2, 1, 3, 1]);
    assert!(
        state
            .stacks()
            .iter()
            .all(|st| st.flags == StackFlags::default())
    );
    // A placement ends the stack's turn; a stack may be placed again in the next round of the
    // phase, which starts after the last stack's turn (R7c).
    act(&mut state, &r, Command::Wait).unwrap();
    let events = act(&mut state, &r, Command::Move { to: at(&r, 6, 0) }).unwrap();
    assert_eq!(
        kinds(&events),
        [
            EventKind::Moved {
                stack: StackId(3),
                from: at(&r, 1, 9),
                to: at(&r, 6, 0)
            },
            EventKind::TurnStarted { stack: StackId(2) },
        ]
    );
    let mut seen = vec![];
    for _ in 0..5 {
        act(&mut state, &r, Command::Wait).unwrap();
        seen.push(active(&state));
    }
    // 2 and 0 wait, 0 and 2 end their turns, 1 (waited) ends its turn: a new round.
    assert_eq!(seen, [0, 0, 2, 1, 1]);
    act(&mut state, &r, Command::Wait).unwrap();
    assert_eq!(active(&state), 3);
    act(&mut state, &r, Command::Move { to: at(&r, 7, 10) }).unwrap();
    assert_eq!(state.stack(StackId(3)).unwrap().hex, at(&r, 7, 10));
    // Round 1 resets the phase's turn flags (BAT-TRN-003).
    act(&mut state, &r, Command::EndTactics).unwrap();
    assert!(
        state
            .stacks()
            .iter()
            .all(|st| !st.flags.contains(StackFlags::WAITED))
    );
}

// covers: BAT-TAC-001, BAT-TAC-002
#[test]
fn placements_ignore_speed_but_stay_in_the_zone() {
    let r = ruleset();
    let s = led(
        &r,
        &[(0, "slow", 5), (1, "fixed", 5)],
        &[(0, "fixed", 5)],
        [Some(3), None],
    );
    let (state, _) = start(&r, &s);
    // `fixed` (speed 5, at (1, 7)) acts first; `slow` (speed 2) stands at (1, 3).
    assert_eq!(active(&state), 1);
    let cases = [
        (at(&r, 8, 7), Err(CommandError::OutsideZone)),
        (at(&r, 0, 7), Err(CommandError::InvalidHex)),
        (Hex(250), Err(CommandError::InvalidHex)),
        (at(&r, 1, 7), Err(CommandError::Unreachable)),
        (at(&r, 1, 3), Err(CommandError::Unreachable)),
        // Ten steps away, far beyond the speed (batch 1b R2).
        (at(&r, 7, 0), Ok(())),
    ];
    for (to, expected) in cases {
        let mut copy = state.clone();
        let result = act(&mut copy, &r, Command::Move { to }).map(|_| ());
        assert_eq!(result, expected, "{to:?}");
        if expected.is_err() {
            assert_eq!(copy, state);
        }
    }
    // Every legal placement lies in the zone, and every free hex of the zone is one.
    let zone = state.tactics().unwrap();
    let placements = moves(&state, &r);
    assert!(placements.iter().all(|&h| zone.contains(field(&r), h)));
    assert_eq!(placements.len(), 7 * 11 - 2);
    // Everything but placements, Wait and EndTactics is refused in the phase.
    for command in [
        Command::Defend,
        Command::Attack {
            target: StackId(2),
            from: at(&r, 14, 5),
        },
        Command::Shoot {
            target: StackId(2),
            at: at(&r, 15, 5),
        },
        Command::Heal { target: StackId(0) },
    ] {
        let mut copy = state.clone();
        assert_eq!(
            act(&mut copy, &r, command),
            Err(CommandError::InTacticsPhase)
        );
        assert_eq!(copy, state);
    }
}

// covers: BAT-TAC-002
#[test]
fn walkers_need_a_path_and_flyers_do_not() {
    let r = ruleset();
    // Batch 1b R2: a free hex of the zone enclosed by its neighbours.
    let f = field(&r);
    let enclosed = at(&r, 4, 0);
    let mut s = led(
        &r,
        &[(0, "slow", 5), (1, "flyer", 5)],
        &[(0, "fixed", 5)],
        [Some(3), None],
    );
    s.obstacles = blocked(
        f.neighbors(enclosed)
            .into_iter()
            .flatten()
            .map(|h| h.0)
            .collect(),
    );
    let (mut state, _) = start(&r, &s);
    // The flyer (speed 7) comes first; it waits, the walker follows.
    assert_eq!(active(&state), 1);
    act(&mut state, &r, Command::Wait).unwrap();
    assert!(!moves(&state, &r).contains(&enclosed));
    let mut copy = state.clone();
    assert_eq!(
        act(&mut copy, &r, Command::Move { to: enclosed }),
        Err(CommandError::Unreachable)
    );
    // The walker waits too; the waiting stacks come back slowest first: the walker, the flyer.
    act(&mut state, &r, Command::Wait).unwrap();
    act(&mut state, &r, Command::Wait).unwrap();
    assert_eq!(active(&state), 1);
    assert!(moves(&state, &r).contains(&enclosed));
    act(&mut state, &r, Command::Move { to: enclosed }).unwrap();
    assert_eq!(state.stack(StackId(1)).unwrap().hex, enclosed);
}

// covers: BAT-TAC-002
#[test]
fn a_walking_path_may_leave_the_zone() {
    let r = ruleset();
    let f = field(&r);
    // Basic Tactics: columns 1..=3. The target's neighbours in the zone are blocked, so the only
    // way in leads through column 4, outside the zone (user decision 2026-10-04, as VCMI).
    let target = at(&r, 3, 5);
    let mut s = led(&r, &[(0, "slow", 5)], &[(0, "fixed", 5)], [Some(1), None]);
    s.obstacles = blocked(
        f.neighbors(target)
            .into_iter()
            .flatten()
            .filter(|&h| f.coords(h).0 <= 3)
            .map(|h| h.0)
            .collect(),
    );
    let (mut state, _) = start(&r, &s);
    assert_eq!(
        act(&mut state.clone(), &r, Command::Move { to: at(&r, 4, 5) }),
        Err(CommandError::OutsideZone)
    );
    act(&mut state, &r, Command::Move { to: target }).unwrap();
    assert_eq!(state.stack(StackId(0)).unwrap().hex, target);
}

// covers: BAT-TAC-002
#[test]
fn two_hex_stacks_are_placed_with_both_hexes_in_the_zone() {
    let r = ruleset();
    let s = led(&r, &[(0, "wide", 5)], &[(0, "fixed", 5)], [Some(1), None]);
    let (state, _) = start(&r, &s);
    // The zone is columns 1 to 3; the front hex stands right of the position, so positions are
    // in columns 1 and 2 only.
    let columns: Vec<i32> = moves(&state, &r)
        .iter()
        .map(|&h| field(&r).coords(h).0)
        .collect();
    assert!(columns.contains(&1) && columns.contains(&2));
    assert!(columns.iter().all(|x| (1..=2).contains(x)));
}

// covers: BAT-TAC-002
#[test]
fn war_machines_are_neither_placed_nor_active_in_the_phase() {
    let r = ruleset();
    let mut s = BattleSetup {
        war_machines: [
            vec![
                r.creature_by_key("t:ballista").unwrap(),
                r.creature_by_key("t:tent").unwrap(),
            ],
            vec![r.creature_by_key("t:ballista").unwrap()],
        ],
        ..led(&r, &[(0, "fixed", 5)], &[(0, "fixed", 5)], [None, Some(2)])
    };
    s.heroes[1].as_mut().unwrap().war_machines.tent = Some(TentControl {
        heal_bonus: Rate::bp(0),
    });
    let (mut state, events) = start(&r, &s);
    // The defender places; the attacker's automatic machines do not act before round 1.
    assert_eq!(state.tactics().unwrap().side, BattleSide::Defender);
    assert!(
        !events
            .iter()
            .any(|e| matches!(e.kind, EventKind::Struck { .. }))
    );
    let first = active(&state);
    for _ in 0..3 {
        act(&mut state, &r, Command::Wait).unwrap();
        assert_eq!(active(&state), first);
    }
}

// covers: BAT-TAC-002
#[test]
fn end_tactics_starts_round_one() {
    let r = ruleset();
    let s = led(
        &r,
        &[(0, "slow", 5), (1, "fixed", 5)],
        &[(0, "fast", 5)],
        [None, Some(1)],
    );
    let (mut state, _) = start(&r, &s);
    // The defender's zone: columns 13..=15; its stack moves forward, then the phase ends.
    act(&mut state, &r, Command::Move { to: at(&r, 13, 2) }).unwrap();
    let before = state.clone();
    assert_eq!(
        act(&mut state.clone(), &r, Command::Move { to: at(&r, 12, 2) }),
        Err(CommandError::OutsideZone)
    );
    let events = act(&mut state, &r, Command::EndTactics).unwrap();
    // Round 1 starts as usual: the fastest stack first (the defender's speed 9).
    assert_eq!(
        kinds(&events),
        [
            EventKind::TacticsEnded,
            EventKind::RoundStarted { round: 1 },
            EventKind::TurnStarted { stack: StackId(2) },
        ]
    );
    assert_eq!((state.round(), state.tactics()), (1, None));
    assert_eq!(state.last_side_acted(), None);
    assert_eq!(state.stack(StackId(2)).unwrap().hex, at(&r, 13, 2));
    assert_ne!(before, state);
    // No second phase; the usual commands are back.
    assert_eq!(
        act(&mut state.clone(), &r, Command::EndTactics),
        Err(CommandError::NoTacticsPhase)
    );
    let mut legal = LegalCommands::new();
    legal_commands(&state, &r, BattleSide::Defender, &mut legal);
    let legal = legal.as_slice();
    assert!(legal.contains(&Command::Defend) && !legal.contains(&Command::EndTactics));
    assert_eq!(order(&mut state, &r, 3), [2, 1, 0]);
}

// covers: BAT-TAC-002
#[test]
fn legal_commands_of_the_phase() {
    let r = ruleset();
    let (state, _) = start(
        &r,
        &led(&r, &[(0, "fixed", 5)], &[(0, "fixed", 5)], [Some(1), None]),
    );
    let mut legal = LegalCommands::new();
    legal_commands(&state, &r, BattleSide::Attacker, &mut legal);
    assert_eq!(legal.as_slice()[..2], [Command::Wait, Command::EndTactics]);
    // Three columns of eleven rows, less the stack's own hex; then Retreat (BAT-FLE-001).
    assert_eq!(legal.len(), 2 + 3 * 11 - 1 + 1);
    assert_eq!(legal.as_slice().last(), Some(&Command::Retreat));
    legal_commands(&state, &r, BattleSide::Defender, &mut legal);
    assert!(legal.is_empty());
}
