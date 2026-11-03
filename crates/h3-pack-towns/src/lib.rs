//! Adapter: parses the packs' `towns.toml` files (one pack file per context, ADR-0006's E08
//! amendment) into the town context's definitions, which `h3-towns` compiles into its ruleset part
//! (ADR-0028 point 5): [`compile`] over the enabled packs' files in dependency order onto a
//! compiled core, overrides as ADR-0006's; a pack folder without the file reads as an empty one
//! (`packs/README.md`). The base pack holds the eight factions' town records, the AB pack
//! Conflux's, the SoD pack its overrides (harvested by `h3-harvest` since E09 T5b2b2:
//! [`ORIGINAL_PACKS`]). Parsing is strict like
//! `h3-pack`'s (unknown fields are errors, numbers must fit their types), and the compiler
//! validates everything again (E09 increment 1's design note §2).
//!
//! # Format (E09 T5a1)
//!
//! ```toml
//! [town_params]                    # exactly one enabled pack gives them when any pack has a
//!                                  # town record
//! gold = "gold"                    # the core resource the halls and the Grail pay
//! hall_gold = [500, 1000, 2000, 4000] # a day by the highest standing hall, 1 to 4 (TWN-INC-001)
//! grail_gold = 5000                # a day more with a `grail` building ever built
//! reveal_radius = 5                # the start's and a capture's reveal (TWN-CAP-001)
//! level_2_dwelling_chance = 30     # in 100: an owned standard town with a Fort (TWN-SET-003)
//! neutral_garrison = [             # TWN-SET-004: levels 1 to 4, a chance in 100 and the
//!   { chance = 33, count = [8, 15] }, { chance = 33, count = [5, 7] }, # count's bounds,
//!   { chance = 20, count = [3, 5] },  { chance = 13, count = [1, 3] } ] # 1 <= least <= most
//! capitol_demoted_to = "city_hall" # TWN-CAP-002: the key of a hall-3 building of every record
//!
//! [[town]]                         # one per faction with towns, a faction of the closure
//! faction = "castle"
//! moat = "harmless"                # absent | harmless | damaging, from a Citadel (a Tower's
//!                                  # land mines; ADR-0023 point 2)
//! siege_obstacles = false          # optional: natural obstacles in a walled siege (TWN-SIE-005)
//! silo = { wood = 1, ore = 1 }     # optional: the Resource Silo's resources a day, each >= 1
//! creatures = [["pikeman", "halberdier"], ["archer", "marksman"]] # one to seven levels from
//!                                  # 1: the faction's creature of the level, then optionally
//!                                  # one of its upgrades
//!   [[town.building]]              # every building of the faction, roles or not; at most 64
//!   key = "town_hall"              # [a-z0-9_]+, unique within the record
//!   name = "Town Hall"             # the name only (Q13)
//!   slot = 1                       # optional: the map editor's slot, 0 to 40, unique
//!   supersedes = ["village_hall"]  # optional: direct; the compiler closes them, no cycle
//!   forbidden_with = "horde_1"     # optional: forbidden whenever that one is (a horde's
//!                                  # upgrade with its horde), no cycle
//!   auto_with = ["horde_1", "upg_dwelling_3"] # optional: built once each of these is ever
//!                                  # built (a horde's upgrade, E09 decision 29), not itself
//!   hall = 2                       # its roles, each optional (below)
//!
//! [[town_override]]                # ADR-0006: a record a pack of the closure defined; only the
//! faction = "castle"               # listed fields change, the silo as a whole, in load order
//! moat = "damaging"
//! siege_obstacles = false
//! silo = { wood = 1, ore = 1 }
//! ```
//!
//! A building's roles, each read over one of a town's two sets (E09 increment 1's design note §4,
//! §6): the buildings **standing** (built and not superseded by a built one, the original's +0x150)
//! or the buildings **ever built** (+0x158).
//!
//! | Role | Value | Read over |
//! |---|---|---|
//! | `hall` | 1 Village Hall to 4 Capitol; one building of each level | standing |
//! | `fort` | 1 Fort to 3 Castle; one building of each level | standing |
//! | `tavern = true` | counts for a viewer's Tavern level (TWN-VEW-002) | standing |
//! | `silo = true` | one building: pays the record's `silo` (TWN-INC-002) | standing |
//! | `grail = true` | one building: pays `grail_gold` | ever built |
//! | `dwelling = [level, upgraded]` | a level of the record, upgraded only with the level's upgrade; each once, `[1, false]` and `[2, false]` present | the standard set's setup (TWN-SET-003) |
//! | `morale = n` | the siege's defender, -3 to 3 (the Brotherhood of the Sword replaces the Tavern) | standing |
//! | `luck = n` | the siege's defender, -3 to 3 (the Fountain of Fortune) | ever built |
//! | `escape_tunnel = true` | the siege's defender may retreat | ever built |
//! | `defender = { attack, defense, power, mana }` | the defending hero's bonuses, each optional (ADR-0023 point 3) | ever built |
//! | `reveal = n` or `"all"` | a reveal larger than `reveal_radius` (the Lookout Tower, a Grail) | ever built |
//!
//! Keys without a pack prefix name an entity of the file's own pack; core references (factions,
//! creatures, resources) must lie in the pack's dependency closure.
#![forbid(unsafe_code)]
#![warn(missing_docs)]

