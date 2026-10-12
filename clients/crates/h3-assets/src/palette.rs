//! Palettes: the 256 colours embedded in PCX images and DEF sprites (768 bytes of RGB triples), and
//! the palette files (`*.PAL`, RIFF palettes) of `H3bitmap.lod` (E17 M1 T2).
//!
//! A palette holds the colours as the files store them, 8 bits per channel. Which indices the game
//! replaces when it draws (transparency, shadow, player colours) and how a colour reaches the screen
//! are the compositor's (M1 T1, T6).

use alloc::vec::Vec;

use crate::Error;
use crate::read::{array, index, slice, u16_at, u32_at};

/// The size of an embedded palette: 256 RGB triples.
pub const PALETTE_BYTES: usize = 768;

/// One colour, 8 bits per channel.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Rgb {
    /// Red.
    pub r: u8,
    /// Green.
    pub g: u8,
    /// Blue.
    pub b: u8,
}

/// A palette of 256 colours, indexed by a pixel's byte.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Palette(pub [Rgb; 256]);

impl Default for Palette {
    fn default() -> Self {
        Self([Rgb::default(); 256])
    }
}

impl Palette {
    /// The palette from 256 RGB triples, as PCX images and DEF sprites embed theirs.
    #[must_use]
    pub fn from_triples(bytes: &[u8; PALETTE_BYTES]) -> Self {
        let mut colors = [Rgb::default(); 256];
        for (color, &[r, g, b]) in colors.iter_mut().zip(bytes.as_chunks::<3>().0) {
            *color = Rgb { r, g, b };
        }
        Self(colors)
    }

    /// The 768 bytes [`Palette::from_triples`] reads.
    #[must_use]
    pub fn to_triples(&self) -> [u8; PALETTE_BYTES] {
        let mut bytes = [0; PALETTE_BYTES];
        for (&color, triple) in self.0.iter().zip(bytes.as_chunks_mut::<3>().0) {
            *triple = [color.r, color.g, color.b];
        }
        bytes
    }

    /// The colour of a pixel's index.
    #[must_use]
    pub fn color(&self, index: u8) -> Rgb {
        self.0.get(usize::from(index)).copied().unwrap_or_default()
    }
}

/// A palette file: a RIFF file of form `PAL ` whose `data` chunk holds a Windows `LOGPALETTE`
/// (version `0x300`, 256 entries of red, green, blue and a flags byte).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PalFile {
    /// The colours.
    pub palette: Palette,
    /// Each entry's flags byte (`PALETTEENTRY::peFlags`), as stored.
    pub flags: [u8; 256],
    /// The other chunks in file order (the game's `game.pal` and `PLAYERS.PAL` carry `offl`,
    /// `tran` and `unde`, 32 bytes each; their use is not known).
    pub chunks: Vec<Chunk>,
}

/// A RIFF chunk the decoder keeps but does not interpret.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chunk {
    /// The chunk's four-character identifier.
    pub id: [u8; 4],
    /// Its bytes.
    pub data: Vec<u8>,
}

/// The `LOGPALETTE` version the game's files carry.
pub const PAL_VERSION: u16 = 0x300;

/// Decodes a palette file. The RIFF size must match the file's length and every chunk (with its
/// pad byte) must fit; the `data` chunk must come exactly once, with version [`PAL_VERSION`] and
/// 256 entries. Bytes after the entries inside the `data` chunk are ignored (`font.pal` has four).
pub fn decode_pal(bytes: &[u8]) -> Result<PalFile, Error> {
    let riff_size = u32_at(bytes, 4).ok_or(Error::Truncated)?;
    if array::<4>(bytes, 0) != Some(*b"RIFF") || array::<4>(bytes, 8) != Some(*b"PAL ") {
        return Err(Error::Magic);
    }
    if index(riff_size).saturating_add(8) != bytes.len() {
        return Err(Error::Field("riff size"));
    }
    let mut colors = None;
    let mut chunks = Vec::new();
    let mut at = 12;
    while at < bytes.len() {
        let id = array::<4>(bytes, at).ok_or(Error::Truncated)?;
        let size = index(u32_at(bytes, at + 4).ok_or(Error::Truncated)?);
        let data = slice(bytes, at + 8, size).ok_or(Error::Truncated)?;
        at = (at + 8 + data.len()).saturating_add(size & 1);
        if at > bytes.len() {
            return Err(Error::Truncated);
        }
        if &id == b"data" {
            if colors.is_some() {
                return Err(Error::Field("data"));
            }
            colors = Some(log_palette(data)?);
        } else {
            chunks.push(Chunk {
                id,
                data: data.to_vec(),
            });
        }
    }
    let (palette, flags) = colors.ok_or(Error::Field("data"))?;
    Ok(PalFile {
        palette,
        flags,
        chunks,
    })
}

