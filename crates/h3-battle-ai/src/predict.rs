//! The computer's predictions (0x422A40 with its flag 0, the side's run 0x422880, the melee
//! prediction 0x4227A0 and the predicted strike 0x4224E0; `docs/formats/installation.md` §7.42,
//! §7.52; BAT-AI-015): before it values a spell, the spell context plans the damage its side's
//! stacks are expected to deal before the other side's next stack acts.
//!
//! From the start of the computer's order of action ([`order::list`]), each stack that cannot act
//! (blind, turned to stone or paralysed, dead, the First Aid Tent or the Ammo Cart, berserk, the
//! Catapult) or has no hit points left after the damage planned on it is passed over, of either
//! side; the first other stack whose controller is not the side ends the run. Each of the side's
//! stacks before it takes its turn as a prediction: one that can shoot now its shot
//! ([`computer::predicted_shot`]: no wall shot), which plans its damage by its hit points left on
//! the stack shot at; any other the walker's turn ([`Walker::predicting`]: no cast, its candidates
//! drawn from the stream), whose move and attack plans the strike, the target's retaliation (none
//! on Easy for a computer side) and a second strike ([`strike`]). Any other action plans nothing.
//! A stack's planned damage never exceeds its hit points (0x4430D0).
//!
//! The draws are the predicted walkers' (0x421B02, one per candidate): the context of every
//! creature cast moves the stream by its run's candidates.
//!
//! The retreat test predicts with flag 1 (0x422A40 with 1; BAT-AI-024, [`plan_both`]): the
//! side's run, then the other side's from where the first ended. Each predicted turn also leaves
//! threat records behind (survey slip 26, [`Overwrite`]): a walker's of its own side (0x422B20
//! with flag 1, 0x421F80), a shot's of the other side (0x422B20 with flag 0, 0x41F099).

use h3_battle::computer::{self, Planned, Shot};
use h3_battle::{BattleSide, BattleState, Command, Hex, Stack, StackId, StackTraits, Status};
use h3_content_battle::BattleRuleset;
use h3_kernel::Rng;

use crate::hexes::{self, Shape};
use crate::order;
use crate::walker::Walker;

/// The damage `side`'s run plans (0x422A40 with flag 0; BAT-AI-015): from nothing planned, over
/// the computer's order of action with the active stack's side first among equal keys (+0x132B8),
/// the side's stacks that act before the other side's next one ([`run`]). Their walkers' draws
/// come from `rng`.
// rule: BAT-AI-015
pub(crate) fn plan(
    state: &BattleState,
    ruleset: &BattleRuleset,
    side: BattleSide,
    rng: &mut Rng,
) -> Planned {
    let first = state
        .active()
        .map_or(side, |a| state.stacks()[a.index()].side);
    let (list, n) = order::list(state, ruleset, first);
    let mut planned = Planned::default();
    run(state, ruleset, (&list[..n], side), &mut planned, rng, None);
    planned
}

/// What a predicted turn left in a side's threat records (+0x538 to +0x544; survey slip 26): a
/// predicted walker's records of its side on its turn (0x422B20 with flag 1, the walker's own
/// cleared) or a predicted shot's records of the other side (0x422B20 with flag 0, 0x41F099), each
/// in the context of the side that acted with the damage planned before that turn.
#[derive(Debug, Clone, Copy)]
pub(crate) enum Overwrite {
    /// The walker's records of its side.
    Walker {
        /// The walker.
        walker: StackId,
        /// The damage planned before its turn.
        planned: Planned,
    },
    /// A shooter's records of the other side.
    Shot {
        /// The damage planned before its shot.
        planned: Planned,
    },
}

/// The last [`Overwrite`] of each side's threat records in a prediction, by side index.
pub(crate) type Overwrites = [Option<Overwrite>; 2];

