//! The battle's sound in the window (E17 M2 T5b; ADR-0024 point 5, `docs/epics/E17-design.md`
//! §8.2): the opening's start sound and music (`h3_view::sound`), and each stack's sounds as the
//! timeline's cues start and stop them. The sounds are decoded when the battle loads: the WAVs
//! of the sound archives by `h3_assets::wav`, the music by `nanomp3`; `h3-sdl` plays the PCM, a
//! stream per source, SDL mixing them.
//!
//! As the executable plays them (`play_sample` 0x59A210): creature sounds share eight channels
//! (their category's, 0x684AB8): a sound takes a free channel, else the next one in turn,
//! cutting what that one played; a move's sound loops until its stop; a sound a creature lacks
//! plays nothing. The tent's heal sound and a resisted spell's (E17 M3 T7b-1) play in category 2,
//! on the start sound's stream (E17 M3 T3b; *provisional*: the category's other three channels
//! are not built), as the start sound at its loader's volume (127, E17 M3 T8); a spell's own sound
//! in the creatures' category 3 at that volume (`play_sound_async` 0x59A590 from 0x5A0331). The battle's end (T8) ends every effect and plays its
//! tune once on the music's stream, at the music's volume, from its start. An adapter: it reads
//! files and drives the audio device.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use h3_assets::snd::SndArchive;
use h3_assets::wav::{Pcm, decode_wav};
use h3_sdl::Audio;
use h3_view::anim::{Cue, Sound};
use h3_view::sound::{
    EFFECT_VOLUME, FILE_VOLUME, Lengths, MUSIC_VOLUME, Opening, START_WAIT, Tune, length_ms,
    music_volume,
};
use h3_view::tables::Sounds;

use crate::gate::find_ci;
use crate::sounds::SOUND_ARCHIVES;

/// The creature sounds' channels (category 3 of 0x684AB8: channels 6 to 13).
const CHANNELS: usize = 8;
/// How far ahead a looping sound and the music are queued, in ms.
const AHEAD_MS: u64 = 1_000;
/// Nanoseconds in a millisecond.
const MS: u64 = 1_000_000;

/// SDL's gain for a volume on Miles' scale (0 to 127). The adapter's only floating point: SDL
/// takes the gain as a float.
fn gain(volume: u8) -> f32 {
    f32::from(volume) / f32::from(MUSIC_VOLUME)
}

/// A decoded MP3: its rate, channels and samples.
type Music = (u32, u16, Vec<i16>);

/// The creatures' sounds a battle's cues may name (E17 M5 T8b2, R6-1): each stack of the record's
/// first snapshot by its id, and each creature the battle may create by its handle (a created
/// stack's cues name its creature, [`Cue::creature`]: an id it reuses keeps no sound of the
/// stack that held it before).
#[derive(Debug, Clone, Default)]
pub struct Voices {
    /// The first snapshot's stacks' sounds, by id.
    pub stacks: BTreeMap<u8, Sounds>,
    /// The sounds of the creatures the battle may create, by handle.
    pub creatures: BTreeMap<u16, Sounds>,
}

impl Voices {
    /// The sounds `cue`'s stack plays: its creature's if the battle created it, else its own.
    pub fn of(&self, cue: &Cue) -> Option<&Sounds> {
        match cue.creature {
            Some(creature) => self.creatures.get(&creature),
            None => self.stacks.get(&cue.stack),
        }
    }

    /// The entry names of every sound.
    fn names(&self) -> impl Iterator<Item = &String> {
        let stacks = self.stacks.values().flat_map(Sounds::names);
        stacks.chain(self.creatures.values().flat_map(Sounds::names))
    }
}

/// The sounds a battle plays, decoded.
#[derive(Debug)]
pub struct Library {
    /// The sound archives' entries the battle names, by upper-cased name.
    clips: BTreeMap<String, Pcm>,
    /// The music: its rate, channels and samples.
    music: Option<Music>,
    /// The end's tunes (E17 M3 T8), each the battle may end with (E17 M5 T1b: a battle fought
    /// as it is played does not know which yet).
    tunes: Vec<(Tune, Music)>,
    /// The start sound's name.
    start: String,
    /// How long the decoding took, in ms.
    took: u128,
    /// The creatures' sounds, by stack and by created creature (E17 M5 T8b2, R6-1).
    voices: Voices,
    /// The battle's own sounds by cue, upper-cased: the tent's heal (E17 M3 T3b), a resisted
    /// spell's and the cast spells' (E17 M3 T7b-1).
    named: BTreeMap<Sound, String>,
}

