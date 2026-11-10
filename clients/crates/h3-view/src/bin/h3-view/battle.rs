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
use h3_towns::TownRuleset;
use h3_ui::sprite::Sprite;
use h3_view::anim::Timeline;
use h3_view::anim::{
    self, Casts, Growing, Look, MagicLook, Playhead, SiegeStage, Sound, Speed, Stage,
};
use h3_view::army_info::Drawn;
use h3_view::auto::{Acting, Auto, turn_at};
use h3_view::batch::{self, Batch};
use h3_view::battle_log::Logged;
use h3_view::compose::compose;
use h3_view::hero::Heroes;
use h3_view::layout::{self, BattleLayout};
use h3_view::message::{MessageBox, Picture, Style};
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
use crate::live::{Computer, Live};
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
    /// human ones, the others the computer's; none, the scenario's own and the computer plays
    /// both.
    pub humans: [bool; 2],
    /// Who plays the computer's sides of a scenario's battle (E17 M6 T5b1: the window's the
    /// engine's original computer by `--sight`, `play`'s the fixed bot).
    pub computer: Computer,
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
    /// The town part its packs compile to here (E17 M6 T3c3: the stats dialog's building lines),
    /// beside the fight's ruleset; none: a record file's, or a server's game whose packs do not
    /// compile here.
    towns: Option<TownRuleset>,
    /// The sides a human plays here ([`Source::humans`]).
    humans: [bool; 2],
    /// Who plays the computer's sides ([`Source::computer`]).
    computer: Computer,
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

    /// The ruleset of the window's own previews (E17 M5 T6b1: an aim's area) and of the result
    /// window's creatures' values (E17 M5 T8b2, R5b-2: hit points, AI values): a battle fought
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

    /// The town part of the battle's packs, when its ruleset is the previews' (E17 M6 T3c3).
    fn towns(&self) -> Option<&TownRuleset> {
        self.previews()?;
        self.towns.as_ref()
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
            let players = (humans, source.computer);
            let opening = Live::new(&ruleset, setup.clone(), source.seed, players)?.record()?;
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
        let manifest = &record.header.manifest;
        let towns = match &fight {
            Some(Fought::Here(_)) => Some(crate::scenario::towns(&source.packs, manifest)?),
            Some(Fought::Server(served)) if served.ruleset.is_some() => {
                crate::scenario::towns(&source.packs, manifest).ok()
            }
            _ => None,
        };
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
            towns,
            humans,
            computer: source.computer,
            lengths: std::cell::OnceCell::new(),
        })
    }
}

