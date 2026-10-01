//! One test group per battle rule (docs/rules/catalog/battle.toml).

use std::vec;
use std::vec::Vec;

use h3_contracts::{ArmyStack, BattleSide};
use h3_kernel::Rng;

use super::support::*;
use crate::damage::{Factors, apply_damage, attack_defense, base_damage, effective_defense};
use crate::event::StrikeFlags;

/// Damage of `base` with only the attack/defense factor (BAT-DMG-002).
fn modified_damage(base: u64, attack: u16, defense: u16, p: &h3_content::BattleParams) -> u64 {
    let mut f = Factors::default();
    attack_defense(&mut f, attack, defense, p);
    f.apply(base)
}
use crate::{
    Command, CommandError, Event, EventKind, LegalCommands, StackFlags, StackId, legal_commands,
};

// ---------------------------------------------------------------- BAT-SET-001 placement

// covers: BAT-SET-001
#[test]
fn armies_start_on_their_edge_columns_in_formation_rows() {
    let r = ruleset();
    let s = battle(
        &r,
        &[(4, "fixed", 1), (0, "slow", 2), (2, "fast", 3)],
        &[(6, "tank", 1)],
        1,
    );
    // Attacker stacks get ids in slot order; rows come from formation[3 - 1] = [2, 5, 8].
    let st = s.stacks();
    assert_eq!(st.len(), 4);
    assert_eq!((st[0].slot, st[0].hex), (0, at(&r, 1, 2)));
    assert_eq!((st[1].slot, st[1].hex), (2, at(&r, 1, 5)));
    assert_eq!((st[2].slot, st[2].hex), (4, at(&r, 1, 8)));
    // A one-stack army uses formation[0] = [5] in the last standable column.
    assert_eq!(
        (st[3].side, st[3].slot, st[3].hex),
        (BattleSide::Defender, 6, at(&r, 15, 5))
    );
    // Creature stats are copied into the stack.
    assert_eq!(
        (st[3].defense, st[3].max_hp, st[3].top_hp, st[3].count),
        (20, 100, 100, 1)
    );
    assert_eq!(s.round(), 1);
}

// ---------------------------------------------------------------- BAT-TRN-001 turn order

// covers: BAT-TRN-001
#[test]
fn faster_stacks_act_first_and_speed_ties_alternate_sides() {
    let r = ruleset();
    // Attacker: 0 = fixed (5), 1 = fast (9). Defender: 2 = fast (9), 3 = fixed (5), 4 = slow (2).
    let mut s = battle(
        &r,
        &[(0, "fixed", 1), (1, "fast", 1)],
        &[(0, "fast", 1), (1, "fixed", 1), (2, "slow", 1)],
        1,
    );
    // Speed 9 tie at the start: attacker first. Then the side that did not act last wins ties.
    assert_eq!(order(&mut s, &r, 5), vec![1, 2, 0, 3, 4]);
    assert_eq!(s.round(), 2);
    // The defender acted last in round 1, so the attacker wins the next tie again.
    assert_eq!(order(&mut s, &r, 5), vec![1, 2, 0, 3, 4]);
}

// covers: BAT-TRN-001
#[test]
fn tie_breaking_carries_over_between_rounds_and_uses_slot_order_within_a_side() {
    let r = ruleset();
    // Attacker: 0 = fixed (5, slot 0), 1 = slow (2). Defender: 2 = fixed (5).
    let mut s = battle(
        &r,
        &[(0, "fixed", 1), (3, "slow", 1)],
        &[(0, "fixed", 1)],
        1,
    );
    assert_eq!(order(&mut s, &r, 3), vec![0, 2, 1]);
    // The attacker (slow) acted last in round 1: the defender wins the speed-5 tie in round 2.
    assert_eq!(order(&mut s, &r, 3), vec![2, 0, 1]);
    // Same side, same speed: lower slot first.
    let mut t = battle(
        &r,
        &[(5, "fixed", 1), (1, "fixed", 1)],
        &[(0, "slow", 1)],
        1,
    );
    assert_eq!(t.stacks()[0].slot, 1);
    assert_eq!(order(&mut t, &r, 3), vec![0, 1, 2]);
}

