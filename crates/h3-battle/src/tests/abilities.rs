//! Tests of the melee abilities: double attacks (BAT-ATK-002), attack patterns (BAT-ATK-003),
//! strike and return (BAT-ATK-004), retaliation rules (BAT-RET-002/003), damage abilities
//! (BAT-DMG-004..009) and regeneration (BAT-TRN-005).
//!
//! Most test creatures are 10/10 with damage 2 and 10 HP (`support::CREATURES`), so a strike of
//! `n` of them on a 10-defense stack deals exactly `2n`.

use std::vec;
use std::vec::Vec;

use h3_content_battle::{Ability, BattleRuleset, CompiledAbility};
use h3_contracts::{BattleResult, BattleSide};

use super::support::*;
use crate::damage::reduced_defense;
use crate::event::StrikeFlags;
use crate::geometry::Direction;
use crate::state::{UNLIMITED_RETALIATIONS, retaliations_per_round};
use crate::{BattleState, Command, Event, EventKind, NoopSink, StackFlags, StackId};

const N: StrikeFlags = StrikeFlags::NONE;
const R: StrikeFlags = StrikeFlags::RETALIATION;
const S: StrikeFlags = StrikeFlags::SECONDARY;

/// `(attacker, target, damage, kills, flags)` of every strike in `events`.
fn strikes(events: &[Event]) -> Vec<(u8, u8, u64, u32, StrikeFlags)> {
    events
        .iter()
        .filter_map(|e| match e.kind {
            EventKind::Struck {
                attacker,
                target,
                damage,
                kills,
                flags,
            } => Some((attacker.0, target.0, damage, kills, flags)),
            _ => None,
        })
        .collect()
}

/// The targets struck by `attacker` in `events`.
fn targets_of(events: &[Event], attacker: u8) -> Vec<u8> {
    strikes(events)
        .into_iter()
        .filter(|s| s.0 == attacker)
        .map(|s| s.1)
        .collect()
}

/// The active stack attacks `target` from `(x, y)`.
fn hit(s: &mut BattleState, r: &BattleRuleset, target: u8, (x, y): (i32, i32)) -> Vec<Event> {
    let from = at(r, x, y);
    act(
        s,
        r,
        Command::Attack {
            target: StackId(target),
            from,
        },
    )
    .unwrap()
}

/// Places the stacks of `s` at the given positions, in stack-id order.
fn arrange(s: &mut BattleState, r: &BattleRuleset, positions: &[(i32, i32)]) {
    for (id, &(x, y)) in (0u8..).zip(positions) {
        place(s, r, id, x, y);
    }
}

fn stack(s: &mut BattleState, id: u8) -> &mut crate::Stack {
    &mut s.stacks.as_mut_slice()[usize::from(id)]
}

fn ended(events: &[Event]) -> bool {
    events
        .iter()
        .any(|e| matches!(e.kind, EventKind::BattleEnded { .. }))
}

// ---------------------------------------------------------------- BAT-ATK-002 double attack

// covers: BAT-ATK-002
#[test]
fn creatures_that_strike_twice_attack_again_after_the_retaliation() {
    let r = ruleset();
    let mut s = battle(&r, &[(0, "twin", 5)], &[(0, "fixed", 10)], 1);
    arrange(&mut s, &r, &[(7, 5), (8, 5)]);
    let events = hit(&mut s, &r, 1, (7, 5));
    // 5 twins deal 10 (one kill); 9 survivors retaliate for 18; the 4 twins left strike for 8.
    assert_eq!(
        strikes(&events),
        vec![(0, 1, 10, 1, N), (1, 0, 18, 1, R), (0, 1, 8, 0, N)]
    );
}

// covers: BAT-ATK-002
#[test]
fn the_second_attack_needs_both_stacks_alive_and_shooters_strike_once_in_melee() {
    let r = ruleset();
    // The first strike kills the target: nothing follows.
    let mut s = battle(&r, &[(0, "twin", 50)], &[(0, "slow", 1)], 1);
    arrange(&mut s, &r, &[(7, 5), (8, 5)]);
    let events = hit(&mut s, &r, 1, (7, 5));
    assert_eq!(targets_of(&events, 0), vec![1]);
    assert!(ended(&events));
    // The retaliation kills the attacker: no second attack.
    let mut s = battle(&r, &[(0, "twin", 1)], &[(0, "fixed", 100)], 1);
    arrange(&mut s, &r, &[(7, 5), (8, 5)]);
    let events = hit(&mut s, &r, 1, (7, 5));
    assert_eq!(strikes(&events).len(), 2);
    assert_eq!(
        s.outcome().unwrap().result,
        BattleResult::Victory(BattleSide::Defender)
    );
    // A blocked shooter that shoots twice strikes once in melee.
    let mut s = battle(&r, &[(0, "sniper", 5)], &[(0, "fixed", 10)], 1);
    arrange(&mut s, &r, &[(7, 5), (8, 5)]);
    activate(&mut s, 0);
    let events = hit(&mut s, &r, 1, (7, 5));
    let flags: Vec<StrikeFlags> = strikes(&events).iter().map(|s| s.4).collect();
    assert_eq!(flags, vec![N, R]);
}

