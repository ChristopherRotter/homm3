//! Adapter: parses the packs' `rmg.toml` files into the map generation context's tables, which
//! `h3-rmg` compiles into its rmg part (ADR-0029 point 5): [`compile`] over the enabled packs'
//! files in the core's load order; a pack folder without the file reads as an empty one.
//! `packs/ab/rmg.toml` holds AB's three tables as the game reads them (`rmg.txt`'s templates,
//! `objects.txt`'s template lines, `rand_trn.txt`'s rows), `packs/sod/rmg.toml` SoD's additions
//! (three templates; `objects.txt`'s added and changed lines) and the generator's constants read
//! from SoD's executable (`[generator]`, E14 T2c); both are harvested by `h3-harvest` (E14 T2b,
//! [`ORIGINAL_PACKS`]), each `objects.txt` line with its picture's `.msk` (E14 T4b). The format: `packs/README.md`, "The map generator's tables".
//! Parsing is strict like `h3-pack`'s (unknown fields are errors, numbers must fit their types, a
//! mark string has its length and only `0` and `1`); the compiler checks the references.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use std::fmt;

use h3_content::{PackManifest, Ruleset};
use h3_pack::PackError;
use h3_rmg::{
    Connection, ConnectionOverride, GeneratorTables, MAX_PICTURES, OBJECT_TYPES, ObjectSettings,
    ObjectTemplate, ObjectTemplateLine, ObjectTemplateOverride, Obstacle, PictureSet,
    RmgCompileError, RmgPack, RmgRuleset, RockPicture, Strength, Template, Zone, ZoneKind,
};
use serde::Deserialize;

/// The synthetic pack's generator tables (tests; invented names).
pub const SYNTHETIC_RMG: &str = include_str!("../../../packs/synthetic/rmg.toml");
/// The AB pack's generator tables (harvested, E14 T2b): AB's three tables as the game reads them.
pub const AB_RMG: &str = include_str!("../../../packs/ab/rmg.toml");
/// The SoD pack's generator tables (harvested): what SoD's tables add to AB's.
pub const SOD_RMG: &str = include_str!("../../../packs/sod/rmg.toml");

/// The original packs' `(core.toml, rmg.toml)` texts in load order (`base` has none), for
/// [`compile`]: SoD's tables, the ones the game reads.
pub const ORIGINAL_PACKS: [(&str, &str); 3] = [
    (h3_pack::BASE_CORE, ""),
    (h3_pack::AB_CORE, AB_RMG),
    (h3_pack::SOD_CORE, SOD_RMG),
];

/// Errors of this adapter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RmgPackError {
    /// The text is not valid TOML for the expected schema.
    Syntax(String),
    /// A value the schema cannot check: a mark string's length or characters, or an
    /// `objects.txt` line's `before` or `line` in the wrong list.
    Invalid(String),
    /// A core pack does not parse, or the core packs do not compile.
    Core(PackError),
    /// The rmg packs do not compile into the rmg part.
    Compile(RmgCompileError),
}

