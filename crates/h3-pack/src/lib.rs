//! Adapter: parses human-editable content packs and scenarios (TOML) into content definitions and
//! battle setups (ADR-0006). Pack text is untrusted input: parsing is strict (unknown fields are
//! errors, numbers must fit their types) and every definition is validated again by the
//! `Ruleset` compiler.
//!
//! Formats: `packs/README.md`; examples: `packs/synthetic/`.

use std::collections::BTreeMap;
use std::fmt;

use h3_content::{
    BattleParamsDef, CompileError, ContentKey, ContentPack, CreatureDef, KeyError, PackManifest,
    ProfileDef, Ruleset,
};
use h3_contracts::{Army, ArmyStack, BattleSetup};
use serde::Deserialize;

/// The synthetic content pack shipped with the repo (tests, benchmarks, FFI smoke, replays).
pub const SYNTHETIC_PACK: &str = include_str!("../../../packs/synthetic/pack.toml");
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
}

impl fmt::Display for PackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Syntax(e) => write!(f, "syntax: {e}"),
            Self::Key(e) => write!(f, "{e}"),
            Self::Compile(e) => write!(f, "{e}"),
            Self::UnknownCreature(k) => write!(f, "unknown creature `{k}`"),
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
    creature: Vec<CreatureDto>,
    #[serde(default)]
    profile: Vec<ProfileDto>,
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
        formation: b.formation,
    });
    Ok(ContentPack {
        manifest: PackManifest {
            id: pack_id,
            version: file.pack.version,
            depends: file.pack.depends,
        },
        creatures,
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
    profile: String,
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
    /// Profile key the scenario is meant to run with.
    pub profile: String,
    attacker: Vec<(u8, String, u32)>,
    defender: Vec<(u8, String, u32)>,
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
        Ok(BattleSetup {
            attacker: army(&self.attacker)?,
            defender: army(&self.defender)?,
        })
    }
}

/// Parses a scenario.
pub fn parse_scenario(text: &str) -> Result<ScenarioDef, PackError> {
    let file: ScenarioFile = toml::from_str(text).map_err(|e| PackError::Syntax(e.to_string()))?;
    ContentKey::parse(&file.scenario.profile)?;
    let convert = |v: Vec<StackDto>| {
        v.into_iter()
            .map(|s| (s.slot, s.creature, s.count))
            .collect()
    };
    Ok(ScenarioDef {
        id: file.scenario.id,
        profile: file.scenario.profile,
        attacker: convert(file.attacker),
        defender: convert(file.defender),
    })
}

/// Parses and compiles packs (in load order) with a profile.
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
            "id = \"faithful\"",
            "id = \"faithful\"\nquirks = { \"quirk.x\" = true }",
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
        let empty = parse_scenario("[scenario]\nid = \"e\"\nprofile = \"a:b\"\n").unwrap();
        assert_eq!(empty.id, "e");
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
    }
}