use std::collections::BTreeMap;
use std::fmt;

use h3_content::{ContentKey, KeyError, PackManifest, Ruleset};
use h3_pack::PackError;
use h3_towns::{
    BuildingDef, BuildingRoles, CreatureLevelDef, DefenderBonus, Dwelling, GarrisonRow, Moat,
    Reveal, TownCompileError, TownDef, TownOverride, TownPack, TownParamsDef, TownPatch,
    TownRuleset,
};
use serde::Deserialize;

/// The synthetic pack's town file (tests).
pub const SYNTHETIC_TOWNS: &str = include_str!("../../../packs/synthetic/towns.toml");
/// The synthetic expansion's town file (tests).
pub const SYNTHETIC_EXT_TOWNS: &str = include_str!("../../../packs/synthetic-ext/towns.toml");
/// The base pack's town file (harvested, E09 T5b2b2): the eight RoE factions' records and the
/// parameters.
pub const BASE_TOWNS: &str = include_str!("../../../packs/base/towns.toml");
/// The AB pack's town file (harvested): Conflux's record.
pub const AB_TOWNS: &str = include_str!("../../../packs/ab/towns.toml");
/// The SoD pack's town file (harvested): the moats as SoD plays them, overrides.
pub const SOD_TOWNS: &str = include_str!("../../../packs/sod/towns.toml");

/// The original packs' `(core.toml, towns.toml)` texts in load order (`base`, `ab`, `sod`), for
/// [`compile`].
pub const ORIGINAL_PACKS: [(&str, &str); 3] = [
    (h3_pack::BASE_CORE, BASE_TOWNS),
    (h3_pack::AB_CORE, AB_TOWNS),
    (h3_pack::SOD_CORE, SOD_TOWNS),
];

/// Errors of this adapter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TownPackError {
    /// The text is not valid TOML for the expected schema.
    Syntax(String),
    /// A key or id is malformed.
    Key(KeyError),
    /// A level of a town record's `creatures` does not hold one or two creatures; the payload is
    /// the record's faction as written.
    CreatureLevel(String),
    /// A core pack does not parse, or the core packs do not compile.
    Core(PackError),
    /// The town packs do not compile into the town part.
    Compile(TownCompileError),
}

