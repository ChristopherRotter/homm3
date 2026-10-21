//! The balance of the sides' able strengths (0x435EC0's +0x14, +0x18 and +0x1C; E23 T4a): the
//! raise's doubling reads it (BAT-AI-012). The casts themselves: the harvested
//! `harvested_computer_casts.rs`.

use h3_battle::{BattleState, computer};
use h3_contracts::BattleSide;

use super::{IMP, ruleset, setup};
use crate::casts::{Balance, balance};

/// The balance of `side` with `attacker` and `defender` imps.
fn of(attacker: u32, defender: u32, side: BattleSide) -> Balance {
    let r = ruleset();
    let state = BattleState::new(&r, &setup(&[(IMP, attacker)], &[(IMP, defender)]), 1).unwrap();
    balance(&state, &r, &computer::context(&state, &r, side), side)
}

// covers: BAT-AI-012
#[test]
fn the_balance_class_by_the_able_strengths() {
    use BattleSide::{Attacker, Defender};
    // Equal sides: a ratio of 1.0, below every threshold, 7.
    let even = of(10, 10, Attacker);
    assert!(even.own > 0);
    assert_eq!(even.own, even.other);
    assert_eq!(even.class, 7);
    // 5 against 4 imps: 1.25 reaches 1.2, the fifth threshold, 5; from either side.
    assert_eq!(of(50, 40, Attacker).class, 5);
    assert_eq!(of(50, 40, Defender).class, 5);
    let weaker = of(50, 40, Defender);
    assert!(weaker.own < weaker.other);
    // 2 to 1 reaches 1.9: 2; 3 to 1 reaches 2.6: 1; at 6 to 1 the smaller is under a fifth: 1.
    assert_eq!(of(20, 10, Attacker).class, 2);
    assert_eq!(of(30, 10, Defender).class, 1);
    assert_eq!(of(60, 10, Attacker).class, 1);
}
