//! The video of revision `b` (FFmpeg 6.1.1's `libavcodec/bink.c`: `decode_frame`,
//! `binkb_decode_plane`, the `binkb_*` bundle functions, `read_dct_coeffs`, `read_residue`).
//!
//! A frame is three planes, Y then U and V at half the size, each coded as rows of 8 × 8 blocks;
//! revision `b` decodes it in place over the frame before (FFmpeg's `ff_reget_buffer`; the first
//! frame over planes of zeros, Y = U = V = 0 as FFmpeg's buffer is allocated, which shows green,
//! not black (E17 M5 T8b1, R2a-2); its motion vectors biased 15 rows up, `ybias`). Each block
//! row first refills the ten *bundles* that are used up (a 13-bit count, then that many values
//! of the bundle's width; a count of 0 ends the bundle for the plane), then each block takes its
//! type and its values from them and reads what else it needs (a run's lengths, a DCT's
//! coefficients) from the stream itself. A plane ends on a multiple of 32 bits.

use alloc::vec;
use alloc::vec::Vec;

use crate::bits::Bits;
use crate::dsp;
use crate::tables::{DLL_INTER, DLL_INTRA, PATTERNS, RUN_BITS, SCAN};
use crate::{Clip, Error, MAX_HEIGHT, MAX_WIDTH, Reading};

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
    /// A plane of zeros, `blocks_wide` × `blocks_high` blocks showing `width` × `height`.
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
    /// The residues' reading (CLT-Q68).
    reading: Reading,
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
    /// A decoder for `clip`'s video, its frame's planes all 0 (Y = U = V = 0).
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
            reading: Reading::Fixed,
        })
    }

    /// The decoder reading as `reading` says (default [`Reading::Fixed`]).
    #[must_use]
    pub fn reading(mut self, reading: Reading) -> Self {
        self.reading = reading;
        self
    }

    /// The frame as last decoded (all 0 before the first).
    #[must_use]
    pub fn frame(&self) -> &Frame {
        &self.frame
    }

    /// Decodes the next frame from its video packet ([`Clip::video`]) over the one before and
    /// returns it (`decode_frame`). On an error the frame stays as far as it was decoded, and the
    /// next decodes over it, as in FFmpeg.
    pub fn decode(&mut self, packet: &[u8]) -> Result<&Frame, Error> {
        // An empty packet never reaches FFmpeg's decoder: `avcodec_send_packet` refuses it, so no
        // frame comes and the next is decoded as if it were not there, the key frame's count
        // too (E17 M5 T8b1, R2a's doubt c, checked through PyAV).
        if packet.is_empty() {
            return Err(Error::Invalid("empty packet"));
        }
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
        let reading = self.reading;
        let Self { frame, bundles, .. } = self;
        let pixels = &mut frame.planes[index % 3].pixels;
        bundles.reset();
        for by in 0..grid.high {
            for i in 0..BUNDLES {
                bundles.refill(bits, i)?;
            }
            for bx in 0..grid.wide {
                let at = 8 * by * stride + 8 * bx;
                block_at(bits, bundles, pixels, grid, at, (ybias, reading))?;
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
    (ybias, reading): (i32, Reading),
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
            coefficients(bits, &mut block)?;
            dsp::dll_put(pixels, at, stride, &block, &DLL_INTRA[(q & 15) as usize]);
        }
        3 => {
            motion(bundles, pixels, grid, at, ybias);
            let mut block = [0; 64];
            let count = bundles.take(INTER_COEFS);
            residue(bits, &mut block, count, reading);
            dsp::add_residue(pixels, at, stride, &block);
        }
        4 => {
            motion(bundles, pixels, grid, at, ybias);
            let mut block = [0; 64];
            block[0] = bundles.take(INTER_DC);
            let q = bundles.take(INTER_Q);
            coefficients(bits, &mut block)?;
            dsp::dll_add(pixels, at, stride, &block, &DLL_INTER[(q & 15) as usize]);
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
fn coefficients(bits: &mut Bits<'_>, block: &mut [i32; 64]) -> Result<(), Error> {
    if !bits.has(4) {
        return Err(Error::Invalid("coefficients"));
    }
    let mut lists = Lists::new(&DCT_LISTS);
    let mut store = |position: u8, value: i32| {
        block[usize::from(SCAN[usize::from(position & 63)])] = value;
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
                            store(p, value);
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
                    store(position, value);
                    lists.set(at, 0, 0);
                    at += 1;
                }
            }
        }
        magnitude -= 1;
    }
    Ok(())
}

/// `read_residue`'s state: the stream, the block, the coefficients set so far by their place in
/// the block (each at most once) and how many more may be set or refined.
struct Residue<'b, 'a> {
    bits: &'b mut Bits<'a>,
    block: &'b mut [i16; 64],
    set: [u8; 64],
    sets: usize,
    left: i32,
    /// The DLL's bytes ([`Reading::Original`]): each coefficient kept a signed byte.
    bytes: bool,
}

