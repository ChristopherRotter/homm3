//! Synthetic Smacker files for tests (the writer's side of [`crate::Clip`] and
//! [`crate::Decoder`]; *provisional*, E27 M2 T1): a bit writer, the header's trees from their
//! shapes, a palette's change from its operations, a file around given frames, and [`sample`],
//! a clip of every block type and palette operation in random variations, whose decoding
//! `SMACKW32.DLL`'s checks (`h3-view video --smacker --sample`, [`crate::check`]).
//!
//! The writer does not validate: a test gets what it asks for (a tree past FFmpeg's limits, a
//! palette copy past its end, a code for a value no leaf holds: the first leaf's), so that the
//! decoder's checks can be tested. The trees' sizes are RAD's (four bytes an entry, three entries
//! more for the cache, as every one of the game's clips has them).

use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;

use crate::bits::Bits;
use crate::clip::{Track, Version};
use crate::tree::{COLORS, FULL, MAP, TYPES, Trees};
use crate::video::{MONO, RUNS, SKIP};

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
            self.flag((value >> k) & 1 == 1);
        }
    }

    /// One bit.
    pub fn flag(&mut self, flag: bool) {
        if self.bits.is_multiple_of(8) {
            self.bytes.push(0);
        }
        if flag && let Some(last) = self.bytes.last_mut() {
            *last |= 1 << (self.bits % 8);
        }
        self.bits += 1;
    }

    /// The bits written, the last byte's unused bits 0.
    #[must_use]
    pub fn bytes(self) -> Vec<u8> {
        self.bytes
    }
}

/// A tree's shape: a leaf's value, or a node's branches (the 0 branch first).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Shape {
    /// A leaf.
    Leaf(u32),
    /// A node.
    Node(Box<Shape>, Box<Shape>),
}

impl Shape {
    /// A node of the two branches.
    #[must_use]
    pub fn node(zero: Self, one: Self) -> Self {
        Self::Node(Box::new(zero), Box::new(one))
    }

    /// A tree of `values`' leaves in order, balanced (an empty list: a leaf of 0).
    #[must_use]
    pub fn balanced(values: &[u32]) -> Self {
        match values {
            [] => Self::Leaf(0),
            [value] => Self::Leaf(*value),
            _ => {
                let (zero, one) = values.split_at(values.len() / 2);
                Self::node(Self::balanced(zero), Self::balanced(one))
            }
        }
    }

    /// Its entries (nodes and leaves).
    #[must_use]
    pub fn entries(&self) -> u32 {
        match self {
            Self::Leaf(_) => 1,
            Self::Node(zero, one) => 1 + zero.entries() + one.entries(),
        }
    }

    /// Its leaves' values in order.
    fn leaves(&self, out: &mut Vec<u32>) {
        match self {
            Self::Leaf(value) => out.push(*value),
            Self::Node(zero, one) => {
                zero.leaves(out);
                one.leaves(out);
            }
        }
    }

    /// The path to the first leaf holding `value`.
    fn path(&self, value: u32) -> Option<Vec<bool>> {
        match self {
            Self::Leaf(v) => (*v == value).then(Vec::new),
            Self::Node(zero, one) => {
                let (first, mut path) = match zero.path(value) {
                    Some(path) => (false, path),
                    None => (true, one.path(value)?),
                };
                path.insert(0, first);
                Some(path)
            }
        }
    }

    /// Writes the bits of the path to `value`'s leaf (the first leaf's: all 0s, when none
    /// holds it); none for a leaf.
    fn code(&self, w: &mut BitWriter, value: u32) {
        let path = self.path(value).unwrap_or_else(|| self.zeros());
        for bit in path {
            w.flag(bit);
        }
    }

    /// The path to the first leaf.
    fn zeros(&self) -> Vec<bool> {
        match self {
            Self::Leaf(_) => Vec::new(),
            Self::Node(zero, _) => {
                let mut path = zero.zeros();
                path.insert(0, false);
                path
            }
        }
    }

