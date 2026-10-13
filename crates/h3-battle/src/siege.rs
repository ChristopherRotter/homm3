//! A battle at a town (E04c T5, ADR-0023): the walls, the gate and the moat (BAT-SIE-001 to
//! BAT-SIE-008). The town's facts come through `BattleSetup::siege`; its faction's siege record
//! and the siege's field come from the ruleset (`h3_content_battle::Siege`, `SiegeField`). The
//! original's routines: `docs/formats/installation.md` §7.27 and §7.28.
//!
//! The walls' hit points join with the Catapult (T6): until then every wall stands. The
//! turrets, the Catapult and the town's other rules (the controls, the experience, the
//! defenders' morale and luck, Teleport's and the summon's limits) come with E04c T6 to T8
//! (BAT-STUB-2).

use h3_content::SpellId;
use h3_content_battle::{BattleRuleset, BattlefieldId};
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

    /// The moat's hexes now (BAT-SIE-004): the bridges only while the gate is closed.
    pub fn moat(&self) -> HexSet {
        if self.gate_open {
            self.moat.without(self.bridges())
        } else {
            self.moat
        }
    }

    /// The wall hexes (BAT-SIE-002).
    pub const fn walls(&self) -> HexSet {
        self.line
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
    (setup, heroes, battlefield): (&BattleSetup, &[Option<HeroRecord>; 2], Option<BattlefieldId>),
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
                rounds: spell.magic.obstacle.as_ref().map_or(0, |shape| shape.rounds),
            }
        });
    let kept = line
        .union(moat)
        .union(mines.map_or(HexSet::new(), |m| m.hexes));
    if obstacles.intersects(kept) {
        return Err(SetupError::InvalidSiege);
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
    };
    Ok(Some((siege, mines)))
}

/// The hexes the setup may not place stacks on (BAT-SIE-002): the walls and the closed gate's.
pub(crate) fn placement_blocked(siege: Option<&SiegeState>) -> HexSet {
    siege.map_or(HexSet::new(), |s| blocks(s, false))
}

/// The hexes the walls and the gate block (BAT-SIE-002, BAT-SIE-003): every wall hex but the
/// gate's; while the gate is closed also the gate's and the drawbridge's, unless `passes`.
fn blocks(s: &SiegeState, passes: bool) -> HexSet {
    if s.gate_open || passes {
        s.line.without(s.gate_hexes())
    } else {
        let mut blocked = s.line;
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
/// every town (the mover on one of them included, as the original reads the hexes).
// rule: BAT-SIE-002, BAT-SIE-003
pub(crate) fn blocked_for(state: &BattleState, mover: &Stack) -> HexSet {
    state.siege.map_or(HexSet::new(), |s| {
        let passes = !s.gate_open
            && mover.controller() == BattleSide::Defender
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
    let Some(s) = state.siege.filter(|s| s.gate_open && s.fort >= FortLevel::Fort) else {
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
    let mut hexes = fp.hexes();
    if right { hexes.max() } else { hexes.min() }.unwrap_or(fp.front)
}

/// Whether the straight line from `from` to `to` meets a wall (BAT-SIE-008; the original's
/// 0x467840): on the column/row grid, from `from`'s column and row in 17 × the larger difference
/// steps of 1/17 of a cell along the longer axis and the matching fraction along the other, each
/// position a single-precision value and truncated to a hex at every step; the gate's hex
/// blocks while the gate is closed, any other wall hex while it stands.
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
        } else if s.line.contains(hex) {
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