impl fmt::Display for TownPackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Syntax(e) => write!(f, "syntax: {e}"),
            Self::Key(e) => write!(f, "{e}"),
            Self::CreatureLevel(faction) => write!(
                f,
                "town `{faction}`: a creature level holds the creature and at most one upgrade"
            ),
            Self::Core(e) => write!(f, "core packs: {e}"),
            Self::Compile(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for TownPackError {}

impl From<KeyError> for TownPackError {
    fn from(e: KeyError) -> Self {
        Self::Key(e)
    }
}

impl From<PackError> for TownPackError {
    fn from(e: PackError) -> Self {
        Self::Core(e)
    }
}

impl From<TownCompileError> for TownPackError {
    fn from(e: TownCompileError) -> Self {
        Self::Compile(e)
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TownsFile {
    town_params: Option<ParamsDto>,
    #[serde(default)]
    town: Vec<TownDto>,
    #[serde(default)]
    town_override: Vec<OverrideDto>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ParamsDto {
    gold: String,
    hall_gold: [u32; 4],
    grail_gold: u32,
    reveal_radius: u8,
    level_2_dwelling_chance: u8,
    neutral_garrison: [GarrisonDto; 4],
    capitol_demoted_to: String,
}

#[derive(Deserialize, Clone, Copy)]
#[serde(deny_unknown_fields)]
struct GarrisonDto {
    chance: u8,
    count: [u16; 2],
}

#[derive(Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
enum MoatDto {
    Absent,
    Harmless,
    Damaging,
}

impl From<MoatDto> for Moat {
    fn from(moat: MoatDto) -> Self {
        match moat {
            MoatDto::Absent => Self::Absent,
            MoatDto::Harmless => Self::Harmless,
            MoatDto::Damaging => Self::Damaging,
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TownDto {
    faction: String,
    moat: MoatDto,
    #[serde(default)]
    siege_obstacles: bool,
    #[serde(default)]
    silo: BTreeMap<String, u32>,
    creatures: Vec<Vec<String>>,
    #[serde(default)]
    building: Vec<BuildingDto>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BuildingDto {
    key: String,
    name: String,
    slot: Option<u8>,
    #[serde(default)]
    supersedes: Vec<String>,
    forbidden_with: Option<String>,
    #[serde(default)]
    auto_with: Vec<String>,
    hall: Option<u8>,
    fort: Option<u8>,
    #[serde(default)]
    tavern: bool,
    #[serde(default)]
    silo: bool,
    #[serde(default)]
    grail: bool,
    dwelling: Option<(u8, bool)>,
    #[serde(default)]
    morale: i8,
    #[serde(default)]
    luck: i8,
    #[serde(default)]
    escape_tunnel: bool,
    #[serde(default)]
    defender: DefenderDto,
    reveal: Option<RevealDto>,
}

#[derive(Deserialize, Default, Clone, Copy)]
#[serde(deny_unknown_fields)]
struct DefenderDto {
    #[serde(default)]
    attack: u8,
    #[serde(default)]
    defense: u8,
    #[serde(default)]
    power: u8,
    #[serde(default)]
    mana: u16,
}

#[derive(Deserialize, Clone, Copy)]
#[serde(untagged)]
enum RevealDto {
    Radius(u8),
    Word(RevealWord),
}

#[derive(Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
enum RevealWord {
    All,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct OverrideDto {
    faction: String,
    moat: Option<MoatDto>,
    siege_obstacles: Option<bool>,
    silo: Option<BTreeMap<String, u32>>,
}

/// A key as written: `pack:name`, or a name of the file's own pack.
fn reference(pack: &str, text: &str) -> Result<ContentKey, KeyError> {
    if text.contains(':') {
        ContentKey::parse(text)
    } else {
        ContentKey::new(pack, text)
    }
}

fn silo(pack: &str, silo: BTreeMap<String, u32>) -> Result<Vec<(ContentKey, u32)>, KeyError> {
    silo.into_iter()
        .map(|(resource, amount)| Ok((reference(pack, &resource)?, amount)))
        .collect()
}

fn building(b: BuildingDto) -> BuildingDef {
    BuildingDef {
        key: b.key,
        name: b.name,
        slot: b.slot,
        supersedes: b.supersedes,
        forbidden_with: b.forbidden_with,
        auto_with: b.auto_with,
        roles: BuildingRoles {
            hall: b.hall,
            fort: b.fort,
            tavern: b.tavern,
            silo: b.silo,
            grail: b.grail,
            dwelling: b
                .dwelling
                .map(|(level, upgraded)| Dwelling { level, upgraded }),
            morale: b.morale,
            luck: b.luck,
            escape_tunnel: b.escape_tunnel,
            defender: DefenderBonus {
                attack: b.defender.attack,
                defense: b.defender.defense,
                power: b.defender.power,
                mana: b.defender.mana,
            },
            reveal: b.reveal.map(|r| match r {
                RevealDto::Radius(radius) => Reveal::Radius(radius),
                RevealDto::Word(RevealWord::All) => Reveal::All,
            }),
        },
    }
}

fn town(pack: &str, t: TownDto) -> Result<TownDef, TownPackError> {
    let creatures = t
        .creatures
        .iter()
        .map(|level| match level.as_slice() {
            [base] => Ok(CreatureLevelDef {
                base: reference(pack, base)?,
                upgrade: None,
            }),
            [base, upgrade] => Ok(CreatureLevelDef {
                base: reference(pack, base)?,
                upgrade: Some(reference(pack, upgrade)?),
            }),
            _ => Err(TownPackError::CreatureLevel(t.faction.clone())),
        })
        .collect::<Result<_, TownPackError>>()?;
    Ok(TownDef {
        faction: reference(pack, &t.faction)?,
        moat: t.moat.into(),
        siege_obstacles: t.siege_obstacles,
        silo: silo(pack, t.silo)?,
        creatures,
        buildings: t.building.into_iter().map(building).collect(),
    })
}

/// Parses the `towns.toml` of the core pack with this manifest (`""` for a pack without the
/// file).
///
/// # Errors
///
/// [`TownPackError::Syntax`] for text that does not fit the format, [`TownPackError::Key`] for
/// a malformed key, [`TownPackError::CreatureLevel`] for a level of other than one or two
/// creatures.
pub fn parse_towns(manifest: &PackManifest, text: &str) -> Result<TownPack, TownPackError> {
    let file: TownsFile = toml::from_str(text).map_err(|e| TownPackError::Syntax(e.to_string()))?;
    let pack = manifest.id.as_str();
    let params = file
        .town_params
        .map(|p| {
            Ok::<_, KeyError>(TownParamsDef {
                gold: reference(pack, &p.gold)?,
                hall_gold: p.hall_gold,
                grail_gold: p.grail_gold,
                reveal_radius: p.reveal_radius,
                level_2_dwelling_chance: p.level_2_dwelling_chance,
                neutral_garrison: p.neutral_garrison.map(|row| GarrisonRow {
                    chance: row.chance,
                    count: row.count,
                }),
                capitol_demoted_to: p.capitol_demoted_to,
            })
        })
        .transpose()?;
    let towns = file
        .town
        .into_iter()
        .map(|t| town(pack, t))
        .collect::<Result<_, _>>()?;
    let overrides = file
        .town_override
        .into_iter()
        .map(|o| {
            Ok::<_, KeyError>(TownOverride {
                faction: reference(pack, &o.faction)?,
                patch: TownPatch {
                    moat: o.moat.map(Moat::from),
                    siege_obstacles: o.siege_obstacles,
                    silo: o.silo.map(|s| silo(pack, s)).transpose()?,
                },
            })
        })
        .collect::<Result<_, _>>()?;
    Ok(TownPack {
        manifest: manifest.clone(),
        params,
        towns,
        overrides,
    })
}

/// Compiles the core packs and the town part from `(core.toml, towns.toml)` texts in load order
/// (`""` for a pack folder without a `towns.toml`), with the profile `profile`.
///
/// # Errors
///
/// [`TownPackError::Core`] if a core pack does not parse or the core does not compile, the
/// parse errors of [`parse_towns`], [`TownPackError::Compile`] if the town packs do not compile.
pub fn compile(packs: &[(&str, &str)], profile: &str) -> Result<TownRuleset, TownPackError> {
    let core_packs = packs
        .iter()
        .map(|(core, _)| h3_pack::parse_pack(core))
        .collect::<Result<Vec<_>, _>>()?;
    let towns = core_packs
        .iter()
        .zip(packs)
        .map(|(core, (_, text))| parse_towns(&core.manifest, text))
        .collect::<Result<Vec<_>, _>>()?;
    let core = Ruleset::compile(&core_packs, profile).map_err(PackError::from)?;
    Ok(TownRuleset::compile(&core, &towns)?)
}

#[cfg(test)]
mod tests;
