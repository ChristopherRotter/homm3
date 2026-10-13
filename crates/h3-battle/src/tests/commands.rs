//! Command validation, setup validation, legal commands, observation and event projection.

use std::vec;
use std::vec::Vec;

use h3_contracts::{ArmyStack, BattleSetup, BattleSide};

use super::support::*;
use crate::{
    BattleState, Command, CommandError, Event, EventKind, Hex, LegalCommands, NoopSink, SetupError,
    Sink, StackId, Visibility, apply, legal_commands, observe, project, validate,
};

#[test]
fn only_the_side_to_act_may_command() {
    let r = ruleset();
    let s = battle(&r, &[(0, "fixed", 1)], &[(0, "slow", 1)], 1);
    assert_eq!(s.side_to_act(), Some(BattleSide::Attacker));
    for cmd in [
        Command::Wait,
        Command::Defend,
        Command::Move { to: at(&r, 2, 5) },
    ] {
        assert_eq!(
            validate(&s, &r, BattleSide::Defender, cmd),
            Err(CommandError::NotYourTurn)
        );
        assert_eq!(validate(&s, &r, BattleSide::Attacker, cmd), Ok(()));
    }
    let mut legal = LegalCommands::new();
    legal_commands(&s, &r, BattleSide::Defender, &mut legal);
    assert!(legal.is_empty());
}

#[test]
fn attack_targets_must_be_living_enemies() {
    let r = ruleset();
    let mut s = battle(
        &r,
        &[(0, "fixed", 1), (1, "slow", 1)],
        &[(0, "slow", 1), (1, "slow", 1)],
        1,
    );
    activate(&mut s, 0);
    place(&mut s, &r, 0, 1, 5);
    place(&mut s, &r, 1, 1, 6);
    place(&mut s, &r, 2, 2, 5);
    let attack = |target| Command::Attack {
        target: StackId(target),
        from: at(&r, 1, 5),
    };
    assert_eq!(
        validate(&s, &r, BattleSide::Attacker, attack(1)),
        Err(CommandError::NotAnEnemy)
    );
    assert_eq!(
        validate(&s, &r, BattleSide::Attacker, attack(9)),
        Err(CommandError::NoSuchTarget)
    );
    assert_eq!(validate(&s, &r, BattleSide::Attacker, attack(2)), Ok(()));
    s.stacks.as_mut_slice()[2].count = 0;
    assert_eq!(
        validate(&s, &r, BattleSide::Attacker, attack(2)),
        Err(CommandError::NoSuchTarget)
    );
}

#[test]
fn setup_is_validated() {
    let r = ruleset();
    let good = setup(&r, &[(0, "fixed", 1)], &[(0, "slow", 1)]);
    assert!(BattleState::new(&r, &good, 1).is_ok());
    let with = |f: fn(&mut BattleSetup)| {
        let mut s = good.clone();
        f(&mut s);
        BattleState::new(&r, &s, 1).unwrap_err()
    };
    assert_eq!(
        with(|s| s.attacker.stacks.clear()),
        SetupError::EmptyArmy(BattleSide::Attacker)
    );
    assert_eq!(
        with(|s| s.defender.stacks.clear()),
        SetupError::EmptyArmy(BattleSide::Defender)
    );
    assert_eq!(
        with(|s| s.attacker.stacks[0].slot = 7),
        SetupError::SlotOutOfRange {
            side: BattleSide::Attacker,
            slot: 7
        }
    );
    assert_eq!(
        with(|s| s.defender.stacks[0].slot = 200),
        SetupError::SlotOutOfRange {
            side: BattleSide::Defender,
            slot: 200
        }
    );
    assert_eq!(
        with(|s| s.attacker.stacks.push(s.attacker.stacks[0])),
        SetupError::DuplicateSlot {
            side: BattleSide::Attacker,
            slot: 0
        }
    );
    assert_eq!(
        with(|s| s.attacker.stacks[0].count = 0),
        SetupError::EmptyStack {
            side: BattleSide::Attacker,
            slot: 0
        }
    );
    assert_eq!(
        with(|s| s.defender.stacks[0].creature = h3_content::CreatureTypeId::from_raw(999)),
        SetupError::UnknownCreature {
            side: BattleSide::Defender,
            slot: 0
        }
    );
    // A full army of seven stacks is fine; an eighth stack cannot have a valid, unused slot.
    let full: Vec<ArmyStack> = (0..7)
        .map(|slot| ArmyStack {
            slot,
            ..good.attacker.stacks[0]
        })
        .collect();
    let mut s = good.clone();
    s.attacker.stacks = full;
    assert_eq!(BattleState::new(&r, &s, 1).map(|b| b.stacks().len()), Ok(8));
}

