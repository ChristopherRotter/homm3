//! The rectangles `SMACKW32.DLL` gives for what a frame changed (E27 M2 T5a2, design §10):
//! `SmackToBufferRect` (0x100072D0) as the game's `video_show` (0x5979D0) calls it, ported from
//! a reading of the DLL's code with capstone (no source; *read*, checked on every frame of the
//! game's clips against the DLL under Wine: `smackprobe rects`, `clients/scripts/smacker-check.sh`).
//!
//! The DLL's parts, each a method here: an iterator over the decoder's map of changed spans
//! (0x10012760, [`Rects::span`]; the map is [`Frame::spans`]) gives runs of changed spans, each
//! cut at a row's end; a filler (0x10007410, [`Rects::fill`]) joins the runs while the box over
//! them stays within 1.25 times their areas and aligns it to 8 pixels across; a list of at most
//! four rectangles (the handle's +0x4C8 and +0x4CC), each new one joined to one it shares a whole
//! edge with (0x10007680, [`Rects::join`]), is filled up and its first taken ([`Rects::next`]).
//! The handle's fields are kept as the DLL keeps them, from frame to frame, so the cases the
//! game meets come out as they do there: a frame that changes nothing gives no rectangle and the
//! game shows the last one again; a frame that changes only its palette gives an empty one at
//! (0, 0) and then the iterator's last run again; a run that ends at a row's end looks at the
//! next two spans for the next row's part (a word read for a byte), so a changed span after an
//! unchanged one joins the unchanged one in.
//!
//! Only the game's case is ported: the buffer's type neither 1 (the filler joins every run) nor
//! 2 (no rectangles), and a clip neither interlaced nor doubled (the header's flags 2 and 4:
//! the DLL's rows doubled), which no clip of the game is.

use alloc::vec::Vec;

use crate::Frame;

/// A rectangle in the buffer: x, y, width, height.
pub type Rect = [u32; 4];

/// A span's width in pixels: 16 blocks of 4.
const SPAN: u32 = 64;

/// A row of blocks' height in pixels.
const ROW: u32 = 4;

/// `SmackToBufferRect`'s state for one clip (the handle's and the decoder state's fields).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rects {
    /// Where `SmackToBuffer` put the clip in the buffer (the handle's +0x394, +0x398).
    at: [u32; 2],
    /// The clip's width and height (the decoder state's +0x24, +0x28).
    size: [u32; 2],
    /// The iterator: done, inside a run cut at a row's end, started (+0x20 to +0x22).
    done: bool,
    wrapped: bool,
    started: bool,
    /// The iterator's place: x and y in pixels, the span (+0x2C, +0x30, +0x34).
    x: u32,
    y: u32,
    next: usize,
    /// The iterator's last run: x, y, width, height (the handle's +0x40C).
    run: Rect,
    /// The filler's stage (+0x408): 0 nothing to fill, 1 a frame decoded, 2 filling, 3 done.
    stage: u8,
    /// The list (+0x4CC), each as left, top, right, bottom, and how many it holds (+0x4C8).
    list: [Rect; 4],
    count: usize,
    /// The rectangle taken last (+0x380 to +0x38C).
    last: Rect,
}

impl Rects {
    /// The state of a clip of `size` that `SmackToBuffer` put at `at` in its buffer, as the
    /// handle holds it before its first frame (zeros).
    #[must_use]
    pub const fn new(size: [u32; 2], at: [u32; 2]) -> Self {
        Self {
            at,
            size,
            done: false,
            wrapped: false,
            started: false,
            x: 0,
            y: 0,
            next: 0,
            run: [0; 4],
            stage: 0,
            list: [[0; 4]; 4],
            count: 0,
            last: [0; 4],
        }
    }

