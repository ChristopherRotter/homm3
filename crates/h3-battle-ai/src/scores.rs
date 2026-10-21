//! The walker's hex scores (0x421680's first step; `docs/formats/installation.md` §7.35, §7.36;
//! BAT-AI-007): what standing on each hex costs or gains the computer's walker, as the plan and
//! the walk read them (BAT-AI-005, BAT-AI-006). From 0, in the original's order:
//!
//! - every Fire Wall's hex loses the value of the wall's damage to the walker (0x4214F0);
//! - under SoD rules, a damaging moat's hexes the value of its damage (0x421590);
//! - on Normal and above or for a human side, the enemies' reach (0x420260): an enemy that cannot
//!   strike any stack of the walker's side this turn takes its strike value on the walker off
//!   every hex its map reaches; one that can adds nothing, but on Hard and above for a computer
//!   side a multi-headed one takes it off the hexes its strike would hit around the walker's side
//!   (0x41FD60);
//! - on Hard and above or for a human side, the crowding (0x41FB60): the hexes around the walker's
//!   side's other stacks lose the area threat on the walker (0x41F920: the area attackers that can
//!   strike this turn and the enemy hero's best area spell), and the hex behind each such hex that
//!   its own dragons' breath reaches loses the value of their strike on it.
//!
//! The last three clamp each score they lower at the floor: minus the value of killing the walker
//! outright. The values are the computer's (`h3_battle::computer`: worth, kill values, damage and
//! strike estimates); the slips among them follow BAT-QUIRK-25 (ADR-0027 point 9; survey
//! `E23-battle-ai.md` §4): slip 3, the multi-headed strike's value taken with its sign untested;
//! slip 18, the Fire Wall's damage valued without the spell's immunity test; slip 19, the Fire
//! Wall's and the moat's values taken with the least attack bonus for both bonuses. Draws: none.

use h3_battle::computer::{self, Context, Map};
use h3_battle::{
    BattleSide, BattleState, Direction, Field, Hex, Stack, StackId, StackTraits, Status,
};
use h3_content::ORIGINAL_AI_SLIPS;
use h3_content_battle::BattleRuleset;

use crate::hexes::{self, Shape};
use crate::walker::{HEXES, Scores};

/// The most hexes a multi-headed enemy may strike a stack from (0x420060: eight around a two-hex
/// stack and three beyond it for a two-hex enemy).
const STRIKE_POSITIONS: usize = 11;

/// The scores of the walker `id` of `side` with its context `ctx` (0x421680's 0x4214F0, 0x421590,
/// 0x420260, 0x41FB60; BAT-AI-007). The reach's mask of the enemies that can strike the side this
/// turn (by record bit: those that can shoot now and those within a turn of one of its stacks)
/// goes to the area threat (0x41F920; the fallback's own call is E23 T3c's).
// rule: BAT-AI-007
pub(crate) fn build(
    state: &BattleState,
    ruleset: &BattleRuleset,
    (id, side): (StackId, BattleSide),
    ctx: &Context,
) -> Scores {
    let mut s = Scorer {
        state,
        ruleset,
        field: Field::new(ruleset.battle().field),
        id,
        side,
        ctx,
        slips: ruleset.quirks().is_enabled(ORIGINAL_AI_SLIPS),
        shoots: computer::shoots(state, ruleset, id),
        floor: 0,
    };
    let mut scores = [0; HEXES];
    s.fire_walls(&mut scores);
    s.moat(&mut scores);
    let difficulty = computer::difficulty(state);
    let human = computer::human(state, side);
    let mut mask = 0;
    if difficulty > 0 || human {
        s.floor = s.floor();
        mask = s.reach(&mut scores);
    }
    if difficulty >= 2 || human {
        s.crowding(&mut scores, mask);
    }
    scores
}

/// The score builders' inputs.
struct Scorer<'a> {
    state: &'a BattleState,
    ruleset: &'a BattleRuleset,
    field: Field,
    id: StackId,
    side: BattleSide,
    ctx: &'a Context,
    /// BAT-QUIRK-25: the original's slips in its estimates.
    slips: bool,
    /// The walker can shoot now (0x4428F0: the shot flag of its values).
    shoots: bool,
    /// The floor (0x420260, 0x41FB60): minus the walker's whole value.
    floor: i32,
}

/// The low 32 bits of `x` as the original's `int`.
fn int32(x: u64) -> i32 {
    x as u32 as i32
}

