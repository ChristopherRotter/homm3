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
//! max_map_size = 144               # setup bounds (E08 T5a): the widest and highest map,
//! max_levels = 2                   # its levels and the players of a game
//! max_players = 8
//! army_slots = 7                   # the slots of a hero's army
//! starting_stack_chances = [88, 25] # percent for the 2nd, 3rd... starting slot of a hero's
//!                                  # starting army; the 1st always (GAM-SET-001, E08 T6b)
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
//!
//! [[object_type]]                  # E08 T12a; ADR-0016 point 4, ADV-OBJ-002
//! id = "pile_gold"
//! name = "Gold"
//! visit = "neighbour"              # or "enter"
//! from_above = true                # visitable from the three tiles above
//! # exactly one kind table; resources, terrains and types: `name` means `<own pack>:name`
//! pickup = { resource = "gold", amount = [5, 10], multiplier = 100 } # multiplier: 1 if left out
//! # pickup_one_of = ["pile_wood", "pile_ore"]                       # a type drawn at setup
//! # pickup_paired = { amount = [4, 6], resources = ["wood"], paired = "gold", multiplier = 100 }
//! # reveal_terrain = { terrains = ["water"], price = { gold = 1000 } }
//! # reveal_area = { radius = 20 }
//! # key_tent = { key = 0 }         # also key_guard, key_gate (from_above = true); keys 0-7
//!
//! [[difficulty]]                   # E08 T13a, GAM-RES-002: the starting resources by the game's
//!                                  # difficulty, from 0 (easy); exactly one enabled pack gives
//!                                  # them; resources as above, a resource left out starts at 0
//! human = { wood = 30, gold = 30000 }
//! computer = { wood = 5, gold = 5000 }
//! ```
//!
//! Costs are at least 1. A pack without the file has an empty adventure part; every enabled pack
//! has one, so the part shares the core's load order and dependency closures.

use std::fmt;
use std::num::{NonZeroU8, NonZeroU16};

use std::collections::BTreeMap;
use std::num::NonZeroU32;

