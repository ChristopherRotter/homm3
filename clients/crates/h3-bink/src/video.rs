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
                dsp::set(&mut self.data, at, byte);
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
        dsp::set(pixels, at + (p & 7) + (p >> 3) * stride, value);
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
        dsp::set(pixels, at + (i & 7) + (i >> 3) * grid.stride, value);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build::{self, BitWriter, Block, Header, Run};
    use crate::check;
    use alloc::vec;
    use proptest::prelude::*;

    /// A decoder for a `width` × `height` clip of revision `b`.
    fn decoder(width: u32, height: u32) -> Decoder {
        let bytes = build::file(&Header::new(width, height), &[vec![0, 0]]);
        Decoder::new(&Clip::parse(&bytes).unwrap()).unwrap()
    }

    /// Decodes one frame of the given blocks per plane.
    fn decode(d: &mut Decoder, planes: [&[Block]; 3]) -> Result<Frame, Error> {
        let packet = build::frame(d.width, d.height, planes);
        d.decode(&packet).cloned()
    }

    fn at(plane: &Plane, x: usize, y: usize) -> u8 {
        plane.pixels()[y * plane.stride() + x]
    }

    /// Plane `p`'s 8 × 8 block at block column `bx` (block row 0), row by row.
    fn block(frame: &Frame, p: usize, bx: usize) -> [u8; 64] {
        core::array::from_fn(|i| at(&frame.planes()[p], 8 * bx + i % 8, i / 8))
    }

    const NONE: &[Block] = &[];

    #[test]
    fn fills_patterns_raw_blocks_and_skips() {
        let mut d = decoder(32, 8);
        let raw: [u8; 64] = core::array::from_fn(|i| i as u8);
        let mut rows = [0b1010_1010; 8];
        rows[7] = 0b0000_0001;
        let luma = [
            Block::Fill(7),
            Block::Pattern {
                colours: [1, 2],
                rows,
            },
            Block::Raw(raw),
            Block::Skip,
        ];
        let frame = decode(&mut d, [&luma, &[Block::Fill(3)], &[Block::Fill(9)]]).unwrap();
        assert_eq!(block(&frame, 0, 0), [7; 64]);
        let pattern = block(&frame, 0, 1);
        assert_eq!(
            pattern[..8],
            [1, 2, 1, 2, 1, 2, 1, 2],
            "the lowest bit leftmost"
        );
        assert_eq!(pattern[56..], [2, 1, 1, 1, 1, 1, 1, 1]);
        assert_eq!(block(&frame, 0, 2), raw);
        assert_eq!(block(&frame, 0, 3), [0; 64], "skipped over the black frame");
        assert_eq!(block(&frame, 1, 0), [3; 64]);
        assert_eq!(block(&frame, 2, 0), [9; 64]);
        // The next frame decodes over this one: skipped blocks keep it, a motion block copies.
        let frame = decode(
            &mut d,
            [&[Block::Skip, Block::Motion { x: -8, y: 0 }], NONE, NONE],
        )
        .unwrap();
        assert_eq!(block(&frame, 0, 0), [7; 64]);
        assert_eq!(block(&frame, 0, 1), [7; 64]);
        assert_eq!(block(&frame, 0, 2), raw);
        assert_eq!(block(&frame, 1, 0), [3; 64]);
        assert_eq!(d.frame(), &frame);
    }

    #[test]
    fn runs_follow_their_scan_order() {
        let mut d = decoder(16, 8);
        let pixels: Vec<u8> = (1..=8).collect();
        let runs = vec![
            Run::Fill(5, 8),
            Run::Pixels(pixels.clone()),
            Run::Fill(6, 47),
        ];
        // 63 pixels by the runs, the last one's colour after them; then 64 by the runs alone.
        let luma = [
            Block::Run {
                order: 0,
                runs,
                last: 9,
            },
            Block::Run {
                order: 1,
                runs: vec![Run::Fill(4, 60), Run::Fill(8, 4)],
                last: 99,
            },
        ];
        let frame = decode(&mut d, [&luma, NONE, NONE]).unwrap();
        let first = block(&frame, 0, 0);
        let order = PATTERNS[0];
        for (i, &p) in order.iter().enumerate() {
            let expected = match i {
                0..8 => 5,
                8..16 => pixels[i - 8],
                16..63 => 6,
                _ => 9,
            };
            assert_eq!(first[usize::from(p)], expected, "scan index {i}");
        }
        let second = block(&frame, 0, 1);
        for (i, &p) in PATTERNS[1].iter().enumerate() {
            assert_eq!(second[usize::from(p)], if i < 60 { 4 } else { 8 });
        }
    }

    /// The pixels `block` (AC coefficients by scan position, the DC apart) gives through
    /// unquantization and the inverse DCT put on black, or added to `base`.
    fn transformed(
        dc: i32,
        quant: &[i32; 64],
        coefficients: &[i16; 64],
        base: Option<u8>,
    ) -> [u8; 64] {
        let mut block = [0; 64];
        block[0] = dc;
        let mut read = Vec::new();
        for (p, &c) in coefficients.iter().enumerate().skip(1) {
            if c != 0 {
                block[usize::from(SCAN[p])] = i32::from(c);
                read.push(p as u8);
            }
        }
        dsp::unquantize(&mut block, quant, &read);
        let mut pixels = [base.unwrap_or(0); 64];
        match base {
            None => dsp::idct_put(&mut pixels, 0, 8, &block),
            Some(_) => dsp::idct_add(&mut pixels, 0, 8, &block),
        }
        pixels
    }

    #[test]
    fn dct_blocks_decode_their_coefficients() {
        let mut d = decoder(24, 8);
        let mut coefficients = [0i16; 64];
        // Coefficients of every magnitude band and both signs, spread over the lists' groups.
        for (p, v) in [
            (1, 1),
            (2, -1),
            (3, 300),
            (5, -2),
            (9, 7),
            (30, -1000),
            (63, 12),
        ] {
            coefficients[p] = v;
        }
        let luma = [
            Block::Intra {
                dc: 800,
                q: 0,
                coefficients: [0; 64],
            },
            Block::Intra {
                dc: 1234,
                q: 7,
                coefficients,
            },
            Block::Fill(50),
        ];
        let frame = decode(&mut d, [&luma, NONE, NONE]).unwrap();
        // A DC alone: 800 × 65536 >> 11, then (v + 127) >> 8 everywhere.
        assert_eq!(block(&frame, 0, 0), [100; 64]);
        assert_eq!(
            block(&frame, 0, 1),
            transformed(1234, &INTRA_QUANT[7], &coefficients, None)
        );
        let inter = [
            Block::Skip,
            Block::Skip,
            Block::Inter {
                x: 0,
                y: 0,
                dc: -300,
                q: 15,
                coefficients,
            },
        ];
        let frame = decode(&mut d, [&inter, NONE, NONE]).unwrap();
        assert_eq!(
            block(&frame, 0, 2),
            transformed(-300, &INTER_QUANT[15], &coefficients, Some(50))
        );
    }

    #[test]
    fn residues_are_added_to_the_copy() {
        let mut d = decoder(32, 8);
        decode(&mut d, [&[Block::Fill(100), Block::Fill(10)], NONE, NONE]).unwrap();
        let mut coefficients = [0i16; 64];
        for (p, v) in [(0, 5), (3, -3), (20, 128), (40, 1), (63, -77)] {
            coefficients[p] = v;
        }
        let mut all = [0i16; 64];
        all.fill(-255);
        let luma = [
            Block::Residue {
                x: 8,
                y: 0,
                coefficients,
            },
            Block::Residue {
                x: 0,
                y: 0,
                coefficients: [0; 64],
            },
            // 512 events: the count stops it after 128; the stream stays in step.
            Block::Residue {
                x: 0,
                y: 0,
                coefficients: all,
            },
            Block::Fill(42),
        ];
        let frame = decode(&mut d, [&luma, NONE, NONE]).unwrap();
        let mut expected = [10u8; 64];
        for (p, &v) in coefficients.iter().enumerate() {
            let at = usize::from(SCAN[p]);
            expected[at] = expected[at].wrapping_add(v as u8);
        }
        assert_eq!(block(&frame, 0, 0), expected);
        assert_eq!(block(&frame, 0, 1), [10; 64]);
        assert_eq!(block(&frame, 0, 3), [42; 64]);
    }

    #[test]
    fn a_residue_stops_at_its_count_wherever_it_falls() {
        let mut d = decoder(24, 8);
        // One event, in a group; two events, the last a single entry; the count then a refinement.
        let mut group = [0i16; 64];
        group[5] = 1;
        let mut single = [0i16; 64];
        single[0] = 2;
        single[1] = -1;
        let mut refined = [0i16; 64];
        refined[2] = 3;
        let luma: Vec<Block> = [group, single, refined]
            .into_iter()
            .map(|coefficients| Block::Residue {
                x: 0,
                y: 0,
                coefficients,
            })
            .collect();
        let frame = decode(&mut d, [&luma, NONE, NONE]).unwrap();
        for (bx, coefficients) in [group, single, refined].iter().enumerate() {
            let pixels = block(&frame, 0, bx);
            for (p, &v) in coefficients.iter().enumerate() {
                assert_eq!(
                    pixels[usize::from(SCAN[p])],
                    v as u8,
                    "block {bx}, position {p}"
                );
            }
        }
    }

    #[test]
    fn the_key_frame_biases_vectors_15_rows_up_and_motion_copies_as_before() {
        let mut d = decoder(8, 32);
        let column = [
            Block::Fill(10),
            Block::Fill(20),
            Block::Motion { x: 0, y: 0 },
            // Out of the plane: before its start, then past its last block.
            Block::Motion { x: -16, y: -16 },
        ];
        let frame = decode(&mut d, [&column, NONE, NONE]).unwrap();
        let y = &frame.planes()[0];
        // Block 2 (rows 16 to 23) from row 1: rows 1 to 7 are 10, row 8 is 20.
        assert!((16..23).all(|r| at(y, 3, r) == 10));
        assert_eq!(at(y, 3, 23), 20);
        assert_eq!(
            at(y, 0, 24),
            0,
            "the vector reaches row 9 - 15 - 16: left alone"
        );
        let column = [
            Block::Raw(core::array::from_fn(|i| (i / 8) as u8)),
            Block::Motion { x: 0, y: -4 },
            Block::Skip,
            Block::Motion { x: 1, y: 0 },
        ];
        let frame = decode(&mut d, [&column, NONE, NONE]).unwrap();
        let y = &frame.planes()[0];
        // Rows 8 to 11 from rows 4 to 7, rows 12 to 15 from rows 8 to 11 as they were.
        let rows: Vec<u8> = (8..16).map(|r| at(y, 0, r)).collect();
        assert_eq!(rows, [4, 5, 6, 7, 20, 20, 20, 20]);
        assert_eq!(
            at(y, 0, 24),
            0,
            "past the last block's top left: left alone"
        );
    }

    /// A packet of one block row per plane written by hand: the ten bundles' counts and values
    /// (`(count, values)` each), then `inline`.
    fn by_hand(bundles: &[(u32, &[u32])], inline: &BitWriter) -> BitWriter {
        let mut w = BitWriter::new();
        for b in 0..BUNDLES {
            let (count, values) = bundles.get(b).copied().unwrap_or((0, &[]));
            w.put(count, COUNT_BITS);
            for &v in values {
                w.put(v, WIDTH[b]);
            }
        }
        w.append(inline);
        w.align32();
        w
    }

    #[test]
    fn an_ended_bundle_gives_its_values_as_left_and_an_overfull_one_fails() {
        let mut d = decoder(8, 8);
        decode(
            &mut d,
            [&[Block::Fill(77)], &[Block::Fill(1)], &[Block::Fill(2)]],
        )
        .unwrap();
        // A fill whose colour bundle ends at once: it takes the colour left there, the V plane's.
        let mut packet = by_hand(&[(1, &[5])], &BitWriter::new());
        for _ in 0..2 {
            packet.append(&by_hand(&[], &BitWriter::new()));
        }
        assert!(packet.len() > 0 && !packet.is_empty());
        let frame = d.decode(&packet.into_bytes()).unwrap();
        assert_eq!(block(frame, 0, 0), [2; 64]);
        // 65 colours do not fit a bundle of one block's 64 bytes.
        let values = [0; 65];
        let packet = by_hand(&[(1, &[5]), (65, &values)], &BitWriter::new());
        assert_eq!(
            d.decode(&packet.into_bytes()),
            Err(Error::Invalid("bundle overflow"))
        );
    }

    #[test]
    fn a_packet_ending_after_a_plane_leaves_the_others() {
        let mut d = decoder(8, 8);
        let plane = by_hand(&[(1, &[5]), (1, &[33])], &BitWriter::new());
        assert_eq!(plane.len(), 160);
        let frame = d.decode(&plane.into_bytes()).unwrap();
        assert_eq!(block(frame, 0, 0), [33; 64]);
        // Decoded, U would take the type and colour left in the bundles: a fill of 33.
        assert_eq!(block(frame, 1, 0), [0; 64]);
    }

    #[test]
    fn bad_blocks_fail_the_frame() {
        let mut d = decoder(8, 8);
        assert_eq!(
            decode(&mut d, [&[Block::Unknown(9)], NONE, NONE]),
            Err(Error::Invalid("block type"))
        );
        // At 49 pixels a run has 4 bits: 16 more is 65.
        let run = Block::Run {
            order: 0,
            runs: vec![Run::Fill(1, 49), Run::Fill(2, 16)],
            last: 0,
        };
        assert_eq!(
            decode(&mut d, [&[run], NONE, NONE]),
            Err(Error::Invalid("run"))
        );
        // A DCT block with fewer than 4 bits left, intra then inter: the packet cut after the
        // bundles (130 bits of counts, 4 of type, 11 of DC, 4 of quantizer; inter 10 of vector).
        let intra = Block::Intra {
            dc: 1,
            q: 1,
            coefficients: [0; 64],
        };
        let packet = build::frame(8, 8, [&[intra], NONE, NONE]);
        assert_eq!(d.decode(&packet[..19]), Err(Error::Invalid("coefficients")));
        let inter = Block::Inter {
            x: 0,
            y: 0,
            dc: 1,
            q: 1,
            coefficients: [0; 64],
        };
        let packet = build::frame(8, 8, [&[inter], NONE, NONE]);
        assert_eq!(d.decode(&packet[..20]), Err(Error::Invalid("coefficients")));
    }

    #[test]
    fn only_revision_b_without_alpha_and_of_a_bounded_size_is_taken() {
        let file = |header: &Header| build::file(header, &[vec![0, 0]]);
        let mut header = Header::new(8, 8);
        header.revision = b'f';
        let bytes = file(&header);
        assert_eq!(
            Decoder::new(&Clip::parse(&bytes).unwrap()).err(),
            Some(Error::Revision(b'f'))
        );
        let mut header = Header::new(8, 8);
        header.flags = FLAG_ALPHA;
        let bytes = file(&header);
        assert_eq!(
            Decoder::new(&Clip::parse(&bytes).unwrap()).err(),
            Some(Error::Alpha)
        );
        for (w, h) in [(MAX_WIDTH + 1, 8), (8, MAX_HEIGHT + 1)] {
            let bytes = file(&Header::new(w, h));
            assert_eq!(
                Decoder::new(&Clip::parse(&bytes).unwrap()).err(),
                Some(Error::Size)
            );
        }
        let bytes = file(&Header::new(8, 8));
        let mut clip = Clip::parse(&bytes).unwrap();
        clip.width = 0;
        assert_eq!(Decoder::new(&clip).err(), Some(Error::Size));
    }

    #[test]
    fn planes_cover_the_block_grid_and_show_the_picture() {
        let d = decoder(15, 9);
        let [y, u, v] = d.frame().planes();
        assert_eq!((y.width(), y.height(), y.stride()), (15, 9, 16));
        assert_eq!(y.pixels().len(), 16 * 16);
        assert_eq!((u.width(), u.height(), u.stride()), (8, 5, 8));
        assert_eq!(u.pixels().len(), 64);
        assert_eq!(u, v);
        assert_eq!(y.row(0).len(), 15);
        assert_eq!(y.row(8).len(), 15);
        assert!(y.row(9).is_empty());
        assert!(
            y.pixels().iter().all(|&p| p == 0),
            "black before the first frame"
        );
    }

    /// The hashes of the samples' frames as FFmpeg 6.1.1's decoder gives them (`bink-check.sh`
    /// at E17 M5 T4b2a, through PyAV; seeds 1 to 8 checked there).
    #[test]
    fn the_samples_decode_as_ffmpeg_decodes_them() {
        for (seed, hash) in [
            (1, 0x474c_73f5_718c_0ddc_u64),
            (2, 0x6329_f584_b68e_62e6),
            (3, 0x5542_c890_ab86_20a4),
        ] {
            let frames = check::frames(&build::sample(seed)).unwrap();
            assert_eq!(frames.len(), build::SAMPLE_FRAMES);
            assert_eq!(check::hash(&frames), hash, "seed {seed}");
        }
    }

    #[test]
    fn checks_report_what_fails() {
        assert_eq!(check::frames(b"junk").err(), Some(Error::Truncated));
        let mut header = Header::new(8, 8);
        header.revision = b'g';
        let bytes = build::file(&header, &[vec![0, 0]]);
        assert_eq!(check::frames(&bytes).err(), Some(Error::Revision(b'g')));
        let mut header = Header::new(8, 8);
        header.audio_tracks = 1;
        let bytes = build::file(&header, &[vec![99, 0, 0, 0]]);
        assert_eq!(
            check::frames(&bytes).err(),
            Some(Error::Field("audio size"))
        );
        // A failing frame is left out; the others stay.
        let bad = build::frame(8, 8, [&[Block::Unknown(15)], NONE, NONE]);
        let good = build::frame(8, 8, [&[Block::Fill(3)], NONE, NONE]);
        let bytes = build::file(&Header::new(8, 8), &[good, bad]);
        assert_eq!(check::frames(&bytes).unwrap().len(), 1);
    }

    #[test]
    fn the_writer_counts_its_bits() {
        let mut w = BitWriter::new();
        assert!(w.is_empty());
        w.put(0b101, 3);
        w.flag(true);
        assert_eq!(w.len(), 4);
        w.align32();
        assert_eq!(w.into_bytes(), [0b1101, 0, 0, 0]);
        let mut rng = build::Random::new(0);
        assert_eq!(rng.below(0), 0);
        assert!((0..100).all(|_| (-3..=3).contains(&rng.range(-3, 3))));
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(64))]

        #[test]
        fn decoding_never_panics_on_mutated_samples(
            seed in 1u64..1000,
            flips in proptest::collection::vec((any::<usize>(), any::<u8>()), 1..40),
        ) {
            let mut bytes = build::sample(seed);
            let clip = Clip::parse(&bytes).unwrap();
            let start = clip.frames()[0].offset;
            let len = bytes.len() - start;
            for (at, value) in flips {
                bytes[start + at % len] ^= value;
            }
            let _ = check::frames(&bytes);
        }

        #[test]
        fn decoding_never_panics_on_random_packets(
            width in 1u32..40,
            height in 1u32..40,
            packets in proptest::collection::vec(proptest::collection::vec(any::<u8>(), 0..300), 1..4),
        ) {
            let mut d = decoder(width, height);
            for packet in packets {
                let _ = d.decode(&packet);
            }
        }
    }
}
