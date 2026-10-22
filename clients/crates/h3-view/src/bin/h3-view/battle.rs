//! A battle ready to draw at any time (E17 M2 T4b-2a's `play`, shared with `window` since T5a):
//! the record (a scenario fought to its end by the bot, or a record file), the asset tables and
//! the layout, the timeline at the original's timing with the viewer's own fidgets, and, once
//! loaded, the checkpoint batch. An adapter: it reads files and the archives.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use h3_formats::lod::LodArchive;
use h3_protocol::wire::{Command, Observation};
use h3_render::Bugs;
use h3_render::screen::Screen;
use h3_view::anim::{self, Sound, Speed, Timeline};
use h3_view::batch::{self, Batch, Sprite};
use h3_view::battle_log::Logged;
use h3_view::compose::compose;
use h3_view::layout::{self, BattleLayout};
use h3_view::mouse::track::{Input, Shown};
use h3_view::record::{self, Record};
use h3_view::scene::Panels;
use h3_view::scene::{self, Draw, Moment};
use h3_view::sound::Tune;
use h3_view::tables::{Sounds, Tables};

use crate::gate::timed;
use crate::render::{lookup, read_archives, write};
use crate::{assets, motion};

/// Where a battle comes from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Source {
    /// A scenario (fought by the bot) or a record file (`.jsonl`).
    pub input: PathBuf,
    /// The scenario's seed (obstacles, damage rolls) and the fidgets' seed.
    pub seed: u64,
    /// The combat speed the commands play at.
    pub speed: Speed,
    /// The packs a scenario names.
    pub packs: PathBuf,
    /// The installation copy whose archives are read.
    pub install: PathBuf,
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