impl Library {
    /// Decodes the start sound, every sound of `voices` and the battle's own sounds `named` by cue
    /// from the sound archives of `install`, and the music and the end's `tunes` from its `MP3`
    /// folder. A missing or undecodable sound costs that sound, noted in `lines`.
    pub fn load(
        install: &Path,
        opening: Opening,
        (voices, named): (Voices, BTreeMap<Sound, String>),
        tunes: &[Tune],
        lines: &mut Vec<String>,
    ) -> Result<Self, String> {
        let (result, took) =
            crate::gate::timed(|| Self::decode(install, opening, (voices, named), tunes, lines));
        let mut library = result?;
        library.took = took.as_millis();
        Ok(library)
    }

    /// [`Library::load`] without its timing.
    fn decode(
        install: &Path,
        opening: Opening,
        (voices, named): (Voices, BTreeMap<Sound, String>),
        tunes: &[Tune],
        lines: &mut Vec<String>,
    ) -> Result<Self, String> {
        let start = opening.start_sound();
        let named: BTreeMap<Sound, String> = named
            .into_iter()
            .map(|(cue, name)| (cue, name.to_ascii_uppercase()))
            .collect();
        let mut wanted: Vec<String> = voices
            .names()
            .chain(std::iter::once(&start))
            .chain(named.values())
            .map(|n| n.to_ascii_uppercase())
            .collect();
        wanted.sort_unstable();
        wanted.dedup();
        let clips = clips(install, &wanted, lines)?;
        let music = mp3(install, &opening.music_file(), lines)?;
        let mut decoded = Vec::new();
        for &tune in tunes {
            if let Some(music) = mp3(install, tune.file(), lines)? {
                decoded.push((tune, music));
            }
        }
        Ok(Self {
            clips,
            music,
            tunes: decoded,
            start: start.to_ascii_uppercase(),
            took: 0,
            voices,
            named,
        })
    }

    /// How long each sample a cue may name plays (E17 M5 T5b2; `h3_view::sound::Lengths`): the
    /// battle's waits for them.
    pub fn lengths(&self) -> Lengths {
        let ms = |name: &str| {
            self.clips
                .get(&name.to_ascii_uppercase())
                .map(|c| length_ms(c.frames() as u64, c.rate))
        };
        // Each kind of a voice's sounds that has a sample.
        let kinds = |sounds: &Sounds| -> Vec<(Sound, u32)> {
            let length = |kind| sounds.name(kind).and_then(ms).map(|l| (kind, l));
            KINDS.iter().filter_map(|&kind| length(kind)).collect()
        };
        Lengths {
            stacks: (self.voices.stacks.iter())
                .flat_map(|(&id, sounds)| kinds(sounds).into_iter().map(move |(k, l)| ((id, k), l)))
                .collect(),
            creatures: (self.voices.creatures.iter())
                .flat_map(|(&c, sounds)| kinds(sounds).into_iter().map(move |(k, l)| ((c, k), l)))
                .collect(),
            named: self
                .named
                .iter()
                .filter_map(|(&cue, name)| ms(name).map(|l| (cue, l)))
                .collect(),
        }
    }
}

/// A stack's eight sounds (`stack_load_graphics` 0x43D9F0's order).
const KINDS: [Sound; 8] = [
    Sound::Move,
    Sound::Attack,
    Sound::Wince,
    Sound::Shot,
    Sound::Death,
    Sound::Defend,
    Sound::Start,
    Sound::End,
];

/// How long the samples of `voices` and `named` play (E17 M5 T5b2): [`Library::lengths`] without
/// the music and the start sound, for a comparison that plays its sounds (`h3-view sequences`'s
/// `sound` sequences).
pub fn lengths(
    install: &Path,
    (voices, named): (Voices, BTreeMap<Sound, String>),
    lines: &mut Vec<String>,
) -> Result<Lengths, String> {
    let named: BTreeMap<Sound, String> = named
        .into_iter()
        .map(|(cue, name)| (cue, name.to_ascii_uppercase()))
        .collect();
    let mut wanted: Vec<String> = voices
        .names()
        .chain(named.values())
        .map(|n| n.to_ascii_uppercase())
        .collect();
    wanted.sort_unstable();
    wanted.dedup();
    let library = Library {
        clips: clips(install, &wanted, lines)?,
        music: None,
        tunes: Vec::new(),
        start: String::new(),
        took: 0,
        voices,
        named,
    };
    Ok(library.lengths())
}

