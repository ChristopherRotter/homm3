//! War machines (BAT-WAR-001..006): their automatic turns and which of them their side commands.
//!
//! Without a hero's Artillery or First Aid skill war machines are controlled by the engine: when
//! one becomes active it acts at once and its turn ends. The Ballista shoots the target VCMI's
//! automatic control picks (BAT-WAR-002), the First Aid Tent heals a random damaged allied stack
//! (BAT-WAR-003); the Ammo Cart never becomes active (BAT-WAR-004). A hero with Artillery
//! commands its side's Ballista instead (BAT-WAR-006).

use core::cmp::Ordering;

use h3_content::{Ability, Ruleset};
use h3_contracts::BallistaControl;
use h3_kernel::FixedVec;
use h3_kernel::arith::cmp_fractions;

use crate::combat::{can_shoot, shoot, shot_damage_range};
use crate::damage::apply_damage;
use crate::event::{Event, EventKind, Sink};
use crate::geometry::Field;
use crate::hero;
use crate::movement::reachable;
use crate::state::{BattleState, STACK_CAPACITY, StackId, StackTraits};

/// How the side of stack `id` commands it, if `id` is a shooting war machine (the Ballista) and
/// the side's hero has Artillery (BAT-WAR-006).
pub(crate) fn commanded_ballista(state: &BattleState, id: StackId) -> Option<BallistaControl> {
    let s = state.at(id);
    if s.has(StackTraits::WAR_MACHINE) && s.has(StackTraits::SHOOTER) {
        hero::ballista(&state.heroes, s.side)
    } else {
        None
    }
}

/// Whether war machine `id` waits for its side's command on its turn instead of acting on its
/// own: a commanded Ballista that can shoot (BAT-WAR-006). Otherwise its turn is automatic
/// (BAT-WAR-002/003; a commanded Ballista that cannot shoot does nothing).
// rule: BAT-WAR-006
pub(crate) fn is_commanded(state: &BattleState, field: Field, id: StackId) -> bool {
    commanded_ballista(state, id).is_some() && can_shoot(state, field, id)
}

/// The automatic turn of war machine `id` (BAT-WAR-002/003): heal, shoot, or nothing.
pub(crate) fn act<S: Sink>(state: &mut BattleState, ruleset: &Ruleset, id: StackId, sink: &mut S) {
    let field = Field::new(ruleset.battle().field);
    if state.at(id).has(StackTraits::FIRST_AID) {
        first_aid(state, ruleset, id, sink);
    } else if can_shoot(state, field, id) {
        // While the battle goes on there is a living enemy to shoot at.
        shooting_target(state, ruleset, field, id)
            .map(|target| (target, state.at(target).hex))
            .into_iter()
            .for_each(|(target, at)| shoot(state, ruleset, id, target, at, sink));
    }
}

/// Whether stack `id` could attack next turn: it can shoot, or some enemy has an attack position
/// it can reach (VCMI `getCanAttackNextTurn`).
pub(crate) fn can_attack_next_turn(state: &BattleState, field: Field, id: StackId) -> bool {
    if can_shoot(state, field, id) {
        return true;
    }
    let me = state.at(id);
    let reach = reachable(state, field, id, me);
    state
        .stacks
        .iter()
        .filter(|t| t.is_alive() && t.side != me.side)
        .any(|t| {
            field
                .attack_positions(me.is_two_hex(), me.side, t.footprint(field))
                .into_iter()
                .flatten()
                .any(|from| from == me.hex || reach.can_move_to(from))
        })
}

/// A candidate target with its ranking keys (VCMI `tryMakeAutomaticActionOfRangedUnit`).
#[derive(Debug, Clone, Copy, Default)]
struct Candidate {
    id: StackId,
    /// Not a war machine (preferred).
    creature: bool,
    /// Value of the expected damage: `numerator / denominator` (see [`value`]).
    value: (u128, u128),
}

