//! A battle at a town (E04c T5, ADR-0023): the walls, the gate and the moat (BAT-SIE-001 to
//! BAT-SIE-008). The town's facts come through `BattleSetup::siege`; its faction's siege record
//! and the siege's field come from the ruleset (`h3_content_battle::Siege`, `SiegeField`). The
//! original's routines: `docs/formats/installation.md` §7.27 and §7.28.
//!
//! The wall shots (E04c T6, `installation.md` §7.29; BAT-SIE-009 to BAT-SIE-015): the wall-shot
//! targets' hit points, the attacker's Catapult (set up by the battle, BAT-SIE-009) and the
//! Cyclops shooting them, a destroyed wall's hex and a destroyed gate, Earthquake. The turrets
//! and the town's other rules (the controls, the experience, the defenders' morale and luck,
//! Teleport's and the summon's limits) come with E04c T7 and T8 (BAT-STUB-2).

use h3_content::SpellId;
use h3_content_battle::{Ability, BattleRuleset, BattlefieldId, MAX_WALL_TARGETS, WallKind};
use h3_contracts::{BattleSetup, BattleSide, FortLevel, Moat};
use h3_kernel::arith::{DOUBLE_BITS, SINGLE_BITS, X87};
use h3_kernel::{CanonicalHash, StateHasher};

use crate::error::SetupError;
use crate::event::{Event, EventKind, Sink};
use crate::geometry::{Field, Hex, HexSet};
use crate::hero::HeroRecord;
use crate::state::{BattleState, NO_CORPSE, Stack, StackId};

/// What the battle keeps of the town it is fought at (public: the original shows the walls, the
/// gate and the moat; ADR-0023 point 5).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SiegeState {
    /// The town's faction.
    pub faction: h3_content::FactionId,
    /// The town's fort level.
    pub fort: FortLevel,
    /// The town's moat as the setup gives it.
    pub moat_kind: Moat,
    /// The gate is open (BAT-SIE-003); a town without a fort has no gate.
    pub gate_open: bool,
    /// The battlefield.
    field: Field,
    /// The wall hexes, the gate's included: the wall line (none without a fort; BAT-SIE-002).
    line: HexSet,
    /// The gate's hex.
    gate: Hex,
    /// The drawbridge's hex: the moat hex before the gate.
    bridge: Hex,
    /// The outer bridge's hex: the second moat row's hex on the gate's row.
    outer_bridge: Hex,
    /// The moat's hexes, the drawbridge's (and with a second row the outer bridge's) among them;
    /// none without a moat (BAT-SIE-004).
    moat: HexSet,
    /// The moat has the second row (the Fortress).
    second_row: bool,
    /// The damage of entering the moat; 0 for a harmless one (BAT-SIE-005).
    moat_damage: u32,
    /// The defense a struck stack loses in the moat (BAT-SIE-006).
    defense_penalty: u8,
    /// Each wall-shot target's hit points (BAT-SIE-010), the absent turrets' too (they soak
    /// Earthquake's hits, BAT-QUIRK-22); 0 once destroyed.
    pub parts: [u8; MAX_WALL_TARGETS],
    /// The number of wall-shot targets (none without a fort).
    pub targets: u8,
    /// Each target's kind.
    kinds: [WallKind; MAX_WALL_TARGETS],
    /// The hex each target's destruction opens (a wall's; the gate's own hex for the gate).
    opens: [Option<Hex>; MAX_WALL_TARGETS],
    /// The targets the fort level sets up, as bits (BAT-SIE-010: the towers with a Castle, the
    /// keep from a Citadel, the rest from a Fort).
    present: u8,
    /// The hexes destroyed targets opened: walls' hexes, the gate's once it is destroyed
    /// (BAT-SIE-012).
    broken: HexSet,
}

/// The land mines a town lays at the battle's start (BAT-SIE-007).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TownMines {
    /// Their spell.
    pub spell: SpellId,
    /// Their hexes.
    pub hexes: HexSet,
    /// Each one's value.
    pub value: u64,
    /// Rounds they last (0: the whole battle).
    pub rounds: u8,
}

impl SiegeState {
    /// The gate's and the drawbridge's hexes.
    fn gate_hexes(&self) -> HexSet {
        let mut set = HexSet::new();
        set.insert(self.gate);
        set.insert(self.bridge);
        set
    }

