//! The compiled, immutable ruleset (ADR-0006): content packs are validated once and compiled into
//! dense tables addressed by handles; the hot path never parses or looks up strings.

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

use h3_kernel::{CanonicalHash, Handle, StateHasher};

use crate::error::CompileError;
use crate::key::{ContentKey, is_valid_part};
use crate::pack::{BattleParamsDef, ContentPack, CreatureDef};
use crate::quirks::{QuirkRegistry, Quirks, REGISTRY};

/// Engine limit for army slots per side (the original uses 7, a ruleset parameter). The battle
/// context sizes its fixed-capacity state from this.
pub const MAX_ARMY_SLOTS: usize = 16;

/// Engine limit for battlefield size: hexes are addressed by `u8`.
pub const MAX_FIELD_HEXES: usize = 255;

/// Version of the ruleset hash encoding. Bump when the encoding below changes.
pub const RULESET_HASH_VERSION: u32 = 1;

/// A compiled creature type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CreatureType {
    /// Attack skill.
    pub attack: u16,
    /// Defense skill.
    pub defense: u16,
    /// Minimum damage per creature (≥ 1).
    pub damage_min: u16,
    /// Maximum damage per creature (≥ `damage_min`).
    pub damage_max: u16,
    /// Hit points per creature (≥ 1).
    pub hit_points: u32,
    /// Speed (≥ 1).
    pub speed: u8,
}

/// Handle of a creature type in a [`Ruleset`].
pub type CreatureTypeId = Handle<CreatureType>;

/// Battlefield dimensions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FieldParams {
    /// Columns including side columns.
    pub width: u8,
    /// Rows.
    pub height: u8,
    /// Non-standable columns on each edge.
    pub side_columns: u8,
}

/// Compiled battle parameters (Tier 2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BattleParams {
    /// Battlefield dimensions.
    pub field: FieldParams,
    /// Army slots per side (≤ [`MAX_ARMY_SLOTS`]).
    pub army_slots: u8,
    /// Damage rolls sampled for large stacks.
    pub damage_roll_samples: u8,
    /// Damage bonus per point of attack over defense, percent.
    pub attack_bonus_per_point_percent: u16,
    /// Cap of that bonus, percent.
    pub attack_bonus_cap_percent: u16,
    /// Damage reduction per point of defense over attack, per mille.
    pub defense_reduction_per_point_permille: u16,
    /// Cap of that reduction, per mille (≤ 1000).
    pub defense_reduction_cap_permille: u16,
    /// Defense bonus when defending, percent.
    pub defend_bonus_percent: u16,
    /// Minimum defense bonus when defending.
    pub defend_bonus_min: u16,
    formation: [[u8; MAX_ARMY_SLOTS]; MAX_ARMY_SLOTS],
}

impl BattleParams {
    /// Starting row of the `index`-th stack (in slot order) of an army with `stacks` stacks.
    pub fn formation_row(&self, stacks: usize, index: usize) -> Option<u8> {
        if index >= stacks || stacks > usize::from(self.army_slots) {
            return None;
        }
        // `1 <= stacks <= army_slots <= MAX_ARMY_SLOTS` here, so the lookups cannot fail.
        self.formation
            .get(stacks - 1)
            .and_then(|rows| rows.get(index))
            .copied()
    }
}

/// The immutable compiled ruleset.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ruleset {
    packs: Vec<(String, String)>,
    profile: ContentKey,
    quirks: Quirks,
    creatures: Vec<CreatureType>,
    creature_keys: Vec<ContentKey>,
    creature_index: BTreeMap<String, CreatureTypeId>,
    battle: BattleParams,
    hash: u64,
}

impl Ruleset {
    /// Compiles `packs` (in load order) with the profile `profile` (a `pack:name` key), using the
    /// engine's quirk registry.
    pub fn compile(packs: &[ContentPack], profile: &str) -> Result<Self, CompileError> {
        Self::compile_with(packs, profile, &REGISTRY)
    }

