//! The game's bitmap fonts (`.FNT`; E17 M3 T4a, `docs/epics/E17-design.md` §3.6): what the font
//! loader keeps of a file (0x55B750 → 0x4B5070) and the text drawer reads (0x4B5490 → 0x4B51A0).
//!
//! A file is 4 128 bytes the loader copies verbatim, then the glyphs' pixels:
//!
//! - a 32-byte header: byte 5 the line height, byte 6 a signed offset every line is drawn down
//!   by; the drawer reads no other byte (the files hold `1F FF 08 00 00` or `1E FF 08 00 00`
//!   before them, zeros after);
//! - 256 glyph metrics, three little-endian `i32`s each: the space before the glyph's pixels,
//!   their width, the space after (the advance is the sum; either space may be negative);
//! - 256 little-endian `u32` offsets into the pixels;
//! - the pixels: a glyph's are `width × height` bytes at its offset, row by row, top row first.
//!   A pixel 0 is not drawn, 255 is drawn in the text's colour, any other in the shadow's
//!   (0x4B51A0).
//!
//! The loader checks nothing; the decoder rejects a glyph whose pixels lie past the end. A glyph
//! of width 0 or less has no pixels (the drawer's row loop draws none) but keeps its advance.
//! Checked against the user's files by the real-asset gate: the nine fonts of the four LODs.

use alloc::vec::Vec;

use crate::Error;
use crate::read::{index, slice, u8_at, u32_at};

/// The bytes the loader copies before the pixels: the header, the metrics and the offsets.
pub const HEADER_BYTES: usize = 0x1020;
/// Where the glyph metrics start.
const METRICS: usize = 32;
/// Where the pixel offsets start.
const OFFSETS: usize = METRICS + 256 * 12;
/// The header byte holding the line height.
const HEIGHT: usize = 5;
/// The header byte holding the lines' y offset.
const Y_OFFSET: usize = 6;

/// A glyph pixel drawn in the text's colour; 0 is not drawn, any other in the shadow's.
pub const INK: u8 = 0xFF;

/// A decoded font.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Font {
    /// The line height: every glyph's rows.
    pub height: u8,
    /// What each line is drawn down by (0x4B5271; 0 in every file of the game).
    pub y_offset: i8,
    /// The 256 glyphs, by byte value.
    pub glyphs: Vec<Glyph>,
}

/// A glyph: its metrics and pixels.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Glyph {
    /// The space before its pixels (may be negative).
    pub left: i32,
    /// Its pixels' width.
    pub width: i32,
    /// The space after its pixels (may be negative).
    pub right: i32,
    /// `width × height` pixels, row by row; none when `width` is 0 or less.
    pub pixels: Vec<u8>,
}

impl Glyph {
    /// How far the pen moves past it: the three metrics' sum (wrapping, as the executable's
    /// 32-bit additions).
    #[must_use]
    pub fn advance(&self) -> i32 {
        self.left.wrapping_add(self.width).wrapping_add(self.right)
    }
}

impl Font {
    /// The glyph of byte `c` (a font always holds 256).
    #[must_use]
    pub fn glyph(&self, c: u8) -> &Glyph {
        const EMPTY: &Glyph = &Glyph {
            left: 0,
            width: 0,
            right: 0,
            pixels: Vec::new(),
        };
        self.glyphs.get(usize::from(c)).unwrap_or(EMPTY)
    }
}

