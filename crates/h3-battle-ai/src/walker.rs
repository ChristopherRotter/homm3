//! The computer's walker (0x421F80, 0x421680; `docs/formats/installation.md` §7.35, §7.37,
//! §7.38; BAT-AI-005, BAT-AI-006): a stack that cannot shoot now values each enemy by the best
//! hex to strike it from (the plan: 0x4360C0, 0x436840 and 0x436300 with the shooters it would
//! block, the multi-headed and breath strikes and the enemy's retaliation), its melee value with
//! the run-up (0x435C70) and a draw of 75 to 100 % (0x421B02), takes the best, and moves and
//! attacks it within one turn, else walks towards the plan's hex (0x41F580). Nothing chosen:
//! Wait on Hard and above or for a human side, outside the wait phase; else Defend.
//!
//! Not built yet (BAT-STUB-4): the hex scores (0x4214F0, 0x421590, 0x420260, 0x41FB60; E23 T3b)
//! stand at 0 and "the stronger side" (0x420A80; T3c) is false; the last retaliation's gain
//! (0x41F3B0), the fallbacks (0x41F920, 0x420760, 0x4208F0), a besieger's walk to its wall and a
//! defender holding the walls (0x4213F0) are T3c's; the creature casts (0x421280) T4's. The
//! walker's turn in the tactics phase is T6's (BAT-STUB-5).

use h3_battle::computer::{self, Context, Map};
use h3_battle::{
    BattleSide, BattleState, Command, Field, Hex, Stack, StackId, StackTraits, Status,
};
use h3_content_battle::{Ability, BattleRuleset};
use h3_kernel::Rng;
use h3_kernel::arith::{DOUBLE_BITS, X87};

use crate::hexes::{self, Shape};

/// Hexes a table of hex scores covers.
const HEXES: usize = 256;

/// The draw of a candidate's value, percent (0x421B02: `random(75, 100)`).
const DRAW: (u32, u32) = (75, 100);

/// The walker's hex scores (0x421680's 187 integers): what standing on a hex costs or gains.
pub(crate) type Scores = [i32; HEXES];

/// The best hex to strike an enemy from (0x4360C0's +0x1C to +0x24): the hex, the turns to it and
/// the value of the hex's extras and dangers.
#[derive(Debug, Clone, Copy)]
struct Plan {
    hex: Hex,
    turns: u32,
    value: i32,
}

/// The best candidate so far (0x421680's locals): the enemy, its plan, its drawn value per turn
/// and whether only the plan's hex counts. (Its drawn plan value per turn, which the fallbacks and
/// the creature casts read, joins with T3c and T4.)
#[derive(Debug, Clone, Copy)]
struct Best {
    enemy: StackId,
    plan: Plan,
    value: i32,
    only: bool,
}

/// A striker's estimate of a strike from a position (0x436620, 0x436760): the striker, its shape
/// there and its position, and the creatures that strike.
#[derive(Debug, Clone, Copy)]
struct Striker {
    id: StackId,
    shape: Shape,
    at: Hex,
    count: u64,
}

/// One walker's choice: its battle, side, context, map and scores.
pub(crate) struct Walker<'a> {
    state: &'a BattleState,
    ruleset: &'a BattleRuleset,
    field: Field,
    id: StackId,
    me: &'a Stack,
    side: BattleSide,
    ctx: Context,
    map: Map,
    /// The hex scores (T3b: all 0).
    scores: Scores,
    /// The living stack on each hex (0x4E7170).
    on: [Option<StackId>; HEXES],
    /// The game is above Easy or the side's player human (`[game]+0x1F6D8 > 0`, +0x54A4).
    normal: bool,
}

/// The command of the computer's walker `id` of `side` (0x421F80; BAT-AI-005): the walker's choice
/// ([`Walker::choose`], its draws from `rng`); without one, Wait on Hard and above or for a human
/// side outside the wait phase, else Defend.
// rule: BAT-AI-005
pub(crate) fn turn(
    state: &BattleState,
    ruleset: &BattleRuleset,
    (id, side): (StackId, BattleSide),
    rng: &mut Rng,
) -> Command {
    let walker = Walker::new(state, ruleset, (id, side));
    // stub: BAT-STUB-4
    // A choice of nothing asks the creature's cast entry again (0x421280): T4.
    walker.choose(rng).unwrap_or_else(|| {
        let hard = computer::difficulty(state) >= 2 || computer::human(state, side);
        if hard && !computer::wait_phase(state) {
            Command::Wait
        } else {
            Command::Defend
        }
    })
}