    /// Writes a byte tree (a leaf: 0 and its 8 bits; a node: 1, then its branches).
    fn write_bytes(&self, w: &mut BitWriter) {
        match self {
            Self::Leaf(value) => {
                w.flag(false);
                w.put(*value, 8);
            }
            Self::Node(zero, one) => {
                w.flag(true);
                zero.write_bytes(w);
                one.write_bytes(w);
            }
        }
    }
}

/// A header tree: its byte trees (`None`: left out, every byte 0), its three escapes (the
/// values whose leaves hold the cache) and its shape of 16-bit values.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeSpec {
    /// The low bytes' tree.
    pub low: Option<Shape>,
    /// The high bytes' tree.
    pub high: Option<Shape>,
    /// The escapes (16 bits each).
    pub escapes: [u32; 3],
    /// The values' tree.
    pub tree: Shape,
}

impl TreeSpec {
    /// A tree of `values`' leaves, balanced, its byte trees balanced over the bytes the values
    /// hold (in their first order), its escapes 0xFFFD to 0xFFFF (a value among them becomes a
    /// cache's leaf).
    #[must_use]
    pub fn of(values: &[u32]) -> Self {
        let bytes = |shift: u32| {
            let mut seen = Vec::new();
            for value in values {
                let byte = (value >> shift) & 0xFF;
                if !seen.contains(&byte) {
                    seen.push(byte);
                }
            }
            Some(Shape::balanced(&seen))
        };
        Self {
            low: bytes(0),
            high: bytes(8),
            escapes: [0xFFFD, 0xFFFE, 0xFFFF],
            tree: Shape::balanced(values),
        }
    }

    /// The header's size of the tree (RAD's: four bytes an entry, the cache's three counted).
    #[must_use]
    pub fn size(&self) -> u32 {
        4 * (self.tree.entries() + 3)
    }

    /// Writes the tree (after its present bit).
    pub(crate) fn write(&self, w: &mut BitWriter) {
        for bytes in [&self.low, &self.high] {
            match bytes {
                None => w.flag(false),
                Some(shape) => {
                    w.flag(true);
                    shape.write_bytes(w);
                    w.flag(false);
                }
            }
        }
        for escape in self.escapes {
            w.put(escape, 16);
        }
        self.write_values(w, &self.tree);
        w.flag(false);
    }

    fn write_values(&self, w: &mut BitWriter, shape: &Shape) {
        match shape {
            Shape::Leaf(value) => {
                w.flag(false);
                for (bytes, byte) in [(&self.low, value & 0xFF), (&self.high, value >> 8)] {
                    if let Some(tree) = bytes {
                        tree.code(w, byte);
                    }
                }
            }
            Shape::Node(zero, one) => {
                w.flag(true);
                self.write_values(w, zero);
                self.write_values(w, one);
            }
        }
    }
}

/// A file's header: what the decoder reads of it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Header {
    /// The signature's version.
    pub version: Version,
    /// The frame's width and height in pixels.
    pub width: u32,
    /// See [`Header::width`].
    pub height: u32,
    /// The frame time (the header's word).
    pub rate: i32,
    /// Whether the last frame is a ring frame (the header's flag).
    pub ring: bool,
    /// The audio tracks (their largest packet: the writer's count).
    pub tracks: [Track; 7],
    /// The four trees (map, colours, full, types); `None` left out.
    pub trees: [Option<TreeSpec>; 4],
}

impl Header {
    /// An `SMK2` header of 15 frames a second without sound, its trees `trees`.
    #[must_use]
    pub fn new(width: u32, height: u32, trees: [Option<TreeSpec>; 4]) -> Self {
        Self {
            version: Version::Smk2,
            width,
            height,
            rate: -6666,
            ring: false,
            tracks: [Track::default(); 7],
            trees,
        }
    }

    /// The trees' bytes.
    #[must_use]
    pub fn trees(&self) -> Vec<u8> {
        let mut w = BitWriter::new();
        for tree in &self.trees {
            w.flag(tree.is_some());
            if let Some(tree) = tree {
                tree.write(&mut w);
            }
        }
        w.bytes()
    }

