//! The original's computer player on the adventure map (E31; `docs/epics/E31-original-computer-
//! player.md`): the turns of a player the original's computer plays, read from `Heroes3.exe`
//! (SoD's code) and ported as a policy outside the game state, one validated [`GameCommand`]
//! a call. Its rules are `CPU-` ids (`docs/rules/catalog/computer.toml`), its quirks `AI-QUIRK-`
//! (`docs/rules/quirks-ai.md`), its values in the original's x87 arithmetic
//! (`h3_kernel::arith::X87`, `SignedX87`).
//!
//! The ai lane's by path (`lanes.toml`; the user, AI-Q31): a module of `h3-game` for now, using
//! only the crate's public surface and the crates below it, so that its later move into a crate of
//! its own changes no choice. The design: ADR-0030 (`docs/adr/0030-original-computer-player.md`).
//!
//! Built so far (E31 increment 2): [`choose`] and [`Memory`] (ADR-0030 points 2, 5), the turn's
//! frame (T2a: the passes, the order of heroes, a hero's turn with its garrison step); the rest
//! of a turn is stubbed by `AI-STUB-2` to `-6` (`docs/stubs.toml`). The fight estimate (T2c,
//! `fight`) on the simulated combat, which lives in `h3_battle::estimate` (T2b and T2c, AI-Q34),
//! waits for the values that ask for it (T2h, T2i); the turn start's records (T2d, `start`); a
//! hero's spell gains (T2e, `spells`, on `h3_battle::estimate`'s caster record, AI-Q36) and its
//! skills' values (T2e, `skills`, the flag-set path of `h3_heroes::choice`'s switch); the artifacts'
//! values (T2f, `artifacts`: the effects' table, the values to a hero and to a player, the turn
//! start's mean); the army plan's values (T2g, `armies`: a stack's worth, the exchange, the
//! purchase) and the dwelling and town values on them (T2g, `towns`), waiting for the object value
//! that asks for them (T2h).
//!
//! [`GameCommand`]: crate::GameCommand

// The army plan's values have no caller until the town and dwelling values' callers (T2h).
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "the army plan's values have no caller until the object value, E31 T2h"
    )
)]
mod armies;
mod artifacts;
// The values that ask for a fight's worth (T2h, T2i) are the fight estimate's first callers
// outside the tests; the turn's start calls the capture estimate beside it (T2d).
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "the fight estimate has no caller until the values, E31 T2h, T2i"
    )
)]
mod fight;
mod memory;
mod order;
// The skills' values have no caller until the values that ask for them (T2g's town buildings,
// T2h's object value: the University, the Witch Hut, the Seer's reward).
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "the skills' values have no caller until the values, E31 T2g, T2h"
    )
)]
mod skills;
mod spells;
mod start;
// The town and dwelling values are the object value's (T2h), their only callers.
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "the town and dwelling values have no caller until the object value, E31 T2h"
    )
)]
mod towns;
mod turn;

#[cfg(test)]
mod tests;

use h3_contracts::PlayerId;
use h3_kernel::Rng;

pub use memory::{MEMORY_ENCODING_VERSION, Memory, MemoryError};

use crate::{GameCommand, GameRuleset, GameState, default_answer};

/// The policy's behaviour version (ADR-0030 point 13): bumped with every change of a choice, so
/// that a record names its computer player (the ai lane's engine identity includes it). Replays
/// store commands and do not change with it. v1 (E31 T2a): the turn's frame, each hero done at
/// once, the garrison steps, the open decisions' default answers.
pub const POLICY_VERSION: u32 = 1;

/// The next command of `player`, a player the original's computer plays, whatever its `human`
/// says (ADR-0030 points 2, 11; CPU-TRN-001), with its memory `memory` and the caller's stream
/// `rng` (point 8): `None` while another player's level-up holds the game (GAM-DEC-002) and for a
/// battle, whose side the host's battle seat plays (point 7: `h3_battle_ai::choose`, or
/// `choose_fair` in the fair variant, as the player's `GameCommand::Battle`); an open decision's
/// answer, in its turn or not ([`default_answer`] until increment 4, `AI-STUB-6`); in its turn the
/// next command of the turn's frame (`ai_player_turn` 0x525E80, CPU-TRN-002), the last `EndTurn`;
/// `None` outside its turn. One call, one command: the host applies it as the player's and calls
/// again. Every command is one `apply` accepts (point 3; the property tests assert it); the same
/// state, memory and stream give the same command and memory.
// rule: CPU-TRN-001
// Never inlined: the gated benchmark counts the instructions inside it (ADR-0030 point 13).
#[inline(never)]
pub fn choose(
    state: &GameState,
    rules: &GameRuleset,
    player: PlayerId,
    memory: &mut Memory,
    rng: &mut Rng,
) -> Option<GameCommand> {
    if state.waiting_for().is_some_and(|owner| owner != player) {
        return None;
    }
    if state.has_decision(player) {
        // stub: AI-STUB-6
        return default_answer(state, rules, player).map(GameCommand::Answer);
    }
    state
        .is_acting(player)
        .then(|| turn::step(state, rules, player, memory, rng))
}
