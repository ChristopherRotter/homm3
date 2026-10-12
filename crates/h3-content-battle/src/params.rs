//! The battle parameters (Tier 2): the battlefield, damage, morale and luck, war machines, the
//! formations' rows, tactics zones, surrender, casts per round and the side stack limit, with the
//! battle's engine bounds.

use alloc::collections::BTreeSet;
use alloc::vec::Vec;

use h3_content::{Formation, MAX_ARMY_SLOTS, MAX_SKILL_LEVEL, ResourceId};
use h3_kernel::{CanonicalHash, StateHasher};

use crate::error::BattleCompileError;
use crate::pack::BattleParamsDef;

/// Engine limit for the stacks of one side in a battle, those created in it included (the
/// original keeps 21 records per side, `docs/formats/installation.md` §7.7). The battle sizes its
/// fixed-capacity state from this; `BattleParams::side_stack_limit` stays below it.
pub const MAX_SIDE_STACKS: usize = 21;

/// Engine limit for battlefield size: hexes are addressed by `u8`.
pub const MAX_FIELD_HEXES: usize = 255;

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

/// Most distinct morale/luck aura values (kind, amount, affected side) in a ruleset; sizes the
/// battle's modifier pool (ADR-0012 §3).
pub const MAX_AURA_VALUES: usize = 8;

/// Most levels of a morale or luck roll table.
pub const MAX_ROLL_LEVELS: usize = 8;

/// Chances of a morale or luck roll by level: level `l` (1-based) triggers with
/// `chance(l)` out of the die's sides (Tier 2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RollTable {
    len: u8,
    chances: [u8; MAX_ROLL_LEVELS],
}

impl RollTable {
    /// The highest level (the number of entries).
    pub const fn levels(&self) -> u8 {
        self.len
    }

    /// The chance of level `level` (1-based, capped at the highest level), or 0 for level 0.
    pub fn chance(&self, level: u8) -> u8 {
        let i = usize::from(level.min(self.len));
        // `i <= len <= MAX_ROLL_LEVELS`; level 0 has no entry.
        i.checked_sub(1)
            .and_then(|i| self.chances.get(i))
            .copied()
            .unwrap_or(0)
    }
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
    /// Shots reach at full damage up to this hex distance (BAT-RNG-002).
    pub ranged_full_damage_distance: u8,
    /// Damage reduction of shots beyond that distance, percent (BAT-RNG-002).
    pub ranged_penalty_percent: u8,
    /// Damage reduction of shooters' melee strikes, percent (BAT-RNG-003).
    pub melee_penalty_percent: u8,
    /// Sides of the morale die (≥ 1).
    pub morale_dice: u8,
    /// Good-morale chances by morale level; morale is at most `levels()` (BAT-MOR-001/003).
    pub good_morale: RollTable,
    /// Bad-morale chances by negative morale level; morale is at least `-levels()`
    /// (BAT-MOR-001/004).
    pub bad_morale: RollTable,
    /// Sides of the luck die (≥ 1).
    pub luck_dice: u8,
    /// Lucky-strike chances by luck level; luck is at most `levels()` (BAT-LCK-001/002).
    pub good_luck: RollTable,
    /// Head hexes `(x, y)` of the attacker's war machines by kind (shooting machine, ammo
    /// supply, first aid); the defender's are mirrored (BAT-WAR-001).
    pub war_machine_positions: [(u8, u8); WAR_MACHINE_KINDS],
    /// Starting rows by formation ([`Formation::index`]), army size and stack index.
    formations: [[[u8; MAX_ARMY_SLOTS]; MAX_ARMY_SLOTS]; Formation::COUNT],
    /// Width in columns of the tactics zone by Tactics level difference minus one, each between
    /// 1 and the number of standable columns (BAT-TAC-001).
    pub tactics_zone_columns: [u8; MAX_SKILL_LEVEL as usize],
    /// How a surrender is priced (BAT-FLE-002); `None`: no side can surrender.
    pub surrender: Option<SurrenderParams>,
    /// How many spells a hero may cast per round (BAT-MAG-001).
    pub hero_casts_per_round: u8,
    /// A side may create a stack (Clone, Summon Elemental) only while it has fewer stacks, dead
    /// ones included (BAT-MAG-037; 20 in the original), at most [`MAX_SIDE_STACKS`].
    pub side_stack_limit: u8,
}

