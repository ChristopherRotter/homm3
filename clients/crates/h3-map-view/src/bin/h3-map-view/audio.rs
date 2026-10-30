//! The map's sound in the window (E24 M2 T5, *provisional*; design §3.11): the timeline's cues
//! played as the original's sound manager plays them, on `h3-sdl`'s audio (one stream a source,
//! SDL mixing them; `h3-view`'s battle sound the model, E17 design §8.2).
//!
//! - **The music** on one stream as the original's music thread plays it (M2 T6a2:
//!   `h3_ui::music`, the model of `play_music` 0x59ACB0, `music_thread` 0x59A840 and
//!   `music_stop_fade` 0x59B080): a request dropped for the name pending or playing, the old track
//!   faded out (its place kept where the fade began) after any fade-in under way, then the last
//!   asked for faded in where its name last faded out. The model runs on the frames the device
//!   has played, so the fades follow the device's time (paused with it), as the original's thread
//!   follows real time; its copy queues a second ahead, and a request it takes takes that back
//!   (the stream cleared, queued again from what was heard). A track is decoded on a thread of its
//!   own when first asked for (the map's first before the window opens); the music waits for it
//!   (the original opens its file at once). Every track is 44.1 kHz stereo (the nine terrain
//!   tracks are); one of another format is refused, as one that does not decode.
//! - **The step sound** on one stream (category 0's one sample), looping, at 127.
//! - **The ambient loops**, a stream per slot (4; the original's category 3 takes 8 handles, its
//!   slots at most 4 of them), looping, each at its cue's volume (32, 28, 20 or 10 by distance).
//! - **A sound played once** (E24 M3 T5b2: the day's status window's `newday.wav`) on a stream of
//!   its own at 127, the next one cutting it off.
//!
//! The volumes are SDL gains of volume / 127 (both settings at their highest, as `h3-view`'s:
//! no settings yet), the music's fades scaling its samples. The music and the effects (the step
//! and ambient sounds) can be switched off, which holds their gains at 0. A sound that is missing
//! or does not decode costs that sound, noted in the report. An adapter: it reads files and
//! drives the audio device.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::thread::{self, JoinHandle};

use h3_assets::wav::Pcm;
use h3_map_view::sound::{Cue, SLOTS, SoundRules};
use h3_render::Bugs;
use h3_sdl::Audio;
use h3_ui::music::{FULL, Queue, Run, Track};

use crate::sounds::{self, Music};

/// The music stream's format: every track's (44.1 kHz stereo, as the nine terrain tracks).
const MUSIC_RATE: u32 = 44_100;
const MUSIC_CHANNELS: u16 = 2;
/// How far ahead a loop and the music are queued, in ms.
const AHEAD_MS: u64 = 1_000;

/// SDL's gain for a volume on Miles' scale (0 to 127). The adapter's only floating point: SDL
/// takes the gain as a float.
fn gain(volume: u8) -> f32 {
    f32::from(volume) / f32::from(FULL)
}

/// The map's sounds, decoded: every WAV the rules name (when the map loads), the music tracks
/// (when first asked for).
#[derive(Debug)]
pub struct Library {
    install: PathBuf,
    /// The WAVs by stem, upper-cased.
    clips: BTreeMap<String, Pcm>,
    /// The tracks asked for and decoded, by name: none for one missing, not decodable or of
    /// another format than the music stream's.
    tracks: BTreeMap<String, Option<Music>>,
    /// The tracks being decoded.
    decoding: BTreeMap<String, JoinHandle<Result<Music, String>>>,
}

impl Library {
    /// Decodes every WAV `rules` name from the installation's sound archives; one missing or
    /// undecodable is noted in `lines`.
    pub fn load(
        install: &Path,
        rules: &SoundRules,
        lines: &mut Vec<String>,
    ) -> Result<Self, String> {
        let bytes = sounds::read_archives(install)?;
        let archives = sounds::parse_archives(&bytes)?;
        let mut clips = BTreeMap::new();
        let mut missing = Vec::new();
        for name in rules.wavs() {
            let stem = sounds::stem(&name);
            if clips.contains_key(&stem) {
                continue;
            }
            match sounds::decode(&archives, &stem) {
                Some(Ok(pcm)) => {
                    clips.insert(stem, pcm);
                }
                Some(Err(e)) => lines.push(format!("sound: {stem}: {e}")),
                None => missing.push(stem),
            }
        }
        if !missing.is_empty() {
            lines.push(format!("sound: not found: {}", missing.join(", ")));
        }
        Ok(Self {
            install: install.to_path_buf(),
            clips,
            tracks: BTreeMap::new(),
            decoding: BTreeMap::new(),
        })
    }