#[test]
fn legal_commands_list_waits_moves_and_attack_positions() {
    let r = ruleset();
    let mut s = battle(&r, &[(0, "slow", 1)], &[(0, "fast", 1)], 1);
    // The fast defender goes first; make the slow attacker (speed 2) active at (5, 5) next to it.
    activate(&mut s, 0);
    place(&mut s, &r, 0, 5, 5);
    place(&mut s, &r, 1, 6, 5);
    let mut legal = LegalCommands::new();
    legal_commands(&s, &r, BattleSide::Attacker, &mut legal);
    assert_eq!(legal.as_slice()[..2], [Command::Wait, Command::Defend]);
    let moves = legal
        .iter()
        .filter(|c| matches!(c, Command::Move { .. }))
        .count();
    // Speed 2: 6 hexes at distance 1 (one occupied by the target) + 12 at distance 2, except the
    // one behind the target, which is only reachable through the target's hex.
    assert_eq!(moves, 5 + 11);
    let attacks: Vec<Hex> = legal
        .iter()
        .filter_map(|c| match c {
            Command::Attack {
                target: StackId(1),
                from,
            } => Some(*from),
            _ => None,
        })
        .collect();
    // Five of the target's neighbours can be used (one is the attacker's own hex); the one behind
    // the target is out of reach.
    assert_eq!(attacks.len(), 5);
    assert!(!attacks.contains(&at(&r, 7, 5)));
    assert!(attacks.contains(&at(&r, 5, 5)));
    for c in legal.iter() {
        assert_eq!(validate(&s, &r, BattleSide::Attacker, *c), Ok(()), "{c:?}");
    }
}

#[test]
fn observation_hides_only_the_rng() {
    let r = ruleset();
    let a = battle(&r, &[(0, "fixed", 1)], &[(0, "slow", 1)], 1);
    let b = battle(&r, &[(0, "fixed", 1)], &[(0, "slow", 1)], 2);
    assert_ne!(a.state_hash(), b.state_hash(), "the hash covers the RNG");
    for viewer in BattleSide::BOTH {
        let o = observe(&a, &r, viewer);
        assert_eq!(o, observe(&b, &r, viewer));
        assert_eq!(o.viewer, viewer);
        assert_eq!(
            (o.round, o.active, o.last_side_acted, o.result),
            (1, Some(StackId(0)), None, None)
        );
        assert_eq!(o.stacks.as_slice(), a.stacks());
    }
}

#[test]
fn events_are_projected_per_viewer() {
    let public = Event::public(EventKind::Waited { stack: StackId(0) });
    let private = Event {
        visibility: Visibility::Side(BattleSide::Defender),
        kind: public.kind,
    };
    assert_eq!(project(&public, BattleSide::Attacker), Some(public));
    assert_eq!(project(&private, BattleSide::Defender), Some(private));
    assert_eq!(project(&private, BattleSide::Attacker), None);
    let mut sink = NoopSink;
    sink.emit(public);
    let mut log: Vec<Event> = vec![];
    log.emit(public);
    assert_eq!(log, [public]);
}

#[test]
fn applying_with_a_noop_sink_matches_a_recording_sink() {
    let r = ruleset();
    let mut a = battle(&r, &[(0, "fixed", 3)], &[(0, "slow", 4)], 7);
    let mut b = a.clone();
    let cmd = Command::Move { to: at(&r, 4, 5) };
    apply(&mut a, &r, BattleSide::Attacker, cmd, &mut NoopSink).unwrap();
    let mut events = Vec::new();
    apply(&mut b, &r, BattleSide::Attacker, cmd, &mut events).unwrap();
    assert_eq!(a, b);
    assert_eq!(a.state_hash(), b.state_hash());
    assert_eq!(a.last_side_acted(), Some(BattleSide::Attacker));
    assert_eq!(a.stack(StackId(0)).map(|s| s.hex), Some(at(&r, 4, 5)));
    assert_eq!(a.stack(StackId(5)), None);
    assert_eq!(StackId(3).index(), 3);
    assert_eq!(Command::default(), Command::Wait);
}

