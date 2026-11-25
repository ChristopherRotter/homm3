//! `generate`'s frame (ADR-0029 points 2, 7 and 8; survey `docs/rules/surveys/E14-generator.md`
//! §1.2 to §1.5): the run's adjustments, the seed, water's Random, the templates kept and drawn,
//! the banned heroes and the positions, each draw reported to a [`Trace`]. The fifteen steps and
//! the writer's content come with increments 3 to 7.

use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

use h3_content::{RMG_ROE_MONSTER_TOWN_BYTE, RMG_UNOWNED_START_SEVENTH, Ruleset};
use h3_kernel::rng::CrtRand;

use crate::part::RmgRuleset;
use crate::settings::{COLOURS, Settings, SettingsError, Water};
use crate::template::{Fit, KeptTemplate, PositionError, kept_templates, positions};

/// The generator's version (ADR-0029 point 7): bumped whenever the same rmg part, profile,
/// settings and seed can give another map.
pub const RMG_GENERATOR_VERSION: u32 = 1;

/// The call site of water's Random draw (`rmg_generator_create`, `rmg_water_draw`).
pub const SITE_WATER: u32 = 0x0053_7C89;

/// The call site of the template's draw (`rmg_generate_map`, `rmg_template_draw`).
pub const SITE_TEMPLATE: u32 = 0x0054_9975;

/// Where the generation reports its draws and steps (ADR-0029 point 8): the comparison seam with
/// the phase oracle (`docs/lanes/rmg/ORACLE.md`). [`NoTrace`] in play.
pub trait Trace {
    /// A `rand()` drawn at `site` (the call's address in `Heroes3.exe`) that returned `value`,
    /// as the phase oracle's log line `<site> <value>` gives it.
    fn draw(&mut self, site: u32, value: u32);
    /// The end of `step`: 0 the frame before step 1 (the setup, the template, the positions; the
    /// oracle's dump 0), 1 to 15 the steps.
    fn step_end(&mut self, step: u8);
}

/// The trace that keeps nothing.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct NoTrace;

impl Trace for NoTrace {
    #[inline]
    fn draw(&mut self, _site: u32, _value: u32) {}
    #[inline]
    fn step_end(&mut self, _step: u8) {}
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
        }
    }
}

/// A generated map: so far what the frame makes before step 1 (stub RMG-STUB-1: the steps and the
/// writer's content come with increments 3 to 7).
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
}

/// The stream: `CrtRand` seeded once, each draw reported with its call site.
struct Stream<'t, T: Trace> {
    rand: CrtRand,
    trace: &'t mut T,
}

impl<T: Trace> Stream<'_, T> {
    fn draw(&mut self, site: u32) -> u32 {
        let value = self.rand.rand();
        self.trace.draw(site, value);
        value
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
/// positions.
///
/// # Errors
///
/// A [`GenerateError`]: refused settings, a part of another core or without the generator's
/// tables, no template kept, the drawn template's positions outside the original's arrays.
// rule: RMG-SET-005, RMG-SET-006, RMG-SET-007, RMG-TPL-007
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
    stream.trace.step_end(0);
    // stub: RMG-STUB-1
    Ok(GeneratedMap {
        settings: run,
        seed,
        template,
        template_name: source.name.clone(),
        positions,
        banned_heroes,
    })
}
