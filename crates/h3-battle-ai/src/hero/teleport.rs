//! The hero's Teleport, 0x43BB20's case of its own (`docs/formats/installation.md` §7.55, §7.58;
//! BAT-AI-020; E23 T5b2a): each stack of the side it may move is asked its walker's turn twice,
//! as it is and on the destination map, and the gain of the second is Teleport's value there. The
//! values are 32-bit integers as the original's; the policy casts only at the targets the engine
//! takes (ADR-0027 point 3).

use h3_battle::second_targets;
use h3_battle::{CastTarget, CastTargets, Command, HexSet, StackFlags, StackId, Status, computer};
use h3_content::SpellId;
use h3_kernel::Rng;

use super::{Hero, Valued};
use crate::walker::Walker;

/// Teleport's value (0x43AA60; BAT-AI-020) for the spell `spell`: over the side's stacks in record
/// order, alive, not acted this round, that the cast can affect ([`computer::spell_affects`],
/// 0x5A3C80) and that cannot shoot now, the stack's walker's turn as it is (`v₀`) and on the
/// destination map of the hexes Teleport may move it to ([`Walker::teleport_turns`]: both turns
/// with their draws from `rng`, the second's map at the stops of the last search with
/// BAT-QUIRK-25, survey slip 30). When the second attacks, `v₁ − v₀` (32 bits) strictly above the
/// best so far (from 0) takes the stack as the target and its plan's hex as the destination, at
/// a stack the engine takes. Due now when the stack is the active one or blind, petrified or
/// paralysed, or when no other stack of the side can act this round: never otherwise, as the
/// stack itself still can. Once a walker was asked the original fills both sides' threat records
/// again (0x422B20 with flag 0), which leaves them as the spell context built them (until the
/// retreat test's predictions, T6: survey slip 26).
// rule: BAT-AI-020
pub(super) fn teleport(h: &Hero<'_, '_>, spell: SpellId, rng: &mut Rng) -> Valued {
    let (state, ruleset, side) = (h.state, h.ruleset, h.side);
    let targets = h.targets(spell);
    let mut best = Valued::ZERO;
    let mut list = CastTargets::new();
    for (i, st) in (0u8..).zip(state.stacks()) {
        let r = StackId(i);
        if st.side != side
            || !st.is_alive()
            || st.flags.contains(StackFlags::ACTED)
            || !computer::spell_affects(state, ruleset, side, spell, r)
            || computer::shoots(state, ruleset, r)
        {
            continue;
        }
        second_targets(
            state,
            ruleset,
            side,
            (spell, CastTarget::Stack(r)),
            &mut list,
        );
        let destinations = list.as_slice().iter().fold(HexSet::new(), |mut set, t| {
            if let CastTarget::Hex(hex) = *t {
                set.insert(hex);
            }
            set
        });
        let (before, after) = Walker::teleport_turns(state, ruleset, (r, side), destinations, rng);
        // The walker never attacks itself (BAT-STUB-10): the original's test that the attacked
        // stack is not the one moved (0x43AB34) always passes.
        let (Command::Attack { from, .. }, value) = after else {
            continue;
        };
        let d = value.wrapping_sub(before);
        if d > best.value && targets.takes_stack(r) {
            let disabled = st.spells.status.contains(Status::NO_TURNS);
            best = Valued {
                value: d,
                target: CastTarget::Stack(r),
                second: CastTarget::Hex(from),
                now: r == h.active || disabled || h.last_to_act(),
            };
        }
    }
    best
}
