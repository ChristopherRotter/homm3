//! A battle ready to draw at any time (E17 M2 T4b-2a's `play`, shared with `window` since T5a),
//! played as it is drawn since E17 M5 T1b. What it is drawn from is read once ([`Ground`]): the
//! record (a record file's whole, or a scenario's opening: its battle is then fought as it is
//! played, [`crate::live`]), the asset tables and the layout, the archives, the stage's numbers
//! and the heroes. The battle ([`Battle`]) grows over it: the next command is played when a frame
//! at or past the timeline's end is asked for, and the record, the timeline at the original's
//! timing with the viewer's own fidgets ([`h3_view::anim::Growing`]), the panel and the mouse
//! grow with it; the checkpoint batch takes the assets as the frames name them. A seat played on a
//! server (E17 M5 T6b1, [`crate::online`]) grows by what the server sends instead: the window
//! takes it at each frame ([`Battle::poll`]), its timeline idling at its end meanwhile. An
//! adapter: it reads files and the archives.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

use h3_battle::BattleSetup;
use h3_content_battle::BattleRuleset;
use h3_formats::lod::LodArchive;
use h3_protocol::wire::{Command, Observation, Seat, ServerMessage, Side};
use h3_render::Bugs;
use h3_render::screen::Screen;
use h3_ui::sprite::Sprite;
use h3_view::anim::Timeline;
use h3_view::anim::{
    self, Casts, Growing, Look, MagicLook, Playhead, SiegeStage, Sound, Speed, Stage,
};
use h3_view::auto::Auto;
use h3_view::batch::{self, Batch};
use h3_view::battle_log::Logged;
use h3_view::compose::compose;
use h3_view::hero::Heroes;
use h3_view::layout::{self, BattleLayout};
use h3_view::message::{Picture, Style};
use h3_view::mouse::aim::Target;
use h3_view::mouse::track::{Input, Lens, Shown};
use h3_view::mouse::{self, Field};
use h3_view::record::{self, Record};
use h3_view::scene::Panels;
use h3_view::scene::{self, Draw, Moment};
use h3_view::sound::{Lengths, Tune};
use h3_view::tables::{Sounds, Tables};

use crate::audio::Voices;
use crate::boxes::{Answered, Asked, Leave};
use crate::live::Live;
use crate::online::{Online, Server};
use crate::render::{lookup, read_archives};
use crate::{assets, motion};

/// How long the battle's loading waits for a seat's record on a server (*provisional*).
const OPENING_WAIT: Duration = Duration::from_secs(30);

/// Where a battle comes from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Source {
    /// A scenario (fought by the bot as it is played) or a record file (`.jsonl`).
    pub input: PathBuf,
    /// The scenario's seed (obstacles, damage rolls) and the fidgets' seed.
    pub seed: u64,
    /// The combat speed the commands play at.
    pub speed: Speed,
    /// The packs a scenario names.
    pub packs: PathBuf,
    /// The installation copy whose archives are read.
    pub install: PathBuf,
    /// The sides a human plays here, a scenario's (E17 M5 T2b): their players are the battle's
    /// human ones, the others the computer's (the bot's); none, the scenario's own and the bot
    /// plays both.
    pub humans: [bool; 2],
    /// The seat played on a server instead (E17 M5 T6b1): the input is none then.
    pub server: Option<Server>,
}

impl Source {
    /// Whether the input is a scenario (anything but a `.jsonl` record; never a server's seat).
    pub fn scenario(&self) -> bool {
        self.server.is_none() && self.input.extension().is_none_or(|e| e != "jsonl")
    }

    /// The input's file stem (`battle` if none); a server's seat by its address and seat.
    pub fn stem(&self) -> String {
        if let Some(server) = &self.server {
            return format!("{} {:?}", server.address, server.seat).to_lowercase();
        }
        self.input
            .file_stem()
            .map_or_else(|| "battle".into(), |s| s.to_string_lossy().into_owned())
    }
}

/// A battle fought as it is played (E17 M5 T1b): here, or by a seat on a server (E17 M5 T6b1).
#[derive(Debug)]
enum Fought {
    /// A scenario's battle: its ruleset, setup and seed.
    Here(Box<(BattleRuleset, BattleSetup, u64)>),
    /// A seat's on a server.
    Server(Box<Served>),
}

/// A seat's battle on a server (E17 M5 T6b1): its connection, the content check and the ruleset
/// compiled here from the game's packs (none: absent).
#[derive(Debug)]
struct Served {
    online: RefCell<Online>,
    content: h3_view::remote::Content,
    ruleset: Option<BattleRuleset>,
}

/// Where a press gives a command (E17 M5 T2b).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Press {
    /// A left press at a point of the battlefield.
    At([i32; 2]),
    /// A click of the panel's button.
    Button(u16),
}

/// What a battle is drawn from, read once (E17 M5 T1b): the battle grows over it.
#[derive(Debug)]
pub struct Ground {
    /// The record as read: a record file's whole; a scenario's opening, its battle fought as it
    /// is played.
    record: Record,
    /// The asset tables.
    pub tables: Tables,
    layout: BattleLayout,
    /// The archives' bytes (CLT-Q2: whole archives): the batch takes the assets the frames name.
    archives: Vec<(&'static str, Vec<u8>)>,
    /// The video archives' directories (E17 M5 T4b2b): the result window's clips.
    videos: crate::videos::Videos,
    /// The executable's own strings (a panel made without the text tables).
    exe: Vec<u8>,
    /// The stage's numbers: each creature's, the magic animations', the casts', a siege's.
    looks: BTreeMap<u16, Look>,
    magic: BTreeMap<u8, MagicLook>,
    casts: Casts,
    siege: Option<SiegeStage>,
    /// The heroes beside the battlefield.
    heroes: Heroes,
    /// What words the log's lines and the mouse's hints (none without the text tables).
    writer: Option<crate::log::Writer>,
    /// The viewer's own fidgets' seed and the combat speed.
    seed: u32,
    speed: Speed,
    /// A battle fought as it is played (none: a record file's).
    fight: Option<Fought>,
    /// The sides a human plays here ([`Source::humans`]).
    humans: [bool; 2],
    /// The samples' lengths once the window plays the battle's sounds (E17 M5 T5b2,
    /// [`Ground::play_sounds`]): the battle waits for them; unset, no sound and no waits.
    lengths: std::cell::OnceCell<Lengths>,
}

/// The opening of the battle `online`'s seat plays (E17 M5 T6b1): its record's first snapshot,
/// with the seat's side the one a human plays here (`remote_turn`: the other seat's player is the
/// network's).
fn seat_opening(online: &Online) -> Result<Record, String> {
    let mut record = online.record().cloned().ok_or("no record")?;
    record.messages.truncate(1);
    let index = usize::from(online.seat() == Seat::Defender);
    if let Some(ServerMessage::Snapshot { observation, .. }) = record.messages.first_mut() {
        for (i, side) in observation.sides.iter_mut().enumerate() {
            side.human = i == index;
        }
    }
    Ok(record)
}

impl Ground {
    /// The sounds of each stack living at the start, by its id, and of each creature the battle
    /// may create ([`stack_sounds`]; E17 M5 T8b2, R6-1: every creature with a look, as a battle
    /// fought as it is played may create any, [`motion::add_looks`]).
    pub fn stack_sounds(&self) -> Voices {
        let mut voices = stack_sounds(&self.record, &self.tables);
        let more = creature_sounds(&self.record, &self.tables, self.looks.keys().copied());
        voices.creatures.extend(more);
        voices
    }

    /// The battle's own sounds a cue may name, by cue ([`battle_sounds`]).
    pub fn battle_sounds(&self) -> BTreeMap<Sound, String> {
        battle_sounds(&self.record, &self.tables)
    }

    /// The ruleset of a battle fought here, or the one compiled here from a server's game's packs
    /// (E17 M5 T6b1: the spell book and the result window draw from it); none for a record file's
    /// or a server's game whose packs are absent here.
    fn rules(&self) -> Option<&BattleRuleset> {
        match self.fight.as_ref()? {
            Fought::Here(here) => Some(&here.0),
            Fought::Server(served) => served.ruleset.as_ref(),
        }
    }

    /// The ruleset of the window's own previews (E17 M5 T6b1: an aim's area): a battle fought
    /// here's, or a server's game's when the content check finds it the same.
    fn previews(&self) -> Option<&BattleRuleset> {
        match self.fight.as_ref()? {
            Fought::Here(here) => Some(&here.0),
            Fought::Server(served) => served
                .ruleset
                .as_ref()
                .filter(|_| served.content == h3_view::remote::Content::Same),
        }
    }

    /// The content check of a server's game (E17 M5 T6b1; none: a battle here or a record).
    pub fn content(&self) -> Option<h3_view::remote::Content> {
        match self.fight.as_ref()? {
            Fought::Server(served) => Some(served.content),
            Fought::Here(..) => None,
        }
    }

    /// The seat's connection on a server (E17 M5 T6b1; none: a battle here or a record).
    pub fn online(&self) -> Option<&RefCell<Online>> {
        match self.fight.as_ref()? {
            Fought::Server(served) => Some(&served.online),
            Fought::Here(..) => None,
        }
    }

    /// The battle plays its sounds, whose samples last `lengths` (E17 M5 T5b2): a battle made
    /// from here on waits for them (`wait_for_sample`); set once, by the window when its sound
    /// opened.
    pub fn play_sounds(&self, lengths: Lengths) {
        let _ = self.lengths.set(lengths);
    }

