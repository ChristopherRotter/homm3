//! The computer's order of action and its library sort (0x41F140, 0x423340; BAT-AI-009).

use std::vec;
use std::vec::Vec;

use h3_battle::{BattleState, Command, StackId};
use h3_contracts::BattleSide;
use h3_kernel::Rng;
use proptest::prelude::{any, prop_assert, proptest};
use proptest::test_runner::Config;

use super::{GOG, IMP, RUNNER, TENT, hero, play, ruleset, setup};
use crate::order::{library_sort, ranks};

/// An element of a sorted list: its key, its record and its place in the input.
type Item = (i32, u32, usize);

fn before(a: &Item, b: &Item) -> bool {
    a.0 > b.0 || (a.0 == b.0 && a.1 < b.1)
}

// covers: BAT-AI-009
#[test]
fn the_library_sort_keeps_equivalent_elements_in_order_up_to_sixteen() {
    // Sixteen equivalent pairs (the same key and record, two sides): the insertion sort keeps
    // the list's order, the attacker's first.
    let mut v: Vec<Item> = (0..16)
        .map(|i| (5, u32::try_from(i / 2).unwrap(), i))
        .collect();
    v.reverse();
    let mut sorted = v.clone();
    library_sort(&mut sorted, &before);
    let places: Vec<usize> = sorted.iter().map(|e| e.2).collect();
    let mut want: Vec<Item> = v.clone();
    want.sort_by_key(|e| e.1);
    assert_eq!(places, want.iter().map(|e| e.2).collect::<Vec<_>>());
    // A key first: the higher key, then the lower record.
    let mut v: Vec<Item> = vec![(1, 0, 0), (3, 2, 1), (3, 1, 2), (-4, 0, 3)];
    library_sort(&mut v, &before);
    assert_eq!(v, vec![(3, 1, 2), (3, 2, 1), (1, 0, 0), (-4, 0, 3)]);
}

// covers: BAT-AI-009
#[test]
fn above_sixteen_the_partitions_order_equivalent_elements() {
    // Twenty elements, two equivalent ones among them (key 7, record 3): the quicksort's
    // partitions swap them, the later first.
    let keys = [
        9, 7, 2, 8, 1, 7, 3, 6, 5, 4, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18,
    ];
    let mut v: Vec<Item> = keys
        .iter()
        .enumerate()
        .map(|(i, &k)| (k, if k == 7 { 3 } else { 0 }, i))
        .collect();
    library_sort(&mut v, &before);
    let sevens: Vec<usize> = v.iter().filter(|e| e.0 == 7).map(|e| e.2).collect();
    assert_eq!(sevens, vec![5, 1]);
    assert!(v.windows(2).all(|w| !before(&w[1], &w[0])));
}

proptest! {
    #![proptest_config(Config::with_cases(64))]

    /// The library sort sorts: every element in, none before the one in front of it.
    // covers: BAT-AI-009
    #[test]
    fn the_library_sort_sorts(seed in any::<u64>()) {
        let mut rng = Rng::from_seed(seed);
        let n = usize::try_from(rng.below(40)).unwrap();
        let mut v: Vec<Item> = (0..n)
            .map(|i| {
                let key = i32::try_from(rng.below(6)).unwrap() - 2;
                (key, rng.below(3), i)
            })
            .collect();
        library_sort(&mut v, &before);
        prop_assert!(v.windows(2).all(|w| !before(&w[1], &w[0])));
        let mut places: Vec<usize> = v.iter().map(|e| e.2).collect();
        places.sort_unstable();
        prop_assert!(places == (0..n).collect::<Vec<_>>());
    }
}

/// The rank of stack `id`.
fn rank(state: &BattleState, first: BattleSide, id: u8) -> u8 {
    ranks(state, &ruleset(), first)[usize::from(id)]
}

// covers: BAT-AI-009
#[test]
fn the_order_by_speed_with_equal_speeds_alternating() {
    let r = ruleset();
    // Imps (5) of both sides, a gog (3) and the defender's runners (20); the tent is last. The
    // first side expected takes the first of two equal keys.
    let mut setup = setup(&[(IMP, 1), (GOG, 1)], &[(IMP, 1), (RUNNER, 1)]);
    setup.heroes[1] = Some(hero(0));
    setup.war_machines[1] = vec![super::creature(TENT)];
    let state = BattleState::new(&r, &setup, 1).unwrap();
    let ids: Vec<u8> = (0u8..).take(state.stacks().len()).collect();
    assert_eq!(ids.len(), 5);
    // Attacker: 0 imp, 1 gog; defender: 2 imp, 3 runner, 4 tent.
    let for_attacker: Vec<u8> = ids
        .iter()
        .map(|&i| rank(&state, BattleSide::Attacker, i))
        .collect();
    assert_eq!(for_attacker, vec![4, 2, 3, 5, 1]);
    let for_defender: Vec<u8> = ids
        .iter()
        .map(|&i| rank(&state, BattleSide::Defender, i))
        .collect();
    // The runner first; then the attacker's side is expected: its imp before the defender's.
    assert_eq!(for_defender, vec![4, 2, 3, 5, 1]);
    // Once the runner acted (speed − 1000) and the attacker's imp waited (−5), the defender's imp
    // leads among the rest.
    let mut state = state;
    play(&mut state, &r, Command::Defend);
    assert_eq!(state.active(), Some(StackId(0)));
    play(&mut state, &r, Command::Wait);
    let after: Vec<u8> = ids
        .iter()
        .map(|&i| rank(&state, BattleSide::Defender, i))
        .collect();
    // The defender's imp 5, the gog 4, then the waited imp (−5) beats the acted runner (−980).
    assert_eq!(after, vec![3, 4, 5, 2, 1]);
}
