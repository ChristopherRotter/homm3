//! Attacks and their sequences: melee attacks with their abilities (BAT-ATK-001..004,
//! BAT-RET-001..003, BAT-DMG-004..009), shots (BAT-RNG-001..004) and the end of the battle
//! (BAT-END-001).
//!
//! An *attack* is one swing of a stack: one death-blow roll, a strike on the target, strikes on
//! the secondary targets its attack pattern hits, then life drain and death stare. A melee attack
//! (BAT-ATK-001) is the attack, the target's retaliation, and for creatures that strike twice a
//! second attack.

use h3_content::{Ability, BattleParams, CompiledAbility, Ruleset};
use h3_contracts::{BattleResult, BattleSide};
use h3_kernel::arith::mul_div_floor;

use crate::damage::{
    Factors, apply_damage, attack_defense, base_damage, effective_defense, reduced_defense,
    resurrect,
};
use crate::event::{Event, EventKind, Sink, StrikeFlags};
use crate::geometry::{Direction, Field, Footprint, Hex, HexSet};
use crate::morale;
use crate::state::{
    BattleState, STACK_CAPACITY, Stack, StackFlags, StackId, StackTraits, UNLIMITED_RETALIATIONS,
};

/// The secondary targets of an attack, by stack index.
type Targets = [bool; STACK_CAPACITY];

/// No secondary targets.
const NO_TARGETS: Targets = [false; STACK_CAPACITY];

/// Whether stack `id` is next to a living enemy (its footprint touches one), which keeps a
/// shooter from shooting (BAT-RNG-001).
pub(crate) fn is_blocked(state: &BattleState, field: Field, id: StackId) -> bool {
    let me = state.at(id);
    let fp = me.footprint(field);
    state
        .stacks
        .iter()
        .any(|s| s.is_alive() && s.side != me.side && field.touches(fp, s.footprint(field)))
}

/// Whether stack `id` can shoot now: a shooter with shots left that is not blocked
/// (BAT-RNG-001); war machines shoot even when blocked (BAT-WAR-001).
// rule: BAT-RNG-001, BAT-WAR-001
pub(crate) fn can_shoot(state: &BattleState, field: Field, id: StackId) -> bool {
    let s = state.at(id);
    s.has(StackTraits::SHOOTER)
        && s.shots > 0
        && (s.has(StackTraits::WAR_MACHINE) || !is_blocked(state, field, id))
}

/// Whether a living stack of `side` supplies ammunition (BAT-WAR-004).
// rule: BAT-WAR-004
fn ammo_supplied(state: &BattleState, side: BattleSide) -> bool {
    state
        .stacks
        .iter()
        .any(|s| s.side == side && s.is_alive() && s.has(StackTraits::AMMO_SUPPLY))
}

/// What all strikes of one attack share.
#[derive(Debug, Clone, Copy)]
struct Blow {
    /// Hexes travelled right before the attack (jousting, BAT-DMG-005): 0 for retaliations,
    /// second attacks and shots.
    charge: u8,
    /// The attack is a death blow (BAT-DMG-007).
    death_blow: bool,
    /// The strike is lucky (BAT-LCK-002; only the strike on the target of the attack, Q18).
    lucky: bool,
}

/// Whether the creature of `stack` has an ability matching `f`.
fn has_ability(ruleset: &Ruleset, stack: &Stack, f: fn(&CompiledAbility) -> bool) -> bool {
    ruleset.abilities(stack.creature).iter().any(f)
}

/// One strike of `attacker` on `target`: base damage, factors, damage applied, event. Returns the
/// hit points the target lost (the damage, capped at what it had).
fn strike<S: Sink>(
    state: &mut BattleState,
    ruleset: &Ruleset,
    (attacker, target): (StackId, StackId),
    flags: StrikeFlags,
    blow: Blow,
    sink: &mut S,
) -> u64 {
    let params = ruleset.battle();
    let striker = *state.at(attacker);
    let victim = *state.at(target);
    let base = base_damage(
        &mut state.rng,
        striker.count,
        striker.damage_min,
        striker.damage_max,
        params.damage_roll_samples,
    );
    let (factors, flags) = strike_factors(ruleset, (&striker, &victim), flags, blow);
    let damage = factors.apply(base);
    let lost = damage.min(victim.total_hp());
    let kills = apply_damage(state.at_mut(target), damage);
    sink.emit(Event::public(EventKind::Struck {
        attacker,
        target,
        damage,
        kills,
        flags,
    }));
    lost
}