    /// The trees' sizes in the header.
    #[must_use]
    pub fn tree_sizes(&self) -> [u32; 4] {
        self.trees.clone().map(|tree| tree.map_or(12, |t| t.size()))
    }
}

/// A palette change's operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    /// Keeps the next colours (1 to 128).
    Skip(u8),
    /// Copies colours (1 to 64) from the palette before, from colour `from` on.
    Copy {
        /// The colours copied.
        count: u8,
        /// The first copied.
        from: u8,
    },
    /// A new colour: red, green and blue of 0 to 63 (red's bits 6 and 7 must be clear; green's
    /// and blue's are ignored).
    New([u8; 3]),
}

/// A palette change's bytes (its size's byte left to [`file`]).
#[must_use]
pub fn palette(ops: &[Op]) -> Vec<u8> {
    let mut out = Vec::new();
    for op in ops {
        match *op {
            Op::Skip(n) => out.push(0x80 | (n.wrapping_sub(1) & 0x7F)),
            Op::Copy { count, from } => out.extend([0x40 | (count.wrapping_sub(1) & 0x3F), from]),
            Op::New(rgb) => out.extend(rgb),
        }
    }
    out
}

/// A frame: its palette change (the operations' bytes), its audio packets (a track and the
/// bytes after the packet's size, a decoded size first where the track has one), its video.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FrameSpec {
    /// Marked a key frame in its size.
    pub key: bool,
    /// The palette change's bytes ([`palette`]).
    pub palette: Option<Vec<u8>>,
    /// The audio packets, in the tracks' order.
    pub audio: Vec<(usize, Vec<u8>)>,
    /// The video's bytes.
    pub video: Vec<u8>,
}

impl FrameSpec {
    /// A frame of `video` alone.
    #[must_use]
    pub fn video(video: Vec<u8>) -> Self {
        Self {
            video,
            ..Self::default()
        }
    }

    /// The frame's bytes (padded to a multiple of 4), its size's word and its flags.
    fn bytes(&self) -> (Vec<u8>, u32, u8) {
        let mut out = Vec::new();
        let mut flags = 0u8;
        if let Some(change) = &self.palette {
            let size = (change.len() + 1).div_ceil(4);
            out.push(size as u8);
            out.extend(change);
            out.resize(4 * size, 0);
            flags |= 1;
        }
        for (track, bytes) in &self.audio {
            out.extend(((bytes.len() + 4) as u32).to_le_bytes());
            out.extend(bytes);
            flags |= 2 << track;
        }
        out.extend(&self.video);
        out.resize(out.len().next_multiple_of(4), 0);
        let size = out.len() as u32 | u32::from(self.key);
        (out, size, flags)
    }
}

/// A packed sound packet for `track` (E27 M2 T2; the writer's side of [`crate::sound::decode`]):
/// its bytes after the size word, as [`FrameSpec::audio`] holds them. Its decoded size, then
/// the flags (sound, stereo and 16-bit as the track's), then `trees` (16 bits: a channel's low
/// bytes' and high bytes' trees, `trees[2 * channel]` and `trees[2 * channel + 1]`; 8 bits:
/// `trees[channel]`), each after a 1 and before a 0 as RAD's encoder writes them (FFmpeg skips
/// both bits), then each channel's first sample of `first` (the last channel's first; a 16-bit
/// one byte-swapped, as FFmpeg reads it), then for each value of `steps` (the differences of the
/// samples after the first, the channels interleaved) the code of its low byte and then of its
/// high byte in that channel's trees (a code for a byte no leaf holds: the first leaf's).
#[must_use]
pub fn sound(track: Track, trees: &[Shape], first: [u32; 2], steps: &[u32]) -> Vec<u8> {
    let stereo = track.flags & 0x10 != 0;
    let sixteen = track.flags & 0x20 != 0;
    let channels = 1 + usize::from(stereo);
    let bytes = 1 + usize::from(sixteen);
    let mut w = BitWriter::new();
    w.flag(true);
    w.flag(stereo);
    w.flag(sixteen);
    for tree in trees.iter().take(channels * bytes) {
        w.flag(true);
        tree.write_bytes(&mut w);
        w.flag(false);
    }
    for &value in first[..channels].iter().rev() {
        if sixteen {
            w.put(u32::from((value as u16).swap_bytes()), 16);
        } else {
            w.put(value, 8);
        }
    }
    let leaf = |k: usize| trees.get(k).cloned().unwrap_or(Shape::Leaf(0));
    for (i, &step) in steps.iter().enumerate() {
        let channel = (i + channels) % channels;
        if sixteen {
            leaf(2 * channel).code(&mut w, step & 0xFF);
            leaf(2 * channel + 1).code(&mut w, (step >> 8) & 0xFF);
        } else {
            leaf(channel).code(&mut w, step & 0xFF);
        }
    }
    let size = ((channels + steps.len()) * bytes) as u32;
    let mut out = size.to_le_bytes().to_vec();
    out.extend(w.bytes());
    out
}

