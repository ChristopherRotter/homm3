//! Tests of the scene over time. The three recorded sequences are replayed from their events with
//! the numbers of their creatures (the asset tables' times, the DEF groups' frame counts and the
//! standing frame's top edge, read from the user's archives: numbers only, ADR-0009), and must
//! give the timelines the harness recorded in the original (`clients/situations/timelines/`).

use super::*;
use alloc::string::ToString;
use h3_protocol::wire::{Stack, Strike, WireU64};

use crate::record::tests::observation;
use crate::scene::tests::{layout, stack, tables};

/// The recorded timelines (design §7.6).
const WALK: &str = include_str!("../../../../situations/timelines/walk.txt");
const SHOT: &str = include_str!("../../../../situations/timelines/shot.txt");
const MELEE: &str = include_str!("../../../../situations/timelines/melee.txt");

/// Creature handles of the tests.
const CHAMPION: u16 = 0;
const MARKSMAN: u16 = 1;
const WOLF_RIDER: u16 = 2;
const GOBLIN: u16 = 3;
const ANGEL: u16 = 4;
const PIKEMAN: u16 = 5;
const GNOLL: u16 = 6;

/// A look: `times` and the frames of each group.
fn look(times: [u32; 4], frames: &[(u32, u32)], top: i32) -> Look {
    Look {
        frames: frames.iter().copied().collect(),
        times,
        climax: 0,
        missile_offsets: [[0, 0]; 3],
        top,
        missile: None,
    }
}

/// The numbers of the sequences' creatures (`clients/assets/base.toml`, their DEFs).
fn looks() -> BTreeMap<u16, Look> {
    let mut marksman = look(
        [9000, 500, 500, 115],
        &[
            (0, 8),
            (2, 8),
            (3, 6),
            (14, 8),
            (15, 8),
            (16, 8),
            (20, 2),
            (21, 2),
        ],
        190,
    );
    marksman.climax = 7;
    marksman.missile_offsets = [[34, -71], [38, -61], [17, -53]];
    // `PLCBOWX.DEF`: 30 x 30, nine frames in group 0 (the user's archive; numbers only).
    marksman.missile = Some(Missile {
        def: "PLCBOWX.DEF".to_string(),
        size: [30, 30],
        angles: vec![90, 72, 45, 27, 0, -27, -45, -72, -90],
    });
    let base = [
        (
            CHAMPION,
            look(
                [9000, 450, 500, 115],
                &[(0, 8), (2, 4), (3, 8), (12, 10), (20, 3), (21, 4)],
                165,
            ),
        ),
        (MARKSMAN, marksman),
        (
            WOLF_RIDER,
            look(
                [9000, 465, 500, 115],
                &[
                    (0, 6),
                    (2, 8),
                    (3, 7),
                    (4, 8),
                    (5, 7),
                    (11, 7),
                    (12, 7),
                    (13, 7),
                    (20, 1),
                    (21, 1),
                ],
                179,
            ),
        ),
        (
            GOBLIN,
            look(
                [9000, 605, 500, 115],
                &[(0, 9), (2, 8), (3, 7), (4, 9), (5, 9), (20, 1), (21, 1)],
                200,
            ),
        ),
        (
            ANGEL,
            look(
                [9000, 410, 500, 115],
                &[
                    (0, 7),
                    (2, 8),
                    (3, 6),
                    (4, 10),
                    (5, 8),
                    (11, 6),
                    (12, 6),
                    (13, 6),
                    (20, 2),
                    (21, 3),
                ],
                170,
            ),
        ),
        (PIKEMAN, look([9000, 575, 500, 115], &[(0, 6), (2, 7)], 171)),
        (GNOLL, look([9000, 500, 500, 115], &[(2, 8)], 192)),
    ];
    base.into_iter().collect()
}

/// A living stack of creature `creature` on `hex`.
fn at(id: u8, side: Side, creature: u16, hex: u8, two_hex: bool) -> Stack {
    let mut s = stack(id, side, creature, hex, two_hex);
    s.count = 40;
    s
}

/// The snapshot with `stacks`, the first one active (the human's, as in the sequences).
fn snapshot(stacks: Vec<Stack>) -> Observation {
    let mut o = observation();
    o.active = stacks.first().map(|s| s.id);
    o.stacks = stacks;
    o
}

