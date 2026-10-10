//! Raising spells (E04b T6a; the original's code, `docs/formats/installation.md` §7.18):
//! Resurrection and Animate Dead raise the creatures of a living or dead stack of the caster's
//! side (BAT-MAG-035), Sacrifice kills one of the side's stacks to raise another (BAT-MAG-036).
//! A dead stack lies on its hexes as a corpse, the last to die on top
//! ([`BattleState::corpse`]). The Phoenix's rebirth raises it the same way (BAT-MAG-059).

use h3_content::{Ability, Ruleset, SACRIFICE_TESTS_AS_RESURRECTION, Spell};
use h3_contracts::BattleSide;

use crate::damage::resurrect;
use crate::event::{Event, EventKind, Sink};
use crate::geometry::{Field, HexSet};
use crate::hero::HeroRecord;
use crate::magic::{
    Guard, SpellBonus, SpellRef, affects, barred, base_value, kept_off, mastery, specialist_bonus,
};
use crate::state::{BattleState, NO_CORPSE, StackFlags, StackId, StackTraits};

/// Marks the corpses of the stacks that died since the last mark, by stack id, each above every
/// earlier one (BAT-MAG-035): after every command, before every activation and after every
/// automatic berserk action, so the stacks marked together died in one action and lie on no
/// common hex. A stack the battle created leaves none (BAT-MAG-037).
pub(crate) fn mark_corpses(state: &mut BattleState) {
    let corpses = &mut state.raised.corpse;
    let fresh = |s: &crate::state::Stack, c: u16| !s.is_alive() && c == 0;
    if !state
        .stacks
        .iter()
        .zip(corpses.iter())
        .any(|(s, &c)| fresh(s, c))
    {
        return;
    }
    let mut top = corpses
        .iter()
        .copied()
        .filter(|&c| c != NO_CORPSE)
        .max()
        .unwrap_or(0);
    for (s, c) in state.stacks.iter().zip(corpses.iter_mut()) {
        if fresh(s, *c) && s.is_created() {
            // rule: BAT-MAG-037
            *c = NO_CORPSE;
        } else if fresh(s, *c) {
            top = top.saturating_add(1).min(NO_CORPSE - 1);
            *c = top;
        }
    }
}

/// How a cast tests the stack it raises (the original's 0x5A8090 for Resurrection and Animate
/// Dead, case 0x5A8467): the spell whose conditions, level and immunities apply and, unless the
/// test has none, the hit points the cast raises, with the caster's bonuses for that spell, which
/// must reach one creature's health.
#[derive(Clone, Copy)]
pub(crate) struct RaiseTest<'r> {
    spell: SpellRef<'r>,
    hit_points: Option<(u64, SpellBonus)>,
}

/// The test of a stack raiser's raise (BAT-MAG-051; the original's 0x5A3CC0 with its third
/// argument 1, `installation.md` §7.21): the raise spell `spell`'s, with the `hit_points` the
/// raiser raises and no caster's bonuses.
pub(crate) const fn creature_test(spell: SpellRef<'_>, hit_points: u64) -> RaiseTest<'_> {
    RaiseTest {
        spell,
        hit_points: Some((
            hit_points,
            SpellBonus {
                spell: spell.0,
                damage_bp: 0,
                per_level_bp: 0,
                level: 0,
            },
        )),
    }
}

/// The test of `hero`'s cast of `spell` at `mastery` for the stack it raises: the spell's own with
/// its hit points (BAT-MAG-035); for a sacrifice, with quirk BAT-QUIRK-5 that of the spell it tests
/// as (Resurrection's, at the hero's mastery in it), else its own without hit points
/// (BAT-MAG-036).
pub(crate) fn test_for<'r>(
    hero: &HeroRecord,
    ruleset: &'r Ruleset,
    ((id, spell), at): (SpellRef<'r>, u8),
) -> RaiseTest<'r> {
    let as_spell = spell
        .tests_as
        .filter(|_| ruleset.quirks().is_enabled(SACRIFICE_TESTS_AS_RESURRECTION))
        .and_then(|t| ruleset.spell(t).map(|s| (t, s)));
    match (spell.magic.sacrifices(), as_spell) {
        (true, None) => RaiseTest {
            spell: (id, spell),
            hit_points: None,
        },
        (true, Some((t, s))) => RaiseTest {
            spell: (t, s),
            hit_points: Some((
                base_value(hero, s, mastery(hero, t)),
                hero.magic.bonuses.spell(t),
            )),
        },
        (false, _) => RaiseTest {
            spell: (id, spell),
            hit_points: Some((base_value(hero, spell, at), hero.magic.bonuses.spell(id))),
        },
    }
}

