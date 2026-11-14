//! The Smacker file (FFmpeg 6.1.1's demuxer `libavformat/smacker.c`: `read_header` and
//! `read_packet`, its audio streams read: E27 M2 T2): a 104-byte header, the frames' sizes and flags,
//! the Huffman trees, then the frames, each its palette's change, its audio tracks' packets and
//! its video.
//!
//! The header's words: the signature (`SMK2` or `SMK4`), width, height, the frames' count, the
//! frame time (negative: in 10 µs, else in ms), the flags (bit 0: a ring frame follows the
//! frames, which turns the last into the first for a loop), each audio track's largest packet,
//! the trees' size in bytes, each tree's size (the decoder's tables), each audio track's rate
//! (three bytes) and flags (one), a word unused. A frame's size has its two lowest bits as
//! marks (bit 0 a key frame); its flags' byte has bit 0 for a palette change and bits 1 to 7
//! for the audio tracks with a packet in it.
//!
//! The frames are read in order, as FFmpeg's demuxer reads them ([`Reader`]): a palette change
//! is applied to the palette the frames before left (its entries copied from that palette,
//! skipped or new), and a frame whose data break a check gives an error, the next frame read
//! from where its size puts it. Past the file's end every byte reads as 0 (`avio_r8`), and a
//! frame's video or a track's packet running past it gives [`Error::Truncated`]
//! (`ffio_read_size`, `av_get_packet`). A packet of a track the header does not describe is
//! skipped (FFmpeg has no stream for it). FFmpeg's library gives a frame's packets one by one,
//! so a frame failing in a later track's packet has given the earlier tracks' first; here the
//! frame fails whole (no game clip has more than one track).

use alloc::vec::Vec;

use crate::Error;

/// The header's bytes.
const HEADER: usize = 104;
/// The signatures (`MKTAG('S', 'M', 'K', '2')` and `'4'`).
const SMK2: u32 = u32::from_le_bytes(*b"SMK2");
const SMK4: u32 = u32::from_le_bytes(*b"SMK4");
/// The most frames FFmpeg's demuxer takes, the ring frame counted.
const MAX_FRAMES: u32 = 0xFF_FFFF;
/// An audio track's flags: 16-bit samples, the Bink audio codecs, the packed (Smacker) one.
const AUDIO_16: u8 = 0x20;
const AUDIO_BINK: u8 = 0x08;
const AUDIO_DCT: u8 = 0x04;
const AUDIO_PACKED: u8 = 0x80;

/// The palette's 6-bit levels as 8-bit ones (`smk_pal`).
const LEVELS: [u8; 64] = [
    0x00, 0x04, 0x08, 0x0C, 0x10, 0x14, 0x18, 0x1C, 0x20, 0x24, 0x28, 0x2C, 0x30, 0x34, 0x38, 0x3C,
    0x41, 0x45, 0x49, 0x4D, 0x51, 0x55, 0x59, 0x5D, 0x61, 0x65, 0x69, 0x6D, 0x71, 0x75, 0x79, 0x7D,
    0x82, 0x86, 0x8A, 0x8E, 0x92, 0x96, 0x9A, 0x9E, 0xA2, 0xA6, 0xAA, 0xAE, 0xB2, 0xB6, 0xBA, 0xBE,
    0xC3, 0xC7, 0xCB, 0xCF, 0xD3, 0xD7, 0xDB, 0xDF, 0xE3, 0xE7, 0xEB, 0xEF, 0xF3, 0xF7, 0xFB, 0xFF,
];

/// The file's version: `SMK4` adds two modes of full blocks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Version {
    /// `SMK2`, the game's clips'.
    Smk2,
    /// `SMK4`.
    Smk4,
}

/// An audio track as the header describes it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Track {
    /// Samples a second (0: no track).
    pub rate: u32,
    /// Its flags' byte (0x80 packed, 0x40 present, 0x20 16-bit, 0x10 stereo, 0x08 and 0x04
    /// Bink's codecs).
    pub flags: u8,
    /// Its largest packet, as the header gives it.
    pub largest: u32,
}