fn moved(stack: u8, from: u8, to: u8) -> Event {
    Event::Moved { stack, from, to }
}

fn flags(ranged: bool, retaliation: bool, secondary: bool) -> Strike {
    Strike {
        retaliation,
        ranged,
        secondary,
        death_blow: false,
        lucky: false,
        double_damage: false,
    }
}

fn struck(attacker: u8, target: u8, kills: u32, flags: Strike) -> Event {
    Event::Struck {
        attacker,
        target,
        damage: WireU64(1),
        kills,
        flags,
    }
}

/// The timeline of `snapshot` and `commands` (each with its time) at speed 0.
fn play(snapshot: &Observation, commands: &[(Option<u32>, Vec<Event>)]) -> Timeline {
    let layout = layout();
    let stage = Stage {
        layout: &layout,
        width: 17,
        groups: tables().battle.groups,
    };
    let commands: Vec<Command<'_>> = commands
        .iter()
        .map(|(at, events)| Command { at: *at, events })
        .collect();
    timeline(snapshot, &commands, &looks(), stage, Speed::default()).unwrap()
}

/// The lines up to `ms` (the harness's recording ends there).
fn until(lines: &str, ms: u32) -> String {
    lines
        .lines()
        .filter(|l| l.split(' ').next().and_then(|t| t.parse::<u32>().ok()) <= Some(ms))
        .map(|l| format!("{l}\n"))
        .collect()
}

#[test]
fn the_combat_speeds_scale_as_the_executable() {
    let [normal, fast, fastest] = [0, 1, 2].map(Speed::new);
    assert_eq!(Speed::new(7), fastest);
    assert_eq!(fast.setting(), 1);
    // §7.1: 100 gives 100, 62, 40; 150 gives 150, 94, 60; 33 gives 33, 20, 13; 115: 115, 72, 46.
    for (ms, scaled) in [
        (100, [100, 62, 40]),
        (150, [150, 94, 60]),
        (33, [33, 20, 13]),
        (115, [115, 72, 46]),
    ] {
        assert_eq!([normal, fast, fastest].map(|s| s.scale(ms)), scaled);
    }
    // §7.3: the Champions' walk frame (W 450, 8 frames) is 56, 35 and 22 ms.
    assert_eq!(
        [normal, fast, fastest].map(|s| s.ratio(450, 8)),
        [56, 35, 22]
    );
    assert_eq!(normal.ratio(450, 0), 450);
}

#[test]
fn the_walk_sequence_plays_as_recorded() {
    // `seq-walk.toml`: the Champions walk from hex 35 to 39 (the harness clicks hex 40, the
    // front; the engine's position is the rear hex), hex by hex.
    let o = snapshot(vec![
        at(0, Side::Attacker, CHAMPION, 35, true),
        at(1, Side::Attacker, MARKSMAN, 137, false),
        at(2, Side::Defender, WOLF_RIDER, 49, true),
        at(3, Side::Defender, GOBLIN, 151, false),
    ]);
    let walk = vec![
        moved(0, 35, 36),
        moved(0, 36, 37),
        moved(0, 37, 38),
        moved(0, 38, 39),
        Event::TurnStarted { stack: 1 },
    ];
    let t = play(&o, &[(None, walk)]);
    // T4b-1, the reference frames (§7.9): the Champions' outline until the walk's first frame
    // shows, the Marksmen's from the walk's end; the opening's idle step, none while the walk
    // plays, two at once when it ends, then one per 100 ms.
    let outlined: Vec<Option<u8>> = [0, 99, 100, 2591, 2592].map(|t2| t.outlined(t2)).into();
    assert_eq!(outlined, [Some(0), Some(0), None, None, Some(1)]);
    let pulse: Vec<u32> = [0, 99, 2591, 2592, 2691, 2692, 2792]
        .map(|t2| t.pulse(t2))
        .into();
    assert_eq!(pulse, [1, 1, 1, 3, 3, 4, 5]);
    assert_eq!(t.projectile(500), None);
    assert_eq!(t.lines(), WALK);
    // What shows: the walk's first frame from 100 ms, standing from 2592; the next turn then.
    assert_eq!(t.at(99)[&0], still(2, 35));
    assert_eq!(t.at(100)[&0], pose(20, 0, 35));
    let mid = t.at(500)[&0];
    assert_eq!(
        (mid.group, mid.frame, mid.hex, mid.offset),
        (0, 1, 35, [5, 0])
    );
    assert_eq!(t.at(2592)[&0], still(2, 39));
    assert_eq!(t.end, 2592);
    assert_eq!((t.active(0), t.active(2592)), (None, Some(1)));
}

