//! WAV files (E17 M2 T5b): the game's sounds. Every entry of the user's two sound archives is a
//! RIFF WAV of Microsoft IMA ADPCM (format 17, 4 bits per sample, 22 050 Hz, mono or (nine of
//! them) stereo, blocks of 512 or 1 024 bytes holding 1 017 sample frames, a `fact` chunk); the
//! decoder reads that format and gives 16-bit PCM.
//!
//! It reads the chunks and counts the frames as SDL 3's loader (`SDL_LoadWAV`, its `SDL_wave.c`,
//! Zlib) does with the hints `SDL_WAVE_TRUNCATION=dropframe` and `SDL_WAVE_FACT_CHUNK=truncate`:
//! every complete sample frame, those of the last, cut block too, at most the `fact` chunk's
//! count, which is the sound's own length (`docs/epics/E17-design.md` §8.2). SDL's defaults stop
//! after the last whole block instead. In a cut block every channel's nibbles are read from its
//! own 4-byte word of the sub-block, as the format lays them out. The real-asset gate compares
//! every entry's samples with SDL's.

use alloc::vec::Vec;

use crate::Error;
use crate::read::{array, index, u16_at, u32_at};

/// The format tag of Microsoft IMA ADPCM.
pub const IMA_ADPCM: u16 = 17;

/// The IMA ADPCM step sizes by step index.
const STEPS: [u16; 89] = [
    7, 8, 9, 10, 11, 12, 13, 14, 16, 17, 19, 21, 23, 25, 28, 31, 34, 37, 41, 45, 50, 55, 60, 66,
    73, 80, 88, 97, 107, 118, 130, 143, 157, 173, 190, 209, 230, 253, 279, 307, 337, 371, 408, 449,
    494, 544, 598, 658, 724, 796, 876, 963, 1060, 1166, 1282, 1411, 1552, 1707, 1878, 2066, 2272,
    2499, 2749, 3024, 3327, 3660, 4026, 4428, 4871, 5358, 5894, 6484, 7132, 7845, 8630, 9493,
    10442, 11487, 12635, 13899, 15289, 16818, 18500, 20350, 22385, 24623, 27086, 29794, 32767,
];
/// The step index's change by a nibble's magnitude (its low three bits).
const INDEX_CHANGE: [i8; 8] = [-1, -1, -1, -1, 2, 4, 6, 8];
/// The highest step index.
const MAX_INDEX: i8 = 88;

/// Decoded sound: interleaved 16-bit samples.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pcm {
    /// Sample frames per second.
    pub rate: u32,
    /// Channels per frame (1 or more).
    pub channels: u16,
    /// The samples, frame by frame, each frame's channels in order.
    pub samples: Vec<i16>,
}

impl Pcm {
    /// The number of sample frames.
    #[must_use]
    pub fn frames(&self) -> usize {
        self.samples.len() / usize::from(self.channels.max(1))
    }
}

/// The fields of the `fmt ` chunk the decoder uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Format {
    rate: u32,
    channels: usize,
    /// The block's bytes (`nBlockAlign`).
    block: usize,
    /// Sample frames per block (`wSamplesPerBlock`): the header's and the data's.
    frames_per_block: usize,
}

impl Format {
    /// The block header's bytes: per channel a sample, a step index and a reserved byte.
    const fn header(&self) -> usize {
        self.channels * 4
    }

    /// A sub-block's bytes: per channel a 4-byte word of eight nibbles.
    const fn sub_block(&self) -> usize {
        self.channels * 4
    }

    /// The complete frames after the header in `bytes` of block data: eight per whole sub-block,
    /// and in a cut one those every channel's word holds (its last channel's word decides).
    const fn data_frames(&self, bytes: usize) -> usize {
        let sub = self.sub_block();
        let part = bytes % sub;
        let cut = if part > sub - 4 { part % 4 * 2 } else { 0 };
        bytes / sub * 8 + cut
    }
}

/// The chunks the decoder reads.
#[derive(Debug, Clone, Copy)]
struct Chunks<'a> {
    fmt: &'a [u8],
    data: &'a [u8],
    fact: Option<u32>,
}

