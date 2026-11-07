//! The block transforms: the 8 × 8 inverse DCT as the game's `BINKW32.DLL` computes it (E17 M6
//! T6c1, CLT-Q30: its dequantization and both passes in x87 floating point, emulated in integers
//! by [`crate::x87`]), stored or added to the pixels; the residue added (FFmpeg 6.1.1's
//! `add_pixels8_c`, `libavcodec/binkdsp.c`). A pixel is stored or added modulo 256 (no clipping),
//! as both do.

use crate::x87::X;

/// `BINKW32.DLL`'s constants of the transform (binary32 at 0x1001813C, 0x10018138, 0x10018134,
/// 0x10018130): √2, 1.848, 2.613, 1.082.
const SQRT2: X = X::from_bits32(0x3FB5_04F3);
const K1848: X = X::from_bits32(0x3FEC_835E);
const K2613: X = X::from_bits32(0x4027_3D75);
const K1082: X = X::from_bits32(0x3F8A_8BD4);

/// The DLL's columns' pass (0x10001640's loop 0x10001664 to 0x1000188E; 0x10001A50's the
/// same): each column's coefficients (16-bit words, `movsx`) times their dequantizers on the x87
/// stack (exact: 16 by 24 bits), then the butterflies, each result stored as binary32 or kept on
/// the stack (53 bits) exactly where the DLL keeps it (its trace: `docs/epics/E17-design.md`
/// §7.27). A column whose rows 1 to 7 are all zero takes the DLL's shortcut, its first product
/// stored in every row (the same values the butterflies give).
fn dll_columns(block: &[i32; 64], quant: &[X; 64]) -> [X; 64] {
    let mut out = [X::ZERO; 64];
    for x in 0..8 {
        let word = |r: usize| i32::from(block[x + 8 * r] as i16);
        let p: [X; 8] = core::array::from_fn(|r| X::int(word(r)).mul(quant[x + 8 * r]));
        let column = if (1..8).all(|r| word(r) == 0) {
            [p[0].single(); 8]
        } else {
            dll_column(p)
        };
        for (r, v) in column.into_iter().enumerate() {
            out[x + 8 * r] = v;
        }
    }
    out
}

/// One column's butterflies from its eight products (0x100016D0 to 0x10001879).
fn dll_column(p: [X; 8]) -> [X; 8] {
    let s7 = p[7].single();
    let a6 = p[1].add(s7).single();
    let a4 = p[5].add(p[3]).single();
    let b0 = a6.add(a4).single();
    let s6 = p[6].single();
    let s2 = p[2].single();
    let a2 = s6.add(s2);
    let a0 = p[4].add(p[0]).single();
    let e0 = a2.add(a0).single();
    let a7 = p[1].sub(s7);
    let a5 = p[5].sub(p[3]).single();
    let b1 = a7.add(a5).mul(K1848);
    let b2 = b1.sub(a5.mul(K2613)).sub(b0).single();
    let a3 = s2.sub(s6).mul(SQRT2).sub(a2).single();
    let a1 = p[0].sub(p[4]);
    let e1 = a3.add(a1).single();
    let b3 = a6.sub(a4).mul(SQRT2).sub(b2).single();
    let e2 = a1.sub(a3);
    let b4 = a7.mul(K1082).sub(b1).add(b3).single();
    let e3 = a0.sub(a2).single();
    [
        b0.add(e0),
        b2.add(e1),
        b3.add(e2),
        e3.sub(b4),
        b4.add(e3),
        e2.sub(b3),
        e1.sub(b2),
        e0.sub(b0),
    ]
    .map(X::single)
}