    /// The bridges: the drawbridge's hex and, with a second moat row, the outer bridge's; moat
    /// only while the gate is closed (the lowered drawbridge covers them).
    fn bridges(&self) -> HexSet {
        let mut set = HexSet::new();
        set.insert(self.bridge);
        if self.second_row {
            set.insert(self.outer_bridge);
        }
        set
    }

    /// Whether wall-shot target `t` stands: set up by the fort level and with hit points left
    /// (BAT-SIE-010; the original's 0x476440).
    pub fn stands(&self, t: u8) -> bool {
        t < self.targets && self.present & (1 << t) != 0 && self.parts[usize::from(t)] > 0
    }

    /// Whether target `t` is set up at all by the fort level.
    pub fn present(&self, t: u8) -> bool {
        t < self.targets && self.present & (1 << t) != 0
    }

    /// Target `t`'s kind.
    pub fn kind(&self, t: u8) -> Option<WallKind> {
        (t < self.targets).then(|| self.kinds[usize::from(t)])
    }

    /// Whether the gate is destroyed (BAT-SIE-012).
    pub fn gate_destroyed(&self) -> bool {
        self.broken.contains(self.gate)
    }

    /// The moat's hexes now (BAT-SIE-004): the bridges only while the gate is closed.
    pub fn moat(&self) -> HexSet {
        if self.gate_open {
            self.moat.without(self.bridges())
        } else {
            self.moat
        }
    }

    /// The wall hexes that stand (BAT-SIE-002): the wall line but the hexes destroyed walls
    /// opened and a destroyed gate's (BAT-SIE-012).
    pub fn walls(&self) -> HexSet {
        self.line.without(self.broken)
    }

    /// Whether `hex` is outside the walls on its row: left of the row's wall hex (BAT-SIE-008).
    fn outside(&self, hex: Hex) -> bool {
        let row = self.field.coords(hex).1;
        self.line
            .iter()
            .any(|w| self.field.coords(w).1 == row && hex.0 < w.0)
    }

    /// Whether `hex` is inside the walls on its row: the row's wall hex or right of it.
    fn inside(&self, hex: Hex) -> bool {
        let row = self.field.coords(hex).1;
        self.line
            .iter()
            .any(|w| self.field.coords(w).1 == row && hex.0 >= w.0)
    }
}

impl CanonicalHash for SiegeState {
    /// Faction, fort level, moat kind, the gate, the wall line, the gate's, drawbridge's and
    /// outer bridge's hexes, the moat, its second row, its damage, the defense penalty (state
    /// hash v46).
    fn hash_into(&self, h: &mut StateHasher) {
        self.faction.hash_into(h);
        h.write_u8(self.fort as u8);
        h.write_u8(self.moat_kind as u8);
        h.write_bool(self.gate_open);
        self.line.hash_into(h);
        for hex in [self.gate, self.bridge, self.outer_bridge] {
            hex.hash_into(h);
        }
        self.moat.hash_into(h);
        h.write_bool(self.second_row);
        h.write_u32(self.moat_damage);
        h.write_u8(self.defense_penalty);
        // State hash v47 (E04c T6): the wall shots' state.
        h.write_u8(self.targets);
        h.write_bytes(&self.parts);
        for t in 0..usize::from(self.targets) {
            h.write_u8(self.kinds[t] as u8);
            h.write_u8(self.opens[t].map_or(u8::MAX, |hex| hex.0));
        }
        h.write_u8(self.present);
        self.broken.hash_into(h);
    }
}

