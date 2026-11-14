//! A clip's sound (E27 M2 T2; CLT-Q104, decision 21): the samples of a Bink clip's first track
//! (`h3_bink::audio`) or of a Smacker clip's tracks (`h3_smacker::sound`), each the game's DLL's
//! (`clients/scripts/bink-check.sh`, `smacker-check.sh`), and the volume a clip plays at.
//!
//! **The volume** (`docs/epics/E27-design.md` §8): the game plays a Smacker clip in a dialog at
//! its effects setting (`SmackVolumePan` with 3640 times the setting, 0 to 9, at 0x598CBE), which
//! `SMACKW32.DLL` takes to Miles' scale (0x10008110: `(min(v, 0x8000) × 1270 / 0x8000 + 5) /
//! 10`). It never sets a Bink clip's volume (no `BinkSetVolume`; Miles' master volume untouched),
//! so `BINKW32.DLL`'s Miles sample keeps 127 (0x1000FE4D): the start's clips play at full volume
//! whatever the setting. The client plays every clip at the effects setting (CLT-Q121, decision
//! 27), the original's full volume for the comparisons only ([`Bugs::Original`]).
//!
//! **An 8-bit Bink track** (`H3X1intr`'s, a clip the shell never plays): the DLL decodes 16-bit
//! samples and hands Miles their high bytes (0x10005430), so they play with their low byte 0.

use alloc::vec::Vec;

use h3_bink::Clip as BinkClip;
use h3_bink::audio::Audio;
use h3_render::Bugs;
use h3_smacker::Samples;

/// The flag of a Bink track's 16-bit samples.
const BINK_16: u16 = 0x4000;

/// A track's samples, the channels interleaved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClipSound {
    /// Samples a second.
    pub rate: u32,
    /// Channels (1 or 2).
    pub channels: u16,
    /// The samples, 16-bit.
    pub samples: Vec<i16>,
}

/// A clip's format, which sets its volume.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    /// A Bink clip.
    Bink,
    /// A Smacker clip.
    Smacker,
}

impl ClipSound {
    /// A Bink clip's sound: its first track's, every frame's packet in order; none without a
    /// track.
    pub fn bink(bytes: &[u8]) -> Result<Option<Self>, h3_bink::Error> {
        let clip = BinkClip::parse(bytes)?;
        let Some(&track) = clip.tracks.first() else {
            return Ok(None);
        };
        let mut audio = Audio::new(track)?;
        let mut samples = Vec::new();
        for n in 0..clip.frames().len() {
            if let Some(packet) = clip.audio(n, 0)? {
                let bytes = audio.decode(packet);
                samples.extend(
                    bytes
                        .chunks_exact(2)
                        .map(|b| i16::from_le_bytes([b[0], b[1]])),
                );
            }
        }
        if track.flags & BINK_16 == 0 {
            for s in &mut samples {
                *s &= !0xFF;
            }
        }
        Ok(Some(Self {
            rate: u32::from(track.rate),
            channels: track.channels() as u16,
            samples,
        }))
    }

    /// A Smacker clip's sounds: each track the header describes, in its order (the game opens
    /// all seven: 0x598BAB), its 8-bit samples widened as Miles mixes them ((v − 128) × 256).
    pub fn smacker(bytes: &[u8]) -> Result<Vec<Self>, h3_smacker::Error> {
        let clip = h3_smacker::Clip::parse(bytes)?;
        let mut out: Vec<(usize, Self)> = clip
            .tracks
            .iter()
            .enumerate()
            .filter(|(_, t)| t.rate != 0)
            .map(|(k, t)| {
                let sound = Self {
                    rate: t.rate,
                    channels: t.channels() as u16,
                    samples: Vec::new(),
                };
                (k, sound)
            })
            .collect();
        for packet in clip.reader().flatten() {
            for (k, sound) in &mut out {
                let (Some(data), Some(&track)) = (packet.audio[*k], clip.tracks.get(*k)) else {
                    continue;
                };
                match h3_smacker::sound::decode(track, data) {
                    Ok(Some(Samples::Sixteen(s))) => sound.samples.extend(s),
                    Ok(Some(Samples::Eight(s))) => sound
                        .samples
                        .extend(s.iter().map(|&v| (i16::from(v) - 128) << 8)),
                    _ => {}
                }
            }
        }
        Ok(out.into_iter().map(|(_, sound)| sound).collect())
    }
}

