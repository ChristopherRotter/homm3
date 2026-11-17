//! Tests of a creature bank's battle (E08 T124a; ADR-0022 point 8; `BattleSetup::creature_bank`):
//! the stacks at the bank positions by their ordinal (BAT-SET-001), no tactics phase
//! (BAT-TAC-001), no obstacles and a setup refusing them or a siege (BAT-GEO-004), no war machines
//! on the field with the Ammo Cart's supply kept (BAT-WAR-001, BAT-WAR-004).

use std::vec;
use std::vec::Vec;

use h3_content::{ContentKey, Formation};
use h3_content_battle::BattleRuleset;
use h3_contracts::{BattleSetup, FortLevel, Moat, Siege};
use h3_kernel::{Rng, StreamId};

use super::support::*;
use crate::{
    BattleState, Command, Event, EventKind, Hex, SetupError, StackId, StackTraits,
    generate_setup_obstacles,
};

/// A creature bank's battle: the attacker led by the test hero, the guards without one.
fn bank(r: &BattleRuleset, attacker: &[Spec], defender: &[Spec]) -> BattleSetup {
    BattleSetup {
        creature_bank: true,
        heroes: [Some(hero(r)), None],
        ..setup(r, attacker, defender)
    }
}

/// Each stack's hexes by id, its front hex first (BAT-GEO-002).
fn cells(r: &BattleRuleset, s: &BattleState) -> Vec<Vec<u8>> {
    s.stacks()
        .iter()
        .map(|st| st.footprint(field(r)).hexes().map(|h| h.0).collect())
        .collect()
}

fn listed(r: &BattleRuleset, names: &[&str]) -> Vec<h3_content::CreatureTypeId> {
    names
        .iter()
        .map(|n| r.creature_by_key(&std::format!("t:{n}")).unwrap())
        .collect()
}

/// The shots `shooter` has left after one shot at `target`.
fn shots_after_a_shot(s: &mut BattleState, r: &BattleRuleset, shooter: u8, target: u8) -> u16 {
    activate(s, shooter);
    let at = s.stacks()[usize::from(target)].hex;
    let shot = Command::Shoot {
        target: StackId(target),
        at,
    };
    act(s, r, shot).unwrap();
    s.stacks()[usize::from(shooter)].shots
}

// covers: BAT-SET-001, BAT-GEO-002
#[test]
fn a_banks_stacks_stand_at_the_bank_positions_by_their_ordinal() {
    let r = ruleset();
    // The attacker's slots 1, 3 and 6 are its ordinals 0 to 2; the guards fill all seven.
    let attacker = [(1, "wide", 5), (3, "fixed", 5), (6, "archer", 5)];
    let defender = [
        (0, "wide", 5),
        (1, "fixed", 5),
        (2, "wide", 5),
        (3, "fixed", 5),
        (4, "wide", 5),
        (5, "fixed", 5),
        (6, "fixed", 5),
    ];
    let s = BattleState::new(&r, &bank(&r, &attacker, &defender), 1).unwrap();
    // The original's hexes 57, 61, 90 and 15, 185, 172, 2, 100, 87, 8 (0x63D0E0); a two-hex stack
    // at its rear hex, its front one column toward the enemy: the attacker's to the right, the
    // guards' to the left wherever they stand (batch E13's 57-58, 14-15, 171-172, 99-100).
    let expected: [&[u8]; 10] = [
        &[58, 57],
        &[61],
        &[90],
        &[14, 15],
        &[185],
        &[171, 172],
        &[2],
        &[99, 100],
        &[87],
        &[8],
    ];
    assert_eq!(cells(&r, &s), expected.map(<[u8]>::to_vec));
    assert_eq!(s.stacks()[0].hex, Hex(57));
    // The formation plays no part.
    let tight = BattleSetup {
        formations: [Formation::Tight; 2],
        ..bank(&r, &attacker, &defender)
    };
    let t = BattleState::new(&r, &tight, 1).unwrap();
    assert_eq!(cells(&r, &t), cells(&r, &s));
    // Without the flag the same armies stand in their formation's rows at the edges.
    let mut plain = bank(&r, &attacker, &defender);
    plain.creature_bank = false;
    let p = BattleState::new(&r, &plain, 1).unwrap();
    assert!(p.stacks().iter().all(|st| {
        let (x, _) = field(&r).coords(st.hex);
        x <= 2 || x >= 14
    }));
}

// covers: BAT-TAC-001
#[test]
fn a_banks_battle_has_no_tactics_phase() {
    let r = ruleset();
    let mut setup = bank(&r, &[(0, "fixed", 5)], &[(0, "fixed", 5)]);
    if let Some(h) = setup.heroes[0].as_mut() {
        h.tactics = 3;
    }
    let mut events: Vec<Event> = Vec::new();
    let s = BattleState::start(&r, &setup, 1, &mut events).unwrap();
    assert_eq!(s.tactics(), None);
    assert_eq!(s.round(), 1);
    assert!(
        !events
            .iter()
            .any(|e| matches!(e.kind, EventKind::TacticsStarted { .. }))
    );
    // Set up and begun apart (CLT-Q69 (a)), the battle keeps the flag; without it the phase runs.
    let mut apart = BattleState::setup(&r, &setup, 1, &mut events).unwrap();
    apart.begin(&r, &mut events);
    assert_eq!(apart.tactics(), None);
    setup.creature_bank = false;
    let t = BattleState::new(&r, &setup, 1).unwrap();
    assert!(t.tactics().is_some());
}