// ---------------------------------------------------------------- BAT-RET-002 no retaliation

// covers: BAT-RET-002
#[test]
fn attacks_without_enemy_retaliation_are_not_retaliated_and_cost_no_retaliation() {
    let r = ruleset();
    // 0 = naga, 1 = serf (attacker), 2 = fixed (defender).
    let mut s = battle(
        &r,
        &[(0, "naga", 5), (1, "serf", 5)],
        &[(0, "fixed", 10)],
        1,
    );
    arrange(&mut s, &r, &[(7, 5), (9, 5), (8, 5)]);
    let events = hit(&mut s, &r, 2, (7, 5));
    assert_eq!(strikes(&events), vec![(0, 2, 10, 1, N)]);
    let target = s.stacks()[2];
    assert_eq!(target.retaliations_left, 1);
    assert!(!target.flags.contains(StackFlags::RETALIATED));
    // The target still retaliates against the next attacker.
    advance_to(&mut s, &r, 1);
    let events = hit(&mut s, &r, 2, (9, 5));
    assert_eq!(targets_of(&events, 2), vec![1]);
}

// ---------------------------------------------------------------- BAT-RET-003 retaliation count

// covers: BAT-RET-003
#[test]
fn retaliations_per_round_come_from_the_abilities() {
    let r = ruleset();
    let none: [CompiledAbility; 0] = [];
    assert_eq!(retaliations_per_round(&none), 1);
    assert_eq!(
        retaliations_per_round(&[Ability::ExtraRetaliations { count: 1 }]),
        2
    );
    assert_eq!(
        retaliations_per_round(&[Ability::UnlimitedRetaliations]),
        UNLIMITED_RETALIATIONS
    );
    // Absurd counts stay below "unlimited".
    assert_eq!(
        retaliations_per_round(&[Ability::ExtraRetaliations { count: u8::MAX }]),
        UNLIMITED_RETALIATIONS - 1
    );
    let s = battle(&r, &[(0, "griffin", 1)], &[(0, "royal", 1)], 1);
    assert_eq!(
        (s.stacks()[0].retaliations, s.stacks()[1].retaliations),
        (2, UNLIMITED_RETALIATIONS)
    );
}

/// Three tanks attack the defender `name` ×10 in turn; returns whether each attack was
/// retaliated, and the battle.
fn three_attacks(r: &BattleRuleset, name: &'static str) -> (Vec<bool>, BattleState) {
    let mut s = battle(
        r,
        &[(0, "tank", 1), (1, "tank", 1), (2, "tank", 1)],
        &[(0, name, 10)],
        1,
    );
    arrange(&mut s, r, &[(7, 5), (9, 5), (8, 4), (8, 5)]);
    let mut retaliated = Vec::new();
    for (id, from) in [(0, (7, 5)), (1, (9, 5)), (2, (8, 4))] {
        activate(&mut s, id);
        let events = hit(&mut s, r, 3, from);
        // A tank deals 1; the 10 defenders retaliate for 20 − 25 % = 15.
        retaliated.push(strikes(&events).contains(&(3, id, 15, 0, R)));
    }
    (retaliated, s)
}

// covers: BAT-RET-003
#[test]
fn extra_retaliations_allow_more_retaliations_per_round() {
    let r = ruleset();
    let (retaliated, mut s) = three_attacks(&r, "griffin");
    assert_eq!(retaliated, vec![true, true, false]);
    assert_eq!(s.stacks()[3].retaliations_left, 0);
    assert!(s.stacks()[3].flags.contains(StackFlags::RETALIATED));
    // A new round restores them.
    crate::turn::start_round(&mut s, &r, &mut NoopSink);
    assert_eq!(s.stacks()[3].retaliations_left, 2);
    assert!(!s.stacks()[3].flags.contains(StackFlags::RETALIATED));
}

// covers: BAT-RET-003
#[test]
fn unlimited_retaliations_never_run_out() {
    let r = ruleset();
    let (retaliated, s) = three_attacks(&r, "royal");
    assert_eq!(retaliated, vec![true, true, true]);
    assert_eq!(s.stacks()[3].retaliations_left, UNLIMITED_RETALIATIONS);
    assert!(!s.stacks()[3].flags.contains(StackFlags::RETALIATED));
}

// ---------------------------------------------------------------- BAT-ATK-003 attack patterns

