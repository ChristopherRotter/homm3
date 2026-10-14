//! The arrow towers (E04c T7; `docs/formats/installation.md` §7.30): the town's turrets and
//! their stacks (BAT-SIE-016), a destroyed tower's turret (BAT-SIE-017), the turret's shot
//! (BAT-SIE-018) and what never reaches a turret (BAT-SIE-019), on the test ruleset with the
//! turret ([`turret_ruleset`]: the keep on 254 for target 7, the upper tower on 255 for target 0,
//! the lower tower on 251 for target 6; the keep's least count 4).

use std::vec;
use std::vec::Vec;

use h3_content::FULL_BP;
use h3_content_battle::BattleRuleset;
use h3_contracts::{
    BallistaControl, BattleHero, BattleSetup, BattleSide, FortLevel, Moat, Rate, Siege,
};

use super::support::*;
use crate::effects::Status;
use crate::geometry::Hex;
use crate::siege::damage_part;
use crate::state::{NO_CORPSE, TURRET_SLOT};
use crate::{
    BattleState, Command, CommandError, Event, EventKind, LegalCommands, SetupError, StackId,
    StackTraits, legal_commands,
};

/// The town with `fort` and `structures`, no moat.
fn town(r: &BattleRuleset, fort: FortLevel, structures: u8) -> Siege {
    Siege {
        faction: r.faction_by_key("t:castle").unwrap(),
        fort,
        moat: Moat::Absent,
        structures,
        escape_tunnel: false,
        morale: 0,
        luck: 0,
    }
}

/// The setup of a battle at `town` with `heroes`.
fn at(
    r: &BattleRuleset,
    town: Siege,
    heroes: [Option<BattleHero>; 2],
    attacker: &[Spec],
    defender: &[Spec],
) -> BattleSetup {
    BattleSetup {
        heroes,
        siege: Some(town),
        ..setup(r, attacker, defender)
    }
}

/// The battle of `setup`, started with its events.
fn start(r: &BattleRuleset, setup: &BattleSetup, seed: u64) -> (BattleState, Vec<Event>) {
    let mut events = Vec::new();
    let s = BattleState::start(r, setup, seed, &mut events).unwrap();
    (s, events)
}

/// The turrets' stack ids, positions and counts, in stack order.
fn turrets(s: &BattleState) -> Vec<(u8, u8, u32)> {
    (0u8..)
        .zip(s.stacks())
        .filter(|(_, st)| st.is_turret())
        .map(|(i, st)| (i, st.hex.0, st.count))
        .collect()
}

/// The strikes of `events`: attacker, target, damage.
fn strikes(events: &[Event]) -> Vec<(u8, u8, u64)> {
    events
        .iter()
        .filter_map(|e| match e.kind {
            EventKind::Struck {
                attacker,
                target,
                damage,
                ..
            } => Some((attacker.0, target.0, damage)),
            _ => None,
        })
        .collect()
}

/// A hero with Artillery: it commands its Ballista, two shots a turn (BAT-WAR-006), and its
/// turrets.
fn gunner(r: &BattleRuleset) -> BattleHero {
    let mut h = hero(r);
    h.war_machines.ballista = Some(BallistaControl {
        shots: 2,
        double_damage_chance_bp: 0,
    });
    h
}

// ---------------------------------------------------------------- BAT-SIE-016

