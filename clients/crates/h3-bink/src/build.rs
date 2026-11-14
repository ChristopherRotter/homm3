//! Synthetic Bink files for tests (the writer's side of [`crate::Clip`] and
//! [`crate::Decoder`]; *provisional*, E17 M5 T4b2a): a bit writer, a file around given frames, a
//! video packet of given blocks per plane (its bundles refilled exactly when the decoder refills
//! them, a DCT block's and a residue's coefficients coded as the decoder reads them), and
//! [`sample`], a clip of every block type in random variations, whose decoding FFmpeg's checks
//! (`h3-view video --sample`, [`crate::check`]).
//!
//! The writer does not validate: a test gets what it asks for (a run past the block, a type the
//! decoder rejects), so that the decoder's checks can be tested.

use alloc::vec;
use alloc::vec::Vec;

use crate::audio::Track;
use crate::x87::X;

use crate::tables::RUN_BITS;
use crate::video::{
    BLOCK_TYPES, BUNDLES, COLORS, COUNT_BITS, DCT_LISTS, INTER_COEFS, INTER_DC, INTER_Q, INTRA_DC,
    INTRA_Q, Lists, PATTERN, RESIDUE_LISTS, SIGNED, WIDTH, X_OFF, Y_OFF,
};

/// Bits written lowest first into each byte, as the decoder reads them.
#[derive(Debug, Clone, Default)]
pub struct BitWriter {
    bytes: Vec<u8>,
    bits: usize,
}

impl BitWriter {
    /// An empty writer.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// The low `n` bits of `value` (`n` at most 32), the lowest first.
    pub fn put(&mut self, value: u32, n: u32) {
        for k in 0..n {
            if self.bits.is_multiple_of(8) {
                self.bytes.push(0);
            }
            if (value >> k) & 1 == 1
                && let Some(last) = self.bytes.last_mut()
            {
                *last |= 1 << (self.bits % 8);
            }
            self.bits += 1;
        }
    }

    /// One flag.
    pub fn flag(&mut self, flag: bool) {
        self.put(u32::from(flag), 1);
    }

    /// Every bit `other` holds, after these.
    pub fn append(&mut self, other: &Self) {
        for k in 0..other.bits {
            let byte = other.bytes.get(k / 8).copied().unwrap_or(0);
            self.put(u32::from(byte >> (k % 8)), 1);
        }
    }

    /// Zeros up to the next multiple of 32 bits.
    pub fn align32(&mut self) {
        while !self.bits.is_multiple_of(32) {
            self.put(0, 1);
        }
    }

    /// How many bits were written.
    #[must_use]
    pub fn len(&self) -> usize {
        self.bits
    }

    /// Whether nothing was written.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.bits == 0
    }

    /// The bytes, the last one's unused high bits 0.
    #[must_use]
    pub fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }
}

/// A file's header fields.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Header {
    /// The signature's fourth byte.
    pub revision: u8,
    /// The frame's width.
    pub width: u32,
    /// The frame's height.
    pub height: u32,
    /// The frame rate's numerator.
    pub rate_num: u32,
    /// The frame rate's denominator.
    pub rate_den: u32,
    /// The video's flags.
    pub flags: u32,
    /// How many audio tracks the header describes (each frame's packets are the caller's).
    pub audio_tracks: u32,
    /// The tracks' descriptions (E27 M2 T2), in order; a track past them described by zeros.
    pub tracks: Vec<Track>,
}

impl Header {
    /// A clip of revision `b` at 15 frames a second, no flags, no audio, as the game's are.
    #[must_use]
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            revision: b'b',
            width,
            height,
            rate_num: 15,
            rate_den: 1,
            flags: 0,
            audio_tracks: 0,
            tracks: Vec::new(),
        }
    }
}

fn word(value: usize) -> [u8; 4] {
    u32::try_from(value).unwrap_or(u32::MAX).to_le_bytes()
}

