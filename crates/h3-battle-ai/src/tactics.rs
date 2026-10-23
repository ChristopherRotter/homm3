//! The computer's tactics phase (`docs/formats/installation.md` §7.34, §7.60; BAT-AI-023): the
//! phase's end (round_start 0x475ED0), the shooters' placement (0x422060) and the walkers' turns
//! in the phase (0x421F80 with the phase's flag, [`walker`]).
//!
//! The original's computer ends its phase at a round's end when no action of the round placed a
//! stack, or after its third round: the policy gives `EndTactics` at the first turn after such a
//! round, by the rounds the battle counts for it ([`computer::tactics_rounds`]). A shooter moves
//! to the position of the zone with the fewest free neighbours, never one next to a shooter, and
//! stays on a tie; a walker walks towards its plan's hex as far as the zone lets it.
//!
//! The original's Defend in the phase ends the stack's turn (0x479706), the engine's Wait (the
//! game's "next creature", BAT-TAC-002) makes it wait first: the policy gives Wait for Defend, and
//! below Hard, where a walker never waits, a walker that has waited gives Wait again at once. On
//! Hard and above a walker that defended is asked again among the waited stacks (BAT-STUB-16).

use h3_battle::computer;
use h3_battle::{
    BattleSide, BattleState, Command, Field, Hex, StackFlags, StackId, StackTraits, TacticsPhase,
};
use h3_content_battle::BattleRuleset;
use h3_kernel::Rng;

use crate::hexes::{self, Shape};
use crate::walker;

/// The most rounds of a computer's tactics phase (round_start's counter, 0x475ED0).
const MOST_ROUNDS: u8 = 3;

/// A shooter's score at its own position to begin with (0x422060).
const START: i32 = 100;

/// The score a shooter among the neighbours gives a hex, the count going on from it (0x422060).
const BESIDE_A_SHOOTER: i32 = 1000;

/// The command of the computer's side in the tactics phase `phase` for its stack `id`
/// (BAT-AI-023): `EndTactics` after a completed round without a placement or after three; a
/// stack that can shoot now places itself ([`place`]); a walker below Hard that has waited gives
/// Wait (it gave Defend); any other walker takes its turn ([`walker::turn`], its draws from
/// `rng`). Defend becomes Wait.
// rule: BAT-AI-023
pub(crate) fn turn(
    state: &BattleState,
    ruleset: &BattleRuleset,
    (id, side, phase): (StackId, BattleSide, TacticsPhase),
    rng: &mut Rng,
) -> Command {
    let rounds = computer::tactics_rounds(state);
    if rounds.completed >= MOST_ROUNDS || (rounds.completed > 0 && !rounds.placed_last) {
        return Command::EndTactics;
    }
    let me = &state.stacks()[id.index()];
    let hard = computer::difficulty(state) >= 2 || computer::human(state, side);
    let command = if computer::shoots(state, ruleset, id) {
        place(state, ruleset, (id, side, phase))
    } else if !hard && me.flags.contains(StackFlags::WAITED) {
        Command::Wait
    } else {
        walker::turn(state, ruleset, (id, side), rng)
    };
    if command == Command::Defend {
        // stub: BAT-STUB-16
        // The original's Defend ends the stack's turn in the phase (0x479706); the engine's Wait
        // makes it wait, and on Hard and above the walker is asked again among the waited stacks.
        Command::Wait
    } else {
        command
    }
}

/// The shooter `id`'s placement in the phase (0x422060; BAT-AI-023): Wait without speed, or on
/// Easy for a computer side; else over the positions of the field in hex order that its map with
/// the phase's flag reaches ([`computer::map`]: a flyer's hexes flown over, which the original's
/// cells also mark, are none) and that lie in the zone, a score from its neighbours (six, eight
/// for a two-hex stack; 0x523DF0) on the field: one more for each without a stack or with the
/// shooter itself, and a shooter of either side sets it to 1000; the lowest score below 100 at
/// its own position, an equal one replacing the best only at its own position. A best other than
/// its own position: a move there; else Wait.
// rule: BAT-AI-023
fn place(
    state: &BattleState,
    ruleset: &BattleRuleset,
    (id, side, phase): (StackId, BattleSide, TacticsPhase),
) -> Command {
    let me = &state.stacks()[id.index()];
    if me.speed_now() == 0 || (computer::difficulty(state) == 0 && !computer::human(state, side)) {
        return Command::Wait;
    }
    let field = Field::new(ruleset.battle().field);
    let map = computer::map(state, ruleset, id, side);
    let shape = Shape::of(me);
    let on = walker::stacks_on(state, field);
    let mut best = (me.hex, START);
    for h in (0u8..).take(field.hex_count()).map(Hex) {
        if map.position(h).is_none() || !in_zone(field, phase, shape, h) {
            continue;
        }
        let mut score = 0;
        for d in 0..hexes::directions(shape) {
            let Some(n) = hexes::stack_neighbor(field, shape, h, d) else {
                continue;
            };
            match on[n.index()] {
                Some(x) if x != id => {
                    if state.stacks()[x.index()].has(StackTraits::SHOOTER) {
                        score = BESIDE_A_SHOOTER;
                    }
                }
                _ => score += 1,
            }
        }
        if score < best.1 || (score == best.1 && h == me.hex) {
            best = (h, score);
        }
    }
    if best.0 == me.hex {
        Command::Wait
    } else {
        Command::Move { to: best.0 }
    }
}

/// Whether a stack of `shape` may end a move in the phase at `h` (0x4763F0: the position hex in
/// the side's zone, [`TacticsPhase::contains`]).
// stub: BAT-STUB-15
// The original tests the position hex alone; the engine places a two-hex stack only with both
// its hexes in the zone (BAT-TAC-002), so its other hex must lie there too.
pub(crate) fn in_zone(field: Field, phase: TacticsPhase, shape: Shape, h: Hex) -> bool {
    phase.contains(field, h) && phase.contains(field, shape.other(field, h))
}
