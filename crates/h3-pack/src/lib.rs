//! Adapter: parses human-editable content packs and scenarios (TOML) into content definitions and
//! battle setups (ADR-0006). Pack text is untrusted input: parsing is strict (unknown fields are
//! errors, numbers must fit their types) and every definition is validated again by the
//! `Ruleset` compiler.
//!
//! Formats: `packs/README.md`; examples: `packs/synthetic/`.

use std::collections::BTreeMap;
use std::fmt;

use h3_content::{
    AbilityDef, AreaShot, AuraTarget, BattleParamsDef, CompileError, ContentKey, ContentPack,
    CreatureDef, CreatureInfoDef, CreatureOverride, CreaturePatch, FactionDef, KeyError,
    ObstacleDef, PackManifest, ProfileDef, ResourceDef, Ruleset, TerrainDef,
};
use h3_contracts::{Army, ArmyStack, BattleSetup, BattleSide};
use serde::Deserialize;

/// The synthetic content pack shipped with the repo (tests, benchmarks, FFI smoke, replays).
pub const SYNTHETIC_PACK: &str = include_str!("../../../packs/synthetic/pack.toml");
/// Synthetic expansion pack (depends on the synthetic pack; dependencies and overrides).
pub const SYNTHETIC_EXT_PACK: &str = include_str!("../../../packs/synthetic-ext/pack.toml");
/// Harvested pack `base`: The Restoration of Erathia, latest patch (E02, OPEN-QUESTIONS Q13/Q15).
pub const BASE_PACK: &str = include_str!("../../../packs/base/pack.toml");
/// Harvested pack `ab`: Armageddon's Blade (depends on `base`).
pub const AB_PACK: &str = include_str!("../../../packs/ab/pack.toml");
/// Harvested pack `sod`: The Shadow of Death (depends on `ab`).
pub const SOD_PACK: &str = include_str!("../../../packs/sod/pack.toml");
/// Synthetic scenario: three stacks per side.
pub const SYNTHETIC_SKIRMISH: &str =
    include_str!("../../../packs/synthetic/scenarios/skirmish.toml");
/// Synthetic scenario: one stack per side.
pub const SYNTHETIC_DUEL: &str = include_str!("../../../packs/synthetic/scenarios/duel.toml");
/// Synthetic scenario: seven stacks per side.
pub const SYNTHETIC_FULL: &str = include_str!("../../../packs/synthetic/scenarios/full.toml");

/// Errors of this adapter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PackError {
    /// The text is not valid TOML for the expected schema.
    Syntax(String),
    /// A key or id is malformed.
    Key(KeyError),
    /// The packs do not compile into a ruleset.
    Compile(CompileError),
    /// A scenario references a creature the ruleset does not define.
    UnknownCreature(String),
    /// A scenario references a terrain the ruleset does not define.
    UnknownTerrain(String),
}

impl fmt::Display for PackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Syntax(e) => write!(f, "syntax: {e}"),
            Self::Key(e) => write!(f, "{e}"),
            Self::Compile(e) => write!(f, "{e}"),
            Self::UnknownCreature(k) => write!(f, "unknown creature `{k}`"),
            Self::UnknownTerrain(k) => write!(f, "unknown terrain `{k}`"),
        }
    }
}

impl std::error::Error for PackError {}

impl From<KeyError> for PackError {
    fn from(e: KeyError) -> Self {
        Self::Key(e)
    }
}