// ---------------------------------------------------------------- BAT-TRN-002 wait

// covers: BAT-TRN-002
#[test]
fn waiting_stacks_act_after_everyone_else_slowest_first() {
    let r = ruleset();
    // Attacker: 0 = fixed (5), 1 = fast (9). Defender: 2 = slow (2).
    let mut s = battle(&r, &[(0, "fixed", 1), (1, "fast", 1)], &[(0, "slow", 1)], 1);
    assert_eq!(s.active(), Some(StackId(1)));
    let events = act(&mut s, &r, Command::Wait).unwrap();
    assert_eq!(
        events[0],
        Event::public(EventKind::Waited { stack: StackId(1) })
    );
    assert_eq!(s.active(), Some(StackId(0)));
    act(&mut s, &r, Command::Wait).unwrap();
    assert_eq!(s.active(), Some(StackId(2)));
    act(&mut s, &r, Command::Defend).unwrap();
    // Wait phase: ascending speed, so fixed (5) before fast (9).
    assert_eq!(s.active(), Some(StackId(0)));
    assert_eq!(
        act(&mut s, &r, Command::Wait),
        Err(CommandError::AlreadyWaited)
    );
    // Wait-then-defend is allowed.
    act(&mut s, &r, Command::Defend).unwrap();
    assert_eq!(s.active(), Some(StackId(1)));
    act(&mut s, &r, Command::Defend).unwrap();
    assert_eq!(s.round(), 2);
    // A waited stack cannot wait again, so Wait is not offered.
    let mut legal = LegalCommands::new();
    let mut t = battle(&r, &[(0, "fixed", 1)], &[(0, "slow", 1)], 1);
    act(&mut t, &r, Command::Wait).unwrap();
    act(&mut t, &r, Command::Defend).unwrap();
    legal_commands(&t, &r, BattleSide::Attacker, &mut legal);
    assert!(!legal.iter().any(|c| *c == Command::Wait));
    assert!(legal.iter().any(|c| *c == Command::Defend));
}

// covers: BAT-TRN-002
#[test]
fn wait_phase_ties_follow_the_same_tie_rules() {
    let r = ruleset();
    // Attacker 0 = fixed (5); defender 1 = fixed (5). Both wait.
    let mut s = battle(&r, &[(0, "fixed", 1)], &[(0, "fixed", 1)], 1);
    assert_eq!(s.active(), Some(StackId(0)));
    act(&mut s, &r, Command::Wait).unwrap();
    act(&mut s, &r, Command::Wait).unwrap();
    // The defender waited last, so the attacker wins the tie in the wait phase.
    assert_eq!(s.active(), Some(StackId(0)));
}

// ---------------------------------------------------------------- BAT-TRN-003 rounds

// covers: BAT-TRN-003
#[test]
fn new_round_resets_per_round_flags() {
    let r = ruleset();
    let mut s = battle(&r, &[(0, "fixed", 5)], &[(0, "fixed", 5)], 1);
    place(&mut s, &r, 1, 2, 5);
    // Attacker attacks from its own hex (adjacent); defender retaliates.
    let events = act(
        &mut s,
        &r,
        Command::Attack {
            target: StackId(1),
            from: at(&r, 1, 5),
        },
    )
    .unwrap();
    assert!(s.stacks()[1].flags.contains(StackFlags::RETALIATED));
    assert!(s.stacks()[0].flags.contains(StackFlags::ACTED));
    assert!(matches!(
        events.last(),
        Some(Event {
            kind: EventKind::TurnStarted { stack: StackId(1) },
            ..
        })
    ));
    let events = act(&mut s, &r, Command::Wait).unwrap();
    assert!(
        events
            .iter()
            .all(|e| !matches!(e.kind, EventKind::RoundStarted { .. }))
    );
    let events = act(&mut s, &r, Command::Defend).unwrap();
    assert!(events.contains(&Event::public(EventKind::RoundStarted { round: 2 })));
    assert_eq!(s.round(), 2);
    for st in s.stacks() {
        assert!(!st.flags.contains(StackFlags::ACTED));
        assert!(!st.flags.contains(StackFlags::WAITED));
        assert!(!st.flags.contains(StackFlags::RETALIATED));
    }
}

