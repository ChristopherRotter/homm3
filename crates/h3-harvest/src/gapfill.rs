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
    /// Artifact facts (E04a T3b).
    pub artifacts: Artifacts,
    /// What spells do in battle beyond the executable's flags (E04b T2a).
    #[serde(default)]
    pub spells: Spells,
}

/// Spell facts the executable's spell table does not hold (E04b T2a).
#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Spells {
    /// Areas, chains and target conditions, per spell.
    #[serde(default)]
    pub targeting: Vec<SpellTargeting>,
    /// What timed spells change (E04b T3a), per spell.
    #[serde(default)]
    pub timed: Vec<SpellTimed>,
    /// Stack spells of the other effect kinds (dispel and heal, E04b T3b1b; teleport, T5a), per
    /// spell.
    #[serde(default)]
    pub other: Vec<SpellOther>,
    /// Spells that place battlefield obstacles (E04b T5a), per spell.
    #[serde(default)]
    pub obstacle: Vec<SpellObstacle>,
    /// What specialists in some spells add by a rule of the spell's own (E04b T3c).
    #[serde(default)]
    pub specialist: Vec<SpellSpecialist>,
    /// The order in which the original checks a damage spell's schools for the target's
    /// protections (school names; E04b T3b1, `installation.md` §7.8, §7.10): written as the
    /// schools' `protection_rank`.
    #[serde(default)]
    pub protection_order: Vec<String>,
    /// Sources of `protection_order`.
    #[serde(default)]
    pub protection_sources: Vec<String>,
}

/// The timed effects of one spell (pack format v5, `packs/README.md`; the original's code,
/// `docs/formats/installation.md` §7.9). Target, side, mass version, mind and the war-machine
/// exclusion come from the flag word.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpellTimed {
    /// Spell name (as in `SPTRAITS.TXT`).
    pub spell: String,
    /// Timed change names (`speed`, `attack_loss`, …).
    pub changes: Vec<String>,
    /// Spells (by name) whose effects a cast ends.
    #[serde(default)]
    pub counters: Vec<String>,
    /// `permanent` for spells that last the battle, `own_turn` for Frenzy.
    #[serde(default)]
    pub duration: Option<String>,
    /// Whether a cast on an affected stack applies the changes again.
    #[serde(default)]
    pub cumulative: bool,
    /// `damaged` for effects that any damage ends (Blind).
    #[serde(default)]
    pub ends: Option<String>,
    /// The mastery of the mass version without quirk Q-0011 (Forgetfulness: `expert`).
    #[serde(default)]
    pub fixed_mass: Option<String>,
    /// Creature flags a stack needs (Forgetfulness: `shooter`).
    #[serde(default)]
    pub requires: Vec<String>,
    /// Creature flags a stack may not have, besides `war_machine` from the flag word.
    #[serde(default)]
    pub excludes: Vec<String>,
    /// The rounds of `duration = "fixed"` (Berserk: 255).
    #[serde(default)]
    pub fixed_rounds: Option<u16>,
    /// `area = { inner, radius }` of an area timed spell (flag `0x80`: Berserk), copied verbatim.
    #[serde(default)]
    pub area: Option<toml::Value>,
    /// The side an area timed spell reaches (`any`, `enemy` or `own`; Berserk: `any`).
    #[serde(default)]
    pub side: Option<String>,
    /// A stack's full hit points must not exceed the cast's value (Hypnotize).
    #[serde(default)]
    pub health_limit: bool,
    /// Timed change names whose active effects keep a stack from being a target (Hypnotize:
    /// `magic_mirror`).
    #[serde(default)]
    pub refuses: Vec<String>,
    /// Source ids.
    pub sources: Vec<String>,
}

/// A stack spell with effects other than damage and timed changes (pack format v7,
/// `packs/README.md`; the original's code, `docs/formats/installation.md` §7.10). Target and
/// mass version come from the flag word, the side from the polarity unless given.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpellOther {
    /// Spell name (as in `SPTRAITS.TXT`).
    pub spell: String,
    /// Effect kind names (`dispel`, `heal`), in order.
    pub effects: Vec<String>,
    /// The side (`own` or `enemy`) of a neutral spell.
    #[serde(default)]
    pub side: Option<String>,
    /// The mastery from which it reaches either side.
    #[serde(default)]
    pub any_side: Option<String>,
    /// A stack it is aimed at needs an active effect.
    #[serde(default)]
    pub needs_effect: bool,
    /// No immunity, resistance or aura applies.
    #[serde(default)]
    pub ignores_immunity: bool,
    /// Spells (by name) whose effects alone its dispel effect ends.
    #[serde(default)]
    pub dispels: Vec<String>,
    /// Spells (by name) whose effects its dispel effect spares.
    #[serde(default)]
    pub spares: Vec<String>,
    /// The mastery from which a cast also removes every spell obstacle (Dispel: `expert`).
    #[serde(default)]
    pub clears_obstacles: Option<String>,
    /// Source ids.
    pub sources: Vec<String>,
}