impl Residue<'_, '_> {
    /// `value`, a signed byte's when the DLL's bytes are read.
    fn kept(&self, value: i16) -> i16 {
        if self.bytes {
            i16::from(value as i8)
        } else {
            value
        }
    }

    /// Sets the coefficient at scan position `position` to ±`mask`; false once the count is spent.
    fn start(&mut self, position: u8, mask: i16) -> bool {
        let at = SCAN[usize::from(position & 63)];
        self.set[self.sets & 63] = at;
        self.sets += 1;
        let value = if self.bits.flag() { -mask } else { mask };
        self.block[usize::from(at & 63)] = self.kept(value);
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
            let value = self.block[at];
            self.block[at] = self.kept(if value < 0 {
                value - mask
            } else {
                value + mask
            });
            self.left -= 1;
            if self.left < 0 {
                return false;
            }
        }
        true
    }
}

/// `read_residue`: the residue's coefficients into `block`, bit plane by bit plane from the
/// largest (`n` + 1 of them, `n` the first 3 bits), until `count` + 1 coefficients were set or
/// refined (`masks_count`).
///
/// [`Reading::Original`] reads as `BINKW32.DLL` does (0x10010630, E17 M6 T6c1): the mask and the
/// coefficients signed bytes, the mask halved by an arithmetic shift. Below a first mask of 128
/// that is the same; from 128 (0x80, −128 as a byte) its masks run −128, −64, …, −1, so each
/// coefficient started below 128 takes the opposite sign and each refinement shrinks it, where
/// the format means magnitudes up to 255 ([`Reading::Fixed`], FFmpeg's). CLT-Q68 (answered
/// fix): no game clip has such a residue.
fn residue(bits: &mut Bits<'_>, block: &mut [i16; 64], count: i32, reading: Reading) {
    let mut lists = Lists::new(&RESIDUE_LISTS);
    let planes = bits.read(3);
    let bytes = reading == Reading::Original;
    let mut mask = if bytes {
        i16::from((1u8 << planes).cast_signed())
    } else {
        1 << planes
    };
    let mut state = Residue {
        bits,
        block,
        set: [0; 64],
        sets: 0,
        left: count,
        bytes,
    };
    for _ in 0..=planes {
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
    use crate::x87::X;
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
        assert_eq!(
            block(&frame, 0, 3),
            [0; 64],
            "skipped over the zeroed frame"
        );
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

    /// The pixels `block` (AC coefficients by scan position, the DC apart) gives through the
    /// DLL's inverse DCT put on zeros, or added to `base`.
    fn transformed(
        dc: i32,
        quant: &[X; 64],
        coefficients: &[i16; 64],
        base: Option<u8>,
    ) -> [u8; 64] {
        let mut block = [0; 64];
        block[0] = dc;
        for (p, &c) in coefficients.iter().enumerate().skip(1) {
            block[usize::from(SCAN[p])] = i32::from(c);
        }
        let mut pixels = [base.unwrap_or(0); 64];
        match base {
            None => dsp::dll_put(&mut pixels, 0, 8, &block, quant),
            Some(_) => dsp::dll_add(&mut pixels, 0, 8, &block, quant),
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
        // A DC alone: 800 × 0.125 (q 0's DC dequantizer) everywhere.
        assert_eq!(block(&frame, 0, 0), [100; 64]);
        assert_eq!(
            block(&frame, 0, 1),
            transformed(1234, &DLL_INTRA[7], &coefficients, None)
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
            transformed(-300, &DLL_INTER[15], &coefficients, Some(50))
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
    fn a_residue_from_128_reads_as_the_format_means_or_as_the_dll() {
        // A first bit plane of 128 (CLT-Q68): fixed, the coefficients as written; the DLL's
        // byte masks (−128, −64, …) give the later ones the other sign, each refinement
        // shrinking them: 5 (4, then 1) is −3, −3 (2, then 1) is 1.
        let mut coefficients = [0i16; 64];
        for (p, v) in [(0, 128), (9, 5), (30, -3)] {
            coefficients[p] = v;
        }
        // The key frame: its vector's y biased by −15.
        let luma = [
            Block::Fill(100),
            Block::Residue {
                x: -8,
                y: 15,
                coefficients,
            },
        ];
        for (reading, values) in [
            (Reading::Fixed, [128, 5, -3]),
            (Reading::Original, [-128, -3, 1]),
        ] {
            let mut d = decoder(16, 8).reading(reading);
            let frame = decode(&mut d, [&luma, NONE, NONE]).unwrap();
            let pixels = block(&frame, 0, 1);
            for ((p, _), v) in [(0, 128), (9, 5), (30, -3)].into_iter().zip(values) {
                assert_eq!(
                    pixels[usize::from(SCAN[p])],
                    100u8.wrapping_add(v as u8),
                    "{reading:?} {p}"
                );
            }
            assert_eq!(pixels[1], 100);
        }
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
        for (b, &width) in WIDTH.iter().enumerate() {
            let (count, values) = bundles.get(b).copied().unwrap_or((0, &[]));
            w.put(count, COUNT_BITS);
            for &v in values {
                w.put(v, width);
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
    fn an_eleven_bit_bundle_counts_two_bytes_a_value() {
        // E17 M5 T8b1, R2a-3: a DC takes two of its bundle's 64 bytes, so 32 fit one block's
        // buffer and 33 overflow it, intra and inter alike.
        for bundle in [INTRA_DC, INTER_DC] {
            let mut d = decoder(8, 8);
            let fits = [0; 32];
            let mut counts: Vec<(u32, &[u32])> = vec![(0, &[][..]); BUNDLES];
            counts[bundle] = (32, &fits[..]);
            let packet = by_hand(&counts, &BitWriter::new());
            assert!(d.decode(&packet.into_bytes()).is_ok(), "bundle {bundle}");
            let over = [0; 33];
            counts[bundle] = (33, &over[..]);
            let packet = by_hand(&counts, &BitWriter::new());
            assert_eq!(
                d.decode(&packet.into_bytes()),
                Err(Error::Invalid("bundle overflow")),
                "bundle {bundle}"
            );
        }
    }

    #[test]
    fn an_empty_packet_gives_no_frame_and_changes_nothing() {
        // E17 M5 T8b1, R2a's doubt c: FFmpeg's `avcodec_send_packet` refuses it (PyAV: EINVAL),
        // and the next frame decodes as without it, the key frame's bias included.
        let first = build::frame(8, 8, [&[Block::Fill(40)], NONE, NONE]);
        let second = build::frame(8, 8, [&[Block::Motion { x: 0, y: 1 }], NONE, NONE]);
        let mut with = decoder(8, 8);
        with.decode(&first).unwrap();
        let before = with.frame().clone();
        assert_eq!(with.decode(&[]), Err(Error::Invalid("empty packet")));
        assert_eq!(with.frame(), &before);
        let mut without = decoder(8, 8);
        without.decode(&first).unwrap();
        assert_eq!(with.decode(&second), without.decode(&second));
        let header = Header::new(8, 8);
        let mut audio = header.clone();
        audio.audio_tracks = 1;
        let tracked = |video: &[u8]| [&[0u8, 0, 0, 0][..], video].concat();
        let skipped = build::file(&audio, &[tracked(&first), tracked(&[]), tracked(&second)]);
        let plain = build::file(&header, &[first, second]);
        assert_eq!(check::frames(&skipped), check::frames(&plain));
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
            "all 0 before the first frame"
        );
    }

    /// The samples' frames as the game's `BINKW32.DLL` decodes them, read as it reads
    /// ([`Reading::Original`]): the hashes of its own planes (`bink-check.sh` at E17 M6 T6c1,
    /// `binkprobe frames` in YUY2), seeds 1 to 8 checked there. Fixed, a residue starting at
    /// 128 reads as the format means (CLT-Q68): each of these seeds has one.
    #[test]
    fn the_samples_decode_as_the_dll_decodes_them() {
        for (seed, dll, fixed) in [
            (1, 0x0ded_f76e_9553_c246_u64, 0xd72f_eb4d_f297_7af0_u64),
            (2, 0x06bf_035a_c01f_6294, 0x327a_3f4d_0b16_c206),
            (3, 0x4e3d_9476_fd5a_e872, 0x1a37_c79f_56ff_65e8),
        ] {
            let bytes = build::sample(seed);
            let frames = check::frames_read(&bytes, Reading::Original).unwrap();
            assert_eq!(frames.len(), build::SAMPLE_FRAMES);
            assert_eq!(check::hash(&frames), dll, "seed {seed}");
            let frames = check::frames(&bytes).unwrap();
            assert_eq!(check::hash(&frames), fixed, "seed {seed}, fixed");
        }
    }

    /// The samples at the game's clips' size, 256 × 120 (E17 M5 T8b1, R2a-4): the chroma's
    /// block grid holds four rows below the picture, which motion vectors read and write, as the
    /// DLL does (its planes' hashes, E17 M6 T6c1).
    #[test]
    fn the_samples_at_the_games_size_decode_as_the_dll_decodes_them() {
        for (seed, hash) in [
            (1, 0x7d9e_01ff_e109_cb39_u64),
            (2, 0xfb9a_df33_3f31_416c),
            (3, 0xfba7_de40_7f9b_a8fe),
        ] {
            let frames =
                check::frames_read(&build::sample_sized(seed, 256, 120), Reading::Original)
                    .unwrap();
            assert_eq!(frames.len(), build::SAMPLE_FRAMES);
            assert_eq!(frames[0].planes()[1].pixels().len(), 128 * 64);
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
