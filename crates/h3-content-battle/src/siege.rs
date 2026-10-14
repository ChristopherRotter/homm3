//! Sieges (E04c T5, ADR-0023): the siege's field, part of the battle parameters (the wall, gate
//! and moat hexes, harvested from the executable's tables, `docs/formats/installation.md` §7.27,
//! §7.28), and one record per town faction (`[[siege]]`: the moat's damage, the Fortress's second
//! moat row, the Tower's land mines instead of a moat). A battle at a town names its faction
//! through `BattleSetup::siege`; the battle reads the record of that faction (BAT-SIE-001).
//!
//! Since E04c T6 (pack format v39) the field also holds the wall shots' data (§7.29): the
//! targets (their kinds and the wall hexes their destruction opens), the lists a missed and an
//! automatic shot choose from, the Catapult's place, a Castle's extra wall hit points and the
//! Ballistics table (`BALLIST.TXT`); a record holds each target's hit points (`WALLS.TXT`).

use alloc::string::String;
use alloc::vec::Vec;

use h3_content::{CompileError, ContentKey, FactionId, SpellId};
use h3_kernel::{CanonicalHash, StateHasher};

use crate::error::BattleCompileError;
use crate::params::FieldParams;

/// Most wall-shot targets a siege field may have (the original's eight).
pub const MAX_WALL_TARGETS: usize = 8;

/// Ballistics levels: none, Basic, Advanced, Expert (the records of the Ballistics table).
pub const BALLISTICS_LEVELS: usize = 4;

/// What a wall-shot target is: its hit chance's column in a Ballistics record (§7.29).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum WallKind {
    /// The keep (stands from a Citadel).
    Keep,
    /// An upper or lower tower (stands with a Castle).
    Tower,
    /// The gate.
    Gate,
    /// A wall.
    Wall,
}

impl WallKind {
    /// All kinds, in code order (the record's columns).
    pub const ALL: [Self; 4] = [Self::Keep, Self::Tower, Self::Gate, Self::Wall];

    /// Pack name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Keep => "keep",
            Self::Tower => "tower",
            Self::Gate => "gate",
            Self::Wall => "wall",
        }
    }

    /// Looks a kind up by its pack name.
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|k| k.name() == name)
    }
}

/// A wall-shot target (`targets` of the siege field).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WallTarget {
    /// Its kind.
    pub kind: WallKind,
    /// The wall hex its destruction opens (a wall's), or the gate's hex (the gate's); none for the
    /// towers and the keep.
    pub wall: Option<u8>,
}

/// One level's record of the Ballistics table (`BALLIST.TXT`, §7.29).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BallisticsRecord {
    /// Chance to hit, in percent, by [`WallKind`] (keep, tower, gate, wall).
    pub hit: [u8; 4],
    /// Shots per wall-shot action.
    pub shots: u8,
    /// Chances of 0, 1 and 2 damage, in percent (they sum to 100).
    pub damage: [u8; 3],
}

/// Most shots a Ballistics record may give.
pub const MAX_WALL_SHOTS: u8 = 8;

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
    /// The attacker's Catapult's position (its rear hex, 0x46377C).
    pub catapult: u8,
    /// The wall-shot targets by index (0x63BE60, its kinds from 0x445EFF).
    pub targets: Vec<WallTarget>,
    /// The walls a missed shot may land on, in order (0x63B840).
    pub missed: Vec<u8>,
    /// The walls the automatic Catapult aims at, the weakest first (0x63D54C).
    pub automatic: Vec<u8>,
    /// The automatic Catapult's targets once none of `automatic` stands, in order (0x670198).
    pub fallback: Vec<u8>,
    /// Hit points a Castle adds to each wall (0x466348).
    pub castle_bonus: u8,
    /// The Ballistics table, by level 0 (none) to 3 (Expert).
    pub ballistics: Vec<BallisticsRecord>,
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
    /// The Catapult's position.
    pub catapult: u8,
    /// The wall-shot targets.
    pub targets: Vec<WallTarget>,
    /// The walls a missed shot may land on.
    pub missed: Vec<u8>,
    /// The walls the automatic Catapult aims at.
    pub automatic: Vec<u8>,
    /// The automatic Catapult's other targets.
    pub fallback: Vec<u8>,
    /// Hit points a Castle adds to each wall.
    pub castle_bonus: u8,
    /// The Ballistics table, levels 0 to 3.
    pub ballistics: [BallisticsRecord; BALLISTICS_LEVELS],
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
    let ballistics = compile_wall_shots(def, field)?;
    Ok(SiegeField {
        walls: def.walls.clone(),
        gate: def.gate,
        moat: def.moat.clone(),
        second_moat: def.second_moat.clone(),
        moat_defense_penalty: def.moat_defense_penalty,
        mine_power: def.mine_power,
        catapult: def.catapult,
        targets: def.targets.clone(),
        missed: def.missed.clone(),
        automatic: def.automatic.clone(),
        fallback: def.fallback.clone(),
        castle_bonus: def.castle_bonus,
        ballistics,
    })
}

