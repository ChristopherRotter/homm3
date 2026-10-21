//! The computer's aim for the automatic turrets (BAT-SIE-020 to BAT-SIE-023) and the automatic
//! Ballista (BAT-WAR-008): which stack an arrow tower or a Ballista without Artillery shoots when it
//! acts on its own. The original's turret has a routine of its own (0x41E190,
//! `docs/formats/installation.md` §7.31): every attacker stack scores the kills of the turret's
//! mean damage times the computer's worth of one of its creatures (0x442A50), divided by the turns
//! it needs to attack (0x448BD0); the best score wins, a tie the later stack. The Ballista takes the
//! computer's shooter (0x41F060, §7.33): every stack it may shoot scores the kills of the
//! computer's estimate of its shot times the same worth, divided by its turns, by 5 or by 10; a
//! stack that keeps its turns first, then the best score, a tie the later stack. Their numbers are
//! the original's x87 arithmetic in every profile: they decide a choice, not an amount.
//!
//! The computer's shooter (0x41F060) is also the original's battle AI's (`h3-battle-ai`, E23):
//! [`shot`] takes the target the same way for every computer shooter, an area shooter by its
//! value at a hex ([`area_value`], E23 T2b), and asks its caller whether a wall shooter batters
//! the walls instead (the Cyclops, 0x41EEA0: the policy's, with its draw).

use core::cell::Cell;

use h3_content::{CreatureFlag, ORIGINAL_AI_SLIPS, SpellId};
use h3_content_battle::{
    BattleRuleset, BattleSpell, MAX_MASTERY, SpellAiKind, SpellEffect, SpellTarget, TimedChange,
    WallKind,
};
use h3_contracts::{BattleSide, FortLevel};
use h3_kernel::arith::{DOUBLE_BITS, SINGLE_BITS, X87};

use crate::binds;
use crate::combat::{can_shoot, can_shoot_without, melee_damage, shot_damage, spares};
use crate::damage::{Strike, effective_defense, frenzied};
use crate::effects::{DamageRoll, Status};
use crate::geometry::{Field, Hex, HexSet};
use crate::hero;
use crate::magic;
use crate::movement::fits;
use crate::obstacles;
use crate::siege;
use crate::state::{BattleState, STACK_CAPACITY, Stack, StackId, StackTraits};

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

/// The factor of a Ballista's offense by its hero's Artillery level, none to Expert (0x63B7F0:
/// 1.0, 1.5, 1.5, 2.0; above Basic the offense is doubled first).
const ARTILLERY: [(u32, u32); 4] = [(1, 1), (3, 2), (3, 2), (2, 1)];

/// The divisor of a shot's value at a stack that loses its turns (0x435CB0).
const DISABLED: i32 = 10;

/// The computer's battle context for a side (0x435EC0): the least attack and defense bonus of
/// the stacks on the field, whether the side is desperate (BAT-SIE-023), and the strengths of
/// the side and of the other side over all their records (+0xC, +0x10).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Context {
    pub(crate) side: BattleSide,
    pub(crate) min_attack: i64,
    pub(crate) min_defense: i64,
    pub(crate) desperate: bool,
    pub(crate) strengths: (i32, i32),
}

impl Context {
    /// The side's own strength and the other side's (+0xC, +0x10: 0x41EAC0 with flag 1 over
    /// their living stacks but the turrets, as 32-bit sums).
    pub const fn strengths(&self) -> (i32, i32) {
        self.strengths
    }

    /// Whether the side is desperate (+8; BAT-SIE-023): its values take every creature at 1000.
    pub const fn desperate(&self) -> bool {
        self.desperate
    }

    /// The context with its least attack bonus also as its least defense bonus: the Fire Wall's
    /// and the moat's hex scores pass the one for both to the kill value (0x42155B, 0x4215EC; the
    /// walker's, E23 T3b: with quirk BAT-QUIRK-25 the original's slip 19, else the context as it
    /// is).
    #[must_use]
    pub const fn attack_for_both(self) -> Self {
        Self {
            min_defense: self.min_attack,
            ..self
        }
    }
}

/// The value `x` an `_ftol` gives its callers (0x6179D4): truncated toward 0 into 64 bits
/// (`fistp qword`; from 2^63 on the integer indefinite, 2^63), of which they keep the low 32 bits.
pub(crate) fn ftol(x: X87) -> i32 {
    let t = x.trunc();
    let low = if t >= 1 << 63 { 0 } else { t & 0xFFFF_FFFF };
    i32::from_ne_bytes(u32::try_from(low).unwrap_or(0).to_ne_bytes())
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
        && !s.is_catapult()
        && !s.is_turret()
}

/// The shot flag of `id` for the computer: the Ballista and a turret always, a stack that can
/// shoot now (0x442E60, 0x442A50).
fn shoots(state: &BattleState, field: Field, id: StackId) -> bool {
    let s = state.at(id);
    is_ballista(s) || s.is_turret() || can_shoot(state, field, id)
}

