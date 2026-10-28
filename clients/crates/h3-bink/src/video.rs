//! The video of revision `b` (FFmpeg 6.1.1's `libavcodec/bink.c`: `decode_frame`,
//! `binkb_decode_plane`, the `binkb_*` bundle functions, `read_dct_coeffs`, `read_residue`).
//!
//! A frame is three planes, Y then U and V at half the size, each coded as rows of 8 × 8 blocks;
//! revision `b` decodes it in place over the frame before (FFmpeg's `ff_reget_buffer`; the first
//! frame over a black one, its motion vectors biased 15 rows up, `ybias`). Each block row first
//! refills the ten *bundles* that are used up (a 13-bit count, then that many values of the
//! bundle's width; a count of 0 ends the bundle for the plane), then each block takes its type
//! and its values from them and reads what else it needs (a run's lengths, a DCT's
//! coefficients) from the stream itself. A plane ends on a multiple of 32 bits.

use alloc::vec;
use alloc::vec::Vec;

use crate::bits::Bits;
use crate::dsp;
use crate::tables::{INTER_QUANT, INTRA_QUANT, PATTERNS, RUN_BITS, SCAN};
use crate::{Clip, Error, MAX_HEIGHT, MAX_WIDTH};

/// The video flags' alpha plane (FFmpeg's `BINK_FLAG_ALPHA`).
pub const FLAG_ALPHA: u32 = 0x0010_0000;

/// The bundles (FFmpeg's `OldSources`), in the order a block row refills them.
pub(crate) const BLOCK_TYPES: usize = 0;
pub(crate) const COLORS: usize = 1;
pub(crate) const PATTERN: usize = 2;
pub(crate) const X_OFF: usize = 3;
pub(crate) const Y_OFF: usize = 4;
pub(crate) const INTRA_DC: usize = 5;
pub(crate) const INTER_DC: usize = 6;
pub(crate) const INTRA_Q: usize = 7;
pub(crate) const INTER_Q: usize = 8;
pub(crate) const INTER_COEFS: usize = 9;
/// How many bundles there are.
pub(crate) const BUNDLES: usize = 10;
/// Each bundle's values' width in bits (`binkb_bundle_sizes`); wider than 8, a value takes two
/// bytes of its buffer.
pub(crate) const WIDTH: [u32; BUNDLES] = [4, 8, 8, 5, 5, 11, 11, 4, 4, 7];
/// Whether a bundle's values are signed, stored less half their range (`binkb_bundle_signed`).
pub(crate) const SIGNED: [bool; BUNDLES] = [
    false, false, false, true, true, false, true, false, false, false,
];
/// The bits of a bundle's count (`binkb_init_bundle`).
pub(crate) const COUNT_BITS: u32 = 13;
/// The coefficient lists' first entries for a DCT block's AC coefficients and for a residue.
pub(crate) const DCT_LISTS: [(u8, u8); 6] = [(4, 0), (24, 0), (44, 0), (1, 3), (2, 3), (3, 3)];
pub(crate) const RESIDUE_LISTS: [(u8, u8); 4] = [(4, 0), (24, 0), (44, 0), (0, 2)];

/// One plane of a frame: its pixels in rows of [`Plane::stride`] bytes over its whole block grid,
/// of which the top left [`Plane::width`] × [`Plane::height`] are the picture.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plane {
    width: usize,
    height: usize,
    stride: usize,
    pixels: Vec<u8>,
}

impl Plane {
    /// A black plane of `blocks_wide` × `blocks_high` blocks showing `width` × `height`.
    fn new(blocks_wide: usize, blocks_high: usize, width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            stride: 8 * blocks_wide,
            pixels: vec![0; 64 * blocks_wide * blocks_high],
        }
    }

    /// The picture's width in pixels.
    #[must_use]
    pub fn width(&self) -> usize {
        self.width
    }

    /// The picture's height in pixels.
    #[must_use]
    pub fn height(&self) -> usize {
        self.height
    }

    /// The bytes from one row to the next: the block grid's width.
    #[must_use]
    pub fn stride(&self) -> usize {
        self.stride
    }

    /// Every pixel of the block grid, row by row.
    #[must_use]
    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }

    /// The picture's row `y` (empty past its height).
    #[must_use]
    pub fn row(&self, y: usize) -> &[u8] {
        if y >= self.height {
            return &[];
        }
        self.pixels
            .get(y * self.stride..)
            .and_then(|rest| rest.get(..self.width))
            .unwrap_or_default()
    }
}

