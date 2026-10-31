//! What changed between frames (E24 M3 T2b2, *provisional*; `docs/epics/E24-design.md` §4.3):
//! each composition's changed rectangles ([`crate::compose::Composer::damage`]) kept by its
//! number, so that a frame widened into a buffer that holds an older one widens only what changed
//! since, and a window uploads only what changed since the frame its texture holds.

use alloc::collections::VecDeque;
use alloc::vec::Vec;

/// What a frame widened holds (E24 M3 T2b2; in the library since E24 M3 T7's review, F2-8, so
/// that its rectangles are tested): the composition it was widened from (its number in the
/// history) and the cursor's rectangle drawn over it, if any.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Held {
    /// The composition's number.
    pub composed: u64,
    /// The cursor's rectangle drawn over it.
    pub cursor: Option<[i32; 4]>,
}

/// The last compositions' changed rectangles, by number (counted from 1).
#[derive(Debug, Clone, Default)]
pub struct History {
    /// The number of the first kept.
    first: u64,
    damages: VecDeque<Vec<[i32; 4]>>,
    /// How many are kept.
    keep: usize,
}

impl History {
    /// A history keeping the last `keep` compositions (at least one).
    #[must_use]
    pub fn new(keep: usize) -> Self {
        Self {
            first: 1,
            damages: VecDeque::new(),
            keep: keep.max(1),
        }
    }

    /// Records a composition's changed rectangles; its number.
    pub fn push(&mut self, rects: &[[i32; 4]]) -> u64 {
        if self.damages.len() == self.keep {
            self.damages.pop_front();
            self.first += 1;
        }
        self.damages.push_back(rects.to_vec());
        self.now()
    }

    /// The last composition's number (0 before any).
    #[must_use]
    pub fn now(&self) -> u64 {
        self.first + self.damages.len() as u64 - 1
    }

    /// Pushes to `out` every rectangle that changed between compositions `a` and `b` (in either
    /// order: those after the earlier up to the later); `false`, pushing nothing, when they are no
    /// longer all kept (or `b` is to come).
    pub fn between(&self, a: u64, b: u64, out: &mut Vec<[i32; 4]>) -> bool {
        let (from, to) = (a.min(b), a.max(b));
        if from + 1 < self.first || to > self.now() {
            return false;
        }
        let skip = usize::try_from(from + 1 - self.first).unwrap_or(usize::MAX);
        let take = usize::try_from(to - from).unwrap_or(0);
        for rects in self.damages.iter().skip(skip).take(take) {
            out.extend_from_slice(rects);
        }
        true
    }
}

impl History {
    /// Pushes to `out` what a frame holding `held` must widen again to hold composition `now`:
    /// the rectangles composed since and its cursor's (the new cursor drawn over it after);
    /// `false`, pushing nothing, when the history no longer has them (the whole frame then).
    pub fn since(&self, held: Held, now: u64, out: &mut Vec<[i32; 4]>) -> bool {
        let known = self.between(held.composed, now, out);
        if known {
            out.extend(held.cursor);
        }
        known
    }

    /// Pushes to `out` what differs between two frames widened (a texture's upload): the
    /// rectangles composed between theirs and both cursors'; `false` when the history no longer
    /// has them.
    pub fn changed(&self, a: Held, b: Held, out: &mut Vec<[i32; 4]>) -> bool {
        let known = self.since(a, b.composed, out);
        if known {
            out.extend(b.cursor);
        }
        known
    }
}

/// `rects` made fit to upload to a `frame`-sized texture: each cut to the frame, the empty ones
/// and those another holds dropped ([`drop_held`]); more than `most` become the smallest rectangle
/// holding them all.
pub fn tidy(rects: &mut Vec<[i32; 4]>, frame: [u32; 2], most: usize) {
    let side = |v: u32| i32::try_from(v).unwrap_or(i32::MAX);
    let [w, h] = frame.map(side);
    let mut kept: Vec<[i32; 4]> = Vec::with_capacity(rects.len());
    for &[l, t, r, b] in rects.iter() {
        let cut = [l.max(0), t.max(0), r.min(w), b.min(h)];
        if cut[0] < cut[2] && cut[1] < cut[3] {
            kept.push(cut);
        }
    }
    drop_held(&mut kept);
    if kept.len() > most {
        let all = kept.iter().fold(kept[0], |a, r| {
            [
                a[0].min(r[0]),
                a[1].min(r[1]),
                a[2].max(r[2]),
                a[3].max(r[3]),
            ]
        });
        kept.clear();
        kept.push(all);
    }
    *rects = kept;
}

/// `rects` without those another of them holds, repeats too (the first of equal ones kept), in
/// their order: a history's damages over many compositions repeat the same strips (E24 M3 T7's
/// review, F2-7), each widened or uploaded once.
pub fn drop_held(rects: &mut Vec<[i32; 4]>) {
    let holds =
        |a: [i32; 4], b: [i32; 4]| a[0] <= b[0] && a[1] <= b[1] && a[2] >= b[2] && a[3] >= b[3];
    let mut i = 0;
    while i < rects.len() {
        let r = rects[i];
        let held = rects
            .iter()
            .enumerate()
            .any(|(j, &o)| j != i && holds(o, r) && (o != r || j < i));
        if held {
            rects.remove(i);
        } else {
            i += 1;
        }
    }
}

#[cfg(test)]
mod tests;