// ---------------------------------------------------------------- BAT-TRN-004 defend

// covers: BAT-TRN-004
#[test]
fn defend_adds_a_fifth_of_defense_rounded_down_but_at_least_one() {
    let r = ruleset();
    let p = r.battle();
    let s = battle(&r, &[(0, "tank", 1), (1, "slow", 1)], &[(0, "fixed", 1)], 1);
    let mut tank = s.stacks()[0];
    let mut slow = s.stacks()[1];
    assert_eq!(effective_defense(&tank, p), 20);
    tank.flags.set(StackFlags::DEFENDING, true);
    slow.flags.set(StackFlags::DEFENDING, true);
    assert_eq!(effective_defense(&tank, p), 24);
    assert_eq!(effective_defense(&slow, p), 2);
    tank.defense = u16::MAX;
    assert_eq!(effective_defense(&tank, p), u16::MAX);
}

// covers: BAT-TRN-004
#[test]
fn defending_ends_the_turn_and_lasts_until_the_next_activation() {
    let r = ruleset();
    // Attacker: 0 = tank (4). Defender: 1 = fixed (5) ×10, acts first.
    let mut s = battle(&r, &[(0, "tank", 1)], &[(0, "fixed", 10)], 1);
    place(&mut s, &r, 1, 2, 5);
    assert_eq!(s.active(), Some(StackId(1)));
    act(&mut s, &r, Command::Defend).unwrap();
    assert_eq!(s.active(), Some(StackId(0)));
    act(&mut s, &r, Command::Defend).unwrap();
    // Round 2: fixed acts first and attacks the defending tank: base 20, defense 20 + 4 = 24,
    // 14 points over attack 10 → −35 % → 13 (not defending it would be −25 % → 15).
    assert_eq!(s.round(), 2);
    assert!(s.stacks()[0].flags.contains(StackFlags::DEFENDING));
    let events = act(
        &mut s,
        &r,
        Command::Attack {
            target: StackId(0),
            from: at(&r, 2, 5),
        },
    )
    .unwrap();
    assert!(
        matches!(
            events[0].kind,
            EventKind::Struck {
                damage: 13,
                flags: StrikeFlags::NONE,
                ..
            }
        ),
        "{events:?}"
    );
    // The tank's own activation clears the bonus.
    assert_eq!(s.active(), Some(StackId(0)));
    assert!(!s.stacks()[0].flags.contains(StackFlags::DEFENDING));
    assert!(events.contains(&Event::public(EventKind::TurnStarted { stack: StackId(0) })));
}

// ---------------------------------------------------------------- BAT-MOV-001 movement

// covers: BAT-MOV-001
#[test]
fn walkers_move_up_to_speed_along_free_hexes() {
    let r = ruleset();
    // fixed has speed 5.
    let mut s = battle(&r, &[(0, "fixed", 1)], &[(0, "slow", 1)], 1);
    assert_eq!(s.stacks()[0].hex, at(&r, 1, 5));
    assert_eq!(
        act(&mut s.clone(), &r, Command::Move { to: at(&r, 7, 5) }),
        Err(CommandError::Unreachable)
    );
    let events = act(&mut s, &r, Command::Move { to: at(&r, 6, 5) }).unwrap();
    assert_eq!(
        events[0],
        Event::public(EventKind::Moved {
            stack: StackId(0),
            from: at(&r, 1, 5),
            to: at(&r, 6, 5)
        })
    );
    assert_eq!(s.stacks()[0].hex, at(&r, 6, 5));
}