/// A Bink file of `frames` (each a frame's whole bytes, padded with a 0 to an even size as the
/// files' are): the header, the audio tracks' descriptions (each track's largest decoded size,
/// its rate and flags, its id: its index), the index of `frames.len()` + 1 offsets (the first
/// marked a key frame, the last the file's end), the frames.
#[must_use]
pub fn file(header: &Header, frames: &[Vec<u8>]) -> Vec<u8> {
    let tracks = usize::try_from(header.audio_tracks).unwrap_or(0);
    let extra = if header.revision == b'k' { 4 } else { 0 };
    let first = 44 + extra + 12 * tracks + 4 * (frames.len() + 1);
    let sizes: Vec<usize> = frames.iter().map(|f| f.len() + f.len() % 2).collect();
    let size = first + sizes.iter().sum::<usize>();
    let mut out = Vec::with_capacity(size);
    out.extend_from_slice(b"BIK");
    out.push(header.revision);
    out.extend_from_slice(&word(size.saturating_sub(8)));
    out.extend_from_slice(&word(frames.len()));
    out.extend_from_slice(&word(sizes.iter().copied().max().unwrap_or(0)));
    out.extend_from_slice(&word(frames.len()));
    for value in [
        header.width,
        header.height,
        header.rate_num,
        header.rate_den,
        header.flags,
        header.audio_tracks,
    ] {
        out.extend_from_slice(&value.to_le_bytes());
    }
    out.resize(out.len() + extra, 0);
    let track = |k: usize| header.tracks.get(k).copied().unwrap_or_default();
    for k in 0..tracks {
        out.extend_from_slice(&track(k).largest.to_le_bytes());
    }
    for k in 0..tracks {
        let t = track(k);
        out.extend_from_slice(&(u32::from(t.rate) | u32::from(t.flags) << 16).to_le_bytes());
    }
    for k in 0..tracks {
        out.extend_from_slice(&word(k));
    }
    let mut offset = first;
    for (i, &frame) in sizes.iter().enumerate() {
        out.extend_from_slice(&word(offset | usize::from(i == 0)));
        offset += frame;
    }
    out.extend_from_slice(&word(offset));
    for frame in frames {
        out.extend_from_slice(frame);
        if frame.len() % 2 == 1 {
            out.push(0);
        }
    }
    out
}

/// The zeros after a sample's last frame: `BINKW32.DLL` never reads a file's last 8 bytes
/// (black box, E17 M6 T6c1: the header's size field is the file's less 8; a last frame's last
/// block coded there decoded wrongly, and right with 8 bytes more), and the game's clips need
/// none of theirs. A decoder that reads them (FFmpeg, `h3-bink`) finds the packet's planes
/// complete before them.
pub const TAIL: usize = 8;

/// A run of a [`Block::Run`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Run {
    /// `len` pixels of one colour.
    Fill(u8, usize),
    /// A colour for each pixel.
    Pixels(Vec<u8>),
}

/// One 8 × 8 block as the writer codes it, by its type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Block {
    /// Type 0: the block stays as it was.
    Skip,
    /// Type 1: runs of pixels in the scan order `order` (0 to 15); when they cover 63 pixels,
    /// `last` colours the 64th. Each run's length must fit its bits (by the pixels before it).
    Run {
        /// The scan order.
        order: u8,
        /// The runs.
        runs: Vec<Run>,
        /// The 64th pixel's colour, written only after 63 pixels.
        last: u8,
    },
    /// Type 2: an intra DCT block: its DC (11 bits), its quantizer index (0 to 15) and its AC
    /// coefficients by scan position (position 0 unused), each above −2^15.
    Intra {
        /// The DC.
        dc: u16,
        /// The quantizer index.
        q: u8,
        /// The AC coefficients.
        coefficients: [i16; 64],
    },
    /// Type 3: the motion copy and a residue added, its coefficients by scan position, each
    /// below 256 in magnitude.
    Residue {
        /// The vector's x, −16 to 15.
        x: i8,
        /// The vector's y, −16 to 15 (the key frame's biased by −15).
        y: i8,
        /// The residue's coefficients.
        coefficients: [i16; 64],
    },
    /// Type 4: the motion copy and an inter DCT block added.
    Inter {
        /// The vector's x.
        x: i8,
        /// The vector's y.
        y: i8,
        /// The DC (−1024 to 1023).
        dc: i16,
        /// The quantizer index.
        q: u8,
        /// The AC coefficients, as [`Block::Intra`]'s.
        coefficients: [i16; 64],
    },
    /// Type 5: one colour.
    Fill(u8),
    /// Type 6: two colours, a bit for each pixel, a byte a row, its lowest bit the leftmost
    /// pixel's.
    Pattern {
        /// The colours of a 0 bit and a 1 bit.
        colours: [u8; 2],
        /// The rows' bits.
        rows: [u8; 8],
    },
    /// Type 7: the motion copy alone.
    Motion {
        /// The vector's x.
        x: i8,
        /// The vector's y.
        y: i8,
    },
    /// Type 8: every pixel's colour, row by row.
    Raw([u8; 64]),
    /// A type the decoder rejects (9 to 15).
    Unknown(u8),
}