#[test]
fn the_shot_sequence_plays_as_recorded() {
    // `seq-shot.toml`: the Marksmen shoot the Goblins twice; the projectile's 14 steps.
    let o = snapshot(vec![
        at(0, Side::Attacker, MARKSMAN, 35, false),
        at(1, Side::Attacker, PIKEMAN, 137, false),
        at(2, Side::Defender, GOBLIN, 49, false),
        at(3, Side::Defender, GNOLL, 151, false),
    ]);
    let shot = struck(0, 2, 3, flags(true, false, false));
    let t = play(
        &o,
        &[(
            None,
            vec![shot.clone(), shot, Event::TurnStarted { stack: 2 }],
        )],
    );
    assert_eq!(t.lines(), SHOT);
    // The hit's first frame never shows: at 988 the second does, the shooter standing.
    assert_eq!(t.at(988)[&2], pose(3, 1, 49));
    assert_eq!(t.at(988)[&0], still(2, 35));
    // T4b-1 (§7.9, the reference frames): the arrow's 14 steps every 33 ms from the climax
    // frame, frame 4 (straight), its canvas centred on the start and moved by 40, 2 per step.
    let arrow = |at: [i32; 2]| {
        Some(Projectile {
            def: "PLCBOWX.DEF".to_string(),
            frame: 4,
            at,
            mirrored: false,
        })
    };
    assert_eq!(t.projectile(525), None);
    assert_eq!(t.projectile(526), arrow([125, 136]));
    assert_eq!(t.projectile(559), arrow([166, 138]));
    assert_eq!(t.projectile(987), arrow([658, 162]));
    assert_eq!(t.projectile(988), None);
    assert_eq!(t.projectile(2114), arrow([125, 136]));
    assert_eq!(t.flights.len(), 2);
    assert_eq!((t.outlined(99), t.outlined(100)), (Some(0), None));
}

#[test]
fn the_melee_sequence_plays_as_recorded() {
    // `seq-melee.toml`: the Angels fly to hex 43; the computer's Wolf Riders walk to 45 and
    // strike them, the Angels strike back. The second command starts by default 100 ms after
    // the first's last frame shows.
    let mut angels = at(0, Side::Attacker, ANGEL, 35, false);
    angels.flies = true;
    let o = snapshot(vec![
        angels,
        at(1, Side::Attacker, PIKEMAN, 137, false),
        at(2, Side::Defender, WOLF_RIDER, 49, true),
        at(3, Side::Defender, GOBLIN, 151, false),
    ]);
    let fly = vec![moved(0, 35, 43), Event::TurnStarted { stack: 2 }];
    let attack = vec![
        moved(2, 49, 48),
        moved(2, 48, 47),
        moved(2, 47, 46),
        moved(2, 46, 45),
        struck(2, 0, 1, flags(false, false, false)),
        struck(0, 2, 1, flags(false, true, false)),
        Event::TurnStarted { stack: 3 },
    ];
    let t = play(&o, &[(None, fly), (None, attack)]);
    assert_eq!(until(&t.lines(), 6000), MELEE);
    // T4b-1 (§7.9, the reference frames): the flight's point moves on before each frame by
    // f32 (352 / 3) / 7, truncated: the last of 21 frames is 351 pixels on, not 352.
    let offsets: Vec<[i32; 2]> = [300, 358, 1402, 1460].map(|t2| t.at(t2)[&0].offset).into();
    assert_eq!(offsets, [[16, 0], [33, 0], [335, 0], [351, 0]]);
    // The computer's Wolf Riders: outlined with its idle step, until their first frame shows.
    let outlined: Vec<Option<u8>> = [1818, 1918, 2017, 2018].map(|t2| t.outlined(t2)).into();
    assert_eq!(outlined, [None, Some(2), Some(2), None]);
    assert_eq!((t.pulse(1917), t.pulse(1918), t.pulse(5000)), (1, 2, 2));
}