// covers: BAT-SIE-016
#[test]
fn a_citadel_sets_up_the_keep_and_a_castle_the_towers_too() {
    let r = turret_ruleset("t:faithful");
    let army = |r: &BattleRuleset, fort, structures| {
        at(
            r,
            town(r, fort, structures),
            [None, None],
            &[(0, "fixed", 10)],
            &[(0, "fixed", 10), (1, "slow", 5)],
        )
    };
    // A Fort has none; the keep from a Citadel: 4 + the structures creatures, after the
    // defender's army, at the keep's position, in the first turret slot.
    let (s, _) = start(&r, &army(&r, FortLevel::Fort, 3), 1);
    assert!(turrets(&s).is_empty());
    let (s, _) = start(&r, &army(&r, FortLevel::Citadel, 3), 1);
    assert_eq!(turrets(&s), [(3, 254, 7)]);
    assert_eq!(s.stacks()[3].slot, TURRET_SLOT);
    assert_eq!(s.stacks()[3].side, BattleSide::Defender);
    // A Castle: the keep, then the upper and the lower tower with half its count, rounded up.
    let (s, _) = start(&r, &army(&r, FortLevel::Castle, 3), 1);
    assert_eq!(turrets(&s), [(3, 254, 7), (4, 255, 4), (5, 251, 4)]);
    let (s, _) = start(&r, &army(&r, FortLevel::Castle, 0), 1);
    assert_eq!(turrets(&s), [(3, 254, 4), (4, 255, 2), (5, 251, 2)]);
    // The targets hold them: the keep 7, the upper tower 0, the lower tower 6.
    let siege = s.siege.unwrap();
    let held: Vec<_> = (0..8).map(|t| siege.turret(t).map(|id| id.0)).collect();
    assert_eq!(
        held,
        [Some(4), None, None, None, None, None, Some(5), Some(3)]
    );
    // Without a turret creature in the ruleset, none (the other siege tests).
    let plain = ruleset();
    let (s, _) = start(&plain, &army(&plain, FortLevel::Castle, 3), 1);
    assert!(turrets(&s).is_empty());
    // No battle without a town has them; a setup may not list the turret as a war machine.
    let (s, _) = start(&r, &setup(&r, &[(0, "fixed", 10)], &[(0, "fixed", 10)]), 1);
    assert!(turrets(&s).is_empty());
    let mut listed = army(&r, FortLevel::Castle, 3);
    listed.war_machines[1] = vec![r.creature_by_key("t:arrow_tower").unwrap()];
    assert_eq!(
        BattleState::new(&r, &listed, 1).unwrap_err(),
        SetupError::InvalidWarMachine {
            side: BattleSide::Defender
        }
    );
}

// covers: BAT-SIE-016
#[test]
fn the_turrets_act_first_every_round_on_their_own_without_artillery() {
    let r = turret_ruleset("t:faithful");
    // The keep, the upper and the lower tower shoot before every other stack (the slower
    // defender's too), then the stacks by speed; every round.
    let setup = at(
        &r,
        town(&r, FortLevel::Castle, 0),
        [None, None],
        &[(0, "fixed", 50)],
        &[(0, "fast", 20)],
    );
    let (mut s, events) = start(&r, &setup, 3);
    let shots: Vec<u8> = strikes(&events).iter().map(|&(a, ..)| a).collect();
    assert_eq!(shots, [2, 3, 4]);
    assert!(strikes(&events).iter().all(|&(_, t, _)| t == 0));
    // The defender's fast stack is first of the rest (speed), then the attacker's.
    assert_eq!(s.active(), Some(StackId(1)));
    act(&mut s, &r, Command::Defend).unwrap();
    assert_eq!(s.active(), Some(StackId(0)));
    let events = act(&mut s, &r, Command::Defend).unwrap();
    let shots: Vec<u8> = strikes(&events).iter().map(|&(a, ..)| a).collect();
    assert_eq!(shots, [2, 3, 4]);
    // No morale for a turret (a war machine): it never gets a second turn.
    assert!(s.stacks()[2].has(StackTraits::NO_MORALE));
}