/// Whether stack `s` passes `side`'s raise `test` wherever it lies (BAT-MAG-035): a stack of the
/// side (its own, whoever controls it) with fewer creatures than at the battle's start, which the
/// test's spell can affect (the battle's spell level limit, BAT-MAG-062: an Archangel's
/// Resurrection test too; its conditions and immunities; Anti-Magic on a living stack: a dead
/// one's effects ended at its death), and whose creature's health the raised hit points reach.
fn eligible(
    state: &BattleState,
    ruleset: &Ruleset,
    side: BattleSide,
    test: RaiseTest<'_>,
    s: StackId,
) -> bool {
    let stack = state.at(s);
    stack.side == side
        && stack.count < stack.base_count
        && !barred(state, test.spell.1)
        && affects(
            ruleset,
            test.spell,
            stack.creature,
            Guard::of(state, side, s),
        )
        && !(stack.is_alive() && kept_off(stack, test.spell.1))
        && test.hit_points.is_none_or(|(value, own)| {
            let bonus = specialist_bonus(ruleset, test.spell.1, own, stack.creature, value);
            u64::from(stack.max_hp) <= value.saturating_add_signed(bonus)
        })
}

/// Whether stack `s` is a target of `side`'s raise `test` (BAT-MAG-035; the original finds the
/// stack by hex, 0x5A3CC0 and 0x5A3F50): a living stack that passes the test; a dead one that
/// passes it, whose hexes hold no living stack and are not blocked (an obstacle, Force Field), and
/// which on one of its hexes lies above every other such corpse that passes the test.
pub(crate) fn raisable(
    state: &BattleState,
    ruleset: &Ruleset,
    side: BattleSide,
    test: RaiseTest<'_>,
    s: StackId,
) -> bool {
    if state.at(s).is_alive() {
        return eligible(state, ruleset, side, test, s);
    }
    on_top(state, ruleset, s, |t| {
        eligible(state, ruleset, side, test, t)
    })
}

/// Whether the dead stack `s` is a corpse that `qualifies`, whose hexes hold no living stack and
/// are not blocked (an obstacle, Force Field), and which on one of its hexes lies above every
/// other such corpse that qualifies (BAT-MAG-035, BAT-MAG-052; the original's searches from the
/// last corpse of a hex, 0x5A3CC0, 0x5A3F50, 0x5A3E40).
fn on_top(
    state: &BattleState,
    ruleset: &Ruleset,
    s: StackId,
    qualifies: impl Fn(StackId) -> bool,
) -> bool {
    // The cheap tests first: most dead stacks are the other side's or out of the test.
    let corpse = state.corpse(s);
    if corpse == 0 || corpse == NO_CORPSE || !qualifies(s) {
        return false;
    }
    let field = Field::new(ruleset.battle().field);
    let occupied = state.blocked_for(field, s);
    if covers(field, state, s, occupied) {
        return false;
    }
    state.at(s).footprint(field).hexes().any(|h| {
        !(0u8..).take(state.stacks.len()).map(StackId).any(|t| {
            let (other, above) = (state.at(t), state.corpse(t));
            !other.is_alive()
                && above > corpse
                && above != NO_CORPSE
                && other.footprint(field).hexes().any(|x| x == h)
                && qualifies(t)
                && !covers(field, state, t, occupied)
        })
    })
}