/// The damage factors of a strike of `striker` on `victim` (BAT-DMG-002 and the abilities that
/// change them), and its flags with the luck and death-blow flags added.
fn strike_factors(
    ruleset: &Ruleset,
    (striker, victim): (&Stack, &Stack),
    flags: StrikeFlags,
    blow: Blow,
) -> (Factors, StrikeFlags) {
    let params = ruleset.battle();
    let field = Field::new(params.field);
    let mut factors = Factors::default();
    let mut defense = effective_defense(victim, params);
    for ability in ruleset.abilities(striker.creature) {
        match *ability {
            // rule: BAT-DMG-004
            Ability::Hate {
                target: hated,
                percent,
            } if hated == victim.creature => {
                factors.bonus(u32::from(percent));
            }
            // rule: BAT-DMG-005
            Ability::Jousting { percent_per_hex }
                if !has_ability(ruleset, victim, |a| matches!(a, Ability::ChargeImmunity)) =>
            {
                factors.bonus(u32::from(percent_per_hex) * u32::from(blow.charge));
            }
            // rule: BAT-DMG-006
            Ability::DefenseReduction { percent } => defense = reduced_defense(defense, percent),
            _ => {}
        }
    }
    attack_defense(&mut factors, striker.attack, defense, params);
    let mut flags = flags;
    // rule: BAT-LCK-002
    if blow.lucky {
        factors.bonus(100);
        flags = flags.with(StrikeFlags::LUCKY);
    }
    // rule: BAT-DMG-007
    if blow.death_blow {
        factors.bonus(100);
        flags = flags.with(StrikeFlags::DEATH_BLOW);
    }
    if let Some(permille) = penalty(params, field, striker, victim, flags) {
        factors.reduce(permille);
    }
    (factors, flags)
}

/// The least and greatest damage a shot of `attacker` would deal to `target` now, without luck
/// and death blows (the base damage of `count` creatures is between `count · min` and
/// `count · max` for every sampling, BAT-DMG-001).
pub(crate) fn shot_damage_range(
    state: &BattleState,
    ruleset: &Ruleset,
    attacker: StackId,
    target: StackId,
) -> (u64, u64) {
    let striker = state.at(attacker);
    let blow = Blow {
        charge: 0,
        death_blow: false,
        lucky: false,
    };
    let (factors, _) = strike_factors(
        ruleset,
        (striker, state.at(target)),
        StrikeFlags::RANGED,
        blow,
    );
    let n = u64::from(striker.count);
    (
        factors.apply(n * u64::from(striker.damage_min)),
        factors.apply(n * u64::from(striker.damage_max)),
    )
}

/// The ranged or melee penalty of a strike, in per mille, if any: shots at a target farther than
/// the full-damage distance (BAT-RNG-002), or melee strikes of shooters (BAT-RNG-003).
fn penalty(
    params: &BattleParams,
    field: Field,
    striker: &Stack,
    victim: &Stack,
    flags: StrikeFlags,
) -> Option<u16> {
    if flags.contains(StrikeFlags::RANGED) {
        // rule: BAT-RNG-002
        let near = victim.footprint(field).hexes().any(|h| {
            field.distance(striker.hex, h) <= u32::from(params.ranged_full_damage_distance)
        });
        (!near && !striker.has(StackTraits::NO_DISTANCE_PENALTY))
            .then(|| u16::from(params.ranged_penalty_percent) * 10)
    } else {
        // rule: BAT-RNG-003
        (striker.has(StackTraits::SHOOTER) && !striker.has(StackTraits::NO_MELEE_PENALTY))
            .then(|| u16::from(params.melee_penalty_percent) * 10)
    }
}

