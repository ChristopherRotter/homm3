//! The damage preview of an attack's first strike (BAT-DMG-011; AI-Q6, ADR-0025 point 5). The test
//! battles have no heroes (no luck) and no creature with a death blow, so a real strike always
//! lands within its preview.

use std::vec::Vec;

use h3_kernel::rng::Rng;

use super::support::*;
use crate::{
    Command, DamageRoll, Event, EventKind, LegalCommands, StackId, legal_commands, preview,
};

/// The damage and kills of the first strike `events` report on `target`.
fn first_strike(events: &[Event], target: StackId) -> Option<(u64, u32)> {
    events.iter().find_map(|e| match e.kind {
        EventKind::Struck {
            target: t,
            damage,
            kills,
            ..
        } if t == target => Some((damage, kills)),
        _ => None,
    })
}

fn attack_target(command: Command) -> Option<StackId> {
    match command {
        Command::Attack { target, .. } | Command::Shoot { target, .. } => Some(target),
        _ => None,
    }
}

#[test]
fn a_strike_lands_within_its_preview() {
    // covers: BAT-DMG-011
    let r = ruleset();
    let mut checked = [0usize; 2];
    for seed in 0..24 {
        let mut s = battle(
            &r,
            &[(0, "brute", 23), (1, "archer", 12), (2, "knight", 5)],
            &[(0, "spear", 40), (1, "wide", 7), (2, "range", 9)],
            seed,
        );
        let mut rng = Rng::from_seed(seed);
        for _ in 0..60 {
            let Some(side) = s.side_to_act() else {
                break;
            };
            let mut legal = LegalCommands::new();
            legal_commands(&s, &r, side, &mut legal);
            let attacks: Vec<Command> = legal
                .iter()
                .copied()
                .filter(|c| attack_target(*c).is_some())
                .collect();
            // Every attack and shot that validates has a preview, ordered least to greatest.
            for c in &attacks {
                let p = preview(&s, &r, side, *c).unwrap();
                assert!(
                    p.damage[0] <= p.damage[1] && p.kills[0] <= p.kills[1],
                    "{p:?}"
                );
            }
            let staying: Vec<Command> = legal
                .iter()
                .copied()
                .filter(|c| !c.leaves_battle())
                .collect();
            let pool = if attacks.is_empty() {
                &staying
            } else {
                &attacks
            };
            let command = pool[rng.below(pool.len() as u32) as usize];
            let expected = preview(&s, &r, side, command);
            let events = act(&mut s, &r, command).unwrap();
            if let (Some(p), Some(target)) = (expected, attack_target(command)) {
                let (damage, kills) = first_strike(&events, target).unwrap();
                assert!(
                    (p.damage[0]..=p.damage[1]).contains(&damage)
                        && (p.kills[0]..=p.kills[1]).contains(&kills),
                    "{command:?}: {damage} damage, {kills} kills outside {p:?}"
                );
                checked[usize::from(matches!(command, Command::Shoot { .. }))] += 1;
            }
        }
    }
    // Both kinds of strike were compared, melee and shots.
    assert!(checked.iter().all(|&n| n > 20), "{checked:?}");
}

#[test]
fn only_attacks_and_shots_that_validate_have_a_preview() {
    // covers: BAT-DMG-011
    let r = ruleset();
    let s = battle(&r, &[(0, "brute", 10)], &[(0, "spear", 10)], 1);
    let side = s.side_to_act().unwrap();
    for command in [
        Command::Defend,
        Command::Wait,
        Command::Move { to: at(&r, 3, 5) },
    ] {
        assert_eq!(preview(&s, &r, side, command), None, "{command:?}");
    }
    // The brute stands far from the spear and cannot shoot: neither validates.
    let far = Command::Attack {
        target: StackId(1),
        from: at(&r, 13, 5),
    };
    let shot = Command::Shoot {
        target: StackId(1),
        at: s.stacks()[1].hex,
    };
    assert_eq!(preview(&s, &r, side, far), None);
    assert_eq!(preview(&s, &r, side, shot), None);
    // Nor does the other side's command.
    let mut legal = LegalCommands::new();
    legal_commands(&s, &r, side, &mut legal);
    assert!(
        legal
            .iter()
            .all(|c| preview(&s, &r, side.opponent(), *c).is_none())
    );
}

#[test]
fn a_blessed_stack_previews_one_damage_and_a_charge_counts() {
    // covers: BAT-DMG-011
    let r = ruleset();
    let mut s = battle(&r, &[(0, "archer", 10)], &[(0, "tank", 3)], 1);
    let side = s.side_to_act().unwrap();
    let shot = Command::Shoot {
        target: StackId(1),
        at: s.stacks()[1].hex,
    };
    let rolled = preview(&s, &r, side, shot).unwrap();
    assert!(rolled.damage[0] < rolled.damage[1], "{rolled:?}");
    // Bless fixes the damage per creature at the range's maximum plus its value (BAT-MAG-011).
    s.stacks.as_mut_slice()[0].spells.damage = DamageRoll::Maximum(1);
    let blessed = preview(&s, &r, side, shot).unwrap();
    assert_eq!(blessed.damage[0], blessed.damage[1]);
    assert!(
        blessed.damage[0] > rolled.damage[1],
        "{blessed:?} {rolled:?}"
    );
    // A knight's jousting adds 5 % per hex travelled (BAT-DMG-005): the farther start strikes
    // harder against the same target from the same hex.
    let preview_from = |x: i32| {
        let mut s = battle(&r, &[(0, "knight", 10)], &[(0, "tank", 3)], 1);
        place(&mut s, &r, 0, x, 5);
        place(&mut s, &r, 1, 9, 5);
        let side = s.side_to_act().unwrap();
        let attack = Command::Attack {
            target: StackId(1),
            from: at(&r, 8, 5),
        };
        preview(&s, &r, side, attack).unwrap()
    };
    let (near, far) = (preview_from(7), preview_from(3));
    assert!(far.damage[1] > near.damage[1], "{far:?} {near:?}");
}