// covers: BAT-ATK-003
#[test]
fn attacking_all_adjacent_hits_every_touching_enemy_also_when_retaliating() {
    let r = ruleset();
    // 0 = hydra (position 6,5, front hex 7,5), 1 = allied serf; defenders 2 (target), 3 and 4
    // touch the hydra, 5 does not.
    let mut s = battle(
        &r,
        &[(0, "hydra", 10), (1, "serf", 10)],
        &[
            (0, "fixed", 10),
            (1, "fixed", 10),
            (2, "fixed", 10),
            (3, "fixed", 10),
        ],
        1,
    );
    arrange(
        &mut s,
        &r,
        &[(6, 5), (7, 6), (8, 5), (5, 5), (6, 4), (10, 5)],
    );
    let events = hit(&mut s, &r, 2, (6, 5));
    assert_eq!(
        strikes(&events),
        vec![(0, 2, 20, 2, N), (0, 3, 20, 2, S), (0, 4, 20, 2, S),]
    );
    // Stack 2 (8 left) attacks the hydra; the hydra's retaliation hits all three again.
    assert_eq!(s.active(), Some(StackId(2)));
    let events = hit(&mut s, &r, 0, (8, 5));
    assert_eq!(
        strikes(&events),
        vec![
            (2, 0, 16, 1, N),
            (0, 2, 18, 1, R),
            (0, 3, 18, 1, R.with(S)),
            (0, 4, 18, 1, R.with(S)),
        ]
    );
}

/// The enemies hit by a cerberus (position at `position`, its front hex to the right) attacking
/// defender stack 2 with an ally (stack 1) at `ally`, defender stacks at `enemies` (ids 3, 4, …),
/// and the target at `target`; the target is `wide` if set.
fn cerberus_hits(
    position: (i32, i32),
    ally: (i32, i32),
    target: (i32, i32),
    wide: bool,
    enemies: &[(i32, i32)],
) -> Vec<u8> {
    let r = ruleset();
    let mut defenders = vec![(0, if wide { "wide" } else { "fixed" }, 10)];
    for slot in 1..=enemies.len() {
        defenders.push((u8::try_from(slot).unwrap(), "fixed", 10));
    }
    let mut s = battle(&r, &[(0, "cerberus", 10), (1, "serf", 10)], &defenders, 1);
    let mut positions = vec![position, ally, target];
    positions.extend_from_slice(enemies);
    arrange(&mut s, &r, &positions);
    let events = hit(&mut s, &r, 2, position);
    targets_of(&events, 0)
}

// covers: BAT-ATK-003
#[test]
fn three_headed_attacks_hit_the_enemies_beside_the_attacked_hex() {
    // Attacking right: the hexes below-right and above-right of the front hex. The ally above
    // is spared, the enemy touching the rear hex is not in the pattern. No retaliation follows.
    assert_eq!(
        cerberus_hits((6, 5), (7, 4), (8, 5), false, &[(7, 6), (6, 4)]),
        vec![2, 3]
    );
    // Attacking below-left of the front hex: turning towards the rear hex would enter the
    // attacker, so that walk starts on the attacked hex and continues left.
    assert_eq!(
        cerberus_hits((6, 5), (3, 3), (6, 6), false, &[(5, 6), (7, 6)]),
        vec![2, 3, 4]
    );
    // A two-hex target whose rear hex is adjacent is attacked towards the rear hex: the pattern
    // turns from there (the target's front hex is the target itself).
    assert_eq!(
        cerberus_hits((7, 5), (3, 3), (8, 4), true, &[(9, 5), (6, 4)]),
        vec![2, 3]
    );
}

// covers: BAT-ATK-003
#[test]
fn breath_attacks_also_hit_the_stack_behind_friend_or_foe() {
    let r = ruleset();
    // 0 = drake (position 6,5, front hex 7,5), 1 = allied serf behind the target 2.
    let mut s = battle(
        &r,
        &[(0, "drake", 10), (1, "serf", 10)],
        &[(0, "fixed", 10)],
        1,
    );
    arrange(&mut s, &r, &[(6, 5), (9, 5), (8, 5)]);
    let events = hit(&mut s, &r, 2, (6, 5));
    assert_eq!(
        strikes(&events),
        vec![(0, 2, 20, 2, N), (0, 1, 20, 2, S), (2, 0, 16, 1, R)]
    );
    // Behind a two-hex target lies its own rear hex: it is not hit twice, and the stack behind
    // the rear hex is out of reach.
    let mut s = battle(
        &r,
        &[(0, "drake", 10)],
        &[(0, "wide", 10), (1, "fixed", 10)],
        1,
    );
    arrange(&mut s, &r, &[(6, 5), (9, 5), (10, 5)]);
    let events = hit(&mut s, &r, 1, (6, 5));
    assert_eq!(targets_of(&events, 0), vec![1]);
    assert!(strikes(&events)[1].4.contains(R));
    // Off the field behind the target: only the target is hit.
    let mut s = battle(&r, &[(0, "drake", 10)], &[(0, "fixed", 10)], 1);
    arrange(&mut s, &r, &[(6, 1), (7, 0)]);
    let events = hit(&mut s, &r, 1, (6, 1));
    assert_eq!(targets_of(&events, 0), vec![1]);
    // A drake whose rear hex (6,5) alone touches the target breathes from there: the stack
    // behind the target, further left, is hit too.
    let mut s = battle(
        &r,
        &[(0, "drake", 10)],
        &[(0, "fixed", 10), (1, "fixed", 10)],
        1,
    );
    arrange(&mut s, &r, &[(6, 5), (5, 5), (4, 5)]);
    let events = hit(&mut s, &r, 1, (6, 5));
    assert_eq!(targets_of(&events, 0), vec![1, 2]);
}