/// One attack of `attacker` on `target` that also hits the `others` (BAT-ATK-003, BAT-RNG-004):
/// the luck roll (BAT-LCK-002) and the death-blow roll (BAT-DMG-007), the strike on the target
/// (the only one luck applies to, Q18), the strikes on the others in stack-id order, then life
/// drain (BAT-DMG-008) and death stare (BAT-DMG-009).
fn attack<S: Sink>(
    state: &mut BattleState,
    ruleset: &Ruleset,
    pair: (StackId, StackId),
    others: &Targets,
    flags: StrikeFlags,
    charge: u8,
    sink: &mut S,
) {
    let (attacker, target) = pair;
    let abilities = ruleset.abilities(state.at(attacker).creature);
    let (mut death_blow, mut drain, mut stare) = (0u8, 0u16, 0u8);
    for ability in abilities {
        match *ability {
            Ability::DeathBlow { chance_percent } => death_blow = chance_percent,
            Ability::LifeDrain { percent } => drain = percent,
            Ability::DeathStare { chance_percent } => stare = chance_percent,
            _ => {}
        }
    }
    // Luck is rolled before the death blow, as VCMI does.
    let lucky = morale::lucky(state, ruleset, attacker);
    // rule: BAT-DMG-007
    let blow = Blow {
        charge,
        death_blow: death_blow > 0 && state.rng.below(100) < u32::from(death_blow),
        lucky: false,
    };
    // Life drain takes `drain` % of the hit points lost by living targets, rounded down per
    // target.
    let drained = |state: &BattleState, id: StackId, lost: u64| {
        if state.at(id).has(StackTraits::ALIVE) {
            mul_div_floor(lost, u64::from(drain), 100).unwrap_or(u64::MAX)
        } else {
            0
        }
    };
    let lost = strike(state, ruleset, pair, flags, Blow { lucky, ..blow }, sink);
    let mut heal = drained(state, target, lost);
    let secondary = flags.with(StrikeFlags::SECONDARY);
    for (i, _) in others.iter().enumerate().filter(|(_, hit)| **hit) {
        // `i < STACK_CAPACITY <= u8::MAX`.
        let id = StackId(u8::try_from(i).unwrap_or(u8::MAX));
        let lost = strike(state, ruleset, (attacker, id), secondary, blow, sink);
        heal = heal.saturating_add(drained(state, id, lost));
    }
    life_drain(state, attacker, heal, sink);
    if stare > 0 {
        death_stare(state, pair, stare, sink);
    }
}

/// Heals and resurrects `stack` by `hp` hit points drained from its targets (BAT-DMG-008).
// rule: BAT-DMG-008
fn life_drain<S: Sink>(state: &mut BattleState, stack: StackId, hp: u64, sink: &mut S) {
    let (hit_points, resurrected) = resurrect(state.at_mut(stack), hp);
    if hit_points > 0 {
        sink.emit(Event::public(EventKind::LifeDrained {
            stack,
            hit_points,
            resurrected,
        }));
    }
}

/// After its attack, each creature of `attacker` kills one creature of the living, surviving
/// `target` with `chance` % probability, at most `ceil(count · chance / 100)` in total (one trial
/// per creature, stopping at the cap). The kills remove whole creatures, the damaged top one
/// first, so a surviving stack's new top creature has full health (Q21, observed; VCMI keeps the
/// damage instead) (BAT-DMG-009).
// rule: BAT-DMG-009
fn death_stare<S: Sink>(
    state: &mut BattleState,
    (attacker, target): (StackId, StackId),
    chance: u8,
    sink: &mut S,
) {
    let victim = state.at(target);
    if !victim.is_alive() || !victim.has(StackTraits::ALIVE) {
        return;
    }
    let count = state.at(attacker).count;
    let cap = (u64::from(count) * u64::from(chance)).div_ceil(100);
    let mut successes: u64 = 0;
    for _ in 0..count {
        if successes == cap {
            break;
        }
        if state.rng.below(100) < u32::from(chance) {
            successes += 1;
        }
    }
    if successes == 0 {
        return;
    }
    let victim = state.at_mut(target);
    let kills = u32::try_from(successes)
        .unwrap_or(u32::MAX)
        .min(victim.count);
    victim.count -= kills;
    victim.top_hp = if victim.is_alive() { victim.max_hp } else { 0 };
    sink.emit(Event::public(EventKind::DeathStared {
        attacker,
        target,
        kills,
    }));
}

/// The secondary targets of a melee attack of `attacker` on `target` (BAT-ATK-003), from the
/// current positions: every adjacent enemy for creatures that attack all adjacent enemies; for
/// three-headed attacks, enemies on the two hexes beside the attacked hex towards the attacker;
/// for breath attacks, any stack on the hex behind the attacked hex. Never the target itself.
// rule: BAT-ATK-003
fn melee_secondaries(
    state: &BattleState,
    field: Field,
    attacker: StackId,
    target: StackId,
) -> Targets {
    let me = state.at(attacker);
    let all_adjacent = me.has(StackTraits::ALL_ADJACENT);
    if !all_adjacent && !me.has(StackTraits::THREE_HEADED) && !me.has(StackTraits::BREATH) {
        return NO_TARGETS;
    }
    let fp = me.footprint(field);
    let (enemy_hexes, any_hexes) = pattern_hexes(field, me, fp, state.at(target).footprint(field));
    core::array::from_fn(|i| {
        state.stacks.get(i).is_some_and(|s| {
            let enemy = s.side != me.side;
            let other = s.footprint(field);
            i != target.index()
                && s.is_alive()
                && ((enemy && all_adjacent && field.touches(fp, other))
                    || other
                        .hexes()
                        .any(|h| any_hexes.contains(h) || (enemy && enemy_hexes.contains(h))))
        })
    })
}