/// A decoded frame: Y, then U (Cb) and V (Cr) at half the width and height (YUV 4:2:0; the
/// video's range is the limited one, FFmpeg's `AVCOL_RANGE_MPEG`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame {
    planes: [Plane; 3],
}

impl Frame {
    /// The planes: Y, U, V.
    #[must_use]
    pub fn planes(&self) -> &[Plane; 3] {
        &self.planes
    }
}

/// One bundle's place in the bundles' buffer: its range, where its next value is decoded to
/// (`None` once its count was 0 in this plane) and where its next value is taken from.
#[derive(Debug, Clone, Copy, Default)]
struct Bundle {
    end: usize,
    decoded: Option<usize>,
    taken: usize,
}

/// The ten bundles over one buffer, `64` bytes a block of the luma plane each, as FFmpeg
/// allocates them (`init_bundles`): kept from plane to plane and frame to frame, so a value taken
/// past what a plane decoded is the one left there before.
#[derive(Debug, Clone)]
struct Bundles {
    data: Vec<u8>,
    state: [Bundle; BUNDLES],
}

impl Bundles {
    fn new(blocks: usize) -> Self {
        Self {
            data: vec![0; 64 * blocks * BUNDLES],
            state: [Bundle::default(); BUNDLES],
        }
    }

    /// `binkb_init_bundles`: every bundle empty at its buffer's start, for a new plane.
    fn reset(&mut self) {
        let size = self.data.len() / BUNDLES;
        for (i, bundle) in self.state.iter_mut().enumerate() {
            *bundle = Bundle {
                end: size * (i + 1),
                decoded: Some(size * i),
                taken: size * i,
            };
        }
    }

    /// `binkb_read_bundle`: refills bundle `i` when it is used up and not ended.
    fn refill(&mut self, bits: &mut Bits<'_>, i: usize) -> Result<(), Error> {
        let width = WIDTH[i % BUNDLES];
        let signed = SIGNED[i % BUNDLES];
        let bundle = &mut self.state[i % BUNDLES];
        let Some(mut at) = bundle.decoded else {
            return Ok(());
        };
        if at > bundle.taken {
            return Ok(());
        }
        let count = bits.read(COUNT_BITS) as usize;
        if count == 0 {
            bundle.decoded = None;
            return Ok(());
        }
        let bytes = if width > 8 { 2 } else { 1 };
        if bundle.end.saturating_sub(at) < count * bytes {
            return Err(Error::Invalid("bundle overflow"));
        }
        let half = if signed { 1 << (width - 1) } else { 0 };
        for _ in 0..count {
            let value = bits.read(width).wrapping_sub(half) as u16;
            for byte in value.to_le_bytes().into_iter().take(bytes) {
                if let Some(slot) = self.data.get_mut(at) {
                    *slot = byte;
                }
                at += 1;
            }
        }
        bundle.decoded = Some(at);
        Ok(())
    }

    /// `binkb_get_value`: bundle `i`'s next value.
    fn take(&mut self, i: usize) -> i32 {
        let bundle = &mut self.state[i % BUNDLES];
        let at = bundle.taken;
        let byte = |k: usize| self.data.get(at + k).copied().unwrap_or(0);
        if WIDTH[i % BUNDLES] > 8 {
            bundle.taken += 2;
            i32::from(i16::from_le_bytes([byte(0), byte(1)]))
        } else {
            bundle.taken += 1;
            if SIGNED[i % BUNDLES] {
                i32::from(byte(0).cast_signed())
            } else {
                i32::from(byte(0))
            }
        }
    }

    /// A colour (an unsigned byte).
    fn colour(&mut self) -> u8 {
        self.take(COLORS) as u8
    }
}

/// The decoder of one clip's video: the frame it decodes in place and its bundles.
#[derive(Debug, Clone)]
pub struct Decoder {
    width: usize,
    height: usize,
    frame: Frame,
    bundles: Bundles,
    /// How many frames were decoded (`frame_num`): the first is the key frame.
    decoded: u32,
}

/// Where a plane's blocks are: the plane's grid, its stride and the last block's top left (the
/// last place a motion vector may point to, `ref_end`).
#[derive(Debug, Clone, Copy)]
struct Grid {
    wide: usize,
    high: usize,
    stride: usize,
    last: usize,
}