    /// What `SmackDoFrame` does when it decodes a frame: the filler started (0x10005D2F) and the
    /// iterator set back to the map's start (the decoder, 0x10011E3B). The list keeps what the
    /// frame before left in it.
    pub fn decoded(&mut self) {
        self.stage = 1;
        self.done = false;
        self.wrapped = false;
        self.started = false;
        self.x = 0;
        self.y = 0;
    }

    /// The rectangle taken last ([`Rects::next`]; zeros before the first).
    #[must_use]
    pub const fn last(&self) -> Rect {
        self.last
    }

    /// `SmackToBufferRect`: whether it took a rectangle of `frame` (the frame decoded last),
    /// then in [`Rects::last`].
    pub fn next(&mut self, frame: &Frame) -> bool {
        self.take(frame.spans(), frame.palette_changed())
    }

    /// [`Rects::next`] on a frame's map and whether its palette changed.
    fn take(&mut self, map: &[u8], palette: bool) -> bool {
        if self.stage != 0 {
            while let Some(edges) = self.fill(map, palette) {
                // Below 4: the loop stops at 4 and a call starts after a rectangle was taken.
                self.list[self.count] = edges;
                self.count += 1;
                self.join(self.count - 1);
                if self.count >= self.list.len() {
                    break;
                }
            }
        }
        if self.count == 0 {
            return false;
        }
        self.count -= 1;
        let [left, top, right, bottom] = self.list[0];
        self.last = [
            left,
            top,
            right.wrapping_sub(left),
            bottom.wrapping_sub(top),
        ];
        self.list.copy_within(1..=self.count, 0);
        true
    }

    /// The rectangles the game's `video_show` reads for `frame`: the last one after its first
    /// call whatever it returned (the one before when there is none), then one per call that
    /// takes one. The game shows their union (`h3_view::shell::start::shown_union`).
    pub fn shown(&mut self, frame: &Frame) -> Vec<Rect> {
        let mut out = Vec::new();
        self.next(frame);
        out.push(self.last);
        while self.next(frame) {
            out.push(self.last);
        }
        out
    }

    /// The iterator (0x10012760): the next run of changed spans of the map, cut at a row's end,
    /// into `run`; false once past the last row.
    fn span(&mut self, map: &[u8]) -> bool {
        let byte = |k: usize| map.get(k).copied().unwrap_or(0);
        let [width, height] = self.size;
        if self.done {
            return false;
        }
        let mut x;
        if self.wrapped {
            // The rest of a run cut at a row's end, at the next row's start.
            if height <= self.y {
                self.done = true;
                return false;
            }
            self.run[1] = self.y;
            self.run[0] = 0;
            x = self.x;
            if width <= x {
                // A span wider than what is left of the row (a clip narrower than 64).
                self.run[2] = width;
                self.x = x - width;
                self.y += ROW;
                self.run[3] = ROW;
                return true;
            }
            self.wrapped = false;
            self.run[2] = x;
            if byte(self.next) == 0 && byte(self.next + 1) == 0 {
                self.run[3] = ROW;
                return true;
            }
        } else {
            if !self.started {
                self.next = 0;
                self.started = true;
            }
            x = self.x;
            let mut y = self.y;
            if height <= y {
                self.done = true;
                return false;
            }
            while byte(self.next) == 0 {
                self.next += 1;
                x += SPAN;
                while width <= x {
                    y += ROW;
                    if height <= y {
                        self.done = true;
                        return false;
                    }
                    x -= width;
                }
            }
            self.y = y;
            self.run = [x, y, 0, ROW];
        }
        loop {
            let end = x + SPAN;
            if width <= end {
                let over = end - width;
                self.x = over;
                self.wrapped |= over != 0;
                self.run[2] += SPAN - over;
                self.y += ROW;
                self.next += 1;
                return true;
            }
            self.next += 1;
            x = end;
            self.run[2] += SPAN;
            if byte(self.next) == 0 {
                self.x = x;
                return true;
            }
        }
    }