/// One block's values for each bundle (stored as the bundle stores them) and its bits in the
/// stream.
struct Coded {
    values: [Vec<u32>; BUNDLES],
    inline: BitWriter,
}

impl Coded {
    fn push(&mut self, bundle: usize, value: i32) {
        let width = WIDTH[bundle % BUNDLES];
        let half = if SIGNED[bundle % BUNDLES] {
            1 << (width - 1)
        } else {
            0
        };
        let stored = value.wrapping_add(half).cast_unsigned() & ((1 << width) - 1);
        self.values[bundle % BUNDLES].push(stored);
    }
}

/// The highest set bit of `|value|`, −1 for 0.
fn top(value: i16) -> i32 {
    let magnitude = value.unsigned_abs();
    if magnitude == 0 {
        -1
    } else {
        15 - magnitude.leading_zeros() as i32
    }
}

/// The highest magnitude among the coefficients a list entry covers.
fn span(tops: &[i32; 64], position: u8, mode: u8) -> i32 {
    let len = match mode {
        0 => 20,
        1 => 16,
        2 => 4,
        _ => 1,
    };
    let start = usize::from(position);
    tops.get(start..(start + len).min(64))
        .and_then(|s| s.iter().copied().max())
        .unwrap_or(-1)
}

/// A coefficient of `magnitude` bits read as the decoder reads it.
fn coefficient(w: &mut BitWriter, value: i16, magnitude: i32) {
    if magnitude > 0 {
        let low = u32::from(value.unsigned_abs()) - (1 << magnitude);
        w.put(low, magnitude.cast_unsigned());
    }
    w.flag(value < 0);
}

/// A DCT block's AC coefficients as `read_dct_coeffs` reads them: the magnitude's bits + 1, then
/// for each magnitude down to 0 a bit per list entry (whether it holds a coefficient of this
/// magnitude), a group's four a bit each (later, or now and its value).
fn dct(w: &mut BitWriter, coefficients: &[i16; 64]) {
    let tops: [i32; 64] = core::array::from_fn(|p| if p == 0 { -1 } else { top(coefficients[p]) });
    let max = tops.iter().copied().max().unwrap_or(-1);
    w.put((max + 1).cast_unsigned(), 4);
    let mut lists = Lists::new(&DCT_LISTS);
    for magnitude in (0..=max).rev() {
        let mut at = lists.start;
        while at < lists.end {
            let (position, mode) = lists.get(at);
            if position | mode == 0 {
                at += 1;
                continue;
            }
            let open = span(&tops, position, mode) == magnitude;
            w.flag(open);
            if !open {
                at += 1;
                continue;
            }
            match mode {
                0 | 2 => {
                    if mode == 0 {
                        lists.set(at, position + 4, 1);
                    } else {
                        lists.set(at, 0, 0);
                        at += 1;
                    }
                    for p in position..position + 4 {
                        let now = tops[usize::from(p & 63)] == magnitude;
                        w.flag(!now);
                        if now {
                            coefficient(w, coefficients[usize::from(p & 63)], magnitude);
                        } else {
                            lists.insert_single(p);
                        }
                    }
                }
                1 => {
                    lists.set(at, position, 2);
                    for k in 1..4 {
                        lists.append(position + 4 * k, 2);
                    }
                }
                _ => {
                    coefficient(w, coefficients[usize::from(position & 63)], magnitude);
                    lists.set(at, 0, 0);
                    at += 1;
                }
            }
        }
    }
}