/// The value of shooting `target`: with average damage `D = (min + max) / 2` capped at the
/// target's health `T`, average kills `K` and AI value `V` per creature of `M` hit points,
/// `min(T, D) · V / M + K · V` — as the fraction `V · (min(2T, min + max) + (kmin + kmax) · M)`
/// over `2M`, compared exactly ([`cmp_fractions`]). No overflow for any stacks: `min + max` and
/// `2T` are below 2^65, the kills below 2^33 and `M` below 2^32, so the numerator stays below
/// 2^99 and the denominator below 2^33.
pub(crate) fn value(
    state: &BattleState,
    ruleset: &Ruleset,
    shooter: StackId,
    target: StackId,
) -> (u128, u128) {
    let (min, max) = shot_damage_range(state, ruleset, shooter, target);
    let t = state.at(target);
    let kills = |damage: u64| {
        let mut copy = *t;
        u128::from(apply_damage(&mut copy, damage))
    };
    let ai = u128::from(ruleset.creature_info(t.creature).map_or(0, |i| i.ai_value));
    let m = u128::from(t.max_hp);
    let dealt = (2 * u128::from(t.total_hp())).min(u128::from(min) + u128::from(max));
    (ai * (dealt + (kills(min) + kills(max)) * m), 2 * m)
}

/// The target a war machine shoots (BAT-WAR-002, ported from VCMI): among the living enemies,
/// prefer stacks that are not war machines, then stacks that could attack next turn, then the
/// highest value of the expected damage ([`value`]); ties go to the lowest stack id.
///
/// Whether a stack could attack next turn needs its reachability, the expensive key, so it is
/// evaluated lazily: within the best class (creatures if there are any), candidates are taken by
/// value and id until one could attack; if none can, the best by value and id wins.
// rule: BAT-WAR-002
pub(crate) fn shooting_target(
    state: &BattleState,
    ruleset: &Ruleset,
    field: Field,
    shooter: StackId,
) -> Option<StackId> {
    let side = state.at(shooter).side;
    // Fixed capacity, no allocation (ADR-0004 §8): at most every stack is a candidate.
    let mut candidates: FixedVec<Candidate, STACK_CAPACITY> = FixedVec::new();
    for (i, s) in (0u8..).zip(state.stacks.iter()) {
        if s.is_alive() && s.side != side {
            let id = StackId(i);
            let _ = candidates.push(Candidate {
                id,
                creature: !s.has(StackTraits::WAR_MACHINE),
                value: value(state, ruleset, shooter, id),
            });
        }
    }
    let best_class = candidates.iter().any(|c| c.creature);
    // The higher value wins, then the lower id (E03 review: the cross products are compared
    // in 256 bits, so they cannot overflow).
    let rank = |a: &Candidate, b: &Candidate| -> Ordering {
        cmp_fractions(a.value, b.value).then_with(|| b.id.cmp(&a.id))
    };
    // Best first (selection, no sort): the first that could attack next turn wins, else the best.
    let mut taken = [false; STACK_CAPACITY];
    let mut best = None;
    while let Some((i, c)) = candidates
        .iter()
        .enumerate()
        .filter(|(i, c)| c.creature == best_class && !taken[*i])
        .max_by(|a, b| rank(a.1, b.1))
    {
        best.get_or_insert(c.id);
        if can_attack_next_turn(state, field, c.id) {
            return Some(c.id);
        }
        taken[i] = true;
    }
    best
}

/// The First Aid Tent's turn (BAT-WAR-003): a uniformly chosen allied stack that is alive, not
/// a war machine and whose top creature is hurt regains a uniform `min_hp..=max_hp` hit points
/// on its top creature (no resurrection); nothing happens if no stack is hurt.
// rule: BAT-WAR-003
fn first_aid<S: Sink>(state: &mut BattleState, ruleset: &Ruleset, id: StackId, sink: &mut S) {
    let me = state.at(id);
    let (min, max) = ruleset
        .abilities(me.creature)
        .iter()
        .find_map(|a| match *a {
            Ability::FirstAid { min_hp, max_hp } => Some((min_hp, max_hp)),
            _ => None,
        })
        .unwrap_or_default();
    let mut hurt: FixedVec<StackId, STACK_CAPACITY> = FixedVec::new();
    for (i, s) in (0u8..).zip(state.stacks.iter()) {
        if s.side == me.side
            && s.is_alive()
            && !s.has(StackTraits::WAR_MACHINE)
            && s.top_hp < s.max_hp
        {
            let _ = hurt.push(StackId(i));
        }
    }
    let n = u32::try_from(hurt.len()).unwrap_or(u32::MAX);
    if n == 0 {
        return;
    }
    let target = hurt
        .get(usize::try_from(state.rng.below(n)).unwrap_or(0))
        .copied()
        .unwrap_or(id);
    let amount = state.rng.range_inclusive(u32::from(min), u32::from(max));
    let t = state.at_mut(target);
    let hit_points = amount.min(t.max_hp - t.top_hp);
    t.top_hp += hit_points;
    sink.emit(Event::public(EventKind::Healed {
        healer: id,
        target,
        hit_points,
    }));
}