/// A random packed sound packet for `track`: random byte trees ([`byte_tree`]), first samples,
/// and up to 40 samples a channel (whose differences are pairs of the trees' leaves), whole
/// words of them: `SMACKW32.DLL` hands Miles whole words (0x10007DC1), so the last 1 to 3 bytes
/// of a track whose sound does not end on one never reach it (the game's 16-bit stereo clips
/// always do).
fn sample_sound(rng: &mut Rng, track: Track) -> Vec<u8> {
    let channels = 1 + usize::from(track.flags & 0x10 != 0);
    let sixteen = track.flags & 0x20 != 0;
    let count = channels * (1 + usize::from(sixteen));
    let trees: Vec<Shape> = (0..count).map(|_| byte_tree(rng, false)).collect();
    let leaves: Vec<Vec<u32>> = trees
        .iter()
        .map(|t| {
            let mut out = Vec::new();
            t.leaves(&mut out);
            out
        })
        .collect();
    let pick = |rng: &mut Rng, k: usize| leaves[k][rng.below(leaves[k].len() as u32) as usize];
    let first =
        [rng.below(1 << 16), rng.below(1 << 16)].map(|v| if sixteen { v } else { v & 0xFF });
    let unit = 4 / count;
    let per_channel = unit * (1 + rng.below(40 / unit as u32) as usize);
    let steps: Vec<u32> = (0..channels * (per_channel - 1))
        .map(|i| {
            let channel = i % channels;
            if sixteen {
                pick(rng, 2 * channel) | pick(rng, 2 * channel + 1) << 8
            } else {
                pick(rng, channel)
            }
        })
        .collect();
    sound(track, &trees, first, &steps)
}

/// A Smacker file of `frames` (the last the ring frame when the header says so).
#[must_use]
pub fn file(header: &Header, frames: &[FrameSpec]) -> Vec<u8> {
    let trees = header.trees();
    let mut tracks = header.tracks;
    for frame in frames {
        for (track, bytes) in &frame.audio {
            if let Some(track) = tracks.get_mut(*track) {
                // The largest decoded size, as RAD's encoder writes it (a packed packet's first
                // word; `SMACKW32.DLL` sizes its buffer by it), else the packet's.
                let size = match bytes.get(..4) {
                    Some(&[a, b, c, d]) if track.flags & 0x80 != 0 => {
                        u32::from_le_bytes([a, b, c, d])
                    }
                    _ => bytes.len() as u32 + 4,
                };
                track.largest = track.largest.max(size);
            }
        }
    }
    let frames: Vec<_> = frames.iter().map(FrameSpec::bytes).collect();
    let mut out = Vec::new();
    out.extend(match header.version {
        Version::Smk2 => *b"SMK2",
        Version::Smk4 => *b"SMK4",
    });
    let count = frames.len() - usize::from(header.ring && !frames.is_empty());
    for word in [header.width, header.height, count as u32] {
        out.extend(word.to_le_bytes());
    }
    out.extend(header.rate.to_le_bytes());
    out.extend(u32::from(header.ring).to_le_bytes());
    for track in &tracks {
        out.extend(track.largest.to_le_bytes());
    }
    out.extend((trees.len() as u32).to_le_bytes());
    for size in header.tree_sizes() {
        out.extend(size.to_le_bytes());
    }
    for track in &tracks {
        out.extend((track.rate & 0xFF_FFFF | u32::from(track.flags) << 24).to_le_bytes());
    }
    out.extend([0; 4]);
    for (_, size, _) in &frames {
        out.extend(size.to_le_bytes());
    }
    out.extend(frames.iter().map(|(_, _, flags)| flags));
    out.extend(&trees);
    for (bytes, _, _) in &frames {
        out.extend(bytes);
    }
    out
}