// covers: BAT-MOV-001
#[test]
fn living_stacks_block_dead_stacks_do_not() {
    let r = ruleset();
    let mut s = battle(
        &r,
        &[(0, "fixed", 1), (1, "slow", 1), (2, "slow", 1)],
        &[(0, "slow", 1)],
        1,
    );
    activate(&mut s, 0);
    // Wall the mover in at (1, 5): its standable neighbours are (2, 5), (1, 4), (2, 4)... use three
    // stacks on the right-hand neighbours and the field edge (column 0 is not standable).
    place(&mut s, &r, 0, 1, 0);
    place(&mut s, &r, 1, 2, 0);
    place(&mut s, &r, 2, 1, 1);
    place(&mut s, &r, 3, 2, 1);
    let mut legal = LegalCommands::new();
    legal_commands(&s, &r, BattleSide::Attacker, &mut legal);
    assert!(
        !legal.iter().any(|c| matches!(c, Command::Move { .. })),
        "{legal:?}"
    );
    assert_eq!(
        act(&mut s.clone(), &r, Command::Move { to: at(&r, 3, 0) }),
        Err(CommandError::Unreachable)
    );
    // Occupied destination.
    assert_eq!(
        act(&mut s.clone(), &r, Command::Move { to: at(&r, 2, 0) }),
        Err(CommandError::Unreachable)
    );
    // Kill the blocker at (2, 0): its hex becomes free.
    s.stacks.as_mut_slice()[1].count = 0;
    s.stacks.as_mut_slice()[1].top_hp = 0;
    act(&mut s, &r, Command::Move { to: at(&r, 3, 0) }).unwrap();
}

// covers: BAT-MOV-001
#[test]
fn side_columns_and_own_hex_are_not_move_targets() {
    let r = ruleset();
    let mut s = battle(&r, &[(0, "fixed", 1)], &[(0, "slow", 1)], 1);
    assert_eq!(
        act(&mut s.clone(), &r, Command::Move { to: at(&r, 0, 5) }),
        Err(CommandError::InvalidHex)
    );
    assert_eq!(
        act(
            &mut s.clone(),
            &r,
            Command::Move {
                to: crate::Hex(250)
            }
        ),
        Err(CommandError::InvalidHex)
    );
    assert_eq!(
        act(&mut s, &r, Command::Move { to: at(&r, 1, 5) }),
        Err(CommandError::Unreachable)
    );
}

// ---------------------------------------------------------------- BAT-ATK-001 melee attack

// covers: BAT-ATK-001
#[test]
fn melee_attacks_move_to_an_adjacent_hex_first() {
    let r = ruleset();
    let mut s = battle(&r, &[(0, "fixed", 10)], &[(0, "slow", 10)], 3);
    place(&mut s, &r, 1, 5, 5);
    // From (4, 5): 3 steps away, adjacent to the target.
    let events = act(
        &mut s,
        &r,
        Command::Attack {
            target: StackId(1),
            from: at(&r, 4, 5),
        },
    )
    .unwrap();
    assert_eq!(
        events[0],
        Event::public(EventKind::Moved {
            stack: StackId(0),
            from: at(&r, 1, 5),
            to: at(&r, 4, 5)
        })
    );
    assert!(matches!(
        events[1].kind,
        EventKind::Struck {
            attacker: StackId(0),
            target: StackId(1),
            flags: StrikeFlags::NONE,
            ..
        }
    ));
    assert_eq!(s.stacks()[0].hex, at(&r, 4, 5));
}

// covers: BAT-ATK-001
#[test]
fn attack_position_must_be_adjacent_reachable_and_free() {
    let r = ruleset();
    let mut s = battle(
        &r,
        &[(0, "fixed", 10), (1, "slow", 1)],
        &[(0, "slow", 10)],
        3,
    );
    activate(&mut s, 0);
    place(&mut s, &r, 0, 1, 5);
    place(&mut s, &r, 2, 5, 5);
    place(&mut s, &r, 1, 4, 5);
    let attack = |from| Command::Attack {
        target: StackId(2),
        from,
    };
    assert_eq!(
        act(&mut s.clone(), &r, attack(at(&r, 3, 5))),
        Err(CommandError::NotAdjacent)
    );
    assert_eq!(
        act(&mut s.clone(), &r, attack(at(&r, 4, 5))),
        Err(CommandError::Unreachable)
    );
    assert_eq!(
        act(&mut s.clone(), &r, attack(crate::Hex(255))),
        Err(CommandError::InvalidHex)
    );
    place(&mut s, &r, 2, 12, 5);
    assert_eq!(
        act(&mut s.clone(), &r, attack(at(&r, 11, 5))),
        Err(CommandError::Unreachable)
    );
    // Standing next to the target: attack without moving.
    place(&mut s, &r, 2, 2, 5);
    let events = act(&mut s, &r, attack(at(&r, 1, 5))).unwrap();
    assert!(matches!(events[0].kind, EventKind::Struck { .. }));
}

