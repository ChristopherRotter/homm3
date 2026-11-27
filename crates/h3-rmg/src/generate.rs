//! `generate`'s frame (ADR-0029 points 2, 7 and 8; survey `docs/rules/surveys/E14-generator.md`
//! §1.2 to §1.5): the run's adjustments, the seed, water's Random, the templates kept and drawn,
//! the banned heroes and the positions, the generator's map, then the steps (step 1 since E14
//! T3c, step 2 since T3d, step 3 since T3e), each draw and each step's end reported to a
//! [`Trace`]. Steps 4 to 15 and the writer's content come with increments 4 to 7.

use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

use h3_content::{
    RMG_COSMETIC, RMG_PARTITION_UNITS, RMG_ROE_MONSTER_TOWN_BYTE, RMG_TERRAIN_TOWNS_COUNT,
    RMG_UNOWNED_START_SEVENTH, RMG_ZONE_MEMORY_MODEL, Ruleset,
};
use h3_kernel::rng::CrtRand;

use crate::cells::Cells;
use crate::layout::{LayoutInput, ZoneGenerator, lay_out};
use crate::part::RmgRuleset;
use crate::partition::{
    MemoryZone, Partition, PartitionFault, PartitionInput, ZoneMemory, partition,
};
use crate::settings::{COLOURS, Settings, SettingsError, Water};
use crate::template::{Fit, KeptTemplate, PositionError, kept_templates, positions};
use crate::terrain::{TerrainFault, TerrainInput, paint};

/// The generator's version (ADR-0029 point 7): bumped whenever the same rmg part, profile,
/// settings and seed can give another map.
pub const RMG_GENERATOR_VERSION: u32 = 1;

/// The call site of water's Random draw (`rmg_generator_create`, `rmg_water_draw`).
pub const SITE_WATER: u32 = 0x0053_7C89;

/// The call site of the template's draw (`rmg_generate_map`, `rmg_template_draw`).
pub const SITE_TEMPLATE: u32 = 0x0054_9975;

/// What the generation holds at a step's end, as the comparison seam reads it (ADR-0029 point
/// 8): the generator's map (the oracle's dump of that step) and the zone generators (its
/// `layout.txt` after step 1; none before).
#[derive(Debug, Clone, Copy)]
pub struct StepState<'a> {
    /// The generator's map.
    pub cells: &'a Cells,
    /// The zone generators.
    pub zones: &'a [ZoneGenerator],
}

