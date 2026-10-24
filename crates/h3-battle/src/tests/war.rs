//! Tests of war machines (BAT-WAR-001..007).

use std::vec;
use std::vec::Vec;

use h3_content_battle::BattleRuleset;
use h3_contracts::{BallistaControl, BattleHero, BattleResult, BattleSide, TentControl};

use super::support::*;
use crate::event::StrikeFlags;
use crate::{
    BattleSetup, BattleState, Command, Event, EventKind, SetupError, StackFlags, StackId,
    StackTraits,
};

fn creature(r: &BattleRuleset, name: &str) -> h3_content::CreatureTypeId {
    r.creature_by_key(&std::format!("t:{name}")).unwrap()
}

fn with_machines(
    r: &BattleRuleset,
    attacker: &[Spec],
    defender: &[Spec],
    machines: [&[&str]; 2],
) -> BattleSetup {
    lead_machines(
        r,
        BattleSetup {
            war_machines: machines.map(|m| m.iter().map(|k| creature(r, k)).collect()),
            ..setup(r, attacker, defender)
        },
    )
}

fn start(r: &BattleRuleset, s: &BattleSetup, seed: u64) -> BattleState {
    BattleState::new(r, s, seed).unwrap()
}

fn shots_by(events: &[Event], attacker: u8) -> Vec<(u8, StrikeFlags)> {
    events
        .iter()
        .filter_map(|e| match e.kind {
            EventKind::Struck {
                attacker: a,
                target,
                flags,
                ..
            } if a.0 == attacker => Some((target.0, flags)),
            _ => None,
        })
        .collect()
}

fn healed(events: &[Event]) -> Vec<(u8, u8, u32)> {
    events
        .iter()
        .filter_map(|e| match e.kind {
            EventKind::Healed {
                healer,
                target,
                hit_points,
            } => Some((healer.0, target.0, hit_points)),
            _ => None,
        })
        .collect()
}

/// Lets every active stack defend until a round has passed; returns all events.
fn defend_round(s: &mut BattleState, r: &BattleRuleset) -> Vec<Event> {
    let round = s.round();
    let mut events = Vec::new();
    while s.round() == round && !s.is_over() {
        events.extend(act(s, r, Command::Defend).unwrap());
    }
    events
}

/// Gives stack `id` the timed effect of `name` (Hypnotize: the other side controls it,
/// BAT-MAG-026; Berserk, BAT-MAG-025) for five rounds.
fn give(s: &mut BattleState, r: &BattleRuleset, id: u8, name: &str) {
    let spell = r.spell_by_key(&std::format!("t:{name}")).unwrap();
    let cast = crate::effects::TimedCast {
        id: spell,
        spell: r.spell(spell).unwrap(),
        mastery: 1,
        rounds: 5,
        own: crate::magic::SpellBonus::default(),
    };
    crate::effects::apply(s, r, cast, StackId(id), &mut Vec::<Event>::new());
}

// ---------------------------------------------------------------- BAT-WAR-001 placement, traits

// covers: BAT-WAR-001
#[test]
fn war_machines_stand_at_their_places_after_the_army() {
    let r = ruleset();
    let s = start(
        &r,
        &with_machines(
            &r,
            &[(0, "serf", 1)],
            &[(0, "serf", 1)],
            [&["tent", "ballista", "cart"], &["tent", "ballista"]],
        ),
        1,
    );
    let placed: Vec<(&str, u8, (i32, i32))> = s
        .stacks()
        .iter()
        .map(|st| {
            let name = r.creature_key(st.creature).unwrap().name();
            (name, st.slot, field(&r).coords(st.hex))
        })
        .collect();
    assert_eq!(
        placed,
        [
            ("serf", 0, (1, 5)),
            ("ballista", 16, (0, 3)),
            ("tent", 18, (0, 9)),
            ("cart", 17, (1, 1)),
            ("serf", 0, (15, 5)),
            ("ballista", 16, (16, 3)),
            ("tent", 18, (16, 9)),
        ]
    );
    // The ids follow the original's record order (D-23): the Ballista, the Tent, the Ammo Cart.
    // Two-hex machines reach into the side columns with their positions, their front hexes on
    // the ruleset's hexes.
    assert_eq!(s.stacks()[1].footprint(field(&r)).front, at(&r, 1, 3));
    assert_eq!(s.stacks()[6].footprint(field(&r)).front, at(&r, 15, 9));
    // War machines are not affected by morale and not part of the army's alignment.
    assert!(
        s.stacks()[1].has(StackTraits::NO_MORALE) && s.stacks()[1].has(StackTraits::WAR_MACHINE)
    );
    assert_eq!((s.stacks()[0].morale, s.stacks()[1].morale), (1, 0));
}