// ---------------------------------------------------------------- BAT-DMG-001 base damage

// covers: BAT-DMG-001
#[test]
fn small_stacks_roll_every_creature() {
    let mut rng = Rng::from_seed(5);
    assert_eq!(base_damage(&mut rng, 7, 2, 2, 10), 14);
    assert_eq!(base_damage(&mut rng, 10, 2, 2, 10), 20);
    // Each creature rolls independently: replay the draws.
    let mut a = Rng::from_seed(9);
    let mut b = a.clone();
    let expected: u64 = (0..6).map(|_| u64::from(b.range_inclusive(1, 9))).sum();
    assert_eq!(base_damage(&mut a, 6, 1, 9, 10), expected);
}

// covers: BAT-DMG-001
#[test]
fn large_stacks_scale_ten_rolls_rounding_down() {
    let mut rng = Rng::from_seed(5);
    assert_eq!(base_damage(&mut rng, 25, 2, 2, 10), 50);
    let mut a = Rng::from_seed(11);
    let mut b = a.clone();
    let sum: u64 = (0..10).map(|_| u64::from(b.range_inclusive(1, 9))).sum();
    assert_eq!(base_damage(&mut a, 13, 1, 9, 10), sum * 13 / 10);
    // Only ten draws are consumed.
    assert_eq!(a, b);
    // 3 creatures with 1–1 damage and 2 samples: floor(2 * 3 / 2) = 3; 5 creatures: floor(2*5/2).
    assert_eq!(base_damage(&mut rng, 3, 1, 1, 2), 3);
    // The largest possible stack does not overflow.
    assert_eq!(
        base_damage(&mut rng, u32::MAX, u16::MAX, u16::MAX, 1),
        u64::from(u32::MAX) * u64::from(u16::MAX)
    );
}

// covers: BAT-DMG-001
#[test]
fn exactly_ten_creatures_match_both_formulations() {
    // Q12: "10 or more creatures: roll 10, sum, times N/10, round down". For N = 10 that is the
    // same as summing ten individual rolls, and consumes the same ten draws.
    let mut a = Rng::from_seed(77);
    let mut b = a.clone();
    let sum: u64 = (0..10).map(|_| u64::from(b.range_inclusive(1, 9))).sum();
    assert_eq!(base_damage(&mut a, 10, 1, 9, 10), sum * 10 / 10);
    assert_eq!(a, b);
}

// covers: BAT-DMG-001
#[test]
fn damage_rolls_cover_the_range_uniformly() {
    let mut rng = Rng::from_seed(21);
    let mut counts = [0u32; 10];
    for _ in 0..9_000 {
        counts[base_damage(&mut rng, 1, 1, 9, 10) as usize] += 1;
    }
    assert_eq!(counts[0], 0);
    for c in &counts[1..] {
        assert!((850..=1150).contains(c), "{counts:?}");
    }
}

// ---------------------------------------------------------------- BAT-DMG-002 modifiers

// covers: BAT-DMG-002
#[test]
fn attack_and_defense_difference_modifies_damage() {
    let r = ruleset();
    let p = r.battle();
    assert_eq!(modified_damage(100, 10, 10, p), 100);
    assert_eq!(modified_damage(100, 12, 10, p), 110);
    assert_eq!(
        modified_damage(7, 11, 10, p),
        7,
        "7 × 1.05 = 7.35 rounds down"
    );
    assert_eq!(
        modified_damage(100, 10, 13, p),
        92,
        "100 × 0.925 = 92.5 rounds down"
    );
    // Caps: +300 % (60 points), −70 % (28 points).
    assert_eq!(modified_damage(100, 70, 10, p), 400);
    assert_eq!(modified_damage(100, 100, 10, p), 400);
    assert_eq!(modified_damage(100, 0, 28, p), 30);
    assert_eq!(modified_damage(100, 0, 40, p), 30);
    // At least 1.
    assert_eq!(modified_damage(1, 0, 40, p), 1);
    // Saturates instead of overflowing.
    assert_eq!(modified_damage(u64::MAX, 100, 0, p), u64::MAX);
}

