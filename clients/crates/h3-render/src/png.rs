//! A screen as a PNG file's bytes (8-bit RGB, no filter, zlib-compressed): how agents and the user
//! look at a render. Pure; writing the bytes to a file is the adapter's.
//!
//! Each pixel is widened to 8-bit channels by [`PixelFormat::unpack`](crate::screen::PixelFormat::unpack)
//! (*provisional*): a PNG is for looking; exact comparisons with the original use the screen's own
//! 16-bit values (`docs/epics/E17-design.md` §3.2).

use alloc::vec::Vec;

use crate::screen::Screen;

/// The PNG signature.
const SIGNATURE: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];

/// The deflate level (*provisional*: a balance of size and time for stills).
const LEVEL: u8 = 6;

/// `screen` as a PNG file.
#[must_use]
pub fn encode(screen: &Screen) -> Vec<u8> {
    let format = screen.format();
    let width = usize::try_from(screen.width()).unwrap_or(usize::MAX).max(1);
    let mut raw = Vec::with_capacity(screen.pixels().len() * 3 + screen.pixels().len() / width + 1);
    for row in screen.pixels().chunks(width) {
        raw.push(0); // filter: none
        for &pixel in row {
            let c = format.unpack(pixel);
            raw.extend_from_slice(&[c.r, c.g, c.b]);
        }
    }
    let mut header = Vec::with_capacity(13);
    header.extend_from_slice(&screen.width().to_be_bytes());
    header.extend_from_slice(&screen.height().to_be_bytes());
    header.extend_from_slice(&[8, 2, 0, 0, 0]); // 8 bits, RGB, deflate, no filter, no interlace
    let mut out = Vec::from(SIGNATURE);
    chunk(&mut out, *b"IHDR", &header);
    chunk(
        &mut out,
        *b"IDAT",
        &miniz_oxide::deflate::compress_to_vec_zlib(&raw, LEVEL),
    );
    chunk(&mut out, *b"IEND", &[]);
    out
}

/// One chunk: length, type, data, the CRC of type and data.
fn chunk(out: &mut Vec<u8>, kind: [u8; 4], data: &[u8]) {
    out.extend_from_slice(&u32::try_from(data.len()).unwrap_or(u32::MAX).to_be_bytes());
    let start = out.len();
    out.extend_from_slice(&kind);
    out.extend_from_slice(data);
    let crc = crc32(out.get(start..).unwrap_or_default());
    out.extend_from_slice(&crc.to_be_bytes());
}

/// CRC-32 (ISO-HDLC, reflected polynomial `0xEDB88320`), as PNG uses it; bit by bit (stills are
/// small).
fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = u32::MAX;
    for &b in bytes {
        crc ^= u32::from(b);
        for _ in 0..8 {
            let low = crc & 1;
            crc = (crc >> 1) ^ (0xEDB8_8320 & low.wrapping_neg());
        }
    }
    !crc
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::screen::PixelFormat;

    #[test]
    fn crc_matches_the_standard_check_values() {
        assert_eq!(crc32(b"123456789"), 0xCBF4_3926);
        assert_eq!(crc32(b"IEND"), 0xAE42_6082);
        assert_eq!(crc32(b""), 0);
    }

    /// The chunks of a PNG: (type, data), each CRC checked.
    fn chunks(png: &[u8]) -> Vec<([u8; 4], Vec<u8>)> {
        assert_eq!(png[..8], SIGNATURE);
        let mut out = Vec::new();
        let mut at = 8;
        while at < png.len() {
            let len =
                usize::try_from(u32::from_be_bytes(png[at..at + 4].try_into().unwrap())).unwrap();
            let body = &png[at + 4..at + 8 + len];
            let crc = u32::from_be_bytes(png[at + 8 + len..at + 12 + len].try_into().unwrap());
            assert_eq!(crc32(body), crc);
            out.push((body[..4].try_into().unwrap(), body[4..].to_vec()));
            at += 12 + len;
        }
        out
    }

    #[test]
    fn a_screen_becomes_a_png() {
        let mut s = Screen::new(2, 2, PixelFormat::RGB565).unwrap();
        s.fill(0xF800);
        let png = encode(&s);
        let chunks = chunks(&png);
        let kinds: Vec<_> = chunks.iter().map(|(k, _)| *k).collect();
        assert_eq!(kinds, [*b"IHDR", *b"IDAT", *b"IEND"]);
        assert_eq!(chunks[0].1, [0, 0, 0, 2, 0, 0, 0, 2, 8, 2, 0, 0, 0]);
        let raw = miniz_oxide::inflate::decompress_to_vec_zlib(&chunks[1].1).unwrap();
        assert_eq!(raw, [0, 255, 0, 0, 255, 0, 0, 0, 255, 0, 0, 255, 0, 0]);
        assert!(chunks[2].1.is_empty());
    }

    #[test]
    fn an_empty_screen_is_a_valid_png() {
        let s = Screen::new(0, 0, PixelFormat::RGB565).unwrap();
        let chunks = chunks(&encode(&s));
        assert_eq!(chunks[0].1[..8], [0; 8]);
        assert!(
            miniz_oxide::inflate::decompress_to_vec_zlib(&chunks[1].1)
                .unwrap()
                .is_empty()
        );
    }
}