// covers: BAT-WAR-001
#[test]
fn war_machine_setups_are_validated() {
    let r = ruleset();
    let error = |att: &[Spec], machines: [&[&str]; 2], obstacles: &[(i32, i32)]| {
        let mut s = with_machines(&r, att, &[(0, "serf", 1)], machines);
        s.obstacles = blocked(obstacles.iter().map(|&(x, y)| at(&r, x, y).0).collect());
        BattleState::new(&r, &s, 1).unwrap_err()
    };
    let invalid = SetupError::InvalidWarMachine {
        side: BattleSide::Attacker,
    };
    let serf: &[Spec] = &[(0, "serf", 1)];
    assert_eq!(error(serf, [&["serf"], &[]], &[]), invalid);
    assert_eq!(error(serf, [&["gadget"], &[]], &[]), invalid);
    assert_eq!(error(serf, [&["cart", "cart"], &[]], &[]), invalid);
    assert_eq!(
        error(serf, [&[], &["ballista"]], &[(15, 3)]),
        SetupError::NoRoom {
            side: BattleSide::Defender,
            slot: 16
        }
    );
    assert_eq!(
        error(&[(0, "ballista", 1)], [&[], &[]], &[]),
        SetupError::WarMachineInArmy {
            side: BattleSide::Attacker,
            slot: 0
        }
    );
    let mut bad = with_machines(&r, serf, serf, [&[], &[]]);
    bad.war_machines[0].push(h3_content::CreatureTypeId::from_raw(999));
    assert_eq!(BattleState::new(&r, &bad, 1).unwrap_err(), invalid);
    // The compiler keeps the machines' places apart (BAT-WAR-001), but an obstacle of the setup
    // can still stand on one: the Ballista's front hex (1, 3), the ruleset's hex, is hex 52.
    let mut s = with_machines(&r, serf, serf, [&["ballista"], &[]]);
    s.obstacles = blocked(vec![52]);
    assert_eq!(
        BattleState::new(&r, &s, 1).unwrap_err(),
        SetupError::NoRoom {
            side: BattleSide::Attacker,
            slot: 16
        }
    );
}

// covers: BAT-WAR-001
#[test]
fn a_side_without_a_hero_brings_no_war_machines() {
    let r = ruleset();
    let serf: &[Spec] = &[(0, "serf", 1)];
    let mut s = setup(&r, serf, serf);
    s.war_machines = [vec![creature(&r, "ballista")], vec![creature(&r, "tent")]];
    s.heroes[1] = Some(bare_hero(&r));
    // The original's 0x463600 sets up the machines of a side with a hero only (0x46374D).
    let state = start(&r, &s, 1);
    let machines: Vec<(BattleSide, &str)> = state
        .stacks()
        .iter()
        .filter(|st| st.has(StackTraits::WAR_MACHINE))
        .map(|st| (st.side, r.creature_key(st.creature).unwrap().name()))
        .collect();
    assert_eq!(machines, vec![(BattleSide::Defender, "tent")]);
    assert_eq!(state.stacks().len(), 3);
    // The listing is still checked.
    s.war_machines[0] = vec![creature(&r, "serf")];
    assert_eq!(
        BattleState::new(&r, &s, 1).unwrap_err(),
        SetupError::InvalidWarMachine {
            side: BattleSide::Attacker
        }
    );
}

// covers: BAT-WAR-001
#[test]
fn war_machines_do_not_retaliate_and_do_not_count_for_the_end_of_the_battle() {
    let r = ruleset();
    // 0 = fixed (attacker), 1 = serf ×1 (defender), 2 = defender's tent.
    let mut s = start(
        &r,
        &with_machines(&r, &[(0, "fixed", 10)], &[(0, "serf", 1)], [&[], &["tent"]]),
        1,
    );
    place(&mut s, &r, 0, 14, 8);
    let events = act(
        &mut s,
        &r,
        Command::Attack {
            target: StackId(2),
            from: at(&r, 14, 8),
        },
    )
    .unwrap();
    // 20 damage + 50 % (attack 10 against defense 0) on 75 HP, no retaliation.
    assert_eq!(shots_by(&events, 2), vec![]);
    assert_eq!(s.stacks()[2].top_hp, 45);
    // Killing the last creature ends the battle although the tent stands.
    activate(&mut s, 0);
    place(&mut s, &r, 0, 14, 5);
    let events = act(
        &mut s,
        &r,
        Command::Attack {
            target: StackId(1),
            from: at(&r, 14, 5),
        },
    )
    .unwrap();
    assert!(events.iter().any(|e| e.kind
        == EventKind::BattleEnded {
            result: BattleResult::Victory(BattleSide::Attacker)
        }));
    let outcome = s.outcome().unwrap();
    assert!(outcome.survivors[1].stacks.is_empty());
    assert_eq!(outcome.war_machines[1], vec![creature(&r, "tent")]);
    assert!(outcome.war_machines[0].is_empty());
}

