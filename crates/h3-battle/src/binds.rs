//! Bind (E04b T7a3; BAT-MAG-045), after the original's code (`docs/formats/installation.md`
//! §7.20): a stack under Bind cannot move while it lasts (it still attacks next to it); the stacks
//! whose strike bound it are its binders, and its Bind ends when its last binder moves, dies or
//! is teleported, when it is teleported, or as an effect ends (Dispel, its 255 rounds). Kept
//! beside the stacks: the hot paths read one mask.

use h3_content_battle::{BattleRuleset, SpellEffect, TimedChange};
use h3_kernel::{CanonicalHash, StateHasher};

use crate::effects;
use crate::event::Sink;
use crate::state::{BattleState, STACK_CAPACITY, StackId};

// A stack's bit fits a `u64`.
const _: () = assert!(STACK_CAPACITY <= 64);

/// What Bind keeps per stack, by stack id.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Binds {
    /// The living stacks under Bind, a bit per stack id (materialised from the effects).
    pub(crate) bound: u64,
    /// Per stack, its binders: the stacks whose strike bound it, a bit per stack id (the
    /// original's list +0x504; a binder's own list +0x4F4 is the column of its bit). Empty
    /// unless the stack is under Bind.
    pub(crate) by: [u64; STACK_CAPACITY],
}

impl Default for Binds {
    fn default() -> Self {
        Self {
            bound: 0,
            by: [0; STACK_CAPACITY],
        }
    }
}

impl CanonicalHash for Binds {
    /// The stacks under Bind, then per stack its binders (v32).
    fn hash_into(&self, h: &mut StateHasher) {
        h.write_u64(self.bound);
        for &b in &self.by {
            h.write_u64(b);
        }
    }
}

/// Stack `s`'s bit.
const fn bit(s: StackId) -> u64 {
    1 << s.0
}

/// Whether stack `s` is under Bind: it cannot move (BAT-MAG-045; the original's pathfinder,
/// 0x4B2DE0 and 0x4B350B, gives it no movement).
pub(crate) const fn is_bound(state: &BattleState, s: StackId) -> bool {
    state.binds.bound & bit(s) != 0
}

/// Makes `binder` one of `target`'s binders before its strike's Bind is cast on it
/// (BAT-MAG-045). Returns `false`, changing nothing, if it already is one: the strike then does
/// nothing (0x440500).
pub(crate) fn join(state: &mut BattleState, binder: StackId, target: StackId) -> bool {
    let by = &mut state.binds.by[target.index()];
    let new = *by & bit(binder) == 0;
    *by |= bit(binder);
    new
}

/// Sets whether stack `s` is under Bind as its effects say (`bound`) and it lives; one that is
/// not has no binders left (the original's end routine 0x444510 empties its list and takes it
/// off its binders' lists).
pub(crate) fn materialize(state: &mut BattleState, s: StackId, bound: bool) {
    let bound = bound && state.at(s).is_alive();
    let binds = &mut state.binds;
    if bound {
        binds.bound |= bit(s);
    } else {
        binds.bound &= !bit(s);
        binds.by[s.index()] = 0;
    }
}

/// Releases the stacks `binder` binds (BAT-MAG-045; the original's 0x43EE10): it leaves each one's
/// binders, and one left without binders has its Bind ended, reported. Called when the binder
/// moves, dies or is teleported.
// rule: BAT-MAG-045
pub(crate) fn release<S: Sink>(
    state: &mut BattleState,
    ruleset: &BattleRuleset,
    binder: StackId,
    sink: &mut S,
) {
    // Only a stack under Bind has binders.
    if state.binds.bound == 0 {
        return;
    }
    for i in 0..state.stacks.len() {
        let by = &mut state.binds.by[i];
        if *by & bit(binder) == 0 {
            continue;
        }
        *by &= !bit(binder);
        if *by == 0 {
            end(
                state,
                ruleset,
                StackId(u8::try_from(i).unwrap_or(u8::MAX)),
                sink,
            );
        }
    }
}

/// Ends the Bind on stack `s`, reported (Teleport, BAT-MAG-045; the end of its binders).
pub(crate) fn end<S: Sink>(
    state: &mut BattleState,
    ruleset: &BattleRuleset,
    s: StackId,
    sink: &mut S,
) {
    let bind = SpellEffect::Timed(TimedChange::Bind);
    effects::end_spells(state, ruleset, s, sink, |sp| {
        sp.magic.effects.contains(&bind)
    });
}

/// After stack `s` died (the original's 0x444120 calls 0x43EE10): the stacks it binds are
/// released, and it is no longer under Bind (a dead stack loses its timed effects, BAT-MAG-010).
pub(crate) fn died<S: Sink>(
    state: &mut BattleState,
    ruleset: &BattleRuleset,
    s: StackId,
    sink: &mut S,
) {
    release(state, ruleset, s, sink);
    materialize(state, s, false);
}
