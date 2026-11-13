//! The video decoder (FFmpeg 6.1.1's `libavcodec/smacker.c`: `decode_init`, `decode_frame`):
//! the frame is cut into blocks of 4 × 4 pixels, row by row; each run of blocks has a type
//! (the types' tree gives the type, the run's length and a fill colour): *mono* (two colours
//! and a 16-bit map of which pixel takes which), *full* (eight codes of two pixels each; `SMK4`
//! adds blocks of 2 × 2 squares and of doubled rows), *skip* (the frame before's pixels kept)
//! and *fill* (one colour). A frame decodes over the frame before; the first over a frame of 0s.
//! The pixels right of and below the last whole block (a width or height not a multiple of 4)
//! are never written.

use alloc::vec;
use alloc::vec::Vec;

use crate::bits::Bits;
use crate::clip::{Clip, Packet, Version};
use crate::tree::Trees;
use crate::{Error, MAX_HEIGHT, MAX_WIDTH};

/// The runs' lengths by the type's bits 2 to 7 (`block_runs`).
pub(crate) const RUNS: [u32; 64] = [
    1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26,
    27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50,
    51, 52, 53, 54, 55, 56, 57, 58, 59, 128, 256, 512, 1024, 2048,
];

/// The block types (`SmkBlockTypes`).
pub(crate) const MONO: u32 = 0;
pub(crate) const FULL: u32 = 1;
pub(crate) const SKIP: u32 = 2;

/// A decoded frame: 8-bit colour indices and the palette they index.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame {
    width: u32,
    height: u32,
    pixels: Vec<u8>,
    palette: [u8; 768],
    flags: u8,
    /// Which blocks the frame's decoding wrote (mono, full or fill: not skipped), row by row.
    written: Vec<bool>,
}

impl Frame {
    /// The frame's width in pixels.
    #[must_use]
    pub fn width(&self) -> u32 {
        self.width
    }

    /// The frame's height in pixels.
    #[must_use]
    pub fn height(&self) -> u32 {
        self.height
    }

    /// The colour indices, row by row (`width` a row).
    #[must_use]
    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }

    /// The palette: 256 colours of red, green and blue, each 0 to 255 (the file's 6-bit levels
    /// through FFmpeg's table).
    #[must_use]
    pub fn palette(&self) -> &[u8; 768] {
        &self.palette
    }

    /// Whether the palette changed since the frame before (the packet's bit 0).
    #[must_use]
    pub fn palette_changed(&self) -> bool {
        self.flags & 1 != 0
    }

    /// Whether the frame is a key frame (the packet's bit 1: the first, or marked in its size).
    #[must_use]
    pub fn key(&self) -> bool {
        self.flags & 2 != 0
    }

    /// Which blocks of 4 × 4 pixels the frame wrote (a mono, full or fill block: not a skipped
    /// one), row by row, `width / 4` a row: the original's 16-bit copy converts only these
    /// through the frame's palette (`SMACKW32.DLL`, E27 M2 T1), keeping the others' colours.
    #[must_use]
    pub fn written(&self) -> &[bool] {
        &self.written
    }
}

/// Decodes a clip's frames in order, each over the one before.
#[derive(Debug, Clone)]
pub struct Decoder {
    smk4: bool,
    trees: Trees,
    /// Blocks a row and in all.
    row: u32,
    blocks: u32,
    frame: Frame,
}

impl Decoder {
    /// A decoder for `clip`, its trees read (`decode_init`). Fails on a frame empty or larger
    /// than [`MAX_WIDTH`] × [`MAX_HEIGHT`] ([`Error::Size`]), without trees or with trees that
    /// break FFmpeg's checks ([`Error::Invalid`]).
    pub fn new(clip: &Clip<'_>) -> Result<Self, Error> {
        let (width, height) = (clip.width, clip.height);
        if width == 0 || height == 0 || width > MAX_WIDTH || height > MAX_HEIGHT {
            return Err(Error::Size);
        }
        if clip.trees().is_empty() {
            return Err(Error::Invalid("no trees"));
        }
        let trees = Trees::read(clip.tree_sizes, clip.trees())?;
        Ok(Self {
            smk4: clip.version == Version::Smk4,
            trees,
            row: width >> 2,
            blocks: (width >> 2) * (height >> 2),
            frame: Frame {
                width,
                height,
                pixels: vec![0; (width * height) as usize],
                palette: [0; 768],
                flags: 0,
                written: vec![false; ((width >> 2) * (height >> 2)) as usize],
            },
        })
    }

