//! Content packs as authored (Tier 1 data and Tier 2 parameters, ADR-0006), before compilation.
//!
//! These types are the output of an adapter (e.g. `h3-pack` parsing TOML) and the input of
//! [`crate::Ruleset::compile`]. They are plain data: all validation happens in the compiler, so
//! every adapter gets the same checks.

use alloc::string::String;
use alloc::vec::Vec;

use crate::ability::AbilityDef;
use crate::artifacts::ArtifactClass;
use crate::effect::HeroEffectDef;
use crate::heroes::{SpellCategory, SpellPolarity};
use crate::key::ContentKey;
use crate::spell::SpellMagic;

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

/// A battlefield obstacle type (E03, BAT-GEO-004).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObstacleDef {
    /// Namespaced key.
    pub key: ContentKey,
    /// Terrains it can appear on (at least one).
    pub terrains: Vec<ContentKey>,
    /// Usual obstacles: free hexes needed to the right of its anchor (including it).
    pub width: u8,
    /// Usual obstacles: rows needed above its anchor.
    pub height: u8,
    /// Blocked hexes: for usual obstacles `(dx, dy)` from the anchor (its bottom-left hex;
    /// `dy <= 0` is upwards), for absolute ones `(x, y)` on the field.
    pub blocked: Vec<(i8, i8)>,
    /// Absolute obstacles have a fixed position and are placed at most once per battle.
    pub absolute: bool,
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
            shots: 0,
            casts: 0,
            adventure_count: [0, 0],
            flags: Vec::new(),
            abilities: Vec::new(),
            upgrades: Vec::new(),
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
}

/// A spell's data row (E04a T3): the numbers of the original spell table, and since pack format
/// v4 (E04b T2a) what it does in battle.
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
    /// Positive, neutral or negative for its target.
    pub polarity: SpellPolarity,
    /// Targeting and effects in battle (default: none, not castable yet).
    pub magic: SpellMagic,
    /// Spells whose effects a cast ends (pack format v5).
    pub counters: Vec<ContentKey>,
}

/// A secondary skill with the effects of its three levels (E04a T3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecondarySkillDef {
    /// Namespaced key.
    pub key: ContentKey,
    /// Display name.
    pub name: String,
    /// Effects at Basic, Advanced and Expert. Effects that matter only outside battle are not
    /// modelled yet (`packs/README.md`).
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
    /// Head hexes `[x, y]` of the attacker's war machines by kind (shooting machine, ammo
    /// supply, first aid); the defender's are mirrored (BAT-WAR-001).
    pub war_machine_positions: Vec<[u8; 2]>,
    /// Starting rows of the loose formation: `loose_formation[n - 1][i]` is the row of the i-th
    /// stack (by slot order) of an army with `n` stacks (BAT-SET-001).
    pub loose_formation: Vec<Vec<u8>>,
    /// Starting rows of the tight formation, laid out like `loose_formation` (BAT-SET-001).
    pub tight_formation: Vec<Vec<u8>>,
    /// Width in columns of the tactics zone by Tactics level difference:
    /// `tactics_zone_columns[d - 1]` for a side whose level exceeds the enemy's by `d`, one
    /// entry per skill level (BAT-TAC-001; 3, 5, 7 in the original).
    pub tactics_zone_columns: Vec<u8>,
    /// How a surrender is priced (BAT-FLE-002); `None`: no side can surrender.
    pub surrender: Option<SurrenderDef>,
    /// How many spells a hero may cast per round (BAT-MAG-001; 1 in the original).
    pub hero_casts_per_round: u8,
}

/// The price of a surrender (BAT-FLE-002): `cost_percent` of the summed cost in `resource` of
/// the surrendering side's living stacks (50 % of their gold cost in the original).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SurrenderDef {
    /// The resource the price is paid in.
    pub resource: ContentKey,
    /// The share of the stacks' cost, percent.
    pub cost_percent: u16,
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
    /// Obstacle types.
    pub obstacles: Vec<ObstacleDef>,
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
    /// Artifact slots (E04a T3b).
    pub artifact_slots: Vec<ArtifactSlotDef>,
    /// Changes to artifact slots of the closure.
    pub artifact_slot_overrides: Vec<ArtifactSlotOverride>,
    /// Artifact types (E04a T3b).
    pub artifacts: Vec<ArtifactDef>,
    /// Profiles.
    pub profiles: Vec<ProfileDef>,
    /// Battle parameters, if this pack provides them.
    pub battle: Option<BattleParamsDef>,
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
            obstacles: Vec::new(),
            creatures: Vec::new(),
            creature_overrides: Vec::new(),
            spell_schools: Vec::new(),
            spells: Vec::new(),
            skills: Vec::new(),
            hero_classes: Vec::new(),
            heroes: Vec::new(),
            artifact_slots: Vec::new(),
            artifact_slot_overrides: Vec::new(),
            artifacts: Vec::new(),
            profiles: Vec::new(),
            battle: None,
            hero_params: None,
        }
    }
}
