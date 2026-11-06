//! The original's battle AI (ADR-0027, E23): the policy of a battle side that the original's
//! computer plays, one pure, deterministic function of the battle's state, its ruleset, the
//! side to act and the caller's stream ([`choose`]). It reads the whole state, never an
//! observation, and writes nothing (ADR-0027 point 4); the computer's numbers the battle's rules
//! use come from `h3-battle`'s public [`computer`] module, one implementation for both (point 6).
//! Its callers are the AI lane's batch (`Policy::System`) and the adventure's neutral seat; they
//! get it once a side without a hero is complete (E23 T4). The battle client's Auto Combat asks
//! it part by part for a human's side, as the original does ([`Decision`], E17 M6 T5b).
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
//! it can cast now by its value, the best cast when it is due now; the raises, Sacrifice and
//! Earthquake by cases of their own (§7.57), Teleport by each stack's walker turn as it is and on
//! the hexes Teleport could put it (§7.58), a caster among them asking its own cast entry from its
//! record (§7.59). A commanded Catapult aims as the original's computer does with Ballistics: the
//! gate while it stands, then the weakest wall while all four stand, then the rest (0x473C00;
//! E23 T6); a commanded First Aid Tent heals the patient with the most hit points lost by its top
//! creature (0x473EA0). In the tactics phase ([`tactics`], §7.60) the side ends its phase after a
//! round without a placement or after three, its shooters move to the hex in the zone with the
//! fewest free neighbours (0x422060) and its walkers walk towards their plans as far as the zone
//! lets them (0x421F80 with the phase's flag). The retreat test (0x41E570; [`retreat`], E23 T6)
//! weighs the hero's worth, the predictions of both sides' runs and the side's share of the
//! battle's Fight Value: the hero's cast asks it before its choice (a yes makes the choice
//! desperate), the stack's action again outside the tactics phase (a yes is a retreat; the
//! computer never surrenders).
//!
//! The original's slips in its own estimates (survey `E23-battle-ai.md` §4) follow the quirk
//! BAT-QUIRK-25 (`quirk.battle.original_ai_slips`, ADR-0027 point 9): on in `faithful`, off in
//! `strict-fixed`.
//!
//! The fair system actor ([`choose_fair`], E26; ADR-0027 point 4 as amended) is the same
//! computer on its side's world of the battle ([`BattleState::determinized`]): it decides only
//! on what its side may know, so that it may train agents and be the default seat; [`choose`]
//! stays the all-seeing original, a game's option.
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
mod decision;
mod ends;
mod gain;
mod hero;
mod hexes;
mod odds;
mod order;
mod predict;
mod retreat;
mod scores;
mod shooter;
mod spells;
mod tactics;
mod walker;

#[cfg(test)]
mod tests;

use h3_battle::{BattleSide, BattleState, Command, StackId, StackTraits, computer};
use h3_content_battle::BattleRuleset;
use h3_kernel::Rng;

pub use decision::{Decision, Sight};
pub use retreat::Leaving;

/// The policy's behaviour version (ADR-0027 point 12): bumped with every change of a choice, so
/// that a training record names its opponent (the AI lane's engine identity includes it).
/// Replays store commands and do not change with it.
pub const POLICY_VERSION: u32 = 19;

/// The command the original's computer gives for `side`, the side to act in `state` (the active
/// stack's controller, BAT-MAG-026), whatever `BattleSetup.human` says (ADR-0027 point 8):
/// [`unchecked`]'s choice when the engine accepts it, else Defend (ADR-0027 point 3; one such
/// choice is known, a hypnotized shooter's shot at itself, BAT-STUB-18; the property tests assert
/// no other). Its draws come from `rng`, the caller's stream, never the battle's (point 5); the
/// same state and stream give the same command. For a side not to act, or a battle that is over,
/// it gives Defend, which the engine refuses.
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

/// The fair system actor's command for `side` (ADR-0027 point 4 as amended, E26; BAT-AI-025):
/// the original's computer ([`unchecked`]) on `side`'s world of the battle
/// ([`BattleState::determinized`] without a guess: everything `side` may not know replaced, the
/// other side's hero a hero of its type with the public part the battle shows, its hidden spell
/// obstacles placed again by their rule), its choice checked on the battle as [`choose`]'s.
/// The world's seed is drawn from a copy of `rng`, so the policy's own draws keep the original's
/// order and leave `rng` where [`choose`] leaves it: where `side` may know everything (no other
/// hero, no hidden obstacle of the other side) the command is [`choose`]'s. Defend for a world
/// the battle refuses (none is known: a hero of its type alone always passes).
// rule: BAT-AI-025
// Never inlined, as `choose`: the world's cost is part of a decision's (ADR-0027 point 11).
#[inline(never)]
pub fn choose_fair(
    state: &BattleState,
    ruleset: &BattleRuleset,
    side: BattleSide,
    rng: &mut Rng,
) -> Command {
    let seed = rng.clone().next_u64();
    state
        .determinized(ruleset, side, None, seed)
        .map_or(Command::Defend, |world| {
            checked(state, ruleset, side, unchecked(&world, ruleset, side, rng))
        })
}