/// The hexes hit by the three-headed (enemies only) and breath (any stack) patterns of `me`, at
/// footprint `fp`, attacking the stack at `victim`.
///
/// Ported from VCMI `CBattleInfoCallback::getPotentiallyAttackableHexes` (1.7.5): the pattern
/// starts at the attacker's head, or at its tail if the head does not touch the target; the
/// attack direction points at the target's tail if that is adjacent, else at its head. Each
/// pattern is a walk of directions turned relative to the attack direction; a walk whose first
/// step would enter the attacker starts one step further, on the attacked hex. Hexes of the
/// attacker are never hit.
fn pattern_hexes(field: Field, me: &Stack, fp: Footprint, victim: Footprint) -> (HexSet, HexSet) {
    let mut enemy = HexSet::new();
    let mut any = HexSet::new();
    let head = Footprint {
        head: fp.head,
        tail: None,
    };
    // A footprint that touches the target with its tail only is two-hex.
    let origin = if field.touches(head, victim) {
        fp.head
    } else {
        fp.tail.unwrap_or(fp.head)
    };
    // The origin touches the target, so one of the directions exists.
    let dir = victim
        .tail
        .and_then(|t| field.direction(origin, t))
        .or_else(|| field.direction(origin, victim.head))
        .unwrap_or(Direction::Right);
    let walk = |first: u8, more: &[u8]| {
        let start = if field
            .neighbor(origin, dir.turned(first))
            .is_some_and(|h| fp.covers(h))
        {
            field.neighbor(origin, dir)
        } else {
            Some(origin)
        };
        core::iter::once(first)
            .chain(more.iter().copied())
            .fold(start, |hex, turn| {
                hex.and_then(|h| field.neighbor(h, dir.turned(turn)))
            })
            .filter(|h| !fp.covers(*h))
    };
    if me.has(StackTraits::THREE_HEADED) {
        for hex in [walk(1, &[]), walk(5, &[])].into_iter().flatten() {
            enemy.insert(hex);
        }
    }
    if me.has(StackTraits::BREATH)
        && let Some(hex) = walk(0, &[0])
    {
        any.insert(hex);
    }
    (enemy, any)
}

/// A melee attack of `attacker`, which started its turn on `start` and travelled `charge` hexes,
/// on `target` (BAT-ATK-001): the attack, the target's retaliation (BAT-RET-001..003), for
/// creatures that strike twice a second attack (BAT-ATK-002), the return to `start` of creatures
/// that strike and return (BAT-ATK-004), then the end-of-battle check (BAT-END-001).
pub(crate) fn melee<S: Sink>(
    state: &mut BattleState,
    ruleset: &Ruleset,
    attacker: StackId,
    target: StackId,
    start: Hex,
    charge: u8,
    sink: &mut S,
) {
    let field = Field::new(ruleset.battle().field);
    let me = state.at(attacker);
    // rule: BAT-ATK-002
    let times = if me.has(StackTraits::STRIKES_TWICE) && !me.has(StackTraits::SHOOTER) {
        2
    } else {
        1
    };
    for i in 0..times {
        if !state.at(attacker).is_alive() || !state.at(target).is_alive() {
            break;
        }
        let others = melee_secondaries(state, field, attacker, target);
        let charge = if i == 0 { charge } else { 0 };
        let pair = (attacker, target);
        attack(
            state,
            ruleset,
            pair,
            &others,
            StrikeFlags::NONE,
            charge,
            sink,
        );
        if i == 0 {
            retaliate(state, ruleset, target, attacker, sink);
        }
    }
    // rule: BAT-ATK-004
    let me = state.at(attacker);
    if me.has(StackTraits::STRIKE_AND_RETURN) && me.is_alive() && me.hex != start {
        let from = me.hex;
        state.at_mut(attacker).hex = start;
        sink.emit(Event::public(EventKind::Moved {
            stack: attacker,
            from,
            to: start,
        }));
    }
    check_end(state, sink);
}

