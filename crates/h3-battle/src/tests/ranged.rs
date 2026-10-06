//! Tests of ranged combat (BAT-RNG-001..004).

use std::vec::Vec;

use h3_contracts::BattleSide;

use super::support::*;
use crate::event::StrikeFlags;
use crate::{
    BattleState, Command, CommandError, Event, EventKind, LegalCommands, StackFlags, StackId,
};

/// Fixes the damage of stack `id` to `d` per creature (deterministic tests).
fn fixed_damage(s: &mut BattleState, id: u8, d: u16) {
    let st = &mut s.stacks.as_mut_slice()[usize::from(id)];
    st.damage_min = d;
    st.damage_max = d;
}

/// `(target, damage, flags)` of every strike in `events`.
fn strikes(events: &[Event]) -> Vec<(u8, u64, StrikeFlags)> {
    events
        .iter()
        .filter_map(|e| match e.kind {
            EventKind::Struck {
                target,
                damage,
                flags,
                ..
            } => Some((target.0, damage, flags)),
            _ => None,
        })
        .collect()
}

fn shoot(target: u8, s: &BattleState) -> Command {
    Command::Shoot {
        target: StackId(target),
        at: s.stacks()[usize::from(target)].hex,
    }
}

/// Shoots at `target`'s head with the active stack.
fn fire(
    s: &mut BattleState,
    r: &h3_content::Ruleset,
    target: u8,
) -> Result<Vec<Event>, CommandError> {
    let c = shoot(target, s);
    act(s, r, c)
}

fn legal(s: &BattleState, r: &h3_content::Ruleset) -> Vec<Command> {
    let mut out = LegalCommands::new();
    crate::legal_commands(s, r, s.side_to_act().unwrap(), &mut out);
    out.iter().copied().collect()
}

// covers: BAT-RNG-001
#[test]
fn unblocked_shooters_shoot_and_may_not_melee() {
    let r = ruleset();
    let mut s = battle(&r, &[(0, "archer", 10)], &[(0, "fixed", 10)], 1);
    activate(&mut s, 0);
    place(&mut s, &r, 1, 5, 5);
    // Every living enemy can be shot at; there are no melee attacks.
    let l = legal(&s, &r);
    assert!(l.contains(&shoot(1, &s)));
    assert!(!l.iter().any(|c| matches!(c, Command::Attack { .. })));
    assert_eq!(
        act(
            &mut s.clone(),
            &r,
            Command::Attack {
                target: StackId(1),
                from: at(&r, 4, 5)
            }
        ),
        Err(CommandError::MustShoot)
    );
    let events = fire(&mut s, &r, 1).unwrap();
    // One ranged strike, no retaliation; a shot is used.
    let st = strikes(&events);
    assert_eq!(st.len(), 1);
    assert_eq!((st[0].0, st[0].2), (1, StrikeFlags::RANGED));
    assert_eq!(s.stacks()[0].shots, 11);
    assert!(!s.stacks()[1].flags.contains(StackFlags::RETALIATED));
    assert_eq!(s.stacks()[0].hex, at(&r, 1, 5), "shooting does not move");
}

// covers: BAT-RNG-001
#[test]
fn blocked_or_empty_shooters_fight_in_melee() {
    let r = ruleset();
    let mut s = battle(
        &r,
        &[(0, "archer", 10)],
        &[(0, "fixed", 10), (1, "slow", 1)],
        1,
    );
    activate(&mut s, 0);
    place(&mut s, &r, 1, 2, 5); // adjacent enemy: blocked
    place(&mut s, &r, 2, 9, 5);
    assert_eq!(fire(&mut s.clone(), &r, 2), Err(CommandError::CannotShoot));
    let l = legal(&s, &r);
    assert!(!l.iter().any(|c| matches!(c, Command::Shoot { .. })));
    act(
        &mut s.clone(),
        &r,
        Command::Attack {
            target: StackId(1),
            from: at(&r, 1, 5),
        },
    )
    .unwrap();
    // Out of shots: no shooting, melee anywhere it can reach.
    place(&mut s, &r, 1, 4, 5);
    s.stacks.as_mut_slice()[0].shots = 0;
    assert_eq!(fire(&mut s.clone(), &r, 1), Err(CommandError::CannotShoot));
    assert!(
        legal(&s, &r)
            .iter()
            .any(|c| matches!(c, Command::Attack { .. }))
    );
    // A creature that is not a shooter cannot shoot either.
    let mut melee = battle(&r, &[(0, "fixed", 1)], &[(0, "slow", 1)], 1);
    activate(&mut melee, 0);
    assert_eq!(
        fire(&mut melee.clone(), &r, 1),
        Err(CommandError::CannotShoot)
    );
    // Targets must be living enemies.
    assert_eq!(
        act(
            &mut melee.clone(),
            &r,
            Command::Shoot {
                target: StackId(0),
                at: melee.stacks()[0].hex
            }
        ),
        Err(CommandError::NotAnEnemy)
    );
    assert_eq!(
        act(
            &mut melee,
            &r,
            Command::Shoot {
                target: StackId(9),
                at: at(&r, 5, 5)
            }
        ),
        Err(CommandError::NoSuchTarget)
    );
}

