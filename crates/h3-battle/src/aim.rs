//! The computer's aim for the automatic turrets (BAT-SIE-020 to BAT-SIE-023): which attacker stack
//! an arrow tower shoots when it acts on its own. The original's turret has a routine of its own
//! (0x41E190, `docs/formats/installation.md` §7.31), not the computer's shooter: every attacker
//! stack scores the kills of the turret's mean damage times the computer's worth of one of its
//! creatures (0x442A50), divided by the turns it needs to attack (0x448BD0); the best score wins,
//! a tie the later stack. Its numbers are the original's x87 arithmetic in every profile: they
//! decide a choice, not an amount.

use h3_content_battle::BattleRuleset;
use h3_contracts::BattleSide;
use h3_kernel::arith::{DOUBLE_BITS, X87};

use crate::binds;
use crate::combat::can_shoot;
use crate::damage::{Strike, effective_defense, frenzied};
use crate::effects::{DamageRoll, Status};
use crate::geometry::{Field, Hex, HexSet};
use crate::hero;
use crate::movement::fits;
use crate::obstacles;
use crate::siege;
use crate::state::{BattleState, Stack, StackId, StackTraits};

/// The pathfinder's range (0x422B20 asks 0x4B2DA0 for 127 steps).
const AI_RANGE: u32 = 127;

/// The most turns the score divides by (0x41E190: more count as 5).
const MAX_TURNS: i32 = 5;

/// A cost no position has.
const UNREACHED: u32 = u32::MAX;

/// The cost buckets: every cost up to the range.
const BUCKETS: usize = AI_RANGE as usize + 1;

/// The queue's entries: every position at most once.
const ENTRIES: usize = h3_content_battle::MAX_FIELD_HEXES + 1;

/// No entry.
const NO_ENTRY: u16 = u16::MAX;

/// The worth of every creature in desperate mode (0x442FD0: 1000.0).
const DESPERATE_WORTH: u64 = 1000;

/// The computer's battle context for a side (0x435EC0): the least attack and defense bonus of
/// the stacks on the field, and whether the side is desperate (BAT-SIE-023).
#[derive(Debug, Clone, Copy)]
pub(crate) struct Context {
    pub(crate) min_attack: i64,
    pub(crate) min_defense: i64,
    pub(crate) desperate: bool,
}

/// The value `x` an `_ftol` gives: truncated toward 0; one outside `i32` is `i32::MIN` (the x87's
/// integer indefinite).
fn ftol(x: X87) -> i32 {
    i32::try_from(x.trunc()).unwrap_or(i32::MIN)
}

/// An integer as `fild` loads it and a store keeps it: exact up to 2^53.
fn double(n: u64) -> X87 {
    X87::int(n).round(DOUBLE_BITS)
}

/// The double `1 + 0.05 · x` of an attack or defense difference `x`, each step rounded to the
/// double (0x442A50): `fild`, `fmul 0.05`, `fadd 1.0`. Below 1 for a negative `x`; 0 from −20 on,
/// which no stack reaches (Bloodlust's +6 is the largest gap between a stack's attacks).
pub(crate) fn advantage(x: i64) -> X87 {
    let rate = X87::ratio(1, 20, DOUBLE_BITS).unwrap_or(X87::ZERO);
    let step = X87::int(x.unsigned_abs()).times(rate);
    if x >= 0 {
        X87::ONE.plus(step)
    } else {
        X87::ONE.minus(step)
    }
}

/// The stage-3 factor of a hit on `s` (0x443D90): its Air Shield for a shot, its Shield
/// otherwise, the half while petrified, the Armorer of its controller's hero.
fn protection(state: &BattleState, s: &Stack, shot: bool) -> X87 {
    let mut inputs = Strike::new(0, 0);
    inputs.protection_percent = u16::from(if shot {
        s.spells.ranged_protection
    } else {
        s.spells.melee_protection
    });
    inputs.petrified = s.spells.status.contains(Status::PETRIFIED);
    inputs.hero_reduction = hero::damage(&state.heroes, s.controller()).reduction;
    inputs.protection()
}