    /// Like [`Ruleset::compile`] with an explicit quirk registry (used by tests).
    pub fn compile_with(
        packs: &[ContentPack],
        profile: &str,
        registry: &QuirkRegistry,
    ) -> Result<Self, CompileError> {
        let mut pack_ids: Vec<(String, String)> = Vec::new();
        for pack in packs {
            let id = &pack.manifest.id;
            if !is_valid_part(id) {
                return Err(CompileError::InvalidPackId(id.clone()));
            }
            if pack_ids.iter().any(|(p, _)| p == id) {
                return Err(CompileError::DuplicatePack(id.clone()));
            }
            for dep in &pack.manifest.depends {
                if !pack_ids.iter().any(|(p, _)| p == dep) {
                    return Err(CompileError::MissingDependency {
                        pack: id.clone(),
                        depends: dep.clone(),
                    });
                }
            }
            pack_ids.push((id.clone(), pack.manifest.version.clone()));
        }

        let mut creatures = Vec::new();
        let mut creature_keys = Vec::new();
        let mut creature_index = BTreeMap::new();
        let mut battle_def: Option<&BattleParamsDef> = None;
        for pack in packs {
            let id = &pack.manifest.id;
            for def in &pack.creatures {
                check_namespace(&def.key, id)?;
                let handle = CreatureTypeId::from_index(creatures.len())
                    .ok_or(CompileError::TooManyCreatures)?;
                if creature_index
                    .insert(String::from(def.key.as_str()), handle)
                    .is_some()
                {
                    return Err(CompileError::DuplicateKey(String::from(def.key.as_str())));
                }
                creatures.push(compile_creature(def)?);
                creature_keys.push(def.key.clone());
            }
            if let Some(b) = &pack.battle {
                if battle_def.is_some() {
                    return Err(CompileError::DuplicateBattleParams(id.clone()));
                }
                battle_def = Some(b);
            }
        }
        let battle = compile_battle(battle_def.ok_or(CompileError::MissingBattleParams)?)?;

        let mut profiles = BTreeMap::new();
        for pack in packs {
            for p in &pack.profiles {
                check_namespace(&p.key, &pack.manifest.id)?;
                if profiles.insert(p.key.as_str(), p).is_some() {
                    return Err(CompileError::DuplicateKey(String::from(p.key.as_str())));
                }
            }
        }
        let selected = profiles
            .get(profile)
            .ok_or_else(|| CompileError::UnknownProfile(String::from(profile)))?;
        let mut quirks = Quirks::default();
        for (flag, on) in &selected.quirks {
            let f = registry
                .lookup(flag)
                .ok_or_else(|| CompileError::UnknownQuirk {
                    profile: String::from(profile),
                    flag: flag.clone(),
                })?;
            quirks = quirks.with(f, *on);
        }

        let mut ruleset = Self {
            packs: pack_ids,
            profile: selected.key.clone(),
            quirks,
            creatures,
            creature_keys,
            creature_index,
            battle,
            hash: 0,
        };
        ruleset.hash = h3_kernel::hash::hash_of(&ruleset);
        Ok(ruleset)
    }

    /// The creature type behind a handle, if it belongs to this ruleset.
    pub fn creature(&self, id: CreatureTypeId) -> Option<&CreatureType> {
        self.creatures.get(id.index())
    }

    /// The key of a creature type.
    pub fn creature_key(&self, id: CreatureTypeId) -> Option<&ContentKey> {
        self.creature_keys.get(id.index())
    }

    /// Looks a creature type up by its `pack:name` key (not for the hot path).
    pub fn creature_by_key(&self, key: &str) -> Option<CreatureTypeId> {
        self.creature_index.get(key).copied()
    }

    /// Number of creature types.
    pub fn creature_count(&self) -> usize {
        self.creatures.len()
    }

    /// Battle parameters.
    pub fn battle(&self) -> &BattleParams {
        &self.battle
    }

    /// Enabled quirk flags.
    pub fn quirks(&self) -> Quirks {
        self.quirks
    }

    /// The selected profile.
    pub fn profile(&self) -> &ContentKey {
        &self.profile
    }

    /// Loaded packs as `(id, version)`, in load order.
    pub fn packs(&self) -> &[(String, String)] {
        &self.packs
    }

    /// Canonical hash of the compiled content (recorded in replays; a replay only verifies against
    /// the ruleset it was recorded with).
    pub fn content_hash(&self) -> u64 {
        self.hash
    }
}

fn check_namespace(key: &ContentKey, pack: &str) -> Result<(), CompileError> {
    if key.pack() == pack {
        Ok(())
    } else {
        Err(CompileError::KeyOutsidePack {
            key: String::from(key.as_str()),
            pack: String::from(pack),
        })
    }
}