/// A `LOGPALETTE`: version, count, then the entries.
fn log_palette(data: &[u8]) -> Result<(Palette, [u8; 256]), Error> {
    let count = u16_at(data, 2).ok_or(Error::Truncated)?;
    if u16_at(data, 0) != Some(PAL_VERSION) {
        return Err(Error::Field("version"));
    }
    if count != 256 {
        return Err(Error::Field("count"));
    }
    let entries = array::<1024>(data, 4).ok_or(Error::Truncated)?;
    let mut palette = Palette::default();
    let mut flags = [0; 256];
    for ((color, flag), &[r, g, b, f]) in palette
        .0
        .iter_mut()
        .zip(flags.iter_mut())
        .zip(entries.as_chunks::<4>().0)
    {
        *color = Rgb { r, g, b };
        *flag = f;
    }
    Ok((palette, flags))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build;
    use alloc::vec;
    use proptest::prelude::*;

    fn ramp() -> Palette {
        let mut p = Palette::default();
        for (i, c) in p.0.iter_mut().enumerate() {
            let v = u8::try_from(i).unwrap();
            *c = Rgb {
                r: v,
                g: v ^ 0x55,
                b: 255 - v,
            };
        }
        p
    }

    #[test]
    fn triples_round_trip() {
        let p = ramp();
        let bytes = p.to_triples();
        assert_eq!(&bytes[..6], &[0, 0x55, 255, 1, 0x54, 254]);
        assert_eq!(Palette::from_triples(&bytes), p);
        assert_eq!(
            p.color(2),
            Rgb {
                r: 2,
                g: 0x57,
                b: 253
            }
        );
    }

    #[test]
    fn a_pal_file_round_trips_with_its_chunks() {
        let mut flags = [0; 256];
        flags[7] = 4;
        let chunks = vec![
            Chunk {
                id: *b"offl",
                data: vec![1; 32],
            },
            Chunk {
                id: *b"odd ",
                data: vec![2; 3],
            },
        ];
        let bytes = build::pal(&ramp(), &flags, 0, &chunks);
        let pal = decode_pal(&bytes).unwrap();
        assert_eq!(
            pal,
            PalFile {
                palette: ramp(),
                flags,
                chunks
            }
        );
    }

    #[test]
    fn bytes_after_the_entries_are_ignored() {
        let bytes = build::pal(&ramp(), &[0; 256], 4, &[]);
        assert_eq!(decode_pal(&bytes).unwrap().palette, ramp());
    }

    #[test]
    fn malformed_files_are_rejected() {
        let good = build::pal(&ramp(), &[0; 256], 0, &[]);
        assert_eq!(decode_pal(&good[..7]), Err(Error::Truncated));
        let mut bad = good.clone();
        bad[0] = b'X';
        assert_eq!(decode_pal(&bad), Err(Error::Magic));
        let mut bad = good.clone();
        bad[8] = b'X';
        assert_eq!(decode_pal(&bad), Err(Error::Magic));
        let mut bad = good.clone();
        bad[4] ^= 1;
        assert_eq!(decode_pal(&bad), Err(Error::Field("riff size")));
        // A chunk header cut short, a chunk longer than the file, a missing pad byte.
        for tail in [
            &b"ab"[..],
            b"abcd\x01\0",
            b"abcd\x09\0\0\0x",
            b"abcd\x01\0\0\0x",
        ] {
            let mut bad = build::pal(&ramp(), &[0; 256], 0, &[]);
            bad.extend_from_slice(tail);
            let size = u32::try_from(bad.len() - 8).unwrap();
            bad[4..8].copy_from_slice(&size.to_le_bytes());
            assert_eq!(decode_pal(&bad), Err(Error::Truncated), "{tail:?}");
        }
    }

    #[test]
    fn the_data_chunk_is_checked() {
        let riff = |chunks: &[(&[u8; 4], Vec<u8>)]| {
            let mut body = b"PAL ".to_vec();
            for (id, data) in chunks {
                body.extend_from_slice(*id);
                body.extend_from_slice(&u32::try_from(data.len()).unwrap().to_le_bytes());
                body.extend_from_slice(data);
            }
            let mut out = b"RIFF".to_vec();
            out.extend_from_slice(&u32::try_from(body.len()).unwrap().to_le_bytes());
            out.extend(body);
            out
        };
        let entries = |version: u16, count: u16, n: usize| {
            let mut d = version.to_le_bytes().to_vec();
            d.extend_from_slice(&count.to_le_bytes());
            d.resize(4 + n, 0);
            d
        };
        let ok = entries(PAL_VERSION, 256, 1024);
        assert!(decode_pal(&riff(&[(b"data", ok.clone())])).is_ok());
        assert_eq!(decode_pal(&riff(&[])), Err(Error::Field("data")));
        assert_eq!(
            decode_pal(&riff(&[(b"data", ok.clone()), (b"data", ok)])),
            Err(Error::Field("data"))
        );
        assert_eq!(
            decode_pal(&riff(&[(b"data", entries(0x200, 256, 1024))])),
            Err(Error::Field("version"))
        );
        assert_eq!(
            decode_pal(&riff(&[(b"data", entries(PAL_VERSION, 16, 64))])),
            Err(Error::Field("count"))
        );
        assert_eq!(
            decode_pal(&riff(&[(b"data", entries(PAL_VERSION, 256, 1020))])),
            Err(Error::Truncated)
        );
        assert_eq!(
            decode_pal(&riff(&[(b"data", vec![0, 3])])),
            Err(Error::Truncated)
        );
    }

    proptest! {
        #[test]
        fn decode_pal_never_panics(bytes in proptest::collection::vec(any::<u8>(), 0..1200)) {
            let _ = decode_pal(&bytes);
        }

        #[test]
        fn a_mutated_pal_file_never_panics(at in 0usize..1100, value: u8) {
            let mut bytes = build::pal(&ramp(), &[0; 256], 0, &[]);
            if let Some(b) = bytes.get_mut(at) {
                *b = value;
            }
            let _ = decode_pal(&bytes);
        }
    }
}