/// A residue as `read_residue` reads it; returns the count its bundle must give (the events less
/// one, at most 127: the decoder stops after the last; with none, 0 and every bit written).
fn residue(w: &mut BitWriter, coefficients: &[i16; 64]) -> i32 {
    let tops: [i32; 64] = core::array::from_fn(|p| top(coefficients[p]));
    let events: u32 = coefficients
        .iter()
        .map(|c| c.unsigned_abs().count_ones())
        .sum();
    let count = events.saturating_sub(1).min(127).cast_signed();
    let mut left = count;
    let max = tops.iter().copied().max().unwrap_or(-1).max(0);
    w.put(max.cast_unsigned(), 3);
    let mut lists = Lists::new(&RESIDUE_LISTS);
    let mut set: Vec<usize> = Vec::new();
    for magnitude in (0..=max).rev() {
        for &p in &set {
            let bit = coefficients[p & 63].unsigned_abs() >> magnitude & 1 == 1;
            w.flag(bit);
            if bit {
                left -= 1;
                if left < 0 {
                    return count;
                }
            }
        }
        let mut at = lists.start;
        while at < lists.end {
            let (position, mode) = lists.get(at);
            if position | mode == 0 {
                at += 1;
                continue;
            }
            let open = span(&tops, position, mode) == magnitude;
            w.flag(open);
            if !open {
                at += 1;
                continue;
            }
            let mut start = |w: &mut BitWriter, p: u8| {
                w.flag(coefficients[usize::from(p & 63)] < 0);
                set.push(usize::from(p & 63));
                left -= 1;
                left >= 0
            };
            match mode {
                0 | 2 => {
                    if mode == 0 {
                        lists.set(at, position + 4, 1);
                    } else {
                        lists.set(at, 0, 0);
                        at += 1;
                    }
                    for p in position..position + 4 {
                        let now = tops[usize::from(p & 63)] == magnitude;
                        w.flag(!now);
                        if !now {
                            lists.insert_single(p);
                        } else if !start(w, p) {
                            return count;
                        }
                    }
                }
                1 => {
                    lists.set(at, position, 2);
                    for k in 1..4 {
                        lists.append(position + 4 * k, 2);
                    }
                }
                _ => {
                    lists.set(at, 0, 0);
                    at += 1;
                    if !start(w, position) {
                        return count;
                    }
                }
            }
        }
    }
    count
}

/// A block's values and bits.
fn code(block: &Block) -> Coded {
    let mut c = Coded {
        values: Default::default(),
        inline: BitWriter::new(),
    };
    match block {
        Block::Skip => c.push(BLOCK_TYPES, 0),
        Block::Run { order, runs, last } => {
            c.push(BLOCK_TYPES, 1);
            c.inline.put(u32::from(*order), 4);
            let mut done = 0;
            for run in runs {
                let (fill, len) = match run {
                    Run::Fill(_, len) => (true, *len),
                    Run::Pixels(colours) => (false, colours.len()),
                };
                c.inline.flag(fill);
                let bits = u32::from(RUN_BITS[done & 63]);
                c.inline
                    .put(u32::try_from(len.saturating_sub(1)).unwrap_or(0), bits);
                match run {
                    Run::Fill(colour, _) => c.push(COLORS, i32::from(*colour)),
                    Run::Pixels(colours) => {
                        for &colour in colours {
                            c.push(COLORS, i32::from(colour));
                        }
                    }
                }
                done += len;
            }
            if done == 63 {
                c.push(COLORS, i32::from(*last));
            }
        }
        Block::Intra {
            dc,
            q,
            coefficients,
        } => {
            c.push(BLOCK_TYPES, 2);
            c.push(INTRA_DC, i32::from(*dc));
            c.push(INTRA_Q, i32::from(*q));
            dct(&mut c.inline, coefficients);
        }
        Block::Residue { x, y, coefficients } => {
            c.push(BLOCK_TYPES, 3);
            c.push(X_OFF, i32::from(*x));
            c.push(Y_OFF, i32::from(*y));
            let count = residue(&mut c.inline, coefficients);
            c.push(INTER_COEFS, count);
        }
        Block::Inter {
            x,
            y,
            dc,
            q,
            coefficients,
        } => {
            c.push(BLOCK_TYPES, 4);
            c.push(X_OFF, i32::from(*x));
            c.push(Y_OFF, i32::from(*y));
            c.push(INTER_DC, i32::from(*dc));
            c.push(INTER_Q, i32::from(*q));
            dct(&mut c.inline, coefficients);
        }
        Block::Fill(colour) => {
            c.push(BLOCK_TYPES, 5);
            c.push(COLORS, i32::from(*colour));
        }
        Block::Pattern { colours, rows } => {
            c.push(BLOCK_TYPES, 6);
            for &colour in colours {
                c.push(COLORS, i32::from(colour));
            }
            for &row in rows {
                c.push(PATTERN, i32::from(row));
            }
        }
        Block::Motion { x, y } => {
            c.push(BLOCK_TYPES, 7);
            c.push(X_OFF, i32::from(*x));
            c.push(Y_OFF, i32::from(*y));
        }
        Block::Raw(colours) => {
            c.push(BLOCK_TYPES, 8);
            for &colour in colours {
                c.push(COLORS, i32::from(colour));
            }
        }
        Block::Unknown(kind) => c.push(BLOCK_TYPES, i32::from(*kind)),
    }
    c
}

