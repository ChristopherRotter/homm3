//! The audio tracks' sound (E27 M2 T2; CLT-Q104, decision 21): FFmpeg 6.1.1's Smacker audio
//! decoder (`libavcodec/smacker.c`'s `smka_decode_frame`) for the packed tracks, the game's, and
//! its PCM decoders for the plain ones (`libavcodec/pcmenc.c`'s `pcm_u8` and `pcm_s16le`).
//!
//! A packed packet holds its decoded size, then bits: whether it holds sound, whether it is
//! stereo and 16-bit (both as its track says), a byte tree per channel and byte (two or four
//! for 16 bits, one or two for 8), each channel's first sample, then each later sample's
//! difference from the one before in that channel, its low byte and then its high byte coded by
//! that channel's trees. The sums wrap, as FFmpeg's and the DLL's do (no clipping). Every
//! packet stands alone: its trees and first samples are its own.
//!
//! Checked against the game's `SMACKW32.DLL` (`clients/scripts/smacker-check.sh`:
//! `smackprobe audio`, the samples it hands Miles): byte for byte on the shell's clips and the
//! synthetic samples (design §8).

use alloc::vec::Vec;

use crate::Error;
use crate::bits::Bits;
use crate::clip::Track;
use crate::tree::ByteTree;

/// The largest decoded size a packet may give (FFmpeg's `1U << 24`).
const LARGEST: u32 = 1 << 24;

/// A packet's samples, the channels interleaved (left first).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Samples {
    /// 8-bit samples, unsigned (128 the middle).
    Eight(Vec<u8>),
    /// 16-bit samples.
    Sixteen(Vec<i16>),
}

impl Samples {
    /// The samples' count (all channels').
    #[must_use]
    pub fn len(&self) -> usize {
        match self {
            Self::Eight(s) => s.len(),
            Self::Sixteen(s) => s.len(),
        }
    }

    /// Whether there are none.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// The samples as the DLL hands them to Miles: bytes, or 16-bit words little-endian.
    #[must_use]
    pub fn bytes(&self) -> Vec<u8> {
        match self {
            Self::Eight(s) => s.clone(),
            Self::Sixteen(s) => s.iter().flat_map(|v| v.to_le_bytes()).collect(),
        }
    }
}

/// How a track's packets are coded (`smacker_read_header`'s choice of codec).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Codec {
    /// Smacker's own (the flag 0x80): the game's clips.
    Packed,
    /// Plain samples.
    Plain,
    /// Bink's audio (the flags 0x08 or 0x04, `SMK4`'s): not decoded here (`SMACKW32.DLL`, of
    /// 1997, knows `SMK2` alone).
    Bink,
}

impl Track {
    /// Its channels: 2 with the stereo flag (0x10), else 1.
    #[must_use]
    pub fn channels(self) -> u32 {
        if self.flags & 0x10 != 0 { 2 } else { 1 }
    }

    /// Its samples' bits: 16 with the flag 0x20, else 8.
    #[must_use]
    pub fn bits(self) -> u32 {
        if self.flags & 0x20 != 0 { 16 } else { 8 }
    }

    /// How its packets are coded.
    #[must_use]
    pub fn codec(self) -> Codec {
        if self.flags & 0x0C != 0 {
            Codec::Bink
        } else if self.flags & 0x80 != 0 {
            Codec::Packed
        } else {
            Codec::Plain
        }
    }
}

/// Decodes one of `track`'s packets (its bytes after the size word, as [`crate::Packet::audio`]
/// gives them); `None` for a packet that says it holds no sound (FFmpeg's "Sound: no data", no
/// frame output).
pub fn decode(track: Track, packet: &[u8]) -> Result<Option<Samples>, Error> {
    match track.codec() {
        Codec::Packed => packed(track, packet),
        Codec::Plain => Ok(Some(plain(track, packet)?)),
        Codec::Bink => Err(Error::Unsupported("Bink's audio in a Smacker file")),
    }
}

