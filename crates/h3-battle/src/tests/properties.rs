//! Mandatory invariant property tests (docs/05-testing.md "Invariants every context must
//! provide"): atomicity, legality consistency, ownership isolation, conservation, observation
//! hygiene, clone equivalence and determinism.
//!
//! Battles are generated from random armies; each step either plays a random legal command or
//! throws an arbitrary (usually invalid) command at the engine, possibly from the wrong actor.

use std::vec::Vec;

use h3_content::{Formation, Ruleset};
use h3_contracts::{Army, ArmyStack, BallistaControl, BattleSetup, BattleSide, TentControl};
use h3_kernel::Rng;
use proptest::collection::{btree_map, vec};
use proptest::prelude::{Just, Strategy, any, prop_assert, prop_assert_eq, prop_oneof, proptest};
use proptest::test_runner::Config;

use super::support::{CREATURES, hero, ruleset_with};
use crate::event::StrikeFlags;
use crate::{
    BattleState, Command, Event, EventKind, Hex, LegalCommands, StackId, apply, legal_commands,
    observe,
};

/// The test ruleset with frequent good and bad morale and luck (a 4-sided die) and the quirk
/// Q-0003 on, so that generated games exercise them.
fn lively() -> Ruleset {
    ruleset_with("t:quirky", |b| {
        b.morale_dice = 4;
        b.good_morale_chances = std::vec![1, 2, 3];
        b.bad_morale_chances = std::vec![1, 2, 3];
        b.luck_dice = 4;
        b.good_luck_chances = std::vec![1, 2, 3];
    })
}

/// One step of a generated game.
#[derive(Debug, Clone)]
enum Step {
    /// Play the `n`-th legal command that does not leave the battle (modulo their number;
    /// leaving is exercised by arbitrary steps and the legality probes).
    Legal(u16),
    /// Try an arbitrary command, as the side to act or its opponent.
    Arbitrary { command: Command, wrong_actor: bool },
}

/// An arbitrary command; leaving the battle is rare, so that games run long.
fn arb_command() -> impl Strategy<Value = Command> {
    prop_oneof![
        8 => Just(Command::Wait),
        8 => Just(Command::Defend),
        8 => Just(Command::EndTactics),
        8 => any::<u8>().prop_map(|h| Command::Move { to: Hex(h) }),
        8 => (0u8..40, any::<u8>()).prop_map(|(t, h)| Command::Attack {
            target: StackId(t),
            from: Hex(h)
        }),
        8 => (0u8..40).prop_map(|t| Command::Heal { target: StackId(t) }),
        1 => Just(Command::Retreat),
        1 => Just(Command::Surrender),
    ]
}

fn arb_step() -> impl Strategy<Value = Step> {
    prop_oneof![
        3 => any::<u16>().prop_map(Step::Legal),
        1 => (arb_command(), any::<bool>()).prop_map(|(command, wrong_actor)| Step::Arbitrary { command, wrong_actor }),
    ]
}

/// Indices of the test creatures that may be in an army (war machines may not).
fn army_creatures() -> Vec<u8> {
    (0u8..)
        .zip(CREATURES)
        .filter(|(_, c)| !c.7.contains(&"war_machine"))
        .map(|(i, _)| i)
        .collect()
}