/// The volume on Miles' scale (0 to 127) a clip's sound plays at, by the effects setting (0 to
/// 9): a Smacker clip's as `SMACKW32.DLL` scales the game's 3640 × setting; a Bink clip's the
/// same, 127 in the original's ([`Bugs::Original`], CLT-Q121).
#[must_use]
pub fn volume(format: Format, effects: u8, bugs: Bugs) -> u8 {
    if format == Format::Bink && bugs == Bugs::Original {
        return 127;
    }
    let v = (3640 * u32::from(effects)).min(0x8000);
    u8::try_from((v * 1270 / 0x8000 + 5) / 10).unwrap_or(127)
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    #[test]
    fn a_bink_clips_sound_is_its_first_tracks() {
        let bytes = h3_bink::build::sample(2);
        let sound = ClipSound::bink(&bytes).unwrap().unwrap();
        let clip = BinkClip::parse(&bytes).unwrap();
        assert_eq!(sound.rate, u32::from(clip.tracks[0].rate));
        assert_eq!(sound.channels, clip.tracks[0].channels() as u16);
        let pcm = h3_bink::check::sound(&bytes).unwrap();
        assert_eq!(sound.samples.len() * 2, pcm.len());
        // Without its 16-bit flag the samples keep their high bytes.
        let mut eight = bytes.clone();
        eight[44 + 4 + 3] &= !0x40;
        let low = ClipSound::bink(&eight).unwrap().unwrap();
        assert!(low.samples.iter().all(|s| s & 0xFF == 0));
        assert_eq!(low.samples.len(), sound.samples.len());
        // No track, and a file that is none.
        let silent = h3_bink::build::file(&h3_bink::build::Header::new(16, 8), &[vec![0]]);
        assert_eq!(ClipSound::bink(&silent), Ok(None));
        assert!(ClipSound::bink(b"BIKb").is_err());
        let mut dct = bytes;
        dct[44 + 4 + 3] |= 0x10;
        assert!(ClipSound::bink(&dct).is_err());
        // A frame without a packet adds nothing; a packet past its frame fails.
        let mut header = h3_bink::build::Header::new(16, 8);
        header.audio_tracks = 1;
        header.tracks = vec![h3_bink::audio::Track {
            largest: 4096,
            rate: 22050,
            flags: BINK_16,
        }];
        let empty = h3_bink::build::frame_with_sound(&[], &[0, 0]);
        let none = ClipSound::bink(&h3_bink::build::file(
            &header,
            core::slice::from_ref(&empty),
        ));
        assert_eq!(none.unwrap().unwrap().samples, Vec::<i16>::new());
        let mut past = empty;
        past[0] = 100;
        assert!(ClipSound::bink(&h3_bink::build::file(&header, &[past])).is_err());
    }

    #[test]
    fn a_smacker_clips_sounds_are_its_tracks() {
        // Seeds 6 (16-bit stereo) and 1 (8-bit mono), each one track.
        let s = ClipSound::smacker(&h3_smacker::build::sample(6)).unwrap();
        assert_eq!((s.len(), s[0].rate, s[0].channels), (1, 22050, 2));
        let pcm = h3_smacker::check::sound(&h3_smacker::build::sample(6)).unwrap();
        let bytes: Vec<u8> = s[0].samples.iter().flat_map(|v| v.to_le_bytes()).collect();
        assert_eq!(bytes, pcm[0].1);
        let s = ClipSound::smacker(&h3_smacker::build::sample(1)).unwrap();
        let pcm = h3_smacker::check::sound(&h3_smacker::build::sample(1)).unwrap();
        let widened: Vec<i16> = pcm[0]
            .1
            .iter()
            .map(|&v| (i16::from(v) - 128) << 8)
            .collect();
        assert_eq!(s[0].samples, widened);
        // A packet that fails adds nothing; a clip without tracks has none; no file, an error.
        let mut broken = h3_smacker::build::sample(6);
        let n = broken.len();
        broken[n - 40..].fill(0xFF);
        assert!(ClipSound::smacker(&broken).is_ok());
        // A track Bink-coded (its flags' 0x08, track 0's in the header's byte 75): no samples.
        let mut bink = h3_smacker::build::sample(6);
        bink[75] |= 0x08;
        let s = ClipSound::smacker(&bink).unwrap();
        assert!(s[0].samples.is_empty());
        assert_eq!(
            ClipSound::smacker(&h3_smacker::build::sample(2)),
            Ok(Vec::new())
        );
        assert!(ClipSound::smacker(b"SMK").is_err());
    }

    #[test]
    fn clips_play_at_the_effects_setting_bink_at_full_in_the_original() {
        let smacker: Vec<u8> = (0..10)
            .map(|s| volume(Format::Smacker, s, Bugs::Original))
            .collect();
        assert_eq!(smacker, [0, 14, 28, 42, 56, 71, 85, 99, 113, 127]);
        assert_eq!(volume(Format::Bink, 4, Bugs::Original), 127);
        assert_eq!(volume(Format::Bink, 4, Bugs::Fixed), 56);
        assert_eq!(volume(Format::Smacker, 200, Bugs::Fixed), 127);
    }
}
