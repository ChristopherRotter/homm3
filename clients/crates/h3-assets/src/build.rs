//! Encoders for synthetic assets: the tests of the decoders and of the crates above them build
//! their sprites, images and palettes with these, since no original file may enter the repository
//! or CI (ADR-0009). They write what the decoders read; they are not tuned to match the original
//! encoder's choice of runs.

use alloc::vec;
use alloc::vec::Vec;

use crate::def::{Encoding, Frame};
use crate::fnt::Font;
use crate::msk::Mask;
use crate::palette::{Chunk, PAL_VERSION, Palette, Rgb};
use crate::read::index;
use crate::wav::{IMA_ADPCM, decode_nibble, step_size};

fn put_u32(out: &mut Vec<u8>, value: u32) {
    out.extend_from_slice(&value.to_le_bytes());
}

/// A length or offset as the files' `u32`, saturating.
fn word(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}

/// A palette file with `palette`, `flags`, `padding` zero bytes after the entries inside the
/// `data` chunk, and `chunks` after it (each padded to an even length).
#[must_use]
pub fn pal(palette: &Palette, flags: &[u8; 256], padding: usize, chunks: &[Chunk]) -> Vec<u8> {
    let mut data = Vec::from(PAL_VERSION.to_le_bytes());
    data.extend_from_slice(&256u16.to_le_bytes());
    for (color, &flag) in palette.0.iter().zip(flags) {
        data.extend_from_slice(&[color.r, color.g, color.b, flag]);
    }
    data.resize(data.len() + padding, 0);
    let mut body = Vec::from(*b"PAL ");
    for (id, bytes) in
        core::iter::once((b"data", &data)).chain(chunks.iter().map(|c| (&c.id, &c.data)))
    {
        body.extend_from_slice(id);
        put_u32(&mut body, word(bytes.len()));
        body.extend_from_slice(bytes);
        body.resize(body.len() + (bytes.len() & 1), 0);
    }
    let mut out = Vec::from(*b"RIFF");
    put_u32(&mut out, word(body.len()));
    out.extend(body);
    out
}

/// An 8-bit PCX image.
#[must_use]
pub fn pcx_indexed(width: u32, height: u32, pixels: &[u8], palette: &Palette) -> Vec<u8> {
    let mut out = Vec::new();
    for v in [word(pixels.len()), width, height] {
        put_u32(&mut out, v);
    }
    out.extend_from_slice(pixels);
    out.extend_from_slice(&palette.to_triples());
    out
}

/// A 24-bit PCX image.
#[must_use]
pub fn pcx_rgb(width: u32, height: u32, pixels: &[Rgb]) -> Vec<u8> {
    let mut out = Vec::new();
    for v in [word(pixels.len() * 3), width, height] {
        put_u32(&mut out, v);
    }
    for p in pixels {
        out.extend_from_slice(&[p.b, p.g, p.r]);
    }
    out
}

/// An MSK file.
#[must_use]
pub fn msk(mask: &Mask) -> Vec<u8> {
    let mut out = Vec::from([mask.width, mask.height]);
    for m in &mask.masks {
        out.extend_from_slice(m);
    }
    out
}

/// A DEF: the header, `palette`, the groups (id; frames as a name and an index into `frames`), then
/// the frames encoded by [`frame`] in order.
#[must_use]
pub fn def(
    kind: u32,
    size: (u32, u32),
    palette: &Palette,
    groups: &[(u32, &[(&str, usize)])],
    frames: &[Frame],
) -> Vec<u8> {
    let mut out = Vec::new();
    for v in [kind, size.0, size.1, word(groups.len())] {
        put_u32(&mut out, v);
    }
    out.extend_from_slice(&palette.to_triples());
    let table: usize = groups.iter().map(|(_, f)| 16 + f.len() * 17).sum();
    let encoded: Vec<Vec<u8>> = frames.iter().map(frame).collect();
    let mut offsets = Vec::new();
    let mut at = out.len() + table;
    for e in &encoded {
        offsets.push(word(at));
        at += e.len();
    }
    for (id, refs) in groups {
        for v in [*id, word(refs.len()), 0, 0] {
            put_u32(&mut out, v);
        }
        for (name, _) in *refs {
            let mut field = [0u8; crate::def::NAME_BYTES];
            for (slot, b) in field.iter_mut().zip(name.bytes()) {
                *slot = b;
            }
            out.extend_from_slice(&field);
        }
        for (_, i) in *refs {
            put_u32(&mut out, offsets.get(*i).copied().unwrap_or(u32::MAX));
        }
    }
    for e in encoded {
        out.extend(e);
    }
    out
}

