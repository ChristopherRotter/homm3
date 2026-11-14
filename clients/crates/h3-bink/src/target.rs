//! The frames as the game gets them (E27 M2 T2b, CLT-Q122): `BINKW32.DLL`'s `BinkCopyToBuffer`
//! (0x10005550) copies a decoded frame into the caller's buffer, and without its flag
//! `BINKCOPYALL` (0x4000000), which the game never passes (`video_tick` 0x44DAA0 and
//! `video_redraw` 0x44D9E0 give only the surface type, `video_surface_type` 0x694CA0), it copies
//! only what the decoding marked changed: the buffer keeps the rest from the copies before.
//!
//! **The copy** (read in the DLL; the YUY2 copy `YUV_blit_YUY2_mask` 0x10009D90 the probe
//! takes and the 5-6-5 one `YUV_blit_16bpp_mask` 0x100096B0 the game takes alike): the mask has
//! a byte per 16 × 16 luma pixels (`Decoder::changed`); when every byte is set (a `strlen` of the
//! mask against its size) the whole frame, else each whole cell of the picture whose byte is set,
//! two cells at a time, then, whatever the mask, the strip right of the last whole column of cells
//! and the strip below the last whole row (the full copy `YUV_blit_YUY2` 0x10008330, or
//! `YUV_blit_16bpp` 0x10007A90, on what is left of `width` & !15 and `height` & !15). The chroma
//! goes with its luma: a cell's chroma is its 8 × 8 of each chroma plane.
//!
//! **The fault** is the mask's: `ExpandPlane` files the chroma's blocks under other cells
//! (`Decoder::mark`), so where a chroma block changed and none of the cell's luma blocks did,
//! the change reaches the buffer only when a later frame marks that cell (in the shell's clips
//! a frame or a few later, up to a few levels of chroma: `NWCLOGO` from frame 7, `LOSEGAME` from
//! frame 6, `H3INTRO` and `H3X1intr` alike; none of the battle's clips, whose every such chroma
//! change comes with its luma). [`crate::Reading::Fixed`] shows the decoded frame whole (the
//! client); a target is the comparisons' ([`crate::Reading::Original`], `bink-check.sh`).

use crate::{Decoder, Frame, dsp};

/// A buffer `BinkCopyToBuffer` copies a clip's frames into, as the game calls it (the module
/// doc), in the decoder's planes: what lies outside the picture (the block grid's rows below it)
/// copied as on the strips, so that a target's frame differs from the decoder's only in the
/// cells it kept.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Target {
    frame: Frame,
}

impl Target {
    /// A buffer for `decoder`'s frames, every pixel 0 (Y = U = V = 0, as `binkprobe`'s buffer
    /// starts; the game's holds what was drawn there before the clip).
    #[must_use]
    pub fn new(decoder: &Decoder) -> Self {
        let mut frame = decoder.frame().clone();
        for plane in frame.planes_mut() {
            plane.pixels_mut().fill(0);
        }
        Self { frame }
    }

    /// Copies `decoder`'s last frame in as the DLL does (the module doc) and returns the buffer.
    pub fn copy(&mut self, decoder: &Decoder) -> &Frame {
        let planes = self.frame.planes_mut().iter_mut();
        for (p, (target, source)) in planes.zip(decoder.frame().planes()).enumerate() {
            // A chroma pixel's luma position: twice its own.
            let shift = usize::from(p > 0);
            let stride = target.stride();
            for (y, row) in source.pixels().chunks(stride).enumerate() {
                for (x, &value) in row.iter().enumerate() {
                    if copies(decoder, x << shift, y << shift) {
                        dsp::set(target.pixels_mut(), y * stride + x, value);
                    }
                }
            }
        }
        &self.frame
    }

    /// The buffer as last copied into.
    #[must_use]
    pub fn frame(&self) -> &Frame {
        &self.frame
    }
}

