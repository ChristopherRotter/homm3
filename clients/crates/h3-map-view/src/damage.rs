//! What changed between frames (E24 M3 T2b2, *provisional*; `docs/epics/E24-design.md` §4.3):
//! each composition's changed rectangles ([`crate::compose::Composer::damage`]) kept by its
//! number, so that a frame widened into a buffer that holds an older one widens only what changed
//! since, and a window uploads only what changed since the frame its texture holds.

use alloc::collections::VecDeque;
use alloc::vec::Vec;

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

/// `rects` made fit to upload to a `frame`-sized texture: each cut to the frame, the empty and
/// repeated ones dropped; more than `most` become the smallest rectangle holding them all.
pub fn tidy(rects: &mut Vec<[i32; 4]>, frame: [u32; 2], most: usize) {
    let side = |v: u32| i32::try_from(v).unwrap_or(i32::MAX);
    let [w, h] = frame.map(side);
    let mut kept: Vec<[i32; 4]> = Vec::with_capacity(rects.len());
    for &[l, t, r, b] in rects.iter() {
        let cut = [l.max(0), t.max(0), r.min(w), b.min(h)];
        if cut[0] < cut[2] && cut[1] < cut[3] && !kept.contains(&cut) {
            kept.push(cut);
        }
    }
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

#[cfg(test)]
mod tests;