    /// Reads what the battle of `source` is drawn from: a record file, or a scenario's opening
    /// (its battle is fought as it is played: every magic animation and spell read for it, as its
    /// events are not known yet).
    pub fn load(source: &Source) -> Result<Self, String> {
        let mut humans = source.humans;
        if humans.contains(&true) && !source.scenario() {
            return Err("a human plays a scenario's battle, not a record's".into());
        }
        let (record, fight) = if let Some(server) = &source.server {
            // A seat on a server (E17 M5 T6b1): its record's opening, its player the one human
            // here (`remote_turn`: the other seat's player is the network's).
            let mut online = Online::open(
                server.clone(),
                &source.packs,
                crate::net::os_roots,
                OPENING_WAIT,
            )?;
            let record = seat_opening(&online)?;
            let index = usize::from(server.seat == Seat::Defender);
            humans = [index == 0, index == 1];
            let (content, ruleset) = (online.content(), online.take_rules());
            // The drawing needs the game's packs (the stage's spells and creatures): absent here,
            // the window says so and stops (*provisional*).
            if content == h3_view::remote::Content::Absent {
                let packs: Vec<&str> = (record.header.manifest.packs.iter())
                    .map(|p| p.id.as_str())
                    .collect();
                return Err(format!(
                    "the game's packs ({}) do not compile from {}: the window draws a server's game \
                     from them",
                    packs.join(", "),
                    source.packs.display()
                ));
            }
            let fought = Fought::Server(Box::new(Served {
                online: RefCell::new(online),
                content,
                ruleset,
            }));
            (record, Some(fought))
        } else if source.scenario() {
            let (ruleset, mut setup) =
                crate::scenario::battle(&source.input, &source.packs, source.seed)?;
            // The sides a human plays here are the battle's human players, the bot's the
            // computer's (E17 M5 T2b, *provisional*).
            if humans.contains(&true) {
                setup.human = humans;
            }
            let opening = Live::new(&ruleset, setup.clone(), source.seed, humans)?.record()?;
            let record = record::parse(&opening).map_err(|e| e.to_string())?;
            (
                record,
                Some(Fought::Here(Box::new((ruleset, setup, source.seed)))),
            )
        } else {
            let text = fs::read_to_string(&source.input)
                .map_err(|e| format!("{}: {e}", source.input.display()))?;
            (record::parse(&text).map_err(|e| e.to_string())?, None)
        };
        record
            .first_snapshot()
            .ok_or("the record holds no snapshot")?;
        let top = record
            .header
            .manifest
            .packs
            .last()
            .map_or("", |p| p.id.as_str());
        let tables: Tables = assets::load(&assets::default_dir(), top)?;
        let layout_path = assets::default_dir().join("../layouts/battle.toml");
        let layout = layout::parse_battle(
            &fs::read_to_string(&layout_path)
                .map_err(|e| format!("{}: {e}", layout_path.display()))?,
        )
        .map_err(|e| format!("{}: {e}", layout_path.display()))?;
        let archives = read_archives(&source.install)?;
        let videos = crate::videos::Videos::open(&source.install);
        let parsed = parse(&archives)?;
        // The executable's own strings: read only when a line needs them.
        let exe = crate::text::executable(&source.install).unwrap_or_default();
        let every = fight.is_some();
        let looks = motion::looks(&record, &tables, &parsed)?;
        let magic = motion::magic(&record, &tables, &parsed, every)?;
        let siege = motion::siege(&record, &tables, &parsed)?;
        let casts = motion::casts(&record, &tables, &parsed, &source.packs, every)?;
        // A battle fought as it is played may create any creature a spell or a raiser makes.
        let mut looks = looks;
        let made: Vec<u16> = casts.bodies.keys().copied().collect();
        motion::add_looks(&record, &tables, &parsed, &made, &mut looks)?;
        // The viewer's own fidgets and the heroes' reactions, from the battle's seed.
        let seed = u32::try_from(source.seed & u64::from(u32::MAX)).unwrap_or(1);
        let heroes = motion::heroes(&record, &tables, &parsed, Some(seed))?;
        let writer = crate::log::Writer::new(&tables, &layout, (&parsed, &exe), Bugs::Fixed).ok();
        drop(parsed);
        Ok(Self {
            record,
            tables,
            layout,
            archives,
            videos,
            exe,
            looks,
            magic,
            casts,
            siege,
            heroes,
            writer,
            seed,
            speed: source.speed,
            fight,
            humans,
            lengths: std::cell::OnceCell::new(),
        })
    }
}

/// The sounds of each stack living at the start, by its id (E17 M2 T5b); a siege's turrets
/// (off the field) the town's shooter's, as their look (E17 M4 T3a, R2-1). And those of each
/// creature the record's events create, by its handle (E17 M5 T8b2, R6-1: a summoned stack's,
/// a clone's, a Pit Lord's Demons'; a created stack's cues name its creature).
pub fn stack_sounds(record: &Record, tables: &Tables) -> Voices {
    let made = record.messages.iter().flat_map(|m| match m {
        ServerMessage::Events { events, .. } => events.as_slice(),
        _ => &[],
    });
    let made = made.filter_map(|e| match *e {
        h3_protocol::wire::Event::StackCreated { creature, .. }
        | h3_protocol::wire::Event::RoseFromCorpse { creature, .. } => Some(creature),
        _ => None,
    });
    Voices {
        stacks: first_sounds(record, tables),
        creatures: creature_sounds(record, tables, made),
    }
}

/// The sounds of the creatures `made`, by handle (E17 M5 T8b2, R6-1).
fn creature_sounds(
    record: &Record,
    tables: &Tables,
    made: impl Iterator<Item = u16>,
) -> BTreeMap<u16, Sounds> {
    let keys = &record.header.manifest.keys.creatures;
    made.filter_map(|creature| {
        let entry = tables.creatures.get(keys.get(usize::from(creature))?)?;
        Some((creature, entry.sounds.clone()))
    })
    .collect()
}

/// The sounds of each stack of the record's first snapshot, by its id ([`stack_sounds`]).
fn first_sounds(record: &Record, tables: &Tables) -> BTreeMap<u8, Sounds> {
    let keys = &record.header.manifest.keys.creatures;
    let field = record.header.manifest.field;
    let cells = u16::from(field.width) * u16::from(field.height);
    let shooter = motion::town(record, tables)
        .and_then(|town| motion::shooter(town, tables))
        .ok();
    record
        .first_snapshot()
        .map(|s| s.stacks.as_slice())
        .unwrap_or_default()
        .iter()
        .filter_map(|s| {
            let creature = if u16::from(s.hex) < cells {
                let key = keys.get(usize::from(s.creature))?;
                tables.creatures.get(key)?
            } else {
                shooter?
            };
            Some((s.id, creature.sounds.clone()))
        })
        .collect()
}

/// The battle's own sounds a cue may name, by cue (not a creature's; design §8.2): the tent's
/// heal (E17 M3 T3b), a resisted spell's and each spell's (E17 M3 T7b-1), morale's, Fear's
/// and good luck's (E17 M3 T7b-2b-2b), Life Drain's, the Thunderbird's bolt's and the spells'
/// the striker's effects play (E17 M3 T7b-2b-3), the sound placed with each object (E17 M3
/// T7c-2), a siege's drawbridge and a mine's blast (E17 M4 T2c-1); while the battle is still
/// fought, every one it may play (E17 M5 T1b: its cues are not known yet; T2b: waiting for a
/// human too): here every one it may play, the battle's own list keeping those its cues name
/// once it is over ([`Battle::battle_sounds`]).
pub fn battle_sounds(record: &Record, tables: &Tables) -> BTreeMap<Sound, String> {
    let spells = &record.header.manifest.keys.spells;
    let battle = &tables.battle;
    let spell = |handle: u16, object: bool| {
        let entry = tables.spells.get(spells.get(usize::from(handle))?)?;
        if object {
            entry.object_sound.clone()
        } else {
            entry.sound.clone()
        }
    };
    let mut sounds: Vec<(Sound, Option<String>)> = vec![
        (Sound::Heal, Some(battle.heal_sound.clone())),
        (Sound::Resisted, battle.resist_sound.clone()),
        (Sound::GoodMorale, battle.good_morale_sound.clone()),
        (Sound::BadMorale, battle.bad_morale_sound.clone()),
        (Sound::Fear, battle.fear_sound.clone()),
        (Sound::Luck, battle.luck_sound.clone()),
        (Sound::Drain, battle.drain_sound.clone()),
        (Sound::Thunderbolt, battle.thunderbolt_sound.clone()),
        // A siege's drawbridge and a mine's blast (E17 M4 T2c-1).
        (
            Sound::Drawbridge,
            battle.siege_sounds.as_ref().map(|s| s[0].clone()),
        ),
        (
            Sound::Mine,
            battle.siege_sounds.as_ref().map(|s| s[1].clone()),
        ),
        // A wall shot's landing (E17 M5 T5b2: its wait; the window played none before).
        (
            Sound::WallHit,
            battle.wall_sounds.as_ref().map(|s| s[0].clone()),
        ),
        (
            Sound::WallMiss,
            battle.wall_sounds.as_ref().map(|s| s[1].clone()),
        ),
    ];
    for handle in (0..spells.len()).filter_map(|h| u16::try_from(h).ok()) {
        sounds.push((Sound::Spell(handle), spell(handle, false)));
        sounds.push((Sound::SpellFile(handle), spell(handle, false)));
        sounds.push((Sound::Object(handle), spell(handle, true)));
    }
    sounds
        .into_iter()
        .filter_map(|(cue, name)| Some((cue, name?)))
        .collect()
}

/// The battle's timeline as it is played: growing, waiting at its end for a human's command
/// (E17 M5 T2b: the timeline shown then, [`Growing::waiting`]), or finished once the battle is
/// over or the record ends.
#[derive(Debug)]
enum Played<'g> {
    Growing(Box<Growing<'g>>),
    Waiting(Box<Growing<'g>>, Box<Timeline>),
    Finished(Box<Timeline>),
}

impl Played<'_> {
    fn timeline(&self) -> &Timeline {
        match self {
            Self::Growing(g) => g.timeline(),
            Self::Waiting(_, t) | Self::Finished(t) => t,
        }
    }

    /// Before when the timeline's moments are final (E17 M5 T8a, R3-1): a battle still fought
    /// adds from its last command's end on ([`Growing::push`]; a waiting one's tail shows the
    /// first idle step after it), a finished one nothing.
    fn settled(&self) -> u32 {
        match self {
            Self::Growing(g) | Self::Waiting(g, _) => g.timeline().end,
            Self::Finished(_) => u32::MAX,
        }
    }
}

/// The battle, played as it is drawn (the module's text).
#[derive(Debug)]
pub struct Battle<'g> {
    ground: &'g Ground,
    /// The record so far: the opening, then what each command played sent.
    record: Record,
    /// The seat's record's timelines when it opened (E17 M5 T6b2: [`Battle::opened_afresh`]).
    timelines: u32,
    /// The timeline at the original's timing.
    played: Played<'g>,
    /// The commands played.
    commands: usize,
    /// The battle being fought (none: a record file's).
    fight: Option<Fight<'g>>,
    /// The index in the record of the server's `YourTurn` whose turn the mouse and the book have
    /// (E17 M5 T6b1: [`Battle::turned`]'s last; none while the seat has no turn to play).
    served: Option<usize>,
    /// How many of the record's messages were events when [`Battle::turned`]'s last turn was
    /// served: a turn served again with none since (a re-join's `YourTurn`) replaces it.
    served_events: Option<usize>,
    /// The timeline with the stacks the mouse entered fidgeting (E17 M3 T6c-2; none: as played).
    entered: Option<Timeline>,
    /// The timeline shown read forward as the frames are asked for (E17 M5 T1a): a frame costs
    /// what changed since the one before it, not the battle's length.
    head: Playhead,
    /// The panel over the battle: its turns and its log (E17 M3 T6b).
    pub panels: Panels,
    /// The battle and its legal commands at each turn (a fought scenario's; none for a record).
    turned: Vec<crate::scenario::Turned>,
    /// The assets the frames drew so far.
    batch: Batch,
    /// The mouse over the battle: its moves and what they showed.
    pub shown: Shown,
    /// The mouse followed on as the battle grows (E17 M5 T8a, R3-1): its mark's lists are the
    /// start of `shown`'s.
    follower: crate::pointer::Follower<'g>,
    inputs: Vec<Input>,
    /// The log scrolled (each at a time, by entries).
    scrolls: Vec<(u32, i64)>,
    /// The panel's button pressed (the mouse's or a hotkey's).
    pub pressed: Option<u16>,
    /// The hero's spell book open over the battle, when it opened and its hero's player's colour
    /// (E17 M5 T3a): the battle behind it stands as it was then (the dialog's modal loop runs no
    /// idle step).
    book: Option<(h3_view::book::Open, u32, u8)>,
    /// What the book remembers between openings (the original's globals: one per game).
    book_memory: h3_view::book::Memory,
    /// The spell aimed (E17 M5 T3b): its handle and whether a creature casts it (the Faerie
    /// Dragon's key F), from the book's or the key's choice to the click's cast or a cancel.
    aiming: Option<(u16, bool)>,
    /// A message box open over the battle (E17 M5 T4a; design §7.19): the box, when it opened
    /// and what its OK does. The battle (and the book, when the box is the book's) stands still
    /// behind it (its modal loop runs no idle step).
    message: Option<(h3_view::message::Open, u32, Asked)>,
    /// The result window was closed (E17 M5 T4b1): the battle's screen is done.
    result_closed: bool,
    /// The battle behind the result window while it is open (E17 M5 T8a, R3-2).
    behind: h3_view::result::Behind,
    /// The result window's clips (E17 M5 T8b1, R2b-3 and R2b-5): decoded into the batch as soon
    /// as the battle's end, and so its tune, is known.
    clips: Clips,
    /// How many of the timeline's notices opened (E17 M5 T5a).
    notices: usize,
    /// The tactics phase's message the battle opened with is still open (E17 M5 T4c2a, design
    /// §7.23): the battle behind it stands before its first turn.
    opening: bool,
}

