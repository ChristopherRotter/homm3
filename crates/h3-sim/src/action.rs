//! Dense action encoding (docs/07-simulation-and-ai.md §1: "dense action encoding is defined by
//! the sim layer, not the domain").
//!
//! Layout, version 8 (independent of the ruleset, so agents see a fixed-size action space):
//!
//! | index | action |
//! |---|---|
//! | 0 | Wait |
//! | 1 | Defend |
//! | 2 + h, h < 255 | Move to hex `h` (the head of a two-hex stack) |
//! | 257 + 24·s + k | Attack stack `s` from its canonical attack position `k` (`Field::attack_positions` for the active stack's shape) |
//! | 1169 + 2·s + a | Shoot at stack `s`, aiming at its head (`a = 0`) or tail (`a = 1`) |
//! | 1245 + s | Heal stack `s` (the commanded First Aid Tent, BAT-WAR-007) |
//! | 1283 | End the tactics phase (BAT-TAC-002) |
//! | 1284 | Retreat (BAT-FLE-001) |
//! | 1285 | Surrender (BAT-FLE-002) |
//!
//! In the tactics phase `Move` indices place the active stack and Wait (0) passes to the next
//! stack (BAT-TAC-002).
//!
//! Version 2 (E03 T3) replaced the six neighbour directions of version 1 by the 24 canonical
//! attack positions, which cover two-hex attackers and targets; version 3 (E03 T4) added shots;
//! version 4 (E03 T8) grew the stack slots from 32 to 38 (`STACK_CAPACITY` includes war machines);
//! version 5 (E04a T7b) appended the heals, version 6 (E04a T8) `EndTactics`, version 7
//! (E04a T9a) Retreat and version 8 (E04a T9b) Surrender, so the earlier indices keep their
//! meaning.

use h3_battle::{
    ATTACK_SLOTS, BattleState, Command, Field, Hex, LegalCommands, STACK_CAPACITY, StackId,
};
use h3_content::{MAX_FIELD_HEXES, Ruleset};
use h3_contracts::BattleSide;

/// Version of this encoding; bump on any change of the layout above.
pub const ACTION_ENCODING_VERSION: u32 = 8;

const MOVE_BASE: usize = 2;
const ATTACK_BASE: usize = MOVE_BASE + MAX_FIELD_HEXES;

const SHOOT_BASE: usize = ATTACK_BASE + ATTACK_SLOTS * STACK_CAPACITY;

const HEAL_BASE: usize = SHOOT_BASE + 2 * STACK_CAPACITY;

const END_TACTICS: usize = HEAL_BASE + STACK_CAPACITY;

const RETREAT: usize = END_TACTICS + 1;

const SURRENDER: usize = RETREAT + 1;