/// The damage of one creature of `s` as the original's stack record holds it (+0xD0, +0xD4), which
/// the computer reads. A war machine's stack keeps its creature's range times its side's hero's
/// attack + 1 (BAT-WAR-005), the original only its creature's range (with the hero's changes,
/// BAT-HER-005) and multiplies in a strike's base damage (0x4431AC): the hero's share comes off.
fn damage(state: &BattleState, ruleset: &BattleRuleset, s: &Stack) -> (u64, u64) {
    let (min, max) = (u64::from(s.damage_min), u64::from(s.damage_max));
    if !s.has(StackTraits::WAR_MACHINE) {
        return (min, max);
    }
    let attack = state.heroes[s.side.index()].map_or(0, |h| u64::from(h.view.attack));
    let (own_min, own_max) = ruleset.creature(s.creature).map_or((0, 0), |c| {
        (u64::from(c.damage_min), u64::from(c.damage_max))
    });
    (
        min.saturating_sub(own_min * attack),
        max.saturating_sub(own_max * attack),
    )
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
/// `sqrt(offense · defense) · Fight Value`, the Ballista's offense grown by its hero's Artillery,
/// for a war machine or a created stack times its side's other hit points over its own and theirs.
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
    if is_ballista(s) {
        // The Ballista's offense by its hero's Artillery level (0x442C0F; 0 without a hero or the
        // skill): doubled above Basic, then times 0x63B7F0's factor, each a double.
        let level = hero::ballista(&state.heroes, s.controller()).map_or(0, |c| c.level);
        if level > 1 {
            offense = offense.plus(offense);
        }
        let (n, d) = ARTILLERY.get(usize::from(level)).copied().unwrap_or((1, 1));
        offense = offense.times(X87::ratio(n, d, DOUBLE_BITS).unwrap_or(X87::ONE));
    }
    let (min, max) = damage(state, ruleset, s);
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
        // A living stack has hit points: the original's 0.1 for none never arises here. At most
        // 42 stacks below 2^64 hit points each: below 2^70, saturated into the doubles.
        let own = u128::from(hit_points(s));
        let others: u128 = state
            .stacks
            .iter()
            .filter(|t| {
                t.side == s.side
                    && t.is_alive()
                    && !t.has(StackTraits::WAR_MACHINE)
                    && !t.has(StackTraits::SUMMONED)
            })
            .map(|t| u128::from(t.total_hp()))
            .sum();
        let wide = |n: u128| double(u64::try_from(n).unwrap_or(u64::MAX));
        v = wide(others)
            .times(v)
            .over(wide(own + others))
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
        damage.saturating_add(lost)
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

/// The strength of stack `id` for the computer (0x442E60): its worth (with its own shot flag)
/// times its total hit points over its hit points per creature, truncated (a clone's creatures
/// times its worth over 5).
pub(crate) fn stack_strength(
    state: &BattleState,
    (ruleset, field): (&BattleRuleset, Field),
    ctx: &Context,
    id: StackId,
) -> i32 {
    let s = state.at(id);
    let v = worth(state, (ruleset, field), ctx, id, shoots(state, field, id));
    if s.has(StackTraits::CLONE) {
        kill_value(s, 0, v)
    } else {
        ftol(
            double(s.total_hp())
                .times(v)
                .over(double(s.max_hp.max(1).into()))
                .unwrap_or(X87::ZERO),
        )
    }
}

/// A side's strength for the computer (0x41EAC0 with flag 1 over 0x442E60): over its living
/// stacks but the turrets, each stack's [`stack_strength`], summed as the original's 32-bit
/// integers.
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
        sum = sum.wrapping_add(stack_strength(state, (ruleset, field), ctx, StackId(i)));
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
        side,
        min_attack,
        min_defense,
        desperate: false,
        strengths: (0, 0),
    };
    let own = strength(state, (ruleset, field), &ctx, side);
    let other = strength(state, (ruleset, field), &ctx, side.opponent());
    ctx.strengths = (own, other);
    ctx.desperate =
        own.wrapping_mul(2) < other && (state.difficulty > 0 || state.human[side.index()]);
    ctx
}

/// The hexes `mover` covers at the position `hex` (none off the field).
fn covered(field: Field, mover: &Stack, hex: Hex) -> HexSet {
    field
        .footprint(hex, mover.is_two_hex(), mover.side)
        .map_or(HexSet::new(), |fp| {
            fp.hexes().fold(HexSet::new(), |mut set, h| {
                set.insert(h);
                set
            })
        })
}

/// The hexes where a walk of `mover` stops: the moat and the pits both sides see (BAT-SIE-004,
/// BAT-MAG-031: a hidden pit must not change a choice both sides are told), not its own hexes.
fn stops(state: &BattleState, field: Field, mover: &Stack) -> HexSet {
    obstacles::public_pits(state)
        .union(siege::stops(state, mover.controller()))
        .without(covered(field, mover, mover.hex))
}

/// Whether the computer's pathfinder keeps `mover` out of its stops (0x4B3400's prologue,
/// 0x4B3420–0x4B34B2, read at 0x4B375C; BAT-SIE-022, BAT-MAG-025): at a town, for a stack whose
/// controller's player is no human (0x474BF0), with at most 4 times the town's moat damage in hit
/// points (the computer's, 1 for a clone: 0x443080), or at most 40 times when the gate is not
/// closed or the path fills the defender's `records` (the aim's, its own side; a berserk turn's
/// paths fill none, side -1: 0x445582, 0x42232C). The damage is the town's entry in the table
/// (0x63BD18) whatever its moat does; the original's gate is closed from the setup on (3), at a
/// town without a fort too.
// rule: BAT-SIE-022, BAT-MAG-025
pub(crate) fn avoids_stops(
    state: &BattleState,
    ruleset: &BattleRuleset,
    mover: &Stack,
    records: Option<BattleSide>,
) -> bool {
    !state.human[mover.controller().index()] && fears_moat(state, ruleset, mover, records)
}