/// Row `y` of the DLL's rows' pass (0x100018AB to 0x10001987) and its pixels (0x100019A0: the
/// last sums on the stack, `fistp word`, the low byte); its own rounding points, not the
/// columns'.
fn dll_row(columns: &[X; 64], y: usize) -> [u8; 8] {
    let v: [X; 8] = core::array::from_fn(|k| columns[8 * (y & 7) + k]);
    let a2 = v[6].add(v[2]);
    let a0 = v[4].add(v[0]);
    let e0 = a2.add(a0).single();
    let a6 = v[7].add(v[1]).single();
    let a4 = v[5].add(v[3]);
    let b0 = a6.add(a4).single();
    let a3 = v[2].sub(v[6]).mul(SQRT2).sub(a2).single();
    let a1 = v[0].sub(v[4]).single();
    let e1 = a3.add(a1).single();
    let a7 = v[1].sub(v[7]).single();
    let a5 = v[5].sub(v[3]);
    let b1 = a7.add(a5).mul(K1848).single();
    let b2 = b1.sub(a5.mul(K2613)).sub(b0).single();
    let e2 = a1.sub(a3).single();
    let b3 = a6.sub(a4).mul(SQRT2).sub(b2).single();
    let e3 = a0.sub(a2).single();
    let b4 = a7.mul(K1082).sub(b1).add(b3).single();
    [
        e0.add(b0),
        e1.add(b2),
        e2.add(b3),
        e3.sub(b4),
        e3.add(b4),
        e2.sub(b3),
        e1.sub(b2),
        e0.sub(b0),
    ]
    .map(X::pixel)
}

/// The DLL's inverse DCT stored into the pixels (0x10001A20 into 0x10001640): `block` the
/// coefficients by natural position, `quant` the dequantizers.
pub(crate) fn dll_put(
    pixels: &mut [u8],
    at: usize,
    stride: usize,
    block: &[i32; 64],
    quant: &[X; 64],
) {
    let columns = dll_columns(block, quant);
    let rows: [[u8; 8]; 8] = core::array::from_fn(|y| dll_row(&columns, y));
    each(pixels, at, stride, |i, _| rows[i >> 3][i & 7]);
}

/// The DLL's inverse DCT added to the pixels (0x10001A50: the moved block plus each pixel's low
/// byte, modulo 256).
pub(crate) fn dll_add(
    pixels: &mut [u8],
    at: usize,
    stride: usize,
    block: &[i32; 64],
    quant: &[X; 64],
) {
    let columns = dll_columns(block, quant);
    let rows: [[u8; 8]; 8] = core::array::from_fn(|y| dll_row(&columns, y));
    each(pixels, at, stride, |i, p| {
        p.wrapping_add(rows[i >> 3][i & 7])
    });
}

/// Stores `value` at `at`; nothing outside `pixels` (the decoder's writes always lie inside: its
/// planes and bundles are sized for every block and value).
pub(crate) fn set(pixels: &mut [u8], at: usize, value: u8) {
    if let Some(slot) = pixels.get_mut(at) {
        *slot = value;
    }
}

/// Each pixel of the 8 × 8 block at `at` (rows `stride` apart) through `f`; a pixel outside
/// `pixels` is left alone (the callers' blocks always lie inside).
fn each(pixels: &mut [u8], at: usize, stride: usize, mut f: impl FnMut(usize, u8) -> u8) {
    for y in 0..8 {
        for x in 0..8 {
            if let Some(p) = pixels.get_mut(at + y * stride + x) {
                *p = f(8 * y + x, *p);
            }
        }
    }
}

