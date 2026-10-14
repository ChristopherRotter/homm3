//! Sieges (E04c T5, ADR-0023): the siege's field, part of the battle parameters (the wall, gate
//! and moat hexes, harvested from the executable's tables, `docs/formats/installation.md` §7.27,
//! §7.28), and one record per town faction (`[[siege]]`: the moat's damage, the Fortress's second
//! moat row, the Tower's land mines instead of a moat). A battle at a town names its faction
//! through `BattleSetup::siege`; the battle reads the record of that faction (BAT-SIE-001).

use alloc::string::String;
use alloc::vec::Vec;

use h3_content::{CompileError, ContentKey, FactionId, SpellId};
use h3_kernel::{CanonicalHash, StateHasher};

use crate::error::BattleCompileError;
use crate::params::FieldParams;

/// The siege's field as authored (`siege` in `[battle]`, pack format v37).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SiegeFieldDef {
    /// Each row's wall hex, row by row (0x63BD00): the hexes left of it are outside the walls.
    pub walls: Vec<u8>,
    /// The gate's hex, one of the wall hexes (the original's 96).
    pub gate: u8,
    /// Each row's moat hex, row by row (0x63BCE8); the one on the gate's row is the drawbridge's.
    pub moat: Vec<u8>,
    /// Each row's hex of the second moat row a faction may have (the Fortress's, 0x63BCF4).
    pub second_moat: Vec<u8>,
    /// Defense a stack loses in the moat (0x442646).
    pub moat_defense_penalty: u8,
    /// The power of the land mines of a town without a defending hero in a game before SoD
    /// (0x466460).
    pub mine_power: u8,
}

/// A compiled siege field (see [`SiegeFieldDef`]): one hex of each kind per row, the moat left of
/// the wall and the second moat row left of the moat.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SiegeField {
    /// Each row's wall hex.
    pub walls: Vec<u8>,
    /// The gate's hex.
    pub gate: u8,
    /// Each row's moat hex.
    pub moat: Vec<u8>,
    /// Each row's hex of the second moat row.
    pub second_moat: Vec<u8>,
    /// Defense a stack loses in the moat.
    pub moat_defense_penalty: u8,
    /// The mines' power without a hero before SoD.
    pub mine_power: u8,
}

impl SiegeField {
    /// The gate's row.
    fn gate_row(&self) -> usize {
        self.walls.iter().position(|&w| w == self.gate).unwrap_or(0)
    }

    /// The drawbridge's hex: the moat hex on the gate's row (the original's 95).
    pub fn bridge(&self) -> u8 {
        self.moat.get(self.gate_row()).copied().unwrap_or(0)
    }

    /// The second moat row's hex on the gate's row (the original's 94): a stack or corpse there
    /// keeps the defender from opening the closed gate, in every town (BAT-SIE-003).
    pub fn outer_bridge(&self) -> u8 {
        self.second_moat.get(self.gate_row()).copied().unwrap_or(0)
    }
}

/// Checks a siege field against the battlefield: one hex of each kind on each row, every one a
/// standable hex, the second moat row left of the moat and the moat left of the wall on each
/// row, and the gate one of the wall hexes.
pub(crate) fn compile_field(
    def: &SiegeFieldDef,
    field: FieldParams,
) -> Result<SiegeField, BattleCompileError> {
    let invalid = |reason| Err(BattleCompileError::InvalidBattleParams(reason));
    let rows = usize::from(field.height);
    if [&def.walls, &def.moat, &def.second_moat]
        .iter()
        .any(|list| list.len() != rows)
    {
        return invalid("the siege field needs one wall, moat and second moat hex per row");
    }
    let column = |hex: u8, row: usize| {
        let (x, y) = (hex % field.width, usize::from(hex / field.width));
        (y == row && x >= field.side_columns && x < field.width - field.side_columns).then_some(x)
    };
    for (row, ((&w, &m), &s)) in def
        .walls
        .iter()
        .zip(&def.moat)
        .zip(&def.second_moat)
        .enumerate()
    {
        match (column(s, row), column(m, row), column(w, row)) {
            (Some(s), Some(m), Some(w)) if s < m && m < w => {}
            _ => {
                return invalid(
                    "each siege row needs standable hexes: the second moat, the moat, the wall, \
                     left to right",
                );
            }
        }
    }
    if !def.walls.contains(&def.gate) {
        return invalid("the gate must be one of the wall hexes");
    }
    Ok(SiegeField {
        walls: def.walls.clone(),
        gate: def.gate,
        moat: def.moat.clone(),
        second_moat: def.second_moat.clone(),
        moat_defense_penalty: def.moat_defense_penalty,
        mine_power: def.mine_power,
    })
}

impl CanonicalHash for SiegeField {
    /// Walls, moat and second moat as length + bytes, the gate, the defense penalty, the mines'
    /// power (battle part v3).
    fn hash_into(&self, h: &mut StateHasher) {
        for list in [&self.walls, &self.moat, &self.second_moat] {
            h.write_len(list.len());
            h.write_bytes(list);
        }
        h.write_u8(self.gate);
        h.write_u8(self.moat_defense_penalty);
        h.write_u8(self.mine_power);
    }
}

/// A town faction's siege as authored (`[[siege]]` in `battle.toml`, pack format v37).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SiegeDef {
    /// The town's faction.
    pub faction: ContentKey,
    /// Damage to a stack entering the moat in a game under SoD rules (0x63BD18); with `mines`,
    /// the mines' least damage there.
    pub moat_damage: u32,
    /// The moat has the second row (the Fortress).
    pub second_moat: bool,
    /// Land mines of this spell on the moat's hexes instead of a moat (the Tower).
    pub mines: Option<ContentKey>,
}

/// A compiled siege record (see [`SiegeDef`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Siege {
    /// The town's faction.
    pub faction: FactionId,
    /// The moat's damage under SoD rules, or the mines' least damage.
    pub moat_damage: u32,
    /// The moat's second row.
    pub second_moat: bool,
    /// The spell of the land mines that take the moat's place.
    pub mines: Option<SpellId>,
}

/// Resolves a siege record: its faction and its mines' spell, a damage of at most
/// [`MAX_MOAT_DAMAGE`], no second row beside mines.
pub(crate) fn compile(
    def: &SiegeDef,
    faction: FactionId,
    mines: Option<SpellId>,
) -> Result<Siege, CompileError> {
    let invalid = |reason| CompileError::InvalidContent {
        kind: "siege",
        key: String::from(def.faction.as_str()),
        reason,
    };
    if def.moat_damage > MAX_MOAT_DAMAGE {
        return Err(invalid("the moat's damage is at most 10 000"));
    }
    if mines.is_some() && def.second_moat {
        return Err(invalid(
            "land mines take the moat's place: no second moat row",
        ));
    }
    Ok(Siege {
        faction,
        moat_damage: def.moat_damage,
        second_moat: def.second_moat,
        mines,
    })
}

/// The largest moat damage a record may give.
pub const MAX_MOAT_DAMAGE: u32 = 10_000;

impl CanonicalHash for Siege {
    /// Faction, damage, second row, the mines' spell (battle part v3).
    fn hash_into(&self, h: &mut StateHasher) {
        self.faction.hash_into(h);
        h.write_u32(self.moat_damage);
        h.write_bool(self.second_moat);
        self.mines.hash_into(h);
    }
}