impl Decoder {
    /// A decoder for `clip`'s video, its frame black.
    pub fn new(clip: &Clip<'_>) -> Result<Self, Error> {
        if clip.revision != b'b' {
            return Err(Error::Revision(clip.revision));
        }
        if clip.flags & FLAG_ALPHA != 0 {
            return Err(Error::Alpha);
        }
        if !(1..=MAX_WIDTH).contains(&clip.width) || !(1..=MAX_HEIGHT).contains(&clip.height) {
            return Err(Error::Size);
        }
        let (width, height) = (clip.width as usize, clip.height as usize);
        let (wide, high) = (width.div_ceil(8), height.div_ceil(8));
        let chroma = Plane::new(
            width.div_ceil(16),
            height.div_ceil(16),
            width.div_ceil(2),
            height.div_ceil(2),
        );
        Ok(Self {
            width,
            height,
            frame: Frame {
                planes: [
                    Plane::new(wide, high, width, height),
                    chroma.clone(),
                    chroma,
                ],
            },
            bundles: Bundles::new(wide * high),
            decoded: 0,
        })
    }

    /// The frame as last decoded (black before the first).
    #[must_use]
    pub fn frame(&self) -> &Frame {
        &self.frame
    }

    /// Decodes the next frame from its video packet ([`Clip::video`]) over the one before and
    /// returns it (`decode_frame`). On an error the frame stays as far as it was decoded, and the
    /// next decodes over it, as in FFmpeg.
    pub fn decode(&mut self, packet: &[u8]) -> Result<&Frame, Error> {
        let mut bits = Bits::new(packet);
        self.decoded = self.decoded.wrapping_add(1);
        let key = self.decoded == 1;
        for plane in 0..3 {
            self.plane(&mut bits, plane, key)?;
            if bits.count() >= bits.size() {
                break;
            }
        }
        Ok(&self.frame)
    }

    /// `binkb_decode_plane`.
    fn plane(&mut self, bits: &mut Bits<'_>, index: usize, key: bool) -> Result<(), Error> {
        let block = if index == 0 { 8 } else { 16 };
        let (wide, high) = (self.width.div_ceil(block), self.height.div_ceil(block));
        let stride = 8 * wide;
        let grid = Grid {
            wide,
            high,
            stride,
            last: 8 * (high - 1) * stride + 8 * (wide - 1),
        };
        let ybias = if key { -15 } else { 0 };
        let Self { frame, bundles, .. } = self;
        let pixels = &mut frame.planes[index % 3].pixels;
        bundles.reset();
        for by in 0..grid.high {
            for i in 0..BUNDLES {
                bundles.refill(bits, i)?;
            }
            for bx in 0..grid.wide {
                let at = 8 * by * stride + 8 * bx;
                block_at(bits, bundles, pixels, grid, at, ybias)?;
            }
        }
        bits.align32();
        Ok(())
    }
}