/// Where the result window's clips stand (E17 M5 T8b1, R2b-3 and R2b-5).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Clips {
    /// Not loaded: the battle's end is not known yet, or nothing here opens its window (a record
    /// file's battle).
    Unknown,
    /// In the batch, or not in the video archives (the slot then stays bare).
    Loaded,
    /// One of them could not be read or decoded (`h3-bink` refused it): the slot stays bare, as
    /// without the archives, instead of the window failing.
    Refused,
}

/// The battle being fought: here, its copy and session in-process (E17 M5 T1b), or by a seat on a
/// server (E17 M5 T6b1).
#[derive(Debug)]
enum Fight<'g> {
    Here(Box<Live<'g>>),
    Server(&'g RefCell<Online>),
}

impl<'g> Fight<'g> {
    /// The battle fought here.
    fn here(fight: Option<&mut Self>) -> Option<&mut Live<'g>> {
        match fight? {
            Fight::Here(live) => Some(live),
            Fight::Server(_) => None,
        }
    }
}

/// What a message box did with an input (E17 M5 T4a).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageDid {
    /// Nothing the screen shows.
    Nothing,
    /// The box changed, closed or gave way to another: draw again.
    Changed,
    /// The box's OK gave a command: Retreat or Surrender.
    Give(Command),
    /// The result window's OK closed the battle's screen (E17 M5 T4b1).
    Ended,
}

/// The original's spells aimed at a second target too (Sacrifice, Teleport): their aims are not
/// built (E17 M5 T3b).
const SECOND_TARGET: [u16; 2] = [40, 63];

/// A cursor's look (E17 M5 T3a): its DEF, its frame and the frame's hot spot (the combat cursor
/// over the battle, the default one the spell book sets).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct CursorLook {
    /// The DEF.
    pub def: String,
    /// The frame of group 0.
    pub frame: u32,
    /// The hot spot in the frame's canvas.
    pub hot: [i32; 2],
}

/// What the spell book did with an input (E17 M5 T3a).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BookDid {
    /// Nothing the screen shows.
    Nothing,
    /// The book or its status line or cursor changed: draw again.
    Changed,
    /// The book closed on a spell without a target: the cast to give.
    Cast(Command),
    /// The book closed on a spell that takes a target: its aim starts (E17 M5 T3b).
    Aim(u16),
}

impl<'g> Battle<'g> {
    /// The battle over `ground`: a record file's played whole, a scenario's before its first
    /// command, a seat's on a server from its record's opening.
    pub fn new(ground: &'g Ground) -> Result<Self, String> {
        Self::open(ground, ground.record.clone())
    }

    /// The battle a seat on a server plays once its record opened afresh (E17 M5 T6b2: a
    /// restarted server's restored game): a new timeline from the record's new opening, the one
    /// before dropped.
    ///
    /// # Errors
    /// No seat on a server, or as [`Battle::new`].
    pub fn afresh(&self) -> Result<Self, String> {
        let online = self.ground.online().ok_or("not a seat on a server")?;
        let opening = seat_opening(&online.borrow())?;
        Self::open(self.ground, opening)
    }

    /// Whether the seat's record opened afresh since this battle opened (E17 M5 T6b2): the window
    /// then plays [`Battle::afresh`].
    pub fn opened_afresh(&self) -> bool {
        self.ground
            .online()
            .is_some_and(|o| o.borrow().timelines() != self.timelines)
    }

    /// The battle over `ground` from `record`, its opening (the ground's own, or a seat's record
    /// opened afresh).
    fn open(ground: &'g Ground, record: Record) -> Result<Self, String> {
        let snapshot = record
            .first_snapshot()
            .ok_or("the record holds no snapshot")?
            .clone();
        let snapshot = &snapshot;
        let fight = match &ground.fight {
            Some(Fought::Here(here)) => {
                let (ruleset, setup, seed) = &**here;
                Some(Fight::Here(Box::new(Live::new(
                    ruleset,
                    setup.clone(),
                    *seed,
                    ground.humans,
                )?)))
            }
            Some(Fought::Server(served)) => Some(Fight::Server(&served.online)),
            None => None,
        };
        // A battle a human plays here grows from every side's events: the screen shows the side
        // to act's hidden objects (E17 M5 T2b), the snapshot's among them.
        let mut opening = snapshot.clone();
        if let Some(Fight::Here(fight)) = fight.as_ref().filter(|f| match f {
            Fight::Here(live) => live.human(),
            Fight::Server(_) => false,
        }) {
            opening.spell_obstacles = fight.obstacles();
        }
        let stage = Stage {
            lengths: ground.lengths.get(),
            ..motion::stage(
                &record,
                &ground.tables,
                &ground.layout,
                (&ground.magic, &ground.casts, ground.siege.as_ref()),
            )
        };
        let mut growing = Growing::new(
            &opening,
            &ground.looks,
            stage,
            ground.speed,
            Some(ground.seed),
        )
        .map_err(|e| e.to_string())?
        .with_heroes(ground.heroes.clone());
        let mut turned = Vec::new();
        let played = match &fight {
            Some(Fight::Here(fight)) => {
                turned.extend(fight.turn());
                if fight.over() {
                    Played::Finished(Box::new(growing.finish()))
                } else if fight.waiting() {
                    let tail = Box::new(growing.waiting());
                    Played::Waiting(Box::new(growing), tail)
                } else {
                    Played::Growing(Box::new(growing))
                }
            }
            // A seat on a server idles at its end until the server's next message (E17 M5 T6b1).
            Some(Fight::Server(_)) => {
                let tail = Box::new(growing.waiting());
                Played::Waiting(Box::new(growing), tail)
            }
            None => {
                // A record's commands, each the computer's when it has no time.
                let commands = anim::commands(&record.messages);
                for (i, command) in commands.iter().enumerate() {
                    let computer = commands.get(i + 1).is_some_and(|c| c.at.is_none());
                    growing.push(command, computer);
                }
                Played::Finished(Box::new(growing.finish()))
            }
        };
        let commands = match &played {
            Played::Growing(_) | Played::Waiting(..) => 0,
            Played::Finished(_) => anim::commands(&record.messages).len(),
        };
        let head = Playhead::new(played.timeline());
        let timelines = ground.online().map_or(0, |o| o.borrow().timelines());
        let mut battle = Self {
            ground,
            record,
            timelines,
            played,
            commands,
            fight,
            served: None,
            served_events: None,
            entered: None,
            head,
            panels: Panels::new(snapshot, &[], Vec::new()),
            turned,
            batch: Batch::default(),
            shown: Shown::default(),
            follower: crate::pointer::Follower::default(),
            inputs: Vec::new(),
            scrolls: Vec::new(),
            pressed: None,
            book: None,
            book_memory: h3_view::book::Memory::default(),
            aiming: None,
            message: None,
            result_closed: false,
            behind: h3_view::result::Behind::default(),
            clips: Clips::Unknown,
            notices: 0,
            opening: false,
        };
        // A seat on a server takes what its record holds already (E17 M5 T6b1).
        battle.take_served(None)?;
        battle.serve_turn();
        battle.follow()?;
        // The tactics phase's message (E17 M5 T4c2a; `battle_start` 0x4630EE to 0x4631E5): when
        // a human plays the phase's side here, before its first turn.
        let told = snapshot
            .tactics
            .as_ref()
            .is_some_and(|t| ground.humans[usize::from(t.side == Side::Defender)]);
        if let Some(writer) = ground.writer.as_ref().filter(|_| told) {
            let text = writer
                .catalogue
                .text("battle.tactics.phase", &h3_ui::text::Args::new());
            battle.opening = battle.tell(0, &text, (Style::Ok, None), Asked::Told);
        }
        Ok(battle)
    }

    /// The timeline shown: as played, with the stacks the mouse entered.
    pub fn timeline(&self) -> &Timeline {
        self.entered
            .as_ref()
            .unwrap_or_else(|| self.played.timeline())
    }

    /// Whether the bot's commands are still to come (a battle being fought, not over yet, not
    /// waiting for a human's command).
    pub fn playing(&self) -> bool {
        matches!(self.played, Played::Growing(_))
    }

    /// Whether the battle waits at its end for a human's command (E17 M5 T2b): its timeline
    /// idles on, the press giving the command ([`Battle::command`]).
    pub fn waiting(&self) -> bool {
        matches!(self.played, Played::Waiting(..))
    }

    /// The commands played.
    pub fn commands(&self) -> usize {
        self.commands
    }

    /// The fought battle's record so far (none: a record file's).
    pub fn record_text(&mut self) -> Option<Result<String, String>> {
        match self.fight.as_mut()? {
            Fight::Here(live) => Some(live.record()),
            Fight::Server(online) => {
                let online = online.borrow();
                Some(
                    online
                        .record()
                        .map(crate::connect::text)
                        .ok_or_else(|| "no record".into()),
                )
            }
        }
    }

    /// Plays the battle on while its timeline ends by `t` (a frame at `t` needs what comes then);
    /// whether a command was played.
    pub fn grow(&mut self, t: u32) -> Result<bool, String> {
        let mut grew = false;
        while self.playing() && self.played.timeline().end <= t {
            self.play_next()?;
            grew = true;
        }
        if grew {
            self.follow()?;
        }
        Ok(grew)
    }

    /// Plays the battle's next command, if one is to come.
    pub fn grow_one(&mut self) -> Result<bool, String> {
        if !self.playing() {
            return Ok(false);
        }
        self.play_next()?;
        self.follow()?;
        Ok(true)
    }

    /// The next command: the bot's, sent from its seat ([`Battle::take`]); none while the
    /// battle waits for a human's. A human's turn Auto Combat no longer plays (E17 M5 T4c1: a
    /// press stopped it after the command before was played) waits.
    fn play_next(&mut self) -> Result<(), String> {
        let auto = self.auto_turn(false);
        let (Some(live), Played::Growing(_)) = (Fight::here(self.fight.as_mut()), &self.played)
        else {
            return Ok(());
        };
        live.set_auto(auto);
        if live.waiting() {
            let placeholder = Played::Finished(Box::default());
            if let Played::Growing(mut growing) = std::mem::replace(&mut self.played, placeholder) {
                // Outlined from its start as a human's turn (pushed as the computer's).
                growing.outline_waiting();
                let tail = Box::new(growing.waiting());
                self.played = Played::Waiting(growing, tail);
            }
            self.head = Playhead::new(self.timeline());
            return Ok(());
        }
        match live.next()? {
            Some(played) => self.take(None, played),
            None => Ok(()),
        }
    }

    /// Whether Auto Combat plays the last turn started from its start, or (`next`) the turn to
    /// come after it (E17 M5 T4c1): switched on before and no key or button pressed since.
    fn auto_turn(&self, next: bool) -> bool {
        let mut starts: Vec<u32> = std::iter::once(0)
            .chain(self.played.timeline().turns.iter().map(|&(t, _)| t))
            .collect();
        if next {
            starts.push(u32::MAX);
        }
        Auto::of(&self.inputs, &starts).from_start(starts.len() - 1)
    }

    /// When the first turn started after `at` starts (E17 M5 T4c1: the one a stop pressed at `at`
    /// comes at), if the battle grew that far.
    pub fn turn_after(&self, at: u32) -> Option<u32> {
        self.played
            .timeline()
            .turns
            .iter()
            .map(|&(t, _)| t)
            .find(|&t| t > at)
    }

    /// Whether Auto Combat is on for the turns to come (E17 M5 T4c1): a key or a button pressed
    /// now stops it ([`Battle::stop_auto`]).
    pub fn auto_on(&self) -> bool {
        matches!(self.fight, Some(Fight::Here(_))) && self.auto_turn(true)
    }