// ---------------------------------------------------------------- BAT-ATK-004 strike and return

/// The `(stack, from, to)` of every move in `events`.
fn moves(events: &[Event]) -> Vec<(u8, crate::Hex, crate::Hex)> {
    events
        .iter()
        .filter_map(|e| match e.kind {
            EventKind::Moved { stack, from, to } => Some((stack.0, from, to)),
            _ => None,
        })
        .collect()
}

// covers: BAT-ATK-004
#[test]
fn strike_and_return_flies_back_after_the_retaliation() {
    let r = ruleset();
    let mut s = battle(&r, &[(0, "harpy", 5)], &[(0, "fixed", 10)], 1);
    arrange(&mut s, &r, &[(3, 5), (8, 5)]);
    let events = hit(&mut s, &r, 1, (7, 5));
    let (start, there) = (at(&r, 3, 5), at(&r, 7, 5));
    assert_eq!(moves(&events), vec![(0, start, there), (0, there, start)]);
    assert_eq!(strikes(&events), vec![(0, 1, 10, 1, N), (1, 0, 18, 1, R)]);
    // The return comes after the retaliation.
    let last_strike = events
        .iter()
        .rposition(|e| matches!(e.kind, EventKind::Struck { .. }))
        .unwrap();
    let back = events
        .iter()
        .rposition(|e| matches!(e.kind, EventKind::Moved { .. }))
        .unwrap();
    assert!(back > last_strike);
    assert_eq!(s.stacks()[0].hex, start);
}

// covers: BAT-ATK-004
#[test]
fn strike_and_return_stays_without_a_move_or_when_killed() {
    let r = ruleset();
    let mut s = battle(&r, &[(0, "harpy", 5)], &[(0, "fixed", 10)], 1);
    arrange(&mut s, &r, &[(7, 5), (8, 5)]);
    let events = hit(&mut s, &r, 1, (7, 5));
    assert!(moves(&events).is_empty());
    let mut s = battle(&r, &[(0, "harpy", 1)], &[(0, "fixed", 100)], 1);
    arrange(&mut s, &r, &[(3, 5), (8, 5)]);
    let events = hit(&mut s, &r, 1, (7, 5));
    assert_eq!(moves(&events).len(), 1);
    assert_eq!(s.stacks()[0].hex, at(&r, 7, 5));
    assert!(ended(&events));
}

// ---------------------------------------------------------------- BAT-DMG-004 hate

/// The damage of the first strike of `attacker` ×10 standing at (7,5) on `target` ×10 at (8,5).
fn first_strike(attacker: &'static str, target: &'static str) -> u64 {
    let r = ruleset();
    let mut s = battle(&r, &[(0, attacker, 10)], &[(0, target, 10)], 1);
    arrange(&mut s, &r, &[(7, 5), (8, 5)]);
    let events = hit(&mut s, &r, 1, (7, 5));
    strikes(&events)[0].2
}

// covers: BAT-DMG-004
#[test]
fn hate_adds_its_bonus_against_the_hated_creature_only() {
    assert_eq!(first_strike("hater", "fixed"), 30);
    assert_eq!(first_strike("hater", "serf"), 20);
}

// ---------------------------------------------------------------- BAT-DMG-005 jousting

/// The damage of a knight ×10 starting at (3,5) attacking `target` ×10 at (8,5) from `from`.
fn charge(target: &'static str, from: (i32, i32), obstacles: &[(i32, i32)]) -> u64 {
    let r = ruleset();
    let mut s = battle_with_obstacles(&r, &[(0, "knight", 10)], &[(0, target, 10)], obstacles);
    arrange(&mut s, &r, &[(3, 5), (8, 5)]);
    let events = hit(&mut s, &r, 1, from);
    strikes(&events)[0].2
}