// covers: BAT-SIE-016, BAT-SIE-018
#[test]
fn a_defender_with_artillery_commands_its_turrets() {
    let r = turret_ruleset("t:faithful");
    let setup = at(
        &r,
        town(&r, FortLevel::Citadel, 0),
        [None, Some(gunner(&r))],
        &[(0, "fixed", 20), (1, "archer", 10)],
        &[(0, "fast", 20)],
    );
    let (mut s, events) = start(&r, &setup, 3);
    // The keep waits for its side's command: no shot yet.
    assert!(strikes(&events).is_empty());
    assert_eq!(s.active(), Some(StackId(3)));
    assert_eq!(s.side_to_act(), Some(BattleSide::Defender));
    // Wait, Defend and a shot at each living enemy; no move, no attack, no turret target.
    let mut out = LegalCommands::new();
    legal_commands(&s, &r, BattleSide::Defender, &mut out);
    let fronts = |id: u8| s.stacks()[usize::from(id)].footprint(field(&r)).front;
    let listed: Vec<Command> = out
        .iter()
        .copied()
        .filter(|c| *c != Command::Retreat)
        .collect();
    assert_eq!(
        listed,
        [
            Command::Wait,
            Command::Defend,
            Command::Shoot {
                target: StackId(0),
                at: fronts(0),
            },
            Command::Shoot {
                target: StackId(1),
                at: fronts(1),
            },
        ]
    );
    // One shot, whatever the Ballista's shots; no shot used.
    let aim = fronts(1);
    let events = act(
        &mut s,
        &r,
        Command::Shoot {
            target: StackId(1),
            at: aim,
        },
    )
    .unwrap();
    assert_eq!(strikes(&events).len(), 1);
    assert_eq!(s.stacks()[3].shots, 99);
    // It may wait: it then acts first in the wait phase.
    let (mut s, _) = start(&r, &setup, 3);
    act(&mut s, &r, Command::Wait).unwrap();
    assert_eq!(s.active(), Some(StackId(2)));
}

// ---------------------------------------------------------------- BAT-SIE-017

// covers: BAT-SIE-017
#[test]
fn a_destroyed_tower_or_keep_takes_its_turret_down() {
    let r = turret_ruleset("t:faithful");
    let setup = at(
        &r,
        town(&r, FortLevel::Castle, 0),
        [None, None],
        &[(0, "fixed", 50)],
        &[(0, "fast", 20)],
    );
    let (mut s, _) = start(&r, &setup, 3);
    let mut events = Vec::new();
    // The keep (target 7): its turret dies, leaves no corpse, and never acts again.
    damage_part(&mut s, 7, 2, &mut events);
    assert!(!s.stacks()[2].is_alive());
    assert_eq!(s.corpse(StackId(2)), NO_CORPSE);
    assert!(
        events
            .iter()
            .any(|e| e.kind == EventKind::WallDestroyed { part: 7 })
    );
    // A wall's destruction kills nothing.
    damage_part(&mut s, 1, 2, &mut events);
    assert_eq!(turrets(&s).iter().filter(|t| t.2 > 0).count(), 2);
    // The next round: only the towers shoot.
    act(&mut s, &r, Command::Defend).unwrap();
    let shots: Vec<u8> = strikes(&act(&mut s, &r, Command::Defend).unwrap())
        .iter()
        .map(|&(a, ..)| a)
        .collect();
    assert_eq!(shots, [3, 4]);
    // At a Citadel the absent towers' targets hold no turret: their fall harms no stack
    // (BAT-QUIRK-22).
    let setup = BattleSetup {
        siege: Some(town(&r, FortLevel::Citadel, 0)),
        ..setup
    };
    let (mut s, _) = start(&r, &setup, 3);
    let before: Vec<u32> = s.stacks().iter().map(|st| st.count).collect();
    damage_part(&mut s, 0, 2, &mut events);
    damage_part(&mut s, 6, 2, &mut events);
    let after: Vec<u32> = s.stacks().iter().map(|st| st.count).collect();
    assert_eq!(before, after);
}

// ---------------------------------------------------------------- BAT-SIE-018