/// The sound archives' entries `wanted` (upper-cased), decoded; a missing or undecodable one
/// noted in `lines`.
fn clips(
    install: &Path,
    wanted: &[String],
    lines: &mut Vec<String>,
) -> Result<BTreeMap<String, Pcm>, String> {
    let mut clips = BTreeMap::new();
    if let Ok(data) = find_ci(install, "Data") {
        for archive in SOUND_ARCHIVES {
            let Ok(path) = find_ci(&data, archive) else {
                continue;
            };
            let bytes = fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
            let snd = SndArchive::parse(&bytes).map_err(|e| format!("{archive}: {e:?}"))?;
            for name in wanted {
                if clips.contains_key(name) {
                    continue;
                }
                // A sound that does not decode costs that sound, as a missing one does.
                match snd.find(name).map(decode_wav) {
                    Some(Ok(pcm)) => {
                        clips.insert(name.clone(), pcm);
                    }
                    Some(Err(e)) => lines.push(format!("sound: {name}: {e:?}")),
                    None => {}
                }
            }
        }
    }
    let missing: Vec<&str> = wanted
        .iter()
        .filter(|n| !clips.contains_key(*n))
        .map(String::as_str)
        .collect();
    if !missing.is_empty() {
        lines.push(format!("sound: not found: {}", missing.join(", ")));
    }
    Ok(clips)
}

impl Library {
    /// The end's `tune`, if it was decoded.
    fn tune(&self, tune: Tune) -> Option<&Music> {
        self.tunes.iter().find(|(t, _)| *t == tune).map(|(_, m)| m)
    }

    /// The clip `cue` plays, if the creature has it and it was found (a created stack's by its
    /// creature, [`Voices::of`]).
    fn clip(&self, cue: &Cue) -> Option<&Pcm> {
        if let Some(name) = self.named.get(&cue.sound) {
            return self.clips.get(name);
        }
        let name = self.voices.of(cue)?.name(cue.sound)?;
        self.clips.get(&name.to_ascii_uppercase())
    }

    /// What was loaded, for the report.
    pub fn describe(&self) -> String {
        let music = self.music.as_ref().map_or_else(
            || "no music".to_owned(),
            |(rate, channels, samples)| {
                let seconds = samples.len() / usize::from(*channels).max(1) / *rate as usize;
                format!("music {rate} Hz × {channels}, {seconds} s")
            },
        );
        let tune = match self.tunes.len() {
            0 => String::new(),
            1 => ", the end's tune".to_owned(),
            n => format!(", the end's {n} tunes"),
        };
        format!(
            "sound: {} clips, {music}{tune}, decoded in {} ms",
            self.clips.len(),
            self.took
        )
    }
}

/// One channel of the creature sounds: its stream, the format queued on it, what plays there.
#[derive(Debug)]
struct Channel {
    stream: usize,
    format: (u32, u16),
    /// The cue that plays here (its stack, its sound and its voice).
    owner: Option<Cue>,
    /// Whether that sound loops.
    looping: bool,
}

/// The audio output of a battle.
#[derive(Debug)]
pub struct Player {
    out: Audio,
    music_stream: usize,
    start_stream: usize,
    channels: Vec<Channel>,
    /// The next channel taken when none is free.
    turn: usize,
    /// When the music started (SDL's nanoseconds; moved on by each pause) and the next sample to
    /// queue.
    music: Option<(u64, usize)>,
    /// The music's format on its stream.
    music_format: (u32, u16),
    /// The end's tune playing instead of the music, and its next sample to queue (E17 M3 T8).
    tune: Option<(Tune, usize)>,
    /// Since when the sound is held.
    paused: Option<u64>,
    /// Whether the music and the effects are heard (the keys M and E).
    pub music_on: bool,
    pub effects_on: bool,
}

impl Player {
    /// Opens the default playback device and the streams: the music's, the start sound's and the
    /// creature sounds' channels.
    pub fn open(library: &Library) -> Result<Self, String> {
        let mut out = Audio::open()?;
        let (rate, channels) = library.music.as_ref().map_or((44_100, 2), |m| (m.0, m.1));
        let music_stream = out.add_stream(rate, channels)?;
        let start_stream = out.add_stream(22_050, 1)?;
        out.set_gain(start_stream, gain(FILE_VOLUME));
        let mut list = Vec::with_capacity(CHANNELS);
        for _ in 0..CHANNELS {
            let stream = out.add_stream(22_050, 1)?;
            out.set_gain(stream, gain(EFFECT_VOLUME));
            list.push(Channel {
                stream,
                format: (22_050, 1),
                owner: None,
                looping: false,
            });
        }
        Ok(Self {
            out,
            music_stream,
            start_stream,
            channels: list,
            turn: 0,
            music: None,
            music_format: (rate, channels),
            tune: None,
            paused: None,
            music_on: true,
            effects_on: true,
        })
    }

