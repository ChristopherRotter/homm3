//! The game's PCX images (E17 M1 T2). Despite the extension they are not ZSoft PCX: a 12-byte
//! header (`size`, `width`, `height`, little-endian `u32`s), then either `width × height` palette
//! indices followed by a 768-byte palette (`size == width × height`; the archives' type 16), or
//! `width × height` pixels of three bytes in the order blue, green, red (`size == 3 × width ×
//! height`; type 17). Both are row-major, top row first. Checked against the user's files by the
//! real-asset gate: every PCX of the four LODs has exactly this length.

use alloc::boxed::Box;
use alloc::vec::Vec;

use crate::Error;
use crate::palette::{PALETTE_BYTES, Palette, Rgb};
use crate::read::u32s;

/// The size of the header.
pub const HEADER_BYTES: usize = 12;

/// A decoded PCX image.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Pcx {
    /// 8-bit: palette indices and their palette (boxed: the palette is 768 bytes).
    Indexed(Box<IndexedImage>),
    /// 24-bit colour.
    Rgb(RgbImage),
}

impl Pcx {
    /// The image's width in pixels.
    #[must_use]
    pub fn width(&self) -> u32 {
        match self {
            Self::Indexed(i) => i.width,
            Self::Rgb(i) => i.width,
        }
    }

    /// The image's height in pixels.
    #[must_use]
    pub fn height(&self) -> u32 {
        match self {
            Self::Indexed(i) => i.height,
            Self::Rgb(i) => i.height,
        }
    }
}

/// An 8-bit image.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexedImage {
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// `width × height` palette indices, row-major, top row first.
    pub pixels: Vec<u8>,
    /// The image's palette.
    pub palette: Palette,
}

/// A 24-bit image.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RgbImage {
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// `width × height` colours, row-major, top row first.
    pub pixels: Vec<Rgb>,
}

/// Decodes a PCX image. The length must be exact: a shorter input is [`Error::Truncated`], a
/// longer one [`Error::TrailingBytes`]; a `size` that is neither `width × height` nor three times
/// that is [`Error::Field`]`("size")`. When both match (an empty image), the image is 8-bit.
pub fn decode_pcx(bytes: &[u8]) -> Result<Pcx, Error> {
    let [size, width, height] = u32s(bytes, 0).ok_or(Error::Truncated)?;
    let size = u64::from(size);
    let body = bytes.get(HEADER_BYTES..).unwrap_or_default();
    let pixels = u64::from(width) * u64::from(height);
    if size == pixels {
        let (indices, rest) = body
            .split_at_checked(usize::try_from(pixels).unwrap_or(usize::MAX))
            .ok_or(Error::Truncated)?;
        let palette = exact::<PALETTE_BYTES>(rest)?;
        Ok(Pcx::Indexed(Box::new(IndexedImage {
            width,
            height,
            pixels: indices.to_vec(),
            palette: Palette::from_triples(palette),
        })))
    } else if pixels.checked_mul(3) == Some(size) {
        let (colors, rest) = body
            .split_at_checked(usize::try_from(size).unwrap_or(usize::MAX))
            .ok_or(Error::Truncated)?;
        exact::<0>(rest)?;
        Ok(Pcx::Rgb(RgbImage {
            width,
            height,
            pixels: colors
                .as_chunks::<3>()
                .0
                .iter()
                .map(|&[b, g, r]| Rgb { r, g, b })
                .collect(),
        }))
    } else {
        Err(Error::Field("size"))
    }
}

