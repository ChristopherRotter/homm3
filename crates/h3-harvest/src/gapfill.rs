//! The committed gap-fill file (`data/gapfill.toml`): facts the installation cannot tell, each with
//! a labelled source.

use std::collections::BTreeSet;

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
    pub formation: Vec<Vec<u8>>,
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
                .chain(&g.battle.formation_sources);
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
        assert_eq!(g.battle.formation.len(), usize::from(g.battle.army_slots));
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
            "sources = [\"wiki-cerberus\"]",
            "sources = [\"wiki-angel\"]",
        );
        assert!(GapFill::parse(&unused).unwrap_err().contains("not used"));
        let no_url = GAPFILL.replace("https://heroes.thelazy.net/index.php/Cerberus", "n/a");
        assert!(GapFill::parse(&no_url).unwrap_err().contains("https URL"));
    }
}