/// One block's decoding (`binkb_decode_plane`'s switch).
fn block_at(
    bits: &mut Bits<'_>,
    bundles: &mut Bundles,
    pixels: &mut [u8],
    grid: Grid,
    at: usize,
    ybias: i32,
) -> Result<(), Error> {
    let stride = grid.stride;
    // The pixel at position `p` of the block (`coordmap`), its row the position's high bits.
    let put = |pixels: &mut [u8], p: u8, value: u8| {
        let p = usize::from(p & 63);
        if let Some(slot) = pixels.get_mut(at + (p & 7) + (p >> 3) * stride) {
            *slot = value;
        }
    };
    match bundles.take(BLOCK_TYPES) {
        0 => {}
        1 => {
            let scan = &PATTERNS[(bits.read(4) & 15) as usize];
            let mut done = 0;
            loop {
                let fill = bits.flag();
                // `done` is below 63 here: the loop stops at 63.
                let run = bits.read(u32::from(RUN_BITS[done & 63])) as usize + 1;
                if done + run > 64 {
                    return Err(Error::Invalid("run"));
                }
                let colour = if fill { Some(bundles.colour()) } else { None };
                for &p in &scan[done..done + run] {
                    put(pixels, p, colour.unwrap_or_else(|| bundles.colour()));
                }
                done += run;
                if done >= 63 {
                    break;
                }
            }
            if done == 63 {
                put(pixels, scan[63], bundles.colour());
            }
        }
        2 => {
            let mut block = [0; 64];
            block[0] = bundles.take(INTRA_DC);
            let q = bundles.take(INTRA_Q);
            let read = coefficients(bits, &mut block)?;
            dsp::unquantize(&mut block, &INTRA_QUANT[(q & 15) as usize], read.as_slice());
            dsp::idct_put(pixels, at, stride, &block);
        }
        3 => {
            motion(bundles, pixels, grid, at, ybias);
            let mut block = [0; 64];
            let count = bundles.take(INTER_COEFS);
            residue(bits, &mut block, count);
            dsp::add_residue(pixels, at, stride, &block);
        }
        4 => {
            motion(bundles, pixels, grid, at, ybias);
            let mut block = [0; 64];
            block[0] = bundles.take(INTER_DC);
            let q = bundles.take(INTER_Q);
            let read = coefficients(bits, &mut block)?;
            dsp::unquantize(&mut block, &INTER_QUANT[(q & 15) as usize], read.as_slice());
            dsp::idct_add(pixels, at, stride, &block);
        }
        5 => {
            let colour = bundles.colour();
            for p in 0..64 {
                put(pixels, p, colour);
            }
        }
        6 => {
            let colours = [bundles.colour(), bundles.colour()];
            for y in 0..8 {
                let mut row = bundles.take(PATTERN);
                for x in 0..8 {
                    put(pixels, 8 * y + x, colours[(row & 1) as usize]);
                    row >>= 1;
                }
            }
        }
        7 => motion(bundles, pixels, grid, at, ybias),
        8 => {
            for p in 0..64 {
                put(pixels, p, bundles.colour());
            }
        }
        _ => return Err(Error::Invalid("block type")),
    }
    Ok(())
}

/// A motion block's copy: the 8 × 8 pixels at the vector's offset from the block, in the frame
/// being decoded, copied as they were before the copy; none when the source's top left lies
/// before the plane or past its last block's (FFmpeg warns and leaves the block).
fn motion(bundles: &mut Bundles, pixels: &mut [u8], grid: Grid, at: usize, ybias: i32) {
    let x = bundles.take(X_OFF);
    let y = bundles.take(Y_OFF) + ybias;
    let stride = grid.stride as isize;
    let from = at as isize + x as isize + y as isize * stride;
    if from < 0 || from > grid.last as isize {
        return;
    }
    let from = from as usize;
    let mut copy = [0u8; 64];
    for (i, slot) in copy.iter_mut().enumerate() {
        let offset = (i & 7) + (i >> 3) * grid.stride;
        *slot = pixels.get(from + offset).copied().unwrap_or(0);
    }
    for (i, &value) in copy.iter().enumerate() {
        if let Some(slot) = pixels.get_mut(at + (i & 7) + (i >> 3) * grid.stride) {
            *slot = value;
        }
    }
}

/// The scan positions a block's coefficients were read at, in order.
#[derive(Debug)]
struct Read {
    positions: [u8; 64],
    count: usize,
}

impl Read {
    fn as_slice(&self) -> &[u8] {
        self.positions.get(..self.count).unwrap_or_default()
    }
}

/// The coefficient lists both readers walk: entries of a scan position and a mode (0 a group of
/// four and the next sixteen, 1 four groups, 2 a group, 3 one coefficient; position 0 with mode
/// 0 a used entry), new single ones inserted before the start, new groups appended at the end.
/// The start never goes below 0 (each of the 64 positions inserted at most once) and the end
/// past 79 (at most 6 entries, 9 appended).
pub(crate) struct Lists {
    position: [u8; 128],
    mode: [u8; 128],
    pub(crate) start: usize,
    pub(crate) end: usize,
}

impl Lists {
    pub(crate) fn new(entries: &[(u8, u8)]) -> Self {
        let mut lists = Self {
            position: [0; 128],
            mode: [0; 128],
            start: 64,
            end: 64,
        };
        for &(position, mode) in entries {
            lists.append(position, mode);
        }
        lists
    }

    pub(crate) fn append(&mut self, position: u8, mode: u8) {
        self.position[self.end & 127] = position;
        self.mode[self.end & 127] = mode;
        self.end += 1;
    }

    pub(crate) fn insert_single(&mut self, position: u8) {
        self.start -= 1;
        self.position[self.start & 127] = position;
        self.mode[self.start & 127] = 3;
    }

