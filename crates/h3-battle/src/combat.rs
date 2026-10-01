//! Strikes and their sequences: melee exchanges (BAT-ATK-001, BAT-RET-001), shots
//! (BAT-RNG-001..004) and the end of the battle (BAT-END-001).

use h3_content::BattleParams;
use h3_contracts::BattleSide;

use crate::damage::{Factors, apply_damage, attack_defense, base_damage, effective_defense};
use crate::event::{Event, EventKind, Sink, StrikeFlags};
use crate::geometry::{Field, Hex};
use crate::state::{BattleState, Stack, StackFlags, StackId, StackTraits};

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
/// (BAT-RNG-001).
// rule: BAT-RNG-001
pub(crate) fn can_shoot(state: &BattleState, field: Field, id: StackId) -> bool {
    let s = state.at(id);
    s.has(StackTraits::SHOOTER) && s.shots > 0 && !is_blocked(state, field, id)
}

/// One strike of `attacker` on `target`: base damage, factors, damage applied, event.
pub(crate) fn strike<S: Sink>(
    state: &mut BattleState,
    params: &BattleParams,
    field: Field,
    attacker: StackId,
    target: StackId,
    flags: StrikeFlags,
    sink: &mut S,
) {
    let striker = *state.at(attacker);
    let victim = *state.at(target);
    let base = base_damage(
        &mut state.rng,
        striker.count,
        striker.damage_min,
        striker.damage_max,
        params.damage_roll_samples,
    );
    let mut factors = Factors::default();
    attack_defense(
        &mut factors,
        striker.attack,
        effective_defense(&victim, params),
        params,
    );
    if let Some(permille) = penalty(params, field, &striker, &victim, flags) {
        factors.reduce(permille);
    }
    let damage = factors.apply(base);
    let kills = apply_damage(state.at_mut(target), damage);
    sink.emit(Event::public(EventKind::Struck {
        attacker,
        target,
        damage,
        kills,
        flags,
    }));
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

/// A melee exchange: the strike, then the target's retaliation if it survived and has not
/// retaliated this round (BAT-RET-001), then the end-of-battle check (BAT-END-001).
pub(crate) fn melee<S: Sink>(
    state: &mut BattleState,
    params: &BattleParams,
    field: Field,
    attacker: StackId,
    target: StackId,
    sink: &mut S,
) {
    strike(
        state,
        params,
        field,
        attacker,
        target,
        StrikeFlags::NONE,
        sink,
    );
    let t = state.at(target);
    // rule: BAT-RET-001
    if t.is_alive() && !t.flags.contains(StackFlags::RETALIATED) {
        state.at_mut(target).flags.set(StackFlags::RETALIATED, true);
        strike(
            state,
            params,
            field,
            target,
            attacker,
            StrikeFlags::RETALIATION,
            sink,
        );
    }
    check_end(state, sink);
}

/// A shot at `target`, aimed at its hex `at`: one ranged strike per shot (two for shooters that
/// strike twice, as long as the target lives and shots are left), each using a shot; area shots
/// also strike every other living stack around `at` that the area does not spare (BAT-RNG-004).
/// Shots are not retaliated. Then the end-of-battle check.
// rule: BAT-RNG-001
pub(crate) fn shoot<S: Sink>(
    state: &mut BattleState,
    params: &BattleParams,
    field: Field,
    shooter: StackId,
    target: StackId,
    at: Hex,
    sink: &mut S,
) {
    let times = if state.at(shooter).has(StackTraits::STRIKES_TWICE) {
        2
    } else {
        1
    };
    for i in 0..times {
        if i > 0 && (!state.at(target).is_alive() || state.at(shooter).shots == 0) {
            break;
        }
        let s = state.at_mut(shooter);
        s.shots -= 1;
        strike(
            state,
            params,
            field,
            shooter,
            target,
            StrikeFlags::RANGED,
            sink,
        );
        if state.at(shooter).has(StackTraits::AREA_SHOT) {
            area(state, params, field, shooter, target, at, sink);
        }
    }
    check_end(state, sink);
}

/// The area part of an area shot: every living stack other than the primary target with a hex
/// in `at` or next to it, in stack-id order, unless the area spares it — a fireball spares
/// fire-immune stacks, a death cloud stacks that are not alive (BAT-RNG-004).
// rule: BAT-RNG-004
fn area<S: Sink>(
    state: &mut BattleState,
    params: &BattleParams,
    field: Field,
    shooter: StackId,
    target: StackId,
    at: Hex,
    sink: &mut S,
) {
    let spared: fn(&Stack) -> bool = if state.at(shooter).has(StackTraits::DEATH_CLOUD) {
        |s| !s.has(StackTraits::ALIVE)
    } else {
        |s| s.has(StackTraits::FIRE_IMMUNE)
    };
    let hit: [bool; crate::state::STACK_CAPACITY] = core::array::from_fn(|i| {
        state.stacks.get(i).is_some_and(|s| {
            s.is_alive()
                && i != target.index()
                && !spared(s)
                && s.footprint(field)
                    .hexes()
                    .any(|h| field.distance(h, at) <= 1)
        })
    });
    for (i, _) in hit.iter().enumerate().filter(|(_, h)| **h) {
        let flags = StrikeFlags::RANGED.with(StrikeFlags::SECONDARY);
        // `i < STACK_CAPACITY <= u8::MAX`.
        let id = StackId(u8::try_from(i).unwrap_or(u8::MAX));
        strike(state, params, field, shooter, id, flags, sink);
    }
}

/// The battle ends when one side has no living stacks; the other side wins.
// rule: BAT-END-001
pub(crate) fn check_end<S: Sink>(state: &mut BattleState, sink: &mut S) {
    let alive = |side: BattleSide| {
        state
            .stacks
            .iter()
            .any(|s: &Stack| s.side == side && s.is_alive())
    };
    let winner = match (alive(BattleSide::Attacker), alive(BattleSide::Defender)) {
        (true, false) => Some(BattleSide::Attacker),
        (false, true) => Some(BattleSide::Defender),
        _ => None,
    };
    if let Some(w) = winner {
        state.winner = Some(w);
        sink.emit(Event::public(EventKind::BattleEnded { winner: w }));
    }
}
