//! Mandatory invariant property tests (docs/05-testing.md "Invariants every context must
//! provide"): atomicity, legality consistency, ownership isolation, conservation, observation
//! hygiene, clone equivalence and determinism.
//!
//! Battles are generated from random armies; each step either plays a random legal command or
//! throws an arbitrary (usually invalid) command at the engine, possibly from the wrong actor.

use std::vec::Vec;

use h3_content::Ruleset;
use h3_contracts::{Army, ArmyStack, BattleSetup, BattleSide};
use h3_kernel::Rng;
use proptest::collection::{btree_map, vec};
use proptest::prelude::{Just, Strategy, any, prop_assert, prop_assert_eq, prop_oneof, proptest};
use proptest::test_runner::Config;

use super::support::{CREATURES, ruleset};
use crate::{
    BattleState, Command, Event, Hex, LegalCommands, StackId, apply, legal_commands, observe,
};

/// One step of a generated game.
#[derive(Debug, Clone)]
enum Step {
    /// Play the `n`-th legal command (modulo the number of legal commands).
    Legal(u16),
    /// Try an arbitrary command, as the side to act or its opponent.
    Arbitrary { command: Command, wrong_actor: bool },
}

fn arb_command() -> impl Strategy<Value = Command> {
    prop_oneof![
        Just(Command::Wait),
        Just(Command::Defend),
        any::<u8>().prop_map(|h| Command::Move { to: Hex(h) }),
        (0u8..40, any::<u8>()).prop_map(|(t, h)| Command::Attack {
            target: StackId(t),
            from: Hex(h)
        }),
    ]
}

fn arb_step() -> impl Strategy<Value = Step> {
    prop_oneof![
        3 => any::<u16>().prop_map(Step::Legal),
        1 => (arb_command(), any::<bool>()).prop_map(|(command, wrong_actor)| Step::Arbitrary { command, wrong_actor }),
    ]
}

fn arb_army() -> impl Strategy<Value = ArmySpec> {
    btree_map(0u8..7, (0..CREATURES.len() as u8, 1u32..60), 1..=7)
        .prop_map(|m| m.into_iter().map(|(slot, (c, n))| (slot, c, n)).collect())
}

/// `(slot, creature index, count)` per stack.
type ArmySpec = Vec<(u8, u8, u32)>;
/// Attacker, defender, obstacles, seed, steps.
type Game = (ArmySpec, ArmySpec, Vec<u8>, u64, Vec<Step>);

/// Up to eight obstacle hexes away from the edge columns (x in 2..=14 of the 17-wide field).
fn arb_obstacles() -> impl Strategy<Value = Vec<u8>> {
    proptest::collection::btree_set((2u8..15, 0u8..11).prop_map(|(x, y)| x + 17 * y), 0..8)
        .prop_map(|s| s.into_iter().collect())
}

fn arb_game() -> impl Strategy<Value = Game> {
    (
        arb_army(),
        arb_army(),
        arb_obstacles(),
        any::<u64>(),
        vec(arb_step(), 1..160),
    )
}

fn make_setup(
    r: &Ruleset,
    attacker: &[(u8, u8, u32)],
    defender: &[(u8, u8, u32)],
    obstacles: &[u8],
) -> BattleSetup {
    let army = |spec: &[(u8, u8, u32)]| Army {
        stacks: spec
            .iter()
            .map(|&(slot, c, count)| ArmyStack {
                slot,
                creature: r
                    .creature_by_key(&std::format!("t:{}", CREATURES[usize::from(c)].0))
                    .unwrap(),
                count,
            })
            .collect(),
    };
    BattleSetup {
        attacker: army(attacker),
        defender: army(defender),
        obstacles: obstacles.to_vec(),
    }
}

/// Resolves a step into `(actor, command)` for the current state.
fn resolve(
    state: &BattleState,
    r: &Ruleset,
    step: &Step,
    legal: &mut LegalCommands,
) -> (BattleSide, Command) {
    let side = state.side_to_act().unwrap_or(BattleSide::Attacker);
    match step {
        Step::Legal(n) => {
            legal_commands(state, r, side, legal);
            let cmd = legal
                .get(usize::from(*n) % legal.len().max(1))
                .copied()
                .unwrap_or(Command::Wait);
            (side, cmd)
        }
        Step::Arbitrary {
            command,
            wrong_actor,
        } => (if *wrong_actor { side.opponent() } else { side }, *command),
    }
}