// covers: BAT-GEO-004
#[test]
fn a_banks_battle_has_no_obstacles_and_refuses_them_or_a_siege() {
    let r = ruleset();
    let draw = |creature_bank: bool, battlefield: Option<&str>| {
        let setup = BattleSetup {
            terrain: Some(r.terrain_by_key("t:plain").unwrap()),
            battlefield: battlefield.and_then(|k| ContentKey::parse(k).ok()),
            creature_bank,
            ..setup(&r, &[(0, "fixed", 1)], &[(0, "fixed", 1)])
        };
        let mut rng = Rng::for_stream(9, StreamId::OBSTACLES);
        generate_setup_obstacles(&r, &setup, &mut rng)
    };
    assert!(!draw(false, None).is_empty());
    assert!(draw(true, None).is_empty());
    // Nor the deck's boats (the original's 0x4662B3 returns before them).
    assert!(!draw(false, Some("t:boats")).is_empty());
    assert!(draw(true, Some("t:boats")).is_empty());
    // A caller's obstacles or a town beside the flag are refused.
    let with_obstacles = BattleSetup {
        obstacles: blocked(vec![at(&r, 8, 5).0]),
        ..bank(&r, &[(0, "fixed", 1)], &[(0, "fixed", 1)])
    };
    let town = Siege {
        faction: r.faction_by_key("t:castle").unwrap(),
        fort: FortLevel::None,
        moat: Moat::Absent,
        structures: 0,
        escape_tunnel: false,
        bonuses: h3_contracts::TownBonuses::NONE,
    };
    let at_town = BattleSetup {
        siege: Some(town),
        ..bank(&r, &[(0, "fixed", 1)], &[(0, "fixed", 1)])
    };
    for bad in [with_obstacles, at_town] {
        assert_eq!(
            BattleState::new(&r, &bad, 1).err(),
            Some(SetupError::InvalidCreatureBank)
        );
    }
}

// covers: BAT-WAR-001, BAT-WAR-004
#[test]
fn a_banks_battle_sets_up_no_war_machine_and_keeps_the_ammo_carts_supply() {
    let r = ruleset();
    let armies = (
        &[(0, "archer", 5), (1, "fixed", 5)][..],
        &[(0, "archer", 5), (1, "tank", 5)][..],
    );
    // The hero lists all three machines; the guards list a cart without a hero.
    let machines = [
        listed(&r, &["ballista", "cart", "tent"]),
        listed(&r, &["cart"]),
    ];
    let setup = BattleSetup {
        war_machines: machines.clone(),
        ..bank(&r, armies.0, armies.1)
    };
    let mut s = BattleState::new(&r, &setup, 1).unwrap();
    // Only the armies' four stacks: ids 0, 1 the attacker's, 2, 3 the guards'.
    assert_eq!(s.stacks().len(), 4);
    assert!(
        s.stacks()
            .iter()
            .all(|st| !st.has(StackTraits::WAR_MACHINE))
    );
    // The hero's archers keep their shots, the guards' use theirs up.
    assert_eq!(shots_after_a_shot(&mut s, &r, 0, 3), 12);
    assert_eq!(shots_after_a_shot(&mut s, &r, 0, 3), 12);
    assert_eq!(shots_after_a_shot(&mut s, &r, 2, 1), 11);
    // A hero without a cart is not supplied.
    let no_cart = BattleSetup {
        war_machines: [listed(&r, &["ballista"]), vec![]],
        ..setup.clone()
    };
    let mut n = BattleState::new(&r, &no_cart, 1).unwrap();
    assert_eq!(n.stacks().len(), 4);
    assert_eq!(shots_after_a_shot(&mut n, &r, 0, 3), 11);
    // Without the flag the hero's machines stand on the field.
    let plain = BattleSetup {
        creature_bank: false,
        ..setup
    };
    assert_eq!(BattleState::new(&r, &plain, 1).unwrap().stacks().len(), 7);
}

// covers: BAT-SET-001
#[test]
fn the_state_hash_covers_the_bank_flag_and_supply() {
    let r = ruleset();
    let s = BattleState::new(&r, &bank(&r, &[(0, "fixed", 1)], &[(0, "fixed", 1)]), 1).unwrap();
    let mut flag = s.clone();
    flag.creature_bank = false;
    let mut supply = s.clone();
    supply.bank_supply[1] = true;
    for other in [flag, supply] {
        assert_ne!(other.state_hash(), s.state_hash());
    }
}