/// Whether the moat's damage keeps `mover` out of its stops when the computer plays it
/// ([`avoids_stops`] without the player's test: a stack the original's battle AI moves is one
/// it plays, 0x474BF0; E23 T3a).
// rule: BAT-SIE-022
pub(crate) fn fears_moat(
    state: &BattleState,
    ruleset: &BattleRuleset,
    mover: &Stack,
    records: Option<BattleSide>,
) -> bool {
    let Some(town) = state.siege else {
        return false;
    };
    let damage = ruleset
        .siege(town.faction)
        .map_or(0, |record| u64::from(record.moat_damage));
    let closed = town.fort == FortLevel::None || !town.gate_open;
    let hp = hit_points(mover);
    hp <= 4 * damage || ((records == Some(BattleSide::Defender) || !closed) && hp <= 40 * damage)
}

/// Whether `mover` stepping from the position `from` to `to` enters a hex of `stops`: one it
/// covers at `to` and not at `from` (0x4B3701–0x4B3725: a two-hex walker's rear on the stop its
/// front leaves is no new stop).
// rule: BAT-SIE-022
fn on_stop(field: Field, mover: &Stack, (from, to): (Hex, Hex), stops: HexSet) -> bool {
    covered(field, mover, to)
        .without(covered(field, mover, from))
        .intersects(stops)
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
/// stop costs the rest of the turn's move and they go on from it, unless the computer keeps them
/// out of their stops ([`avoids_stops`]); flyers (and teleporters) reach every free position at
/// its hex distance; a bound stack stays. Costs above the range are not reached.
fn costs(
    state: &BattleState,
    (ruleset, field): (&BattleRuleset, Field),
    id: StackId,
) -> ([u32; 256], HexSet) {
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
    let avoids = avoids_stops(state, ruleset, mover, Some(mover.side));
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
                let stop = on_stop(field, mover, (hex, to), stops);
                if stop && avoids {
                    continue;
                }
                let step = if stop { speed - c % speed } else { 1 };
                if c + step <= range && c + step < cost[to.index()] {
                    cost[to.index()] = c + step;
                    queue.push(c + step, to);
                }
            }
        }
    }
    (cost, stops)
}

/// Whether stack `id` is blind or turned to stone for more than one round (0x422BD0 to 0x422BE9:
/// the durations of Blind and of the Medusas' stone above 1): the computer records no enemy for
/// it. Only a stack that loses its turns can be.
pub(crate) fn kept_out(state: &BattleState, ruleset: &BattleRuleset, id: StackId) -> bool {
    state.at(id).spells.status.contains(Status::NO_TURNS)
        && state.effects.of(id).iter().any(|e| {
            e.rounds.is_none_or(|r| r > 1)
                && ruleset.spell(e.spell).is_some_and(|sp| {
                    sp.magic
                        .effects
                        .iter()
                        .filter_map(|c| c.timed())
                        .any(|c| matches!(c, TimedChange::Blind | TimedChange::Petrify))
                })
        })
}

