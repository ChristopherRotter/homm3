//! Tests of the E03 geometry rules: footprints of two-hex stacks (BAT-GEO-002), obstacles
//! (BAT-GEO-003), placement around them (BAT-SET-001), walking and flying movement
//! (BAT-MOV-001, BAT-MOV-002) and melee attack positions (BAT-ATK-001).

use std::vec;
use std::vec::Vec;

use h3_contracts::BattleSide;
use h3_kernel::hash::hash_of;

use super::support::*;
use crate::geometry::{ATTACK_SLOTS, Footprint, HexSet};
use crate::{BattleState, Command, CommandError, Hex, LegalCommands, SetupError, StackId};

fn legal(s: &BattleState, r: &h3_content_battle::BattleRuleset) -> Vec<Command> {
    let mut out = LegalCommands::new();
    legal_commands_for(s, r, &mut out);
    out.iter().copied().collect()
}

fn legal_commands_for(
    s: &BattleState,
    r: &h3_content_battle::BattleRuleset,
    out: &mut LegalCommands,
) {
    crate::legal_commands(s, r, s.side_to_act().unwrap(), out);
}

// ---------------------------------------------------------------- BAT-GEO-002 footprints

// covers: BAT-GEO-002
#[test]
fn two_hex_footprints_extend_in_front_of_the_position() {
    let r = ruleset();
    let f = field(&r);
    let position = at(&r, 5, 5);
    let one = |position| Footprint {
        position,
        other: None,
    };
    assert_eq!(
        f.footprint(position, true, BattleSide::Attacker),
        Some(Footprint {
            position,
            other: Some(at(&r, 6, 5))
        })
    );
    assert_eq!(
        f.footprint(position, true, BattleSide::Defender),
        Some(Footprint {
            position,
            other: Some(at(&r, 4, 5))
        })
    );
    assert_eq!(
        f.footprint(position, false, BattleSide::Defender),
        Some(one(position))
    );
    // A position or other hex off the field has no footprint.
    assert_eq!(f.footprint(at(&r, 16, 3), true, BattleSide::Attacker), None);
    assert_eq!(f.footprint(at(&r, 0, 3), true, BattleSide::Defender), None);
    assert_eq!(f.footprint(Hex(200), false, BattleSide::Attacker), None);
    let fp = f.footprint(position, true, BattleSide::Attacker).unwrap();
    assert!(fp.covers(at(&r, 6, 5)) && fp.covers(position) && !fp.covers(at(&r, 4, 5)));
    // The front hex first, then the rear hex (the position).
    assert_eq!(fp.front(), at(&r, 6, 5));
    assert_eq!(fp.rear(), Some(position));
    assert_eq!(fp.hexes().collect::<Vec<_>>(), [at(&r, 6, 5), position]);
    assert_eq!(one(position).front(), position);
    assert_eq!(one(position).rear(), None);
    assert_eq!(one(position).hexes().collect::<Vec<_>>(), [position]);
    // A footprint from its front hex.
    for side in BattleSide::BOTH {
        for two_hex in [false, true] {
            let fp = f.footprint(position, two_hex, side).unwrap();
            assert_eq!(f.footprint_at_front(fp.front(), two_hex, side), Some(fp));
        }
    }
    assert_eq!(
        f.footprint_at_front(at(&r, 0, 3), true, BattleSide::Attacker),
        None
    );
    // Distance between footprints is the smallest hex distance.
    assert_eq!(f.footprint_distance(fp, one(at(&r, 9, 5))), 3);
    assert!(!f.touches(fp, one(at(&r, 9, 5))));
    assert!(f.touches(fp, one(at(&r, 7, 5))));
    // Overlap is not touching.
    assert!(!f.touches(fp, one(position)));
}