/// The siege of `setup`, if the battle is at a town (BAT-SIE-001), and the land mines its town
/// lays: its faction's record and the siege's field from the ruleset; the walls and the closed
/// gate from a Fort; the moat from a Citadel, by the setup's moat (the game's version), or a
/// record's mines instead (BAT-SIE-007). `heroes` are the sides' heroes, `battlefield` the
/// special battlefield (the mines' mastery). The natural obstacles (`obstacles`) must keep off
/// the walls, the moat and the mines.
///
/// # Errors
/// [`SetupError::UnknownSiege`] for a faction without a siege record,
/// [`SetupError::InvalidSiege`] for a moat below a Citadel or an obstacle on the walls, the moat
/// or a mine.
// rule: BAT-SIE-001
pub(crate) fn setup(
    ruleset: &BattleRuleset,
    field: Field,
    (setup, heroes, battlefield): (
        &BattleSetup,
        &[Option<HeroRecord>; 2],
        Option<BattlefieldId>,
    ),
    obstacles: HexSet,
) -> Result<Option<(SiegeState, Option<TownMines>)>, SetupError> {
    let Some(town) = setup.siege else {
        return Ok(None);
    };
    let (record, sf) = ruleset
        .siege(town.faction)
        .zip(ruleset.siege_field())
        .ok_or(SetupError::UnknownSiege)?;
    if town.moat != Moat::Absent && town.fort < FortLevel::Citadel {
        return Err(SetupError::InvalidSiege);
    }
    let set = |hexes: &mut dyn Iterator<Item = &u8>| {
        hexes.fold(HexSet::new(), |mut set, &h| {
            set.insert(Hex(h));
            set
        })
    };
    let walled = town.fort >= FortLevel::Fort;
    let watered = town.moat != Moat::Absent;
    let line = if walled {
        set(&mut sf.walls.iter())
    } else {
        HexSet::new()
    };
    let moat = match (watered && record.mines.is_none(), record.second_moat) {
        (false, _) => HexSet::new(),
        (true, false) => set(&mut sf.moat.iter()),
        (true, true) => set(&mut sf.moat.iter().chain(&sf.second_moat)),
    };
    // rule: BAT-SIE-007
    let bridge = sf.bridge();
    let mines = record
        .mines
        .filter(|_| watered)
        .and_then(|id| ruleset.spell(id).map(|spell| (id, spell)))
        .map(|(id, spell)| {
            let heros = heroes[BattleSide::Defender.index()].as_ref().map(|h| {
                let mastery = crate::magic::mastery_at(ruleset, battlefield, h, id);
                crate::magic::base_value(h, spell, mastery)
            });
            let least = u64::from(record.moat_damage);
            let value = if town.moat == Moat::Damaging {
                heros.map_or(least, |v| v.max(least))
            } else {
                heros.unwrap_or(
                    u64::from(sf.mine_power) * u64::from(spell.power) + u64::from(spell.effect[0]),
                )
            };
            TownMines {
                spell: id,
                hexes: set(&mut sf.moat.iter().filter(|&&h| h != bridge)),
                value,
                rounds: spell
                    .magic
                    .obstacle
                    .as_ref()
                    .map_or(0, |shape| shape.rounds),
            }
        });
    let kept = line
        .union(moat)
        .union(mines.map_or(HexSet::new(), |m| m.hexes));
    if obstacles.intersects(kept) {
        return Err(SetupError::InvalidSiege);
    }
    // rule: BAT-SIE-010
    let n = if walled { sf.targets.len() } else { 0 };
    let (mut parts, mut kinds, mut opens) = (
        [0; MAX_WALL_TARGETS],
        [WallKind::Wall; MAX_WALL_TARGETS],
        [None; MAX_WALL_TARGETS],
    );
    let mut present = 0u8;
    for (t, target) in sf.targets.iter().enumerate().take(n) {
        let castle = town.fort == FortLevel::Castle;
        let bonus = if castle && target.kind == WallKind::Wall {
            sf.castle_bonus
        } else {
            0
        };
        parts[t] = record.hit_points[t].saturating_add(bonus);
        kinds[t] = target.kind;
        opens[t] = target.wall.map(Hex);
        let set_up = match target.kind {
            WallKind::Tower => castle,
            WallKind::Keep => town.fort >= FortLevel::Citadel,
            WallKind::Gate | WallKind::Wall => true,
        };
        present |= u8::from(set_up) << t;
    }
    let siege = SiegeState {
        faction: town.faction,
        fort: town.fort,
        moat_kind: town.moat,
        gate_open: !walled,
        field,
        line,
        gate: Hex(sf.gate),
        bridge: Hex(bridge),
        outer_bridge: Hex(sf.outer_bridge()),
        moat,
        second_row: record.second_moat,
        moat_damage: if town.moat == Moat::Damaging {
            record.moat_damage
        } else {
            0
        },
        defense_penalty: sf.moat_defense_penalty,
        parts,
        // `n <= MAX_WALL_TARGETS`.
        targets: u8::try_from(n).unwrap_or(u8::MAX),
        kinds,
        opens,
        present,
        broken: HexSet::new(),
    };
    Ok(Some((siege, mines)))
}