    pub(crate) fn get(&self, at: usize) -> (u8, u8) {
        (self.position[at & 127], self.mode[at & 127])
    }

    pub(crate) fn set(&mut self, at: usize, position: u8, mode: u8) {
        self.position[at & 127] = position;
        self.mode[at & 127] = mode;
    }
}

/// A coefficient of `bits` bits' magnitude: its low bits and sign (`read_dct_coeffs`), or ±1.
fn coefficient(bits: &mut Bits<'_>, magnitude: i32) -> i32 {
    if magnitude == 0 {
        return if bits.flag() { -1 } else { 1 };
    }
    let value = bits.read(magnitude as u32) as i32 | 1 << magnitude;
    if bits.flag() { -value } else { value }
}

/// `read_dct_coeffs` (the quantizer index given by the bundles): the AC coefficients into
/// `block` by the scan order, from the largest magnitude down.
fn coefficients(bits: &mut Bits<'_>, block: &mut [i32; 64]) -> Result<Read, Error> {
    if !bits.has(4) {
        return Err(Error::Invalid("coefficients"));
    }
    let mut read = Read {
        positions: [0; 64],
        count: 0,
    };
    let mut lists = Lists::new(&DCT_LISTS);
    let mut store = |read: &mut Read, position: u8, value: i32| {
        block[usize::from(SCAN[usize::from(position & 63)])] = value;
        // At most 63 coefficients: each position is read once.
        read.positions[read.count & 63] = position;
        read.count += 1;
    };
    let mut magnitude = bits.read(4) as i32 - 1;
    while magnitude >= 0 {
        let mut at = lists.start;
        while at < lists.end {
            let (position, mode) = lists.get(at);
            if position | mode == 0 || !bits.flag() {
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
                        if bits.flag() {
                            lists.insert_single(p);
                        } else {
                            let value = coefficient(bits, magnitude);
                            store(&mut read, p, value);
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
                    let value = coefficient(bits, magnitude);
                    store(&mut read, position, value);
                    lists.set(at, 0, 0);
                    at += 1;
                }
            }
        }
        magnitude -= 1;
    }
    Ok(read)
}

/// `read_residue`'s state: the stream, the block, the coefficients set so far by their place in
/// the block (each at most once) and how many more may be set or refined.
struct Residue<'b, 'a> {
    bits: &'b mut Bits<'a>,
    block: &'b mut [i16; 64],
    set: [u8; 64],
    sets: usize,
    left: i32,
}

impl Residue<'_, '_> {
    /// Sets the coefficient at scan position `position` to ±`mask`; false once the count is spent.
    fn start(&mut self, position: u8, mask: i16) -> bool {
        let at = SCAN[usize::from(position & 63)];
        self.set[self.sets & 63] = at;
        self.sets += 1;
        self.block[usize::from(at & 63)] = if self.bits.flag() { -mask } else { mask };
        self.left -= 1;
        self.left >= 0
    }

    /// Adds `mask` to the magnitude of each coefficient set before whose bit says so; false once
    /// the count is spent.
    fn refine(&mut self, mask: i16) -> bool {
        for i in 0..self.sets {
            if !self.bits.flag() {
                continue;
            }
            let at = usize::from(self.set[i & 63] & 63);
            if self.block[at] < 0 {
                self.block[at] -= mask;
            } else {
                self.block[at] += mask;
            }
            self.left -= 1;
            if self.left < 0 {
                return false;
            }
        }
        true
    }
}

/// `read_residue`: the residue's coefficients into `block`, bit plane by bit plane from the
/// largest, until `count` + 1 of them were set or refined (`masks_count`).
fn residue(bits: &mut Bits<'_>, block: &mut [i16; 64], count: i32) {
    let mut lists = Lists::new(&RESIDUE_LISTS);
    let mut mask: i16 = 1 << bits.read(3);
    let mut state = Residue {
        bits,
        block,
        set: [0; 64],
        sets: 0,
        left: count,
    };
    while mask != 0 {
        if !state.refine(mask) {
            return;
        }
        let mut at = lists.start;
        while at < lists.end {
            let (position, mode) = lists.get(at);
            if position | mode == 0 || !state.bits.flag() {
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
                        if state.bits.flag() {
                            lists.insert_single(p);
                        } else if !state.start(p, mask) {
                            return;
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
                    if !state.start(position, mask) {
                        return;
                    }
                }
            }
        }
        mask >>= 1;
    }
}