/// Decodes an IMA ADPCM WAV into 16-bit PCM.
pub fn decode_wav(bytes: &[u8]) -> Result<Pcm, Error> {
    let chunks = chunks(bytes)?;
    let format = format(chunks.fmt)?;
    let frames = frames(&format, chunks.data.len(), chunks.fact);
    let samples = decode(&format, chunks.data, frames);
    Ok(Pcm {
        rate: format.rate,
        channels: u16::try_from(format.channels).unwrap_or_default(),
        samples,
    })
}

/// The first `fmt `, `data` and `fact` chunks inside the RIFF size (the whole file when it is 0),
/// each cut at the file's end. A `fmt ` chunk after the `data` chunk is an error, as for SDL.
fn chunks(bytes: &[u8]) -> Result<Chunks<'_>, Error> {
    if array::<4>(bytes, 0) != Some(*b"RIFF") || array::<4>(bytes, 8) != Some(*b"WAVE") {
        return Err(Error::Magic);
    }
    let size = index(u32_at(bytes, 4).unwrap_or_default());
    let end = if size == 0 {
        bytes.len()
    } else {
        size.saturating_add(8)
    };
    let (mut fmt, mut data, mut fact, mut fact_seen) = (None, None, None, false);
    let mut at = 12;
    while at < end {
        let Some(header) = array::<8>(bytes, at) else {
            break;
        };
        let [i0, i1, i2, i3, l0, l1, l2, l3] = header;
        let length = index(u32::from_le_bytes([l0, l1, l2, l3]));
        let body = bytes.get(at + 8..).unwrap_or_default();
        let body = body.get(..length).unwrap_or(body);
        match &[i0, i1, i2, i3] {
            b"fmt " if fmt.is_none() => {
                if data.is_some() {
                    return Err(Error::Field("chunk order"));
                }
                fmt = Some(body);
            }
            b"data" if data.is_none() => data = Some(body),
            b"fact" if !fact_seen => {
                fact_seen = true;
                fact = u32_at(body, 0);
            }
            _ => {}
        }
        at = (at + 8).saturating_add(length).saturating_add(length & 1);
    }
    Ok(Chunks {
        fmt: fmt.ok_or(Error::Truncated)?,
        data: data.ok_or(Error::Truncated)?,
        fact,
    })
}

/// Reads and checks the `fmt ` chunk: IMA ADPCM, 4 bits, a block that holds its header and the
/// frames it declares (`wSamplesPerBlock` when the extension has it, else as many as fit).
fn format(fmt: &[u8]) -> Result<Format, Error> {
    if fmt.len() < 16 {
        return Err(Error::Truncated);
    }
    let field = |at| u16_at(fmt, at).unwrap_or_default();
    let rate = u32_at(fmt, 4).unwrap_or_default();
    if field(0) != IMA_ADPCM {
        return Err(Error::Field("format"));
    }
    let channels = usize::from(field(2));
    if channels == 0 {
        return Err(Error::Field("channels"));
    }
    if rate == 0 || rate > i32::MAX.unsigned_abs() {
        return Err(Error::Field("sample rate"));
    }
    if field(14) != 4 {
        return Err(Error::Field("bits per sample"));
    }
    let block = usize::from(field(12));
    if block < channels * 4 || block % 4 != 0 {
        return Err(Error::Field("block align"));
    }
    let extension = if fmt.len() >= 18 { field(16) } else { 0 };
    let declared = if fmt.len() >= 20 && extension >= 2 {
        usize::from(field(18))
    } else {
        0
    };
    let mut format = Format {
        rate,
        channels,
        block,
        frames_per_block: declared,
    };
    let fit = format.data_frames(block - format.header());
    if format.frames_per_block == 0 {
        format.frames_per_block = fit + 1;
    }
    if fit < format.frames_per_block - 1 {
        return Err(Error::Field("samples per block"));
    }
    Ok(format)
}

/// The frames to decode from `data` bytes: every whole block's, then those of a cut last block
/// that holds at least its header's samples and step indices; at most `fact`.
fn frames(format: &Format, data: usize, fact: Option<u32>) -> usize {
    let whole = data / format.block * format.frames_per_block;
    let cut = data % format.block;
    let last = if cut + 1 >= format.header() {
        let after = cut.saturating_sub(format.header());
        (1 + format.data_frames(after)).min(format.frames_per_block)
    } else {
        0
    };
    let frames = whole + last;
    fact.map_or(frames, |fact| frames.min(index(fact)))
}