/// The battle, its timeline and its assets.
#[derive(Debug)]
pub struct Battle {
    record: Record,
    /// The asset tables.
    pub tables: Tables,
    layout: BattleLayout,
    /// The battle at the original's timing.
    pub timeline: Timeline,
    /// The panel over the battle: its turns and its log (E17 M3 T6b).
    pub panels: Panels,
    /// The commands played.
    pub commands: usize,
    /// The archives' bytes until the batch is loaded (CLT-Q2: whole archives).
    archives: Vec<(&'static str, Vec<u8>)>,
    batch: Batch,
    /// The battle at the original's timing before the mouse entered stacks (E17 M3 T6c-2).
    base: Timeline,
    /// The battle and its legal commands at each turn (a fought scenario's; none for a record).
    turned: Vec<(Observation, Vec<Command>)>,
    /// What words the log's lines and the mouse's hints (none without the text tables).
    writer: Option<crate::log::Writer>,
    /// Each stack's frames of the fidget's group, by id.
    fidget_frames: BTreeMap<u8, u32>,
    /// The mouse over the battle.
    pub shown: Shown,
    /// The log scrolled (each at a time, by entries).
    scrolls: Vec<(u32, i64)>,
    /// The panel's button pressed (the mouse's or a hotkey's).
    pub pressed: Option<u16>,
}

impl Battle {
    /// Fights the scenario (writing its record to `record_to`) or reads the record, and times
    /// it; a line on the record fought goes to `lines`.
    pub fn load(
        source: &Source,
        record_to: Option<&Path>,
        lines: &mut Vec<String>,
    ) -> Result<Self, String> {
        let mut sent = Vec::new();
        let text = if source.scenario() {
            let (text, took) =
                timed(|| crate::scenario::fought(&source.input, &source.packs, source.seed));
            let (text, commands) = text?;
            sent = commands;
            if let Some(path) = record_to {
                write(path, text.as_bytes())?;
                lines.push(format!(
                    "record: {} (fought in {} ms)",
                    path.display(),
                    took.as_millis()
                ));
            }
            text
        } else {
            fs::read_to_string(&source.input)
                .map_err(|e| format!("{}: {e}", source.input.display()))?
        };
        let record = record::parse(&text).map_err(|e| e.to_string())?;
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
        let looks = motion::looks(&record, &tables, &parsed)?;
        let magic = motion::magic(&record, &tables, &parsed)?;
        let casts = motion::casts(&record, &tables, &parsed, &source.packs)?;
        let stage = motion::stage(&record, &tables, &layout, (&magic, &casts));
        let commands = anim::commands(&record.messages);
        // The viewer's own fidgets, from the battle's seed (`h3_view::anim::Fidgets`).
        let seed = u32::try_from(source.seed & u64::from(u32::MAX)).unwrap_or(1);
        let timeline = anim::fidgeting(snapshot, &commands, &looks, stage, source.speed, seed)
            .map_err(|e| e.to_string())?
            .with_heroes(motion::heroes(&record, &tables, &parsed, Some(seed))?);
        let commands = commands.len();
        let panels = crate::log::panels(&record, &timeline.log, &tables, &layout, (&parsed, &exe))?;
        // The mouse (E17 M3 T6c-2): each turn's battle from the engine, the words of its hints,
        // the stacks' fidgets' frames.
        let turned = if source.scenario() {
            crate::scenario::turned(&source.input, &source.packs, source.seed, &sent)?
        } else {
            Vec::new()
        };
        let writer = crate::log::Writer::new(&tables, &layout, (&parsed, &exe)).ok();
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
            base: timeline.clone(),
            timeline,
            panels,
            commands,
            archives,
            batch: Batch::default(),
            turned,
            writer,
            fidget_frames,
            shown: Shown::default(),
            scrolls: Vec::new(),
            pressed: None,
        })
    }

    /// Follows the mouse's `inputs` and the log's `scrolls` (each at a time, in time order; E17
    /// M3 T6c-2): the cursor, the stack it marks, the stacks it enters fidgeting, its hints in
    /// the log; whether anything shown changed. Nothing without the text tables.
    pub fn mouse(&mut self, inputs: &[Input], scrolls: &[(u32, i64)]) -> Result<bool, String> {
        let Some(writer) = &self.writer else {
            return Ok(false);
        };
        let shown = crate::pointer::shown(
            &self.record,
            &self.base,
            &self.turned,
            inputs,
            &self.layout,
            &writer.catalogue,
        );
        if shown == self.shown && scrolls == self.scrolls.as_slice() {
            return Ok(false);
        }
        if shown.entered != self.shown.entered {
            let mut timeline = self.base.clone();
            let group = self.tables.battle.groups.mouse_over;
            let frames = |id: u8| self.fidget_frames.get(&id).copied().unwrap_or(0);
            timeline.enter(&shown.entered, u32::from(group), &frames);
            self.timeline = timeline;
        }
        // The scrolls after the log's entries at their time.
        let mut logged = shown.log.clone();
        for &(at, by) in scrolls {
            let i = logged.partition_point(|(t, _)| *t <= at);
            logged.insert(i, (at, Logged::Scroll(by)));
        }
        self.panels = writer.panels(&self.record, &logged)?;
        self.shown = shown;
        self.scrolls = scrolls.to_vec();
        Ok(true)
    }

    /// The combat cursor's frame at `t` (the panel's pointer before the mouse is anywhere).
    pub fn cursor_frame(&self, t: u32) -> u32 {
        self.shown.frame(t).unwrap_or(self.layout.mouse.panel_frame)
    }

    /// The battle's layout.
    pub fn layout(&self) -> &BattleLayout {
        &self.layout
    }

    /// The draw list of the battle at `moment` (at `t`), the panel's after it as the battle's
    /// turns and lines left it (E17 M3 T6b).
    pub fn list(&self, t: u32, moment: &Moment) -> Result<Vec<Draw>, String> {
        let snapshot = self
            .record
            .first_snapshot()
            .ok_or("the record holds no snapshot")?;
        // The stack the mouse marks (E17 M3 T6c-2).
        let mut moment = moment.clone();
        moment.marked = self.shown.marked(t);
        let mut list = scene::battle_at(
            snapshot,
            &moment,
            &self.record.header.manifest,
            &self.tables,
            &self.layout,
        )
        .map_err(|e| format!("{t} ms: {e}"))?;
        let text = self.panels.text(t);
        list.extend(self.panels.at_pressed(
            snapshot,
            &self.timeline.turns,
            t,
            &self.layout,
            text,
            self.pressed,
        ));
        Ok(list)
    }

    /// The draw list of the battle at `t`.
    pub fn list_at(&self, t: u32) -> Result<Vec<Draw>, String> {
        self.list(t, &Moment::at(&self.timeline, t))
    }

    /// Every distinct frame up to `until` (past the end the battle idles): its time, moment and
    /// draw list; a moment that draws what the one before drew is not a new frame.
    pub fn frames(&self, until: u32) -> Result<Vec<(u32, Moment, Vec<Draw>)>, String> {
        let mut frames: Vec<(u32, Moment, Vec<Draw>)> = Vec::new();
        for t in self.timeline.changes(until) {
            let moment = Moment::at(&self.timeline, t);
            let list = self.list(t, &moment)?;
            if frames.last().is_some_and(|(_, _, last)| *last == list) {
                continue;
            }
            frames.push((t, moment, list));
        }
        Ok(frames)
    }

    /// Loads the checkpoint batch: every name `lists` need and `extra`; the archives' bytes go.
    pub fn load_batch<'a>(
        &mut self,
        lists: impl IntoIterator<Item = &'a [Draw]>,
        extra: &[&str],
    ) -> Result<(), String> {
        let mut names = BTreeSet::new();
        for list in lists {
            names.extend(batch::names(list, &self.layout));
        }
        names.extend(extra.iter().map(|n| n.to_ascii_uppercase()));
        let parsed = parse(&self.archives)?;
        for name in &names {
            self.batch
                .add(name, &lookup(&parsed, name)?)
                .map_err(|e| e.to_string())?;
        }
        drop(parsed);
        self.archives = Vec::new();
        Ok(())
    }

    /// `list` drawn into a frame with the batch.
    pub fn compose(&self, t: u32, list: &[Draw]) -> Result<Screen, String> {
        compose(list, &self.batch, &self.layout, Bugs::Fixed).map_err(|e| format!("{t} ms: {e}"))
    }

    /// A DEF of the batch.
    pub fn sprite(&self, name: &str) -> Option<&Sprite> {
        self.batch.sprite(name)
    }

    /// The sounds of each stack living at the start, by its id (E17 M2 T5b).
    pub fn stack_sounds(&self) -> BTreeMap<u8, Sounds> {
        let keys = &self.record.header.manifest.keys.creatures;
        self.record
            .first_snapshot()
            .map(|s| s.stacks.as_slice())
            .unwrap_or_default()
            .iter()
            .filter_map(|s| {
                let key = keys.get(usize::from(s.creature))?;
                let creature = self.tables.creatures.get(key)?;
                Some((s.id, creature.sounds.clone()))
            })
            .collect()
    }

    /// The battle's own sounds its cues name, by cue (not a creature's; design §8.2): the tent's
    /// heal (E17 M3 T3b), a resisted spell's and each cast spell's (E17 M3 T7b-1), morale's, Fear's
    /// and good luck's (E17 M3 T7b-2b-2b), Life Drain's, the Thunderbird's bolt's and the spells'
    /// the striker's effects play (E17 M3 T7b-2b-3), the sound placed with each object (E17 M3
    /// T7c-2).
    pub fn battle_sounds(&self) -> BTreeMap<Sound, String> {
        let spells = &self.record.header.manifest.keys.spells;
        let battle = &self.tables.battle;
        self.timeline
            .cues
            .iter()
            .filter_map(|c| {
                let name = match c.sound {
                    Sound::Heal => Some(&battle.heal_sound),
                    Sound::Resisted => battle.resist_sound.as_ref(),
                    Sound::GoodMorale => battle.good_morale_sound.as_ref(),
                    Sound::BadMorale => battle.bad_morale_sound.as_ref(),
                    Sound::Fear => battle.fear_sound.as_ref(),
                    Sound::Luck => battle.luck_sound.as_ref(),
                    Sound::Drain => battle.drain_sound.as_ref(),
                    Sound::Thunderbolt => battle.thunderbolt_sound.as_ref(),
                    Sound::Spell(handle) | Sound::SpellFile(handle) => spells
                        .get(usize::from(handle))
                        .and_then(|key| self.tables.spells.get(key)?.sound.as_ref()),
                    Sound::Object(handle) => spells
                        .get(usize::from(handle))
                        .and_then(|key| self.tables.spells.get(key)?.object_sound.as_ref()),
                    _ => None,
                };
                Some((c.sound, name?.clone()))
            })
            .collect()
    }

    /// When the battle is decided and the tune its end plays then (E17 M3 T8, design §8.2): the
    /// result's window shows the human side's view by the snapshot's `human`; the battle is at no
    /// town (*provisional*: sieges are not drawn yet).
    pub fn end_tune(&self) -> Option<(u32, Tune)> {
        let (at, outcome) = self.timeline.ended?;
        let human = self.record.first_snapshot().map_or([false; 2], |s| {
            [0, 1].map(|i| s.sides.get(i).is_some_and(|d| d.human))
        });
        Some((at, h3_view::sound::end_tune(outcome, human, false)))
    }

    /// The frame's size, as the layout gives it.
    pub fn frame_size(&self) -> [u32; 2] {
        self.layout.frame
    }
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
