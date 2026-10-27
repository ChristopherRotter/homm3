//! The map's sound in the window (E24 M2 T5, *provisional*; design §3.11): the timeline's cues
//! played as the original's sound manager plays them, on `h3-sdl`'s audio (one stream a source,
//! SDL mixing them; `h3-view`'s battle sound the model, E17 design §8.2).
//!
//! - **The music** on one stream, looping, at the music's volume (127, a setting of 10). A track
//!   asked for while another plays: the old one fades out (its volume less a tenth every 100 ms:
//!   114 … 12, `music_stop_fade` 0x59B080), its place kept under its name; after 900 ms the new
//!   one starts where its name last stopped (from its start the first time) and fades in (12, 25,
//!   … 114 every 100 ms, then 127 after 900 ms: E17 §8.2). The fades scale the samples as they
//!   are queued, so they follow the device's time (paused with it), as the original's thread
//!   follows real time; a track asked for takes back what is queued of the old one, its fade-out
//!   starting at what plays. A track is decoded
//!   on a thread of its own when first asked for (during the old one's fade-out; with none
//!   playing, it starts once decoded, the frames going on; the map's first before the window
//!   opens).
//! - **The step sound** on one stream (category 0's one sample), looping, at 127.
//! - **The ambient loops**, a stream per slot (4; the original's category 3 takes 8 handles, its
//!   slots at most 4 of them), looping, each at its cue's volume (32, 28, 20 or 10 by distance).
//!
//! The volumes are SDL gains of volume / 127 (both settings at their highest, as `h3-view`'s:
//! no settings yet). The music and the effects (the step and ambient sounds) can be switched
//! off, which holds their gains at 0. A sound that is missing or does not decode costs that
//! sound, noted in the report. An adapter: it reads files and drives the audio device.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::thread::{self, JoinHandle};

use h3_assets::wav::Pcm;
use h3_map_view::sound::{Cue, SLOTS, SoundRules};
use h3_sdl::Audio;

use crate::sounds::{self, Music};