// covers: BAT-RNG-001
#[test]
fn double_shots_use_two_shots_while_the_target_lives() {
    let r = ruleset();
    let mut s = battle(&r, &[(0, "sniper", 10)], &[(0, "tank", 5)], 1);
    activate(&mut s, 0);
    let events = fire(&mut s.clone(), &r, 1).unwrap();
    assert_eq!(strikes(&events).len(), 2);
    // One shot left: a single strike.
    let mut one = s.clone();
    one.stacks.as_mut_slice()[0].shots = 1;
    let events = fire(&mut one, &r, 1).unwrap();
    assert_eq!(strikes(&events).len(), 1);
    assert_eq!(one.stacks()[0].shots, 0);
    // The first shot kills the target: no second shot.
    let mut kill = battle(
        &r,
        &[(0, "sniper", 50)],
        &[(0, "slow", 1), (1, "slow", 1)],
        1,
    );
    activate(&mut kill, 0);
    let events = fire(&mut kill, &r, 1).unwrap();
    assert_eq!(strikes(&events).len(), 1);
    assert_eq!(kill.stacks()[0].shots, 2);
}

// covers: BAT-RNG-002
#[test]
fn shots_beyond_ten_hexes_deal_half_damage() {
    let r = ruleset();
    // archer attack 6 vs fixed defense 10: 100 base · 0.9 = 90 at full damage, 45 at half.
    let mut s = battle(&r, &[(0, "archer", 10)], &[(0, "fixed", 10)], 1);
    activate(&mut s, 0);
    fixed_damage(&mut s, 0, 10);
    let damage = |s: &BattleState, x: i32| {
        let mut t = s.clone();
        place(&mut t, &r, 1, x, 5);
        strikes(&fire(&mut t, &r, 1).unwrap())[0].1
    };
    assert_eq!(damage(&s, 11), 90, "distance 10: full damage");
    assert_eq!(damage(&s, 12), 45, "distance 11: half damage");
    // A shooter without the distance penalty.
    let mut sniper = battle(&r, &[(0, "sniper", 10)], &[(0, "fixed", 10)], 1);
    activate(&mut sniper, 0);
    fixed_damage(&mut sniper, 0, 10);
    sniper.stacks.as_mut_slice()[0].shots = 1;
    assert_eq!(damage(&sniper, 15), 90);
    // A two-hex target counts as near if either hex is within ten hexes.
    let mut wide = battle(&r, &[(0, "archer", 10)], &[(0, "wide", 10)], 1);
    activate(&mut wide, 0);
    fixed_damage(&mut wide, 0, 10);
    // wide (defense 6, two hexes: head (12, 5), tail (13, 5)) is farther than 10; head (10, 5)
    // with tail (11, 5) is near. archer attack 6 = defense 6: 100 base.
    assert_eq!(damage(&wide, 12), 50);
    assert_eq!(damage(&wide, 10), 100);
}

// covers: BAT-RNG-003
#[test]
fn shooters_deal_half_damage_in_melee_unless_exempt() {
    let r = ruleset();
    let mut s = battle(&r, &[(0, "archer", 10)], &[(0, "fixed", 10)], 1);
    activate(&mut s, 0);
    place(&mut s, &r, 1, 2, 5);
    fixed_damage(&mut s, 0, 10);
    fixed_damage(&mut s, 1, 1);
    let events = act(
        &mut s.clone(),
        &r,
        Command::Attack {
            target: StackId(1),
            from: at(&r, 1, 5),
        },
    )
    .unwrap();
    let st = strikes(&events);
    // archer attack 6 vs defense 10: 90 → 45 with the melee penalty.
    assert_eq!(st[0].1, 45);
    // Its retaliation is halved too: fixed (attack 10) against defense 3: 10 base · 1.35 = 13.
    let mut back = s.clone();
    activate(&mut back, 1);
    let st = strikes(
        &act(
            &mut back,
            &r,
            Command::Attack {
                target: StackId(0),
                from: at(&r, 2, 5),
            },
        )
        .unwrap(),
    );
    assert_eq!(st[1].2, StrikeFlags::RETALIATION);
    // The 13 damage killed one archer: 9 × 10 = 90 base, attack 6 vs defense 10 → 81, halved
    // → 40.
    assert_eq!(st[1].1, 40);
    // No melee penalty for the sniper.
    let mut sn = battle(&r, &[(0, "sniper", 10)], &[(0, "fixed", 10)], 1);
    activate(&mut sn, 0);
    place(&mut sn, &r, 1, 2, 5);
    fixed_damage(&mut sn, 0, 10);
    let st = strikes(
        &act(
            &mut sn,
            &r,
            Command::Attack {
                target: StackId(1),
                from: at(&r, 1, 5),
            },
        )
        .unwrap(),
    );
    assert_eq!(st[0].1, 90);
}

