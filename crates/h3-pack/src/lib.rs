//! Adapter: parses human-editable content packs and scenarios (TOML) into content definitions and
//! battle setups (ADR-0006). Pack text is untrusted input: parsing is strict (unknown fields are
//! errors, numbers must fit their types) and every definition is validated again by the
//! `Ruleset` compiler.
//!
//! Formats: `packs/README.md`; examples: `packs/synthetic/`.

use std::collections::BTreeMap;
use std::fmt;

use h3_content::{
    BattleParamsDef, CompileError, ContentKey, ContentPack, CreatureDef, CreatureInfoDef,
    CreatureOverride, CreaturePatch, FactionDef, KeyError, PackManifest, ProfileDef, ResourceDef,
    Ruleset,
};
use h3_contracts::{Army, ArmyStack, BattleSetup};
use serde::Deserialize;

/// The synthetic content pack shipped with the repo (tests, benchmarks, FFI smoke, replays).
pub const SYNTHETIC_PACK: &str = include_str!("../../../packs/synthetic/pack.toml");
/// Synthetic expansion pack (depends on the synthetic pack; dependencies and overrides).
pub const SYNTHETIC_EXT_PACK: &str = include_str!("../../../packs/synthetic-ext/pack.toml");
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
    resource: Vec<NamedDto>,
    #[serde(default)]
    faction: Vec<NamedDto>,
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
    let factions = named(file.faction)?
        .into_iter()
        .map(|(key, name)| FactionDef { key, name })
        .collect();
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
        packs: file.scenario.packs,
        profile: file.scenario.profile,
        attacker: convert(file.attacker),
        defender: convert(file.defender),
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
        let empty =
            parse_scenario("[scenario]\nid = \"e\"\npacks = []\nprofile = \"a:b\"\n").unwrap();
        assert_eq!(empty.id, "e");
        assert!(empty.packs.is_empty());
        assert_eq!(parse_scenario(SYNTHETIC_DUEL).unwrap().packs, ["synthetic"]);
    }

    #[test]
    fn format_v2_fields_and_expansion_packs() {
        let base = compile(&[SYNTHETIC_PACK], "synthetic:faithful").unwrap();
        assert_eq!(base.resources().len(), 2);
        assert_eq!(base.factions()[0].name, "Vale");
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
        assert!(info.flags.contains(h3_content::CreatureFlag::TwoHex));
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
        assert_eq!((info.name.as_str(), info.shots, info.casts), ("Pike", 1, 2));
        assert!(info.flags.contains(h3_content::CreatureFlag::Undead));

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