/// The keep's shot at the attacker's stack 0 at the battle's start, with the target's `edit`
/// and the heroes `heroes`: the damage, and the sum of the keep's roll (the same draws).
fn keep_shot(
    r: &BattleRuleset,
    heroes: [Option<BattleHero>; 2],
    edit: impl Fn(&mut BattleState),
) -> (u64, u64) {
    let setup = at(
        r,
        town(r, FortLevel::Citadel, 2),
        heroes,
        &[(0, "fixed", 200)],
        &[(0, "fast", 20)],
    );
    // The setup's draws before the keep's turn are the same with or without the edit: build the
    // battle, edit the target, then let the keep act.
    let mut s = BattleState::new(r, &setup, 5).unwrap();
    let keep = turrets(&s)[0];
    // The target as it was before the keep's first shot.
    s.stacks.as_mut_slice()[0].count = 200;
    s.stacks.as_mut_slice()[0].top_hp = s.stacks()[0].max_hp;
    edit(&mut s);
    let mut rng = s.rng.clone();
    let sum: u64 = (0..keep.2)
        .map(|_| u64::from(rng.range_inclusive(2, 4)))
        .sum();
    let mut events = Vec::new();
    crate::combat::shoot(&mut s, r, StackId(keep.0), StackId(0), Hex(0), &mut events);
    (strikes(&events)[0].2, sum)
}

// covers: BAT-SIE-018
#[test]
fn the_turret_s_shot_sums_a_draw_per_creature_and_takes_only_the_protections() {
    let r = turret_ruleset("t:faithful");
    // The keep of 6 (4 + 2 structures): 2 to 4 per creature, nothing else.
    let (damage, sum) = keep_shot(&r, [None, None], |_| {});
    assert_eq!(damage, sum);
    assert!((12..=24).contains(&sum));
    // No attack or defense, Bless or luck, hero's Archery: the target's defense 50, the
    // defender's hero with +200 % shots and attack 30, luck everywhere.
    let mut archer = hero(&r);
    archer.attack = 30;
    archer.damage.ranged = Rate::bp(20_000);
    let (damage, sum) = keep_shot(&r, [None, Some(archer)], |s| {
        let t = &mut s.stacks.as_mut_slice()[0];
        t.defense = 50;
        t.luck = 3;
    });
    assert_eq!(damage, sum);
    // Air Shield's share, the half while petrified, the target's hero's Armorer, each in turn.
    let (damage, sum) = keep_shot(&r, [None, None], |s| {
        s.stacks.as_mut_slice()[0].spells.ranged_protection = 50;
    });
    assert_eq!(damage, sum / 2);
    let (damage, sum) = keep_shot(&r, [None, None], |s| {
        let st = &mut s.stacks.as_mut_slice()[0].spells.status;
        *st = st.with(Status::PETRIFIED, true);
    });
    assert_eq!(damage, sum / 2);
    let mut armorer = hero(&r);
    armorer.damage.reduction = Rate::bp(1500);
    let (damage, sum) = keep_shot(&r, [Some(armorer.clone()), None], |_| {});
    assert_eq!(damage, sum * 85 / 100);
    // All three: 0.5 · 0.5 · 0.85, truncated; at least 1 with the Armorer at 100 %.
    let (damage, sum) = keep_shot(&r, [Some(armorer), None], |s| {
        let t = &mut s.stacks.as_mut_slice()[0];
        t.spells.ranged_protection = 50;
        t.spells.status = t.spells.status.with(Status::PETRIFIED, true);
    });
    assert_eq!(damage, sum * 2125 / 10_000);
    let mut plated = hero(&r);
    plated.damage.reduction = Rate::bp(FULL_BP.into());
    let (damage, _) = keep_shot(&r, [Some(plated), None], |_| {});
    assert_eq!(damage, 1);
    // With the executable's stages (quirk Q-0005) the same truncation.
    let staged = turret_ruleset("t:single");
    let (damage, sum) = keep_shot(&staged, [None, None], |s| {
        s.stacks.as_mut_slice()[0].spells.ranged_protection = 50;
    });
    assert_eq!(damage, sum / 2);
}