// covers: BAT-END-002, BAT-WAR-002
#[test]
fn a_shot_that_kills_the_last_creatures_of_both_sides_is_a_draw() {
    let r = ruleset();
    // 0 = serf (attacker), 1 = its firebomb, 2 = serf (defender), side by side.
    let mut s = start(
        &r,
        &with_machines(
            &r,
            &[(0, "serf", 1)],
            &[(0, "serf", 1)],
            [&["firebomb"], &[]],
        ),
        1,
    );
    place(&mut s, &r, 0, 9, 5);
    place(&mut s, &r, 2, 10, 5);
    let mut events = act(&mut s, &r, Command::Defend).unwrap();
    assert_eq!(s.active(), Some(StackId(2)));
    events.extend(act(&mut s, &r, Command::Defend).unwrap());
    // The firebomb's shot at the defender's serf also hits the attacker's serf: both die and
    // neither side has a living creature left. (Before BAT-END-002 the battle never ended: the
    // firebomb kept getting turns with nothing to shoot.)
    assert_eq!(
        shots_by(&events, 1),
        vec![
            (2, StrikeFlags::RANGED),
            (0, StrikeFlags::RANGED.with(StrikeFlags::SECONDARY))
        ]
    );
    assert_eq!(
        events.last(),
        Some(&Event::public(EventKind::BattleEnded {
            result: BattleResult::Draw
        }))
    );
    assert!(s.is_over());
    assert_eq!(s.result(), Some(BattleResult::Draw));
    assert_eq!(s.winner(), None);
    assert_eq!(s.active(), None);
    // The result is part of the state hash.
    let mut undecided = s.clone();
    undecided.result = None;
    assert_ne!(s.state_hash(), undecided.state_hash());
    let outcome = s.outcome().unwrap();
    assert_eq!(outcome.result, BattleResult::Draw);
    assert!(outcome.survivors.iter().all(|a| a.stacks.is_empty()));
    assert_eq!(
        outcome.war_machines,
        [vec![creature(&r, "firebomb")], vec![]]
    );
    for side in BattleSide::BOTH {
        assert_eq!(
            crate::validate(&s, &r, side, Command::Defend),
            Err(crate::CommandError::BattleOver)
        );
    }
}

// ---------------------------------------------------------------- BAT-WAR-002 automatic shots

// covers: BAT-WAR-002
#[test]
fn the_ballista_shoots_last_in_the_normal_phase_also_when_blocked() {
    let r = ruleset();
    // 0 = serf, 1 = ballista (attacker); 2 = fixed (defender), standing next to the ballista.
    let mut s = start(
        &r,
        &with_machines(
            &r,
            &[(0, "serf", 1)],
            &[(0, "fixed", 10)],
            [&["ballista"], &[]],
        ),
        1,
    );
    place(&mut s, &r, 2, 2, 3);
    // The serf defends, the defender waits: the ballista acts before the wait phase.
    let events = act(&mut s, &r, Command::Defend).unwrap();
    assert!(shots_by(&events, 1).is_empty());
    let events = act(&mut s, &r, Command::Wait).unwrap();
    assert_eq!(shots_by(&events, 1), vec![(2, StrikeFlags::RANGED)]);
    let started: Vec<u8> = events
        .iter()
        .filter_map(|e| match e.kind {
            EventKind::TurnStarted { stack } => Some(stack.0),
            _ => None,
        })
        .collect();
    assert_eq!(started, vec![1, 2]);
    assert_eq!(s.stacks()[1].shots, 23);
    assert!(s.stacks()[1].flags.contains(StackFlags::ACTED));
}

// covers: BAT-WAR-002
#[test]
fn the_ballista_can_win_the_battle_and_stops_without_shots() {
    let r = ruleset();
    let mut s = start(
        &r,
        &with_machines(
            &r,
            &[(0, "serf", 1)],
            &[(0, "slow", 1)],
            [&["ballista"], &[]],
        ),
        1,
    );
    // 1–2 damage per shot over the long distance: the slow creature falls within five rounds.
    let events: Vec<Event> = (0..5).flat_map(|_| defend_round(&mut s, &r)).collect();
    assert!(events.iter().any(|e| e.kind
        == EventKind::BattleEnded {
            result: BattleResult::Victory(BattleSide::Attacker)
        }));
    assert_eq!(s.active(), None);
    let mut s = start(
        &r,
        &with_machines(
            &r,
            &[(0, "serf", 1)],
            &[(0, "tank", 1)],
            [&["ballista"], &[]],
        ),
        1,
    );
    s.stacks.as_mut_slice()[1].shots = 0;
    assert!(shots_by(&defend_round(&mut s, &r), 1).is_empty());
}

// covers: BAT-WAR-002
#[test]
fn under_quirk_q_0004_the_ballista_shoots_on_without_shots() {
    // The original's Ballista keeps shooting after its 24 shots (batch 1a R7); the count stays
    // at 0 here. Other shooters and strict rules are unaffected.
    for (profile, shoots) in [("t:unlimited", true), ("t:faithful", false)] {
        let r = ruleset_with(profile, |_| {});
        let mut s = start(
            &r,
            &with_machines(
                &r,
                &[(0, "archer", 1)],
                &[(0, "tank", 9)],
                [&["ballista"], &[]],
            ),
            1,
        );
        assert_eq!(
            s.stacks()[1].has(StackTraits::UNLIMITED_SHOTS),
            shoots,
            "{profile}"
        );
        assert!(!s.stacks()[0].has(StackTraits::UNLIMITED_SHOTS));
        s.stacks.as_mut_slice()[1].shots = 1;
        for round in 0..3 {
            let fired = shots_by(&defend_round(&mut s, &r), 1).len();
            assert_eq!(
                fired,
                usize::from(round == 0 || shoots),
                "{profile} {round}"
            );
            assert_eq!(s.stacks()[1].shots, 0);
        }
    }
}