/// The sounds of each stack living at the start, by its id (E17 M2 T5b); a siege's turrets
/// (off the field) the town's shooter's, as their look (E17 M4 T3a, R2-1). And those of each
/// creature the record's events create, by its handle (E17 M5 T8b2, R6-1: a summoned stack's,
/// a clone's, a Pit Lord's Demons'; a created stack's cues name its creature).
pub fn stack_sounds(record: &Record, tables: &Tables) -> Voices {
    let made = record.events().filter_map(|e| match *e {
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
        (Sound::ManaDrain, battle.mana_drain_sound.clone()),
        (Sound::DeathBlow, battle.death_blow_sound.clone()),
        // Ice Bolt's explosion and the Familiars' channel (E17 M6 T5d2b).
        (Sound::IceBolt, battle.ice_bolt_sound.clone()),
        (
            Sound::ChannelDrain,
            battle.channel_sounds.as_ref().map(|[d, _]| d.clone()),
        ),
        (
            Sound::ChannelFill,
            battle.channel_sounds.as_ref().map(|[_, f]| f.clone()),
        ),
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
    // The pickup sounds of Eagle Eye's message after the result window (E17 M6 T5c), by N.
    for (n, name) in (1..).zip(battle.pickup_sounds.iter().flatten()) {
        sounds.push((Sound::Pickup(n), Some(name.clone())));
    }
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
    /// Auto Combat's switches and stops the battle took (E17 M5 T4c1; T8b2, R5c-3 and R5c-4:
    /// apart from the window's inputs, so a seek back or Restart keeps them and a turn's Auto
    /// Combat costs the turns only).
    switches: Vec<Input>,
    /// The combat options dialog's closes that changed the Auto Combat boxes, each with its turn
    /// and time, after the boxes the battle began with (E17 M6 T4b1, *provisional*; empty while
    /// none changed: the settings' boxes then): kept as the switches, so a seek back or Restart
    /// keeps which turns played themselves.
    boxes: Vec<h3_view::auto::BoxChange>,
    /// The Auto Combat boxes as the options dialog opened, until its close is applied
    /// ([`Battle::boxes_closed`]).
    boxes_open: Option<[bool; 5]>,
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
    /// A stack's stats dialog open over the battle and when it opened (E17 M6 T2a2a; design
    /// §12.2): the battle stands as it was then (the dialog's modal loop runs no idle step), its
    /// creature stepping each 100 ms from 100 ms after the opening.
    stats: Option<(crate::stats::Open, u32)>,
    /// What the stats dialog remembers between openings (the original's global: one per game),
    /// in the client's profile.
    stats_memory: h3_view::stats::Memory,
    /// The aim the stats dialog's cast button started (E17 M6 T2a2b1; 0x468954): the dialog's
    /// close (`Input::Looked`) comes as the aim ends.
    look_aim: bool,
    /// A message box open over the battle (E17 M5 T4a; design §7.19): the box, when it opened
    /// and what its OK does. The battle (and the book, when the box is the book's) stands still
    /// behind it (its modal loop runs no idle step).
    message: Option<(h3_view::message::Open, u32, Asked)>,
    /// When Auto Combat's question opened (E17 M6 T5b2a): the battle stands still from then
    /// until the answer that gives its command (`Battle::answer`, `anim::Growing::hold`).
    asked_at: Option<u32>,
    /// The combat options dialog open over the battle and when it opened (E17 M6 T4a; design
    /// §12.12): the battle stands as it was then (`battle_combat_options`' modal loop runs no
    /// idle step); a box over it (its help, the no-sound box) is `message`.
    options: Option<(h3_view::options::Open, u32)>,
    /// The options dialog closed and the mouse not moved since: the dialog's arrow stays (nothing
    /// set or posted at its close, oddity 10).
    options_arrow: bool,
    /// The options dialog closed after a click (+0x4C): its settings to save (0x46F7A2).
    options_save: bool,
    /// The combat options' settings (the window's settings file's; E17 M6 T4a).
    pub settings: h3_view::options::Settings,
    /// The window's audio device opened: without it the options' bars at volume 0 tell so.
    pub audio: bool,
    /// The combat speed the battle grows at: the ground's, then the options dialog's.
    speed: Speed,
    /// The result window was closed (E17 M5 T4b1): the battle's screen is done (since E17 M6 T5c
    /// after Eagle Eye's message, when it shows one).
    result_closed: bool,
    /// The boxes of Eagle Eye's message still to open after the result window (E17 M6 T5c,
    /// design §12.21), each with its pickup sound's N: laid out as the window opened.
    eagle_eye: Vec<(MessageBox, u8)>,
    /// The pickup sound of the box of Eagle Eye's message that opened last (its N), until the
    /// window takes it to play ([`Battle::take_pickup`]).
    pickup: Option<u8>,
    /// The result window under Eagle Eye's boxes (E17 M6 T5c, recorded: `eagle-eye`), its OK up:
    /// when it opened, and its clip's time frozen at its close ([`Battle::window_closed_at`]) or
    /// else at the first box's first draw.
    window_behind: Option<(MessageBox, u32, Option<u32>)>,
    /// The boxes opened so far (E17 M6 T5c, the reviewer): two boxes alike in a row (Eagle Eye's
    /// eight and seven pictures, one size) are two openings, each emptying the input queue.
    opened: u32,
    /// The battle behind the result window while it is open (E17 M5 T8a, R3-2).
    behind: h3_view::result::Behind,
    /// The battle's end as the record shows it (`Battle::end_seen`), replayed once (E17 M5 T8b2,
    /// R5b-1).
    end_seen: std::cell::OnceCell<Observation>,
    /// Whether each side has army survivors at the battle's end (`Battle::survivors`), read once
    /// (E17 M5 T8b2, R5b-3).
    survivors: std::cell::OnceCell<[bool; 2]>,
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

    /// [`Fight::here`], to read.
    fn here_ref(fight: Option<&Self>) -> Option<&Live<'g>> {
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
    /// Auto Combat's question or offer answered (E17 M6 T5b2a): its OK (true) or its Cancel, the
    /// computer's next for the side ([`Battle::answer`]).
    Answer(bool),
    /// A turret's info closed by its OK (E17 M6 T2b2): the battle takes the mouse again as
    /// after a stats dialog (`Input::Looked`).
    Looked,
    /// A refusal of the spell book button or of the aim's entry closed by its OK (E17 M6 T5a,
    /// design §12.17; [`Asked::Refused`]): the box being modal, only now does the button's path
    /// post the mouse where it rests (0x474F72, `Input::Cancel`).
    Refused,
}

/// What a box that asked `asked` leads to as it closes, before its buttons' answer
/// ([`crate::boxes::answered`]): the result window closes the battle's screen, a turret's info
/// hands the mouse back as a stats dialog, a refusal posts the mouse; none: by the button.
fn closed(asked: Asked) -> Option<MessageDid> {
    match asked {
        Asked::Result | Asked::EagleEye => Some(MessageDid::Ended),
        Asked::Turret => Some(MessageDid::Looked),
        Asked::Refused => Some(MessageDid::Refused),
        _ => None,
    }
}

/// The original's spells aimed at a second target too (Sacrifice, Teleport): their aims are not
/// built (E17 M5 T3b). By number only where the battle's ruleset is not known (a server's game
/// of other content; E17 M5 T8b2, R4b-4: [`crate::aims::second_target`]).
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

/// When a stats dialog's creature first steps after the dialog opened (its modal run sets the
/// frame deadline 100 ms after the opening, 0x5F4801).
const STATS_FIRST_STEP: u32 = 100;

/// What the stats dialog did with an input (E17 M6 T2a2a).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatsDid {
    /// Nothing the screen shows.
    Nothing,
    /// The dialog, its status line or its cursor changed: draw again.
    Changed,
    /// Its OK closed it, or the right button's held dialog ended (E17 M6 T2b1): the battle
    /// takes the mouse again (`Input::Looked`).
    Closed,
    /// Its cast button closed it (E17 M6 T2a2b1; 0x468943): the acting Faerie Dragon's cast
    /// starts ([`Battle::look_cast`]), the battle taking the mouse as the aim ends.
    Cast,
}

/// What the combat options dialog did with an input (E17 M6 T4a): to draw again, its samples, its
/// no-sound box, its close.
pub type OptionsDid = h3_view::options::Did;

/// What the spell book button did (E17 M5 T3a; its refusals E17 M6 T5a).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Opened {
    /// Nothing: the battle does not take it.
    No,
    /// The book opened.
    Book,
    /// A box tells why the hero may not cast (an anti-magic garrison, the Orb of Inhibition;
    /// design §12.17): the battle takes the mouse again after its OK (`hero_book_button`'s −1,
    /// the mouse posted, 0x474F72; [`MessageDid::Refused`]).
    Refused,
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
    /// The book closed on a spell the engine refuses and a box tells why over the battle (E17
    /// M6 T5a, design §12.17): the battle takes the mouse again after its OK
    /// ([`MessageDid::Refused`]).
    Refused,
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
        let mut battle = Self::open(self.ground, opening)?;
        // The combat options' settings and speed carry over (E17 M6 T4a).
        (battle.settings, battle.audio) = (self.settings, self.audio);
        battle.set_speed(self.speed);
        Ok(battle)
    }

    /// Every animation after now at combat speed `speed` (E17 M6 T4a: the options dialog's; a
    /// record file's battle, played whole, keeps its own).
    pub fn set_speed(&mut self, speed: Speed) {
        self.speed = speed;
        match &mut self.played {
            Played::Growing(g) | Played::Waiting(g, _) => g.set_speed(speed),
            Played::Finished(_) => {}
        }
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
                    (ground.humans, ground.computer),
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
            switches: Vec::new(),
            boxes: Vec::new(),
            boxes_open: None,
            scrolls: Vec::new(),
            pressed: None,
            book: None,
            book_memory: h3_view::book::Memory::default(),
            aiming: None,
            stats: None,
            stats_memory: h3_view::stats::Memory::new(Bugs::Fixed),
            look_aim: false,
            message: None,
            asked_at: None,
            options: None,
            options_arrow: false,
            options_save: false,
            settings: h3_view::options::Settings::FIRST_RUN,
            audio: false,
            speed: ground.speed,
            result_closed: false,
            eagle_eye: Vec::new(),
            pickup: None,
            window_behind: None,
            opened: 0,
            behind: h3_view::result::Behind::default(),
            end_seen: std::cell::OnceCell::new(),
            survivors: std::cell::OnceCell::new(),
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
        let phase = snapshot
            .tactics
            .as_ref()
            .map(|t| t.side)
            .filter(|&side| ground.humans[usize::from(side == Side::Defender)]);
        if let (Some(writer), Some(side)) = (ground.writer.as_ref(), phase) {
            let text = writer
                .catalogue
                .text("battle.tactics.phase", &h3_ui::text::Args::new());
            let shown = (Style::Ok, None);
            battle.opening = battle.tell_framed((0, Some(side)), &text, shown, Asked::Told);
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

    /// The next command: the computer's, sent from its seat ([`Battle::take`]); none while the
    /// battle waits for a human's. A human's turn Auto Combat no longer plays (E17 M5 T4c1: a
    /// press stopped it after the command before was played) waits; so does one whose start
    /// asked the computer for its hero's cast and got none (E17 M6 T5b1). E17 M6 T5b2a: Auto
    /// Combat's question (the retreat test's yes) waits for the human's answer from the action's
    /// millisecond, its box open (*provisional*: no outline, the turn the computer's). E17 M6
    /// T5b2b (CLT-Q61): so does a turn whose computer's cast for the hero a key or a button
    /// stopped, from the cast's end: the cast put its outline back there (§12.18, as on a turn
    /// the human plays; *provisional*: the outline's first tick and the panel's switch at that
    /// millisecond, not recorded).
    fn play_next(&mut self) -> Result<(), String> {
        let (auto, cast) = self.auto_turn(false);
        let (Some(live), Played::Growing(_)) = (Fight::here(self.fight.as_mut()), &self.played)
        else {
            return Ok(());
        };
        live.set_auto(auto);
        live.set_cast(cast);
        if let Some(played) = live.next()? {
            return self.take(None, played);
        }
        let asked = live.question().is_some();
        let placeholder = Played::Finished(Box::default());
        let mut end = 0;
        if let Played::Growing(mut growing) = std::mem::replace(&mut self.played, placeholder) {
            // Outlined from its start as a human's turn (pushed as the computer's); after an
            // action of its own (the computer's cast Auto Combat stopped at the end of) already.
            if !asked && !within(growing.timeline()) {
                growing.outline_waiting();
            }
            end = growing.timeline().end;
            let tail = Box::new(growing.waiting());
            self.played = Played::Waiting(growing, tail);
        }
        self.head = Playhead::new(self.timeline());
        if asked {
            self.open_question(end)?;
        }
        Ok(())
    }

    /// Whether Auto Combat plays the last turn started, or (`next`) the turn to come after it
    /// (E17 M5 T4c1): switched on before and no key or button pressed since, its stack playing
    /// itself (E17 M6 T4b1: by its box; the battle tells `Live` so, which then plays it by the
    /// computer or waits); the last turn started from when Auto Combat handed it over, a switch
    /// in it too (E17 M6 T5b1: its stack's action after the hero's cast; E17 M6 T5b2b: not after
    /// a key or a button pressed during that cast, CLT-Q61, the stop read at its end). And
    /// whether its start asked the computer for a human's hero's cast (E17 M6 T5b1, design
    /// §12.18: Auto Combat on as it started and Combat Auto Spells set then, `Auto::asks_cast`;
    /// in the client's profile only on a turn it plays, CLT-Q62).
    fn auto_turn(&self, next: bool) -> (bool, bool) {
        let mut starts: Vec<u32> = std::iter::once(0)
            .chain(self.played.timeline().turns.iter().map(|&(t, _)| t))
            .collect();
        if next {
            starts.push(u32::MAX);
        }
        let k = starts.len() - 1;
        let casts = crate::pointer::casts(&self.record, self.played.timeline());
        let auto = Auto::casting(&self.switches, &starts, &self.itself(next), &casts);
        let plays = if next {
            auto.from_start(k)
        } else {
            auto.plays(k, u32::MAX)
        };
        (plays, auto.asks_cast(k, self.boxes_at(k), Bugs::Fixed))
    }

    /// The Auto Combat boxes as turn `k` started (E17 M6 T5b1, `h3_view::auto::boxes_at`): the
    /// battle's first, then the options dialog's closes before it.
    fn boxes_at(&self, k: usize) -> [bool; 5] {
        let first = first_boxes(&self.boxes, self.boxes_open, self.settings.auto);
        h3_view::auto::boxes_at(first, self.boxes.get(1..).unwrap_or_default(), k)
    }

    /// From when each turn's stack plays itself under Auto Combat (E17 M6 T4b1, design §12.13,
    /// `h3_view::auto::itself`): the opening's, then each one started, then (`next`) the turn to
    /// come after them (the side to act's, fought here); by the boxes the battle began with and
    /// the options dialog's changes since.
    fn itself(&self, next: bool) -> Vec<Option<u32>> {
        let tables = &self.ground.tables;
        let timeline = self.played.timeline();
        let mut stacks = crate::pointer::stacks(&self.record, timeline, &self.turned, tables);
        if next {
            let manifest = &self.record.header.manifest;
            let to_come = Fight::here_ref(self.fight.as_ref()).and_then(Live::turn);
            stacks.push(to_come.map_or(Acting::Itself, |turn| {
                crate::pointer::stack_of(manifest, tables, &turn.observation)
            }));
        }
        let first = first_boxes(&self.boxes, self.boxes_open, self.settings.auto);
        h3_view::auto::itself(&stacks, first, self.boxes.get(1..).unwrap_or_default())
    }

    /// Auto Combat over the battle's turns as played so far (E17 M5 T4c1).
    fn auto(&self) -> Auto {
        let itself = self.itself(false);
        let played = (
            self.played.timeline(),
            crate::pointer::human_sides(&self.record),
        );
        crate::pointer::auto(&self.switches, played, &itself)
    }

    /// When Auto Combat stopped in turn `k` at the end of the computer's cast for the human's
    /// hero (E17 M6 T5b2b, CLT-Q61, `Auto::stopped_at`): the turn the human's from then, the key
    /// that stopped it acting there.
    pub fn auto_stopped_at(&self, k: usize) -> Option<u32> {
        self.auto().stopped_at(k)
    }

    /// Whether a key or a button pressed now stops Auto Combat (E17 M5 T4c1;
    /// [`Battle::stop_auto`]): fought here, on (the flag, after the last turn's start) with no
    /// stop waiting, and no human's turn it does not play waiting for its command (E17 M6 T4b1:
    /// the press is that turn's), nor playing the computer's cast for its hero (E17 M6 T5b1: the
    /// original reads no stop on that turn, the key reaching the turn after the cast; here it is
    /// dropped, as a press while an action plays on a human's turn).
    pub fn auto_on(&self) -> bool {
        if !matches!(self.fight, Some(Fight::Here(_))) || self.waiting() {
            return false;
        }
        let timeline = self.played.timeline();
        let (auto, k) = (self.auto(), timeline.turns.len());
        // An action of the last turn started, from its start on: the computer's cast (or one
        // that took no time and started it, whose press a turn not played drops all the same).
        let within = within(timeline);
        auto.on(k, u32::MAX) && !auto.stopped(k) && (auto.plays(k, u32::MAX) || !within)
    }

    /// While the battle fought here grows, the end of the computer's cast for the human's hero
    /// it played last, on a turn Auto Combat plays (E17 M6 T5b2b, CLT-Q61, `Auto::holds`): the
    /// stack's action is asked only once the clock reaches it (the window's `Changes::after`),
    /// a key or a button pressed until then stopping Auto Combat there.
    pub fn cast_end(&self) -> Option<u32> {
        let timeline = self.played.timeline();
        let here = matches!(self.fight, Some(Fight::Here(_)));
        let cast = timeline.actions.last().is_some_and(|a| a.cast.is_some());
        if !self.playing() || !here || !cast {
            return None;
        }
        let k = timeline.turns.len();
        let casts = crate::pointer::casts(&self.record, timeline);
        let last = timeline.actions.len().saturating_sub(1);
        self.auto().holds(k, casts.get(k).copied().flatten(), last)
    }

    /// Whether Auto Combat was on as turn `k` started (0 the opening's; E17 M6 T4b1): a stop
    /// waiting for it then stopped it only if not (a turn it does not play reads none, the press
    /// dropped: CLT-Q45 (2)).
    pub fn auto_at_start(&self, k: usize) -> bool {
        self.auto().on_at_start(k)
    }

    /// Auto Combat's button clicked (or its key released) at `at` (E17 M5 T4c1, design §7.22;
    /// E17 M6 T4b1, §12.13): on a human's turn waiting for its command, the button enabled (not
    /// both sides human, 0x46BB40), it toggles: on over a stack that plays itself (its box set),
    /// the bot playing the turn from then; on over one that does not, or off, nothing more (the
    /// panel's button shows which, CLT-Q45 (1)). The switch joins the battle's, in the turn
    /// waiting (T8b2, R5c-1: the bot's command may take no time, the next turn starting at `at`
    /// too). Whether it did.
    pub fn switch_auto(&mut self, at: u32) -> Result<bool, String> {
        if !self.takes_auto() || !self.takes(at) {
            return Ok(false);
        }
        let turn = self.played.timeline().turns.len();
        self.switches.push(Input::Auto { at, turn });
        self.hand_over(turn, at, true)
    }

    /// The computer's command for the turn `turn` waiting if Auto Combat plays it at `at` (a
    /// switch or a box set); E17 M6 T5b1 (design §12.18): a switch on runs the turn's start
    /// again, so with Combat Auto Spells set (the boxes now) the computer is asked for the
    /// human's hero's cast first; the original's whether the stack plays itself or not, the
    /// client's (CLT-Q62) only when it does (`Auto::asks_cast_at`). Else the panel made again
    /// (its button). Whether anything changed.
    fn hand_over(&mut self, turn: usize, at: u32, switched: bool) -> Result<bool, String> {
        let auto = self.auto();
        let plays = auto.plays(turn, at);
        let cast = switched && auto.asks_cast_at(turn, at, self.boxes_at(usize::MAX), Bugs::Fixed);
        if let Some(live) = Fight::here(self.fight.as_mut()) {
            live.set_auto(plays);
            if cast {
                live.cast_again();
            }
        }
        if (plays || cast) && self.computer_at(at)? {
            return Ok(true);
        }
        self.panels = self.panel()?;
        Ok(true)
    }

    /// The computer's next command for the turn waiting, played at `at` (Auto Combat's hand-over:
    /// the hero's cast asked, or the stack's action): as a human's [`Battle::command`], the
    /// command the computer's. Whether one was played (none: a cast asked gave none, the turn
    /// waits on; E17 M6 T5b1).
    fn computer_at(&mut self, at: u32) -> Result<bool, String> {
        let (Some(live), Played::Waiting(..)) = (Fight::here(self.fight.as_mut()), &self.played)
        else {
            return Ok(false);
        };
        let Some(played) = live.next()? else {
            // Auto Combat's question at the switch's own turn (E17 M6 T5b2a).
            return self.open_question(at);
        };
        self.grow_from_waiting(at, played)?;
        Ok(true)
    }

    /// Opens the box of Auto Combat's question or offer waiting for the human's answer (E17 M6
    /// T5b2a, design §12.19; `crate::boxes::question`) at `at`, the battle standing still behind
    /// it; whether it opened (none waits, or a box is open: [`Battle::poll`] opens it once that
    /// one closed). Without the text tables or the side's view it cannot be shown: the question
    /// is answered Cancel (the battle fights on; never a battle waiting on a box it cannot
    /// show); the offer's gold, its DEF not loaded, is left out. *Not built* for a seat on a
    /// server: it plays no Auto Combat (`Battle::takes_auto`), so asks no question.
    ///
    /// # Errors
    /// The answer's, played (`Battle::answer`).
    fn open_question(&mut self, at: u32) -> Result<bool, String> {
        let Some(question) = Fight::here_ref(self.fight.as_ref()).and_then(Live::question) else {
            return Ok(false);
        };
        if self.message.is_some() {
            return Ok(false);
        }
        // The resources' pictures' DEF, for its size (the offer's gold).
        let def = self.ground.layout.message.resources.clone();
        let size = match question {
            crate::live::Question::Offer { .. } => {
                // Not loaded: no size, the picture's slot empty.
                let _ = self.load_names(vec![def.to_ascii_uppercase()]);
                self.batch.sprite(&def).map_or([0, 0], |s| {
                    [s.def.width, s.def.height].map(|v| i32::try_from(v).unwrap_or(i32::MAX))
                })
            }
            crate::live::Question::Retreat => [0, 0],
        };
        let (Some(writer), Some(own)) = (self.ground.writer.as_ref(), self.own_view()) else {
            return self.answer(at, false);
        };
        let hero_types = &self.record.header.manifest.keys.hero_types;
        let (text, picture, asked) =
            crate::boxes::question(question, &own, hero_types, &writer.catalogue, (&def, size));
        if !self.tell(at, &text, (Style::OkCancel, picture.as_ref()), asked) {
            // Not laid out: as without the tables.
            return self.answer(at, false);
        }
        if self.asked_at.is_none() {
            self.asked_at = Some(at);
        }
        Ok(true)
    }

    /// The human's answer `ok` at `at` to Auto Combat's question or offer, its box closed (E17 M6
    /// T5b2a, `Live::answer`): the computer's command for the side (the stack's action, the
    /// Retreat or the Surrender) starts at `at`, the battle having stood still behind the boxes
    /// since the question opened (`Growing::hold`: its outline kept, no idle step, the loop's one
    /// tick at the answer; recorded); or the offer's box opens at once. Whether anything changed.
    pub fn answer(&mut self, at: u32, ok: bool) -> Result<bool, String> {
        let (Some(live), Played::Waiting(..)) = (Fight::here(self.fight.as_mut()), &self.played)
        else {
            return Ok(false);
        };
        let Some(played) = live.answer(ok)? else {
            return self.open_question(at);
        };
        let placeholder = Played::Finished(Box::default());
        if let Played::Waiting(mut growing, _) = std::mem::replace(&mut self.played, placeholder) {
            growing.hold((self.asked_at.take().unwrap_or(at), at));
            self.played = Played::Growing(growing);
            self.take(None, played)?;
        }
        self.head = Playhead::new(self.timeline());
        self.follow()?;
        Ok(true)
    }

    /// What the open message box asks, if one is open (E17 M6 T5b2a: the unattended presser's
    /// answer to Auto Combat's boxes).
    pub fn message_asked(&self) -> Option<Asked> {
        self.message.as_ref().map(|&(_, _, asked)| asked)
    }

    /// The options dialog's close applied at `at` (E17 M6 T4b1, *provisional*; the window asks
    /// every pass): Auto Combat boxes it changed join the battle's with the turn waiting and the
    /// time, counting for the turns to come; under Auto Combat a box set for the turn waiting
    /// hands it to the bot at once (the original's oddity 4, 0x4744C0, whose panel stays lit:
    /// here it dims as at a switch). Whether anything changed.
    pub fn boxes_closed(&mut self, at: u32) -> Result<bool, String> {
        if self.options.is_some() {
            return Ok(false);
        }
        let Some(before) = self.boxes_open.take() else {
            return Ok(false);
        };
        if before == self.settings.auto {
            return Ok(false);
        }
        let turn = self.played.timeline().turns.len();
        if self.boxes.is_empty() {
            self.boxes.push((0, 0, before));
        }
        self.boxes.push((turn, at, self.settings.auto));
        if !self.waiting() || !self.takes(at) {
            return Ok(false);
        }
        self.hand_over(turn, at, false)
    }

    /// Whether the battle takes Auto Combat's switch at all: fought here (not on a server, E17 M5
    /// T6b1, *provisional*: the M5 review list), not both sides human (0x46BB40). Its button is
    /// drawn disabled otherwise (T8b2, R5c-7).
    fn takes_auto(&self) -> bool {
        let both = self
            .turned
            .last()
            .is_some_and(|t| t.observation.sides.iter().all(|s| s.human));
        matches!(self.fight, Some(Fight::Here(_))) && !both
    }

    /// A key or a mouse button pressed at `at` while Auto Combat is on (E17 M5 T4c1): it stops as
    /// the next turn starts, the press doing nothing else; the stop joins the battle's. The
    /// index of the turn it stops at (none while it is off): the one after the turn started last
    /// by `at`, but never one the bot already played (T8b2, R5c-4: a press while Restart replays
    /// the battle stops it at its next turn to come). E17 M6 T5b2b (CLT-Q61): pressed during the
    /// computer's cast for the human's hero on a turn Auto Combat plays, it stops at the cast's
    /// end, that turn's (`Auto::casting`, `Auto::stop_comes`: only the press that made that stop,
    /// a press after a switch back on waiting as any); the stack's action is asked only once the
    /// clock reaches the cast's end (`Battle::cast_end`, the window's `Changes::after`), its
    /// stop read before it (`Battle::play_next`). A press the battle reads after a later action
    /// started (Restart's replay) counts from that action's start ([`stop_read`]), never for a
    /// cast already passed.
    pub fn stop_auto(&mut self, at: u32) -> Option<usize> {
        if !self.auto_on() {
            return None;
        }
        let (at, turn) = stop_read(self.played.timeline(), at);
        self.switches.push(Input::Stop { at, turn });
        Some(self.auto().stop_comes(turn))
    }

    /// A human's `command`, pressed at `at` while the battle waits for it (E17 M5 T2b): sent
    /// from the side to act's seat, the session validating it, its action starting at `at` or at
    /// the timeline's end if later; whether the battle took it (none waits, or the session
    /// rejected it: nothing, the turn waits on as a server's does; E17 M5 T8b2, R4b-4).
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
        let Some(played) = live.play(command)? else {
            return Ok(false);
        };
        self.grow_from_waiting(at, played)?;
        Ok(true)
    }

    /// A command `played` on the turn waiting at `at` (a human's, or Auto Combat's hand-over):
    /// its action starts at `at` or at the timeline's end if later.
    fn grow_from_waiting(&mut self, at: u32, played: crate::live::Played) -> Result<(), String> {
        let placeholder = Played::Finished(Box::default());
        if let Played::Waiting(growing, _) = std::mem::replace(&mut self.played, placeholder) {
            let at = at.max(growing.timeline().end);
            self.played = Played::Growing(growing);
            self.take(Some(at), played)?;
        }
        // The timeline shown was the one waiting: a playhead afresh.
        self.head = Playhead::new(self.timeline());
        self.follow()
    }

    /// A command `played` at `at` (none: the computer's, as soon as the last one ended): what
    /// the spectator was sent for it joins the record, its events the timeline (every side's when
    /// a human plays here), the battle at the turn after it the mouse's; the next command the
    /// computer's unless the battle is over or a human's comes next. Over, the timeline is
    /// finished; a human's next, it waits. E17 M6 T5b1: a command that starts no turn (a hero's
    /// cast) leaves the turn under way to go on.
    fn take(&mut self, at: Option<u32>, played: crate::live::Played) -> Result<(), String> {
        // Auto Combat plays the turn this command starts unless stopped by then (E17 M5 T4c1),
        // and its start asks the computer for a human's hero's cast (E17 M6 T5b1).
        let (auto, cast) = self.auto_turn(played.turned);
        let (Some(live), Played::Growing(growing)) =
            (Fight::here(self.fight.as_mut()), &mut self.played)
        else {
            return Ok(());
        };
        live.set_auto(auto);
        live.set_cast(cast);
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
        // Auto Combat's question come while another box was open (a held popup): opens once none
        // is, not before its own time (E17 M6 T5b2a's review: never a battle waiting on a box
        // not shown).
        if self.message.is_none()
            && Fight::here_ref(self.fight.as_ref())
                .and_then(Live::question)
                .is_some()
        {
            let end = self.timeline().end;
            return self.open_question(at.max(end));
        }
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
        let first = self.record.messages.len();
        self.record.messages.extend(new);
        // Each command's events grow it; the battle's start's, which the opening snapshot shows
        // already, do not (protocol v38: a seat that waited for the other one hears it then).
        let opening = record::opening_at(&self.record.messages);
        let mut pushed = 0;
        for (i, message) in self.record.messages.iter().enumerate().skip(first) {
            if let ServerMessage::Events { events, .. } = message
                && Some(i) != opening
            {
                let at = at.map(|at| at.max(growing.timeline().end));
                growing.push(&anim::Command { at, events }, false);
                pushed += 1;
            }
        }
        self.commands += pushed;
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
        let held = observation.stacks.iter().map(|s| s.id);
        let mut aims: Vec<_> = (observation.drawn)
            .zip(online.targets(None))
            .and_then(|(spell, targets)| {
                crate::aims::served(self.ground.previews(), (spell, 0, true), targets, held)
            })
            .into_iter()
            .collect();
        let legal = legal.to_vec();
        drop(online);
        let commands = self.record.commands().count();
        if self.served_events == Some(commands) {
            // The same turn served again (the seat dialed again, E17 M5 T6b2; a command the
            // server rejected): the hero's aim under way keeps its spell's (E17 M5 T8b2: else a
            // press cast nothing while the window aims).
            let again = self.turned.pop();
            if let (Some(again), Some((spell, false))) = (again, self.aiming) {
                let aimed = again.aims.into_iter();
                aims.extend(aimed.filter(|a| a.spell == spell && !a.creature));
            }
        }
        // The status line's damage at Creature Info's levels 1 and 2 (E17 M6 T4b2a, CLT-Q46,
        // *provisional*): a seat holds no battle state and the wire no preview yet, so its hints
        // keep the level-0 line. The hook: the server lane's previews of the turn's attacks and
        // shots (its inbox) go here, as `scenario::previews` gives them where the state is.
        let turn = crate::scenario::Turned {
            observation,
            commands: legal,
            aims,
            previews: Vec::new(),
        };
        self.follower.touch(self.turned.len());
        self.turned.push(turn);
        (self.served, self.served_events) = (index, Some(commands));
        true
    }

    /// Every command's event of the record, in order (the battle's start's, which its snapshot
    /// shows already, left out: `Record::events`).
    fn events(&self) -> Vec<h3_protocol::wire::Event> {
        self.record.events().cloned().collect()
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

    /// Whether `spell` (its original number `original`) is cast at a second target too
    /// (Teleport, Sacrifice; their aims not built): the battle's ruleset's answer (E17 M5 T8b2,
    /// R4b-4: a spell the asset tables do not know took the common aim, a cast the session
    /// rejects); by the number where it is not known (a server's game of other content).
    fn second_target(&self, spell: u16, original: Option<u16>) -> bool {
        match self.ground.previews() {
            Some(ruleset) => crate::aims::second_target(ruleset, spell),
            None => original.is_some_and(|o| SECOND_TARGET.contains(&o)),
        }
    }

    /// The side to act's hero's aim of `spell` now (E17 M5 T3b): the engine's; on a server from
    /// the server's targets, at the hero's mastery its own view shows (E17 M5 T6b1).
    fn hero_aim(&self, spell: u16) -> Option<mouse::aim::Aim> {
        match self.fight.as_ref()? {
            Fight::Here(live) => live.hero_aim(spell),
            Fight::Server(online) => {
                let own = self.own_view()?;
                let mastery = (own.own_hero.as_ref()?.spell_book.iter())
                    .find(|b| b.spell == spell)
                    .map_or(0, |b| b.mastery);
                let online = online.borrow();
                let targets = online.targets(Some(spell))?;
                let held = own.stacks.iter().map(|s| s.id);
                let aimed = (spell, mastery, false);
                crate::aims::served(self.ground.previews(), aimed, targets, held)
            }
        }
    }

    /// The outcome of a seat's battle on a server as its record shows it (E17 M5 T6b1): the
    /// game's end, every stack's count at the start and at the end ([`Battle::end_seen`]), its
    /// own hero's Eagle Eye picks from the battle as it ended (the record's last snapshot,
    /// protocol v38).
    fn served_outcome(&self) -> Option<h3_contracts::BattleOutcome> {
        let result = self.record.messages.iter().find_map(|m| match m {
            ServerMessage::GameEnded { result, .. } => Some(*result),
            _ => None,
        })?;
        let snapshot = self.record.first_snapshot()?;
        let ended = self.record.ended();
        Some(crate::results::served(
            snapshot,
            (self.end_seen()?, ended),
            result,
        ))
    }

    /// The battle's end as the record shows it once the battle is decided (E17 M5 T6b1:
    /// `h3_view::remote::seen` at the timeline's end): a replay of the whole record, so read once
    /// and kept (E17 M5 T8b2, R5b-1).
    fn end_seen(&self) -> Option<&Observation> {
        if let Some(end) = self.end_seen.get() {
            return Some(end);
        }
        let timeline = self.played.timeline();
        timeline.ended?;
        let snapshot = self.record.first_snapshot()?;
        let events = self.events();
        let end = h3_view::remote::seen(snapshot, &events, timeline, timeline.end, |_, _| 0);
        Some(self.end_seen.get_or_init(|| end))
    }

    /// Whether each side has army survivors at the battle's end (E17 M5 T8b2, R5b-3: a side whose
    /// opponent gave up wins only with some, BAT-END-001), read once the battle is decided: by the
    /// engine's outcome of a battle fought here, else by the record's end.
    fn survivors(&self) -> Option<[bool; 2]> {
        if let Some(&found) = self.survivors.get() {
            return Some(found);
        }
        let found = match self.fight.as_ref() {
            Some(Fight::Here(live)) => crate::results::survivors(&live.outcome()?.0.stacks),
            _ => {
                let opening = self.record.first_snapshot()?;
                crate::results::survivors(&crate::results::records(opening, self.end_seen()?))
            }
        };
        Some(*self.survivors.get_or_init(|| found))
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
        let itself = self.itself(false);
        let words = (&ground.layout, &name as _, Bugs::Fixed);
        (self.follower).shown(timeline, (inputs, &self.switches, &itself), words)
    }

    /// The mouse, the timeline shown and the panel after the battle grew (or as it starts): the
    /// playhead goes on over what was played, unless the mouse's fidgets go in afresh.
    fn follow(&mut self) -> Result<(), String> {
        let (from, tail) = self.pointer(None, true);
        self.shown.splice(from, tail);
        if self.shown.entered.is_empty() && self.entered.is_none() {
            self.head.extend(self.played.timeline());
        } else {
            // The playhead carried on from the timeline it showed (E17 M5 T8b2, R3-1's rest):
            // the played one grown, then the mouse's fidgets in it.
            let old = self.entered.take();
            if old.is_none() {
                self.head.extend(self.played.timeline());
            }
            self.entered = self.entered(&self.shown);
            let played = self.played.timeline();
            let old = old.as_ref().unwrap_or(played);
            self.head
                .carry(old, self.entered.as_ref().unwrap_or(played));
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
            // The playhead carried on, not played again from the start (E17 M5 T8b2, R3-1's rest).
            let old = self.entered.take();
            self.entered = self.entered(&self.shown);
            let played = self.played.timeline();
            let old = old.as_ref().unwrap_or(played);
            self.head
                .carry(old, self.entered.as_ref().unwrap_or(played));
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
    /// the mouse moved (else the timeline's own), Auto Combat as the window switched it (E17 M5
    /// T4c1), its button disabled on a server's seat (T8b2, R5c-7: the window takes no switch
    /// there, review item 55, as the original's when both players are human).
    fn panel(&self) -> Result<Panels, String> {
        let mut panels = self.logged_panel()?;
        if let Some(snapshot) = self.record.first_snapshot() {
            panels = panels.with_auto(snapshot, self.auto(), Bugs::Fixed);
        }
        if matches!(self.fight, Some(Fight::Server(_))) {
            panels = panels.without_switch();
        }
        Ok(panels)
    }

    /// [`Battle::panel`] but Auto Combat.
    fn logged_panel(&self) -> Result<Panels, String> {
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
        writer.panels(&self.record, (&logged, &self.played.timeline().controllers))
    }

    /// The cursor at `t`: the stats dialog's or the spell book's once it set one (E17 M5 T3a, M6
    /// T2a2a), else the combat cursor's frame (the panel's pointer before the mouse is anywhere).
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
        // A message box sets the default cursor's frame 0 (0x4F74AC; E17 M5 T4a), the combat
        // options dialog its type 0 frame 0 (E17 M6 T4a), which stays after its close until the
        // mouse moves (oddity 10).
        if self.message.is_some() || self.options.is_some() || self.options_arrow {
            let book = &self.ground.layout.book;
            return CursorLook {
                def: self.ground.layout.message.cursor.clone(),
                frame: 0,
                hot: hot_of(&book.hot, 0),
            };
        }
        // The stats dialog's once its moves set one (E17 M6 T2a2a).
        if let Some(frame) = self.stats.as_ref().and_then(|(open, _)| open.modal.cursor) {
            let book = &self.ground.layout.book;
            return CursorLook {
                def: book.cursor.clone(),
                frame,
                hot: hot_of(&book.hot, frame),
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
            // The book's arrow a wall's aim keeps in the original's profile (E17 M6 T4c).
            Some(mouse::track::Look::Arrow) => CursorLook {
                def: self.ground.layout.message.cursor.clone(),
                frame: 0,
                hot: hot_of(&self.ground.layout.book.hot, 0),
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
    /// panel's button takes. Whether it opened, or a box told why not (E17 M6 T5a, design
    /// §12.17: an anti-magic garrison, the Orb of Inhibition; [`crate::boxes::before_book`]).
    pub fn open_book(&mut self, at: u32) -> Opened {
        if self.book.is_some() || !self.takes(at) {
            return Opened::No;
        }
        let ground = self.ground;
        // The side to act's own view: its hero's spells (as its seat is sent them).
        let (Some(observation), Some(ruleset), Some(writer)) =
            (self.own_view(), ground.rules(), ground.writer.as_ref())
        else {
            return Opened::No;
        };
        if !crate::spellbook::takes(&observation) {
            return Opened::No;
        }
        let refused =
            (self.facts()).and_then(|facts| crate::boxes::before_book(&facts, &writer.catalogue));
        if let Some(text) = refused {
            return if self.tell(at, &text, (Style::Ok, None), Asked::Refused) {
                Opened::Refused
            } else {
                Opened::No
            };
        }
        let names = (&ground.tables, &writer.catalogue);
        let manifest = &self.record.header.manifest;
        let Some(hero) = crate::spellbook::hero(
            &observation,
            (manifest, ruleset),
            names,
            &ground.layout.book,
        ) else {
            return Opened::No;
        };
        let open = h3_view::book::Open::new(&mut self.book_memory, hero, &writer.catalogue);
        // The hero's player's colours (the record names none: red, blue; CLT-Q6).
        let index = usize::from(observation.viewer == Side::Defender);
        let player = observation.sides[index]
            .colour
            .unwrap_or_else(|| u8::try_from(index).unwrap_or(0));
        self.book = Some((open, at, player));
        Opened::Book
    }

    /// What the book's and the aim's refusals read now (E17 M6 T5a, design §12.17): the side to
    /// act's own view and the battle's opening; each hero's magic from both sides' views for a
    /// battle fought here, its own alone for a seat on a server (gap G1).
    fn facts(&self) -> Option<crate::boxes::Facts> {
        let own = self.own_view()?;
        let first = self.record.first_snapshot()?;
        let magic = match self.fight.as_ref()? {
            Fight::Here(live) => live.magic(),
            // The seat's own hero's alone (gap G1, CLT-STUB-1).
            Fight::Server(_) => crate::boxes::seat_magic(&own),
        };
        let writer = self.ground.writer.as_ref()?;
        let names = (
            &self.record.header.manifest,
            &self.ground.tables,
            &writer.catalogue,
        );
        let [attacker, defender] = &magic;
        Some(crate::boxes::Facts::of(
            &own,
            first,
            [attacker.as_ref(), defender.as_ref()],
            self.ground.previews(),
            names,
        ))
    }

    /// The box the aim's refusal of `spell` (its original number `original`) shows, the engine
    /// giving it no target and no cast without one (E17 M6 T5a, design §12.17;
    /// [`crate::boxes::after_book`] in the client's profile); none: no box.
    fn refusal(&self, spell: u16, original: Option<u16>) -> Option<String> {
        let original = u32::from(original?);
        let level = self.ground.rules()?.spells().get(usize::from(spell))?.level;
        let facts = self.facts()?;
        let catalogue = &self.ground.writer.as_ref()?.catalogue;
        crate::boxes::after_book(&facts, (original, level), Bugs::Fixed, catalogue)
    }

    /// Force Field's or Fire Wall's aim of `spell` with no free hex (E17 M6 T5a, design §12.17:
    /// it opens all the same, every hex refused), at the hero's mastery its own view shows.
    fn nowhere(&self, spell: u16) -> Option<mouse::aim::Aim> {
        let own = self.own_view()?;
        let mastery = (own.own_hero.as_ref()?.spell_book.iter())
            .find(|b| b.spell == spell)
            .map_or(0, |b| b.mastery);
        Some(crate::aims::nowhere(self.ground.previews(), spell, mastery))
    }

    /// The open book's answer to `outcome` (E17 M5 T3a): closed, or closed on a spell, whose cast
    /// is given when the engine takes it without a target (one that takes a target is aimed first:
    /// T3b's); a refusal or a description shows its message box over the book, a spell the
    /// engine refuses its refusal's box over the battle (E17 M5 T4a: a stack spell no hex takes,
    /// GENRLTXT 186; E17 M6 T5a, design §12.17: the others, [`BookDid::Refused`]).
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
                // built); a spell the engine refuses tells why as the original's book button and
                // aim's entry do (E17 M5 T4a: a stack spell or Teleport no hex takes, GENRLTXT
                // 186; E17 M6 T5a, design §12.17: the others), the move posted after the box's OK
                // as after a book closed without a spell; a wall with no free hex opens its aim
                // all the same.
                let keys = &self.record.header.manifest.keys.spells;
                let original =
                    motion::spell_entry_in(keys, &self.ground.tables, spell).map(|e| e.original);
                let second = self.second_target(spell, original);
                let mut aim = self.hero_aim(spell);
                if aim.is_none() {
                    if let Some(text) = self.refusal(spell, original) {
                        let told = self.tell(opened, &text, (Style::Ok, None), Asked::Refused);
                        return if told {
                            BookDid::Refused
                        } else {
                            BookDid::Changed
                        };
                    }
                    if original.is_some_and(h3_view::shades::wall) {
                        aim = self.nowhere(spell);
                    }
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

    /// The original's number of the spell being aimed (E17 M6 T4c: whether its aim turns the
    /// movement shadow off, `h3_view::shades::Options::aiming`; 0 for one the tables lack).
    pub fn aim_original(&self) -> Option<u16> {
        let (spell, creature) = self.aiming?;
        let aims = self.turned.last().map(|t| &t.aims);
        let aim = aims.and_then(|a| {
            a.iter()
                .find(|a| a.spell == spell && a.creature == creature)
        });
        Some(aim.map_or(0, |a| a.original))
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

    /// Whether a stack's stats dialog is open over the battle (E17 M6 T2a2a).
    pub fn stats_open(&self) -> bool {
        self.stats.is_some()
    }

    /// Opens a stack's stats dialog at `at` with the mouse at `point` (E17 M6 T2a2a;
    /// `stack_stats_dialog` 0x468860, modal; design §12.2): of the stack a left press there looks
    /// at (`battle_click_action`'s kind 5, 0x476EDC: one of the side to act's own, any in the
    /// tactics phase), or, `acting`, of the acting stack (the key T, 0x475DB6; not in the tactics
    /// phase); while the battle waits for a human here, from the side to act's own view. Whether
    /// it opened (not without the text tables, nor in a server's game of other content).
    pub fn look(&mut self, at: u32, point: [i32; 2], acting: bool) -> bool {
        if self.stats.is_some() || !self.takes(at) {
            return false;
        }
        let ground = self.ground;
        let id = if acting {
            self.turned
                .last()
                .map(|t| &t.observation)
                .filter(|o| o.tactics.is_none())
                .and_then(|o| o.active)
        } else {
            self.taken(at, |field| mouse::looked_at(field, &ground.layout, point))
        };
        self.open_stats(at, (id, point), false)
    }

    /// Shows the stats dialog of the stack on the hex at `point` while the right button is held,
    /// pressed at `at` (E17 M6 T2b1; `battle_right_click` 0x4769C0, `stack_stats_dialog` with its
    /// second argument 1; design §12.5): any living stack there, either side's, from the side to
    /// act's own view, without the OK and the cast button, its creature standing, the cursor
    /// hidden; its ends close it ([`Self::stats_held`]). Whether it opened (as [`Self::look`]).
    pub fn look_held(&mut self, at: u32, point: [i32; 2]) -> bool {
        if self.stats.is_some() || !self.takes(at) {
            return false;
        }
        let ground = self.ground;
        let id = self.taken(at, |field| mouse::held_at(field, &ground.layout, point));
        self.open_stats(at, (id, point), true)
    }

    /// Opens a siege turret's info at `at` with the mouse at `point` (E17 M6 T2b2; `turret_info`
    /// 0x47A380, design §12.6): by a left press where the cursor is a turret's, a box with its OK
    /// ([`Asked::Turret`]: its close as a stats dialog's); by the `right` button over the keep or
    /// a Castle's towers, a popup held while the button is ([`Asked::Held`]); framed as a box
    /// ([`Self::box_player`]), its counts the turn's observation's. Whether it opened (as
    /// [`Self::look`]; not without the text tables).
    pub fn turret_info(&mut self, at: u32, point: [i32; 2], right: bool) -> bool {
        if self.stats.is_some()
            || self.message.is_some()
            || self.book.is_some()
            || self.aiming.is_some()
            || !self.takes(at)
        {
            return false;
        }
        let ground = self.ground;
        let Some(writer) = ground.writer.as_ref() else {
            return false;
        };
        let factions = &self.record.header.manifest.keys.factions;
        let part = |faction: u16, part: usize| {
            let key = factions.get(usize::from(faction))?;
            Some(format!("wall.{key}.{part}"))
        };
        let text = self.taken(at, |field| {
            mouse::turret_at(field, &ground.layout, point, right)
                .then(|| h3_view::turret::text(field.observation, &writer.catalogue, &part))
                .flatten()
        });
        let Some(text) = text else {
            return false;
        };
        let shown = if right {
            (Style::Popup, Asked::Held)
        } else {
            (Style::Ok, Asked::Turret)
        };
        self.tell(at, &text, (shown.0, None), shown.1)
    }

    /// Shows the help of the panel's control at `point` while the right button pressed at `at` is
    /// held (E17 M6 T3a, design §12.7; `battle_control_help` 0x472A50): the control's right-click
    /// text in a popup centred on the screen ([`Asked::Held`], as a turret's info by the right
    /// button), a disabled control's too; nothing over the panel's picture or off the panel.
    /// Whether it opened (not without the text tables).
    pub fn control_help(&mut self, at: u32, point: [i32; 2]) -> bool {
        if self.stats.is_some()
            || self.message.is_some()
            || self.book.is_some()
            || self.aiming.is_some()
            || !self.takes(at)
        {
            return false;
        }
        let Some(key) = h3_view::panel::help_at(self.panel_at(at), point) else {
            return false;
        };
        let Some(writer) = self.ground.writer.as_ref() else {
            return false;
        };
        let text = writer.catalogue.text(key, &h3_ui::text::Args::new());
        self.tell(at, &text, (Style::Popup, None), Asked::Held)
    }

    /// Shows a side's hero's info window while the right button pressed at `at` over the hero's
    /// rectangle at `point` is held (E17 M6 T3b1, design §12.8; `battle_right_click` 0x4769C0's
    /// "hexes" 0xFC and 0xFD): at the side's place, the hero as the turn's observation shows it
    /// ([`Self::hero_window`]), held as the right button's popups ([`Asked::Held`]: the cursor
    /// hidden, every input but its ends dropped, nothing posted after); nothing over a side
    /// without a hero. The side whose window opened (none without the text tables): the window
    /// gives the mouse its `Input::HeroInfo` (E17 M6 T4b2b1: at Creature Info's levels 1 and 2
    /// the window stays shown after the release). The client's profile shows it even when it is
    /// shown already at level 0 (CLT-Q50, fixed: `Windows::held_shows`).
    pub fn hero_info(&mut self, at: u32, point: [i32; 2]) -> Option<Side> {
        if self.stats.is_some()
            || self.message.is_some()
            || self.book.is_some()
            || self.aiming.is_some()
            || !self.takes(at)
        {
            return None;
        }
        let layout = &self.ground.layout;
        let side = self.taken(at, |field| {
            h3_view::hero_info::at(layout, field.observation, point)
        })?;
        let observation = &self.turned.last()?.observation;
        let window = self.hero_window(observation, side)?;
        self.opened = self.opened.wrapping_add(1);
        self.message = Some((h3_view::message::Open::new(window), at, Asked::Held));
        Some(side)
    }

    /// Side `side`'s hero's info window as `observation`'s battle fills it (E17 M6 T3b1,
    /// design §12.8: `army_info_hero_set` 0x46CC10; the right press's and, E17 M6 T4b2b1, the
    /// hover's): none without a hero there or the text tables.
    fn hero_window(&self, observation: &Observation, side: Side) -> Option<MessageBox> {
        let ground = self.ground;
        let writer = ground.writer.as_ref()?;
        let manifest = &self.record.header.manifest;
        let keys = &manifest.keys.hero_types;
        let portrait = |hero_type: u16| crate::results::portrait(keys, &ground.tables, hero_type);
        let special = crate::stats::field(manifest, observation);
        let info = h3_view::hero_info::Info::of(observation, side, special, &portrait)?;
        Some(h3_view::hero_info::window(
            &ground.layout.hero_info,
            &writer.catalogue,
            side,
            &info,
        ))
    }

    /// A stack's army info window `shown` as `observation`'s battle fills it (E17 M6 T4b2b2,
    /// design §12.15: `army_info_stack_set` 0x46DC30, the stats dialog's numbers), in the
    /// client's profile (CLT-Q48: a Ballista's Dmg the dialog's): none without the stack there,
    /// the rules or the text tables (*provisional*: built anew at every frame, as the heroes').
    fn stack_window(
        &self,
        observation: &Observation,
        shown: h3_view::army_info::StackShown,
    ) -> Option<MessageBox> {
        let ground = self.ground;
        let (ruleset, writer) = (ground.previews()?, ground.writer.as_ref()?);
        let sources = crate::stats::Sources {
            manifest: &self.record.header.manifest,
            ruleset,
            tables: &ground.tables,
            catalogue: &writer.catalogue,
            towns: ground.towns(),
        };
        let viewing = self.viewing_player();
        let info = crate::stats::army_info(&sources, observation, shown.stack, viewing)?;
        Some(h3_view::army_info::stack_window(
            &ground.layout.army_info,
            &writer.catalogue,
            (shown.side, shown.kind),
            &info,
            Bugs::Fixed,
        ))
    }

    /// Whether the combat options dialog is open over the battle (E17 M6 T4a).
    pub fn options_open(&self) -> bool {
        self.options.is_some()
    }

    /// Opens the combat options dialog at `at` (E17 M6 T4a, design §12.12; the panel's Options
    /// 2003 or its key O, `battle_combat_options` 0x4686B0): only while the battle takes a command
    /// (`battle_buttons`' +0x132B4: a battle fought here whose human's turn waits, its last
    /// action played), nothing else open over it; its background in the acting side's player's
    /// colours ([`Self::box_player`]), its settings the battle's. Whether it opened (not without
    /// the text tables).
    pub fn open_options(&mut self, at: u32) -> bool {
        if self.stats.is_some()
            || self.message.is_some()
            || self.book.is_some()
            || self.aiming.is_some()
            || self.options.is_some()
            || self.fight.is_none()
            || !self.takes(at)
        {
            return false;
        }
        let player = self.box_player(at, None);
        let Some(writer) = self.ground.writer.as_ref() else {
            return false;
        };
        let open = h3_view::options::Open::new(
            &self.ground.layout.options,
            &writer.catalogue,
            self.settings,
            (player, self.audio),
        );
        self.options = Some((open, at));
        self.boxes_open.get_or_insert(self.settings.auto);
        true
    }

    /// A left press at `point` while the options dialog is open.
    pub fn options_press(&mut self, point: [i32; 2]) -> OptionsDid {
        self.options_did(|o| o.press(point))
    }

    /// The mouse moved to `point` while the options dialog is open: the button held follows it.
    pub fn options_moved(&mut self, point: [i32; 2]) -> OptionsDid {
        self.options_did(|o| o.moved(point))
    }

    /// The left button released while the options dialog is open.
    pub fn options_release(&mut self) -> OptionsDid {
        self.options_did(h3_view::options::Open::release)
    }

    /// The OK's key down while the options dialog is open (`by` the key, whose release clicks).
    pub fn options_key(&mut self, by: &str) -> OptionsDid {
        self.options_did(|o| o.key(by))
    }

    /// A key up while the options dialog is open.
    pub fn options_key_up(&mut self, by: &str) -> OptionsDid {
        self.options_did(|o| o.key_up(by))
    }

    /// The right button pressed at `point` while the options dialog is open (design §12.12): the
    /// help of the control there in a popup centred on the screen while the button is held
    /// ([`Asked::Held`]); nothing over the picture or a text. Whether it opened.
    pub fn options_right(&mut self, point: [i32; 2]) -> bool {
        let Some((open, opened)) = self.options.as_ref() else {
            return false;
        };
        let (Some(key), Some(writer)) = (open.help_at(point), self.ground.writer.as_ref()) else {
            return false;
        };
        let text = writer.catalogue.text(&key, &h3_ui::text::Args::new());
        let opened = *opened;
        self.tell(opened, &text, (Style::Popup, None), Asked::Held)
    }

    /// The mouse moved over the battle: the options dialog's arrow left at its close goes.
    pub fn mouse_moved(&mut self) {
        self.options_arrow = false;
    }

    /// The settings to save once the options dialog closed after a click (+0x4C; 0x46F7A2),
    /// taken once.
    pub fn options_saved(&mut self) -> Option<h3_view::options::Settings> {
        core::mem::take(&mut self.options_save).then_some(self.settings)
    }

    /// The open options dialog's answer to an input (`combat_options_proc` 0x46F7B0): a bar at
    /// volume 0 without the audio device opens the no-sound box (GENRLTXT 151, an OK box); the
    /// OK's click closes it, the battle taking its settings, its speed from the next command on
    /// (the redraw: the window's), the arrow staying until the mouse moves.
    fn options_did(
        &mut self,
        input: impl FnOnce(&mut h3_view::options::Open) -> h3_view::options::Did,
    ) -> OptionsDid {
        let Some((open, opened)) = self.options.as_mut() else {
            return OptionsDid::default();
        };
        let did = input(open);
        let (settings, clicked, opened) = (open.settings, open.clicked, *opened);
        self.settings = settings;
        if did.no_sound
            && let Some(writer) = self.ground.writer.as_ref()
        {
            let text = writer
                .catalogue
                .text(h3_view::options::NO_SOUND, &h3_ui::text::Args::new());
            self.tell(opened, &text, (Style::Ok, None), Asked::Told);
        }
        if did.closed {
            self.options = None;
            self.options_arrow = true;
            self.options_save = clicked;
            self.set_speed(Speed::new(settings.speed));
        }
        did
    }

    /// Whether the right button's popup over the battle is held (E17 M6 T2b2: a turret's info; T3a:
    /// a control's help; `popup_run_held` 0x602A40: the cursor hidden, every input but its ends
    /// dropped).
    pub fn popup_held(&self) -> bool {
        self.message
            .as_ref()
            .is_some_and(|(_, _, asked)| *asked == Asked::Held)
    }

    /// The held popup's end (the right button's release, a left press or release; E17 M6 T2b2):
    /// it closes and the battle goes on behind it, its cursor shown again as it was (the right
    /// button's path posts nothing, 0x476BBF). Whether one closed.
    pub fn popup_end(&mut self) -> bool {
        if !self.popup_held() {
            return false;
        }
        self.message = None;
        true
    }

    /// Opens the stats dialog of stack `id` at `at`, the mouse at `point`: modal, or by the
    /// `right` button, held.
    fn open_stats(&mut self, at: u32, (id, point): (Option<u8>, [i32; 2]), right: bool) -> bool {
        let ground = self.ground;
        let (Some(id), Some(observation)) = (id, self.own_view()) else {
            return false;
        };
        let manifest = &self.record.header.manifest;
        let tables = &ground.tables;
        let def = observation
            .stacks
            .iter()
            .find(|s| s.id == id)
            .and_then(|s| manifest.keys.creatures.get(usize::from(s.creature)))
            .and_then(|key| tables.creatures.get(key))
            .map(|e| e.def.to_ascii_uppercase());
        // The creature's DEF's groups, for its steps: the battle drew it, so the batch holds it.
        if let Some(def) = def.as_ref().filter(|d| !held(&self.batch, d))
            && self.load_names(vec![def.clone()]).is_err()
        {
            return false;
        }
        let frames = def
            .and_then(|d| self.batch.sprite(&d))
            .map(|s| {
                s.def
                    .groups
                    .iter()
                    .map(|g| (g.id, u32::try_from(g.frames.len()).unwrap_or(0)))
                    .collect()
            })
            .unwrap_or_default();
        // Morale's and luck's descriptions' pictures (E17 M6 T2a2b2): loaded for their sizes
        // and for the popup to draw them.
        let described = [
            ground.layout.stats.morale_described.clone(),
            ground.layout.stats.luck_described.clone(),
        ];
        let unheld: Vec<String> = described
            .iter()
            .map(|d| d.to_ascii_uppercase())
            .filter(|d| !held(&self.batch, d))
            .collect();
        if !unheld.is_empty() && self.load_names(unheld).is_err() {
            return false;
        }
        let sizes = described.map(|d| {
            self.batch.sprite(&d).map_or([0, 0], |s| {
                [s.def.width, s.def.height].map(|v| i32::try_from(v).unwrap_or(i32::MAX))
            })
        });
        let (Some(ruleset), Some(writer)) = (ground.previews(), ground.writer.as_ref()) else {
            return false;
        };
        let Some(stack) = observation.stacks.iter().find(|s| s.id == id) else {
            return false;
        };
        let anchor = ground.layout.anchor(stack.hex / 17, stack.hex % 17);
        let sources = crate::stats::Sources {
            manifest: &self.record.header.manifest,
            ruleset,
            tables: &ground.tables,
            catalogue: &writer.catalogue,
            towns: ground.towns(),
        };
        // The viewer's own generator for the creature's later groups (§7.2), seeded by the time.
        let seed = at.wrapping_mul(2_654_435_761) | 1;
        let viewing = self.viewing_player();
        let open = crate::stats::open(
            &sources,
            &observation,
            (id, point, right),
            (&ground.layout.stats, anchor, sizes),
            (frames, viewing, seed),
            &mut self.stats_memory,
        );
        let Some(open) = open else {
            return false;
        };
        self.stats = Some((open, at));
        true
    }

    /// The mouse moved to `point` while the stats dialog is open: whether its status line,
    /// cursor or OK changed.
    pub fn stats_moved(&mut self, point: [i32; 2]) -> bool {
        let Some((open, _)) = self.stats.as_mut() else {
            return false;
        };
        let hovered = open.modal.hover(&mut self.stats_memory, point);
        let moved = open.modal.moved(point) != h3_view::message::Outcome::Nothing;
        hovered || moved
    }

    /// A left press at `point` while the stats dialog is open: its OK goes down.
    pub fn stats_press(&mut self, point: [i32; 2]) -> StatsDid {
        self.stats_did(|m| m.press(point))
    }

    /// The left button released while the stats dialog is open: the OK held clicks.
    pub fn stats_release(&mut self) -> StatsDid {
        self.stats_did(h3_view::stats::Modal::release)
    }

    /// The OK's key down while the stats dialog is open (`by` the key, whose release clicks).
    pub fn stats_key(&mut self, by: &str) -> StatsDid {
        self.stats_did(|m| m.key(by))
    }

    /// A key up while the stats dialog is open.
    pub fn stats_key_up(&mut self, by: &str) -> StatsDid {
        self.stats_did(|m| m.key_up(by))
    }

    fn stats_did(
        &mut self,
        input: impl FnOnce(&mut h3_view::stats::Modal) -> h3_view::message::Outcome,
    ) -> StatsDid {
        use h3_view::message::Outcome;
        let Some((open, _)) = self.stats.as_mut() else {
            return StatsDid::Nothing;
        };
        match input(&mut open.modal) {
            Outcome::Nothing => StatsDid::Nothing,
            Outcome::Changed => StatsDid::Changed,
            Outcome::Closed(id) => {
                self.stats = None;
                if id == h3_view::stats::CAST {
                    StatsDid::Cast
                } else {
                    StatsDid::Closed
                }
            }
        }
    }

    /// The right button pressed at `at` with the mouse at `point` while the stats dialog is open
    /// (E17 M6 T2a2b1; `creature_dialog_proc` 0x5F48AD, design §12.3): the description of the
    /// item there in a popup (style 4) held while the button is, framed as a box ([`Self::box_player`]).
    /// Whether it opened (none over an item without one, nor without the text tables).
    pub fn stats_right(&mut self, at: u32, point: [i32; 2]) -> bool {
        let player = self.box_player(at, None);
        let Some((open, _)) = self.stats.as_ref() else {
            return false;
        };
        if open.modal.popup.is_some() {
            return false;
        }
        let (Some((text, picture)), Some(writer)) =
            (open.modal.describe(point), self.ground.writer.as_ref())
        else {
            return false;
        };
        let Some(window) = writer.message(
            &self.ground.layout,
            &text,
            Style::Popup,
            picture.as_ref(),
            player,
        ) else {
            return false;
        };
        let Some((open, opened)) = self.stats.as_mut() else {
            return false;
        };
        open.modal.popup = Some(window);
        open.holds.push([at.saturating_sub(*opened), u32::MAX]);
        true
    }

    /// Whether a popup is held (`popup_run_held` 0x602A40, `mouse_hide`: the cursor hidden,
    /// every input but its ends dropped): the stats dialog's description (E17 M6 T2a2b1), or the
    /// stats dialog the right button shows over the battle (E17 M6 T2b1).
    pub fn stats_popup(&self) -> bool {
        self.stats
            .as_ref()
            .is_some_and(|(open, _)| open.held || open.modal.popup.is_some())
    }

    /// An input at `at` while a popup is held ([`h3_view::stats::Modal::held_input`]): `ends`
    /// it (the right button's release, a left press or release), a key's release `key_up`, a left
    /// release `left_release`. The right button's dialog over the battle closes at its end
    /// (E17 M6 T2b1: then as the modal one, 0x476B1A, [`StatsDid::Closed`]).
    pub fn stats_held(
        &mut self,
        at: u32,
        ends: bool,
        key_up: Option<&str>,
        left_release: bool,
    ) -> StatsDid {
        let Some((open, opened)) = self.stats.as_mut() else {
            return StatsDid::Nothing;
        };
        if open.held {
            if ends {
                self.stats = None;
                return StatsDid::Closed;
            }
            return StatsDid::Nothing;
        }
        let held = open.modal.popup.is_some();
        let changed = open.modal.held_input(ends, key_up, left_release);
        if held
            && open.modal.popup.is_none()
            && let Some(hold) = open.holds.last_mut()
        {
            hold[1] = at.saturating_sub(*opened);
        }
        if changed {
            StatsDid::Changed
        } else {
            StatsDid::Nothing
        }
    }

    /// The stats dialog's cast button clicked at `at` (E17 M6 T2a2b1; `stack_stats_dialog`
    /// 0x468943): the acting Faerie Dragon's drawn spell aimed as by its key F (0x468954 into
    /// `enter_player_cast`, the action 10), the dialog's close coming as the aim ends
    /// ([`Self::looked_after_aim`]). The spell aimed, if any (none: its side's hero cast this
    /// round, and the dialog's close comes at once).
    pub fn look_cast(&mut self, at: u32) -> Option<u16> {
        let spell = self.dragon_aim(at)?;
        self.look_aim = true;
        Some(spell)
    }

    /// Whether the aim that just ended was the cast button's (E17 M6 T2a2b1): the stats dialog's
    /// close follows it (0x476EE1 after `stack_stats_dialog` returns).
    pub fn looked_after_aim(&mut self) -> bool {
        core::mem::take(&mut self.look_aim)
    }

    /// When the open stats dialog's creature next steps after `t` (none: no dialog open, or its
    /// popup held).
    pub fn stats_step_after(&self, t: u32) -> Option<u32> {
        let (open, opened) = self.stats.as_ref()?;
        let since = t.saturating_sub(*opened);
        let next = h3_view::stats::Steps::next_around(STATS_FIRST_STEP, &open.holds, since)?;
        Some(opened.saturating_add(next))
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
        match self.ground.humans {
            [true, _] => self.side_colour(0),
            [false, true] => self.side_colour(1),
            [false, false] => 0,
        }
    }

    /// Side `index`'s player colour (the record's first snapshot; none named: red for the
    /// attacker, blue for the defender, CLT-Q6).
    fn side_colour(&self, index: usize) -> u8 {
        self.record
            .first_snapshot()
            .and_then(|s| s.sides.get(index))
            .and_then(|side| side.colour)
            .unwrap_or_else(|| u8::try_from(index).unwrap_or(0))
    }

    /// The colours of a box opened at `at` (CLT-Q28 (2), answered (c) at E17 M5 T8c;
    /// `h3_view::message::framed`): the acting side's player, `side` when given (the tactics
    /// phase's, before any turn), else the controller of the turn `at` falls in as the panel's
    /// (`Timeline::controllers`), before the first turn the timeline holds the opening's active
    /// stack's (E17 M6 T2b2: its turn starts before the record opens, as the panel's
    /// `Panels::new`); the viewing player's without any.
    fn box_player(&self, at: u32, side: Option<Side>) -> u8 {
        let timeline = self.timeline();
        let acting = side.or_else(
            || match timeline.turns.iter().rposition(|&(t, _)| t <= at) {
                Some(turn) => timeline.controllers.get(turn).copied(),
                None => {
                    let opening = self.record.first_snapshot()?;
                    let id = opening.active?;
                    opening
                        .stacks
                        .iter()
                        .find(|s| s.id == id)
                        .map(|s| s.controller)
                }
            },
        );
        let acting = acting.map(|s| self.side_colour(usize::from(s == Side::Defender)));
        h3_view::message::framed(Bugs::Fixed, self.viewing_player(), acting)
    }

    /// Opens a message box of `style` showing `text` and `picture` (E17 M5 T4a), the battle
    /// standing still behind it from `at`; what its OK does is `asked`. Whether it opened (not
    /// without the text tables). Framed in the acting side's colours ([`Self::box_player`]).
    fn tell(
        &mut self,
        at: u32,
        text: &str,
        shown: (Style, Option<&Picture>),
        asked: Asked,
    ) -> bool {
        self.tell_framed((at, None), text, shown, asked)
    }

    /// [`Self::tell`], the acting side `side` when given.
    fn tell_framed(
        &mut self,
        (at, side): (u32, Option<Side>),
        text: &str,
        (style, picture): (Style, Option<&Picture>),
        asked: Asked,
    ) -> bool {
        let player = self.box_player(at, side);
        let Some(writer) = self.ground.writer.as_ref() else {
            return false;
        };
        let Some(message) = writer.message(&self.ground.layout, text, style, picture, player)
        else {
            return false;
        };
        self.opened = self.opened.wrapping_add(1);
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
        let taken = self.message.take();
        // The tactics phase's message closed: the first turn starts (E17 M5 T4c2a).
        self.opening = false;
        // The result window's OK, then each box's of Eagle Eye's message (E17 M6 T5c, design
        // §12.21): the next box opens, its pickup sound starting, over the screen darkened and
        // the window as it showed, the battle standing at its end; the last one's ends the
        // battle's screen. The experience given after the window shows nothing here (§12.21).
        let after_window = matches!(asked, Asked::Result | Asked::EagleEye);
        if after_window && !self.eagle_eye.is_empty() {
            if let Some((window, opened, Asked::Result)) = taken {
                let window = crate::results::released(&window.message);
                self.window_behind = Some((window, opened, None));
            }
            let (next, pickup) = self.eagle_eye.remove(0);
            self.opened = self.opened.wrapping_add(1);
            self.message = Some((h3_view::message::Open::new(next), at, Asked::EagleEye));
            self.pickup = Some(pickup);
            return MessageDid::Changed;
        }
        self.result_closed |= after_window;
        self.window_behind = None;
        if let Some(did) = closed(asked) {
            return did;
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
            Answered::Auto(then) => MessageDid::Answer(then.ok()),
        }
    }

    /// Opens the battle's result window once `at` reaches the battle's end (E17 M5 T4b1; design
    /// §7.20): as `battle_end` 0x477470 opens it as the battle is decided, over the whole screen
    /// darkened, the battle standing as it ended; from the engine's outcome of the battle fought
    /// here (none for a record file's). Whether it opened. Built once (E17 M5 T8b2, R5b-1): it
    /// waits only for the outcome (a server's game's end), then always opens.
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
                Some((outcome, ruleset)) => (outcome, Some(ruleset)),
                None => return false,
            },
            // A seat on a server: the outcome as its record shows it (E17 M5 T6b1), the
            // creatures' values by the ruleset compiled here only when the content check finds it
            // the game's (E17 M5 T8b2, R5b-2; else none, `crate::results::window`).
            Fight::Server(_) => match self.served_outcome() {
                Some(outcome) => (outcome, self.ground.previews()),
                None => return false,
            },
        };
        let sources = crate::results::Sources {
            manifest: &self.record.header.manifest,
            tables: &self.ground.tables,
            ruleset,
            catalogue: &writer.catalogue,
            layout: &self.ground.layout,
            bugs: Bugs::Fixed,
        };
        let player = self.viewing_player();
        let (window, _) = crate::results::window(&sources, opening, &outcome, player);
        // Eagle Eye's message after it (E17 M6 T5c), the pictures' size once their DEF is read.
        let def = self.ground.layout.message.spell_pictures.clone();
        let told = crate::results::eagle_eye(&sources, opening, &outcome, (&def, [0, 0]));
        self.opened = self.opened.wrapping_add(1);
        self.message = Some((h3_view::message::Open::new(window), ended, Asked::Result));
        self.behind = h3_view::result::Behind::default();
        self.eagle_eye = self.eagle_eye_boxes(told, player);
        true
    }

    /// Eagle Eye's boxes `told` laid out (E17 M6 T5c, design §12.21), their pictures of the size
    /// of the spells' DEF (read now; without it the pictures are drawn empty, as what the client
    /// lacks), their frames in `player`'s colours (the result window's), each with its pickup
    /// sound's N ([`h3_view::eagle_eye::pickups`], *provisional*: drawn from the viewer's seed, the
    /// battle's generator not shared); none without the fonts.
    fn eagle_eye_boxes(
        &mut self,
        mut told: Vec<h3_view::eagle_eye::Told>,
        player: u8,
    ) -> Vec<(MessageBox, u8)> {
        if told.is_empty() {
            return Vec::new();
        }
        let def = self.ground.layout.message.spell_pictures.clone();
        let _ = self.load_names(vec![def.to_ascii_uppercase()]);
        let size = self.batch.sprite(&def).map_or([0, 0], |s| {
            [s.def.width, s.def.height].map(|v| i32::try_from(v).unwrap_or(i32::MAX))
        });
        for picture in told.iter_mut().flat_map(|t| t.pictures.iter_mut()) {
            picture.size = size;
        }
        let pickups = h3_view::eagle_eye::pickups(self.ground.seed, told.len());
        let Some(writer) = self.ground.writer.as_ref() else {
            return Vec::new();
        };
        let layout = &self.ground.layout;
        told.iter()
            .zip(pickups)
            .filter_map(|(t, n)| Some((writer.pictures(layout, &t.text, &t.pictures, player)?, n)))
            .collect()
    }

    /// The pickup sound of the box of Eagle Eye's message that opened since the last call, its N
    /// (E17 M6 T5c): the window plays it as the box opens.
    pub const fn take_pickup(&mut self) -> Option<u8> {
        self.pickup.take()
    }

    /// Whether the open box stands over the whole screen darkened (E17 M6 T5c): the result
    /// window, or a box of Eagle Eye's message after it; the battle behind stands at its end.
    fn darkened(&self) -> bool {
        self.message
            .as_ref()
            .is_some_and(|(_, _, asked)| matches!(asked, Asked::Result | Asked::EagleEye))
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
    /// The result window under Eagle Eye's boxes stands from `at`, its close's millisecond (E17
    /// M6 T5c, recorded: `eagle-eye-one`; the reviewer), unless a time was set already; the
    /// stills and sequences without the window's clock take the first box's first draw.
    pub fn window_closed_at(&mut self, at: u32) {
        if let Some((_, opened, since @ None)) = self.window_behind.as_mut() {
            *since = Some(at.saturating_sub(*opened));
        }
    }

    /// How many boxes have opened so far (E17 M6 T5c): the window empties its input queue at each.
    pub const fn boxes_opened(&self) -> u32 {
        self.opened
    }

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
        // The movement shadow outside the tactics phase, the grid and the cursor shadow as the
        // mouse's follow has them (E17 M6 T4c, design §12.16); behind the tactics phase's message
        // the grid alone (the background composed with it, no turn yet).
        if let Some(drawn) = self.shown.shades(t) {
            if self.opening {
                moment.overlays.grid = drawn.overlays.grid;
            } else {
                moment.shade.extend(&drawn.shade);
                moment.overlays = drawn.overlays.clone();
            }
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
            list.extend(self.panels.before(snapshot, layout));
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
        // The army info windows the mouse shows (E17 M6 T4b2b1, design §12.15): each
        // `battle_redraw` draws the shown ones again while Creature Info's level is not 0, each
        // filled by the battle of the turn that showed it (the stacks' since T4b2b2).
        for drawn in self.shown.army(t).drawn() {
            let window = match drawn {
                Drawn::Hero(side, turn) => {
                    (self.follower.observation(turn)).and_then(|o| self.hero_window(o, side))
                }
                Drawn::Stack(shown) => (self.follower.observation(shown.turn))
                    .and_then(|o| self.stack_window(o, shown)),
            };
            if let Some(window) = window {
                let draws = window.dialog.draws(&window.shown, &layout.player_palette);
                list.extend(draws.into_iter().map(Draw::Dialog));
            }
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
                    (!self.second_target(spell, Some(original.original))).then_some(Command::Cast {
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

    /// Whether the fought battle's record so far is the start of the battle fought again at once
    /// with the same setup (E17 M5 T2b: a battle pressed by the presser's choices; E17 M6 T5b1:
    /// the computer's choices asked again in the same order from the seats' streams afresh,
    /// `live::refight`); none for a record.
    pub fn as_the_bot_fights(&mut self) -> Option<Result<bool, String>> {
        let Some(Fought::Here(here)) = self.ground.fight.as_ref() else {
            return None;
        };
        let (ruleset, setup, seed) = &**here;
        let mut check = || {
            let live = Fight::here(self.fight.as_mut()).ok_or("no battle")?;
            let (players, asks) = (live.players(), live.asks().to_vec());
            let bot = crate::live::refight(ruleset, setup.clone(), *seed, players, &asks)?;
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
        // Behind a stack's stats dialog as it was when the dialog opened (E17 M6 T2a2a), behind
        // the combat options dialog too (E17 M6 T4a).
        let stats = self.stats.as_ref().map(|&(_, opened)| opened);
        let options = self.options.as_ref().map(|&(_, opened)| opened);
        let at = [Some(t), book, message, stats, options]
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
        if self.darkened() {
            self.behind
                .show(&mut moment, at, |t| Moment::at(timeline, t));
        }
        let mut list = self.list(at, &moment)?;
        if let Some((open, _, player)) = &self.book {
            let layout = &self.ground.layout;
            list.extend(open.draws(&layout.book, &layout.player_palette, *player));
        }
        if let Some((open, opened)) = &self.stats {
            // Its creature stepped since it opened.
            let window = open.at(STATS_FIRST_STEP, t.saturating_sub(*opened));
            list.extend(
                window
                    .dialog
                    .draws(&window.shown, &self.ground.layout.player_palette)
                    .into_iter()
                    .map(Draw::Dialog),
            );
            // The right button's description held over it (E17 M6 T2a2b1).
            if let Some(popup) = &open.modal.popup {
                list.extend(
                    popup
                        .dialog
                        .draws(&popup.shown, &self.ground.layout.player_palette)
                        .into_iter()
                        .map(Draw::Dialog),
                );
            }
        }
        // The combat options dialog with its shadow (E17 M6 T4a), under its boxes.
        if let Some((open, _)) = &self.options {
            let window = &open.window;
            list.extend(
                window
                    .dialog
                    .draws(&window.shown, &self.ground.layout.player_palette)
                    .into_iter()
                    .map(Draw::Dialog),
            );
        }
        // The window under Eagle Eye's boxes (E17 M6 T5c), its clip held at its close's
        // millisecond (`window_closed_at`, as the sequences', `eagle-eye-one`), else at the first
        // box's first draw.
        let eagle_eye = matches!(self.message, Some((_, _, Asked::EagleEye)));
        let behind = match self.window_behind.as_mut() {
            Some((window, opened, since)) if eagle_eye => {
                let since = *since.get_or_insert(t.saturating_sub(*opened));
                Some((window.clone(), since))
            }
            _ => None,
        };
        if let Some((open, opened, asked)) = &self.message {
            // The result window over the whole screen darkened (E17 M5 T4b1), Eagle Eye's boxes
            // after it too (E17 M6 T5c).
            if matches!(asked, Asked::Result | Asked::EagleEye) {
                list.push(h3_view::result::backdrop(self.ground.layout.frame));
            }
            if let Some((window, since)) = &behind {
                let palette = &self.ground.layout.player_palette;
                list.extend(
                    window
                        .dialog
                        .draws(&window.shown, palette)
                        .into_iter()
                        .map(Draw::Dialog),
                );
                list.extend(self.clip(*since));
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
    /// is the record's siege, a fort or not (the manager's +0x53C8; E17 M4 T2c-2, `defended`); a
    /// side whose opponent gave up wins only with army survivors (E17 M5 T8b2, R5b-3:
    /// `Battle::survivors`, read only then; taken as both having some when not known).
    pub fn end_tune(&self) -> Option<(u32, Tune)> {
        use h3_protocol::wire::Outcome;
        let (at, outcome) = self.timeline().ended?;
        let snapshot = self.record.first_snapshot();
        let human = snapshot.map_or([false; 2], |s| {
            [0, 1].map(|i| s.sides.get(i).is_some_and(|d| d.human))
        });
        let town = snapshot.is_some_and(|s| s.siege.is_some());
        let gave_up = matches!(outcome, Outcome::Retreat { .. } | Outcome::Surrender { .. });
        let survivors = if gave_up { self.survivors() } else { None };
        let survivors = survivors.unwrap_or([true; 2]);
        Some((
            at,
            h3_view::sound::end_tune(outcome, (human, town), survivors, Bugs::Fixed),
        ))
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

/// The Auto Combat boxes the battle began with (E17 M6 T4b1, design §12.13): those before the
/// first change the options dialog's closes made (`boxes`), else, while the dialog is open (or its
/// close not applied yet), the boxes as it opened (`open`: its clicks, which write the settings at
/// once, count only from its close; the reviewer of T4b2a), else the settings' (`settings`).
fn first_boxes(
    boxes: &[h3_view::auto::BoxChange],
    open: Option<[bool; 5]>,
    settings: [bool; 5],
) -> [bool; 5] {
    boxes.first().map_or(open.unwrap_or(settings), |c| c.2)
}

/// Whether an action of the last turn started in `timeline` was played, from its start on (E17
/// M6 T5b1; T5b2b: the computer's cast for the human's hero, or a command that took no time and
/// started the turn).
fn within(timeline: &Timeline) -> bool {
    let start = timeline.turns.last().map_or(0, |&(t, _)| t);
    (timeline.actions.last()).is_some_and(|a| a.start >= start)
}

/// A stop pressed at `at` as the battle reads it in `timeline` ([`Battle::stop_auto`]): its time,
/// never before the last action's start (E17 M6 T5b2b: a press read after it started, Restart's
/// replay, counts from then), and its turn, the one started last by then but never one before
/// the last (T8b2, R5c-4).
fn stop_read(timeline: &Timeline, at: u32) -> (u32, usize) {
    let at = at.max(timeline.actions.last().map_or(0, |a| a.start));
    let turns = &timeline.turns;
    (at, turn_at(turns, at).max(turns.len().saturating_sub(1)))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A timeline whose turns start at `turns` (each the stack 0's) with the actions `actions`
    /// (each from its start to its end).
    fn timeline(turns: &[u32], actions: &[(u32, u32)]) -> Timeline {
        Timeline {
            turns: turns.iter().map(|&t| (t, 0)).collect(),
            actions: (actions.iter())
                .map(|&(start, end)| h3_view::hero::Action {
                    start,
                    end,
                    ..h3_view::hero::Action::default()
                })
                .collect(),
            ..Timeline::default()
        }
    }

    #[test]
    fn an_action_from_the_last_turn_s_start_is_within_it() {
        // E17 M6 T5b2b (the reviewer): `play_next` outlines the turn waiting only when nothing of
        // it was played yet; the cast Auto Combat stopped at the end of put the outline back.
        assert!(!within(&timeline(&[], &[])));
        assert!(within(&timeline(&[], &[(0, 300)])));
        assert!(!within(&timeline(&[300], &[(0, 300)])));
        assert!(within(&timeline(&[300], &[(0, 300), (300, 900)])));
        assert!(within(&timeline(&[300], &[(0, 300), (300, 300)])));
    }

    #[test]
    fn a_stop_is_read_from_the_last_action_s_start_in_the_turn_then() {
        // E17 M6 T5b2b (the reviewer): during the cast (300 to 900, turn 1) a press at 500 is
        // read then; one read after the stack's action started at 900 (Restart's replay of an
        // earlier press) from 900, never inside the cast passed. T8b2 R5c-4: never a turn
        // before the last.
        let cast = timeline(&[300], &[(0, 300), (300, 900)]);
        assert_eq!(stop_read(&cast, 500), (500, 1));
        let action = timeline(&[300], &[(0, 300), (300, 900), (900, 1200)]);
        assert_eq!(stop_read(&action, 500), (900, 1));
        assert_eq!(stop_read(&action, 1000), (1000, 1));
        assert_eq!(stop_read(&timeline(&[300, 600], &[]), 100), (100, 1));
        assert_eq!(stop_read(&timeline(&[], &[]), 100), (100, 0));
    }

    #[test]
    fn a_refusal_s_box_hands_the_mouse_back_as_it_closes() {
        // E17 M6 T5a (the review, design §12.17): the book's and the aim's refusals post the mouse
        // after their OK; the result window and a turret's info keep their own close; the other
        // boxes answer by their button.
        assert_eq!(closed(Asked::Refused), Some(MessageDid::Refused));
        assert_eq!(closed(Asked::Result), Some(MessageDid::Ended));
        assert_eq!(closed(Asked::Turret), Some(MessageDid::Looked));
        assert_eq!(closed(Asked::Told), None);
        assert_eq!(closed(Asked::Retreat), None);
    }

    #[test]
    fn the_boxes_the_battle_began_with_ignore_the_open_dialog_s_clicks() {
        // E17 M6 T4b2a (the reviewer, §12.13): a change counts only from the dialog's close.
        let (none, creatures, all) = ([false; 5], [true, false, false, false, false], [true; 5]);
        assert_eq!(first_boxes(&[], None, creatures), creatures);
        assert_eq!(first_boxes(&[], Some(creatures), all), creatures);
        let changes = [(0, 0, none), (3, 900, all)];
        assert_eq!(first_boxes(&changes, Some(creatures), all), none);
        assert_eq!(first_boxes(&changes, None, creatures), none);
    }

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
