//! DEF sprites (E17 M1 T3): a type, the size of the canvas, a palette, the animation groups (each a
//! list of named frames) and the frames, each in one of four encodings and placed inside the
//! canvas by its own offset.
//!
//! The layout, from the community's documentation of the format and checked against the user's
//! files by the real-asset gate (every DEF of the four LODs decodes): a header (`type`, `width`,
//! `height`, the number of groups: `u32`s), the palette (768 bytes), then per group its id, its
//! frame count `n`, eight bytes the game does not need (in the files: leftovers of pointers), `n`
//! frame names of 13 bytes (up to the first NUL; garbage may follow it) and `n` frame offsets
//! (`u32`, from the start of the file). Several groups may name the same frame (the same offset).
//!
//! A frame: a 32-byte header (`size` of the data after the header, the encoding, the canvas's
//! width and height, the frame's width and height, its left and top offset in the canvas; the
//! offsets signed), then `size` bytes of data. The four encodings ([`Encoding`]) store rows of
//! palette indices; the run-length ones start with a table of offsets (relative to the data's
//! start) into their runs.
//!
//! **The short header.** Two spell sprites of `H3sprite.lod`, `SGTWMTA.def` and `SGTWMTB.def`,
//! have frames with a 16-byte header: `size`, the encoding (always 1), the canvas's width and
//! height; the frame fills the canvas. Read as a full header, its width and height are the first
//! two row offsets, larger than the canvas. A frame of encoding 1 whose width and height both
//! exceed the canvas's is read with the short header (origin: VCMI 1.7.5,
//! `client/render/CDefFile.cpp`, `CDefFile::loadFrame`, the "old format defs" case). The game
//! would read a 32-byte header there, and nothing names the two files (`docs/epics/E17-design.md`
//! §3.4).

use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use crate::Error;
use crate::palette::{PALETTE_BYTES, Palette};
use crate::read::{array, index, slice, u8_at, u16_at, u32_at, u32s};

/// The longest side of a frame the decoder accepts (*provisional*: the game's largest is 800).
/// It bounds the decoder's loops; memory is bounded by the input, since every pixel needs input.
pub const MAX_SIDE: u32 = 1 << 14;

/// The length of a frame name in the group table.
pub const NAME_BYTES: usize = 13;

/// The size of a frame's header.
pub const FRAME_HEADER_BYTES: usize = 32;

/// The size of the short frame header (see the module documentation).
pub const SHORT_FRAME_HEADER_BYTES: usize = 16;

/// The size of the file header before the palette.
const HEADER_BYTES: usize = 16;

/// A parsed DEF: its header, palette and group table. The frames are decoded on demand
/// ([`decode_frame`] with an offset from [`Def::offsets`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Def {
    /// The type (the files use `0x40..=0x49`: spell, sprite, creature, map object, map hero,
    /// terrain, cursor, interface, sprite frame, battle hero, in VCMI's names).
    pub kind: u32,
    /// The canvas's width.
    pub width: u32,
    /// The canvas's height.
    pub height: u32,
    /// The palette.
    pub palette: Palette,
    /// The groups in file order.
    pub groups: Vec<Group>,
}

/// An animation group.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Group {
    /// The group's id (what an animation asks for: standing, moving, …).
    pub id: u32,
    /// Its frames in order.
    pub frames: Vec<FrameRef>,
}

/// A frame as the group table names it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrameRef {
    /// The frame's name: the bytes before the first NUL, each byte one character.
    pub name: String,
    /// Where its header starts, from the start of the file.
    pub offset: u32,
}

impl Def {
    /// Every frame offset the groups name, once each, ascending.
    #[must_use]
    pub fn offsets(&self) -> BTreeSet<u32> {
        self.groups
            .iter()
            .flat_map(|g| g.frames.iter().map(|f| f.offset))
            .collect()
    }
}

/// How a frame's rows are stored.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Encoding {
    /// 0: the rows' indices as they are.
    Raw,
    /// 1: a `u32` offset per row; each row a list of runs `(index, length − 1)` of one byte each,
    /// index `0xFF` meaning that `length` literal indices follow.
    Rows,
    /// 2: a `u16` offset per row; each row a list of one-byte runs, the top three bits an index
    /// (`0..=6`) repeated `length` times or `7` for `length` literal indices following, the low
    /// five bits `length − 1`.
    PackedRows,
    /// 3: as [`Encoding::PackedRows`], but a `u16` offset per 32 pixels of each row (the width a
    /// multiple of 32); each block's runs end at the block's end.
    PackedBlocks,
}