/// One frame: its header (the short one if `short_header`) and its data in its encoding. Pixels
/// marked in [`Frame::runs`] become runs where the encoding can repeat their index (encoding 1:
/// any index but `0xFF`; encodings 2 and 3: `0..=6`), the others literal bytes; encoding 0 has no
/// runs. Long runs are split at the encoding's limit.
#[must_use]
pub fn frame(frame: &Frame) -> Vec<u8> {
    let (w, h) = (index(frame.width), index(frame.height));
    // A part's pixels and run marks (empty past the end, as an invalid frame may be short).
    let part = |at: usize, len: usize| (cut(&frame.pixels, at, len), cut(&frame.runs, at, len));
    let rows: Vec<_> = (0..h).map(|y| part(y * w, w)).collect();
    let blocks: Vec<_> = (0..frame.pixels.len().div_ceil(32))
        .map(|b| part(b * 32, 32))
        .collect();
    let data = match frame.encoding {
        Encoding::Raw => frame.pixels.clone(),
        Encoding::Rows => with_table(&rows, 4, |(row, runs), out| {
            for run in segments(row, runs, 256, |v| v != 0xFF) {
                match run {
                    Run::Fill(v, n) => out.extend_from_slice(&[v, byte(n - 1)]),
                    Run::Literal(bytes) => {
                        out.extend_from_slice(&[0xFF, byte(bytes.len() - 1)]);
                        out.extend_from_slice(bytes);
                    }
                }
            }
        }),
        Encoding::PackedRows => with_table(&rows, 2, packed),
        Encoding::PackedBlocks => with_table(&blocks, 2, packed),
    };
    let mut out = Vec::new();
    let header: &[u32] = if frame.short_header {
        &[word(data.len()), 1, frame.full_width, frame.full_height]
    } else {
        &[
            word(data.len()),
            frame.encoding.number(),
            frame.full_width,
            frame.full_height,
            frame.width,
            frame.height,
            frame.left.cast_unsigned(),
            frame.top.cast_unsigned(),
        ]
    };
    for &v in header {
        put_u32(&mut out, v);
    }
    out.extend(data);
    out
}

/// Up to `len` items of `all` from `at` (empty past the end).
fn cut<T>(all: &[T], at: usize, len: usize) -> &[T] {
    let tail = all.get(at..).unwrap_or_default();
    tail.get(..len).unwrap_or(tail)
}

fn byte(value: usize) -> u8 {
    u8::try_from(value).unwrap_or(u8::MAX)
}

/// One-byte runs of encodings 2 and 3.
fn packed(&(row, runs): &(&[u8], &[bool]), out: &mut Vec<u8>) {
    for run in segments(row, runs, 32, |v| v < 7) {
        match run {
            Run::Fill(v, n) => out.push(v << 5 | byte(n - 1)),
            Run::Literal(bytes) => {
                out.push(7 << 5 | byte(bytes.len() - 1));
                out.extend_from_slice(bytes);
            }
        }
    }
}

/// A table of offsets (`width` bytes each, little-endian) to each part's encoding, then the parts.
fn with_table<T>(parts: &[T], width: usize, encode: impl Fn(&T, &mut Vec<u8>)) -> Vec<u8> {
    let mut body = Vec::new();
    let mut table = Vec::new();
    for part in parts {
        let offset = (parts.len() * width + body.len()).to_le_bytes();
        table.extend_from_slice(offset.get(..width).unwrap_or_default());
        encode(part, &mut body);
    }
    table.extend(body);
    table
}

enum Run<'a> {
    Fill(u8, usize),
    Literal(&'a [u8]),
}

