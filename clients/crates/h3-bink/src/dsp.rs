//! The block transforms (FFmpeg 6.1.1's `libavcodec/binkdsp.c`, its C functions, and
//! `unquantize_dct_coeffs` in `libavcodec/bink.c`): the 8 × 8 inverse DCT put or added to the
//! pixels, the residue added. FFmpeg's arithmetic exactly: the multiplications unsigned and
//! wrapping, a pixel stored or added modulo 256 (no clipping).
//!
//! No sum below overflows an `i32`: every coefficient leaves [`unquantize`] in [-2^20, 2^20), the
//! column pass's outputs stay under 13 × 2^20 and the row pass's under 13 × 2^24.

use crate::tables::SCAN;

const A1: i32 = 2896;
const A2: i32 = 2217;
const A3: i32 = 3784;
const A4: i32 = -5352;

/// `MUL(X, Y)` and the unquantization's product: unsigned and wrapping, then shifted
/// arithmetically by 11.
fn mul(x: i32, y: i32) -> i32 {
    (x.cast_unsigned()
        .wrapping_mul(y.cast_unsigned())
        .cast_signed())
        >> 11
}

/// `IDCT_TRANSFORM` on eight values.
fn transform(s: [i32; 8]) -> [i32; 8] {
    let a0 = s[0] + s[4];
    let a1 = s[0] - s[4];
    let a2 = s[2] + s[6];
    let a3 = mul(A1, s[2] - s[6]);
    let a4 = s[5] + s[3];
    let a5 = s[5] - s[3];
    let a6 = s[1] + s[7];
    let a7 = s[1] - s[7];
    let b0 = a4 + a6;
    let b1 = mul(A3, a5 + a7);
    let b2 = mul(A4, a5) - b0 + b1;
    let b3 = mul(A1, a6 - a4) - b2;
    let b4 = mul(A2, a7) + b3 - b1;
    [
        a0 + a2 + b0,
        a1 + a3 - a2 + b2,
        a1 - a3 + a2 + b3,
        a0 - a2 - b4,
        a0 - a2 + b4,
        a1 - a3 + a2 - b3,
        a1 + a3 - a2 - b2,
        a0 + a2 - b0,
    ]
}

/// The columns' pass (`bink_idct_col` over the eight columns): a column whose only coefficient is
/// its first keeps it in every row.
fn columns(block: &[i32; 64]) -> [i32; 64] {
    let mut out = [0; 64];
    for x in 0..8 {
        let column: [i32; 8] = core::array::from_fn(|y| block[x + 8 * y]);
        let done = if column[1..].iter().all(|&v| v == 0) {
            [column[0]; 8]
        } else {
            transform(column)
        };
        for (y, v) in done.into_iter().enumerate() {
            out[x + 8 * y] = v;
        }
    }
    out
}

/// Row `y` of the rows' pass (`IDCT_ROW` with `MUNGE_ROW`).
fn row(columns: &[i32; 64], y: usize) -> [i32; 8] {
    let start = 8 * (y & 7);
    let values: [i32; 8] = core::array::from_fn(|x| columns[start + x]);
    transform(values).map(|v| (v + 0x7F) >> 8)
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

/// `unquantize_dct_coeffs`: the DC and each coefficient read (by its scan position, in the order
/// read) times its quantizer.
pub(crate) fn unquantize(block: &mut [i32; 64], quant: &[i32; 64], read: &[u8]) {
    block[0] = mul(block[0], quant[0]);
    for &position in read {
        // A scan position is below 64 (the coefficient lists' structure).
        let position = usize::from(position & 63);
        let at = usize::from(SCAN[position]);
        block[at] = mul(block[at], quant[position]);
    }
}

/// `bink_idct_put_c`: the inverse DCT of `block` stored into the pixels.
pub(crate) fn idct_put(pixels: &mut [u8], at: usize, stride: usize, block: &[i32; 64]) {
    let columns = columns(block);
    let rows: [[i32; 8]; 8] = core::array::from_fn(|y| row(&columns, y));
    each(pixels, at, stride, |i, _| rows[i >> 3][i & 7] as u8);
}

/// `bink_idct_add_c`: the inverse DCT of `block` added to the pixels.
pub(crate) fn idct_add(pixels: &mut [u8], at: usize, stride: usize, block: &[i32; 64]) {
    let columns = columns(block);
    let rows: [[i32; 8]; 8] = core::array::from_fn(|y| row(&columns, y));
    each(pixels, at, stride, |i, p| {
        p.wrapping_add(rows[i >> 3][i & 7] as u8)
    });
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

    #[test]
    fn a_dc_alone_fills_the_block_rounded() {
        let mut block = [0; 64];
        // The columns keep the DC in every row, each row's transform in every pixel, then
        // (v + 0x7F) >> 8.
        block[0] = 256;
        let mut pixels = [0u8; 64];
        idct_put(&mut pixels, 0, 8, &block);
        assert!(pixels.iter().all(|&p| p == 1), "{pixels:?}");
        idct_add(&mut pixels, 0, 8, &block);
        assert!(pixels.iter().all(|&p| p == 2));
    }

    #[test]
    fn stores_and_additions_wrap_modulo_256() {
        let mut block = [0; 64];
        block[0] = -256;
        let mut pixels = [0u8; 64];
        idct_put(&mut pixels, 0, 8, &block);
        assert!(pixels.iter().all(|&p| p == 0xFF), "-1 stored as 255");
        let mut residue = [0i16; 64];
        residue[9] = 2;
        residue[10] = -300;
        add_residue(&mut pixels, 0, 8, &residue);
        assert_eq!(
            (pixels[9], pixels[10], pixels[11]),
            (1, 0xFF_u8.wrapping_add(212), 0xFF)
        );
    }

    #[test]
    fn the_multiplication_is_unsigned_and_wraps() {
        assert_eq!(mul(A1, 1 << 11), A1);
        assert_eq!(mul(A4, 1 << 11), A4);
        // 2^21 × 2896 wraps past 2^32 as FFmpeg's unsigned product does.
        assert_eq!(
            mul(1 << 21, A1),
            ((2896u64 << 21) as u32).cast_signed() >> 11
        );
        let mut block = [0; 64];
        block[0] = 3;
        block[1] = -2;
        let mut quant = [0; 64];
        quant[0] = 1 << 12;
        quant[1] = 1 << 13;
        unquantize(&mut block, &quant, &[1]);
        assert_eq!((block[0], block[1]), (6, -8));
    }

    #[test]
    fn a_full_transform_matches_a_hand_computed_column() {
        // One AC coefficient in the first column: the column's transform, then each row's.
        let mut block = [0; 64];
        block[8] = 1 << 11;
        let columns = columns(&block);
        let column = transform([0, 1 << 11, 0, 0, 0, 0, 0, 0]);
        for y in 0..8 {
            assert_eq!(columns[8 * y], column[y]);
            assert_eq!(columns[8 * y + 1], 0);
        }
        let mut pixels = [0u8; 80];
        idct_put(&mut pixels, 8, 9, &block);
        assert_eq!(pixels[..8], [0; 8], "outside the block");
        assert_eq!(pixels[8], row(&columns, 0)[0] as u8);
        assert_eq!(pixels[8 + 9 * 7 + 7], row(&columns, 7)[7] as u8);
        // A block running past the pixels' end is cut there.
        let mut short = [7u8; 10];
        idct_put(&mut short, 0, 8, &block);
        assert_eq!(short[9], row(&columns, 1)[1] as u8);
    }
}