impl Track {
    /// The bytes a packet of this track starts with that give its decoded size
    /// (`duration_size`): 4 but for a plain 16-bit track (and an absent one).
    fn duration_bytes(self) -> u32 {
        let plain = self.flags & (AUDIO_BINK | AUDIO_DCT | AUDIO_PACKED) == 0;
        if self.rate == 0 || (plain && self.flags & AUDIO_16 != 0) {
            0
        } else {
            4
        }
    }
}

/// A parsed Smacker file: its header, frame table and trees over the file's bytes.
#[derive(Debug, Clone)]
pub struct Clip<'a> {
    bytes: &'a [u8],
    /// The signature's version.
    pub version: Version,
    /// The frame's width and height in pixels.
    pub width: u32,
    /// See [`Clip::width`].
    pub height: u32,
    /// The frame time as the header gives it: negative in units of 10 µs (−6666: 66.66 ms),
    /// else in ms.
    pub rate: i32,
    /// The header's flags (bit 0: a ring frame).
    pub flags: u32,
    /// The audio tracks.
    pub tracks: [Track; 7],
    /// The trees' sizes (the map's, the colours', the full blocks', the types'), each the
    /// decoder's table size in bytes.
    pub tree_sizes: [u32; 4],
    /// The frames' sizes as the table gives them (their two lowest bits marks).
    sizes: Vec<u32>,
    /// The frames' flags.
    frame_flags: &'a [u8],
    /// The trees' bytes.
    trees: &'a [u8],
    /// Where the first frame starts.
    data: usize,
}

/// The little-endian word at `at`, 0 for each byte past the end.
fn word(bytes: &[u8], at: usize) -> u32 {
    let byte = |k: usize| u32::from(bytes.get(at.saturating_add(k)).copied().unwrap_or(0));
    byte(0) | byte(1) << 8 | byte(2) << 16 | byte(3) << 24
}

impl<'a> Clip<'a> {
    /// Parses the header, the frame table and the trees (`smacker_read_header`); the frames
    /// are read by [`Clip::reader`].
    pub fn parse(bytes: &'a [u8]) -> Result<Self, Error> {
        let version = match word(bytes, 0) {
            SMK2 => Version::Smk2,
            SMK4 => Version::Smk4,
            _ => return Err(Error::Magic),
        };
        if bytes.len() < HEADER {
            return Err(Error::Truncated);
        }
        let rate = word(bytes, 16) as i32;
        if rate > i32::MAX / 100 || rate == i32::MIN {
            return Err(Error::Field("frame time"));
        }
        let flags = word(bytes, 20);
        let frames = word(bytes, 12).wrapping_add(flags & 1);
        if frames > MAX_FRAMES {
            return Err(Error::Field("frames"));
        }
        let tree_bytes = word(bytes, 52);
        if tree_bytes >= u32::MAX / 4 {
            return Err(Error::Field("trees"));
        }
        let mut tracks = [Track::default(); 7];
        for (k, track) in tracks.iter_mut().enumerate() {
            let rate = word(bytes, 72 + 4 * k);
            *track = Track {
                rate: rate & 0xFF_FFFF,
                flags: (rate >> 24) as u8,
                largest: word(bytes, 24 + 4 * k),
            };
        }
        let count = frames as usize;
        let flags_at = HEADER + 4 * count;
        let trees_at = flags_at + count;
        let data = trees_at + tree_bytes as usize;
        let (Some(frame_flags), Some(trees)) =
            (bytes.get(flags_at..trees_at), bytes.get(trees_at..data))
        else {
            return Err(Error::Truncated);
        };
        Ok(Self {
            bytes,
            version,
            width: word(bytes, 4),
            height: word(bytes, 8),
            rate,
            flags,
            tracks,
            tree_sizes: [56, 60, 64, 68].map(|at| word(bytes, at)),
            sizes: (0..count).map(|n| word(bytes, HEADER + 4 * n)).collect(),
            frame_flags,
            trees,
            data,
        })
    }

    /// The frames' count, a ring frame included.
    #[must_use]
    pub fn frames(&self) -> usize {
        self.sizes.len()
    }

    /// Whether a ring frame follows the frames (the header's flag): the last of
    /// [`Clip::frames`].
    #[must_use]
    pub fn ring(&self) -> bool {
        self.flags & 1 == 1
    }