// ---------------------------------------------------------------- BAT-WAR-005 hero's attack

/// A hero for the attacker with `attack` and, if `ballista` is set, Artillery.
fn led(r: &BattleRuleset, attack: u8, ballista: Option<(u8, u16)>) -> BattleHero {
    let mut h = hero(r);
    h.attack = attack;
    h.war_machines.ballista = ballista.map(|(shots, chance)| BallistaControl {
        level: 1,
        shots,
        double_damage_chance_bp: chance,
    });
    h
}

// covers: BAT-WAR-005
#[test]
fn a_hero_multiplies_its_war_machines_damage_by_its_attack_plus_one() {
    let r = ruleset();
    let mut setup = with_machines(
        &r,
        &[(0, "fixed", 1)],
        &[(0, "fixed", 1)],
        [&["ballista", "tent"], &["ballista"]],
    );
    setup.heroes[0] = Some(led(&r, 20, None));
    let s = start(&r, &setup, 1);
    let damage = |id: usize| (s.stacks()[id].damage_min, s.stacks()[id].damage_max);
    // Ids: 0 fixed, 1 ballista, 2 tent (attacker); 3 fixed, 4 ballista (defender, no hero).
    assert_eq!(damage(1), (42, 63), "2–3 × 21");
    assert_eq!(damage(2), (0, 0));
    assert_eq!(damage(4), (2, 3));
    // Creatures keep their damage; the hero's attack also counts in the Ballista's attack.
    assert_eq!(damage(0), damage(3));
    assert_eq!(s.stacks()[1].attack, 30);
    // An attack of 255 saturates nothing: 3 × 256 fits.
    setup.heroes[0] = Some(led(&r, 255, None));
    let s = start(&r, &setup, 1);
    assert_eq!(
        (s.stacks()[1].damage_min, s.stacks()[1].damage_max),
        (512, 768)
    );
}

// ---------------------------------------------------------------- BAT-WAR-006 Artillery

/// A battle where the attacker's hero commands its Ballista (`shots`, `chance`): 0 = serf,
/// 1 = ballista (attacker); 2 = `enemy`, 3 = the defender's ballista.
fn commanded(enemy: Spec, shots: u8, chance: u16, seed: u64) -> (BattleRuleset, BattleState) {
    let r = ruleset();
    let mut setup = with_machines(
        &r,
        &[(0, "serf", 1)],
        &[enemy],
        [&["ballista"], &["ballista"]],
    );
    setup.heroes[0] = Some(led(&r, 0, Some((shots, chance))));
    let s = start(&r, &setup, seed);
    (r, s)
}

/// Lets every stack defend until the attacker's Ballista (stack 1) is active.
fn to_ballista(s: &mut BattleState, r: &BattleRuleset) {
    while s.active() != Some(StackId(1)) {
        act(s, r, Command::Defend).unwrap();
    }
}

fn shoot_at(s: &mut BattleState, r: &BattleRuleset, target: u8) -> Vec<Event> {
    let at = s.stacks()[usize::from(target)].footprint(field(r)).front;
    act(
        s,
        r,
        Command::Shoot {
            target: StackId(target),
            at,
        },
    )
    .unwrap()
}

// covers: BAT-WAR-006
#[test]
fn a_commanded_ballista_waits_for_its_sides_command() {
    let (r, mut s) = commanded((0, "tank", 5), 1, 0, 1);
    to_ballista(&mut s, &r);
    assert_eq!(s.side_to_act(), Some(BattleSide::Attacker));
    let mut legal = crate::LegalCommands::new();
    crate::legal_commands(&s, &r, BattleSide::Attacker, &mut legal);
    let at = |id: usize| s.stacks()[id].footprint(field(&r)).front;
    // Wait, Defend and a shot at every enemy, the enemy's war machine included; no moves; the
    // hero's side may retreat (BAT-FLE-001).
    assert_eq!(
        legal.as_slice(),
        [
            Command::Wait,
            Command::Defend,
            Command::Shoot {
                target: StackId(2),
                at: at(2)
            },
            Command::Shoot {
                target: StackId(3),
                at: at(3)
            },
            Command::Retreat,
        ]
    );
    // It may wait (batch 1a R6) and comes back in the wait phase; the defender's Ballista,
    // without a hero, has shot on its own meanwhile.
    let events = act(&mut s, &r, Command::Wait).unwrap();
    assert!(!shots_by(&events, 3).is_empty());
    to_ballista(&mut s, &r);
    assert!(s.stacks()[1].flags.contains(StackFlags::WAITED));
    let events = shoot_at(&mut s, &r, 3);
    assert_eq!(shots_by(&events, 1), vec![(3, StrikeFlags::RANGED)]);
    assert_eq!(s.stacks()[1].shots, 23);
}