/// The most values one count gives (13 bits).
const MAX_COUNT: usize = (1 << COUNT_BITS) - 1;

/// One plane of `wide` × `high` blocks (row by row; missing ones skipped): at each block row's
/// start every used-up bundle is refilled with the values of as many whole rows as one count
/// holds, or ended with a count of 0 when no row after needs it; then each block's own bits.
fn plane(w: &mut BitWriter, blocks: &[Block], wide: usize, high: usize) {
    let coded: Vec<Coded> = (0..wide * high)
        .map(|i| code(blocks.get(i).unwrap_or(&Block::Skip)))
        .collect();
    let rows: Vec<[usize; BUNDLES]> = coded
        .chunks(wide.max(1))
        .map(|row| core::array::from_fn(|b| row.iter().map(|c| c.values[b].len()).sum()))
        .collect();
    let mut written = [0usize; BUNDLES];
    let mut taken = [0usize; BUNDLES];
    let mut ended = [false; BUNDLES];
    for (r, row) in coded.chunks(wide.max(1)).enumerate() {
        for b in 0..BUNDLES {
            if ended[b] || written[b] > taken[b] {
                continue;
            }
            let mut count = 0;
            let mut until = r;
            while let Some(more) = rows.get(until).map(|row| row[b])
                && count + more <= MAX_COUNT
            {
                count += more;
                until += 1;
            }
            w.put(u32::try_from(count).unwrap_or(0), COUNT_BITS);
            if count == 0 {
                ended[b] = true;
                continue;
            }
            for block in coded.get(r * wide..until * wide).unwrap_or_default() {
                for &value in &block.values[b] {
                    w.put(value, WIDTH[b]);
                }
            }
            written[b] += count;
        }
        for block in row {
            for (taken, values) in taken.iter_mut().zip(&block.values) {
                *taken += values.len();
            }
            w.append(&block.inline);
        }
    }
    w.align32();
}

/// A video packet of a `width` × `height` frame from each plane's blocks (Y, U, V; row by row,
/// missing ones skipped).
#[must_use]
pub fn frame(width: usize, height: usize, planes: [&[Block]; 3]) -> Vec<u8> {
    let mut w = BitWriter::new();
    for (i, blocks) in planes.into_iter().enumerate() {
        let size = if i == 0 { 8 } else { 16 };
        plane(&mut w, blocks, width.div_ceil(size), height.div_ceil(size));
    }
    w.into_bytes()
}

/// A small deterministic random source (xorshift64*).
#[derive(Debug, Clone)]
pub struct Random(u64);

impl Random {
    /// A source from a seed (0 is taken as 1).
    #[must_use]
    pub fn new(seed: u64) -> Self {
        Self(seed.max(1))
    }

    /// The next 32 random bits.
    pub fn bits32(&mut self) -> u32 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        (self.0.wrapping_mul(0x2545_F491_4F6C_DD1D) >> 32) as u32
    }

    /// A number below `n` (0 for 0).
    pub fn below(&mut self, n: u32) -> u32 {
        if n == 0 { 0 } else { self.bits32() % n }
    }

    /// A number from `low` to `high` inclusive.
    pub fn range(&mut self, low: i32, high: i32) -> i32 {
        low + self.below((high - low + 1).cast_unsigned()).cast_signed()
    }
}