    /// A WAV by its name.
    fn clip(&self, name: &str) -> Option<&Pcm> {
        self.clips.get(&sounds::stem(name))
    }

    /// A track's decoding started on a thread of its own, unless it is decoded or being decoded.
    #[expect(
        clippy::disallowed_methods,
        reason = "an adapter's thread: a track decodes while the window runs (clippy.toml forbids threads in logic)"
    )]
    pub fn request(&mut self, name: &str) {
        if self.tracks.contains_key(name) || self.decoding.contains_key(name) {
            return;
        }
        let (install, track) = (self.install.clone(), name.to_owned());
        let handle = thread::spawn(move || sounds::mp3(&install, &track));
        self.decoding.insert(name.to_owned(), handle);
    }

    /// A decoding joined: the track kept if it is of the music stream's format, else noted.
    fn join(
        &mut self,
        name: String,
        handle: JoinHandle<Result<Music, String>>,
        lines: &mut Vec<String>,
    ) {
        let music = handle
            .join()
            .unwrap_or_else(|_| Err(format!("{name}: its decoding failed")))
            .and_then(|music| {
                if (music.0, music.1) == (MUSIC_RATE, MUSIC_CHANNELS) {
                    Ok(music)
                } else {
                    Err(format!(
                        "{name}: {} Hz, {} channels (the music's stream takes {MUSIC_RATE} Hz, \
                         {MUSIC_CHANNELS})",
                        music.0, music.1
                    ))
                }
            })
            .map_err(|e| lines.push(format!("music: {e}")))
            .ok();
        self.tracks.insert(name, music);
    }

    /// The decodings that have finished, kept (none waited for).
    fn collect(&mut self, lines: &mut Vec<String>) {
        let done: Vec<String> = self
            .decoding
            .iter()
            .filter(|(_, h)| h.is_finished())
            .map(|(n, _)| n.clone())
            .collect();
        for name in done {
            if let Some(handle) = self.decoding.remove(&name) {
                self.join(name, handle, lines);
            }
        }
    }

    /// What the music knows of a track: its frames once decoded.
    fn state(&self, name: &str) -> Track {
        match self.tracks.get(name) {
            Some(Some((_, channels, samples))) => {
                let frames = samples.len() / usize::from((*channels).max(1));
                Track::Frames(u64::try_from(frames).unwrap_or(u64::MAX))
            }
            Some(None) => Track::Missing,
            None => Track::Decoding,
        }
    }

    /// The tracks the cues ask for, decoded now (the map's first, before the window opens).
    pub fn prepare(&mut self, cues: &[(u32, Cue)], lines: &mut Vec<String>) {
        for (_, cue) in cues {
            if let Cue::Music(name) = cue {
                self.request(name);
                if let Some(handle) = self.decoding.remove(name.as_str()) {
                    self.join(name.clone(), handle, lines);
                }
            }
        }
    }

    /// A run's samples at its volume; none for a track not decoded.
    fn samples(&self, run: &Run) -> Vec<i16> {
        self.tracks
            .get(&run.name)
            .and_then(Option::as_ref)
            .map_or_else(Vec::new, |music| run_samples(music, run))
    }

    /// What was loaded, for the report.
    pub fn describe(&self) -> String {
        let tracks: Vec<&str> = self
            .tracks
            .iter()
            .filter(|(_, m)| m.is_some())
            .map(|(n, _)| n.as_str())
            .collect();
        format!(
            "sound: {} clips decoded; music decoded: {}",
            self.clips.len(),
            if tracks.is_empty() {
                "none".to_owned()
            } else {
                tracks.join(", ")
            }
        )
    }
}

/// A stream playing one clip looping: its format and clip.
#[derive(Debug)]
struct Lane {
    stream: usize,
    format: (u32, u16),
    clip: Option<String>,
    volume: u8,
}