// covers: BAT-WAR-006
#[test]
fn a_commanded_ballista_shoots_its_shots_at_one_target() {
    // Two shots at the same target (Expert Artillery, batch 1a R7) …
    let (r, mut s) = commanded((0, "tank", 5), 2, 0, 1);
    to_ballista(&mut s, &r);
    let events = shoot_at(&mut s, &r, 2);
    assert_eq!(shots_by(&events, 1), vec![(2, StrikeFlags::RANGED); 2]);
    assert_eq!(s.stacks()[1].shots, 22);
    // … but not at a dead one: a serf with 1 hit point falls to the first.
    let (r, mut s) = commanded((0, "serf", 1), 2, 0, 1);
    s.stacks.as_mut_slice()[2].top_hp = 1;
    to_ballista(&mut s, &r);
    let events = shoot_at(&mut s, &r, 2);
    assert_eq!(shots_by(&events, 1).len(), 1);
    assert!(s.is_over());
    // Nor without shots left.
    let (r, mut s) = commanded((0, "tank", 5), 2, 0, 1);
    s.stacks.as_mut_slice()[1].shots = 1;
    to_ballista(&mut s, &r);
    assert_eq!(shots_by(&shoot_at(&mut s, &r, 2), 1).len(), 1);
}

// covers: BAT-WAR-006
#[test]
fn artillery_doubles_the_ballistas_damage_by_chance() {
    let damage = |events: &[Event]| -> Vec<(u64, StrikeFlags)> {
        events
            .iter()
            .filter_map(|e| match e.kind {
                EventKind::Struck {
                    attacker: StackId(1),
                    damage,
                    flags,
                    ..
                } => Some((damage, flags)),
                _ => None,
            })
            .collect()
    };
    let doubled = StrikeFlags::RANGED.with(StrikeFlags::DOUBLE_DAMAGE);
    let mut seen = [false; 2];
    for seed in 0..40 {
        // Always (Expert) doubles; never without the chance; 50 % sometimes.
        for (chance, expect) in [(10_000, Some(true)), (0, Some(false)), (5_000, None)] {
            let (r, mut s) = commanded((0, "fixed", 9), 1, chance, seed);
            // Directly, so that nobody defends.
            activate(&mut s, 1);
            let hit = damage(&shoot_at(&mut s, &r, 2));
            let is_double = hit[0].1 == doubled;
            assert!(is_double || hit[0].1 == StrikeFlags::RANGED);
            // Attack 10 against defense 10, base 2–3, halved over the distance: +100 % makes
            // it 2–3, otherwise 1.
            let (min, max) = if is_double { (2, 3) } else { (1, 1) };
            assert!((min..=max).contains(&hit[0].0), "{chance} {hit:?}");
            match expect {
                Some(e) => assert_eq!(is_double, e),
                None => seen[usize::from(is_double)] = true,
            }
        }
    }
    assert_eq!(seen, [true, true]);
}

// covers: BAT-WAR-006
#[test]
fn a_commanded_ballista_that_cannot_shoot_passes_its_turn() {
    let (r, mut s) = commanded((0, "tank", 5), 1, 0, 1);
    s.stacks.as_mut_slice()[1].shots = 0;
    let events = defend_round(&mut s, &r);
    assert!(
        events
            .iter()
            .any(|e| e.kind == EventKind::TurnStarted { stack: StackId(1) })
    );
    assert!(shots_by(&events, 1).is_empty());
    assert!(s.stacks()[1].flags.contains(StackFlags::ACTED) || s.round() == 2);
}

// ---------------------------------------------------------------- BAT-WAR-003 first aid

// covers: BAT-WAR-003
#[test]
fn the_tent_heals_a_hurt_allied_stack_by_one_to_twenty_five() {
    let r = ruleset();
    let mut amounts = std::collections::BTreeSet::new();
    for seed in 0..300u64 {
        // 0 = tank (attacker, 100 HP each), 1 = attacker's tent, 2 = attacker's ballista.
        let mut s = start(
            &r,
            &with_machines(
                &r,
                &[(0, "tank", 3)],
                &[(0, "tank", 1)],
                [&["tent", "ballista"], &[]],
            ),
            seed,
        );
        s.stacks.as_mut_slice()[0].top_hp = 50;
        s.stacks.as_mut_slice()[2].top_hp = 10;
        let events = defend_round(&mut s, &r);
        let heals = healed(&events);
        assert_eq!(heals.len(), 1);
        let (healer, target, hp) = heals[0];
        // Never the hurt ballista; the top creature only.
        assert_eq!((healer, target), (2, 0));
        assert!((1..=25).contains(&hp));
        assert_eq!(s.stacks()[0].top_hp, 50 + hp);
        assert_eq!(s.stacks()[0].count, 3);
        amounts.insert(hp);
    }
    assert_eq!(amounts.len(), 25, "every amount occurs");
}

