//! The Bink file (FFmpeg 6.1.1's demuxer `libavformat/bink.c`: `probe`'s checks, `read_header`
//! and `read_packet`): a 44-byte header, the audio tracks' descriptions, the frame index, then the
//! frames, each its audio tracks' packets (a size and the bytes) and the video packet.
//!
//! The header's words: the signature (`BIK` and the revision), the file's size less 8, the
//! frames' count, the largest frame's size, the count again, width, height, the frame rate's
//! numerator and denominator, the video's flags, the audio tracks' count. Revision `k` adds a
//! word. Each audio track adds three (its largest decoded size; its rate and flags; its id). The
//! index gives each frame's offset, the lowest bit marking a key frame; FFmpeg reads one word
//! per frame (the files have one more, the end) and ends the last frame at the header's size.

use alloc::vec::Vec;

use crate::Error;

/// The header's bytes.
const HEADER: usize = 44;
/// FFmpeg's demuxer's limits.
const MAX_FRAMES: u32 = 1_000_000;
const MAX_WIDTH: u32 = 7680;
const MAX_HEIGHT: u32 = 4800;
const MAX_AUDIO_TRACKS: u32 = 256;

/// One frame's place in the file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameEntry {
    /// Where its bytes start in the file.
    pub offset: usize,
    /// How many bytes it has (its audio packets and its video packet).
    pub size: usize,
    /// Whether the index marks it a key frame (the first always is).
    pub key: bool,
}

/// A parsed Bink file: its header and frame index over the file's bytes.
#[derive(Debug, Clone)]
pub struct Clip<'a> {
    bytes: &'a [u8],
    /// The signature's fourth byte (`b'b'` for the game's clips).
    pub revision: u8,
    /// The frame's width in pixels.
    pub width: u32,
    /// The frame's height in pixels.
    pub height: u32,
    /// The frame rate's numerator: `rate_num / rate_den` frames a second.
    pub rate_num: u32,
    /// The frame rate's denominator.
    pub rate_den: u32,
    /// The video's flags (FFmpeg's extradata; [`crate::FLAG_ALPHA`] an alpha plane).
    pub flags: u32,
    /// How many audio tracks the frames carry (their packets are skipped).
    pub audio_tracks: u32,
    frames: Vec<FrameEntry>,
}

/// The little-endian word at `at`.
fn word(bytes: &[u8], at: usize) -> Option<u32> {
    let chunk = bytes.get(at..)?.first_chunk::<4>()?;
    Some(u32::from_le_bytes(*chunk))
}

/// A 32-bit offset or size as an index.
fn index(value: u32) -> usize {
    usize::try_from(value).unwrap_or(usize::MAX)
}

impl<'a> Clip<'a> {
    /// Parses the header and the frame index, checking what FFmpeg's probe and header reader
    /// check and that every frame lies inside `bytes`.
    pub fn parse(bytes: &'a [u8]) -> Result<Self, Error> {
        let mut head = [0u32; HEADER / 4];
        for (i, w) in head.iter_mut().enumerate() {
            *w = word(bytes, 4 * i).ok_or(Error::Truncated)?;
        }
        let [
            signature,
            size,
            count,
            largest,
            _,
            width,
            height,
            rate_num,
            rate_den,
            flags,
            audio,
        ] = head;
        let [b0, b1, b2, revision] = signature.to_le_bytes();
        if [b0, b1, b2] != *b"BIK" {
            return Err(Error::Magic);
        }
        // `file_size`: a `uint32_t` in FFmpeg, so the sum wraps.
        let file_size = size.wrapping_add(8);
        if count == 0 || count > MAX_FRAMES {
            return Err(Error::Field("frames"));
        }
        if largest > file_size {
            return Err(Error::Field("largest frame"));
        }
        if width == 0 || width > MAX_WIDTH {
            return Err(Error::Field("width"));
        }
        if height == 0 || height > MAX_HEIGHT {
            return Err(Error::Field("height"));
        }
        if rate_num == 0 || rate_den == 0 {
            return Err(Error::Field("frame rate"));
        }
        if audio > MAX_AUDIO_TRACKS {
            return Err(Error::Field("audio tracks"));
        }
        let extra = if revision == b'k' { 4 } else { 0 };
        let mut at = HEADER + extra + 12 * index(audio);
        let mut frames = Vec::with_capacity(index(count).min(bytes.len() / 4));
        let mut next = word(bytes, at).ok_or(Error::Truncated)?;
        let mut next_key = true;
        for i in 0..count {
            let (offset, key) = (next, next_key);
            if i + 1 == count {
                (next, next_key) = (file_size, false);
            } else {
                at += 4;
                next = word(bytes, at).ok_or(Error::Truncated)?;
                next_key = next & 1 == 1;
            }
            let (offset, end) = (offset & !1, next & !1);
            if end <= offset {
                return Err(Error::Field("frame index"));
            }
            if index(end) > bytes.len() {
                return Err(Error::Truncated);
            }
            frames.push(FrameEntry {
                offset: index(offset),
                size: index(end - offset),
                key,
            });
        }
        Ok(Self {
            bytes,
            revision,
            width,
            height,
            rate_num,
            rate_den,
            flags,
            audio_tracks: audio,
            frames,
        })
    }