// ---------------------------------------------------------------- BAT-DMG-003 applying damage

// covers: BAT-DMG-003
#[test]
fn damage_hits_the_top_creature_first() {
    let r = ruleset();
    let s = battle(&r, &[(0, "fixed", 5)], &[(0, "slow", 1)], 1);
    // Wiki example: 10 HP creatures, 25 damage kills 2 and leaves the top one at 5 HP.
    let mut st = s.stacks()[0];
    assert_eq!(apply_damage(&mut st, 25), 2);
    assert_eq!((st.count, st.top_hp, st.total_hp()), (3, 5, 25));
    assert_eq!(apply_damage(&mut st, 3), 0);
    assert_eq!((st.count, st.top_hp), (3, 2));
    assert_eq!(apply_damage(&mut st, 2), 1);
    assert_eq!((st.count, st.top_hp), (2, 10));
    assert_eq!(apply_damage(&mut st, 20), 2);
    assert_eq!((st.count, st.top_hp, st.total_hp()), (0, 0, 0));
    assert!(!st.is_alive());
    let mut big = s.stacks()[0];
    assert_eq!(apply_damage(&mut big, 1000), 5);
}

// ---------------------------------------------------------------- BAT-RET-001 retaliation

// covers: BAT-RET-001
#[test]
fn surviving_target_retaliates_once_per_round_with_its_remaining_creatures() {
    let r = ruleset();
    // 0 = fixed ×10 (attacker), 1 = fixed ×10, 2 = slow ×1 (defender).
    let mut s = battle(
        &r,
        &[(0, "fixed", 10), (1, "fixed", 10)],
        &[(0, "fixed", 10)],
        1,
    );
    place(&mut s, &r, 2, 2, 5);
    place(&mut s, &r, 0, 1, 5);
    place(&mut s, &r, 1, 3, 5);
    // Order: 0 (A), 2 (D), 1 (A). Stack 0 hits: 10 × 2 = 20 damage kills 2 of 10-HP creatures;
    // 8 survivors retaliate for 16, killing 1 and leaving the top attacker at 4 HP.
    let events = act(
        &mut s,
        &r,
        Command::Attack {
            target: StackId(2),
            from: at(&r, 1, 5),
        },
    )
    .unwrap();
    assert_eq!(
        &events[..2],
        &[
            Event::public(EventKind::Struck {
                attacker: StackId(0),
                target: StackId(2),
                damage: 20,
                kills: 2,
                flags: StrikeFlags::NONE
            }),
            Event::public(EventKind::Struck {
                attacker: StackId(2),
                target: StackId(0),
                damage: 16,
                kills: 1,
                flags: StrikeFlags::RETALIATION
            }),
        ]
    );
    assert_eq!((s.stacks()[0].count, s.stacks()[0].top_hp), (9, 4));
    act(&mut s, &r, Command::Defend).unwrap();
    // Second attack in the same round: no retaliation left.
    let events = act(
        &mut s,
        &r,
        Command::Attack {
            target: StackId(2),
            from: at(&r, 3, 5),
        },
    )
    .unwrap();
    assert!(
        !events.iter().any(|e| matches!(
            e.kind,
            EventKind::Struck {
                flags: StrikeFlags::RETALIATION,
                ..
            }
        )),
        "{events:?}"
    );
    // Next round, the retaliation is back (the defender acts first in round 2 and defends).
    assert_eq!(s.round(), 2);
    advance_to(&mut s, &r, 0);
    let events = act(
        &mut s,
        &r,
        Command::Attack {
            target: StackId(2),
            from: at(&r, 1, 5),
        },
    )
    .unwrap();
    assert!(
        events.iter().any(|e| matches!(
            e.kind,
            EventKind::Struck {
                flags: StrikeFlags::RETALIATION,
                ..
            }
        )),
        "{events:?}"
    );
}

