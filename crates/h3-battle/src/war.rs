//! War machines (BAT-WAR-001..007): their automatic turns and which of them their side commands.
//!
//! Without a hero's Artillery or First Aid skill war machines are controlled by the engine: when
//! one becomes active it acts at once and its turn ends. The Ballista shoots the target VCMI's
//! automatic control picks (BAT-WAR-002), the First Aid Tent heals a random damaged stack of its
//! side (BAT-WAR-003); the Ammo Cart never becomes active (BAT-WAR-004). A hero with Artillery
//! commands its side's Ballista instead (BAT-WAR-006), one with First Aid its tent (BAT-WAR-007).

use core::cmp::Ordering;

use h3_content::FULL_BP;
use h3_content::SINGLE_PRECISION_RATES;
use h3_content_battle::{Ability, BattleRuleset};
use h3_contracts::{BallistaControl, TentControl};
use h3_kernel::FixedVec;
use h3_kernel::arith::{X87, cmp_fractions, mul_div_floor};

use crate::combat::{can_shoot, shoot, shot_damage_range};
use crate::damage::apply_damage;
use crate::effects::Status;
use crate::event::{Event, EventKind, Sink};
use crate::geometry::Field;
use crate::hero;
use crate::magic::hero_rate;
use crate::movement::reachable_in_public;
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

/// How the side of stack `id` commands it, if `id` is a first-aid war machine (the First Aid
/// Tent) and the side's hero has First Aid (BAT-WAR-007).
pub(crate) fn commanded_tent(state: &BattleState, id: StackId) -> Option<TentControl> {
    let s = state.at(id);
    if s.has(StackTraits::FIRST_AID) {
        hero::tent(&state.heroes, s.side)
    } else {
        None
    }
}

/// Whether war machine `id` waits for its side's command on its turn instead of acting on its
/// own: a commanded Ballista that can shoot (BAT-WAR-006), a commanded First Aid Tent with a
/// patient (BAT-WAR-007). Otherwise its turn is automatic (BAT-WAR-002/003; a commanded machine
/// that has nothing to do does nothing).
// rule: BAT-WAR-006, BAT-WAR-007
pub(crate) fn is_commanded(
    state: &BattleState,
    ruleset: &BattleRuleset,
    field: Field,
    id: StackId,
) -> bool {
    (commanded_ballista(state, id).is_some() && can_shoot(state, field, id))
        || (commanded_tent(state, id).is_some() && !patients(state, id).is_empty())
        // rule: BAT-SIE-011
        || (state.at(id).slot == crate::siege::CATAPULT_SLOT
            && crate::siege::commanded_wall_shots(state, ruleset, id).is_some())
}

/// Whether the first-aid war machine `healer` may heal stack `target`: a stack of its own side,
/// whoever controls it (BAT-MAG-026: the original's automatic tent walks the stacks of its side
/// and its cursor, 0x476490, wants the patient's own side), that is alive, not a war machine and
/// whose top creature is hurt (BAT-WAR-003).
pub(crate) fn is_patient(state: &BattleState, healer: StackId, target: StackId) -> bool {
    let me = state.at(healer);
    state.stack(target).is_some_and(|s| {
        s.side == me.side && s.is_alive() && !s.has(StackTraits::WAR_MACHINE) && s.top_hp < s.max_hp
    })
}

/// The stacks the first-aid war machine `healer` may heal, by id ([`is_patient`]).
pub(crate) fn patients(state: &BattleState, healer: StackId) -> FixedVec<StackId, STACK_CAPACITY> {
    let mut out = FixedVec::new();
    for i in 0..state.stacks.len() {
        // `i < STACK_CAPACITY <= u8::MAX`.
        let id = StackId(u8::try_from(i).unwrap_or(u8::MAX));
        if is_patient(state, healer, id) {
            let _ = out.push(id);
        }
    }
    out
}

/// The automatic turn of war machine `id` (BAT-WAR-002/003, BAT-SIE-011): heal, shoot, shoot
/// the walls, or nothing.
pub(crate) fn act<S: Sink>(
    state: &mut BattleState,
    ruleset: &BattleRuleset,
    id: StackId,
    sink: &mut S,
) {
    let field = Field::new(ruleset.battle().field);
    if state.at(id).slot == crate::siege::CATAPULT_SLOT {
        // rule: BAT-SIE-011
        let level = crate::siege::shooter_level(state, ruleset, id).unwrap_or(0);
        if let Some(aim) = crate::siege::automatic_aim(state, ruleset) {
            crate::siege::shoot_walls(state, ruleset, (id, aim, level), sink);
        }
    } else if state.at(id).has(StackTraits::FIRST_AID) {
        first_aid(state, ruleset, id, sink);
    } else if can_shoot(state, field, id) {
        // While the battle goes on there is a living enemy to shoot at.
        shooting_target(state, ruleset, field, id)
            .map(|target| (target, state.at(target).footprint(field).front))
            .into_iter()
            .for_each(|(target, at)| shoot(state, ruleset, id, target, at, sink));
    }
}