/// The turns attacker stack `id` needs before it can attack, for the computer (0x422B20,
/// 0x448840, 0x448BD0; BAT-SIE-022): 1 while it can shoot; else from the least cost of a
/// position next to a defender (a full move more from a stop), or of a dead defender's place
/// within its speed: that cost over its speed, rounded up, at least 1. `None` for a stack the
/// computer gives no target (a war machine but the Ballista, a hypnotized or berserk stack, one
/// blind or turned to stone for more than a round) or that reaches none within the range.
pub(crate) fn turns(
    state: &BattleState,
    (ruleset, field): (&BattleRuleset, Field),
    id: StackId,
) -> Option<u32> {
    let me = state.at(id);
    let status = me.spells.status;
    if (me.has(StackTraits::WAR_MACHINE) && !is_ballista(me))
        || status.contains(Status::HYPNOTIZED)
        || status.contains(Status::BERSERK)
        || kept_out(state, ruleset, id)
    {
        return None;
    }
    if shoots(state, field, id) {
        return Some(1);
    }
    let (cost, stops) = costs(state, (ruleset, field), id);
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

/// The [`turns`] of the stacks a shooter's choice values, each computed at most once per choice:
/// the original builds its threat records (0x422B20) once before the shooter's choice (0x41F060),
/// and the shot values of every target, area and desperate mode's second choice read them
/// (ADR-0027 point 11: an optimisation that changes no choice). Computed when first asked, so a
/// value that needs no turns (on Easy, at a disabled stack) runs no search.
pub(crate) struct Threats([Cell<Option<Option<u32>>>; STACK_CAPACITY]);

impl Threats {
    /// No turns computed yet.
    pub(crate) fn new() -> Self {
        Self([const { Cell::new(None) }; STACK_CAPACITY])
    }

    /// The [`turns`] of stack `id`, computed the first time.
    fn turns(
        &self,
        state: &BattleState,
        (ruleset, field): (&BattleRuleset, Field),
        id: StackId,
    ) -> Option<u32> {
        let cell = &self.0[id.index()];
        cell.get().unwrap_or_else(|| {
            let t = turns(state, (ruleset, field), id);
            cell.set(Some(t));
            t
        })
    }
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
        turns(state, (ruleset, field), id).map_or(MAX_TURNS, |t| {
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

/// The computer's estimate of the damage a shot of `shooter` deals to `target` (0x4359B0,
/// 0x442780; BAT-WAR-008): [`damage_estimate`] of the shooter's creatures by its hit points
/// (rounded up), with the shot flag, at most the target's hit points.
pub(crate) fn estimate(
    state: &BattleState,
    ruleset: &BattleRuleset,
    (shooter, target): (StackId, StackId),
) -> u64 {
    let s = state.at(shooter);
    let count = hit_points(s).div_ceil(u64::from(s.max_hp.max(1)));
    damage_estimate(state, ruleset, (shooter, target), count, true, 0)
}

/// The computer's damage of `count` creatures of `attacker` on `target` (0x442780; BAT-WAR-008,
/// BAT-AI-005): `count` times one creature's damage as its record holds it ([`damage`]: a
/// Ballista's 2–3 without its hero's multiplier; the maximum and Bless's bonus while blessed, the
/// minimum less Curse's penalty but at least 1 while cursed, else the mean, a double), truncated,
/// through every stage of a real shot (`shot`) or of a melee strike after `jousting` hexes, without
/// their draws ([`shot_damage`], [`melee_damage`]); at least 1 (a count of 0 too); a shot doubled
/// for a creature that strikes twice; at most the target's hit points (1 for a clone): the
/// original's cap flag, which every caller so far sets (the spell context's uncapped strike: E23
/// T4).
pub(crate) fn damage_estimate(
    state: &BattleState,
    ruleset: &BattleRuleset,
    (attacker, target): (StackId, StackId),
    count: u64,
    shot: bool,
    jousting: u8,
) -> u64 {
    let s = state.at(attacker);
    let (min, max) = damage(state, ruleset, s);
    let each = match s.spells.damage {
        DamageRoll::Maximum(bonus) => double(max + u64::from(bonus)),
        DamageRoll::Minimum(penalty) => double(min.saturating_sub(u64::from(penalty)).max(1)),
        DamageRoll::Random => double(min + max).over(X87::int(2)).unwrap_or(X87::ZERO),
    };
    let base = u64::try_from(ftol(double(count).times(each))).unwrap_or(0);
    let mut damage = if shot {
        shot_damage(state, ruleset, (attacker, target), base)
    } else {
        melee_damage(state, ruleset, (attacker, target), (base, jousting))
    }
    .max(1);
    if shot && s.has(StackTraits::STRIKES_TWICE) {
        damage = damage.saturating_mul(2);
    }
    damage.min(hit_points(state.at(target)))
}

/// The computer's estimate of what Fire Shield on `struck` reflects onto `striker` from a strike
/// estimated at `damage` on `struck` with `left` hit points (0x422440): the rule's reflection
/// ([`magic::reflection`]: none for a fire-immune striker or without a shield) of the lesser of
/// the two.
fn reflection_estimate(
    state: &BattleState,
    ruleset: &BattleRuleset,
    (striker, struck): (StackId, StackId),
    damage: u64,
    left: u64,
) -> u64 {
    magic::reflection(
        state,
        ruleset,
        (state.at(striker), struck),
        damage.min(left),
    )
}

/// The computer's estimate of a strike's exchange (0x4359B0 without the shot flag; BAT-AI-005):
/// `a` with `hit_points.0` strikes `t` with `hit_points.1` after `jousting` hexes, and the hit
/// points both have left. Its creatures by its hit points (rounded up) deal [`damage_estimate`]
/// to `t`, and Fire Shield's reflection ([`reflection_estimate`]) comes off `a`. Then, while both
/// live: `t`'s retaliation by its creatures left, unless `a`'s strikes are not retaliated, `t` is
/// turned to stone or has none left, or the game is on Easy for the context's computer side
/// (`side`), with its own reflection onto `t`; and while both still live, `a`'s second strike for
/// a creature that strikes twice, by its creatures left, without a run-up. Hit points never fall
/// below 0. (A shot's exchange, 0x435CB0's, is [`kill_shot_value`]'s.)
// rule: BAT-AI-005
fn exchange(
    state: &BattleState,
    ruleset: &BattleRuleset,
    side: BattleSide,
    ((a, mut a_hp), (t, mut t_hp)): ((StackId, u64), (StackId, u64)),
    jousting: u8,
) -> (u64, u64) {
    let (sa, st) = (state.at(a), state.at(t));
    let per = |s: &Stack, hp: u64| hp.div_ceil(u64::from(s.max_hp.max(1)));
    let dealt = damage_estimate(state, ruleset, (a, t), per(sa, a_hp), false, jousting);
    a_hp = a_hp.saturating_sub(reflection_estimate(state, ruleset, (a, t), dealt, t_hp));
    t_hp = t_hp.saturating_sub(dealt);
    if a_hp == 0 || t_hp == 0 {
        return (a_hp, t_hp);
    }
    let retaliates = !sa.has(StackTraits::NO_ENEMY_RETALIATION)
        && !st.spells.status.contains(Status::PETRIFIED)
        && st.retaliations_left > 0
        && (state.difficulty > 0 || state.human[side.index()]);
    if retaliates {
        let back = damage_estimate(state, ruleset, (t, a), per(st, t_hp), false, 0);
        t_hp = t_hp.saturating_sub(reflection_estimate(state, ruleset, (t, a), back, a_hp));
        a_hp = a_hp.saturating_sub(back);
        if a_hp == 0 || t_hp == 0 {
            return (a_hp, t_hp);
        }
    }
    if sa.has(StackTraits::STRIKES_TWICE) {
        let again = damage_estimate(state, ruleset, (a, t), per(sa, a_hp), false, 0);
        a_hp = a_hp.saturating_sub(reflection_estimate(state, ruleset, (a, t), again, t_hp));
        t_hp = t_hp.saturating_sub(again);
    }
    (a_hp, t_hp)
}

/// The value of `damage` to stack `id` for the computer (0x442FD0): [`kill_value`] at its worth
/// in the context's least bonuses (with the shot flag `shot`), or at 1000 per creature when
/// `desperate` (a clone keeps its worth).
pub(crate) fn damage_value(
    state: &BattleState,
    (ruleset, field): (&BattleRuleset, Field),
    ctx: &Context,
    (id, damage): (StackId, u64),
    (shot, desperate): (bool, bool),
) -> i32 {
    let s = state.at(id);
    let v = if desperate && !s.has(StackTraits::CLONE) {
        X87::int(DESPERATE_WORTH)
    } else {
        worth(state, (ruleset, field), ctx, id, shot)
    };
    kill_value(s, i64::try_from(damage).unwrap_or(i64::MAX), v)
}

/// The best damage of `caster`'s hero's area spells on stack `s` as the computer's area threat
/// estimates it (0x41F920's loop; E23 T3b): none while casting is forbidden (an anti-magic
/// garrison, the Orb of Inhibition on either hero, BAT-MAG-061), without a hero or a spell in its
/// book; else over the spells of its book aimed at a hex with an area and a damage effect (the
/// original's Frost Ring, Fireball, Inferno and Meteor Shower, 20 to 23 by number: the same set in
/// the packs) whose cost at its mastery (BAT-MAG-054, the battlefield's) its mana pays, the
/// largest `trunc(f32(d) × f)`: `d` the spell's value with the hero's bonuses and the stack's
/// reductions and protection (BAT-MAG-004, 0x5A7890), `f` the effect test's factor
/// ([`magic::effect_factor`], 0x5A8090), at 53 bits; at least 0. Whether the hero has cast this
/// round is not asked.
pub(crate) fn best_area_spell(
    state: &BattleState,
    ruleset: &BattleRuleset,
    caster: BattleSide,
    s: StackId,
) -> i32 {
    let Some(hero) = state.heroes[caster.index()]
        .as_ref()
        .filter(|_| !magic::casting_forbidden(state))
    else {
        return 0;
    };
    let target = state.at(s);
    hero.magic
        .book
        .iter()
        .filter_map(|id| ruleset.spell(id).map(|spell| (id, spell)))
        .filter(|(_, spell)| {
            spell.magic.target == SpellTarget::Hex
                && spell.magic.area.is_some()
                && spell.magic.effects.contains(&SpellEffect::Damage)
        })
        .filter_map(|(id, spell)| {
            let mastery = magic::mastery(state, ruleset, hero, id);
            let cost = crate::mana::cost(spell, mastery, hero.magic.cost_change);
            (cost <= hero.view.mana).then(|| {
                let value = magic::base_value(hero, spell, mastery);
                let (_, damage) = magic::spell_damage(
                    ruleset,
                    ((id, spell), Some(&hero.magic.bonuses)),
                    value,
                    (target.creature, &target.spells.school_protection),
                );
                let f = magic::effect_factor(state, ruleset, caster, (id, spell), s);
                ftol(X87::int(damage).round(SINGLE_BITS).times(f))
            })
        })
        .fold(0, i32::max)
}

/// The value curve of a hero's damage spell in the odds (the table 0x640398 read by 0x5270E0,
/// `installation.md` §7.39), 14 rows in thousandths, each the nearest double to its decimal: the
/// casts' factor φ, the threshold t (row 13's unread), the slope α and the intercept β.
const SPELL_CURVE: [[u32; 4]; 14] = [
    [0, 4800, 0, 4000],
    [830, 1557, 500, 1600],
    [1530, 752, 987, 842],
    [2110, 433, 1452, 492],
    [2590, 275, 1888, 303],
    [2990, 186, 2291, 192],
    [3330, 131, 2657, 124],
    [3600, 95, 2986, 81],
    [3840, 71, 3277, 53],
    [4030, 54, 3353, 35],
    [4190, 41, 3755, 23],
    [4330, 32, 3947, 16],
    [4440, 25, 4112, 10],
    [4530, 0, 4530, 0],
];

/// The casts the curve's factor stops growing at (0x5270E0: `min(n, 13)`).
const CURVE_CASTS: i32 = 13;

/// The casts a free spell allows (0x5273D0: 1000 at cost 0).
const FREE_CASTS: i32 = 1000;

/// The highest Spell Power the odds take (0x526D40 holds it to 1–99).
const MAX_POWER: u8 = 99;

/// A curve entry as the double the table holds.
fn curve(row: usize, column: usize) -> X87 {
    let n = SPELL_CURVE[row.min(SPELL_CURVE.len() - 1)][column];
    X87::ratio(n, 1000, DOUBLE_BITS).unwrap_or(X87::ZERO)
}

/// The computer's value of `side`'s hero in the odds (0x420A80's caster record 0x526D40 and best
/// spell 0x5275B0 with the AI bit `0x8000`; BAT-AI-008, `installation.md` §7.39), with
/// `strength` its side's whole strength: 0 without a hero or its spell book, or while casting is
/// forbidden (an anti-magic garrison, the Orb of Inhibition on either hero, BAT-MAG-061); else
/// the best from 0 among the spells of its book whose AI kinds hold `damage`, none above its own
/// spell level limit (Recanter's Cloak on the hero itself), of [`spell_value`]. The hero's power
/// held to 1–99 and its maximum mana (the adventure map's formula, not the points it has left).
/// Whether it has cast this round is not asked.
// rule: BAT-AI-008
pub(crate) fn hero_value(
    state: &BattleState,
    ruleset: &BattleRuleset,
    side: BattleSide,
    strength: i32,
) -> i32 {
    let Some(hero) = state.heroes[side.index()]
        .as_ref()
        .filter(|h| h.view.spell_book && !magic::casting_forbidden(state))
    else {
        return 0;
    };
    let caster = Caster {
        power: hero.view.power.clamp(1, MAX_POWER),
        mana: i32::from(hero.view.max_mana),
        strength,
    };
    hero.magic
        .book
        .iter()
        .filter_map(|id| ruleset.spell(id).map(|spell| (id, spell)))
        .filter(|(_, spell)| {
            spell.ai.kinds.contains(SpellAiKind::Damage) && spell.level <= hero.magic.level_limit
        })
        .map(|(id, spell)| spell_value(state, ruleset, hero, (id, spell), caster))
        .fold(0, i32::max)
}

/// The caster record's numbers the odds read (0x526D40's +8, +0x10 and the side's strength as
/// +4).
#[derive(Debug, Clone, Copy)]
struct Caster {
    power: u8,
    mana: i32,
    strength: i32,
}

/// The odds' value of `hero`'s spell `id` (0x5273D0, 0x5270E0; §7.39): its mastery `m` on the
/// battlefield (the original's on the hero's own adventure tile, the battle's for a defending
/// hero; the attacker's tile is not in the seam: BAT-STUB-7); its cost at the hero's own mastery
/// with no army's change (0x4E5240 given no enemy army skips both, the own Mages too; BAT-MAG-054
/// at mastery without the battlefield's raise); 0 above the mana, else `n` casts, the mana over
/// the cost (1000 at cost 0). A spell of the kind `damage` alone: the base `b = v_m × (power +
/// m)` (`v_m` its AI value at `m`), `d` the hero's bonuses on it without a target
/// ([`magic::untargeted_bonus`]), `X = 10 d / S` in doubles with `S` the side's strength, `Y =
/// α_k X + β_k` for the first row `k` of 0–12 whose threshold `X` reaches (13 without one), `Z =
/// φ_min(n, 13) X`, and `trunc(S × Z)` when `Y > Z`, else `trunc(S × Y)`: 0 for `S` = 0 (the
/// original's NaN). Any other kinds (Titan's Lightning Bolt's): 1.
// rule: BAT-AI-008
fn spell_value(
    state: &BattleState,
    ruleset: &BattleRuleset,
    hero: &hero::HeroRecord,
    (id, spell): (SpellId, &BattleSpell),
    c: Caster,
) -> i32 {
    // stub: BAT-STUB-7
    // The original's mastery is on the hero's own adventure tile (slip 6), which the seam lacks
    // for an attacking hero: the battlefield's.
    let m = magic::mastery(state, ruleset, hero, id);
    let cost = i32::from(crate::mana::cost(spell, hero.magic.masteries.get(id), 0));
    if cost > c.mana {
        return 0;
    }
    let casts = if cost > 0 { c.mana / cost } else { FREE_CASTS };
    if spell.ai.kinds.only() != Some(SpellAiKind::Damage) {
        return 1;
    }
    let v = spell.ai.values[usize::from(m.min(MAX_MASTERY))];
    let b = u64::from(v) * u64::from(c.power + m);
    let d = magic::untargeted_bonus(ruleset, spell, &hero.magic.bonuses, b);
    // `10 d`, an `int` in the original (far below 2^31: the base is at most 65 535 × 102).
    let ten = d.saturating_mul(10).min(i32::MAX.unsigned_abs().into());
    let s = X87::int(u64::try_from(c.strength).unwrap_or(0));
    let Some(x) = X87::int(ten).over(s) else {
        return 0;
    };
    let k = (0..SPELL_CURVE.len() - 1)
        .find(|&k| x >= curve(k, 1))
        .unwrap_or(SPELL_CURVE.len() - 1);
    let y = x.times(curve(k, 2)).plus(curve(k, 3)).round(DOUBLE_BITS);
    let n = usize::try_from(casts.clamp(0, CURVE_CASTS)).unwrap_or(0);
    let z = x.times(curve(n, 0)).round(DOUBLE_BITS);
    ftol(s.times(if y > z { z } else { y }))
}

/// The computer's value of `a` striking `t` after `jousting` hexes (0x435C70, 0x435B90 without the
/// shot flag, the walker's; BAT-AI-005): with both stacks' whole hit points, the [`exchange`]'s
/// damage to `t` valued at `t`'s worth (1000 per creature in desperate mode), less the hit points
/// `a` loses at `a`'s worth (never desperate), both without the shot flag; as the original's
/// 32-bit integers.
// rule: BAT-AI-005
pub(crate) fn strike_value(
    state: &BattleState,
    (ruleset, field): (&BattleRuleset, Field),
    ctx: &Context,
    (a, t): (StackId, StackId),
    jousting: u8,
) -> i32 {
    let (a0, t0) = (hit_points(state.at(a)), hit_points(state.at(t)));
    let (a1, t1) = exchange(state, ruleset, ctx.side, ((a, a0), (t, t0)), jousting);
    let gain = damage_value(
        state,
        (ruleset, field),
        ctx,
        (t, t0 - t1),
        (false, ctx.desperate),
    );
    if a0 > a1 {
        gain.wrapping_sub(damage_value(
            state,
            (ruleset, field),
            ctx,
            (a, a0 - a1),
            (false, false),
        ))
    } else {
        gain
    }
}

/// Whether stack `id` can shoot now for the computer with `blocker` left out of its blockers
/// (0x4428F0 with it): the Ballista and a turret always.
pub(crate) fn shoots_without(
    state: &BattleState,
    field: Field,
    id: StackId,
    blocker: StackId,
) -> bool {
    let s = state.at(id);
    is_ballista(s) || s.is_turret() || can_shoot_without(state, field, id, blocker)
}

/// The total hit points of stack `id` for the computer (0x443080 without the planned damage): 1
/// for a clone.
pub(crate) fn total_hit_points(state: &BattleState, id: StackId) -> u64 {
    hit_points(state.at(id))
}

/// The computer's value of a shot of `shooter` at `target` before any divisor (0x435B90 with the
/// shot flag; as 0x435C70 gives it): the kills of its estimated damage ([`estimate`]) times the
/// worth of one of the target's creatures with the shot flag ([`kill_value`]; 1000 in `desperate`
/// mode).
fn kill_shot_value(
    state: &BattleState,
    (ruleset, field): (&BattleRuleset, Field),
    ctx: &Context,
    (shooter, target): (StackId, StackId),
    desperate: bool,
) -> i32 {
    let t = state.at(target);
    let dealt = estimate(state, ruleset, (shooter, target));
    let v = if desperate && !t.has(StackTraits::CLONE) {
        X87::int(DESPERATE_WORTH)
    } else {
        worth(state, (ruleset, field), ctx, target, true)
    };
    kill_value(t, i64::try_from(dealt).unwrap_or(i64::MAX), v)
}

/// The computer's value of a shot of `shooter` at `target` (0x435CB0, 0x435B90; BAT-WAR-008): its
/// [`kill_shot_value`], whole on Easy for a computer side; else divided by 10 for a stack that
/// loses its turns, by 5 for a First Aid Tent or an Ammo Cart, a stack without a target
/// ([`turns`], through `threats`) or one more than 5 turns away, else by its turns.
// rule: BAT-WAR-008
pub(crate) fn shot_value(
    state: &BattleState,
    (ruleset, field): (&BattleRuleset, Field),
    (ctx, threats): (&Context, &Threats),
    (shooter, target): (StackId, StackId),
    desperate: bool,
) -> i32 {
    let t = state.at(target);
    let value = kill_shot_value(state, (ruleset, field), ctx, (shooter, target), desperate);
    let side = state.at(shooter).controller();
    if state.difficulty == 0 && !state.human[side.index()] {
        return value;
    }
    let divisor = if t.spells.status.contains(Status::NO_TURNS) {
        DISABLED
    } else if t.has(StackTraits::FIRST_AID) || t.has(StackTraits::AMMO_SUPPLY) {
        MAX_TURNS
    } else {
        threats
            .turns(state, (ruleset, field), target)
            .map_or(MAX_TURNS, |n| {
                i32::try_from(n).unwrap_or(MAX_TURNS).min(MAX_TURNS)
            })
    };
    value / divisor
}

/// The value of an area shot of `shooter` aimed at `at` for the computer (0x41EDA0, the Magogs',
/// Liches' and Power Liches'; BAT-AI-003): over the living stacks with a hex at `at` or next to it
/// (0x5A46F0, whose area never takes a side column: a war machine's hex there beside `at` has
/// its other hex beside `at` too, so the stacks are the same), minus the
/// [`kill_shot_value`] of each stack of the acting side and plus the [`shot_value`] of each
/// other, summed as the original's 32-bit integers. With BAT-QUIRK-25 (survey E23 §4, slip 1) an
/// undead shooter leaves out the undead stacks that do not cover `at` and no other immunity
/// counts, as the original estimates; without it every stack but `target` that the shot's area
/// spares is left out (BAT-RNG-004: a fireball the fire-immune, a death cloud the lifeless).
// rule: BAT-AI-003
fn area_value(
    state: &BattleState,
    (ruleset, field): (&BattleRuleset, Field),
    (ctx, threats): (&Context, &Threats),
    (shooter, target, at): (StackId, StackId, Hex),
    desperate: bool,
) -> i32 {
    let me = state.at(shooter);
    let side = me.controller();
    let slips = ruleset.quirks().is_enabled(ORIGINAL_AI_SLIPS);
    let undead = |s: &Stack| {
        ruleset
            .creature_info(s.creature)
            .is_some_and(|info| info.flags.contains(CreatureFlag::Undead))
    };
    let undead_shooter = undead(me);
    let mut sum = 0i32;
    for (i, s) in (0u8..).zip(state.stacks.iter()) {
        if !s.is_alive() || s.is_turret() {
            continue;
        }
        let fp = s.footprint(field);
        if !fp.hexes().any(|h| field.distance(h, at) <= 1) {
            continue;
        }
        let id = StackId(i);
        // BAT-QUIRK-25 (slip 1): the original's estimate, or the shot's rule.
        let left_out = if slips {
            undead_shooter && undead(s) && !fp.covers(at)
        } else {
            id != target && spares(me, s)
        };
        if left_out {
            continue;
        }
        let pair = (shooter, id);
        sum = if s.side == side {
            sum.wrapping_sub(kill_shot_value(
                state,
                (ruleset, field),
                ctx,
                pair,
                desperate,
            ))
        } else {
            sum.wrapping_add(shot_value(
                state,
                (ruleset, field),
                (ctx, threats),
                pair,
                desperate,
            ))
        };
    }
    sum
}

/// A target the computer's shooter picks (0x41EB80): the stack, the hex the shot is aimed at and
/// its value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Pick {
    value: i32,
    target: StackId,
    at: Hex,
}

/// The target of the computer's shooter `shooter` (0x41EB80; BAT-WAR-008, BAT-AI-002, BAT-AI-003):
/// of the living stacks it may shoot (the other side's, whoever controls them, but the turrets),
/// a stack that keeps its turns before one that loses them, then the best value, ties to the later
/// stack. A stack's value is its [`shot_value`], the shot aimed at its front hex; an area
/// shooter's is its [`area_value`] at the stack's position and, for a two-hex stack, at its other
/// hex when strictly higher there (the shot then aimed there), and a stack whose value is below 0
/// is no target. `None` without a target.
// rule: BAT-WAR-008, BAT-AI-002, BAT-AI-003
fn pick(
    state: &BattleState,
    (ruleset, field): (&BattleRuleset, Field),
    (ctx, threats): (&Context, &Threats),
    shooter: StackId,
    desperate: bool,
) -> Option<Pick> {
    let me = state.at(shooter);
    let mut best: Option<(bool, Pick)> = None;
    for (i, t) in (0u8..).zip(state.stacks.iter()) {
        if !t.is_alive() || !me.may_attack(t) {
            continue;
        }
        let target = StackId(i);
        let fp = t.footprint(field);
        let (value, at) = if me.has(StackTraits::AREA_SHOT) {
            let area = |at| {
                area_value(
                    state,
                    (ruleset, field),
                    (ctx, threats),
                    (shooter, target, at),
                    desperate,
                )
            };
            let mut aim = (area(t.hex), t.hex);
            if fp.rear.is_some() {
                let other = area(fp.front);
                if other > aim.0 {
                    aim = (other, fp.front);
                }
            }
            if aim.0 < 0 {
                continue;
            }
            aim
        } else {
            let pair = (shooter, target);
            (
                shot_value(state, (ruleset, field), (ctx, threats), pair, desperate),
                fp.front,
            )
        };
        let keeps = !t.spells.status.contains(Status::NO_TURNS);
        if best.is_none_or(|(k, b)| (keeps, value) >= (k, b.value)) {
            best = Some((keeps, Pick { value, target, at }));
        }
    }
    best.map(|(_, p)| p)
}

/// What the computer's shooter does (0x41F060): shoot a stack, batter a wall, or nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shot {
    /// A shot at `target` aimed at `at`.
    Stack {
        /// The stack shot at.
        target: StackId,
        /// The hex the shot is aimed at: the target's front hex, or an area shot's best hex.
        at: Hex,
    },
    /// A wall shot at wall-shot target `part` (the Cyclops's, BAT-SIE-013).
    Wall {
        /// The wall-shot target.
        part: u8,
    },
    /// No target: the original's Defend (action 3).
    Nothing,
}

/// What the computer's shooter `shooter` does (0x41F060): the context for its controller, its
/// target ([`pick`]; in desperate mode, BAT-SIE-023, first with every creature worth 1000); a wall
/// shooter (the ability `wall_shots`, the original's flag 0x20: the Cyclops) then asks `wall` with
/// the context and the target's value (0 without one) for a wall to batter (0x41EEA0, the battle
/// AI's); without a target nothing; in desperate mode a target of value 0 is chosen again without
/// it (when the area values then leave none, the original aims at hex −1: nothing here).
pub(crate) fn shot(
    state: &BattleState,
    ruleset: &BattleRuleset,
    field: Field,
    shooter: StackId,
    wall: impl FnOnce(&Context, i32) -> Option<u8>,
) -> Shot {
    let side = state.at(shooter).controller();
    let ctx = context(state, ruleset, field, side);
    let threats = Threats::new();
    let first = pick(
        state,
        (ruleset, field),
        (&ctx, &threats),
        shooter,
        ctx.desperate,
    );
    if siege::shooter_level(state, ruleset, shooter).is_some()
        && let Some(part) = wall(&ctx, first.map_or(0, |p| p.value))
    {
        return Shot::Wall { part };
    }
    let Some(first) = first else {
        return Shot::Nothing;
    };
    let chosen = if ctx.desperate && first.value == 0 {
        pick(state, (ruleset, field), (&ctx, &threats), shooter, false)
    } else {
        Some(first)
    };
    chosen.map_or(Shot::Nothing, |p| Shot::Stack {
        target: p.target,
        at: p.at,
    })
}

/// The stack the automatic Ballista `shooter` shoots (0x41F060, 0x41EB80; BAT-WAR-008): the
/// computer's [`shot`] without a wall shot, its target; `None` without a stack to shoot.
// rule: BAT-WAR-008, BAT-SIE-023
pub(crate) fn ballista_target(
    state: &BattleState,
    ruleset: &BattleRuleset,
    field: Field,
    shooter: StackId,
) -> Option<StackId> {
    match shot(state, ruleset, field, shooter, |_, _| None) {
        Shot::Stack { target, .. } => Some(target),
        Shot::Wall { .. } | Shot::Nothing => None,
    }
}

/// The standing walls of a siege with their hit points, in index order (the wall-shot targets of
/// kind wall, BAT-SIE-010: the original's list 0x63ABD0 of the upper, mid-upper, mid-lower and
/// lower walls); none outside a siege.
pub(crate) fn walls(state: &BattleState) -> impl Iterator<Item = (u8, u8)> + '_ {
    state.siege.iter().flat_map(|s| {
        (0..s.targets)
            .filter(move |&t| s.kind(t) == Some(WallKind::Wall) && s.stands(t))
            .map(move |t| (t, s.parts[usize::from(t)]))
    })
}