// covers: BAT-RET-001
#[test]
fn killed_targets_do_not_retaliate() {
    let r = ruleset();
    let mut s = battle(
        &r,
        &[(0, "fixed", 50)],
        &[(0, "slow", 1), (1, "slow", 1)],
        1,
    );
    place(&mut s, &r, 1, 2, 5);
    let events = act(
        &mut s,
        &r,
        Command::Attack {
            target: StackId(1),
            from: at(&r, 1, 5),
        },
    )
    .unwrap();
    assert!(matches!(events[0].kind, EventKind::Struck { kills: 1, .. }));
    assert!(!events.iter().any(|e| matches!(
        e.kind,
        EventKind::Struck {
            flags: StrikeFlags::RETALIATION,
            ..
        }
    )));
    assert!(!s.stacks()[1].is_alive());
    assert!(!s.is_over());
}

// ---------------------------------------------------------------- BAT-END-001 battle end

// covers: BAT-END-001
#[test]
fn battle_ends_when_one_side_has_no_living_stacks() {
    let r = ruleset();
    let mut s = battle(
        &r,
        &[(0, "fixed", 50), (3, "slow", 2)],
        &[(2, "slow", 1)],
        1,
    );
    place(&mut s, &r, 2, 2, 5);
    place(&mut s, &r, 0, 1, 5);
    let events = act(
        &mut s,
        &r,
        Command::Attack {
            target: StackId(2),
            from: at(&r, 1, 5),
        },
    )
    .unwrap();
    assert_eq!(
        events.last(),
        Some(&Event::public(EventKind::BattleEnded {
            winner: BattleSide::Attacker
        }))
    );
    assert!(s.is_over());
    assert_eq!(s.winner(), Some(BattleSide::Attacker));
    assert_eq!(s.active(), None);
    assert_eq!(s.side_to_act(), None);
    let outcome = s.outcome().unwrap();
    assert_eq!(outcome.winner, BattleSide::Attacker);
    assert_eq!(
        outcome.survivors[0].stacks,
        vec![
            ArmyStack {
                slot: 0,
                creature: s.stacks()[0].creature,
                count: 50
            },
            ArmyStack {
                slot: 3,
                creature: s.stacks()[1].creature,
                count: 2
            },
        ]
    );
    assert!(outcome.survivors[1].stacks.is_empty());
    for side in BattleSide::BOTH {
        assert_eq!(
            act(&mut s.clone(), &r, Command::Defend),
            Err(CommandError::BattleOver)
        );
        let mut legal = LegalCommands::new();
        legal_commands(&s, &r, side, &mut legal);
        assert!(legal.is_empty());
    }
}

// covers: BAT-END-001, BAT-RET-001
#[test]
fn a_retaliation_can_end_the_battle() {
    let r = ruleset();
    // A single slow creature attacks 10 fixed: it deals 1 damage and dies to the retaliation.
    let mut s = battle(&r, &[(0, "slow", 1)], &[(0, "fixed", 10)], 1);
    place(&mut s, &r, 1, 2, 5);
    advance_to(&mut s, &r, 0);
    let events = act(
        &mut s,
        &r,
        Command::Attack {
            target: StackId(1),
            from: at(&r, 1, 5),
        },
    )
    .unwrap();
    assert!(events.contains(&Event::public(EventKind::BattleEnded {
        winner: BattleSide::Defender
    })));
    assert_eq!(s.winner(), Some(BattleSide::Defender));
    assert!(s.outcome().unwrap().survivors[0].stacks.is_empty());
    assert_eq!(s.outcome().map(|o| o.survivors[1].stacks.len()), Some(1));
}

#[test]
fn the_outcome_is_only_available_after_the_battle() {
    let r = ruleset();
    let s = battle(&r, &[(0, "slow", 1)], &[(0, "fixed", 10)], 1);
    assert_eq!(s.outcome(), None);
    let _ = Vec::<u8>::new();
}