/// Whether the dead stack `s` is a corpse a corpse raiser of `side` may raise (BAT-MAG-052; the
/// original's 0x5A3E40, `installation.md` §7.21): a stack of `side` whose creature is alive,
/// whose hexes hold no living stack and are not blocked, on top of the corpses of such stacks on
/// one of its hexes; there is no spell test.
pub(crate) fn corpse_raisable(
    state: &BattleState,
    ruleset: &Ruleset,
    side: BattleSide,
    s: StackId,
) -> bool {
    let qualifies = |t: StackId| {
        let stack = state.at(t);
        stack.side == side && stack.has(StackTraits::ALIVE)
    };
    !state.at(s).is_alive() && on_top(state, ruleset, s, qualifies)
}

/// Whether one of stack `t`'s hexes is in `set`.
fn covers(field: Field, state: &BattleState, t: StackId, set: HexSet) -> bool {
    state
        .at(t)
        .footprint(field)
        .hexes()
        .any(|h| set.contains(h))
}

/// Whether stack `v` may be the victim of `side`'s cast of the sacrifice `spell` (BAT-MAG-036;
/// the original's 0x5A8090, case 0x5A8523): a living stack of the side that the side controls
/// (not hypnotized), not created in the battle (BAT-MAG-037), which the spell can affect (the
/// spell level limit, its conditions, immunities and Anti-Magic).
pub(crate) fn victim(
    state: &BattleState,
    ruleset: &Ruleset,
    side: BattleSide,
    spell: SpellRef<'_>,
    v: StackId,
) -> bool {
    state.stack(v).is_some_and(|stack| {
        stack.is_alive()
            && stack.side == side
            && stack.controller() == side
            && !stack.is_created()
            && !barred(state, spell.1)
            && affects(ruleset, spell, stack.creature, Guard::of(state, side, v))
            && !kept_off(stack, spell.1)
    })
}

/// Raises stack `s` by `hit_points` (BAT-MAG-035): its total health grows up to its count at the
/// battle's start (the top creature first); `temporary` creatures count for this battle only. A
/// dead stack comes back on its hexes with no effects (they ended at its death) and keeps its
/// flags for the round (the original's raise 0x5A7560 clears only its dead bit).
pub(crate) fn raise<S: Sink>(
    state: &mut BattleState,
    ruleset: &Ruleset,
    (s, hit_points): (StackId, u64),
    temporary: bool,
    sink: &mut S,
) {
    let (healed, creatures) = revive(state, ruleset, s, hit_points);
    if temporary {
        let base = state.at(s).base_count;
        let t = &mut state.raised.temporary[s.index()];
        *t = t.saturating_add(creatures).min(base);
    }
    sink.emit(Event::public(EventKind::SpellRaised {
        target: s,
        hit_points: healed,
        creatures,
        temporary,
    }));
}

/// Raises stack `s` by `hit_points` for good (the original's 0x5A7560): a dead stack comes back
/// on its hexes, leaves the corpses and gets its effects recomputed (none: they ended at its
/// death, by the end of the command it died in at the latest). Returns the hit points regained
/// and the creatures brought back.
fn revive(state: &mut BattleState, ruleset: &Ruleset, s: StackId, hit_points: u64) -> (u64, u32) {
    let was_dead = !state.at(s).is_alive();
    let raised = resurrect(state.at_mut(s), hit_points);
    if was_dead && state.at(s).is_alive() {
        state.raised.corpse[s.index()] = 0;
        crate::effects::recompute(state, ruleset, s);
    }
    raised
}