// covers: BAT-DMG-005
#[test]
fn jousting_adds_its_bonus_per_hex_travelled() {
    // Four hexes: +20 %.
    assert_eq!(charge("fixed", (7, 5), &[]), 24);
    // Around obstacles the path is five hexes long (+25 %) although the distance is four.
    assert_eq!(charge("fixed", (7, 5), &[(4, 5), (5, 5), (6, 5)]), 25);
    // Charge immunity cancels the bonus.
    assert_eq!(charge("pikeman", (7, 5), &[]), 20);
    // No bonus without moving, nor on the retaliation.
    let r = ruleset();
    let mut s = battle(&r, &[(0, "knight", 10)], &[(0, "fixed", 10)], 1);
    arrange(&mut s, &r, &[(7, 5), (8, 5)]);
    assert_eq!(strikes(&hit(&mut s, &r, 1, (7, 5)))[0].2, 20);
    // The defender (8 left) attacks the knight (84 HP left, 68 after): 7 retaliate for 14.
    let events = hit(&mut s, &r, 0, (8, 5));
    assert_eq!(strikes(&events)[1], (0, 1, 14, 1, R));
}

// ---------------------------------------------------------------- BAT-DMG-006 defense reduction

// covers: BAT-DMG-006
#[test]
fn defense_reduction_ignores_a_share_of_the_defense_rounded_down() {
    // Without quirk Q-0005 the defense left rounds toward 0 (Q19).
    assert_eq!(reduced_defense(17, 40, false), 10);
    assert_eq!(reduced_defense(17, 80, false), 3);
    assert_eq!(reduced_defense(7, 40, false), 4);
    assert_eq!(reduced_defense(-3, 40, false), -1);
    assert_eq!(reduced_defense(0, 40, false), 0);
    assert_eq!(reduced_defense(5, 100, false), 0);
    // With it the defense left is the executable's, computed from single-precision values and
    // then rounded toward 0 (0x4425C8 to 0x4425F5): 17 − 17 × 0.40000001 = 10.19 → 10,
    // 17 − 17 × 0.80000001 = 3.39 → 3, 7 → 4.19 → 4 and 1.39 → 1, 10 − 4.00000006 = 5.99 → 5,
    // 5 − 4.00000006 = 0.99 → 0, 1 → 0.59 → 0, −3 → −1.79 → −1. (This pinned the share taken
    // rounded on its own, one point more each: 11, 4, 5, 2, 6, −2; review r2b F-1.)
    assert_eq!(reduced_defense(17, 40, true), 10);
    assert_eq!(reduced_defense(17, 80, true), 3);
    assert_eq!(reduced_defense(7, 40, true), 4);
    assert_eq!(reduced_defense(7, 80, true), 1);
    assert_eq!(reduced_defense(10, 40, true), 5);
    assert_eq!(reduced_defense(5, 80, true), 0);
    assert_eq!(reduced_defense(1, 40, true), 0);
    assert_eq!(reduced_defense(-3, 40, true), -1);
    assert_eq!(reduced_defense(0, 40, true), 0);
    assert_eq!(reduced_defense(5, 100, true), 0);
    // An independent source for every defense up to 300 (exact rational arithmetic on the two
    // single-precision constants gives the same): VCMI ignores `floor(defense × percent / 100)
    // + 1` points, at most the defense (lib/battle/DamageCalculator.cpp).
    for defense in 1..=300 {
        for percent in [40u8, 80] {
            let ignored = (defense * i32::from(percent) / 100 + 1).min(defense);
            let left = defense - ignored;
            assert_eq!(reduced_defense(defense, percent, true), left);
            assert_eq!(reduced_defense(-defense, percent, true), -left);
        }
    }
    let r = ruleset();
    // Tank defense 20 → 12: 10 − 12 = −2 points, −5 %: 20 → 19.
    let mut s = battle(&r, &[(0, "behemoth", 10)], &[(0, "tank", 1)], 1);
    arrange(&mut s, &r, &[(7, 5), (8, 5)]);
    assert_eq!(strikes(&hit(&mut s, &r, 1, (7, 5)))[0].2, 19);
    // Defending (20 + 4 = 24) is reduced as a whole: 24 → 14, −10 %: 18.
    let mut s = battle(&r, &[(0, "behemoth", 10)], &[(0, "tank", 1)], 1);
    arrange(&mut s, &r, &[(7, 5), (8, 5)]);
    stack(&mut s, 1).flags.set(StackFlags::DEFENDING, true);
    assert_eq!(strikes(&hit(&mut s, &r, 1, (7, 5)))[0].2, 18);
    // With quirk Q-0005 the strike reads the executable's value: the defending Tank's 24 leave
    // 14 (24 − 9.6000001 = 14.39), −10 %: 18 (the share rounded on its own left 15: 17).
    let r = ruleset_with("t:single", |_| {});
    let mut s = battle(&r, &[(0, "behemoth", 10)], &[(0, "tank", 1)], 1);
    arrange(&mut s, &r, &[(7, 5), (8, 5)]);
    stack(&mut s, 1).flags.set(StackFlags::DEFENDING, true);
    assert_eq!(strikes(&hit(&mut s, &r, 1, (7, 5)))[0].2, 18);
}