/// The damage the predictions with flag 1 plan (0x422A40 with 1; the retreat test, BAT-AI-024):
/// from nothing planned, over the computer's order of action with the active stack's side first
/// among equal keys, `side`'s run ([`run`]) and then the other side's from the stack that ended
/// the first. Their walkers' draws come from `rng`. With `overwrites`, the last overwrite of each
/// side's threat records goes there (survey slip 26).
// rule: BAT-AI-024
pub(crate) fn plan_both(
    state: &BattleState,
    ruleset: &BattleRuleset,
    side: BattleSide,
    rng: &mut Rng,
    mut overwrites: Option<&mut Overwrites>,
) -> Planned {
    let first = state
        .active()
        .map_or(side, |a| state.stacks()[a.index()].side);
    let (list, n) = order::list(state, ruleset, first);
    let mut planned = Planned::default();
    let end = run(
        state,
        ruleset,
        (&list[..n], side),
        &mut planned,
        rng,
        overwrites.as_deref_mut(),
    );
    run(
        state,
        ruleset,
        (&list[end..n], side.opponent()),
        &mut planned,
        rng,
        overwrites,
    );
    planned
}

/// Whether the run passes over stack `s` (0x4228BC to 0x422924): one that cannot act (blind,
/// turned to stone or paralysed at any duration, dead, the First Aid Tent or the Ammo Cart), a
/// berserk one or the Catapult.
fn passed_over(s: &Stack) -> bool {
    s.spells.status.contains(Status::NO_TURNS)
        || !s.is_alive()
        || s.has(StackTraits::FIRST_AID)
        || s.has(StackTraits::AMMO_SUPPLY)
        || s.spells.status.contains(Status::BERSERK)
        || s.is_catapult()
}

/// The creatures `hp` hit points of `s` give (0x435980's count: rounded up).
fn creatures(s: &Stack, hp: u64) -> u64 {
    hp.div_ceil(u64::from(s.max_hp.max(1)))
}

/// `side`'s run over the order `list` (0x422880; BAT-AI-015): each stack passed over
/// ([`passed_over`], or without hit points left after `planned`) is skipped; the first other whose
/// controller is not `side` ends it, and the run answers its place in `list` (the list's length
/// when none does); each other predicts its turn. A shooter's shot plans the damage of its
/// creatures by its hit points left (the shot flag, capped) on the stack shot at; a walker's move
/// and attack plans its melee ([`melee`]). With `overwrites`, each turn notes the threat records it
/// leaves ([`Overwrite`]).
fn run(
    state: &BattleState,
    ruleset: &BattleRuleset,
    (list, side): (&[StackId], BattleSide),
    planned: &mut Planned,
    rng: &mut Rng,
    mut overwrites: Option<&mut Overwrites>,
) -> usize {
    for (at, &id) in list.iter().enumerate() {
        let s = &state.stacks()[id.index()];
        if passed_over(s) || planned.left(state, id) == 0 {
            continue;
        }
        if s.controller() != side {
            return at;
        }
        let shoots = computer::shoots(state, ruleset, id);
        if let Some(o) = overwrites.as_deref_mut() {
            let planned = *planned;
            let (written, overwrite) = if shoots {
                (side.opponent(), Overwrite::Shot { planned })
            } else {
                (
                    side,
                    Overwrite::Walker {
                        walker: id,
                        planned,
                    },
                )
            };
            o[written.index()] = Some(overwrite);
        }
        if shoots {
            if let Shot::Stack { target, .. } =
                computer::predicted_shot(state, ruleset, (id, side), planned)
            {
                let count = creatures(s, planned.left(state, id));
                let damage =
                    computer::damage_estimate(state, ruleset, (id, target), count, true, 0);
                planned.add(state, target, damage);
            }
            continue;
        }
        let w = Walker::predicting(state, ruleset, (id, side), planned);
        let mut value = 0;
        if let Some(Command::Attack { target, from }) = w.choose(rng, &mut value) {
            melee(&w, planned, (id, from), target);
        }
    }
    list.len()
}

/// A predicted move and attack of the walker `w`'s stack `s` from `s_hex` on `t` (0x4227A0;
/// BAT-AI-015): `s`'s [`strike`]; `t`'s retaliation from its position when `s`'s strikes are
/// retaliated, `t` is not turned to stone, has a retaliation left and the game is above Easy or
/// the side's player human (whether `t` is still alive is not asked: one predicted dead strikes
/// with no hit points, so not at all); then a second strike of a creature that strikes twice
/// while `t`'s planned damage is below its hit points.
fn melee(w: &Walker<'_>, planned: &mut Planned, (s, s_hex): (StackId, Hex), t: StackId) {
    let state = w.state;
    let (me, them) = (&state.stacks()[s.index()], &state.stacks()[t.index()]);
    let hp = computer::hit_points(state, t);
    strike(w, planned, (s, s_hex), (t, them.hex), w.side);
    let retaliates = !me.has(StackTraits::NO_ENEMY_RETALIATION)
        && !them.spells.status.contains(Status::PETRIFIED)
        && them.retaliations_left > 0
        && w.normal;
    if retaliates {
        strike(w, planned, (t, them.hex), (s, s_hex), w.side.opponent());
    }
    if me.has(StackTraits::STRIKES_TWICE) && planned.on(t) < hp {
        strike(w, planned, (s, s_hex), (t, them.hex), w.side);
    }
}