/// The original's computer's choice for `side` before validation (the property tests assert the
/// engine accepts it): in the tactics phase the phase's turn ([`tactics::turn`], BAT-AI-023);
/// else first its hero's cast ([`hero::cast`], 0x422DA0; BAT-AI-017: its spell
/// context's, its retreat test's and its choice's draws from `rng`), then the stack's action
/// (0x477EE0, 0x4221F0; BAT-AI-001): the commanded Catapult's shot and First Aid Tent's heal;
/// for any other stack the retreat test ([`retreat::test`], BAT-AI-024: a yes is Retreat), then
/// a stack that can shoot now ([`computer::shoots`]; the original's Ballista always can) shoots
/// ([`shooter::shoot`], BAT-AI-002 to BAT-AI-004: the Cyclops's wall shot draws from `rng`); any
/// other is a walker ([`walker::turn`], BAT-AI-005, BAT-AI-006: its candidates draw from `rng`).
// rule: BAT-AI-001
pub fn unchecked(
    state: &BattleState,
    ruleset: &BattleRuleset,
    side: BattleSide,
    rng: &mut Rng,
) -> Command {
    let Some(id) = acting(state, side) else {
        return Command::Defend;
    };
    if let Some(phase) = state.tactics() {
        return tactics::turn(state, ruleset, (id, side, phase), rng);
    }
    if let Some(cast) = hero::cast(state, ruleset, (id, side), false, rng) {
        return cast;
    }
    stack(state, ruleset, (id, side), rng, true)
}

/// The active stack of `side` when `side` is to act in `state`.
fn acting(state: &BattleState, side: BattleSide) -> Option<StackId> {
    state.active().filter(|_| state.side_to_act() == Some(side))
}

/// The stack's action outside the tactics phase (0x477EE0, 0x4221F0; BAT-AI-001): the commanded
/// Catapult's shot and First Aid Tent's heal; for any other stack, with `retreat`, the retreat
/// test first ([`retreat::test`], BAT-AI-024: a yes is Retreat), then a shooter's shot or a
/// walker's turn. Without `retreat` the test is not asked (its draws not made): [`Decision`]'s
/// action, the test its caller's.
fn stack(
    state: &BattleState,
    ruleset: &BattleRuleset,
    (id, side): (StackId, BattleSide),
    rng: &mut Rng,
    retreat: bool,
) -> Command {
    let me = &state.stacks()[id.index()];
    if me.is_catapult() {
        return catapult(state, ruleset, id, rng);
    }
    if me.has(StackTraits::FIRST_AID) {
        return tent(state, id);
    }
    if retreat && retreat::test(state, ruleset, side, rng, false).retreat {
        return Command::Retreat;
    }
    if computer::shoots(state, ruleset, id) {
        return shooter::shoot(state, ruleset, id, rng);
    }
    walker::turn(state, ruleset, (id, side), rng)
}

/// `command` when the engine accepts it from `side`, else Defend, in the tactics phase Wait,
/// which ends no turn there (ADR-0027 point 3).
fn checked(
    state: &BattleState,
    ruleset: &BattleRuleset,
    side: BattleSide,
    command: Command,
) -> Command {
    if h3_battle::validate(state, ruleset, side, command).is_ok() {
        command
    } else if state.tactics().is_some() {
        Command::Wait
    } else {
        Command::Defend
    }
}

/// The commanded First Aid Tent's heal (0x473EA0; BAT-AI-022): the patient the battle's
/// automatic tent heals too ([`computer::tent_patient`], BAT-WAR-003: the stack whose top creature
/// lost the most, the later at a tie); Defend without one (the engine asks a commanded tent only
/// with a patient, BAT-WAR-007).
// rule: BAT-AI-022
fn tent(state: &BattleState, id: StackId) -> Command {
    computer::tent_patient(state, id).map_or(Command::Defend, |target| Command::Heal { target })
}

/// The commanded Catapult's shot (0x473C00 with Ballistics; BAT-AI-021): at the computer's aim
/// ([`computer::catapult_aim`]: the gate while it stands, then the weakest wall while all stand,
/// its tie drawn from `rng`), or Defend with nothing standing (the engine passes such a turn).
// rule: BAT-AI-021
fn catapult(state: &BattleState, ruleset: &BattleRuleset, id: StackId, rng: &mut Rng) -> Command {
    computer::catapult_aim(state, ruleset, id, rng)
        .map_or(Command::Defend, |part| Command::ShootWall { part })
}