    /// The trees' bytes (the decoder's extradata after its sizes).
    #[must_use]
    pub fn trees(&self) -> &'a [u8] {
        self.trees
    }

    /// A reader of the frames from the first.
    #[must_use]
    pub fn reader(&self) -> Reader<'_, 'a> {
        Reader {
            clip: self,
            at: self.data as u64,
            frame: 0,
            palette: [0; 768],
            changed: false,
        }
    }
}

/// A frame's video as FFmpeg's demuxer gives it to the decoder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Packet<'a> {
    /// The frame's number.
    pub frame: usize,
    /// Bit 0: the palette changed since the last packet; bit 1: a key frame (the first frame,
    /// or bit 0 of its size).
    pub flags: u8,
    /// The palette, 256 colours of red, green and blue.
    pub palette: [u8; 768],
    /// Each track's packet in the frame: its bytes after the size word ([`crate::sound::decode`]
    /// takes them), `None` for a track with none.
    pub audio: [Option<&'a [u8]>; 7],
    /// The video's bytes.
    pub video: &'a [u8],
}

/// The frames in order (`smacker_read_packet`, the audio streams read): the palette is
/// the reader's, each frame's change applied to it.
#[derive(Debug, Clone)]
pub struct Reader<'c, 'a> {
    clip: &'c Clip<'a>,
    /// The position in the file (past its end after a skip).
    at: u64,
    frame: usize,
    palette: [u8; 768],
    /// A palette change not yet given in a packet.
    changed: bool,
}

impl<'a> Reader<'_, 'a> {
    /// The next byte (`avio_r8`): 0 past the end.
    fn byte(&mut self) -> u8 {
        let byte = usize::try_from(self.at)
            .ok()
            .and_then(|at| self.clip.bytes.get(at).copied());
        if byte.is_some() {
            self.at += 1;
        }
        byte.unwrap_or(0)
    }

    /// The next little-endian word (`avio_rl32`).
    fn word(&mut self) -> u32 {
        let mut value = 0;
        for k in 0..4 {
            value |= u32::from(self.byte()) << (8 * k);
        }
        value
    }

    /// Applies a palette change from here, `frame_size` the frame's bytes left; the bytes left
    /// after it.
    fn palette(&mut self, frame_size: u32) -> Result<u32, Error> {
        let old = self.palette;
        let size = u32::from(self.byte()) * 4;
        if size > frame_size {
            return Err(Error::Invalid("palette larger than its frame"));
        }
        // The change's end: its size counts the size's byte (a size of 0 ends before it).
        let end = (self.at + u64::from(size)).saturating_sub(1);
        // The colours set so far (indices below 256 by the loop's condition: direct indexing).
        let mut count = 0usize;
        while count < 256 {
            let t = self.byte();
            if t & 0x80 != 0 {
                count += usize::from(t & 0x7F) + 1;
            } else if t & 0x40 != 0 {
                let from = usize::from(self.byte());
                let n = usize::from(t & 0x3F) + 1;
                if from + n > 256 {
                    return Err(Error::Invalid("palette copy past its end"));
                }
                for k in 0..n.min(256 - count) {
                    let (to, from) = (3 * (count + k), 3 * (from + k));
                    self.palette[to..to + 3].copy_from_slice(&old[from..from + 3]);
                }
                count += n.min(256 - count);
            } else {
                let (g, b) = (self.byte(), self.byte());
                self.palette[3 * count..3 * count + 3].copy_from_slice(&[
                    LEVELS[usize::from(t)],
                    LEVELS[usize::from(g & 0x3F)],
                    LEVELS[usize::from(b & 0x3F)],
                ]);
                count += 1;
            }
        }
        self.at = end;
        self.changed = true;
        Ok(frame_size - size)
    }

    /// Reads the frame from here; its packet or why it failed.
    fn read(&mut self, frame: usize) -> Result<Packet<'a>, Error> {
        let size = self.clip.sizes.get(frame).copied().unwrap_or(0);
        let flags = self.clip.frame_flags.get(frame).copied().unwrap_or(0);
        let mut left = size & !3;
        if flags & 1 != 0 {
            left = self.palette(left)?;
        }
        let mut audio = [None; 7];
        for (k, track) in self.clip.tracks.iter().enumerate() {
            if flags & (2 << k) == 0 {
                continue;
            }
            let bytes = self.word();
            let duration = track.duration_bytes();
            if (bytes as i32) < 4 + duration as i32 || bytes > left {
                return Err(Error::Invalid("audio packet's size"));
            }
            left -= bytes;
            if track.rate == 0 {
                // No stream for it: its decoded size read, the rest skipped.
                if duration != 0 {
                    self.word();
                }
                self.at += u64::from(bytes - 4 - duration);
                continue;
            }
            let start = usize::try_from(self.at).unwrap_or(usize::MAX);
            let end = start.saturating_add((bytes - 4) as usize);
            audio[k] = Some(self.clip.bytes.get(start..end).ok_or(Error::Truncated)?);
            self.at = end as u64;
        }
        if left >= i32::MAX as u32 / 2 {
            return Err(Error::Invalid("video's size"));
        }
        let start = usize::try_from(self.at).unwrap_or(usize::MAX);
        let end = start.saturating_add(left as usize);
        let video = self.clip.bytes.get(start..end).ok_or(Error::Truncated)?;
        self.at = end as u64;
        let key = size & 1 != 0 || frame == 0;
        let packet = Packet {
            frame,
            flags: u8::from(self.changed) | u8::from(key) << 1,
            palette: self.palette,
            audio,
            video,
        };
        self.changed = false;
        Ok(packet)
    }
}