/// Checks the wall shots' data (§7.29): one to [`MAX_WALL_TARGETS`] targets, one of them the gate
/// on the gate's hex, a wall on a wall hex other than the gate's, the towers and the keep on
/// none; lists of target indices without repeats; the Catapult's two hexes on the field (its rear
/// hex in a side column, the front one standable); one record per Ballistics level, chances at
/// most 100 %, the damage chances summing to 100 %, 1 to [`MAX_WALL_SHOTS`] shots.
fn compile_wall_shots(
    def: &SiegeFieldDef,
    field: FieldParams,
) -> Result<[BallisticsRecord; BALLISTICS_LEVELS], BattleCompileError> {
    let invalid = |reason| Err(BattleCompileError::InvalidBattleParams(reason));
    let n = def.targets.len();
    if n == 0 || n > MAX_WALL_TARGETS {
        return invalid("the siege field needs 1 to 8 wall-shot targets");
    }
    let gates = def.targets.iter().filter(|t| t.kind == WallKind::Gate);
    if gates.clone().count() != 1 || gates.clone().any(|t| t.wall != Some(def.gate)) {
        return invalid("one wall-shot target must be the gate, on the gate's hex");
    }
    let placed = def.targets.iter().all(|t| match t.kind {
        WallKind::Gate => true,
        WallKind::Wall => t
            .wall
            .is_some_and(|w| w != def.gate && def.walls.contains(&w)),
        WallKind::Keep | WallKind::Tower => t.wall.is_none(),
    });
    if !placed {
        return invalid("a wall target needs a wall hex other than the gate's, a turret none");
    }
    let indices = |list: &[u8]| {
        list.iter()
            .enumerate()
            .all(|(i, &t)| usize::from(t) < n && !list[..i].contains(&t))
    };
    if ![&def.missed, &def.automatic, &def.fallback]
        .iter()
        .all(|l| indices(l))
    {
        return invalid("the wall-shot lists name targets, each once");
    }
    let (x, y) = (def.catapult % field.width, def.catapult / field.width);
    if y >= field.height
        || x + 1 >= field.width
        || x >= field.side_columns
        || x + 1 < field.side_columns
    {
        return invalid("the Catapult stands in the side column, its front hex standable");
    }
    let records: [BallisticsRecord; BALLISTICS_LEVELS] = def.ballistics.as_slice().try_into().or(
        invalid("the Ballistics table needs one record per level, none to Expert"),
    )?;
    let sound = |r: &BallisticsRecord| {
        r.hit.iter().all(|&c| c <= 100)
            && r.damage.iter().map(|&c| u32::from(c)).sum::<u32>() == 100
            && (1..=MAX_WALL_SHOTS).contains(&r.shots)
    };
    if !records.iter().all(sound) {
        return invalid(
            "a Ballistics record needs chances of at most 100, damage chances summing to 100 and \
             1 to 8 shots",
        );
    }
    Ok(records)
}

impl CanonicalHash for SiegeField {
    /// Walls, moat and second moat as length + bytes, the gate, the defense penalty, the mines'
    /// power (battle part v3); the Catapult's hex, the targets as length + each's kind and wall
    /// hex (0xFF none), the three lists as length + bytes, the Castle's bonus, the Ballistics
    /// records' bytes (battle part v4).
    fn hash_into(&self, h: &mut StateHasher) {
        for list in [&self.walls, &self.moat, &self.second_moat] {
            h.write_len(list.len());
            h.write_bytes(list);
        }
        h.write_u8(self.gate);
        h.write_u8(self.moat_defense_penalty);
        h.write_u8(self.mine_power);
        h.write_u8(self.catapult);
        h.write_len(self.targets.len());
        for t in &self.targets {
            h.write_u8(t.kind as u8);
            h.write_u8(t.wall.unwrap_or(u8::MAX));
        }
        for list in [&self.missed, &self.automatic, &self.fallback] {
            h.write_len(list.len());
            h.write_bytes(list);
        }
        h.write_u8(self.castle_bonus);
        for r in &self.ballistics {
            h.write_bytes(&r.hit);
            h.write_u8(r.shots);
            h.write_bytes(&r.damage);
        }
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
    /// Each wall-shot target's hit points at a siege's start (`WALLS.TXT`, pack format v39).
    pub hit_points: Vec<u8>,
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
    /// Each wall-shot target's hit points at a siege's start (0 for none of the field's).
    pub hit_points: [u8; MAX_WALL_TARGETS],
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
    if def.hit_points.len() > MAX_WALL_TARGETS
        || def
            .hit_points
            .iter()
            .any(|&p| p == 0 || p > MAX_WALL_HIT_POINTS)
    {
        return Err(invalid(
            "the wall targets' hit points are 1 to 10, at most eight of them",
        ));
    }
    let mut hit_points = [0; MAX_WALL_TARGETS];
    hit_points[..def.hit_points.len()].copy_from_slice(&def.hit_points);
    Ok(Siege {
        faction,
        moat_damage: def.moat_damage,
        second_moat: def.second_moat,
        mines,
        hit_points,
    })
}

/// The most hit points a wall-shot target may start with.
pub const MAX_WALL_HIT_POINTS: u8 = 10;

/// The largest moat damage a record may give.
pub const MAX_MOAT_DAMAGE: u32 = 10_000;

impl CanonicalHash for Siege {
    /// Faction, damage, second row, the mines' spell (battle part v3); the targets' hit points
    /// (battle part v4).
    fn hash_into(&self, h: &mut StateHasher) {
        self.faction.hash_into(h);
        h.write_u32(self.moat_damage);
        h.write_bool(self.second_moat);
        self.mines.hash_into(h);
        h.write_bytes(&self.hit_points);
    }
}