/// The retaliation of `retaliator` on `attacker` (BAT-RET-001): if both live, the attacker's
/// strikes allow retaliation (BAT-RET-002) and the retaliator has a retaliation left this round
/// (BAT-RET-003), it attacks back with its full attack pattern.
// rule: BAT-RET-001, BAT-RET-002, BAT-RET-003
fn retaliate<S: Sink>(
    state: &mut BattleState,
    ruleset: &Ruleset,
    retaliator: StackId,
    attacker: StackId,
    sink: &mut S,
) {
    let r = state.at(retaliator);
    let a = state.at(attacker);
    // War machines never retaliate (BAT-WAR-001).
    if !r.is_alive()
        || !a.is_alive()
        || a.has(StackTraits::NO_ENEMY_RETALIATION)
        || r.has(StackTraits::WAR_MACHINE)
        || r.retaliations_left == 0
    {
        return;
    }
    let r = state.at_mut(retaliator);
    if r.retaliations != UNLIMITED_RETALIATIONS {
        r.retaliations_left -= 1;
        r.flags
            .set(StackFlags::RETALIATED, r.retaliations_left == 0);
    }
    let field = Field::new(ruleset.battle().field);
    let others = melee_secondaries(state, field, retaliator, attacker);
    let pair = (retaliator, attacker);
    attack(
        state,
        ruleset,
        pair,
        &others,
        StrikeFlags::RETALIATION,
        0,
        sink,
    );
}

/// A shot at `target`, aimed at its hex `at`: one ranged attack per shot (two for shooters that
/// strike twice, as long as the target lives and shots are left), each using a shot; area shots
/// also strike every other living stack around `at` that the area does not spare (BAT-RNG-004).
/// Shots are not retaliated. Then the end-of-battle check.
// rule: BAT-RNG-001
pub(crate) fn shoot<S: Sink>(
    state: &mut BattleState,
    ruleset: &Ruleset,
    shooter: StackId,
    target: StackId,
    at: Hex,
    sink: &mut S,
) {
    let field = Field::new(ruleset.battle().field);
    let times = if state.at(shooter).has(StackTraits::STRIKES_TWICE) {
        2
    } else {
        1
    };
    for i in 0..times {
        if i > 0 && (!state.at(target).is_alive() || state.at(shooter).shots == 0) {
            break;
        }
        if !ammo_supplied(state, state.at(shooter).side) {
            state.at_mut(shooter).shots -= 1;
        }
        let others = area(state, field, shooter, target, at);
        let pair = (shooter, target);
        attack(state, ruleset, pair, &others, StrikeFlags::RANGED, 0, sink);
    }
    check_end(state, sink);
}

/// The secondary targets of a shot: for area shots every living stack other than the primary
/// target with a hex in `at` or next to it, unless the area spares it — a fireball spares
/// fire-immune stacks, a death cloud stacks that are not alive (BAT-RNG-004).
// rule: BAT-RNG-004
fn area(state: &BattleState, field: Field, shooter: StackId, target: StackId, at: Hex) -> Targets {
    let me = state.at(shooter);
    if !me.has(StackTraits::AREA_SHOT) {
        return NO_TARGETS;
    }
    let spared: fn(&Stack) -> bool = if me.has(StackTraits::DEATH_CLOUD) {
        |s| !s.has(StackTraits::ALIVE)
    } else {
        |s| s.has(StackTraits::FIRE_IMMUNE)
    };
    core::array::from_fn(|i| {
        state.stacks.get(i).is_some_and(|s| {
            s.is_alive()
                && i != target.index()
                && !spared(s)
                && s.footprint(field)
                    .hexes()
                    .any(|h| field.distance(h, at) <= 1)
        })
    })
}

/// The battle ends when a side has no living stacks other than war machines (BAT-WAR-001): the
/// other side wins (BAT-END-001), or nobody if neither side has any left (BAT-END-002).
// rule: BAT-END-001, BAT-END-002
pub(crate) fn check_end<S: Sink>(state: &mut BattleState, sink: &mut S) {
    let alive = |side: BattleSide| {
        state
            .stacks
            .iter()
            .any(|s: &Stack| s.side == side && s.is_alive() && !s.has(StackTraits::WAR_MACHINE))
    };
    let result = match (alive(BattleSide::Attacker), alive(BattleSide::Defender)) {
        (true, true) => None,
        (true, false) => Some(BattleResult::Victory(BattleSide::Attacker)),
        (false, true) => Some(BattleResult::Victory(BattleSide::Defender)),
        (false, false) => Some(BattleResult::Draw),
    };
    if let Some(r) = result {
        state.result = Some(r);
        sink.emit(Event::public(EventKind::BattleEnded { result: r }));
    }
}
