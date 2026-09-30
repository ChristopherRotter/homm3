//! Content packs as authored (Tier 1 data and Tier 2 parameters, ADR-0006), before compilation.
//!
//! These types are the output of an adapter (e.g. `h3-pack` parsing TOML) and the input of
//! [`crate::Ruleset::compile`]. They are plain data: all validation happens in the compiler, so
//! every adapter gets the same checks.

use alloc::string::String;
use alloc::vec::Vec;

use crate::key::ContentKey;

/// Identity and dependencies of a pack.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackManifest {
    /// Pack id, the namespace of its keys (`[a-z0-9_-]+`).
    pub id: String,
    /// Free-form version string, recorded in the ruleset identity.
    pub version: String,
    /// Ids of packs that must be loaded before this one.
    pub depends: Vec<String>,
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
    /// Creature types.
    pub creatures: Vec<CreatureDef>,
    /// Profiles.
    pub profiles: Vec<ProfileDef>,
    /// Battle parameters, if this pack provides them.
    pub battle: Option<BattleParamsDef>,
}
