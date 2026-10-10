//! Adapter: parses the packs' `adventure.toml` files (one pack file per context, ADR-0006's E08
//! amendment) into the adventure context's definitions, which `h3-adventure` compiles into its
//! ruleset part (ADR-0015 point 2). The files are hand-written source the harvester never touches
//! (E08 decision 10); parsing is strict like `h3-pack`'s (unknown fields are errors, numbers must
//! fit their types), and the compiler validates everything again.
//!
//! # Format (E08 T4)
//!
//! ```toml
//! [params]                         # exactly one enabled pack provides them
//! native_cost = 100                # a step on terrain native to the whole army (ADV-MOV-004)
//! land_points = [1300, 1360, 1430] # a day's points on land by the slowest speed, from speed 0;
//!                                  # faster armies take the last entry (ADV-MOV-001)
//! scouting_radius = 5              # without bonuses (ADV-FOG-001)
//!
//! [[terrain]]                      # every core terrain needs one entry, from any pack of
//! terrain = "grass"                # its closure; `name` means `<own pack>:name`
//! cost = 100                       # a straight step (ADV-MOV-003)
//! # passage = "land"               # land (default) or water need a cost; impassable has none
//!
//! [[road]]
//! id = "dirt_road"
//! name = "Dirt road"
//! cost = 75                        # a straight step along it (ADV-MOV-003)
//!
//! [[river]]                        # appearance only
//! id = "clear_river"
//! name = "Clear river"
//! ```
//!
//! Costs are at least 1. A pack without the file has an empty adventure part; every enabled pack
//! has one, so the part shares the core's load order and dependency closures.

use std::fmt;
use std::num::NonZeroU16;

use h3_adventure::{
    AdventureCompileError, AdventurePack, AdventureParamsDef, AdventureRuleset, Passage, RiverDef,
    RoadDef, TerrainMovementDef,
};
use h3_content::{ContentKey, KeyError, PackManifest, Ruleset};
use h3_pack::PackError;
use serde::Deserialize;

/// The synthetic pack's adventure file (tests).
pub const SYNTHETIC_ADVENTURE: &str = include_str!("../../../packs/synthetic/adventure.toml");
/// The synthetic expansion's adventure file (tests).
pub const SYNTHETIC_EXT_ADVENTURE: &str =
    include_str!("../../../packs/synthetic-ext/adventure.toml");
/// The Restoration of Erathia's adventure file.
pub const BASE_ADVENTURE: &str = include_str!("../../../packs/base/adventure.toml");
/// Armageddon's Blade's adventure file.
pub const AB_ADVENTURE: &str = include_str!("../../../packs/ab/adventure.toml");
/// The Shadow of Death's adventure file.
pub const SOD_ADVENTURE: &str = include_str!("../../../packs/sod/adventure.toml");

/// Errors of this adapter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdventurePackError {
    /// The text is not valid TOML for the expected schema.
    Syntax(String),
    /// A key or id is malformed.
    Key(KeyError),
    /// A terrain entry's cost does not fit its passage (land and water need one, impassable
    /// has none); the payload is the terrain as written.
    TerrainCost(String),
    /// A core pack does not parse, or the core packs do not compile.
    Core(PackError),
    /// The adventure packs do not compile into the adventure part.
    Compile(AdventureCompileError),
}