/// The price of a surrender (BAT-FLE-002): `cost_percent` of the summed cost in `resource` of the
/// surrendering side's living stacks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SurrenderParams {
    /// The resource the price is paid in.
    pub resource: ResourceId,
    /// The share of the stacks' cost, percent.
    pub cost_percent: u16,
}

/// Kinds of war machines a battle places (shooting machine, ammo supply, first aid; BAT-WAR-001).
pub const WAR_MACHINE_KINDS: usize = 3;

impl BattleParams {
    /// Starting row of the `index`-th stack (in slot order) of an army with `stacks` stacks in
    /// `formation`.
    pub fn formation_row(&self, formation: Formation, stacks: usize, index: usize) -> Option<u8> {
        if index >= stacks || stacks > usize::from(self.army_slots) {
            return None;
        }
        // `1 <= stacks <= army_slots <= MAX_ARMY_SLOTS` here, so the lookups cannot fail.
        self.formations[formation.index()]
            .get(stacks - 1)
            .and_then(|rows| rows.get(index))
            .copied()
    }
}

pub(crate) fn compile_battle(
    def: &BattleParamsDef,
    surrender: Option<SurrenderParams>,
) -> Result<BattleParams, BattleCompileError> {
    let invalid = |reason| Err(BattleCompileError::InvalidBattleParams(reason));
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
    if def.ranged_penalty_percent > 100 || def.melee_penalty_percent > 100 {
        return invalid("ranged and melee penalties must not exceed 100 percent");
    }
    let table = |dice: u8, chances: &[u8]| -> Result<RollTable, BattleCompileError> {
        if dice == 0 {
            return Err(BattleCompileError::InvalidBattleParams(
                "morale_dice and luck_dice must be at least 1",
            ));
        }
        if chances.len() > MAX_ROLL_LEVELS || chances.iter().any(|&c| c > dice) {
            return Err(BattleCompileError::InvalidBattleParams(
                "morale and luck chances: at most 8 levels, each at most the die's sides",
            ));
        }
        let mut t = RollTable::default();
        for (slot, &c) in t.chances.iter_mut().zip(chances) {
            *slot = c;
        }
        // `chances.len() <= MAX_ROLL_LEVELS`.
        t.len = u8::try_from(chances.len()).unwrap_or(0);
        Ok(t)
    };
    let good_morale = table(def.morale_dice, &def.good_morale_chances)?;
    let bad_morale = table(def.morale_dice, &def.bad_morale_chances)?;
    // A certain freeze would let no stack ever act again (BAT-MOR-004).
    if def.bad_morale_chances.iter().any(|&c| c >= def.morale_dice) {
        return invalid("bad morale chances must be below morale_dice");
    }
    let good_luck = table(def.luck_dice, &def.good_luck_chances)?;
    let mut war_machine_positions = [(0, 0); WAR_MACHINE_KINDS];
    if def.war_machine_positions.len() != WAR_MACHINE_KINDS
        || def
            .war_machine_positions
            .iter()
            .any(|&[x, y]| usize::from(x) >= w || usize::from(y) >= h)
    {
        return invalid("war_machine_positions needs 3 hexes [x, y] on the field");
    }
    // rule: BAT-WAR-001
    // Any machine may be two hexes wide, with its tail on the hex behind its head (x - 1 for
    // the attacker, x + 1 for the mirrored defender): every footprint must fit the field, and
    // no two footprints of either side may share a hex.
    let mut machine_hexes: BTreeSet<(usize, usize)> = BTreeSet::new();
    for &[x, y] in &def.war_machine_positions {
        let (x, y) = (usize::from(x), usize::from(y));
        let Some(tail) = x.checked_sub(1) else {
            return invalid("war machine positions need room for a two-hex machine behind them");
        };
        // `x < w`, so the mirrored head and its tail `w - x` stay on the field.
        let mirrored = w - 1 - x;
        for hex in [(x, y), (tail, y), (mirrored, y), (w - x, y)] {
            if !machine_hexes.insert(hex) {
                return invalid("war machine positions overlap (as two-hex footprints)");
            }
        }
    }
    for (slot, &[x, y]) in war_machine_positions
        .iter_mut()
        .zip(&def.war_machine_positions)
    {
        *slot = (x, y);
    }
    let mut formations = [[[0u8; MAX_ARMY_SLOTS]; MAX_ARMY_SLOTS]; Formation::COUNT];
    for (table, out) in [&def.loose_formation, &def.tight_formation]
        .into_iter()
        .zip(formations.iter_mut())
    {
        compile_formation(table, slots, h, out)?;
    }
    // rule: BAT-TAC-001
    let mut tactics_zone_columns = [0u8; MAX_SKILL_LEVEL as usize];
    if def.tactics_zone_columns.len() != tactics_zone_columns.len()
        || def
            .tactics_zone_columns
            .iter()
            .any(|&c| c == 0 || usize::from(c) > w - 2 * side)
    {
        return invalid(
            "tactics_zone_columns needs one width per skill level, each between 1 and the standable columns",
        );
    }
    tactics_zone_columns.copy_from_slice(&def.tactics_zone_columns);
    // rule: BAT-MAG-037
    if usize::from(def.side_stack_limit) > MAX_SIDE_STACKS {
        return invalid("side_stack_limit must be at most 21");
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
        ranged_full_damage_distance: def.ranged_full_damage_distance,
        ranged_penalty_percent: def.ranged_penalty_percent,
        melee_penalty_percent: def.melee_penalty_percent,
        morale_dice: def.morale_dice,
        good_morale,
        bad_morale,
        luck_dice: def.luck_dice,
        good_luck,
        war_machine_positions,
        formations,
        tactics_zone_columns,
        surrender,
        hero_casts_per_round: def.hero_casts_per_round,
        side_stack_limit: def.side_stack_limit,
    })
}