#[test]
fn a_creature_without_numbers_is_an_error() {
    let mut o = snapshot(vec![at(0, Side::Attacker, 99, 35, false)]);
    let layout = layout();
    let stage = Stage {
        layout: &layout,
        width: 0,
        groups: tables().battle.groups,
    };
    let err = timeline(&o, &[], &looks(), stage, Speed::default()).unwrap_err();
    assert_eq!(err, AnimError::Look(99));
    assert_eq!(
        err.to_string(),
        "no animation numbers for creature handle 99"
    );
    // A dead stack needs none and is not in the timeline.
    o.stacks[0].alive = false;
    let t = timeline(&o, &[], &looks(), stage, Speed::default()).unwrap();
    assert!(t.members.is_empty());
    assert_eq!((t.lines(), t.end), (String::new(), 0));
}

#[test]
fn moves_of_unknown_stacks_and_broken_runs() {
    let o = snapshot(vec![
        at(0, Side::Attacker, PIKEMAN, 35, false),
        at(1, Side::Attacker, PIKEMAN, 52, false),
    ]);
    let events = vec![
        // An unknown stack: nothing.
        moved(9, 1, 2),
        // A run broken by another stack's move, and by a move that does not chain.
        moved(0, 35, 36),
        moved(1, 52, 53),
        moved(1, 60, 61),
        Event::Waited { stack: 0 },
    ];
    let t = play(&o, &[(None, events)]);
    let at_end = t.at(t.end);
    // The Pikemen have no move start or end groups: a cycle per hex, then standing.
    assert_eq!(at_end[&0], still(2, 36));
    assert_eq!(at_end[&1], still(2, 61));
    let walked: Vec<u8> = t
        .keys
        .iter()
        .filter(|k| k.stack == 1)
        .map(|k| k.pose.hex)
        .collect();
    // From 52 a step to 53, then the move 60 to 61 on its own (drawn from 60).
    assert!(walked.contains(&52) && walked.contains(&60));
}

#[test]
fn a_flight_without_a_distance_is_one_cycle() {
    let mut angels = at(0, Side::Attacker, ANGEL, 35, false);
    angels.flies = true;
    let mut numbers = looks();
    numbers.get_mut(&ANGEL).unwrap().times[3] = 0;
    let layout = layout();
    let stage = Stage {
        layout: &layout,
        width: 17,
        groups: tables().battle.groups,
    };
    let o = snapshot(vec![angels]);
    let events = [moved(0, 35, 37)];
    let commands = [Command {
        at: None,
        events: &events,
    }];
    let t = timeline(&o, &commands, &numbers, stage, Speed::default()).unwrap();
    let cycle = t.keys.iter().filter(|k| k.pose.group == 0).count();
    assert_eq!(cycle, 7);
}

#[test]
fn strikes_react_by_direction_defence_and_death() {
    let o = snapshot(vec![
        at(0, Side::Attacker, WOLF_RIDER, 52, true),
        at(1, Side::Defender, GOBLIN, 36, false),
        at(2, Side::Defender, GOBLIN, 70, false),
        at(3, Side::Defender, ANGEL, 54, false),
    ]);
    let melee = flags(false, false, false);
    let area = flags(false, false, true);
    let commands = [
        // The Goblins above defend; the strike up finds them defending (group 4).
        (
            None,
            vec![Event::Defended { stack: 1 }, struck(0, 1, 1, melee)],
        ),
        // Their turn clears it: hit (3). A stray secondary hit only counts its kills.
        (
            Some(9000),
            vec![
                Event::TurnStarted { stack: 1 },
                struck(0, 1, 1, melee),
                struck(0, 3, 1, area),
            ],
        ),
        // Down at the Goblins below, the Angels beside hit too and die (a shorter death keeps its
        // last frame while the longer reaction runs); a strike of another attacker and a
        // non-secondary strike end the run.
        (
            None,
            vec![
                struck(0, 2, 40, melee),
                struck(0, 3, 39, area),
                struck(0, 1, 0, melee),
                struck(1, 0, 0, area),
            ],
        ),
    ];
    let t = play(&o, &commands);
    let groups = |stack: u8| -> Vec<u32> {
        let mut g: Vec<u32> = t
            .keys
            .iter()
            .filter(|k| k.stack == stack)
            .map(|k| k.pose.group)
            .collect();
        g.dedup();
        g
    };
    assert_eq!(groups(0), [11, 2, 11, 2, 13, 2, 11, 2]);
    assert_eq!(groups(1), [2, 4, 2, 3, 2, 3, 2]);
    // The Goblins below: dead, their death's last frame stays; the Angels too.
    let last = t.at(t.end);
    assert_eq!(last[&2], pose(5, 8, 70));
    assert_eq!(last[&3], pose(5, 7, 54));
    // A dead stack takes no part later: strikes at or by it are skipped.
    let more = [
        struck(2, 0, 1, melee),
        struck(0, 2, 1, melee),
        moved(2, 70, 71),
    ];
    let mut commands = commands.to_vec();
    commands.push((None, more.to_vec()));
    let after = play(&o, &commands);
    assert_eq!(after.at(after.end)[&2], pose(5, 8, 70));
}