    /// The frame as decoded last.
    #[must_use]
    pub fn frame(&self) -> &Frame {
        &self.frame
    }

    /// Decodes `packet` over the frame before (`decode_frame`). A packet without video fails
    /// and changes nothing ([`Error::Invalid`]); a frame whose data break a check fails left
    /// partly decoded, its palette taken, as FFmpeg leaves it, and the next decodes over it.
    pub fn decode(&mut self, packet: &Packet<'_>) -> Result<&Frame, Error> {
        if packet.video.is_empty() {
            return Err(Error::Invalid("no video"));
        }
        self.frame.flags = packet.flags;
        self.frame.palette = packet.palette;
        self.trees.reset();
        self.frame.written.fill(false);
        let mut bits = Bits::new(packet.video);
        let mut block = 0;
        while block < self.blocks {
            let kind = self.trees.types.code(&mut bits)?;
            let mut run = RUNS[((kind >> 2) & 0x3F) as usize];
            let mode = match kind & 3 {
                FULL if self.smk4 => {
                    if bits.left() < 1 {
                        return Err(Error::Invalid("code past the end"));
                    }
                    if bits.bit() == 1 {
                        1
                    } else if bits.bit() == 1 {
                        2
                    } else {
                        0
                    }
                }
                _ => 0,
            };
            while run > 0 && block < self.blocks {
                run -= 1;
                match kind & 3 {
                    MONO => self.mono(&mut bits, block)?,
                    FULL => self.full(&mut bits, block, mode)?,
                    SKIP => {}
                    _ => self.fill(block, (kind >> 8) as u8),
                }
                if kind & 3 != SKIP
                    && let Some(written) = self.frame.written.get_mut(block as usize)
                {
                    *written = true;
                }
                block += 1;
            }
        }
        Ok(&self.frame)
    }

    /// Where block `block`'s first row starts in the pixels.
    fn start(&self, block: u32) -> usize {
        let (x, y) = (block % self.row, block / self.row);
        (y * 4 * self.frame.width + x * 4) as usize
    }

    /// Writes four pixels at `at` (inside the frame by construction: every block lies in it).
    fn put(&mut self, at: usize, pixels: [u8; 4]) {
        if let Some(row) = self.frame.pixels.get_mut(at..at + 4) {
            row.copy_from_slice(&pixels);
        }
    }

    /// The row after the one at `at`.
    fn below(&self, at: usize) -> usize {
        at + self.frame.width as usize
    }

    fn mono(&mut self, bits: &mut Bits<'_>, block: u32) -> Result<(), Error> {
        let colors = self.trees.colors.code(bits)?;
        let mut map = self.trees.map.code(bits)?;
        let (high, low) = ((colors >> 8) as u8, colors as u8);
        let mut at = self.start(block);
        for _ in 0..4 {
            let pick = |k: u32| if map & (1 << k) != 0 { high } else { low };
            self.put(at, [pick(0), pick(1), pick(2), pick(3)]);
            map >>= 4;
            at = self.below(at);
        }
        Ok(())
    }

