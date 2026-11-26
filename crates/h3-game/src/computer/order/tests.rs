//! Tests of the library's `std::sort` (E31 T2d): the median's six cases, the insertion sort's order of
//! equal keys, and the quicksort's order of equal keys on 35 entries, highest and lowest first
//! (the expected orders from an independent model of Visual C++ 5/6's `std::sort`, not
//! committed); a property test that the result is ordered and a permutation.

use alloc::vec::Vec;

use proptest::prelude::*;

use super::{library_sort, median};

/// Keys with ties, more than the insertion sort's 16.
const KEYS: [i32; 35] = [
    5, 3, 5, 1, 9, 9, 0, 2, 5, 7, 3, 3, 8, 1, 0, 0, 5, 9, 2, 4, 6, 6, 1, 8, 3, 5, 7, 0, 2, 9, 4, 4,
    1, 6, 5,
];

/// The indexes of `keys` in the order the sort leaves them, with `before`.
fn order(keys: &[i32], before: impl Fn(i32, i32) -> bool) -> Vec<usize> {
    let mut entries: Vec<(i32, usize)> = keys.iter().copied().zip(0..).collect();
    library_sort(&mut entries, &|a: &(i32, usize), b: &(i32, usize)| {
        before(a.0, b.0)
    });
    entries.into_iter().map(|(_, index)| index).collect()
}

#[test]
fn the_median_of_three_takes_each_of_its_six_paths() {
    let less = |a: &i32, b: &i32| a < b;
    for (triple, median_of) in [
        ((1, 2, 3), 2),
        ((1, 3, 2), 2),
        ((2, 3, 1), 2),
        ((2, 1, 3), 2),
        ((3, 1, 2), 2),
        ((3, 2, 1), 2),
    ] {
        assert_eq!(median(triple, &less), median_of, "{triple:?}");
    }
}

#[test]
fn up_to_sixteen_entries_are_insertion_sorted_and_equal_keys_keep_their_order() {
    let keys = [2, 1, 2, 3, 1, 3, 0, 2];
    assert_eq!(order(&keys, |a, b| a > b), [3, 5, 0, 2, 7, 1, 4, 6]);
    assert_eq!(order(&keys, |a, b| a < b), [6, 1, 4, 0, 2, 7, 3, 5]);
    assert_eq!(order(&[], |a, b| a > b), Vec::<usize>::new());
}

#[test]
fn more_entries_are_partitioned_first_and_equal_keys_end_where_the_library_leaves_them() {
    // Highest first: the quicksort recurses into its left part (the smaller one) here.
    assert_eq!(
        order(&KEYS, |a, b| a > b),
        [
            29, 4, 5, 17, 23, 12, 26, 9, 33, 21, 20, 34, 25, 16, 0, 2, 8, 31, 30, 19, 1, 24, 10,
            11, 7, 28, 18, 22, 3, 13, 32, 6, 27, 15, 14
        ]
    );
    // Lowest first: into its right part.
    assert_eq!(
        order(&KEYS, |a, b| a < b),
        [
            6, 27, 14, 15, 32, 3, 22, 13, 18, 7, 28, 1, 10, 11, 24, 19, 30, 31, 25, 34, 16, 8, 2,
            0, 20, 21, 33, 26, 9, 23, 12, 17, 29, 5, 4
        ]
    );
}

proptest! {
    #[test]
    fn the_sort_orders_its_entries_and_keeps_each_once(
        keys in proptest::collection::vec(-20_i32..20, 0..80),
    ) {
        let sorted = order(&keys, |a, b| a > b);
        let mut seen = sorted.clone();
        let _ = h3_kernel::order::sort_by_unique_key(&mut seen, |&i| i);
        prop_assert_eq!(seen, (0..keys.len()).collect::<Vec<_>>());
        prop_assert!(sorted.windows(2).all(|w| keys[w[0]] >= keys[w[1]]));
    }
}
