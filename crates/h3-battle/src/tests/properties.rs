//! Mandatory invariant property tests (docs/05-testing.md "Invariants every context must
//! provide"): atomicity, legality consistency, ownership isolation, conservation, observation
//! hygiene, clone equivalence and determinism.
//!
//! Battles are generated from random armies; each step either plays a random legal command or
//! throws an arbitrary (usually invalid) command at the engine, possibly from the wrong actor.

use std::vec::Vec;

use h3_content::{
    Affects, CreatureFlag, CreatureSelector, CreatureStat, Formation, ObstacleKind, Ruleset,
    SpellId, SpellScope,
};
use h3_contracts::{
    Army, ArmyStack, BallistaControl, BattleEffect, BattleHero, BattleSetup, BattleSide,
    DamageBonuses, StackOrigin, StackRecord, StatOp, TentControl,
};
use h3_kernel::Rng;
use proptest::collection::{btree_map, vec};
use proptest::prelude::{Just, Strategy, any, prop_assert, prop_assert_eq, prop_oneof, proptest};
use proptest::test_runner::Config;

use super::support::{CREATURES, Rate, hero, ruleset_with};
use crate::event::StrikeFlags;
use crate::{
    BattleState, CastTarget, CastTargets, Command, Event, EventKind, Hex, LegalCommands, SpellSet,
    StackId, apply, cast_targets, castable_spells, creature_cast_targets, legal_commands, observe,
    project, second_targets,
};

/// The test ruleset with frequent good and bad morale and luck (a 4-sided die) and the quirk
/// Q-0003 on, so that generated games exercise them.
fn lively() -> Ruleset {
    lively_with("t:quirky")
}