impl Encoding {
    /// The encoding's number in the frame header.
    #[must_use]
    pub fn number(self) -> u32 {
        match self {
            Self::Raw => 0,
            Self::Rows => 1,
            Self::PackedRows => 2,
            Self::PackedBlocks => 3,
        }
    }

    fn from_number(number: u32) -> Result<Self, Error> {
        match number {
            0 => Ok(Self::Raw),
            1 => Ok(Self::Rows),
            2 => Ok(Self::PackedRows),
            3 => Ok(Self::PackedBlocks),
            _ => Err(Error::Field("encoding")),
        }
    }
}

/// A decoded frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame {
    /// How it was stored.
    pub encoding: Encoding,
    /// The canvas's width, as the frame header gives it.
    pub full_width: u32,
    /// The canvas's height, as the frame header gives it.
    pub full_height: u32,
    /// The frame's left edge in the canvas.
    pub left: i32,
    /// The frame's top edge in the canvas.
    pub top: i32,
    /// The frame's width.
    pub width: u32,
    /// The frame's height.
    pub height: u32,
    /// `width × height` palette indices, row-major, top row first.
    pub pixels: Vec<u8>,
    /// Per pixel of [`Frame::pixels`], whether it came from a run (an index repeated by the
    /// encoding) rather than from literal bytes; encoding 0 has no runs. The game's drawers treat
    /// the two apart: a run of a keyed index (1–7) is a shadow or an outline, a literal one a
    /// colour, and some drawers skip runs of any other index (`docs/epics/E17-design.md` §3.3).
    pub runs: Vec<bool>,
    /// Read with the short header (module documentation).
    pub short_header: bool,
}

/// Parses a DEF's header, palette and group table. Frame offsets are not checked here.
pub fn parse_def(bytes: &[u8]) -> Result<Def, Error> {
    let [kind, width, height, count] = u32s(bytes, 0).ok_or(Error::Truncated)?;
    let palette = array::<PALETTE_BYTES>(bytes, HEADER_BYTES).ok_or(Error::Truncated)?;
    let mut at = HEADER_BYTES + PALETTE_BYTES;
    let mut groups = Vec::new();
    for _ in 0..count {
        let [id, frames, _, _] = u32s(bytes, at).ok_or(Error::Truncated)?;
        let n = index(frames);
        let names = slice(bytes, at + 16, n.saturating_mul(NAME_BYTES)).ok_or(Error::Truncated)?;
        let offsets_at = at + 16 + names.len();
        let offsets = slice(bytes, offsets_at, n.saturating_mul(4)).ok_or(Error::Truncated)?;
        let frames = names
            .as_chunks::<NAME_BYTES>()
            .0
            .iter()
            .zip(offsets.as_chunks::<4>().0)
            .map(|(name, &offset)| FrameRef {
                name: name
                    .iter()
                    .take_while(|&&b| b != 0)
                    .map(|&b| char::from(b))
                    .collect(),
                offset: u32::from_le_bytes(offset),
            })
            .collect();
        groups.push(Group { id, frames });
        at = offsets_at + offsets.len();
    }
    Ok(Def {
        kind,
        width,
        height,
        palette: Palette::from_triples(&palette),
        groups,
    })
}