// ---------------------------------------------------------------- BAT-DMG-007 death blow

// covers: BAT-DMG-007
#[test]
fn death_blows_add_the_base_damage_again_also_on_retaliations() {
    let db = StrikeFlags::DEATH_BLOW;
    let r = ruleset();
    let mut s = battle(&r, &[(0, "doom", 10)], &[(0, "fixed", 10)], 1);
    arrange(&mut s, &r, &[(7, 5), (8, 5)]);
    let events = hit(&mut s, &r, 1, (7, 5));
    assert_eq!(strikes(&events), vec![(0, 1, 40, 4, db), (1, 0, 12, 1, R)]);
    // The defender (6 left) attacks; 8 dooms retaliate with a death blow: 16 · 2 = 32.
    let events = hit(&mut s, &r, 0, (8, 5));
    assert_eq!(strikes(&events)[1], (0, 1, 32, 3, R.with(db)));
    // The bonus adds to the others before the reductions: 20 · (1 + 1) · 0.75 = 30.
    let mut s = battle(&r, &[(0, "doom", 10)], &[(0, "tank", 1)], 1);
    arrange(&mut s, &r, &[(7, 5), (8, 5)]);
    assert_eq!(strikes(&hit(&mut s, &r, 1, (7, 5)))[0].2, 30);
}

// covers: BAT-DMG-007
#[test]
fn death_blows_happen_with_their_chance() {
    let r = ruleset();
    let blows = (0..500u64)
        .filter(|&seed| {
            let mut s = battle(&r, &[(0, "dread", 10)], &[(0, "fixed", 10)], seed);
            arrange(&mut s, &r, &[(7, 5), (8, 5)]);
            let first = strikes(&hit(&mut s, &r, 1, (7, 5)))[0];
            assert_eq!(
                first.2,
                if first.4.contains(StrikeFlags::DEATH_BLOW) {
                    40
                } else {
                    20
                }
            );
            first.4.contains(StrikeFlags::DEATH_BLOW)
        })
        .count();
    // 20 % of 500 = 100 (σ ≈ 9).
    assert!((75..=125).contains(&blows), "{blows}");
}

// ---------------------------------------------------------------- BAT-DMG-008 life drain

/// A vampire ×`base` brought down to `count` creatures with `top` HP attacks `target` ×`n`;
/// returns the events and the vampire afterwards.
fn drain(
    base: u32,
    count: u32,
    top: u32,
    target: &'static str,
    n: u32,
) -> (Vec<Event>, crate::Stack) {
    let r = ruleset();
    let mut s = battle(&r, &[(0, "vampire", base)], &[(0, target, n)], 1);
    arrange(&mut s, &r, &[(7, 5), (8, 5)]);
    let v = stack(&mut s, 0);
    v.count = count;
    v.top_hp = top;
    let events = hit(&mut s, &r, 1, (7, 5));
    (events, s.stacks()[0])
}

fn drained(events: &[Event]) -> Vec<(u64, u32)> {
    events
        .iter()
        .filter_map(|e| match e.kind {
            EventKind::LifeDrained {
                hit_points,
                resurrected,
                ..
            } => Some((hit_points, resurrected)),
            _ => None,
        })
        .collect()
}

// covers: BAT-DMG-008
#[test]
fn life_drain_heals_and_resurrects_before_the_retaliation() {
    // 7 of 10 vampires deal 14: back to 84 HP (9 creatures, 4 HP on top); then 9 serfs
    // retaliate for 18.
    let (events, v) = drain(10, 7, 10, "serf", 10);
    let kinds: Vec<EventKind> = events.iter().map(|e| e.kind).collect();
    let at_drain = kinds
        .iter()
        .position(|k| matches!(k, EventKind::LifeDrained { .. }))
        .unwrap();
    assert!(matches!(
        kinds[at_drain - 1],
        EventKind::Struck { flags: N, .. }
    ));
    assert!(matches!(
        kinds[at_drain + 1],
        EventKind::Struck { flags: R, .. }
    ));
    assert_eq!(drained(&events), vec![(14, 2)]);
    assert_eq!((v.count, v.top_hp), (7, 6));
}

// covers: BAT-DMG-008
#[test]
fn life_drain_is_capped_by_the_starting_count_and_the_targets_health() {
    // 95 of 100 HP: only 5 can be regained.
    let (events, _) = drain(10, 10, 5, "serf", 10);
    assert_eq!(drained(&events), vec![(5, 0)]);
    // 20 damage on a 10-HP target drains 10.
    let (events, v) = drain(20, 10, 10, "serf", 1);
    assert_eq!(drained(&events), vec![(10, 1)]);
    assert_eq!((v.count, v.top_hp), (11, 10));
    // Nothing to heal, or a target that is not alive: no drain.
    assert!(drained(&drain(10, 10, 10, "serf", 10).0).is_empty());
    assert!(drained(&drain(10, 7, 10, "fixed", 10).0).is_empty());
}

