//! War machines (BAT-WAR-001..007): their automatic turns and which of them their side commands.
//!
//! Without a hero's Artillery or First Aid skill war machines are controlled by the engine: when
//! one becomes active it acts at once and its turn ends. The Ballista shoots the target the
//! original's computer picks (BAT-WAR-002, BAT-WAR-008: [`crate::aim::ballista_target`]), the First
//! Aid Tent heals a random damaged stack of its side (BAT-WAR-003); the Ammo Cart never becomes
//! active (BAT-WAR-004). A hero with Artillery commands its side's Ballista instead (BAT-WAR-006),
//! one with First Aid its tent (BAT-WAR-007).

use h3_content::FULL_BP;
use h3_content::SINGLE_PRECISION_RATES;
use h3_content_battle::{Ability, BattleRuleset};
use h3_contracts::{BallistaControl, TentControl};
use h3_kernel::FixedVec;
use h3_kernel::arith::{X87, mul_div_floor};

use crate::combat::{can_shoot, shoot};
use crate::event::{Event, EventKind, Sink};
use crate::geometry::Field;
use crate::hero;
use crate::magic::hero_rate;
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
        || (state.at(id).is_catapult()
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
    if state.at(id).is_catapult() {
        // rule: BAT-SIE-011
        let level = crate::siege::shooter_level(state, ruleset, id).unwrap_or(0);
        // The Catapult stands only in a siege, whose field the ruleset has (BAT-SIE-001).
        if let Some((siege, field)) = state.siege.zip(ruleset.siege_field())
            && let Some(aim) = crate::siege::automatic_aim(state, (siege, field))
        {
            crate::siege::shoot_walls(state, field, (id, aim, level), sink);
        }
    } else if state.at(id).has(StackTraits::FIRST_AID) {
        first_aid(state, ruleset, id, sink);
    } else if state.at(id).is_turret() {
        // rule: BAT-SIE-020
        crate::aim::turret_target(state, ruleset, field, id)
            .map(|target| (target, state.at(target).hex))
            .into_iter()
            .for_each(|(target, at)| shoot(state, ruleset, id, target, at, sink));
    } else if can_shoot(state, field, id) {
        // rule: BAT-WAR-008
        // While the battle goes on there is a living enemy to shoot at, aimed at its position
        // (the original's action 7 at the record's +0x38).
        crate::aim::ballista_target(state, ruleset, field, id)
            .map(|target| (target, state.at(target).hex))
            .into_iter()
            .for_each(|(target, at)| shoot(state, ruleset, id, target, at, sink));
    }
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
