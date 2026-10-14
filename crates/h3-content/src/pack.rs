//! Content packs as authored (Tier 1 data and Tier 2 parameters, ADR-0006), before compilation.
//!
//! These types are the output of an adapter (e.g. `h3-pack` parsing TOML) and the input of
//! [`crate::Ruleset::compile`]. They are plain data: all validation happens in the compiler, so
//! every adapter gets the same checks. The battle's part of a pack (obstacles, battle parameters,
//! creatures' and spells' battle rows) is `h3-content-battle`'s `BattlePack` (R01).

use alloc::string::String;
use alloc::vec::Vec;

use crate::artifacts::ArtifactClass;
use crate::effect::HeroEffectDef;
use crate::heroes::SpellCategory;
use crate::key::ContentKey;

/// Identity and dependencies of a pack.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackManifest {
    /// Pack id, the namespace of its keys (`[a-z0-9_-]+`).
    pub id: String,
    /// Free-form version string, recorded in the ruleset identity.
    pub version: String,
    /// Ids of packs this pack requires. They must be enabled too; the compiler orders packs so
    /// that dependencies load first (ADR-0006, E02 amendment).
    pub depends: Vec<String>,
}

/// A resource kind (gold, wood, …). Resource kinds are data, not a fixed enum (ADR-0006
/// guardrails).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceDef {
    /// Namespaced key.
    pub key: ContentKey,
    /// Display name.
    pub name: String,
}

/// A faction (town type). Creatures refer to it; towns come later (E09).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FactionDef {
    /// Namespaced key.
    pub key: ContentKey,
    /// Display name.
    pub name: String,
    /// Native terrain of its creatures (BAT-TER-001), if any.
    pub native_terrain: Option<ContentKey>,
}

/// A terrain type (E03): battles are fought on one (BAT-TER-001, BAT-GEO-004).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerrainDef {
    /// Namespaced key.
    pub key: ContentKey,
    /// Display name.
    pub name: String,
}

/// Non-battle attributes of a creature type (E02 pack format v2; its battle row is
/// `h3-content-battle`'s since R01). The defaults describe a neutral, free, level-1 creature
/// without flags.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatureInfoDef {
    /// Display name, singular.
    pub name: String,
    /// Display name, plural.
    pub plural: String,
    /// Faction, `None` for neutral creatures.
    pub faction: Option<ContentKey>,
    /// Level (1 = lowest dwelling tier).
    pub level: u8,
    /// Recruitment cost per creature: `(resource, amount)`.
    pub cost: Vec<(ContentKey, u32)>,
    /// Weekly growth of the basic dwelling.
    pub growth: u16,
    /// Extra weekly growth from the horde building.
    pub horde_growth: u16,
    /// Fight value (original table column; used for army strength estimates).
    pub fight_value: u32,
    /// AI value (original table column).
    pub ai_value: u32,
    /// Size range of a wandering stack on the adventure map, `[low, high]`.
    pub adventure_count: [u16; 2],
    /// Creature flags by name, from the closed set [`crate::CreatureFlag`].
    pub flags: Vec<String>,
    /// The creature types this one upgrades into (E04a T3; at most
    /// [`crate::MAX_UPGRADES`]).
    pub upgrades: Vec<ContentKey>,
}

impl Default for CreatureInfoDef {
    fn default() -> Self {
        Self {
            name: String::new(),
            plural: String::new(),
            faction: None,
            level: 1,
            cost: Vec::new(),
            growth: 0,
            horde_growth: 0,
            fight_value: 0,
            ai_value: 0,
            adventure_count: [0, 0],
            flags: Vec::new(),
            upgrades: Vec::new(),
        }
    }
}

/// A creature type definition: its identity and non-battle data (the battle row is
/// `h3-content-battle`'s, R01).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatureDef {
    /// Namespaced key.
    pub key: ContentKey,
    /// Speed (hexes per turn; also initiative).
    pub speed: u8,
    /// Non-battle attributes.
    pub info: CreatureInfoDef,
}

/// A partial change to a creature defined by a pack in the overriding pack's dependency closure
/// (e.g. SoD changing an AB creature): its core half (the battle half is `h3-content-battle`'s,
/// gated by the same flag, R01).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatureOverride {
    /// Key of the creature to change (`pack:name`).
    pub target: ContentKey,
    /// The data quirk this override fixes (a `quirk.content` flag), if it is gated: then it is
    /// applied only when the active profile turns the flag off (ADR-0007, amendment E04b T8e).
    pub fixes: Option<String>,
    /// The fields to change.
    pub patch: CreaturePatch,
}