// covers: BAT-GEO-002, BAT-ATK-001
#[test]
fn attack_positions_cover_every_touching_position_once() {
    let r = ruleset();
    let f = field(&r);
    let count = |two_hex, side, target| {
        f.attack_positions(two_hex, side, target)
            .iter()
            .flatten()
            .count()
    };
    let single = f
        .footprint(at(&r, 8, 5), false, BattleSide::Defender)
        .unwrap();
    let double = f
        .footprint(at(&r, 8, 5), true, BattleSide::Defender)
        .unwrap();
    // One-hex attacker: the 6 neighbours of a one-hex target, the 8 of a two-hex one.
    assert_eq!(count(false, BattleSide::Attacker, single), 6);
    assert_eq!(count(false, BattleSide::Attacker, double), 8);
    // Two-hex attacker: also positions where only its rear hex touches (either side's).
    for side in BattleSide::BOTH {
        assert_eq!(count(true, side, single), 8);
        assert_eq!(count(true, side, double), 10);
    }
    for (side, two_hex, target) in BattleSide::BOTH.into_iter().flat_map(|side| {
        [
            (side, false, single),
            (side, false, double),
            (side, true, single),
            (side, true, double),
        ]
    }) {
        let positions = f.attack_positions(two_hex, side, target);
        assert_eq!(positions.len(), ATTACK_SLOTS);
        for p in positions.iter().flatten() {
            let fp = f.footprint(*p, two_hex, side).unwrap();
            assert!(f.touches(fp, target));
        }
        // Brute force: every touching position appears.
        let all = (0..187u8)
            .filter_map(|h| f.footprint(Hex(h), two_hex, side))
            .filter(|fp| f.touches(*fp, target))
            .count();
        assert_eq!(all, positions.iter().flatten().count());
    }
    // In a corner, positions off the field are missing.
    let corner = f
        .footprint(at(&r, 0, 0), false, BattleSide::Attacker)
        .unwrap();
    assert_eq!(count(false, BattleSide::Defender, corner), 3);
}

// ---------------------------------------------------------------- BAT-GEO-003 obstacles

// covers: BAT-GEO-003
#[test]
fn obstacles_are_validated_and_observed() {
    let r = ruleset();
    let s = battle_with_obstacles(&r, &[(0, "fixed", 1)], &[(0, "slow", 1)], &[(6, 5), (7, 2)]);
    let obstacles = s.obstacles();
    assert_eq!(obstacles.len(), 2);
    assert!(obstacles.contains(at(&r, 6, 5)) && obstacles.contains(at(&r, 7, 2)));
    assert_eq!(
        obstacles.iter().collect::<Vec<_>>(),
        [at(&r, 7, 2), at(&r, 6, 5)]
    );
    for viewer in BattleSide::BOTH {
        assert_eq!(crate::observe(&s, &r, viewer).obstacles, obstacles);
    }
    let mut bad = setup(&r, &[(0, "fixed", 1)], &[(0, "slow", 1)]);
    for hexes in [vec![at(&r, 0, 5).0], vec![200], vec![40, 41, 40]] {
        let repeated = *hexes.last().unwrap();
        bad.obstacles = blocked(hexes);
        assert_eq!(
            BattleState::new(&r, &bad, 1),
            Err(SetupError::InvalidObstacle(repeated))
        );
    }
}

#[test]
fn hex_sets() {
    let mut set = HexSet::new();
    assert!(set.is_empty());
    set.insert(Hex(0));
    set.insert(Hex(255));
    set.insert(Hex(64));
    set.insert(Hex(64));
    assert_eq!(set.len(), 3);
    assert!(!set.is_empty() && set.contains(Hex(255)) && !set.contains(Hex(1)));
    assert_eq!(set.iter().collect::<Vec<_>>(), [Hex(0), Hex(64), Hex(255)]);
    assert_eq!(set, set.clone());
    assert_ne!(hash_of(&set), hash_of(&HexSet::default()));
}

// ---------------------------------------------------------------- BAT-SET-001 placement