    /// Auto Combat's button clicked (or its key released) at `at` (E17 M5 T4c1, design §7.22):
    /// on a human's turn waiting for its command, the button enabled (not both sides human,
    /// 0x46BB40), it switches on, the bot playing the turn from then; the switch joins `inputs`
    /// (the window's). Whether it did.
    pub fn switch_auto(&mut self, at: u32, inputs: &mut Vec<Input>) -> Result<bool, String> {
        let both = self
            .turned
            .last()
            .is_some_and(|t| t.observation.sides.iter().all(|s| s.human));
        // Not on a server (E17 M5 T6b1, *provisional*: the M5 review list).
        if !matches!(self.fight, Some(Fight::Here(_))) || both || !self.takes(at) {
            return Ok(false);
        }
        inputs.retain(|i| i.at() <= at);
        inputs.push(Input::Auto { at });
        self.inputs.clone_from(inputs);
        let choice = self.choice().ok_or("no command for the turn waiting")?;
        self.command(at, choice)
    }

    /// A key or a mouse button pressed at `at` while Auto Combat is on (E17 M5 T4c1): it stops as
    /// the next turn starts, the press doing nothing else; the stop joins `inputs`. Whether it
    /// did (nothing while it is off).
    pub fn stop_auto(&mut self, at: u32, inputs: &mut Vec<Input>) -> bool {
        if !self.auto_on() {
            return false;
        }
        inputs.retain(|i| i.at() <= at);
        inputs.push(Input::Stop { at });
        self.inputs.clone_from(inputs);
        true
    }

    /// A human's `command`, pressed at `at` while the battle waits for it (E17 M5 T2b): sent
    /// from the side to act's seat, the session validating it, its action starting at `at` or at
    /// the timeline's end if later; whether the battle took it (none waits: nothing).
    pub fn command(&mut self, at: u32, command: Command) -> Result<bool, String> {
        // A seat on a server sends it (E17 M5 T6b1): the battle idles on until the server's
        // answer and events come ([`Battle::poll`]); it takes no other meanwhile.
        if let (Some(Fight::Server(online)), Played::Waiting(..)) = (&self.fight, &self.played) {
            if self.served.is_none() {
                return Ok(false);
            }
            online.borrow_mut().send(command)?;
            self.served = None;
            return Ok(true);
        }
        let (Some(live), Played::Waiting(..)) = (Fight::here(self.fight.as_mut()), &self.played)
        else {
            return Ok(false);
        };
        let played = live.play(command)?;
        let placeholder = Played::Finished(Box::default());
        if let Played::Waiting(growing, _) = std::mem::replace(&mut self.played, placeholder) {
            let at = at.max(growing.timeline().end);
            self.played = Played::Growing(growing);
            self.take(Some(at), played)?;
        }
        // The timeline shown was the one waiting: a playhead afresh.
        self.head = Playhead::new(self.timeline());
        self.follow()?;
        Ok(true)
    }

    /// A command `played` at `at` (none: the computer's, as soon as the last one ended): what
    /// the spectator was sent for it joins the record, its events the timeline (every side's when
    /// a human plays here), the battle at the turn after it the mouse's; the next command the
    /// computer's unless the battle is over or a human's comes next. Over, the timeline is
    /// finished; a human's next, it waits.
    fn take(&mut self, at: Option<u32>, played: crate::live::Played) -> Result<(), String> {
        // Auto Combat plays the turn this command starts unless stopped by then (E17 M5 T4c1).
        let auto = self.auto_turn(true);
        let (Some(live), Played::Growing(growing)) =
            (Fight::here(self.fight.as_mut()), &mut self.played)
        else {
            return Ok(());
        };
        live.set_auto(auto);
        let (over, waiting) = (live.over(), live.waiting());
        let computer = !over && !waiting;
        let first = self.record.messages.len();
        self.record.messages.extend(played.messages);
        if live.human() {
            let (events, seen): (Vec<_>, Vec<_>) = played.events.into_iter().unzip();
            growing.push_seen(
                &anim::Command {
                    at,
                    events: &events,
                },
                computer,
                &seen,
            );
            self.commands += 1;
        } else {
            for message in &self.record.messages[first..] {
                if let ServerMessage::Events { events, .. } = message {
                    growing.push(&anim::Command { at, events }, computer);
                    self.commands += 1;
                }
            }
        }
        self.turned.extend(live.turn());
        if over || waiting {
            let placeholder = Played::Finished(Box::default());
            if let Played::Growing(growing) = std::mem::replace(&mut self.played, placeholder) {
                self.played = if over {
                    Played::Finished(Box::new(growing.finish()))
                } else {
                    let tail = Box::new(growing.waiting());
                    Played::Waiting(growing, tail)
                };
            }
        }
        Ok(())
    }

    /// Whether a seat on a server is played (E17 M5 T6b1).
    pub const fn online(&self) -> bool {
        matches!(self.fight, Some(Fight::Server(_)))
    }

    /// Takes what a seat's server sent since (E17 M5 T6b1; the window's, each frame, at `at` of the
    /// battle's clock and `now` of the window's, in milliseconds: a dropped connection is dialed
    /// again by the latter, which no pause stops):
    /// the record and the timeline grow by it, the turn it asks the seat to play comes to the
    /// mouse and the book. Whether the battle changed (nothing for a battle not on a server).
    ///
    /// # Errors
    /// The connection's fault, a refusal, another game's record at a re-join.
    pub fn poll(&mut self, at: u32, now: u64) -> Result<bool, String> {
        let Some(Fight::Server(online)) = &self.fight else {
            return Ok(false);
        };
        let grew = online.borrow_mut().poll(now)?;
        // A record opened afresh is a new timeline's, not this one's (E17 M5 T6b2).
        if self.opened_afresh() {
            return Ok(true);
        }
        let mut changed = grew && self.take_served(Some(at))?;
        changed |= self.serve_turn();
        if changed {
            self.follow()?;
        }
        Ok(changed)
    }

    /// The seat's record's messages not taken yet join the battle's (E17 M5 T6b1): each one's
    /// events grow the timeline, from `at` or the end of the action before (none: back to back,
    /// as a record's), each turn they start outlined as a human's (the other seat's too:
    /// *provisional*, the original's network turn not read); the battle then idles at its end, or
    /// is finished once the game ended. Whether any came.
    fn take_served(&mut self, at: Option<u32>) -> Result<bool, String> {
        let Some(Fight::Server(online)) = &self.fight else {
            return Ok(false);
        };
        let new: Vec<ServerMessage> = online
            .borrow()
            .record()
            .and_then(|r| r.messages.get(self.record.messages.len()..))
            .map(<[ServerMessage]>::to_vec)
            .unwrap_or_default();
        if new.is_empty() {
            return Ok(false);
        }
        let placeholder = Played::Finished(Box::default());
        let mut growing = match std::mem::replace(&mut self.played, placeholder) {
            Played::Waiting(growing, _) | Played::Growing(growing) => growing,
            finished @ Played::Finished(_) => {
                self.played = finished;
                self.record.messages.extend(new);
                return Ok(true);
            }
        };
        for message in &new {
            if let ServerMessage::Events { events, .. } = message {
                let at = at.map(|at| at.max(growing.timeline().end));
                growing.push(&anim::Command { at, events }, false);
                self.commands += 1;
            }
        }
        self.record.messages.extend(new);
        let over =
            (self.record.messages.iter()).any(|m| matches!(m, ServerMessage::GameEnded { .. }));
        self.played = if over {
            Played::Finished(Box::new(growing.finish()))
        } else {
            let tail = Box::new(growing.waiting());
            Played::Waiting(growing, tail)
        };
        self.head = Playhead::new(self.timeline());
        Ok(true)
    }

    /// The turn the server asks the seat to play, once every target it asked for came (E17 M5
    /// T6b1): the battle as the seat sees it at the timeline's end (`h3_view::remote::seen`),
    /// the server's legal commands and its active stack's own cast's aim join the turns the
    /// mouse and the book read. Whether a turn came.
    fn serve_turn(&mut self) -> bool {
        let Some(Fight::Server(online)) = &self.fight else {
            return false;
        };
        let online = online.borrow();
        let Some((legal, _)) = online.turn() else {
            self.served = None;
            return false;
        };
        let index = (self.record.messages.iter())
            .rposition(|m| matches!(m, ServerMessage::YourTurn { .. }));
        if index.is_none() || self.served == index {
            return false;
        }
        let Some(snapshot) = self.record.first_snapshot() else {
            return false;
        };
        let events = self.events();
        let timeline = self.played.timeline();
        let ruleset = self.ground.rules();
        let cost = |side: Side, spell: u16| {
            ruleset.map_or(0, |r| crate::spellbook::cost(snapshot, r, side, spell))
        };
        let observation = h3_view::remote::seen(snapshot, &events, timeline, timeline.end, cost);
        // The active stack's own cast, its drawn spell's (the Faerie Dragon's key F).
        let aims = (observation.drawn)
            .zip(online.targets(None))
            .and_then(|(spell, targets)| {
                crate::aims::served(self.ground.previews(), (spell, 0, true), targets)
            })
            .into_iter()
            .collect();
        let turn = crate::scenario::Turned {
            observation,
            commands: legal.to_vec(),
            aims,
        };
        drop(online);
        let commands = (self.record.messages.iter())
            .filter(|m| matches!(m, ServerMessage::Events { .. }))
            .count();
        if self.served_events == Some(commands) {
            self.turned.pop();
        }
        self.follower.touch(self.turned.len());
        self.turned.push(turn);
        (self.served, self.served_events) = (index, Some(commands));
        true
    }

    /// Every event of the record, in order.
    fn events(&self) -> Vec<h3_protocol::wire::Event> {
        (self.record.messages.iter())
            .filter_map(|m| match m {
                ServerMessage::Events { events, .. } => Some(events.iter().cloned()),
                _ => None,
            })
            .flatten()
            .collect()
    }

    /// The side to act's own observation (E17 M5 T3a: its hero's spell book): the engine's view
    /// of a battle fought here; on a server the seat's battle of the turn it plays (E17 M5 T6b1).
    fn own_view(&self) -> Option<Observation> {
        match self.fight.as_ref()? {
            Fight::Here(live) => live.own_view(),
            Fight::Server(_) => self
                .served
                .and_then(|_| self.turned.last())
                .map(|t| t.observation.clone()),
        }
    }

    /// Whether the side to act's hero casts `spell` without a target now (E17 M5 T3a): the
    /// engine's; on a server the server's answer (E17 M5 T6b1).
    fn casts_untargeted(&self, spell: u16) -> bool {
        match &self.fight {
            Some(Fight::Here(live)) => live.casts_untargeted(spell),
            Some(Fight::Server(online)) => online.borrow().targets(Some(spell)).is_some_and(|t| {
                t.iter()
                    .any(|t| matches!(t, h3_protocol::wire::Target::None {}))
            }),
            None => false,
        }
    }

    /// The side to act's hero's aim of `spell` now (E17 M5 T3b): the engine's; on a server from
    /// the server's targets, at the hero's mastery its own view shows (E17 M5 T6b1).
    fn hero_aim(&self, spell: u16) -> Option<mouse::aim::Aim> {
        match self.fight.as_ref()? {
            Fight::Here(live) => live.hero_aim(spell),
            Fight::Server(online) => {
                let mastery = self
                    .own_view()?
                    .own_hero?
                    .spell_book
                    .iter()
                    .find(|b| b.spell == spell)
                    .map_or(0, |b| b.mastery);
                let online = online.borrow();
                let targets = online.targets(Some(spell))?;
                crate::aims::served(self.ground.previews(), (spell, mastery, false), targets)
            }
        }
    }

    /// The outcome of a seat's battle on a server as its record shows it (E17 M5 T6b1): the
    /// game's end, every stack's count at the start and at the end.
    fn served_outcome(&self) -> Option<h3_contracts::BattleOutcome> {
        let result = self.record.messages.iter().find_map(|m| match m {
            ServerMessage::GameEnded { result, .. } => Some(*result),
            _ => None,
        })?;
        let snapshot = self.record.first_snapshot()?;
        let events = self.events();
        let timeline = self.played.timeline();
        let end = h3_view::remote::seen(snapshot, &events, timeline, timeline.end, |_, _| 0);
        Some(crate::results::served(snapshot, &end, result))
    }

