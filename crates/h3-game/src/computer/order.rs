//! The library's `std::sort` (Visual C++ 5/6) as the computer's code runs it over its 12-byte
//! army entries (E31 T2d; survey §1.5; logic crates sort only through `h3_kernel::order`, ADR-0003
//! §9, but this one must order as the original, its ties included): `army_entries_sort_part`
//! 0x434CE0 and its ascending twin `army_entries_sort_part_low` 0x434E80 (the quicksort loop, the
//! median 0x435240 and the partition 0x435290), `army_entries_insertion_sort` 0x435160 and `_low`
//! 0x435310, the unguarded insertion 0x435020. Equal keys keep the order the algorithm leaves them in, which
//! decides the resource weights' three dearest stacks (`ai_resource_weights` 0x428740).
//!
//! The same algorithm as `h3_battle_ai`'s `library_sort` (the battle's 0x423340, `installation.md`
//! §7.37), copied: `h3-game` does not depend on the battle AI.

/// The most elements sorted by insertion alone (the library's `_stl_threshold`, 16).
const SORT_MAX: usize = 16;

/// `v` sorted as the library's `std::sort` sorts it with `before`: more than [`SORT_MAX`] elements
/// are first partitioned around the median of the first, middle and last element down to parts
/// of at most [`SORT_MAX`] ([`sort_part`]), then the first [`SORT_MAX`] are insertion-sorted and
/// each further one inserted unguarded; at most [`SORT_MAX`] are insertion-sorted alone, which
/// keeps equal elements in their order.
pub(super) fn library_sort<T: Copy>(v: &mut [T], before: &impl Fn(&T, &T) -> bool) {
    if v.len() <= SORT_MAX {
        insertion_sort(v, before);
        return;
    }
    sort_part(v, (0, v.len()), before);
    insertion_sort(&mut v[..SORT_MAX], before);
    for i in SORT_MAX..v.len() {
        unguarded_insert(v, i, before);
    }
}

/// The quicksort of `v[f..l]` (0x434CE0): while more than [`SORT_MAX`] elements are left,
/// partition around the median ([`median`], [`partition`]) and recurse into the part not larger
/// than the other (the right one on a tie), looping on the other.
fn sort_part<T: Copy>(
    v: &mut [T],
    (mut f, mut l): (usize, usize),
    before: &impl Fn(&T, &T) -> bool,
) {
    while l - f > SORT_MAX {
        let pivot = median((v[f], v[f + (l - f) / 2], v[l - 1]), before);
        let m = partition(v, (f, l), pivot, before);
        if l - m <= m - f {
            sort_part(v, (m, l), before);
            l = m;
        } else {
            sort_part(v, (f, m), before);
            f = m;
        }
    }
}

/// The median of three (0x435240, inlined in 0x434CE0).
fn median<T: Copy>((x, y, z): (T, T, T), before: &impl Fn(&T, &T) -> bool) -> T {
    if before(&x, &y) {
        if before(&y, &z) {
            y
        } else if before(&x, &z) {
            z
        } else {
            x
        }
    } else if before(&x, &z) {
        x
    } else if before(&y, &z) {
        z
    } else {
        y
    }
}

/// The unguarded partition of `v[f..l]` around `pivot` (0x435290): the first place of the part
/// not before it. The median of three keeps both scans within the part.
fn partition<T: Copy>(
    v: &mut [T],
    (mut f, mut l): (usize, usize),
    pivot: T,
    before: &impl Fn(&T, &T) -> bool,
) -> usize {
    loop {
        while before(&v[f], &pivot) {
            f += 1;
        }
        l -= 1;
        while before(&pivot, &v[l]) {
            l -= 1;
        }
        if l <= f {
            return f;
        }
        v.swap(f, l);
        f += 1;
    }
}

/// The insertion sort (0x435160): each element goes to the front when before the first, else is
/// inserted unguarded.
fn insertion_sort<T: Copy>(v: &mut [T], before: &impl Fn(&T, &T) -> bool) {
    for m in 1..v.len() {
        if before(&v[m], &v[0]) {
            v[..=m].rotate_right(1);
        } else {
            unguarded_insert(v, m, before);
        }
    }
}

/// The unguarded insertion of `v[at]` (0x435020): moved down past every element it is before.
fn unguarded_insert<T: Copy>(v: &mut [T], mut at: usize, before: &impl Fn(&T, &T) -> bool) {
    let value = v[at];
    while before(&value, &v[at - 1]) {
        v[at] = v[at - 1];
        at -= 1;
    }
    v[at] = value;
}

#[cfg(test)]
mod tests;