/// BAT-Q51's row (batch 4c TP): the original's automatic tent heals the stack whose top creature
/// lost the most (0x473EA0), not a uniformly drawn one; at a tie the later stack.
// covers: BAT-WAR-003
#[test]
fn the_tent_heals_the_stack_that_lost_the_most_the_later_at_a_tie() {
    let r = ruleset();
    // 0 and 1 = tanks (attacker, 100 HP each), 2 = the attacker's tent.
    for (hp, patient) in [([60, 30], 1), ([30, 60], 0), ([40, 40], 1)] {
        for seed in 0..20u64 {
            let mut s = start(
                &r,
                &with_machines(
                    &r,
                    &[(0, "tank", 3), (1, "tank", 3)],
                    &[(0, "tank", 1)],
                    [&["tent"], &[]],
                ),
                seed,
            );
            s.stacks.as_mut_slice()[0].top_hp = hp[0];
            s.stacks.as_mut_slice()[1].top_hp = hp[1];
            let heals = healed(&defend_round(&mut s, &r));
            assert_eq!(
                heals.iter().map(|h| (h.0, h.1)).collect::<Vec<_>>(),
                vec![(2, patient)],
                "{hp:?}, seed {seed}"
            );
        }
    }
}

// covers: BAT-WAR-003
#[test]
fn the_tent_heals_at_most_the_missing_hit_points_and_waits_without_patients() {
    let r = ruleset();
    let mut s = start(
        &r,
        &with_machines(&r, &[(0, "tank", 1)], &[(0, "tank", 1)], [&["tent"], &[]]),
        3,
    );
    s.stacks.as_mut_slice()[0].top_hp = 99;
    let events = defend_round(&mut s, &r);
    assert_eq!(healed(&events), vec![(1, 0, 1)]);
    assert!(healed(&defend_round(&mut s, &r)).is_empty());
}

/// E04b T11b review (F-r3-control-battlefield-1, scenario e): the First Aid Tent heals the
/// stacks of its own side, a hypnotized one too, and no stack of the other side that its side
/// controls (the reviewer's reading: the original's automatic tent walks its side's stacks
/// without a Hypnotize test, and its cursor, 0x476490, wants the patient's own side to be the
/// acting side).
// covers: BAT-WAR-003, BAT-WAR-007, BAT-MAG-026
#[test]
fn the_tent_heals_its_own_sides_stacks_whoever_controls_them() {
    use crate::CommandError::NotAPatient;
    let r = ruleset();
    // Ids: 0 tank, 1 tent (attacker); 2 tank (hurt and hypnotized), 3 tent (defender).
    let mut setup = with_machines(
        &r,
        &[(0, "tank", 3)],
        &[(0, "tank", 3)],
        [&["tent"], &["tent"]],
    );
    let hurt = |setup: &BattleSetup| {
        let mut s = start(&r, setup, 1);
        s.stacks.as_mut_slice()[2].top_hp = 50;
        give(&mut s, &r, 2, "hypnotize");
        assert_eq!(s.stacks()[2].controller(), BattleSide::Attacker);
        s
    };
    // The automatic tents: the defender's heals it, the attacker's has no patient.
    let mut s = hurt(&setup);
    let heals = healed(&defend_round(&mut s, &r));
    assert_eq!(heals.len(), 1);
    assert_eq!((heals[0].0, heals[0].1), (3, 2));
    // Commanded tents: the defender's is offered the heal, the attacker's is refused it.
    let mut nurse = hero(&r);
    nurse.war_machines.tent = Some(TentControl {
        heal_bonus: Rate::bp(0),
    });
    setup.heroes = [Some(nurse.clone()), Some(nurse)];
    let mut s = hurt(&setup);
    activate(&mut s, 1);
    assert_eq!(
        act(&mut s, &r, Command::Heal { target: StackId(2) }),
        Err(NotAPatient)
    );
    activate(&mut s, 3);
    let mut legal = crate::LegalCommands::new();
    crate::legal_commands(&s, &r, BattleSide::Defender, &mut legal);
    assert!(
        legal
            .as_slice()
            .contains(&Command::Heal { target: StackId(2) })
    );
    let events = act(&mut s, &r, Command::Heal { target: StackId(2) }).unwrap();
    assert_eq!(healed(&events).len(), 1);
}

// ---------------------------------------------------------------- BAT-WAR-007 First Aid

/// A battle where the attacker's hero commands its tent with `bonus`: 0 = tank ×3 (top creature
/// at 50 of 100 hit points), 1 = serf, 2 = ballista (hurt), 3 = tent (attacker); 4 = `enemy`.
fn nursed(bonus: u32, enemy: Spec, seed: u64) -> (BattleRuleset, BattleState) {
    nursed_in("t:faithful", Rate::bp(bonus), enemy, seed)
}