/// A spell that places battlefield obstacles (pack format v13, `packs/README.md`; the original's
/// code, `docs/formats/installation.md` §7.15, §7.16). The target hex comes from the flag word;
/// a random shape (v14) has none.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpellObstacle {
    /// Spell name (as in `SPTRAITS.TXT`).
    pub spell: String,
    /// The obstacle kind (`wall`, `fire`, `pit`, `mine`).
    pub obstacle: String,
    /// `{ steps, hexes, random, mirrored, separate, rounds }`, copied verbatim.
    pub shape: toml::Value,
    /// Source ids.
    pub sources: Vec<String>,
}

/// The bonus a specialist in some spells adds to their value by a rule of the spells' own (pack
/// format v9, `packs/README.md`; the original's code, `docs/formats/installation.md` §7.12).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpellSpecialist {
    /// Spell names (as in `SPTRAITS.TXT`).
    pub spells: Vec<String>,
    /// `bonus = { kind, … }` in pack syntax, copied verbatim.
    pub bonus: toml::Value,
    /// Source ids.
    pub sources: Vec<String>,
}

/// The targeting of one spell beyond its harvested target kind (pack syntax, `packs/README.md`).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpellTargeting {
    /// Spell name (as in `SPTRAITS.TXT`).
    pub spell: String,
    /// `area = { inner, radius }`, copied verbatim.
    #[serde(default)]
    pub area: Option<toml::Value>,
    /// `chain = { targets, factor_bp }`, copied verbatim.
    #[serde(default)]
    pub chain: Option<toml::Value>,
    /// Creature flags a stack needs.
    #[serde(default)]
    pub requires: Vec<String>,
    /// Creature flags a stack may not have.
    #[serde(default)]
    pub excludes: Vec<String>,
    /// The side a stack spell may be aimed at (`enemy` or `own`) when the flag word and the
    /// polarity do not give it (Chain Lightning: the original's targeting code, E04b T2e).
    #[serde(default)]
    pub side: Option<String>,
    /// Source ids.
    pub sources: Vec<String>,
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
    /// Parameters of spell specialties (E04b T2a).
    pub spell_specialty: SpellSpecialty,
    /// Effects of skills without a value table in the executable.
    pub skill_effects: Vec<SkillEffects>,
    /// Specialty parts the executable's records cannot hold.
    #[serde(default)]
    pub specialty_extra: Vec<SpecialtyExtra>,
    /// Limits of a hero's details (E04a T4a).
    pub params: HeroParams,
}

/// The pack's `[hero_params]` (E04a T4a).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HeroParams {
    /// Lowest effective primary skills.
    pub primary_min: [u8; 4],
    /// Highest primary skill.
    pub primary_max: u8,
    /// Backpack capacity.
    pub backpack: u16,
    /// Source ids.
    pub sources: Vec<String>,
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

/// Parameters of spell specialties (specialty kind 3).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpellSpecialty {
    /// Growth per hero level before the division by the target's level, basis points.
    pub bp_per_level: u16,
    /// Heroes whose specialty is a fixed damage bonus for their spell instead.
    #[serde(default)]
    pub fixed: Vec<FixedSpellSpecialty>,
    /// Source ids.
    pub sources: Vec<String>,
}

/// A spell specialist with a fixed damage bonus.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FixedSpellSpecialty {
    /// Hero name (as in `HOTRAITS.TXT`).
    pub hero: String,
    /// Bonus in basis points.
    pub bp: u16,
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

/// Artifact facts the tables do not hold (E04a T3b). Artifacts are named as in the SoD
/// `ARTRAITS.TXT`.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Artifacts {
    /// The AB-era table's artifacts that are not RoE ones.
    pub ab: Vec<String>,
    /// Source ids for `ab`.
    pub ab_sources: Vec<String>,
    /// SoD rows that are not artifacts of the game (not harvested).
    pub unused: Vec<String>,
    /// Source ids for `unused`.
    pub unused_sources: Vec<String>,
    /// Slot kinds whose artifacts never go to the backpack (E04a T4a).
    pub no_backpack: Vec<String>,
    /// Source ids for `no_backpack`.
    pub no_backpack_sources: Vec<String>,
    /// Artifacts that place a war machine.
    #[serde(default)]
    pub war_machines: Vec<ArtifactWarMachine>,
    /// Combination artifacts and their parts.
    #[serde(default)]
    pub combinations: Vec<Combination>,
    /// Effects beyond the executable's primary skills.
    #[serde(default)]
    pub effects: Vec<ArtifactEffects>,
    /// Artifacts with effects no epic has modelled yet.
    #[serde(default)]
    pub pending: Vec<Pending>,
}