/// The attack of `s` against no target (0x442410): its attack with Precision for a shot or
/// Bloodlust otherwise, no Slayer, Frenzy's share of its own defense.
fn attack(ruleset: &BattleRuleset, s: &Stack, shot: bool) -> i64 {
    let mut attack = s.attack_now(shot);
    if s.spells.frenzy > 0 {
        let staged = ruleset
            .quirks()
            .is_enabled(h3_content::SINGLE_PRECISION_RATES);
        let defense = effective_defense(s, ruleset.battle());
        attack = frenzied(attack, defense, s.spells.frenzy, staged);
    }
    i64::from(attack)
}

/// The defense of `s` against no striker (0x442590 with flag 1): 0 while frenzied, else its
/// defense with the Defend bonus less the moat's penalty.
fn defense(state: &BattleState, ruleset: &BattleRuleset, s: &Stack) -> i64 {
    if s.spells.frenzy > 0 {
        0
    } else {
        i64::from(effective_defense(s, ruleset.battle()))
            - i64::from(siege::defense_penalty(state, s))
    }
}

/// The base attack and defense of `s`'s creature (the table's +0x54, +0x58).
fn base(ruleset: &BattleRuleset, s: &Stack) -> (i64, i64) {
    ruleset
        .creature(s.creature)
        .map_or((0, 0), |c| (i64::from(c.attack), i64::from(c.defense)))
}

/// Whether `s` is the Ballista: a shooting war machine that is neither the Catapult nor a turret.
fn is_ballista(s: &Stack) -> bool {
    s.has(StackTraits::WAR_MACHINE)
        && s.has(StackTraits::SHOOTER)
        && s.slot != siege::CATAPULT_SLOT
        && !s.is_turret()
}

/// The shot flag of `id` for the computer: the Ballista and a turret always, a stack that can
/// shoot now (0x442E60, 0x442A50).
fn shoots(state: &BattleState, field: Field, id: StackId) -> bool {
    let s = state.at(id);
    is_ballista(s) || s.is_turret() || can_shoot(state, field, id)
}

/// The total hit points of `s` for the computer: 1 for a clone (0x442A50, 0x443080).
fn hit_points(s: &Stack) -> u64 {
    if s.has(StackTraits::CLONE) {
        1
    } else {
        s.total_hp()
    }
}

/// The computer's worth of one creature of stack `id` (0x442A50; BAT-SIE-021), a double:
/// `sqrt(offense · defense) · Fight Value`, for a war machine or a created stack times its side's
/// other hit points over its own and theirs.
// rule: BAT-SIE-021
pub(crate) fn worth(
    state: &BattleState,
    (ruleset, field): (&BattleRuleset, Field),
    ctx: &Context,
    id: StackId,
    shot: bool,
) -> X87 {
    let s = state.at(id);
    let (base_attack, base_defense) = base(ruleset, s);
    let off = attack(ruleset, s, shot) - base_attack - ctx.min_attack;
    let def = defense(state, ruleset, s) - base_defense - ctx.min_defense;
    let defense = advantage(def).times(protection(state, s, shot));
    // The shot flag stays for the Ballista and a turret, else only while the stack can shoot.
    let shot = shot && (is_ballista(s) || s.is_turret() || can_shoot(state, field, id));
    let mut offense = advantage(off);
    if !shot && s.has(StackTraits::SHOOTER) {
        offense = offense.times(X87::ratio(1, 2, DOUBLE_BITS).unwrap_or(X87::ZERO));
    }
    // stub: BAT-STUB-3 — a commanded Ballista's Artillery factors (0x63B7F0) need the hero's level.
    let (min, max) = (u64::from(s.damage_min), u64::from(s.damage_max));
    let roll = match s.spells.damage {
        DamageRoll::Random => None,
        DamageRoll::Maximum(bonus) => Some(max + u64::from(bonus)),
        DamageRoll::Minimum(penalty) => Some(min.saturating_sub(u64::from(penalty)).max(1)),
    };
    if let Some(n) = roll {
        let mean = double(min + max).over(X87::int(2)).unwrap_or(X87::ZERO);
        let ratio = double(n).over(mean).unwrap_or(X87::ZERO);
        offense = ratio.times(offense);
    }
    if shot && s.has(StackTraits::STRIKES_TWICE) {
        offense = offense.plus(offense);
    }
    let fight = ruleset
        .creature_info(s.creature)
        .map_or(0, |info| info.fight_value);
    let mut v = offense.times(defense).sqrt().times(double(fight.into()));
    if s.has(StackTraits::WAR_MACHINE) || s.has(StackTraits::SUMMONED) {
        // A living stack has hit points: the original's 0.1 for none never arises here.
        let own = hit_points(s);
        let others: u64 = state
            .stacks
            .iter()
            .filter(|t| {
                t.side == s.side
                    && t.is_alive()
                    && !t.has(StackTraits::WAR_MACHINE)
                    && !t.has(StackTraits::SUMMONED)
            })
            .map(Stack::total_hp)
            .sum();
        v = double(others)
            .times(v)
            .over(double(own + others))
            .unwrap_or(X87::ZERO);
    }
    v
}

