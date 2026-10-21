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
const CLIMB: &str = include_str!("../../../../situations/timelines/climb.txt");
const BEHIND: &str = include_str!("../../../../situations/timelines/behind.txt");
const SHOTS: &str = include_str!("../../../../situations/timelines/shots.txt");
const FLY: &str = include_str!("../../../../situations/timelines/fly.txt");
const CAST: &str = include_str!("../../../../situations/timelines/cast.txt");
const RESIST: &str = include_str!("../../../../situations/timelines/resist.txt");
const ARROW: &str = include_str!("../../../../situations/timelines/arrow.txt");
const HASTE: &str = include_str!("../../../../situations/timelines/haste.txt");
const DISPEL: &str = include_str!("../../../../situations/timelines/dispel.txt");

/// Creature handles of the tests.
const CHAMPION: u16 = 0;
const MARKSMAN: u16 = 1;
const WOLF_RIDER: u16 = 2;
const GOBLIN: u16 = 3;
const ANGEL: u16 = 4;
const PIKEMAN: u16 = 5;
const GNOLL: u16 = 6;
const CENTAUR_CAPTAIN: u16 = 7;
const SILVER_PEGASUS: u16 = 8;
const CYCLOPS: u16 = 9;
const ARCH_MAGE: u16 = 10;
const BEHOLDER: u16 = 11;
const BLACK_DRAGON: u16 = 12;
/// The First Aid Tent (E17 M3 T3b).
const TENT: u16 = 13;
/// The casts' creatures (E17 M3 T7b-1).
const SWORDSMAN: u16 = 14;
const ARCHANGEL: u16 = 15;
const BATTLE_DWARF: u16 = 16;
const VAMPIRE_LORD: u16 = 17;