impl fmt::Display for RmgPackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Syntax(e) => write!(f, "syntax: {e}"),
            Self::Invalid(e) => write!(f, "{e}"),
            Self::Core(e) => write!(f, "core packs: {e}"),
            Self::Compile(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for RmgPackError {}

impl From<PackError> for RmgPackError {
    fn from(e: PackError) -> Self {
        Self::Core(e)
    }
}

impl From<RmgCompileError> for RmgPackError {
    fn from(e: RmgCompileError) -> Self {
        Self::Compile(e)
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RmgFile {
    #[serde(default)]
    object_templates: Vec<ObjectDto>,
    #[serde(default)]
    object_template_overrides: Vec<ObjectDto>,
    #[serde(default)]
    template: Vec<TemplateDto>,
    #[serde(default)]
    connection_override: Vec<ConnectionOverrideDto>,
    obstacle: Option<Vec<ObstacleDto>>,
    generator: Option<GeneratorDto>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ObjectDto {
    before: Option<usize>,
    line: Option<usize>,
    picture: String,
    passability: String,
    actions: String,
    editor_terrains: String,
    terrains: String,
    class: u32,
    subclass: u32,
    group: u8,
    overlay: u8,
    width: u8,
    height: u8,
    colour: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TemplateDto {
    name: String,
    sizes: [i32; 2],
    #[serde(default)]
    zones: Vec<ZoneDto>,
    #[serde(default)]
    connections: Vec<ConnectionDto>,
}

#[derive(Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
enum KindDto {
    HumanStart,
    ComputerStart,
    Treasure,
    Junction,
}

#[derive(Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
enum StrengthDto {
    None,
    Weak,
    Average,
    Strong,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ZoneDto {
    id: i32,
    kind: KindDto,
    base_size: i32,
    humans: [i32; 2],
    players: [i32; 2],
    ownership: i32,
    player_towns: [i32; 4],
    neutral_towns: [i32; 4],
    same_type: bool,
    town_types: String,
    mines: [i32; 7],
    mine_density: [i32; 7],
    terrain_match: bool,
    terrains: String,
    strength: StrengthDto,
    monster_match: bool,
    neutral_monsters: bool,
    monster_towns: String,
    treasure: [[i32; 3]; 3],
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ConnectionDto {
    zones: [i32; 2],
    value: i32,
    wide: bool,
    border_guard: bool,
    humans: [i32; 2],
    players: [i32; 2],
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ConnectionOverrideDto {
    template: usize,
    connection: usize,
    fixes: Option<String>,
    zones: [i32; 2],
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ObstacleDto {
    #[serde(rename = "type")]
    object_type: i32,
    subtype: i32,
    terrain: i32,
    modifiers: [i32; 9],
    adjacent: Vec<i32>,
    overlap: Vec<i32>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct GeneratorDto {
    heroes_roe: String,
    heroes_expansions: String,
    heroes_campaign: String,
    map_limits: Vec<(u8, i32)>,
    zone_limits: Vec<(u8, i32)>,
    layout_cos: Vec<i32>,
    layout_sin: Vec<i32>,
    terrain_towns: Vec<[i32; 4]>,
    native_terrains: Vec<i32>,
    picture_sets: Vec<PictureSetDto>,
    rock_pictures: Vec<(u8, u8, u8, usize)>,
    object_not_entered: String,
    object_from_above: String,
    object_removable: String,
    object_class_groups: Vec<(u8, u8)>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PictureSetDto {
    through_dirt: bool,
    land: bool,
    share: u8,
    pictures: Vec<(u8, u8, usize)>,
}

/// A 0 or 1 as a flag.
fn bit(value: u8, what: &str) -> Result<bool, RmgPackError> {
    match value {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(RmgPackError::Invalid(format!(
            "{what} {value} is not 0 or 1"
        ))),
    }
}

/// Runs of an entry and its count expanded in order, at most [`MAX_PICTURES`] entries.
fn expand<T: Copy>(runs: &[(T, usize)], what: &str) -> Result<Vec<T>, RmgPackError> {
    let mut out = Vec::new();
    for &(entry, count) in runs {
        if count > MAX_PICTURES - out.len() {
            return Err(RmgPackError::Invalid(format!(
                "{what} hold more than {MAX_PICTURES} pictures"
            )));
        }
        out.extend(core::iter::repeat_n(entry, count));
    }
    Ok(out)
}

/// The painter's picture sets and rock's pictures from their runs.
fn pictures(g: &GeneratorDto) -> Result<(Vec<PictureSet>, Vec<RockPicture>), RmgPackError> {
    let sets = g
        .picture_sets
        .iter()
        .map(|set| {
            let runs = set
                .pictures
                .iter()
                .map(|&(shape, flag, n)| {
                    Ok(((shape, bit(flag, "generator.picture_sets' flag")?), n))
                })
                .collect::<Result<Vec<_>, RmgPackError>>()?;
            Ok(PictureSet {
                through_dirt: set.through_dirt,
                land: set.land,
                share: set.share,
                pictures: expand(&runs, "generator.picture_sets")?,
            })
        })
        .collect::<Result<_, RmgPackError>>()?;
    let runs = g
        .rock_pictures
        .iter()
        .map(|&(shape, a, b, n)| {
            let variant = |v| bit(v, "generator.rock_pictures' variant");
            Ok(((shape, [variant(a)?, variant(b)?]), n))
        })
        .collect::<Result<Vec<_>, RmgPackError>>()?;
    Ok((sets, expand(&runs, "generator.rock_pictures")?))
}

/// The object settings by class (E14 T4b): three strings of one mark per class, [`OBJECT_TYPES`]
/// each, and the classes whose group is not their own as pairs of class and group, in order (a
/// class given twice keeps its last).
fn object_settings(g: &GeneratorDto) -> Result<Vec<ObjectSettings>, RmgPackError> {
    let per_class = |text: &str, what: &str| {
        let marks = flags(text, what)?;
        if marks.len() == OBJECT_TYPES {
            Ok(marks)
        } else {
            Err(RmgPackError::Invalid(format!(
                "{what} has {} marks, not one per class ({OBJECT_TYPES})",
                marks.len()
            )))
        }
    };
    let not_entered = per_class(&g.object_not_entered, "generator.object_not_entered")?;
    let from_above = per_class(&g.object_from_above, "generator.object_from_above")?;
    let removable = per_class(&g.object_removable, "generator.object_removable")?;
    let mut out: Vec<ObjectSettings> = (0..=u8::MAX)
        .zip(not_entered.iter().zip(&from_above).zip(&removable))
        .map(
            |(class_group, ((&not_entered, &from_above), &removable))| ObjectSettings {
                not_entered,
                from_above,
                removable,
                class_group,
            },
        )
        .collect();
    for &(class, group) in &g.object_class_groups {
        let settings = out.get_mut(usize::from(class)).ok_or_else(|| {
            RmgPackError::Invalid(format!(
                "generator.object_class_groups names class {class}, past {}",
                OBJECT_TYPES - 1
            ))
        })?;
        settings.class_group = group;
    }
    Ok(out)
}

/// A string of one mark per entry, each `0` or `1`, as flags.
fn flags(text: &str, what: &str) -> Result<Vec<bool>, RmgPackError> {
    text.chars()
        .map(|c| match c {
            '0' => Ok(false),
            '1' => Ok(true),
            _ => Err(RmgPackError::Invalid(format!(
                "{what} `{text}` is not of 0 and 1"
            ))),
        })
        .collect()
}

/// A mark string as a bit set: exactly `len` characters, each `0` or `1`; bit `k` is the `k`-th.
fn marks(text: &str, len: usize, what: &str) -> Result<u64, RmgPackError> {
    let invalid = || {
        RmgPackError::Invalid(format!(
            "{what} `{text}` is not {len} characters of 0 and 1"
        ))
    };
    if text.len() != len {
        return Err(invalid());
    }
    text.bytes()
        .enumerate()
        .try_fold(0u64, |bits, (k, c)| match c {
            b'0' => Ok(bits),
            b'1' => Ok(bits | 1 << k),
            _ => Err(invalid()),
        })
}

/// A mark string as a bit set of a narrower type.
fn narrow_marks<T: TryFrom<u64>>(text: &str, len: usize, what: &str) -> Result<T, RmgPackError> {
    T::try_from(marks(text, len, what)?)
        .map_err(|_| RmgPackError::Invalid(format!("{what} `{text}` has too many marks")))
}

/// An `objects.txt` line with its picture's `.msk`: a footprint of 1 to 8 columns and 1 to 6 rows
/// (the masks' cells).
fn object(o: ObjectDto) -> Result<ObjectTemplate, RmgPackError> {
    let what = |field: &str| format!("object template `{}`'s {field}", o.picture);
    if !(1..=8).contains(&o.width) || !(1..=6).contains(&o.height) {
        return Err(RmgPackError::Invalid(format!(
            "{} {} x {} is not within the masks' 8 x 6",
            what("footprint"),
            o.width,
            o.height
        )));
    }
    Ok(ObjectTemplate {
        passability: marks(&o.passability, 48, &what("passability"))?,
        actions: marks(&o.actions, 48, &what("actions"))?,
        editor_terrains: narrow_marks(&o.editor_terrains, 9, &what("editor_terrains"))?,
        terrains: narrow_marks(&o.terrains, 9, &what("terrains"))?,
        class: o.class,
        subclass: o.subclass,
        group: o.group,
        overlay: o.overlay,
        width: o.width,
        height: o.height,
        colour: marks(&o.colour, 48, &what("colour"))?,
        picture: o.picture,
    })
}

fn zone(template: &str, z: ZoneDto) -> Result<Zone, RmgPackError> {
    let what = |field: &str| format!("template `{template}` zone {}'s {field}", z.id);
    Ok(Zone {
        id: z.id,
        kind: match z.kind {
            KindDto::HumanStart => ZoneKind::HumanStart,
            KindDto::ComputerStart => ZoneKind::ComputerStart,
            KindDto::Treasure => ZoneKind::Treasure,
            KindDto::Junction => ZoneKind::Junction,
        },
        base_size: z.base_size,
        humans: z.humans,
        players: z.players,
        ownership: z.ownership,
        player_towns: z.player_towns,
        neutral_towns: z.neutral_towns,
        same_type: z.same_type,
        town_types: narrow_marks(&z.town_types, 9, &what("town_types"))?,
        mines: z.mines,
        mine_density: z.mine_density,
        terrain_match: z.terrain_match,
        terrains: narrow_marks(&z.terrains, 8, &what("terrains"))?,
        strength: match z.strength {
            StrengthDto::None => Strength::None,
            StrengthDto::Weak => Strength::Weak,
            StrengthDto::Average => Strength::Average,
            StrengthDto::Strong => Strength::Strong,
        },
        monster_match: z.monster_match,
        neutral_monsters: z.neutral_monsters,
        monster_towns: narrow_marks(&z.monster_towns, 9, &what("monster_towns"))?,
        treasure: z.treasure,
    })
}

fn template(t: TemplateDto) -> Result<Template, RmgPackError> {
    let zones = t
        .zones
        .into_iter()
        .map(|z| zone(&t.name, z))
        .collect::<Result<_, _>>()?;
    let connections = t
        .connections
        .into_iter()
        .map(|c| Connection {
            zones: c.zones,
            value: c.value,
            wide: c.wide,
            border_guard: c.border_guard,
            humans: c.humans,
            players: c.players,
        })
        .collect();
    Ok(Template {
        name: t.name,
        sizes: t.sizes,
        zones,
        connections,
    })
}

/// Parses one pack's `rmg.toml` (`""` for a pack folder without one) for the core pack of
/// `manifest`.
///
/// # Errors
///
/// [`RmgPackError::Syntax`] for text that does not fit the schema, [`RmgPackError::Invalid`] for a
/// malformed mark string, a footprint past the masks, object settings not one per class, or an
/// `objects.txt` line with `line` among the added lines or `before` (or no `line`) among the
/// replaced ones.
pub fn parse_rmg(manifest: &PackManifest, text: &str) -> Result<RmgPack, RmgPackError> {
    let file: RmgFile = toml::from_str(text).map_err(|e| RmgPackError::Syntax(e.to_string()))?;
    let misplaced = |o: &ObjectDto, list: &str| {
        RmgPackError::Invalid(format!(
            "object template `{}` in `{list}`: added lines take `before`, replaced ones `line`",
            o.picture
        ))
    };
    let object_templates = file
        .object_templates
        .into_iter()
        .map(|o| match o.line {
            Some(_) => Err(misplaced(&o, "object_templates")),
            None => Ok(ObjectTemplateLine {
                before: o.before,
                template: object(o)?,
            }),
        })
        .collect::<Result<_, _>>()?;
    let object_template_overrides = file
        .object_template_overrides
        .into_iter()
        .map(|o| match (o.line, o.before) {
            (Some(line), None) => Ok(ObjectTemplateOverride {
                line,
                template: object(o)?,
            }),
            _ => Err(misplaced(&o, "object_template_overrides")),
        })
        .collect::<Result<_, _>>()?;
    let templates = file
        .template
        .into_iter()
        .map(template)
        .collect::<Result<_, _>>()?;
    let connection_overrides = file
        .connection_override
        .into_iter()
        .map(|o| ConnectionOverride {
            template: o.template,
            connection: o.connection,
            fixes: o.fixes,
            zones: o.zones,
        })
        .collect();
    let obstacles = file.obstacle.map(|rows| {
        rows.into_iter()
            .map(|o| Obstacle {
                object_type: o.object_type,
                subtype: o.subtype,
                terrain: o.terrain,
                modifiers: o.modifiers,
                adjacent: o.adjacent,
                overlap: o.overlap,
            })
            .collect()
    });
    let generator = file
        .generator
        .map(|g| {
            let (picture_sets, rock_pictures) = pictures(&g)?;
            let object_settings = object_settings(&g)?;
            Ok::<_, RmgPackError>(GeneratorTables {
                heroes_roe: flags(&g.heroes_roe, "generator.heroes_roe")?,
                heroes_expansions: flags(&g.heroes_expansions, "generator.heroes_expansions")?,
                heroes_campaign: flags(&g.heroes_campaign, "generator.heroes_campaign")?,
                map_limits: g.map_limits,
                zone_limits: g.zone_limits,
                layout_cos: g.layout_cos,
                layout_sin: g.layout_sin,
                terrain_towns: g.terrain_towns,
                native_terrains: g.native_terrains,
                picture_sets,
                rock_pictures,
                object_settings,
            })
        })
        .transpose()?;
    Ok(RmgPack {
        manifest: manifest.clone(),
        templates,
        connection_overrides,
        object_templates,
        object_template_overrides,
        obstacles,
        generator,
    })
}

/// Compiles the core packs and the rmg part from `(core.toml, rmg.toml)` texts in load order
/// (`""` for a pack folder without an `rmg.toml`), with the profile `profile`.
///
/// # Errors
///
/// [`RmgPackError::Core`] if a core pack does not parse or the core does not compile, the parse
/// errors of [`parse_rmg`], [`RmgPackError::Compile`] if the rmg packs do not compile.
pub fn compile(packs: &[(&str, &str)], profile: &str) -> Result<RmgRuleset, RmgPackError> {
    let core_packs = packs
        .iter()
        .map(|(core, _)| h3_pack::parse_pack(core))
        .collect::<Result<Vec<_>, _>>()?;
    let rmg = core_packs
        .iter()
        .zip(packs)
        .map(|(core, (_, text))| parse_rmg(&core.manifest, text))
        .collect::<Result<Vec<_>, _>>()?;
    let core = Ruleset::compile(&core_packs, profile).map_err(PackError::from)?;
    Ok(RmgRuleset::compile(&core, &rmg)?)
}

#[cfg(test)]
mod tests;
