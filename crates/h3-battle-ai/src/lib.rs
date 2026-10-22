//! The original's battle AI (ADR-0027, E23): the policy of a battle side that the original's
//! computer plays, one pure, deterministic function of the battle's state, its ruleset, the
//! side to act and the caller's stream ([`choose`]). It reads the whole state, never an
//! observation, and writes nothing (ADR-0027 point 4); the computer's numbers the battle's rules
//! use come from `h3-battle`'s public [`computer`] module, one implementation for both (point 6).
//! Its callers are the AI lane's batch (`Policy::System`) and the adventure's neutral seat; they
//! get it once a side without a hero is complete (E23 T4).
//!
//! So far (E23 T2, T3, T4): the stack's action of the original's dispatch (0x477EE0, 0x4221F0;
//! `docs/formats/installation.md` §7.34). A stack that can shoot now shoots the computer's target
//! (0x41F060, 0x41EB80; §7.33, the automatic Ballista's), an area shooter by its value at a hex
//! (0x41EDA0; §7.35), and the Cyclops batters the weakest wall when much of its side's strength
//! is locked out (0x41EEA0). Any other stack is a walker (0x421F80, 0x421680; §7.35, §7.37,
//! §7.38): it values each enemy by the best hex to strike it from, its melee value and on Hard the
//! gain when it takes the enemy's last retaliation, with a draw, attacks the best within a turn
//! or walks towards it; its hex scores (0x4214F0, 0x421590, 0x420260, 0x41FB60; §7.36) weigh the
//! Fire Walls, the moat, the enemies' reach and the crowding into the plan and the walk. The odds
//! (0x420A80: the stronger at range, the hero's best damage spell and the turrets included, §7.39)
//! let a defender hold its walls and send a losing walker to the fallbacks; without a candidate
//! a besieger walks to its wall, any other guards a shooter or steps to safety (§7.38), and else
//! waits (Hard and up) or defends. Before its action and after a choice of nothing a walker
//! asks the creature cast entry (0x421280; §7.42, §7.43): the Archangel's and the Pit Lord's
//! raise when it beats the walker's value, the Faerie Dragon's drawn spell wherever it is worth
//! anything, the Ogre Mage's Bloodlust (0x420D20; §7.44 to §7.46, §7.51) at a stack as strong as
//! itself when it beats the walker's value, valued by the spell context ([`spells`]), which first
//! predicts the damage its side's stacks deal before the other side's next one ([`predict`],
//! §7.52: their turns as predictions, with their draws), and the Master Genie's drawn spell by its
//! mean value over its 21 spells' evaluators (0x43C4A0, §7.53) the same way. With BAT-QUIRK-25
//! off the elementals protect through the same scan, the shooters among them before their shot
//! (survey slip 25). Before any of this the side's hero weighs a cast at each activation of a
//! stack of its side but a war machine (0x422DA0, 0x43C800; §7.55, §7.56; [`hero`]): every spell
//! it can cast now by its value, the best cast when it is due now. The rest stands in until its
//! task builds it: the raises, Sacrifice, Earthquake and Teleport are worth nothing to the hero
//! (BAT-STUB-14, T5b), it never retreats (BAT-STUB-13), the tactics phase ends at once, the First
//! Aid Tent defends and a commanded Catapult shoots the first wall (BAT-STUB-5, T6).
//!
//! The original's slips in its own estimates (survey `E23-battle-ai.md` §4) follow the quirk
//! BAT-QUIRK-25 (`quirk.battle.original_ai_slips`, ADR-0027 point 9): on in `faithful`, off in
//! `strict-fixed`.
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

extern crate alloc;

#[cfg(test)]
extern crate std;

mod casts;
mod ends;
mod gain;
mod hero;
mod hexes;
mod odds;
mod order;
mod predict;
mod scores;
mod shooter;
mod spells;
mod walker;

#[cfg(test)]
mod tests;

use h3_battle::{
    BattleSide, BattleState, Command, LegalCommands, Stack, StackTraits, computer, legal_commands,
};
use h3_content_battle::BattleRuleset;
use h3_kernel::Rng;

/// The policy's behaviour version (ADR-0027 point 12): bumped with every change of a choice, so
/// that a training record names its opponent (the AI lane's engine identity includes it).
/// Replays store commands and do not change with it.
pub const POLICY_VERSION: u32 = 12;

/// The command the original's computer gives for `side`, the side to act in `state` (the active
/// stack's controller, BAT-MAG-026), whatever `BattleSetup.human` says (ADR-0027 point 8):
/// [`unchecked`]'s choice when the engine accepts it, else Defend (ADR-0027 point 3; no such
/// choice is known, the property tests assert none). Its draws come from `rng`, the caller's
/// stream, never the battle's (point 5); the same state and stream give the same command. For a
/// side not to act, or a battle that is over, it gives Defend, which the engine refuses.
// rule: BAT-AI-001
// Never inlined: the gated benchmark counts the instructions inside it (ADR-0027 point 11).
#[inline(never)]
pub fn choose(
    state: &BattleState,
    ruleset: &BattleRuleset,
    side: BattleSide,
    rng: &mut Rng,
) -> Command {
    checked(state, ruleset, side, unchecked(state, ruleset, side, rng))
}

/// The original's computer's choice for `side` before validation (the property tests assert the
/// engine accepts it): first its hero's cast ([`hero::cast`], 0x422DA0; BAT-AI-017: its spell
/// context's and its choice's draws from `rng`), then the stack's action (0x477EE0, 0x4221F0;
/// BAT-AI-001). A stack that can shoot
/// now ([`computer::shoots`]; the original's Ballista always can) shoots ([`shooter::shoot`],
/// BAT-AI-002 to BAT-AI-004: the Cyclops's wall shot draws from `rng`); any other is a walker
/// ([`walker::turn`], BAT-AI-005, BAT-AI-006: its candidates draw from `rng`).
// rule: BAT-AI-001
pub fn unchecked(
    state: &BattleState,
    ruleset: &BattleRuleset,
    side: BattleSide,
    rng: &mut Rng,
) -> Command {
    let Some(id) = state.active().filter(|_| state.side_to_act() == Some(side)) else {
        return Command::Defend;
    };
    if state.tactics().is_some() {
        // stub: BAT-STUB-5
        // The shooters' placement and the walkers' choice in the zone (§7.34): T6.
        return Command::EndTactics;
    }
    if let Some(cast) = hero::cast(state, ruleset, (id, side), rng) {
        return cast;
    }
    if computer::shoots(state, ruleset, id) {
        return shooter::shoot(state, ruleset, id, rng);
    }
    if state.stack(id).is_some_and(Stack::is_catapult) {
        // stub: BAT-STUB-5
        // The commanded Catapult's aim (§7.29, the gate first): T6.
        return first_wall_shot(state, ruleset, side);
    }
    if state
        .stack(id)
        .is_some_and(|s| s.has(StackTraits::FIRST_AID))
    {
        // stub: BAT-STUB-5
        // The First Aid Tent's patient (§7.14, BAT-Q51): T6; until then the original's default
        // action, Defend.
        return Command::Defend;
    }
    walker::turn(state, ruleset, (id, side), rng)
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