/// Whether the DLL's copy of `decoder`'s last frame writes the luma position (`x`, `y`) (the
/// module doc): its cell marked changed, or on the strips right of the last whole column of cells
/// and below the last whole row. A pixel of the game's 5-6-5 buffer takes its colour from its
/// luma and its 2 × 2 block's chroma, both inside its cell, so the copy writes it exactly when
/// this holds (E27 M2 T5a: the shell's clips copied onto what the game's buffer held before).
#[must_use]
pub fn copies(decoder: &Decoder, x: usize, y: usize) -> bool {
    let (width, height) = decoder.size();
    let (changed, wide) = decoder.changed();
    x >= width & !15
        || y >= height & !15
        || changed
            .get((y >> 4) * wide + (x >> 4))
            .is_some_and(|&c| c != 0)
}

/// A rectangle of the picture: x, y, width and height in pixels.
pub type Rect = [i32; 4];

/// The most rectangles `BinkGetRects` gives.
pub const MAX_RECTS: usize = 8;

/// The rectangles of `decoder`'s last frame as `BINKW32.DLL`'s `BinkGetRects` (0x10006380, read
/// at E27 M2 T5a) gives them for the game's surface type (its flags neither 0x40000000, no
/// rectangles, nor 0x80000000, one bounding rectangle): the bounding rectangle of the cells the
/// frame marked changed ([`copies`]' mask) over the luma's stride and rows (the picture rounded
/// up to 16; 0x10006650), then, while there are fewer than [`MAX_RECTS`], the rectangle whose cut
/// saves the most area cut in two (0x100067B0: across at half its width or down at half its
/// height, each half rounded up to 16 and shrunk to its own marked cells; the cut that saves
/// more, across on a tie; none that saves nothing), the new half last. Then they are given in
/// the order of their top edge, then their left (the key y · 65 536 + x, the first of equal keys
/// first), each cut at the picture's right and bottom edge. A frame that marked nothing has none.
/// The game shows their union (`video_show` 0x5979D0), which its own arithmetic can leave short
/// (E27 M2 T5a: `h3_view::shell::start`).
#[must_use]
pub fn rects(decoder: &Decoder) -> alloc::vec::Vec<Rect> {
    let (mask, wide) = decoder.changed();
    let high = mask.len() / wide.max(1);
    let marked = |row: i32, col: i32| {
        let at = usize::try_from(row).ok()? * wide + usize::try_from(col).ok()?;
        mask.get(at).map(|&m| m != 0)
    };
    let side = |n: usize| i32::try_from(n * 16).unwrap_or(i32::MAX);
    let Some(first) = bounds(&marked, [0, 0, side(wide), side(high)]) else {
        return alloc::vec::Vec::new();
    };
    let mut found = alloc::vec![first];
    let mut cuts = alloc::vec![cut(&marked, first)];
    while found.len() < MAX_RECTS {
        // The cut that saves the most, the first of equals; none that saves nothing.
        let mut best: Option<(usize, i32)> = None;
        for (k, &(saved, _, _)) in cuts.iter().enumerate() {
            if saved > best.map_or(0, |(_, b)| b) {
                best = Some((k, saved));
            }
        }
        let Some((k, _)) = best else { break };
        let (_, a, b) = cuts[k];
        found[k] = a;
        found.push(b);
        if found.len() == MAX_RECTS {
            break;
        }
        cuts[k] = cut(&marked, a);
        cuts.push(cut(&marked, b));
    }
    let (width, height) = decoder.size();
    let size = |n: usize| i32::try_from(n).unwrap_or(i32::MAX);
    let (width, height) = (size(width), size(height));
    let mut sorted = alloc::vec::Vec::with_capacity(found.len());
    let mut left: alloc::vec::Vec<Option<Rect>> = found.into_iter().map(Some).collect();
    while let Some(k) = (0..left.len())
        .filter(|&k| left[k].is_some())
        .min_by_key(|&k| {
            left[k].map(|[x, y, _, _]| (y as u32).wrapping_shl(16).wrapping_add(x as u32))
        })
    {
        if let Some([x, y, mut w, mut h]) = left[k].take() {
            if x + w > width {
                w = width - x;
            }
            if y + h > height {
                h = height - y;
            }
            sorted.push([x, y, w, h]);
        }
    }
    sorted
}