fn compile_creature(def: &CreatureDef) -> Result<CreatureType, CompileError> {
    let invalid = |reason| CompileError::InvalidCreature {
        key: String::from(def.key.as_str()),
        reason,
    };
    if def.damage_min == 0 {
        return Err(invalid("damage_min must be at least 1"));
    }
    if def.damage_max < def.damage_min {
        return Err(invalid("damage_max must not be below damage_min"));
    }
    if def.hit_points == 0 {
        return Err(invalid("hit_points must be at least 1"));
    }
    if def.speed == 0 {
        return Err(invalid("speed must be at least 1"));
    }
    Ok(CreatureType {
        attack: def.attack,
        defense: def.defense,
        damage_min: def.damage_min,
        damage_max: def.damage_max,
        hit_points: def.hit_points,
        speed: def.speed,
    })
}

fn compile_battle(def: &BattleParamsDef) -> Result<BattleParams, CompileError> {
    let invalid = |reason| Err(CompileError::InvalidBattleParams(reason));
    let w = usize::from(def.field_width);
    let h = usize::from(def.field_height);
    let side = usize::from(def.side_columns);
    if h == 0 || w < 2 * side + 2 {
        return invalid("the field needs at least one row and two standable columns");
    }
    if w * h > MAX_FIELD_HEXES {
        return invalid("the field has more than 255 hexes");
    }
    let slots = usize::from(def.army_slots);
    if slots == 0 || slots > MAX_ARMY_SLOTS || slots > h {
        return invalid("army_slots must be between 1 and min(16, field_height)");
    }
    if def.damage_roll_samples == 0 {
        return invalid("damage_roll_samples must be at least 1");
    }
    if def.defense_reduction_cap_permille > 1000 {
        return invalid("defense_reduction_cap_permille must not exceed 1000");
    }
    if def.formation.len() != slots {
        return invalid("formation needs one entry per army size 1..=army_slots");
    }
    let mut formation = [[0u8; MAX_ARMY_SLOTS]; MAX_ARMY_SLOTS];
    for (n, (rows, out)) in def.formation.iter().zip(formation.iter_mut()).enumerate() {
        if rows.len() != n + 1 {
            return invalid("formation entry n must list n rows");
        }
        for (i, (&row, slot)) in rows.iter().zip(out.iter_mut()).enumerate() {
            if usize::from(row) >= h {
                return invalid("formation row outside the field");
            }
            if rows.get(..i).is_some_and(|earlier| earlier.contains(&row)) {
                return invalid("formation rows must be distinct");
            }
            *slot = row;
        }
    }
    Ok(BattleParams {
        field: FieldParams {
            width: def.field_width,
            height: def.field_height,
            side_columns: def.side_columns,
        },
        army_slots: def.army_slots,
        damage_roll_samples: def.damage_roll_samples,
        attack_bonus_per_point_percent: def.attack_bonus_per_point_percent,
        attack_bonus_cap_percent: def.attack_bonus_cap_percent,
        defense_reduction_per_point_permille: def.defense_reduction_per_point_permille,
        defense_reduction_cap_permille: def.defense_reduction_cap_permille,
        defend_bonus_percent: def.defend_bonus_percent,
        defend_bonus_min: def.defend_bonus_min,
        formation,
    })
}

impl CanonicalHash for Ruleset {
    /// Encoding v1: version, packs (id, version), profile key, quirk bits, creatures (key + stats in
    /// field order), battle parameters in field order with the full formation table.
    fn hash_into(&self, h: &mut StateHasher) {
        h.write_u32(RULESET_HASH_VERSION);
        h.write_len(self.packs.len());
        for (id, version) in &self.packs {
            id.as_str().hash_into(h);
            version.as_str().hash_into(h);
        }
        self.profile.as_str().hash_into(h);
        h.write_u64(self.quirks.bits());
        h.write_len(self.creatures.len());
        for (c, key) in self.creatures.iter().zip(&self.creature_keys) {
            key.as_str().hash_into(h);
            h.write_u16(c.attack);
            h.write_u16(c.defense);
            h.write_u16(c.damage_min);
            h.write_u16(c.damage_max);
            h.write_u32(c.hit_points);
            h.write_u8(c.speed);
        }
        let b = &self.battle;
        h.write_u8(b.field.width);
        h.write_u8(b.field.height);
        h.write_u8(b.field.side_columns);
        h.write_u8(b.army_slots);
        h.write_u8(b.damage_roll_samples);
        h.write_u16(b.attack_bonus_per_point_percent);
        h.write_u16(b.attack_bonus_cap_percent);
        h.write_u16(b.defense_reduction_per_point_permille);
        h.write_u16(b.defense_reduction_cap_permille);
        h.write_u16(b.defend_bonus_percent);
        h.write_u16(b.defend_bonus_min);
        for row in &b.formation {
            h.write_bytes(row);
        }
    }
}

#[cfg(test)]
mod tests;
