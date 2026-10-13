//! The battle's part of a content pack as authored (Tier 1 data and Tier 2 parameters, ADR-0006),
//! before compilation: obstacles, battle parameters, and the battle rows of the pack's creatures
//! and spells (R01). Each belongs to the core pack whose manifest it carries; the core's
//! [`ContentPack`](h3_content::ContentPack) holds the rest.
//!
//! These types are the output of an adapter (e.g. `h3-pack` parsing TOML) and the input of
//! [`crate::BattleRuleset::compile`]. They are plain data: all validation happens in the compiler.

use alloc::string::String;
use alloc::vec::Vec;

use h3_content::{ContentKey, PackManifest};

use crate::ability::AbilityDef;
use crate::spell::{SpellMagic, SpellPolarity};

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
    /// A side may create a stack (Clone, Summon Elemental) only while it has fewer stacks, dead
    /// ones included (BAT-MAG-037; 20 in the original).
    pub side_stack_limit: u8,
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

/// A creature type's battle row (the core's `CreatureDef` holds its identity, speed and flags).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BattleCreatureDef {
    /// Key of the creature, defined by this pack's core part.
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
    /// Ranged shots (0 = not a shooter by ammunition).
    pub shots: u16,
    /// Spell casts per battle.
    pub casts: u8,
    /// Abilities (ADR-0012), from the closed set [`crate::Ability`].
    pub abilities: Vec<AbilityDef>,
}

/// The battle half of a creature override: a partial change to the battle row of a creature of
/// the overriding pack's dependency closure. Its `fixes` gates it like the core half's (ADR-0007,
/// amendment E04b T8e).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BattleCreatureOverride {
    /// Key of the creature to change (`pack:name`).
    pub target: ContentKey,
    /// The data quirk this override fixes (a `quirk.content` flag), if it is gated: then it is
    /// applied only when the active profile turns the flag off.
    pub fixes: Option<String>,
    /// The fields to change.
    pub patch: BattleCreaturePatch,
}

/// The changed fields of a [`BattleCreatureOverride`]. `None` fields stay as they are; list
/// fields replace the whole list.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BattleCreaturePatch {
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
    /// New shots.
    pub shots: Option<u16>,
    /// New casts.
    pub casts: Option<u8>,
    /// New abilities (replaces the whole list).
    pub abilities: Option<Vec<AbilityDef>>,
}

/// A spell's battle row (since pack format v4, E04b T2a; the core's `SpellDef` holds its data
/// row). An adventure spell may have none: it then has no battle effects.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BattleSpellDef {
    /// Key of the spell, defined by this pack's core part.
    pub key: ContentKey,
    /// Positive, neutral or negative for its target.
    pub polarity: SpellPolarity,
    /// Targeting and effects in battle (default: none, not castable yet).
    pub magic: SpellMagic,
    /// Spells whose effects a cast ends (pack format v5).
    pub counters: Vec<ContentKey>,
    /// The only spells whose effects its dispel effect ends (pack format v7).
    pub dispels: Vec<ContentKey>,
    /// Spells whose effects its dispel effect spares, without `dispels` (pack format v7).
    pub spares: Vec<ContentKey>,
    /// The raise spell whose target test a sacrifice effect's target takes under quirk
    /// BAT-QUIRK-5 (pack format v17; Sacrifice: Resurrection).
    pub tests_as: Option<ContentKey>,
    /// The creature a summon effect summons (pack format v18; Summon Fire Elemental: the Fire
    /// Elemental).
    pub summons: Option<ContentKey>,
}

/// The battle's part of one content pack.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BattlePack {
    /// The core pack's manifest (identity and dependencies).
    pub manifest: PackManifest,
    /// Obstacle types.
    pub obstacles: Vec<ObstacleDef>,
    /// The battle rows of the core pack's creatures.
    pub creatures: Vec<BattleCreatureDef>,
    /// The battle halves of the pack's creature overrides.
    pub creature_overrides: Vec<BattleCreatureOverride>,
    /// The battle rows of the core pack's spells.
    pub spells: Vec<BattleSpellDef>,
    /// Battle parameters, if this pack provides them.
    pub battle: Option<BattleParamsDef>,
}

impl BattlePack {
    /// An empty battle pack for the core pack with this manifest.
    pub fn new(manifest: PackManifest) -> Self {
        Self {
            manifest,
            obstacles: Vec::new(),
            creatures: Vec::new(),
            creature_overrides: Vec::new(),
            spells: Vec::new(),
            battle: None,
        }
    }
}