/// `add_pixels8_c`: the residue added to the pixels.
pub(crate) fn add_residue(pixels: &mut [u8], at: usize, stride: usize, block: &[i16; 64]) {
    each(pixels, at, stride, |i, p| {
        p.wrapping_add(block[i & 63] as u8)
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tables::{DLL_INTER, DLL_INTRA};

    /// A block of `(position, coefficient)` pairs by natural position.
    fn block(pairs: &[(usize, i32)]) -> [i32; 64] {
        let mut block = [0; 64];
        for &(at, v) in pairs {
            block[at] = v;
        }
        block
    }

    #[test]
    fn a_dc_alone_fills_the_block() {
        // Every column takes the shortcut, so each pixel is the DC times its dequantizer, the
        // rows' sums of eight equal values over the DCT's scale (q 0's DC dequantizer 0.125).
        let mut pixels = [0u8; 64];
        dll_put(&mut pixels, 0, 8, &block(&[(0, 64)]), &DLL_INTRA[0]);
        assert!(pixels.iter().all(|&p| p == 8), "{pixels:?}");
        dll_add(&mut pixels, 0, 8, &block(&[(0, 64)]), &DLL_INTRA[0]);
        assert!(pixels.iter().all(|&p| p == 16));
        // Negative and past a byte: the low byte of the rounded word.
        dll_put(&mut pixels, 0, 8, &block(&[(0, -8)]), &DLL_INTRA[0]);
        assert!(pixels.iter().all(|&p| p == 0xFF), "{pixels:?}");
        dll_put(&mut pixels, 0, 8, &block(&[(0, 2400)]), &DLL_INTRA[0]);
        assert!(pixels.iter().all(|&p| p == 44), "300 stored as 44");
    }

    #[test]
    fn coefficients_are_the_dlls_words() {
        // The DLL keeps coefficients as 16-bit words: 65 536 + 64 is 64 to it.
        let mut wide = [0u8; 64];
        let mut narrow = [0u8; 64];
        dll_put(
            &mut wide,
            0,
            8,
            &block(&[(0, 65_536 + 64), (9, 65_536)]),
            &DLL_INTRA[0],
        );
        dll_put(&mut narrow, 0, 8, &block(&[(0, 64)]), &DLL_INTRA[0]);
        assert_eq!(wide, narrow);
    }

    #[test]
    fn the_shortcut_gives_what_the_butterflies_give() {
        // A column whose rows 1 to 7 are zero: the DLL stores its first product in every row,
        // which is what the butterflies give it (each sum then adds zeros).
        for (c, q) in [(100, 3), (-2048, 15), (1, 0), (32_767, 9)] {
            let p = X::int(c).mul(DLL_INTER[q][0]);
            let mut column = [X::ZERO; 8];
            column[0] = p;
            assert_eq!(dll_column(column), [p.single(); 8], "{c} at {q}");
        }
        // A column through the butterflies: its rows differ; an empty column stays zero.
        let full = dll_columns(&block(&[(0, 100), (8, 40)]), &DLL_INTER[3]);
        assert_ne!(full[0], full[8 * 7]);
        assert_eq!(full[1], X::ZERO);
    }

    #[test]
    fn a_block_through_both_passes_rounds_as_the_dll() {
        // An intra block and an inter one added: the DLL's pixels (a clip of the two blocks,
        // their first and last pixels and sums from `binkprobe frames`, E17 M6 T6c1).
        let intra = block(&[(0, 1234), (1, -77), (8, 300), (19, 9), (63, -2048), (36, 1)]);
        let mut pixels = [0u8; 80];
        dll_put(&mut pixels, 8, 9, &intra, &DLL_INTRA[7]);
        assert_eq!(pixels[..8], [0; 8], "outside the block");
        let sum: u32 = pixels.iter().map(|&p| u32::from(p)).sum();
        assert_eq!((pixels[8], pixels[8 + 9 * 7 + 7], sum), (145, 230, 6_902));
        let inter = block(&[(0, -300), (2, 55), (40, 1000), (27, -3)]);
        let mut added = [50u8; 64];
        dll_add(&mut added, 0, 8, &inter, &DLL_INTER[15]);
        let sum: u32 = added.iter().map(|&p| u32::from(p)).sum();
        assert_eq!((added[0], added[63], sum), (212, 82, 8_383));
        // A block running past the pixels' end is cut there, as is a single store.
        let mut short = [7u8; 10];
        dll_put(&mut short, 0, 8, &intra, &DLL_INTRA[7]);
        assert_eq!(short[9], dll_row(&dll_columns(&intra, &DLL_INTRA[7]), 1)[1]);
        let mut one = [1u8];
        set(&mut one, 1, 9);
        set(&mut one, 0, 2);
        assert_eq!(one, [2]);
    }

    #[test]
    fn words_out_of_range_store_the_indefinites_low_byte() {
        // DC 2047 and 32 767 at natural position 1, quantizer index 15: five of the first row's
        // sums leave −32 768 to 32 767, so `fistp` stores 0x8000 and the pixel is 0. The DLL's
        // pixels (a clip of the block through `binkprobe frames`, E17 M6 T6c1).
        let mut pixels = [0u8; 64];
        dll_put(
            &mut pixels,
            0,
            8,
            &block(&[(0, 2047), (1, 32_767)]),
            &DLL_INTRA[15],
        );
        assert_eq!(pixels[..8], [0, 0, 0, 35, 218, 74, 0, 0]);
    }

    #[test]
    fn residues_add_modulo_256() {
        let mut pixels = [0xFFu8; 64];
        let mut residue = [0i16; 64];
        residue[9] = 2;
        residue[10] = -300;
        add_residue(&mut pixels, 0, 8, &residue);
        assert_eq!(
            (pixels[9], pixels[10], pixels[11]),
            (1, 0xFF_u8.wrapping_add(212), 0xFF)
        );
    }
}