/// The value of `damage` dealt to stack `id` at worth `v` (0x442FD0): its kills times the worth,
/// the top creature's lost hit points counted when the damage finishes it; a clone's creatures
/// times the worth over 5.
pub(crate) fn kill_value(s: &Stack, damage: i64, v: X87) -> i32 {
    if s.has(StackTraits::CLONE) {
        return ftol(
            double(s.count.into())
                .times(v)
                .over(X87::int(5))
                .unwrap_or(X87::ZERO),
        );
    }
    let hp = i64::from(s.max_hp.max(1));
    let lost = i64::from(s.max_hp - s.top_hp);
    let dealt = if damage % hp + lost >= hp {
        damage + lost
    } else {
        damage
    };
    ftol(
        double(dealt.unsigned_abs())
            .times(v)
            .over(double(hp.unsigned_abs()))
            .unwrap_or(X87::ZERO),
    )
}

/// A side's strength for the computer (0x41EAC0 with flag 1 over 0x442E60): over its living
/// stacks but the turrets, each stack's worth (with its own shot flag) times its total hit points
/// over its hit points per creature, truncated (a clone's creatures times its worth over 5);
/// summed as the original's 32-bit integers.
fn strength(
    state: &BattleState,
    (ruleset, field): (&BattleRuleset, Field),
    ctx: &Context,
    side: BattleSide,
) -> i32 {
    let mut sum = 0i32;
    for (i, s) in (0u8..).zip(state.stacks.iter()) {
        if s.side != side || !s.is_alive() || s.is_turret() {
            continue;
        }
        let id = StackId(i);
        let v = worth(state, (ruleset, field), ctx, id, shoots(state, field, id));
        let term = if s.has(StackTraits::CLONE) {
            kill_value(s, 0, v)
        } else {
            ftol(
                double(s.total_hp())
                    .times(v)
                    .over(double(s.max_hp.max(1).into()))
                    .unwrap_or(X87::ZERO),
            )
        };
        sum = sum.wrapping_add(term);
    }
    sum
}

/// The computer's context for `side` (0x435EC0; BAT-SIE-023): the least attack and defense
/// bonus over the living stacks of both sides but the turrets, and its desperate mode: twice its
/// strength below the other side's, with the game's difficulty above Easy or a human player.
// rule: BAT-SIE-023
pub(crate) fn context(
    state: &BattleState,
    ruleset: &BattleRuleset,
    field: Field,
    side: BattleSide,
) -> Context {
    let mut least: Option<(i64, i64)> = None;
    for (i, s) in (0u8..).zip(state.stacks.iter()) {
        if !s.is_alive() || s.is_turret() {
            continue;
        }
        let (base_attack, base_defense) = base(ruleset, s);
        let a = attack(ruleset, s, can_shoot(state, field, StackId(i))) - base_attack;
        let d = defense(state, ruleset, s) - base_defense;
        least = Some(least.map_or((a, d), |(la, ld)| (la.min(a), ld.min(d))));
    }
    let (min_attack, min_defense) = least.unwrap_or_default();
    let mut ctx = Context {
        min_attack,
        min_defense,
        desperate: false,
    };
    let own = strength(state, (ruleset, field), &ctx, side);
    let other = strength(state, (ruleset, field), &ctx, side.opponent());
    ctx.desperate =
        own.wrapping_mul(2) < other && (state.difficulty > 0 || state.human[side.index()]);
    ctx
}

/// The hexes where a walk of `mover` stops: the moat and the pits both sides see (BAT-SIE-004,
/// BAT-MAG-031: a hidden pit must not change a choice both sides are told), not its own hexes.
fn stops(state: &BattleState, field: Field, mover: &Stack) -> HexSet {
    let own = mover
        .footprint(field)
        .hexes()
        .fold(HexSet::new(), |mut own, h| {
            own.insert(h);
            own
        });
    obstacles::public_pits(state)
        .union(siege::stops(state, mover.controller()))
        .without(own)
}

