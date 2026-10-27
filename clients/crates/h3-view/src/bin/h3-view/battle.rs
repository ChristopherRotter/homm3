//! A battle ready to draw at any time (E17 M2 T4b-2a's `play`, shared with `window` since T5a),
//! played as it is drawn since E17 M5 T1b. What it is drawn from is read once ([`Ground`]): the
//! record (a record file's whole, or a scenario's opening: its battle is then fought as it is
//! played, [`crate::live`]), the asset tables and the layout, the archives, the stage's numbers
//! and the heroes. The battle ([`Battle`]) grows over it: the next command is played when a frame
//! at or past the timeline's end is asked for, and the record, the timeline at the original's
//! timing with the viewer's own fidgets ([`h3_view::anim::Growing`]), the panel and the mouse
//! grow with it; the checkpoint batch takes the assets as the frames name them. An adapter: it
//! reads files and the archives.

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use h3_battle::BattleSetup;
use h3_content_battle::BattleRuleset;
use h3_formats::lod::LodArchive;
use h3_protocol::wire::{Command, Observation, ServerMessage};
use h3_render::Bugs;
use h3_render::screen::Screen;
use h3_view::anim::Timeline;
use h3_view::anim::{self, Casts, Growing, Look, MagicLook, Playhead, SiegeStage, Sound, Speed};
use h3_view::batch::{self, Batch, Sprite};
use h3_view::battle_log::Logged;
use h3_view::compose::compose;
use h3_view::hero::Heroes;
use h3_view::layout::{self, BattleLayout};
use h3_view::mouse::track::{Input, Shown};
use h3_view::mouse::{self, Field};
use h3_view::record::{self, Record};
use h3_view::scene::Panels;
use h3_view::scene::{self, Draw, Moment};
use h3_view::sound::Tune;
use h3_view::tables::{Sounds, Tables};

use crate::live::Live;
use crate::render::{lookup, read_archives};
use crate::{assets, motion};

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
}

impl Source {
    /// Whether the input is a scenario (anything but a `.jsonl` record).
    pub fn scenario(&self) -> bool {
        self.input.extension().is_none_or(|e| e != "jsonl")
    }

    /// The input's file stem (`battle` if none).
    pub fn stem(&self) -> String {
        self.input
            .file_stem()
            .map_or_else(|| "battle".into(), |s| s.to_string_lossy().into_owned())
    }
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
    /// Each stack's frames of the fidget's group, by id.
    fidget_frames: BTreeMap<u8, u32>,
    /// The viewer's own fidgets' seed and the combat speed.
    seed: u32,
    speed: Speed,
    /// A scenario's battle: its ruleset, setup and seed (none: a record file's).
    fight: Option<(BattleRuleset, BattleSetup, u64)>,
    /// The sides a human plays here ([`Source::humans`]).
    humans: [bool; 2],
}

impl Ground {
    /// Reads what the battle of `source` is drawn from: a record file, or a scenario's opening
    /// (its battle is fought as it is played: every magic animation and spell read for it, as its
    /// events are not known yet).
    pub fn load(source: &Source) -> Result<Self, String> {
        let humans = source.humans;
        if humans.contains(&true) && !source.scenario() {
            return Err("a human plays a scenario's battle, not a record's".into());
        }
        let (record, fight) = if source.scenario() {
            let (ruleset, mut setup) =
                crate::scenario::battle(&source.input, &source.packs, source.seed)?;
            // The sides a human plays here are the battle's human players, the bot's the
            // computer's (E17 M5 T2b, *provisional*).
            if humans.contains(&true) {
                setup.human = humans;
            }
            let opening = Live::new(&ruleset, setup.clone(), source.seed, humans)?.record()?;
            let record = record::parse(&opening).map_err(|e| e.to_string())?;
            (record, Some((ruleset, setup, source.seed)))
        } else {
            let text = fs::read_to_string(&source.input)
                .map_err(|e| format!("{}: {e}", source.input.display()))?;
            (record::parse(&text).map_err(|e| e.to_string())?, None)
        };
        let snapshot = record
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
        let parsed = parse(&archives)?;
        // The executable's own strings: read only when a line needs them.
        let exe = crate::text::executable(&source.install).unwrap_or_default();
        let every = fight.is_some();
        let looks = motion::looks(&record, &tables, &parsed)?;
        let magic = motion::magic(&record, &tables, &parsed, every)?;
        let siege = motion::siege(&record, &tables, &parsed)?;
        let casts = motion::casts(&record, &tables, &parsed, &source.packs, every)?;
        // The viewer's own fidgets and the heroes' reactions, from the battle's seed.
        let seed = u32::try_from(source.seed & u64::from(u32::MAX)).unwrap_or(1);
        let heroes = motion::heroes(&record, &tables, &parsed, Some(seed))?;
        let writer = crate::log::Writer::new(&tables, &layout, (&parsed, &exe), Bugs::Fixed).ok();
        let group = tables.battle.groups.mouse_over;
        let fidget_frames = snapshot
            .stacks
            .iter()
            .filter_map(|s| {
                let frames = looks.get(&s.creature)?.frames.get(&u32::from(group))?;
                Some((s.id, *frames))
            })
            .collect();
        drop(parsed);
        Ok(Self {
            record,
            tables,
            layout,
            archives,
            exe,
            looks,
            magic,
            casts,
            siege,
            heroes,
            writer,
            fidget_frames,
            seed,
            speed: source.speed,
            fight,
            humans,
        })
    }
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
}