// covers: BAT-HER-003
#[test]
fn the_state_hash_tells_luck_modifiers_from_morale_ones() {
    // The modifier pool's stat is encoded (0 morale, 1 luck): a modifier of the same amount on
    // the other stat hashes differently (ADR-0012).
    use h3_kernel::{CanonicalHash, StateHasher};
    let encode = |stat: crate::morale::Stat| {
        let mut h = StateHasher::new();
        stat.hash_into(&mut h);
        h.finish()
    };
    assert_ne!(
        encode(crate::morale::Stat::Morale),
        encode(crate::morale::Stat::Luck)
    );
}

#[test]
fn state_hash_is_pinned() {
    // Golden value: pins the canonical encoding of the initial state (bump
    // BATTLE_STATE_HASH_VERSION when the encoding changes).
    let r = ruleset();
    let mut led = hero(&r);
    // Advanced Tactics against no hero: the state is in a tactics phase (BAT-TAC-001).
    led.tactics = 2;
    led.damage = h3_contracts::DamageBonuses {
        ranged: Rate::bp(1000),
        melee: Rate::bp(2000),
        reduction: Rate::bp(500),
    };
    led.war_machines.ballista = Some(h3_contracts::BallistaControl {
        shots: 2,
        double_damage_chance_bp: 7500,
    });
    led.war_machines.tent = Some(h3_contracts::TentControl {
        heal_bonus: Rate::bp(45_000),
    });
    led.surrender_discount_bp = 4000;
    // A regeneration for every stack of the side (BAT-HER-006).
    // And magic: a spell book, masteries and spell bonuses (BAT-MAG-001 to BAT-MAG-004).
    let bolt = r.spell_by_key("t:bolt").unwrap();
    led.effects = vec![
        h3_contracts::BattleEffect::Regeneration {
            creatures: h3_content::CreatureSelector::All,
            hit_points: 50,
        },
        h3_contracts::BattleEffect::SpellDamage {
            scope: h3_content::SpellScope::All,
            rate: Rate::bp(1500),
        },
        h3_contracts::BattleEffect::SpellSpecialty {
            spell: bolt,
            bp_per_level: 300,
            level: 10,
        },
        h3_contracts::BattleEffect::MagicResistance {
            rate: Rate::bp(500),
        },
        h3_contracts::BattleEffect::SpellDuration { rounds: 2 },
        // The casting limits and the spell defences (v40).
        h3_contracts::BattleEffect::NoCasting,
        h3_contracts::BattleEffect::SpellLevelLimit { max_level: 3 },
        h3_contracts::BattleEffect::SpellImmunity { spell: bolt },
        h3_contracts::BattleEffect::SpellLevelImmunity { max_level: 4 },
        h3_contracts::BattleEffect::MindImmunity,
        h3_contracts::BattleEffect::NegateImmunities,
        // An opening spell, still due in the tactics phase (v41).
        h3_contracts::BattleEffect::OpeningSpell {
            spell: r.spell_by_key("t:prayer").unwrap(),
            mastery: 3,
            power: 10,
        },
    ];
    led.magic.spell_book = Some(vec![bolt]);
    // Eagle Eye (v41): 44 %, the bolt.
    led.magic.eagle_eye = Some(h3_contracts::EagleEye {
        chance: Rate::bp(4400),
        spells: vec![bolt],
    });
    led.magic.masteries = masteries(&r, [2, 1, 0, 0]);
    // One removable obstacle (v28): a stone with its anchor beside it.
    let stone = h3_contracts::Obstacle {
        hexes: vec![at(&r, 7, 4).0, at(&r, 8, 4).0],
        anchor: Some(at(&r, 6, 4).0),
    };
    let setup = BattleSetup {
        heroes: [Some(led), None],
        funds: [1234, 99],
        obstacles: vec![stone],
        ..setup(&r, &[(0, "fixed", 3), (2, "wisp", 7)], &[(0, "slow", 4)])
    };
    let s = BattleState::new(&r, &setup, 7).unwrap();
    assert!(s.tactics().is_some());
    assert_eq!(s.state_hash(), GOLDEN, "got {:#018x}", s.state_hash());
    assert_eq!(crate::BATTLE_STATE_HASH_VERSION, 42);
}

const GOLDEN: u64 = 0xd81f_d842_f177_cb51;