    /// The frames in order.
    #[must_use]
    pub fn frames(&self) -> &[FrameEntry] {
        &self.frames
    }

    /// Frame `n`'s video packet: its bytes past its audio tracks' packets (`read_packet`).
    pub fn video(&self, n: usize) -> Result<&'a [u8], Error> {
        let entry = self.frames.get(n).ok_or(Error::Field("frame"))?;
        // Inside the file: `parse` checked every frame's range.
        let mut data = self
            .bytes
            .get(entry.offset..)
            .and_then(|rest| rest.get(..entry.size))
            .unwrap_or_default();
        for _ in 0..self.audio_tracks {
            let size = index(word(data, 0).ok_or(Error::Truncated)?);
            data = data
                .get(4..)
                .and_then(|rest| rest.get(size..))
                .ok_or(Error::Field("audio size"))?;
        }
        Ok(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build::{self, Header};
    use alloc::vec;
    use proptest::prelude::*;

    fn header() -> Header {
        Header::new(16, 8)
    }

    #[test]
    fn the_header_and_index_are_read() {
        // "one" padded to an even size, as the files' frames are.
        let bytes = build::file(&header(), &[b"one".to_vec(), b"second".to_vec()]);
        let clip = Clip::parse(&bytes).unwrap();
        assert_eq!((clip.revision, clip.width, clip.height), (b'b', 16, 8));
        assert_eq!((clip.rate_num, clip.rate_den, clip.flags), (15, 1, 0));
        assert_eq!(clip.audio_tracks, 0);
        let frames = clip.frames();
        assert_eq!(frames.len(), 2);
        assert!(frames[0].key && !frames[1].key);
        assert_eq!(frames[0].offset, 44 + 12);
        assert_eq!(clip.video(0), Ok(&b"one\0"[..]));
        assert_eq!(clip.video(1), Ok(&b"second"[..]));
        assert_eq!(clip.video(2), Err(Error::Field("frame")));
    }

    #[test]
    fn a_marked_frame_is_a_key_frame_and_odd_ends_are_even() {
        let mut bytes = build::file(&header(), &[vec![1; 3], vec![2; 4], vec![3; 2]]);
        // Mark frame 1 a key frame and frame 0's offset's low bit (ignored: frame 0 always is).
        bytes[44] |= 1;
        bytes[48] |= 1;
        let clip = Clip::parse(&bytes).unwrap();
        let keys: Vec<bool> = clip.frames().iter().map(|f| f.key).collect();
        assert_eq!(keys, [true, true, false]);
        // An odd size in the header still ends the last frame on the even byte before it.
        let size = u32::from_le_bytes(bytes[4..8].try_into().unwrap());
        bytes[4..8].copy_from_slice(&(size + 1).to_le_bytes());
        bytes.push(0);
        let clip = Clip::parse(&bytes).unwrap();
        assert_eq!(clip.frames()[2].size, 2);
    }

    #[test]
    fn audio_packets_are_skipped() {
        let mut header = header();
        header.audio_tracks = 2;
        let mut frame = Vec::new();
        for audio in [&b"ab"[..], b""] {
            frame.extend_from_slice(&(audio.len() as u32).to_le_bytes());
            frame.extend_from_slice(audio);
        }
        frame.extend_from_slice(b"videos");
        let bytes = build::file(&header, &[frame.clone()]);
        let clip = Clip::parse(&bytes).unwrap();
        assert_eq!(clip.frames()[0].offset, 44 + 24 + 8);
        assert_eq!(clip.video(0), Ok(&b"videos"[..]));
        // An audio size past the frame, or a frame too short for a size.
        frame[0] = 99;
        let clip_bytes = build::file(&header, &[frame]);
        assert_eq!(
            Clip::parse(&clip_bytes).unwrap().video(0),
            Err(Error::Field("audio size"))
        );
        let short = build::file(&header, &[vec![0, 0, 0, 0, 1, 2]]);
        assert_eq!(Clip::parse(&short).unwrap().video(0), Err(Error::Truncated));
    }

    #[test]
    fn revision_k_has_one_more_word() {
        let mut header = header();
        header.revision = b'k';
        let bytes = build::file(&header, &[b"xy".to_vec()]);
        let clip = Clip::parse(&bytes).unwrap();
        assert_eq!(clip.revision, b'k');
        assert_eq!(clip.video(0), Ok(&b"xy"[..]));
    }

    #[test]
    fn what_ffmpeg_rejects_is_rejected() {
        let good = build::file(&header(), &[b"frame".to_vec()]);
        let with = |at: usize, value: u32| {
            let mut bytes = good.clone();
            bytes[at..at + 4].copy_from_slice(&value.to_le_bytes());
            Clip::parse(&bytes).err()
        };
        assert_eq!(with(0, u32::from_le_bytes(*b"SMK2")), Some(Error::Magic));
        assert_eq!(with(8, 0), Some(Error::Field("frames")));
        assert_eq!(with(8, MAX_FRAMES + 1), Some(Error::Field("frames")));
        assert_eq!(with(12, u32::MAX), Some(Error::Field("largest frame")));
        assert_eq!(with(20, 0), Some(Error::Field("width")));
        assert_eq!(with(20, MAX_WIDTH + 1), Some(Error::Field("width")));
        assert_eq!(with(24, 0), Some(Error::Field("height")));
        assert_eq!(with(24, MAX_HEIGHT + 1), Some(Error::Field("height")));
        assert_eq!(with(28, 0), Some(Error::Field("frame rate")));
        assert_eq!(with(32, 0), Some(Error::Field("frame rate")));
        assert_eq!(
            with(40, MAX_AUDIO_TRACKS + 1),
            Some(Error::Field("audio tracks"))
        );
        // The first frame's offset past the next one's, or the end past the bytes.
        assert_eq!(with(44, 1000), Some(Error::Field("frame index")));
        assert_eq!(with(4, 1000), Some(Error::Truncated));
        // The size's wrap: 0xFFFF_FFF8 + 8 is 0, before the frame.
        assert_eq!(with(4, u32::MAX - 7), Some(Error::Field("largest frame")));
        // The index past the file's end (the audio tracks' descriptions take it all).
        assert_eq!(with(40, MAX_AUDIO_TRACKS), Some(Error::Truncated));
        // Cut anywhere in the header or the index.
        for cut in [0, 10, 43, 44, 47] {
            assert_eq!(
                Clip::parse(&good[..cut]).err(),
                Some(Error::Truncated),
                "{cut}"
            );
        }
        let two = build::file(&header(), &[b"a".to_vec(), b"b".to_vec()]);
        assert_eq!(Clip::parse(&two[..49]).err(), Some(Error::Truncated));
    }

    proptest! {
        #[test]
        fn parse_never_panics(mut bytes in proptest::collection::vec(any::<u8>(), 0..200), fix in any::<bool>()) {
            if fix && bytes.len() >= 4 {
                bytes[..4].copy_from_slice(b"BIKb");
            }
            if let Ok(clip) = Clip::parse(&bytes) {
                for n in 0..clip.frames().len() + 1 {
                    let _ = clip.video(n);
                }
            }
        }
    }
}