/// The battle, played as it is drawn (the module's text).
#[derive(Debug)]
pub struct Battle<'g> {
    ground: &'g Ground,
    /// The record so far: the opening, then what each command played sent.
    record: Record,
    /// The timeline at the original's timing.
    played: Played<'g>,
    /// The commands played.
    commands: usize,
    /// The battle being fought (none: a record file's).
    live: Option<Live<'g>>,
    /// The timeline with the stacks the mouse entered fidgeting (E17 M3 T6c-2; none: as played).
    entered: Option<Timeline>,
    /// The timeline shown read forward as the frames are asked for (E17 M5 T1a): a frame costs
    /// what changed since the one before it, not the battle's length.
    head: Playhead,
    /// The panel over the battle: its turns and its log (E17 M3 T6b).
    pub panels: Panels,
    /// The battle and its legal commands at each turn (a fought scenario's; none for a record).
    turned: Vec<(Observation, Vec<Command>)>,
    /// The assets the frames drew so far.
    batch: Batch,
    /// The mouse over the battle: its moves and what they showed.
    pub shown: Shown,
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
}

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
}

impl<'g> Battle<'g> {
    /// The battle over `ground`: a record file's played whole, a scenario's before its first
    /// command.
    pub fn new(ground: &'g Ground) -> Result<Self, String> {
        let snapshot = ground
            .record
            .first_snapshot()
            .ok_or("the record holds no snapshot")?;
        let fight = match &ground.fight {
            Some((ruleset, setup, seed)) => {
                Some(Live::new(ruleset, setup.clone(), *seed, ground.humans)?)
            }
            None => None,
        };
        // A battle a human plays here grows from every side's events: the screen shows the side
        // to act's hidden objects (E17 M5 T2b), the snapshot's among them.
        let mut opening = snapshot.clone();
        if let Some(fight) = fight.as_ref().filter(|f| f.human()) {
            opening.spell_obstacles = fight.obstacles();
        }
        let stage = motion::stage(
            &ground.record,
            &ground.tables,
            &ground.layout,
            (&ground.magic, &ground.casts, ground.siege.as_ref()),
        );
        let mut growing = Growing::new(
            &opening,
            &ground.looks,
            stage,
            ground.speed,
            Some(ground.seed),
        )
        .map_err(|e| e.to_string())?
        .with_heroes(ground.heroes.clone());
        let mut live = None;
        let mut turned = Vec::new();
        let played = match fight {
            Some(fight) => {
                turned.extend(fight.turn());
                let played = if fight.over() {
                    Played::Finished(Box::new(growing.finish()))
                } else if fight.waiting() {
                    let tail = Box::new(growing.waiting());
                    Played::Waiting(Box::new(growing), tail)
                } else {
                    Played::Growing(Box::new(growing))
                };
                live = Some(fight);
                played
            }
            None => {
                // A record's commands, each the computer's when it has no time.
                let commands = anim::commands(&ground.record.messages);
                for (i, command) in commands.iter().enumerate() {
                    let computer = commands.get(i + 1).is_some_and(|c| c.at.is_none());
                    growing.push(command, computer);
                }
                Played::Finished(Box::new(growing.finish()))
            }
        };
        let commands = match &played {
            Played::Growing(_) | Played::Waiting(..) => 0,
            Played::Finished(_) => anim::commands(&ground.record.messages).len(),
        };
        let head = Playhead::new(played.timeline());
        let mut battle = Self {
            ground,
            record: ground.record.clone(),
            played,
            commands,
            live,
            entered: None,
            head,
            panels: Panels::new(snapshot, &[], Vec::new()),
            turned,
            batch: Batch::default(),
            shown: Shown::default(),
            inputs: Vec::new(),
            scrolls: Vec::new(),
            pressed: None,
            book: None,
            book_memory: h3_view::book::Memory::default(),
        };
        battle.follow()?;
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
        self.live.as_mut().map(Live::record)
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
    /// battle waits for a human's.
    fn play_next(&mut self) -> Result<(), String> {
        let (Some(live), Played::Growing(_)) = (self.live.as_mut(), &self.played) else {
            return Ok(());
        };
        match live.next()? {
            Some(played) => self.take(None, played),
            None => Ok(()),
        }
    }

    /// A human's `command`, pressed at `at` while the battle waits for it (E17 M5 T2b): sent
    /// from the side to act's seat, the session validating it, its action starting at `at` or at
    /// the timeline's end if later; whether the battle took it (none waits: nothing).
    pub fn command(&mut self, at: u32, command: Command) -> Result<bool, String> {
        let (Some(live), Played::Waiting(..)) = (self.live.as_mut(), &self.played) else {
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
        let (Some(live), Played::Growing(growing)) = (self.live.as_mut(), &mut self.played) else {
            return Ok(());
        };
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

    /// Follows the mouse's `inputs` and the log's `scrolls` (each at a time, in time order; E17
    /// M3 T6c-2): the cursor, the stack it marks, the stacks it enters fidgeting, its hints in
    /// the log; whether anything shown changed. Nothing without the text tables.
    pub fn mouse(&mut self, inputs: &[Input], scrolls: &[(u32, i64)]) -> Result<bool, String> {
        if self.ground.writer.is_none() {
            return Ok(false);
        }
        let shown = self.pointer(inputs);
        if shown == self.shown && scrolls == self.scrolls.as_slice() {
            return Ok(false);
        }
        self.inputs = inputs.to_vec();
        self.scrolls = scrolls.to_vec();
        self.show(shown)?;
        Ok(true)
    }

    /// The mouse over the battle as played so far (E17 M3 T6c-2).
    fn pointer(&self, inputs: &[Input]) -> Shown {
        self.ground
            .writer
            .as_ref()
            .map_or_else(Shown::default, |writer| {
                crate::pointer::shown(
                    &self.record,
                    self.played.timeline(),
                    &self.turned,
                    inputs,
                    (&self.ground.layout, &writer.catalogue),
                    (&self.ground.tables, Bugs::Fixed),
                )
            })
    }

    /// The mouse, the timeline shown and the panel after the battle grew (or as it starts): the
    /// playhead goes on over what was played, unless the mouse's fidgets go in afresh.
    fn follow(&mut self) -> Result<(), String> {
        let shown = self.pointer(&self.inputs);
        if shown.entered.is_empty() && self.entered.is_none() {
            self.head.extend(self.played.timeline());
        } else {
            self.entered = self.entered(&shown);
            self.head = Playhead::new(self.timeline());
        }
        self.shown = shown;
        self.panels = self.panel()?;
        Ok(())
    }

    /// Shows the mouse's `shown`: the stacks it entered fidgeting in the timeline, its hints and
    /// the log's scrolls in the panel.
    fn show(&mut self, shown: Shown) -> Result<(), String> {
        if shown.entered != self.shown.entered {
            self.entered = self.entered(&shown);
            self.head = Playhead::new(self.timeline());
        }
        self.shown = shown;
        self.panels = self.panel()?;
        Ok(())
    }

    /// The timeline as played with the stacks `shown` entered fidgeting (none: none entered).
    fn entered(&self, shown: &Shown) -> Option<Timeline> {
        (!shown.entered.is_empty()).then(|| {
            let mut timeline = self.played.timeline().clone();
            let group = self.ground.tables.battle.groups.mouse_over;
            let frames = |id: u8| self.ground.fidget_frames.get(&id).copied().unwrap_or(0);
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
            return crate::log::panels(
                &self.record,
                &self.played.timeline().log,
                &ground.tables,
                &ground.layout,
                (&parsed, &ground.exe),
                Bugs::Fixed,
            );
        };
        if self.inputs.is_empty() && self.scrolls.is_empty() {
            return writer.panels(&self.record, &self.played.timeline().log);
        }
        // The scrolls after the log's entries at their time.
        let mut logged = self.shown.log.clone();
        for &(at, by) in &self.scrolls {
            let i = logged.partition_point(|(t, _)| *t <= at);
            logged.insert(i, (at, Logged::Scroll(by)));
        }
        writer.panels(&self.record, &logged)
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
        if let Some(frame) = self.book.as_ref().and_then(|(open, _, _)| open.cursor) {
            let book = &self.ground.layout.book;
            return CursorLook {
                def: book.cursor.clone(),
                frame,
                hot: hot_of(&book.hot, frame),
            };
        }
        let mouse = &self.ground.layout.mouse;
        let frame = self.shown.frame(t).unwrap_or(mouse.panel_frame);
        CursorLook {
            def: mouse.cursor.clone(),
            frame,
            hot: hot_of(&mouse.hot, frame),
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
        let (Some(observation), Some((ruleset, _, _)), Some(writer)) = (
            self.live.as_ref().and_then(Live::own_view),
            ground.fight.as_ref(),
            ground.writer.as_ref(),
        ) else {
            return false;
        };
        if !crate::spellbook::takes(&observation) {
            return false;
        }
        let names = (&ground.tables, &writer.catalogue);
        let manifest = &self.record.header.manifest;
        let Some(hero) =
            crate::spellbook::hero(&observation, (manifest, ruleset), names, &ground.layout.book)
        else {
            return false;
        };
        let open = h3_view::book::Open::new(&mut self.book_memory, hero, &writer.catalogue);
        // The hero's player's colours (the record names none: red, blue; CLT-Q6).
        let index = usize::from(observation.viewer == h3_protocol::wire::Side::Defender);
        let player = observation.sides[index]
            .colour
            .unwrap_or_else(|| u8::try_from(index).unwrap_or(0));
        self.book = Some((open, at, player));
        true
    }

    /// The open book's answer to `outcome` (E17 M5 T3a): closed, or closed on a spell, whose cast
    /// is given when the engine takes it without a target (one that takes a target is aimed first:
    /// T3b's); a refusal's and a description's message boxes are T4's.
    fn book_did(&mut self, outcome: h3_view::book::Outcome) -> BookDid {
        use h3_view::book::Outcome;
        match outcome {
            Outcome::Nothing | Outcome::Refused { .. } | Outcome::Described(_) => BookDid::Nothing,
            Outcome::Turned => BookDid::Changed,
            Outcome::Closed => {
                self.book = None;
                BookDid::Changed
            }
            Outcome::Chosen(spell) => {
                self.book = None;
                let untargeted = Command::Cast {
                    spell,
                    target: h3_protocol::wire::Target::None {},
                    second: h3_protocol::wire::Target::None {},
                };
                let takes = self
                    .live
                    .as_ref()
                    .is_some_and(|live| live.casts_untargeted(spell));
                if takes {
                    BookDid::Cast(untargeted)
                } else {
                    BookDid::Changed
                }
            }
        }
    }

    /// A key down while the book is open (its name, as the book's: `C`, `Escape`, `Return`,
    /// the arrows).
    pub fn book_key(&mut self, key: &str) -> BookDid {
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
        let (Some(writer), Some((open, _, _))) = (ground.writer.as_ref(), self.book.as_mut()) else {
            return false;
        };
        open.hover(
            &mut self.book_memory,
            (&ground.layout.book, &writer.catalogue),
            point,
        )
    }

    /// The battle's layout.
    pub fn layout(&self) -> &BattleLayout {
        &self.ground.layout
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
        let mut list = scene::battle_at(
            snapshot,
            &moment,
            &self.record.header.manifest,
            tables,
            layout,
        )
        .map_err(|e| format!("{t} ms: {e}"))?;
        let text = self.panels.text(t);
        list.extend(self.panels.at_pressed(
            snapshot,
            &self.timeline().turns,
            t,
            layout,
            text,
            self.pressed,
        ));
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
    /// panel's Wait or Defend, else a point of `points` ([`crate::pointer::press_points`]) or the
    /// siege's rectangles; none if none gives it (a hero's cast, the tactics phase's end: M5 T3,
    /// T4; CLT-Q25's step back) or the battle takes no press then.
    pub fn press_for(
        &self,
        at: u32,
        wanted: Command,
        points: &[([i32; 2], Vec<[i32; 2]>)],
    ) -> Option<Press> {
        let layout = &self.ground.layout;
        let rects = [layout.mouse.keep, layout.mouse.tower].map(crate::pointer::middle);
        self.taken(at, |field| {
            let button = [mouse::WAIT, mouse::DEFEND]
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
        self.live.as_mut()?.choice()
    }

    /// Whether the fought battle's record so far is the start of the battle the bot fights at
    /// once with the same setup (E17 M5 T2b: a battle pressed by the bot's choices); none for a
    /// record.
    pub fn as_the_bot_fights(&mut self) -> Option<Result<bool, String>> {
        let (ruleset, setup, seed) = self.ground.fight.as_ref()?;
        let mut check = || {
            let bot = crate::live::bot_record(ruleset, setup.clone(), *seed)?;
            let bot = record::parse(&bot).map_err(|e| e.to_string())?;
            let ours = self.live.as_mut().ok_or("no battle")?.record()?;
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
        self.live.is_none() || (self.waiting() && at >= self.played.timeline().end)
    }

    /// What `give` gives in the battle of the turn waiting at `at`: none unless the battle
    /// waits for a human's command and its last action played (the timeline's end reached; a
    /// press made before is dropped as the turn starts, design §9.11).
    fn taken<T>(&self, at: u32, give: impl Fn(&Field<'_>) -> Option<T>) -> Option<T> {
        if self.live.is_none() || !self.takes(at) {
            return None;
        }
        let (observation, commands) = self.turned.last()?;
        let tables = &self.ground.tables;
        let manifest = &self.record.header.manifest;
        let legal = crate::pointer::legal(manifest, tables, observation, commands);
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
        // Behind the open spell book the battle stands as it was when it opened (E17 M5 T3a).
        let at = self.book.as_ref().map_or(t, |&(_, opened, _)| opened.min(t));
        self.grow(at)?;
        let timeline = self
            .entered
            .as_ref()
            .unwrap_or_else(|| self.played.timeline());
        self.head.seek(timeline, at);
        let moment = self.head.moment(timeline);
        let mut list = self.list(at, &moment)?;
        if let Some((open, _, player)) = &self.book {
            let layout = &self.ground.layout;
            list.extend(open.draws(&layout.book, &layout.player_palette, *player));
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
            self.batch
                .add(name, &lookup(&parsed, name)?)
                .map_err(|e| e.to_string())?;
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

    /// The sounds of each stack living at the start, by its id (E17 M2 T5b); a siege's turrets
    /// (off the field) the town's shooter's, as their look (E17 M4 T3a, R2-1).
    pub fn stack_sounds(&self) -> BTreeMap<u8, Sounds> {
        let (record, tables) = (&self.record, &self.ground.tables);
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
    /// human too).
    pub fn battle_sounds(&self) -> BTreeMap<Sound, String> {
        let spells = &self.record.header.manifest.keys.spells;
        let tables = &self.ground.tables;
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
        ];
        for handle in (0..spells.len()).filter_map(|h| u16::try_from(h).ok()) {
            sounds.push((Sound::Spell(handle), spell(handle, false)));
            sounds.push((Sound::SpellFile(handle), spell(handle, false)));
            sounds.push((Sound::Object(handle), spell(handle, true)));
        }
        let cued = |cue: &Sound| self.timeline().cues.iter().any(|c| c.sound == *cue);
        // Still fought: by the bot or waiting for a human (E17 M5 T2b).
        let fought = !matches!(self.played, Played::Finished(_));
        sounds
            .into_iter()
            .filter(|(cue, _)| fought || cued(cue))
            .filter_map(|(cue, name)| Some((cue, name?)))
            .collect()
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

/// Whether the batch holds the asset `name`.
fn held(batch: &Batch, name: &str) -> bool {
    batch.sprite(name).is_some()
        || batch.picture(name).is_some()
        || batch.palette(name).is_some()
        || batch.font(name).is_some()
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