/// A plain packet (`pcm_decode_frame`): its whole samples, a part one at the end dropped.
fn plain(track: Track, packet: &[u8]) -> Result<Samples, Error> {
    let block = (track.channels() * track.bits() / 8) as usize;
    if packet.len() < block {
        return Err(Error::Invalid("plain sound packet too small"));
    }
    let whole = &packet[..packet.len() - packet.len() % block];
    Ok(if track.bits() == 16 {
        Samples::Sixteen(
            whole
                .chunks_exact(2)
                .map(|b| i16::from_le_bytes([b[0], b[1]]))
                .collect(),
        )
    } else {
        Samples::Eight(whole.to_vec())
    })
}

/// A packed packet (`smka_decode_frame`).
fn packed(track: Track, packet: &[u8]) -> Result<Option<Samples>, Error> {
    let (Some(size), Some(rest)) = (packet.get(..4), packet.get(4..)) else {
        return Err(Error::Invalid("sound packet too small"));
    };
    if rest.is_empty() {
        return Err(Error::Invalid("sound packet too small"));
    }
    let unpacked = u32::from_le_bytes([size[0], size[1], size[2], size[3]]);
    if unpacked > LARGEST {
        return Err(Error::Invalid("sound packet too big"));
    }
    let mut bits = Bits::new(rest);
    if bits.bit() == 0 {
        return Ok(None);
    }
    let stereo = bits.bit() as usize;
    let sixteen = bits.bit();
    if (stereo == 1) != (track.channels() != 1) {
        return Err(Error::Invalid("sound channels not the track's"));
    }
    if (sixteen == 1) != (track.bits() == 16) {
        return Err(Error::Invalid("sound bits not the track's"));
    }
    let block = track.channels() * (sixteen + 1);
    if !unpacked.is_multiple_of(block) {
        return Err(Error::Invalid("sound not whole samples"));
    }
    if unpacked / block == 0 {
        // `ff_get_buffer` refuses a frame of no samples.
        return Err(Error::Invalid("sound of no samples"));
    }
    let mut trees = [const { ByteTree::Value(0) }; 4];
    for tree in trees.iter_mut().take(1 << (sixteen as usize + stereo)) {
        bits.bit();
        *tree = ByteTree::parse(&mut bits)?;
        bits.bit();
    }
    // Each channel's first sample, the last channel's read first.
    let mut pred = [0u32; 2];
    let width = if sixteen == 1 { 16 } else { 8 };
    for k in (0..=stereo).rev() {
        let first = bits.bits(width);
        pred[k] = if sixteen == 1 {
            u32::from((first as u16).swap_bytes())
        } else {
            first
        };
    }
    let total = (unpacked >> sixteen) as usize;
    let mut out: Vec<u32> = Vec::with_capacity(total.max(2));
    out.extend_from_slice(&pred[..=stereo]);
    let mut i = stereo + 1;
    if sixteen == 1 {
        let coded = [0, 1, 2 * stereo, 2 * stereo + 1]
            .iter()
            .any(|&k| trees[k].coded());
        if coded {
            while i < total {
                let at = 2 * (i & stereo);
                if bits.left() < 0 {
                    return Err(Error::Invalid("sound past its packet"));
                }
                let low = trees[at].read(&mut bits);
                let high = trees[at + 1].read(&mut bits);
                let channel = at / 2;
                pred[channel] = pred[channel].wrapping_add(low | high << 8);
                out.push(pred[channel]);
                i += 1;
            }
        } else {
            // Every tree one value: a constant difference a channel, no bits read (FFmpeg's
            // shortcut, which writes a stereo pair at a time and checks no bits left).
            let mut step = |k: usize| trees[k + 1].read(&mut bits) << 8 | trees[k].read(&mut bits);
            let (left, right) = (step(0), step(2));
            while i < total {
                pred[0] = pred[0].wrapping_add(left);
                out.push(pred[0]);
                if stereo == 1 {
                    pred[1] = pred[1].wrapping_add(right);
                    out.push(pred[1]);
                    i += 2;
                } else {
                    i += 1;
                }
            }
        }
        out.truncate(total);
        Ok(Some(Samples::Sixteen(
            out.into_iter().map(|v| v as u16 as i16).collect(),
        )))
    } else {
        while i < total {
            let channel = i & stereo;
            if bits.left() < 0 {
                return Err(Error::Invalid("sound past its packet"));
            }
            pred[channel] = pred[channel].wrapping_add(trees[channel].read(&mut bits));
            out.push(pred[channel]);
            i += 1;
        }
        out.truncate(total);
        Ok(Some(Samples::Eight(
            out.into_iter().map(|v| v as u8).collect(),
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build::{self, Shape};
    use alloc::vec;

    fn track(flags: u8) -> Track {
        Track {
            rate: 22050,
            flags,
            largest: 0,
        }
    }

    /// The track the game's clips have: packed, 16-bit stereo.
    const GAME: u8 = 0xF0;

    #[test]
    fn a_tracks_flags_say_its_channels_bits_and_codec() {
        assert_eq!((track(GAME).channels(), track(GAME).bits()), (2, 16));
        assert_eq!((track(0x80).channels(), track(0x80).bits()), (1, 8));
        assert_eq!(track(GAME).codec(), Codec::Packed);
        assert_eq!(track(0x40).codec(), Codec::Plain);
        assert_eq!(track(0x48).codec(), Codec::Bink);
        assert_eq!(track(0xC4).codec(), Codec::Bink);
        assert_eq!(
            decode(track(0x48), &[0; 8]),
            Err(Error::Unsupported("Bink's audio in a Smacker file"))
        );
    }

    #[test]
    fn plain_packets_are_their_whole_samples() {
        let s = decode(track(0x70), &[1, 0, 2, 0, 0xFF, 0xFF, 9])
            .unwrap()
            .unwrap();
        // 16-bit stereo: one whole pair, the rest dropped.
        assert_eq!(s, Samples::Sixteen(vec![1, 2]));
        assert_eq!(
            (s.len(), s.is_empty(), s.bytes()),
            (2, false, vec![1, 0, 2, 0])
        );
        let s = decode(track(0x40), &[7, 8, 9]).unwrap().unwrap();
        assert_eq!((s.bytes(), s.len()), (vec![7, 8, 9], 3));
        assert_eq!(
            decode(track(0x70), &[1, 2, 3]),
            Err(Error::Invalid("plain sound packet too small"))
        );
        assert!(Samples::Eight(Vec::new()).is_empty());
    }

    /// The packet of `samples` (the channels interleaved) for `track`, coded by `trees`.
    fn packet(track: Track, trees: &[Shape], samples: &[u32]) -> Vec<u8> {
        let channels = track.channels() as usize;
        let mask = if track.bits() == 16 { 0xFFFF } else { 0xFF };
        let steps: Vec<u32> = (channels..samples.len())
            .map(|i| samples[i].wrapping_sub(samples[i - channels]) & mask)
            .collect();
        let first = [samples[0], *samples.get(1).unwrap_or(&0)];
        build::sound(track, trees, first, &steps)
    }

    /// A tree of every byte value.
    fn every() -> Shape {
        Shape::balanced(&(0..256).collect::<Vec<_>>())
    }

    #[test]
    fn sixteen_bit_samples_are_their_channels_sums() {
        let samples = [
            0x1234, 0xFFFF, 0x1300, 0x0001, 0x0000, 0x8000, 0xFFFF, 0x7FFF,
        ];
        let p = packet(track(GAME), &[every(), every(), every(), every()], &samples);
        let expect: Vec<i16> = samples.iter().map(|&v| v as u16 as i16).collect();
        assert_eq!(decode(track(GAME), &p), Ok(Some(Samples::Sixteen(expect))));
        // Mono, one tree a single value (no bits for its bytes).
        let mono = track(0xE0);
        let samples = [5, 5 + 0x0102, 5 + 2 * 0x0102];
        let trees = [every(), Shape::Leaf(1)];
        let p = packet(mono, &trees, &samples);
        assert_eq!(
            decode(mono, &p),
            Ok(Some(Samples::Sixteen(vec![5, 0x0107, 0x0209])))
        );
    }

    #[test]
    fn single_valued_trees_add_a_constant_without_bits() {
        // Stereo: 0x0201 on the left, 0x0403 on the right each sample; wrapping.
        let trees = [1, 2, 3, 4].map(Shape::Leaf);
        let p = build::sound(track(GAME), &trees, [0xFFFF, 0], &[0x0201, 0x0403, 0, 0]);
        assert_eq!(
            decode(track(GAME), &p),
            Ok(Some(Samples::Sixteen(vec![
                -1, 0, 0x0200, 0x0403, 0x0401, 0x0806
            ])))
        );
        let mono = track(0xE0);
        let p = build::sound(mono, &[Shape::Leaf(2), Shape::Leaf(0)], [10, 0], &[2, 2]);
        assert_eq!(
            decode(mono, &p),
            Ok(Some(Samples::Sixteen(vec![10, 12, 14])))
        );
    }

    #[test]
    fn eight_bit_samples_are_their_channels_sums() {
        let stereo = track(0xD0);
        let samples = [0x80, 0x10, 0x7F, 0xFF, 0x00, 0x11];
        let p = packet(stereo, &[every(), every()], &samples);
        assert_eq!(
            decode(stereo, &p),
            Ok(Some(Samples::Eight(samples.map(|v| v as u8).to_vec())))
        );
        let mono = track(0xC0);
        let p = packet(mono, &[Shape::Leaf(3)], &[250, 253, 0]);
        assert_eq!(
            decode(mono, &p),
            Ok(Some(Samples::Eight(vec![250, 253, 0])))
        );
    }

    #[test]
    fn broken_packets_fail_as_ffmpegs_do() {
        let small = Err(Error::Invalid("sound packet too small"));
        assert_eq!(decode(track(GAME), &[4, 0, 0, 0]), small);
        assert_eq!(decode(track(GAME), &[4, 0, 0]), small);
        assert_eq!(
            decode(track(GAME), &[1, 0, 0, 1, 0xFF]),
            Err(Error::Invalid("sound packet too big"))
        );
        // The first bit clear: no sound, no samples.
        assert_eq!(decode(track(GAME), &[4, 0, 0, 0, 0xFE]), Ok(None));
        // Mono and 8 bits where the track says stereo and 16.
        assert_eq!(
            decode(track(GAME), &[4, 0, 0, 0, 0b101]),
            Err(Error::Invalid("sound channels not the track's"))
        );
        assert_eq!(
            decode(track(GAME), &[4, 0, 0, 0, 0b011]),
            Err(Error::Invalid("sound bits not the track's"))
        );
        assert_eq!(
            decode(track(GAME), &[6, 0, 0, 0, 0b111]),
            Err(Error::Invalid("sound not whole samples"))
        );
        assert_eq!(
            decode(track(GAME), &[0, 0, 0, 0, 0b111]),
            Err(Error::Invalid("sound of no samples"))
        );
        // A tree whose leaf runs past the packet.
        assert_eq!(
            decode(track(0xC0), &[1, 0, 0, 0, 0b0000_0001]),
            Err(Error::Invalid("byte tree past the end"))
        );
        // Codes running past the packet, in either width.
        for (flags, trees) in [(GAME, vec![every(); 4]), (0xC0, vec![every()])] {
            let samples: Vec<u32> = (0..64).map(|v| v * 37).collect();
            let mut p = packet(track(flags), &trees, &samples);
            p.truncate(p.len() - 8);
            assert_eq!(
                decode(track(flags), &p),
                Err(Error::Invalid("sound past its packet"))
            );
        }
    }
}