    /// Follows the mouse's `inputs` and the log's `scrolls` (each at a time, in time order; E17
    /// M3 T6c-2): the cursor, the stack it marks, the stacks it enters fidgeting, its hints in
    /// the log; whether anything shown changed. Nothing without the text tables.
    pub fn mouse(&mut self, inputs: &[Input], scrolls: &[(u32, i64)]) -> Result<bool, String> {
        if self.ground.writer.is_none() {
            return Ok(false);
        }
        let (from, tail) = self.pointer(Some(inputs), false);
        if self.shown.ends_with(from, &tail) && scrolls == self.scrolls.as_slice() {
            return Ok(false);
        }
        self.inputs = inputs.to_vec();
        self.scrolls = scrolls.to_vec();
        self.show(from, tail)?;
        Ok(true)
    }

    /// The mouse over the battle as played so far (E17 M3 T6c-2) by `inputs` (none: the ones it
    /// took last), followed on from its last turn's start (E17 M5 T8a, R3-1; `grew`: the timeline
    /// grew since): where in [`Battle::shown`] (cut there) the lists returned go.
    fn pointer(&mut self, inputs: Option<&[Input]>, grew: bool) -> (Lens, Shown) {
        let ground = self.ground;
        let Some(writer) = ground.writer.as_ref() else {
            return (Lens::default(), Shown::default());
        };
        let timeline = self.played.timeline();
        let read = (timeline, self.turned.as_slice());
        let settled = (self.played.settled(), grew);
        (self.follower).read(&self.record, read, &ground.tables, settled);
        let record = &self.record;
        let name = |hint: &mouse::Hint, observation: &Observation| {
            crate::pointer::hint_text(record, &writer.catalogue, hint, observation)
        };
        let inputs = inputs.unwrap_or(&self.inputs);
        let words = (&ground.layout, &name as _, Bugs::Fixed);
        self.follower.shown(timeline, inputs, words)
    }

    /// The mouse, the timeline shown and the panel after the battle grew (or as it starts): the
    /// playhead goes on over what was played, unless the mouse's fidgets go in afresh.
    fn follow(&mut self) -> Result<(), String> {
        let (from, tail) = self.pointer(None, true);
        self.shown.splice(from, tail);
        if self.shown.entered.is_empty() && self.entered.is_none() {
            self.head.extend(self.played.timeline());
        } else {
            self.entered = self.entered(&self.shown);
            self.head = Playhead::new(self.timeline());
        }
        self.panels = self.panel()?;
        self.load_clips();
        Ok(())
    }

    /// Shows what the mouse shows now, `tail` from `from` on: the stacks it entered fidgeting in
    /// the timeline, its hints and the log's scrolls in the panel.
    fn show(&mut self, from: Lens, tail: Shown) -> Result<(), String> {
        let added = tail.entered.len();
        let cut = self.shown.splice(from, tail);
        let entered = &self.shown.entered;
        if cut.entered[..] != entered[entered.len() - added..] {
            self.entered = self.entered(&self.shown);
            self.head = Playhead::new(self.timeline());
        }
        self.panels = self.panel()?;
        Ok(())
    }

    /// The timeline as played with the stacks `shown` entered fidgeting (none: none entered).
    fn entered(&self, shown: &Shown) -> Option<Timeline> {
        (!shown.entered.is_empty()).then(|| {
            let mut timeline = self.played.timeline().clone();
            let group = self.ground.tables.battle.groups.mouse_over;
            // Each stack's frames of the fidget's group, by the creature its opening shows.
            let stacks = self.record.first_snapshot().map(|o| o.stacks.as_slice());
            let frames = |id: u8| {
                let s = stacks.unwrap_or_default().iter().find(|s| s.id == id);
                s.and_then(|s| {
                    self.ground
                        .looks
                        .get(&s.creature)?
                        .frames
                        .get(&u32::from(group))
                })
                .copied()
                .unwrap_or(0)
            };
            timeline.enter(&shown.entered, u32::from(group), &frames);
            timeline
        })
    }

    /// The panel over the record so far, its log through the mouse's hints and the scrolls once
    /// the mouse moved (else the timeline's own).
    fn panel(&self) -> Result<Panels, String> {
        let ground = self.ground;
        let Some(writer) = &ground.writer else {
            let parsed = parse(&ground.archives)?;
            let timeline = self.played.timeline();
            return crate::log::panels(
                &self.record,
                (&timeline.log, &timeline.controllers),
                &ground.tables,
                &ground.layout,
                (&parsed, &ground.exe),
                Bugs::Fixed,
            );
        };
        if self.inputs.is_empty() && self.scrolls.is_empty() {
            let timeline = self.played.timeline();
            return writer.panels(&self.record, (&timeline.log, &timeline.controllers));
        }
        // The scrolls after the log's entries at their time.
        let mut logged = self.shown.log.clone();
        for &(at, by) in &self.scrolls {
            let i = logged.partition_point(|(t, _)| *t <= at);
            logged.insert(i, (at, Logged::Scroll(by)));
        }
        // Auto Combat as the window switched it (E17 M5 T4c1).
        let snapshot = self
            .record
            .first_snapshot()
            .ok_or("the record holds no snapshot")?;
        let auto = crate::pointer::auto(&self.inputs, self.played.timeline());
        Ok(writer
            .panels(&self.record, (&logged, &self.played.timeline().controllers))?
            .with_auto(snapshot, auto))
    }

    /// The cursor at `t`: the spell book's once it set one (E17 M5 T3a), else the combat cursor's
    /// frame (the panel's pointer before the mouse is anywhere).
    pub fn cursor(&self, t: u32) -> CursorLook {
        let hot_of = |hot: &[[i32; 2]], frame: u32| {
            usize::try_from(frame)
                .ok()
                .and_then(|f| hot.get(f).copied())
                .unwrap_or_default()
        };
        // The result window leaves the combat cursor's pointer `battle_end` set (E17 M5 T4b1).
        if self
            .message
            .as_ref()
            .is_some_and(|(_, _, a)| *a == Asked::Result)
        {
            let mouse = &self.ground.layout.mouse;
            let frame = h3_view::result::CURSOR_FRAME;
            return CursorLook {
                def: mouse.cursor.clone(),
                frame,
                hot: hot_of(&mouse.hot, frame),
            };
        }
        // A message box sets the default cursor's frame 0 (0x4F74AC; E17 M5 T4a).
        if self.message.is_some() {
            let book = &self.ground.layout.book;
            return CursorLook {
                def: self.ground.layout.message.cursor.clone(),
                frame: 0,
                hot: hot_of(&book.hot, 0),
            };
        }
        if let Some(frame) = self.book.as_ref().and_then(|(open, _, _)| open.cursor) {
            let book = &self.ground.layout.book;
            return CursorLook {
                def: book.cursor.clone(),
                frame,
                hot: hot_of(&book.hot, frame),
            };
        }
        let mouse = &self.ground.layout.mouse;
        match self.shown.look(t, mouse) {
            // The spell cursor while an aim takes the hex (E17 M5 T3b).
            Some(mouse::track::Look::Spell(frame)) => CursorLook {
                def: mouse.spell_cursor.clone(),
                frame,
                hot: mouse.spell_hot,
            },
            look => {
                let frame = match look {
                    Some(mouse::track::Look::Combat(frame)) => frame,
                    _ => mouse.panel_frame,
                };
                CursorLook {
                    def: mouse.cursor.clone(),
                    frame,
                    hot: hot_of(&mouse.hot, frame),
                }
            }
        }
    }

    /// Whether the spell book is open over the battle.
    pub fn book_open(&self) -> bool {
        self.book.is_some()
    }

    /// Opens the acting hero's spell book at `at` (E17 M5 T3a; `hero_book_button` 0x59E900 from
    /// the panel's button or its key C): while the battle waits for a human whose hero's book the
    /// panel's button takes. Whether it opened. *Not modelled:* the refusals' message boxes (an
    /// anti-magic garrison, the Orb of Inhibition), T4's.
    pub fn open_book(&mut self, at: u32) -> bool {
        if self.book.is_some() || !self.takes(at) {
            return false;
        }
        let ground = self.ground;
        // The side to act's own view: its hero's spells (as its seat is sent them).
        let (Some(observation), Some(ruleset), Some(writer)) =
            (self.own_view(), ground.rules(), ground.writer.as_ref())
        else {
            return false;
        };
        if !crate::spellbook::takes(&observation) {
            return false;
        }
        let names = (&ground.tables, &writer.catalogue);
        let manifest = &self.record.header.manifest;
        let Some(hero) = crate::spellbook::hero(
            &observation,
            (manifest, ruleset),
            names,
            &ground.layout.book,
        ) else {
            return false;
        };
        let open = h3_view::book::Open::new(&mut self.book_memory, hero, &writer.catalogue);
        // The hero's player's colours (the record names none: red, blue; CLT-Q6).
        let index = usize::from(observation.viewer == Side::Defender);
        let player = observation.sides[index]
            .colour
            .unwrap_or_else(|| u8::try_from(index).unwrap_or(0));
        self.book = Some((open, at, player));
        true
    }

    /// The open book's answer to `outcome` (E17 M5 T3a): closed, or closed on a spell, whose cast
    /// is given when the engine takes it without a target (one that takes a target is aimed first:
    /// T3b's); a refusal or a description shows its message box over the book, a stack spell
    /// that no hex takes "That spell will affect no one!" over the battle (E17 M5 T4a).
    fn book_did(&mut self, outcome: h3_view::book::Outcome) -> BookDid {
        use h3_view::book::Outcome;
        match outcome {
            Outcome::Nothing => BookDid::Nothing,
            Outcome::Refused { .. } | Outcome::Described(_) => {
                self.book_box(outcome);
                if self.message.is_some() {
                    BookDid::Changed
                } else {
                    BookDid::Nothing
                }
            }
            Outcome::Turned => BookDid::Changed,
            Outcome::Closed => {
                self.book = None;
                BookDid::Changed
            }
            Outcome::Chosen(spell) => {
                let opened = self.book.take().map_or(0, |(_, at, _)| at);
                let untargeted = Command::Cast {
                    spell,
                    target: h3_protocol::wire::Target::None {},
                    second: h3_protocol::wire::Target::None {},
                };
                if self.fight.is_none() {
                    return BookDid::Changed;
                }
                if self.casts_untargeted(spell) {
                    return BookDid::Cast(untargeted);
                }
                // A spell with a second target (Teleport, Sacrifice) has aims of its own (not
                // built); a stack spell or Teleport the engine casts at nothing tells so
                // (GENRLTXT 186, 0x59F135; E17 M5 T4a).
                let keys = &self.record.header.manifest.keys.spells;
                let original =
                    motion::spell_entry_in(keys, &self.ground.tables, spell).map(|e| e.original);
                let second = original.is_some_and(|o| SECOND_TARGET.contains(&o));
                let aim = self.hero_aim(spell);
                if aim.is_none()
                    && original.is_some_and(|o| crate::boxes::affects_someone_only(u32::from(o)))
                {
                    let text = self
                        .ground
                        .writer
                        .as_ref()
                        .map(|w| crate::boxes::no_one(&w.catalogue))
                        .unwrap_or_default();
                    self.tell(opened, &text, (Style::Ok, None), Asked::Told);
                    return BookDid::Changed;
                }
                match aim.filter(|_| !second) {
                    Some(aim) => {
                        self.follower.touch(self.turned.len().saturating_sub(1));
                        if let Some(turn) = self.turned.last_mut() {
                            turn.aims.retain(|a| a.spell != spell || a.creature);
                            turn.aims.push(aim);
                        }
                        self.aiming = Some((spell, false));
                        BookDid::Aim(spell)
                    }
                    None => BookDid::Changed,
                }
            }
        }
    }

