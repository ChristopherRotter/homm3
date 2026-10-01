//! Dense action encoding (docs/07-simulation-and-ai.md §1: "dense action encoding is defined by
//! the sim layer, not the domain").
//!
//! Layout, version 2 (independent of the ruleset, so agents see a fixed-size action space):
//!
//! | index | action |
//! |---|---|
//! | 0 | Wait |
//! | 1 | Defend |
//! | 2 + h, h < 255 | Move to hex `h` (the head of a two-hex stack) |
//! | 257 + 24·s + k | Attack stack `s` from its canonical attack position `k` (`Field::attack_positions` for the active stack's shape) |
//!
//! Version 2 (E03 T3) replaced the six neighbour directions of version 1 by the 24 canonical
//! attack positions, which cover two-hex attackers and targets.

use h3_battle::{
    ATTACK_SLOTS, BattleState, Command, Field, Hex, LegalCommands, STACK_CAPACITY, StackId,
};
use h3_content::{MAX_FIELD_HEXES, Ruleset};
use h3_contracts::BattleSide;

/// Version of this encoding; bump on any change of the layout above.
pub const ACTION_ENCODING_VERSION: u32 = 2;

const MOVE_BASE: usize = 2;
const ATTACK_BASE: usize = MOVE_BASE + MAX_FIELD_HEXES;

/// Number of actions.
pub const ACTION_COUNT: usize = ATTACK_BASE + ATTACK_SLOTS * STACK_CAPACITY;

/// Legal-action mask.
pub type ActionMask = [bool; ACTION_COUNT];

/// The canonical attack positions of the active stack against `target`.
fn positions(
    state: &BattleState,
    ruleset: &Ruleset,
    target: StackId,
) -> Option<[Option<Hex>; ATTACK_SLOTS]> {
    let field = Field::new(ruleset.battle().field);
    let attacker = state.stack(state.active()?)?;
    let victim = state.stack(target)?;
    Some(field.attack_positions(
        attacker.is_two_hex(),
        attacker.side,
        victim.footprint(field),
    ))
}

/// Encodes a command. Attacks are encoded relative to the target's position and the active
/// stack's shape, so encoding needs the state; returns `None` for commands that cannot be
/// expressed (no active stack, unknown target, or a position that is not an attack position).
pub fn encode(state: &BattleState, ruleset: &Ruleset, command: Command) -> Option<usize> {
    match command {
        Command::Wait => Some(0),
        Command::Defend => Some(1),
        Command::Move { to } => (to.index() < MAX_FIELD_HEXES).then_some(MOVE_BASE + to.index()),
        Command::Attack { target, from } => {
            let k = positions(state, ruleset, target)?
                .iter()
                .position(|p| *p == Some(from))?;
            Some(ATTACK_BASE + ATTACK_SLOTS * target.index() + k)
        }
    }
}

/// Decodes an action index into a command for the current state (`None` if out of range or the
/// slot holds no attack position). The result still has to pass validation.
pub fn decode(state: &BattleState, ruleset: &Ruleset, action: usize) -> Option<Command> {
    match action {
        0 => Some(Command::Wait),
        1 => Some(Command::Defend),
        a if a < ATTACK_BASE => u8::try_from(a - MOVE_BASE)
            .ok()
            .map(|h| Command::Move { to: Hex(h) }),
        a if a < ACTION_COUNT => {
            let target = StackId(u8::try_from((a - ATTACK_BASE) / ATTACK_SLOTS).ok()?);
            let from = positions(state, ruleset, target)?[(a - ATTACK_BASE) % ATTACK_SLOTS]?;
            Some(Command::Attack { target, from })
        }
        _ => None,
    }
}

/// Fills `mask` with the legal actions of `actor` (all false if it is not the actor's turn).
pub fn legal_mask(
    state: &BattleState,
    ruleset: &Ruleset,
    actor: BattleSide,
    legal: &mut LegalCommands,
    mask: &mut ActionMask,
) {
    mask.fill(false);
    h3_battle::legal_commands(state, ruleset, actor, legal);
    for c in legal.iter() {
        if let Some(i) = encode(state, ruleset, *c) {
            mask[i] = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env() -> (Ruleset, BattleState) {
        let (r, setup) = h3_pack::synthetic(h3_pack::SYNTHETIC_SKIRMISH).unwrap();
        let s = BattleState::new(&r, &setup, 1).unwrap();
        (r, s)
    }

    #[test]
    fn layout() {
        assert_eq!(ACTION_COUNT, 2 + 255 + 24 * 32);
        assert_eq!(ACTION_ENCODING_VERSION, 2);
    }

    #[test]
    fn every_legal_command_round_trips() {
        let (r, s) = env();
        let side = s.side_to_act().unwrap();
        let mut legal = LegalCommands::new();
        let mut mask = [false; ACTION_COUNT];
        legal_mask(&s, &r, side, &mut legal, &mut mask);
        assert_eq!(mask.iter().filter(|m| **m).count(), legal.len());
        for c in legal.iter() {
            let i = encode(&s, &r, *c).unwrap();
            assert!(mask[i]);
            assert_eq!(decode(&s, &r, i), Some(*c));
        }
        legal_mask(&s, &r, side.opponent(), &mut legal, &mut mask);
        assert!(mask.iter().all(|m| !m));
    }

    #[test]
    fn unrepresentable_and_out_of_range() {
        let (r, s) = env();
        assert_eq!(encode(&s, &r, Command::Move { to: Hex(255) }), None);
        assert_eq!(
            encode(
                &s,
                &r,
                Command::Attack {
                    target: StackId(40),
                    from: Hex(0)
                }
            ),
            None
        );
        let far = Command::Attack {
            target: StackId(3),
            from: Hex(0),
        };
        assert_eq!(encode(&s, &r, far), None);
        assert_eq!(decode(&s, &r, ACTION_COUNT), None);
        // Stack 31 does not exist in a 6-stack battle.
        assert_eq!(decode(&s, &r, ATTACK_BASE + ATTACK_SLOTS * 31), None);
        assert_eq!(decode(&s, &r, 2 + 17), Some(Command::Move { to: Hex(17) }));
        // A one-hex target has no tail slots.
        let one_hex = s.stacks().iter().position(|t| !t.is_two_hex()).unwrap();
        assert_eq!(
            decode(&s, &r, ATTACK_BASE + ATTACK_SLOTS * one_hex + 12),
            None
        );
        // Stack 0 stands in the first standable column: some of its attack positions touch the
        // side column, so they decode, and validation rejects them later.
        let side = s.side_to_act().unwrap();
        let rejected = (0..ATTACK_SLOTS)
            .filter_map(|k| decode(&s, &r, ATTACK_BASE + k))
            .filter(|c| h3_battle::validate(&s, &r, side, *c).is_err())
            .count();
        assert!(rejected > 0);
    }
}
