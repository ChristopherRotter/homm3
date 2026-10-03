//! The committed gap-fill file (`data/gapfill.toml`): facts the installation cannot tell, each with
//! a labelled source.

use std::collections::{BTreeMap, BTreeSet};

use serde::Deserialize;

/// The gap-fill file shipped with the harvester.
pub const GAPFILL: &str = include_str!("../data/gapfill.toml");

/// Parsed gap-fill file.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GapFill {
    /// Labelled sources.
    pub source: Vec<Source>,
    /// What Armageddon's Blade added.
    pub ab: AbAdditions,
    /// RoE creatures whose level/faction differ in RoE.
    #[serde(default)]
    pub roe_creature: Vec<RoeCreature>,
    /// Classified rows that differ between the AB-era and SoD tables.
    #[serde(default)]
    pub table_change: Vec<TableChange>,
    /// Battle parameters of `base`.
    pub battle: Battle,
    /// Creature abilities (E03 T2; ADR-0012): behaviour the original hard-codes per creature.
    #[serde(default)]
    pub creature_abilities: Vec<CreatureAbilities>,
    /// Native terrains of the factions (E03 T7, BAT-TER-001).
    #[serde(default)]
    pub native_terrain: Vec<NativeTerrain>,
    /// Obstacle types (E03 T7, BAT-GEO-004).
    pub obstacles: Obstacles,
    /// War machines harvested like creatures (E03 T8).
    pub war_machines: WarMachines,
    /// Hero content (E04a T3).
    pub heroes: Heroes,
}

/// Hero content the installation does not hold (E04a T3).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Heroes {
    /// Upgrades the pairing rule cannot find, as (creature, upgrade) singular names.
    pub upgrades: Vec<[String; 2]>,
    /// Source ids of `upgrades`.
    pub upgrade_sources: Vec<String>,
    /// Parameters of creature specialties.
    pub creature_specialty: CreatureSpecialty,
    /// Effects of skills without a value table in the executable.
    pub skill_effects: Vec<SkillEffects>,
    /// Specialty parts the executable's records cannot hold.
    #[serde(default)]
    pub specialty_extra: Vec<SpecialtyExtra>,
}

/// Parameters of creature specialties (specialty kind 1).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreatureSpecialty {
    /// Growth per step, basis points.
    pub bp_per_step: u16,
    /// Speed bonus.
    pub speed: u8,
    /// Speed bonus for war machines.
    pub war_machine_speed: u8,
    /// Source ids.
    pub sources: Vec<String>,
}

/// The effects of one skill's levels, in pack syntax (`kind` plus parameters, no references).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SkillEffects {
    /// Skill name (as in `SSTRAITS.TXT`).
    pub skill: String,
    /// Basic.
    pub basic: Vec<AbilitySpec>,
    /// Advanced.
    pub advanced: Vec<AbilitySpec>,
    /// Expert.
    pub expert: Vec<AbilitySpec>,
    /// Source ids.
    pub sources: Vec<String>,
}

/// An extra `creature_stat` effect of a hero's specialty.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpecialtyExtra {
    /// Hero name (as in `HOTRAITS.TXT`).
    pub hero: String,
    /// Creature singular name (the effect applies to it and its upgrades).
    pub creature: String,
    /// Statistic (pack name).
    pub stat: String,
    /// Change.
    pub amount: i16,
    /// Source ids.
    pub sources: Vec<String>,
}

/// The war machines the battle supports (E03 T8, BAT-WAR-001); the others (siege machines)
/// stay unharvested.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WarMachines {
    /// Singular names (as in `CRTRAITS.TXT`).
    pub harvest: Vec<String>,
    /// Source ids.
    pub sources: Vec<String>,
}

/// The native terrain of a faction.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeTerrain {
    /// Town name (as in `TownType.txt`).
    pub faction: String,
    /// Terrain name (as in `TERRNAME.txt`).
    pub terrain: String,
    /// Source ids.
    pub sources: Vec<String>,
}

/// The obstacle types, with their sources.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Obstacles {
    /// Source ids.
    pub sources: Vec<String>,
    /// The obstacle types.
    pub list: Vec<ObstacleSpec>,
}

/// One obstacle type (pack syntax, `packs/README.md`), terrains by name.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
#[allow(
    missing_docs,
    reason = "field names mirror the pack format (packs/README.md)"
)]
pub struct ObstacleSpec {
    pub id: u32,
    pub terrains: Vec<String>,
    #[serde(default)]
    pub width: u8,
    #[serde(default)]
    pub height: u8,
    pub blocked: Vec<[i8; 2]>,
    #[serde(default)]
    pub absolute: bool,
}

/// The abilities of one creature (`packs/README.md`, "Abilities").
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreatureAbilities {
    /// Singular name (as in `CRTRAITS.TXT`).
    pub creature: String,
    /// The abilities, in pack order.
    pub abilities: Vec<AbilitySpec>,
    /// Source ids.
    pub sources: Vec<String>,
}

/// One ability as written in the gap-fill file: the pack syntax, except that a referenced
/// creature is named by its singular name. The harvester turns the name into a key; the other
/// parameters are copied verbatim (the generated packs are compiled, which validates them).
#[derive(Debug, Clone, Deserialize)]
pub struct AbilitySpec {
    /// Ability kind (`hate`, `jousting`, …).
    pub kind: String,
    /// Referenced creature (`hate`), by singular name.
    #[serde(default)]
    pub creature: Option<String>,
    /// The other parameters.
    #[serde(flatten)]
    pub params: BTreeMap<String, toml::Value>,
}