/// Plays a game, returning the final state, the event log and the state hash after every step.
fn play(
    r: &Ruleset,
    setup: &BattleSetup,
    seed: u64,
    steps: &[Step],
) -> (BattleState, Vec<Event>, Vec<u64>) {
    let mut state = BattleState::new(r, setup, seed).unwrap();
    let mut events = Vec::new();
    let mut hashes = Vec::new();
    let mut legal = LegalCommands::new();
    for step in steps {
        let (actor, cmd) = resolve(&state, r, step, &mut legal);
        let _ = apply(&mut state, r, actor, cmd, &mut events);
        hashes.push(state.state_hash());
    }
    (state, events, hashes)
}

proptest! {
    #![proptest_config(Config { cases: 96, ..Config::default() })]

    /// Atomicity, legality consistency, ownership isolation and conservation, checked at every step.
    #[test]
    fn every_step_preserves_the_invariants((att, def, obstacles, seed, steps) in arb_game()) {
        let r = ruleset();
        let setup = make_setup(&r, &att, &def, &obstacles);
        let mut state = BattleState::new(&r, &setup, seed).unwrap();
        let mut legal = LegalCommands::new();
        for step in &steps {
            let (actor, cmd) = resolve(&state, &r, step, &mut legal);

            // Legality consistency: the legal set is exactly what apply accepts. Every listed
            // command is accepted; the opponent has no legal commands.
            legal_commands(&state, &r, actor, &mut legal);
            for c in legal.iter() {
                let mut probe = state.clone();
                prop_assert!(apply(&mut probe, &r, actor, *c, &mut crate::NoopSink).is_ok(), "legal {c:?} rejected");
            }
            let mut other = LegalCommands::new();
            legal_commands(&state, &r, actor.opponent(), &mut other);
            prop_assert!(other.is_empty() || state.side_to_act() == Some(actor.opponent()));

            let before = state.clone();
            let before_hash = state.state_hash();
            let result = apply(&mut state, &r, actor, cmd, &mut Vec::new());
            match result {
                Err(_) => {
                    // Atomicity: a rejected command changes nothing.
                    prop_assert_eq!(&state, &before);
                    prop_assert_eq!(state.state_hash(), before_hash);
                    // ... and it was not in the legal set.
                    prop_assert!(!legal.iter().any(|c| *c == cmd), "legal {cmd:?} rejected");
                }
                Ok(()) => {
                    prop_assert!(legal.iter().any(|c| *c == cmd), "illegal {cmd:?} accepted");
                    let target = match cmd {
                        Command::Attack { target, .. } | Command::Shoot { target, .. } => Some(target),
                        _ => None,
                    };
                    // Area shots may also hit the stacks around the aimed hex (BAT-RNG-004).
                    let field = crate::Field::new(r.battle().field);
                    let area = |s: &crate::Stack| match cmd {
                        Command::Shoot { at, .. } => {
                            let shooter = before.active().map(|id| before.stacks()[id.index()]);
                            shooter.is_some_and(|sh| sh.has(crate::StackTraits::AREA_SHOT))
                                && s.footprint(field).hexes().any(|h| field.distance(h, at) <= 1)
                        }
                        _ => false,
                    };
                    for (i, (b, a)) in before.stacks().iter().zip(state.stacks()).enumerate() {
                        // Conservation: creatures only ever die.
                        prop_assert!(a.count <= b.count);
                        prop_assert!(a.is_alive() || a.top_hp == 0);
                        prop_assert!(!a.is_alive() || (1..=a.max_hp).contains(&a.top_hp));
                        // Ownership isolation: the actor's command cannot move or re-stat enemy
                        // stacks, and can only damage the attacked one.
                        prop_assert_eq!((a.creature, a.side, a.slot, a.attack, a.defense, a.speed), (b.creature, b.side, b.slot, b.attack, b.defense, b.speed));
                        if b.side != actor {
                            prop_assert_eq!(a.hex, b.hex);
                            if target != Some(StackId(i as u8)) && !area(b) {
                                prop_assert_eq!((a.count, a.top_hp), (b.count, b.top_hp));
                            }
                        }
                    }
                }
            }
            // A battle that is over has no active stack, and vice versa.
            prop_assert_eq!(state.is_over(), state.active().is_none());
        }
    }

    /// Observation hygiene: two states that differ only in hidden information (the RNG) look the
    /// same to both sides.
    #[test]
    fn observations_do_not_depend_on_hidden_state((att, def, obstacles, seed, steps) in arb_game(), other_seed: u64) {
        let r = ruleset();
        let (state, _, _) = play(&r, &make_setup(&r, &att, &def, &obstacles), seed, &steps);
        let mut twin = state.clone();
        twin.rng = Rng::from_seed(other_seed);
        for viewer in BattleSide::BOTH {
            prop_assert_eq!(observe(&state, viewer), observe(&twin, viewer));
        }
    }

    /// Clone equivalence: a clone taken mid-game evolves identically to the original.
    #[test]
    fn clones_evolve_identically((att, def, obstacles, seed, steps) in arb_game(), split in 0usize..160) {
        let r = ruleset();
        let setup = make_setup(&r, &att, &def, &obstacles);
        let split = split.min(steps.len());
        let (mut original, _, _) = play(&r, &setup, seed, &steps[..split]);
        let mut clone = original.clone();
        let mut legal = LegalCommands::new();
        for step in &steps[split..] {
            let (actor, cmd) = resolve(&original, &r, step, &mut legal);
            let a = apply(&mut original, &r, actor, cmd, &mut crate::NoopSink);
            let b = apply(&mut clone, &r, actor, cmd, &mut crate::NoopSink);
            prop_assert_eq!(a, b);
            prop_assert_eq!(original.state_hash(), clone.state_hash());
        }
        prop_assert_eq!(original, clone);
    }

    /// Determinism: the same inputs give the same events and state hashes.
    #[test]
    fn replays_are_deterministic((att, def, obstacles, seed, steps) in arb_game()) {
        let r = ruleset();
        let setup = make_setup(&r, &att, &def, &obstacles);
        let first = play(&r, &setup, seed, &steps);
        let second = play(&r, &setup, seed, &steps);
        prop_assert_eq!(first.1, second.1);
        prop_assert_eq!(first.2, second.2);
        prop_assert_eq!(first.0, second.0);
    }
}