/// The changed fields of a [`CreatureOverride`]. `None` fields stay as they are; list fields
/// replace the whole list.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CreaturePatch {
    /// New speed.
    pub speed: Option<u8>,
    /// New singular name.
    pub name: Option<String>,
    /// New plural name.
    pub plural: Option<String>,
    /// New faction; `Some(None)` makes the creature neutral.
    pub faction: Option<Option<ContentKey>>,
    /// New level.
    pub level: Option<u8>,
    /// New cost (replaces the whole list).
    pub cost: Option<Vec<(ContentKey, u32)>>,
    /// New growth.
    pub growth: Option<u16>,
    /// New horde growth.
    pub horde_growth: Option<u16>,
    /// New fight value.
    pub fight_value: Option<u32>,
    /// New AI value.
    pub ai_value: Option<u32>,
    /// New adventure-map count range.
    pub adventure_count: Option<[u16; 2]>,
    /// New flags (replaces the whole list).
    pub flags: Option<Vec<String>>,
    /// New upgrades (replaces the whole list).
    pub upgrades: Option<Vec<ContentKey>>,
}

/// A school of magic (E04a T3): spells belong to schools, school skills raise their mastery.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpellSchoolDef {
    /// Namespaced key.
    pub key: ContentKey,
    /// Display name.
    pub name: String,
    /// Where a damage spell of several schools checks this school's protection under quirk
    /// Q-0007, lower first (pack format v6; default 0).
    pub protection_rank: u8,
}

/// A spell's data row (E04a T3): the numbers of the original spell table. What it does in battle
/// (since pack format v4, E04b T2a) is its battle row, `h3-content-battle`'s (R01).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpellDef {
    /// Namespaced key.
    pub key: ContentKey,
    /// Display name.
    pub name: String,
    /// Where it is used.
    pub category: SpellCategory,
    /// Spell level (1..=5; creature abilities 0..=5).
    pub level: u8,
    /// Schools it belongs to.
    pub schools: Vec<ContentKey>,
    /// Mana cost by mastery: none, basic, advanced, expert.
    pub cost: [u16; 4],
    /// Power factor (effect per point of spell power).
    pub power: u16,
    /// Base effect value by mastery: none, basic, advanced, expert.
    pub effect: [u16; 4],
    /// Only a grant naming the spell gives it (pack format v30; Titan's Lightning Bolt).
    pub granted_by_name_only: bool,
}

/// A secondary skill with the effects of its three levels (E04a T3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecondarySkillDef {
    /// Namespaced key.
    pub key: ContentKey,
    /// Display name.
    pub name: String,
    /// Effects at Basic, Advanced and Expert. Effects that matter only outside battle are
    /// modelled as their epics need them (E08 increment 3: Necromancy, Learning, Intelligence,
    /// Diplomacy's join bonus; the rest `packs/README.md` lists as pending).
    pub levels: [Vec<HeroEffectDef>; 3],
}

/// A hero class (E04a T3): the faction it belongs to, starting primary skills and the
/// probabilities of a level-up (hero progression arrives with E08–E10).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeroClassDef {
    /// Namespaced key.
    pub key: ContentKey,
    /// Display name.
    pub name: String,
    /// The faction (town) of its heroes.
    pub faction: ContentKey,
    /// Starting primary skills: attack, defense, power, knowledge.
    pub primary: [u8; 4],
    /// Chance in percent that a level-up raises each primary skill, for levels 2–9 and 10+
    /// (each row sums to 100).
    pub primary_gain: [[u8; 4]; 2],
    /// Relative weights of the secondary skills offered on level-ups; unlisted skills weigh 0.
    pub skill_weights: Vec<(ContentKey, u8)>,
}

/// One stack of a hero type's starting army.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StartingStackDef {
    /// The creature type.
    pub creature: ContentKey,
    /// Size range `[low, high]`.
    pub count: [u16; 2],
}

/// A partial change to a hero type of the overriding pack's dependency closure, the pack itself
/// included (E04b T8e: Roland's specialty, quirk Q-0006).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeroOverride {
    /// Key of the hero type to change (`pack:name`).
    pub target: ContentKey,
    /// The data quirk this override fixes, if it is gated (as [`CreatureOverride::fixes`]).
    pub fixes: Option<String>,
    /// The fields to change.
    pub patch: HeroPatch,
}

/// The changed fields of a [`HeroOverride`]. `None` fields stay as they are; list fields replace
/// the whole list.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct HeroPatch {
    /// New specialty (replaces the whole list).
    pub specialty: Option<Vec<HeroEffectDef>>,
}

/// A hero type (E04a T3): who a hero is when it enters the game. Level, experience, artifacts and
/// current skills belong to the hero's details (`h3-heroes`, ADR-0013).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeroTypeDef {
    /// Namespaced key.
    pub key: ContentKey,
    /// Display name.
    pub name: String,
    /// Hero class.
    pub class: ContentKey,
    /// Starting secondary skills with their levels (1..=3), in display order.
    pub skills: Vec<(ContentKey, u8)>,
    /// Starts with a spell book.
    pub spell_book: bool,
    /// Starting spells (need the spell book).
    pub spells: Vec<ContentKey>,
    /// Starting army.
    pub army: Vec<StartingStackDef>,
    /// Starting war machines (creature types with flag `war_machine`).
    pub war_machines: Vec<ContentKey>,
    /// The specialty, as effects (empty: none, or not modelled yet).
    pub specialty: Vec<HeroEffectDef>,
}