/// Whether `mover` at the position `hex` covers a hex of `stops`.
fn on_stop(field: Field, mover: &Stack, hex: Hex, stops: HexSet) -> bool {
    field
        .footprint(hex, mover.is_two_hex(), mover.side)
        .is_some_and(|fp| fp.hexes().any(|h| stops.contains(h)))
}

/// Dial's buckets by cost (Dijkstra for small integer costs, without allocation).
struct Buckets {
    head: [u16; BUCKETS],
    entries: [(Hex, u16); ENTRIES],
    len: usize,
}

impl Buckets {
    /// Queues `hex` at cost `c`, at most the range. A position's first cost is its least (a
    /// step costs 1, a stop's entry the rest of the turn, which never falls as the cost grows),
    /// so each is queued once: at most the field's positions.
    fn push(&mut self, c: u32, hex: Hex) {
        let c = usize::try_from(c).unwrap_or(0);
        self.entries[self.len] = (hex, self.head[c]);
        self.head[c] = u16::try_from(self.len).unwrap_or(NO_ENTRY);
        self.len += 1;
    }

    /// Takes a position of cost `c`, if one is queued.
    fn pop(&mut self, c: u32) -> Option<Hex> {
        let c = usize::try_from(c).unwrap_or(0);
        let &(hex, next) = self.entries.get(usize::from(self.head[c]))?;
        self.head[c] = next;
        Some(hex)
    }
}

/// Where `mover` (stack `id`) can stand for the computer and at what cost (0x4B3400 with range
/// 127, BAT-SIE-022), with its walk's stops: walkers step between free positions, entering a
/// stop costs the rest of the turn's move and they go on from it; flyers (and teleporters)
/// reach every free position at its hex distance; a bound stack stays. Costs above the range
/// are not reached.
fn costs(state: &BattleState, field: Field, id: StackId) -> ([u32; 256], HexSet) {
    let mover = state.at(id);
    let blocked = state.blocked_for(field, id);
    let speed = u32::from(mover.speed_now());
    let range = if binds::is_bound(state, id) {
        0
    } else {
        AI_RANGE
    };
    let mut cost = [UNREACHED; 256];
    if mover.has(StackTraits::FLIES) {
        for (h, c) in (0u8..).zip(cost.iter_mut().take(field.hex_count())) {
            let d = field.distance(mover.hex, Hex(h));
            if d <= range && (d == 0 || fits(field, mover, Hex(h), blocked)) {
                *c = d;
            }
        }
        return (cost, HexSet::new());
    }
    let stops = stops(state, field, mover);
    let mut queue = Buckets {
        head: [NO_ENTRY; BUCKETS],
        entries: [(Hex(0), NO_ENTRY); ENTRIES],
        len: 0,
    };
    cost[mover.hex.index()] = 0;
    queue.push(0, mover.hex);
    // A stack that walks has a speed of at least 1 (a slowed one too, BAT-MAG-011).
    let speed = speed.max(1);
    for c in 0..=range {
        while let Some(hex) = queue.pop(c) {
            for to in field.neighbors(hex).into_iter().flatten() {
                if !fits(field, mover, to, blocked) {
                    continue;
                }
                let step = if on_stop(field, mover, to, stops) {
                    speed - c % speed
                } else {
                    1
                };
                if c + step <= range && c + step < cost[to.index()] {
                    cost[to.index()] = c + step;
                    queue.push(c + step, to);
                }
            }
        }
    }
    (cost, stops)
}