// ---------------------------------------------------------------- BAT-DMG-009 death stare

fn stares(events: &[Event]) -> Vec<(u8, u8, u32)> {
    events
        .iter()
        .filter_map(|e| match e.kind {
            EventKind::DeathStared {
                attacker,
                target,
                kills,
            } => Some((attacker.0, target.0, kills)),
            _ => None,
        })
        .collect()
}

// covers: BAT-DMG-009
#[test]
fn death_stare_kills_whole_creatures_after_the_strike() {
    let r = ruleset();
    // 3 starers deal 6 (94 HP left, the top serf at 4 HP), then stare 3 serfs dead, the damaged
    // top one first: 7 left, the new top one at full health (Q21, batch 1a R4); the 7 retaliate
    // for 14.
    let mut s = battle(&r, &[(0, "starer", 3)], &[(0, "serf", 10)], 1);
    arrange(&mut s, &r, &[(7, 5), (8, 5)]);
    let events = hit(&mut s, &r, 1, (7, 5));
    assert_eq!(stares(&events), vec![(0, 1, 3)]);
    assert_eq!((s.stacks()[1].count, s.stacks()[1].top_hp), (7, 10));
    assert_eq!(strikes(&events), vec![(0, 1, 6, 0, N), (1, 0, 14, 1, R)]);
    let kinds: Vec<EventKind> = events.iter().map(|e| e.kind).collect();
    assert!(matches!(kinds[0], EventKind::Struck { .. }));
    assert!(matches!(kinds[1], EventKind::DeathStared { .. }));
    assert!(matches!(kinds[2], EventKind::Struck { .. }));
    // A retaliation stares too: 1 starer left after 20 damage, it retaliates (the top serf at
    // 8 HP) and kills 1: 9 left, the top one at full health.
    let mut s = battle(&r, &[(0, "serf", 10)], &[(0, "starer", 3)], 1);
    arrange(&mut s, &r, &[(7, 5), (8, 5)]);
    let events = hit(&mut s, &r, 1, (7, 5));
    assert_eq!(strikes(&events), vec![(0, 1, 20, 2, N), (1, 0, 2, 0, R)]);
    assert_eq!(stares(&events), vec![(1, 0, 1)]);
    assert_eq!((s.stacks()[0].count, s.stacks()[0].top_hp), (9, 10));
}

// covers: BAT-DMG-009
#[test]
fn death_stare_needs_a_living_surviving_target_and_stops_at_its_size() {
    let r = ruleset();
    let run = |starers: u32, target: &'static str, n: u32| {
        let mut s = battle(&r, &[(0, "starer", starers)], &[(0, target, n)], 1);
        arrange(&mut s, &r, &[(7, 5), (8, 5)]);
        let events = hit(&mut s, &r, 1, (7, 5));
        (stares(&events), ended(&events))
    };
    // Not alive.
    assert_eq!(run(3, "fixed", 10), (vec![], false));
    // Killed by the strike.
    assert_eq!(run(10, "serf", 1), (vec![], true));
    // 5 starers deal 10 (2 serfs left) and stare both dead.
    assert_eq!(run(5, "serf", 3), (vec![(0, 1, 2)], true));
}

// covers: BAT-DMG-009
#[test]
fn death_stare_kills_at_most_a_tenth_rounded_up_with_its_chance() {
    let r = ruleset();
    let mut stared = 0;
    for seed in 0..300u64 {
        let mut s = battle(&r, &[(0, "gorgon", 10)], &[(0, "serf", 100)], seed);
        arrange(&mut s, &r, &[(7, 5), (8, 5)]);
        let found = stares(&hit(&mut s, &r, 1, (7, 5)));
        // At most ceil(10 · 10 / 100) = 1 kill.
        assert!(found.iter().all(|&(_, _, kills)| kills == 1), "{found:?}");
        stared += found.len();
    }
    // P(at least one of 10 succeeds) = 1 − 0.9^10 ≈ 65 %: 195 of 300 (σ ≈ 8).
    assert!((165..=225).contains(&stared), "{stared}");
}

// covers: BAT-DMG-009
#[test]
fn a_huge_stack_draws_its_stare_kills_at_once() {
    let r = ruleset();
    // 100 000 Gorgons (above the roll limit) deal 200 000 to 2 000 000 serfs (20 000 die), then
    // stare: about 10 % of them, at most 10 000 (ceil of a tenth), drawn at once (KER-RNG-003).
    const { assert!(100_000 > crate::combat::STARE_ROLL_LIMIT) };
    let mut capped = 0;
    for seed in 0..20u64 {
        let mut s = battle(
            &r,
            &[(0, "gorgon", 100_000)],
            &[(0, "serf", 2_000_000)],
            seed,
        );
        arrange(&mut s, &r, &[(7, 5), (8, 5)]);
        let found = stares(&hit(&mut s, &r, 1, (7, 5)));
        let kills = found[0].2;
        assert!((9_500..=10_000).contains(&kills), "{kills}");
        assert_eq!(s.stacks()[1].count, 2_000_000 - 20_000 - kills);
        capped += usize::from(kills == 10_000);
    }
    // The cap is the mean: about half the draws reach it.
    assert!((4..=16).contains(&capped), "{capped}");
}