/// Where the generation reports its draws and steps (ADR-0029 point 8): the comparison seam with
/// the phase oracle (`docs/lanes/rmg/ORACLE.md`). [`NoTrace`] in play.
pub trait Trace {
    /// A `rand()` drawn at `site` (the call's address in `Heroes3.exe`) that returned `value`,
    /// as the phase oracle's log line `<site> <value>` gives it.
    fn draw(&mut self, site: u32, value: u32);
    /// The end of `step` and what it left: 0 the frame before step 1 (the setup, the template,
    /// the positions; the oracle's dump 0), 1 to 15 the steps.
    fn step_end(&mut self, step: u8, state: StepState<'_>);
    /// The bytes step 2's `index`-th zone (in the call order, from 0: each level's temporary
    /// zone, then its water zones) holds unwritten (RMG-QUIRK-1, in `faithful` only): none, the
    /// oracle model's own ([`ZoneMemory::TEMPORARY`], [`ZoneMemory::WATER`]). The comparison
    /// with the phase oracle gives the measured ones (`zones.txt`).
    fn zone_memory(&mut self, _kind: MemoryZone, _index: usize) -> Option<ZoneMemory> {
        None
    }
}

/// The trace that keeps nothing.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct NoTrace;

impl Trace for NoTrace {
    #[inline]
    fn draw(&mut self, _site: u32, _value: u32) {}
    #[inline]
    fn step_end(&mut self, _step: u8, _state: StepState<'_>) {}
}

/// Why a map is not generated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GenerateError {
    /// The settings are refused.
    Settings(SettingsError),
    /// The rmg part was compiled on another core ruleset.
    Core {
        /// The part's core hash.
        part: u64,
        /// The given core's.
        core: u64,
    },
    /// The rmg part has no generator's tables (a ruleset without `sod`, ADR-0029 point 5).
    NoGenerator,
    /// No template fits the settings (the original's code 3, GENRLTXT 752).
    NoTemplate,
    /// The drawn template's start positions index the original's arrays outside (RMG-QUIRK-4).
    Positions {
        /// The template's index in the part's table.
        template: usize,
        /// What went wrong.
        error: PositionError,
    },
    /// Step 1 found no position for a zone (RMG-QUIRK-3: the original's draw divides by zero).
    NoPosition {
        /// The zone's index among the kept zones.
        zone: usize,
    },
    /// Step 1's layout spans nothing (RMG-QUIRK-3: the original's scaling divides by zero).
    LayoutSpan,
    /// Step 2 met on `level` what the original divides by zero at (RMG-QUIRK-3) or reads or
    /// writes outside its arrays or never ends at (RMG-QUIRK-4).
    Partition {
        /// The level.
        level: i32,
        /// What went wrong.
        fault: PartitionFault,
    },
    /// Step 3 met what the original divides by zero at (RMG-QUIRK-3) or reads outside its
    /// arrays or never ends at (RMG-QUIRK-4).
    Terrain(TerrainFault),
}

impl From<SettingsError> for GenerateError {
    fn from(e: SettingsError) -> Self {
        Self::Settings(e)
    }
}

impl fmt::Display for GenerateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Settings(e) => write!(f, "settings: {e}"),
            Self::Core { part, core } => write!(
                f,
                "the rmg part was compiled on core {part:#018x}, not on {core:#018x}"
            ),
            Self::NoGenerator => f.write_str("the rmg part has no generator's tables"),
            Self::NoTemplate => f.write_str("no template fits the settings"),
            Self::Positions { template, error } => match error {
                PositionError::Outside { zone, position } => write!(
                    f,
                    "template {template}: zone {zone}'s start position {position} is outside 0 to 7"
                ),
                PositionError::Exhausted => write!(
                    f,
                    "template {template}: fewer start positions than the players and computers"
                ),
            },
            Self::NoPosition { zone } => write!(f, "step 1: no position for zone {zone}"),
            Self::LayoutSpan => f.write_str("step 1: the layout spans nothing"),
            Self::Partition { level, fault } => write!(f, "step 2, level {level}: {fault:?}"),
            Self::Terrain(fault) => write!(f, "step 3: {fault:?}"),
        }
    }
}

/// A generated map: so far what the frame and steps 1 to 3 make (stub RMG-STUB-1: steps 4 to 15
/// and the writer's content come with increments 4 to 7).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedMap {
    /// The settings as the generator ran them: water drawn, and under two players in all one
    /// player and one computer (RMG-SET-005).
    pub settings: Settings,
    /// The seed.
    pub seed: u32,
    /// The drawn template as kept.
    pub template: KeptTemplate,
    /// Its name, line breaks as read.
    pub template_name: String,
    /// Each start position's colour.
    pub positions: [Option<u8>; COLOURS],
    /// Each hero's ban, by id (RMG-SET-008).
    pub banned_heroes: Vec<bool>,
    /// The generator's map after the last step ported.
    pub cells: Cells,
    /// The zone generators after the last step ported (step 2's water zones appended).
    pub zones: Vec<ZoneGenerator>,
    /// Step 2's polygons and distances.
    pub partition: Partition,
}

/// The stream: `CrtRand` seeded once, each draw reported with its call site.
pub(crate) struct Stream<'t, T: Trace> {
    rand: CrtRand,
    trace: &'t mut T,
}

impl<'t, T: Trace> Stream<'t, T> {
    pub(crate) fn draw(&mut self, site: u32) -> u32 {
        let value = self.rand.rand();
        self.trace.draw(site, value);
        value
    }

    pub(crate) fn zone_memory(&mut self, kind: MemoryZone, index: usize) -> Option<ZoneMemory> {
        self.trace.zone_memory(kind, index)
    }

    /// A stream seeded with `seed` reporting to `trace` (the steps' unit tests).
    #[cfg(test)]
    pub(crate) fn new(seed: u32, trace: &'t mut T) -> Self {
        Self {
            rand: CrtRand::new(seed),
            trace,
        }
    }
}