impl<'a> Iterator for Reader<'_, 'a> {
    type Item = Result<Packet<'a>, Error>;

    /// The next frame's packet, or why it failed (the frame after it read next, from where the
    /// failed frame's size ends it); `None` after the last.
    fn next(&mut self) -> Option<Self::Item> {
        let frame = self.frame;
        let size = *self.clip.sizes.get(frame)?;
        let next = self.at + u64::from(size & !3);
        self.frame += 1;
        let packet = self.read(frame);
        if packet.is_err() {
            self.at = next;
        }
        Some(packet)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build::{self, FrameSpec, Header, Op, TreeSpec};
    use alloc::boxed::Box;
    use alloc::vec;
    use proptest::prelude::*;

    fn header() -> Header {
        Header::new(8, 4, [None, None, None, Some(TreeSpec::of(&[1, 2]))])
    }

    fn file(header: &Header, frames: &[FrameSpec]) -> Vec<u8> {
        build::file(header, frames)
    }

    #[test]
    fn the_header_and_table_are_read() {
        let mut h = header();
        h.version = Version::Smk4;
        h.rate = 100;
        h.ring = true;
        h.tracks[2] = Track {
            rate: 22050,
            flags: 0xF0,
            largest: 0,
        };
        let frames = [
            FrameSpec::video(vec![1]),
            FrameSpec {
                key: true,
                audio: vec![(2, vec![7, 0, 0, 0, 1, 2])],
                ..FrameSpec::video(vec![2, 3])
            },
        ];
        let bytes = file(&h, &frames);
        let clip = Clip::parse(&bytes).unwrap();
        assert_eq!(
            (clip.version, clip.width, clip.height, clip.rate),
            (Version::Smk4, 8, 4, 100)
        );
        assert_eq!((clip.flags, clip.frames(), clip.ring()), (1, 2, true));
        assert_eq!(
            clip.tracks[2],
            Track {
                rate: 22050,
                flags: 0xF0,
                largest: 7
            }
        );
        assert_eq!(clip.tracks[0], Track::default());
        assert_eq!(clip.tree_sizes, h.tree_sizes());
        assert_eq!(clip.trees(), &h.trees()[..]);
        // The header's count leaves the ring frame out; the table has it.
        assert_eq!(word(&bytes, 12), 1);
        let packets: Vec<_> = clip.reader().collect();
        assert_eq!(packets.len(), 2);
        let first = packets[0].as_ref().unwrap();
        assert_eq!(
            (first.frame, first.flags, first.video),
            (0, 2, &[1, 0, 0, 0][..])
        );
        let second = packets[1].as_ref().unwrap();
        assert_eq!((second.frame, second.flags), (1, 2));
        assert_eq!(second.video, [2, 3]);
    }

    #[test]
    fn headers_ffmpeg_refuses_are_refused() {
        let good = file(&header(), &[FrameSpec::video(vec![1])]);
        let with = |at: usize, value: u32| {
            let mut bytes = good.clone();
            bytes[at..at + 4].copy_from_slice(&value.to_le_bytes());
            Clip::parse(&bytes).map(|clip| clip.frames())
        };
        assert_eq!(Clip::parse(b"BIKb").err(), Some(Error::Magic));
        assert_eq!(Clip::parse(b"SM").err(), Some(Error::Magic));
        assert_eq!(Clip::parse(&good[..103]).err(), Some(Error::Truncated));
        assert_eq!(
            Clip::parse(&good[..good.len() - 5]).err(),
            Some(Error::Truncated)
        );
        assert_eq!(
            with(16, (i32::MAX / 100 + 1) as u32),
            Err(Error::Field("frame time"))
        );
        assert_eq!(with(16, (i32::MAX / 100) as u32), Ok(1));
        assert_eq!(with(16, i32::MIN as u32), Err(Error::Field("frame time")));
        assert_eq!(with(16, (i32::MIN + 1) as u32), Ok(1));
        assert_eq!(with(12, MAX_FRAMES + 1), Err(Error::Field("frames")));
        assert_eq!(with(52, u32::MAX / 4), Err(Error::Field("trees")));
        // The table's size: a huge count past the bytes is truncated, not allocated.
        assert_eq!(with(12, MAX_FRAMES), Err(Error::Truncated));
        // FFmpeg's count wraps with the ring frame: no frame at all.
        let mut wrapped = good.clone();
        wrapped[12..16].copy_from_slice(&u32::MAX.to_le_bytes());
        wrapped[20] = 1;
        assert_eq!(Clip::parse(&wrapped).map(|clip| clip.frames()), Ok(0));
    }

    #[test]
    fn tracks_say_whether_their_packets_start_with_a_size() {
        let track = |rate, flags| {
            Track {
                rate,
                flags,
                largest: 0,
            }
            .duration_bytes()
        };
        assert_eq!(track(0, 0xF0), 0);
        assert_eq!(track(22050, 0x60), 0);
        assert_eq!(track(22050, 0x40), 4);
        assert_eq!(track(22050, 0xE0), 4);
        assert_eq!(track(22050, 0x28), 4);
        assert_eq!(track(22050, 0x24), 4);
    }

    /// The packets of one frame after a first frame of a new palette's colours.
    fn after(change: FrameSpec) -> Vec<Result<Packet<'static>, Error>> {
        let colors: Vec<Op> = (0..256u32)
            .map(|k| Op::New([(k % 64) as u8, (k / 4 % 64) as u8, 63 - (k % 64) as u8]))
            .collect();
        let first = FrameSpec {
            palette: Some(build::palette(&colors)),
            ..FrameSpec::video(vec![1])
        };
        let bytes = file(&header(), &[first, change]).leak();
        let clip = Box::leak(Box::new(Clip::parse(bytes).unwrap()));
        clip.reader().collect()
    }

    fn color(packet: &Packet<'_>, k: usize) -> [u8; 3] {
        packet.palette[3 * k..3 * k + 3].try_into().unwrap()
    }

    #[test]
    fn a_palette_change_skips_copies_and_sets_colours() {
        let ops = [
            Op::Skip(2),
            Op::Copy { count: 3, from: 10 },
            Op::New([63, 0x40 | 32, 0xC0 | 1]),
            Op::Skip(128),
            Op::Skip(122),
        ];
        let packets = after(FrameSpec {
            palette: Some(build::palette(&ops)),
            ..FrameSpec::video(vec![1])
        });
        let first = packets[0].as_ref().unwrap();
        assert_eq!(first.flags, 3);
        // 6-bit levels through FFmpeg's table.
        assert_eq!(color(first, 1), [0x04, 0x00, 0xFB]);
        assert_eq!(color(first, 63), [0xFF, 0x3C, 0x00]);
        let second = packets[1].as_ref().unwrap();
        assert_eq!(second.flags, 1);
        assert_eq!(color(second, 1), color(first, 1));
        assert_eq!(
            [color(second, 2), color(second, 3), color(second, 4)],
            [color(first, 10), color(first, 11), color(first, 12)]
        );
        // Green's and blue's top bits ignored.
        assert_eq!(color(second, 5), [0xFF, 0x82, 0x04]);
        assert_eq!(color(second, 6), color(first, 6));
        assert_eq!(second.video, [1, 0, 0, 0]);
    }

    #[test]
    fn a_copy_ends_at_the_palettes_end() {
        // 250 kept, then a copy of 20 of which 6 fit.
        let ops = [
            Op::Skip(128),
            Op::Skip(122),
            Op::Copy { count: 20, from: 0 },
        ];
        let packets = after(FrameSpec {
            palette: Some(build::palette(&ops)),
            ..FrameSpec::video(vec![1])
        });
        let (first, second) = (packets[0].as_ref().unwrap(), packets[1].as_ref().unwrap());
        assert_eq!(color(second, 255), color(first, 5));
        assert_eq!(color(second, 249), color(first, 249));
    }

    #[test]
    fn a_bad_palette_change_fails_its_frame() {
        // A copy past the palette's end: the colours set before it stay, the frame fails and
        // the next is read from where its size ends it; no change is told (FFmpeg marks one
        // only when it ends).
        let bad = FrameSpec {
            palette: Some(build::palette(&[
                Op::New([1, 2, 3]),
                Op::Copy {
                    count: 2,
                    from: 255,
                },
            ])),
            ..FrameSpec::video(vec![1])
        };
        let mut bytes = file(
            &header(),
            &[
                FrameSpec::video(vec![1]),
                bad.clone(),
                FrameSpec::video(vec![5]),
            ],
        );
        let clip = Clip::parse(&bytes).unwrap();
        let packets: Vec<_> = clip.reader().collect();
        assert_eq!(packets[1], Err(Error::Invalid("palette copy past its end")));
        let third = packets[2].as_ref().unwrap();
        assert_eq!((third.flags, color(third, 0)), (0, [0x04, 0x08, 0x0C]));
        assert_eq!(third.video, [5, 0, 0, 0]);
        // A size larger than the frame.
        let at = clip.data + 4;
        bytes[at] = 9;
        let clip = Clip::parse(&bytes).unwrap();
        let packets: Vec<_> = clip.reader().collect();
        assert_eq!(
            packets[1],
            Err(Error::Invalid("palette larger than its frame"))
        );
        assert_eq!(packets[2].as_ref().unwrap().video, [5, 0, 0, 0]);
        // A size of 0: the change is read after its size's byte, the rest from the byte itself
        // (FFmpeg's seek one byte back), so the video is the whole frame.
        let kept = FrameSpec {
            palette: Some(build::palette(&[Op::Skip(128), Op::Skip(128)])),
            ..FrameSpec::video(vec![5])
        };
        let mut zero = file(&header(), &[FrameSpec::video(vec![1]), kept]);
        let at = Clip::parse(&zero).unwrap().data + 4;
        zero[at] = 0;
        let clip = Clip::parse(&zero).unwrap();
        let packets: Vec<_> = clip.reader().collect();
        let second = packets[1].as_ref().unwrap();
        assert_eq!(
            (second.flags, second.video),
            (1, &[0, 0xFF, 0xFF, 0, 5, 0, 0, 0][..])
        );
        // A change running past the file's end reads 0s there (new colours of level 0); the
        // frame's video, past the end too, fails it.
        let tail = file(&header(), &[FrameSpec::video(vec![1])]);
        let mut cut = tail.clone();
        let flags_at = HEADER + 4;
        cut[flags_at] = 1;
        cut.truncate(cut.len() - 3);
        let clip = Clip::parse(&cut).unwrap();
        assert_eq!(clip.reader().next(), Some(Err(Error::Truncated)));
    }

    #[test]
    fn audio_packets_are_given_skipped_or_fail_their_frame() {
        let mut h = header();
        h.tracks[0] = Track {
            rate: 22050,
            flags: 0xF0,
            largest: 0,
        };
        let frame = |audio: Vec<(usize, Vec<u8>)>| FrameSpec {
            audio,
            ..FrameSpec::video(vec![9])
        };
        // Track 0 has a decoded size; track 1 is not in the header (none). A frame failing in
        // its audio after its palette change: the change is told with the next packet.
        let change = FrameSpec {
            palette: Some(build::palette(&[Op::Skip(128), Op::Skip(128)])),
            ..frame(vec![(0, vec![1, 2, 3])])
        };
        let bytes = file(
            &h,
            &[
                frame(vec![(0, vec![0; 4]), (1, vec![])]),
                change,
                frame(Vec::new()),
            ],
        );
        let clip = Clip::parse(&bytes).unwrap();
        let packets: Vec<_> = clip.reader().collect();
        assert_eq!(packets[0].as_ref().unwrap().video, [9, 0, 0, 0]);
        // Track 0's packet given; track 1's skipped.
        assert_eq!(
            packets[0].as_ref().unwrap().audio[0],
            Some([0; 4].as_slice())
        );
        assert_eq!(packets[0].as_ref().unwrap().audio[1], None);
        assert_eq!(packets[1], Err(Error::Invalid("audio packet's size")));
        assert_eq!(packets[2].as_ref().unwrap().flags, 1);
        assert_eq!(packets[2].as_ref().unwrap().audio, [None; 7]);
        // A packet larger than its frame.
        let mut big = bytes.clone();
        let at = clip.data;
        big[at] = 200;
        let clip = Clip::parse(&big).unwrap();
        assert_eq!(
            clip.reader().next(),
            Some(Err(Error::Invalid("audio packet's size")))
        );
        // A size past i32's: negative to FFmpeg.
        big[at..at + 4].copy_from_slice(&0x8000_0004u32.to_le_bytes());
        let clip = Clip::parse(&big).unwrap();
        assert_eq!(
            clip.reader().next(),
            Some(Err(Error::Invalid("audio packet's size")))
        );
        // A packet running past the file's end.
        let cut = file(&h, &[frame(vec![(0, vec![7; 12])])]);
        let mut short = cut.clone();
        short.truncate(cut.len() - 8);
        let clip = Clip::parse(&short).unwrap();
        assert_eq!(clip.reader().next(), Some(Err(Error::Truncated)));
    }

    #[test]
    fn a_frames_video_must_be_in_the_file() {
        let bytes = file(
            &header(),
            &[FrameSpec::video(vec![1]), FrameSpec::video(vec![2])],
        );
        let mut sized = bytes.clone();
        sized[HEADER + 4..HEADER + 8].copy_from_slice(&0x4000_0000u32.to_le_bytes());
        let clip = Clip::parse(&sized).unwrap();
        let packets: Vec<_> = clip.reader().collect();
        assert_eq!(packets[1], Err(Error::Invalid("video's size")));
        sized[HEADER + 4..HEADER + 8].copy_from_slice(&0x3FFF_FFF8u32.to_le_bytes());
        let clip = Clip::parse(&sized).unwrap();
        assert_eq!(clip.reader().nth(1), Some(Err(Error::Truncated)));
        // A key frame's mark in its size; a frame of no bytes gives an empty video.
        let mut marked = bytes.clone();
        marked[HEADER + 4] |= 1;
        marked[HEADER] = 0;
        let clip = Clip::parse(&marked).unwrap();
        let packets: Vec<_> = clip.reader().collect();
        assert_eq!(packets[0].as_ref().unwrap().video, []);
        assert_eq!(packets[1].as_ref().unwrap().flags, 2);
        assert_eq!(packets[1].as_ref().unwrap().video, [1, 0, 0, 0]);
    }

    proptest! {
        #[test]
        fn parse_and_read_never_panic(
            mut bytes in proptest::collection::vec(any::<u8>(), 0..400),
            fix in any::<bool>(),
        ) {
            if fix && bytes.len() >= 56 {
                bytes[..4].copy_from_slice(b"SMK2");
                bytes[12..16].copy_from_slice(&3u32.to_le_bytes());
                bytes[20] &= 1;
                bytes[52..56].copy_from_slice(&8u32.to_le_bytes());
            }
            if let Ok(clip) = Clip::parse(&bytes) {
                for _ in clip.reader() {}
            }
        }
    }
}