// ---------------------------------------------------------------- BAT-TRN-005 regeneration

fn regenerated(events: &[Event]) -> Vec<(u8, u32)> {
    events
        .iter()
        .filter_map(|e| match e.kind {
            EventKind::Regenerated { stack, hit_points } => Some((stack.0, hit_points)),
            _ => None,
        })
        .collect()
}

// covers: BAT-TRN-005
#[test]
fn regeneration_restores_the_top_creature_at_the_start_of_its_normal_turn() {
    let r = ruleset();
    // 0 = fixed (attacker, acts first), 1 = wight ×5.
    let mut s = battle(&r, &[(0, "fixed", 1)], &[(0, "wight", 5)], 1);
    stack(&mut s, 1).top_hp = 3;
    let events = act(&mut s, &r, Command::Defend).unwrap();
    assert_eq!(regenerated(&events), vec![(1, 7)]);
    assert_eq!(s.stacks()[1].top_hp, 10);
    // Not when it acts in the wait phase.
    stack(&mut s, 1).top_hp = 3;
    let events = act(&mut s, &r, Command::Wait).unwrap();
    assert_eq!(s.active(), Some(StackId(1)));
    assert!(regenerated(&events).is_empty());
    assert_eq!(s.stacks()[1].top_hp, 3);
    // Next round, at its normal turn again.
    act(&mut s, &r, Command::Defend).unwrap();
    let events = act(&mut s, &r, Command::Defend).unwrap();
    assert_eq!(regenerated(&events), vec![(1, 7)]);
    // Nothing happens at full health.
    act(&mut s, &r, Command::Defend).unwrap();
    let events = act(&mut s, &r, Command::Defend).unwrap();
    assert_eq!(s.active(), Some(StackId(1)));
    assert!(regenerated(&events).is_empty());
}

// ---------------------------------------------------------------- geometry helpers

// covers: BAT-ATK-003
#[test]
fn directions_turn_clockwise_and_connect_neighbours() {
    assert_eq!(Direction::Right.turned(1), Direction::BottomRight);
    assert_eq!(Direction::TopRight.turned(1), Direction::Right);
    assert_eq!(Direction::Left.turned(6), Direction::Left);
    let r = ruleset();
    let f = field(&r);
    let (a, b) = (at(&r, 7, 5), at(&r, 6, 6));
    assert_eq!(f.direction(a, b), Some(Direction::BottomLeft));
    assert_eq!(f.direction(b, a), Some(Direction::TopRight));
    assert_eq!(f.direction(a, at(&r, 9, 5)), None);
}

// covers: BAT-ATK-003
#[test]
fn with_quirk_bat_quirk_28_a_breath_never_reaches_a_side_column() {
    // Batch 2g BR, BC (BAT-Q62): the drake's front hex on (15, 5) strikes the stack on (15, 4),
    // and the defender's Ballista's rear hex (16, 3) lies up-right behind it: VCMI's pattern
    // breathes onto it, the original's neighbour table (no side columns) does not. The control:
    // from (13, 3) onto the stack on (14, 3) the breath reaches the Ballista's front hex (15, 3).
    for (profile, reaches) in [("t:faithful", true), ("t:table_breath", false)] {
        let r = ruleset_with(profile, |_| {});
        let ballista = r.creature_by_key("t:ballista").unwrap();
        let led = crate::BattleSetup {
            heroes: [None, Some(bare_hero(&r))],
            war_machines: [vec![], vec![ballista]],
            ..setup(&r, &[(0, "drake", 10)], &[(0, "fixed", 10)])
        };
        // Stacks: 0 the drake, 1 the fixed, 2 the Ballista on (16, 3), its front hex (15, 3).
        let mut s = BattleState::new(&r, &led, 1).unwrap();
        assert_eq!(field(&r).coords(s.stacks()[2].hex), (16, 3));
        arrange(&mut s, &r, &[(14, 5), (15, 4)]);
        let events = hit(&mut s, &r, 1, (14, 5));
        assert_eq!(targets_of(&events, 0).contains(&2), reaches, "{profile}");
        let mut s = BattleState::new(&r, &led, 1).unwrap();
        arrange(&mut s, &r, &[(12, 3), (14, 3)]);
        let events = hit(&mut s, &r, 1, (12, 3));
        assert_eq!(targets_of(&events, 0), [1, 2], "{profile}");
    }
}