/// The hexes the setup may not place stacks on (BAT-SIE-002): the walls and the closed gate's.
pub(crate) fn placement_blocked(siege: Option<&SiegeState>) -> HexSet {
    siege.map_or(HexSet::new(), |s| blocks(s, false))
}

/// The hexes the walls and the gate block (BAT-SIE-002, BAT-SIE-003): every standing wall hex
/// but the gate's (BAT-SIE-012: not those destroyed walls opened); while the gate is closed also
/// the gate's and the drawbridge's, unless `passes`.
fn blocks(s: &SiegeState, passes: bool) -> HexSet {
    if s.gate_open || passes {
        s.walls().without(s.gate_hexes())
    } else {
        let mut blocked = s.walls();
        blocked.insert(s.bridge);
        blocked
    }
}

/// Whether a living stack or a corpse lies on `hex` (the original's hex records: a stack, its
/// corpses).
fn lies(state: &BattleState, field: Field, hex: Hex) -> bool {
    state
        .stacks
        .iter()
        .zip(state.raised.corpse.iter())
        .any(|(s, &corpse)| corpse != NO_CORPSE && s.footprint(field).covers(hex))
}

/// The hexes the walls and the gate block for stack `mover` (BAT-SIE-002, BAT-SIE-003; the
/// original's 0x469A10): the wall hexes but the gate's always; while the gate is closed the
/// gate's and the drawbridge's too, but for a stack its controller commands for the defender
/// when no living stack and no corpse lies on the drawbridge's or the outer bridge's hex, in
/// every town (the mover on one of them included, as the original reads the hexes). Without a
/// mover (a stack being placed) the closed gate blocks.
// rule: BAT-SIE-002, BAT-SIE-003
pub(crate) fn blocked_for(state: &BattleState, mover: Option<&Stack>) -> HexSet {
    state.siege.map_or(HexSet::new(), |s| {
        let passes = !s.gate_open
            && mover.is_some_and(|m| m.controller() == BattleSide::Defender)
            && !lies(state, s.field, s.bridge)
            && !lies(state, s.field, s.outer_bridge);
        blocks(&s, passes)
    })
}

/// Where a walk of a stack its controller commands for `side` stops (BAT-SIE-004; the original's
/// 0x4B3290): every moat hex, the bridges only while the gate is closed and for the attacker.
// rule: BAT-SIE-004
pub(crate) fn stops(state: &BattleState, side: BattleSide) -> HexSet {
    state.siege.map_or(HexSet::new(), |s| {
        if side == BattleSide::Defender {
            s.moat.without(s.bridges())
        } else {
            s.moat()
        }
    })
}

/// Whether entering a moat hex damages a stack (BAT-SIE-005): a damaging moat lies on the
/// battlefield.
pub(crate) fn damaging(state: &BattleState) -> bool {
    state.siege.is_some_and(|s| s.moat_damage > 0)
}

/// Whether entering `hex` damages a stack: a hex of a damaging moat now (BAT-SIE-005).
pub(crate) fn damages_on(state: &BattleState, hex: Hex) -> bool {
    state
        .siege
        .is_some_and(|s| s.moat_damage > 0 && s.moat().contains(hex))
}

/// Stack `s` entered `hex` of a damaging moat (BAT-SIE-005; the original's 0x469E50): the moat's
/// damage straight to its hit points (no defense, no resistance), to a stack of either side,
/// then what damage brings (BAT-MAG-010's ends); the caller handles the rebirth and the battle's
/// end as after a Fire Wall's burn.
// rule: BAT-SIE-005
pub(crate) fn moat_damage<S: Sink>(
    state: &mut BattleState,
    ruleset: &BattleRuleset,
    (s, hex): (StackId, Hex),
    sink: &mut S,
) {
    let damage = state.siege.map_or(0, |siege| u64::from(siege.moat_damage));
    let kills = crate::damage::apply_damage(state.at_mut(s), damage);
    sink.emit(Event::public(EventKind::MoatDamage {
        stack: s,
        hex,
        damage,
        kills,
    }));
    crate::effects::damaged(state, ruleset, s, sink);
}