/// A clip's video written code by code: each code the path to the first leaf of a value in a
/// tree of the header's (none for a tree left out).
#[derive(Debug, Clone)]
pub struct Video<'h> {
    header: &'h Header,
    w: BitWriter,
}

/// The trees' indices: the map's, the colours', the full blocks', the types'.
pub const TREES: [usize; 4] = [MAP, COLORS, FULL, TYPES];

impl<'h> Video<'h> {
    /// An empty video for `header`'s trees.
    #[must_use]
    pub fn new(header: &'h Header) -> Self {
        Self {
            header,
            w: BitWriter::new(),
        }
    }

    /// The code of `value` in tree `tree` (0 the map's to 3 the types', [`TREES`]).
    pub fn code(&mut self, tree: usize, value: u32) -> &mut Self {
        if let Some(Some(spec)) = self.header.trees.get(tree) {
            spec.tree.code(&mut self.w, value);
        }
        self
    }

    /// A bit (an `SMK4` full block's mode).
    pub fn bit(&mut self, bit: bool) -> &mut Self {
        self.w.flag(bit);
        self
    }

    /// The video's bytes.
    #[must_use]
    pub fn bytes(&self) -> Vec<u8> {
        self.w.clone().bytes()
    }
}

/// The frames a [`sample`] has, its ring frame left out.
pub const SAMPLE_FRAMES: usize = 8;

/// SplitMix64.
#[derive(Debug, Clone)]
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// A number below `n` (`n` at least 1).
    fn below(&mut self, n: u32) -> u32 {
        (self.next() % u64::from(n)) as u32
    }

    fn chance(&mut self, one_in: u32) -> bool {
        self.below(one_in) == 0
    }
}

/// A random tree shape over `leaves` values (`leaves` at least 1) drawn by `value`.
fn shape(rng: &mut Rng, leaves: u32, value: &mut impl FnMut(&mut Rng) -> u32) -> Shape {
    if leaves <= 1 {
        return Shape::Leaf(value(rng));
    }
    let zero = 1 + rng.below(leaves - 1);
    let zero_shape = shape(rng, zero, value);
    Shape::node(zero_shape, shape(rng, leaves - zero, value))
}

/// A random byte tree: one leaf or up to 12 (`types`: low bytes biased to short runs, as a
/// types' tree's are).
fn byte_tree(rng: &mut Rng, types: bool) -> Shape {
    let leaves = if rng.chance(5) { 1 } else { 2 + rng.below(11) };
    shape(rng, leaves, &mut |rng| {
        if types && !rng.chance(4) {
            rng.below(4) | rng.below(8) << 2
        } else {
            rng.below(256)
        }
    })
}