fn arb_army() -> impl Strategy<Value = ArmySpec> {
    let pool = army_creatures();
    btree_map(0u8..7, (0..pool.len(), 1u32..60), 1..=7).prop_map(move |m| {
        m.into_iter()
            .map(|(slot, (c, n))| (slot, pool[c], n))
            .collect()
    })
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

/// A setup from generated armies and obstacles; bits 0–5 of `machines` choose the attacker's
/// and the defender's war machines (ballista, ammo cart, first aid tent), bits 6–7 whether the
/// attacker and the defender stand in tight formation, bit 8 whether the attacker's hero
/// commands its Ballista (bit 9: two shots; bits 10–23: the double-damage chance), bit 24
/// whether it commands its First Aid Tent (bits 25–40: the heal bonus), bits 41–42 the attacker
/// hero's Tactics level if bit 43 is set, bit 44 whether the defender is led by a hero too, with
/// Tactics level bits 45–46 (so that some games start with a tactics phase, BAT-TAC-001), bit
/// 47 whether the attacker's hero wears Shackles of War (no retreat, BAT-FLE-001), bits 48–55
/// and 56–63 the attacker's and the defender's funds in hundreds of gold (BAT-FLE-002).
fn make_setup(
    r: &Ruleset,
    attacker: &[(u8, u8, u32)],
    defender: &[(u8, u8, u32)],
    obstacles: &[u8],
    machines: u64,
) -> BattleSetup {
    let pick = |bits: u64| -> Vec<_> {
        ["ballista", "cart", "tent"]
            .iter()
            .enumerate()
            .filter(|(i, _)| bits & (1 << i) != 0)
            .map(|(_, k)| r.creature_by_key(&std::format!("t:{k}")).unwrap())
            .collect()
    };
    let war_machines = [pick(machines & 7), pick((machines >> 3) & 7)];
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
    let mut led = hero(r);
    if machines & (1 << 8) != 0 {
        led.war_machines.ballista = Some(BallistaControl {
            shots: 1 + u8::from(machines & (1 << 9) != 0),
            double_damage_chance_bp: u16::try_from((machines >> 10) % 16_384)
                .unwrap()
                .min(10_000),
        });
    }
    if machines & (1 << 24) != 0 {
        led.war_machines.tent = Some(TentControl {
            heal_bonus_bp: u32::try_from((machines >> 25) & 0xffff).unwrap(),
        });
    }
    if machines & (1 << 43) != 0 {
        led.tactics = u8::try_from((machines >> 41) & 3).unwrap();
    }
    if machines & (1 << 47) != 0 {
        led.effects.push(h3_contracts::BattleEffect::NoRetreat);
    }
    let defender_hero = (machines & (1 << 44) != 0).then(|| {
        let mut h = hero(r);
        h.tactics = u8::try_from((machines >> 45) & 3).unwrap();
        h
    });
    BattleSetup {
        attacker: army(attacker),
        defender: army(defender),
        obstacles: obstacles.to_vec(),
        heroes: [Some(led), defender_hero],
        terrain: None,
        war_machines,
        funds: [48, 56].map(|bit| u32::try_from((machines >> bit) & 0xff).unwrap() * 100),
        formations: [6, 7].map(|bit| {
            if machines & (1 << bit) == 0 {
                Formation::Loose
            } else {
                Formation::Tight
            }
        }),
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
            // The commands that leave the battle come last.
            let staying = legal.iter().take_while(|c| !c.leaves_battle()).count();
            let cmd = legal
                .get(usize::from(*n) % staying.max(1))
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
        let r = lively();
        let setup = make_setup(&r, &att, &def, &obstacles, seed);
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
            let mut events = Vec::new();
            let result = apply(&mut state, &r, actor, cmd, &mut events);
            match result {
                Err(_) => {
                    // Atomicity: a rejected command changes nothing.
                    prop_assert_eq!(&state, &before);
                    prop_assert_eq!(state.state_hash(), before_hash);
                    prop_assert!(events.is_empty());
                    // ... and it was not in the legal set.
                    prop_assert!(!legal.iter().any(|c| *c == cmd), "legal {cmd:?} rejected");
                }
                Ok(()) => {
                    prop_assert!(legal.iter().any(|c| *c == cmd), "illegal {cmd:?} accepted");
                    let mover = before.active().unwrap();
                    let target = match cmd {
                        Command::Attack { target, .. } | Command::Shoot { target, .. } => Some(target),
                        _ => None,
                    };
                    // The mover's primary strikes hit the commanded target; retaliations hit the
                    // mover (BAT-ATK-001, BAT-RET-001).
                    for e in &events {
                        if let EventKind::Struck { attacker, target: hit, flags, .. } = e.kind {
                            if flags.contains(StrikeFlags::SECONDARY) {
                                continue;
                            }
                            if attacker == mover {
                                prop_assert_eq!(Some(hit), target);
                            } else if !before.stacks()[attacker.index()].has(crate::StackTraits::WAR_MACHINE) {
                                // War machines act on their own after the command (BAT-WAR-002).
                                prop_assert!(flags.contains(StrikeFlags::RETALIATION));
                                prop_assert_eq!(hit, mover);
                            }
                        }
                    }
                    // Good morale keeps the mover active for another turn (BAT-MOR-003).
                    if events.iter().any(|e| matches!(e.kind, EventKind::GoodMorale { .. })) {
                        prop_assert_eq!(state.active(), Some(mover));
                        prop_assert!(state.stacks()[mover.index()].flags.contains(crate::StackFlags::HAD_MORALE));
                    }
                    let mentioned = |i: usize, f: fn(&EventKind, usize) -> bool| events.iter().any(|e| f(&e.kind, i));
                    for (i, (b, a)) in before.stacks().iter().zip(state.stacks()).enumerate() {
                        // Conservation: creatures only ever die, except that life drain brings
                        // back up to the starting count (BAT-DMG-008).
                        let drained = mentioned(i, |k, i| matches!(k, EventKind::LifeDrained { stack, .. } if stack.index() == i));
                        prop_assert!(a.count <= b.count || (drained && a.count <= a.base_count));
                        prop_assert!(a.is_alive() || a.top_hp == 0);
                        prop_assert!(!a.is_alive() || (1..=a.max_hp).contains(&a.top_hp));
                        // Morale and luck are materialised once and stay in range.
                        prop_assert_eq!((a.morale, a.luck), (b.morale, b.luck));
                        prop_assert!((-3..=3).contains(&a.morale) && (0..=3).contains(&a.luck));
                        // Ownership isolation: the actor's command cannot move or re-stat enemy
                        // stacks, and only changes the enemy stacks it strikes (or that
                        // regenerate at the start of their turn, BAT-TRN-005).
                        prop_assert_eq!((a.creature, a.side, a.slot, a.attack, a.defense, a.speed), (b.creature, b.side, b.slot, b.attack, b.defense, b.speed));
                        if b.side != actor {
                            prop_assert_eq!(a.hex, b.hex);
                            let explained = mentioned(i, |k, i| matches!(k,
                                EventKind::Struck { target, .. } | EventKind::DeathStared { target, .. } if target.index() == i))
                                || mentioned(i, |k, i| matches!(k, EventKind::Regenerated { stack, .. } if stack.index() == i))
                                || mentioned(i, |k, i| matches!(k, EventKind::Healed { target, .. } if target.index() == i));
                            if !explained {
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

    /// Observation hygiene: two states that differ only in hidden information (the RNG, the
    /// heroes' Tactics levels) look the same to both sides.
    #[test]
    fn observations_do_not_depend_on_hidden_state((att, def, obstacles, seed, steps) in arb_game(), other_seed: u64) {
        let r = lively();
        let (state, _, _) = play(&r, &make_setup(&r, &att, &def, &obstacles, seed), seed, &steps);
        let mut twin = state.clone();
        twin.rng = Rng::from_seed(other_seed);
        for hero in twin.heroes.iter_mut().flatten() {
            hero.tactics = 3 - hero.tactics;
        }
        for viewer in BattleSide::BOTH {
            prop_assert_eq!(observe(&state, viewer), observe(&twin, viewer));
        }
    }

    /// Clone equivalence: a clone taken mid-game evolves identically to the original.
    #[test]
    fn clones_evolve_identically((att, def, obstacles, seed, steps) in arb_game(), split in 0usize..160) {
        let r = lively();
        let setup = make_setup(&r, &att, &def, &obstacles, seed);
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
        let r = lively();
        let setup = make_setup(&r, &att, &def, &obstacles, seed);
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
    let r = lively();
    let setup = make_setup(
        &r,
        &[(0, 5, 20), (1, 6, 8), (2, 10, 2)],
        &[(0, 7, 15), (3, 5, 12), (4, 8, 6)],
        &[60, 77, 94],
        0b111_111,
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

/// The Ballista's target by the full ranking of BAT-WAR-002 (VCMI's order, every key evaluated
/// eagerly): not a war machine, then could attack next turn, then value, then the lower id.
fn reference_target(state: &BattleState, r: &Ruleset, shooter: StackId) -> Option<StackId> {
    let field = crate::Field::new(r.battle().field);
    let side = state.stacks()[shooter.index()].side;
    (0u8..)
        .zip(state.stacks())
        .filter(|(_, s)| s.is_alive() && s.side != side)
        .map(|(i, s)| {
            let id = StackId(i);
            let creature = !s.has(crate::StackTraits::WAR_MACHINE);
            let dangerous = crate::war::can_attack_next_turn(state, field, id);
            (
                id,
                creature,
                dangerous,
                crate::war::value(state, r, shooter, id),
            )
        })
        .max_by(|a, b| {
            (a.1, a.2)
                .cmp(&(b.1, b.2))
                .then_with(|| h3_kernel::arith::cmp_fractions(a.3, b.3))
                .then_with(|| b.0.cmp(&a.0))
        })
        .map(|c| c.0)
}

proptest! {
    #![proptest_config(Config { cases: 32, ..Config::default() })]

    /// The lazy target choice of the Ballista (BAT-WAR-002) equals the eager reference ranking
    /// in every state of random games with ballistae on both sides.
    #[test]
    fn the_lazy_ballista_target_equals_the_reference((att, def, obstacles, seed, steps) in arb_game()) {
        let r = lively();
        // Ballistae on both sides (bits 0 and 3), the other machines and formations at random.
        let setup = make_setup(&r, &att, &def, &obstacles, seed | 0b1001);
        let mut state = BattleState::new(&r, &setup, seed).unwrap();
        let mut legal = LegalCommands::new();
        let field = crate::Field::new(r.battle().field);
        for step in &steps {
            for (i, s) in (0u8..).zip(state.stacks()) {
                if s.is_alive() && s.has(crate::StackTraits::WAR_MACHINE) && s.has(crate::StackTraits::SHOOTER) {
                    let id = StackId(i);
                    prop_assert_eq!(
                        crate::war::shooting_target(&state, &r, field, id),
                        reference_target(&state, &r, id)
                    );
                }
            }
            let (actor, cmd) = resolve(&state, &r, step, &mut legal);
            let _ = apply(&mut state, &r, actor, cmd, &mut crate::NoopSink);
        }
    }
}