/// The defense a stack struck now loses in the moat (BAT-SIE-006; the original's 0x442590): the
/// penalty while either of its hexes is a moat hex, else 0; never for a frenzied stack, whose
/// defense is 0 (the caller's).
// rule: BAT-SIE-006
pub(crate) fn defense_penalty(state: &BattleState, victim: &Stack) -> i32 {
    state.siege.map_or(0, |s| {
        let moat = s.moat();
        if victim.footprint(s.field).hexes().any(|h| moat.contains(h)) {
            i32::from(s.defense_penalty)
        } else {
            0
        }
    })
}

/// Stack `mover` stands at its position now: if its controller commands it for the defender,
/// the gate is closed and it covers the gate's, the drawbridge's or the outer bridge's hex, the
/// gate opens (BAT-SIE-003; the original's 0x467130, 0x4671C0: a walk's step, a flight's
/// landing).
// rule: BAT-SIE-003
pub(crate) fn entered<S: Sink>(state: &mut BattleState, mover: StackId, sink: &mut S) {
    let stack = *state.at(mover);
    let Some(s) = state.siege.as_mut() else {
        return;
    };
    let on_gate = stack
        .footprint(s.field)
        .hexes()
        .any(|h| h == s.gate || h == s.bridge || h == s.outer_bridge);
    if !s.gate_open && stack.controller() == BattleSide::Defender && on_gate {
        s.gate_open = true;
        sink.emit(Event::public(EventKind::GateOpened));
    }
}

/// After a move or an action: an open gate closes unless a living stack or a corpse lies on the
/// gate's or the drawbridge's hex, or with a second moat row on the outer bridge's (BAT-SIE-003;
/// the original's 0x4672E0).
// rule: BAT-SIE-003
pub(crate) fn close_gate<S: Sink>(state: &mut BattleState, sink: &mut S) {
    let Some(s) = state
        .siege
        .filter(|s| s.gate_open && s.fort >= FortLevel::Fort && !s.gate_destroyed())
    else {
        return;
    };
    let held = lies(state, s.field, s.gate)
        || lies(state, s.field, s.bridge)
        || (s.second_row && lies(state, s.field, s.outer_bridge));
    if !held {
        state.siege = Some(SiegeState {
            gate_open: false,
            ..s
        });
        sink.emit(Event::public(EventKind::GateClosed));
    }
}

/// Whether a shot of `shooter` at `target` is halved by the walls (BAT-SIE-008; the original's
/// 0x467510): in a battle with walls, a shooter without `exempt` whose hex, turned to face the
/// target as the original turns it, is outside the walls on its row, at a target whose position
/// is inside on its row, when the straight line between them on the column/row grid meets a
/// wall ([`line_blocked`]).
// rule: BAT-SIE-008
pub(crate) fn wall_penalty(
    state: &BattleState,
    shooter: &Stack,
    target: &Stack,
    exempt: bool,
) -> bool {
    state.siege.is_some_and(|s| {
        let from = turned_front(s.field, shooter, target.hex);
        !exempt
            && !s.line.is_empty()
            && s.outside(from)
            && s.inside(target.hex)
            && line_blocked(&s, from, target.hex)
    })
}

/// The screen x of `hex`'s anchor in the original's units (44 per column, even rows 22 to the
/// right; the constant offset left out).
fn screen_x(field: Field, hex: Hex) -> i32 {
    let (x, y) = field.coords(hex);
    44 * x + if y % 2 == 0 { 22 } else { 0 }
}

/// The hex a shooter shoots from, as the original turns it (`installation.md` §7.25): it faces
/// right if the target's position lies right of its front hex on the screen, left if left of it,
/// else as it stands (the attacker's stacks right), and shoots from the hex of its footprint on
/// the side it faces (a one-hex shooter from its hex).
fn turned_front(field: Field, shooter: &Stack, target: Hex) -> Hex {
    let fp = shooter.footprint(field);
    let (front, tx) = (screen_x(field, fp.front), screen_x(field, target));
    // Right of the front hex, or level with it for a stack facing right.
    let attacker = u8::from(shooter.side == BattleSide::Attacker);
    let right = (tx, attacker) > (front, 0);
    let hexes = fp.hexes();
    if right { hexes.max() } else { hexes.min() }.unwrap_or(fp.front)
}