/// A random block of type `kind` (0 to 8), its motion vectors' x and y drawn from `vectors`
/// (each a range within −16 to 15).
fn random_block(rng: &mut Random, kind: u32, vectors: [(i32, i32); 2]) -> Block {
    let colour = |rng: &mut Random| rng.below(256) as u8;
    let [(x0, x1), (y0, y1)] = vectors;
    let vector = |rng: &mut Random, axis: usize| {
        if axis == 0 {
            rng.range(x0, x1) as i8
        } else {
            rng.range(y0, y1) as i8
        }
    };
    let coefficients = |rng: &mut Random, first: usize, bits: u32| {
        let mut out = [0i16; 64];
        for _ in 0..rng.below(12) {
            let p = first + rng.below(64 - first as u32) as usize;
            let scale = rng.below(bits);
            let magnitude = (1 + rng.below(1 << scale)).min(32_767) as i16;
            out[p & 63] = if rng.below(2) == 0 {
                magnitude
            } else {
                -magnitude
            };
        }
        out
    };
    match kind {
        0 => Block::Skip,
        1 => {
            let mut runs = Vec::new();
            let mut done = 0usize;
            while done < 63 {
                let longest = (1usize << RUN_BITS[done & 63]).min(64 - done);
                let len = 1 + rng.below(longest as u32) as usize;
                runs.push(if rng.below(2) == 0 {
                    Run::Fill(colour(rng), len)
                } else {
                    Run::Pixels((0..len).map(|_| colour(rng)).collect())
                });
                done += len;
            }
            Block::Run {
                order: rng.below(16) as u8,
                runs,
                last: colour(rng),
            }
        }
        2 => Block::Intra {
            dc: rng.below(2048) as u16,
            q: rng.below(16) as u8,
            coefficients: coefficients(rng, 1, 16),
        },
        3 => Block::Residue {
            x: vector(rng, 0),
            y: vector(rng, 1),
            coefficients: coefficients(rng, 0, 8),
        },
        4 => Block::Inter {
            x: vector(rng, 0),
            y: vector(rng, 1),
            dc: rng.range(-1024, 1023) as i16,
            q: rng.below(16) as u8,
            coefficients: coefficients(rng, 1, 16),
        },
        5 => Block::Fill(colour(rng)),
        6 => Block::Pattern {
            colours: [colour(rng), colour(rng)],
            rows: core::array::from_fn(|_| colour(rng)),
        },
        7 => Block::Motion {
            x: vector(rng, 0),
            y: vector(rng, 1),
        },
        _ => Block::Raw(core::array::from_fn(|_| colour(rng))),
    }
}

/// The vectors' ranges that keep the block at column `x`, row `y` of a plane of `columns` ×
/// `rows` blocks copying from inside the plane (the key frame's y biased by −15, the decoder's):
/// where a vector reaches outside, FFmpeg leaves the block and `BINKW32.DLL` reads memory beyond
/// its plane (E17 M6 T6c1), so the samples stay inside, as the game's clips do.
fn inside(x: usize, y: usize, columns: usize, rows: usize, key: bool) -> [(i32, i32); 2] {
    let span = |at: usize, count: usize, bias: i32| {
        let at = 8 * at as i32 + bias;
        let last = 8 * (count as i32 - 1);
        ((-at).max(-16), (last - at).min(15))
    };
    [
        span(x, columns, 0),
        span(y, rows, if key { -15 } else { 0 }),
    ]
}

/// A sound block (E27 M2 T2; the writer's side of [`crate::audio::Audio`]): the spectrum's two
/// ends as binary32 bits, a quantizer byte per band, then the coefficients from the third in
/// runs of 16 (the last cut at the frame's end), each run a width and its values, a value not 0
/// followed by its sign (`coefficients[k]` the (k + 2)-th; missing ones 0, the run's width the
/// smallest that holds its values unless `widths` gives it), then zeros to a whole word.
#[must_use]
pub fn sound_block(
    ends: [u32; 2],
    quantizers: &[u8],
    frame: usize,
    coefficients: &[i32],
    widths: &[u32],
) -> Vec<u8> {
    let mut w = BitWriter::new();
    for &q in quantizers {
        w.put(u32::from(q), 8);
    }
    let value = |k: usize| coefficients.get(k).copied().unwrap_or(0);
    for (run, start) in (0..frame.saturating_sub(2)).step_by(16).enumerate() {
        let end = (start + 16).min(frame - 2);
        let needed = (start..end)
            .map(|k| 32 - value(k).unsigned_abs().leading_zeros())
            .max()
            .unwrap_or(0);
        let width = widths.get(run).copied().unwrap_or(needed);
        w.put(width, 4);
        if width > 0 {
            for k in start..end {
                let v = value(k);
                w.put(v.unsigned_abs(), width);
                if v != 0 {
                    w.flag(v < 0);
                }
            }
        }
    }
    w.align32();
    let mut out: Vec<u8> = ends.iter().flat_map(|e| e.to_le_bytes()).collect();
    out.extend(w.into_bytes());
    out
}

