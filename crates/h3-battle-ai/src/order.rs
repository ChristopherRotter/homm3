//! The computer's order of action (0x41F140 with its sort 0x423340; `docs/formats/installation.md`
//! §7.34, §7.37; BAT-AI-009): the order in which the computer expects the stacks to act this
//! round, as ranks. The walker's threat records on its turn take an enemy only when it acts after
//! the record's stack ([`crate::gain`]).
//!
//! Each living stack of both sides but the turrets gets a key: the First Aid Tent and the Ammo
//! Cart −100 000; a stack blind or turned to stone for more than one round −10 000; one that has
//! acted this round or is blind, turned to stone or paralysed its speed − 1000; one that has
//! waited, or any in the wait phase, minus its speed; any other its speed. The list (the
//! attacker's stacks in record order, then the defender's) is sorted by the higher key, then the
//! lower record index, by Visual C++ 5/6's `std::sort` ([`library_sort`]: its order of equal
//! pairs of the two sides above 16 stacks is that algorithm's own, ADR-0027 point 9). Equal keys
//! then alternate between the sides, the active stack's side first: where the stack at a place is
//! not of the side expected, the next stack with the same key of that side is swapped in. A
//! stack's rank is the number of stacks less its place.

use h3_battle::{
    BattleSide, BattleState, STACK_CAPACITY, Stack, StackFlags, StackId, StackTraits, Status,
    computer,
};
use h3_content_battle::BattleRuleset;

/// The library sort's cut-off (Visual C++ 5/6's `_SORT_MAX`; 0x423340's 0x40 bytes): a part of
/// at most this many elements is left to the insertion sort.
const SORT_MAX: usize = 16;

/// The key of a Tent or an Ammo Cart (0x41F23A).
const MACHINE_KEY: i32 = -100_000;

/// The key of a stack blind or turned to stone for more than one round (0x41F233).
const KEPT_OUT_KEY: i32 = -10_000;

/// What a stack that has acted or cannot act loses from its speed (0x41F22C).
const ACTED: i32 = 1000;

/// A stack in the order: its id, key, record index (+0xF8) and side.
#[derive(Debug, Clone, Copy)]
struct Entry {
    id: StackId,
    key: i32,
    record: u32,
    side: BattleSide,
}

/// 0x4235C0's predicate: the higher key, then the lower record index (the side is not compared).
fn before(a: &Entry, b: &Entry) -> bool {
    a.key > b.key || (a.key == b.key && a.record < b.record)
}

/// The key of stack `s` (`id`; 0x41F140).
fn key(state: &BattleState, ruleset: &BattleRuleset, (id, s): (StackId, &Stack)) -> i32 {
    let speed = i32::from(s.speed_now());
    if s.has(StackTraits::FIRST_AID) || s.has(StackTraits::AMMO_SUPPLY) {
        MACHINE_KEY
    } else if computer::kept_out(state, ruleset, id) {
        KEPT_OUT_KEY
    } else if s.flags.contains(StackFlags::ACTED) || s.spells.status.contains(Status::NO_TURNS) {
        speed - ACTED
    } else if s.flags.contains(StackFlags::WAITED) || computer::wait_phase(state) {
        -speed
    } else {
        speed
    }
}

/// The ranks of the computer's order of action (0x41F140 without a list; BAT-AI-009), by stack
/// id: the number of stacks less the stack's place, `first`'s stack first among equal keys (the
/// side of the stack activated last, +0x132B8: the active stack's); 0 for the dead and the
/// turrets, which are left out.
// rule: BAT-AI-009
pub(crate) fn ranks(
    state: &BattleState,
    ruleset: &BattleRuleset,
    first: BattleSide,
) -> [u8; STACK_CAPACITY] {
    let mut list = [Entry {
        id: StackId(0),
        key: 0,
        record: 0,
        side: first,
    }; STACK_CAPACITY];
    let mut n = 0;
    for side in BattleSide::BOTH {
        let records = (0u8..).zip(state.stacks()).filter(|(_, s)| s.side == side);
        for ((i, s), record) in records.zip(0u32..) {
            if s.is_alive() && !s.is_turret() {
                let id = StackId(i);
                let key = key(state, ruleset, (id, s));
                list[n] = Entry {
                    id,
                    key,
                    record,
                    side,
                };
                n += 1;
            }
        }
    }
    let list = &mut list[..n];
    library_sort(list, &before);
    // Equal keys alternate between the sides (0x41F29D to 0x41F323).
    let mut expected = first;
    for i in 0..n {
        if list[i].side != expected {
            let k = list[i].key;
            let other = (i + 1..n)
                .take_while(|&j| list[j].key == k)
                .find(|&j| list[j].side == expected);
            if let Some(j) = other {
                list.swap(i, j);
            }
        }
        expected = list[i].side.opponent();
    }
    let mut ranks = [0; STACK_CAPACITY];
    for (i, e) in list.iter().enumerate() {
        ranks[e.id.index()] = u8::try_from(n - i).unwrap_or(u8::MAX);
    }
    ranks
}

/// Visual C++ 5/6's `std::sort` as the original's 0x423340 runs it (`installation.md` §7.37): more
/// than [`SORT_MAX`] elements are first partitioned by a quicksort around the median of the first,
/// middle and last element ([`sort_part`]) down to parts of at most [`SORT_MAX`], then the first
/// [`SORT_MAX`] are insertion-sorted and each further one inserted unguarded; at most
/// [`SORT_MAX`] are insertion-sorted alone, which keeps equivalent elements in their order.
pub(crate) fn library_sort<T: Copy>(v: &mut [T], before: &impl Fn(&T, &T) -> bool) {
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

/// The quicksort of `v[f..l]` (0x423630, `_Sort`): while more than [`SORT_MAX`] elements are
/// left, partition around the median ([`median`], [`partition`]) and recurse into the part not
/// larger than the other, looping on the other.
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

/// The median of three (`_Median`, inlined in 0x423340 and 0x423630).
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

/// The unguarded partition of `v[f..l]` around `pivot` (0x423820, `_Unguarded_partition`): the
/// first place of the part not before it. The median of three keeps both scans within the part.
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

/// The insertion sort (`_Insertion_sort`): each element goes to the front when before the
/// first, else is inserted unguarded.
fn insertion_sort<T: Copy>(v: &mut [T], before: &impl Fn(&T, &T) -> bool) {
    for m in 1..v.len() {
        if before(&v[m], &v[0]) {
            v[..=m].rotate_right(1);
        } else {
            unguarded_insert(v, m, before);
        }
    }
}

/// The unguarded insertion of `v[at]` (0x4237C0, `_Unguarded_insert`): moved down past every
/// element it is before.
fn unguarded_insert<T: Copy>(v: &mut [T], mut at: usize, before: &impl Fn(&T, &T) -> bool) {
    let value = v[at];
    while before(&value, &v[at - 1]) {
        v[at] = v[at - 1];
        at -= 1;
    }
    v[at] = value;
}
