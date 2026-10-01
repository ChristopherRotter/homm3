//! War machines (BAT-WAR-001..004): their automatic turns.
//!
//! Without a hero's Artillery or First Aid skill (E04) war machines are controlled by the
//! engine: when one becomes active it acts at once and its turn ends. The Ballista shoots the
//! target VCMI's automatic control picks (BAT-WAR-002), the First Aid Tent heals a random
//! damaged allied stack (BAT-WAR-003); the Ammo Cart never becomes active (BAT-WAR-004).

use alloc::vec::Vec;
use core::cmp::Ordering;

use h3_content::{Ability, Ruleset};

use crate::combat::{can_shoot, shoot, shot_damage_range};
use crate::damage::apply_damage;
use crate::event::{Event, EventKind, Sink};
use crate::geometry::Field;
use crate::movement::reachable;
use crate::state::{BattleState, StackId, StackTraits};

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
fn can_attack_next_turn(state: &BattleState, field: Field, id: StackId) -> bool {
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
struct Candidate {
    id: StackId,
    /// Not a war machine (preferred).
    creature: bool,
    /// Can attack next turn (preferred).
    dangerous: bool,
    /// Value of the expected damage: `numerator / denominator` (see [`value`]).
    value: (u128, u128),
}

/// The value of shooting `target`: with average damage `D = (min + max) / 2` capped at the
/// target's health `T`, average kills `K` and AI value `V` per creature of `M` hit points,
/// `min(T, D) · V / M + K · V` — as the fraction `V · (min(2T, min + max) + (kmin + kmax) · M)`
/// over `2M`, compared exactly.
fn value(
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
// rule: BAT-WAR-002
fn shooting_target(
    state: &BattleState,
    ruleset: &Ruleset,
    field: Field,
    shooter: StackId,
) -> Option<StackId> {
    let side = state.at(shooter).side;
    let candidates: Vec<Candidate> = (0u8..)
        .zip(state.stacks.iter())
        .filter(|(_, s)| s.is_alive() && s.side != side)
        .map(|(i, s)| {
            let id = StackId(i);
            Candidate {
                id,
                creature: !s.has(StackTraits::WAR_MACHINE),
                dangerous: can_attack_next_turn(state, field, id),
                value: value(state, ruleset, shooter, id),
            }
        })
        .collect();
    let better = |a: &Candidate, b: &Candidate| -> Ordering {
        (a.creature, a.dangerous)
            .cmp(&(b.creature, b.dangerous))
            .then_with(|| (a.value.0 * b.value.1).cmp(&(b.value.0 * a.value.1)))
            // The lower id wins ties.
            .then_with(|| b.id.cmp(&a.id))
    };
    candidates.iter().max_by(|a, b| better(a, b)).map(|c| c.id)
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
    let hurt: Vec<StackId> = (0u8..)
        .zip(state.stacks.iter())
        .filter(|(_, s)| {
            s.side == me.side
                && s.is_alive()
                && !s.has(StackTraits::WAR_MACHINE)
                && s.top_hp < s.max_hp
        })
        .map(|(i, _)| StackId(i))
        .collect();
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
