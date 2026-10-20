//! The original's battle AI (ADR-0027, E23): the policy of a battle side that the original's
//! computer plays, one pure, deterministic function of the battle's state, its ruleset, the
//! side to act and the caller's stream ([`choose`]). It reads the whole state, never an
//! observation, and writes nothing (ADR-0027 point 4); the computer's numbers the battle's rules
//! use come from `h3-battle`'s public [`computer`] module, one implementation for both (point 6).
//! Its callers are the AI lane's batch (`Policy::System`) and the adventure's neutral seat; they
//! get it once a side without a hero is complete (E23 T4).
//!
//! So far (E23 T2a): the stack's action of the original's dispatch (0x477EE0, 0x4221F0;
//! `docs/formats/installation.md` §7.34) for a stack that can shoot now, which shoots the
//! computer's target (0x41F060, 0x41EB80; §7.33, the automatic Ballista's). The rest stands in
//! with a legal command until its task builds it: a walker defends (BAT-STUB-4, T3); the hero's
//! side casts nothing, ends the tactics phase at once and shoots the first wall with a commanded
//! Catapult (BAT-STUB-5, T5 and T6); an area shooter and the Cyclops shoot as the others
//! (BAT-STUB-6, T2b).
//!
//! Logic crate: pure, deterministic, `no_std` + `alloc` (ADR-0003, ADR-0005). The lint header
//! below is required and checked by `cargo xtask check-arch`.
#![no_std]
#![forbid(unsafe_code)]
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::todo,
    clippy::unimplemented,
    clippy::unreachable,
    clippy::float_arithmetic,
    clippy::print_stdout,
    clippy::print_stderr,
    clippy::exit
)]
#![warn(missing_docs)]

#[cfg(test)]
extern crate std;

mod shooter;

#[cfg(test)]
mod tests;

use h3_battle::{BattleSide, BattleState, Command, LegalCommands, Stack, computer, legal_commands};
use h3_content_battle::BattleRuleset;
use h3_kernel::Rng;

/// The policy's behaviour version (ADR-0027 point 12): bumped with every change of a choice, so
/// that a training record names its opponent (the AI lane's engine identity includes it).
/// Replays store commands and do not change with it.
pub const POLICY_VERSION: u32 = 1;

/// The command the original's computer gives for `side`, the side to act in `state` (the active
/// stack's controller, BAT-MAG-026), whatever `BattleSetup.human` says (ADR-0027 point 8):
/// [`unchecked`]'s choice when the engine accepts it, else Defend (ADR-0027 point 3; no such
/// choice is known, the property tests assert none). Its draws come from `rng`, the caller's
/// stream, never the battle's (point 5); the same state and stream give the same command. For a
/// side not to act, or a battle that is over, it gives Defend, which the engine refuses.
// rule: BAT-AI-001
pub fn choose(
    state: &BattleState,
    ruleset: &BattleRuleset,
    side: BattleSide,
    rng: &mut Rng,
) -> Command {
    checked(state, ruleset, side, unchecked(state, ruleset, side, rng))
}

/// The original's computer's choice for `side` before validation (the property tests assert the
/// engine accepts it): the stack's action (0x477EE0, 0x4221F0; BAT-AI-001). A stack that can shoot
/// now ([`computer::shoots`]; the original's Ballista always can) shoots ([`shooter::shoot`],
/// BAT-AI-002); any other is a walker, which defends until E23 T3 builds its choice. The stream is
/// not drawn from yet (the walker's draw, T3).
// rule: BAT-AI-001
pub fn unchecked(
    state: &BattleState,
    ruleset: &BattleRuleset,
    side: BattleSide,
    _rng: &mut Rng,
) -> Command {
    let Some(id) = state.active().filter(|_| state.side_to_act() == Some(side)) else {
        return Command::Defend;
    };
    if state.tactics().is_some() {
        // stub: BAT-STUB-5
        // The shooters' placement and the walkers' choice in the zone (§7.34): T6.
        return Command::EndTactics;
    }
    if computer::shoots(state, ruleset, id) {
        return shooter::shoot(state, ruleset, id);
    }
    if state.stack(id).is_some_and(Stack::is_catapult) {
        // stub: BAT-STUB-5
        // The commanded Catapult's aim (§7.29, the gate first): T6.
        return first_wall_shot(state, ruleset, side);
    }
    // stub: BAT-STUB-4
    // The walker's choice (0x421F80, 0x421680; §7.35 to §7.38) and the First Aid Tent's patient
    // (§7.14, BAT-Q51): T3 and T6; until then the original's default action, Defend.
    Command::Defend
}

/// `command` when the engine accepts it from `side`, else Defend (ADR-0027 point 3).
fn checked(
    state: &BattleState,
    ruleset: &BattleRuleset,
    side: BattleSide,
    command: Command,
) -> Command {
    if h3_battle::validate(state, ruleset, side, command).is_ok() {
        command
    } else {
        Command::Defend
    }
}

/// The first wall shot the engine lists for `side`'s commanded Catapult (a stand-in, BAT-STUB-5),
/// or Defend without one.
fn first_wall_shot(state: &BattleState, ruleset: &BattleRuleset, side: BattleSide) -> Command {
    let mut legal = LegalCommands::new();
    legal_commands(state, ruleset, side, &mut legal);
    legal
        .iter()
        .copied()
        .find(|c| matches!(c, Command::ShootWall { .. }))
        .unwrap_or(Command::Defend)
}