    /// The Faerie Dragon's key F at `at` (E17 M5 T3b; `battle_buttons` 0x475C17): its drawn
    /// spell's aim starts while the battle waits for a human whose acting stack may cast it
    /// (`enter_player_cast` returns at once once its side's hero cast this round: the engine
    /// lists no cast then). The spell aimed, if any.
    pub fn dragon_aim(&mut self, at: u32) -> Option<u16> {
        if self.aiming.is_some() || self.book.is_some() || !self.takes(at) {
            return None;
        }
        let turn = self.turned.last()?;
        // The acting side's hero cast this round: the key does nothing (0x59EC76).
        let o = &turn.observation;
        let side = o
            .active
            .and_then(|id| o.stacks.iter().find(|s| s.id == id))
            .map(|s| usize::from(s.controller == Side::Defender))?;
        if o.sides[side].hero.as_ref().is_some_and(|h| h.casts > 0) {
            return None;
        }
        let spell = turn.aims.iter().find(|a| a.creature)?.spell;
        self.aiming = Some((spell, true));
        Some(spell)
    }

    /// The screen's centre of the open book's place showing `spell`, if its page shows it (E17
    /// M5 T3b: an unattended run's press).
    pub fn book_place_of(&self, spell: u16) -> Option<[i32; 2]> {
        let (open, _, _) = self.book.as_ref()?;
        let k = open
            .page
            .places
            .iter()
            .position(|p| p.as_ref().is_some_and(|p| p.handle == spell))?;
        Some(crate::pointer::book_place(u8::try_from(k).ok()?))
    }

    /// Whether the open book's page has a next one.
    pub fn book_next(&self) -> bool {
        self.book
            .as_ref()
            .is_some_and(|(open, _, _)| open.page.next)
    }

    /// The hex of the stack `id` in the battle waiting (its position).
    pub fn stack_hex(&self, id: u8) -> Option<u8> {
        let turn = self.turned.last()?;
        Some(turn.observation.stacks.iter().find(|s| s.id == id)?.hex)
    }

    /// Whether a spell is being aimed (E17 M5 T3b).
    pub fn aiming(&self) -> bool {
        self.aiming.is_some()
    }

    /// The aim given up (Escape, the right button; E17 M5 T3b).
    pub fn cancel_aim(&mut self) {
        self.aiming = None;
    }

    /// A left press at `at` while a spell is aimed (E17 M5 T3b; 0x5A30E4): the cast at the hex the
    /// last move's cursor took, none over a hex it does not take (the aim goes on).
    pub fn aim_press(&mut self, at: u32) -> Option<Command> {
        let (spell, creature) = self.aiming?;
        let hex = self.shown.aimed(at)?;
        let command = self.taken(at, |field| {
            let aim = field
                .legal
                .aims
                .iter()
                .find(|a| a.spell == spell && a.creature == creature)?;
            let target = match aim.target(field, hex)? {
                Target::Stack(stack) => h3_protocol::wire::Target::Stack { stack },
                Target::Hex(hex) => h3_protocol::wire::Target::Hex { hex },
            };
            Some(if creature {
                Command::CreatureCast { target }
            } else {
                Command::Cast {
                    spell,
                    target,
                    second: h3_protocol::wire::Target::None {},
                }
            })
        })?;
        self.aiming = None;
        Some(command)
    }

    /// A key down giving the book's `key` while it is open (E24 M3 T1b: by the table of keys).
    pub fn book_key(&mut self, key: h3_view::book::Key) -> BookDid {
        let Some(writer) = self.ground.writer.as_ref() else {
            return BookDid::Nothing;
        };
        let Some((open, _, _)) = self.book.as_mut() else {
            return BookDid::Nothing;
        };
        let outcome = open.key(&mut self.book_memory, key, &writer.catalogue);
        self.book_did(outcome)
    }

    /// A left press at `point` while the book is open.
    pub fn book_press(&mut self, point: [i32; 2]) -> BookDid {
        let ground = self.ground;
        let Some(writer) = ground.writer.as_ref() else {
            return BookDid::Nothing;
        };
        let Some((open, _, _)) = self.book.as_mut() else {
            return BookDid::Nothing;
        };
        let outcome = open.press(
            &mut self.book_memory,
            (&ground.layout.book, &writer.catalogue),
            point,
        );
        self.book_did(outcome)
    }

    /// The mouse moved to `point` while the book is open: whether its status line or cursor
    /// changed.
    pub fn book_hover(&mut self, point: [i32; 2]) -> bool {
        let ground = self.ground;
        let (Some(writer), Some((open, _, _))) = (ground.writer.as_ref(), self.book.as_mut())
        else {
            return false;
        };
        open.hover(
            &mut self.book_memory,
            (&ground.layout.book, &writer.catalogue),
            point,
        )
    }

    /// Whether a message box is open over the battle (E17 M5 T4a).
    pub fn message_open(&self) -> bool {
        self.message.is_some()
    }

    /// The colour whose view the screen shows (`viewing_player` 0x4CEA20; E17 M5 T4a): in hot
    /// seat the adventure map's current player when human, which the client takes as the
    /// attacker's when a human plays it here, else the defender's when one does, else red
    /// (*provisional*: the client knows no adventure map's turn, and the executable falls back on
    /// the game's highest human colour, not the battle's).
    fn viewing_player(&self) -> u8 {
        let colour = |index: usize| {
            self.record
                .first_snapshot()
                .and_then(|s| s.sides.get(index))
                .and_then(|side| side.colour)
                .unwrap_or_else(|| u8::try_from(index).unwrap_or(0))
        };
        match self.ground.humans {
            [true, _] => colour(0),
            [false, true] => colour(1),
            [false, false] => 0,
        }
    }

    /// Opens a message box of `style` showing `text` and `picture` (E17 M5 T4a), the battle
    /// standing still behind it from `at`; what its OK does is `asked`. Whether it opened (not
    /// without the text tables).
    fn tell(
        &mut self,
        at: u32,
        text: &str,
        (style, picture): (Style, Option<&Picture>),
        asked: Asked,
    ) -> bool {
        let Some(writer) = self.ground.writer.as_ref() else {
            return false;
        };
        let player = self.viewing_player();
        let Some(message) = writer.message(&self.ground.layout, text, style, picture, player)
        else {
            return false;
        };
        self.message = Some((h3_view::message::Open::new(message), at, asked));
        true
    }

    /// The panel's Surrender or Retreat clicked at `at` (E17 M5 T4a; `battle_surrender_button`
    /// 0x474FE9, `battle_retreat_button` 0x474FA1): while the battle waits for a human whose side
    /// may leave, the question (GENRLTXT 29 to retreat; 33 with the other side's hero's name and
    /// the price to surrender), its OK the command. The side may retreat when the engine lists
    /// it, surrender at all when its own view prices it (whether it can pay is told after the
    /// OK, as the original's). Whether a box opened.
    pub fn ask_leave(&mut self, at: u32, id: u16) -> bool {
        if self.message.is_some()
            || self.book.is_some()
            || self.aiming.is_some()
            || self.fight.is_none()
            || !self.takes(at)
        {
            return false;
        }
        let (Some(writer), Some(view), Some(turn)) = (
            self.ground.writer.as_ref(),
            self.own_view(),
            self.turned.last(),
        ) else {
            return false;
        };
        let catalogue = &writer.catalogue;
        let keys = &self.record.header.manifest.keys;
        let leave = Leave::of(&view, &turn.commands, &keys.hero_types, catalogue);
        match leave.ask(catalogue, id) {
            Some((text, asked)) => self.tell(at, &text, (Style::OkCancel, None), asked),
            None => false,
        }
    }

    /// A left press at `point` while a message box is open (E17 M5 T4a).
    pub fn message_press(&mut self, point: [i32; 2]) -> MessageDid {
        self.message_did(|open| open.press(point))
    }

    /// The mouse moved to `point` while a message box is open: whether its button changed.
    pub fn message_moved(&mut self, point: [i32; 2]) -> bool {
        self.message_did(|open| open.moved(point)) != MessageDid::Nothing
    }

    /// The left button released while a message box is open.
    pub fn message_release(&mut self) -> MessageDid {
        self.message_did(h3_view::message::Open::release)
    }

    /// A key down giving the box's `key` while one is open (E24 M3 T1b: by the table of keys; `by`
    /// the key's name, whose release clicks).
    pub fn message_key(&mut self, key: h3_view::message::Key, by: &str) -> MessageDid {
        self.message_did(|open| open.key(key, by))
    }

    /// A key up while a message box is open.
    pub fn message_key_up(&mut self, key: &str) -> MessageDid {
        self.message_did(|open| open.key_up(key))
    }

    /// The open box's answer to an input (E17 M5 T4a): a click on a button closes it; Retreat's
    /// OK retreats, Surrender's surrenders or, when the side's gold is below the price, tells it
    /// so (GENRLTXT 30, 0x475119) at once, the battle still standing.
    fn message_did(
        &mut self,
        input: impl FnOnce(&mut h3_view::message::Open) -> h3_view::message::Outcome,
    ) -> MessageDid {
        use h3_view::message::Outcome;
        let Some((open, at, asked)) = self.message.as_mut() else {
            return MessageDid::Nothing;
        };
        let (at, asked) = (*at, *asked);
        let id = match input(open) {
            Outcome::Nothing => return MessageDid::Nothing,
            Outcome::Changed => return MessageDid::Changed,
            Outcome::Closed(id) => id,
        };
        self.message = None;
        // The tactics phase's message closed: the first turn starts (E17 M5 T4c2a).
        self.opening = false;
        if asked == Asked::Result {
            self.result_closed = true;
            return MessageDid::Ended;
        }
        let Some(writer) = self.ground.writer.as_ref() else {
            return MessageDid::Changed;
        };
        match crate::boxes::answered(asked, id, &writer.catalogue) {
            Answered::Give(command) => MessageDid::Give(command),
            Answered::Tell(text) => {
                self.tell(at, &text, (Style::Ok, None), Asked::Told);
                MessageDid::Changed
            }
            Answered::Closed => MessageDid::Changed,
        }
    }

    /// Opens the battle's result window once `at` reaches the battle's end (E17 M5 T4b1; design
    /// §7.20): as `battle_end` 0x477470 opens it as the battle is decided, over the whole screen
    /// darkened, the battle standing as it ended; from the engine's outcome of the battle fought
    /// here (none for a record file's). Whether it opened.
    pub fn open_result(&mut self, at: u32) -> bool {
        if self.result_closed || self.message.is_some() {
            return false;
        }
        let Some((ended, _)) = self.timeline().ended.filter(|&(t, _)| t <= at) else {
            return false;
        };
        let (Some(fight), Some(writer), Some(opening)) = (
            self.fight.as_ref(),
            self.ground.writer.as_ref(),
            self.record.first_snapshot(),
        ) else {
            return false;
        };
        let (outcome, ruleset) = match fight {
            Fight::Here(live) => match live.outcome() {
                Some(found) => found,
                None => return false,
            },
            // A seat on a server: the outcome as its record shows it (E17 M5 T6b1), the window
            // by the ruleset compiled here (none without the packs).
            Fight::Server(_) => match (self.served_outcome(), self.ground.rules()) {
                (Some(outcome), Some(ruleset)) => (outcome, ruleset),
                _ => return false,
            },
        };
        let sources = crate::results::Sources {
            manifest: &self.record.header.manifest,
            tables: &self.ground.tables,
            ruleset,
            catalogue: &writer.catalogue,
            layout: &self.ground.layout,
        };
        let player = self.viewing_player();
        let Some((window, _)) = crate::results::window(&sources, opening, &outcome, player) else {
            return false;
        };
        self.message = Some((h3_view::message::Open::new(window), ended, Asked::Result));
        self.behind = h3_view::result::Behind::default();
        true
    }