/// The rectangle `rect` shrunk to its marked cells (0x10006650): its rows from the first with a
/// marked cell to the last, then its columns from the first to the last over those rows, in
/// whole cells of 16 (a partial cell at its right or bottom edge is not looked at); none when
/// none is marked.
fn bounds(marked: &impl Fn(i32, i32) -> Option<bool>, rect: Rect) -> Option<Rect> {
    let [x, y, w, h] = rect;
    let (col0, row0, cols, rows) = (x / 16, y / 16, w / 16, h / 16);
    let any_in_row = |r: i32, cs: core::ops::Range<i32>| {
        cs.into_iter()
            .any(|c| marked(row0 + r, col0 + c) == Some(true))
    };
    let top = (0..rows).find(|&r| any_in_row(r, 0..cols))?;
    let mut out = [x, y + 16 * top, w, h - 16 * top];
    let mut rows_left = rows - top;
    if out[3] > 1 {
        let empty = (0..rows_left)
            .take_while(|&k| !any_in_row(top + rows_left - 1 - k, 0..cols))
            .count();
        let empty = i32::try_from(empty).unwrap_or(0);
        out[3] -= 16 * empty;
        rows_left -= empty;
    }
    let any_in_col =
        |c: i32| (0..rows_left).any(|r| marked(row0 + top + r, col0 + c) == Some(true));
    let first = (0..cols).find(|&c| any_in_col(c)).unwrap_or(cols);
    out[0] = x + 16 * first;
    out[2] = w - 16 * first;
    let cols_left = cols - first;
    if out[2] > 1 {
        let last = (0..cols_left)
            .rev()
            .find(|&c| any_in_col(first + c))
            .unwrap_or(-1);
        out[2] += 16 * (last - cols_left + 1);
    }
    Some(out)
}