/// Decodes `frames` frames, block by block.
fn decode(format: &Format, data: &[u8], frames: usize) -> Vec<i16> {
    let channels = format.channels;
    let mut samples = Vec::with_capacity(frames * channels);
    let mut left = frames;
    let mut indices = alloc::vec![0i8; channels];
    for block in data.chunks(format.block) {
        if left == 0 {
            break;
        }
        // The header's frame: each channel's first sample and step index.
        for (c, step) in indices.iter_mut().enumerate() {
            let [lo, hi, at] = array::<3>(block, c * 4).unwrap_or_default();
            samples.push(i16::from_le_bytes([lo, hi]));
            *step = i8::from_le_bytes([at]);
        }
        left -= 1;
        let body = block.get(format.header()..).unwrap_or_default();
        let count = left
            .min(format.frames_per_block - 1)
            .min(format.data_frames(body.len()));
        for (g, sub) in body.chunks(format.sub_block()).enumerate() {
            let done = g * 8;
            if done >= count {
                break;
            }
            let n = (count - done).min(8);
            let start = samples.len();
            samples.resize(start + n * channels, 0);
            for (c, (word, step)) in sub.chunks(4).zip(indices.iter_mut()).enumerate() {
                let mut previous = samples
                    .get(start + c - channels)
                    .copied()
                    .unwrap_or_default();
                let outputs = samples.iter_mut().skip(start + c).step_by(channels);
                for (i, out) in outputs.take(n).enumerate() {
                    let byte = word.get(i / 2).copied().unwrap_or_default();
                    let nibble = (byte >> (4 * (i & 1))) & 15;
                    previous = decode_nibble(previous, step, nibble);
                    *out = previous;
                }
            }
        }
        left -= count;
    }
    samples
}

/// The step size at a step index, which counts as 0 to 88.
pub(crate) fn step_size(step_index: i8) -> i32 {
    let at = usize::from(step_index.clamp(0, MAX_INDEX).unsigned_abs());
    i32::from(STEPS.get(at).copied().unwrap_or_default())
}