    /// Opens the next of the timeline's notices once `at` reaches its time (E17 M5 T5a, design
    /// §7.25: Clone's message when it found no place, `clone_place_and_create` 0x5A6E22), the
    /// battle standing behind it from that time. Its time when it opened.
    pub fn open_notice(&mut self, at: u32) -> Option<u32> {
        if self.message.is_some() {
            return None;
        }
        let &(t, notice) = self
            .timeline()
            .notices
            .get(self.notices)
            .filter(|&&(t, _)| t <= at)?;
        self.notices += 1;
        let key = match notice {
            anim::Notice::NoPlace => "battle.clone.no-place",
        };
        let text = self
            .ground
            .writer
            .as_ref()?
            .catalogue
            .text(key, &h3_ui::text::Args::new());
        self.tell(t, &text, (Style::Ok, None), Asked::Told)
            .then_some(t)
    }

    /// Whether the result window is open (E17 M5 T4b1).
    pub fn result_open(&self) -> bool {
        self.message
            .as_ref()
            .is_some_and(|(_, _, asked)| *asked == Asked::Result)
    }

    /// Whether the result window was closed (E17 M5 T4b1): the battle's screen is done.
    pub const fn result_closed(&self) -> bool {
        self.result_closed
    }

    /// The centre of the open box's button `id` on the screen, if it has one (E17 M5 T4a: an
    /// unattended run's press).
    pub fn message_button(&self, id: u16) -> Option<[i32; 2]> {
        let (open, _, _) = self.message.as_ref()?;
        let dialog = &open.message.dialog;
        let (at, size) = dialog.item(id)?.area()?;
        Some([
            dialog.place[0] + at[0] + size[0] / 2,
            dialog.place[1] + at[1] + size[1] / 2,
        ])
    }

    /// The box the book's refusal of a spell above the hero's spell points shows (GENRLTXT 207
    /// with the cost and the points, 0x59D462), or an adventure spell's description with its
    /// picture and name (0x59D4DC), over the book (E17 M5 T4a).
    fn book_box(&mut self, outcome: h3_view::book::Outcome) {
        let Some(&(_, opened, _)) = self.book.as_ref() else {
            return;
        };
        // The spells' pictures' DEF, for its size (a description's picture).
        let def = self.ground.layout.message.spell_pictures.clone();
        let size = match outcome {
            h3_view::book::Outcome::Described(_) => {
                if self.load_names(vec![def.to_ascii_uppercase()]).is_err() {
                    return;
                }
                self.batch.sprite(&def).map_or([0, 0], |s| {
                    [s.def.width, s.def.height].map(|v| i32::try_from(v).unwrap_or(i32::MAX))
                })
            }
            _ => [0, 0],
        };
        let (Some(writer), Some((open, _, _))) = (self.ground.writer.as_ref(), self.book.as_ref())
        else {
            return;
        };
        let Some((text, picture)) =
            crate::boxes::book_box(outcome, &open.hero, &writer.catalogue, (&def, size))
        else {
            return;
        };
        self.tell(opened, &text, (Style::Ok, picture.as_ref()), Asked::Told);
    }

    /// The battle's layout.
    pub fn layout(&self) -> &BattleLayout {
        &self.ground.layout
    }

    /// The panel shown at `at` (E17 M5 T4c2b, design §7.24): the tactics phase's own until the
    /// phase ends, then the battle's (`combat_dialog_battle_panel` 0x4731F0 swaps them).
    pub fn panel_at(&self, at: u32) -> &h3_ui::ui::Dialog {
        let layout = &self.ground.layout;
        self.record
            .first_snapshot()
            .map_or(&layout.panel, |snapshot| {
                layout.panel_in(self.panels.now(snapshot, &self.timeline().turns, at))
            })
    }

    /// Whether the tactics phase's panel shows at `at` (E17 M5 T4c2b): its keys come before the
    /// battle's.
    pub fn in_tactics(&self, at: u32) -> bool {
        std::ptr::eq(self.panel_at(at), &self.ground.layout.tactics_panel)
    }

    /// The draw list of the battle at `moment` (at `t`), the panel's after it as the battle's
    /// turns and lines left it (E17 M3 T6b).
    pub fn list(&self, t: u32, moment: &Moment) -> Result<Vec<Draw>, String> {
        let snapshot = self
            .record
            .first_snapshot()
            .ok_or("the record holds no snapshot")?;
        let (tables, layout) = (&self.ground.tables, &self.ground.layout);
        // The stack the mouse marks (E17 M3 T6c-2).
        let mut moment = moment.clone();
        moment.marked = self.shown.marked(t);
        moment.area = self.shown.area(t).to_vec();
        // Behind the tactics phase's message the battle before its first turn; a human's turn
        // in the phase, waiting, its stack's hexes shaded (E17 M5 T4c2a, design §7.23).
        if self.opening {
            moment.before_first_turn();
        } else if self.waiting()
            && t >= self.timeline().end
            && let Some(turn) = self.turned.last()
        {
            moment.shade = h3_view::tactics::shade(&turn.observation);
        }
        let mut list = scene::battle_at(
            snapshot,
            &moment,
            &self.record.header.manifest,
            tables,
            layout,
        )
        .map_err(|e| format!("{t} ms: {e}"))?;
        let text = self.panels.text(t);
        if self.opening {
            list.extend(Panels::before(snapshot, layout));
        } else {
            list.extend(self.panels.at_pressed(
                snapshot,
                &self.timeline().turns,
                t,
                layout,
                text,
                self.pressed,
            ));
        }
        // Earthquake's shake (E17 M4 T2c-2): the screen as it was, shifted.
        if let Some(steps) = moment.shake {
            list = vec![scene::shaken(list, steps, layout)];
        }
        Ok(list)
    }

    /// The command a left press at `point` gives at `at` (E17 M5 T2b; `h3_view::mouse::click`,
    /// design §9.11), by the battle the waiting turn asks.
    pub fn press(&self, at: u32, point: [i32; 2]) -> Option<Command> {
        self.taken(at, |field| mouse::click(field, &self.ground.layout, point))
    }

    /// The command the panel's button `id` gives when clicked at `at` (E17 M5 T2b;
    /// `h3_view::mouse::button`): Wait and Defend when listed.
    pub fn button(&self, at: u32, id: u16) -> Option<Command> {
        self.taken(at, |field| mouse::button(field.legal, id))
    }

    /// Where a press at `at` gives `wanted` (E17 M5 T2b: the window's unattended presses): the
    /// panel's Wait or Defend (in the tactics phase its Next Creature or Start Combat, E17 M5
    /// T4c2b), else a point of `points` ([`crate::pointer::press_points`]) or the siege's
    /// rectangles; none if none gives it (a hero's cast: M5 T3; CLT-Q25's step back) or the
    /// battle takes no press then.
    pub fn press_for(
        &self,
        at: u32,
        wanted: Command,
        points: &[([i32; 2], Vec<[i32; 2]>)],
    ) -> Option<Press> {
        let layout = &self.ground.layout;
        let rects = [layout.mouse.keep, layout.mouse.tower].map(crate::pointer::middle);
        self.taken(at, |field| {
            let button = [
                mouse::WAIT,
                mouse::DEFEND,
                mouse::NEXT_CREATURE,
                mouse::START_COMBAT,
            ]
            .into_iter()
            .find(|&id| mouse::button(field.legal, id) == Some(wanted));
            if let Some(id) = button {
                return Some(Press::Button(id));
            }
            points
                .iter()
                .flat_map(|(centre, slots)| std::iter::once(centre).chain(slots))
                .chain(&rects)
                .copied()
                .find(|&p| mouse::click(field, layout, p) == Some(wanted))
                .map(Press::At)
        })
    }

    /// The bot's choice for the side to act (E17 M5 T2b: what the window's unattended run
    /// presses for a human); none for a record or once the battle is over.
    pub fn choice(&mut self) -> Option<Command> {
        match self.fight.as_mut()? {
            Fight::Here(live) => live.choice(),
            // A seat on a server (*provisional*, a test's player): its hero casts first the first
            // spell the server lets it cast at its first target (not one with a second target),
            // else the plain player's choice of the server's legal commands (`connect`'s).
            Fight::Server(online) => {
                let online = online.borrow();
                let (legal, castable) = online.turn()?;
                let keys = &self.record.header.manifest.keys.spells;
                let cast = castable.iter().find_map(|&spell| {
                    let original = motion::spell_entry_in(keys, &self.ground.tables, spell)?;
                    let target = *online.targets(Some(spell))?.first()?;
                    (!SECOND_TARGET.contains(&original.original)).then_some(Command::Cast {
                        spell,
                        target,
                        second: h3_protocol::wire::Target::None {},
                    })
                });
                let width = self.record.header.manifest.field.width;
                cast.or_else(|| crate::connect::plain(legal, online.seat(), width))
            }
        }
    }

    /// Whether the fought battle's record so far is the start of the battle the bot fights at
    /// once with the same setup (E17 M5 T2b: a battle pressed by the bot's choices); none for a
    /// record.
    pub fn as_the_bot_fights(&mut self) -> Option<Result<bool, String>> {
        let Some(Fought::Here(here)) = self.ground.fight.as_ref() else {
            return None;
        };
        let (ruleset, setup, seed) = &**here;
        let mut check = || {
            let bot = crate::live::bot_record(ruleset, setup.clone(), *seed)?;
            let bot = record::parse(&bot).map_err(|e| e.to_string())?;
            let ours = Fight::here(self.fight.as_mut())
                .ok_or("no battle")?
                .record()?;
            let ours = record::parse(&ours).map_err(|e| e.to_string())?;
            Ok(bot.header == ours.header && bot.messages.starts_with(&ours.messages))
        };
        Some(check())
    }

    /// Whether the battle takes the mouse's presses and the keys at `at` (E17 M5 T2b, design
    /// §9.11): a record's always (the viewer's scrolls and buttons); a battle fought here only
    /// while it waits for a human's command and its last action played (input made before is
    /// dropped as the turn starts).
    pub fn takes(&self, at: u32) -> bool {
        let served = match &self.fight {
            None => return true,
            Some(Fight::Here(_)) => true,
            // A seat on a server: while it has a turn to play (E17 M5 T6b1).
            Some(Fight::Server(_)) => self.served.is_some(),
        };
        served && self.waiting() && at >= self.played.timeline().end
    }