/// Random legal play always terminates in a decided battle eventually (sanity check of the turn
/// machinery; not a game rule: battles have no round limit).
#[test]
fn random_legal_play_finishes_battles() {
    let r = ruleset();
    let setup = make_setup(
        &r,
        &[(0, 5, 20), (1, 6, 8), (2, 10, 2)],
        &[(0, 7, 15), (3, 5, 12), (4, 8, 6)],
        &[60, 77, 94],
    );
    let mut decided = 0;
    for seed in 0..20u64 {
        let mut state = BattleState::new(&r, &setup, seed).unwrap();
        let mut agent = Rng::for_stream(seed, h3_kernel::StreamId::AGENT);
        let mut legal = LegalCommands::new();
        for _ in 0..5_000 {
            let Some(side) = state.side_to_act() else {
                break;
            };
            legal_commands(&state, &r, side, &mut legal);
            // Prefer attacks so that random games end quickly.
            let attacks: Vec<Command> = legal
                .iter()
                .copied()
                .filter(|c| matches!(c, Command::Attack { .. }))
                .collect();
            let pool: &[Command] = if attacks.is_empty() {
                legal.as_slice()
            } else {
                &attacks
            };
            let cmd = pool[agent.below(pool.len() as u32) as usize];
            apply(&mut state, &r, side, cmd, &mut crate::NoopSink).unwrap();
        }
        decided += usize::from(state.is_over());
    }
    assert_eq!(decided, 20);
}