/// Whether the straight line from `from` to `to` meets a wall (BAT-SIE-008; the original's
/// 0x467840): on the column/row grid, from `from`'s column and row in 17 × the larger difference
/// steps of 1/17 of a cell along the longer axis and the matching fraction along the other, each
/// position a single-precision value and truncated to a hex at every step; the gate's hex
/// blocks while the gate is closed, any other wall hex while it stands (a destroyed wall's hex
/// no longer, BAT-SIE-012).
fn line_blocked(s: &SiegeState, from: Hex, to: Hex) -> bool {
    let (fx, fy) = s.field.coords(from);
    let (tx, ty) = s.field.coords(to);
    let (dx, dy) = (tx - fx, ty - fy);
    let long = dx.unsigned_abs().max(dy.unsigned_abs());
    // Each axis's step: ±1 along the longer one, the other's difference over the longer one's,
    // stored as an `f32`; then divided by 17 and stored again (0x4678D9 to 0x46795B).
    let step = |d: i32| -> (bool, X87) {
        let unit = X87::ratio(d.unsigned_abs(), long, DOUBLE_BITS)
            .unwrap_or(X87::ZERO)
            .round(SINGLE_BITS);
        (d < 0, seventeenth(unit))
    };
    let (sx, sy) = (step(dx), step(dy));
    let mut x = X87::int(u64::from(fx.unsigned_abs()));
    let mut y = X87::int(u64::from(fy.unsigned_abs()));
    let advance = |v: X87, (back, by): (bool, X87)| {
        if back { v.minus(by) } else { v.plus(by) }.round(SINGLE_BITS)
    };
    for _ in 0..17 * long {
        x = advance(x, sx);
        y = advance(y, sy);
        // The positions stay between the two hexes' columns and rows.
        let hex = s
            .field
            .hex(
                i32::try_from(x.trunc()).unwrap_or(i32::MAX),
                i32::try_from(y.trunc()).unwrap_or(i32::MAX),
            )
            .unwrap_or(Hex(u8::MAX));
        if hex == s.gate {
            if !s.gate_open {
                return true;
            }
        } else if s.line.contains(hex) && !s.broken.contains(hex) {
            return true;
        }
    }
    false
}

/// `v / 17` as the original computes it: an `f32` divided by the `f32` constant 17 in the FPU's
/// precision, stored as an `f32`.
fn seventeenth(v: X87) -> X87 {
    // `v` is an `f32`: its bits give its mantissa `m` and the power of two `v / m` (at most 2^0
    // for the steps, at least 2^-27).
    let bits = v.to_f32_bits();
    if bits == 0 {
        return X87::ZERO;
    }
    let mantissa = (bits & 0x7f_ffff) | 0x80_0000;
    let shift = 150 - (bits >> 23);
    X87::ratio(mantissa, 17u32 << shift.min(27), DOUBLE_BITS)
        .unwrap_or(X87::ZERO)
        .round(SINGLE_BITS)
}

/// The Ballistics level `id` shoots the walls with, if it is a wall shooter of this battle (the
/// original's 0x445D30): the Catapult its side's hero's level (0 without Ballistics,
/// BAT-SIE-011), a stack with `wall_shots` its ability's (BAT-SIE-013).
pub(crate) fn shooter_level(
    state: &BattleState,
    ruleset: &BattleRuleset,
    id: StackId,
) -> Option<u8> {
    let s = state.at(id);
    if s.slot == CATAPULT_SLOT {
        return Some(crate::hero::catapult(&state.heroes, s.side).map_or(0, |c| c.level));
    }
    ruleset.abilities(s.creature).iter().find_map(|a| match *a {
        Ability::WallShots { level } => Some(level),
        _ => None,
    })
}

/// The army slot of the Catapult (BAT-SIE-009): after the army slots and the other war machines.
pub(crate) const CATAPULT_SLOT: u8 = crate::state::CATAPULT_SLOT;

/// Every target index, in order.
const ALL_TARGETS: [u8; MAX_WALL_TARGETS] = [0, 1, 2, 3, 4, 5, 6, 7];

/// Whether any wall-shot target stands (BAT-SIE-010).
pub(crate) fn any_standing(state: &BattleState) -> bool {
    state
        .siege
        .is_some_and(|s| (0..s.targets).any(|t| s.stands(t)))
}

