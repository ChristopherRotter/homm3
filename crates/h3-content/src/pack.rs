//! Content packs as authored (Tier 1 data and Tier 2 parameters, ADR-0006), before compilation.
//!
//! These types are the output of an adapter (e.g. `h3-pack` parsing TOML) and the input of
//! [`crate::Ruleset::compile`]. They are plain data: all validation happens in the compiler, so
//! every adapter gets the same checks.

use alloc::string::String;
use alloc::vec::Vec;

use crate::ability::AbilityDef;
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
}

/// Non-battle attributes of a creature type (E02 pack format v2). The defaults describe a
/// neutral, free, level-1 creature without flags.
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
    /// Ranged shots (0 = not a shooter by ammunition).
    pub shots: u16,
    /// Spell casts per battle.
    pub casts: u8,
    /// Size range of a wandering stack on the adventure map, `[low, high]`.
    pub adventure_count: [u16; 2],
    /// Creature flags by name, from the closed set [`crate::CreatureFlag`].
    pub flags: Vec<String>,
    /// Abilities (ADR-0012), from the closed set [`crate::Ability`].
    pub abilities: Vec<AbilityDef>,
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
            shots: 0,
            casts: 0,
            adventure_count: [0, 0],
            flags: Vec::new(),
            abilities: Vec::new(),
        }
    }
}

/// A creature type definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatureDef {
    /// Namespaced key.
    pub key: ContentKey,
    /// Attack skill.
    pub attack: u16,
    /// Defense skill.
    pub defense: u16,
    /// Minimum damage per creature.
    pub damage_min: u16,
    /// Maximum damage per creature.
    pub damage_max: u16,
    /// Hit points per creature.
    pub hit_points: u32,
    /// Speed (hexes per turn; also initiative).
    pub speed: u8,
    /// Non-battle attributes.
    pub info: CreatureInfoDef,
}

/// A partial change to a creature defined by a pack in the overriding pack's dependency closure
/// (e.g. SoD changing an AB creature).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatureOverride {
    /// Key of the creature to change (`pack:name`).
    pub target: ContentKey,
    /// The fields to change.
    pub patch: CreaturePatch,
}

/// The changed fields of a [`CreatureOverride`]. `None` fields stay as they are; list fields
/// replace the whole list.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CreaturePatch {
    /// New attack.
    pub attack: Option<u16>,
    /// New defense.
    pub defense: Option<u16>,
    /// New minimum damage.
    pub damage_min: Option<u16>,
    /// New maximum damage.
    pub damage_max: Option<u16>,
    /// New hit points.
    pub hit_points: Option<u32>,
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
    /// New shots.
    pub shots: Option<u16>,
    /// New casts.
    pub casts: Option<u8>,
    /// New adventure-map count range.
    pub adventure_count: Option<[u16; 2]>,
    /// New flags (replaces the whole list).
    pub flags: Option<Vec<String>>,
    /// New abilities (replaces the whole list).
    pub abilities: Option<Vec<AbilityDef>>,
}

/// Battle parameters (Tier 2). Exactly one pack in a ruleset provides them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BattleParamsDef {
    /// Battlefield columns, including side columns.
    pub field_width: u8,
    /// Battlefield rows.
    pub field_height: u8,
    /// Non-standable columns on each edge (hero/war-machine columns in the original).
    pub side_columns: u8,
    /// Army slots per side.
    pub army_slots: u8,
    /// Damage rolls sampled for large stacks (BAT-DMG-001).
    pub damage_roll_samples: u8,
    /// Damage bonus per point of attack over defense, in percent (BAT-DMG-002).
    pub attack_bonus_per_point_percent: u16,
    /// Cap of that bonus, in percent.
    pub attack_bonus_cap_percent: u16,
    /// Damage reduction per point of defense over attack, in per mille (BAT-DMG-002).
    pub defense_reduction_per_point_permille: u16,
    /// Cap of that reduction, in per mille.
    pub defense_reduction_cap_permille: u16,
    /// Defense bonus of a defending stack, in percent (BAT-TRN-004).
    pub defend_bonus_percent: u16,
    /// Minimum defense bonus of a defending stack (BAT-TRN-004).
    pub defend_bonus_min: u16,
    /// Shots reach at full damage up to this hex distance (BAT-RNG-002).
    pub ranged_full_damage_distance: u8,
    /// Damage reduction of shots beyond that distance, percent (BAT-RNG-002).
    pub ranged_penalty_percent: u8,
    /// Damage reduction of shooters' melee strikes, percent (BAT-RNG-003).
    pub melee_penalty_percent: u8,
    /// Sides of the morale die (BAT-MOR-003/004).
    pub morale_dice: u8,
    /// Good-morale chance per positive morale level, out of `morale_dice` (BAT-MOR-003).
    pub good_morale_chances: Vec<u8>,
    /// Bad-morale chance per negative morale level, out of `morale_dice` (BAT-MOR-004).
    pub bad_morale_chances: Vec<u8>,
    /// Sides of the luck die (BAT-LCK-002).
    pub luck_dice: u8,
    /// Lucky-strike chance per positive luck level, out of `luck_dice` (BAT-LCK-002).
    pub good_luck_chances: Vec<u8>,
    /// Starting rows: `formation[n - 1][i]` is the row of the i-th stack (by slot order) of an
    /// army with `n` stacks (BAT-SET-001).
    pub formation: Vec<Vec<u8>>,
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
    /// Creature types.
    pub creatures: Vec<CreatureDef>,
    /// Changes to creatures of packs this pack depends on.
    pub creature_overrides: Vec<CreatureOverride>,
    /// Profiles.
    pub profiles: Vec<ProfileDef>,
    /// Battle parameters, if this pack provides them.
    pub battle: Option<BattleParamsDef>,
}

impl ContentPack {
    /// An empty pack with the given manifest.
    pub fn new(manifest: PackManifest) -> Self {
        Self {
            manifest,
            resources: Vec::new(),
            factions: Vec::new(),
            creatures: Vec::new(),
            creature_overrides: Vec::new(),
            profiles: Vec::new(),
            battle: None,
        }
    }
}
