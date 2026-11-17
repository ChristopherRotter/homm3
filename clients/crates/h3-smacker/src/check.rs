//! What the checks compare (E27 M2 T1): a clip's every frame decoded, the frames as raw bytes
//! and their hash, and the original's 16-bit copy of them, for the decoder's tests and
//! `h3-view video --smacker` (against `SMACKW32.DLL`'s decoding of the same file:
//! `clients/scripts/smacker-check.sh`).

use alloc::vec;
use alloc::vec::Vec;

use crate::{Clip, Decoder, Error, Frame, Rect, Rects};

/// Every frame of the Smacker file `bytes` decoded in order, those that fail left out (FFmpeg
/// outputs none for them; the next decodes over what they left).
pub fn frames(bytes: &[u8]) -> Result<Vec<Frame>, Error> {
    let clip = Clip::parse(bytes)?;
    let mut decoder = Decoder::new(&clip)?;
    let mut frames = Vec::new();
    for packet in clip.reader().flatten() {
        if let Ok(frame) = decoder.decode(&packet) {
            frames.push(frame.clone());
        }
    }
    Ok(frames)
}

/// The frames as raw bytes (the reference's format, `smacker-dll.py`'s): each frame's colour
/// indices row by row, then its palette (256 times red, green, blue).
#[must_use]
pub fn raw(frames: &[Frame]) -> Vec<u8> {
    let mut out = Vec::new();
    for frame in frames {
        out.extend_from_slice(frame.pixels());
        out.extend_from_slice(frame.palette());
    }
    out
}

/// The FNV-1a hash of [`raw`]'s bytes.
#[must_use]
pub fn hash(frames: &[Frame]) -> u64 {
    raw(frames)
        .iter()
        .fold(0xCBF2_9CE4_8422_2325, |hash, &byte| {
            (hash ^ u64::from(byte)).wrapping_mul(0x0100_0000_01B3)
        })
}

/// The 16-bit formats the game asks `SMACKW32.DLL` for (`SmackToBuffer`'s flags 0x80000000 and
/// 0xC0000000, by the screen's format: `smacker_open_in_dialog` 0x598AF0).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    /// 5-5-5.
    Rgb555,
    /// 5-6-5.
    Rgb565,
}

/// The original's 16-bit copy of the frames, as `SMACKW32.DLL` writes it into the game's
/// buffer ([`Frame::copy_16`] of the written blocks; CLT-Q119: for the comparisons only, the
/// shell's client profile converts every pixel), into a buffer of the first frame's size (0
/// before the first), so a palette change leaves the skipped blocks' colours as they were.
/// Each frame's pixels as little-endian words row by row, then its palette.
#[must_use]
pub fn original_16(frames: &[Frame], format: Format) -> Vec<u8> {
    let Some(first) = frames.first() else {
        return Vec::new();
    };
    let width = first.width() as usize;
    let mut screen = vec![0u16; first.pixels().len()];
    let mut out = Vec::new();
    for frame in frames {
        frame.copy_16(format, false, &mut screen, width, [0, 0]);
        out.extend(screen.iter().flat_map(|p| p.to_le_bytes()));
        out.extend_from_slice(frame.palette());
    }
    out
}

/// Every frame's rectangles as `SMACKW32.DLL`'s `SmackToBufferRect` gives them (E27 M2 T5a2;
/// `smackprobe rects`): the clip put at `at` of its buffer, each frame that decodes in order
/// ([`Rects::decoded`]), then the calls until one takes none: for each whether it took one and
/// the last rectangle after it ([`Rects::last`]).
pub fn rects(bytes: &[u8], at: [u32; 2]) -> Result<Vec<Vec<(bool, Rect)>>, Error> {
    let clip = Clip::parse(bytes)?;
    let mut decoder = Decoder::new(&clip)?;
    let mut rects = Rects::new([clip.width, clip.height], at);
    let mut out = Vec::new();
    for packet in clip.reader().flatten() {
        if let Ok(frame) = decoder.decode(&packet) {
            rects.decoded();
            let mut calls = Vec::new();
            loop {
                let took = rects.next(frame);
                calls.push((took, rects.last()));
                if !took {
                    break;
                }
            }
            out.push(calls);
        }
    }
    Ok(out)
}

