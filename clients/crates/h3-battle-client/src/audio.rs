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
//! in the creatures' category 3 at that volume (`play_sound_async` 0x59A590 from 0x5A0331). The battle's end (T8) ends every effect and asks for its
//! tune, played once (E17 M7 T3c: after the combat music's 900 ms fade-out). Every volume is taken
//! through the combat options' settings (E17 M6 T4a, design §12.12; `sound_volume` 0x5996C0,
//! `h3_view::options::sound_volume`): the music's by the music setting, every other by the
//! effects', changed at once as the dialog's bars set them ([`Player::set_volumes`]); the dialog's
//! press sample is the buttons' (`button.wav`, [`Player::click`]).
//!
//! **The host's sound** (E17 M7 T3c, *provisional*; design §13.6): the device and the music are
//! the host's ([`HostSound`]), lent to the battle it runs and taken back once the battle closed. The
//! music plays as the original's one sound manager plays it (`h3_ui::music`, the model of
//! `play_music` 0x59ACB0 and its thread, common with the map viewer): the combat music asked for
//! at the opening's end fades in after the host's track's fade-out, the end's tune follows the
//! combat music's fade-out (and the fade-in's end when the battle ends within it), and a track the
//! host asks for afterwards fades the tune out and comes back where its own fade began. The
//! battle's effects take streams of their own on the host's device for its run. The focus lost
//! holds every effect playing where it is and fades the music out; regained, they play on and the
//! music fades in (`sound_focus_lost` 0x599C40, `sound_focus_regained` 0x599B90; E24 design
//! §4.15). An adapter: it reads files and drives the audio device.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use h3_assets::snd::SndArchive;
use h3_assets::wav::{Pcm, decode_wav};
use h3_render::Bugs;
use h3_sdl::Audio;
use h3_ui::music::{FULL, Queue, Request, Run, Track};
use h3_view::anim::{Cue, Sound};
use h3_view::options::sound_volume;
use h3_view::sound::{
    EFFECT_VOLUME, FILE_VOLUME, Lengths, MUSIC_VOLUME, Opening, START_WAIT, Tune, length_ms,
};
use h3_view::tables::Sounds;

use crate::gate::find_ci;
use crate::sounds::SOUND_ARCHIVES;

/// The creature sounds' channels (category 3 of 0x684AB8: channels 6 to 13).
const CHANNELS: usize = 8;
/// How far ahead a looping sound and the music are queued, in ms.
const AHEAD_MS: u64 = 1_000;
/// The music stream's format: every track's (the battle's four and its end's tunes, as the map's
/// terrain tracks, are 44.1 kHz stereo).
pub const MUSIC_RATE: u32 = 44_100;
/// The music stream's channels.
pub const MUSIC_CHANNELS: u16 = 2;

/// SDL's gain for a volume on Miles' scale (0 to 127). The adapter's only floating point: SDL
/// takes the gain as a float.
fn gain(volume: u8) -> f32 {
    f32::from(volume) / f32::from(MUSIC_VOLUME)
}

/// A decoded MP3: its rate, channels and samples.
pub type Music = (u32, u16, Vec<i16>);

/// Whether the host's sound has a track already ([`HostSound::has_track`]; E17 M7 T4, R4-4).
pub type Kept<'a> = &'a dyn Fn(&str) -> bool;

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
    /// The combat music's name (the opening's).
    combat: String,
    /// The music's tracks by name, until the host's sound takes them ([`Player::open`]): the
    /// combat music and the end's tunes (E17 M3 T8), each the battle may end with (E17 M5 T1b: a
    /// battle fought as it is played does not know which yet); none, a track missing. Those the
    /// host's sound has already are not decoded again (E17 M7 T4, R4-4: `kept`).
    tracks: Vec<(String, Option<Music>)>,
    /// The music's part of the report, made as the tracks were decoded.
    music_note: String,
    /// The start sound's name.
    start: String,
    /// How long the decoding took, in ms.
    took: u128,
    /// The creatures' sounds, by stack and by created creature (E17 M5 T8b2, R6-1).
    voices: Voices,
    /// The battle's own sounds by cue, upper-cased: the tent's heal (E17 M3 T3b), a resisted
    /// spell's and the cast spells' (E17 M3 T7b-1).
    named: BTreeMap<Sound, String>,
    /// The buttons' press sample, upper-cased (E17 M6 T4a: the combat options dialog's).
    button: String,
}

