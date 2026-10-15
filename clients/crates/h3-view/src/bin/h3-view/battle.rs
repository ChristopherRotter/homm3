//! A battle ready to draw at any time (E17 M2 T4b-2a's `play`, shared with `window` since T5a):
//! the record (a scenario fought to its end by the bot, or a record file), the asset tables and
//! the layout, the timeline at the original's timing with the viewer's own fidgets, and, once
//! loaded, the checkpoint batch. An adapter: it reads files and the archives.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use h3_formats::lod::LodArchive;
use h3_render::screen::Screen;
use h3_view::anim::{self, Speed, Timeline};
use h3_view::batch::{self, Batch, Sprite};
use h3_view::compose::compose;
use h3_view::layout::{self, BattleLayout};
use h3_view::record::{self, Record};
use h3_view::scene::{self, Draw, Moment};
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
    /// The commands played.
    pub commands: usize,
    /// The archives' bytes until the batch is loaded (CLT-Q2: whole archives).
    archives: Vec<(&'static str, Vec<u8>)>,
    batch: Batch,
}

impl Battle {
    /// Fights the scenario (writing its record to `record_to`) or reads the record, and times
    /// it; a line on the record fought goes to `lines`.
    pub fn load(
        source: &Source,
        record_to: Option<&Path>,
        lines: &mut Vec<String>,
    ) -> Result<Self, String> {
        let text = if source.scenario() {
            let (text, took) =
                timed(|| crate::scenario::fought(&source.input, &source.packs, source.seed));
            let text = text?;
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
        let looks = motion::looks(&record, &tables, &parsed)?;
        let magic = motion::magic(&record, &tables, &parsed)?;
        let stage = motion::stage(&record, &tables, &layout, &magic);
        let commands = anim::commands(&record.messages);
        // The viewer's own fidgets, from the battle's seed (`h3_view::anim::Fidgets`).
        let seed = u32::try_from(source.seed & u64::from(u32::MAX)).unwrap_or(1);
        let timeline = anim::fidgeting(snapshot, &commands, &looks, stage, source.speed, seed)
            .map_err(|e| e.to_string())?
            .with_heroes(motion::heroes(&record, &tables, &parsed, Some(seed))?);
        let commands = commands.len();
        Ok(Self {
            record,
            tables,
            layout,
            timeline,
            commands,
            archives,
            batch: Batch::default(),
        })
    }

    /// The draw list of the battle at `moment`.
    pub fn list(&self, t: u32, moment: &Moment) -> Result<Vec<Draw>, String> {
        let snapshot = self
            .record
            .first_snapshot()
            .ok_or("the record holds no snapshot")?;
        scene::battle_at(
            snapshot,
            moment,
            &self.record.header.manifest,
            &self.tables,
            &self.layout,
        )
        .map_err(|e| format!("{t} ms: {e}"))
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
        compose(list, &self.batch, &self.layout).map_err(|e| format!("{t} ms: {e}"))
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