/// Every track's sound decoded in order (E27 M2 T2), as the DLL hands it to Miles
/// (`smackprobe audio`): for each track the header describes (its index and the samples'
/// bytes, [`crate::Samples::bytes`]), its packets' samples one after the other; a frame or a
/// packet that fails, or holds no sound, adds none (FFmpeg outputs none for it).
pub fn sound(bytes: &[u8]) -> Result<Vec<(usize, Vec<u8>)>, Error> {
    let clip = Clip::parse(bytes)?;
    let mut tracks: Vec<(usize, Vec<u8>)> = clip
        .tracks
        .iter()
        .enumerate()
        .filter(|(_, track)| track.rate != 0)
        .map(|(k, _)| (k, Vec::new()))
        .collect();
    for packet in clip.reader().flatten() {
        for (k, out) in &mut tracks {
            if let (Some(data), Some(&track)) = (packet.audio[*k], clip.tracks.get(*k))
                && let Ok(Some(samples)) = crate::sound::decode(track, data)
            {
                out.extend(samples.bytes());
            }
        }
    }
    Ok(tracks)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build::{self, FrameSpec, Header, Op, TreeSpec, Video};
    use crate::tree::TYPES;

    /// Two frames of 8 × 4: a fill of colour 1 over both blocks, then a palette change and a
    /// fill of colour 2 over the first block, the second skipped.
    fn frames() -> Vec<Frame> {
        let (fill_both, fill_one, skip) = (3 | 1 << 2 | 1 << 8, 3 | 2 << 8, 2);
        let types = TreeSpec::of(&[fill_both, fill_one, skip]);
        let h = Header::new(8, 4, [None, None, None, Some(types)]);
        let video = |values: &[u32]| {
            let mut video = Video::new(&h);
            for &value in values {
                video.code(TYPES, value);
            }
            video.bytes()
        };
        let palette = |level: u8| {
            let colors: Vec<Op> = (0..256).map(|_| Op::New([level, level, level])).collect();
            Some(build::palette(&colors))
        };
        let file = build::file(
            &h,
            &[
                FrameSpec {
                    palette: palette(63),
                    ..FrameSpec::video(video(&[fill_both]))
                },
                FrameSpec {
                    palette: palette(31),
                    ..FrameSpec::video(video(&[fill_one, skip]))
                },
            ],
        );
        super::frames(&file).unwrap()
    }

    #[test]
    fn the_originals_16_bit_copy_keeps_the_skipped_blocks_colours() {
        let frames = frames();
        let copy = original_16(&frames, Format::Rgb565);
        let size = 8 * 4 * 2 + 768;
        assert_eq!(copy.len(), 2 * size);
        let pixel = |frame: usize, x: usize| {
            u16::from_le_bytes([copy[frame * size + 2 * x], copy[frame * size + 2 * x + 1]])
        };
        // Level 63 (0xFF) and level 31 (0x7D) in 5-6-5.
        assert_eq!((pixel(0, 0), pixel(0, 7)), (0xFFFF, 0xFFFF));
        assert_eq!((pixel(1, 0), pixel(1, 7)), (0x7BEF, 0xFFFF));
        assert_eq!(&copy[size - 768..size], frames[0].palette());
        let five = original_16(&frames, Format::Rgb555);
        assert_eq!(u16::from_le_bytes([five[size], five[size + 1]]), 0x3DEF);
        assert_eq!(original_16(&[], Format::Rgb565), Vec::<u8>::new());
        // Frames larger than the first write only where the first's pixels are.
        let large = super::frames(&build::sample(5)).unwrap();
        let mixed = original_16(&[frames[0].clone(), large[0].clone()], Format::Rgb565);
        assert_eq!(mixed.len(), 2 * size);
    }

    #[test]
    fn a_frame_copies_at_its_place_cut_at_the_buffers_edges() {
        let frames = frames();
        let check = |buffer: &[u16], inside: &dyn Fn(usize) -> u16| {
            for (k, &p) in buffer.iter().enumerate() {
                let (x, y) = (k % 10, k / 10);
                let want = if x >= 4 && y >= 1 { inside(x) } else { 0 };
                assert_eq!(p, want, "{x}, {y}");
            }
        };
        // Every pixel at (4, 1) of a buffer 10 wide and 4 high: the columns past 10 and the row
        // past the 4th cut.
        let mut buffer = vec![0u16; 40];
        frames[0].copy_16(Format::Rgb565, true, &mut buffer, 10, [4, 1]);
        check(&buffer, &|_| 0xFFFF);
        // The written blocks alone (the DLL's): the second frame's first block, the skipped one
        // keeping the first frame's colour.
        frames[1].copy_16(Format::Rgb565, false, &mut buffer, 10, [4, 1]);
        check(&buffer, &|x| if x < 8 { 0x7BEF } else { 0xFFFF });
        // Every pixel (the client's): the skipped block's indices through the new palette too.
        frames[1].copy_16(Format::Rgb565, true, &mut buffer, 10, [4, 1]);
        check(&buffer, &|_| 0x7BEF);
    }

    #[test]
    fn frames_that_fail_are_left_out() {
        let mut bytes = build::sample(3);
        let all = super::frames(&bytes).unwrap().len();
        bytes.truncate(bytes.len() - 1);
        assert_eq!(super::frames(&bytes).unwrap().len(), all - 1);
        assert_eq!(super::frames(b"SMK2").err(), Some(Error::Truncated));
        let h = Header::new(0, 4, [None, None, None, Some(TreeSpec::of(&[0]))]);
        assert_eq!(
            super::frames(&build::file(&h, &[])).err(),
            Some(Error::Size)
        );
        assert_eq!(hash(&[]), 0xCBF2_9CE4_8422_2325);
    }

    #[test]
    fn each_described_tracks_sound_is_its_packets_one_after_another() {
        use crate::Track;
        use crate::build::Shape;
        let mut h = Header::new(4, 4, [None, None, None, Some(TreeSpec::of(&[2]))]);
        // Track 1 packed mono 8-bit, track 3 described but given no packets.
        h.tracks[1] = Track {
            rate: 11025,
            flags: 0xC0,
            largest: 0,
        };
        h.tracks[3] = Track {
            rate: 11025,
            flags: 0xC0,
            largest: 0,
        };
        let packet =
            |first: u32| build::sound(h.tracks[1], &[Shape::Leaf(1)], [first, 0], &[1, 1, 1]);
        let frame = |audio: Vec<(usize, Vec<u8>)>| FrameSpec {
            audio,
            ..FrameSpec::video(vec![0])
        };
        let no_sound = vec![4, 0, 0, 0, 0];
        let bytes = build::file(
            &h,
            &[
                frame(vec![(1, packet(10))]),
                // A packet holding no sound, one that fails, and a track not described.
                frame(vec![(1, no_sound), (5, vec![1])]),
                frame(vec![(1, vec![1, 2])]),
                frame(vec![(1, packet(20))]),
            ],
        );
        assert_eq!(
            sound(&bytes),
            Ok(vec![
                (1, vec![10, 11, 12, 13, 20, 21, 22, 23]),
                (3, Vec::new())
            ])
        );
        assert_eq!(sound(b"SMK"), Err(Error::Magic));
    }
}