/// Generates a map from the rmg part `part` compiled on `core` (whose profile's flags apply), the
/// settings and the seed (the original's `time()` in seconds), reporting to `trace`. Pure; never
/// panics.
///
/// The frame, as `rmg_generate_and_write` 0x54BF60, `rmg_generator_create` 0x537B10 and
/// `rmg_generate_map` 0x549930 run it: under two players in all, one player and one computer;
/// `srand(seed)`; water's Random `rand() % 3` (0x537C89); the templates kept for the size
/// index and the counts; the heroes banned; the template `rand() % kept` (0x549975); the
/// positions; the generator's map (`rmg_map_init`); step 1, the zones laid out
/// (`rmg_zones_lay_out` 0x53BCB0).
///
/// # Errors
///
/// A [`GenerateError`]: refused settings, a part of another core or without the generator's
/// tables, no template kept, the drawn template's positions outside the original's arrays, step
/// 1's divisions by zero (RMG-QUIRK-3).
// rule: RMG-SET-005, RMG-SET-006, RMG-SET-007, RMG-SET-010, RMG-TPL-007
pub fn generate<T: Trace>(
    core: &Ruleset,
    part: &RmgRuleset,
    settings: &Settings,
    seed: u32,
    trace: &mut T,
) -> Result<GeneratedMap, GenerateError> {
    settings.check()?;
    if part.core_hash() != core.content_hash() {
        return Err(GenerateError::Core {
            part: part.core_hash(),
            core: core.content_hash(),
        });
    }
    let tables = part.generator().ok_or(GenerateError::NoGenerator)?;
    let quirks = core.quirks();
    let mut run = *settings;
    if run.players + run.computers < 2 {
        run.players = 1;
        run.computers = 1;
    }
    let mut stream = Stream {
        rand: CrtRand::new(seed),
        trace,
    };
    if run.water == Water::Random {
        run.water = match stream.draw(SITE_WATER) % 3 {
            0 => Water::None,
            1 => Water::Normal,
            _ => Water::Islands,
        };
    }
    let fit = Fit::new(
        run.size.side(),
        1 + i32::from(run.two_levels),
        run.water,
        i32::from(run.players),
        i32::from(run.computers),
    );
    let kept = kept_templates(part.templates(), fit);
    if kept.is_empty() {
        return Err(GenerateError::NoTemplate);
    }
    let banned_heroes = tables.banned_heroes(run.format);
    // `rand()` is at most 0x7FFF; the kept indices are of the part's table.
    let index = kept[stream.draw(SITE_TEMPLATE) as usize % kept.len()];
    let source = &part.templates()[index];
    let template = KeptTemplate::new(
        index,
        source,
        fit,
        run.format,
        quirks.is_enabled(RMG_ROE_MONSTER_TOWN_BYTE),
    );
    let positions = positions(
        &template.zones,
        &run.humans,
        usize::from(run.players),
        usize::from(run.computers),
        quirks.is_enabled(RMG_UNOWNED_START_SEVENTH),
    )
    .map_err(|error| GenerateError::Positions {
        template: index,
        error,
    })?;
    let side = run.size.side();
    let levels = 1 + i32::from(run.two_levels);
    let mut cells = Cells::new(side, levels);
    stream.trace.step_end(
        0,
        StepState {
            cells: &cells,
            zones: &[],
        },
    );
    let mut zones = lay_out(
        &mut stream,
        &LayoutInput {
            zones: &template.zones,
            connections: &template.connections,
            positions: &positions,
            towns: &run.towns,
            side,
            levels,
            water: run.water,
            format: run.format,
            tables,
            all_four: quirks.is_enabled(RMG_TERRAIN_TOWNS_COUNT),
        },
    )?;
    stream.trace.step_end(
        1,
        StepState {
            cells: &cells,
            zones: &zones,
        },
    );
    let mut template = template;
    let partition = partition(
        &mut stream,
        &PartitionInput {
            levels,
            water: run.water,
            tables,
            memory_model: quirks.is_enabled(RMG_ZONE_MEMORY_MODEL),
            partition_units: quirks.is_enabled(RMG_PARTITION_UNITS),
        },
        &mut template.zones,
        &mut template.connections,
        &mut zones,
        &mut cells,
    )?;
    stream.trace.step_end(
        2,
        StepState {
            cells: &cells,
            zones: &zones,
        },
    );
    paint(
        &mut stream,
        &TerrainInput {
            water: run.water,
            cosmetic: quirks.is_enabled(RMG_COSMETIC),
            tables,
            zones: &template.zones,
            polygons: &partition.polygons,
        },
        &mut zones,
        &mut cells,
    )?;
    stream.trace.step_end(
        3,
        StepState {
            cells: &cells,
            zones: &zones,
        },
    );
    // stub: RMG-STUB-1
    Ok(GeneratedMap {
        settings: run,
        seed,
        template,
        template_name: source.name.clone(),
        positions,
        banned_heroes,
        cells,
        zones,
        partition,
    })
}