/// The rebirths after an action that may have killed stacks (BAT-MAG-059; the original's 0x469440
/// at the end of a strike action, after a cast's damage and after Land Mine's and Fire Wall's
/// damage, `installation.md` §7.21), before the battle's end is checked: every stack of either
/// side that died in this action (no corpse marked yet), in stack order, with the ability
/// `rebirth` and a cast left that is not a clone uses one cast and comes back for good, without
/// the effects it died with, with its count at the battle's start div the ability's divisor, plus
/// one for each of the count mod the divisor others with a 1-in-divisor chance, and it has acted
/// this round. None comes back if none is drawn. A sacrificed stack left no corpse and is never
/// reborn (BAT-QUIRK-17: the original, which checks after every damaging action but not after
/// Sacrifice, brings it back at the next one onto its hexes, whatever stands there); nor is one
/// that died earlier (a creature with more casts that drew none: the original would bring it
/// back at a later check, alike; the Phoenix has one cast).
// rule: BAT-MAG-059
pub(crate) fn rebirths<S: Sink>(state: &mut BattleState, ruleset: &Ruleset, sink: &mut S) {
    for s in (0u8..).take(state.stacks.len()).map(StackId) {
        let stack = state.at(s);
        // The cheap tests first: most stacks cannot come back or are alive.
        if !stack.has(StackTraits::REBIRTH)
            || stack.is_alive()
            || stack.has(StackTraits::CLONE)
            || state.casts.left[s.index()] == 0
            // Sacrificed (no corpse, BAT-QUIRK-17) or dead before this action (a corpse).
            || state.corpse(s) != 0
        {
            continue;
        }
        let divisor = ruleset
            .abilities(stack.creature)
            .iter()
            .find_map(|a| match a {
                Ability::Rebirth { divisor } => Some(u32::from(*divisor)),
                _ => None,
            })
            .unwrap_or(1);
        let (start, health) = (stack.base_count, u64::from(stack.max_hp));
        crate::casters::spend(state, s);
        let drawn = (0..start % divisor)
            .filter(|_| state.rng.below(divisor) == 0)
            .count();
        let creatures = start / divisor + u32::try_from(drawn).unwrap_or(0);
        if creatures == 0 {
            continue;
        }
        state.at_mut(s).flags.set(StackFlags::ACTED, true);
        // Its effects end at its death (the original's 0x444120); the command's end has not
        // removed them yet.
        crate::effects::remove_dead(state, ruleset);
        let (_, creatures) = revive(state, ruleset, s, u64::from(creatures) * health);
        sink.emit(Event::public(EventKind::Reborn {
            stack: s,
            creatures,
        }));
    }
}

/// The hit points `hero`'s cast of a raise spell `(id, spell)` at `value` raises on stack `s`:
/// the value plus the caster's specialist bonus for the stack (BAT-MAG-020, BAT-MAG-035).
pub(crate) fn raise_value(
    state: &BattleState,
    ruleset: &Ruleset,
    (spell, own): (&Spell, SpellBonus),
    value: u64,
    s: StackId,
) -> u64 {
    let creature = state.at(s).creature;
    value.saturating_add_signed(specialist_bonus(ruleset, spell, own, creature, value))
}

/// Sacrifices stack `v` (BAT-MAG-036): all its creatures die and it leaves no corpse; returns the
/// hit points it gives at the cast's `value`: `(its creature's hit points + value) × its
/// creatures`, the creature's base hit points, not the stack's.
pub(crate) fn sacrifice<S: Sink>(
    state: &mut BattleState,
    ruleset: &Ruleset,
    v: StackId,
    value: u64,
    sink: &mut S,
) -> u64 {
    let stack = state.at_mut(v);
    let creatures = stack.count;
    let health = ruleset.creature(stack.creature).map_or(0, |c| c.hit_points);
    stack.count = 0;
    stack.top_hp = 0;
    state.raised.corpse[v.index()] = NO_CORPSE;
    sink.emit(Event::public(EventKind::Sacrificed {
        victim: v,
        creatures,
    }));
    // rule: BAT-MAG-038
    crate::created::died(state, ruleset, v, sink);
    (u64::from(health) + value).saturating_mul(u64::from(creatures))
}