/// An artifact that places a war machine.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArtifactWarMachine {
    /// Artifact name.
    pub artifact: String,
    /// The war machine's singular name (`CRTRAITS.TXT`).
    pub creature: String,
    /// Source ids.
    pub sources: Vec<String>,
}

/// A combination artifact.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Combination {
    /// Artifact name.
    pub artifact: String,
    /// Its parts' names.
    pub parts: Vec<String>,
    /// Source ids.
    pub sources: Vec<String>,
}

/// Effects of one or more artifacts that share them.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArtifactEffects {
    /// Artifact names.
    pub artifacts: Vec<String>,
    /// Hero effects (`packs/README.md`); `creatures = { creature = "<name>" }` and
    /// `factions = ["<town name>", …]` are translated to keys.
    pub effects: Vec<AbilitySpec>,
    /// The secondary skill (`SSTRAITS.TXT` name) the effects need.
    #[serde(default)]
    pub needs_skill: Option<String>,
    /// Source ids.
    pub sources: Vec<String>,
}

/// Artifacts whose effects wait for a later epic (a note in the pack, checked for coverage).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pending {
    /// The epic that models the effects.
    pub epic: String,
    /// What waits, in a few words.
    pub what: String,
    /// Artifact names.
    pub artifacts: Vec<String>,
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
    pub tactics_zone_columns: Vec<u8>,
    pub tactics_sources: Vec<String>,
    pub surrender: Surrender,
    pub surrender_sources: Vec<String>,
    pub hero_casts_per_round: u8,
    pub magic_sources: Vec<String>,
}

/// The surrender price parameters of `base` (BAT-FLE-002).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Surrender {
    /// The resource the price is paid in (a key of `base`).
    pub resource: String,
    /// The share of the stacks' cost, percent.
    pub cost_percent: u16,
}

impl GapFill {
    /// Parses a gap-fill file and checks that every referenced source exists and has a URL and
    /// retrieval date.
    ///
    /// # Errors
    /// Syntax errors, unknown source ids, sources without URL (an `original-data` source may
    /// name a `docs/` file instead) or date, unused sources.
    pub fn parse(text: &str) -> Result<Self, String> {
        let g: Self = toml::from_str(text).map_err(|e| format!("gapfill: {e}"))?;
        let ids: BTreeSet<&str> = g.source.iter().map(|s| s.id.as_str()).collect();
        if ids.len() != g.source.len() {
            return Err("gapfill: duplicate source id".into());
        }
        for s in &g.source {
            // Code reading is cited by the repository document that records it.
            let url = s.url.starts_with("https://")
                || (s.kind == "original-data" && s.url.starts_with("docs/"));
            if !url || s.retrieved.len() != 10 {
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
                .chain(&g.battle.tactics_sources)
                .chain(&g.battle.surrender_sources)
                .chain(&g.battle.magic_sources)
                .chain(g.creature_abilities.iter().flat_map(|a| &a.sources))
                .chain(g.native_terrain.iter().flat_map(|n| &n.sources))
                .chain(&g.obstacles.sources)
                .chain(&g.war_machines.sources)
                .chain(&g.heroes.upgrade_sources)
                .chain(&g.heroes.creature_specialty.sources)
                .chain(&g.heroes.spell_specialty.sources)
                .chain(&g.heroes.params.sources)
                .chain(g.heroes.skill_effects.iter().flat_map(|e| &e.sources))
                .chain(g.heroes.specialty_extra.iter().flat_map(|e| &e.sources))
                .chain(&g.artifacts.ab_sources)
                .chain(&g.artifacts.unused_sources)
                .chain(&g.artifacts.no_backpack_sources)
                .chain(g.artifacts.war_machines.iter().flat_map(|w| &w.sources))
                .chain(g.artifacts.combinations.iter().flat_map(|c| &c.sources))
                .chain(g.artifacts.effects.iter().flat_map(|e| &e.sources))
                .chain(g.spells.targeting.iter().flat_map(|t| &t.sources))
                .chain(g.spells.timed.iter().flat_map(|t| &t.sources))
                .chain(g.spells.other.iter().flat_map(|t| &t.sources))
                .chain(g.spells.obstacle.iter().flat_map(|t| &t.sources))
                .chain(g.spells.specialist.iter().flat_map(|t| &t.sources))
                .chain(&g.spells.protection_sources);
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
        assert_eq!(g.artifacts.combinations.len(), 12);
        assert_eq!(g.artifacts.war_machines.len(), 3);
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
        // Only code reading may cite a repository document.
        let doc = GAPFILL.replace(
            "https://heroes.thelazy.net/index.php/Cerberus",
            "docs/formats/installation.md",
        );
        assert!(GapFill::parse(&doc).unwrap_err().contains("https URL"));
        assert!(GAPFILL.contains("kind = \"original-data\"\nref"));
    }
}
