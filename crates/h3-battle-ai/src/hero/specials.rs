//! The hero's special spells, 0x43BB20's cases of their own (`docs/formats/installation.md`
//! §7.55, §7.57; BAT-AI-019; E23 T5b1): Resurrection and Animate Dead ([`raise`], 0x43ACA0),
//! Sacrifice ([`sacrifice`], 0x43B1E0 with 0x43AF50) and Earthquake ([`earthquake`], 0x43B8F0).
//! Teleport (0x43AA60) has a module of its own ([`super::teleport`]). Every value is a 32-bit integer as the
//! original's; the policy casts only at the targets the engine takes (ADR-0027 point 3).

use h3_battle::{
    BattleSide, CastTarget, CastTargets, Stack, StackId, StackTraits, Status, computer,
    second_targets,
};
use h3_content::{ORIGINAL_AI_SLIPS, SpellId};
use h3_content_battle::BattleSpell;
use h3_kernel::arith::X87;

use super::{Hero, Valued};
use crate::spells::Candidate;

/// The least wall-shot target's hit points before the first is counted (0x43B927).
const NO_PART: i32 = 0x7FFF;

/// A count or hit points as the original's signed 32 bits.
fn int(n: u32) -> i32 {
    i32::try_from(n).unwrap_or(i32::MAX)
}

/// The creatures a raise of `n` brings back to stack `st` (0x43ADBE to 0x43AE06; Sacrifice's
/// 0x43B023 to 0x43B065): `l` its creatures lost since the start; more than `l` counts only when
/// `l` is at least `trunc(3 × start / 4)` or the enemy is `beaten`, else the stack is passed over;
/// `k = min(n, l)`, none below 1. The top creature's lost hit points are not counted.
fn creatures(st: &Stack, n: i32, beaten: bool) -> Option<i32> {
    let start = int(st.base_count);
    let lost = start.wrapping_sub(int(st.count));
    if n > lost && lost < start.wrapping_mul(3) / 4 && !beaten {
        return None;
    }
    Some(n.min(lost)).filter(|&k| k >= 1)
}

/// `k` creatures of stack `t` raised: `trunc(k × w)`, `w` one creature's worth (0x442A50 with the
/// context's least bonuses, no desperate worth) with the shot flag of stack `shot_of` (0x4428F0).
fn raised_worth(h: &Hero<'_, '_>, (t, shot_of): (StackId, StackId), k: i32) -> i32 {
    let shot = computer::shoots(h.state, h.ruleset, shot_of);
    let w = computer::worth(h.state, h.ruleset, h.sc.view().context(), t, shot);
    computer::ftol(w.times(X87::int(u64::try_from(k).unwrap_or(0))))
}

/// Whether a raise of stack `t` is due now (0x43AE77 to 0x43AF23): `t` is the active stack, the
/// predictions beat the enemy, or no other stack of the side can still act this round.
fn due(h: &Hero<'_, '_>, t: StackId) -> bool {
    t == h.active || h.sc.beaten() || h.last_to_act()
}

/// The value doubled when the side is overwhelmingly the stronger ([`SpellContext::raises_double`]
/// (crate::spells::SpellContext::raises_double)).
fn doubled(h: &Hero<'_, '_>, v: i32) -> i32 {
    if h.sc.raises_double() {
        v.wrapping_add(v)
    } else {
        v
    }
}

/// Resurrection's or Animate Dead's value (0x43ACA0; BAT-AI-019), its candidate `c`: over the
/// side's stacks in record order that the engine takes as the raise's target (living or a corpse
/// on top, BAT-MAG-035), the creatures raised ([`creatures`]) of `n` = (power factor × power + the
/// value at the mastery) / the stack's hit points per creature, worth `trunc(k × w)` with the
/// stack's own shot flag ([`raised_worth`]), doubled when the side is overwhelmingly the stronger;
/// a strictly larger one takes the stack, due now as [`due`] says. A raise that lasts only for the
/// battle at this mastery (Resurrection below Advanced) is worth nothing once the predictions beat
/// the enemy.
// rule: BAT-AI-019
pub(super) fn raise(h: &Hero<'_, '_>, (id, s): (SpellId, &BattleSpell), c: Candidate) -> Valued {
    let beaten = h.sc.beaten();
    if beaten && s.magic.temporary_at(c.mastery) {
        return Valued::ZERO;
    }
    let targets = h.targets(id);
    let at = usize::from(c.mastery.min(3));
    let hit_points = i32::from(s.power)
        .wrapping_mul(i32::try_from(c.power).unwrap_or(i32::MAX))
        .wrapping_add(i32::from(s.effect[at]));
    let mut best = Valued::ZERO;
    for (i, st) in (0u8..).zip(h.state.stacks()) {
        let r = StackId(i);
        if st.side != h.side || !targets.takes_stack(r) {
            continue;
        }
        let n = hit_points.checked_div(int(st.max_hp)).unwrap_or(0);
        let Some(k) = creatures(st, n, beaten) else {
            continue;
        };
        let v = doubled(h, raised_worth(h, (r, r), k));
        if v > best.value {
            best = Valued {
                value: v,
                target: CastTarget::Stack(r),
                second: CastTarget::None,
                now: due(h, r),
            };
        }
    }
    best
}