impl<'a> Walker<'a> {
    pub(crate) fn new(
        state: &'a BattleState,
        ruleset: &'a BattleRuleset,
        (id, side): (StackId, BattleSide),
    ) -> Self {
        let field = Field::new(ruleset.battle().field);
        let mut on = [None; HEXES];
        for (i, s) in (0u8..).zip(state.stacks()) {
            if s.is_alive() && !s.is_turret() {
                for h in s.footprint(field).hexes() {
                    on[h.index()] = Some(StackId(i));
                }
            }
        }
        Self {
            state,
            ruleset,
            field,
            id,
            me: &state.stacks()[id.index()],
            side,
            ctx: computer::context(state, ruleset, side),
            map: computer::map(state, ruleset, id, side),
            // stub: BAT-STUB-4
            // The hex scores (§7.35 step 1, §7.36): T3b.
            scores: [0; HEXES],
            on,
            normal: computer::difficulty(state) > 0 || computer::human(state, side),
        }
    }

    /// The walker with the hex scores `scores` (the tests' stand-in for T3b's).
    #[cfg(test)]
    pub(crate) fn scored(self, scores: Scores) -> Self {
        Self { scores, ..self }
    }

    fn stack(&self, id: StackId) -> &'a Stack {
        &self.state.stacks()[id.index()]
    }

    /// The record index of stack `id` among its side's (+0xF8): the bit the computer's masks set.
    fn record(&self, id: StackId) -> u32 {
        let side = self.stack(id).side;
        let before = self.state.stacks().iter().take(id.index());
        let n = before.filter(|s| s.side == side).count();
        u32::try_from(n).unwrap_or(31)
    }

    fn score(&self, hex: Hex) -> i32 {
        self.scores[hex.index()]
    }

    /// The walker's choice (0x421680 outside a prediction; BAT-AI-005): over the other side's
    /// living stacks but the turrets, in record order (a bound walker only those next to it),
    /// each with a [`Walker::plan`]: its value `v` is the melee value with the plan's cost as the
    /// run-up when the plan takes one turn (0x435C70), and the plan's value `P` joins it
    /// (outside Easy for a computer side, a positive `P` of a stack not summoned: the Harpies take
    /// the larger, `P` alone for a negative `v` when the side is the stronger, else the sum); a
    /// negative `v` outside Easy drops the enemy when `P` is below its hex's score; then the draw,
    /// `v × random(75, 100) / 100`. The best: one not blind, petrified or paralysed, then fewer
    /// turns, the value (the best's per turn against the candidate's whole), more hit points lost
    /// by the top creature, the lower cost to the plan's hex, the later record. Then: only the
    /// plan's hex when `P` was taken (a walk there, nothing on it); an attack within one turn; a
    /// walk towards the plan's hex (nothing on it). No candidate: nothing.
    // rule: BAT-AI-005
    pub(crate) fn choose(&self, rng: &mut Rng) -> Option<Command> {
        let me = self.me;
        let still = me.speed_now() == 0 || computer::bound(self.state, self.id);
        let other = self.side.opponent();
        let mut best: Option<Best> = None;
        for (i, t) in (0u8..).zip(self.state.stacks()) {
            let enemy = StackId(i);
            // A bound walker or one without speed only those next to it (its map's marks at 0).
            if t.side != other
                || !t.is_alive()
                || t.is_turret()
                || (still && self.map.cost(t.hex) > 0)
            {
                continue;
            }
            // stub: BAT-STUB-4
            // A defender holding the walls takes only enemies inside them (0x4213F0), and on Hard
            // the last retaliation's gain starts the value (0x41F3B0): T3c.
            let Some(plan) = self.plan(enemy) else {
                continue;
            };
            // The melee value with the run-up when the plan takes one turn: a walker on its turn can
            // act (the original also asks whether it is blind, petrified or paralysed, or the First
            // Aid Tent or the Ammo Cart, for the predictions' stacks: T4).
            let run_up = u8::try_from(self.map.cost(plan.hex) * u16::from(plan.turns == 1))
                .unwrap_or(u8::MAX);
            let pair = (self.id, enemy);
            let ctx = &self.ctx;
            let mut v =
                computer::strike_value(self.state, self.ruleset, ctx, pair, (false, run_up));
            let p = plan.value;
            let mut only = false;
            // stub: BAT-STUB-4
            // With a negative `v` and the side the stronger at range, `P` alone: T3c (0x420A80).
            if p > 0
                && !me.has(StackTraits::SUMMONED)
                && self.normal
                && me.has(StackTraits::STRIKE_AND_RETURN)
            {
                // The Harpies and Harpy Hags (72, 73 by number; the engine: strike and return).
                only = v < p;
                v = v.max(p);
            } else {
                v = v.wrapping_add(p);
            }
            if v < 0 && self.normal && p < self.score(plan.hex) {
                continue;
            }
            let r = i32::try_from(rng.range_inclusive(DRAW.0, DRAW.1)).unwrap_or(100);
            let v = r.wrapping_mul(v) / 100;
            if best.is_some_and(|b| !self.better(&b, (t, &plan), v)) {
                continue;
            }
            let turns = i32::try_from(plan.turns).unwrap_or(1);
            best = Some(Best {
                enemy,
                plan,
                value: v / turns,
                only,
            });
        }
        // stub: BAT-STUB-4
        // Without a candidate, a besieger walks to its row's wall, else the fallbacks guard a
        // shooter or step to safety (§7.35, §7.38), and with the best's value per turn below 0
        // and below its drawn `P` the side the stronger tries them first: T3c.
        let b = best?;
        // stub: BAT-STUB-4
        // The creature's cast entry first, outside the tactics phase (0x421280): T4.
        if b.plan.turns <= 1 && !b.only {
            // The original's move and attack stops where a step enters a stop and then skips the
            // attack (0x441FA0, 0x478971; §7.16): a two-hex walker whose other hex lies on one
            // at the plan's hex (the plan counts a stop by the position hex alone) only moves.
            let other = Shape::of(me).other(self.field, b.plan.hex);
            return Some(if self.map.is_stop(other) {
                Command::Move { to: b.plan.hex }
            } else {
                Command::Attack {
                    target: b.enemy,
                    from: b.plan.hex,
                }
            });
        }
        if b.plan.hex == me.hex {
            return None;
        }
        self.walk(b.plan.hex, b.plan.turns > 1)
    }

    /// Whether the candidate `t` with `plan` and the drawn value `v` replaces `best` (0x421B2F to
    /// 0x421C16): one not blind, petrified or paralysed over one that is; then fewer turns, the
    /// higher value, more hit points lost by the top creature, and a cost to the plan's hex no
    /// higher than the best's (so the later record at a tie).
    fn better(&self, best: &Best, (t, plan): (&Stack, &Plan), v: i32) -> bool {
        let able = |s: &Stack| !s.spells.status.contains(Status::NO_TURNS);
        let lost = |s: &Stack| s.max_hp - s.top_hp;
        let b = self.stack(best.enemy);
        able(t)
            .cmp(&able(b))
            .then(plan.turns.cmp(&best.plan.turns).reverse())
            .then(v.cmp(&best.value))
            .then(lost(t).cmp(&lost(b)))
            .then(self.map.cost(best.plan.hex).cmp(&self.map.cost(plan.hex)))
            .is_ge()
    }

    /// The plan against `enemy` (0x4360C0, 0x436840; BAT-AI-005): the best hex to strike it from,
    /// [`Walker::around`] its position, a two-hex enemy's other hex and, for a two-hex walker,
    /// three hexes beyond the hex next to the enemy's side that faces it (left of it for a walker
    /// facing right, right for one facing left); none without such a hex.
    fn plan(&self, enemy: StackId) -> Option<Plan> {
        let (me, t) = (self.me, self.stack(enemy));
        let per = |s: &Stack, hp: u64| hp.div_ceil(u64::from(s.max_hp.max(1)));
        let own = per(me, computer::hit_points(self.state, self.id));
        let pair = (self.id, enemy);
        let dealt =
            computer::damage_estimate(self.state, self.ruleset, pair, own, (false, true), 0);
        let left = per(
            t,
            computer::hit_points(self.state, enemy).saturating_sub(dealt),
        );
        let mut best = None;
        let (shape, t_shape) = (Shape::of(me), Shape::of(t));
        let job = (enemy, own, left);
        self.around(&mut best, job, (t.hex, 0..6));
        if t.is_two_hex() {
            self.around(&mut best, job, (t_shape.other(self.field, t.hex), 0..6));
        }
        if me.is_two_hex() {
            // 0x436880: the enemy's hex on the walker's side when they face each other.
            let right = me.side == BattleSide::Attacker;
            let facing = t.is_two_hex() && (t.side == BattleSide::Attacker) != right;
            let at = if facing {
                t_shape.other(self.field, t.hex)
            } else {
                t.hex
            };
            let (d, dirs) = if right { (4, 3..6) } else { (1, 0..3) };
            if let Some(beyond) = hexes::neighbor(self.field, shape, at, d) {
                self.around(&mut best, job, (beyond, dirs));
            }
        }
        best
    }

    /// The hexes around `h` in the original's directions `dirs` as the plan's candidates against
    /// `enemy` (0x436300; BAT-AI-005), the walker striking by `own` creatures and the enemy by
    /// `left` after it: a position the walker's map reaches standing there; its turns `⌈cost /
    /// speed⌉`, one more on a stop, at least 1, no more than the best's. Its value `P`: the shooters it would block ([`Walker::blocked`]); outside Easy
    /// for a computer side the multi-headed and breath strikes' values ([`Walker::heads`],
    /// [`Walker::breath`]) less those of the enemy's retaliation while it has one (the walker's
    /// strikes retaliated, the enemy not turned to stone); a two-hex walker's other hex's blocked
    /// shooters (one mask) and the lower of its hexes' scores, else the hex's score. The best:
    /// fewer turns, the higher `P`, the lower cost (for the jousters outside Easy the higher: a
    /// longer run-up), the earlier.
    fn around(
        &self,
        best: &mut Option<Plan>,
        (enemy, own, left): (StackId, u64, u64),
        (h, dirs): (Hex, core::ops::Range<u8>),
    ) {
        let (me, t) = (self.me, self.stack(enemy));
        let shape = Shape::of(me);
        let speed = u32::from(me.speed_now());
        let retaliates = !me.has(StackTraits::NO_ENEMY_RETALIATION)
            && !t.spells.status.contains(Status::PETRIFIED)
            && t.retaliations_left > 0;
        let jousts = self
            .ruleset
            .abilities(me.creature)
            .iter()
            .any(|a| matches!(a, Ability::Jousting { .. }));
        for d in dirs {
            let Some(n) = hexes::neighbor(self.field, shape, h, d) else {
                continue;
            };
            let Some(cost) = self.map.position(n).map(u32::from) else {
                continue;
            };
            // Without speed 1 at cost 0, else 100: only a war machine has none, and none walks.
            let turns = (cost.div_ceil(speed.max(1)) + u32::from(self.map.is_stop(n))).max(1);
            if best.is_some_and(|b| b.turns < turns) {
                continue;
            }
            let mut mask = 0;
            let mut p = self.blocked(n, &mut mask);
            if self.normal {
                let me_at = Striker {
                    id: self.id,
                    shape,
                    at: n,
                    count: own,
                };
                let them = Striker {
                    id: enemy,
                    shape: Shape::of(t),
                    at: t.hex,
                    count: left,
                };
                p = p
                    .wrapping_add(self.heads(self.side, me_at, (enemy, t.hex)))
                    .wrapping_add(self.breath(self.side, me_at, (enemy, t.hex)));
                if retaliates {
                    p = p
                        .wrapping_sub(self.heads(t.side, them, (self.id, n)))
                        .wrapping_sub(self.breath(t.side, them, (self.id, n)));
                }
            }
            let other = shape.other(self.field, n);
            if me.is_two_hex() {
                p = p.wrapping_add(self.blocked(other, &mut mask));
            }
            p = p.wrapping_add(self.score(n).min(self.score(other)));
            let take = best.is_none_or(|b| {
                b.turns != turns
                    || p.cmp(&b.value).then_with(|| {
                        let (c, kept) = (self.map.cost(n), self.map.cost(b.hex));
                        if jousts && self.normal {
                            c.cmp(&kept)
                        } else {
                            kept.cmp(&c)
                        }
                    }) == core::cmp::Ordering::Greater
            });
            if take {
                *best = Some(Plan {
                    hex: n,
                    turns,
                    value: p,
                });
            }
        }
    }

    /// The shooters a walker at `h` would block (0x436180; BAT-AI-005): 0 on Easy for a computer
    /// side; else over `h`'s neighbours, each stack not of the walker's side, not the walker and
    /// not in `mask` (now added to it) that can shoot now with the walker left out of its blockers
    /// counts `max(1, ⌊h × (w₁ − w₀) / hp⌋)`: its hit points times its worth with the shot flag
    /// less without, over its hit points per creature, as the original's doubles (a shooter the
    /// walker blocks already has no shot flag in `w₁`: 1).
    fn blocked(&self, h: Hex, mask: &mut u32) -> i32 {
        if !self.normal {
            return 0;
        }
        let shape = Shape::of(self.me);
        let double = |n: u64| X87::int(n).round(DOUBLE_BITS);
        (0..6)
            .filter_map(|d| {
                hexes::neighbor(self.field, shape, h, d).and_then(|n| self.on[n.index()])
            })
            .fold(0i32, |sum, x| {
                let s = self.stack(x);
                let bit = 1u32 << self.record(x);
                if s.side == self.side || x == self.id || *mask & bit != 0 {
                    return sum;
                }
                *mask |= bit;
                if !computer::shoots_without(self.state, self.ruleset, x, self.id) {
                    return sum;
                }
                let w1 = computer::worth(self.state, self.ruleset, &self.ctx, x, true);
                let w0 = computer::worth(self.state, self.ruleset, &self.ctx, x, false);
                let gain = double(computer::hit_points(self.state, x))
                    .times(w1.minus(w0))
                    .over(double(s.max_hp.into()))
                    .unwrap_or(X87::ZERO);
                sum.wrapping_add(computer::ftol(gain).max(1))
            })
    }

    /// The value of a multi-headed strike of `striker` on the stack `t` at `t_hex` (0x436620;
    /// BAT-AI-005): 0 for a striker that is not multi-headed; else over the directions its strike
    /// hits from its position ([`hexes::pattern`]: every one, the Cerberi three), each stack there
    /// not of `side`, not `t` and not counted yet adds the value of the striker's damage on it
    /// (by its count, capped; the context's worth, no shot flag).
    fn heads(&self, side: BattleSide, striker: Striker, (t, t_hex): (StackId, Hex)) -> i32 {
        let s = self.stack(striker.id);
        if !s.has(StackTraits::ALL_ADJACENT) && !s.has(StackTraits::THREE_HEADED) {
            return 0;
        }
        let shape = (striker.shape, striker.at, s.has(StackTraits::THREE_HEADED));
        let pattern = hexes::pattern(self.field, shape, (Shape::of(self.stack(t)), t_hex));
        let mut counted = 1u32 << self.record(t);
        (0..8u8)
            .filter(|d| pattern & 1 << d != 0)
            .filter_map(|d| {
                hexes::stack_neighbor(self.field, striker.shape, striker.at, d)
                    .and_then(|n| self.on[n.index()])
            })
            .fold(0i32, |sum, x| {
                let bit = 1u32 << self.record(x);
                if self.stack(x).side == side || counted & bit != 0 {
                    return sum;
                }
                counted |= bit;
                let pair = (striker.id, x);
                let damage = computer::damage_estimate(
                    self.state,
                    self.ruleset,
                    pair,
                    striker.count,
                    (false, true),
                    0,
                );
                let value = computer::damage_value(
                    self.state,
                    self.ruleset,
                    &self.ctx,
                    (x, damage),
                    (false, false),
                );
                sum.wrapping_add(value)
            })
    }

    /// The value of a breath strike of `striker` on the stack `t` at `t_hex` (0x436760;
    /// BAT-AI-005): 0 for a striker without breath; else the stack on the hex one further from
    /// the hex it strikes, in the direction it strikes ([`hexes::direction_to`]), unless it is
    /// `t`: the value of the striker's damage on it (by its count, capped; in the context's
    /// desperate mode at 1000 per creature), negative for a stack of `side` (the walker's own hexes
    /// count where it stands now).
    fn breath(&self, side: BattleSide, striker: Striker, (t, t_hex): (StackId, Hex)) -> i32 {
        if !self.stack(striker.id).has(StackTraits::BREATH) {
            return 0;
        }
        let target = (Shape::of(self.stack(t)), t_hex);
        hexes::direction_to(self.field, (striker.shape, striker.at), target)
            .and_then(|d| {
                let first = hexes::stack_neighbor(self.field, striker.shape, striker.at, d)?;
                hexes::neighbor(self.field, striker.shape, first, d)
            })
            .and_then(|h| self.on[h.index()])
            .filter(|&x| x != t)
            .map_or(0, |x| {
                let pair = (striker.id, x);
                let damage = computer::damage_estimate(
                    self.state,
                    self.ruleset,
                    pair,
                    striker.count,
                    (false, true),
                    0,
                );
                let desperate = self.ctx.desperate();
                let value = computer::damage_value(
                    self.state,
                    self.ruleset,
                    &self.ctx,
                    (x, damage),
                    (false, desperate),
                );
                if self.stack(x).side == side {
                    value.wrapping_neg()
                } else {
                    value
                }
            })
    }

    /// The computer's walk towards the position `to` (0x41F580; BAT-AI-006): nothing without a
    /// path ([`computer::path`]; for a bound walker or one without speed: T3c's callers, the
    /// plan's hex of such a walker is its own). Along the path, at most
    /// its speed in steps and ending after a stop it enters (a flyer has none), each position it
    /// stands on may become the end. With `careful` on Hard and above or for a human side
    /// (the walker's own side), outside the wait phase, every end must score no lower than the
    /// last end (both hexes of a two-hex walker; its start's score first), else only from the step
    /// that leaves at most `s × ⌊(n − 1) / s⌋` of the path's `n` steps once an end was found, so it
    /// first reaches the hex from which the rest takes whole turns. Then, careful: an end short of
    /// the last hex reached with a score no lower than the start's is Wait; an end at its own
    /// position Wait, else Defend; any other end a move there.
    // rule: BAT-AI-006
    pub(crate) fn walk(&self, to: Hex, careful: bool) -> Option<Command> {
        let me = self.me;
        let speed = usize::from(me.speed_now());
        let path = computer::path(self.state, self.ruleset, (self.id, self.side), to);
        let steps = path.steps();
        let n = steps.len();
        let hard = computer::difficulty(self.state) >= 2 || computer::human(self.state, me.side);
        let careful = careful && hard && !computer::wait_phase(self.state);
        let shape = Shape::of(me);
        let start = self
            .score(me.hex)
            .min(self.score(shape.other(self.field, me.hex)));
        let whole = speed * (n.saturating_sub(1) / speed.max(1));
        let (mut end, mut found, mut kept, mut last) = (me.hex, false, start, me.hex);
        for (k, &(hex, flown)) in steps.iter().take(speed).enumerate() {
            last = hex;
            if flown {
                continue;
            }
            let other = shape.other(self.field, hex);
            let tested = careful || (n - k <= whole && found);
            if !tested || (self.score(hex) >= kept && self.score(other) >= kept) {
                (end, found, kept) = (hex, true, self.score(hex).min(self.score(other)));
            }
            if !me.has(StackTraits::FLIES) && (path.is_stop(hex) || path.is_stop(other)) {
                break;
            }
        }
        let command = if end == me.hex {
            if careful {
                Command::Wait
            } else {
                Command::Defend
            }
        } else if careful && end != last && kept >= start {
            Command::Wait
        } else {
            Command::Move { to: end }
        };
        // No path (the walker's map reaches the plan's hex, so its walk finds one): nothing.
        (n > 0).then_some(command)
    }
}