impl fmt::Display for AdventurePackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Syntax(e) => write!(f, "syntax: {e}"),
            Self::Key(e) => write!(f, "{e}"),
            Self::TerrainCost(t) => write!(
                f,
                "terrain `{t}`: land and water need a cost, an impassable terrain has none"
            ),
            Self::Core(e) => write!(f, "core packs: {e}"),
            Self::Compile(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for AdventurePackError {}

impl From<KeyError> for AdventurePackError {
    fn from(e: KeyError) -> Self {
        Self::Key(e)
    }
}

impl From<PackError> for AdventurePackError {
    fn from(e: PackError) -> Self {
        Self::Core(e)
    }
}

impl From<AdventureCompileError> for AdventurePackError {
    fn from(e: AdventureCompileError) -> Self {
        Self::Compile(e)
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AdventureFile {
    params: Option<ParamsDto>,
    #[serde(default)]
    terrain: Vec<TerrainDto>,
    #[serde(default)]
    road: Vec<RoadDto>,
    #[serde(default)]
    river: Vec<RiverDto>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ParamsDto {
    native_cost: NonZeroU16,
    land_points: Vec<u32>,
    scouting_radius: u8,
}

#[derive(Deserialize, Default, Clone, Copy)]
#[serde(rename_all = "snake_case")]
enum PassageDto {
    #[default]
    Land,
    Water,
    Impassable,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TerrainDto {
    terrain: String,
    #[serde(default)]
    passage: PassageDto,
    cost: Option<NonZeroU16>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RoadDto {
    id: String,
    name: String,
    cost: NonZeroU16,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RiverDto {
    id: String,
    name: String,
}

/// Resolves a reference: `name` means `<own pack>:name`, `pack:name` is taken as written.
fn reference(pack: &str, text: &str) -> Result<ContentKey, KeyError> {
    if text.contains(':') {
        ContentKey::parse(text)
    } else {
        ContentKey::new(pack, text)
    }
}

/// Parses the `adventure.toml` of the core pack with this manifest (`""` for a pack without the
/// file).
pub fn parse_adventure(
    manifest: &PackManifest,
    text: &str,
) -> Result<AdventurePack, AdventurePackError> {
    let file: AdventureFile =
        toml::from_str(text).map_err(|e| AdventurePackError::Syntax(e.to_string()))?;
    let pack = manifest.id.as_str();
    let terrains = file
        .terrain
        .into_iter()
        .map(|t| {
            let passage = match (t.passage, t.cost) {
                (PassageDto::Land, Some(cost)) => Passage::Land(cost),
                (PassageDto::Water, Some(cost)) => Passage::Water(cost),
                (PassageDto::Impassable, None) => Passage::Impassable,
                _ => return Err(AdventurePackError::TerrainCost(t.terrain)),
            };
            Ok(TerrainMovementDef {
                terrain: reference(pack, &t.terrain)?,
                passage,
            })
        })
        .collect::<Result<_, AdventurePackError>>()?;
    let roads = file
        .road
        .into_iter()
        .map(|r| {
            Ok(RoadDef {
                key: ContentKey::new(pack, &r.id)?,
                name: r.name,
                cost: r.cost,
            })
        })
        .collect::<Result<_, KeyError>>()?;
    let rivers = file
        .river
        .into_iter()
        .map(|r| {
            Ok(RiverDef {
                key: ContentKey::new(pack, &r.id)?,
                name: r.name,
            })
        })
        .collect::<Result<_, KeyError>>()?;
    Ok(AdventurePack {
        terrains,
        roads,
        rivers,
        params: file.params.map(|p| AdventureParamsDef {
            native_cost: p.native_cost,
            land_points: p.land_points,
            scouting_radius: p.scouting_radius,
        }),
        ..AdventurePack::new(manifest.clone())
    })
}

/// Parses and compiles packs given as `(pack.toml, adventure.toml)` texts (`""` for a pack
/// without an adventure file) with the profile `profile`: the core ruleset and its adventure part.
pub fn compile(
    packs: &[(&str, &str)],
    profile: &str,
) -> Result<(Ruleset, AdventureRuleset), AdventurePackError> {
    let core_packs = packs
        .iter()
        .map(|(core, _)| h3_pack::parse_pack(core))
        .collect::<Result<Vec<_>, _>>()?;
    let core = Ruleset::compile(&core_packs, profile).map_err(PackError::from)?;
    let adventure = core_packs
        .iter()
        .zip(packs)
        .map(|(core_pack, (_, text))| parse_adventure(&core_pack.manifest, text))
        .collect::<Result<Vec<_>, _>>()?;
    let part = AdventureRuleset::compile(&core, &adventure)?;
    Ok((core, part))
}

/// The three original packs, RoE to SoD, as `(pack.toml, adventure.toml)` texts.
pub const ORIGINAL_PACKS: [(&str, &str); 3] = [
    (h3_pack::BASE_PACK, BASE_ADVENTURE),
    (h3_pack::AB_PACK, AB_ADVENTURE),
    (h3_pack::SOD_PACK, SOD_ADVENTURE),
];

#[cfg(test)]
mod tests;
