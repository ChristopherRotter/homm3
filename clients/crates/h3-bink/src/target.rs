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
        let (width, height) = decoder.size();
        let (right, bottom) = (width & !15, height & !15);
        let (changed, wide) = decoder.changed();
        let planes = self.frame.planes_mut().iter_mut();
        for (p, (target, source)) in planes.zip(decoder.frame().planes()).enumerate() {
            // A chroma pixel's luma position: twice its own.
            let shift = usize::from(p > 0);
            let stride = target.stride();
            for (y, row) in source.pixels().chunks(stride).enumerate() {
                let ly = y << shift;
                for (x, &value) in row.iter().enumerate() {
                    let lx = x << shift;
                    let cell = (ly >> 4) * wide + (lx >> 4);
                    if lx >= right || ly >= bottom || changed.get(cell).is_some_and(|&c| c != 0) {
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
    fn a_target_starts_empty_and_a_skip_changes_nothing() {
        let key = build::frame(40, 40, [&[Block::Skip], &[], &[]]);
        let (decoder, marks, target) = decoded(&[key]);
        assert_eq!(marks[0], vec![0; 9]);
        assert!(target.frame().planes().iter().all(|p| p.pixels().iter().all(|&b| b == 0)));
        assert!(Target::new(&decoder) == target);
    }

    #[test]
    fn the_original_reading_gives_the_targets_frames() {
        // The luma standing still in cell 4 while the chroma changes there (the module's fault).
        let key = build::frame(40, 40, [&fills(25, 10), &fills(9, 20), &fills(9, 30)]);
        let next = build::frame(40, 40, [&vec![Block::Skip; 25], &fills(9, 21), &fills(9, 31)]);
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