// covers: BAT-SET-001
#[test]
fn two_hex_stacks_and_obstacles_shift_the_starting_position() {
    let r = ruleset();
    // Formation [2, 5, 8]: the two-hex stack's position (its rear hex) takes the edge column.
    let s = battle_with_obstacles(
        &r,
        &[(0, "wide", 1), (1, "fixed", 1), (2, "fixed", 1)],
        &[(0, "wide", 1), (1, "fixed", 1)],
        // (1, 5) is blocked: the second attacker stack takes the closest free hex, preferring
        // the column nearer the centre.
        &[(1, 5)],
    );
    let st = s.stacks();
    assert_eq!(st[0].hex, at(&r, 1, 2));
    assert_eq!(st[0].footprint(field(&r)).front(), at(&r, 2, 2));
    assert_eq!(st[1].hex, at(&r, 2, 5));
    assert_eq!(st[2].hex, at(&r, 1, 8));
    // Defender: formation [3, 7] in column 15, a two-hex stack's front hex one column further in.
    assert_eq!(st[3].hex, at(&r, 15, 3));
    assert_eq!(st[3].footprint(field(&r)).front(), at(&r, 14, 3));
    assert_eq!(st[4].hex, at(&r, 15, 7));
}

// covers: BAT-SET-001
#[test]
fn placement_fails_when_no_hex_is_free() {
    let r = ruleset();
    let mut sp = setup(&r, &[(0, "fixed", 1)], &[(0, "slow", 1)]);
    // Block every standable hex.
    sp.obstacles = blocked(
        (0..187u8)
            .filter(|h| field(&r).is_standable(Hex(*h)))
            .collect(),
    );
    assert_eq!(
        BattleState::new(&r, &sp, 1),
        Err(SetupError::NoRoom {
            side: BattleSide::Attacker,
            slot: 0
        })
    );
}

// ---------------------------------------------------------------- BAT-MOV-001 walking

// covers: BAT-MOV-001, BAT-GEO-003
#[test]
fn obstacles_block_walkers() {
    let r = ruleset();
    // fixed (speed 5) at (1, 5); a wall of obstacles in column 3 around row 5.
    let wall = [
        (3, 3),
        (3, 4),
        (3, 5),
        (2, 6),
        (3, 6),
        (2, 7),
        (3, 7),
        (2, 4),
        (2, 3),
    ];
    let s = battle_with_obstacles(&r, &[(0, "fixed", 1)], &[(0, "slow", 1)], &wall);
    assert_eq!(
        act(&mut s.clone(), &r, Command::Move { to: at(&r, 3, 5) }),
        Err(CommandError::Unreachable)
    );
    // (4, 5) is two hexes away in a straight line, but the walk around the wall is longer than 5.
    assert_eq!(
        act(&mut s.clone(), &r, Command::Move { to: at(&r, 5, 5) }),
        Err(CommandError::Unreachable)
    );
    act(&mut s.clone(), &r, Command::Move { to: at(&r, 2, 5) }).unwrap();
}

// covers: BAT-MOV-001, BAT-GEO-002
#[test]
fn two_hex_walkers_need_room_for_both_hexes() {
    let r = ruleset();
    let mut s = battle(&r, &[(0, "wide", 1)], &[(0, "slow", 1)], 1);
    // wide (speed 5) has its position, its rear hex, at (1, 5) and its front hex at (2, 5).
    assert_eq!(s.stacks()[0].hex, at(&r, 1, 5));
    // Its position cannot be in column 0: the rear hex would be in the side column.
    assert_eq!(
        act(&mut s.clone(), &r, Command::Move { to: at(&r, 0, 4) }),
        Err(CommandError::InvalidHex)
    );
    // Nor in column 15: the front hex would be in the other side column.
    place(&mut s, &r, 0, 13, 5);
    assert_eq!(
        act(&mut s.clone(), &r, Command::Move { to: at(&r, 15, 4) }),
        Err(CommandError::Unreachable)
    );
    place(&mut s, &r, 0, 1, 5);
    // Room for one hex is too small: with a stack on (4, 5), the position (3, 5) would put the
    // front hex on it.
    place(&mut s, &r, 1, 4, 5);
    assert_eq!(
        act(&mut s.clone(), &r, Command::Move { to: at(&r, 3, 5) }),
        Err(CommandError::Unreachable)
    );
    // Moving one hex on is fine: its own hexes do not block it.
    let mut on = s.clone();
    act(&mut on, &r, Command::Move { to: at(&r, 2, 5) }).unwrap();
    assert_eq!(on.stacks()[0].footprint(field(&r)).front(), at(&r, 3, 5));
}