/// Whether `s` strikes every stack around it or the Cerberi's three (the original's flag
/// `0x80000`; the engine's `attacks_all_adjacent` and `three_headed_attack`).
fn multi_headed(s: &Stack) -> bool {
    s.has(StackTraits::ALL_ADJACENT) || s.has(StackTraits::THREE_HEADED)
}

impl Scorer<'_> {
    /// The hexes of the field.
    fn hexes(&self) -> impl Iterator<Item = Hex> + use<> {
        (0u8..).take(self.field.hex_count()).map(Hex)
    }

    /// The living stacks of `side` but the turrets, in record order, each with its id and its
    /// record bit (+0xF8: the stacks of its side before it, turrets included).
    fn records(&self, side: BattleSide) -> impl Iterator<Item = (StackId, &Stack, u32)> + '_ {
        (0u8..)
            .zip(self.state.stacks())
            .filter(move |(_, s)| s.side == side)
            .zip(0u32..)
            .filter(|((_, s), _)| s.is_alive() && !s.is_turret())
            .map(|((i, s), r)| (StackId(i), s, 1u32 << (r & 31)))
    }

    /// The computer's value of `damage` to the walker (0x442FD0 in the context, its shot flag, no
    /// desperate mode), as the floor, the area threat and the breath lines take it.
    fn value_on_me(&self, damage: u64) -> i32 {
        let flags = (self.shoots, false);
        computer::damage_value(self.state, self.ruleset, self.ctx, (self.id, damage), flags)
    }

    /// The floor (0x420260, 0x41FB60): minus the value of the walker's whole hit points to it.
    fn floor(&self) -> i32 {
        let hp = computer::hit_points(self.state, self.id);
        self.value_on_me(hp).wrapping_neg()
    }

    /// Adds `v` to the score of `hex`, clamped at the floor (a score below it rises to it).
    fn lower(&self, scores: &mut Scores, hex: Hex, v: i32) {
        let score = &mut scores[hex.index()];
        *score = score.wrapping_add(v).max(self.floor);
    }

    /// Takes the value of a hazard's `damage` to the walker off the score of `hex`, unclamped
    /// (0x442FD0, no shot flag, the context's desperate mode; with BAT-QUIRK-25 the least attack
    /// bonus for both bonuses, slip 19, else the context as it is).
    fn hazard(&self, scores: &mut Scores, hex: Hex, damage: u64) {
        let ctx = if self.slips {
            self.ctx.attack_for_both()
        } else {
            *self.ctx
        };
        let flags = (false, self.ctx.desperate());
        let value =
            computer::damage_value(self.state, self.ruleset, &ctx, (self.id, damage), flags);
        let score = &mut scores[hex.index()];
        *score = score.wrapping_sub(value);
    }

    /// The Fire Walls (0x4214F0): each hex with one loses the value of its damage to the walker
    /// (the wall's value with its caster's bonuses, the walker's reductions and protection); with
    /// BAT-QUIRK-25 whether or not the wall's spell can hurt the walker (slip 18), else only if it
    /// can.
    fn fire_walls(&self, scores: &mut Scores) {
        for hex in self.hexes() {
            let wall = computer::fire_wall_damage(self.state, self.ruleset, hex, self.id)
                .filter(|&(_, hurts)| hurts || self.slips);
            if let Some((damage, _)) = wall {
                self.hazard(scores, hex, damage);
            }
        }
    }

    /// The moat (0x421590, under SoD rules only): each hex of a damaging moat loses the value of its
    /// damage to the walker (the drawbridge's none while the gate is open).
    fn moat(&self, scores: &mut Scores) {
        if let Some((hexes, damage)) = computer::damaging_moat(self.state) {
            for hex in hexes.iter() {
                self.hazard(scores, hex, damage);
            }
        }
    }

    /// The enemies' reach (0x420260; Normal and above or a human side): over the other side's
    /// living stacks but the blind, petrified and paralysed, the First Aid Tent, the Ammo Cart and
    /// the turrets, in record order, an enemy that can shoot now only joins the mask; any other
    /// searches its map with its speed + 1 as the range and the turn's speed
    /// ([`computer::reach_map`]). When it can strike a living stack of the walker's side this turn
    /// (the walker too: the stack's position marked at a cost within its speed) it joins the mask
    /// and adds nothing, but a multi-headed one on Hard and above for a computer side
    /// ([`Scorer::heads`]); else, with its strike value on the walker `t` above 0 (0x435C70, no
    /// run-up), every hex its map reached loses `t`, and for a two-hex enemy also the hex beside
    /// each reached one but a stop, on its facing side, that was not reached itself. Returns the
    /// mask.
    fn reach(&self, scores: &mut Scores) -> u32 {
        let (state, ruleset, field) = (self.state, self.ruleset, self.field);
        let hard = computer::difficulty(state) >= 2 && !computer::human(state, self.side);
        let mut mask = 0;
        for (enemy, e, bit) in self.records(self.side.opponent()) {
            if e.spells.status.contains(Status::NO_TURNS)
                || e.has(StackTraits::FIRST_AID)
                || e.has(StackTraits::AMMO_SUPPLY)
            {
                continue;
            }
            if computer::shoots(state, ruleset, enemy) {
                mask |= bit;
                continue;
            }
            let map = computer::reach_map(state, ruleset, enemy);
            let speed = u16::from(e.speed_now());
            if self
                .records(self.side)
                .any(|(_, o, _)| map.mark(o.hex).is_some_and(|c| c <= speed))
            {
                mask |= bit;
                if hard && multi_headed(e) {
                    self.heads(scores, (enemy, e), &map);
                }
                continue;
            }
            let pair = (enemy, self.id);
            let minus = computer::strike_value(state, ruleset, self.ctx, pair, 0).wrapping_neg();
            if minus >= 0 {
                continue;
            }
            for hex in self.hexes().filter(|&h| map.reached(h)) {
                self.lower(scores, hex, minus);
            }
            let d = if Shape::of(e).faces_right() {
                Direction::Right
            } else {
                Direction::Left
            };
            let beside = self
                .hexes()
                .filter(|_| e.is_two_hex())
                .filter(|&h| map.reached(h) && !map.is_stop(h))
                .filter_map(|h| field.neighbor(h, d))
                .filter(|&n| !map.reached(n));
            for hex in beside {
                self.lower(scores, hex, minus);
            }
        }
        mask
    }

    /// A multi-headed enemy's strike (0x41FD60): with `t` its strike value on the walker (its sign
    /// untested with BAT-QUIRK-25, slip 3; else nothing unless above 0), over the walker's side's
    /// other living stacks it can strike this turn, in record order: the positions it strikes the
    /// stack from that its map reached within its speed (0x420060: the stack's neighbours, and for
    /// a two-hex enemy the three beyond the hex beside the stack on the side it comes from), and
    /// from each, the last found first, the hexes its strike hits there ([`hexes::pattern`]), the
    /// last direction first: each loses `t` the first time any stack gives it and 1 at every later
    /// stack, once per stack.
    fn heads(&self, scores: &mut Scores, (enemy, e): (StackId, &Stack), map: &Map) {
        let pair = (enemy, self.id);
        let t = computer::strike_value(self.state, self.ruleset, self.ctx, pair, 0);
        if t <= 0 && !self.slips {
            return;
        }
        let minus = t.wrapping_neg();
        let speed = u16::from(e.speed_now());
        let (field, shape, three) = (self.field, Shape::of(e), e.has(StackTraits::THREE_HEADED));
        let (beyond, from) = match (shape.two_hex, shape.faces_right()) {
            (false, _) => (0..0, 0),
            (true, true) => (3..6, -1),
            (true, false) => (0..3, 1),
        };
        let mut seen = [false; HEXES];
        let struck = self
            .records(self.side)
            .filter(|&(o, s, _)| o != self.id && map.mark(s.hex).is_some_and(|c| c <= speed));
        for (_, o, _) in struck {
            let (o_shape, pos) = (Shape::of(o), i32::from(o.hex.0));
            let at = |hex: i32, d: u8| hexes::stack_neighbor_at(field, o_shape, hex, d);
            let found = (0..hexes::directions(o_shape))
                .filter_map(|d| at(pos, d))
                .chain(beyond.clone().filter_map(|d| at(pos + from, d)))
                .filter(|&h| map.reached(h) && map.cost(h) <= speed);
            let mut positions = [Hex(0); STRIKE_POSITIONS];
            let mut n = 0;
            for (slot, hex) in positions.iter_mut().zip(found) {
                *slot = hex;
                n += 1;
            }
            let mut once = [false; HEXES];
            for &p in positions.iter().take(n).rev() {
                let pattern = hexes::pattern(field, (shape, p, three), (o_shape, o.hex));
                let hit = (0..hexes::directions(shape))
                    .rev()
                    .filter(|d| pattern & 1 << d != 0)
                    .filter_map(|d| hexes::stack_neighbor(field, shape, p, d))
                    .filter(|h| !once[h.index()]);
                for hex in hit.collect::<Hits>().iter() {
                    let v = if seen[hex.index()] { -1 } else { minus };
                    self.lower(scores, hex, v);
                    seen[hex.index()] = true;
                    once[hex.index()] = true;
                }
            }
        }
    }

    /// The crowding and breath lines (0x41FB60; Hard and above or a human side): with `T` the area
    /// threat on the walker ([`Scorer::area_threat`]), over the walker's side's other living stacks
    /// but the turrets, in record order, each with `K` the value to the walker of its own strike on
    /// it when it breathes (0x442780 by its creatures, capped; 0x442FD0), else 0; a stack with both
    /// 0 is skipped. Its neighbours, the last direction first: each loses `T` the first time and 1
    /// at every later stack (one set of marks), and for a breath attacker the hex one further in
    /// the same direction loses `K`.
    fn crowding(&self, scores: &mut Scores, mask: u32) {
        let t = self.area_threat(mask);
        let field = self.field;
        let mut seen = [false; HEXES];
        for (o, s, _) in self.records(self.side).filter(|&(o, _, _)| o != self.id) {
            let breath = s.has(StackTraits::BREATH);
            let k = if breath {
                let count = u64::from(s.count);
                let pair = (o, self.id);
                let damage =
                    computer::damage_estimate(self.state, self.ruleset, pair, count, false, 0);
                self.value_on_me(damage)
            } else {
                0
            };
            if t == 0 && k == 0 {
                continue;
            }
            let shape = Shape::of(s);
            let around = (0..hexes::directions(shape))
                .rev()
                .filter_map(|d| hexes::stack_neighbor(field, shape, s.hex, d).map(|h| (d, h)));
            for (d, hex) in around {
                if t != 0 {
                    let v = if seen[hex.index()] {
                        -1
                    } else {
                        t.wrapping_neg()
                    };
                    seen[hex.index()] = true;
                    self.lower(scores, hex, v);
                }
                if let Some(behind) = hexes::neighbor(field, shape, hex, d).filter(|_| breath) {
                    self.lower(scores, behind, k.wrapping_neg());
                }
            }
        }
    }

    /// The area threat on the walker (0x41F920): the damage of the other side's stacks in the mask
    /// that shoot balls and can shoot now (the area_shot ability: the Magogs, Liches and Power
    /// Liches) or breathe, by their creatures, capped at the walker's hit points (0x442780), plus
    /// the other side's hero's best area spell on it ([`computer::area_spell_damage`]), as 32-bit
    /// integers; the value of that damage to the walker when above 0 (0x442FD0), else the sum.
    fn area_threat(&self, mask: u32) -> i32 {
        let (state, ruleset) = (self.state, self.ruleset);
        let other = self.side.opponent();
        let mut d = 0i32;
        for (r, s, _) in self.records(other).filter(|&(_, _, bit)| mask & bit != 0) {
            let count = u64::from(s.count);
            let pair = (r, self.id);
            let estimate = |shot| {
                int32(computer::damage_estimate(
                    state, ruleset, pair, count, shot, 0,
                ))
            };
            if s.has(StackTraits::AREA_SHOT) && computer::shoots(state, ruleset, r) {
                d = d.wrapping_add(estimate(true));
            }
            if s.has(StackTraits::BREATH) {
                d = d.wrapping_add(estimate(false));
            }
        }
        d = d.wrapping_add(computer::area_spell_damage(state, ruleset, other, self.id));
        if d > 0 {
            self.value_on_me(u64::from(d.unsigned_abs()))
        } else {
            d
        }
    }
}

/// The hexes a multi-headed strike hits from one position (at most eight).
struct Hits {
    hexes: [Hex; 8],
    len: usize,
}

impl Hits {
    fn iter(&self) -> impl Iterator<Item = Hex> + '_ {
        self.hexes.iter().take(self.len).copied()
    }
}

impl FromIterator<Hex> for Hits {
    fn from_iter<I: IntoIterator<Item = Hex>>(iter: I) -> Self {
        let mut hits = Self {
            hexes: [Hex(0); 8],
            len: 0,
        };
        for (slot, hex) in hits.hexes.iter_mut().zip(iter) {
            *slot = hex;
            hits.len += 1;
        }
        hits
    }
}