/// The turns attacker stack `id` needs before it can attack, for the computer (0x422B20,
/// 0x448840, 0x448BD0; BAT-SIE-022): 1 while it can shoot; else from the least cost of a
/// position next to a defender (a full move more from a stop), or of a dead defender's place
/// within its speed: that cost over its speed, rounded up, at least 1. `None` for a stack the
/// computer gives no target (a war machine but the Ballista, a hypnotized or berserk stack) or
/// that reaches none within the range.
pub(crate) fn turns(state: &BattleState, field: Field, id: StackId) -> Option<u32> {
    let me = state.at(id);
    let status = me.spells.status;
    if (me.has(StackTraits::WAR_MACHINE) && !is_ballista(me))
        || status.contains(Status::HYPNOTIZED)
        || status.contains(Status::BERSERK)
    {
        return None;
    }
    if shoots(state, field, id) {
        return Some(1);
    }
    let (cost, stops) = costs(state, field, id);
    let speed = u32::from(me.speed_now());
    let reached = |hex: Hex| cost.get(hex.index()).copied().filter(|&c| c != UNREACHED);
    let mut least: Option<u32> = None;
    for t in state.stacks.iter() {
        if t.side == me.side || t.is_turret() {
            continue;
        }
        let found = if t.is_alive() {
            field
                .attack_positions(me.is_two_hex(), me.side, t.footprint(field))
                .into_iter()
                .flatten()
                .filter_map(|p| {
                    let c = reached(p)?;
                    // rule: BAT-SIE-022
                    let c = c + if stops.contains(p) { speed } else { 0 };
                    (c <= AI_RANGE).then_some(c)
                })
                .min()
        } else {
            // A dead defender's place counts as a target where the stack can stand this turn
            // (the pathfinder's marks, 0x4B2E69, read by 0x422CC5).
            reached(t.hex).filter(|&c| c <= speed)
        };
        if let Some(c) = found {
            least = Some(least.map_or(c, |l| l.min(c)));
        }
    }
    // A stack that does not shoot has a speed of at least 1 (the original records no target
    // at a distance for one of speed 0, 0x422D1C, and counts its turns as 1, 0x448CA6).
    least.map(|c| c.div_ceil(speed.max(1)).max(1))
}

/// The score of attacker stack `id` for a turret of damage `mean` (the turret's `3n`; 0x41E190;
/// BAT-SIE-020): the value of the damage through its protection at worth `v` (in desperate mode
/// 1000 per creature), divided by its turns (5 when disabled, a tent or a cart, without a target
/// or above 5).
// rule: BAT-SIE-020
fn score(
    state: &BattleState,
    (ruleset, field): (&BattleRuleset, Field),
    ctx: &Context,
    (id, mean): (StackId, u64),
    desperate: bool,
) -> i32 {
    let s = state.at(id);
    let damage = i64::from(ftol(double(mean).times(protection(state, s, true))));
    let v = worth(state, (ruleset, field), ctx, id, true);
    let v = if desperate && !s.has(StackTraits::CLONE) {
        X87::int(DESPERATE_WORTH)
    } else {
        v
    };
    let value = kill_value(s, damage, v);
    let disabled = s.spells.status.contains(Status::NO_TURNS)
        || s.has(StackTraits::FIRST_AID)
        || s.has(StackTraits::AMMO_SUPPLY);
    let divisor = if disabled {
        MAX_TURNS
    } else {
        turns(state, field, id).map_or(MAX_TURNS, |t| {
            i32::try_from(t).unwrap_or(MAX_TURNS).min(MAX_TURNS)
        })
    };
    value / divisor
}

/// The attacker stack an automatic turret shoots (0x41E190; BAT-SIE-020): of the attacker's
/// living stacks, whoever controls them, the best [`score`], from 0 and ties to the later stack;
/// in desperate mode (BAT-SIE-023) first with every creature worth 1000, and again with the real
/// worth if no score is above 0. `None` without a living attacker stack.
// rule: BAT-SIE-020, BAT-SIE-023
pub(crate) fn turret_target(
    state: &BattleState,
    ruleset: &BattleRuleset,
    field: Field,
    turret: StackId,
) -> Option<StackId> {
    let me = state.at(turret);
    let ctx = context(state, ruleset, field, me.side);
    let mean = 3 * u64::from(me.count);
    let mut best: (i32, Option<StackId>) = (0, None);
    let passes: &[bool] = if ctx.desperate {
        &[true, false]
    } else {
        &[false]
    };
    for &desperate in passes {
        for (i, s) in (0u8..).zip(state.stacks.iter()) {
            if s.side == me.side || !s.is_alive() {
                continue;
            }
            let id = StackId(i);
            let v = score(state, (ruleset, field), &ctx, (id, mean), desperate);
            if v >= best.0 {
                best = (v, Some(id));
            }
        }
        if best.1.is_some() && best.0 > 0 {
            break;
        }
    }
    best.1
}