    /// Plays the start sound; how long the battle waits for it, in ms (at most [`START_WAIT`]).
    pub fn start(&mut self, library: &Library) -> u32 {
        let Some(clip) = library.clips.get(&library.start) else {
            return 0;
        };
        self.out
            .set_format(self.start_stream, clip.rate, clip.channels);
        if self
            .out
            .put(self.start_stream, &clip.samples, true)
            .is_err()
        {
            return 0;
        }
        let ms = clip.frames() as u64 * 1000 / u64::from(clip.rate.max(1));
        u32::try_from(ms).unwrap_or(u32::MAX).min(START_WAIT)
    }

    /// Starts the music at `now` (SDL's nanoseconds); the creature sounds still playing stop
    /// (0x59ADAC). After the end's tune (the battle shown again from earlier), its stream takes
    /// the music's format back.
    pub fn start_music(&mut self, now: u64) {
        self.silence();
        if self.tune.take().is_some() {
            self.out.clear(self.music_stream);
            let (rate, channels) = self.music_format;
            self.out.set_format(self.music_stream, rate, channels);
        }
        self.music = Some((now, 0));
    }

    /// The battle's end (E17 M3 T8, design §8.2): every effect ends, the start sound's stream
    /// too, and the end's tune replaces the music, played once from its start at the music's
    /// volume (`play_music` with a loop count of 1: no fade-in): `tune`, if it was decoded.
    pub fn end(&mut self, library: &Library, tune: Tune) {
        self.silence();
        self.out.clear(self.start_stream);
        self.out.clear(self.music_stream);
        self.music = None;
        if let Some((rate, channels, _)) = library.tune(tune) {
            self.out.set_format(self.music_stream, *rate, *channels);
            self.tune = Some((tune, 0));
        }
    }

    /// Plays a cue: starts the stack's sound on a channel, or stops its looping move sound.
    pub fn cue(&mut self, library: &Library, cue: &Cue) {
        if cue.stop {
            for c in &mut self.channels {
                let own = c
                    .owner
                    .is_some_and(|o| (o.stack, o.sound) == (cue.stack, cue.sound));
                if own && c.looping {
                    self.out.clear(c.stream);
                    c.owner = None;
                    c.looping = false;
                }
            }
            return;
        }
        if !self.effects_on {
            return;
        }
        let Some(clip) = library.clip(cue) else {
            return;
        };
        if matches!(
            cue.sound,
            Sound::Heal
                | Sound::Resisted
                | Sound::GoodMorale
                | Sound::BadMorale
                | Sound::Fear
                | Sound::Drain
                | Sound::Thunderbolt
                | Sound::SpellFile(_)
                | Sound::Drawbridge
                | Sound::Mine
                | Sound::TurretShot
        ) {
            // Category 2, the start sound's (`play_sound_file` 0x59A470; E17 M3 T3b, T7b-1,
            // T7b-2b-2b, T7b-2b-3, M4 T2c-1; a turret's shot, M4 T3a, R2-1).
            self.out.clear(self.start_stream);
            self.out
                .set_format(self.start_stream, clip.rate, clip.channels);
            let _ = self.out.put(self.start_stream, &clip.samples, true);
            return;
        }
        let free = self
            .channels
            .iter()
            .position(|c| !c.looping && self.out.queued(c.stream) == 0);
        let index = free.unwrap_or_else(|| {
            let i = self.turn % CHANNELS;
            self.turn = i + 1;
            i
        });
        let Some(channel) = self.channels.get_mut(index) else {
            return;
        };
        self.out.clear(channel.stream);
        // A spell's sound, good luck's, an object's and a wall shot's landing at their loader's
        // volume, a creature's at the effects' (E17 M3 T7b-1, T7b-2b-2b; T7c-2: `wav_load`
        // 0x55C720, *provisional*; the landing since E17 M5 T5b2, 0x44618B: volume 127).
        let volume = match cue.sound {
            Sound::Spell(_) | Sound::Luck | Sound::Object(_) | Sound::WallHit | Sound::WallMiss => {
                FILE_VOLUME
            }
            _ => EFFECT_VOLUME,
        };
        self.out.set_gain(channel.stream, gain(volume));
        if channel.format != (clip.rate, clip.channels) {
            self.out
                .set_format(channel.stream, clip.rate, clip.channels);
            channel.format = (clip.rate, clip.channels);
        }
        // A one-shot is flushed so that it plays to its end; a loop is topped up instead.
        let looping = cue.sound == Sound::Move;
        if self
            .out
            .put(channel.stream, &clip.samples, !looping)
            .is_ok()
        {
            channel.owner = Some(*cue);
            channel.looping = looping;
        }
    }