/// [`lively`] with the profile `profile`.
fn lively_with(profile: &str) -> Ruleset {
    ruleset_with(profile, |b| {
        b.morale_dice = 4;
        b.good_morale_chances = std::vec![1, 2, 3];
        b.bad_morale_chances = std::vec![1, 2, 3];
        b.luck_dice = 4;
        b.good_luck_chances = std::vec![1, 2, 3];
        // Low enough for the casts of a battle to reach it (BAT-MAG-037).
        b.side_stack_limit = 12;
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
    /// Cast the `n`-th castable spell at its `n`-th target (modulo their numbers), if any; for an
    /// odd `n` (or no castable spell) the active stack casts at its `n`-th target instead, if it
    /// may (BAT-MAG-048).
    Cast(u16),
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
        8 => (0u16..12, 0u8..3, any::<u8>(), any::<Option<u8>>()).prop_map(|(spell, kind, t, h)| Command::Cast {
            spell: SpellId::from_raw(spell),
            target: match kind {
                0 => CastTarget::None,
                1 => CastTarget::Stack(StackId(t % 40)),
                _ => CastTarget::Hex(Hex(t)),
            },
            second: h.map_or(CastTarget::None, |h| CastTarget::Hex(Hex(h))),
        }),
        8 => (0u8..3, any::<u8>()).prop_map(|(kind, t)| Command::CreatureCast {
            target: match kind {
                0 => CastTarget::None,
                1 => CastTarget::Stack(StackId(t % 40)),
                _ => CastTarget::Hex(Hex(t)),
            },
        }),
        1 => Just(Command::Retreat),
        1 => Just(Command::Surrender),
    ]
}

fn arb_step() -> impl Strategy<Value = Step> {
    prop_oneof![
        3 => any::<u16>().prop_map(Step::Legal),
        1 => (arb_command(), any::<bool>()).prop_map(|(command, wrong_actor)| Step::Arbitrary { command, wrong_actor }),
        1 => any::<u16>().prop_map(Step::Cast),
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

/// The obstacles over `hexes`, one per hex: by the hex modulo 3 without an anchor, anchored on
/// itself, or anchored on its left neighbour (which it does not block, BAT-QUIRK-3; perhaps
/// another obstacle's hex).
fn pieces(hexes: &[u8]) -> Vec<h3_contracts::Obstacle> {
    hexes
        .iter()
        .map(|&h| h3_contracts::Obstacle {
            hexes: std::vec![h],
            anchor: [None, Some(h), Some(h - 1)][usize::from(h % 3)],
        })
        .collect()
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
/// and 56–63 the attacker's and the defender's funds in hundreds of gold (BAT-FLE-002). Both
/// heroes have spell books (every combat spell of the test ruleset) and the attacker's some
/// masteries and spell bonuses; their mana comes from bits 0–5 and 6–11 of `machines`
/// rotated by 13 (BAT-MAG-001 to BAT-MAG-004), the battlefield's terrain from its bits 14–15
/// (none, the plain or lava, so that some stacks are native: BAT-TER-001, BAT-MAG-031).
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
            heal_bonus: Rate::bp(u32::try_from((machines >> 25) & 0xffff).unwrap()),
        });
    }
    if machines & (1 << 43) != 0 {
        led.tactics = u8::try_from((machines >> 41) & 3).unwrap();
    }
    if machines & (1 << 47) != 0 {
        led.effects.push(BattleEffect::NoRetreat);
    }
    let magic = machines.rotate_left(13);
    let book: Vec<SpellId> = [
        "bolt",
        "smite",
        "ward",
        "nova",
        "portal",
        "chain",
        "storm",
        "ring",
        "haste",
        "sloth",
        "prayer",
        "weaken",
        "ray",
        "skin",
        "lust",
        "aim",
        "bless",
        "curse",
        "mirth",
        "sorrow",
        "fortune",
        "misfortune",
        "dispel",
        "wall",
        "firewall",
        "teleport",
        "quicksand",
        "landmine",
        "remove",
        "rise",
        "animate",
        "offering",
        "copy",
        "sprites",
        "serfs",
    ]
    .iter()
    .map(|k| r.spell_by_key(&std::format!("t:{k}")).unwrap())
    .collect();
    led.magic.spell_book = Some(book.clone());
    led.magic.mana = u16::try_from(magic & 63).unwrap();
    led.magic.schools = [u8::try_from((magic >> 12) & 3).unwrap(), 1, 0, 0];
    led.power = 2;
    led.effects.extend([
        BattleEffect::SpellDamage {
            scope: SpellScope::All,
            rate: Rate::bp(1000),
        },
        BattleEffect::SpellSpecialty {
            spell: book[0],
            bp_per_level: 300,
            level: 5,
        },
        BattleEffect::MagicResistance {
            rate: Rate::bp(500),
        },
        BattleEffect::SpellDuration { rounds: 1 },
    ]);
    let defender_hero = (machines & (1 << 44) != 0).then(|| {
        let mut h = hero(r);
        h.tactics = u8::try_from((machines >> 45) & 3).unwrap();
        h.magic.spell_book = Some(book.clone());
        h.magic.mana = u16::try_from((magic >> 6) & 63).unwrap();
        h
    });
    BattleSetup {
        attacker: army(attacker),
        defender: army(defender),
        obstacles: pieces(obstacles),
        heroes: [Some(led), defender_hero],
        terrain: match (magic >> 14) & 3 {
            1 => r.terrain_by_key("t:plain"),
            2 => r.terrain_by_key("t:lava"),
            _ => None,
        },
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
        Step::Cast(n) => {
            let spells: Vec<SpellId> = castable_spells(state, r, side).iter().collect();
            let mut creature = CastTargets::new();
            creature_cast_targets(state, r, side, &mut creature);
            if !creature.is_empty() && (n % 2 == 1 || spells.is_empty()) {
                let target = creature.as_slice()[usize::from(*n) % creature.len()];
                return (side, Command::CreatureCast { target });
            }
            let Some(&spell) = spells.get(usize::from(*n) % spells.len().max(1)) else {
                return (side, Command::Wait);
            };
            let mut targets = CastTargets::new();
            cast_targets(state, r, side, spell, &mut targets);
            let target = targets.as_slice()[usize::from(*n) % targets.len()];
            second_targets(state, r, side, (spell, target), &mut targets);
            let second = targets.as_slice()[usize::from(*n) % targets.len()];
            (
                side,
                Command::Cast {
                    spell,
                    target,
                    second,
                },
            )
        }
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
            // The same for casts: every listed spell, target and second target is accepted
            // (BAT-MAG-001, BAT-MAG-029).
            let castable = castable_spells(&state, &r, actor);
            let mut targets = CastTargets::new();
            let mut seconds = CastTargets::new();
            for spell in castable.iter() {
                cast_targets(&state, &r, actor, spell, &mut targets);
                prop_assert!(!targets.is_empty());
                for target in targets.iter() {
                    second_targets(&state, &r, actor, (spell, *target), &mut seconds);
                    prop_assert!(!seconds.is_empty());
                    for second in seconds.iter() {
                        let mut probe = state.clone();
                        let c = Command::Cast { spell, target: *target, second: *second };
                        prop_assert!(apply(&mut probe, &r, actor, c, &mut crate::NoopSink).is_ok(), "legal {c:?} rejected");
                    }
                }
            }
            prop_assert!(castable_spells(&state, &r, actor.opponent()).is_empty() || state.side_to_act() == Some(actor.opponent()));
            // And for the active stack's casts (BAT-MAG-048).
            creature_cast_targets(&state, &r, actor, &mut targets);
            for target in targets.iter() {
                let mut probe = state.clone();
                let c = Command::CreatureCast { target: *target };
                prop_assert!(apply(&mut probe, &r, actor, c, &mut crate::NoopSink).is_ok(), "legal {c:?} rejected");
            }
            creature_cast_targets(&state, &r, actor.opponent(), &mut targets);
            prop_assert!(targets.is_empty() || state.side_to_act() == Some(actor.opponent()));
            let listed = |c: Command| match c {
                Command::Cast { spell, target, second } => {
                    let mut t = CastTargets::new();
                    cast_targets(&state, &r, actor, spell, &mut t);
                    let first = castable.contains(spell) && t.iter().any(|x| *x == target);
                    second_targets(&state, &r, actor, (spell, target), &mut t);
                    first && t.iter().any(|x| *x == second)
                }
                Command::CreatureCast { target } => {
                    let mut t = CastTargets::new();
                    creature_cast_targets(&state, &r, actor, &mut t);
                    t.iter().any(|x| *x == target)
                }
                other => legal.iter().any(|c| *c == other),
            };
            let cmd_listed = listed(cmd);

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
                    prop_assert!(!cmd_listed, "legal {cmd:?} rejected");
                }
                Ok(()) => {
                    prop_assert!(cmd_listed, "illegal {cmd:?} accepted");
                    // Mana is spent only by casts, by the spell's cost, at most once per round
                    // and side (BAT-MAG-001, BAT-MAG-002); a cast keeps the turn ...
                    let per_round = r.battle().hero_casts_per_round;
                    for side in BattleSide::BOTH {
                        let (b, a) = (before.hero(side), state.hero(side));
                        if let (Some(b), Some(a)) = (b, a) {
                            prop_assert!(a.mana <= b.mana && a.casts <= per_round);
                            let cast_by = matches!(cmd, Command::Cast { .. }) && side == actor;
                            prop_assert!(cast_by || a.mana == b.mana);
                        }
                    }
                    // ... unless it killed the active stack, whose turn then ends (BAT-MAG-004,
                    // BAT-MAG-036).
                    if let Command::Cast { .. } = cmd {
                        let killed = before.active().is_some_and(|a| !state.stacks()[a.index()].is_alive());
                        prop_assert!(state.is_over() || state.active() == before.active() || killed);
                    }
                    let mover = before.active().unwrap();
                    // A stack's cast uses one of its casts and nothing else does (BAT-MAG-048).
                    let used = usize::from(matches!(cmd, Command::CreatureCast { .. }));
                    for (i, _) in before.stacks().iter().enumerate() {
                        let id = StackId(u8::try_from(i).unwrap());
                        let spent = usize::from(id == mover) * used;
                        let reused = events.iter().any(|e| matches!(e.kind, EventKind::StackCreated { stack, .. } if stack == id));
                        prop_assert!(reused || usize::from(before.casts_left(id)) == usize::from(state.casts_left(id)) + spent);
                    }
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
                        // A created stack, or a stack raised from a corpse, may take a dead
                        // created stack's slot (BAT-MAG-037, BAT-MAG-052).
                        if mentioned(i, |k, i| matches!(k, EventKind::StackCreated { stack, .. } | EventKind::RoseFromCorpse { stack, .. } if stack.index() == i)) {
                            prop_assert!(!b.is_alive() && b.is_created() && a.side == b.side);
                            prop_assert!(a.is_created() != a.has(crate::StackTraits::RAISED));
                            continue;
                        }
                        // Conservation: creatures only ever die, except that life drain and the
                        // raising spells bring back up to the starting count (BAT-DMG-008,
                        // BAT-MAG-035); the creatures raised for the battle only are among them.
                        let drained = mentioned(i, |k, i| matches!(k, EventKind::LifeDrained { stack, .. } | EventKind::SpellRaised { target: stack, .. } if stack.index() == i));
                        prop_assert!(a.count <= b.count || (drained && a.count <= a.base_count));
                        let id = StackId(u8::try_from(i).unwrap());
                        prop_assert!(state.temporary(id) <= a.base_count && (state.temporary(id) == before.temporary(id) || drained));
                        prop_assert!(a.is_alive() || a.top_hp == 0);
                        prop_assert!(!a.is_alive() || (1..=a.max_hp).contains(&a.top_hp));
                        // Morale and luck are materialised and change only with the stack's timed
                        // effects (BAT-MAG-011); they stay in range.
                        prop_assert!((a.morale, a.luck) == (b.morale, b.luck) || a.spells != b.spells);
                        prop_assert!((-3..=3).contains(&a.morale) && (0..=3).contains(&a.luck));
                        // Ownership isolation: the actor's command cannot move or re-stat enemy
                        // stacks, and only changes the enemy stacks it strikes (or that
                        // regenerate at the start of their turn, BAT-TRN-005).
                        prop_assert_eq!((a.creature, a.side, a.slot, a.attack, a.defense, a.speed), (b.creature, b.side, b.slot, b.attack, b.defense, b.speed));
                        if b.side != actor {
                            prop_assert_eq!(a.hex, b.hex);
                            let explained = mentioned(i, |k, i| matches!(k,
                                EventKind::Struck { target, .. } | EventKind::DeathStared { target, .. } | EventKind::SpellDamage { target, .. } | EventKind::StrikeDamage { target, .. } if target.index() == i))
                                || mentioned(i, |k, i| matches!(k, EventKind::Regenerated { stack, .. } | EventKind::Vanished { stack, .. } if stack.index() == i))
                                || mentioned(i, |k, i| matches!(k, EventKind::Healed { target, .. } if target.index() == i))
                                // The other side's periodic Cure at its caster's activation after
                                // the command (BAT-MAG-053).
                                || mentioned(i, |k, i| matches!(k, EventKind::SpellHealed { target, .. } if target.index() == i))
                                // Poison and Age change the hit points per creature (BAT-MAG-044).
                                || mentioned(i, |k, i| matches!(k, EventKind::Poisoned { stack, .. } | EventKind::EffectEnded { target: stack, .. } if stack.index() == i));
                            if !explained {
                                prop_assert_eq!((a.count, a.top_hp), (b.count, b.top_hp));
                            }
                        }
                    }
                }
            }
            // The effect list (ADR-0014 point 5): sorted by stack and spell, each once, on
            // living stacks only, timed spells only, never at 0 rounds; each stack's values are
            // those of its entries (BAT-MAG-009 to 011).
            let effects = state.effects();
            prop_assert!(effects.windows(2).all(|w| (w[0].stack, w[0].spell) < (w[1].stack, w[1].spell)));
            for e in effects {
                prop_assert!(state.stacks()[e.stack.index()].is_alive());
                prop_assert!(e.rounds != Some(0));
                prop_assert!(r.spell(e.spell).unwrap().magic.is_timed());
            }
            for i in 0..state.stacks().len() {
                let mut fresh = state.clone();
                crate::effects::recompute(&mut fresh, &r, StackId(u8::try_from(i).unwrap()));
                if state.stacks()[i].is_alive() {
                    prop_assert_eq!(fresh.stacks()[i].spells, state.stacks()[i].spells);
                }
            }
            // A battle that is over has no active stack, and vice versa.
            prop_assert_eq!(state.is_over(), state.active().is_none());
            // Living stacks never share a hex (BAT-GEO-002; a clone neither, BAT-QUIRK-7); a side
            // has at most the stack limit of stacks; exactly the living clones have rounds left,
            // and created stacks leave no corpse (BAT-MAG-037, BAT-MAG-038).
            let field = crate::geometry::Field::new(r.battle().field);
            let mut taken = crate::geometry::HexSet::new();
            for (i, s) in state.stacks().iter().enumerate() {
                let id = StackId(u8::try_from(i).unwrap());
                if s.is_alive() {
                    for h in s.footprint(field).hexes() {
                        prop_assert!(!taken.contains(h), "two stacks on {h:?}");
                        taken.insert(h);
                    }
                }
                let clone = s.is_alive() && s.has(crate::StackTraits::CLONE);
                prop_assert_eq!(clone, state.clone_rounds(id) > 0);
                prop_assert!(s.is_alive() || !s.is_created() || state.corpse(id) == crate::NO_CORPSE);
            }
            for side in BattleSide::BOTH {
                let n = state.stacks().iter().filter(|s| s.side == side).count();
                prop_assert!(n <= usize::from(r.battle().side_stack_limit));
            }
        }
    }

    /// Observation hygiene (ADR-0013 point 9): hidden information does not show. The RNG shows
    /// to neither side; the private part of a side's hero (Tactics level, damage bonuses, how
    /// its war machines work, surrender discount, funds) only to that side, and so does the spell
    /// its random caster drew (ADR-0014 point 8, BAT-MAG-049).
    #[test]
    fn observations_do_not_depend_on_hidden_state(
        (att, def, obstacles, seed, steps) in arb_game(),
        other_seed: u64,
        damage: [u16; 2],
        (reduction, chance, discount) in (0u16..=10_000, 0u16..=10_000, 0u16..=10_000),
        (shots, heal, funds) in (1u8..=3, any::<u32>(), any::<u32>()),
    ) {
        let r = lively();
        let (state, _, _) = play(&r, &make_setup(&r, &att, &def, &obstacles, seed), seed, &steps);
        let mut twin = state.clone();
        twin.rng = Rng::from_seed(other_seed);
        for viewer in BattleSide::BOTH {
            prop_assert_eq!(observe(&state, &r, viewer), observe(&twin, &r, viewer));
        }
        for side in BattleSide::BOTH {
            let mut twin = state.clone();
            for hero in twin.heroes[side.index()].iter_mut() {
                hero.tactics = 3 - hero.tactics;
                hero.damage = DamageBonuses {
                    ranged: Rate::bp(damage[0].into()),
                    melee: Rate::bp(damage[1].into()),
                    reduction: Rate::bp(reduction.into()),
                };
                let ballista = BallistaControl { shots, double_damage_chance_bp: chance };
                hero.ballista = hero.ballista.and(Some(ballista));
                hero.tent = hero.tent.and(Some(TentControl { heal_bonus: Rate::bp(heal )}));
                hero.surrender_discount_bp = discount;
                hero.funds = funds;
                hero.magic.book = SpellSet::default();
                hero.magic.schools = [3; 4];
                hero.magic.bonuses.all = Rate::bp(discount.into());
            }
            // covers: BAT-MAG-049
            twin.casts.drawn = twin.casts.drawn.map(|(s, spell)| {
                let other = SpellId::from_raw(spell.raw() ^ 1);
                (s, if twin.at(s).controller() == side { other } else { spell })
            });
            let viewer = side.opponent();
            prop_assert_eq!(observe(&state, &r, viewer), observe(&twin, &r, viewer));
        }
    }

    /// Observation hygiene for hidden obstacles (ADR-0014 point 7, BAT-MAG-031): what a side
    /// does not see changes neither its observation nor what it may do (its legal commands,
    /// castable spells and their targets), with the natives' sight of either profile; nor what
    /// its commands report to it while no hidden obstacle shows itself (a pit it runs into, a
    /// mine that goes off: BAT-MAG-032, BAT-MAG-033; a native it raises gives its side sight
    /// without quirk BAT-QUIRK-1, BAT-MAG-031), casts included: without quirk BAT-QUIRK-4
    /// a random obstacle spell draws among the hexes its side sees free (BAT-MAG-032), and Remove
    /// Obstacle finds only what the caster sees (BAT-MAG-034). Anchors (BAT-QUIRK-3) in half the
    /// games.
    #[test]
    fn hidden_obstacles_do_not_show(
        (att, def, obstacles, seed, steps) in arb_game(),
        at_start: bool,
        anchors: bool,
        extra in proptest::collection::btree_set((1u8..16, 0u8..11), 0..6),
    ) {
        let r = match (at_start, anchors) {
            (true, _) => lively_with("t:sight"),
            (false, true) => lively_with("t:anchors"),
            (false, false) => lively_with("t:quirky"),
        };
        let (mut state, _, _) =
            play(&r, &make_setup(&r, &att, &def, &obstacles, seed), seed, &steps);
        // More pits and mines of either side, so that most games have hidden ones.
        let kinds = [("t:quicksand", ObstacleKind::Pit), ("t:landmine", ObstacleKind::Mine)];
        for (i, (x, y)) in extra.into_iter().enumerate() {
            let side = BattleSide::BOTH[i % 2];
            let (key, kind) = kinds[i / 2 % 2];
            let id = r.spell_by_key(key).unwrap();
            let shape = r.spell(id).unwrap().magic.obstacle.clone().unwrap();
            let hex = [Hex(x + 17 * y)];
            let placed = (id, kind, &shape);
            crate::obstacles::place(&mut state, placed, side, (&hex, 40), &mut Vec::new());
        }
        for viewer in BattleSide::BOTH {
            let mut twin = state.clone();
            let seen: Vec<_> = crate::obstacles::seen_by(&state, viewer);
            twin.spell_obstacles.retain(|o| seen.contains(o));
            prop_assert_eq!(observe(&state, &r, viewer), observe(&twin, &r, viewer));
            let (mut a, mut b) = (LegalCommands::new(), LegalCommands::new());
            legal_commands(&state, &r, viewer, &mut a);
            legal_commands(&twin, &r, viewer, &mut b);
            prop_assert_eq!(a.as_slice(), b.as_slice());
            let mut commands: Vec<Command> = a.as_slice().to_vec();
            let spells = castable_spells(&state, &r, viewer);
            prop_assert_eq!(spells, castable_spells(&twin, &r, viewer));
            for spell in spells.iter() {
                let (mut ta, mut tb) = (CastTargets::new(), CastTargets::new());
                cast_targets(&state, &r, viewer, spell, &mut ta);
                cast_targets(&twin, &r, viewer, spell, &mut tb);
                prop_assert_eq!(ta.as_slice(), tb.as_slice());
                // Casts too: the first, middle and last target, each with its first second
                // target (legal commands list no casts).
                let n = ta.len();
                let mut picks = std::vec![0, n / 2, n.saturating_sub(1)];
                picks.dedup();
                for target in picks.into_iter().filter_map(|i| ta.get(i).copied()) {
                    let (mut sa, mut sb) = (CastTargets::new(), CastTargets::new());
                    second_targets(&state, &r, viewer, (spell, target), &mut sa);
                    second_targets(&twin, &r, viewer, (spell, target), &mut sb);
                    prop_assert_eq!(sa.as_slice(), sb.as_slice());
                    if let Some(&second) = sa.get(0) {
                        commands.push(Command::Cast { spell, target, second });
                    }
                }
            }
            for &command in &commands {
                let (mut x, mut y) = (state.clone(), twin.clone());
                let (mut ex, mut ey) = (Vec::new(), Vec::new());
                prop_assert!(apply(&mut x, &r, viewer, command, &mut ex).is_ok());
                prop_assert!(apply(&mut y, &r, viewer, command, &mut ey).is_ok());
                if ex.iter().any(|e| matches!(e.kind, EventKind::ObstacleRevealed { .. })) {
                    continue;
                }
                let told = |events: &[Event]| -> Vec<Event> {
                    events.iter().filter_map(|e| project(e, viewer)).collect()
                };
                // A native the command raises gives its side sight again (BAT-MAG-031 without
                // BAT-QUIRK-1): the hidden obstacles it now sees show themselves.
                let unseen: Vec<_> = state
                    .spell_obstacles()
                    .iter()
                    .filter(|o| !seen.contains(o))
                    .map(|o| (o.side, o.hexes))
                    .collect();
                let sight = crate::obstacles::seen_by(&x, viewer);
                if sight.iter().any(|o| unseen.contains(&(o.side, o.hexes))) {
                    continue;
                }
                prop_assert_eq!(told(&ex), told(&ey));
                prop_assert_eq!(observe(&x, &r, viewer), observe(&y, &r, viewer));
            }
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

/// Plays `state` on with random legal commands, attacks first, until the battle is over (or a
/// step limit): with `leave` the commands that leave the battle are among them.
fn finish(state: &mut BattleState, r: &Ruleset, seed: u64, leave: bool) {
    let mut agent = Rng::for_stream(seed, h3_kernel::StreamId::AGENT);
    let mut legal = LegalCommands::new();
    for _ in 0..5_000 {
        let Some(side) = state.side_to_act() else {
            break;
        };
        legal_commands(state, r, side, &mut legal);
        let allowed = |c: &&Command| leave || !c.leaves_battle();
        // Prefer attacks so that random games end quickly.
        let attacks: Vec<Command> = legal
            .iter()
            .filter(allowed)
            .copied()
            .filter(|c| matches!(c, Command::Attack { .. }))
            .collect();
        let pool: Vec<Command> = if attacks.is_empty() {
            legal.iter().filter(allowed).copied().collect()
        } else {
            attacks
        };
        let cmd = pool[agent.below(pool.len() as u32) as usize];
        apply(state, r, side, cmd, &mut crate::NoopSink).unwrap();
    }
}

proptest! {
    #![proptest_config(Config { cases: 48, ..Config::default() })]

    /// Conservation with casualties accounted (E04a exit criterion 4, BAT-END-003): a finished
    /// battle's outcome lists every stack once, in stack order per side, with its start and end
    /// counts; the survivors and war machines are exactly the records with creatures left.
    #[test]
    fn outcomes_account_for_every_stack((att, def, obstacles, seed, steps) in arb_game()) {
        let r = lively();
        let (mut state, _, _) = play(&r, &make_setup(&r, &att, &def, &obstacles, seed), seed, &steps);
        finish(&mut state, &r, seed, false);
        prop_assert!(state.is_over());
        let o = state.outcome().unwrap();
        prop_assert_eq!(Some(o.result), state.result());
        prop_assert_eq!(o.rounds, state.round());
        for side in BattleSide::BOTH {
            // Every stack is reported in stack order, by origin (BAT-MAG-037, BAT-Q13).
            let stacks: Vec<_> = (0u8..).zip(state.stacks()).filter(|(_, s)| s.side == side).collect();
            let records = &o.stacks[side.index()];
            prop_assert_eq!(records.len(), stacks.len());
            for (record, &(id, s)) in records.iter().zip(&stacks) {
                let temporary = state.temporary(StackId(id));
                prop_assert!(temporary <= s.base_count);
                prop_assert_eq!((record.creature, record.start), (s.creature, s.base_count));
                let origin = if s.is_created() {
                    StackOrigin::Created
                } else if s.has(crate::StackTraits::WAR_MACHINE) {
                    StackOrigin::WarMachine
                } else if s.has(crate::StackTraits::RAISED) {
                    StackOrigin::Raised(s.slot)
                } else {
                    StackOrigin::Army(s.slot)
                };
                prop_assert_eq!(record.origin, origin);
                let army = matches!(origin, StackOrigin::Army(_) | StackOrigin::Raised(_));
                // A created stack that vanished keeps the creatures it vanished with (BAT-Q13).
                let count = s.count.max(state.created.vanished[usize::from(id)]);
                // Creatures raised for the battle only are left out, but an army whose living
                // creatures are all such keeps one in its last living stack (BAT-MAG-035).
                let kept = record.end == 1 && count.saturating_sub(temporary) == 0 && s.is_alive() && army;
                prop_assert!(record.end == count.saturating_sub(temporary) || kept);
                prop_assert!(record.end <= record.start);
            }
            let army = |rec: &&StackRecord| matches!(rec.origin, StackOrigin::Army(_) | StackOrigin::Raised(_));
            let any_army_alive = stacks.iter().any(|(_, s)| s.is_alive() && !s.has(crate::StackTraits::WAR_MACHINE) && !s.is_created());
            prop_assert_eq!(records.iter().filter(army).any(|rec| rec.end > 0), any_army_alive);
            let in_army = |rec: &&StackRecord| rec.end > 0 && army(rec);
            let machine = |rec: &&StackRecord| rec.end > 0 && rec.origin == StackOrigin::WarMachine;
            prop_assert_eq!(o.survivors[side.index()].stacks.len(), records.iter().filter(in_army).count());
            prop_assert_eq!(o.war_machines[side.index()].len(), records.iter().filter(machine).count());
            prop_assert_eq!(o.survivors[side.index()].stacks.iter().map(|s| s.count).sum::<u32>(), records.iter().filter(in_army).map(|rec| rec.end).sum::<u32>());
            // One survivor per army slot, in slot order (a raised stack takes a dead stack's
            // slot, BAT-MAG-052).
            prop_assert!(o.survivors[side.index()].stacks.windows(2).all(|w| w[0].slot < w[1].slot));
        }
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
        finish(&mut state, &r, seed, true);
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

/// One effect of an extreme hero: kind, selector (all, a flag, a creature), statistic,
/// operation, amount (also the morale/luck value and the regeneration), affected sides.
type ExtremeEffect = (u8, u8, usize, u8, i16, u8);

/// A battle effect from its generated parts; every kind except `NoRetreat` and the alignment
/// groups (both covered by `make_setup` and `tests/heroes.rs`).
fn extreme_effect(
    r: &Ruleset,
    &(kind, selector, creature, stat, amount, affects): &ExtremeEffect,
) -> BattleEffect {
    let creatures = match selector {
        0 => CreatureSelector::All,
        1 => CreatureSelector::Flag(if creature % 2 == 0 {
            CreatureFlag::Alive
        } else {
            CreatureFlag::Shooter
        }),
        _ => CreatureSelector::Creature(
            r.creature_by_key(&std::format!(
                "t:{}",
                CREATURES[creature % CREATURES.len()].0
            ))
            .unwrap(),
        ),
    };
    let stat = CreatureStat::ALL[usize::from(stat) % CreatureStat::ALL.len()];
    let affects = Affects::ALL[usize::from(affects) % 3];
    let op = [StatOp::Add, StatOp::PercentBp, StatOp::PercentBpDown][usize::from(kind) % 3];
    let value = i8::try_from(amount.clamp(i8::MIN.into(), i8::MAX.into())).unwrap();
    match kind {
        0..=2 => BattleEffect::StackStat {
            creatures,
            stat,
            op,
            amount,
        },
        3 => BattleEffect::Regeneration {
            creatures,
            hit_points: amount.unsigned_abs(),
        },
        4 => BattleEffect::NoRangePenalty { creatures },
        5 => BattleEffect::ShootWhenAdjacent { creatures },
        6 => BattleEffect::Morale {
            amount: value,
            affects,
        },
        7 => BattleEffect::Luck {
            amount: value,
            affects,
        },
        8 => BattleEffect::MoraleAtMost { value, affects },
        _ => BattleEffect::LuckAtMost { value, affects },
    }
}

/// A sound hero with values no real hero could have (BAT-HER-001, OPEN-QUESTIONS Q31): any
/// primary skills and damage bonuses, any Ballista and First Aid Tent control, any surrender
/// discount, any funds and up to 24 effects with extreme amounts.
fn arb_extreme_hero() -> impl Strategy<Value = Option<(BattleHero, u32)>> {
    let effect = (
        0u8..10,
        0u8..3,
        0usize..64,
        any::<u8>(),
        any::<i16>(),
        any::<u8>(),
    );
    proptest::option::weighted(
        0.9,
        (
            any::<[u8; 4]>(),
            0u8..=3,
            (any::<u16>(), any::<u16>(), 0u16..=10_000),
            proptest::option::of((1u8..=255, 0u16..=10_000)),
            proptest::option::of(any::<u32>()),
            0u16..=10_000,
            vec(effect, 0..24),
            any::<u32>(),
        ),
    )
    .prop_map(|raw| {
        raw.map(
            |(
                primary,
                tactics,
                (ranged_bp, melee_bp, reduction_bp),
                ballista,
                tent,
                discount,
                effects,
                funds,
            )| {
                let r = lively();
                let mut h = hero(&r);
                [h.attack, h.defense, h.power, h.knowledge] = primary;
                h.tactics = tactics;
                h.damage = DamageBonuses {
                    ranged: Rate::bp(ranged_bp.into()),
                    melee: Rate::bp(melee_bp.into()),
                    reduction: Rate::bp(reduction_bp.into()),
                };
                h.war_machines.ballista =
                    ballista.map(|(shots, double_damage_chance_bp)| BallistaControl {
                        shots,
                        double_damage_chance_bp,
                    });
                h.war_machines.tent = tent.map(|bp| TentControl {
                    heal_bonus: Rate::bp(bp),
                });
                h.surrender_discount_bp = discount;
                h.effects = effects.iter().map(|e| extreme_effect(&r, e)).collect();
                (h, funds)
            },
        )
    })
}

/// Armies with counts up to `u32::MAX` (above the death stare's roll limit too, KER-RNG-003).
fn arb_big_army() -> impl Strategy<Value = ArmySpec> {
    let pool = army_creatures();
    let count = prop_oneof![1u32..100, 60_000u32..200_000, 1u32..=u32::MAX];
    btree_map(0u8..7, (0..pool.len(), count), 1..=7).prop_map(move |m| {
        m.into_iter()
            .map(|(slot, (c, n))| (slot, pool[c], n))
            .collect()
    })
}

proptest! {
    #![proptest_config(Config { cases: 48, ..Config::default() })]

    // covers: BAT-HER-001
    /// Robustness (E04a T12c review): battles led on both sides by sound heroes with extreme
    /// values (BAT-HER-001 accepts them) and armies of up to `u32::MAX` creatures start, play
    /// with every legal command accepted (leaving in a quarter of them), and report an outcome when over;
    /// no arithmetic overflows.
    #[test]
    fn extreme_heroes_and_counts_play_without_failures(
        att in arb_big_army(),
        def in arb_big_army(),
        heroes in (arb_extreme_hero(), arb_extreme_hero()),
        machines in any::<u64>(),
        seed in any::<u64>(),
    ) {
        let r = lively();
        let mut setup = make_setup(&r, &att, &def, &[], machines);
        let [a, d] = [heroes.0, heroes.1];
        setup.funds = [a.as_ref().map_or(0, |h| h.1), d.as_ref().map_or(0, |h| h.1)];
        setup.heroes = [a.map(|h| h.0), d.map(|h| h.0)];
        let mut state = BattleState::new(&r, &setup, seed).unwrap();
        // A quarter of the games may leave the battle (random agents leave early).
        finish(&mut state, &r, seed, seed % 4 == 0);
        for side in BattleSide::BOTH {
            let _ = observe(&state, &r, side);
        }
        prop_assert_eq!(state.outcome().is_some(), state.is_over());
    }
}