/// A random header tree (`types`: the types' tree): both byte trees (`SMACKW32.DLL` fails
/// without either), its values pairs of their leaves, each escape the value of one of its leaves
/// or random.
fn tree_spec(rng: &mut Rng, types: bool) -> TreeSpec {
    let (low, high) = (byte_tree(rng, types), byte_tree(rng, false));
    let (mut lows, mut highs) = (Vec::new(), Vec::new());
    low.leaves(&mut lows);
    high.leaves(&mut highs);
    let pick = |rng: &mut Rng, from: &[u32]| from[rng.below(from.len() as u32) as usize];
    let leaves = 1 + rng.below(24);
    let tree = shape(rng, leaves, &mut |rng| {
        pick(rng, &lows) | pick(rng, &highs) << 8
    });
    let mut values = Vec::new();
    tree.leaves(&mut values);
    let escapes = [0; 3].map(|_| {
        if rng.chance(2) {
            pick(rng, &values)
        } else {
            rng.below(0x1_0000)
        }
    });
    TreeSpec {
        low: Some(low),
        high: Some(high),
        escapes,
        tree,
    }
}

/// A random palette change: operations over all 256 colours (the first frame's mostly new).
fn palette_change(rng: &mut Rng, first: bool) -> Vec<u8> {
    let mut ops = Vec::new();
    let mut count = 0u32;
    while count < 256 {
        let left = 256 - count;
        let n = match rng.below(if first { 8 } else { 3 }) {
            0 => {
                let n = 1 + rng.below(left.min(128));
                ops.push(Op::Skip(n as u8));
                n
            }
            1 if !first => {
                let n = 1 + rng.below(left.min(64));
                let from = rng.below(257 - n);
                ops.push(Op::Copy {
                    count: n as u8,
                    from: from as u8,
                });
                n
            }
            _ => {
                ops.push(Op::New([0; 3].map(|_| rng.below(64) as u8)));
                1
            }
        };
        count += n;
    }
    palette(&ops)
}

/// A random walk to a leaf of tree `index`: its bits written, the value the decoder gets.
fn walk(rng: &mut Rng, trees: &mut Trees, index: usize, w: &mut BitWriter) -> u32 {
    let tree = trees.get_mut(index);
    let path = tree.path(|| rng.chance(2));
    let mut bits = BitWriter::new();
    for &bit in &path {
        w.flag(bit);
        bits.flag(bit);
    }
    let bytes = bits.bytes();
    tree.code(&mut Bits::new(&bytes)).unwrap_or(0)
}

/// A random frame's video over `trees` (`smk4`: the full blocks' modes written).
fn sample_video(rng: &mut Rng, trees: &mut Trees, blocks: u32, smk4: bool) -> Vec<u8> {
    let mut w = BitWriter::new();
    trees.reset();
    let mut block = 0;
    while block < blocks {
        let kind = walk(rng, trees, TYPES, &mut w);
        let mut run = RUNS[((kind >> 2) & 0x3F) as usize];
        let mut mode = 0;
        if kind & 3 == 1 && smk4 {
            mode = rng.below(3);
            w.flag(mode == 1);
            if mode != 1 {
                w.flag(mode == 2);
            }
        }
        while run > 0 && block < blocks {
            run -= 1;
            block += 1;
            let codes = match kind & 3 {
                MONO => {
                    walk(rng, trees, COLORS, &mut w);
                    walk(rng, trees, MAP, &mut w);
                    0
                }
                SKIP | 3 => 0,
                _ => [8, 2, 4][mode as usize],
            };
            for _ in 0..codes {
                walk(rng, trees, FULL, &mut w);
            }
        }
    }
    let mut video = w.bytes();
    if video.is_empty() {
        video.push(0);
    }
    video
}

/// A random `SMK2` clip ([`sample_version`]).
#[must_use]
pub fn sample(seed: u64) -> Vec<u8> {
    sample_version(seed, Version::Smk2)
}