/// The music's and the files' volume on Miles' scale (127).
const FULL: u8 = 127;
/// The fade-in's volumes, one every [`FADE_STEP`] ms (E17 §8.2; `h3_view::sound::FADE`'s).
const FADE_IN: [u8; 9] = [12, 25, 38, 50, 63, 76, 88, 101, 114];
/// The fade-out's volumes, one every [`FADE_STEP`] ms from 127 (design §3.11: an f32 sum
/// truncated), the stream paused after the last.
const FADE_OUT: [u8; 9] = [114, 101, 88, 76, 63, 50, 38, 25, 12];
/// A fade's step, in ms.
const FADE_STEP: u64 = 100;
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
    /// The tracks asked for, by name: none for one missing.
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

    /// Whether a track is decoded (or failed to be), so that taking it does not wait.
    fn ready(&self, name: &str) -> bool {
        self.tracks.contains_key(name)
            || self.decoding.get(name).is_some_and(JoinHandle::is_finished)
    }

    /// A track, its decoding waited for (started if it was not; a missing one noted in `lines`).
    fn track(&mut self, name: &str, lines: &mut Vec<String>) -> Option<&Music> {
        self.request(name);
        if let Some(handle) = self.decoding.remove(name) {
            let music = handle
                .join()
                .unwrap_or_else(|_| Err(format!("{name}: its decoding failed")))
                .map_err(|e| lines.push(format!("music: {e}")))
                .ok();
            self.tracks.insert(name.to_owned(), music);
        }
        self.tracks.get(name).and_then(Option::as_ref)
    }

    /// The tracks the cues ask for, decoded now (the map's first, before the window opens).
    pub fn prepare(&mut self, cues: &[(u32, Cue)], lines: &mut Vec<String>) {
        for (_, cue) in cues {
            if let Cue::Music(name) = cue {
                self.track(name, lines);
            }
        }
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

/// A track queued: its name, the next sample to queue and the samples queued since it started
/// (the fade-in's clock).
#[derive(Debug, Clone)]
struct Track {
    name: String,
    next: usize,
    queued: usize,
}

/// A track fading out: what remains of it, and the track asked for next.
#[derive(Debug, Clone)]
struct FadeOut {
    old: Track,
    /// The fade's samples queued.
    done: usize,
    next: String,
}

/// The audio output of the map.
#[derive(Debug)]
pub struct Player {
    out: Audio,
    music_stream: usize,
    /// The music's format on its stream.
    music_format: (u32, u16),
    music: Option<Track>,
    /// A track asked for while none played, started once decoded (M2 T6a, the reviewer's R2-6:
    /// waiting for it stopped the frames).
    pending: Option<String>,
    /// The track playing fading out for the one asked for (its samples scaled as they are
    /// queued, so the fade follows the device's time).
    fading: Option<FadeOut>,
    /// Where each track stopped (a sample).
    places: BTreeMap<String, usize>,
    step: Lane,
    slots: Vec<Lane>,
    /// Whether the music and the effects are heard.
    pub music_on: bool,
    pub effects_on: bool,
    /// The cues played, for the report.
    pub played: usize,
}

impl Player {
    /// Opens the default playback device and the streams: the music's, the step sound's and the
    /// ambient slots'.
    pub fn open() -> Result<Self, String> {
        let mut out = Audio::open()?;
        let music_stream = out.add_stream(44_100, 2)?;
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
        Ok(Self {
            out,
            music_stream,
            music_format: (44_100, 2),
            music: None,
            pending: None,
            fading: None,
            places: BTreeMap::new(),
            step,
            slots,
            music_on: true,
            effects_on: true,
            played: 0,
        })
    }

    /// Plays a cue (its decoding of a track started at once).
    pub fn cue(&mut self, library: &mut Library, cue: &Cue, lines: &mut Vec<String>) {
        self.played += 1;
        match cue {
            Cue::Music(name) => {
                library.request(name);
                if let Some(fade) = self.fading.as_mut() {
                    fade.next.clone_from(name);
                } else if let Some(old) = self.music.take() {
                    // What is queued but not heard yet is taken back: the fade starts at what
                    // plays now.
                    let len = track_len(library, &old.name);
                    let back = self.out.queued(self.music_stream);
                    self.out.clear(self.music_stream);
                    let old = Track {
                        next: (old.next + len - back % len.max(1)) % len.max(1),
                        queued: old.queued.saturating_sub(back),
                        ..old
                    };
                    self.fading = Some(FadeOut {
                        old,
                        done: 0,
                        next: name.clone(),
                    });
                } else if library.ready(name) {
                    self.pending = None;
                    self.start_music(library, name, lines);
                } else {
                    self.pending = Some(name.clone());
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
        }
    }

    /// A track started where its name last stopped, fading in.
    fn start_music(&mut self, library: &mut Library, name: &str, lines: &mut Vec<String>) {
        let Some((rate, channels, samples)) = library.track(name, lines) else {
            self.music = None;
            return;
        };
        if self.music_format != (*rate, *channels) {
            self.music_format = (*rate, *channels);
            self.out.set_format(self.music_stream, *rate, *channels);
        }
        let place = self.places.get(name).copied().unwrap_or(0);
        self.music = Some(Track {
            name: name.to_owned(),
            next: if place < samples.len() { place } else { 0 },
            queued: 0,
        });
    }

    /// Every sound stopped, the tracks' places forgotten (the window's restart).
    pub fn reset(&mut self) {
        self.out.clear(self.music_stream);
        self.music = None;
        self.pending = None;
        self.fading = None;
        self.places.clear();
        stop(&mut self.out, &mut self.step);
        for lane in &mut self.slots {
            stop(&mut self.out, lane);
        }
    }

    /// The effects heard or not: the step's and the slots' gains.
    pub fn set_effects(&mut self, on: bool) {
        self.effects_on = on;
        for lane in std::iter::once(&self.step).chain(&self.slots) {
            let v = if on { lane.volume } else { 0 };
            self.out.set_gain(lane.stream, gain(v));
        }
    }

    /// Holds or resumes every sound (the fades hold with it: they follow the samples).
    pub fn pause(&mut self, on: bool) {
        self.out.pause(on);
    }

    /// Keeps the loops and the music queued ahead (the fades applied as the music's samples are
    /// queued).
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
        if let Some(name) = self.pending.take() {
            if library.ready(&name) {
                self.start_music(library, &name, lines);
            } else {
                self.pending = Some(name);
            }
        }
        let (rate, channels) = self.music_format;
        let ahead = ahead(rate, channels);
        let step = fade_step(rate, channels);
        while self.out.queued(self.music_stream) < ahead {
            if let Some(mut fade) = self.fading.take() {
                // The fade-out: 900 ms of the old track, a tenth less every 100 ms; then the new
                // track, once decoded (waited for only when the fade has nearly played out).
                let whole = step * FADE_OUT.len();
                if fade.done >= whole {
                    if library.ready(&fade.next) || self.out.queued(self.music_stream) < step {
                        self.end_fade(library, fade, lines);
                        continue;
                    }
                    self.fading = Some(fade);
                    break;
                }
                let Some((_, _, samples)) =
                    library.tracks.get(&fade.old.name).and_then(Option::as_ref)
                else {
                    fade.done = whole;
                    self.fading = Some(fade);
                    continue;
                };
                let n = (whole - fade.done)
                    .min(ahead)
                    .min(samples.len().saturating_sub(fade.old.next));
                if n == 0 {
                    // Nothing left to fade (R2-1): the fade ends.
                    fade.done = whole;
                    self.fading = Some(fade);
                    continue;
                }
                let chunk = samples
                    .get(fade.old.next..fade.old.next + n)
                    .unwrap_or_default();
                let scaled = scale(chunk, fade.done, step, &FADE_OUT, 0);
                if self.out.put(self.music_stream, &scaled, false).is_err() {
                    self.fading = Some(fade);
                    break;
                }
                fade.done += n;
                fade.old.next = (fade.old.next + n) % samples.len().max(1);
                self.fading = Some(fade);
                continue;
            }
            let Some(track) = self.music.as_mut() else {
                break;
            };
            let Some((_, _, samples)) = library.tracks.get(&track.name).and_then(Option::as_ref)
            else {
                break;
            };
            if samples.is_empty() {
                break;
            }
            let end = (track.next + ahead).min(samples.len());
            let chunk = samples.get(track.next..end).unwrap_or_default();
            let scaled = scale(chunk, track.queued, step, &FADE_IN, FULL);
            if self.out.put(self.music_stream, &scaled, false).is_err() {
                break;
            }
            track.queued += end - track.next;
            track.next = if end == samples.len() { 0 } else { end };
        }
    }

    /// A fade-out's end: the old track's place kept under its name, the new one started.
    fn end_fade(&mut self, library: &mut Library, fade: FadeOut, lines: &mut Vec<String>) {
        let channels = usize::from(self.music_format.1.max(1));
        let place = fade.old.next - fade.old.next % channels;
        self.places.insert(fade.old.name, place);
        self.start_music(library, &fade.next, lines);
    }

    /// The device, for the report.
    pub fn describe(&self) -> String {
        self.out.describe()
    }
}

/// A track's samples (0 when it is not decoded).
fn track_len(library: &Library, name: &str) -> usize {
    library
        .tracks
        .get(name)
        .and_then(Option::as_ref)
        .map_or(0, |(_, _, s)| s.len())
}

/// The samples of a fade's step (100 ms at `rate` × `channels`).
fn fade_step(rate: u32, channels: u16) -> usize {
    let samples = u64::from(rate) * u64::from(channels) * FADE_STEP / 1000;
    usize::try_from(samples).unwrap_or(usize::MAX).max(1)
}

/// `chunk` scaled by a fade's volume: its samples counted from `from` in the fade, a step every
/// `step` samples, `after` past the fade's end (integer arithmetic: v × sample / 127).
fn scale(chunk: &[i16], from: usize, step: usize, fade: &[u8; 9], after: u8) -> Vec<i16> {
    chunk
        .iter()
        .enumerate()
        .map(|(i, &sample)| {
            let v = fade.get((from + i) / step).copied().unwrap_or(after);
            let scaled = i32::from(sample) * i32::from(v) / i32::from(FULL);
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
    fn the_fades_scale_the_samples_step_by_step() {
        // 100 ms of 44.1 kHz stereo is 8 820 samples.
        assert_eq!(fade_step(44_100, 2), 8_820);
        assert_eq!(fade_step(0, 0), 1);
        // Samples 0 to 2 of the fade-out at steps 0 (114) and, from 2, 1 (101).
        assert_eq!(
            scale(&[127, -127, 127], 0, 2, &FADE_OUT, 0),
            [114, -114, 101]
        );
        // Past the fade-in's nine steps, the whole; past the fade-out's, nothing.
        assert_eq!(scale(&[1000], 18, 2, &FADE_IN, FULL), [1000]);
        assert_eq!(scale(&[1000], 18, 2, &FADE_OUT, 0), [0]);
        assert_eq!(scale(&[i16::MIN], 0, 1, &FADE_IN, FULL), [-3096]);
        assert_eq!(ahead(22_050, 1), 22_050);
        assert!((gain(FULL) - 1.0).abs() < f32::EPSILON);
    }
}