/// A track's packet (E27 M2 T2): its decoded size, then its blocks; its size word is the
/// frame's ([`frame_with_sound`]).
#[must_use]
pub fn sound_packet(decoded: u32, blocks: &[Vec<u8>]) -> Vec<u8> {
    let mut out = decoded.to_le_bytes().to_vec();
    for block in blocks {
        out.extend(block);
    }
    out
}

/// A frame's bytes of one audio track's `packet` (its size word first) and the `video`.
#[must_use]
pub fn frame_with_sound(packet: &[u8], video: &[u8]) -> Vec<u8> {
    let mut out = word(packet.len()).to_vec();
    out.extend(packet);
    out.extend(video);
    out
}

/// The sample's random sound track (E27 M2 T2): its rate (11 025, 22 050 or 44 100: the
/// transform's three sizes), its channels, and a packet a frame of 1 to 3 blocks (the first
/// frame's 3), the last packet's decoded size ending inside its last block; each block's ends,
/// quantizers (mostly up to 60, one in eight up to 255: past FFmpeg's cap of 95, the DLL's
/// none) and coefficients of random widths (a run left empty one time in four), drawn from a
/// source of its own so that the sample's video stays as it was.
fn sample_sound(seed: u64) -> (Track, Vec<Vec<u8>>) {
    let mut rng = Random::new(seed ^ 0x50_0D_50_0D);
    let rate = [11_025u16, 22_050, 44_100][rng.below(3) as usize];
    let stereo = rng.below(2) == 1;
    let track = Track {
        largest: 0,
        rate,
        flags: 0xC000 | if stereo { 0x2000 } else { 0 },
    };
    let frame = if rate >= 44_100 {
        2048
    } else if rate >= 22_050 {
        1024
    } else {
        512
    };
    let half = (u32::from(rate) * track.channels() + 1) >> 1;
    let bands = crate::audio::band_count(half);
    let block_bytes = 2 * (frame - frame / 16);
    let mut packets = Vec::new();
    let mut largest = 0;
    for f in 0..SAMPLE_FRAMES {
        let count = if f == 0 { 3 } else { 1 + rng.below(3) as usize };
        let blocks: Vec<Vec<u8>> = (0..count)
            .map(|_| {
                let end = |rng: &mut Random| {
                    let v = rng.range(-4000, 4000);
                    X::int(v).to_bits32().unwrap_or(0)
                };
                let ends = [end(&mut rng), end(&mut rng)];
                let quantizers: Vec<u8> = (0..bands)
                    .map(|_| {
                        if rng.below(8) == 0 {
                            rng.below(256) as u8
                        } else {
                            rng.below(61) as u8
                        }
                    })
                    .collect();
                let mut coefficients = vec![0; frame - 2];
                for run in coefficients.chunks_mut(16) {
                    if rng.below(4) == 0 {
                        continue;
                    }
                    let width = 1 + rng.below(12);
                    for c in run {
                        *c = rng.range(-(1 << width) + 1, (1 << width) - 1);
                    }
                }
                sound_block(ends, &quantizers, frame, &coefficients, &[])
            })
            .collect();
        let mut decoded = (count * block_bytes) as u32;
        if f + 1 == SAMPLE_FRAMES {
            decoded -= 2 * (1 + rng.below(100));
        }
        largest = largest.max(decoded);
        packets.push(sound_packet(decoded, &blocks));
    }
    (Track { largest, ..track }, packets)
}

/// Which 16 × 16 cells of frame `f` keep their luma (E27 M2 T2b): none in the first, else each
/// one time in four, drawn from a source of its own. Where the chroma changes there,
/// `BINKW32.DLL`'s copy may leave it out (CLT-Q122, [`crate::Target`]), so the samples check the
/// DLL's marks.
fn still_cells(rng: &mut Random, f: usize, width: usize, height: usize) -> Vec<bool> {
    (0..width.div_ceil(16) * height.div_ceil(16))
        .map(|_| f > 0 && rng.below(4) == 0)
        .collect()
}