impl Library {
    /// Decodes the start sound, every sound of `voices`, the battle's own sounds `named` by cue
    /// and the buttons' press sample `button` from the sound archives of `install`, and the music
    /// and the end's `tunes` from its `MP3` folder but the tracks the host's sound has already
    /// (`kept`, [`HostSound::has_track`]: an earlier battle's, E17 M7 T4, R4-4). A missing or
    /// undecodable sound costs that sound, noted in `lines`.
    pub fn load(
        install: &Path,
        (opening, button): (Opening, &str),
        (voices, named): (Voices, BTreeMap<Sound, String>),
        (tunes, kept): (&[Tune], Kept<'_>),
        lines: &mut Vec<String>,
    ) -> Result<Self, String> {
        let (result, took) = crate::gate::timed(|| {
            Self::decode(
                install,
                (opening, button),
                (voices, named),
                (tunes, kept),
                lines,
            )
        });
        let mut library = result?;
        library.took = took.as_millis();
        Ok(library)
    }

    /// [`Library::load`] without its timing.
    fn decode(
        install: &Path,
        (opening, button): (Opening, &str),
        (voices, named): (Voices, BTreeMap<Sound, String>),
        (tunes, kept): (&[Tune], Kept<'_>),
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
            .chain(std::iter::once(button.to_ascii_uppercase()))
            .collect();
        wanted.sort_unstable();
        wanted.dedup();
        let clips = clips(install, &wanted, lines)?;
        let combat = opening.music_file();
        let names = std::iter::once(combat.as_str()).chain(tunes.iter().map(|t| t.file()));
        let (mut tracks, mut left) = (Vec::new(), Vec::new());
        for name in names {
            if kept(name) {
                left.push(name.to_owned());
            } else {
                tracks.push((name.to_owned(), mp3(install, name, lines)?));
            }
        }
        let music_note = music_note(&combat, &tracks, &left);
        Ok(Self {
            clips,
            combat,
            tracks,
            music_note,
            start: start.to_ascii_uppercase(),
            took: 0,
            voices,
            named,
            button: button.to_ascii_uppercase(),
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
        combat: String::new(),
        tracks: Vec::new(),
        music_note: String::new(),
        start: String::new(),
        took: 0,
        voices,
        named,
        button: String::new(),
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
        format!(
            "sound: {} clips, {}, decoded in {} ms",
            self.clips.len(),
            self.music_note,
            self.took
        )
    }
}

/// The music's part of the report: the `combat` music's format and length, how many of the
/// end's tunes were decoded (`tracks`), and how many tracks the host's sound had already
/// (`kept`, E17 M7 T4).
fn music_note(combat: &str, tracks: &[(String, Option<Music>)], kept: &[String]) -> String {
    let decoded = (tracks.iter()).find_map(|(n, m)| m.as_ref().filter(|_| n == combat));
    let music = match decoded {
        _ if kept.iter().any(|n| n == combat) => "music kept".to_owned(),
        Some((rate, channels, samples)) => {
            let seconds = samples.len() / usize::from(*channels).max(1) / (*rate).max(1) as usize;
            format!("music {rate} Hz × {channels}, {seconds} s")
        }
        None => "no music".to_owned(),
    };
    let tune = match (tracks.iter())
        .filter(|(n, m)| n != combat && m.is_some())
        .count()
    {
        0 => String::new(),
        1 => ", the end's tune".to_owned(),
        n => format!(", the end's {n} tunes"),
    };
    let others = kept.iter().filter(|n| *n != combat).count();
    let more = if others > 0 {
        format!(", {others} kept")
    } else {
        String::new()
    };
    format!("{music}{tune}{more}")
}

/// One channel of the creature sounds: its stream, the format queued on it, what plays there.
#[derive(Debug)]
struct Channel {
    stream: usize,
    format: (u32, u16),
    /// The cue that plays here (its stack, its sound and its voice).
    owner: Option<Cue>,
    /// The volume of the sound it plays, before the effects' setting (E17 M6 T4a).
    volume: u8,
    /// Whether that sound loops.
    looping: bool,
}

/// The host's sound (E17 M7 T3c, *provisional*; design §13.6): the audio device and the music on
/// one of its streams, played as the original's music thread plays it (`h3_ui::music::Queue`
/// on the frames the device has played: the fades follow the device's time, paused with it). A
/// host opens it ([`HostSound::open`]), plays its own sounds on its device ([`HostSound::audio`]) and its
/// tracks through it ([`HostSound::add_track`], [`HostSound::request`]), and lends it to the battle it runs
/// ([`crate::window::Setup::sound`]), which hands it back once closed
/// ([`crate::window::Hosted::take_sound`]): one music for both, as the original's one sound
/// manager. Every track is 44.1 kHz stereo (the battle's and the map's are); one of another format
/// is kept as missing.
#[derive(Debug)]
pub struct HostSound {
    out: Audio,
    music_stream: usize,
    music: Queue,
    /// The original's filter of the music's requests or the client's (MCL-Q12).
    bugs: Bugs,
    /// The tracks added, by name: none for one missing or of another format. A name not added
    /// is being decoded (the music waits for it).
    tracks: BTreeMap<String, Option<Music>>,
}

impl HostSound {
    /// Opens the default playback device and the music's stream; the music's requests filtered as
    /// `bugs` says.
    ///
    /// # Errors
    /// SDL's error when the device or the stream cannot be opened (no sound device, say).
    pub fn open(bugs: Bugs) -> Result<Self, String> {
        Self::on(Audio::open()?, bugs)
    }

    /// The host's sound on SDL's `dummy` driver (E17 M7 T4: the tests' device, which plays
    /// nothing; [`Audio::open_driver`]).
    #[cfg(test)]
    pub(crate) fn dummy(bugs: Bugs) -> Result<Self, String> {
        Self::on(Audio::open_driver("dummy")?, bugs)
    }

    /// The music's stream added to the device `out`.
    fn on(mut out: Audio, bugs: Bugs) -> Result<Self, String> {
        let music_stream = out.add_stream(MUSIC_RATE, MUSIC_CHANNELS)?;
        Ok(Self {
            out,
            music_stream,
            music: Queue::new(MUSIC_RATE, bugs),
            bugs,
            tracks: BTreeMap::new(),
        })
    }

    /// The device, for the host's own streams (its effects).
    pub fn audio(&mut self) -> &mut Audio {
        &mut self.out
    }

    /// A track the music may play, decoded (none: missing); whether it was kept (one of another
    /// format than the stream's is kept as missing). A name added again replaces the first.
    pub fn add_track(&mut self, name: &str, music: Option<Music>) -> bool {
        let fits = |m: &Music| (m.0, m.1) == (MUSIC_RATE, MUSIC_CHANNELS);
        let kept = music.as_ref().is_none_or(fits);
        self.tracks
            .insert(name.to_owned(), music.filter(|m| fits(m)));
        kept
    }

    /// Whether `name` was added.
    #[must_use]
    pub fn has_track(&self, name: &str) -> bool {
        self.tracks.contains_key(name)
    }

    /// The frames the music's stream still holds.
    fn queued(&self) -> u64 {
        let frames = self.out.queued(self.music_stream) / usize::from(MUSIC_CHANNELS);
        u64::try_from(frames).unwrap_or(u64::MAX)
    }

    /// `play_music` 0x59ACB0 asked for `name`, looping or played `once` (the end's tune); when it
    /// changes the music to come, what is queued is taken back.
    pub fn request(&mut self, name: &str, once: bool) -> Request {
        let queued = self.queued();
        let Self { music, tracks, .. } = self;
        let state = |n: &str| track_state(tracks, n);
        let request = if once {
            music.request_once(name, queued, &state)
        } else {
            music.request(name, queued, &state)
        };
        if request.changed() {
            self.out.clear(self.music_stream);
        }
        request
    }

    /// `name` (looping) asked for through the client's filter whatever the profile's
    /// ([`Queue::request_again`]: E17 M7 T4, R4-2, the battle's music at a Restart after its end,
    /// a viewer's own request); when it changes the music to come, what is queued is taken back.
    pub(crate) fn request_again(&mut self, name: &str) -> Request {
        let queued = self.queued();
        let Self { music, tracks, .. } = self;
        let request = music.request_again(name, queued, &|n: &str| track_state(tracks, n));
        if request.changed() {
            self.out.clear(self.music_stream);
        }
        request
    }

    /// The game's focus lost or regained: the music faded out, or asked for again ([`Queue::focus`]).
    pub fn focus(&mut self, on: bool) {
        let queued = self.queued();
        let Self { music, tracks, .. } = self;
        if music.focus(on, queued, &|n: &str| track_state(tracks, n)) {
            self.out.clear(self.music_stream);
        }
    }

    /// The music afresh: nothing playing, no place kept (a host's restart).
    pub fn reset(&mut self) {
        self.out.clear(self.music_stream);
        self.music = Queue::new(MUSIC_RATE, self.bugs);
    }

    /// The track the music opened last, as heard.
    #[must_use]
    pub fn playing(&self) -> Option<&str> {
        self.music.heard().playing()
    }

    /// Keeps the music queued ahead at `volume` (0 to 127: the setting's whole volume as the
    /// stream's gain, the fades in the samples).
    pub fn service(&mut self, volume: u8) {
        self.out.set_gain(self.music_stream, gain(volume));
        let queued = self.queued();
        let want = u64::from(MUSIC_RATE) * AHEAD_MS / 1000;
        let Self {
            out,
            music_stream,
            music,
            tracks,
            ..
        } = self;
        for run in music.top_up(queued, want, &|n: &str| track_state(tracks, n)) {
            let samples = tracks
                .get(&run.name)
                .and_then(Option::as_ref)
                .map_or_else(Vec::new, |m| run_samples(m, &run));
            if out.put(*music_stream, &samples, false).is_err() {
                // The model and the stream apart: queued again from what was heard.
                out.clear(*music_stream);
                music.cleared();
                break;
            }
        }
    }

    /// The device, for the report.
    #[must_use]
    pub fn describe(&self) -> String {
        self.out.describe()
    }
}

/// What the music knows of a track: its frames once added (a name not added is being decoded).
fn track_state(tracks: &BTreeMap<String, Option<Music>>, name: &str) -> Track {
    match tracks.get(name) {
        Some(Some((_, channels, samples))) => {
            let frames = samples.len() / usize::from((*channels).max(1));
            Track::Frames(u64::try_from(frames).unwrap_or(u64::MAX))
        }
        Some(None) => Track::Missing,
        None => Track::Decoding,
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

/// The audio output of a battle, on its host's [`HostSound`] (E17 M7 T3c).
#[derive(Debug)]
pub struct Player {
    sound: HostSound,
    start_stream: usize,
    channels: Vec<Channel>,
    /// The next channel taken when none is free.
    turn: usize,
    /// The combat music's name.
    combat: String,
    /// Whether the battle holds the device paused.
    paused: bool,
    /// The streams held where they are while the game lacks the focus.
    held: Vec<usize>,
    /// Whether the music and the effects are heard (the keys M and E).
    pub music_on: bool,
    pub effects_on: bool,
    /// The combat options' music and effects volume settings, 0 to 9 (E17 M6 T4a).
    settings: (u8, u8),
}

impl Player {
    /// The battle's sound on the host's `sound`: the start sound's stream and the creature
    /// sounds' channels added to its device, the library's tracks (the combat music, the end's
    /// tunes) given to its music, one refused noted in `lines`; those it had already stay its own
    /// (E17 M7 T4, R4-4: the library left them). On an error the streams added are removed and
    /// the host's sound handed back with it.
    pub fn open(
        mut sound: HostSound,
        library: &mut Library,
        lines: &mut Vec<String>,
    ) -> Result<Self, Box<(String, HostSound)>> {
        let mut streams = Vec::with_capacity(CHANNELS + 1);
        for _ in 0..=CHANNELS {
            match sound.out.add_stream(22_050, 1) {
                Ok(stream) => streams.push(stream),
                Err(e) => {
                    for &stream in &streams {
                        sound.out.remove_stream(stream);
                    }
                    return Err(Box::new((e, sound)));
                }
            }
        }
        for (name, music) in library.tracks.drain(..) {
            if !sound.add_track(&name, music) {
                lines.push(format!(
                    "music: {name}: not {MUSIC_RATE} Hz × {MUSIC_CHANNELS}, the music's stream's"
                ));
            }
        }
        let start_stream = streams.first().copied().unwrap_or_default();
        let list = (streams.iter().skip(1))
            .map(|&stream| Channel {
                stream,
                format: (22_050, 1),
                owner: None,
                looping: false,
                volume: EFFECT_VOLUME,
            })
            .collect();
        let mut player = Self {
            sound,
            start_stream,
            channels: list,
            turn: 0,
            combat: library.combat.clone(),
            paused: false,
            held: Vec::new(),
            music_on: true,
            effects_on: true,
            settings: (0, 0),
        };
        let first = h3_view::options::Settings::FIRST_RUN;
        player.set_volumes(first.music, first.effects);
        Ok(player)
    }

    /// The host's sound handed back: every effect stopped and its streams removed, the device
    /// running (the music the host's again: the end's tune plays on until it asks for its own).
    pub fn close(mut self) -> HostSound {
        self.silence();
        let streams =
            std::iter::once(self.start_stream).chain(self.channels.iter().map(|c| c.stream));
        for stream in streams {
            self.sound.out.remove_stream(stream);
        }
        if self.paused {
            self.sound.out.pause(false);
        }
        self.sound
    }

    /// A stream emptied (it falls silent); one held for the focus's loss is let go, so what it
    /// plays next is heard (the original starts a sample anew).
    fn clear(&mut self, stream: usize) {
        self.sound.out.clear(stream);
        if let Some(i) = self.held.iter().position(|&s| s == stream) {
            self.held.swap_remove(i);
            self.sound.out.hold(stream, false);
        }
    }

    /// The combat options' volume settings `music` and `effects` (0 to 9; E17 M6 T4a), applied at
    /// once (`music_volume_apply` 0x59A1B0, `effects_volume_apply` 0x59A0C0): the start sound's
    /// stream and every creature channel at their sounds' volumes by the effects' setting, the
    /// music by its own from the next service.
    pub fn set_volumes(&mut self, music: u8, effects: u8) {
        self.settings = (music, effects);
        self.sound
            .out
            .set_gain(self.start_stream, gain(sound_volume(FILE_VOLUME, effects)));
        for c in &self.channels {
            self.sound
                .out
                .set_gain(c.stream, gain(sound_volume(c.volume, effects)));
        }
    }

    /// Plays the buttons' press sample once at `volume` (E17 M6 T4a; `button_press` 0x456860,
    /// *provisional*: its loader's volume, `FILE_VOLUME`, on the creature channels; the combat
    /// options' procedure's at 0x40 in category 3, 0x46FDB1), if it was found.
    pub fn click(&mut self, library: &Library, volume: u8) {
        if !self.effects_on {
            return;
        }
        let Some(clip) = library.clips.get(&library.button) else {
            return;
        };
        let index = self.free_channel();
        self.play_on(index, clip, volume, None);
    }

    /// A creature channel free (none looping, nothing queued), else the next one in turn.
    fn free_channel(&mut self) -> usize {
        let free = self
            .channels
            .iter()
            .position(|c| !c.looping && self.sound.out.queued(c.stream) == 0);
        free.unwrap_or_else(|| {
            let i = self.turn % CHANNELS;
            self.turn = i + 1;
            i
        })
    }

    /// `clip` played once on channel `index` at `volume` (through the effects' setting), owned by
    /// `owner` (a stack's cue; none: the dialog's press).
    fn play_on(&mut self, index: usize, clip: &Pcm, volume: u8, owner: Option<Cue>) {
        let effects = self.settings.1;
        let Some(stream) = self.channels.get(index).map(|c| c.stream) else {
            return;
        };
        self.clear(stream);
        let out = &mut self.sound.out;
        let Some(channel) = self.channels.get_mut(index) else {
            return;
        };
        channel.volume = volume;
        out.set_gain(channel.stream, gain(sound_volume(volume, effects)));
        if channel.format != (clip.rate, clip.channels) {
            out.set_format(channel.stream, clip.rate, clip.channels);
            channel.format = (clip.rate, clip.channels);
        }
        if out.put(channel.stream, &clip.samples, true).is_ok() {
            channel.owner = owner;
            channel.looping = false;
        }
    }

    /// Plays the start sound; how long the battle waits for it, in ms (at most [`START_WAIT`]).
    pub fn start(&mut self, library: &Library) -> u32 {
        let Some(clip) = library.clips.get(&library.start) else {
            return 0;
        };
        self.clear(self.start_stream);
        let out = &mut self.sound.out;
        out.set_format(self.start_stream, clip.rate, clip.channels);
        if out.put(self.start_stream, &clip.samples, true).is_err() {
            return 0;
        }
        let ms = clip.frames() as u64 * 1000 / u64::from(clip.rate.max(1));
        u32::try_from(ms).unwrap_or(u32::MAX).min(START_WAIT)
    }

    /// The combat music asked for (`play_music`, loop count 0, the creature sounds still
    /// playing stopped first: 0x59ADAC): it fades in after the host's track's fade-out.
    pub fn start_music(&mut self) {
        self.silence();
        let combat = self.combat.clone();
        let _ = self.sound.request(&combat, false);
    }

    /// The combat music asked for again after the end's tune was (the viewer's Restart, the
    /// battle shown again from earlier; *provisional*): through the client's filter in both
    /// profiles (E17 M7 T4, R4-2, the user: a Restart is the viewer's own, not the original's),
    /// so it always comes back: during its fade-out for the tune where its fade began, during its
    /// fade-in it plays on, after the tune opened it follows the tune's fade-out.
    pub fn restart_music(&mut self) {
        self.silence();
        let combat = self.combat.clone();
        let _ = self.sound.request_again(&combat);
    }

    /// The battle's end (E17 M3 T8, design §8.2): every effect ends, the start sound's stream
    /// too, and the end's `tune` is asked for to be played once (`play_music` with a loop count of
    /// 1, the effects ended): after the combat music's 900 ms fade-out (E17 M7 T3c), from its start
    /// at the music's volume; nothing if it was not decoded.
    pub fn end(&mut self, tune: Tune) {
        self.silence();
        self.clear(self.start_stream);
        let _ = self.sound.request(tune.file(), true);
    }

    /// Plays a cue: starts the stack's sound on a channel, or stops its looping move sound.
    pub fn cue(&mut self, library: &Library, cue: &Cue) {
        if cue.stop {
            let stops: Vec<usize> = (self.channels.iter())
                .filter(|c| stops_on(c, cue))
                .map(|c| c.stream)
                .collect();
            for stream in stops {
                self.clear(stream);
            }
            for c in &mut self.channels {
                if stops_on(c, cue) {
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
                | Sound::ManaDrain
                | Sound::DeathBlow
                | Sound::IceBolt
                | Sound::ChannelDrain
                | Sound::ChannelFill
                | Sound::SpellFile(_)
                | Sound::Drawbridge
                | Sound::Mine
                | Sound::TurretShot
        ) {
            // Category 2, the start sound's (`play_sound_file` 0x59A470; E17 M3 T3b, T7b-1,
            // T7b-2b-2b, T7b-2b-3, M4 T2c-1; a turret's shot, M4 T3a, R2-1; the mana drain, M6
            // T5d1; Death Blow, T5d2a; Ice Bolt's explosion and the Familiars' channel, T5d2b:
            // the channel's fill cuts its drain here when a fast speed overlaps them, the
            // category's other three channels not built).
            self.clear(self.start_stream);
            let out = &mut self.sound.out;
            out.set_format(self.start_stream, clip.rate, clip.channels);
            let _ = out.put(self.start_stream, &clip.samples, true);
            return;
        }
        let index = self.free_channel();
        let effects = self.settings.1;
        let Some(stream) = self.channels.get(index).map(|c| c.stream) else {
            return;
        };
        self.clear(stream);
        let out = &mut self.sound.out;
        let Some(channel) = self.channels.get_mut(index) else {
            return;
        };
        // A spell's sound, good luck's, an object's, a wall shot's landing and Eagle Eye's
        // pickup at their loader's volume, a creature's at the effects' (E17 M3 T7b-1,
        // T7b-2b-2b; T7c-2: `wav_load` 0x55C720, *provisional*; the landing since E17 M5 T5b2,
        // 0x44618B: volume 127; the pickup since E17 M6 T5c, as good luck's).
        let volume = match cue.sound {
            Sound::Spell(_)
            | Sound::Luck
            | Sound::Object(_)
            | Sound::WallHit
            | Sound::WallMiss
            | Sound::Pickup(_) => FILE_VOLUME,
            _ => EFFECT_VOLUME,
        };
        channel.volume = volume;
        out.set_gain(channel.stream, gain(sound_volume(volume, effects)));
        if channel.format != (clip.rate, clip.channels) {
            out.set_format(channel.stream, clip.rate, clip.channels);
            channel.format = (clip.rate, clip.channels);
        }
        // A one-shot is flushed so that it plays to its end; a loop is topped up instead.
        let looping = cue.sound == Sound::Move;
        if out.put(channel.stream, &clip.samples, !looping).is_ok() {
            channel.owner = Some(*cue);
            channel.looping = looping;
        }
    }

    /// Stops every creature sound (a seek, the music's start).
    pub fn silence(&mut self) {
        let streams: Vec<usize> = self.channels.iter().map(|c| c.stream).collect();
        for stream in streams {
            self.clear(stream);
        }
        for c in &mut self.channels {
            c.owner = None;
            c.looping = false;
        }
    }

    /// Holds or resumes every sound (the battle paused or running); the music's fades hold with
    /// it (they follow the frames the device played).
    pub fn pause(&mut self, on: bool) {
        self.sound.out.pause(on);
        self.paused = on;
    }

    /// The game's focus lost or regained (E17 M7 T3c; E24 design §4.15): lost, every effect
    /// playing held where it is and the music faded out (`sound_focus_lost` 0x599C40); regained,
    /// the effects held play on (0x599B90) and the music is asked for again (`music_resume`
    /// 0x59AC00).
    pub fn focus(&mut self, on: bool) {
        self.sound.focus(on);
        if on {
            for stream in self.held.drain(..) {
                self.sound.out.hold(stream, false);
            }
            return;
        }
        let streams =
            std::iter::once(self.start_stream).chain(self.channels.iter().map(|c| c.stream));
        for stream in streams {
            if !self.held.contains(&stream) && self.sound.out.queued(stream) > 0 {
                self.sound.out.hold(stream, true);
                self.held.push(stream);
            }
        }
    }

    /// Keeps the loops and the music queued ahead, the music at its setting's volume (0 with
    /// the music off).
    pub fn service(&mut self, library: &Library) {
        for c in &self.channels {
            let Some(owner) = c.owner.filter(|_| c.looping) else {
                continue;
            };
            let Some(clip) = library.clip(&owner) else {
                continue;
            };
            let ahead = ahead(clip.rate, clip.channels);
            if self.sound.out.queued(c.stream) < ahead {
                let _ = self.sound.out.put(c.stream, &clip.samples, false);
            }
        }
        let volume = if self.music_on {
            sound_volume(MUSIC_VOLUME, self.settings.0)
        } else {
            0
        };
        self.sound.service(volume);
    }

    /// The device, for the report.
    pub fn describe(&self) -> String {
        self.sound.describe()
    }
}

/// Whether `cue`, a stop, stops the sound channel `c` plays.
fn stops_on(c: &Channel, cue: &Cue) -> bool {
    c.looping
        && c.owner
            .is_some_and(|o| (o.stack, o.sound) == (cue.stack, cue.sound))
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

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// The tests' sound devices one at a time: each opens SDL's audio and closes it.
    static DEVICE: std::sync::Mutex<()> = std::sync::Mutex::new(());

    /// The host's sound on SDL's dummy driver (E17 M7 T4), the devices' lock held for the test
    /// (bind it first, so that it is let go after the sound closed); none, noted, where SDL
    /// cannot open it.
    pub(crate) fn dummy(bugs: Bugs) -> Option<(std::sync::MutexGuard<'static, ()>, HostSound)> {
        let lock = DEVICE
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        match HostSound::dummy(bugs) {
            Ok(sound) => Some((lock, sound)),
            Err(e) => {
                eprintln!("skipped: no dummy audio device ({e})");
                None
            }
        }
    }

    /// An installation copy of nothing (`name` its folder's): no sound archive, no music.
    pub(crate) fn bare_install(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("h3-view-{name}-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// `seconds` of a 44.1 kHz stereo track's silence.
    pub(crate) fn track(seconds: usize) -> Music {
        (MUSIC_RATE, MUSIC_CHANNELS, vec![0; 2 * 44_100 * seconds])
    }

    /// The battle's library of an installation copy of nothing, the host's sound having the
    /// tracks `kept` already.
    fn bare_library(install: &Path, kept: Kept<'_>) -> (Library, Vec<String>) {
        let mut lines = Vec::new();
        let opening = h3_view::sound::opening(1);
        let sounds = (Voices::default(), BTreeMap::new());
        let tunes = [Tune::Win, Tune::Lose];
        let music = (&tunes[..], kept);
        let library = Library::load(install, (opening, "BUTTON"), sounds, music, &mut lines);
        (library.unwrap(), lines)
    }

    #[test]
    fn the_tracks_the_host_s_sound_has_are_not_decoded_again() {
        // E17 M7 T4 (R4-4): a later battle on the host's sound reuses the tracks an earlier one
        // left it: they are not decoded again, and the host's stay its own.
        let install = bare_install("kept-tracks");
        let combat = h3_view::sound::opening(1).music_file();
        let kept = |name: &str| name == combat || name == Tune::Win.file();
        let (library, lines) = bare_library(&install, &kept);
        let names: Vec<&str> = library.tracks.iter().map(|(n, _)| n.as_str()).collect();
        assert_eq!(names, [Tune::Lose.file()]);
        let missing: Vec<&String> = lines.iter().filter(|l| l.starts_with("music: ")).collect();
        assert_eq!(missing.len(), 1, "{lines:?}");
        assert!(library.describe().contains("music kept, 1 kept"));
        // Given to the host's sound: its own decoded track stays.
        if let Some((_lock, mut sound)) = dummy(Bugs::Fixed) {
            sound.add_track(&combat, Some(track(1)));
            let (mut library, mut lines) = bare_library(&install, &|n| sound.has_track(n));
            let player = Player::open(sound, &mut library, &mut lines).unwrap();
            let state = track_state(&player.sound.tracks, &combat);
            assert_eq!(state, Track::Frames(44_100));
            assert_eq!(
                track_state(&player.sound.tracks, Tune::Win.file()),
                Track::Missing
            );
            drop(player.close());
        }
        fs::remove_dir_all(&install).unwrap();
    }

    #[test]
    fn a_restart_after_the_end_brings_the_combat_music_back_in_both_profiles() {
        // E17 M7 T4 (R4-2, the user): the end's tune asked for during the combat music's fade-in
        // (a battle ended within 900 ms of it), then a Restart: the combat music plays on and the
        // tune is dropped, under the original's filter too (a Restart is the viewer's own).
        let Some((_lock, sound)) = dummy(Bugs::Original) else {
            return;
        };
        let install = bare_install("restart-music");
        let (mut library, mut lines) = bare_library(&install, &|_| false);
        let mut player = Player::open(sound, &mut library, &mut lines).unwrap();
        let combat = library.combat.clone();
        player.sound.add_track(&combat, Some(track(5)));
        player.sound.add_track(Tune::Win.file(), Some(track(2)));
        player.start_music();
        player.service(&library);
        assert_eq!(player.sound.playing(), Some(combat.as_str()));
        player.end(Tune::Win);
        assert_eq!(player.sound.music.heard().pending(), Some(Tune::Win.file()));
        player.restart_music();
        assert_eq!(player.sound.music.heard().pending(), None);
        assert_eq!(player.sound.playing(), Some(combat.as_str()));
        drop(player.close());
        fs::remove_dir_all(&install).unwrap();
    }

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
    fn a_track_is_known_once_added() {
        let mut tracks = BTreeMap::new();
        tracks.insert("COMBAT01".to_owned(), Some((MUSIC_RATE, 2, vec![0; 10])));
        tracks.insert("win battle".to_owned(), None);
        assert_eq!(track_state(&tracks, "COMBAT01"), Track::Frames(5));
        assert_eq!(track_state(&tracks, "win battle"), Track::Missing);
        // A name not added is being decoded: the music waits for it.
        assert_eq!(track_state(&tracks, "GRASS"), Track::Decoding);
    }

    #[test]
    fn the_reports_music_part() {
        let second: Music = (MUSIC_RATE, 2, vec![0; 2 * 44_100 * 3]);
        let tracks = vec![
            ("COMBAT02".to_owned(), Some(second.clone())),
            ("win battle".to_owned(), Some(second.clone())),
            ("losecombat".to_owned(), None),
        ];
        assert_eq!(
            music_note("COMBAT02", &tracks, &[]),
            "music 44100 Hz × 2, 3 s, the end's tune"
        );
        let more = vec![
            ("COMBAT02".to_owned(), None),
            ("win battle".to_owned(), Some(second.clone())),
            ("losecombat".to_owned(), Some(second)),
        ];
        assert_eq!(
            music_note("COMBAT02", &more, &[]),
            "no music, the end's 2 tunes"
        );
        assert_eq!(music_note("COMBAT02", &[], &[]), "no music");
        // E17 M7 T4: the tracks the host's sound had already.
        let kept = ["COMBAT02".to_owned(), "surrender".to_owned()];
        assert_eq!(
            music_note("COMBAT02", &more[1..], &kept),
            "music kept, the end's 2 tunes, 1 kept"
        );
    }
}