/// [`nursed`] in profile `profile` with the heal bonus `bonus`.
fn nursed_in(profile: &str, bonus: Rate, enemy: Spec, seed: u64) -> (BattleRuleset, BattleState) {
    let r = ruleset_with(profile, |_| {});
    let mut setup = with_machines(
        &r,
        &[(0, "tank", 3), (1, "serf", 2)],
        &[enemy],
        [&["tent", "ballista"], &[]],
    );
    let mut h = hero(&r);
    h.war_machines.tent = Some(TentControl { heal_bonus: bonus });
    setup.heroes[0] = Some(h);
    let mut s = start(&r, &setup, seed);
    s.stacks.as_mut_slice()[0].top_hp = 50;
    s.stacks.as_mut_slice()[2].top_hp = 10;
    (r, s)
}

/// Lets every stack defend until stack `id` is active; returns the events.
fn defend_until(s: &mut BattleState, r: &BattleRuleset, id: u8) -> Vec<Event> {
    let mut events = Vec::new();
    while s.active() != Some(StackId(id)) {
        events.extend(act(s, r, Command::Defend).unwrap());
    }
    events
}

// covers: BAT-WAR-007
#[test]
fn a_commanded_tent_waits_for_a_heal_command_while_an_ally_is_hurt() {
    let (r, mut s) = nursed(10_000, (0, "fixed", 1), 1);
    // The defender's fixed stands next to the tent: still no melee for a war machine.
    place(&mut s, &r, 4, 2, 9);
    let events = defend_until(&mut s, &r, 3);
    assert!(healed(&events).is_empty());
    let mut legal = crate::LegalCommands::new();
    crate::legal_commands(&s, &r, BattleSide::Attacker, &mut legal);
    // Wait, Defend and a heal of the hurt tanks only: not the full serfs, not the hurt
    // ballista (a war machine), not the enemy; and the hero's side may retreat.
    assert_eq!(
        legal.as_slice(),
        [
            Command::Wait,
            Command::Defend,
            Command::Heal { target: StackId(0) },
            Command::Retreat,
        ]
    );
    let attack = Command::Attack {
        target: StackId(4),
        from: s.stacks()[3].hex,
    };
    assert_eq!(act(&mut s, &r, attack), Err(crate::CommandError::NoMelee));
    // It may wait and heal in the wait phase.
    act(&mut s, &r, Command::Wait).unwrap();
    defend_until(&mut s, &r, 3);
    let events = act(&mut s, &r, Command::Heal { target: StackId(0) }).unwrap();
    let heals = healed(&events);
    assert_eq!(heals.len(), 1);
    assert_eq!((heals[0].0, heals[0].1), (3, 0));
    assert_eq!(s.stacks()[0].top_hp, 50 + heals[0].2);
    assert_ne!(s.active(), Some(StackId(3)));
}

// covers: BAT-WAR-007
#[test]
fn a_commanded_tent_without_patients_passes_its_turn() {
    // Batch 1a R6: the tent got no turn while nobody was hurt.
    let (r, mut s) = nursed(10_000, (0, "fixed", 1), 1);
    s.stacks.as_mut_slice()[0].top_hp = 100;
    let events = defend_round(&mut s, &r);
    assert!(
        events
            .iter()
            .any(|e| e.kind == EventKind::TurnStarted { stack: StackId(3) })
    );
    assert!(healed(&events).is_empty());
    assert_ne!(s.active(), Some(StackId(3)));
}

// covers: BAT-WAR-007
#[test]
fn first_aid_grows_the_tents_largest_heal() {
    // Basic First Aid: 1–25 becomes 1–50 (the executable's 25 × (1 + 1)), Advanced 1–75; a
    // bonus of 1 bp of 25 stays 25.
    for (bonus, top) in [(10_000, 50), (20_000, 75), (1, 25)] {
        let mut amounts = std::collections::BTreeSet::new();
        for seed in 0..400 {
            let (r, mut s) = nursed(bonus, (0, "fixed", 1), seed);
            s.stacks.as_mut_slice()[0].top_hp = 1;
            activate(&mut s, 3);
            let events = act(&mut s, &r, Command::Heal { target: StackId(0) }).unwrap();
            amounts.insert(healed(&events)[0].2);
        }
        assert_eq!(amounts.first(), Some(&1));
        assert_eq!(amounts.last(), Some(&top), "{bonus}");
    }
}

// covers: BAT-WAR-007
#[test]
fn a_grown_heal_bonus_is_single_precision_with_quirk_q0005() {
    // A First Aid specialist at level 4 with Expert First Aid: 300 % grown by 20 % is 360 %;
    // the original keeps f32(3.6) = 3.5999999…, so its tent heals at most ⌊25 × 4.5999999…⌋ =
    // 114 (OPEN-QUESTIONS Q45); exact, 115.
    let rion = Rate {
        bp: 36_000,
        single: core::num::NonZeroU32::new(0x4066_6666),
    };
    for (profile, bonus, top) in [
        ("t:faithful", rion, 115),
        ("t:single", rion, 114),
        ("t:single", Rate::bp(36_000), 114),
    ] {
        let mut amounts = std::collections::BTreeSet::new();
        for seed in 0..600 {
            let (r, mut s) = nursed_in(profile, bonus, (0, "fixed", 1), seed);
            // A patient that can take the largest heal.
            let patient = &mut s.stacks.as_mut_slice()[0];
            (patient.max_hp, patient.top_hp) = (200, 1);
            activate(&mut s, 3);
            let events = act(&mut s, &r, Command::Heal { target: StackId(0) }).unwrap();
            amounts.insert(healed(&events)[0].2);
        }
        assert_eq!(amounts.last(), Some(&top), "{profile}");
    }
}