/// `rect`'s best cut (0x100067B0): the area it saves and the two halves (`rect` twice when
/// neither cut is taken; a half with nothing marked, which a rectangle [`bounds`] gave never
/// has, kept as cut).
fn cut(marked: &impl Fn(i32, i32) -> Option<bool>, rect: Rect) -> (i32, Rect, Rect) {
    let [x, y, w, h] = rect;
    let area = |r: Rect| r[2] * r[3];
    let half = |n: i32| (n / 2 + 15) & !15;
    let mut best = (0, rect, rect);
    if w >= 32 {
        let a = [x, y, half(w), h];
        let b = [x + half(w), y, w - half(w), h];
        let (a, b) = (
            bounds(marked, a).unwrap_or(a),
            bounds(marked, b).unwrap_or(b),
        );
        best = (area(rect) - area(a) - area(b), a, b);
    }
    if h >= 32 {
        let a = [x, y, w, half(h)];
        let b = [x, y + half(h), w, h - half(h)];
        let (a, b) = (
            bounds(marked, a).unwrap_or(a),
            bounds(marked, b).unwrap_or(b),
        );
        let saved = area(rect) - area(a) - area(b);
        if saved > best.0 {
            best = (saved, a, b);
        }
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build::{self, Block, Header};
    use crate::{Clip, Reading, check};
    use alloc::vec;
    use alloc::vec::Vec;

    /// A 40 × 40 clip: 5 × 5 luma blocks, 3 × 3 chroma blocks and cells (the last column and row
    /// of cells cut by the picture's edge at 32).
    fn decoded(frames: &[Vec<u8>]) -> (Decoder, Vec<Vec<u8>>, Target) {
        let bytes = build::file(&Header::new(40, 40), frames);
        let clip = Clip::parse(&bytes).unwrap();
        let mut decoder = Decoder::new(&clip).unwrap();
        let mut target = Target::new(&decoder);
        let mut marks = Vec::new();
        for n in 0..frames.len() {
            decoder.decode(clip.video(n).unwrap()).unwrap();
            marks.push(decoder.changed().0.to_vec());
            target.copy(&decoder);
        }
        (decoder, marks, target)
    }

    fn fills(count: usize, colour: u8) -> Vec<Block> {
        vec![Block::Fill(colour); count]
    }

    #[test]
    fn the_luma_marks_its_cells_and_the_chroma_others() {
        // Frame 0: every block filled. Frame 1: the luma skipped but its block (0, 0), the chroma
        // filled anew but U's block (2, 1) and V's (0, 2).
        let key = build::frame(40, 40, [&fills(25, 10), &fills(9, 20), &fills(9, 30)]);
        let mut luma = vec![Block::Skip; 25];
        luma[0] = Block::Fill(11);
        let mut u = fills(9, 21);
        u[3 + 2] = Block::Skip;
        let mut v = fills(9, 31);
        v[2 * 3] = Block::Skip;
        let next = build::frame(40, 40, [&luma, &u, &v]);
        let (decoder, marks, target) = decoded(&[key, next]);
        assert_eq!(decoder.changed().1, 3);
        assert_eq!(marks[0], vec![1; 9], "every cell in the key frame");
        // Luma (0, 0): cell 0. Chroma (bx, by): (by / 2) · 1 + bx, rows 0 and 1 to cells 0 to
        // 2, row 2 to 1 to 3 (U's (2, 1) and V's (0, 2) skipped, the others there). Cell 4, the
        // chroma's block (1, 1), is not marked, nor are 5 to 8.
        assert_eq!(marks[1], vec![1, 1, 1, 1, 0, 0, 0, 0, 0]);
        let [y, u, v] = target.frame().planes();
        let [dy, du, dv] = decoder.frame().planes();
        // Every luma pixel as decoded: only one block changed, in a marked cell.
        assert!((0..40).all(|r| y.row(r) == dy.row(r)));
        // The chroma: cell 4 (chroma 8..16 across and down, inside the picture's whole cells)
        // kept frame 0's; cells 2, 5 to 8 lie on the strips right of and below 32, copied whole.
        for (target, source, old) in [(u, du, 20), (v, dv, 30)] {
            for r in 0..20 {
                for c in 0..20 {
                    let kept = (8..16).contains(&r) && (8..16).contains(&c);
                    let want = if kept { old } else { source.row(r)[c] };
                    assert_eq!(target.row(r)[c], want, "row {r}, column {c}");
                }
            }
        }
        assert_eq!(du.row(8)[8], 21, "the decoder has the change");
    }

    #[test]
    fn copies_names_the_marked_cells_and_the_strips() {
        let key = build::frame(40, 40, [&fills(25, 10), &fills(9, 20), &fills(9, 30)]);
        let mut luma = vec![Block::Skip; 25];
        luma[0] = Block::Fill(11);
        let still = build::frame(
            40,
            40,
            [&luma, &vec![Block::Skip; 9], &vec![Block::Skip; 9]],
        );
        let (decoder, marks, _) = decoded(&[key, still]);
        assert_eq!(marks[1], vec![1, 0, 0, 0, 0, 0, 0, 0, 0]);
        assert!(copies(&decoder, 0, 0) && copies(&decoder, 15, 15));
        assert!(!copies(&decoder, 16, 0) && !copies(&decoder, 31, 31));
        assert!(copies(&decoder, 32, 0) && copies(&decoder, 0, 32) && copies(&decoder, 39, 39));
    }

    /// A 128 × 64 clip (8 × 4 cells) whose second frame changes the luma blocks of `cells`
    /// alone (each cell's top left block), the chroma skipped: the second frame's rectangles.
    fn rects_of(width: usize, height: usize, cells: &[(usize, usize)]) -> Vec<Rect> {
        let (bw, bh) = (width.div_ceil(8), height.div_ceil(8));
        let chroma = width.div_ceil(16) * height.div_ceil(16);
        let key = build::frame(
            width,
            height,
            [&fills(bw * bh, 10), &fills(chroma, 20), &fills(chroma, 30)],
        );
        let mut luma = vec![Block::Skip; bw * bh];
        for &(cx, cy) in cells {
            luma[2 * cy * bw + 2 * cx] = Block::Fill(99);
        }
        let skip = vec![Block::Skip; chroma];
        let next = build::frame(width, height, [&luma, &skip, &skip]);
        let bytes = build::file(&Header::new(width as u32, height as u32), &[key, next]);
        let clip = Clip::parse(&bytes).unwrap();
        let mut decoder = Decoder::new(&clip).unwrap();
        decoder.decode(clip.video(0).unwrap()).unwrap();
        assert_eq!(rects(&decoder), vec![[0, 0, width as i32, height as i32]]);
        decoder.decode(clip.video(1).unwrap()).unwrap();
        rects(&decoder)
    }

    #[test]
    fn the_rectangles_are_the_marked_cells_cut_where_it_saves_most() {
        // Two corners: one cut across (a tie with the cut down keeps across), then nothing
        // more to save; the two in their order, top first.
        assert_eq!(
            rects_of(128, 64, &[(7, 3), (0, 0)]),
            vec![[0, 0, 16, 16], [112, 48, 16, 16]]
        );
        // One cell: its own.
        assert_eq!(rects_of(128, 64, &[(3, 2)]), vec![[48, 32, 16, 16]]);
        // Nothing marked: none.
        assert!(rects_of(128, 64, &[]).is_empty());
        // A row of cells is one rectangle (no cut saves anything).
        let row: Vec<_> = (0..8).map(|x| (x, 1)).collect();
        assert_eq!(rects_of(128, 64, &row), vec![[0, 16, 128, 16]]);
        // Scattered cells: at most eight, ordered by top, then left, the bottom row cut at
        // the picture's edge (60 of 64).
        let spread: Vec<_> = (0..8)
            .flat_map(|x| [(x, 0), (x, 3)])
            .filter(|&(x, _)| x % 2 == 0)
            .collect();
        let got = rects_of(128, 60, &spread);
        assert_eq!(got.len(), MAX_RECTS);
        let keys: Vec<u32> = got
            .iter()
            .map(|r| ((r[1] as u32) << 16) + r[0] as u32)
            .collect();
        assert!(keys.windows(2).all(|k| k[0] < k[1]), "{got:?}");
        assert!(got.iter().all(|r| r[1] + r[3] <= 60 && r[0] + r[2] <= 128));
        assert!(got.iter().any(|r| r[1] == 48 && r[3] == 12));
    }

    #[test]
    fn a_cut_down_wins_when_it_saves_more() {
        // Cells (0, 0) and (1, 3): across at 16 px saves 32 · 64 − 2 · 16 · 16; down at 32 the
        // same: across kept. With (0, 0), (1, 0) and (0, 3): across saves 32 · 64 − 16 · 64 − 16 ·
        // 16, down 32 · 64 − 32 · 16 − 16 · 16: down.
        assert_eq!(
            rects_of(32, 64, &[(0, 0), (1, 3)]),
            vec![[0, 0, 16, 16], [16, 48, 16, 16]]
        );
        assert_eq!(
            rects_of(32, 64, &[(0, 0), (1, 0), (0, 3)]),
            vec![[0, 0, 32, 16], [0, 48, 16, 16]]
        );
    }

    #[test]
    fn a_target_starts_empty_and_a_skip_changes_nothing() {
        let key = build::frame(40, 40, [&[Block::Skip], &[], &[]]);
        let (decoder, marks, target) = decoded(&[key]);
        assert_eq!(marks[0], vec![0; 9]);
        assert!(
            target
                .frame()
                .planes()
                .iter()
                .all(|p| p.pixels().iter().all(|&b| b == 0))
        );
        assert!(Target::new(&decoder) == target);
    }

    #[test]
    fn the_original_reading_gives_the_targets_frames() {
        // The luma standing still in cell 4 while the chroma changes there (the module's fault).
        let key = build::frame(40, 40, [&fills(25, 10), &fills(9, 20), &fills(9, 30)]);
        let next = build::frame(
            40,
            40,
            [&vec![Block::Skip; 25], &fills(9, 21), &fills(9, 31)],
        );
        let bytes = build::file(&Header::new(40, 40), &[key, next]);
        let fixed = check::frames(&bytes).unwrap();
        let original = check::frames_read(&bytes, Reading::Original).unwrap();
        assert!(fixed[0] == original[0]);
        assert_eq!(fixed[1].planes()[1].row(8)[8], 21);
        assert_eq!(original[1].planes()[1].row(8)[8], 20);
        assert_eq!(original[1].planes()[1].row(0)[0], 21);
        assert!(original[1].planes()[0] == fixed[1].planes()[0]);
    }
}