    /// The filler (0x10007410): the next rectangle of the map as left, top, right and bottom
    /// in the buffer, or none (`palette`: whether the frame's palette changed).
    fn fill(&mut self, map: &[u8], palette: bool) -> Option<Rect> {
        match self.stage {
            3 => {
                self.stage = 0;
                return None;
            }
            1 => {
                if !self.span(map) {
                    self.stage = 0;
                    // A frame whose palette changed and nothing else: an empty rectangle.
                    return palette.then_some([0; 4]);
                }
                self.stage = 2;
            }
            _ => {}
        }
        // From the run held (the last taken, at stage 0: an earlier frame's).
        let [x, top, w, h] = self.run;
        let (mut left, mut right, mut bottom) = (x, x.wrapping_add(w), top.wrapping_add(h));
        let mut budget = area(w, h);
        loop {
            if !self.span(map) {
                self.stage = 3;
                break;
            }
            let [nx, ny, nw, nh] = self.run;
            budget = budget.wrapping_add(area(nw, nh));
            let l = nx.min(left);
            let r = nx.wrapping_add(nw).max(right);
            let b = ny.wrapping_add(nh);
            if r.wrapping_sub(l).wrapping_mul(b.wrapping_sub(top)) > budget {
                break;
            }
            (left, right, bottom) = (l, r, b);
        }
        // Aligned to 8 pixels of the buffer: the left edge down while that stays right of the
        // clip's, the width up while that stays inside the clip.
        let [at_x, at_y] = self.at;
        let mut x = left;
        let mut width = right.wrapping_sub(left);
        let mut from = left.wrapping_add(at_x);
        let down = from & !7;
        if from & 7 != 0 && down > at_x {
            width = width.wrapping_add(from - down);
            x = x.wrapping_sub(from - down);
            from = down;
        }
        let up = width.wrapping_add(7) & !7;
        if width & 7 != 0 && self.size[0].wrapping_sub(x) > up {
            width = up;
        }
        Some([
            from,
            at_y.wrapping_add(top),
            from.wrapping_add(width),
            at_y.wrapping_add(bottom),
        ])
    }

    /// The join (0x10007680): the list's rectangle `new` joined into one that shares a whole
    /// edge with it (the first such, in the list's order), and again with the joined one, until
    /// none does. The joined one's index stands for the new one even where the removal moved it
    /// down (the DLL's: a later one's index then names the rectangle after it).
    fn join(&mut self, mut new: usize) {
        // `new` and `k` below 4: `new` the last appended or a `k`, `k` below the count (at
        // most 4).
        'again: loop {
            for k in 0..self.count {
                if k == new {
                    continue;
                }
                let n = self.list[new];
                let a = &mut self.list[k];
                let joined = if n[0] == a[0] && n[2] == a[2] {
                    if n[1] == a[3] {
                        a[3] = n[3];
                        true
                    } else if a[1] == n[3] {
                        a[1] = n[1];
                        true
                    } else {
                        false
                    }
                } else if n[1] == a[1] && n[3] == a[3] {
                    if a[2] == n[0] {
                        a[2] = n[2];
                        true
                    } else if n[2] == a[0] {
                        a[0] = n[0];
                        true
                    } else {
                        false
                    }
                } else {
                    false
                };
                if joined {
                    self.count -= 1;
                    if new > self.count {
                        // `new` past the list's end (the third join of one rectangle, the
                        // second having moved the first down): the DLL's copy would run
                        // backwards over its memory and fault. No clip of the game comes here.
                        return;
                    }
                    self.list.copy_within(new + 1..=self.count, new);
                    new = k;
                    continue 'again;
                }
            }
            return;
        }
    }
}

/// A run's area as the filler weighs it: 1.25 times, rounded down (the DLL's `* 5 >> 2`).
const fn area(w: u32, h: u32) -> u32 {
    h.wrapping_mul(w).wrapping_mul(5) >> 2
}

#[cfg(test)]
mod tests;