    /// What `give` gives in the battle of the turn waiting at `at`: none unless the battle
    /// waits for a human's command and its last action played (the timeline's end reached; a
    /// press made before is dropped as the turn starts, design §9.11).
    fn taken<T>(&self, at: u32, give: impl Fn(&Field<'_>) -> Option<T>) -> Option<T> {
        if self.fight.is_none() || !self.takes(at) {
            return None;
        }
        let turn = self.turned.last()?;
        let observation = &turn.observation;
        let tables = &self.ground.tables;
        let manifest = &self.record.header.manifest;
        let legal =
            crate::pointer::legal(manifest, tables, observation, (&turn.commands, &turn.aims));
        let facing = crate::pointer::facing(self.timeline(), at);
        give(&Field {
            observation,
            legal: &legal,
            facing: &facing,
            bugs: Bugs::Fixed,
        })
    }

    /// Whether the log's scroll buttons take a click at `t` (E17 M3 T9b, the reviewer).
    pub fn scrolls(&self, t: u32) -> bool {
        self.panels.scrolls(&self.timeline().turns, t)
    }

    /// The draw list of the battle at `t` (the playhead's moment: going back starts it over; the
    /// battle played on first if `t` is at or past its timeline's end).
    pub fn list_at(&mut self, t: u32) -> Result<Vec<Draw>, String> {
        // Behind the open spell book the battle stands as it was when it opened (E17 M5 T3a),
        // behind a message box as it was when the box opened (E17 M5 T4a).
        let book = self.book.as_ref().map(|&(_, opened, _)| opened);
        let message = self.message.as_ref().map(|&(_, opened, _)| opened);
        let at = [Some(t), book, message]
            .into_iter()
            .flatten()
            .min()
            .unwrap_or(t);
        self.grow(at)?;
        let timeline = self
            .entered
            .as_ref()
            .unwrap_or_else(|| self.played.timeline());
        self.head.seek(timeline, at);
        let mut moment = self.head.moment(timeline);
        // Behind the result window the battle stands as it ended, its flags as last shown (E17 M5
        // T4b1, `h3_view::result::behind`), the moment before read once (E17 M5 T8a, R3-2).
        if self.result_open() {
            self.behind
                .show(&mut moment, at, |t| Moment::at(timeline, t));
        }
        let mut list = self.list(at, &moment)?;
        if let Some((open, _, player)) = &self.book {
            let layout = &self.ground.layout;
            list.extend(open.draws(&layout.book, &layout.player_palette, *player));
        }
        if let Some((open, opened, asked)) = &self.message {
            // The result window over the whole screen darkened (E17 M5 T4b1).
            if *asked == Asked::Result {
                list.push(h3_view::result::backdrop(self.ground.layout.frame));
            }
            let message = &open.message;
            list.extend(
                message
                    .dialog
                    .draws(&message.shown, &self.ground.layout.player_palette)
                    .into_iter()
                    .map(Draw::Dialog),
            );
            // Its clip over its slot from the moment it opened (E17 M5 T4b2b).
            if *asked == Asked::Result {
                list.extend(self.clip(t.saturating_sub(*opened)));
            }
        }
        Ok(list)
    }

    /// Every distinct frame up to `until` (past the end the battle idles; none: to its end), the
    /// battle played as the frames reach its timeline's end: each frame's time, moment and draw
    /// list; a moment that draws what the one before drew is not a new frame.
    pub fn frames(&mut self, until: Option<u32>) -> Result<Vec<(u32, Moment, Vec<Draw>)>, String> {
        let mut frames: Vec<(u32, Moment, Vec<Draw>)> = Vec::new();
        let mut head = Playhead::new(self.timeline());
        let (mut from, mut times, mut timed) = (0, Vec::new(), None);
        loop {
            let (end, playing) = (self.timeline().end, self.playing());
            // A growing timeline's changes up to its end; past it the next command comes first.
            let horizon = until.map_or(end, |u| if playing { u.min(end) } else { u });
            if timed != Some((end, horizon)) {
                times = self.timeline().changes(horizon);
                timed = Some((end, horizon));
            }
            let next = times.get(times.partition_point(|&c| c < from)).copied();
            let t = match next {
                Some(t) if t < end || !playing => t,
                _ if playing => {
                    self.grow_one()?;
                    head.extend(self.timeline());
                    continue;
                }
                _ => break,
            };
            head.seek(self.timeline(), t);
            let moment = head.moment(self.timeline());
            let list = self.list(t, &moment)?;
            if frames.last().is_none_or(|(_, _, last)| *last != list) {
                frames.push((t, moment, list));
            }
            from = t + 1;
        }
        Ok(frames)
    }

    /// Loads into the batch every name `lists` need and `extra`.
    pub fn load_batch<'a>(
        &mut self,
        lists: impl IntoIterator<Item = &'a [Draw]>,
        extra: &[&str],
    ) -> Result<(), String> {
        let mut names = std::collections::BTreeSet::new();
        for list in lists {
            names.extend(batch::names(list, &self.ground.layout));
        }
        names.extend(extra.iter().map(|n| n.to_ascii_uppercase()));
        self.load_names(
            names
                .into_iter()
                .filter(|n| !held(&self.batch, n))
                .collect(),
        )
    }

    /// Decodes `names` from the archives into the batch.
    fn load_names(&mut self, names: Vec<String>) -> Result<(), String> {
        if names.is_empty() {
            return Ok(());
        }
        let parsed = parse(&self.ground.archives)?;
        for name in &names {
            // A clip from the video archives (E17 M5 T4b2b), every other asset from the LODs.
            let bytes = if name.ends_with(".BIK") {
                self.ground.videos.read(name)?
            } else {
                lookup(&parsed, name)?
            };
            self.batch.add(name, &bytes).map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    /// `list` drawn into a frame with the batch, what it names loaded first.
    pub fn compose(&mut self, t: u32, list: &[Draw]) -> Result<Screen, String> {
        let missing = batch::names(list, &self.ground.layout)
            .into_iter()
            .filter(|n| !held(&self.batch, n))
            .collect();
        self.load_names(missing)?;
        compose(list, &self.batch, &self.ground.layout, Bugs::Fixed)
            .map_err(|e| format!("{t} ms: {e}"))
    }

    /// A DEF of the batch.
    pub fn sprite(&self, name: &str) -> Option<&Sprite> {
        self.batch.sprite(name)
    }

    /// The sounds of each stack living at the start, by its id, and of each creature the battle
    /// may create ([`Ground::stack_sounds`]).
    pub fn stack_sounds(&self) -> Voices {
        self.ground.stack_sounds()
    }

    /// The battle's own sounds a cue may name, by cue ([`Ground::battle_sounds`]): those its
    /// cues name once the battle is over; while it is still fought, every one it may play (E17
    /// M5 T1b: its cues are not known yet; T2b: waiting for a human too).
    pub fn battle_sounds(&self) -> BTreeMap<Sound, String> {
        let sounds = self.ground.battle_sounds();
        let cued = |cue: &Sound| self.timeline().cues.iter().any(|c| c.sound == *cue);
        // Still fought: by the bot or waiting for a human (E17 M5 T2b).
        let fought = !matches!(self.played, Played::Finished(_));
        sounds
            .into_iter()
            .filter(|(cue, _)| fought || cued(cue))
            .collect()
    }

    /// The result window's clip `since` ms after it opened (E17 M5 T4b2b): the one its tune's
    /// branch picks, when the video archives hold it (else the slot stays bare, as the
    /// original's does without them) and `h3-bink` decodes it (E17 M5 T8b1, R2b-3).
    fn clip(&self, since: u32) -> Option<Draw> {
        let (_, tune) = self.end_tune()?;
        let clip = h3_view::clip::of(tune);
        let held = std::iter::once(clip.start)
            .chain(clip.looped)
            .all(|name| self.ground.videos.has(name));
        (held && self.clips != Clips::Refused)
            .then(|| h3_view::result::clip(&self.ground.layout.result, tune, since))
    }

    /// Decodes the result window's clips into the batch once the battle's end is known (E17 M5
    /// T8b1, R2b-5): the batch holds what a screen needs before it is shown, and the end is
    /// known as its last command is played, before its last action has played out and the
    /// window opens. A clip the archives hold but that cannot be read or decoded leaves the
    /// slot bare, a warning on stderr (R2b-3). Only a battle fought here or on a server opens
    /// the window.
    fn load_clips(&mut self) {
        if self.clips != Clips::Unknown || self.fight.is_none() {
            return;
        }
        let Some((_, tune)) = self.end_tune() else {
            return;
        };
        let clip = h3_view::clip::of(tune);
        let names: Vec<&str> = std::iter::once(clip.start).chain(clip.looped).collect();
        self.clips = if add_clips(&self.ground.videos, &mut self.batch, &names) {
            Clips::Loaded
        } else {
            Clips::Refused
        };
    }

    /// When the result window's clip frame on the screen at `t` was decoded and when the next
    /// decode comes (E17 M5 T4b2b; `h3_view::clip::decodes`), the clips loaded as the battle's
    /// end became known (`Battle::load_clips`): the times the window draws at while the result
    /// window stands. None without the window or its clip.
    pub fn clip_times(&self, t: u32) -> Option<(u32, Option<u32>)> {
        let opened = self
            .message
            .as_ref()
            .filter(|(_, _, asked)| *asked == Asked::Result)
            .map(|&(_, opened, _)| opened)?;
        let Some(Draw::Video { start, looped, .. }) = self.clip(0) else {
            return None;
        };
        let timing = |name: &str| self.batch.video(name).map(h3_view::clip::Video::timing);
        let first = timing(&start)?;
        let second = looped.as_deref().and_then(timing);
        // The window's own drawing: the clip's slips fixed (CLT-Q31).
        let since = t.saturating_sub(opened);
        let (began, next) = h3_view::clip::decodes(since, first, second, Bugs::Fixed);
        Some((
            opened.saturating_add(began),
            next.map(|n| opened.saturating_add(n)),
        ))
    }

    /// When the battle is decided and the tune its end plays then (E17 M3 T8, design §8.2): the
    /// result's window shows the human side's view by the snapshot's `human`; a battle at a town
    /// is the record's siege, a fort or not (the manager's +0x53C8; E17 M4 T2c-2, `defended`).
    pub fn end_tune(&self) -> Option<(u32, Tune)> {
        let (at, outcome) = self.timeline().ended?;
        let snapshot = self.record.first_snapshot();
        let human = snapshot.map_or([false; 2], |s| {
            [0, 1].map(|i| s.sides.get(i).is_some_and(|d| d.human))
        });
        let town = snapshot.is_some_and(|s| s.siege.is_some());
        Some((at, h3_view::sound::end_tune(outcome, human, town)))
    }

    /// The frame's size, as the layout gives it.
    pub fn frame_size(&self) -> [u32; 2] {
        self.ground.layout.frame
    }
}

/// Decodes the clips `names` from the video archives into `batch` (E17 M5 T8b1, R2b-3): those
/// the archives lack or the batch holds are left; whether every other one was read and decoded,
/// a failure warned of on stderr.
fn add_clips(videos: &crate::videos::Videos, batch: &mut Batch, names: &[&str]) -> bool {
    let mut all = true;
    for &name in names {
        if !videos.has(name) || held(batch, name) {
            continue;
        }
        let added = videos
            .read(name)
            .and_then(|bytes| batch.add(name, &bytes).map_err(|e| e.to_string()));
        if let Err(e) = added {
            eprintln!(
                "the result window's clip {}; its slot stays bare",
                crate::connect::printable(&e)
            );
            all = false;
        }
    }
    all
}

/// Whether the batch holds the asset `name`.
fn held(batch: &Batch, name: &str) -> bool {
    batch.sprite(name).is_some()
        || batch.picture(name).is_some()
        || batch.palette(name).is_some()
        || batch.font(name).is_some()
        || batch.video(name).is_some()
}

/// The archives' directories.
fn parse<'a>(
    archives: &'a [(&'static str, Vec<u8>)],
) -> Result<Vec<(&'static str, LodArchive<'a>)>, String> {
    archives
        .iter()
        .map(|(name, b)| {
            LodArchive::parse(b)
                .map(|a| (*name, a))
                .map_err(|e| format!("{name}: {e:?}"))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_clip_that_does_not_decode_is_left_out() {
        // E17 M5 T8b1, R2b-3: `h3-bink` refuses the loop, so the pair leaves the slot bare; a
        // clip the archives lack is no failure.
        let dir = std::env::temp_dir().join(format!("h3-view-clips-{}", std::process::id()));
        let data = dir.join("Data");
        fs::create_dir_all(&data).unwrap();
        let sample = h3_bink::build::sample(1);
        let archive = h3_assets::build::vid(&[("rtstart.bik", &sample), ("rtloop.bik", b"BIKb")]);
        fs::write(data.join("Video.vid"), archive).unwrap();
        let videos = crate::videos::Videos::open(&dir);
        let mut batch = Batch::default();
        assert!(add_clips(&videos, &mut batch, &["Win3.bik"]));
        assert!(add_clips(&videos, &mut batch, &["rtstart.bik"]));
        assert!(batch.video("RTSTART.BIK").is_some());
        assert!(!add_clips(
            &videos,
            &mut batch,
            &["rtstart.bik", "rtloop.bik"]
        ));
        assert!(batch.video("RTLOOP.BIK").is_none());
        fs::remove_dir_all(&dir).unwrap();
    }
}