/// Number of actions.
pub const ACTION_COUNT: usize = SURRENDER + 1;

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
        Command::Shoot { target, at } => {
            let field = Field::new(ruleset.battle().field);
            let fp = state.stack(target)?.footprint(field);
            let aim = if at == fp.head {
                0
            } else if fp.tail == Some(at) {
                1
            } else {
                return None;
            };
            Some(SHOOT_BASE + 2 * target.index() + aim)
        }
        Command::Heal { target } => {
            (target.index() < STACK_CAPACITY).then_some(HEAL_BASE + target.index())
        }
        Command::EndTactics => Some(END_TACTICS),
        Command::Retreat => Some(RETREAT),
        Command::Surrender => Some(SURRENDER),
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
        a if a < SHOOT_BASE => {
            let target = StackId(u8::try_from((a - ATTACK_BASE) / ATTACK_SLOTS).ok()?);
            let from = positions(state, ruleset, target)?[(a - ATTACK_BASE) % ATTACK_SLOTS]?;
            Some(Command::Attack { target, from })
        }
        a if a < HEAL_BASE => {
            let target = StackId(u8::try_from((a - SHOOT_BASE) / 2).ok()?);
            let field = Field::new(ruleset.battle().field);
            let fp = state.stack(target)?.footprint(field);
            let at = if (a - SHOOT_BASE).is_multiple_of(2) {
                fp.head
            } else {
                fp.tail?
            };
            Some(Command::Shoot { target, at })
        }
        a if a < END_TACTICS => u8::try_from(a - HEAL_BASE)
            .ok()
            .map(|t| Command::Heal { target: StackId(t) }),
        END_TACTICS => Some(Command::EndTactics),
        RETREAT => Some(Command::Retreat),
        SURRENDER => Some(Command::Surrender),
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

    /// Decodes an index that does not depend on the state.
    fn decode_index(action: usize) -> Option<Command> {
        let (r, s) = env();
        let command = decode(&s, &r, action);
        assert_eq!(
            command.and_then(|c| encode(&s, &r, c)),
            command.map(|_| action)
        );
        command
    }

    #[test]
    fn layout() {
        assert_eq!(ACTION_COUNT, 2 + 255 + 24 * 38 + 2 * 38 + 38 + 3);
        assert_eq!((END_TACTICS, RETREAT, SURRENDER), (1283, 1284, 1285));
        assert_eq!(ACTION_ENCODING_VERSION, 8);
        assert_eq!(decode_index(SURRENDER), Some(Command::Surrender));
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
    fn shots_round_trip_including_tail_aims() {
        use h3_contracts::{Army, ArmyStack, BattleSetup};
        let r = h3_pack::compile(&[h3_pack::BASE_PACK], "base:faithful").unwrap();
        let stack = |key: &str, count| Army {
            stacks: vec![ArmyStack {
                slot: 0,
                creature: r.creature_by_key(key).unwrap(),
                count,
            }],
        };
        let setup = BattleSetup {
            // Magogs (speed 6) act before Hydras (speed 5, two hexes) and shoot.
            attacker: stack("base:magog", 10),
            defender: stack("base:hydra", 2),
            obstacles: vec![],
            heroes: [None, None],
            terrain: None,
            war_machines: [vec![], vec![]],
            formations: Default::default(),
            funds: [0; 2],
        };
        let s = BattleState::new(&r, &setup, 1).unwrap();
        let side = s.side_to_act().unwrap();
        let mut legal = LegalCommands::new();
        let mut mask = [false; ACTION_COUNT];
        legal_mask(&s, &r, side, &mut legal, &mut mask);
        let shots: Vec<Command> = legal
            .iter()
            .copied()
            .filter(|c| matches!(c, Command::Shoot { .. }))
            .collect();
        assert_eq!(shots.len(), 2, "head and tail of the two-hex hydra");
        for c in shots {
            let i = encode(&s, &r, c).unwrap();
            assert!(mask[i]);
            assert_eq!(decode(&s, &r, i), Some(c));
        }
        let off = Command::Shoot {
            target: StackId(1),
            at: Hex(0),
        };
        assert_eq!(encode(&s, &r, off), None);
        // A one-hex target has no tail aim.
        assert_eq!(decode(&s, &r, SHOOT_BASE + 1), None);
    }

    #[test]
    fn heals_of_the_commanded_tent_round_trip() {
        // The full scenario's attacker commands its tent (BAT-WAR-007): play until it may heal.
        let (r, setup) = h3_pack::synthetic(h3_pack::SYNTHETIC_FULL).unwrap();
        let mut s = BattleState::new(&r, &setup, 3).unwrap();
        let mut legal = LegalCommands::new();
        let mut mask = [false; ACTION_COUNT];
        let mut heals = 0;
        for step in 0..2000 {
            let Some(side) = s.side_to_act() else { break };
            legal_mask(&s, &r, side, &mut legal, &mut mask);
            assert_eq!(mask.iter().filter(|m| **m).count(), legal.len());
            for c in legal.iter().filter(|c| matches!(c, Command::Heal { .. })) {
                let i = encode(&s, &r, *c).unwrap();
                assert!(i >= HEAL_BASE && mask[i]);
                assert_eq!(decode(&s, &r, i), Some(*c));
                heals += 1;
            }
            let command = legal.as_slice()[step % legal.len()];
            h3_battle::apply(&mut s, &r, side, command, &mut h3_battle::NoopSink).unwrap();
        }
        assert!(heals > 0);
        assert_eq!(
            encode(
                &s,
                &r,
                Command::Heal {
                    target: StackId(38)
                }
            ),
            None
        );
        assert_eq!(
            decode(&s, &r, END_TACTICS - 1),
            Some(Command::Heal {
                target: StackId(37)
            })
        );
    }

    #[test]
    fn tactics_commands_round_trip() {
        // The full scenario's attacker has Advanced Tactics, the defender no hero (BAT-TAC-001).
        let (r, setup) = h3_pack::synthetic(h3_pack::SYNTHETIC_FULL).unwrap();
        let s = BattleState::new(&r, &setup, 3).unwrap();
        assert!(s.tactics().is_some());
        let mut legal = LegalCommands::new();
        let mut mask = [false; ACTION_COUNT];
        legal_mask(&s, &r, BattleSide::Attacker, &mut legal, &mut mask);
        assert_eq!(mask.iter().filter(|m| **m).count(), legal.len());
        // Brea may retreat in her tactics phase (BAT-FLE-001).
        assert!(mask[0] && mask[END_TACTICS] && mask[RETREAT] && !mask[1]);
        for c in legal.iter() {
            let i = encode(&s, &r, *c).unwrap();
            assert_eq!(decode(&s, &r, i), Some(*c));
        }
        assert_eq!(decode(&s, &r, END_TACTICS), Some(Command::EndTactics));
        assert_eq!(decode(&s, &r, RETREAT), Some(Command::Retreat));
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