impl From<CompileError> for PackError {
    fn from(e: CompileError) -> Self {
        Self::Compile(e)
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PackFile {
    pack: ManifestDto,
    battle: Option<BattleDto>,
    #[serde(default)]
    resource: Vec<NamedDto>,
    #[serde(default)]
    faction: Vec<FactionDto>,
    #[serde(default)]
    terrain: Vec<NamedDto>,
    #[serde(default)]
    obstacle: Vec<ObstacleDto>,
    #[serde(default)]
    creature: Vec<CreatureDto>,
    #[serde(default)]
    creature_override: Vec<OverrideDto>,
    #[serde(default)]
    profile: Vec<ProfileDto>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NamedDto {
    id: String,
    name: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FactionDto {
    id: String,
    name: String,
    native_terrain: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ObstacleDto {
    id: String,
    terrains: Vec<String>,
    #[serde(default)]
    width: u8,
    #[serde(default)]
    height: u8,
    blocked: Vec<[i8; 2]>,
    #[serde(default)]
    absolute: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ManifestDto {
    id: String,
    version: String,
    #[serde(default)]
    depends: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BattleDto {
    field_width: u8,
    field_height: u8,
    side_columns: u8,
    army_slots: u8,
    damage_roll_samples: u8,
    attack_bonus_per_point_percent: u16,
    attack_bonus_cap_percent: u16,
    defense_reduction_per_point_permille: u16,
    defense_reduction_cap_permille: u16,
    defend_bonus_percent: u16,
    defend_bonus_min: u16,
    ranged_full_damage_distance: u8,
    ranged_penalty_percent: u8,
    melee_penalty_percent: u8,
    morale_dice: u8,
    good_morale_chances: Vec<u8>,
    bad_morale_chances: Vec<u8>,
    luck_dice: u8,
    good_luck_chances: Vec<u8>,
    formation: Vec<Vec<u8>>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CreatureDto {
    id: String,
    attack: u16,
    defense: u16,
    damage: [u16; 2],
    hit_points: u32,
    speed: u8,
    #[serde(default)]
    name: String,
    #[serde(default)]
    plural: String,
    faction: Option<String>,
    #[serde(default = "default_level")]
    level: u8,
    #[serde(default)]
    cost: BTreeMap<String, u32>,
    #[serde(default)]
    growth: u16,
    #[serde(default)]
    horde_growth: u16,
    #[serde(default)]
    fight_value: u32,
    #[serde(default)]
    ai_value: u32,
    #[serde(default)]
    shots: u16,
    #[serde(default)]
    casts: u8,
    #[serde(default)]
    adventure_count: [u16; 2],
    #[serde(default)]
    flags: Vec<String>,
    #[serde(default)]
    abilities: Vec<AbilityDto>,
}

/// One ability (`packs/README.md`, "Abilities"): `kind` plus the parameters of that kind.
/// Parameterless kinds are empty struct variants so that `deny_unknown_fields` rejects stray keys.
#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum AbilityDto {
    Hate { creature: String, percent: u16 },
    Jousting { percent_per_hex: u16 },
    ChargeImmunity {},
    DefenseReduction { percent: u8 },
    DeathBlow { chance_percent: u8 },
    LifeDrain { percent: u16 },
    Regeneration {},
    DeathStare { chance_percent: u8 },
    ExtraRetaliations { count: u8 },
    UnlimitedRetaliations {},
    StrikeAndReturn {},
    AttacksAllAdjacent {},
    ThreeHeadedAttack {},
    Teleports {},
    MoraleAura { amount: i8, affects: AuraDto },
    LuckAura { amount: i8, affects: AuraDto },
    MinimumMorale { value: i8 },
    MinimumLuck { value: i8 },
    NoDistancePenalty {},
    AreaShot { effect: AreaShotDto },
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum AuraDto {
    Allies,
    Enemies,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum AreaShotDto {
    Fireball,
    DeathCloud,
}

impl AbilityDto {
    fn into_def(self, pack: &str) -> Result<AbilityDef, KeyError> {
        let aura = |a: AuraDto| match a {
            AuraDto::Allies => AuraTarget::Allies,
            AuraDto::Enemies => AuraTarget::Enemies,
        };
        Ok(match self {
            Self::Hate { creature, percent } => AbilityDef::Hate {
                target: reference(pack, &creature)?,
                percent,
            },
            Self::Jousting { percent_per_hex } => AbilityDef::Jousting { percent_per_hex },
            Self::ChargeImmunity {} => AbilityDef::ChargeImmunity,
            Self::DefenseReduction { percent } => AbilityDef::DefenseReduction { percent },
            Self::DeathBlow { chance_percent } => AbilityDef::DeathBlow { chance_percent },
            Self::LifeDrain { percent } => AbilityDef::LifeDrain { percent },
            Self::Regeneration {} => AbilityDef::Regeneration,
            Self::DeathStare { chance_percent } => AbilityDef::DeathStare { chance_percent },
            Self::ExtraRetaliations { count } => AbilityDef::ExtraRetaliations { count },
            Self::UnlimitedRetaliations {} => AbilityDef::UnlimitedRetaliations,
            Self::StrikeAndReturn {} => AbilityDef::StrikeAndReturn,
            Self::AttacksAllAdjacent {} => AbilityDef::AttacksAllAdjacent,
            Self::ThreeHeadedAttack {} => AbilityDef::ThreeHeadedAttack,
            Self::Teleports {} => AbilityDef::Teleports,
            Self::MoraleAura { amount, affects } => AbilityDef::MoraleAura {
                amount,
                affects: aura(affects),
            },
            Self::LuckAura { amount, affects } => AbilityDef::LuckAura {
                amount,
                affects: aura(affects),
            },
            Self::MinimumMorale { value } => AbilityDef::MinimumMorale { value },
            Self::MinimumLuck { value } => AbilityDef::MinimumLuck { value },
            Self::NoDistancePenalty {} => AbilityDef::NoDistancePenalty,
            Self::AreaShot { effect } => AbilityDef::AreaShot {
                effect: match effect {
                    AreaShotDto::Fireball => AreaShot::Fireball,
                    AreaShotDto::DeathCloud => AreaShot::DeathCloud,
                },
            },
        })
    }
}

fn abilities(pack: &str, list: Vec<AbilityDto>) -> Result<Vec<AbilityDef>, KeyError> {
    list.into_iter().map(|a| a.into_def(pack)).collect()
}

fn default_level() -> u8 {
    1
}

/// `[[creature_override]]`: every field but `target` is optional. `faction = ""` makes the
/// creature neutral.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct OverrideDto {
    target: String,
    attack: Option<u16>,
    defense: Option<u16>,
    damage: Option<[u16; 2]>,
    hit_points: Option<u32>,
    speed: Option<u8>,
    name: Option<String>,
    plural: Option<String>,
    faction: Option<String>,
    level: Option<u8>,
    cost: Option<BTreeMap<String, u32>>,
    growth: Option<u16>,
    horde_growth: Option<u16>,
    fight_value: Option<u32>,
    ai_value: Option<u32>,
    shots: Option<u16>,
    casts: Option<u8>,
    adventure_count: Option<[u16; 2]>,
    flags: Option<Vec<String>>,
    abilities: Option<Vec<AbilityDto>>,
}

/// Resolves a reference: `name` means `<own pack>:name`, `pack:name` is taken as written.
fn reference(pack: &str, text: &str) -> Result<ContentKey, KeyError> {
    if text.contains(':') {
        ContentKey::parse(text)
    } else {
        ContentKey::new(pack, text)
    }
}

fn cost(pack: &str, cost: BTreeMap<String, u32>) -> Result<Vec<(ContentKey, u32)>, KeyError> {
    cost.into_iter()
        .map(|(k, v)| Ok((reference(pack, &k)?, v)))
        .collect()
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ProfileDto {
    id: String,
    #[serde(default)]
    quirks: BTreeMap<String, bool>,
}

/// Parses a content pack.
pub fn parse_pack(text: &str) -> Result<ContentPack, PackError> {
    let file: PackFile = toml::from_str(text).map_err(|e| PackError::Syntax(e.to_string()))?;
    let pack_id = file.pack.id;
    let named = |items: Vec<NamedDto>| {
        items
            .into_iter()
            .map(|n| Ok((ContentKey::new(&pack_id, &n.id)?, n.name)))
            .collect::<Result<Vec<_>, KeyError>>()
    };
    let resources = named(file.resource)?
        .into_iter()
        .map(|(key, name)| ResourceDef { key, name })
        .collect();
    let factions = file
        .faction
        .into_iter()
        .map(|f| {
            Ok(FactionDef {
                key: ContentKey::new(&pack_id, &f.id)?,
                name: f.name,
                native_terrain: f
                    .native_terrain
                    .map(|t| reference(&pack_id, &t))
                    .transpose()?,
            })
        })
        .collect::<Result<Vec<_>, KeyError>>()?;
    let terrains = named(file.terrain)?
        .into_iter()
        .map(|(key, name)| TerrainDef { key, name })
        .collect();
    let obstacles = file
        .obstacle
        .into_iter()
        .map(|o| {
            Ok(ObstacleDef {
                key: ContentKey::new(&pack_id, &o.id)?,
                terrains: o
                    .terrains
                    .iter()
                    .map(|t| reference(&pack_id, t))
                    .collect::<Result<_, _>>()?,
                width: o.width,
                height: o.height,
                blocked: o.blocked.into_iter().map(|[a, b]| (a, b)).collect(),
                absolute: o.absolute,
            })
        })
        .collect::<Result<Vec<_>, KeyError>>()?;
    let creatures = file
        .creature
        .into_iter()
        .map(|c| {
            Ok(CreatureDef {
                key: ContentKey::new(&pack_id, &c.id)?,
                attack: c.attack,
                defense: c.defense,
                damage_min: c.damage[0],
                damage_max: c.damage[1],
                hit_points: c.hit_points,
                speed: c.speed,
                info: CreatureInfoDef {
                    name: c.name,
                    plural: c.plural,
                    faction: c.faction.map(|f| reference(&pack_id, &f)).transpose()?,
                    level: c.level,
                    cost: cost(&pack_id, c.cost)?,
                    growth: c.growth,
                    horde_growth: c.horde_growth,
                    fight_value: c.fight_value,
                    ai_value: c.ai_value,
                    shots: c.shots,
                    casts: c.casts,
                    adventure_count: c.adventure_count,
                    flags: c.flags,
                    abilities: abilities(&pack_id, c.abilities)?,
                },
            })
        })
        .collect::<Result<Vec<_>, PackError>>()?;
    let creature_overrides = file
        .creature_override
        .into_iter()
        .map(|o| {
            let faction = o
                .faction
                .map(|f| (!f.is_empty()).then(|| reference(&pack_id, &f)).transpose())
                .transpose()?;
            Ok(CreatureOverride {
                target: ContentKey::parse(&o.target)?,
                patch: CreaturePatch {
                    attack: o.attack,
                    defense: o.defense,
                    damage_min: o.damage.map(|d| d[0]),
                    damage_max: o.damage.map(|d| d[1]),
                    hit_points: o.hit_points,
                    speed: o.speed,
                    name: o.name,
                    plural: o.plural,
                    faction,
                    level: o.level,
                    cost: o.cost.map(|c| cost(&pack_id, c)).transpose()?,
                    growth: o.growth,
                    horde_growth: o.horde_growth,
                    fight_value: o.fight_value,
                    ai_value: o.ai_value,
                    shots: o.shots,
                    casts: o.casts,
                    adventure_count: o.adventure_count,
                    flags: o.flags,
                    abilities: o.abilities.map(|a| abilities(&pack_id, a)).transpose()?,
                },
            })
        })
        .collect::<Result<Vec<_>, PackError>>()?;
    let profiles = file
        .profile
        .into_iter()
        .map(|p| {
            Ok(ProfileDef {
                key: ContentKey::new(&pack_id, &p.id)?,
                quirks: p.quirks.into_iter().collect(),
            })
        })
        .collect::<Result<Vec<_>, PackError>>()?;
    let battle = file.battle.map(|b| BattleParamsDef {
        field_width: b.field_width,
        field_height: b.field_height,
        side_columns: b.side_columns,
        army_slots: b.army_slots,
        damage_roll_samples: b.damage_roll_samples,
        attack_bonus_per_point_percent: b.attack_bonus_per_point_percent,
        attack_bonus_cap_percent: b.attack_bonus_cap_percent,
        defense_reduction_per_point_permille: b.defense_reduction_per_point_permille,
        defense_reduction_cap_permille: b.defense_reduction_cap_permille,
        defend_bonus_percent: b.defend_bonus_percent,
        defend_bonus_min: b.defend_bonus_min,
        ranged_full_damage_distance: b.ranged_full_damage_distance,
        ranged_penalty_percent: b.ranged_penalty_percent,
        melee_penalty_percent: b.melee_penalty_percent,
        morale_dice: b.morale_dice,
        good_morale_chances: b.good_morale_chances,
        bad_morale_chances: b.bad_morale_chances,
        luck_dice: b.luck_dice,
        good_luck_chances: b.good_luck_chances,
        formation: b.formation,
    });
    Ok(ContentPack {
        manifest: PackManifest {
            id: pack_id,
            version: file.pack.version,
            depends: file.pack.depends,
        },
        resources,
        factions,
        terrains,
        obstacles,
        creatures,
        creature_overrides,
        profiles,
        battle,
    })
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ScenarioFile {
    scenario: ScenarioHeader,
    #[serde(default)]
    attacker: Vec<StackDto>,
    #[serde(default)]
    defender: Vec<StackDto>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ScenarioHeader {
    id: String,
    packs: Vec<String>,
    profile: String,
    /// Obstacle hexes (row-major indices), validated by the battle context.
    #[serde(default)]
    obstacles: Vec<u8>,
    /// Sides led by a hero (`"attacker"`, `"defender"`; Q17).
    #[serde(default)]
    led_by_hero: Vec<String>,
    /// Terrain key of the battlefield (BAT-TER-001).
    terrain: Option<String>,
    /// Generate obstacles for the terrain when the battle starts (BAT-GEO-004).
    #[serde(default)]
    random_obstacles: bool,
}

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
struct StackDto {
    slot: u8,
    creature: String,
    count: u32,
}

/// A parsed scenario whose creature references are not yet resolved against a ruleset.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScenarioDef {
    /// Scenario id.
    pub id: String,
    /// Ids of the enabled packs (any order; the compiler loads dependencies first).
    pub packs: Vec<String>,
    /// Profile key the scenario is meant to run with.
    pub profile: String,
    attacker: Vec<(u8, String, u32)>,
    defender: Vec<(u8, String, u32)>,
    obstacles: Vec<u8>,
    led_by_hero: [bool; 2],
    terrain: Option<String>,
    /// Whether obstacles are generated for the terrain when the battle starts (the caller does
    /// it, with its seed; BAT-GEO-004). Excludes explicit obstacles.
    pub random_obstacles: bool,
}

/// Parses a list of side names (`"attacker"`, `"defender"`, each at most once) into flags indexed
/// by [`BattleSide::index`].
///
/// # Errors
/// An unknown or repeated side name.
pub fn parse_sides(names: &[String]) -> Result<[bool; 2], PackError> {
    let mut sides = [false; 2];
    for name in names {
        let side = match name.as_str() {
            "attacker" => BattleSide::Attacker,
            "defender" => BattleSide::Defender,
            _ => return Err(PackError::Syntax(format!("unknown side `{name}`"))),
        };
        if core::mem::replace(&mut sides[side.index()], true) {
            return Err(PackError::Syntax(format!("side `{name}` listed twice")));
        }
    }
    Ok(sides)
}

/// The side names of the flags set in `sides` (inverse of [`parse_sides`]).
pub fn side_names(sides: [bool; 2]) -> Vec<String> {
    BattleSide::BOTH
        .into_iter()
        .filter(|s| sides[s.index()])
        .map(|s| match s {
            BattleSide::Attacker => "attacker".to_owned(),
            BattleSide::Defender => "defender".to_owned(),
        })
        .collect()
}

impl ScenarioDef {
    /// Resolves creature keys against `ruleset`. The battle context validates the setup itself
    /// (slots, counts) when the battle starts.
    pub fn resolve(&self, ruleset: &Ruleset) -> Result<BattleSetup, PackError> {
        let army = |stacks: &[(u8, String, u32)]| -> Result<Army, PackError> {
            let stacks = stacks
                .iter()
                .map(|(slot, key, count)| {
                    let creature = ruleset
                        .creature_by_key(key)
                        .ok_or_else(|| PackError::UnknownCreature(key.clone()))?;
                    Ok(ArmyStack {
                        slot: *slot,
                        creature,
                        count: *count,
                    })
                })
                .collect::<Result<Vec<_>, PackError>>()?;
            Ok(Army { stacks })
        };
        let terrain = self
            .terrain
            .as_ref()
            .map(|t| {
                ruleset
                    .terrain_by_key(t)
                    .ok_or_else(|| PackError::UnknownTerrain(t.clone()))
            })
            .transpose()?;
        Ok(BattleSetup {
            attacker: army(&self.attacker)?,
            defender: army(&self.defender)?,
            obstacles: self.obstacles.clone(),
            led_by_hero: self.led_by_hero,
            terrain,
        })
    }
}

/// Parses a scenario.
pub fn parse_scenario(text: &str) -> Result<ScenarioDef, PackError> {
    let file: ScenarioFile = toml::from_str(text).map_err(|e| PackError::Syntax(e.to_string()))?;
    ContentKey::parse(&file.scenario.profile)?;
    if file.scenario.random_obstacles && !file.scenario.obstacles.is_empty() {
        return Err(PackError::Syntax(
            "random_obstacles excludes an obstacles list".to_owned(),
        ));
    }
    if file.scenario.random_obstacles && file.scenario.terrain.is_none() {
        return Err(PackError::Syntax(
            "random_obstacles needs a terrain".to_owned(),
        ));
    }
    let convert = |v: Vec<StackDto>| {
        v.into_iter()
            .map(|s| (s.slot, s.creature, s.count))
            .collect()
    };
    Ok(ScenarioDef {
        id: file.scenario.id,
        packs: file.scenario.packs,
        profile: file.scenario.profile,
        attacker: convert(file.attacker),
        defender: convert(file.defender),
        obstacles: file.scenario.obstacles,
        led_by_hero: parse_sides(&file.scenario.led_by_hero)?,
        terrain: file.scenario.terrain,
        random_obstacles: file.scenario.random_obstacles,
    })
}

/// Parses and compiles the enabled packs (any order) with a profile.
pub fn compile(pack_texts: &[&str], profile: &str) -> Result<Ruleset, PackError> {
    let packs = pack_texts
        .iter()
        .map(|t| parse_pack(t))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Ruleset::compile(&packs, profile)?)
}

/// The synthetic ruleset with its scenario resolved: the standard fixture for tests, benchmarks
/// and the C ABI.
pub fn synthetic(scenario_text: &str) -> Result<(Ruleset, BattleSetup), PackError> {
    let scenario = parse_scenario(scenario_text)?;
    let ruleset = compile(&[SYNTHETIC_PACK], &scenario.profile)?;
    let setup = scenario.resolve(&ruleset)?;
    Ok((ruleset, setup))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn synthetic_pack_and_scenarios_load() {
        for text in [SYNTHETIC_SKIRMISH, SYNTHETIC_DUEL, SYNTHETIC_FULL] {
            let (ruleset, setup) = synthetic(text).unwrap();
            assert_eq!(ruleset.creature_count(), 3);
            assert!(!setup.attacker.stacks.is_empty());
            assert!(!setup.defender.stacks.is_empty());
        }
        let (ruleset, setup) = synthetic(SYNTHETIC_SKIRMISH).unwrap();
        assert_eq!(ruleset.profile().as_str(), "synthetic:faithful");
        assert_eq!(ruleset.battle().army_slots, 7);
        let brute = ruleset.creature_by_key("synthetic:brute").unwrap();
        assert_eq!(
            setup.attacker.stacks[1],
            ArmyStack {
                slot: 1,
                creature: brute,
                count: 10
            }
        );
        let spearman = ruleset
            .creature(ruleset.creature_by_key("synthetic:spearman").unwrap())
            .unwrap();
        assert_eq!(
            (
                spearman.damage_min,
                spearman.damage_max,
                spearman.hit_points
            ),
            (1, 3, 10)
        );
        assert!(compile(&[SYNTHETIC_PACK], "synthetic:strict-fixed").is_ok());
    }

    #[test]
    fn strict_parsing() {
        let unknown_field = SYNTHETIC_PACK.replace("speed = 4", "speed = 4\nflying = true");
        assert!(matches!(
            parse_pack(&unknown_field),
            Err(PackError::Syntax(_))
        ));
        let out_of_range = SYNTHETIC_PACK.replace("speed = 4", "speed = 400");
        assert!(matches!(
            parse_pack(&out_of_range),
            Err(PackError::Syntax(_))
        ));
        let bad_name = SYNTHETIC_PACK.replace("id = \"brute\"", "id = \"Brute\"");
        assert!(matches!(parse_pack(&bad_name), Err(PackError::Key(_))));
        let bad_profile = SYNTHETIC_PACK.replace("id = \"faithful\"", "id = \"Faithful\"");
        assert!(matches!(parse_pack(&bad_profile), Err(PackError::Key(_))));
        let quirky = SYNTHETIC_PACK.replace(
            "quirks = { \"quirk.battle.no_luck_without_hero\" = true }",
            "quirks = { \"quirk.x\" = true }",
        );
        let pack = parse_pack(&quirky).unwrap();
        assert_eq!(pack.profiles[0].quirks, vec![("quirk.x".to_owned(), true)]);
        assert!(matches!(
            compile(&[&quirky], "synthetic:faithful"),
            Err(PackError::Compile(_))
        ));
        let no_battle = parse_pack("[pack]\nid = \"x\"\nversion = \"1\"\n").unwrap();
        assert!(no_battle.battle.is_none() && no_battle.creatures.is_empty());
    }

    #[test]
    fn scenario_errors() {
        assert!(matches!(
            parse_scenario("nonsense = ["),
            Err(PackError::Syntax(_))
        ));
        let bad_profile = SYNTHETIC_DUEL.replace("synthetic:faithful", "faithful");
        assert!(matches!(
            parse_scenario(&bad_profile),
            Err(PackError::Key(_))
        ));
        let unknown = SYNTHETIC_DUEL.replace("synthetic:spearman", "synthetic:dragon");
        assert_eq!(
            synthetic(&unknown),
            Err(PackError::UnknownCreature("synthetic:dragon".to_owned()))
        );
        let unknown_attacker = SYNTHETIC_DUEL.replace("synthetic:brute", "synthetic:dragon");
        assert!(synthetic(&unknown_attacker).is_err());
        let empty =
            parse_scenario("[scenario]\nid = \"e\"\npacks = []\nprofile = \"a:b\"\n").unwrap();
        assert_eq!(empty.id, "e");
        assert!(empty.packs.is_empty());
        assert_eq!(parse_scenario(SYNTHETIC_DUEL).unwrap().packs, ["synthetic"]);
        let header = "[scenario]\nid = \"e\"\npacks = []\nprofile = \"a:b\"\n";
        for (extra, error) in [
            (
                "random_obstacles = true\n",
                "random_obstacles needs a terrain",
            ),
            (
                "random_obstacles = true\nterrain = \"a:t\"\nobstacles = [40]\n",
                "random_obstacles excludes an obstacles list",
            ),
        ] {
            assert_eq!(
                parse_scenario(&format!("{header}{extra}")),
                Err(PackError::Syntax(error.to_owned()))
            );
        }
        let duel = parse_scenario(SYNTHETIC_DUEL).unwrap();
        assert!(duel.random_obstacles);
        let (ruleset, setup) = synthetic(SYNTHETIC_DUEL).unwrap();
        assert_eq!(setup.terrain, ruleset.terrain_by_key("synthetic:meadow"));
        let elsewhere = SYNTHETIC_DUEL.replace("synthetic:meadow", "synthetic:moon");
        assert_eq!(
            synthetic(&elsewhere),
            Err(PackError::UnknownTerrain("synthetic:moon".to_owned()))
        );
        for (sides, err) in [
            ("[\"attacker\", \"defender\"]", None),
            ("[\"left\"]", Some("unknown side `left`")),
            (
                "[\"defender\", \"defender\"]",
                Some("side `defender` listed twice"),
            ),
        ] {
            let text = format!("{header}led_by_hero = {sides}\n");
            match err {
                None => assert_eq!(parse_scenario(&text).unwrap().led_by_hero, [true, true]),
                Some(e) => assert_eq!(parse_scenario(&text), Err(PackError::Syntax(e.to_owned()))),
            }
        }
        assert_eq!(
            side_names([false, true]),
            ["defender"],
            "inverse of parse_sides"
        );
        assert_eq!(side_names([true, false]), ["attacker"]);
    }

    #[test]
    fn format_v2_fields_and_expansion_packs() {
        let base = compile(&[SYNTHETIC_PACK], "synthetic:faithful").unwrap();
        assert_eq!(base.resources().len(), 2);
        assert_eq!(base.factions()[0].name, "Vale");
        // Terrain content (E03): terrains, native terrain, obstacles.
        let meadow = base.terrain_by_key("synthetic:meadow").unwrap();
        assert_eq!(base.terrains().len(), 2);
        assert_eq!(base.factions()[0].native_terrain, Some(meadow));
        let obstacles: Vec<(&str, bool, usize)> = base
            .obstacles()
            .iter()
            .map(|o| (o.key.as_str(), o.absolute, o.blocked.len()))
            .collect();
        assert_eq!(
            obstacles,
            [
                ("synthetic:boulder", false, 2),
                ("synthetic:crag", false, 3),
                ("synthetic:ridge", true, 4)
            ]
        );
        assert_eq!(base.obstacles()[1].blocked[2], (1, -1));
        let bad_ref =
            SYNTHETIC_PACK.replace("native_terrain = \"meadow\"", "native_terrain = \"Meadow\"");
        assert!(matches!(parse_pack(&bad_ref), Err(PackError::Key(_))));
        let bad_obstacle = SYNTHETIC_PACK.replace("id = \"boulder\"", "id = \"Boulder\"");
        assert!(matches!(parse_pack(&bad_obstacle), Err(PackError::Key(_))));
        let bad_terrain =
            SYNTHETIC_PACK.replace("terrains = [\"meadow\"]", "terrains = [\"Meadow\"]");
        assert!(matches!(parse_pack(&bad_terrain), Err(PackError::Key(_))));
        let bad_faction = SYNTHETIC_PACK.replace("id = \"vale\"", "id = \"Vale\"");
        assert!(matches!(parse_pack(&bad_faction), Err(PackError::Key(_))));
        let brute = base.creature_by_key("synthetic:brute").unwrap();
        let info = base.creature_info(brute).unwrap();
        assert_eq!(
            (info.name.as_str(), info.plural.as_str()),
            ("Brute", "Brutes")
        );
        assert_eq!(info.level, 2);
        // Cost entries come in key order (`coin` before `shard`).
        let costs: Vec<(&str, u32)> = info
            .cost
            .iter()
            .map(|(r, n)| (base.resources()[r.index()].key.as_str(), *n))
            .collect();
        assert_eq!(costs, [("synthetic:coin", 120), ("synthetic:shard", 1)]);
        assert!(
            base.creature(brute)
                .unwrap()
                .flags
                .contains(h3_content::CreatureFlag::TwoHex)
        );
        let wisp = base.creature_by_key("synthetic:wisp").unwrap();
        assert_eq!(base.creature_info(wisp).unwrap().faction, None);

        // The expansion may be listed first; the compiler loads its dependency first.
        let both = compile(&[SYNTHETIC_EXT_PACK, SYNTHETIC_PACK], "synthetic:faithful").unwrap();
        let ids: Vec<&str> = both.packs().iter().map(|(id, _)| id.as_str()).collect();
        assert_eq!(ids, ["synthetic", "synthetic-ext"]);
        assert_eq!(both.creature_count(), 4);
        let wisp_info = both.creature_info(wisp).unwrap();
        let marsh = wisp_info.faction.unwrap();
        assert_eq!(
            both.factions()[marsh.index()].key.as_str(),
            "synthetic-ext:marsh"
        );
        assert_eq!(wisp_info.level, 2);
        let brute_info = both.creature_info(brute).unwrap();
        assert_eq!((brute_info.growth, brute_info.horde_growth), (8, 3));
        assert_eq!(brute_info.cost[0].1, 110);
        // Battle stats are untouched by these overrides.
        assert_eq!(both.creature(brute), base.creature(brute));

        // The expansion alone is rejected: its dependency is not enabled.
        assert_eq!(
            compile(&[SYNTHETIC_EXT_PACK], "synthetic:faithful"),
            Err(PackError::Compile(CompileError::MissingDependency {
                pack: "synthetic-ext".into(),
                depends: "synthetic".into()
            }))
        );
    }

    #[test]
    fn harvested_packs_compile_in_every_allowed_combination() {
        let profile = "base:faithful";
        let roe = compile(&[BASE_PACK], profile).unwrap();
        let ab = compile(&[BASE_PACK, AB_PACK], profile).unwrap();
        let sod = compile(&[SOD_PACK, AB_PACK, BASE_PACK], profile).unwrap();
        // RoE: 8 towns x 14 + 6 neutrals; AB adds Conflux (10 new) and 13 neutrals.
        assert_eq!(roe.creature_count(), 118);
        assert_eq!(ab.creature_count(), 141);
        assert_eq!(sod.creature_count(), 141);
        assert_eq!((roe.factions().len(), ab.factions().len()), (8, 9));
        // SoD requires AB; AB requires base.
        assert!(matches!(
            compile(&[BASE_PACK, SOD_PACK], profile),
            Err(PackError::Compile(CompileError::MissingDependency { .. }))
        ));
        assert!(compile(&[AB_PACK], profile).is_err());
        // Spot checks of the version differences (sources: crates/h3-harvest/data/gapfill.toml).
        let gems_of = |r: &Ruleset, key: &str| {
            let info = r.creature_info(r.creature_by_key(key).unwrap()).unwrap();
            info.cost
                .iter()
                .find(|(res, _)| r.resources()[res.index()].key.as_str() == "base:gems")
                .map_or(0, |(_, n)| *n)
        };
        assert_eq!(gems_of(&ab, "base:angel"), 0);
        assert_eq!(gems_of(&sod, "base:angel"), 1);
        let air = roe.creature_by_key("base:air_elemental").unwrap();
        assert_eq!(roe.creature_info(air).unwrap().faction, None);
        assert_eq!(roe.creature_info(air).unwrap().level, 4);
        assert_eq!(ab.creature_info(air).unwrap().level, 2);
        let lizard = roe.creature_by_key("base:lizardman").unwrap();
        assert_eq!(roe.creature_info(lizard).unwrap().growth, 9);
        let cerberus = roe.creature_by_key("base:cerberus").unwrap();
        assert_eq!(roe.creature(cerberus).unwrap().damage_max, 5);
        assert_eq!(sod.creature(cerberus).unwrap().damage_max, 7);
        assert_eq!(roe.battle().attack_bonus_cap_percent, 300);
        assert_eq!(roe.battle().formation_row(7, 3), Some(5));
        // Abilities (E03 T2): RoE elementals hate their opposites; AB adds the Conflux ones.
        assert_eq!(roe.abilities(air).len(), 1);
        assert_eq!(ab.abilities(air).len(), 2);
        let cavalier = roe.creature_by_key("base:cavalier").unwrap();
        assert_eq!(
            roe.abilities(cavalier),
            [h3_content::Ability::Jousting { percent_per_hex: 5 }]
        );
    }

    #[test]
    fn override_syntax() {
        let with = |extra: &str| std::format!("{SYNTHETIC_EXT_PACK}\n{extra}");
        let full = with(
            "[[creature_override]]\ntarget = \"synthetic:spearman\"\nattack = 9\ndefense = 9\n\
             damage = [2, 6]\nhit_points = 12\nspeed = 5\nname = \"Pike\"\nplural = \"Pikes\"\n\
             faction = \"\"\nshots = 1\ncasts = 2\nhorde_growth = 1\nfight_value = 1\n\
             ai_value = 2\nadventure_count = [1, 2]\nflags = [\"undead\"]\n",
        );
        let r = compile(&[SYNTHETIC_PACK, &full], "synthetic:faithful").unwrap();
        let id = r.creature_by_key("synthetic:spearman").unwrap();
        let stats = r.creature(id).unwrap();
        assert_eq!(
            (
                stats.attack,
                stats.damage_min,
                stats.damage_max,
                stats.speed
            ),
            (9, 2, 6, 5)
        );
        let info = r.creature_info(id).unwrap();
        assert_eq!(info.faction, None); // `faction = ""` makes it neutral
        assert_eq!(
            (info.name.as_str(), stats.shots, info.casts),
            ("Pike", 1, 2)
        );
        assert!(stats.flags.contains(h3_content::CreatureFlag::Undead));

        let bad_target = with("[[creature_override]]\ntarget = \"spearman\"\n");
        assert!(matches!(parse_pack(&bad_target), Err(PackError::Key(_))));
        let bad_ref =
            with("[[creature_override]]\ntarget = \"synthetic:spearman\"\nfaction = \"A:b\"\n");
        assert!(matches!(parse_pack(&bad_ref), Err(PackError::Key(_))));
        let bad_cost =
            with("[[creature_override]]\ntarget = \"synthetic:spearman\"\ncost = { \"X\" = 1 }\n");
        assert!(matches!(parse_pack(&bad_cost), Err(PackError::Key(_))));
        let bad_resource = SYNTHETIC_PACK.replace("id = \"coin\"", "id = \"Coin\"");
        assert!(matches!(parse_pack(&bad_resource), Err(PackError::Key(_))));
        let bad_flag = SYNTHETIC_PACK.replace("flags = [\"alive\"]", "flags = [\"swims\"]");
        assert!(matches!(
            compile(&[&bad_flag], "synthetic:faithful"),
            Err(PackError::Compile(CompileError::UnknownFlag(_)))
        ));
    }

    #[test]
    fn ability_syntax() {
        use h3_content::{Ability, AreaShot, AuraTarget};
        // Every kind with its parameters; references follow the usual rules.
        let all = "abilities = [\
            { kind = \"hate\", creature = \"wisp\", percent = 50 }, \
            { kind = \"hate\", creature = \"synthetic:brute\", percent = 100 }, \
            { kind = \"jousting\", percent_per_hex = 5 }, { kind = \"charge_immunity\" }, \
            { kind = \"defense_reduction\", percent = 40 }, \
            { kind = \"death_blow\", chance_percent = 20 }, \
            { kind = \"life_drain\", percent = 100 }, { kind = \"regeneration\" }, \
            { kind = \"death_stare\", chance_percent = 10 }, \
            { kind = \"extra_retaliations\", count = 1 }, { kind = \"strike_and_return\" }, \
            { kind = \"teleports\" }, \
            { kind = \"morale_aura\", amount = 1, affects = \"allies\" }, \
            { kind = \"luck_aura\", amount = -1, affects = \"enemies\" }, \
            { kind = \"minimum_morale\", value = 1 }, { kind = \"minimum_luck\", value = 1 }, \
            { kind = \"no_distance_penalty\" }]\n";
        let text = SYNTHETIC_PACK.replace(
            "flags = [\"alive\"]\n",
            &std::format!("flags = [\"alive\"]\n{all}"),
        );
        let r = compile(&[&text], "synthetic:faithful").unwrap();
        let spearman = r.creature_by_key("synthetic:spearman").unwrap();
        let wisp = r.creature_by_key("synthetic:wisp").unwrap();
        let list = r.abilities(spearman);
        assert_eq!(list.len(), 17);
        assert_eq!(
            list[0],
            Ability::Hate {
                target: wisp,
                percent: 50
            }
        );
        assert_eq!(
            list[13],
            Ability::LuckAura {
                amount: -1,
                affects: AuraTarget::Enemies
            }
        );
        // Patterns and area shots need their flags; overrides replace the whole list.
        let ext = std::format!(
            "{SYNTHETIC_EXT_PACK}\n[[creature_override]]\ntarget = \"synthetic:wisp\"\n\
             flags = [\"multi_headed\", \"area_attack\"]\nabilities = [\
             {{ kind = \"attacks_all_adjacent\" }}, {{ kind = \"area_shot\", effect = \"death_cloud\" }}]\n\
             [[creature_override]]\ntarget = \"synthetic:brute\"\nflags = [\"multi_headed\", \"area_attack\"]\n\
             abilities = [{{ kind = \"three_headed_attack\" }}, {{ kind = \"unlimited_retaliations\" }}, \
             {{ kind = \"area_shot\", effect = \"fireball\" }}]\n"
        );
        let r = compile(&[SYNTHETIC_PACK, &ext], "synthetic:faithful").unwrap();
        assert_eq!(
            r.abilities(wisp),
            [
                Ability::AttacksAllAdjacent,
                Ability::AreaShot {
                    effect: AreaShot::DeathCloud
                }
            ]
        );
        let brute = r.creature_by_key("synthetic:brute").unwrap();
        assert_eq!(r.abilities(brute).len(), 3);
        // Strict parsing: unknown kinds, unknown or missing parameters, bad references.
        for bad in [
            "abilities = [{ kind = \"swims\" }]",
            "abilities = [{ kind = \"teleports\", range = 3 }]",
            "abilities = [{ kind = \"jousting\" }]",
            "abilities = [{ kind = \"morale_aura\", amount = 1, affects = \"everyone\" }]",
            "abilities = [{ kind = \"area_shot\", effect = \"lightning\" }]",
        ] {
            let text = SYNTHETIC_PACK.replace(
                "flags = [\"alive\"]\n",
                &std::format!("flags = [\"alive\"]\n{bad}\n"),
            );
            assert!(
                matches!(parse_pack(&text), Err(PackError::Syntax(_))),
                "{bad}"
            );
        }
        let bad_ref = SYNTHETIC_PACK.replace(
            "flags = [\"alive\"]\n",
            "flags = [\"alive\"]\nabilities = [{ kind = \"hate\", creature = \"A:b\", percent = 1 }]\n",
        );
        assert!(matches!(parse_pack(&bad_ref), Err(PackError::Key(_))));
        let bad_override = std::format!(
            "{SYNTHETIC_EXT_PACK}\n[[creature_override]]\ntarget = \"synthetic:wisp\"\n\
             abilities = [{{ kind = \"hate\", creature = \"A:b\", percent = 1 }}]\n"
        );
        assert!(matches!(parse_pack(&bad_override), Err(PackError::Key(_))));
    }

    #[test]
    fn error_display() {
        assert_eq!(PackError::Syntax("x".into()).to_string(), "syntax: x");
        assert_eq!(
            PackError::UnknownCreature("a:b".into()).to_string(),
            "unknown creature `a:b`"
        );
        let key = ContentKey::parse("bad").unwrap_err();
        assert!(PackError::from(key).to_string().contains("not a valid key"));
        assert_eq!(
            PackError::from(CompileError::MissingBattleParams).to_string(),
            "no pack provides battle parameters"
        );
        assert_eq!(
            PackError::UnknownTerrain("a:t".into()).to_string(),
            "unknown terrain `a:t`"
        );
    }
}