    /// Stops every creature sound (a seek, the music's start).
    pub fn silence(&mut self) {
        for c in &mut self.channels {
            self.out.clear(c.stream);
            c.owner = None;
            c.looping = false;
        }
    }

    /// Holds or resumes every sound (the battle paused or running) at `now`; the music's fade-in
    /// holds with it.
    pub fn pause(&mut self, on: bool, now: u64) {
        self.out.pause(on);
        if on {
            self.paused = Some(now);
        } else if let Some(at) = self.paused.take()
            && let Some((started, _)) = self.music.as_mut()
        {
            *started += now.saturating_sub(at);
        }
    }

    /// Keeps the loops and the music queued ahead and the music's fade-in going, at `now`.
    pub fn service(&mut self, library: &Library, now: u64) {
        for c in &self.channels {
            let Some(owner) = c.owner.filter(|_| c.looping) else {
                continue;
            };
            let Some(clip) = library.clip(&owner) else {
                continue;
            };
            let ahead = ahead(clip.rate, clip.channels);
            if self.out.queued(c.stream) < ahead {
                let _ = self.out.put(c.stream, &clip.samples, false);
            }
        }
        let playing = self.tune.and_then(|(t, _)| library.tune(t));
        if let (Some((_, next)), Some((rate, channels, samples))) = (self.tune.as_mut(), playing) {
            let volume = if self.music_on { MUSIC_VOLUME } else { 0 };
            self.out.set_gain(self.music_stream, gain(volume));
            // Once: queued up to its end, never from its start again.
            let ahead = ahead(*rate, *channels);
            while self.out.queued(self.music_stream) < ahead && *next < samples.len() {
                let end = (*next + ahead).min(samples.len());
                let chunk = samples.get(*next..end).unwrap_or_default();
                if self
                    .out
                    .put(self.music_stream, chunk, end == samples.len())
                    .is_err()
                {
                    break;
                }
                *next = end;
            }
            return;
        }
        let (Some((started, next)), Some((rate, channels, samples))) =
            (self.music.as_mut(), library.music.as_ref())
        else {
            return;
        };
        let ms = u32::try_from(now.saturating_sub(*started) / MS).unwrap_or(u32::MAX);
        let volume = if self.music_on { music_volume(ms) } else { 0 };
        self.out.set_gain(self.music_stream, gain(volume));
        let ahead = ahead(*rate, *channels);
        while self.out.queued(self.music_stream) < ahead && !samples.is_empty() {
            let end = (*next + ahead).min(samples.len());
            let chunk = samples.get(*next..end).unwrap_or_default();
            if self.out.put(self.music_stream, chunk, false).is_err() {
                break;
            }
            *next = if end == samples.len() { 0 } else { end };
        }
    }

    /// The device, for the report.
    pub fn describe(&self) -> String {
        self.out.describe()
    }
}

/// The MP3 `stem` of the installation's `MP3` folder (found ignoring case), decoded; none, noted in
/// `lines`, when it is missing or holds no audio.
fn mp3(install: &Path, stem: &str, lines: &mut Vec<String>) -> Result<Option<Music>, String> {
    let file = format!("{stem}.mp3");
    match find_ci(install, "MP3").and_then(|d| find_ci(&d, &file)) {
        Ok(path) => {
            let bytes = fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
            let decoded = nanomp3::decode_all::<i16>(&bytes);
            let channels = decoded.channels.map_or(0, |c| c as u16);
            if channels == 0 || decoded.sample_rate == 0 {
                lines.push(format!("music: {} holds no audio", path.display()));
                Ok(None)
            } else {
                Ok(Some((decoded.sample_rate, channels, decoded.samples)))
            }
        }
        Err(e) => {
            lines.push(format!("music: {file}: {e}"));
            Ok(None)
        }
    }
}

/// [`AHEAD_MS`] of samples at `rate` × `channels`.
fn ahead(rate: u32, channels: u16) -> usize {
    let samples = u64::from(rate) * u64::from(channels) * AHEAD_MS / 1000;
    usize::try_from(samples).unwrap_or(usize::MAX)
}