// covers: BAT-SIE-018
#[test]
fn the_automatic_turret_shoots_the_ballista_s_target() {
    let r = turret_ruleset("t:faithful");
    // The automatic choice prefers a stack that is no war machine: the attacker's Ballista and
    // a creature; the keep shoots the creature (BAT-WAR-002's rule).
    let mut setup = at(
        &r,
        town(&r, FortLevel::Citadel, 0),
        [None, None],
        &[(0, "fixed", 5)],
        &[(0, "fast", 20)],
    );
    setup.war_machines[0] = vec![r.creature_by_key("t:ballista").unwrap()];
    let (s, events) = start(&r, &setup, 3);
    let keep = turrets(&s)[0].0;
    assert_eq!(
        strikes(&events)
            .iter()
            .map(|&(a, t, _)| (a, t))
            .collect::<Vec<_>>(),
        [(keep, 0)]
    );
}

// ---------------------------------------------------------------- BAT-SIE-019

// covers: BAT-SIE-019
#[test]
fn nothing_reaches_a_turret() {
    let r = turret_ruleset("t:faithful");
    let setup = at(
        &r,
        town(&r, FortLevel::Castle, 0),
        [None, None],
        &[(0, "archer", 30)],
        &[(0, "fast", 20)],
    );
    let (mut s, _) = start(&r, &setup, 3);
    // No shot at a turret (stacks 2 to 4): neither listed nor accepted.
    activate(&mut s, 0);
    let mut out = LegalCommands::new();
    legal_commands(&s, &r, BattleSide::Attacker, &mut out);
    let targets: Vec<u8> = out
        .iter()
        .filter_map(|c| match *c {
            Command::Shoot { target, .. } => Some(target.0),
            _ => None,
        })
        .collect();
    assert_eq!(targets, [1]);
    assert_eq!(
        act(
            &mut s,
            &r,
            Command::Shoot {
                target: StackId(2),
                at: Hex(254),
            },
        )
        .unwrap_err(),
        CommandError::NotAnEnemy
    );
    // Immune to every spell (the effect test).
    for (k, spell) in r.spells().iter().enumerate() {
        let id = h3_content::SpellId::from_index(k).unwrap();
        for side in BattleSide::BOTH {
            for turret in 2..5 {
                assert!(!crate::magic::affects_stack(
                    &s,
                    &r,
                    side,
                    (id, spell),
                    StackId(turret)
                ));
            }
        }
    }
    // A berserk shooter picks the nearest other stack, never a turret (the lower tower is
    // nearer by the hexes' arithmetic than the defender's stack across the field).
    place(&mut s, &r, 0, 1, 10);
    place(&mut s, &r, 1, 15, 0);
    let st = &mut s.stacks.as_mut_slice()[0].spells.status;
    *st = st.with(Status::BERSERK, true);
    activate(&mut s, 0);
    let mut events = Vec::new();
    crate::control::berserk(&mut s, &r, StackId(0), &mut events);
    assert_eq!(
        strikes(&events).first().map(|&(a, t, _)| (a, t)),
        Some((0, 1))
    );
}

// covers: BAT-SIE-016, BAT-SIE-019
#[test]
fn the_turrets_are_left_out_of_the_outcome_and_the_surrender_price() {
    let r = turret_ruleset("t:faithful");
    let setup = at(
        &r,
        town(&r, FortLevel::Castle, 0),
        [Some(hero(&r)), Some(hero(&r))],
        &[(0, "fixed", 10)],
        &[(0, "fast", 1)],
    );
    let (mut s, _) = start(&r, &setup, 3);
    let price = crate::flee::surrender_price(&s, &r, BattleSide::Defender);
    assert!(price.is_some());
    let alone = BattleState::new(
        &r,
        &BattleSetup {
            siege: Some(town(&r, FortLevel::Fort, 0)),
            ..setup.clone()
        },
        3,
    )
    .unwrap();
    assert_eq!(
        price,
        crate::flee::surrender_price(&alone, &r, BattleSide::Defender)
    );
    // The defender wins; its outcome lists its army stack only, its war machines none.
    s.stacks.as_mut_slice()[0].count = 0;
    let mut events = Vec::new();
    crate::combat::check_end(&mut s, &mut events);
    let outcome = s.outcome().unwrap();
    assert_eq!(outcome.stacks[1].len(), 1);
    assert!(outcome.war_machines[1].is_empty());
}