/// One nibble: the sample it gives after `previous`; the step index moves on.
pub(crate) fn decode_nibble(previous: i16, step_index: &mut i8, nibble: u8) -> i16 {
    let step = step_size(*step_index);
    let change = INDEX_CHANGE
        .get(usize::from(nibble & 7))
        .copied()
        .unwrap_or_default();
    *step_index = (*step_index).clamp(0, MAX_INDEX) + change;
    let mut delta = step >> 3;
    if nibble & 4 != 0 {
        delta += step;
    }
    if nibble & 2 != 0 {
        delta += step >> 1;
    }
    if nibble & 1 != 0 {
        delta += step >> 2;
    }
    if nibble & 8 != 0 {
        delta = -delta;
    }
    let sample = (i32::from(previous) + delta).clamp(i32::from(i16::MIN), i32::from(i16::MAX));
    i16::try_from(sample).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build;
    use alloc::vec;
    use proptest::prelude::*;

    /// A WAV of the given chunks.
    fn wav(chunks: &[(&[u8; 4], &[u8])]) -> Vec<u8> {
        build::riff_wave(chunks)
    }

    /// The game's format: mono 22 050 Hz, blocks of 512 bytes, 1 017 frames each.
    fn game_fmt(channels: u16) -> Vec<u8> {
        build::ima_fmt(22_050, channels, 512 * channels.min(2), Some(1017))
    }

    /// A ramp and a few jumps: every nibble's bits and the clamps get used.
    fn signal(frames: usize, channels: usize) -> Vec<i16> {
        (0..frames * channels)
            .map(|i| {
                let f = i32::try_from(i / channels).unwrap();
                let c = i32::try_from(i % channels).unwrap();
                let v = match f % 600 {
                    0..200 => f * 97 - c * 300,
                    200..300 => 32_767,
                    300..400 => -32_768,
                    _ => (f * 7919) % 20_000 - 10_000,
                };
                i16::try_from(v.clamp(-32_768, 32_767)).unwrap()
            })
            .collect()
    }

    #[test]
    fn nibbles_follow_the_ima_steps() {
        // Step index 0 (step 7): 7 >> 3 = 0, + 7 + 3 + 1 = 11 for nibble 7; the index moves 8.
        let mut index = 0;
        assert_eq!(decode_nibble(0, &mut index, 7), 11);
        assert_eq!(index, 8);
        // Index 8 (step 16): 2 + 16 + 8 + 4 = 30, negative for nibble 15.
        assert_eq!(decode_nibble(41, &mut index, 15), 11);
        assert_eq!(index, 16);
        // Nibbles 0 to 3 move the index down by one, never below 0.
        let mut low = 0;
        assert_eq!(decode_nibble(5, &mut low, 0), 5);
        assert_eq!(low, -1);
        assert_eq!(decode_nibble(5, &mut low, 8), 5);
        assert_eq!(low, -1);
        // Each bit alone at step 16: 2 + 4, 2 + 8, 2 + 16.
        for (nibble, delta) in [(1, 6), (2, 10), (4, 18)] {
            let mut i = 8;
            assert_eq!(decode_nibble(100, &mut i, nibble), 100 + delta);
        }
        // An index above 88 counts as 88 (step 32 767); the samples clamp to 16 bits.
        let mut high = 96;
        assert_eq!(decode_nibble(30_000, &mut high, 7), i16::MAX);
        assert_eq!(high, 96);
        assert_eq!(decode_nibble(-30_000, &mut high, 15), i16::MIN);
        assert_eq!(step_size(-1), 7);
        assert_eq!(step_size(100), 32_767);
    }

    #[test]
    fn a_mono_sound_decodes_to_what_the_encoder_meant() {
        let pcm = signal(3 * 1017 + 500, 1);
        let (data, expected) = build::ima_data(1, 512, &pcm);
        let frames = u32::try_from(expected.len()).unwrap();
        let bytes = wav(&[
            (b"fmt ", &game_fmt(1)),
            (b"fact", &frames.to_le_bytes()),
            (b"data", &data),
        ]);
        let decoded = decode_wav(&bytes).unwrap();
        assert_eq!(decoded.rate, 22_050);
        assert_eq!(decoded.channels, 1);
        assert_eq!(decoded.frames(), pcm.len());
        assert_eq!(decoded.samples, expected);
        // Close to the signal it encodes, but for the jumps.
        let close = decoded
            .samples
            .iter()
            .zip(&pcm)
            .filter(|(a, b)| (i32::from(**a) - i32::from(**b)).abs() < 2_000)
            .count();
        assert!(close > pcm.len() / 2, "{close}");
    }

    #[test]
    fn a_stereo_sound_keeps_each_channel_in_its_word() {
        // 1 017 + 13 frames: the cut block's second sub-block holds five frames, its words three
        // nibbles of padding each, which the fact chunk leaves out.
        let pcm = signal(1017 + 13, 2);
        let (data, expected) = build::ima_data(2, 1024, &pcm);
        let bytes = wav(&[
            (b"fmt ", &game_fmt(2)),
            (b"fact", &1030u32.to_le_bytes()),
            (b"data", &data),
        ]);
        let decoded = decode_wav(&bytes).unwrap();
        assert_eq!(decoded.channels, 2);
        assert_eq!(decoded.frames(), 1017 + 13);
        assert_eq!(decoded.samples, expected);
    }

    #[test]
    fn the_fact_chunk_caps_the_frames() {
        // The second block: its header and twelve whole words.
        let pcm = signal(1017 + 97, 1);
        let (data, expected) = build::ima_data(1, 512, &pcm);
        for (fact, frames) in [(1000, 1000), (1113, 1113), (5000, 1114), (0, 0)] {
            let bytes = wav(&[
                (b"fmt ", &game_fmt(1)),
                (b"fact", &u32::to_le_bytes(fact)),
                (b"data", &data),
            ]);
            let decoded = decode_wav(&bytes).unwrap();
            assert_eq!(decoded.samples, expected[..frames], "fact {fact}");
        }
        // Only the first fact chunk counts, a short one too; one after the data counts.
        let short = wav(&[
            (b"fmt ", &game_fmt(1)),
            (b"fact", &[1, 0]),
            (b"fact", &3u32.to_le_bytes()),
            (b"data", &data),
        ]);
        assert_eq!(decode_wav(&short).unwrap().frames(), 1114);
        let after = wav(&[
            (b"fmt ", &game_fmt(1)),
            (b"data", &data),
            (b"fact", &3u32.to_le_bytes()),
        ]);
        assert_eq!(decode_wav(&after).unwrap().frames(), 3);
    }

    #[test]
    fn a_cut_block_gives_its_complete_frames() {
        let pcm = signal(1017 * 2, 1);
        let (data, expected) = build::ima_data(1, 512, &pcm);
        let fmt = game_fmt(1);
        // Whole blocks, then the second block cut: its header less the reserved byte, the header,
        // the header and one byte (two nibbles), two words (16 frames) and three bytes.
        for (cut, frames) in [
            (0, 1017),
            (1, 1017),
            (2, 1017),
            (3, 1018),
            (4, 1018),
            (5, 1020),
            (4 + 11, 1017 + 1 + 16 + 6),
        ] {
            let bytes = wav(&[(b"fmt ", &fmt), (b"data", &data[..512 + cut])]);
            let decoded = decode_wav(&bytes).unwrap();
            assert_eq!(decoded.samples, expected[..frames], "cut {cut}");
        }
        // A data chunk longer than the file is cut at its end.
        let mut long = wav(&[(b"fmt ", &fmt), (b"data", &data[..600])]);
        let at = long.len() - 600 - 4;
        long[at..at + 4].copy_from_slice(&5000u32.to_le_bytes());
        assert_eq!(decode_wav(&long).unwrap().frames(), 1017 + 1 + 168);
        // Stereo: a cut word of the second channel decides; the first's alone gives nothing.
        let stereo = signal(1017 * 2, 2);
        let (data, expected) = build::ima_data(2, 1024, &stereo);
        let fmt = game_fmt(2);
        for (cut, frames) in [
            (7, 1018),
            (8 + 3, 1018),
            (8 + 5, 1018 + 2),
            (8 + 8, 1018 + 8),
        ] {
            let bytes = wav(&[(b"fmt ", &fmt), (b"data", &data[..1024 + cut])]);
            assert_eq!(
                decode_wav(&bytes).unwrap().samples,
                expected[..frames * 2],
                "cut {cut}"
            );
        }
    }

    #[test]
    fn the_samples_per_block_come_from_the_extension_or_the_block() {
        let pcm = signal(697, 1);
        // No extension: as many as fit, 1 + 508 x 2.
        let (data, expected) = build::ima_data(1, 512, &pcm);
        let fmt = build::ima_fmt(11_025, 1, 512, None);
        let decoded = decode_wav(&wav(&[(b"fmt ", &fmt[..16]), (b"data", &data)])).unwrap();
        assert_eq!((decoded.rate, decoded.samples.len()), (11_025, 697));
        assert_eq!(decoded.samples, expected);
        // An extension of fewer than two bytes, or a zero count, also means "as many as fit".
        let mut short_ext = fmt[..18].to_vec();
        short_ext[16] = 1;
        assert_eq!(
            decode_wav(&wav(&[(b"fmt ", &short_ext), (b"data", &data)]))
                .unwrap()
                .samples,
            expected
        );
        let zero = build::ima_fmt(11_025, 1, 512, Some(0));
        assert_eq!(
            decode_wav(&wav(&[(b"fmt ", &zero), (b"data", &data)]))
                .unwrap()
                .samples,
            expected
        );
        // A smaller count than fits: the rest of each block is padding.
        let small = build::ima_fmt(22_050, 1, 512, Some(9));
        let mut blocks = Vec::new();
        let mut want = Vec::new();
        for chunk in pcm.chunks(9).take(3) {
            let (block, decoded) = build::ima_data(1, 512, chunk);
            let mut block = block;
            block.resize(512, 0xAA);
            blocks.extend_from_slice(&block);
            want.extend_from_slice(&decoded);
        }
        assert_eq!(
            decode_wav(&wav(&[(b"fmt ", &small), (b"data", &blocks)]))
                .unwrap()
                .samples,
            want
        );
    }

    #[test]
    fn the_riff_size_bounds_the_chunks() {
        let pcm = signal(49, 1);
        let (data, _) = build::ima_data(1, 512, &pcm);
        let fmt = game_fmt(1);
        let mut bytes = wav(&[(b"fmt ", &fmt), (b"LIST", b"abc"), (b"data", &data)]);
        // A RIFF size of 0: the whole file.
        bytes[4..8].fill(0);
        assert_eq!(decode_wav(&bytes).unwrap().frames(), 49);
        // A RIFF size that ends before the data chunk: no data.
        bytes[4..8].copy_from_slice(&40u32.to_le_bytes());
        assert_eq!(decode_wav(&bytes).err(), Some(Error::Truncated));
        // A chunk header cut short at the end is ignored.
        let mut tail = wav(&[(b"fmt ", &fmt), (b"data", &data)]);
        tail.extend_from_slice(b"junk");
        tail[4..8].fill(0);
        assert_eq!(decode_wav(&tail).unwrap().frames(), 49);
        // A second fmt chunk after the data is ignored, a second data chunk too.
        let twice = wav(&[
            (b"fmt ", &fmt),
            (b"data", &data),
            (b"fmt ", b"x"),
            (b"data", b""),
        ]);
        assert_eq!(decode_wav(&twice).unwrap().frames(), 49);
    }

    #[test]
    fn malformed_files_are_rejected() {
        let pcm = signal(50, 1);
        let (data, _) = build::ima_data(1, 512, &pcm);
        let fmt = game_fmt(1);
        assert_eq!(decode_wav(b"RIFF").err(), Some(Error::Magic));
        assert_eq!(decode_wav(b"RIFX\0\0\0\0WAVE").err(), Some(Error::Magic));
        assert_eq!(decode_wav(b"RIFF\0\0\0\0WAVF").err(), Some(Error::Magic));
        assert_eq!(
            decode_wav(&wav(&[(b"data", &data)])).err(),
            Some(Error::Truncated)
        );
        assert_eq!(
            decode_wav(&wav(&[(b"fmt ", &fmt)])).err(),
            Some(Error::Truncated)
        );
        assert_eq!(
            decode_wav(&wav(&[(b"data", &data), (b"fmt ", &fmt)])).err(),
            Some(Error::Field("chunk order"))
        );
        let with = |at: usize, value: &[u8]| {
            let mut f = fmt.clone();
            f[at..at + value.len()].copy_from_slice(value);
            decode_wav(&wav(&[(b"fmt ", &f), (b"data", &data)])).err()
        };
        assert_eq!(with(0, &[1, 0]), Some(Error::Field("format")));
        assert_eq!(with(2, &[0, 0]), Some(Error::Field("channels")));
        assert_eq!(with(4, &[0, 0, 0, 0]), Some(Error::Field("sample rate")));
        assert_eq!(with(4, &[0, 0, 0, 0x80]), Some(Error::Field("sample rate")));
        assert_eq!(with(14, &[3, 0]), Some(Error::Field("bits per sample")));
        assert_eq!(with(12, &[2, 0]), Some(Error::Field("block align")));
        assert_eq!(with(12, &[6, 0]), Some(Error::Field("block align")));
        assert_eq!(
            with(18, &[0xFA, 3]),
            Some(Error::Field("samples per block"))
        );
        assert_eq!(
            decode_wav(&wav(&[(b"fmt ", &fmt[..15]), (b"data", &data)])).err(),
            Some(Error::Truncated)
        );
    }

    #[test]
    fn an_empty_data_chunk_has_no_frames() {
        let bytes = wav(&[(b"fmt ", &game_fmt(1)), (b"data", b"")]);
        let pcm = decode_wav(&bytes).unwrap();
        assert!(pcm.samples.is_empty());
        assert_eq!(pcm.frames(), 0);
        assert_eq!(vec![0i16; 0], pcm.samples);
    }

    proptest! {
        #[test]
        fn decode_wav_never_panics(bytes in proptest::collection::vec(any::<u8>(), 0..700)) {
            let _ = decode_wav(&bytes);
        }

        #[test]
        fn a_mutated_wav_never_panics(at in 0usize..600, value: u8) {
            let (data, _) = build::ima_data(2, 64, &signal(40, 2));
            let fmt = build::ima_fmt(22_050, 2, 64, Some(57));
            let mut bytes = wav(&[(b"fmt ", &fmt), (b"fact", &[30, 0, 0, 0]), (b"data", &data)]);
            if let Some(b) = bytes.get_mut(at) {
                *b = value;
            }
            let _ = decode_wav(&bytes);
        }
    }
}