/// Splits `row` into runs of at most `max` pixels: consecutive pixels that `runs` marks, with
/// one index that `fill` accepts, become fill runs; anything else literal runs.
fn segments<'a>(
    row: &'a [u8],
    runs: &[bool],
    max: usize,
    fill: impl Fn(u8) -> bool,
) -> Vec<Run<'a>> {
    let fills = |i: usize| {
        row.get(i)
            .copied()
            .filter(|&v| runs.get(i) == Some(&true) && fill(v))
    };
    let mut out = Vec::new();
    let mut i = 0;
    while i < row.len() {
        let start = i;
        if let Some(v) = fills(i) {
            while i < row.len() && i - start < max && fills(i) == Some(v) {
                i += 1;
            }
            out.push(Run::Fill(v, i - start));
        } else {
            while i < row.len() && i - start < max && fills(i).is_none() {
                i += 1;
            }
            out.push(Run::Literal(row.get(start..i).unwrap_or_default()));
        }
    }
    out
}

/// The run marks an encoder that repeats every index it can, where it repeats at least twice,
/// would give `pixels` (`width` wide) in `encoding`: a convenient [`Frame::runs`] for synthetic
/// frames. Runs end at a row's end (encodings 1, 2) or a 32-pixel block's (3).
#[must_use]
pub fn runs_by_repetition(encoding: Encoding, width: u32, pixels: &[u8]) -> Vec<bool> {
    let (part, fill): (usize, fn(u8) -> bool) = match encoding {
        Encoding::Raw => return vec![false; pixels.len()],
        Encoding::Rows => (index(width), |v| v != 0xFF),
        Encoding::PackedRows => (index(width), |v| v < 7),
        Encoding::PackedBlocks => (32, |v| v < 7),
    };
    let mut out = Vec::with_capacity(pixels.len());
    for row in pixels.chunks(part.max(1)) {
        let mut i = 0;
        while let Some(&v) = row.get(i) {
            let same = row.iter().skip(i).take_while(|&&b| b == v).count();
            let marked = fill(v) && same >= 2;
            out.resize(out.len() + same, marked);
            i += same;
        }
    }
    out
}

/// A font file (`.FNT`, E17 M3 T4a): the header (the game's first bytes, the height and the y
/// offset), the 256 glyphs' metrics (a font with fewer gets empty ones), their offsets, then
/// each glyph's pixels in order.
#[must_use]
pub fn fnt(font: &Font) -> Vec<u8> {
    let mut out = Vec::from([0x1F, 0xFF, 0x08, 0, 0, font.height]);
    out.extend_from_slice(&font.y_offset.to_le_bytes());
    out.resize(32, 0);
    let glyph = |c: usize| font.glyphs.get(c).cloned().unwrap_or_default();
    for c in 0..256 {
        let g = glyph(c);
        for v in [g.left, g.width, g.right] {
            out.extend_from_slice(&v.to_le_bytes());
        }
    }
    let mut offset = 0;
    for c in 0..256 {
        put_u32(&mut out, word(offset));
        offset += glyph(c).pixels.len();
    }
    for c in 0..256 {
        out.extend_from_slice(&glyph(c).pixels);
    }
    out
}

/// A sound archive (`.snd`) of `entries`, each a stem and its bytes: the directory, then the
/// bytes in the same order. A stem of 40 bytes or more fills the name field without a NUL.
#[must_use]
pub fn snd(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let mut out = Vec::new();
    put_u32(&mut out, word(entries.len()));
    let mut offset = 4 + entries.len() * 48;
    for (name, bytes) in entries {
        let mut field = [0u8; 40];
        for (f, b) in field.iter_mut().zip(name.bytes()) {
            *f = b;
        }
        out.extend_from_slice(&field);
        put_u32(&mut out, word(offset));
        put_u32(&mut out, word(bytes.len()));
        offset += bytes.len();
    }
    for (_, bytes) in entries {
        out.extend_from_slice(bytes);
    }
    out
}