/// The sample's frame size and length.
pub const SAMPLE_WIDTH: u32 = 256;
/// See [`SAMPLE_WIDTH`].
pub const SAMPLE_HEIGHT: u32 = 48;
/// See [`SAMPLE_WIDTH`].
pub const SAMPLE_FRAMES: usize = 8;

/// A 256 × 48 clip of 8 frames (as wide as the game's clips), every block of a random type of
/// the nine with random values (the first frame's skipped blocks raw instead; DCT coefficients
/// of every magnitude up to 15 bits, E17 M6 T6c1), after the first frame a quarter of the 16 ×
/// 16 cells' luma skipped whole ([`still_cells`]), and a sound track (E27 M2 T2,
/// [`sample_sound`]), each seed its own clip: what `h3-view video
/// --sample` writes for the DLL to decode (`bink-check.sh`; FFmpeg until E17 M6 T6c1), and what
/// the decoder's unit test hashes.
#[must_use]
pub fn sample(seed: u64) -> Vec<u8> {
    sample_sized(seed, SAMPLE_WIDTH, SAMPLE_HEIGHT)
}

/// [`sample`]'s clip of `seed` at `width` × `height` (E17 M5 T8b1, R2a-4: 256 × 120, the game's
/// clips' size, whose chroma planes hide four rows below the picture). A width other than 256
/// is decoded by FFmpeg over padded rows (`crate`'s doc): not comparable where a vector crosses
/// a row's end.
#[must_use]
pub fn sample_sized(seed: u64, width: u32, height: u32) -> Vec<u8> {
    let mut rng = Random::new(seed);
    let mut still = Random::new(seed ^ 0x5711_CE11);
    let (track, sound) = sample_sound(seed);
    let header = Header {
        audio_tracks: 1,
        tracks: vec![track],
        ..Header::new(width, height)
    };
    let (width, height) = (width as usize, height as usize);
    let frames: Vec<Vec<u8>> = (0..SAMPLE_FRAMES)
        .map(|f| {
            let still = still_cells(&mut still, f, width, height);
            let planes: [Vec<Block>; 3] = core::array::from_fn(|i| {
                let size = if i == 0 { 8 } else { 16 };
                let (columns, rows) = (width.div_ceil(size), height.div_ceil(size));
                (0..columns * rows)
                    .map(|b| {
                        let kind = rng.below(9);
                        let kind = if f == 0 && kind == 0 { 8 } else { kind };
                        let (x, y) = (b % columns, b / columns);
                        let vectors = inside(x, y, columns, rows, f == 0);
                        // Drawn whole, then skipped: the other blocks' values stay as they were.
                        let block = random_block(&mut rng, kind, vectors);
                        let cell = y / 2 * width.div_ceil(16) + x / 2;
                        if i == 0 && still.get(cell) == Some(&true) {
                            Block::Skip
                        } else {
                            block
                        }
                    })
                    .collect()
            });
            let mut packet = frame(width, height, [&planes[0], &planes[1], &planes[2]]);
            if f + 1 == SAMPLE_FRAMES {
                packet.resize(packet.len() + TAIL, 0);
            }
            frame_with_sound(&sound[f], &packet)
        })
        .collect();
    file(&header, &frames)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sound_blocks_are_written_as_the_decoder_reads_them() {
        // Ends, a quantizer, a run of width 2 (given) holding 1, −2 and zeros, the rest
        // empty, aligned to a word.
        let b = sound_block([1, 2], &[7], 20, &[1, -2], &[2]);
        assert_eq!(&b[..8], &[1, 0, 0, 0, 2, 0, 0, 0]);
        let mut w = BitWriter::new();
        w.put(7, 8);
        w.put(2, 4);
        w.put(1, 2);
        w.flag(false);
        w.put(2, 2);
        w.flag(true);
        for _ in 2..16 {
            w.put(0, 2);
        }
        w.put(0, 4);
        w.align32();
        assert_eq!(&b[8..], w.into_bytes());
        assert_eq!(sound_packet(9, &[vec![1], vec![2]]), [9, 0, 0, 0, 1, 2]);
        assert_eq!(frame_with_sound(&[5], &[6]), [1, 0, 0, 0, 5, 6]);
    }
}