/// Decodes the frame whose header starts at `offset`. Every read stays inside the frame's data
/// (`size` bytes after its header); a run that goes past its row's (or block's) end is
/// [`Error::Overrun`].
pub fn decode_frame(bytes: &[u8], offset: u32) -> Result<Frame, Error> {
    let at = index(offset);
    let [
        size,
        number,
        full_width,
        full_height,
        width,
        height,
        left,
        top,
    ] = u32s(bytes, at).ok_or(Error::Truncated)?;
    let encoding = Encoding::from_number(number)?;
    let short_header = encoding == Encoding::Rows && width > full_width && height > full_height;
    let (header, width, height, left, top) = if short_header {
        (SHORT_FRAME_HEADER_BYTES, full_width, full_height, 0, 0)
    } else {
        (
            FRAME_HEADER_BYTES,
            width,
            height,
            left.cast_signed(),
            top.cast_signed(),
        )
    };
    if width > MAX_SIDE || height > MAX_SIDE {
        return Err(Error::TooLarge);
    }
    let data = slice(bytes, at.saturating_add(header), index(size)).ok_or(Error::Truncated)?;
    let (w, h) = (index(width), index(height));
    let Pixels { pixels, runs } = match encoding {
        Encoding::Raw => {
            let pixels = data.get(..w * h).ok_or(Error::Truncated)?.to_vec();
            let runs = vec![false; pixels.len()];
            Pixels { pixels, runs }
        }
        Encoding::Rows => rows(data, w, h)?,
        Encoding::PackedRows => packed_rows(data, w, h)?,
        Encoding::PackedBlocks => packed_blocks(data, w, h)?,
    };
    Ok(Frame {
        encoding,
        full_width,
        full_height,
        left,
        top,
        width,
        height,
        pixels,
        runs,
        short_header,
    })
}

/// A frame's pixels as the decoders produce them.
struct Pixels {
    pixels: Vec<u8>,
    runs: Vec<bool>,
}

impl Pixels {
    fn with_capacity(n: usize) -> Self {
        Self {
            pixels: Vec::with_capacity(n),
            runs: Vec::with_capacity(n),
        }
    }

    fn literal(&mut self, bytes: &[u8]) {
        self.pixels.extend_from_slice(bytes);
        self.runs.resize(self.pixels.len(), false);
    }

    fn run(&mut self, index: u8, length: usize) {
        self.pixels.resize(self.pixels.len() + length, index);
        self.runs.resize(self.pixels.len(), true);
    }
}

/// Encoding 1.
fn rows(data: &[u8], w: usize, h: usize) -> Result<Pixels, Error> {
    let mut out = Pixels::with_capacity(w * h);
    for y in 0..h {
        let mut at = index(u32_at(data, 4 * y).ok_or(Error::Truncated)?);
        let mut filled = 0;
        while filled < w {
            let [code, length] = array::<2>(data, at).ok_or(Error::Truncated)?;
            let length = usize::from(length) + 1;
            at += 2;
            if filled + length > w {
                return Err(Error::Overrun);
            }
            if code == 0xFF {
                out.literal(slice(data, at, length).ok_or(Error::Truncated)?);
                at += length;
            } else {
                out.run(code, length);
            }
            filled += length;
        }
    }
    Ok(out)
}

/// Encoding 2.
fn packed_rows(data: &[u8], w: usize, h: usize) -> Result<Pixels, Error> {
    let mut out = Pixels::with_capacity(w * h);
    for y in 0..h {
        let at = u16_at(data, 2 * y).ok_or(Error::Truncated)?;
        packed(data, usize::from(at), w, &mut out)?;
    }
    Ok(out)
}

/// Encoding 3.
fn packed_blocks(data: &[u8], w: usize, h: usize) -> Result<Pixels, Error> {
    if !w.is_multiple_of(32) {
        return Err(Error::Field("width"));
    }
    let blocks = w / 32;
    let mut out = Pixels::with_capacity(w * h);
    for block in 0..h * blocks {
        let at = u16_at(data, 2 * block).ok_or(Error::Truncated)?;
        packed(data, usize::from(at), 32, &mut out)?;
    }
    Ok(out)
}

/// `count` pixels of one-byte runs from `at`.
fn packed(data: &[u8], mut at: usize, count: usize, out: &mut Pixels) -> Result<(), Error> {
    let mut filled = 0;
    while filled < count {
        let run = u8_at(data, at).ok_or(Error::Truncated)?;
        at += 1;
        let (code, length) = (run >> 5, usize::from(run & 0x1F) + 1);
        if filled + length > count {
            return Err(Error::Overrun);
        }
        if code == 7 {
            out.literal(slice(data, at, length).ok_or(Error::Truncated)?);
            at += length;
        } else {
            out.run(code, length);
        }
        filled += length;
    }
    Ok(())
}

#[cfg(test)]
mod tests;