/// The Ballistics level with which the active stack `id` may shoot the walls on its side's
/// command now (BAT-SIE-011, BAT-SIE-013; the original's cursor 0x476490): a siege with a
/// standing target; the Catapult when its side's hero commands it; a stack with `wall_shots`
/// its controller commands for the attacker, whatever its shots and neighbours.
pub(crate) fn commanded_wall_shots(
    state: &BattleState,
    ruleset: &BattleRuleset,
    id: StackId,
) -> Option<u8> {
    if !any_standing(state) {
        return None;
    }
    let s = state.at(id);
    if s.slot == CATAPULT_SLOT {
        crate::hero::catapult(&state.heroes, s.side).map(|c| c.level)
    } else if s.controller() == BattleSide::Attacker {
        shooter_level(state, ruleset, id)
    } else {
        None
    }
}

/// The standing target nearest to `from` among `candidates` (by their difference of indices; a
/// tie is replaced with one roll of `random(1, 100)` ≤ 50, in the candidates' order), `from`
/// itself excluded (the original's 0x445EC0, 0x445D30).
pub(crate) fn nearest(state: &mut BattleState, candidates: &[u8], from: u8) -> Option<u8> {
    let siege = state.siege?;
    let mut best: Option<(u8, u8)> = None;
    for &c in candidates {
        if c == from || !siege.stands(c) {
            continue;
        }
        let d = c.abs_diff(from);
        let take = match best {
            None => true,
            Some((_, b)) if d < b => true,
            Some((_, b)) if d == b => state.rng.range_inclusive(1, 100) <= 50,
            Some(_) => false,
        };
        if take {
            best = Some((c, d));
        }
    }
    best.map(|(c, _)| c)
}

/// The automatic Catapult's aim (BAT-SIE-011; the original's 0x473C00 without Ballistics): the
/// weakest standing wall of the field's `automatic` list (the fewest hit points, a tie drawn with
/// `random(1, n)` in the list's order); else the first standing target of `fallback`, else of
/// all targets in order; none when nothing stands.
pub(crate) fn automatic_aim(state: &mut BattleState, ruleset: &BattleRuleset) -> Option<u8> {
    let (siege, field) = state.siege.zip(ruleset.siege_field())?;
    let weakest = field
        .automatic
        .iter()
        .filter(|&&t| siege.stands(t))
        .map(|&t| siege.parts[usize::from(t)])
        .min();
    if let Some(least) = weakest {
        let tied = |t: &&u8| siege.stands(**t) && siege.parts[usize::from(**t)] == least;
        let n = u32::try_from(field.automatic.iter().filter(tied).count()).unwrap_or(1);
        let k = state.rng.range_inclusive(1, n);
        return field
            .automatic
            .iter()
            .filter(tied)
            .nth(usize::try_from(k - 1).unwrap_or(0))
            .copied();
    }
    field
        .fallback
        .iter()
        .copied()
        .chain(0..siege.targets)
        .find(|&t| siege.stands(t))
}

/// Stack `shooter` shoots the walls at target `aimed` with the Ballistics record of `level`
/// (BAT-SIE-014; the original's 0x445D30, 0x445EC0): per shot of the record, the damage (0, 1 or
/// 2 by the record's chances), then the hit by the aimed target's kind: a miss lands on the
/// nearest other standing wall of the field's `missed` list, or on the aimed target if none
/// stands; the target landed on takes the damage ([`damage_part`]). A later shot keeps its aim
/// while the aimed target stands, else aims at the nearest standing target of all; with none
/// left the shots end. No shot is used.
pub(crate) fn shoot_walls<S: Sink>(
    state: &mut BattleState,
    ruleset: &BattleRuleset,
    (shooter, aimed, level): (StackId, u8, u8),
    sink: &mut S,
) {
    let Some(field) = ruleset.siege_field() else {
        return;
    };
    let record = field.ballistics[usize::from(level).min(field.ballistics.len() - 1)];
    let all = &ALL_TARGETS[..usize::from(state.siege.map_or(0, |s| s.targets))];
    let mut aim = aimed;
    for shot in 0..record.shots {
        if shot > 0 && !state.siege.is_some_and(|s| s.stands(aim)) {
            match nearest(state, all, aim) {
                Some(t) => aim = t,
                None => break,
            }
        }
        let roll = state.rng.range_inclusive(1, 100);
        let [none, one, _] = record.damage.map(u32::from);
        let damage = if roll <= none {
            0
        } else if roll <= none + one {
            1
        } else {
            2
        };
        let kind = state
            .siege
            .and_then(|s| s.kind(aim))
            .unwrap_or(WallKind::Wall);
        let hit = if state.rng.range_inclusive(1, 100) <= u32::from(record.hit[kind as usize]) {
            aim
        } else {
            nearest(state, &field.missed, aim).unwrap_or(aim)
        };
        sink.emit(Event::public(EventKind::WallShot {
            stack: shooter,
            aimed: aim,
            part: hit,
            damage,
        }));
        damage_part(state, hit, damage, sink);
    }
}

