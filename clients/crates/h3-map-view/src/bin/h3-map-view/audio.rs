//! The map's sound in the window (E24 M2 T5, *provisional*; design §3.11): the timeline's cues
//! played as the original's sound manager plays them, on `h3-sdl`'s audio (one stream a source,
//! SDL mixing them; `h3-view`'s battle sound the model, E17 design §8.2).
//!
//! - **The device and the music are the host's sound** (E24 M4 T5b1, *provisional*:
//!   `h3_battle_client::audio::HostSound`, E17 design §13.6), lent to a battle the window hosts
//!   and handed back once it closed ([`Player::lend`], [`Player::take_back`]): one music for the
//!   map and its battles, as the original's one sound manager. The music on one stream as the
//!   original's music thread plays it (M2 T6a2: `h3_ui::music`, the model of `play_music`
//!   0x59ACB0, `music_thread` 0x59A840 and `music_stop_fade` 0x59B080): a request dropped for the
//!   name pending or playing, the old track faded out (its place kept where the fade began) after
//!   any fade-in under way, then the last asked for faded in where its name last faded out. The
//!   model runs on the frames the device has played, so the fades follow the device's time
//!   (paused with it), as the original's thread follows real time; it queues a second ahead, and a
//!   request it takes takes that back. A track is decoded on a thread of its own when first asked
//!   for (the map's first before the window opens) and added to the host's sound once decoded; the
//!   music waits for it (the original opens its file at once). Every track is 44.1 kHz stereo (the
//!   nine terrain tracks are); one of another format is kept as missing, noted.
//! - **The step sound** on one stream (category 0's one sample), looping, at 127.
//! - **The ambient loops**, a stream per slot (4; the original's category 3 takes 8 handles, its
//!   slots at most 4 of them), looping, each at its cue's volume (32, 28, 20 or 10 by distance).
//! - **A sound played once** (E24 M3 T5b2: the day's status window's `newday.wav`) on a stream of
//!   its own at 127, the next one cutting it off.
//!
//! - **The focus** (E24 M3 T6b3, design §4.15): lost, every effect playing is held where it is
//!   (its stream unbound) and the music fades out as the original's stub thread fades it;
//!   regained, the effects held play on and the music fades in where its fade began. A cue
//!   meanwhile plays, as the original starts a sample anew.
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
use h3_battle_client::audio::HostSound;
use h3_map_view::sound::{Cue, SLOTS, SoundRules};
use h3_render::Bugs;
use h3_sdl::Audio;
use h3_ui::music::FULL;

use crate::sounds::{self, Music};

/// The music stream's format: every track's (44.1 kHz stereo, as the nine terrain tracks; the host's
/// sound keeps a track of another format as missing).
const MUSIC_RATE: u32 = 44_100;
const MUSIC_CHANNELS: u16 = 2;
/// How far ahead a loop is queued, in ms.
const AHEAD_MS: u64 = 1_000;

/// SDL's gain for a volume on Miles' scale (0 to 127). The adapter's only floating point: SDL
/// takes the gain as a float.
fn gain(volume: u8) -> f32 {
    f32::from(volume) / f32::from(FULL)
}