// covers: BAT-RNG-004
#[test]
fn fireballs_hit_everyone_around_the_target_except_the_fire_immune() {
    let r = ruleset();
    let mut s = battle(
        &r,
        &[(0, "magog", 10), (1, "fixed", 5)],
        &[
            (0, "fixed", 5),
            (1, "salamander", 5),
            (2, "fixed", 5),
            (3, "salamander", 5),
        ],
        1,
    );
    activate(&mut s, 0);
    place(&mut s, &r, 2, 8, 5); // target
    place(&mut s, &r, 3, 9, 5); // fire-immune neighbour: spared
    place(&mut s, &r, 4, 7, 5); // enemy neighbour: hit
    place(&mut s, &r, 1, 8, 4); // friendly neighbour: hit (friendly fire)
    place(&mut s, &r, 5, 12, 5); // far away
    let events = fire(&mut s.clone(), &r, 2).unwrap();
    let st = strikes(&events);
    let ids: Vec<u8> = st.iter().map(|x| x.0).collect();
    assert_eq!(ids, [2, 1, 4]);
    assert_eq!(st[0].2, StrikeFlags::RANGED);
    assert_eq!(st[1].2, StrikeFlags::RANGED.with(StrikeFlags::SECONDARY));
    assert_eq!(s.stacks()[0].shots, 24);
    // A fire-immune primary target is hit.
    let st = strikes(&fire(&mut s.clone(), &r, 3).unwrap());
    assert_eq!(st[0].0, 3);
}

// covers: BAT-RNG-004
#[test]
fn death_clouds_spare_the_non_living_and_area_shots_may_aim_at_a_tail() {
    let r = ruleset();
    let mut s = battle(
        &r,
        &[(0, "lich", 10)],
        &[(0, "salamander", 5), (1, "fixed", 5), (2, "wide", 5)],
        1,
    );
    activate(&mut s, 0);
    place(&mut s, &r, 1, 8, 5); // target (alive)
    place(&mut s, &r, 2, 9, 5); // `fixed` has no `alive` flag: spared
    place(&mut s, &r, 3, 6, 5); // wide: head (6, 5), tail (7, 5) next to the target: alive? no
    let st = strikes(&fire(&mut s.clone(), &r, 1).unwrap());
    assert_eq!(st.iter().map(|x| x.0).collect::<Vec<_>>(), [1]);
    // Aim at the two-hex target's tail: the area moves with it.
    let tail = s.stacks()[3].footprint(field(&r)).tail.unwrap();
    let aim = Command::Shoot {
        target: StackId(3),
        at: tail,
    };
    assert!(legal(&s, &r).contains(&aim));
    act(&mut s.clone(), &r, aim).unwrap();
    // Other shooters must aim at the head; nobody may aim off the target.
    let mut a = battle(&r, &[(0, "archer", 10)], &[(0, "wide", 5)], 1);
    activate(&mut a, 0);
    let tail = a.stacks()[1].footprint(field(&r)).tail.unwrap();
    assert_eq!(
        act(
            &mut a.clone(),
            &r,
            Command::Shoot {
                target: StackId(1),
                at: tail
            }
        ),
        Err(CommandError::NotOnTarget)
    );
    assert_eq!(
        act(
            &mut s,
            &r,
            Command::Shoot {
                target: StackId(1),
                at: at(&r, 2, 2)
            }
        ),
        Err(CommandError::NotOnTarget)
    );
    // Area shots that kill can end the battle.
    let mut end = battle(&r, &[(0, "lich", 100)], &[(0, "salamander", 1)], 1);
    activate(&mut end, 0);
    fire(&mut end, &r, 1).unwrap();
    assert_eq!(end.winner(), Some(BattleSide::Attacker));
}

#[test]
fn strike_flags_and_factor_limits() {
    let f = StrikeFlags::RANGED.with(StrikeFlags::SECONDARY);
    assert_eq!(f.bits(), 0b110);
    assert!(f.contains(StrikeFlags::RANGED) && !f.contains(StrikeFlags::RETALIATION));
    assert_eq!(StrikeFlags::NONE.bits(), 0);
    // Every reduction at its limit still leaves the minimum of 1.
    let p = ruleset();
    let mut all = crate::damage::Strike::new(0, 40);
    all.penalty_percent = 100;
    all.protection_percent = 0;
    all.hero_reduction_bp = 10_000;
    assert_eq!(all.damage(128, p.battle(), false), 1);
    assert_eq!(all.damage(128, p.battle(), true), 1);
}