/// Decodes a font file.
///
/// # Errors
/// [`Error::Truncated`]: shorter than [`HEADER_BYTES`], or a glyph's pixels run past the end.
pub fn decode_fnt(bytes: &[u8]) -> Result<Font, Error> {
    let Some(head) = bytes.first_chunk::<HEADER_BYTES>() else {
        return Err(Error::Truncated);
    };
    // Every read below lies inside `head`.
    let height = u8_at(head, HEIGHT).unwrap_or(0);
    let y_offset = i8::from_le_bytes([u8_at(head, Y_OFFSET).unwrap_or(0)]);
    let word = |at: usize| u32_at(head, at).unwrap_or(0);
    let signed = |at: usize| i32::from_le_bytes(word(at).to_le_bytes());
    let mut glyphs = Vec::with_capacity(256);
    for c in 0..256 {
        let [left, width, right] = [0, 4, 8].map(|d| signed(METRICS + 12 * c + d));
        let rows = usize::from(height);
        let len = usize::try_from(width).unwrap_or(0).saturating_mul(rows);
        let pixels = if len == 0 {
            Vec::new()
        } else {
            let at = HEADER_BYTES.saturating_add(index(word(OFFSETS + 4 * c)));
            slice(bytes, at, len).ok_or(Error::Truncated)?.to_vec()
        };
        glyphs.push(Glyph {
            left,
            width,
            right,
            pixels,
        });
    }
    Ok(Font {
        height,
        y_offset,
        glyphs,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build;
    use alloc::vec;
    use proptest::prelude::*;

    fn glyph(left: i32, width: i32, right: i32, rows: usize) -> Glyph {
        let n = usize::try_from(width).unwrap_or(0) * rows;
        Glyph {
            left,
            width,
            right,
            pixels: (0..n)
                .map(|i| u8::try_from(i % 3).unwrap() * 0x7F)
                .collect(),
        }
    }

    #[test]
    fn a_font_round_trips() {
        let mut glyphs = vec![Glyph::default(); 256];
        glyphs[usize::from(b'1')] = glyph(1, 3, 1, 2);
        glyphs[usize::from(b'A')] = glyph(-1, 2, -2, 2);
        glyphs[7] = glyph(0, -4, 9, 2);
        let font = Font {
            height: 2,
            y_offset: -3,
            glyphs,
        };
        let bytes = build::fnt(&font);
        assert_eq!(bytes.len(), HEADER_BYTES + 6 + 4);
        assert_eq!(decode_fnt(&bytes), Ok(font.clone()));
        assert_eq!(font.glyph(b'1').advance(), 5);
        assert_eq!(font.glyph(b'A').advance(), -1);
        assert_eq!(font.glyph(7).advance(), 5);
        assert!(font.glyph(7).pixels.is_empty());
        let wide = Glyph {
            left: i32::MAX,
            width: 1,
            ..Glyph::default()
        };
        assert_eq!(wide.advance(), i32::MIN);
        // A font missing glyphs (built by hand) answers an empty one.
        let short = Font {
            glyphs: vec![],
            ..font
        };
        assert_eq!(short.glyph(b'1'), &Glyph::default());
    }

    #[test]
    fn short_files_and_stray_offsets_are_truncated() {
        let mut glyphs = vec![Glyph::default(); 256];
        glyphs[0] = glyph(0, 2, 0, 3);
        let font = Font {
            height: 3,
            y_offset: 0,
            glyphs,
        };
        let bytes = build::fnt(&font);
        assert_eq!(
            decode_fnt(&bytes[..HEADER_BYTES - 1]),
            Err(Error::Truncated)
        );
        assert_eq!(decode_fnt(&bytes[..bytes.len() - 1]), Err(Error::Truncated));
        let mut far = bytes.clone();
        far[OFFSETS..OFFSETS + 4].copy_from_slice(&u32::MAX.to_le_bytes());
        assert_eq!(decode_fnt(&far), Err(Error::Truncated));
        // A height of 0: no glyph has pixels.
        let mut flat = bytes;
        flat[HEIGHT] = 0;
        let decoded = decode_fnt(&flat).unwrap();
        assert!(decoded.glyphs.iter().all(|g| g.pixels.is_empty()));
        assert_eq!(decoded.glyphs[0].width, 2);
    }

    proptest! {
        #[test]
        fn decode_fnt_never_panics(
            bytes in proptest::collection::vec(any::<u8>(), HEADER_BYTES..HEADER_BYTES + 64)
        ) {
            let _ = decode_fnt(&bytes);
        }
    }
}