use h3_adventure::{
    AdventureCompileError, AdventurePack, AdventureParamsDef, AdventureRuleset, ObjectKindDef,
    ObjectTypeDef, Passage, PickupDef, RiverDef, RoadDef, StartingResourcesDef, TerrainMovementDef,
    Visit,
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
    /// An object type does not have exactly one kind table; the payload is its id as written.
    ObjectKind(String),
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
            Self::ObjectKind(id) => {
                write!(f, "object type `{id}` needs exactly one kind table")
            }
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
    #[serde(default)]
    object_type: Vec<ObjectTypeDto>,
    #[serde(default)]
    difficulty: Vec<DifficultyDto>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DifficultyDto {
    human: BTreeMap<String, u32>,
    computer: BTreeMap<String, u32>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ParamsDto {
    native_cost: NonZeroU16,
    land_points: Vec<u32>,
    scouting_radius: u8,
    max_map_size: NonZeroU16,
    max_levels: NonZeroU8,
    max_players: NonZeroU8,
    army_slots: NonZeroU8,
    starting_stack_chances: Vec<u8>,
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

#[derive(Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
enum VisitDto {
    Enter,
    Neighbour,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ObjectTypeDto {
    id: String,
    name: String,
    visit: VisitDto,
    from_above: bool,
    pickup: Option<PickupDto>,
    pickup_one_of: Option<Vec<String>>,
    pickup_paired: Option<PairedDto>,
    reveal_terrain: Option<RevealTerrainDto>,
    reveal_area: Option<RevealAreaDto>,
    key_tent: Option<KeyDto>,
    key_guard: Option<KeyDto>,
    key_gate: Option<KeyDto>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PickupDto {
    resource: String,
    amount: [u32; 2],
    multiplier: Option<NonZeroU32>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PairedDto {
    amount: [u32; 2],
    resources: Vec<String>,
    paired: String,
    multiplier: Option<NonZeroU32>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RevealTerrainDto {
    terrains: Vec<String>,
    #[serde(default)]
    price: BTreeMap<String, u32>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RevealAreaDto {
    radius: u8,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct KeyDto {
    key: u8,
}

/// An object type's definition: its one kind table with its references resolved.
fn object_type(pack: &str, t: ObjectTypeDto) -> Result<ObjectTypeDef, AdventurePackError> {
    let refs = |list: Vec<String>| {
        list.iter()
            .map(|r| reference(pack, r))
            .collect::<Result<Vec<_>, _>>()
    };
    let one = NonZeroU32::MIN;
    let mut kinds = Vec::new();
    if let Some(p) = t.pickup {
        kinds.push(ObjectKindDef::Pickup(PickupDef::Resource {
            resource: reference(pack, &p.resource)?,
            amount: p.amount,
            multiplier: p.multiplier.unwrap_or(one),
        }));
    }
    if let Some(list) = t.pickup_one_of {
        kinds.push(ObjectKindDef::Pickup(PickupDef::OneOf(refs(list)?)));
    }
    if let Some(p) = t.pickup_paired {
        kinds.push(ObjectKindDef::Pickup(PickupDef::Paired {
            amount: p.amount,
            resources: refs(p.resources)?,
            paired: reference(pack, &p.paired)?,
            multiplier: p.multiplier.unwrap_or(one),
        }));
    }
    if let Some(r) = t.reveal_terrain {
        kinds.push(ObjectKindDef::RevealTerrain {
            terrains: refs(r.terrains)?,
            price: r
                .price
                .iter()
                .map(|(k, &a)| Ok((reference(pack, k)?, a)))
                .collect::<Result<_, KeyError>>()?,
        });
    }
    kinds.extend(
        t.reveal_area
            .map(|r| ObjectKindDef::RevealArea { radius: r.radius }),
    );
    kinds.extend(t.key_tent.map(|k| ObjectKindDef::KeyTent { key: k.key }));
    kinds.extend(t.key_guard.map(|k| ObjectKindDef::KeyGuard { key: k.key }));
    kinds.extend(t.key_gate.map(|k| ObjectKindDef::KeyGate { key: k.key }));
    let [kind] = <[ObjectKindDef; 1]>::try_from(kinds)
        .map_err(|_| AdventurePackError::ObjectKind(t.id.clone()))?;
    Ok(ObjectTypeDef {
        key: ContentKey::new(pack, &t.id)?,
        name: t.name,
        visit: match t.visit {
            VisitDto::Enter => Visit::Enter,
            VisitDto::Neighbour => Visit::Neighbour,
        },
        from_above: t.from_above,
        kind,
    })
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
    let object_types = file
        .object_type
        .into_iter()
        .map(|t| object_type(pack, t))
        .collect::<Result<_, _>>()?;
    let amounts = |row: &BTreeMap<String, u32>| {
        row.iter()
            .map(|(k, &a)| Ok((reference(pack, k)?, a)))
            .collect::<Result<Vec<_>, KeyError>>()
    };
    let starting_resources = file
        .difficulty
        .iter()
        .map(|d| {
            Ok(StartingResourcesDef {
                human: amounts(&d.human)?,
                computer: amounts(&d.computer)?,
            })
        })
        .collect::<Result<_, KeyError>>()?;
    Ok(AdventurePack {
        terrains,
        roads,
        rivers,
        object_types,
        starting_resources,
        params: file.params.map(|p| AdventureParamsDef {
            native_cost: p.native_cost,
            land_points: p.land_points,
            scouting_radius: p.scouting_radius,
            max_map_size: p.max_map_size,
            max_levels: p.max_levels,
            max_players: p.max_players,
            army_slots: p.army_slots,
            starting_stack_chances: p.starting_stack_chances,
        }),
        ..AdventurePack::new(manifest.clone())
    })
}

/// Parses and compiles packs given as `(core.toml, adventure.toml)` texts (`""` for a pack
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

/// The three original packs, RoE to SoD, as `(core.toml, adventure.toml)` texts.
pub const ORIGINAL_PACKS: [(&str, &str); 3] = [
    (h3_pack::BASE_CORE, BASE_ADVENTURE),
    (h3_pack::AB_CORE, AB_ADVENTURE),
    (h3_pack::SOD_CORE, SOD_ADVENTURE),
];

#[cfg(test)]
mod tests;