/// The audio output of the map.
#[derive(Debug)]
pub struct Player {
    out: Audio,
    music_stream: usize,
    /// The music as the original's thread plays it, at the frames heard and queued ahead.
    music: Queue,
    /// The original's filter of the music's requests or the client's (MCL-Q12).
    bugs: Bugs,
    step: Lane,
    slots: Vec<Lane>,
    /// The sound played once (E24 M3 T5b2): its stream, never queued again.
    effect: Lane,
    /// Whether the music and the effects are heard.
    pub music_on: bool,
    pub effects_on: bool,
    /// The cues played, for the report.
    pub played: usize,
}

impl Player {
    /// Opens the default playback device and the streams: the music's, the step sound's and the
    /// ambient slots'; the music's requests filtered as `bugs` says.
    pub fn open(bugs: Bugs) -> Result<Self, String> {
        let mut out = Audio::open()?;
        let music_stream = out.add_stream(MUSIC_RATE, MUSIC_CHANNELS)?;
        let lane = |out: &mut Audio| -> Result<Lane, String> {
            Ok(Lane {
                stream: out.add_stream(22_050, 1)?,
                format: (22_050, 1),
                clip: None,
                volume: FULL,
            })
        };
        let step = lane(&mut out)?;
        let slots = (0..SLOTS)
            .map(|_| lane(&mut out))
            .collect::<Result<Vec<_>, _>>()?;
        let effect = lane(&mut out)?;
        Ok(Self {
            out,
            music_stream,
            music: Queue::new(MUSIC_RATE, bugs),
            bugs,
            step,
            slots,
            effect,
            music_on: true,
            effects_on: true,
            played: 0,
        })
    }

    /// The frames the music's stream still holds.
    fn music_queued(&self) -> u64 {
        let frames = self.out.queued(self.music_stream) / usize::from(MUSIC_CHANNELS);
        u64::try_from(frames).unwrap_or(u64::MAX)
    }

    /// Plays a cue (its decoding of a track started at once).
    pub fn cue(&mut self, library: &mut Library, cue: &Cue, lines: &mut Vec<String>) {
        self.played += 1;
        match cue {
            Cue::Music(name) => {
                library.request(name);
                library.collect(lines);
                let queued = self.music_queued();
                let tracks = |n: &str| library.state(n);
                if self.music.request(name, queued, &tracks).changed() {
                    // What is queued is taken back: queued again from what was heard.
                    self.out.clear(self.music_stream);
                }
            }
            Cue::Step(name) => {
                let on = self.effects_on;
                play(&mut self.out, &mut self.step, library, name, FULL, on);
            }
            Cue::StepStop => stop(&mut self.out, &mut self.step),
            Cue::Loop { slot, name, volume } => {
                let on = self.effects_on;
                if let Some(lane) = self.slots.get_mut(*slot) {
                    play(&mut self.out, lane, library, name, *volume, on);
                }
            }
            Cue::LoopStop { slot } => {
                if let Some(lane) = self.slots.get_mut(*slot) {
                    stop(&mut self.out, lane);
                }
            }
            Cue::LoopVolume { slot, volume } => {
                if let Some(lane) = self.slots.get_mut(*slot) {
                    lane.volume = *volume;
                    let v = if self.effects_on { *volume } else { 0 };
                    self.out.set_gain(lane.stream, gain(v));
                }
            }
            Cue::Effect(name) => {
                let on = self.effects_on;
                play(&mut self.out, &mut self.effect, library, name, FULL, on);
                // Once: `service` queues only the loops again.
                self.effect.clip = None;
            }
        }
    }

    /// Every sound stopped, the music's model afresh (the window's restart).
    pub fn reset(&mut self) {
        self.out.clear(self.music_stream);
        self.music = Queue::new(MUSIC_RATE, self.bugs);
        stop(&mut self.out, &mut self.step);
        stop(&mut self.out, &mut self.effect);
        for lane in &mut self.slots {
            stop(&mut self.out, lane);
        }
    }

    /// The effects heard or not: the step's and the slots' gains.
    pub fn set_effects(&mut self, on: bool) {
        self.effects_on = on;
        for lane in [&self.step, &self.effect].into_iter().chain(&self.slots) {
            let v = if on { lane.volume } else { 0 };
            self.out.set_gain(lane.stream, gain(v));
        }
    }

    /// Holds or resumes every sound (the fades hold with it: they follow the frames heard).
    pub fn pause(&mut self, on: bool) {
        self.out.pause(on);
    }