    fn full(&mut self, bits: &mut Bits<'_>, block: u32, mode: u32) -> Result<(), Error> {
        let mut at = self.start(block);
        match mode {
            0 => {
                for _ in 0..4 {
                    let right = self.trees.full.code(bits)?.to_le_bytes();
                    let left = self.trees.full.code(bits)?.to_le_bytes();
                    self.put(at, [left[0], left[1], right[0], right[1]]);
                    at = self.below(at);
                }
            }
            1 => {
                for _ in 0..2 {
                    let [a, b, ..] = self.trees.full.code(bits)?.to_le_bytes();
                    for _ in 0..2 {
                        self.put(at, [a, a, b, b]);
                        at = self.below(at);
                    }
                }
            }
            _ => {
                for _ in 0..2 {
                    let right = self.trees.full.code(bits)?.to_le_bytes();
                    let left = self.trees.full.code(bits)?.to_le_bytes();
                    for _ in 0..2 {
                        self.put(at, [left[0], left[1], right[0], right[1]]);
                        at = self.below(at);
                    }
                }
            }
        }
        Ok(())
    }

    fn fill(&mut self, block: u32, color: u8) {
        let mut at = self.start(block);
        for _ in 0..4 {
            self.put(at, [color; 4]);
            at = self.below(at);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bits::Bits;
    use crate::build::{self, FrameSpec, Header, TreeSpec, Video};
    use crate::check;
    use crate::tree::{COLORS, FULL as FULL_TREE, MAP, TYPES};
    use alloc::vec;
    use proptest::prelude::*;

    /// A block type's value: its type, its run's index, its fill colour.
    fn kind(kind: u32, run: u32, color: u32) -> u32 {
        kind | run << 2 | color << 8
    }

    fn header(width: u32, height: u32, values: [&[u32]; 4]) -> Header {
        Header::new(
            width,
            height,
            values.map(|v| (!v.is_empty()).then(|| TreeSpec::of(v))),
        )
    }

    /// Every frame of a clip of `header` and the videos `videos` wrote.
    fn decode(header: &Header, videos: &[Vec<u8>]) -> Vec<Result<Frame, Error>> {
        let frames: Vec<_> = videos.iter().cloned().map(FrameSpec::video).collect();
        let bytes = build::file(header, &frames);
        let clip = Clip::parse(&bytes).unwrap();
        let mut decoder = Decoder::new(&clip).unwrap();
        clip.reader()
            .map(|packet| decoder.decode(&packet.unwrap()).cloned())
            .collect()
    }

    #[test]
    fn mono_blocks_pick_their_two_colours_by_their_map() {
        let h = header(
            8,
            4,
            [&[0xA5C3, 0x000F], &[0x1234], &[], &[kind(MONO, 1, 0)]],
        );
        let mut v = Video::new(&h);
        v.code(COLORS, 0x1234)
            .code(MAP, 0xA5C3)
            .code(COLORS, 0x1234)
            .code(MAP, 0x000F);
        let frames = decode(&h, &[v.bytes()]);
        let frame = frames[0].as_ref().unwrap();
        let (hi, lo) = (0x12, 0x34);
        assert_eq!(
            frame.pixels(),
            [
                hi, hi, lo, lo, hi, hi, hi, hi, //
                lo, lo, hi, hi, lo, lo, lo, lo, //
                hi, lo, hi, lo, lo, lo, lo, lo, //
                lo, hi, lo, hi, lo, lo, lo, lo,
            ]
        );
        assert_eq!(frame.written(), [true, true]);
        assert!(frame.key() && !frame.palette_changed());
        assert_eq!(
            (frame.width(), frame.height(), frame.palette()),
            (8, 4, &[0; 768])
        );
    }

    #[test]
    fn full_blocks_take_pixel_pairs_right_first() {
        let pairs = [
            0x0201, 0x0403, 0x0605, 0x0807, 0x0A09, 0x0C0B, 0x0E0D, 0x100F,
        ];
        let h = header(4, 4, [&[], &[], &pairs, &[kind(FULL, 0, 0)]]);
        let mut v = Video::new(&h);
        for pair in pairs {
            v.code(FULL_TREE, pair);
        }
        let frames = decode(&h, &[v.bytes()]);
        assert_eq!(
            frames[0].as_ref().unwrap().pixels(),
            [3, 4, 1, 2, 7, 8, 5, 6, 11, 12, 9, 10, 15, 16, 13, 14]
        );
    }

    #[test]
    fn smk4_adds_full_blocks_of_squares_and_doubled_rows() {
        let pairs = [0x0201, 0x0403, 0x0605, 0x0807];
        let mut h = header(12, 4, [&[], &[], &pairs, &[kind(FULL, 0, 0)]]);
        h.version = Version::Smk4;
        let mut v = Video::new(&h);
        // Mode 1 (bit 1): two codes, each 2 × 2 squares of its bytes for two rows.
        v.code(TYPES, kind(FULL, 0, 0))
            .bit(true)
            .code(FULL_TREE, pairs[0])
            .code(FULL_TREE, pairs[1]);
        // Mode 2 (bits 0, 1): two pairs of codes, each a doubled row.
        v.code(TYPES, 0).bit(false).bit(true);
        for pair in pairs {
            v.code(FULL_TREE, pair);
        }
        // Mode 0 (bits 0, 0): eight codes.
        v.bit(false).bit(false);
        for pair in pairs.iter().chain(&pairs) {
            v.code(FULL_TREE, *pair);
        }
        let frames = decode(&h, &[v.bytes()]);
        let p = frames[0].as_ref().unwrap().pixels();
        let rows: Vec<&[u8]> = p.chunks(12).collect();
        assert_eq!(rows[0], [1, 1, 2, 2, 3, 4, 1, 2, 3, 4, 1, 2]);
        assert_eq!(rows[1], [1, 1, 2, 2, 3, 4, 1, 2, 7, 8, 5, 6]);
        assert_eq!(rows[2], [3, 3, 4, 4, 7, 8, 5, 6, 3, 4, 1, 2]);
        assert_eq!(rows[3], [3, 3, 4, 4, 7, 8, 5, 6, 7, 8, 5, 6]);
        // A mode's bit past the end fails: 32 blocks of mode 1 (one bit each, their codes of a
        // tree of one leaf none) take the four bytes, the 33rd has no bit for its mode.
        let mut h = header(4 * 33, 4, [&[], &[], &[5], &[kind(FULL, 0, 0)]]);
        h.version = Version::Smk4;
        let frames = decode(&h, &[vec![0xFF; 4]]);
        assert_eq!(frames[0], Err(Error::Invalid("code past the end")));
    }

    #[test]
    fn skip_and_fill_blocks_and_runs() {
        // A fill of 3 blocks, then a skip of 128 cut at the frame's end; the next frame fills
        // one block and skips the rest, which keep the first frame's colour.
        let h = header(
            16,
            8,
            [
                &[],
                &[],
                &[],
                &[kind(3, 2, 0x77), kind(SKIP, 59, 0), kind(3, 0, 0x55)],
            ],
        );
        let mut first = Video::new(&h);
        first
            .code(TYPES, kind(3, 2, 0x77))
            .code(TYPES, kind(SKIP, 59, 0));
        let mut second = Video::new(&h);
        second
            .code(TYPES, kind(3, 0, 0x55))
            .code(TYPES, kind(SKIP, 59, 0));
        let frames = decode(&h, &[first.bytes(), second.bytes()]);
        let one = frames[0].as_ref().unwrap();
        assert_eq!(
            one.written(),
            [true, true, true, false, false, false, false, false]
        );
        assert_eq!(
            &one.pixels()[..16],
            &[0x77; 12]
                .iter()
                .chain(&[0; 4])
                .copied()
                .collect::<Vec<_>>()[..]
        );
        assert_eq!(one.pixels()[16 * 4..], [0; 64]);
        let two = frames[1].as_ref().unwrap();
        assert_eq!(
            two.written(),
            [true, false, false, false, false, false, false, false]
        );
        assert_eq!(
            &two.pixels()[..16],
            [
                0x55, 0x55, 0x55, 0x55, 0x77, 0x77, 0x77, 0x77, 0x77, 0x77, 0x77, 0x77, 0, 0, 0, 0
            ]
        );
    }

    #[test]
    fn a_frame_that_fails_is_left_partly_decoded() {
        let h = header(8, 4, [&[], &[], &[], &[kind(3, 0, 9), kind(MONO, 0, 0)]]);
        let mut v = Video::new(&h);
        // A fill, then a mono block with the colours' and map's trees left out: they read no
        // bit, so the frame decodes; then a frame whose second type's code is cut.
        v.code(TYPES, kind(3, 0, 9)).code(TYPES, kind(MONO, 0, 0));
        let mut cut = Video::new(&h);
        cut.code(TYPES, kind(3, 0, 4));
        let mut cut = cut.bytes();
        cut.truncate(1);
        // The first type's code is one bit: the frame has one byte, whose 7 further bits are 0s
        // (the fill again), then no bit is left.
        let frames = decode(&h, &[v.bytes(), cut]);
        assert_eq!(
            frames[0].as_ref().unwrap().pixels()[..8],
            [9, 9, 9, 9, 0, 0, 0, 0]
        );
        assert!(frames[1].is_ok());
        // An empty packet fails and changes nothing.
        let bytes = build::file(&h, &[FrameSpec::video(v.bytes())]);
        let clip = Clip::parse(&bytes).unwrap();
        let mut decoder = Decoder::new(&clip).unwrap();
        let mut packet = clip.reader().next().unwrap().unwrap();
        decoder.decode(&packet).unwrap();
        let before = decoder.frame().clone();
        packet.video = &[];
        packet.flags = 1;
        assert_eq!(decoder.decode(&packet), Err(Error::Invalid("no video")));
        assert_eq!(decoder.frame(), &before);
        // Each code of a block cut: the colours', then the map's (the colours' tree one leaf,
        // read with no bit), then a full block's of each mode, the first or the second code of
        // a pair (each code a byte: 256 leaves).
        let wide: Vec<u32> = (0..256).collect();
        let decoder = |values: [&[u32]; 4]| {
            let bytes = build::file(&header(4, 4, values), &[]);
            Decoder::new(&Clip::parse(&bytes).unwrap()).unwrap()
        };
        let cut = Err(Error::Invalid("code past the end"));
        let mut empty = Bits::new(&[]);
        assert_eq!(
            decoder([&[1, 2], &[3, 4], &[], &[0]]).mono(&mut empty, 0),
            cut
        );
        assert_eq!(decoder([&[1, 2], &[3], &[], &[0]]).mono(&mut empty, 0), cut);
        let mut full = decoder([&[], &[], &wide, &[0]]);
        for (mode, bytes) in [(0, &[][..]), (0, &[7]), (1, &[]), (2, &[]), (2, &[7])] {
            assert_eq!(
                full.full(&mut Bits::new(bytes), 0, mode),
                cut,
                "{mode} {bytes:?}"
            );
        }
        // Byte 1's bits read lowest first walk to leaf 128, byte 2's to leaf 64: the first code
        // the right pair, the second the left, each pair on two rows.
        assert_eq!(full.full(&mut Bits::new(&[1, 2, 3, 4]), 0, 2), Ok(()));
        assert_eq!(full.frame().pixels()[..8], [64, 0, 128, 0, 64, 0, 128, 0]);
    }

    #[test]
    fn frames_outside_the_bounds_are_refused() {
        let trees = |w, h| Header::new(w, h, [None, None, None, Some(TreeSpec::of(&[0]))]);
        for (w, h) in [(0, 4), (4, 0), (MAX_WIDTH + 1, 4), (4, MAX_HEIGHT + 1)] {
            let bytes = build::file(&trees(w, h), &[]);
            assert_eq!(
                Decoder::new(&Clip::parse(&bytes).unwrap()).err(),
                Some(Error::Size)
            );
        }
        let bytes = build::file(&trees(MAX_WIDTH, MAX_HEIGHT), &[]);
        assert!(Decoder::new(&Clip::parse(&bytes).unwrap()).is_ok());
        // No trees; trees FFmpeg refuses.
        let mut bare = build::file(&trees(4, 4), &[]);
        bare[52..56].copy_from_slice(&0u32.to_le_bytes());
        bare.truncate(104);
        assert_eq!(
            Decoder::new(&Clip::parse(&bare).unwrap()).err(),
            Some(Error::Invalid("no trees"))
        );
        let none = Header::new(4, 4, [None, None, None, None]);
        let bytes = build::file(&none, &[]);
        assert_eq!(
            Decoder::new(&Clip::parse(&bytes).unwrap()).err(),
            Some(Error::Invalid("header trees"))
        );
    }

    #[test]
    fn sizes_not_a_multiple_of_4_leave_their_edges() {
        let h = header(6, 5, [&[], &[], &[], &[kind(3, 0, 1)]]);
        let frames = decode(&h, &[vec![0]]);
        let p = frames[0].as_ref().unwrap().pixels();
        assert_eq!(p.len(), 30);
        for (at, &pixel) in p.iter().enumerate() {
            assert_eq!(pixel, u8::from(at % 6 < 4 && at / 6 < 4), "{at}");
        }
        // Writes outside the frame do nothing (never reached).
        let mut d = Decoder::new(&Clip::parse(&build::file(&h, &[])).unwrap()).unwrap();
        d.put(30, [1; 4]);
        assert_eq!(d.frame().pixels(), [0; 30]);
    }

    #[test]
    fn samples_decode_as_smacker_and_ffmpeg_decode_them() {
        // The frames' hashes of the samples, each checked against SMACKW32.DLL's and FFmpeg
        // 6.1.1's decoding at E27 M2 T1.
        let hashes: Vec<u64> = (1..=8)
            .map(|seed| check::hash(&check::frames(&build::sample(seed)).unwrap()))
            .collect();
        assert_eq!(hashes, SAMPLE_HASHES);
        // Version 4's (FFmpeg 6.1.1's; the DLL reads version 2 only), of seeds whose clips have
        // full blocks (in the others version 4 decodes as version 2).
        let hashes: Vec<u64> = [2, 5, 6, 7]
            .into_iter()
            .map(|seed| {
                check::hash(&check::frames(&build::sample_version(seed, Version::Smk4)).unwrap())
            })
            .collect();
        assert_eq!(hashes, SAMPLE4_HASHES);
    }

    const SAMPLE_HASHES: [u64; 8] = [
        0x950F4B9EE1B1A3C2,
        0x83621BE2E5B230A8,
        0x559A5D305925BD36,
        0x0B8DD8B23A08C10C,
        0xFF0B362DF1FEDC77,
        0x789252D101EC3D96,
        0xDC8A090FBF6C5C34,
        0x31A2671B6F3DFA78,
    ];
    const SAMPLE4_HASHES: [u64; 4] = [
        0xE28DDE104696A6BF,
        0x753F6450C11DF9D9,
        0xFF64D3558022FC40,
        0x83E7DBFEA2073BDA,
    ];

    proptest! {
        #[test]
        fn decoding_never_panics(bytes in proptest::collection::vec(any::<u8>(), 0..64), seed in 0u64..64) {
            let sample = build::sample(seed);
            if let Ok(clip) = Clip::parse(&sample)
                && let Ok(mut decoder) = Decoder::new(&clip)
            {
                for packet in clip.reader().flatten() {
                    let packet = Packet { video: &bytes, ..packet };
                    let _ = decoder.decode(&packet);
                }
            }
        }

        #[test]
        fn damaged_samples_never_panic(seed in 0u64..1000, flips in proptest::collection::vec((any::<usize>(), any::<u8>()), 1..8), cut in any::<usize>()) {
            let mut bytes = build::sample(seed);
            for (at, xor) in flips {
                let n = bytes.len();
                bytes[at % n] ^= xor;
            }
            bytes.truncate(cut % (bytes.len() + 1));
            let _ = check::frames(&bytes);
        }
    }
}