/// A look: `times` and the frames of each group.
fn look(times: [u32; 4], frames: &[(u32, u32)], top: i32) -> Look {
    Look {
        frames: frames.iter().copied().collect(),
        times,
        climax: 0,
        missile_offsets: [[0, 0]; 3],
        top,
        front: 0,
        missile: None,
        ray: None,
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
        (
            PIKEMAN,
            look(
                [9000, 575, 500, 115],
                &[(0, 6), (2, 7), (3, 6), (11, 10), (12, 10), (13, 11)],
                171,
            ),
        ),
        (BLACK_DRAGON, black_dragon()),
        (
            GNOLL,
            look(
                [9000, 500, 500, 115],
                &[
                    (0, 8),
                    (2, 8),
                    (3, 6),
                    (4, 11),
                    (5, 6),
                    (7, 2),
                    (8, 2),
                    (9, 2),
                    (10, 2),
                ],
                192,
            ),
        ),
        (CYCLOPS, cyclops()),
        (ARCH_MAGE, arch_mage()),
        (BEHOLDER, beholder()),
        (
            CENTAUR_CAPTAIN,
            look(
                [9000, 250, 500, 115],
                &[
                    (0, 8),
                    (2, 9),
                    (3, 6),
                    (4, 11),
                    (5, 8),
                    (7, 3),
                    (8, 3),
                    (9, 3),
                    (10, 3),
                    (11, 8),
                    (12, 8),
                    (13, 8),
                    (20, 2),
                    (21, 2),
                ],
                153,
            ),
        ),
        (
            SILVER_PEGASUS,
            look(
                [9000, 360, 500, 115],
                &[
                    (0, 6),
                    (2, 8),
                    (3, 6),
                    (4, 11),
                    (5, 8),
                    (7, 3),
                    (8, 3),
                    (9, 3),
                    (10, 3),
                    (11, 8),
                    (12, 8),
                    (13, 8),
                    (20, 2),
                    (21, 2),
                ],
                179,
            ),
        ),
        // `SMTENT.DEF` (E17 M3 T3b): standing, hit, defend, death only; no fidget, no move.
        (
            TENT,
            look(
                [9000, 500, 500, 115],
                &[(2, 6), (3, 9), (4, 9), (5, 6)],
                144,
            ),
        ),
        // `CSWORD.DEF`, `CRANGL.DEF`, `CBDWAR.DEF` (E17 M3 T7b-1), with their standing frames'
        // fronts.
        (
            SWORDSMAN,
            Look {
                front: 221,
                ..look(
                    [9000, 435, 500, 115],
                    &[
                        (0, 8),
                        (1, 8),
                        (2, 8),
                        (3, 6),
                        (4, 11),
                        (5, 6),
                        (7, 2),
                        (8, 2),
                        (9, 2),
                        (10, 2),
                        (11, 7),
                        (12, 7),
                        (13, 7),
                        (20, 2),
                        (21, 2),
                    ],
                    182,
                )
            },
        ),
        (
            ARCHANGEL,
            Look {
                front: 254,
                ..look(
                    [9000, 410, 500, 115],
                    &[
                        (0, 7),
                        (1, 7),
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
                    164,
                )
            },
        ),
        (
            BATTLE_DWARF,
            Look {
                front: 215,
                ..look(
                    [9000, 485, 500, 115],
                    &[
                        (0, 8),
                        (1, 7),
                        (2, 8),
                        (3, 6),
                        (4, 11),
                        (5, 6),
                        (7, 2),
                        (8, 2),
                        (9, 2),
                        (10, 2),
                        (11, 8),
                        (12, 8),
                        (13, 8),
                        (20, 2),
                        (21, 2),
                    ],
                    196,
                )
            },
        ),
        (
            VAMPIRE_LORD,
            look(
                [9000, 500, 500, 115],
                &[
                    (0, 6),
                    (1, 9),
                    (2, 8),
                    (3, 7),
                    (4, 9),
                    (5, 8),
                    (20, 5),
                    (21, 5),
                ],
                160,
            ),
        ),
    ];
    base.into_iter().collect()
}

/// The magic animations of the tests (E17 M3 T3b): the heal's, three frames of a 96 x 94 DEF,
/// centred on the creature and translucent.
pub(crate) fn magic() -> BTreeMap<u8, MagicLook> {
    [(
        HEAL,
        MagicLook {
            def: "Heal.def".to_string(),
            frames: 3,
            size: [96, 94],
            place: 1,
            translucent: true,
        },
    )]
    .into_iter()
    .collect()
}

/// The Black Dragons (`CBDRGN.DEF`): a breath on two stacks takes groups 17 to 19 (E17 M3 T1b-2).
fn black_dragon() -> Look {
    look(
        [9000, 300, 500, 115],
        &[
            (0, 4),
            (2, 8),
            (3, 7),
            (4, 9),
            (5, 10),
            (7, 3),
            (8, 3),
            (9, 3),
            (10, 3),
            (11, 10),
            (12, 8),
            (13, 11),
            (17, 10),
            (18, 8),
            (19, 11),
            (20, 4),
            (21, 6),
        ],
        120,
    )
}

/// The Cyclops (`CCYCLR.DEF`, its projectile `PCYCLBX.DEF`: 33 x 33, ten frames; numbers only).
fn cyclops() -> Look {
    let mut l = look(
        [9000, 500, 500, 115],
        &[
            (0, 8),
            (2, 6),
            (3, 3),
            (4, 4),
            (5, 5),
            (7, 3),
            (8, 3),
            (9, 3),
            (10, 3),
            (14, 16),
            (15, 17),
            (16, 17),
        ],
        164,
    );
    l.climax = 7;
    l.missile_offsets = [[55, -115], [53, -84], [40, -59]];
    l.missile = Some(Missile {
        def: "PCYCLBX.DEF".to_string(),
        size: [33, 33],
        angles: vec![0; 10],
    });
    l
}

/// The Arch Magi (`CAMAGE.DEF`): a ray shooter (E17 M3 T1b-1).
fn arch_mage() -> Look {
    let mut l = look(
        [9000, 500, 500, 115],
        &[
            (0, 8),
            (2, 8),
            (3, 6),
            (4, 11),
            (5, 8),
            (7, 2),
            (8, 2),
            (9, 2),
            (10, 2),
            (14, 13),
            (15, 13),
            (16, 13),
        ],
        166,
    );
    l.climax = 9;
    l.missile_offsets = [[28, -107], [43, -99], [18, -92]];
    l.missile = Some(Missile {
        def: "PMAGEX.DEF".to_string(),
        size: [30, 30],
        angles: vec![90, 72, 45, 27, 0, -27, -45, -72, -90],
    });
    l.ray = Some(vec![
        [152, 188, 24],
        [124, 216, 124],
        [36, 180, 36],
        [12, 132, 12],
        [0, 96, 0],
    ]);
    l
}

/// The Beholders (`CBEHOL.DEF`): a ray shooter.
fn beholder() -> Look {
    let mut l = look(
        [9000, 500, 500, 115],
        &[
            (0, 11),
            (2, 7),
            (3, 6),
            (4, 11),
            (5, 5),
            (7, 2),
            (8, 2),
            (9, 2),
            (10, 2),
            (14, 8),
            (15, 7),
            (16, 7),
        ],
        180,
    );
    l.climax = 5;
    l.missile_offsets = [[14, -73], [14, -70], [9, -60]];
    l.missile = Some(Missile {
        def: "PMAGEX.DEF".to_string(),
        size: [30, 30],
        angles: vec![0; 9],
    });
    l.ray = Some(vec![
        [192, 192, 192],
        [208, 208, 208],
        [224, 224, 224],
        [240, 240, 240],
        [255, 255, 255],
    ]);
    l
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
        magic: &magic(),
        casts: &Casts::default(),
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
    // Its count box hidden through the walk (E17 M3 T5a).
    assert_eq!(
        t.at(100)[&0],
        Pose {
            boxless: true,
            ..pose(20, 0, 35)
        }
    );
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
    // The hit's first frame never shows: at 988 the second does, the shooter standing; the
    // struck in layer 5 as in a melee (E17 M3 T5b-2a).
    assert_eq!(t.at(988)[&2], layered(left(pose(3, 1, 49)), STRUCK));
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
    // E17 M3 T6b: the first shot's line after its reactions, as the original wrote it.
    assert_eq!(strikes(&t)[0], 1588);
}

/// When the timeline's strike lines are written (E17 M3 T6b).
fn strikes(t: &Timeline) -> Vec<u32> {
    t.log
        .iter()
        .filter(|(_, l)| matches!(l, Logged::Strike { .. }))
        .map(|(at, _)| *at)
        .collect()
}

#[test]
fn the_melee_sequence_plays_as_recorded() {
    // `seq-melee.toml`: the Angels fly to hex 43; the computer's Wolf Riders walk to 45 and
    // strike them, the Angels strike back (to its end since T5b: 7 000 ms). The second command
    // starts by default 100 ms after the first's last frame shows.
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
    assert_eq!(until(&t.lines(), 7000), MELEE);
    // T4b-1 (§7.9, the reference frames): the flight's point moves on before each frame by
    // f32 (352 / 3) / 7, truncated: the last of 21 frames is 351 pixels on, not 352.
    let offsets: Vec<[i32; 2]> = [300, 358, 1402, 1460].map(|t2| t.at(t2)[&0].offset).into();
    assert_eq!(offsets, [[16, 0], [33, 0], [335, 0], [351, 0]]);
    // The computer's Wolf Riders: outlined with its idle step, until their first frame shows.
    let outlined: Vec<Option<u8>> = [1818, 1918, 2017, 2018].map(|t2| t.outlined(t2)).into();
    assert_eq!(outlined, [None, Some(2), Some(2), None]);
    assert_eq!((t.pulse(1917), t.pulse(1918), t.pulse(5000)), (1, 2, 2));
    // E17 M3 T6b: each strike's line after its animation, as the original wrote them.
    assert_eq!(strikes(&t), [5266, 6466]);
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
fn the_climb_sequence_plays_as_recorded() {
    // `climb` (M3 T1a, design §7.10): the turn sequence's armies; the Champions walk two rows up,
    // from 35 to 1 by 19: the first step up and right (no turn), the second up and left with a
    // flip only (onto 20), group 21 on 2, and the plain move's turn back (9, the flip onto 1, 8).
    let o = snapshot(vec![
        at(0, Side::Attacker, CHAMPION, 35, true),
        at(1, Side::Attacker, MARKSMAN, 137, false),
        at(2, Side::Defender, WOLF_RIDER, 49, true),
        at(3, Side::Defender, GOBLIN, 151, false),
    ]);
    let walk = vec![
        moved(0, 35, 19),
        moved(0, 19, 1),
        Event::TurnStarted { stack: 1 },
    ];
    let t = play(&o, &[(None, walk)]);
    assert_eq!(until(&t.lines(), 3500), CLIMB);
}

#[test]
fn the_behind_sequence_plays_as_recorded() {
    // `behind` (M3 T1a, design §7.10): a battle in progress in hot seat, every command at the
    // time its stack's turn started. The Pegasi wait, the Centaurs walk to 41 and the Marksmen to
    // 143, the Goblins defend, the Pegasi land behind the Centaurs; in round 2 the Pegasi wait,
    // the Centaurs strike them standing (both turn, then turn back) and the Marksmen shoot them
    // (they lie left of the Marksmen: a turn, two shots, the turn back).
    let mut pegasi = at(2, Side::Defender, SILVER_PEGASUS, 49, true);
    pegasi.flies = true;
    let o = snapshot(vec![
        at(0, Side::Attacker, CENTAUR_CAPTAIN, 35, true),
        at(1, Side::Attacker, MARKSMAN, 137, false),
        pegasi,
        at(3, Side::Defender, GOBLIN, 151, false),
    ]);
    let walk = |stack: u8, from: u8, next: u8| {
        let mut events: Vec<Event> = (from..from + 6).map(|h| moved(stack, h, h + 1)).collect();
        events.push(Event::TurnStarted { stack: next });
        events
    };
    let shot = flags(true, false, false);
    let commands = [
        (
            Some(0),
            vec![Event::Waited { stack: 2 }, Event::TurnStarted { stack: 0 }],
        ),
        (Some(0), walk(0, 35, 1)),
        (Some(1988), walk(1, 137, 3)),
        (
            Some(5464),
            vec![
                Event::Defended { stack: 3 },
                Event::TurnStarted { stack: 2 },
            ],
        ),
        (
            Some(5464),
            vec![
                moved(2, 49, 40),
                Event::RoundStarted { round: 2 },
                Event::TurnStarted { stack: 2 },
            ],
        ),
        (
            Some(7144),
            vec![Event::Waited { stack: 2 }, Event::TurnStarted { stack: 0 }],
        ),
        (
            Some(7144),
            vec![
                struck(0, 2, 1, flags(false, false, false)),
                struck(2, 0, 8, flags(false, true, false)),
                Event::TurnStarted { stack: 1 },
            ],
        ),
        (
            Some(12544),
            vec![
                struck(1, 2, 1, shot),
                struck(1, 2, 1, shot),
                Event::TurnStarted { stack: 3 },
            ],
        ),
    ];
    let t = play(&o, &commands);
    // To the second shot's end and the Marksmen's turn back (E17 M3 T5b-2a: 4 000 ms).
    assert_eq!(until(&t.lines(), 16544), BEHIND);
}

#[test]
fn the_shots_sequence_plays_as_recorded() {
    // `shots` (M3 T1b-1, design §7.12): a battle in progress in hot seat. Round 1: the Arch
    // Magi's ray at the Beholders, the Cyclops' throw down at the Gnolls (the Cyclops plays its
    // group out after the Gnolls' hit), the Beholders' ray at the Cyclops, the Gnolls defend;
    // round 2: the Arch Magi's ray at the defending Gnolls, the Cyclops' throw kills the
    // Beholders.
    let mut o = snapshot(vec![
        at(0, Side::Attacker, CYCLOPS, 35, false),
        at(1, Side::Attacker, ARCH_MAGE, 137, false),
        at(2, Side::Defender, BEHOLDER, 49, false),
        at(3, Side::Defender, GNOLL, 151, false),
    ]);
    o.active = Some(1);
    let shot = flags(true, false, false);
    let commands = [
        (
            None,
            vec![struck(1, 2, 1, shot), Event::TurnStarted { stack: 0 }],
        ),
        (
            Some(1517),
            vec![struck(0, 3, 6, shot), Event::TurnStarted { stack: 2 }],
        ),
        (
            Some(3471),
            vec![struck(2, 0, 0, shot), Event::TurnStarted { stack: 3 }],
        ),
        (
            Some(4638),
            vec![
                Event::Defended { stack: 3 },
                Event::RoundStarted { round: 2 },
                Event::TurnStarted { stack: 1 },
            ],
        ),
        (
            Some(4638),
            vec![struck(1, 3, 4, shot), Event::TurnStarted { stack: 0 }],
        ),
        (
            Some(6655),
            vec![struck(0, 2, 40, shot), Event::TurnStarted { stack: 3 }],
        ),
    ];
    let t = play(&o, &commands);
    assert_eq!(until(&t.lines(), 9655), SHOTS);
    // Three rays of 11 ticks, each from trunc(115 F) after the climax frame shows; the last
    // tick shows for no time.
    let rays: Vec<(u32, usize)> = t.beams.iter().map(|b| (b.from, b.ticks.len())).collect();
    assert_eq!(rays, [(655, 11), (4086, 11), (5293, 11)]);
    // E17 M3 T6c-2: the cursor hidden from the ray's start until the shot group's rest is
    // over (the original's screen: back at 879).
    assert_eq!(t.beams[0].cursor_back, 879);
    let hidden = [654, 655, 878, 879].map(|ms| t.cursor_hidden(ms));
    assert_eq!(hidden, [false, true, true, false]);
    assert!(t.ray(664).is_none());
    let first = t.ray(665).unwrap();
    assert_eq!(first.colours.len(), 5);
    assert!(t.ray(755).unwrap().strokes.len() > first.strokes.len());
    assert!(t.ray(765).is_none());
    let changes = t.changes(800);
    assert!((665..=765).step_by(10).all(|ms| changes.contains(&ms)));
    // After the ray the shot group's rest: frame 9 at once, 10 and 11 on the deadline; 12 drawn
    // and never shown.
    let shown = t.at(841)[&1];
    assert_eq!((shown.group, shown.frame), (15, 11));
    assert_eq!(t.at(879)[&1].frame, 11);
    assert!(
        t.keys
            .iter()
            .any(|k| k.stack == 1 && k.pose.frame == 12 && k.shown == u32::MAX)
    );
}

#[test]
fn the_fly_sequence_plays_as_recorded() {
    // `fly` (M3 T1b-2, design §7.10, §7.4): a battle in progress in hot seat. The Black Dragons
    // fly from 49 to 54 (row 2 to row 3) and breathe down and to the left through the Pikemen on
    // 69 into the Gnolls on 86 (groups 19, not 13); the Pikemen strike back. Then the Angels fly
    // from 1 to 155 (row 0 to row 9) over the Dragons and five stacks. The other stacks stand.
    let mut angels = at(0, Side::Attacker, ANGEL, 1, true);
    angels.flies = true;
    let mut dragons = at(7, Side::Defender, BLACK_DRAGON, 49, true);
    dragons.flies = true;
    let mut o = snapshot(vec![
        angels,
        at(1, Side::Attacker, PIKEMAN, 35, false),
        at(2, Side::Attacker, PIKEMAN, 69, false),
        at(3, Side::Attacker, GNOLL, 86, false),
        at(4, Side::Attacker, PIKEMAN, 103, false),
        at(5, Side::Attacker, MARKSMAN, 137, false),
        at(6, Side::Attacker, PIKEMAN, 171, false),
        dragons,
        at(8, Side::Defender, GOBLIN, 151, false),
    ]);
    o.active = Some(7);
    let commands = [
        (
            None,
            vec![
                moved(7, 49, 54),
                struck(7, 2, 8, flags(false, false, false)),
                struck(7, 3, 13, flags(false, false, true)),
                struck(2, 7, 0, flags(false, true, false)),
                Event::TurnStarted { stack: 0 },
            ],
        ),
        (
            Some(5900),
            vec![moved(0, 1, 155), Event::TurnStarted { stack: 5 }],
        ),
    ];
    let t = play(&o, &commands);
    assert_eq!(until(&t.lines(), 8900), FLY);
    // Both flights' cycles are drawn over every row (anim::FLYING): the recorded frames match.
    for ms in [700, 1700, 6500, 7200] {
        let flier = if ms < 2000 { 7 } else { 0 };
        let pose = t.at(ms)[&flier];
        assert_eq!((pose.group, pose.layer), (0, FLYING), "{ms}");
    }
    assert_eq!(t.at(3000)[&7].group, 19);
}

#[test]
fn a_creature_without_numbers_is_an_error() {
    let mut o = snapshot(vec![at(0, Side::Attacker, 99, 35, false)]);
    let layout = layout();
    let stage = Stage {
        layout: &layout,
        width: 0,
        groups: tables().battle.groups,
        magic: &magic(),
        casts: &Casts::default(),
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
        magic: &magic(),
        casts: &Casts::default(),
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
    // each back in 4 at the strike's end (T5b: `strike_animation` sets every stack's +0x8 to 4
    // and redraws, 0x46934D, 0x46938A), before a later strike of the command takes its layer.
    assert_eq!(layers(0), [6, 4, 6, 4, 6, 4, 5, 4]);
    assert_eq!(layers(1), [5, 4, 5, 4, 5, 4]);
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
        magic: &magic(),
        casts: &Casts::default(),
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
fn a_ray_without_a_climax_frame_or_a_target() {
    // A ray shooter without a climax frame plays its whole group, then the ray, and no rest of
    // the group (0x5A6130); a shot at no known stack draws no ray.
    let mut numbers = looks();
    numbers.get_mut(&ARCH_MAGE).unwrap().climax = 0;
    let layout = layout();
    let stage = Stage {
        layout: &layout,
        width: 17,
        groups: tables().battle.groups,
        magic: &magic(),
        casts: &Casts::default(),
    };
    let o = snapshot(vec![
        at(0, Side::Attacker, ARCH_MAGE, 137, false),
        at(1, Side::Defender, GOBLIN, 49, false),
    ]);
    let shot = flags(true, false, false);
    let events = [struck(0, 1, 1, shot), struck(0, 9, 1, shot)];
    let commands = [Command {
        at: None,
        events: &events,
    }];
    let t = timeline(&o, &commands, &numbers, stage, Speed::default()).unwrap();
    assert_eq!(t.beams.len(), 1);
    // 13 frames of 38 ms, the first shown at 100, the last at 556; the ray 115 ms on; none
    // hidden.
    assert_eq!(t.beams[0].from, 556 + 115);
    assert!(t.keys.iter().all(|k| k.shown != u32::MAX));
    // Without a climax frame the cursor shows again as the ray's last tick is over.
    let b = &t.beams[0];
    let ticks = u32::try_from(b.ticks.len()).unwrap();
    assert_eq!(b.cursor_back, b.from + ticks * b.step);
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
        magic: &magic(),
        casts: &Casts::default(),
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
        magic: &magic(),
        casts: &Casts::default(),
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
    // At no known stack: straight, no projectile; by no known stack: its target reacts alone
    // (T5c).
    let t = run(&numbers, &[struck(0, 9, 1, flags(true, false, false))]);
    assert!(t.keys.iter().all(|k| k.stack == 0));
    let t = run(&numbers, &[struck(9, 1, 1, flags(true, false, false))]);
    assert!(!t.keys.is_empty() && t.keys.iter().all(|k| k.stack == 1));
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
        magic: &magic(),
        casts: &Casts::default(),
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
    assert_eq!(t.at(988)[&2], layered(left(pose(3, 1, 49)), STRUCK));
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
    // The box shown again: the whole battle redrawn (E17 M3 T5a).
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

/// `stack`'s keys as (group, whether its box is hidden, when it shows).
fn boxless(t: &Timeline, stack: u8) -> Vec<(u32, bool, u32)> {
    t.keys
        .iter()
        .filter(|k| k.stack == stack)
        .map(|k| (k.pose.group, k.pose.boxless, k.shown))
        .collect()
}

#[test]
fn a_walk_and_a_flight_hide_their_count_box() {
    // E17 M3 T5a (design §4.3): `stack_walk` sets +0x30 before its first turn (0x442145) and
    // clears it after the turn back, redrawing the whole battle at once (0x4423D9, 0x4423E7).
    // The Wolf Riders turn, walk, stand, turn back and stand: every frame hidden, then the
    // redraw shows the box with the last frame.
    let o = snapshot(vec![
        at(0, Side::Attacker, PIKEMAN, 137, false),
        at(1, Side::Defender, WOLF_RIDER, 40, true),
    ]);
    let t = play(&o, &[(None, vec![moved(1, 40, 41)])]);
    let k = boxless(&t, 1);
    let (last, frames) = k.split_last().unwrap();
    assert!(frames.iter().all(|&(_, hidden, _)| hidden));
    assert_eq!(*last, (2, false, frames.last().unwrap().2));
    assert!(t.at(t.end)[&1].holds_hex());
    // A flight (0x4B4BD8, 0x4B4E73): its turn and group 20 shown, hidden from the cycles to the
    // standing frame after group 21; the turn back and the last standing frame shown.
    let mut angels = at(0, Side::Attacker, ANGEL, 83, false);
    angels.flies = true;
    let t = play(&snapshot(vec![angels]), &[(None, vec![moved(0, 83, 69)])]);
    let mut hidden: Vec<(u32, bool)> = boxless(&t, 0).iter().map(|k| (k.0, k.1)).collect();
    hidden.dedup();
    assert_eq!(
        hidden,
        [
            (7, false),
            (10, false),
            (2, false),
            (20, false),
            (0, true),
            (21, true),
            (2, true),
            (9, false),
            (8, false),
            (2, false)
        ]
    );
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
    // The Wolf Riders' turn starts after both turned back, and their outline with it (E17 M2
    // T5c, the reviewer: §7.9, none while a command plays).
    assert_eq!(t.turns, [(t.end, 1)]);
    assert_eq!(t.outlined(t.end - 1), None);
    assert_eq!(t.outlined(t.end), Some(1));
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
    // A melee by or at a stack the timeline does not know turns no one: the first's target
    // reacts alone (T5c: a created stack's strike), the second is a strike at nothing.
    let t = play(
        &o,
        &[
            (None, vec![struck(9, 1, 0, flags(false, false, false))]),
            (None, vec![struck(0, 9, 0, flags(false, false, false))]),
        ],
    );
    assert_eq!(runs(&t, 1), [3, 2]);
    assert!(t.cues.iter().all(|c| c.stack != 9));
    assert_eq!(runs(&t, 0), [12, 2]);
    // Killed by it, the target dies and lies among the corpses (the reviewer: it stood on).
    let t = play(
        &o,
        &[(None, vec![struck(9, 1, 40, flags(false, false, false))])],
    );
    assert_eq!(runs(&t, 1), [5]);
    assert_eq!(t.at(t.end)[&1].layer, CORPSE);
}

#[test]
fn a_breath_on_two_stacks_takes_its_own_groups() {
    // `melee_strike` 0x441ACA to 0x441B66: a breath that strikes a second stack, behind its
    // target, takes groups 17 to 19 where the DEF has them. The Angels (given a breath group 18)
    // on 52 strike the Goblins on 53 and breathe on the Pikemen behind them on 54.
    let mut numbers = looks();
    numbers.get_mut(&ANGEL).unwrap().frames.insert(18, 9);
    let layout = layout();
    let stage = Stage {
        layout: &layout,
        width: 17,
        groups: tables().battle.groups,
        magic: &magic(),
        casts: &Casts::default(),
    };
    let o = snapshot(vec![
        at(0, Side::Attacker, ANGEL, 52, false),
        at(1, Side::Defender, GOBLIN, 53, false),
        at(2, Side::Defender, PIKEMAN, 54, false),
        at(3, Side::Defender, GOBLIN, 51, false),
    ]);
    let groups = |events: &[Event], numbers: &BTreeMap<u16, Look>| {
        let commands = [Command { at: None, events }];
        let t = timeline(&o, &commands, numbers, stage, Speed::default()).unwrap();
        runs(&t, 0)
    };
    let main = struck(0, 1, 1, flags(false, false, false));
    let beside = flags(false, false, true);
    let breath = [main.clone(), struck(0, 2, 1, beside)];
    assert_eq!(groups(&breath, &numbers), [18, 2]);
    // A second stack beside the striker (a multi-headed strike, here on 51) keeps the strike's
    // group, as does one strike alone, and a breath where the DEF has no group 18.
    let heads = [main.clone(), struck(0, 3, 1, beside)];
    assert_eq!(groups(&heads, &numbers), [12, 2]);
    assert_eq!(groups(&[main], &numbers), [12, 2]);
    assert_eq!(groups(&breath, &looks()), [12, 2]);
}

#[test]
fn layer_changes_show_at_the_strikes_end() {
    // The reviewer (E17 M2 T5c): a command's last layer changes waited for the next command's
    // first frame, so a corpse lay in the struck stacks' pass through a player's wait, and a
    // survivor's standing pose came back with another stack's walk. Both showed with the first
    // idle step after the strike then; since T5b at the strike's end, whose whole redraw shows
    // them at once (`strike_animation` 0x46938A).
    let o = snapshot(vec![
        at(0, Side::Attacker, WOLF_RIDER, 52, true),
        at(1, Side::Defender, GOBLIN, 54, false),
        at(2, Side::Defender, GOBLIN, 70, false),
        at(3, Side::Attacker, PIKEMAN, 35, false),
    ]);
    let strike = vec![
        struck(0, 1, 1, flags(false, false, false)),
        struck(0, 2, 40, flags(false, false, true)),
    ];
    let walk = vec![moved(3, 35, 36)];
    let t = play(&o, &[(Some(0), strike), (Some(9000), walk)]);
    assert!(
        t.keys
            .iter()
            .filter(|k| k.stack == 1)
            .all(|k| k.drawn < 9000)
    );
    let corpse = t.keys.iter().rfind(|k| k.stack == 2).unwrap();
    assert_eq!(corpse.pose.layer, CORPSE);
    let dying = t
        .keys
        .iter()
        .rfind(|k| k.stack == 2 && k.pose.layer != CORPSE)
        .unwrap();
    assert_eq!((corpse.drawn, corpse.shown), (dying.shown, dying.shown));
}

#[test]
fn a_strikes_kills_show_in_the_count_box_at_its_end() {
    // E17 M3 T5b (design §4.3): the box shows the count before the kills (+0x50) through the
    // strike's iterations and the new one from its end, the whole redraw shown at once; a strike
    // that kills none changes nothing. Two shots, then a strike back at the shooter.
    let o = snapshot(vec![
        at(0, Side::Attacker, MARKSMAN, 35, false),
        at(1, Side::Defender, GOBLIN, 49, false),
    ]);
    let shot = flags(true, false, false);
    let shots = vec![struck(0, 1, 6, shot), struck(0, 1, 5, shot)];
    let back = vec![
        struck(1, 0, 0, flags(false, false, false)),
        struck(0, 1, 29, flags(false, true, false)),
    ];
    let t = play(&o, &[(Some(0), shots), (Some(9000), back)]);
    let ends: Vec<u32> = t.counts.iter().map(|k| k.shown).collect();
    assert_eq!(
        t.counts
            .iter()
            .map(|k| (k.stack, k.count))
            .collect::<Vec<_>>(),
        [(1, 34), (1, 29), (1, 0)]
    );
    // Each as the strike's last frame shows (its reaction's standing frame, drawn a tick before).
    for &end in &ends {
        assert!(
            t.keys
                .iter()
                .any(|k| k.stack == 1 && k.shown == end && k.drawn < end)
        );
    }
    assert!(ends[1] < 9000 && ends[2] > 9000);
    assert_eq!(t.counts(ends[0] - 1), BTreeMap::new());
    assert_eq!(t.counts(ends[0]), BTreeMap::from([(1, 34)]));
    assert_eq!(t.counts(ends[1]), BTreeMap::from([(1, 29)]));
    assert!(t.changes(t.end).contains(&ends[1]));
}

#[test]
fn a_flight_of_one_hex_has_no_cycles() {
    // k = (d + D / 2) / D is 0 for one hex (d 44, D 115); the executable then skips the cycles
    // (0x4B4C5B) and lands at once (0x4B4E1C): group 20, then 21 on the landing hex.
    let mut angels = at(0, Side::Attacker, ANGEL, 35, false);
    angels.flies = true;
    let t = play(&snapshot(vec![angels]), &[(None, vec![moved(0, 35, 36)])]);
    assert_eq!(runs(&t, 0), [20, 21, 2]);
}

#[test]
fn a_battle_is_still_after_its_end_without_outline_or_fidgets() {
    let o = snapshot(vec![at(0, Side::Attacker, PIKEMAN, 35, false)]);
    // The snapshot's active stack outlined, no command: not still.
    assert!(!play(&o, &[]).still());
    // A command's first frame clears the outline; without fidgets nothing changes after it.
    let t = play(&o, &[(None, vec![moved(0, 35, 36)])]);
    assert!(t.still());
    // A stack that fidgets keeps it moving.
    let mut numbers = looks();
    numbers.get_mut(&PIKEMAN).unwrap().frames.insert(1, 4);
    let t = fidgeting_play(&o, &[(None, vec![moved(0, 35, 36)])], &numbers, 1);
    assert!(!t.still());
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
    // A shooter the timeline does not know shoots nothing; its target reacts alone (T5c).
    let t = play(
        &o,
        &[(None, vec![struck(9, 1, 1, flags(true, false, false))])],
    );
    assert!(t.flights.is_empty());
    assert_eq!(runs(&t, 1), [3, 2]);
}

#[test]
fn neighbours_follow_the_layout() {
    let layout = layout();
    let stage = Stage {
        layout: &layout,
        width: 17,
        groups: tables().battle.groups,
        magic: &magic(),
        casts: &Casts::default(),
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
            result: Outcome::Draw {},
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
        magic: &magic(),
        casts: &Casts::default(),
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

#[test]
fn a_target_reacts_as_the_projectile_lands() {
    // A shot marks no striker (0x441A87 only in a melee), so the reactions start with the loop,
    // however many frames the shooter has left after its climax (the Cyclops: 9 of 16; the
    // Marksmen's recorded shot has 1): the shooter's frames 3 to 7 go on beside the hit.
    let mut numbers = looks();
    numbers.get_mut(&MARKSMAN).unwrap().climax = 3;
    let layout = layout();
    let stage = Stage {
        layout: &layout,
        width: 17,
        groups: tables().battle.groups,
        magic: &magic(),
        casts: &Casts::default(),
    };
    let o = snapshot(vec![
        at(0, Side::Attacker, MARKSMAN, 35, false),
        at(1, Side::Defender, GOBLIN, 49, false),
    ]);
    let events = [struck(0, 1, 1, flags(true, false, false))];
    let commands = [Command {
        at: None,
        events: &events,
    }];
    let t = timeline(&o, &commands, &numbers, stage, Speed::default()).unwrap();
    let landed = t.flights[0].from + t.flights[0].steps * t.flights[0].step;
    let hit = first(&t, 1, 3);
    assert_eq!(hit, landed);
    let shooter: Vec<(u32, u32)> = t
        .keys
        .iter()
        .filter(|k| k.stack == 0 && k.drawn >= hit)
        .map(|k| (k.pose.group, k.pose.frame))
        .collect();
    assert_eq!(
        &shooter[..6],
        [(15, 3), (15, 4), (15, 5), (15, 6), (15, 7), (2, 0)]
    );
    // The hit's seven frames, one per iteration from the landing, then standing.
    let goblins: Vec<(u32, u32)> = t
        .keys
        .iter()
        .filter(|k| k.stack == 1 && k.drawn >= hit)
        .map(|k| (k.pose.group, k.pose.frame))
        .collect();
    assert_eq!(
        goblins[..8],
        [
            (3, 0),
            (3, 1),
            (3, 2),
            (3, 3),
            (3, 4),
            (3, 5),
            (3, 6),
            (2, 0)
        ]
    );
}

/// A war machine of army slot `slot` (E17 M3 T3b).
fn machine(id: u8, side: Side, creature: u16, hex: u8, slot: u8) -> Stack {
    let mut s = at(id, side, creature, hex, true);
    (s.war_machine, s.slot, s.speed) = (true, slot, 0);
    s
}

fn healed(healer: u8, target: u8) -> Event {
    Event::Healed {
        healer,
        target,
        hit_points: 6,
    }
}

#[test]
fn a_first_aid_tent_heals_under_its_magic_animation() {
    // E17 M3 T3b (`machines`): the Goblins walk; then the tent's turn plays out in the same
    // command: the loop ticks once as it acts (outlining it), the heal's sound starts and
    // magic animation 79 shows over the Pikemen, a frame per 100 ms on the deadline, then a
    // redraw erases it; the tent stays outlined throughout.
    let o = snapshot(vec![
        at(0, Side::Attacker, PIKEMAN, 86, false),
        machine(1, Side::Attacker, TENT, 153, 18),
        at(2, Side::Defender, GOBLIN, 100, false),
    ]);
    let events = vec![
        moved(2, 100, 99),
        Event::TurnStarted { stack: 1 },
        healed(1, 0),
        Event::TurnStarted { stack: 0 },
    ];
    let t = play(&o, &[(Some(0), events)]);
    let turn = t.turns[0].0;
    assert_eq!(t.turns, [(turn, 1), (turn + 400, 0)]);
    // The tick at the tent's turn, outlining it until the heal's end.
    assert_eq!(t.steps.last(), Some(&turn));
    assert_eq!(
        [turn - 1, turn, turn + 399, turn + 400].map(|ms| t.outlined(ms)),
        [None, Some(1), Some(1), Some(0)]
    );
    // The heal's sound with its first frame, on the patient.
    assert!(t.cues.contains(&Cue {
        at: turn,
        stack: 0,
        sound: Sound::Heal,
        stop: false
    }));
    // The frames, each drawn 100 ms before it shows, and the erasure.
    let keys: Vec<(u32, u32, Option<u32>)> = t
        .magics
        .iter()
        .map(|k| (k.drawn, k.shown, k.frame))
        .collect();
    assert_eq!(
        keys,
        [
            (turn, turn + 100, Some(0)),
            (turn + 100, turn + 200, Some(1)),
            (turn + 200, turn + 300, Some(2)),
            (turn + 300, turn + 400, None),
        ]
    );
    assert_eq!(t.end, turn + 400);
    // Centred on the Pikemen at hex 86 (anchor 80, 338): x = 80 - 96 / 2, y = 338 - 94 / 2 -
    // (267 - 171) / 2.
    let shown = t.magic(turn + 150).unwrap();
    assert_eq!(
        shown,
        Magic {
            def: "Heal.def".to_string(),
            frame: 0,
            translucent: true,
            over: vec![(0, [32, 243], false)],
        }
    );
    assert_eq!(t.magic(turn + 299).map(|m| m.frame), Some(1));
    assert_eq!((t.magic(turn + 99), t.magic(turn + 400)), (None, None));
    // The scene's moment, the changes and the harness's lines know it.
    assert_eq!(crate::scene::Moment::at(&t, turn + 150).magic, Some(shown));
    assert!(t.changes(t.end).contains(&(turn + 300)));
    let lines = t.lines();
    for line in [
        format!("{turn} magic 79 f0 0.0"),
        format!("{} magic 79 f2 0.0", turn + 200),
        format!("{} magic -", turn + 300),
    ] {
        assert!(lines.contains(&line), "{line}\n{lines}");
    }
}

#[test]
fn a_heal_without_its_animation_plays_its_sound_only() {
    // A stage without magic animation 79 (a record that never heals has none loaded), and a
    // tent commanded by its hero (First Aid): its own command, outlined from the heal's first
    // frame.
    let o = snapshot(vec![
        machine(1, Side::Attacker, TENT, 153, 18),
        at(0, Side::Attacker, PIKEMAN, 86, false),
        at(2, Side::Defender, GOBLIN, 100, false),
    ]);
    let layout = layout();
    let none = BTreeMap::new();
    let stage = Stage {
        layout: &layout,
        width: 17,
        groups: tables().battle.groups,
        magic: &none,
        casts: &Casts::default(),
    };
    let events = [healed(1, 0), Event::TurnStarted { stack: 2 }];
    let commands = [Command {
        at: Some(0),
        events: &events,
    }];
    let t = timeline(&o, &commands, &looks(), stage, Speed::default()).unwrap();
    assert!(t.magics.is_empty() && t.cues.len() == 1);
    // With the animation: outlined from the first frame shown, not from a tick.
    let t = play(&o, &[(Some(0), events.to_vec())]);
    assert_eq!(t.magics[0].shown, 100);
    assert_eq!(
        [99, 100, 399].map(|ms| t.outlined(ms)),
        [Some(1), Some(1), Some(1)]
    );
    assert_eq!(t.outlined(400), Some(2));
    // Over a two-hex patient: its middle, toward its front.
    let o = snapshot(vec![
        machine(1, Side::Attacker, TENT, 153, 18),
        at(0, Side::Attacker, PIKEMAN, 86, true),
        at(2, Side::Defender, GOBLIN, 100, true),
    ]);
    let t = play(&o, &[(Some(0), vec![healed(1, 0), healed(1, 2)])]);
    let over = |k: usize| t.magics[k].over[0];
    assert_eq!(over(0), (0, [32 + 22, 243], false));
    // The defender's faces left: mirrored, 22 to the left of its anchor (column 15, row 5: 696).
    let top = looks()[&GOBLIN].top;
    assert_eq!(
        over(4),
        (2, [696 - 22 - 48, 338 - 47 - (267 - top) / 2], true)
    );
}

#[test]
fn magic_animations_stand_where_their_place_says() {
    // Place 0 on the anchor, 2 above the creature, 1 centred; 3 beside its standing frame's front
    // (E17 M3 T7b-1: the anchor + the front − 196 facing right, the test's front 0), as high as 1.
    let o = snapshot(vec![
        machine(1, Side::Attacker, TENT, 153, 18),
        at(0, Side::Attacker, PIKEMAN, 86, false),
    ]);
    let layout = layout();
    let place = |place: u8| {
        let mut magic = magic();
        magic.get_mut(&HEAL).unwrap().place = place;
        let stage = Stage {
            layout: &layout,
            width: 17,
            groups: tables().battle.groups,
            magic: &magic,
            casts: &Casts::default(),
        };
        let events = [healed(1, 0)];
        let commands = [Command {
            at: Some(0),
            events: &events,
        }];
        let t = timeline(&o, &commands, &looks(), stage, Speed::default()).unwrap();
        t.magics[0].over[0].1
    };
    assert_eq!(place(0), [32, 338 - 94]);
    assert_eq!(place(1), [32, 338 - 47 - 48]);
    assert_eq!(place(2), [32, 338 - 94 - 96]);
    assert_eq!(place(3), [80 - 196, 338 - 47 - 48]);
}

#[test]
fn war_machines_take_the_records_after_their_sides_army() {
    // The harness's records (E17 M3 T3b): each side's army, then its war machines in the
    // original's setup order: the Catapult (19), the Ballista (16), the tent (18), the Ammo
    // Cart (17), then the turrets; a stack of another slot after them.
    let o = snapshot(vec![
        at(0, Side::Attacker, PIKEMAN, 86, false),
        machine(1, Side::Attacker, TENT, 51, 16),
        machine(2, Side::Attacker, TENT, 18, 17),
        machine(3, Side::Attacker, TENT, 153, 18),
        machine(4, Side::Attacker, TENT, 119, 19),
        machine(5, Side::Attacker, TENT, 120, 20),
        at(6, Side::Defender, GOBLIN, 100, false),
        machine(7, Side::Defender, TENT, 67, 16),
    ]);
    let t = play(&o, &[]);
    let records: Vec<(u8, u8)> = t.members.iter().map(|m| (m.stack, m.record)).collect();
    assert_eq!(
        records,
        [
            (0, 0),
            (4, 1),
            (1, 2),
            (3, 3),
            (2, 4),
            (5, 5),
            (6, 0),
            (7, 1)
        ]
    );
}

#[test]
fn a_war_machine_is_outlined_from_its_tick_to_its_first_frame() {
    // An automatic war machine's turn (the Ballista's shot in `machines`): the loop ticks as it
    // acts, outlining it until its action's first frame shows; a turn that is the command's
    // last (the player's) does not tick.
    let o = snapshot(vec![
        at(0, Side::Attacker, PIKEMAN, 86, false),
        machine(1, Side::Attacker, TENT, 153, 18),
        at(2, Side::Defender, GOBLIN, 100, false),
    ]);
    let events = vec![
        moved(2, 100, 99),
        Event::TurnStarted { stack: 1 },
        moved(0, 86, 87),
        Event::TurnStarted { stack: 2 },
    ];
    let t = play(&o, &[(Some(0), events)]);
    let turn = t.turns[0].0;
    assert_eq!(t.steps.last(), Some(&turn));
    let first = t.keys.iter().find(|k| k.stack == 0).unwrap().shown;
    assert_eq!(
        [turn, first - 1, first].map(|ms| t.outlined(ms)),
        [Some(1), Some(1), None]
    );
    // Without an animation before it, the tick is not due: no outline.
    let events = vec![
        Event::TurnStarted { stack: 1 },
        moved(0, 86, 87),
        Event::TurnStarted { stack: 2 },
    ];
    let t = play(&o, &[(Some(0), events)]);
    assert_eq!(t.steps, [0]);
    assert_eq!(t.outlined(50), Some(0));
}

#[test]
fn the_log_records_what_the_battle_writes_and_when() {
    // E17 M3 T6b (design §9.8): the Angels (0) and the Marksmen (1) against the Goblins (2, 4)
    // and the Pikemen (3); the attacker's tent (5); the battle opens in the tactics phase.
    let mut o = snapshot(vec![
        at(0, Side::Attacker, ANGEL, 52, false),
        at(1, Side::Attacker, MARKSMAN, 35, false),
        at(2, Side::Defender, GOBLIN, 53, false),
        at(3, Side::Defender, PIKEMAN, 54, false),
        at(4, Side::Defender, GOBLIN, 51, false),
        machine(5, Side::Attacker, TENT, 153, 18),
    ]);
    o.tactics = Some(h3_protocol::wire::Tactics {
        side: Side::Attacker,
        columns: 3,
    });
    let lucky = Strike {
        lucky: true,
        ..flags(false, false, false)
    };
    let beside = flags(false, false, true);
    let shot = flags(true, false, false);
    let lucky_shot = Strike {
        lucky: true,
        ..shot
    };
    let commands = [
        // In the tactics phase: a wait and a round write nothing.
        (
            Some(0),
            vec![
                Event::Waited { stack: 0 },
                Event::RoundStarted { round: 1 },
                Event::TacticsEnded {},
                Event::TurnStarted { stack: 0 },
            ],
        ),
        // A lucky breath (its line the target's) and a multi-headed strike (every hit's).
        (
            Some(100),
            vec![
                struck(0, 2, 1, lucky),
                struck(0, 3, 1, beside),
                struck(0, 2, 0, flags(false, false, false)),
                struck(0, 4, 0, beside),
                Event::TurnStarted { stack: 1 },
            ],
        ),
        // A lucky shot at one target, an area shot (its line as its reactions start).
        (
            None,
            vec![
                struck(1, 3, 1, lucky_shot),
                struck(1, 2, 0, shot),
                struck(
                    1,
                    3,
                    0,
                    Strike {
                        secondary: true,
                        ..shot
                    },
                ),
                Event::TurnStarted { stack: 2 },
            ],
        ),
        // Morale, a defend, a war machine's defend, a heal, a wait and a round.
        (
            None,
            vec![
                Event::GoodMorale { stack: 2 },
                Event::BadMorale { stack: 4 },
                Event::Defended { stack: 2 },
                Event::Defended { stack: 5 },
                healed(5, 0),
                Event::Waited { stack: 4 },
                // An event the timeline does not show.
                Event::TurnPassed { stack: 3 },
                Event::RoundStarted { round: 2 },
                Event::TurnStarted { stack: 0 },
            ],
        ),
    ];
    let t = play(&o, &commands);
    let kinds: Vec<&str> = t
        .log
        .iter()
        .map(|(_, l)| match l {
            Logged::Action => "action",
            Logged::Turn(_) => "turn",
            Logged::Strike {
                spread: Spread::Target,
                ..
            } => "strike",
            Logged::Strike { .. } => "around",
            Logged::Wait { .. } => "wait",
            Logged::Defend { .. } => "defend",
            Logged::Disrupted { .. } => "disrupted",
            Logged::Heal { .. } => "heal",
            Logged::Morale { good: true, .. } => "good",
            Logged::Morale { .. } => "bad",
            Logged::Luck { .. } => "luck",
            Logged::Round => "round",
            Logged::Status { .. } => "status",
            Logged::Scroll(_) => "scroll",
            Logged::Cast { .. } | Logged::Resisted { .. } | Logged::SpellDamage { .. } => "spell",
        })
        .collect();
    // E17 M3 T6c-2: a wait, two strikes, a defend's command.
    let calm: Vec<(bool, bool)> = t.actions.iter().map(|a| (a.stands, a.unmarks)).collect();
    assert_eq!(
        calm,
        [(false, false), (true, true), (true, true), (false, false)]
    );
    assert_eq!(t.actions[1].start, 100);
    assert_eq!(
        kinds,
        [
            "action", "turn", "action", "luck", "strike", "around", "turn", "action", "luck",
            "strike", "around", "turn", "action", "good", "bad", "defend", "heal", "wait", "round",
            "turn"
        ]
    );
    // In time order; the striker by its number then, the hits with their damage and kills.
    assert!(t.log.windows(2).all(|w| w[0].0 <= w[1].0));
    assert!(t.log.iter().any(|(_, l)| *l
        == Logged::Strike {
            attacker: 0,
            count: 40,
            spread: Spread::Target,
            hits: vec![(2, 1, 1), (3, 1, 1)]
        }));
    // The area shot's line comes as its reactions start: before the turn after them.
    let area = t
        .log
        .iter()
        .position(|(_, l)| {
            matches!(
                l,
                Logged::Strike {
                    attacker: 1,
                    spread: Spread::Around,
                    ..
                }
            )
        })
        .unwrap();
    assert!(t.log[area].0 < t.log[area + 1].0);
    let heal = t.log.iter().find(|(_, l)| matches!(l, Logged::Heal { .. }));
    assert_eq!(
        heal.map(|(_, l)| l.clone()),
        Some(Logged::Heal {
            healer: 5,
            target: 0,
            counts: [40, 40],
            hit_points: 6
        })
    );
}

#[test]
fn the_stacks_the_mouse_enters_fidget_on_the_idle_steps() {
    // E17 M3 T6c-2 (design §9.10): the Gnolls (stack 1) have three frames of group 1.
    let o = snapshot(vec![
        at(0, Side::Attacker, PIKEMAN, 35, false),
        at(1, Side::Defender, GNOLL, 49, false),
    ]);
    let fidget = |id: u8| if id == 1 { 3 } else { 0 };
    let pose = |t: &Timeline, id: u8, ms: u32| {
        let p = t.at(ms)[&id];
        (p.group, p.frame)
    };
    // Entered at 50 while the battle idles: frame 0 at once, one frame a step, then standing.
    let mut t = play(&o, &[]);
    t.enter(&[], 1, &fidget);
    t.enter(&[(50, 1)], 1, &fidget);
    let seen = [0, 50, 100, 200, 300].map(|ms| pose(&t, 1, ms));
    assert_eq!(seen, [(2, 0), (1, 0), (1, 1), (1, 2), (2, 0)]);
    let lines = t.lines();
    assert!(
        lines.contains("50 1.0 g1 f0 h49\n100 1.0 g1 f1 h49\n"),
        "{lines}"
    );
    // Entered again while it shows group 1: it stands; then again: group 1 anew. A stack
    // without the group that stands: nothing.
    let mut t = play(&o, &[]);
    t.enter(&[(50, 1), (150, 1), (160, 0), (400, 1)], 1, &fidget);
    assert_eq!(pose(&t, 1, 100), (1, 1));
    assert_eq!(pose(&t, 1, 150), (2, 0));
    assert_eq!(pose(&t, 1, 350), (2, 0));
    assert_eq!(pose(&t, 1, 450), (1, 0));
    assert_eq!(pose(&t, 1, 500), (1, 1));
    assert_eq!(pose(&t, 0, 200), (2, 0));
    // Entered again at a step's time (200): the step comes first, then it stands.
    let mut t = play(&o, &[]);
    t.enter(&[(50, 1), (200, 1)], 1, &fidget);
    assert!(t.lines().contains("200 1.0 g2 f0 h49\n"));
    assert_eq!(pose(&t, 1, 250), (2, 0));
    assert_eq!(t.keys.iter().filter(|k| k.stack == 1).count(), 4);
    // A walk starting at 250 stands it at its start.
    let mut t = play(&o, &[(Some(250), vec![moved(0, 35, 36)])]);
    t.enter(&[(50, 1)], 1, &fidget);
    assert_eq!(pose(&t, 1, 249), (1, 2));
    assert_eq!(pose(&t, 1, 250), (2, 0));
    // Its own walk at 150 ends it: the walk's poses from then.
    let walk = play(&o, &[(Some(150), vec![moved(1, 49, 48)])]);
    let mut t = walk.clone();
    t.enter(&[(50, 1)], 1, &fidget);
    assert_eq!(pose(&t, 1, 100), (1, 1));
    assert_eq!(t.at(150)[&1], walk.at(150)[&1]);
    assert_eq!(t.at(2000)[&1], walk.at(2000)[&1]);
    // Played whole between two commands: the steps before the next one.
    let later = play(&o, &[(Some(2000), vec![moved(0, 35, 36)])]);
    let mut t = later.clone();
    t.enter(&[(50, 1)], 1, &fidget);
    assert_eq!(pose(&t, 1, 200), (1, 2));
    assert_eq!(pose(&t, 1, 300), (2, 0));
    // Entered after its own walk: the mouse's keys after the walk's.
    let mut t = walk.clone();
    t.enter(&[(9000, 1)], 1, &fidget);
    assert_eq!(pose(&t, 1, 9000), (1, 0));
    assert_eq!(t.at(8000)[&1], walk.at(8000)[&1]);
    // An entered stack that is not the battle's: nothing.
    let mut t = play(&o, &[]);
    t.enter(&[(50, 9)], 1, &fidget);
    assert_eq!(t, play(&o, &[]));
}

#[test]
fn the_originals_own_fidgets_are_left_out() {
    let recorded = "0 0.1 g2 f0 h35\n0 0.2 g1 f0 h69\n100 0.2 g1 f1 h69\n200 0.1 g1 f0 h35\n\
                    200 0.2 g2 f0 h69\n300 0.1 g1 f1 h35\n400 0.1 g2 f0 h35\n400 magic 3 f0 0.1\n\
                    500 0.1 g3 f0 h35\n";
    let kept: BTreeSet<String> = ["0.2".to_string()].into_iter().collect();
    assert_eq!(
        unfidgeted(recorded, &kept),
        "0 0.1 g2 f0 h35\n0 0.2 g1 f0 h69\n100 0.2 g1 f1 h69\n200 0.2 g2 f0 h69\n\
         400 magic 3 f0 0.1\n500 0.1 g3 f0 h35\n"
    );
}

#[test]
fn an_actions_start_stands_and_unmarks_by_what_it_does() {
    // E17 M3 T6c-2: up to the next turn; a creature's cast only stands.
    use h3_protocol::wire::Target;
    let none = Target::None {};
    let cast = Event::SpellCast {
        side: Side::Attacker,
        spell: 1,
        target: none,
        second: none,
    };
    let creature = Event::StackCast {
        stack: 0,
        spell: 1,
        target: none,
    };
    let wall = Event::WallShot {
        stack: 0,
        aimed: 1,
        part: 1,
        damage: 1,
    };
    assert_eq!(calms(&[cast]), (true, true));
    assert_eq!(calms(core::slice::from_ref(&creature)), (true, false));
    assert_eq!(calms(&[wall]), (true, true));
    assert_eq!(calms(&[moved(0, 1, 2)]), (true, true));
    let later = [Event::TurnStarted { stack: 1 }, moved(1, 2, 3)];
    assert_eq!(calms(&later), (false, false));
    let round = [Event::RoundStarted { round: 2 }, creature];
    assert_eq!(calms(&round), (false, false));
}

#[test]
fn the_battle_ends_with_the_command_that_decides_it() {
    // E17 M3 T8 (`end`, design §8.2): the Wolf Riders kill the lone Goblins; the battle is
    // decided as the strike's command ends, where the next turn would start, and `lines`
    // writes the harness's `end` then. The commands before it decide nothing.
    let o = snapshot(vec![
        at(0, Side::Attacker, WOLF_RIDER, 52, true),
        at(1, Side::Defender, GOBLIN, 36, false),
    ]);
    let won = Outcome::Victory {
        side: Side::Attacker,
    };
    let commands = [
        (None, vec![Event::Defended { stack: 0 }]),
        (
            Some(1000),
            vec![
                struck(0, 1, 40, flags(false, false, false)),
                Event::BattleEnded { result: won },
            ],
        ),
    ];
    let t = play(&o, &commands);
    let (at, outcome) = t.ended.unwrap();
    assert_eq!(outcome, won);
    assert_eq!(at, t.actions[1].end);
    assert!(at > 1000);
    assert_eq!(t.lines().lines().last(), Some(format!("{at} end").as_str()));
    // Without its event, no end.
    let t = play(&o, &commands[..1]);
    assert_eq!(t.ended, None);
    assert!(!t.lines().contains("end"));
}

/// The magic animations of the casts' tests (E17 M3 T7b-1, the user's DEFs' numbers): Slow's 19
/// (`C09spE0.def`, 20 frames, 74 x 54, place 0), the resisted 78 (`sp09_.def`, 20, 91 x 86, place
/// 3) and Magic Arrow's 64 (`C20SPX.DEF`, 14, 133 x 116, place 1).
fn spell_magic() -> BTreeMap<u8, MagicLook> {
    let entry = |def: &str, frames, size, place| MagicLook {
        def: def.to_string(),
        frames,
        size,
        place,
        translucent: false,
    };
    let mut all = magic();
    all.insert(19, entry("C09spE0.def", 20, [74, 54], 0));
    all.insert(RESISTED, entry("sp09_.def", 20, [91, 86], 3));
    all.insert(64, entry("C20SPX.DEF", 14, [133, 116], 1));
    // E17 M3 T7b-2a: Disrupting Ray's (translucent), Haste's, Cure's and Dispel's.
    all.insert(
        14,
        MagicLook {
            translucent: true,
            ..entry("C07spA1.def", 16, [176, 148], 1)
        },
    );
    all.insert(31, entry("C15spA0.def", 15, [113, 106], 0));
    all.insert(39, entry("C03spW0.def", 15, [94, 119], 1));
    all.insert(41, entry("C05spW0.def", 15, [199, 201], 1));
    all
}

/// What the casts' tests cast (E17 M3 T7b-1): Slow (54) with its animation 19, Magic Arrow (15)
/// with 64 and its five projectile DEFs (`c20spx0`–`4.def`); since T7b-2a Haste (53, 31), Cure
/// (37, 39), Dispel (35, 41) and Disrupting Ray (47, 14 and its projectile); Orrin on the left
/// (`CH00.DEF`'s cast point (92, 67): (49, 48) on the screen, five of its eight casting frames
/// before the spell), no caster on the right.
fn casts() -> Casts {
    let shell = |k: usize, size, frames, angle| Shell {
        def: format!("c20spx{k}.def"),
        size,
        frames,
        angle,
    };
    let arrow = SpellLook {
        original: 15,
        sign: -1,
        value: [0; 2],
        animation: Some(64),
        projectile: vec![
            shell(0, [123, 47], 14, 0),
            shell(1, [123, 59], 15, -16),
            shell(2, [112, 87], 15, -34),
            shell(3, [82, 109], 15, -56),
            shell(4, [50, 119], 15, -83),
        ],
    };
    let timed = |original, sign, animation| SpellLook {
        original,
        sign,
        value: [0; 2],
        animation: Some(animation),
        projectile: Vec::new(),
    };
    // E17 M3 T7b-2a: Disrupting Ray (47) lowers the defense by 3 for either side's hero.
    let ray = SpellLook {
        value: [3; 2],
        projectile: vec![Shell {
            def: "c07spA0.def".to_string(),
            size: [79, 43],
            frames: 8,
            angle: 0,
        }],
        ..timed(47, -1, 14)
    };
    Casts {
        spells: BTreeMap::from([
            (15, arrow),
            (35, timed(35, 0, 41)),
            (37, timed(37, 1, 39)),
            (47, ray),
            (53, timed(53, 1, 31)),
            (54, timed(54, -1, 19)),
        ]),
        casters: [
            Some(Caster {
                point: [49, 48],
                before: 5,
                frames: 8,
            }),
            None,
        ],
        groups: [4, 0],
    }
}

/// The battle's heroes as the casts' sequences show them (Orrin and Crag Hack, `CH00.DEF` and
/// `CH013.DEF`: 1, 4, 11, 11 and 8 frames in groups 0 to 4), both players human.
fn cast_heroes() -> Heroes {
    let look = crate::hero::HeroLook {
        frames: BTreeMap::from([(0, 1), (1, 4), (2, 11), (3, 11), (4, 8)]),
        flag_frames: 8,
    };
    Heroes::new(
        [Some(look.clone()), Some(look)],
        [true, true],
        tables().battle.hero_groups,
        None,
    )
}

/// The timeline of `snapshot` and `commands` with the casts' stage, and the casts' heroes.
fn play_casts(snapshot: &Observation, commands: &[(Option<u32>, Vec<Event>)]) -> Timeline {
    play_casts_with(snapshot, commands, casts())
}

/// [`play_casts`] with Crag Hack casting on the right too (E17 M3 T7b-2a: `CH013.DEF`, five of
/// eight frames before the spell; his cast point not measured, as no projectile of his flies).
fn play_casts_both(snapshot: &Observation, commands: &[(Option<u32>, Vec<Event>)]) -> Timeline {
    let mut casts = casts();
    casts.casters[1] = Some(Caster {
        point: [751, 48],
        before: 5,
        frames: 8,
    });
    play_casts_with(snapshot, commands, casts)
}

/// [`play_casts`] with `casts`.
fn play_casts_with(
    snapshot: &Observation,
    commands: &[(Option<u32>, Vec<Event>)],
    casts: Casts,
) -> Timeline {
    let layout = layout();
    let magic = spell_magic();
    let stage = Stage {
        layout: &layout,
        width: 17,
        groups: tables().battle.groups,
        magic: &magic,
        casts: &casts,
    };
    let commands: Vec<Command<'_>> = commands
        .iter()
        .map(|(at, events)| Command { at: *at, events })
        .collect();
    timeline(snapshot, &commands, &looks(), stage, Speed::default())
        .unwrap()
        .with_heroes(cast_heroes())
}

/// A recorded timeline without its screens' boxes (`screens`, E17 M3 T7a-2b).
fn screenless(lines: &str) -> String {
    lines
        .lines()
        .filter(|l| l.split(' ').nth(1) != Some("screen"))
        .map(|l| format!("{l}\n"))
        .collect()
}

fn cast(spell: u16, target: u8) -> Event {
    Event::SpellCast {
        side: Side::Attacker,
        spell,
        target: Target::Stack { stack: target },
        second: Target::None {},
    }
}

fn applied(target: u8, spell: u16) -> Event {
    Event::EffectApplied {
        target,
        spell,
        rounds: Some(1),
    }
}

fn spell_damage(target: u8, damage: u64, kills: u32) -> Event {
    Event::SpellDamage {
        target,
        value: WireU64(damage),
        caster: WireU64(damage),
        damage: WireU64(damage),
        kills,
    }
}

#[test]
fn the_cast_sequence_plays_as_recorded() {
    // E17 M3 T7b-1 (`seq-cast.toml`): Orrin casts Slow at the Wolf Riders at the opening: his
    // frames 0 to 4, Slow's animation 19 over them after its effect, frames 5 to 7, standing.
    let o = snapshot(vec![
        at(0, Side::Attacker, CHAMPION, 35, true),
        at(1, Side::Attacker, MARKSMAN, 137, false),
        at(2, Side::Defender, WOLF_RIDER, 49, true),
        at(3, Side::Defender, GOBLIN, 151, false),
    ]);
    let t = play_casts(&o, &[(Some(0), vec![cast(54, 2), applied(2, 54)])]);
    assert_eq!(t.lines_with_heroes(3500), until(CAST, 3500));
    // The outline goes at the click and comes back as the hero stands; the line and the spell's
    // sound go with the animation's first frame drawn.
    let outlined: Vec<Option<u8>> = [0, 2899, 2900].map(|t2| t.outlined(t2)).into();
    assert_eq!(outlined, [None, None, Some(0)]);
    assert!(t.log.contains(&(
        500,
        Logged::Cast {
            side: 0,
            spell: 54,
            target: 2,
            count: 40
        }
    )));
    assert_eq!(
        t.cues,
        [Cue {
            at: 500,
            stack: 2,
            sound: Sound::Spell(54),
            stop: false
        }]
    );
    // Shown a deadline after drawn: the hero's frame 0 from 100, standing at once at 2900; the
    // animation standing on the Wolf Riders' anchor.
    let heroes = t.heroes.as_ref().unwrap();
    let group = |p: [Option<crate::hero::HeroPose>; 2]| p[0].map(|h| (h.group, h.frame));
    assert_eq!(group(heroes.at(&t, 99)), Some((0, 0)));
    assert_eq!(group(heroes.at(&t, 100)), Some((4, 0)));
    assert_eq!(group(heroes.drawn_at(&t, 100)), Some((4, 1)));
    assert_eq!(group(heroes.at(&t, 2900)), Some((0, 0)));
    let magic = t.magic(600).unwrap();
    assert_eq!((magic.def.as_str(), magic.frame), ("C09spE0.def", 0));
    assert_eq!(t.magic(2600), None);
}

#[test]
fn the_resist_sequence_plays_as_recorded() {
    // E17 M3 T7b-1 (`seq-resist.toml`): Slow on the Battle Dwarves in round 1, resisted in round
    // 2: the resistance's sound and line, then animation 78 beside them (place 3, facing left:
    // its right edge the anchor less the front less 196).
    let o = snapshot(vec![
        at(1, Side::Defender, BATTLE_DWARF, 100, false),
        at(0, Side::Attacker, PIKEMAN, 86, false),
    ]);
    let commands = [
        (
            Some(0),
            vec![
                Event::Defended { stack: 1 },
                Event::TurnStarted { stack: 0 },
            ],
        ),
        (Some(0), vec![cast(54, 1), applied(1, 54)]),
        (
            Some(2900),
            vec![
                Event::Defended { stack: 0 },
                Event::RoundStarted { round: 2 },
                Event::EffectEnded {
                    target: 1,
                    spell: 54,
                },
                Event::TurnStarted { stack: 1 },
            ],
        ),
        (
            Some(2900),
            vec![
                Event::Defended { stack: 1 },
                Event::TurnStarted { stack: 0 },
            ],
        ),
        (
            Some(2900),
            vec![cast(54, 1), Event::SpellResisted { target: 1 }],
        ),
    ];
    let t = play_casts(&o, &commands);
    assert_eq!(t.lines_with_heroes(6000), until(&screenless(RESIST), 6000));
    assert!(t.log.contains(&(
        3400,
        Logged::Resisted {
            stack: 1,
            count: 40
        }
    )));
    let resisted = Cue {
        at: 3400,
        stack: 1,
        sound: Sound::Resisted,
        stop: false,
    };
    assert_eq!(t.cues.last(), Some(&resisted));
    // Slow's red on the Battle Dwarves' box from its animation's first frame to the next turn's
    // start after the round's start ends it (E17 M3 T7b-2a).
    let hue = |at| t.hues(at).get(&1).copied();
    assert_eq!(
        [hue(599), hue(600), hue(2899), hue(2900)],
        [None, Some(0), Some(0), None]
    );
    let magic = t.magic(3500).unwrap();
    // Hex 100 (column 15, row 5): its anchor 696; the Battle Dwarves' front 215.
    assert_eq!(magic.over[0].1[0], 696 - (215 - 196) - 91);
}

#[test]
fn the_arrow_sequence_plays_as_recorded() {
    // E17 M3 T7b-1 (`seq-arrow.toml`): the Archangel flies beside the Pikemen; Magic Arrow at the
    // Swordsmen at row 0 (round 1), at the Archangel (round 2), at the Swordsmen at row 10 (round
    // 3), each defending: the projectile, then animation 64 over the struck stack with its
    // reaction, the loop as long as the animation.
    let mut angel = at(2, Side::Defender, ARCHANGEL, 83, true);
    angel.flies = true;
    let o = snapshot(vec![
        angel,
        at(0, Side::Attacker, PIKEMAN, 86, false),
        at(1, Side::Defender, SWORDSMAN, 15, false),
        at(3, Side::Defender, SWORDSMAN, 117, false),
        at(4, Side::Defender, SWORDSMAN, 185, false),
    ]);
    let defend = |stack, next| {
        vec![
            Event::Defended { stack },
            Event::TurnStarted { stack: next },
        ]
    };
    let round = |round| {
        vec![
            Event::Defended { stack: 0 },
            Event::RoundStarted { round },
            Event::TurnStarted { stack: 2 },
        ]
    };
    let commands = [
        (
            Some(0),
            vec![moved(2, 83, 174), Event::TurnStarted { stack: 1 }],
        ),
        (Some(2730), defend(1, 3)),
        (Some(2730), defend(3, 4)),
        (Some(2730), defend(4, 0)),
        (Some(2730), vec![cast(15, 1), spell_damage(1, 20, 0)]),
        (Some(5456), round(2)),
        (Some(5456), defend(2, 1)),
        (Some(5456), defend(1, 3)),
        (Some(5456), defend(3, 4)),
        (Some(5456), defend(4, 0)),
        (Some(5456), vec![cast(15, 2), spell_damage(2, 20, 0)]),
        (Some(7951), round(3)),
        (Some(7951), defend(2, 1)),
        (Some(7951), defend(1, 3)),
        (Some(7951), defend(3, 4)),
        (Some(7951), defend(4, 0)),
        (Some(7951), vec![cast(15, 4), spell_damage(4, 20, 0)]),
    ];
    let t = play_casts(&o, &commands);
    assert_eq!(t.lines_with_heroes(11009), until(&screenless(ARROW), 11009));
    // The first projectile: from the hero's frame 4 shown (3230), 22 steps of 33 ms, the level
    // DEF; the line after the loop, when the animation is erased.
    let p = t.projectile(3230).unwrap();
    assert_eq!(
        (p.def.as_str(), p.frame, p.mirrored),
        ("c20spx0.def", 0, false)
    );
    assert_eq!(t.projectile(3263).unwrap().frame, 1);
    assert_eq!(t.projectile(3956), None);
    assert!(t.log.contains(&(
        5156,
        Logged::SpellDamage {
            spell: 15,
            hit: (1, 20, 0)
        }
    )));
}

#[test]
fn casts_without_a_caster_a_stack_or_an_effect_play_what_they_can() {
    // E17 M3 T7b-1: the defender's side has no caster (no frames of its hero); a cast at a hex
    // or at no stack, and one without an effect, play the hero's frames alone; a spell the
    // stage does not know plays nothing of its own; a projectile at a two-hex stack facing right
    // aims at its middle, 22 pixels toward its front.
    let o = snapshot(vec![
        at(0, Side::Attacker, CHAMPION, 35, true),
        at(3, Side::Defender, GOBLIN, 151, false),
    ]);
    let by_defender = Event::SpellCast {
        side: Side::Defender,
        spell: 54,
        target: Target::Stack { stack: 3 },
        second: Target::None {},
    };
    let at_hex = Event::SpellCast {
        side: Side::Attacker,
        spell: 54,
        target: Target::Hex { hex: 70 },
        second: Target::None {},
    };
    let commands = [
        (Some(0), vec![by_defender, applied(3, 54)]),
        (Some(3000), vec![cast(15, 0), spell_damage(0, 5, 0)]),
        (Some(8000), vec![at_hex]),
        (
            Some(10000),
            vec![cast(54, 9), cast(54, 3), Event::TurnStarted { stack: 3 }],
        ),
        (Some(13000), vec![cast(99, 3), applied(3, 99)]),
    ];
    let t = play_casts(&o, &commands);
    // A spell whose animation the stage has not loaded: none drawn, over one stack or marked.
    let field = layout();
    let (magic, mut unloaded) = (spell_magic(), casts());
    for spell in unloaded.spells.values_mut() {
        spell.animation = Some(200);
    }
    let stage = Stage {
        layout: &field,
        width: 17,
        groups: tables().battle.groups,
        magic: &magic,
        casts: &unloaded,
    };
    let bare = [
        vec![cast(54, 3), applied(3, 54)],
        vec![cast(15, 3), spell_damage(3, 5, 0)],
    ];
    let bare: Vec<Command<'_>> = bare
        .iter()
        .map(|events| Command { at: None, events })
        .collect();
    let without = timeline(&o, &bare, &looks(), stage, Speed::default()).unwrap();
    assert!(without.magics.is_empty());
    assert_eq!(without.flights.len(), 1);
    // No hero frames for the defender's cast: Slow's animation from the click.
    assert!(t.hero_keys.iter().all(|k| k.side == 0 && k.drawn >= 3000));
    assert_eq!(t.magics[0].drawn, 0);
    // The projectile at the Champions (hex 35, facing right): their middle, half their height up.
    let [ax, ay] = layout().anchor(2, 1);
    let to = [ax + 22, ay - (267 - 165) / 2];
    let shells = casts().spells[&15].projectile.clone();
    let shell = &shells[shell_index(&shells, [to[0] - 49, to[1] - 48])];
    let flight = &t.flights[0];
    let [w, h] = shell.size;
    let steps = i32::try_from(flight.steps).unwrap();
    assert_eq!(flight.def, shell.def);
    assert_eq!(
        [
            flight.start[0] + w / 2 + steps * flight.delta[0],
            flight.start[1] + h / 2 + steps * flight.delta[1]
        ],
        [
            49 + (to[0] - 49) / steps * steps,
            48 + (to[1] - 48) / steps * steps
        ]
    );
    // The hex's cast, the cast at no stack, the one without an effect and the unknown spell:
    // the hero's eight frames and its standing, no animation, no line, no sound.
    let frames = |from: u32, to: u32| {
        t.hero_keys
            .iter()
            .filter(|k| (from..to).contains(&k.drawn))
            .count()
    };
    assert_eq!(
        [
            frames(8000, 10000),
            frames(10000, 13000),
            frames(13000, 20000)
        ],
        [9, 18, 9]
    );
    assert_eq!(t.magics.iter().filter(|k| k.drawn >= 8000).count(), 0);
    let spoken: Vec<(u32, &Logged)> = t
        .log
        .iter()
        .filter(|(at, l)| *at >= 8000 && !matches!(l, Logged::Action | Logged::Turn(_)))
        .map(|(at, l)| (*at, l))
        .collect();
    let unknown = Logged::Cast {
        side: 0,
        spell: 99,
        target: 3,
        count: 40,
    };
    assert_eq!(spoken, [(13500, &unknown)]);
    // The cast at no stack ends its effects there; the next event starts the next turn.
    assert_eq!(t.active(13000), Some(3));
}

#[test]
fn a_spells_projectile_takes_the_def_nearest_its_angle() {
    // `spell_projectile` 0x467E5D (E17 M3 T7b-1): straight below the last, straight above or
    // level the first; else the count of midpoints (−8, −25, −45, −69.5) at least the angle.
    let shells = casts().spells[&15].projectile.clone();
    assert_eq!(shell_index(&shells, [0, 10]), 4);
    assert_eq!(shell_index(&shells, [0, -10]), 0);
    assert_eq!(shell_index(&shells, [0, 0]), 0);
    assert_eq!(shell_index(&shells, [100, -50]), 0);
    assert_eq!(shell_index(&shells, [100, 7]), 0);
    assert_eq!(shell_index(&shells, [-100, 20]), 1);
    assert_eq!(shell_index(&shells, [100, 70]), 2);
    assert_eq!(shell_index(&shells, [100, 200]), 3);
    assert_eq!(shell_index(&shells, [1, 1000]), 4);
    assert_eq!(shell_index(&[], [1, 1]), 0);
}

#[test]
fn hero_lines_leave_out_a_side_without_a_hero() {
    // E17 M3 T7b-1: `lines_with_heroes` prints the heroes there are; without heroes, none.
    let o = snapshot(vec![
        at(0, Side::Attacker, CHAMPION, 35, true),
        at(3, Side::Defender, GOBLIN, 151, false),
    ]);
    let commands = [(Some(0), vec![cast(54, 3), applied(3, 54)])];
    let t = play_casts(&o, &commands);
    let look = crate::hero::HeroLook {
        frames: BTreeMap::from([(0, 1), (4, 8)]),
        flag_frames: 8,
    };
    let groups = tables().battle.hero_groups;
    let one = Heroes::new([Some(look), None], [true, false], groups, None);
    let lines = t.clone().with_heroes(one).lines_with_heroes(3000);
    assert!(lines.contains("0 hero 0 g0 f0\n") && !lines.contains("hero 1"));
    let none = Timeline {
        heroes: None,
        ..t.clone()
    };
    assert!(!none.lines_with_heroes(3000).contains("hero"));
    assert_eq!(t.lines(), none.lines_with_heroes(3000));
}

/// Crag Hack's cast (the defender's hero; the casts' tests have no caster on the right).
fn defender_cast(spell: u16, target: u8) -> Event {
    Event::SpellCast {
        side: Side::Defender,
        spell,
        target: Target::Stack { stack: target },
        second: Target::None {},
    }
}

#[test]
fn the_haste_sequence_plays_as_recorded() {
    // E17 M3 T7b-2a (`seq-haste.toml`): Orrin hastes the Pikemen (round 1), they walk hasted
    // (round 2: W trunc(575 × 0.65) = 373, 62 ms a frame), Crag Hack slows them (Slow ends
    // Haste) and they walk slowed (round 3: 862, 143 ms a frame).
    let o = snapshot(vec![
        at(1, Side::Defender, VAMPIRE_LORD, 100, false),
        at(0, Side::Attacker, PIKEMAN, 86, false),
    ]);
    let turn = |stack| Event::TurnStarted { stack };
    let commands = [
        (Some(0), vec![Event::Defended { stack: 1 }, turn(0)]),
        (Some(0), vec![cast(53, 0), applied(0, 53)]),
        (
            Some(2400),
            vec![
                Event::Defended { stack: 0 },
                Event::RoundStarted { round: 2 },
                turn(1),
            ],
        ),
        (Some(2400), vec![Event::Defended { stack: 1 }, turn(0)]),
        (
            Some(2400),
            vec![moved(0, 86, 87), moved(0, 87, 88), turn(1)],
        ),
        (
            Some(3244),
            vec![
                defender_cast(54, 0),
                Event::EffectEnded {
                    target: 0,
                    spell: 53,
                },
                applied(0, 54),
            ],
        ),
        (
            Some(6144),
            vec![
                Event::Defended { stack: 1 },
                Event::RoundStarted { round: 3 },
                turn(0),
            ],
        ),
        (Some(6144), vec![moved(0, 88, 89), moved(0, 89, 90)]),
    ];
    let t = play_casts_both(&o, &commands);
    assert_eq!(t.lines_with_heroes(9144), until(&screenless(HASTE), 9144));
    // The box's hue: none before the cast, Haste's green from its animation's first frame
    // (drawn at 500), Slow's red from its first (3 744), each spell's sign summed (§4.3).
    let hue = |at| t.hues(at).get(&0).copied();
    assert_eq!(hue(599), None);
    assert_eq!(hue(600), Some(0x3EAA_B368));
    assert_eq!(hue(3843), Some(0x3EAA_B368));
    assert_eq!(hue(3844), Some(0));
    assert_eq!(t.hues(9000).get(&1), None);
}

#[test]
fn the_dispel_sequence_plays_as_recorded() {
    // E17 M3 T7b-2a (`seq-dispel.toml`): Crag Hack slows the Pikemen; Orrin's Disrupting Ray
    // flies at the Vampire Lords (its projectile, its animation 14 from its third frame, as the
    // flight left the deadline behind), Cure (its effect, then 39), Slow again, Dispel (41, then
    // its effect).
    let mut lords = at(1, Side::Defender, VAMPIRE_LORD, 100, false);
    lords.defense = 10;
    let o = snapshot(vec![lords, at(0, Side::Attacker, PIKEMAN, 86, false)]);
    let turn = |stack| Event::TurnStarted { stack };
    let ended = |target| Event::EffectEnded { target, spell: 54 };
    let ray = Event::EffectApplied {
        target: 1,
        spell: 47,
        rounds: None,
    };
    let commands = [
        (Some(0), vec![defender_cast(54, 0), applied(0, 54)]),
        (Some(2900), vec![Event::Defended { stack: 1 }, turn(0)]),
        (Some(2900), vec![cast(47, 1), ray]),
        (
            Some(5926),
            vec![
                Event::Defended { stack: 0 },
                Event::RoundStarted { round: 2 },
                turn(1),
            ],
        ),
        (Some(5926), vec![Event::Defended { stack: 1 }, turn(0)]),
        (Some(5926), vec![cast(37, 0), ended(0)]),
        (
            Some(8326),
            vec![
                Event::Defended { stack: 0 },
                Event::RoundStarted { round: 3 },
                turn(1),
            ],
        ),
        (Some(8326), vec![defender_cast(54, 0), applied(0, 54)]),
        (Some(11226), vec![Event::Defended { stack: 1 }, turn(0)]),
        (Some(11226), vec![cast(35, 0), ended(0)]),
    ];
    let t = play_casts_both(&o, &commands);
    assert_eq!(
        t.lines_with_heroes(13726),
        until(&screenless(DISPEL), 13726)
    );
    // Disrupting Ray's line after its animation, the defense it took (3 of 10); the Vampire
    // Lords' defend gains 1 of their 7 since.
    let disrupted = (
        5626,
        Logged::Disrupted {
            stack: 1,
            lowered: 3,
        },
    );
    assert!(t.log.contains(&disrupted));
    let defends: Vec<i32> = t
        .log
        .iter()
        .filter_map(|(_, l)| match *l {
            Logged::Defend {
                stack: 1, defense, ..
            } => Some(defense),
            _ => None,
        })
        .collect();
    assert_eq!(defends, [10, 7, 7]);
    // The lines of Cure and Dispel after their animations.
    let cast_line = |spell| Logged::Cast {
        side: 0,
        spell,
        target: 0,
        count: 40,
    };
    assert!(t.log.contains(&(8026, cast_line(37))));
    assert!(t.log.contains(&(13326, cast_line(35))));
    // The hues: the Pikemen's red under Slow from its animation's first frame, gone with Cure's
    // first and Dispel's next redraw after its animation; the Vampire Lords' red from the redraw
    // after Disrupting Ray's animation.
    let hue = |stack, at| t.hues(at).get(&stack).copied();
    assert_eq!(hue(0, 599), None);
    assert_eq!(hue(0, 600), Some(0));
    assert_eq!(hue(0, 6525), Some(0));
    assert_eq!(hue(0, 6526), None);
    assert_eq!(hue(0, 8926), Some(0));
    assert_eq!(hue(0, 13425), Some(0));
    assert_eq!(hue(0, 13426), None);
    assert_eq!(hue(1, 5725), None);
    assert_eq!(hue(1, 5726), Some(0));
}

#[test]
fn spells_tint_boxes_from_the_snapshot_on_and_disrupting_ray_flies_only_from_a_caster() {
    // E17 M3 T7b-2a: a stack slowed in the snapshot shows Slow's hue from the start; Haste on it
    // too gives a mixed one; a spell renewed changes nothing; an effect on a stack gone does
    // nothing; the defender's Disrupting Ray without a caster plays its animation and its line
    // without a projectile, its defense lowered to 0 at least.
    let mut slowed = at(0, Side::Attacker, PIKEMAN, 86, false);
    slowed.effects = vec![h3_protocol::wire::Effect {
        spell: 54,
        rounds: Some(2),
        mastery: 0,
        value: 0,
        attack: 0,
        defense: 0,
        speed: 0,
    }];
    let o = snapshot(vec![
        slowed,
        at(1, Side::Defender, VAMPIRE_LORD, 100, false),
    ]);
    let commands = [
        (
            Some(0),
            vec![
                applied(0, 54),
                applied(9, 54),
                Event::EffectEnded {
                    target: 9,
                    spell: 54,
                },
                applied(0, 53),
            ],
        ),
        (
            Some(1000),
            vec![
                defender_cast(47, 0),
                Event::EffectApplied {
                    target: 0,
                    spell: 47,
                    rounds: None,
                },
            ],
        ),
    ];
    let t = play_casts(&o, &commands);
    let hue = |at| t.hues(at).get(&0).copied();
    assert_eq!(hue(0), Some(0));
    // Haste and Slow: r = 0, the hue f32(0.1667), from the next frame drawn (the ray's first).
    assert_eq!(t.tints.len(), 3);
    assert_eq!(hue(1000), Some(0x3E2A_B368));
    assert!(t.flights.is_empty());
    let lowered: Vec<i32> = t
        .log
        .iter()
        .filter_map(|(_, l)| match *l {
            Logged::Disrupted { lowered, .. } => Some(lowered),
            _ => None,
        })
        .collect();
    assert_eq!(lowered, [1]);
    // Disrupting Ray (negative) with Haste and Slow: r = −1/3.
    assert_eq!(hue(4000), Some(0x3DE3_99E0));
}