// ---------------------------------------------------------------- BAT-MOV-002 flying

// covers: BAT-MOV-002
#[test]
fn flyers_and_teleporters_ignore_what_lies_between() {
    let r = ruleset();
    let wall: Vec<(i32, i32)> = (0..11).map(|y| (4, y)).collect();
    for flyer in ["flyer", "porter"] {
        // flyer: speed 7; porter: speed 6, teleports.
        let s = battle_with_obstacles(&r, &[(0, flyer, 1)], &[(0, "slow", 1)], &wall);
        act(&mut s.clone(), &r, Command::Move { to: at(&r, 7, 5) }).unwrap();
        // The distance limit still applies, and obstacles cannot be landed on.
        assert_eq!(
            act(&mut s.clone(), &r, Command::Move { to: at(&r, 9, 5) }),
            Err(CommandError::Unreachable)
        );
        assert_eq!(
            act(&mut s.clone(), &r, Command::Move { to: at(&r, 4, 5) }),
            Err(CommandError::Unreachable)
        );
    }
    // A walker cannot pass the wall at all.
    let s = battle_with_obstacles(&r, &[(0, "fixed", 1)], &[(0, "slow", 1)], &wall);
    assert_eq!(
        act(&mut s.clone(), &r, Command::Move { to: at(&r, 5, 5) }),
        Err(CommandError::Unreachable)
    );
    // Two-hex flyers need room for both hexes where they land: at (5, 5) the front hex would be
    // the other stack's.
    let mut s = battle(&r, &[(0, "dragon", 1)], &[(0, "slow", 1)], 1);
    place(&mut s, &r, 1, 6, 5);
    assert_eq!(
        act(&mut s.clone(), &r, Command::Move { to: at(&r, 5, 5) }),
        Err(CommandError::Unreachable)
    );
    act(&mut s, &r, Command::Move { to: at(&r, 7, 5) }).unwrap();
}

// ---------------------------------------------------------------- BAT-ATK-001 attack positions

// covers: BAT-ATK-001, BAT-GEO-002
#[test]
fn two_hex_stacks_attack_and_are_attacked_on_every_side() {
    let r = ruleset();
    // A one-hex attacker hits a two-hex target from behind its rear hex.
    let mut s = battle(&r, &[(0, "fast", 1)], &[(0, "wide", 1)], 1);
    activate(&mut s, 0);
    place(&mut s, &r, 1, 9, 5); // defender position (rear hex) (9, 5), front hex (8, 5)
    place(&mut s, &r, 0, 12, 5);
    let behind = Command::Attack {
        target: StackId(1),
        from: at(&r, 10, 5),
    };
    assert!(legal(&s, &r).contains(&behind));
    act(&mut s.clone(), &r, behind).unwrap();
    // A two-hex attacker can strike with only its rear hex touching the target.
    let mut s = battle(&r, &[(0, "wide", 1)], &[(0, "slow", 1)], 1);
    activate(&mut s, 0);
    place(&mut s, &r, 1, 4, 5); // target at (4, 5); attacker at (1, 5), front hex (2, 5)
    let rear_touch = Command::Attack {
        target: StackId(1),
        from: at(&r, 5, 5), // position (5, 5) next to the target, front hex (6, 5)
    };
    assert!(legal(&s, &r).contains(&rear_touch));
    act(&mut s.clone(), &r, rear_touch).unwrap();
    // Positions whose footprint is not on standable hexes are invalid, far ones not adjacent.
    let mut edge = s.clone();
    place(&mut edge, &r, 1, 2, 7);
    assert_eq!(
        act(
            &mut edge.clone(),
            &r,
            Command::Attack {
                target: StackId(1),
                from: at(&r, 0, 7)
            }
        ),
        Err(CommandError::InvalidHex)
    );
    assert_eq!(
        act(
            &mut s,
            &r,
            Command::Attack {
                target: StackId(1),
                from: at(&r, 8, 5)
            }
        ),
        Err(CommandError::NotAdjacent)
    );
}