/// An artifact slot: a kind of equipment position (E04a T3b).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactSlotDef {
    /// Namespaced key.
    pub key: ContentKey,
    /// Display name.
    pub name: String,
    /// Positions of this kind on a hero.
    pub count: u8,
    /// Whether its artifacts may be carried in the backpack (pack default: true; not the war
    /// machines, the Catapult and the spell book, HER-ART-002).
    pub backpack: bool,
}

/// Changes to an artifact slot of the pack's closure (SoD's fifth misc position).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactSlotOverride {
    /// The slot changed.
    pub target: ContentKey,
    /// New display name.
    pub name: Option<String>,
    /// New number of positions.
    pub count: Option<u8>,
}

/// An artifact type (E04a T3b).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactDef {
    /// Namespaced key.
    pub key: ContentKey,
    /// Display name.
    pub name: String,
    /// Class.
    pub class: ArtifactClass,
    /// Value by resource (empty: none).
    pub cost: Vec<(ContentKey, u32)>,
    /// The slot it is worn in; `None`: backpack only.
    pub slot: Option<ContentKey>,
    /// The war machine it places in battle.
    pub war_machine: Option<ContentKey>,
    /// The secondary skill its effects need.
    pub needs_skill: Option<ContentKey>,
    /// Its own effects.
    pub effects: Vec<HeroEffectDef>,
    /// A combination's parts.
    pub parts: Vec<ContentKey>,
}

/// Hero parameters (Tier 2, E04a T4a): the limits of a hero's details.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HeroParamsDef {
    /// Lowest effective primary skills (attack, defense, power, knowledge); lower totals count
    /// as these (HER-DET-001).
    pub primary_min: [u8; 4],
    /// Highest primary skill, base and effective (HER-DET-001).
    pub primary_max: u8,
    /// Most artifacts in a hero's backpack (HER-ART-002).
    pub backpack: u16,
}

/// A profile: a named selection of quirk flags (ADR-0007).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileDef {
    /// Namespaced key.
    pub key: ContentKey,
    /// Quirk flags set explicitly by this profile (`(flag name, enabled)`); unlisted flags are off.
    pub quirks: Vec<(String, bool)>,
}

/// One content pack.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentPack {
    /// Identity and dependencies.
    pub manifest: PackManifest,
    /// Resource kinds.
    pub resources: Vec<ResourceDef>,
    /// Factions.
    pub factions: Vec<FactionDef>,
    /// Terrain types.
    pub terrains: Vec<TerrainDef>,
    /// Creature types.
    pub creatures: Vec<CreatureDef>,
    /// Changes to creatures of packs this pack depends on.
    pub creature_overrides: Vec<CreatureOverride>,
    /// Spell schools.
    pub spell_schools: Vec<SpellSchoolDef>,
    /// Spells (data rows).
    pub spells: Vec<SpellDef>,
    /// Secondary skills.
    pub skills: Vec<SecondarySkillDef>,
    /// Hero classes.
    pub hero_classes: Vec<HeroClassDef>,
    /// Hero types.
    pub heroes: Vec<HeroTypeDef>,
    /// Changes to hero types of the closure (E04b T8e).
    pub hero_overrides: Vec<HeroOverride>,
    /// Artifact slots (E04a T3b).
    pub artifact_slots: Vec<ArtifactSlotDef>,
    /// Changes to artifact slots of the closure.
    pub artifact_slot_overrides: Vec<ArtifactSlotOverride>,
    /// Artifact types (E04a T3b).
    pub artifacts: Vec<ArtifactDef>,
    /// Profiles.
    pub profiles: Vec<ProfileDef>,
    /// Hero parameters, if this pack provides them (E04a T4a).
    pub hero_params: Option<HeroParamsDef>,
}

impl ContentPack {
    /// An empty pack with the given manifest.
    pub fn new(manifest: PackManifest) -> Self {
        Self {
            manifest,
            resources: Vec::new(),
            factions: Vec::new(),
            terrains: Vec::new(),
            creatures: Vec::new(),
            creature_overrides: Vec::new(),
            spell_schools: Vec::new(),
            spells: Vec::new(),
            skills: Vec::new(),
            hero_classes: Vec::new(),
            heroes: Vec::new(),
            hero_overrides: Vec::new(),
            artifact_slots: Vec::new(),
            artifact_slot_overrides: Vec::new(),
            artifacts: Vec::new(),
            profiles: Vec::new(),
            hero_params: None,
        }
    }
}