// covers: BAT-WAR-007
#[test]
fn heal_commands_are_validated() {
    use crate::CommandError::{CannotHeal, NoSuchTarget, NotAPatient};
    let (r, mut s) = nursed(10_000, (0, "fixed", 1), 1);
    let heal = |target| Command::Heal {
        target: StackId(target),
    };
    // Only a commanded tent heals.
    activate(&mut s, 0);
    assert_eq!(act(&mut s, &r, heal(0)), Err(CannotHeal));
    activate(&mut s, 3);
    for (target, error) in [
        (1, NotAPatient),  // full health
        (2, NotAPatient),  // a war machine
        (4, NotAPatient),  // an enemy
        (9, NoSuchTarget), // no such stack
    ] {
        assert_eq!(act(&mut s, &r, heal(target)), Err(error), "{target}");
    }
    s.stacks.as_mut_slice()[1].count = 0;
    s.stacks.as_mut_slice()[1].top_hp = 0;
    assert_eq!(act(&mut s, &r, heal(1)), Err(NoSuchTarget));
    // Without control the tent stays automatic and never waits for a command.
    let (r, mut s) = nursed(10_000, (0, "fixed", 1), 1);
    s.heroes[0] = None;
    let events = defend_round(&mut s, &r);
    assert_eq!(healed(&events).len(), 1);
}

// ---------------------------------------------------------------- BAT-WAR-004 ammo cart

/// E04b T11b review (F-r3-control-battlefield-1, scenario e): a shooter is supplied by the Ammo
/// Cart of its own side, also while the other side controls it (the original's shot asks the
/// hero of the shooter's own side for the cart, the reviewer's reading at 0x43F9E1; VCMI's
/// `battleMatchOwner` in `CStack` agrees), and not by the cart of the side that controls it.
// covers: BAT-WAR-004, BAT-MAG-026
#[test]
fn a_hypnotized_shooter_is_supplied_by_its_own_sides_ammo_cart() {
    let r = ruleset();
    // Ids: 0 archer, 1 tank, 2 cart (attacker); 3 archer, 4 tank (defender, no cart).
    let mut s = start(
        &r,
        &with_machines(
            &r,
            &[(0, "archer", 5), (1, "tank", 5)],
            &[(0, "archer", 5), (1, "tank", 5)],
            [&["cart"], &[]],
        ),
        1,
    );
    give(&mut s, &r, 0, "hypnotize");
    give(&mut s, &r, 3, "hypnotize");
    let mut shoot = |shooter: u8, target: u8| {
        activate(&mut s, shooter);
        let at = s.stacks()[usize::from(target)].hex;
        let shot = Command::Shoot {
            target: StackId(target),
            at,
        };
        act(&mut s, &r, shot).unwrap();
        s.stacks()[usize::from(shooter)].shots
    };
    // The attacker's archers, commanded by the defender, shoot their own side's tanks with
    // their side's cart behind them; the defender's, commanded by the attacker, have none.
    assert_eq!(shoot(0, 1), 12);
    assert_eq!(shoot(3, 4), 11);
}

// covers: BAT-WAR-004
#[test]
fn the_ammo_cart_keeps_shots_full_until_it_is_destroyed_and_never_acts() {
    let r = ruleset();
    // 0 = archer, 1 = cart (attacker); 2 = brute (defender).
    let mut s = start(
        &r,
        &with_machines(&r, &[(0, "archer", 5)], &[(0, "tank", 5)], [&["cart"], &[]]),
        1,
    );
    let shoot = |s: &mut BattleState| {
        activate(s, 0);
        let at = s.stacks()[2].hex;
        act(
            s,
            &r,
            Command::Shoot {
                target: StackId(2),
                at,
            },
        )
        .unwrap()
    };
    let events = shoot(&mut s);
    assert_eq!(s.stacks()[0].shots, 12);
    assert!(
        !events
            .iter()
            .any(|e| e.kind == EventKind::TurnStarted { stack: StackId(1) })
    );
    let events = defend_round(&mut s, &r);
    assert!(
        !events
            .iter()
            .any(|e| e.kind == EventKind::TurnStarted { stack: StackId(1) })
    );
    // Without the cart, shots are used up.
    s.stacks.as_mut_slice()[1].count = 0;
    s.stacks.as_mut_slice()[1].top_hp = 0;
    shoot(&mut s);
    assert_eq!(s.stacks()[0].shots, 11);
}