#[test]
fn a_reaction_without_frames_and_a_strike_at_nothing() {
    let mut numbers = looks();
    numbers.get_mut(&GOBLIN).unwrap().frames.remove(&5);
    let layout = layout();
    let stage = Stage {
        layout: &layout,
        width: 17,
        groups: tables().battle.groups,
    };
    let o = snapshot(vec![
        at(0, Side::Attacker, WOLF_RIDER, 52, true),
        at(1, Side::Defender, GOBLIN, 54, false),
    ]);
    let events = [
        struck(0, 1, 40, flags(false, false, false)),
        struck(0, 9, 1, flags(false, false, false)),
    ];
    let commands = [Command {
        at: None,
        events: &events,
    }];
    let t = timeline(&o, &commands, &numbers, stage, Speed::default()).unwrap();
    // Dead without a death group: it stands.
    assert_eq!(t.at(t.end)[&1], still(2, 54));
    // The second strike, at no known stack, is straight (12).
    assert_eq!(t.keys.last().unwrap().pose, still(2, 52));
    assert!(t.keys.iter().any(|k| k.pose.group == 12));
}

#[test]
fn shots_aim_up_straight_and_down() {
    // The shot's group from the angle (0x469880): above 25° up, below −25° down; straight above
    // or below the sign is inverted. A two-hex target is aimed at from its middle.
    let groups = |shooter: Stack, target: Stack| -> u32 {
        let o = snapshot(vec![shooter, target]);
        let t = play(
            &o,
            &[(None, vec![struck(0, 1, 1, flags(true, false, false))])],
        );
        t.keys[0].pose.group
    };
    let marksmen = |side, hex| at(0, side, MARKSMAN, hex, false);
    let goblins = |side, hex, two| at(1, side, GOBLIN, hex, two);
    assert_eq!(
        groups(
            marksmen(Side::Attacker, 154),
            goblins(Side::Defender, 2, false)
        ),
        14
    );
    assert_eq!(
        groups(
            marksmen(Side::Attacker, 2),
            goblins(Side::Defender, 154, false)
        ),
        16
    );
    assert_eq!(
        groups(
            marksmen(Side::Attacker, 35),
            goblins(Side::Defender, 49, true)
        ),
        15
    );
    assert_eq!(
        groups(
            marksmen(Side::Defender, 49),
            goblins(Side::Attacker, 35, true)
        ),
        15
    );
    // Straight below: dx 0 when the offset cancels the half hex is rare; a missile offset of 0
    // and a target in the same column.
    let mut numbers = looks();
    numbers.get_mut(&MARKSMAN).unwrap().missile_offsets = [[0, 0]; 3];
    let layout = layout();
    let stage = Stage {
        layout: &layout,
        width: 17,
        groups: tables().battle.groups,
    };
    let column = |target: u8| {
        let o = snapshot(vec![
            marksmen(Side::Attacker, 70),
            goblins(Side::Defender, target, false),
        ]);
        let events = [struck(0, 1, 1, flags(true, false, false))];
        let commands = [Command {
            at: None,
            events: &events,
        }];
        timeline(&o, &commands, &numbers, stage, Speed::default())
            .unwrap()
            .keys[0]
            .pose
            .group
    };
    // Two rows below (same column): the target lies lower, dy > 0: +90°, up.
    assert_eq!(column(104), 14);
    // Two rows above, its middle above: −90°, down.
    assert_eq!(column(36), 16);
}