/// `rest` as exactly `N` bytes.
fn exact<const N: usize>(rest: &[u8]) -> Result<&[u8; N], Error> {
    rest.try_into().map_err(|_| {
        if rest.len() < N {
            Error::Truncated
        } else {
            Error::TrailingBytes
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build;
    use alloc::vec;
    use proptest::prelude::*;

    fn palette() -> Palette {
        let mut p = Palette::default();
        p.0[1] = Rgb {
            r: 10,
            g: 20,
            b: 30,
        };
        p.0[255] = Rgb {
            r: 255,
            g: 0,
            b: 255,
        };
        p
    }

    #[test]
    fn an_indexed_image_round_trips() {
        let pixels = [0, 1, 2, 255, 1, 0];
        let bytes = build::pcx_indexed(3, 2, &pixels, &palette());
        assert_eq!(bytes.len(), HEADER_BYTES + 6 + PALETTE_BYTES);
        let pcx = decode_pcx(&bytes).unwrap();
        assert_eq!((pcx.width(), pcx.height()), (3, 2));
        assert_eq!(
            pcx,
            Pcx::Indexed(Box::new(IndexedImage {
                width: 3,
                height: 2,
                pixels: pixels.to_vec(),
                palette: palette()
            }))
        );
    }

    #[test]
    fn a_24_bit_image_is_stored_blue_green_red() {
        let red = Rgb { r: 200, g: 1, b: 2 };
        let bytes = build::pcx_rgb(1, 2, &[red, Rgb::default()]);
        assert_eq!(&bytes[HEADER_BYTES..HEADER_BYTES + 3], &[2, 1, 200]);
        let pcx = decode_pcx(&bytes).unwrap();
        assert_eq!((pcx.width(), pcx.height()), (1, 2));
        assert_eq!(
            pcx,
            Pcx::Rgb(RgbImage {
                width: 1,
                height: 2,
                pixels: vec![red, Rgb::default()]
            })
        );
    }

    #[test]
    fn an_empty_image_is_8_bit() {
        let bytes = build::pcx_indexed(0, 5, &[], &palette());
        assert!(matches!(decode_pcx(&bytes), Ok(Pcx::Indexed(i)) if i.pixels.is_empty()));
    }

    #[test]
    fn lengths_and_sizes_are_exact() {
        let indexed = build::pcx_indexed(2, 2, &[1; 4], &palette());
        let rgb = build::pcx_rgb(2, 1, &[Rgb::default(); 2]);
        for good in [&indexed, &rgb] {
            assert_eq!(decode_pcx(&good[..good.len() - 1]), Err(Error::Truncated));
            let mut long = good.clone();
            long.push(0);
            assert_eq!(decode_pcx(&long), Err(Error::TrailingBytes));
        }
        assert_eq!(
            decode_pcx(&indexed[..HEADER_BYTES + 3]),
            Err(Error::Truncated)
        );
        assert_eq!(decode_pcx(&rgb[..HEADER_BYTES + 5]), Err(Error::Truncated));
        let mut bad = indexed.clone();
        bad[0] = 5;
        assert_eq!(decode_pcx(&bad), Err(Error::Field("size")));
        for cut in [3, 7, 11] {
            assert_eq!(decode_pcx(&indexed[..cut]), Err(Error::Truncated));
        }
    }

    #[test]
    fn huge_headers_fail_without_allocating() {
        let mut bytes = Vec::new();
        for v in [u32::MAX, u32::MAX, 1] {
            bytes.extend_from_slice(&v.to_le_bytes());
        }
        assert_eq!(decode_pcx(&bytes), Err(Error::Truncated));
        bytes[0..4].copy_from_slice(&0xFFFF_FFFDu32.to_le_bytes());
        bytes[4..8].copy_from_slice(&0x5555_5555u32.to_le_bytes());
        bytes[8..12].copy_from_slice(&3u32.to_le_bytes());
        // `size` is neither `width × height` nor three times that.
        assert_eq!(decode_pcx(&bytes), Err(Error::Field("size")));
        bytes[0..4].copy_from_slice(&0xFFFF_FFFFu32.to_le_bytes());
        bytes[4..8].copy_from_slice(&0x5555_5555u32.to_le_bytes());
        bytes[8..12].copy_from_slice(&1u32.to_le_bytes());
        assert_eq!(decode_pcx(&bytes), Err(Error::Truncated));
    }

    proptest! {
        #[test]
        fn decode_pcx_never_panics(bytes in proptest::collection::vec(any::<u8>(), 0..2000)) {
            let _ = decode_pcx(&bytes);
        }

        #[test]
        fn indexed_images_round_trip(w in 0u32..12, h in 0u32..12, seed: u8) {
            let n = usize::try_from(w * h).unwrap();
            let pixels: Vec<u8> = (0..n).map(|i| u8::try_from(i % 256).unwrap() ^ seed).collect();
            let bytes = build::pcx_indexed(w, h, &pixels, &palette());
            let decoded = decode_pcx(&bytes).unwrap();
            prop_assert_eq!(
                decoded,
                Pcx::Indexed(Box::new(IndexedImage { width: w, height: h, pixels, palette: palette() }))
            );
        }

        #[test]
        fn rgb_images_round_trip(w in 1u32..12, h in 1u32..12, seed: u8) {
            let n = usize::try_from(w * h).unwrap();
            let pixels: Vec<Rgb> = (0..n)
                .map(|i| {
                    let v = u8::try_from(i % 256).unwrap();
                    Rgb { r: v, g: seed, b: v ^ seed }
                })
                .collect();
            let bytes = build::pcx_rgb(w, h, &pixels);
            prop_assert_eq!(
                decode_pcx(&bytes).unwrap(),
                Pcx::Rgb(RgbImage { width: w, height: h, pixels })
            );
        }
    }
}
