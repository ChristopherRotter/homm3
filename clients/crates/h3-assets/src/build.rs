//! Encoders for synthetic assets: the tests of the decoders and of the crates above them build
//! their sprites, images and palettes with these, since no original file may enter the repository
//! or CI (ADR-0009). They write what the decoders read; they are not tuned to match the original
//! encoder's choice of runs.

use alloc::vec::Vec;

use crate::def::{Encoding, Frame};
use crate::msk::Mask;
use crate::palette::{Chunk, PAL_VERSION, Palette, Rgb};
use crate::read::index;

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

/// One frame: its header (the short one if `short_header`) and its data in its encoding.
#[must_use]
pub fn frame(frame: &Frame) -> Vec<u8> {
    let (w, h) = (index(frame.width), index(frame.height));
    let rows: Vec<&[u8]> = (0..h)
        .map(|y| {
            let tail = frame.pixels.get(y * w..).unwrap_or_default();
            tail.get(..w).unwrap_or(tail)
        })
        .collect();
    let data = match frame.encoding {
        Encoding::Raw => frame.pixels.clone(),
        Encoding::Rows => with_table(&rows, 4, |row, out| {
            for run in runs(row, 256, |v| v != 0xFF) {
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
        Encoding::PackedBlocks => {
            let blocks: Vec<&[u8]> = frame.pixels.chunks(32).collect();
            with_table(&blocks, 2, packed)
        }
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

fn byte(value: usize) -> u8 {
    u8::try_from(value).unwrap_or(u8::MAX)
}

/// One-byte runs of encodings 2 and 3.
fn packed(row: &[u8], out: &mut Vec<u8>) {
    for run in runs(row, 32, |v| v < 7) {
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
fn with_table(parts: &[&[u8]], width: usize, encode: impl Fn(&[u8], &mut Vec<u8>)) -> Vec<u8> {
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

/// Splits `row` into runs of at most `max` pixels: a repeated index that `fill` accepts becomes a
/// fill run, anything else literal runs.
fn runs(row: &[u8], max: usize, fill: impl Fn(u8) -> bool) -> Vec<Run<'_>> {
    let mut out = Vec::new();
    let mut literal_start = 0;
    let mut i = 0;
    while let Some(&v) = row.get(i) {
        let same = row
            .get(i..)
            .unwrap_or_default()
            .iter()
            .take(max)
            .take_while(|&&b| b == v)
            .count();
        if fill(v) && same >= 2 {
            flush(row, literal_start, i, max, &mut out);
            out.push(Run::Fill(v, same));
            i += same;
            literal_start = i;
        } else {
            i += 1;
        }
    }
    flush(row, literal_start, row.len(), max, &mut out);
    out
}

fn flush<'a>(row: &'a [u8], from: usize, to: usize, max: usize, out: &mut Vec<Run<'a>>) {
    for chunk in row.get(from..to).unwrap_or_default().chunks(max) {
        out.push(Run::Literal(chunk));
    }
}