#[test]
fn shots_without_a_climax_or_a_target() {
    let mut numbers = looks();
    numbers.get_mut(&MARKSMAN).unwrap().climax = 0;
    let layout = layout();
    let stage = Stage {
        layout: &layout,
        width: 17,
        groups: tables().battle.groups,
    };
    let o = snapshot(vec![
        at(0, Side::Attacker, MARKSMAN, 35, false),
        at(1, Side::Defender, GOBLIN, 49, false),
    ]);
    let run = |numbers: &BTreeMap<u16, Look>, events: &[Event]| {
        let commands = [Command { at: None, events }];
        timeline(&o, &commands, numbers, stage, Speed::default()).unwrap()
    };
    let shot = struck(0, 1, 1, flags(true, false, false));
    // No climax: every frame before the projectile, none left for the strike's loop.
    let t = run(&numbers, core::slice::from_ref(&shot));
    assert_eq!(
        t.keys
            .iter()
            .filter(|k| k.stack == 0 && k.pose.group == 15)
            .count(),
        8
    );
    // A climax beyond the group: the same.
    numbers.get_mut(&MARKSMAN).unwrap().climax = 30;
    assert_eq!(run(&numbers, &[shot]).keys, t.keys);
    // At no known stack: straight, no projectile; by no known stack: nothing.
    let t = run(&numbers, &[struck(0, 9, 1, flags(true, false, false))]);
    assert!(t.keys.iter().all(|k| k.stack == 0));
    assert!(
        run(&numbers, &[struck(9, 1, 1, flags(true, false, false))])
            .keys
            .is_empty()
    );
}

#[test]
fn commands_start_at_their_time_or_by_default() {
    let o = snapshot(vec![at(0, Side::Attacker, PIKEMAN, 35, false)]);
    let walk = vec![moved(0, 35, 36)];
    // A Wait draws nothing: the next command starts at once, on the deadline's grid.
    let t = play(
        &o,
        &[
            (None, vec![Event::Waited { stack: 0 }]),
            (None, walk.clone()),
        ],
    );
    assert_eq!(t.keys[0].drawn, 0);
    assert_eq!(t.keys[0].shown, 100);
    // A command given before the deadline: drawn then, shown at the deadline; one given later
    // draws at its time.
    let t = play(
        &o,
        &[
            (None, walk.clone()),
            (Some(10), vec![moved(0, 36, 37)]),
            (Some(5000), vec![moved(0, 37, 38)]),
        ],
    );
    let firsts: Vec<(u32, u32)> = t
        .keys
        .iter()
        .filter(|k| k.pose.frame == 0 && k.pose.group == 0)
        .map(|k| (k.drawn, k.shown))
        .collect();
    // On the grid the earlier frames left (a point every 100 ms), the first one after 5000.
    assert_eq!(firsts[2].0, 5000);
    assert!((5001..=5100).contains(&firsts[2].1));
    assert!(firsts[1].0 >= firsts[0].1);
}

#[test]
fn differences_list_both_sides() {
    assert!(differences("1\n2\n", "1\n2\n").is_empty());
    assert_eq!(differences("1\n3\n", "1\n2\n"), ["- 2", "+ 3"]);
    assert_eq!(differences("1\n2\n", "1\n"), ["+ 2"]);
    assert_eq!(differences("1\n", "1\n2\n"), ["- 2"]);
}

#[test]
fn f32_values_round_as_the_x87_stores_them() {
    let one = 1_i64 << F32_FRAC;
    // Exact values stay; 352 / 3 rounds to the nearest f32, 15 379 115 / 2^17.
    assert_eq!(f32_round(5, 1), 5 * one);
    assert_eq!(f32_round(-5, 2), -5 * one / 2);
    let third = f32_round(352, 3);
    assert_eq!(third, 15_379_115 * (one >> 17));
    // Between 2^20 and 2^21 an f32 steps by 1/8. Ties to even: 2^20 + 1/16 down to 2^20,
    // 2^20 + 3/16 up to 2^20 + 1/4; a remainder past the half (+ 1/48) rounds up.
    let (big, eighth) = (one << 20, one >> 3);
    assert_eq!(f32_round((1 << 24) + 1, 16), big);
    assert_eq!(f32_round((1 << 24) + 3, 16), big + 2 * eighth);
    assert_eq!(f32_round(((1 << 24) + 1) * 3 + 1, 48), big + eighth);
    assert_eq!(f32_trunc(-3 * one / 2), -1);
    assert_eq!(f32_trunc(7 * one / 2), 3);
}