/// Sacrifice's value (0x43B1E0, 0x43AF50; BAT-AI-019), its candidate `c`: over the side's stacks
/// `T` in record order that the engine takes as the target (Resurrection's test with quirk
/// BAT-QUIRK-5, BAT-MAG-036), over the victims `V` the engine takes for it in record order, the
/// creatures raised ([`creatures`]) of `n` = (the value at the mastery + the hit points of `V`'s
/// creature + the power) × `V`'s count / `T`'s hit points per creature, worth `trunc(k × w_T)`
/// less `V`'s strength (0x442E60), only above 0, then doubled as the raises'; a strictly larger
/// one takes `T` and `V`, due now as the raises' for `T`. With quirk BAT-QUIRK-25 `w_T` takes
/// `V`'s shot flag (survey slip 28), else `T`'s own.
// rule: BAT-AI-019
pub(super) fn sacrifice(
    h: &Hero<'_, '_>,
    (id, s): (SpellId, &BattleSpell),
    c: Candidate,
) -> Valued {
    let (state, ruleset) = (h.state, h.ruleset);
    let beaten = h.sc.beaten();
    let targets = h.targets(id);
    let at = usize::from(c.mastery.min(3));
    let base = i32::from(s.effect[at]).wrapping_add(i32::try_from(c.power).unwrap_or(i32::MAX));
    let slips = ruleset.quirks().is_enabled(ORIGINAL_AI_SLIPS);
    let ctx = h.sc.view().context();
    let mut victims = CastTargets::new();
    let mut best = Valued::ZERO;
    for (i, ts) in (0u8..).zip(state.stacks()) {
        let t = StackId(i);
        if ts.side != h.side || !targets.takes_stack(t) {
            continue;
        }
        second_targets(
            state,
            ruleset,
            h.side,
            (id, CastTarget::Stack(t)),
            &mut victims,
        );
        for v in victims
            .as_slice()
            .iter()
            .copied()
            .filter_map(CastTarget::stack)
        {
            let vs = &state.stacks()[v.index()];
            let health = ruleset
                .creature(vs.creature)
                .map_or(0, |cr| int(cr.hit_points));
            let n = base
                .wrapping_add(health)
                .wrapping_mul(int(vs.count))
                .checked_div(int(ts.max_hp))
                .unwrap_or(0);
            let Some(k) = creatures(ts, n, beaten) else {
                continue;
            };
            // The victim's shot flag with BAT-QUIRK-25 (survey slip 28: 0x43B06F).
            let shot_of = if slips { v } else { t };
            let value = raised_worth(h, (t, shot_of), k)
                .wrapping_sub(computer::strength(state, ruleset, ctx, v));
            if value <= 0 {
                continue;
            }
            let value = doubled(h, value);
            if value > best.value {
                best = Valued {
                    value,
                    target: CastTarget::Stack(t),
                    second: CastTarget::Stack(v),
                    now: due(h, t),
                };
            }
        }
    }
    best
}

/// Earthquake's value (0x43B8F0; BAT-AI-019), its candidate `c`: nothing for the defender, once
/// the predictions beat the enemy, or without walls' hit points (`W`, the sum over the wall-shot
/// targets, the absent turrets' too, BAT-QUIRK-22), nor unless a living stack of the other side
/// but the turrets stands inside the walls at its position. Else over the side's stacks in record
/// order but the dead, those that lose their turns, the First Aid Tent and the Ammo Cart: one
/// without a threat enemy (the spell context's records) adds its strength (0x442E60) while no
/// target is destroyed (the least target's hit points above 0); one with a threat enemy `E` that
/// can shoot now and whose shot at `E` the walls halve adds `trunc(s × h / 3W)`, `h` the smaller
/// of `W` and the spell's value at the mastery (32 bits). With quirk BAT-QUIRK-25 the walls are
/// tested from the stack's position (survey slip 29), else from the hex the rule turns it to. Due
/// now, no target.
// rule: BAT-AI-019
pub(super) fn earthquake(h: &Hero<'_, '_>, s: &BattleSpell, c: Candidate) -> Valued {
    let (state, ruleset) = (h.state, h.ruleset);
    if h.side == BattleSide::Defender || h.sc.beaten() {
        return Valued::ZERO;
    }
    let Some(siege) = computer::siege(state) else {
        return Valued::ZERO;
    };
    let (walls, least) = siege
        .parts
        .iter()
        .take(usize::from(siege.targets))
        .fold((0i32, NO_PART), |(w, m), &p| {
            (w.wrapping_add(i32::from(p)), m.min(i32::from(p)))
        });
    let other = h.side.opponent();
    let inside = state
        .stacks()
        .iter()
        .any(|st| st.side == other && st.is_alive() && !st.is_turret() && siege.inside(st.hex));
    if walls == 0 || !inside {
        return Valued::ZERO;
    }
    let hits = i32::from(s.effect[usize::from(c.mastery.min(3))]).min(walls);
    let slips = ruleset.quirks().is_enabled(ORIGINAL_AI_SLIPS);
    let ctx = h.sc.view().context();
    let mut sum = 0i32;
    for (i, st) in (0u8..).zip(state.stacks()) {
        let id = StackId(i);
        if st.side != h.side
            || !st.is_alive()
            || st.spells.status.contains(Status::NO_TURNS)
            || st.has(StackTraits::FIRST_AID)
            || st.has(StackTraits::AMMO_SUPPLY)
        {
            continue;
        }
        let gain = match h.sc.threat_enemy(id) {
            None if least > 0 => computer::strength(state, ruleset, ctx, id),
            None => 0,
            Some(e) => {
                // The stack's position with BAT-QUIRK-25 (survey slip 29: 0x43BAB4).
                let halved = || {
                    if slips {
                        computer::walls_halve_from_position(state, id, e)
                    } else {
                        computer::walls_halve(state, id, e)
                    }
                };
                if computer::shoots(state, ruleset, id) && halved() {
                    computer::strength(state, ruleset, ctx, id)
                        .wrapping_mul(hits)
                        .checked_div(walls.wrapping_mul(3))
                        .unwrap_or(0)
                } else {
                    0
                }
            }
        };
        sum = sum.wrapping_add(gain);
    }
    Valued::untargeted(sum)
}