/// A RIFF file of form `WAVE` holding `chunks` in order, each padded to an even length.
#[must_use]
pub fn riff_wave(chunks: &[(&[u8; 4], &[u8])]) -> Vec<u8> {
    let mut body = Vec::from(*b"WAVE");
    for (id, bytes) in chunks {
        body.extend_from_slice(*id);
        put_u32(&mut body, word(bytes.len()));
        body.extend_from_slice(bytes);
        body.resize(body.len() + (bytes.len() & 1), 0);
    }
    let mut out = Vec::from(*b"RIFF");
    put_u32(&mut out, word(body.len()));
    out.extend_from_slice(&body);
    out
}

/// An IMA ADPCM `fmt ` chunk's data: `rate`, `channels`, `block` bytes per block, 4 bits; with
/// `frames_per_block`, the extension's two bytes of `wSamplesPerBlock` (20 bytes), else an empty
/// extension (18 bytes).
#[must_use]
pub fn ima_fmt(rate: u32, channels: u16, block: u16, frames_per_block: Option<u16>) -> Vec<u8> {
    let mut out = Vec::from(IMA_ADPCM.to_le_bytes());
    out.extend_from_slice(&channels.to_le_bytes());
    put_u32(&mut out, rate);
    put_u32(&mut out, rate / 2 * u32::from(channels));
    out.extend_from_slice(&block.to_le_bytes());
    out.extend_from_slice(&4u16.to_le_bytes());
    match frames_per_block {
        Some(frames) => {
            out.extend_from_slice(&2u16.to_le_bytes());
            out.extend_from_slice(&frames.to_le_bytes());
        }
        None => out.extend_from_slice(&0u16.to_le_bytes()),
    }
    out
}

/// IMA ADPCM data of `samples` (interleaved frames of `channels`, at least one) in blocks of
/// `block` bytes (at least 4 per channel), each with as many frames as fit, `1 + (block - 4 x
/// channels) x 2 / channels`; the last block ends after the words its frames need (a last word's
/// unused nibbles 0). A block's header takes its first frame and the step index the encoding
/// reached (0 at the start); each later nibble is the usual choice, the one whose step comes
/// closest. Returns the bytes and the samples a decoder gives back.
#[must_use]
pub fn ima_data(channels: u16, block: u16, samples: &[i16]) -> (Vec<u8>, Vec<i16>) {
    let channels = usize::from(channels.max(1));
    let per_block = 1 + usize::from(block).saturating_sub(4 * channels) * 2 / channels;
    let frames: Vec<&[i16]> = samples.chunks(channels).collect();
    let mut bytes = Vec::new();
    let mut decoded = Vec::with_capacity(samples.len());
    let mut indices = vec![0i8; channels];
    for (first, rest) in frames.chunks(per_block).filter_map(<[_]>::split_first) {
        let mut previous = first.to_vec();
        for (&sample, &step) in previous.iter().zip(&indices) {
            bytes.extend_from_slice(&sample.to_le_bytes());
            bytes.extend_from_slice(&[step.to_le_bytes()[0], 0]);
        }
        decoded.extend_from_slice(first);
        for group in rest.chunks(8) {
            let start = decoded.len();
            decoded.resize(start + group.len() * channels, 0);
            for (c, (last, step)) in previous.iter_mut().zip(&mut indices).enumerate() {
                let mut nibbles = [0u8; 8];
                let outputs = decoded.iter_mut().skip(start + c).step_by(channels);
                for ((frame, out), nibble) in group.iter().zip(outputs).zip(&mut nibbles) {
                    let target = frame.get(c).copied().unwrap_or_default();
                    *nibble = encode_nibble(*last, *step, target);
                    *last = decode_nibble(*last, step, *nibble);
                    *out = *last;
                }
                let pairs = nibbles.as_chunks::<2>().0;
                bytes.extend(pairs.iter().map(|&[low, high]| low | high << 4));
            }
        }
    }
    (bytes, decoded)
}

/// The nibble that moves `previous` closest to `target` at `step_index`'s step.
fn encode_nibble(previous: i16, step_index: i8, target: i16) -> u8 {
    let mut step = step_size(step_index);
    let mut difference = i32::from(target) - i32::from(previous);
    let mut nibble = 0;
    if difference < 0 {
        nibble = 8;
        difference = -difference;
    }
    for bit in [4, 2, 1] {
        if difference >= step {
            nibble |= bit;
            difference -= step;
        }
        step >>= 1;
    }
    nibble
}
