//! Synthetic Bink files for tests (the writer's side of [`crate::Clip`] and
//! [`crate::Decoder`]; *provisional*, E17 M5 T4b2a): a bit writer, a file around given frames, a
//! video packet of given blocks per plane (its bundles refilled exactly when the decoder refills
//! them, a DCT block's and a residue's coefficients coded as the decoder reads them), and
//! [`sample`], a clip of every block type in random variations, whose decoding FFmpeg's checks
//! (`h3-view video --sample`, [`crate::check`]).
//!
//! The writer does not validate: a test gets what it asks for (a run past the block, a type the
//! decoder rejects), so that the decoder's checks can be tested.

use alloc::vec::Vec;

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
        }
    }
}

fn word(value: usize) -> [u8; 4] {
    u32::try_from(value).unwrap_or(u32::MAX).to_le_bytes()
}

/// A Bink file of `frames` (each a frame's whole bytes, padded with a 0 to an even size as the
/// files' are): the header, the audio tracks' descriptions (zeros), the index of `frames.len()`
/// + 1 offsets (the first marked a key frame, the last the file's end), the frames.
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
    out.resize(out.len() + extra + 12 * tracks, 0);
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

/// A random block of type `kind` (0 to 8).
fn random_block(rng: &mut Random, kind: u32) -> Block {
    let colour = |rng: &mut Random| rng.below(256) as u8;
    let vector = |rng: &mut Random| rng.range(-16, 15) as i8;
    let coefficients = |rng: &mut Random, first: usize, bits: u32| {
        let mut out = [0i16; 64];
        for _ in 0..rng.below(12) {
            let p = first + rng.below(64 - first as u32) as usize;
            let scale = rng.below(bits);
            let magnitude = 1 + rng.below(1 << scale) as i16;
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
            coefficients: coefficients(rng, 1, 11),
        },
        3 => Block::Residue {
            x: vector(rng),
            y: vector(rng),
            coefficients: coefficients(rng, 0, 8),
        },
        4 => Block::Inter {
            x: vector(rng),
            y: vector(rng),
            dc: rng.range(-1024, 1023) as i16,
            q: rng.below(16) as u8,
            coefficients: coefficients(rng, 1, 11),
        },
        5 => Block::Fill(colour(rng)),
        6 => Block::Pattern {
            colours: [colour(rng), colour(rng)],
            rows: core::array::from_fn(|_| colour(rng)),
        },
        7 => Block::Motion {
            x: vector(rng),
            y: vector(rng),
        },
        _ => Block::Raw(core::array::from_fn(|_| colour(rng))),
    }
}

/// The sample's frame size and length.
pub const SAMPLE_WIDTH: u32 = 256;
/// See [`SAMPLE_WIDTH`].
pub const SAMPLE_HEIGHT: u32 = 48;
/// See [`SAMPLE_WIDTH`].
pub const SAMPLE_FRAMES: usize = 8;

/// A 256 × 48 clip of 8 frames (as wide as the game's clips, whose rows FFmpeg does not pad: a
/// reference crossing a row's end reads the same pixels in both), every block of a random type
/// of the nine with random values (the first frame's skipped blocks raw instead), each seed its
/// own clip: what `h3-view video --sample` writes for FFmpeg to decode, and what the decoder's
/// unit test hashes.
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
    let header = Header::new(width, height);
    let (width, height) = (width as usize, height as usize);
    let frames: Vec<Vec<u8>> = (0..SAMPLE_FRAMES)
        .map(|f| {
            let planes: [Vec<Block>; 3] = core::array::from_fn(|i| {
                let size = if i == 0 { 8 } else { 16 };
                let count = width.div_ceil(size) * height.div_ceil(size);
                (0..count)
                    .map(|_| {
                        let kind = rng.below(9);
                        let kind = if f == 0 && kind == 0 { 8 } else { kind };
                        random_block(&mut rng, kind)
                    })
                    .collect()
            });
            frame(width, height, [&planes[0], &planes[1], &planes[2]])
        })
        .collect();
    file(&header, &frames)
}