#[test]
fn tangents_of_half_degrees_are_exact_enough() {
    assert_eq!(tangent(0), 0);
    assert!((tangent(90) - TAN_SCALE).abs() <= 2, "{}", tangent(90));
    assert!((tangent(-90) + TAN_SCALE).abs() <= 2);
    // tan(25°), the shot's threshold.
    assert!((tangent(50) - TAN_25).abs() <= 2);
    assert!(tangent(179) > 114 * TAN_SCALE);
    assert_eq!((tangent(180), tangent(-180)), (i64::MAX, i64::MIN));
}

#[test]
fn a_projectile_frame_follows_the_angle() {
    let missile = Missile {
        def: "A.DEF".to_string(),
        size: [2, 2],
        angles: vec![90, 72, 45, 27, 0, -27, -45, -72, -90],
    };
    let aim = |slope: [i64; 2]| Aim {
        slope,
        ..Aim::default()
    };
    // Straight: frame 4; 30° up: between the midpoints 36 and 13.5, frame 3 (27°); steeply
    // down: frame 8.
    assert_eq!(aim([100, 0]).frame(&missile), 4);
    assert_eq!(aim([100, -58]).frame(&missile), 3);
    assert_eq!(aim([1, 100]).frame(&missile), 8);
    // dx 0: the target below is +90° (frame 0), above −90° (the last).
    assert_eq!(aim([0, 5]).frame(&missile), 0);
    assert_eq!(aim([0, -5]).frame(&missile), 8);
    // A midpoint of 90°: every angle is at most it.
    let up = Missile {
        angles: vec![90, 90, 0],
        ..missile
    };
    assert_eq!(aim([0, 5]).frame(&up), 1);
}

#[test]
fn idle_steps_run_on_their_grid_and_catch_up() {
    let mut steps = Vec::new();
    let mut idle = Idle::default();
    idle.run(250, |t| steps.push(t));
    assert_eq!(steps, [0, 100, 200]);
    // Held up 300 ms: twice at once, then on the new grid.
    idle.from = 600;
    idle.run(700, |t| steps.push(t));
    assert_eq!(steps, [0, 100, 200, 600, 600, 700]);
    // The computer's tick: at most one.
    let mut idle = Idle { from: 0, next: 100 };
    assert!(idle.once(1000, &mut steps));
    assert_eq!(idle.next, 1000);
    assert!(!idle.once(999, &mut steps));
    // Without commands the steps go on from the opening.
    let o = snapshot(vec![at(0, Side::Attacker, PIKEMAN, 35, false)]);
    let t = play(&o, &[]);
    assert_eq!(
        (t.pulse(0), t.pulse(99), t.pulse(100), t.pulse(1000)),
        (1, 1, 2, 11)
    );
}

#[test]
fn a_shooter_without_a_projectile_waits_as_one_flies() {
    let mut numbers = looks();
    numbers.get_mut(&MARKSMAN).unwrap().missile = None;
    let layout = layout();
    let stage = Stage {
        layout: &layout,
        width: 17,
        groups: tables().battle.groups,
    };
    let o = snapshot(vec![
        at(0, Side::Attacker, MARKSMAN, 35, false),
        at(2, Side::Defender, GOBLIN, 49, false),
    ]);
    let shot = [struck(0, 2, 3, flags(true, false, false))];
    let commands = [Command {
        at: None,
        events: &shot,
    }];
    let t = timeline(&o, &commands, &numbers, stage, Speed::default()).unwrap();
    assert!(t.flights.is_empty());
    assert_eq!(t.projectile(600), None);
    // The hit still starts after the 14 steps.
    assert_eq!(t.at(988)[&2], pose(3, 1, 49));
}