    /// Keeps the loops and the music queued ahead (the music's runs as the model plays them).
    pub fn service(&mut self, library: &mut Library, lines: &mut Vec<String>) {
        for lane in std::iter::once(&self.step).chain(&self.slots) {
            let Some(clip) = lane.clip.as_deref().and_then(|n| library.clip(n)) else {
                continue;
            };
            if self.out.queued(lane.stream) < ahead(clip.rate, clip.channels) {
                let _ = self.out.put(lane.stream, &clip.samples, false);
            }
        }
        self.out.set_gain(
            self.music_stream,
            gain(if self.music_on { FULL } else { 0 }),
        );
        library.collect(lines);
        let queued = self.music_queued();
        let want = u64::from(MUSIC_RATE) * AHEAD_MS / 1000;
        let tracks = |n: &str| library.state(n);
        for run in self.music.top_up(queued, want, &tracks) {
            if self
                .out
                .put(self.music_stream, &library.samples(&run), false)
                .is_err()
            {
                // The model and the stream apart: queued again from what was heard.
                self.out.clear(self.music_stream);
                self.music.cleared();
                break;
            }
        }
    }

    /// The device, for the report.
    pub fn describe(&self) -> String {
        self.out.describe()
    }
}

/// A run's frames of a decoded track, scaled by its volume (integer arithmetic: v × sample /
/// 127); none past the track's end.
fn run_samples((_, channels, samples): &Music, run: &Run) -> Vec<i16> {
    let channels = usize::from(*channels);
    let at = |frame: u64| {
        usize::try_from(frame)
            .unwrap_or(usize::MAX)
            .saturating_mul(channels)
    };
    samples
        .get(at(run.from)..at(run.from.saturating_add(run.frames)))
        .unwrap_or_default()
        .iter()
        .map(|&sample| {
            let scaled = i32::from(sample) * i32::from(run.volume) / i32::from(FULL);
            i16::try_from(scaled).unwrap_or(sample)
        })
        .collect()
}

/// A lane's clip started from its beginning, looping, at `volume` (0 while the effects are off).
fn play(out: &mut Audio, lane: &mut Lane, library: &Library, name: &str, volume: u8, on: bool) {
    out.clear(lane.stream);
    lane.clip = None;
    lane.volume = volume;
    let Some(clip) = library.clip(name) else {
        return;
    };
    if lane.format != (clip.rate, clip.channels) {
        out.set_format(lane.stream, clip.rate, clip.channels);
        lane.format = (clip.rate, clip.channels);
    }
    out.set_gain(lane.stream, gain(if on { volume } else { 0 }));
    if out.put(lane.stream, &clip.samples, false).is_ok() {
        lane.clip = Some(name.to_owned());
    }
}

/// A lane silenced.
fn stop(out: &mut Audio, lane: &mut Lane) {
    out.clear(lane.stream);
    lane.clip = None;
}

/// [`AHEAD_MS`] of samples at `rate` × `channels`.
fn ahead(rate: u32, channels: u16) -> usize {
    let samples = u64::from(rate) * u64::from(channels) * AHEAD_MS / 1000;
    usize::try_from(samples).unwrap_or(usize::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_runs_samples_are_scaled_by_its_volume() {
        let music: Music = (MUSIC_RATE, 2, vec![127, -127, 1000, i16::MIN, 5, 6]);
        let run = |from, frames, volume| Run {
            name: "A".to_owned(),
            from,
            frames,
            volume,
        };
        // Frames 1 and 2 at 114: each channel's sample × 114 / 127.
        assert_eq!(run_samples(&music, &run(1, 2, 114)), [897, -29_413, 4, 5]);
        assert_eq!(run_samples(&music, &run(0, 1, FULL)), [127, -127]);
        // Past the end: nothing.
        assert!(run_samples(&music, &run(2, 2, FULL)).is_empty());
    }

    #[test]
    fn the_volumes_are_gains_and_a_second_is_queued_ahead() {
        assert_eq!(ahead(22_050, 1), 22_050);
        assert_eq!(ahead(MUSIC_RATE, MUSIC_CHANNELS), 88_200);
        assert!((gain(FULL) - 1.0).abs() < f32::EPSILON);
        assert!(gain(0).abs() < f32::EPSILON);
    }
}