/// The map's sounds, decoded: every WAV the rules name (when the map loads), the music tracks
/// (when first asked for, then kept by the host's sound).
#[derive(Debug)]
pub struct Library {
    install: PathBuf,
    /// The WAVs by stem, upper-cased.
    clips: BTreeMap<String, Pcm>,
    /// The tracks decoded and of the music's format, by name (the report's).
    decoded: Vec<String>,
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
            decoded: Vec::new(),
            decoding: BTreeMap::new(),
        })
    }

    /// A WAV by its name.
    fn clip(&self, name: &str) -> Option<&Pcm> {
        self.clips.get(&sounds::stem(name))
    }

    /// A track's decoding started on a thread of its own, unless `host` has it or it is being
    /// decoded.
    #[expect(
        clippy::disallowed_methods,
        reason = "an adapter's thread: a track decodes while the window runs (clippy.toml forbids threads in logic)"
    )]
    pub fn request(&mut self, name: &str, host: &HostSound) {
        if host.has_track(name) || self.decoding.contains_key(name) {
            return;
        }
        let (install, track) = (self.install.clone(), name.to_owned());
        let handle = thread::spawn(move || sounds::mp3(&install, &track));
        self.decoding.insert(name.to_owned(), handle);
    }

    /// A decoding joined: the track added to `host`, as missing if it did not decode or is not of
    /// the music stream's format (noted).
    fn join(
        &mut self,
        name: String,
        handle: JoinHandle<Result<Music, String>>,
        host: &mut HostSound,
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
        if music.is_some() {
            self.decoded.push(name.clone());
        }
        host.add_track(&name, music);
    }

    /// The decodings that have finished, added to `host` (none waited for).
    fn collect(&mut self, host: &mut HostSound, lines: &mut Vec<String>) {
        let done: Vec<String> = self
            .decoding
            .iter()
            .filter(|(_, h)| h.is_finished())
            .map(|(n, _)| n.clone())
            .collect();
        for name in done {
            if let Some(handle) = self.decoding.remove(&name) {
                self.join(name, handle, host, lines);
            }
        }
    }

    /// The tracks the cues ask for, decoded now and added to `player`'s sound (the map's first,
    /// before the window opens).
    pub fn prepare(&mut self, cues: &[(u32, Cue)], player: &mut Player, lines: &mut Vec<String>) {
        let Some(host) = player.host.as_mut() else {
            return;
        };
        for (_, cue) in cues {
            if let Cue::Music(name) = cue {
                self.request(name, host);
                if let Some(handle) = self.decoding.remove(name.as_str()) {
                    self.join(name.clone(), handle, host, lines);
                }
            }
        }
    }

    /// What was loaded, for the report.
    pub fn describe(&self) -> String {
        format!(
            "sound: {} clips decoded; music decoded: {}",
            self.clips.len(),
            if self.decoded.is_empty() {
                "none".to_owned()
            } else {
                self.decoded.join(", ")
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
    /// Held where it is while the game lacks the focus.
    held: bool,
}

/// The audio output of the map, on the host's sound (none while a battle has it).
#[derive(Debug)]
pub struct Player {
    host: Option<HostSound>,
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
    /// Opens the host's sound (the default playback device and the music's stream, its requests
    /// filtered as `bugs` says) and the map's streams on its device: the step sound's and the
    /// ambient slots'.
    pub fn open(bugs: Bugs) -> Result<Self, String> {
        let mut host = HostSound::open(bugs)?;
        let lane = |out: &mut Audio| -> Result<Lane, String> {
            Ok(Lane {
                stream: out.add_stream(22_050, 1)?,
                format: (22_050, 1),
                clip: None,
                volume: FULL,
                held: false,
            })
        };
        let step = lane(host.audio())?;
        let slots = (0..SLOTS)
            .map(|_| lane(host.audio()))
            .collect::<Result<Vec<_>, _>>()?;
        let effect = lane(host.audio())?;
        Ok(Self {
            host: Some(host),
            step,
            slots,
            effect,
            music_on: true,
            effects_on: true,
            played: 0,
        })
    }

    /// Plays a cue (its decoding of a track started at once); nothing while a battle has the
    /// sound.
    pub fn cue(&mut self, library: &mut Library, cue: &Cue, lines: &mut Vec<String>) {
        let Some(host) = self.host.as_mut() else {
            return;
        };
        self.played += 1;
        let on = self.effects_on;
        match cue {
            Cue::Music(name) => {
                library.request(name, host);
                library.collect(host, lines);
                host.request(name, false);
            }
            Cue::Step(name) => play(host.audio(), &mut self.step, library, name, FULL, on),
            Cue::StepStop => stop(host.audio(), &mut self.step),
            Cue::Loop { slot, name, volume } => {
                if let Some(lane) = self.slots.get_mut(*slot) {
                    play(host.audio(), lane, library, name, *volume, on);
                }
            }
            Cue::LoopStop { slot } => {
                if let Some(lane) = self.slots.get_mut(*slot) {
                    stop(host.audio(), lane);
                }
            }
            Cue::LoopVolume { slot, volume } => {
                if let Some(lane) = self.slots.get_mut(*slot) {
                    lane.volume = *volume;
                    let v = if on { *volume } else { 0 };
                    host.audio().set_gain(lane.stream, gain(v));
                }
            }
            Cue::Effect(name) => {
                play(host.audio(), &mut self.effect, library, name, FULL, on);
                // Once: `service` queues only the loops again.
                self.effect.clip = None;
            }
        }
    }

    /// The map's streams silenced (the step's, the slots' and the sound played once).
    fn stop_effects(&mut self) {
        if let Some(host) = self.host.as_mut() {
            let out = host.audio();
            for lane in [&mut self.step, &mut self.effect]
                .into_iter()
                .chain(&mut self.slots)
            {
                stop(out, lane);
            }
        }
    }

    /// Every sound stopped, the music's model afresh (the window's restart).
    pub fn reset(&mut self) {
        if let Some(host) = self.host.as_mut() {
            host.reset();
        }
        self.stop_effects();
    }

    /// The effects heard or not: the step's and the slots' gains.
    pub fn set_effects(&mut self, on: bool) {
        self.effects_on = on;
        let Some(host) = self.host.as_mut() else {
            return;
        };
        for lane in [&self.step, &self.effect].into_iter().chain(&self.slots) {
            let v = if on { lane.volume } else { 0 };
            host.audio().set_gain(lane.stream, gain(v));
        }
    }

    /// The game's focus lost or regained (`window_procedure` 0x4F7C00, WM_ACTIVATEAPP; design
    /// §4.15). Lost (`sound_focus_lost` 0x599C40): each effect playing held where it is (its
    /// sample stopped, AIL_stop_sample, whether it played kept at 0x6A3258) and the music faded
    /// out by the model (`music_thread`'s stub 0x59A830); regained: the effects held play on
    /// (0x599B90: AIL_resume_sample on each that played and stands stopped) and the music asked
    /// for again (`music_resume` 0x59AC00).
    pub fn focus(&mut self, library: &mut Library, on: bool, lines: &mut Vec<String>) {
        let Some(host) = self.host.as_mut() else {
            return;
        };
        library.collect(host, lines);
        host.focus(on);
        let out = host.audio();
        for lane in [&mut self.step, &mut self.effect]
            .into_iter()
            .chain(&mut self.slots)
        {
            let hold = !on && out.queued(lane.stream) > 0;
            if hold != lane.held {
                out.hold(lane.stream, hold);
                lane.held = hold;
            }
        }
    }

    /// Holds or resumes every sound (the fades hold with it: they follow the frames heard).
    pub fn pause(&mut self, on: bool) {
        if let Some(host) = self.host.as_mut() {
            host.audio().pause(on);
        }
    }

    /// Keeps the loops and the music queued ahead (the music's runs as the model plays them).
    pub fn service(&mut self, library: &mut Library, lines: &mut Vec<String>) {
        let Some(host) = self.host.as_mut() else {
            return;
        };
        for lane in std::iter::once(&self.step).chain(&self.slots) {
            let Some(clip) = lane.clip.as_deref().and_then(|n| library.clip(n)) else {
                continue;
            };
            let out = host.audio();
            if out.queued(lane.stream) < ahead(clip.rate, clip.channels) {
                let _ = out.put(lane.stream, &clip.samples, false);
            }
        }
        library.collect(host, lines);
        host.service(if self.music_on { FULL } else { 0 });
    }

    /// The host's sound lent to a battle (E24 M4 T5b1, design §5.19: the adventure manager's
    /// exit stops the effects, `adventure_manager_exit` 0x4077E0, and lets the map's track play
    /// on into the battle); none if it is lent already.
    pub fn lend(&mut self) -> Option<HostSound> {
        self.stop_effects();
        self.host.take()
    }

    /// The host's sound handed back by the battle (its own streams removed, the map's streams
    /// still its first). The map's music is not asked for again (MCL-Q47 (a), design §5.19): the
    /// battle's result's tune plays out and nothing follows until the map asks for a track.
    pub fn take_back(&mut self, host: HostSound) {
        self.host = Some(host);
        let on = self.effects_on;
        self.set_effects(on);
    }

    /// The device, for the report.
    pub fn describe(&self) -> String {
        self.host
            .as_ref()
            .map_or_else(|| "sound lent to a battle".to_owned(), HostSound::describe)
    }
}

/// A lane's clip started from its beginning, looping, at `volume` (0 while the effects are off).
fn play(out: &mut Audio, lane: &mut Lane, library: &Library, name: &str, volume: u8, on: bool) {
    stop(out, lane);
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

/// A lane silenced (one held plays on: empty).
fn stop(out: &mut Audio, lane: &mut Lane) {
    out.clear(lane.stream);
    lane.clip = None;
    if lane.held {
        out.hold(lane.stream, false);
        lane.held = false;
    }
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
    fn the_volumes_are_gains_and_a_second_is_queued_ahead() {
        assert_eq!(ahead(22_050, 1), 22_050);
        assert_eq!(ahead(MUSIC_RATE, MUSIC_CHANNELS), 88_200);
        assert!((gain(FULL) - 1.0).abs() < f32::EPSILON);
        assert!(gain(0).abs() < f32::EPSILON);
    }
}