/// Validates a formation table (one entry per army size `1..=slots`, entry `n` with `n`
/// distinct rows of a field with `height` rows) into `out`.
fn compile_formation(
    table: &[Vec<u8>],
    slots: usize,
    height: usize,
    out: &mut [[u8; MAX_ARMY_SLOTS]; MAX_ARMY_SLOTS],
) -> Result<(), BattleCompileError> {
    let invalid = |m| Err(BattleCompileError::InvalidBattleParams(m));
    if table.len() != slots {
        return invalid("formations need one entry per army size 1..=army_slots");
    }
    for (n, (rows, out)) in table.iter().zip(out.iter_mut()).enumerate() {
        if rows.len() != n + 1 {
            return invalid("formation entry n must list n rows");
        }
        for (i, (&row, slot)) in rows.iter().zip(out.iter_mut()).enumerate() {
            if usize::from(row) >= height {
                return invalid("formation row outside the field");
            }
            if rows.get(..i).is_some_and(|earlier| earlier.contains(&row)) {
                return invalid("formation rows must be distinct");
            }
            *slot = row;
        }
    }
    Ok(())
}

impl CanonicalHash for BattleParams {
    /// The parameters in field order (see `BattleRuleset`'s encoding).
    fn hash_into(&self, h: &mut StateHasher) {
        let b = self;
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
        h.write_u8(b.ranged_full_damage_distance);
        h.write_u8(b.ranged_penalty_percent);
        h.write_u8(b.melee_penalty_percent);
        h.write_u8(b.morale_dice);
        for t in [&b.good_morale, &b.bad_morale] {
            h.write_u8(t.len);
            h.write_bytes(&t.chances);
        }
        h.write_u8(b.luck_dice);
        h.write_u8(b.good_luck.len);
        h.write_bytes(&b.good_luck.chances);
        for &(x, y) in &b.war_machine_positions {
            h.write_u8(x);
            h.write_u8(y);
        }
        for row in b.formations.iter().flatten() {
            h.write_bytes(row);
        }
        h.write_bytes(&b.tactics_zone_columns);
        h.write_u8(u8::from(b.surrender.is_some()));
        if let Some(s) = b.surrender {
            s.resource.hash_into(h);
            h.write_u16(s.cost_percent);
        }
        h.write_u8(b.hero_casts_per_round);
        h.write_u8(b.side_stack_limit);
    }
}
