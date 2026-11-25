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
//! of a turn is stubbed by `AI-STUB-2` to `-6` (`docs/stubs.toml`).
//!
//! [`GameCommand`]: crate::GameCommand

mod memory;
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
