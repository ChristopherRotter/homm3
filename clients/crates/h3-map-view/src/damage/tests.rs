use super::*;
use alloc::vec;

#[test]
fn the_rectangles_between_two_compositions_while_kept() {
    let mut h = History::new(3);
    assert_eq!(h.now(), 0);
    let mut out = Vec::new();
    // Nothing composed: nothing between 0 and 0; 1 is to come.
    assert!(h.between(0, 0, &mut out));
    assert!(!h.between(0, 1, &mut out));
    assert_eq!(h.push(&[[0, 0, 1, 1]]), 1);
    assert_eq!(h.push(&[[1, 1, 2, 2], [2, 2, 3, 3]]), 2);
    assert_eq!(h.push(&[]), 3);
    assert!(h.between(0, 2, &mut out));
    assert_eq!(out, [[0, 0, 1, 1], [1, 1, 2, 2], [2, 2, 3, 3]]);
    // Either order; the same composition: nothing.
    out.clear();
    assert!(h.between(3, 1, &mut out));
    assert_eq!(out, [[1, 1, 2, 2], [2, 2, 3, 3]]);
    out.clear();
    assert!(h.between(2, 2, &mut out));
    assert!(out.is_empty());
    // The fourth pushes the first out: from 0 no longer, from 1 still.
    assert_eq!(h.push(&[[5, 5, 6, 6]]), 4);
    assert!(!h.between(0, 4, &mut out));
    assert!(out.is_empty());
    assert!(h.between(1, 4, &mut out));
    assert_eq!(out.len(), 3);
    assert_eq!(History::new(0).push(&[]), 1);
}

#[test]
fn rectangles_cut_to_the_frame_and_bounded() {
    let mut rects = vec![
        [-2, -2, 3, 3],
        [3, 3, 3, 9],
        [0, 0, 3, 3],
        [8, 1, 20, 4],
        [20, 20, 30, 30],
    ];
    tidy(&mut rects, [10, 6], 4);
    assert_eq!(rects, [[0, 0, 3, 3], [8, 1, 10, 4]]);
    tidy(&mut rects, [10, 6], 1);
    assert_eq!(rects, [[0, 0, 10, 4]]);
    let mut none = vec![[4, 4, 4, 5]];
    tidy(&mut none, [10, 6], 1);
    assert!(none.is_empty());
    // A rectangle another holds goes, cut first (E24 M3 T7's review, F2-7).
    let mut held = vec![[1, 1, 2, 2], [0, 0, 4, 4], [-1, 2, 3, 3]];
    tidy(&mut held, [10, 6], 4);
    assert_eq!(held, [[0, 0, 4, 4]]);
}

#[test]
fn the_rectangles_another_holds_dropped() {
    let mut rects = vec![
        [2, 2, 4, 4],
        [0, 0, 5, 5],
        [0, 0, 5, 5],
        [4, 4, 8, 8],
        [5, 5, 6, 6],
        [9, 9, 10, 10],
    ];
    drop_held(&mut rects);
    assert_eq!(rects, [[0, 0, 5, 5], [4, 4, 8, 8], [9, 9, 10, 10]]);
    let mut none: Vec<[i32; 4]> = Vec::new();
    drop_held(&mut none);
    assert!(none.is_empty());
}