/// A random clip of [`SAMPLE_FRAMES`] frames (and a ring frame one time in three) between 16 ×
/// 16 and 64 × 52 pixels: its trees random in shape, size and escapes, each left out one time
/// in eight; its blocks of every type, their runs mostly short; its palette changes of every
/// operation; one time in two a packed sound track (E27 M2 T2), its rate and its flags random
/// (8 or 16 bits, mono or stereo), a packet in each frame but the last and the ring frame
/// ([`sample_sound`]; as the game's clips end: a frame without one makes `SMACKW32.DLL` hand
/// Miles what it holds, 0x10005EBA, so the DLL's check gets every sample).
/// It keeps to what `SMACKW32.DLL` reads as FFmpeg does (E27 M2 T1): every tree has both byte
/// trees, and no frame is marked a key frame in its size (the DLL does not take the mark as
/// FFmpeg does; after either it decodes other pixels or reads memory it should not; no game clip
/// has one), so FFmpeg's handling of those is the unit tests' only.
#[must_use]
pub fn sample_version(seed: u64, version: Version) -> Vec<u8> {
    let mut rng = Rng(seed);
    let width = 4 * (4 + rng.below(13));
    let height = 4 * (4 + rng.below(10));
    let mut trees = [0; 4].map(|_| None);
    for k in TREES {
        // The types' tree whenever the others are left out: one tree at least.
        let needed = k == TYPES && trees.iter().all(Option::is_none);
        if !rng.chance(8) || needed {
            trees[k] = Some(tree_spec(&mut rng, k == TYPES));
        }
    }
    let mut header = Header::new(width, height, trees);
    header.version = version;
    header.ring = rng.chance(3);
    let sound = rng.chance(2);
    if sound {
        header.tracks[0] = Track {
            rate: [11025, 22050, 22050, 44100][rng.below(4) as usize],
            flags: 0xC0 | (rng.below(4) as u8) << 4,
            largest: 0,
        };
    }
    let mut decoded = Trees::read(header.tree_sizes(), &header.trees()).unwrap_or(Trees::absent());
    let blocks = (width / 4) * (height / 4);
    let count = SAMPLE_FRAMES + usize::from(header.ring);
    let frames: Vec<FrameSpec> = (0..count)
        .map(|n| FrameSpec {
            key: false,
            palette: (n == 0 || rng.chance(3)).then(|| palette_change(&mut rng, n == 0)),
            audio: if sound && n + 1 < SAMPLE_FRAMES {
                vec![(0, sample_sound(&mut rng, header.tracks[0]))]
            } else {
                Vec::new()
            },
            video: sample_video(&mut rng, &mut decoded, blocks, version == Version::Smk4),
        })
        .collect();
    file(&header, &frames)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Clip;

    #[test]
    fn the_writer_writes_what_it_is_asked() {
        assert_eq!(Shape::balanced(&[]), Shape::Leaf(0));
        assert_eq!(Shape::balanced(&[1, 2, 3]).entries(), 5);
        // A code for a value no leaf holds: the first leaf's.
        let mut w = BitWriter::new();
        Shape::node(Shape::node(Shape::Leaf(1), Shape::Leaf(2)), Shape::Leaf(3)).code(&mut w, 9);
        w.put(0b11, 2);
        assert_eq!(w.bytes(), [0b1100]);
        // A tree left out writes no code.
        let header = Header::new(4, 4, [None, None, None, Some(TreeSpec::of(&[1, 2]))]);
        let mut video = Video::new(&header);
        video.code(MAP, 1).code(7, 1).code(TYPES, 2).bit(true);
        assert_eq!(video.bytes(), [0b11]);
        assert_eq!(
            palette(&[
                Op::Skip(128),
                Op::Copy { count: 64, from: 3 },
                Op::New([1, 2, 3])
            ]),
            [0xFF, 0x7F, 3, 1, 2, 3]
        );
        // An audio packet of a track past the seventh: written, counted in no track.
        let frame = FrameSpec {
            audio: vec![(7, vec![1, 2])],
            ..FrameSpec::video(vec![1])
        };
        let bytes = file(&header, &[frame]);
        let clip = Clip::parse(&bytes).unwrap();
        assert!(clip.tracks.iter().all(|t| t.largest == 0));
        assert_eq!(clip.frames(), 1);
        assert!(
            file(
                &Header {
                    ring: true,
                    ..header
                },
                &[]
            )
            .len()
                > 104
        );
    }
}