/// A labelled source (docs/04-rules-and-sources.md).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Source {
    /// Id used by entries.
    pub id: String,
    /// Source kind (`wiki`, `vcmi`, …).
    pub kind: String,
    /// Page or file and what it says, in our words.
    #[serde(rename = "ref")]
    pub reference: String,
    /// Link.
    pub url: String,
    /// Retrieval date.
    pub retrieved: String,
}

/// Content added by Armageddon's Blade.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AbAdditions {
    /// Source ids.
    pub sources: Vec<String>,
    /// Town names (as in `TownType.txt`).
    pub factions: Vec<String>,
    /// Creature singular names (as in `CRTRAITS.TXT`).
    pub creatures: Vec<String>,
}

/// A RoE creature that was neutral in RoE at a different level.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RoeCreature {
    /// Singular name.
    pub name: String,
    /// RoE level (1-based).
    pub level: u8,
    /// Source ids.
    pub sources: Vec<String>,
}

/// When a changed table row took effect.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Since {
    /// Shipped in the latest RoE/AB patches too.
    Base,
    /// SoD only.
    Sod,
}

/// A classified row difference.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TableChange {
    /// Singular name.
    pub name: String,
    /// When the SoD values took effect.
    pub since: Since,
    /// Source ids.
    pub sources: Vec<String>,
}

/// Battle parameters of `base` (written verbatim into the pack).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
#[allow(
    missing_docs,
    reason = "field names mirror the pack format (packs/README.md)"
)]
pub struct Battle {
    pub field_width: u8,
    pub field_height: u8,
    pub side_columns: u8,
    pub army_slots: u8,
    pub damage_roll_samples: u8,
    pub attack_bonus_per_point_percent: u16,
    pub attack_bonus_cap_percent: u16,
    pub defense_reduction_per_point_permille: u16,
    pub defense_reduction_cap_permille: u16,
    pub defend_bonus_percent: u16,
    pub defend_bonus_min: u16,
    pub ranged_full_damage_distance: u8,
    pub ranged_penalty_percent: u8,
    pub melee_penalty_percent: u8,
    pub morale_dice: u8,
    pub good_morale_chances: Vec<u8>,
    pub bad_morale_chances: Vec<u8>,
    pub luck_dice: u8,
    pub good_luck_chances: Vec<u8>,
    pub war_machine_positions: Vec<[u8; 2]>,
    pub loose_formation: Vec<Vec<u8>>,
    pub tight_formation: Vec<Vec<u8>>,
    pub formation_sources: Vec<String>,
}

impl GapFill {
    /// Parses a gap-fill file and checks that every referenced source exists and has a URL and
    /// retrieval date.
    ///
    /// # Errors
    /// Syntax errors, unknown source ids, sources without URL/date, unused sources.
    pub fn parse(text: &str) -> Result<Self, String> {
        let g: Self = toml::from_str(text).map_err(|e| format!("gapfill: {e}"))?;
        let ids: BTreeSet<&str> = g.source.iter().map(|s| s.id.as_str()).collect();
        if ids.len() != g.source.len() {
            return Err("gapfill: duplicate source id".into());
        }
        for s in &g.source {
            if !s.url.starts_with("https://") || s.retrieved.len() != 10 {
                return Err(format!(
                    "gapfill: source `{}` needs an https URL and a YYYY-MM-DD date",
                    s.id
                ));
            }
        }
        let mut used = BTreeSet::new();
        let refs =
            g.ab.sources
                .iter()
                .chain(g.roe_creature.iter().flat_map(|r| &r.sources))
                .chain(g.table_change.iter().flat_map(|t| &t.sources))
                .chain(&g.battle.formation_sources)
                .chain(g.creature_abilities.iter().flat_map(|a| &a.sources))
                .chain(g.native_terrain.iter().flat_map(|n| &n.sources))
                .chain(&g.obstacles.sources)
                .chain(&g.war_machines.sources)
                .chain(&g.heroes.upgrade_sources)
                .chain(&g.heroes.creature_specialty.sources)
                .chain(g.heroes.skill_effects.iter().flat_map(|e| &e.sources))
                .chain(g.heroes.specialty_extra.iter().flat_map(|e| &e.sources));
        for r in refs {
            if !ids.contains(r.as_str()) {
                return Err(format!("gapfill: unknown source `{r}`"));
            }
            used.insert(r.as_str());
        }
        if let Some(unused) = ids.difference(&used).next() {
            return Err(format!("gapfill: source `{unused}` is not used"));
        }
        Ok(g)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shipped_file_parses() {
        let g = GapFill::parse(GAPFILL).unwrap();
        assert_eq!(g.ab.factions, ["Conflux"]);
        assert_eq!(g.roe_creature.len(), 4);
        assert_eq!(
            g.battle.loose_formation.len(),
            usize::from(g.battle.army_slots)
        );
        assert_eq!(
            g.battle.tight_formation.len(),
            usize::from(g.battle.army_slots)
        );
    }

    #[test]
    fn source_references_are_checked() {
        let unknown = GAPFILL.replace("sources = [\"wiki-cerberus\"]", "sources = [\"nope\"]");
        assert!(
            GapFill::parse(&unknown)
                .unwrap_err()
                .contains("unknown source `nope`")
        );
        let unused = GAPFILL.replace(
            "formation_sources = [\"vcmi-layouts\", \"vcmi-layouts-tight\", \"wiki-combat\"]",
            "formation_sources = [\"wiki-angel\"]",
        );
        assert!(GapFill::parse(&unused).unwrap_err().contains("not used"));
        let no_url = GAPFILL.replace("https://heroes.thelazy.net/index.php/Cerberus", "n/a");
        assert!(GapFill::parse(&no_url).unwrap_err().contains("https URL"));
    }
}
