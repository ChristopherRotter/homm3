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
const TURN: &str = include_str!("../../../../situations/timelines/turn.txt");

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
            (7, 2),
            (8, 2),
            (9, 2),
            (10, 2),
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
                &[
                    (0, 8),
                    (2, 4),
                    (3, 8),
                    (7, 3),
                    (8, 3),
                    (9, 3),
                    (10, 3),
                    (12, 10),
                    (20, 3),
                    (21, 4),
                ],
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
                    (7, 3),
                    (8, 3),
                    (9, 3),
                    (10, 3),
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
                    (7, 2),
                    (8, 2),
                    (9, 2),
                    (10, 2),
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

/// `pose` facing left (a defender's, or a turned attacker's).
fn left(pose: Pose) -> Pose {
    Pose {
        mirrored: true,
        ..pose
    }
}

/// `pose` in draw layer `layer`.
fn layered(pose: Pose, layer: u8) -> Pose {
    Pose { layer, ..pose }
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
    assert_eq!(t.at(988)[&2], left(pose(3, 1, 49)));
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
fn the_turn_sequence_plays_as_recorded() {
    // `seq-turn.toml` (T4b-2a, design §7.10): the walk sequence's armies; the Champions walk one
    // hex up and to the left, from 35 to 18. The step faces left: the first step's turn,
    // animated (7, the flip onto their other hex 36, 10, standing), the step up in layer 3, group
    // 21 on 19 (18 turned), and a plain move's turn back (9, the flip onto 18, 8, standing).
    let o = snapshot(vec![
        at(0, Side::Attacker, CHAMPION, 35, true),
        at(1, Side::Attacker, MARKSMAN, 137, false),
        at(2, Side::Defender, WOLF_RIDER, 49, true),
        at(3, Side::Defender, GOBLIN, 151, false),
    ]);
    let walk = vec![moved(0, 35, 18), Event::TurnStarted { stack: 1 }];
    let t = play(&o, &[(None, walk)]);
    assert_eq!(t.lines(), TURN);
    let step = keys_of(&t, 0).into_iter().find(|k| k.0 == 0);
    assert_eq!(step, Some((0, 0, 36, true, 3)));
    assert_eq!(t.end, 2648);
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
    // T5a: between its hexes the flier is drawn over every row; on them in standing's pass.
    let layers: Vec<(u32, u8)> = t
        .keys
        .iter()
        .map(|k| (k.pose.group, k.pose.layer))
        .collect();
    assert!(
        layers
            .iter()
            .all(|&(group, layer)| (group == 0) == (layer == FLYING))
    );
    assert_eq!(layers.last(), Some(&(2, 4)));
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
    let layers = |stack: u8| -> Vec<u8> {
        let mut l: Vec<u8> = t
            .keys
            .iter()
            .filter(|k| k.stack == stack)
            .map(|k| k.pose.layer)
            .collect();
        l.dedup();
        l
    };
    // T4b-2a (§7.9): the striker 6, then 5 for a later strike of the command; the struck 5;
    // each back in 4 with the next frame that does not draw it otherwise (the striker's own next
    // strike draws it at once, the last one's end restores it).
    assert_eq!(layers(0), [6, 5, 4]);
    assert_eq!(layers(1), [5, 4, 5, 4]);
    // The Goblins below: dead, their death's last frame stays among the corpses; the Angels too.
    let last = t.at(t.end);
    assert_eq!(last[&2], layered(left(pose(5, 8, 70)), 1));
    assert_eq!(last[&3], layered(left(pose(5, 7, 54)), 1));
    // A dead stack takes no part later: strikes at or by it are skipped.
    let more = [
        struck(2, 0, 1, melee),
        struck(0, 2, 1, melee),
        moved(2, 70, 71),
    ];
    let mut commands = commands.to_vec();
    commands.push((None, more.to_vec()));
    let after = play(&o, &commands);
    assert_eq!(after.at(after.end)[&2], layered(left(pose(5, 8, 70)), 1));
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
    // Dead without a death group: it stands, among the corpses.
    assert_eq!(t.at(t.end)[&1], layered(left(still(2, 54)), 1));
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
        let shot = t.keys.iter().find(|k| (14..=16).contains(&k.pose.group));
        shot.map_or(0, |k| k.pose.group)
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
    assert_eq!(t.at(988)[&2], left(pose(3, 1, 49)));
}

/// Each key of `stack` as (group, frame, hex, mirrored, layer).
fn keys_of(t: &Timeline, stack: u8) -> Vec<(u32, u32, u8, bool, u8)> {
    t.keys
        .iter()
        .filter(|k| k.stack == stack)
        .map(|k| {
            let p = k.pose;
            (p.group, p.frame, p.hex, p.mirrored, p.layer)
        })
        .collect()
}

/// The groups `stack` shows in turn, each run once.
fn runs(t: &Timeline, stack: u8) -> Vec<u32> {
    let mut g: Vec<u32> = keys_of(t, stack).iter().map(|k| k.0).collect();
    g.dedup();
    g
}

#[test]
fn a_walk_faces_each_step_and_a_plain_move_turns_back() {
    // T4b-2a (§7.3; `walk_step` 0x43F0B0, `stack_walk` 0x4423A2, `stack_turn_around` 0x446720):
    // the defending Wolf Riders on 39 and 40 (position 40, facing left) walk one hex right. The
    // first step's turn is animated: group 9 at 40, the flip onto 39 (the position moves onto
    // the other hex), group 8, standing; the step sideways in layer 4; group 21 and standing on
    // 40 (the engine's 41, turned); a plain move then turns back: group 7, the flip onto 41,
    // group 10, standing.
    let o = snapshot(vec![
        at(0, Side::Attacker, PIKEMAN, 137, false),
        at(1, Side::Defender, WOLF_RIDER, 40, true),
    ]);
    let t = play(&o, &[(None, vec![moved(1, 40, 41)])]);
    let mut want = Vec::new();
    want.extend((0..3).map(|f| (9, f, 40, true, 4)));
    want.extend((0..3).map(|f| (8, f, 39, false, 4)));
    want.push((2, 0, 39, false, 4));
    want.push((20, 0, 39, false, 4));
    want.extend((0..6).map(|f| (0, f, 39, false, 4)));
    want.push((21, 0, 40, false, 4));
    want.push((2, 0, 40, false, 4));
    want.extend((0..3).map(|f| (7, f, 40, false, 4)));
    want.extend((0..3).map(|f| (10, f, 41, true, 4)));
    want.push((2, 0, 41, true, 4));
    assert_eq!(keys_of(&t, 1), want);
    // A later step the other way flips at once (0x43F0E5 without its flag): down and left from
    // 41 in layer 7, from the position facing left again; then left in layer 4, and no turn at
    // the end (it faces as it started). Up and left: layer 3.
    let t = play(
        &o,
        &[(
            None,
            vec![moved(1, 40, 41), moved(1, 41, 58), moved(1, 58, 57)],
        )],
    );
    let steps: Vec<(u32, u32, u8, bool, u8)> = keys_of(&t, 1)
        .into_iter()
        .filter(|k| k.0 == 0 && k.1 == 0)
        .collect();
    assert_eq!(
        steps,
        [
            (0, 0, 39, false, 4),
            (0, 0, 41, true, 7),
            (0, 0, 58, true, 4)
        ]
    );
    assert_eq!(runs(&t, 1), [9, 8, 2, 20, 0, 21, 2]);
    let t = play(&o, &[(None, vec![moved(1, 40, 23)])]);
    assert_eq!(runs(&t, 1), [20, 0, 21, 2]);
    assert!(keys_of(&t, 1).iter().any(|k| k.0 == 0 && k.4 == 3));
    // An attacker turned away stands one hex on (+ 1): the Champions on 40 and 41 walk left.
    let o = snapshot(vec![at(0, Side::Attacker, CHAMPION, 40, true)]);
    let t = play(&o, &[(None, vec![moved(0, 40, 39)])]);
    let k = keys_of(&t, 0);
    assert_eq!(k[0], (7, 0, 40, false, 4));
    assert_eq!(k[3], (10, 0, 41, true, 4));
    assert!(k.contains(&(21, 0, 40, true, 4)));
    assert_eq!(k.last(), Some(&(2, 0, 39, false, 4)));
}

#[test]
fn a_flight_turns_by_columns() {
    // A flight the other way by columns turns first (0x4B4A40), then flies and stands
    // (0x4B4E66); a plain move turns back (`stack_fly` 0x4B49F5) and stands once more
    // (0x4B4A15).
    let mut angels = at(0, Side::Attacker, ANGEL, 83, false);
    angels.flies = true;
    let o = snapshot(vec![angels]);
    let t = play(&o, &[(None, vec![moved(0, 83, 69)])]);
    assert_eq!(runs(&t, 0), [7, 10, 2, 20, 0, 21, 2, 9, 8, 2]);
    let k = keys_of(&t, 0);
    assert!(k.contains(&(21, 0, 69, true, 4)));
    let standing = k.iter().filter(|k| k.0 == 2).count();
    assert_eq!(standing, 4);
}

#[test]
fn a_melee_turns_both_stacks_and_back() {
    // The Angels fly to hex 83 (their columns grow: no turn) and strike the Wolf Riders below on
    // 99 and 100 (position 100). Neither has the other on its front side (0x445BCE, 0x4458B0):
    // the Angels turn (7, 10), the Wolf Riders turn (9, 8; onto 99); the strike down (13) in
    // the striker's layer 6, the retaliation up (11) with both in 5; the Wolf Riders turn back
    // (7, 10; onto 100), then the Angels (9, 8).
    let mut angels = at(0, Side::Attacker, ANGEL, 69, false);
    angels.flies = true;
    let o = snapshot(vec![angels, at(1, Side::Defender, WOLF_RIDER, 100, true)]);
    let attack = vec![
        moved(0, 69, 83),
        struck(0, 1, 1, flags(false, false, false)),
        struck(1, 0, 1, flags(false, true, false)),
        Event::TurnStarted { stack: 1 },
    ];
    let t = play(&o, &[(None, attack)]);
    assert_eq!(runs(&t, 0), [20, 0, 21, 2, 7, 10, 2, 13, 2, 3, 2, 9, 8, 2]);
    assert_eq!(runs(&t, 1), [9, 8, 2, 3, 2, 11, 2, 7, 10, 2]);
    let wolves = keys_of(&t, 1);
    assert_eq!(wolves[0], (9, 0, 100, true, 4));
    assert_eq!(wolves[3], (8, 0, 99, false, 4));
    assert!(wolves.contains(&(3, 0, 99, false, 5)));
    assert!(wolves.contains(&(11, 0, 99, false, 5)));
    assert!(wolves.contains(&(10, 0, 100, true, 4)));
    let angels = keys_of(&t, 0);
    assert!(angels.contains(&(13, 0, 83, true, 6)));
    assert!(angels.contains(&(3, 0, 83, true, 5)));
    assert_eq!(angels.last(), Some(&(2, 0, 83, false, 4)));
    // A strike from the front side turns no one: the same attack from hex 81 (up and left of
    // the Wolf Riders' front hex 99, where both face each other).
    let mut angels = at(0, Side::Attacker, ANGEL, 81, false);
    angels.flies = true;
    let o = snapshot(vec![angels, at(1, Side::Defender, WOLF_RIDER, 100, true)]);
    let t = play(
        &o,
        &[(None, vec![struck(0, 1, 1, flags(false, false, false))])],
    );
    assert_eq!(runs(&t, 0), [13, 2]);
    assert_eq!(runs(&t, 1), [2, 3, 2]);
    // A melee by or at a stack the timeline does not know turns no one: the first draws
    // nothing, the second a strike at nothing.
    let t = play(
        &o,
        &[
            (None, vec![struck(9, 1, 0, flags(false, false, false))]),
            (None, vec![struck(0, 9, 0, flags(false, false, false))]),
        ],
    );
    assert!(runs(&t, 1).is_empty());
    assert_eq!(runs(&t, 0), [12, 2]);
}

#[test]
fn a_shot_behind_turns_the_shooter_and_back() {
    // `shot_action` 0x440160: the Marksmen on hex 2 shoot the Goblins on 154, left of them:
    // they turn (7, 10), shoot down facing left, the arrow mirrored, and turn back (9, 8).
    let o = snapshot(vec![
        at(0, Side::Attacker, MARKSMAN, 2, false),
        at(1, Side::Defender, GOBLIN, 154, false),
    ]);
    let shot = struck(0, 1, 1, flags(true, false, false));
    let t = play(&o, &[(None, vec![shot.clone(), shot])]);
    assert_eq!(runs(&t, 0), [7, 10, 2, 16, 2, 16, 2, 9, 8, 2]);
    assert!(keys_of(&t, 0).contains(&(16, 0, 2, true, 4)));
    assert!(t.flights.iter().all(|f| f.mirrored));
    // A shooter the timeline does not know shoots nothing.
    let t = play(
        &o,
        &[(None, vec![struck(9, 1, 1, flags(true, false, false))])],
    );
    assert!(t.flights.is_empty());
}

#[test]
fn neighbours_follow_the_layout() {
    let layout = layout();
    let stage = Stage {
        layout: &layout,
        width: 17,
        groups: tables().battle.groups,
    };
    // Hex 83 (row 4, even, shifted right): up-right 67, right 84, down-right 101, down-left 100,
    // left 82, up-left 66; row 5 (odd): up-right of 100 is 83.
    let around: Vec<Option<u8>> = (0..6).map(|d| stage.neighbor(83, d)).collect();
    assert_eq!(
        around,
        [Some(67), Some(84), Some(101), Some(100), Some(82), Some(66)]
    );
    assert_eq!(stage.neighbor(100, 0), Some(83));
    // Off the field: right of column 16, above row 0, past the hexes a byte holds.
    assert_eq!(stage.neighbor(16, 1), None);
    assert_eq!(stage.neighbor(3, 5), None);
    assert_eq!(stage.neighbor(254, 2), None);
}

#[test]
fn a_timeline_lists_its_changes() {
    let o = snapshot(vec![
        at(0, Side::Attacker, MARKSMAN, 35, false),
        at(2, Side::Defender, GOBLIN, 49, false),
    ]);
    let t = play(
        &o,
        &[(None, vec![struck(0, 2, 3, flags(true, false, false))])],
    );
    let changes = t.changes(t.end + 250);
    // The opening, each frame shown, each arrow step (526 on, every 33 ms) and its erasure, the
    // end, the idle steps after it.
    assert_eq!(changes.first(), Some(&0));
    assert!(changes.contains(&526) && changes.contains(&559) && changes.contains(&988));
    assert!(changes.contains(&t.end) && changes.contains(&(t.end + 200)));
    assert!(changes.windows(2).all(|w| w[0] < w[1]));
    assert!(t.changes(10).iter().all(|c| *c <= 10));
    // What shows at a time, for the scene.
    let moment = crate::scene::Moment::at(&t, 600);
    assert_eq!(moment.poses, t.at(600));
    assert_eq!(
        (moment.active, moment.pulse, moment.projectile),
        (t.outlined(600), t.pulse(600), t.projectile(600))
    );
    // A record's commands: its `Events` messages.
    let events = vec![Event::Waited { stack: 0 }];
    let messages = [
        ServerMessage::Events {
            n: 2,
            events: events.clone(),
        },
        ServerMessage::GameEnded {
            n: 3,
            result: h3_protocol::wire::Outcome::Draw {},
        },
    ];
    let commands = commands(&messages);
    assert_eq!(commands.len(), 1);
    assert_eq!((commands[0].at, commands[0].events), (None, &events[..]));
}

/// The timeline of `snapshot` and `commands` at speed 0 with the viewer's fidgets from `seed`.
fn fidgeting_play(
    snapshot: &Observation,
    commands: &[(Option<u32>, Vec<Event>)],
    numbers: &BTreeMap<u16, Look>,
    seed: u32,
) -> Timeline {
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
    fidgeting(snapshot, &commands, numbers, stage, Speed::default(), seed).unwrap()
}

/// What `stack` shows every 100 ms up to `until`: (time, group, frame).
fn shown(t: &Timeline, stack: u8, until: u32) -> Vec<(u32, u32, u32)> {
    (0..=until / 100)
        .map(|i| {
            let p = t.at(i * 100)[&stack];
            (i * 100, p.group, p.frame)
        })
        .collect()
}

#[test]
fn stacks_fidget_from_the_viewers_own_generator() {
    // §7.2: the C library's `rand()` (srand(1): 41, 18467, 6334).
    let mut f = Fidgets {
        rng: 1,
        group: 1,
        stacks: Vec::new(),
    };
    assert_eq!([f.rand(), f.rand(), f.rand()], [41, 18467, 6334]);
    // The Pikemen (four frames of group 1, interval 9000): a fidget starts at an idle step after
    // the drawn 2r, its frames in order one step apart or stalled, then standing, the next one
    // 0.75 t to 1.25 t after.
    let mut numbers = looks();
    numbers.get_mut(&PIKEMAN).unwrap().frames.insert(1, 4);
    let o = snapshot(vec![at(0, Side::Attacker, PIKEMAN, 35, false)]);
    let t = fidgeting_play(&o, &[], &numbers, 1);
    let seen = shown(&t, 0, 60_000);
    let starts: Vec<u32> = seen
        .windows(2)
        .filter(|w| w[0].1 == 2 && w[1].1 == 1)
        .map(|w| w[1].0)
        .collect();
    assert!(starts.len() >= 3, "{starts:?}");
    assert!(starts[0] >= 100 && starts[0] <= 18_000);
    for pair in starts.windows(2) {
        assert!(
            (6_750..=11_500).contains(&(pair[1] - pair[0])),
            "{starts:?}"
        );
    }
    let mut frames: Vec<u32> = seen.iter().filter(|s| s.1 == 1).map(|s| s.2).collect();
    frames.dedup();
    assert!(frames.starts_with(&[0, 1, 2, 3, 0, 1, 2, 3]), "{frames:?}");
    // A stall: a draw below 8 holds the frame for a step.
    let stalled = (0u32..)
        .find(|&rng| {
            let mut f = Fidgets {
                rng,
                group: 1,
                stacks: Vec::new(),
            };
            f.between(0, 100) < 8
        })
        .unwrap();
    let mut f = Fidgets {
        rng: stalled,
        group: 1,
        stacks: vec![Fidget {
            stack: 0,
            interval: 9000,
            frames: 4,
            clock: 0,
            frame: Some(1),
            base: still(2, 35),
        }],
    };
    assert!(f.step(500).is_empty());
    assert_eq!(f.stacks[0].frame, Some(1));
    // Without fidgets (the reference checks), it stands.
    assert!(shown(&play(&o, &[]), 0, 60_000).iter().all(|s| s.1 == 2));
    // A fidget during the battle: the Pikemen fidget while the Goblins wait to act; a command
    // that draws them stops it, one that draws others leaves it running.
    let o = snapshot(vec![
        at(0, Side::Attacker, PIKEMAN, 35, false),
        at(1, Side::Defender, GOBLIN, 49, false),
    ]);
    let mid = seen
        .iter()
        .find(|s| s.1 == 1 && s.2 == 1)
        .map(|s| s.0)
        .unwrap();
    let them = vec![moved(1, 49, 48)];
    let t = fidgeting_play(&o, &[(Some(mid), them)], &numbers, 1);
    let during = t.at(mid + 500)[&0];
    assert_eq!((during.group, during.frame), (1, 1));
    let after = t.at(t.end)[&0];
    assert_eq!((after.group, after.frame > 1), (1, true));
    let own = vec![moved(0, 35, 36)];
    let t = fidgeting_play(&o, &[(Some(mid), own)], &numbers, 1);
    let after = t.at(t.end + 100)[&0];
    assert_eq!(after.group, 2);
    // A stack the battle killed fidgets no more; one with an interval of 51 or less keeps its
    // clock and starts at the first idle step after it.
    numbers.get_mut(&GOBLIN).unwrap().frames.insert(1, 2);
    numbers.get_mut(&GOBLIN).unwrap().times[0] = 0;
    let kill = vec![struck(0, 1, 40, flags(false, false, false))];
    let t = fidgeting_play(&o, &[(Some(mid), kill)], &numbers, 1);
    assert!(
        shown(&t, 1, 60_000)
            .iter()
            .filter(|s| s.0 > t.end)
            .all(|s| s.1 == 5)
    );
    let t = fidgeting_play(&o, &[], &numbers, 1);
    assert_eq!(t.at(100)[&1].group, 1);
}

/// When `stack` first draws frame 0 of `group`.
fn first(t: &Timeline, stack: u8, group: u32) -> u32 {
    t.keys
        .iter()
        .find(|k| k.stack == stack && k.pose.group == group && k.pose.frame == 0)
        .map(|k| k.drawn)
        .unwrap()
}

/// The cues as (time, stack, sound, stop).
fn cues(t: &Timeline) -> Vec<(u32, u8, Sound, bool)> {
    t.cues
        .iter()
        .map(|c| (c.at, c.stack, c.sound, c.stop))
        .collect()
}

#[test]
fn sounds_start_with_their_groups() {
    // E17 M2 T5b (design §8.2): a flight's start with group 20, its move (looping) with the first
    // cycle, its end with group 21 when the move stops (0x4B4BD4, 0x4B4E3B); the striker's attack
    // with its strike, each struck stack's sound with its reaction (0x468F00).
    let mut angels = at(0, Side::Attacker, ANGEL, 69, false);
    angels.flies = true;
    let o = snapshot(vec![angels, at(1, Side::Defender, WOLF_RIDER, 100, true)]);
    let attack = vec![
        moved(0, 69, 83),
        struck(0, 1, 1, flags(false, false, false)),
        struck(1, 0, 1, flags(false, true, false)),
    ];
    let t = play(&o, &[(None, attack)]);
    let end = first(&t, 0, 21);
    assert_eq!(
        cues(&t),
        [
            (first(&t, 0, 20), 0, Sound::Start, false),
            (first(&t, 0, 0), 0, Sound::Move, false),
            (end, 0, Sound::End, false),
            (end, 0, Sound::Move, true),
            (first(&t, 0, 13), 0, Sound::Attack, false),
            (first(&t, 1, 3), 1, Sound::Wince, false),
            (first(&t, 1, 11), 1, Sound::Attack, false),
            (first(&t, 0, 3), 0, Sound::Wince, false),
        ]
    );
    assert_eq!(t.cue_from(0), 0);
    assert_eq!(t.cue_from(end), 2);
    assert_eq!(t.cue_from(t.end + 1), t.cues.len());
    // A walk: the start and the move with the first step (0x43F19B), the end at the last
    // (0x43F267); the Wolf Riders' turn before them makes none.
    let o = snapshot(vec![
        at(0, Side::Attacker, PIKEMAN, 137, false),
        at(1, Side::Defender, WOLF_RIDER, 40, true),
    ]);
    let t = play(&o, &[(None, vec![moved(1, 40, 41), moved(1, 41, 58)])]);
    let end = first(&t, 1, 21);
    assert_eq!(
        cues(&t),
        [
            (first(&t, 1, 20), 1, Sound::Start, false),
            (first(&t, 1, 0), 1, Sound::Move, false),
            (end, 1, Sound::End, false),
            (end, 1, Sound::Move, true),
        ]
    );
    // A shot (0x43F424) at Goblins that die; then the Wolf Riders strike defending Goblins.
    let o = snapshot(vec![
        at(0, Side::Attacker, MARKSMAN, 1, false),
        at(1, Side::Defender, GOBLIN, 49, false),
        at(2, Side::Defender, GOBLIN, 37, false),
        at(3, Side::Attacker, WOLF_RIDER, 36, true),
    ]);
    let t = play(
        &o,
        &[
            (None, vec![struck(0, 1, 40, flags(true, false, false))]),
            (
                None,
                vec![
                    Event::Defended { stack: 2 },
                    struck(3, 2, 1, flags(false, false, false)),
                ],
            ),
        ],
    );
    let shot = t.keys.iter().find(|k| k.stack == 0).unwrap();
    assert_eq!(
        cues(&t),
        [
            (shot.drawn, 0, Sound::Shot, false),
            (first(&t, 1, 5), 1, Sound::Death, false),
            (first(&t, 3, 12), 3, Sound::Attack, false),
            (first(&t, 2, 4), 2, Sound::Defend, false),
        ]
    );
}