/// A predicted strike of `s` from `s_hex` on `t` at `t_hex` for `side` (0x4224E0; BAT-AI-015), in
/// the walker `w`'s battle and map. Each part strikes only while `s` has hit points left after
/// the planned damage, by the creatures those give (no shot, capped), and plans the damage on
/// the stack struck. A multi-headed `s` strikes, from the last direction of its pattern
/// ([`hexes::pattern`]) to the first, each stack beside it there once that is not of `side` (its
/// own side, not its controller), without a run-up, and Fire Shield's reflection of each strike
/// ([`computer::fire_shield_estimate`] of the damage on the stack's hit points left) is planned on
/// `s`. Any other strikes `t` with the cost of `s_hex` in `w`'s map as the run-up (the original's
/// last search: for the retaliation of `t` at its position, the attack mark's cost), with Fire
/// Shield's reflection alike; then a breathing `s` the stack on the hex beyond the one it strikes
/// in that direction, unless it is `t`, of either side, without a run-up or Fire Shield.
fn strike(
    w: &Walker<'_>,
    planned: &mut Planned,
    (s, s_hex): (StackId, Hex),
    (t, t_hex): (StackId, Hex),
    side: BattleSide,
) {
    let (state, ruleset, field) = (w.state, w.ruleset, w.field);
    let me = &state.stacks()[s.index()];
    let (shape, t_shape) = (Shape::of(me), Shape::of(&state.stacks()[t.index()]));
    let hit = |planned: &Planned, x: StackId, run_up: u8| {
        let count = creatures(me, planned.left(state, s));
        computer::damage_estimate(state, ruleset, (s, x), count, false, run_up)
    };
    let three = me.has(StackTraits::THREE_HEADED);
    if me.has(StackTraits::ALL_ADJACENT) || three {
        let pattern = hexes::pattern(field, (shape, s_hex, three), (t_shape, t_hex));
        let mut counted = 0u32;
        for d in (0..8u8).rev().filter(|d| pattern & 1 << d != 0) {
            let Some(x) =
                hexes::stack_neighbor(field, shape, s_hex, d).and_then(|n| w.on[n.index()])
            else {
                continue;
            };
            let bit = 1u32 << w.record(x);
            if counted & bit != 0 || state.stacks()[x.index()].side == side {
                continue;
            }
            counted |= bit;
            if planned.left(state, s) == 0 {
                continue;
            }
            let damage = hit(planned, x, 0);
            let left = planned.left(state, x);
            let back = computer::fire_shield_estimate(state, ruleset, (s, x), damage, left);
            planned.add(state, s, back);
            planned.add(state, x, damage);
        }
        return;
    }
    if planned.left(state, s) > 0 {
        let run_up = u8::try_from(w.map.cost(s_hex)).unwrap_or(u8::MAX);
        let damage = hit(planned, t, run_up);
        let left = planned.left(state, t);
        let back = computer::fire_shield_estimate(state, ruleset, (s, t), damage, left);
        planned.add(state, s, back);
        planned.add(state, t, damage);
    }
    if !me.has(StackTraits::BREATH) {
        return;
    }
    let beyond = hexes::direction_to(field, (shape, s_hex), (t_shape, t_hex)).and_then(|d| {
        hexes::stack_neighbor(field, shape, s_hex, d)
            .and_then(|first| hexes::neighbor(field, shape, first, d))
    });
    if let Some(x) = beyond.and_then(|h| w.on[h.index()]).filter(|&x| x != t)
        && planned.left(state, s) > 0
    {
        let damage = hit(planned, x, 0);
        planned.add(state, x, damage);
    }
}