/// Target `t` takes `damage` (BAT-SIE-012; the original's 0x465990): its hit points less the
/// damage, at least 0. Destroyed, a set-up wall opens its hex and the gate stands open for good
/// (passable for both sides, its drawbridge's hex no longer moat, the shot's line free); a
/// turret's destruction is T7's. An absent turret destroyed (by Earthquake) does nothing
/// (BAT-QUIRK-22).
pub(crate) fn damage_part<S: Sink>(state: &mut BattleState, t: u8, damage: u8, sink: &mut S) {
    let Some(mut s) = state.siege.filter(|s| damage > 0 && t < s.targets) else {
        return;
    };
    let i = usize::from(t);
    let before = s.parts[i];
    s.parts[i] = before.saturating_sub(damage);
    // rule: BAT-SIE-012, BAT-SIE-015
    if before > 0 && s.parts[i] == 0 && s.present(t) {
        match s.kinds[i] {
            WallKind::Wall => s.opens[i].into_iter().for_each(|h| s.broken.insert(h)),
            WallKind::Gate => {
                s.gate_open = true;
                s.broken.insert(s.gate);
            }
            WallKind::Keep | WallKind::Tower => {}
        }
        state.siege = Some(s);
        sink.emit(Event::public(EventKind::WallDestroyed { part: t }));
    } else {
        state.siege = Some(s);
    }
}

/// Whether Earthquake may be cast (BAT-SIE-015; the original's 0x59ED29): a battle at a town
/// with a fort, by either side, whatever stands.
pub(crate) fn may_quake(state: &BattleState) -> bool {
    state.siege.is_some_and(|s| s.fort >= FortLevel::Fort)
}

/// Earthquake's `hits` (BAT-SIE-015; the original's 0x5A7C80): each hit counts the targets whose
/// hit points exceed the hits drawn on them so far and, with `n` of them, falls on the `random(1,
/// n)`-th target in order whose hit points are above 0, the hits drawn not subtracted (the
/// code's draw, D-14). Every target is drawn from, the absent turrets too (they soak hits;
/// BAT-QUIRK-22 harms no stack). Then each target hit takes its hits as damage
/// ([`damage_part`]).
pub(crate) fn earthquake<S: Sink>(state: &mut BattleState, hits: u32, sink: &mut S) {
    let Some(siege) = state.siege else {
        return;
    };
    let n = usize::from(siege.targets);
    let mut drawn = [0u32; MAX_WALL_TARGETS];
    for _ in 0..hits {
        let open = (0..n)
            .filter(|&t| u32::from(siege.parts[t]) > drawn[t])
            .count();
        let Some(open) = u32::try_from(open).ok().filter(|&o| o > 0) else {
            break;
        };
        let k = state.rng.range_inclusive(1, open);
        let hit = (0..n)
            .filter(|&t| siege.parts[t] > 0)
            .nth(usize::try_from(k - 1).unwrap_or(0))
            .unwrap_or(0);
        drawn[hit] += 1;
    }
    for (t, &count) in drawn.iter().enumerate().take(n) {
        if count > 0 {
            // `t < MAX_WALL_TARGETS`, a hit count at most `hits`.
            let part = u8::try_from(t).unwrap_or(u8::MAX);
            let damage = u8::try_from(count).unwrap_or(u8::MAX);
            sink.emit(Event::public(EventKind::WallQuaked { part, damage }));
            damage_part(state, part, damage, sink);
        }
    }
}