/// Whether stack `id` could attack next turn: it can shoot, or some stack its next turn may
/// strike has an attack position it can reach (VCMI `getCanAttackNextTurn`). Its next turn may
/// strike the stacks it may attack ([`crate::Stack::may_attack`], BAT-MAG-026, as VCMI's
/// `battleCanAttackUnit`) or, while it is berserk, any other stack (BAT-MAG-025: its automatic
/// turn takes any; the engine's own term, VCMI has none). Its reach is the one both sides can
/// tell (BAT-MAG-031): the answer picks the automatic Ballista's target, which both sides are
/// told, so a pit hidden from either side does not count (E04b T11b review).
pub(crate) fn can_attack_next_turn(state: &BattleState, field: Field, id: StackId) -> bool {
    if can_shoot(state, field, id) {
        return true;
    }
    let me = state.at(id);
    let reach = reachable_in_public(state, field, id, me);
    let berserk = me.spells.status.contains(Status::BERSERK);
    (0u8..)
        .zip(state.stacks.iter())
        .filter(|&(i, t)| i != id.0 && t.is_alive() && (berserk || me.may_attack(t)))
        .map(|(_, t)| t)
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
    ruleset: &BattleRuleset,
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

/// The target a war machine shoots (BAT-WAR-002, ported from VCMI): among the living stacks it
/// may shoot, those of the other side whoever controls them ([`crate::Stack::may_attack`],
/// BAT-MAG-025/026: not a berserk or hypnotized stack of its own side, as VCMI compares the
/// owners), prefer stacks that are not war machines, then stacks that could attack next turn,
/// then the highest value of the expected damage ([`value`]); ties go to the lowest stack id.
///
/// Whether a stack could attack next turn needs its reachability, the expensive key, so it is
/// evaluated lazily: within the best class (creatures if there are any), candidates are taken by
/// value and id until one could attack; if none can, the best by value and id wins.
// rule: BAT-WAR-002
pub(crate) fn shooting_target(
    state: &BattleState,
    ruleset: &BattleRuleset,
    field: Field,
    shooter: StackId,
) -> Option<StackId> {
    let me = *state.at(shooter);
    // Fixed capacity, no allocation (ADR-0004 §8): at most every stack is a candidate.
    let mut candidates: FixedVec<Candidate, STACK_CAPACITY> = FixedVec::new();
    for (i, s) in (0u8..).zip(state.stacks.iter()) {
        if i != shooter.0 && s.is_alive() && me.may_attack(s) {
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

/// The First Aid Tent's automatic turn (BAT-WAR-003): a uniformly chosen patient
/// ([`is_patient`]) is healed ([`heal`]); nothing happens without patients.
// rule: BAT-WAR-003
fn first_aid<S: Sink>(state: &mut BattleState, ruleset: &BattleRuleset, id: StackId, sink: &mut S) {
    let hurt = patients(state, id);
    let n = u32::try_from(hurt.len()).unwrap_or(u32::MAX);
    if n == 0 {
        return;
    }
    let target = hurt
        .get(usize::try_from(state.rng.below(n)).unwrap_or(0))
        .copied()
        .unwrap_or(id);
    heal(state, ruleset, id, target, sink);
}

/// The first-aid war machine `healer` heals the top creature of `target` by a uniform
/// `min_hp..=max_hp` hit points of its ability `first_aid`, capped at full health, without
/// resurrection (BAT-WAR-003). A commanded tent's largest heal grows by its `heal_bonus` of
/// itself, rounded down (BAT-WAR-007: 1–25 becomes 1–50, 1–75, 1–100 with First Aid); with quirk
/// Q-0005 as the original computes it, `trunc(max × (1 + bonus))` with the bonus in single
/// precision (a First Aid specialist's growth, OPEN-QUESTIONS Q45).
// rule: BAT-WAR-003, BAT-WAR-007
pub(crate) fn heal<S: Sink>(
    state: &mut BattleState,
    ruleset: &BattleRuleset,
    healer: StackId,
    target: StackId,
    sink: &mut S,
) {
    let (min, max) = ruleset
        .abilities(state.at(healer).creature)
        .iter()
        .find_map(|a| match *a {
            Ability::FirstAid { min_hp, max_hp } => Some((u32::from(min_hp), u32::from(max_hp))),
            _ => None,
        })
        .unwrap_or_default();
    let single = ruleset.quirks().is_enabled(SINGLE_PRECISION_RATES);
    let max = commanded_tent(state, healer).map_or(max, |control| {
        let bonus = hero_rate(single, control.heal_bonus);
        let grown = if single {
            X87::int(max.into())
                .times(X87::ONE.plus(X87::fixed(bonus)))
                .trunc()
        } else {
            let full = u64::from(FULL_BP);
            mul_div_floor(u64::from(max), full + bonus, full).unwrap_or(u64::MAX)
        };
        u32::try_from(grown).unwrap_or(u32::MAX)
    });
    let amount = state.rng.range_inclusive(min, max);
    let t = state.at_mut(target);
    let hit_points = amount.min(t.max_hp - t.top_hp);
    t.top_hp += hit_points;
    sink.emit(Event::public(EventKind::Healed {
        healer,
        target,
        hit_points,
    }));
}
