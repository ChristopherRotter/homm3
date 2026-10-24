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
const LUCK: &str = include_str!("../../../../situations/timelines/luck.txt");
const MORALE: &str = include_str!("../../../../situations/timelines/morale.txt");
const BAD: &str = include_str!("../../../../situations/timelines/bad.txt");
const FEAR: &str = include_str!("../../../../situations/timelines/fear.txt");
const MASS: &str = include_str!("../../../../situations/timelines/mass.txt");
const OBSTACLES: &str = include_str!("../../../../situations/timelines/obstacles.txt");

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
/// A creature caster of the area spells' tests (E17 M3 T7c-1), invented numbers.
const FAERIE_DRAGON: u16 = 18;

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
        explosion: None,
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
        frames: 9,
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
        frames: 10,
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
        frames: 9,
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
        frames: 9,
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
        bugs: Bugs::Original,
        siege: None,
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
    assert_eq!(t.beams[0].cursor, [t.beams[0].from, 879]);
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
        bugs: Bugs::Original,
        siege: None,
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
        bugs: Bugs::Original,
        siege: None,
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
        // Their turn clears it: hit (3).
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
        // non-secondary strike end the run, the stray hit reacting alone (E17 M3 T9b, the
        // reviewer: it counted its kills only).
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
    assert_eq!(groups(0), [11, 2, 11, 2, 13, 2, 11, 2, 3, 2]);
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
    assert_eq!(layers(0), [6, 4, 6, 4, 6, 4, 5, 4, 5, 4]);
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
        bugs: Bugs::Original,
        siege: None,
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
        bugs: Bugs::Original,
        siege: None,
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
    assert_eq!(b.cursor[1], b.from + ticks * b.step);
    assert_eq!(b.until, b.cursor[1]);
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
        bugs: Bugs::Original,
        siege: None,
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
        bugs: Bugs::Original,
        siege: None,
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
        frames: 9,
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
        bugs: Bugs::Original,
        siege: None,
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
        bugs: Bugs::Original,
        siege: None,
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
        bugs: Bugs::Original,
        siege: None,
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
        bugs: Bugs::Original,
        siege: None,
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
        bugs: Bugs::Original,
        siege: None,
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
        bugs: Bugs::Original,
        siege: None,
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
            bugs: Bugs::Original,
            siege: None,
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
            Logged::Fear { .. } => "fear",
            Logged::Round => "round",
            Logged::Status { .. } => "status",
            Logged::Scroll(_) => "scroll",
            Logged::Unmark => "unmark",
            Logged::Cast { .. }
            | Logged::Resisted { .. }
            | Logged::SpellDamage { .. }
            | Logged::CastAll { .. }
            | Logged::MassDamage { .. }
            | Logged::Armageddon { .. }
            | Logged::AreaDamage { .. }
            | Logged::Raised { .. } => "spell",
            Logged::Drain { .. }
            | Logged::DeathStare { .. }
            | Logged::Thunderbolt { .. }
            | Logged::Poisoned { .. }
            | Logged::AcidBreath { .. } => "striker",
            Logged::Turret { .. } => "turret",
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
    // E17 M3 T6c-2: up to the next turn; a creature's cast only stands (CLT-Q8), and unmarks too
    // with the bugs fixed (E17 M3 T7d).
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
    let original = |events: &[Event]| calms(events, Bugs::Original);
    assert_eq!(original(&[cast]), (true, true));
    assert_eq!(original(core::slice::from_ref(&creature)), (true, false));
    assert_eq!(
        calms(core::slice::from_ref(&creature), Bugs::Fixed),
        (true, true)
    );
    assert_eq!(original(&[wall]), (true, true));
    assert_eq!(original(&[moved(0, 1, 2)]), (true, true));
    let later = [Event::TurnStarted { stack: 1 }, moved(1, 2, 3)];
    assert_eq!(original(&later), (false, false));
    let round = [Event::RoundStarted { round: 2 }, creature];
    assert_eq!(calms(&round, Bugs::Fixed), (false, false));
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
        ..SpellLook::default()
    };
    let timed = |original, sign, animation| SpellLook {
        original,
        sign,
        value: [0; 2],
        animation: Some(animation),
        ..SpellLook::default()
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
        ..Casts::default()
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
        bugs: Bugs::Original,
        siege: None,
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
        bugs: Bugs::Original,
        siege: None,
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

/// The rolled effects' magic animations (E17 M3 T7b-2b-2b): Fear's `C0fear` (translucent), good
/// luck's `C09spA0`, good morale's `C09spW0` and bad morale's `C14spE0`, all centred on the
/// creature (the user's archive; numbers only).
fn roll_magic() -> BTreeMap<u8, MagicLook> {
    let entry = |def: &str, frames, size| MagicLook {
        def: def.to_string(),
        frames,
        size,
        place: 1,
        translucent: false,
    };
    let mut all = magic();
    all.insert(
        super::FEAR,
        MagicLook {
            translucent: true,
            ..entry("C0fear.def", 31, [58, 98])
        },
    );
    all.insert(GOOD_LUCK, entry("C09spA0.def", 15, [50, 97]));
    all.insert(GOOD_MORALE, entry("C09spW0.def", 20, [94, 127]));
    all.insert(BAD_MORALE, entry("C14spE0.def", 20, [90, 125]));
    all
}

/// The timeline of `snapshot` and `commands` with the rolled effects' animations, `frighteners`
/// the creatures that frighten.
fn play_rolls(
    snapshot: &Observation,
    commands: &[(Option<u32>, Vec<Event>)],
    magic: &BTreeMap<u8, MagicLook>,
    frighteners: &[u16],
) -> Timeline {
    let layout = layout();
    let casts = Casts {
        frighteners: frighteners.iter().copied().collect(),
        ..Casts::default()
    };
    let stage = Stage {
        layout: &layout,
        width: 17,
        groups: tables().battle.groups,
        magic,
        casts: &casts,
        bugs: Bugs::Original,
        siege: None,
    };
    let commands: Vec<Command<'_>> = commands
        .iter()
        .map(|(at, events)| Command { at: *at, events })
        .collect();
    timeline(snapshot, &commands, &looks(), stage, Speed::default()).unwrap()
}

/// A recorded timeline without the original's own fidgets (the idle step's, which the viewer's
/// timeline check leaves out too).
fn fidgetless(lines: &str) -> String {
    unfidgeted(lines, &BTreeSet::new())
}

#[test]
fn the_luck_sequence_plays_as_recorded() {
    // E17 M3 T7b-2b-2b (`seq-luck.toml`): the Archangel flies to the Pikemen and strikes them, a
    // lucky strike: `goodluck.82m`, the line, then animation 18 over it, the strike 1 600 ms
    // later than `strike`'s.
    let mut pikemen = at(3, Side::Defender, PIKEMAN, 49, false);
    pikemen.count = 10;
    let mut archangel = at(0, Side::Attacker, ARCHANGEL, 35, true);
    archangel.flies = true;
    let o = snapshot(vec![
        archangel,
        at(1, Side::Attacker, GOBLIN, 86, false),
        at(2, Side::Attacker, GNOLL, 137, false),
        pikemen,
        at(4, Side::Defender, GOBLIN, 151, false),
    ]);
    let lucky = Strike {
        lucky: true,
        ..flags(false, false, false)
    };
    let events = vec![
        moved(0, 35, 47),
        struck(0, 3, 10, lucky),
        Event::TurnStarted { stack: 4 },
    ];
    let t = play_rolls(&o, &[(None, events)], &roll_magic(), &[]);
    // To the strike's first frame: the tests' Archangel and Pikemen strike and die by looks of
    // their own (`h3-view sequences luck` checks the rest with the original's DEFs).
    assert_eq!(until(&t.lines(), 4330), until(LUCK, 4330));
    let luck = Logged::Luck {
        stack: 0,
        count: 40,
    };
    let at_luck = t.log.iter().position(|l| *l == (2730, luck.clone()));
    let at_strike = t
        .log
        .iter()
        .position(|(_, l)| matches!(l, Logged::Strike { .. }));
    assert!(at_luck < at_strike, "{:?}", t.log);
    assert!(cues(&t).contains(&(2730, 0, Sound::Luck, false)));
}

#[test]
fn the_morale_sequence_plays_as_recorded() {
    // E17 M3 T7b-2b-2b (`seq-morale.toml`): `shot` with the first shot lucky (animation 18
    // before it), then good morale after the second (`GoodMrle.wav`, animation 20, the line once
    // it is erased, a redraw) and the Marksmen's turn again.
    let o = snapshot(vec![
        at(0, Side::Attacker, MARKSMAN, 35, false),
        at(1, Side::Attacker, PIKEMAN, 137, false),
        at(2, Side::Defender, GOBLIN, 49, false),
        at(3, Side::Defender, GNOLL, 151, false),
    ]);
    let shot = flags(true, false, false);
    let lucky = Strike {
        lucky: true,
        ..shot
    };
    let events = vec![
        struck(0, 2, 4, lucky),
        struck(0, 2, 3, shot),
        Event::GoodMorale { stack: 0 },
        Event::TurnStarted { stack: 0 },
    ];
    let t = play_rolls(&o, &[(None, events)], &roll_magic(), &[]);
    assert_eq!(t.lines(), MORALE);
    let good = Logged::Morale {
        stack: 0,
        count: 40,
        good: true,
    };
    assert!(t.log.contains(&(6876, good)), "{:?}", t.log);
    let cued = cues(&t);
    assert!(cued.contains(&(0, 0, Sound::Luck, false)));
    assert!(cued.contains(&(4776, 0, Sound::GoodMorale, false)));
    assert_eq!([t.active(6875), t.active(6876)], [None, Some(0)]);
    // No outline from the luck's first frame shown (the shot's handler); the Marksmen's again
    // from good morale's first frame (`morale`'s frames).
    assert_eq!(
        [0, 100, 4875, 4876].map(|ms| t.outlined(ms)),
        [Some(0), None, None, Some(0)]
    );
}

#[test]
fn the_bad_morale_and_fear_sequences_play_as_recorded() {
    // E17 M3 T7b-2b-2b (`seq-bad.toml`): the Champions defend; the Goblins' pick freezes them:
    // `BadMrle.wav`, the line, animation 30 with their hit; they never become the active stack,
    // the Dwarves' turn starts once the animation is erased.
    let o = snapshot(vec![
        at(0, Side::Attacker, CHAMPION, 35, true),
        at(1, Side::Attacker, GOBLIN, 86, false),
        at(2, Side::Attacker, GNOLL, 137, false),
        at(3, Side::Defender, PIKEMAN, 100, false),
    ]);
    let events = vec![
        Event::Defended { stack: 0 },
        Event::TurnStarted { stack: 1 },
        Event::BadMorale { stack: 1 },
        Event::TurnStarted { stack: 3 },
    ];
    let t = play_rolls(&o, &[(None, events)], &roll_magic(), &[]);
    assert_eq!(t.lines(), fidgetless(BAD));
    let bad = Logged::Morale {
        stack: 1,
        count: 40,
        good: false,
    };
    assert!(t.log.contains(&(0, bad)));
    assert!(!t.log.contains(&(0, Logged::Turn(1))));
    assert!(cues(&t).contains(&(0, 1, Sound::BadMorale, false)));
    assert_eq!([t.active(2099), t.active(2100)], [None, Some(3)]);
    // The Champions, who defended, stay outlined through the freeze (`bad`'s frames).
    assert_eq!([t.outlined(1000), t.outlined(2100)], [Some(0), Some(3)]);
    // `seq-fear.toml`: the Azure Dragon defends; the Champions' pick frightens them: `Fear.wav`,
    // the line (the Dragon by its number), animation 15 with their hit; the Marksmen's turn.
    let mut dragon = at(2, Side::Defender, BLACK_DRAGON, 100, true);
    dragon.count = 1;
    let mut o = snapshot(vec![
        at(0, Side::Attacker, CHAMPION, 35, true),
        at(1, Side::Attacker, MARKSMAN, 137, false),
        dragon,
    ]);
    o.active = Some(2);
    let events = vec![
        Event::Defended { stack: 2 },
        Event::TurnStarted { stack: 0 },
        Event::Frightened { stack: 0 },
        Event::TurnStarted { stack: 1 },
    ];
    let t = play_rolls(&o, &[(None, events)], &roll_magic(), &[BLACK_DRAGON]);
    assert_eq!(t.lines(), fidgetless(FEAR));
    let fear = Logged::Fear {
        stack: 0,
        count: 40,
        by: 2,
        number: 1,
    };
    assert!(t.log.contains(&(0, fear)));
    assert!(cues(&t).contains(&(0, 0, Sound::Fear, false)));
    assert_eq!([t.active(3199), t.active(3200)], [None, Some(1)]);
    assert_eq!([t.outlined(1000), t.outlined(3200)], [Some(2), Some(1)]);
}

#[test]
fn rolled_effects_without_their_stack_or_a_long_enough_animation() {
    // E17 M3 T7b-2b-2b: two frightening stacks name the first by their sum; a hit longer than
    // bad morale's animation shows the standing frame at once, with no animation frame beside
    // the hit's last ones (not met in the original's recordings: *provisional*).
    let o = snapshot(vec![
        at(0, Side::Attacker, CHAMPION, 35, true),
        at(1, Side::Defender, BLACK_DRAGON, 100, true),
        at(2, Side::Defender, BLACK_DRAGON, 151, true),
    ]);
    let mut magic = roll_magic();
    if let Some(bad) = magic.get_mut(&BAD_MORALE) {
        bad.frames = 3;
    }
    let events = vec![
        Event::Frightened { stack: 0 },
        // A round's start: no stack outlined during the next freeze.
        Event::RoundStarted { round: 2 },
        Event::BadMorale { stack: 0 },
        // Stacks the timeline does not follow.
        Event::GoodMorale { stack: 9 },
        Event::BadMorale { stack: 9 },
        Event::Frightened { stack: 9 },
        struck(
            9,
            1,
            0,
            Strike {
                lucky: true,
                ..flags(true, false, false)
            },
        ),
    ];
    let t = play_rolls(&o, &[(Some(0), events)], &magic, &[BLACK_DRAGON]);
    let fear = Logged::Fear {
        stack: 0,
        count: 40,
        by: 1,
        number: 80,
    };
    assert!(t.log.iter().any(|(_, l)| *l == fear));
    // Fear's animation: the Champions' 8 hit frames, then 23 more and the erasing redraw; bad
    // morale's: 8 hit frames beside 3 of the animation and 5 without, the standing frame shown
    // at once with the last, then the erasing redraw.
    let bad = t.magics.iter().filter(|k| k.animation == BAD_MORALE);
    let frames: Vec<Option<u32>> = bad.map(|k| k.frame).collect();
    assert_eq!(
        frames,
        [
            Some(0),
            Some(1),
            Some(2),
            None,
            None,
            None,
            None,
            None,
            None
        ]
    );
    // Fear's is drawn to 3 100; bad morale's hit frames are drawn from 3 200 and shown a
    // deadline later, the last at 4 000 with the standing frame at once.
    assert_eq!(t.at(3999)[&0], pose(3, 6, 35));
    assert_eq!(t.at(4000)[&0], still(2, 35));
    assert_eq!([t.outlined(100), t.outlined(3300)], [Some(0), None]);
    assert!(t.log.iter().all(|(_, l)| !matches!(
        l,
        Logged::Morale { stack: 9, .. }
            | Logged::Fear { stack: 9, .. }
            | Logged::Luck { stack: 9, .. }
    )));
}

/// The magic animations of the striker's effects and the spells after a strike (E17 M3
/// T7b-2b-3): the bolt from the sky (one frame, above the creature), 49 (12 frames), Poison's 67
/// (5), Life Drain's 74 (4, translucent), Death Stare's 80 (3) and the acid's 81 (4).
fn strike_magic() -> BTreeMap<u8, MagicLook> {
    let entry = |def: &str, frames, place| MagicLook {
        def: def.to_string(),
        frames,
        size: [80, 90],
        place,
        translucent: false,
    };
    let mut all = spell_magic();
    all.insert(BOLT, entry("C11spA0.def", 1, 2));
    all.insert(THUNDERBOLT, entry("C11spA1.def", 12, 0));
    all.insert(67, entry("sp11_.def", 5, 1));
    all.insert(
        LIFE_DRAIN,
        MagicLook {
            translucent: true,
            ..entry("sp06_.def", 4, 1)
        },
    );
    all.insert(DEATH_STARE, entry("c07spe0.def", 3, 1));
    all.insert(ACID, entry("c0acid.def", 4, 1));
    all
}

/// The casts' spells and the strikes': Lightning Bolt (17, its 49), Poison (71, 67), Blind (62,
/// pending but not built), Death Stare (79, 80) and Acid Breath (80, 81) by their own numbers.
fn strike_casts() -> Casts {
    let mut casts = casts();
    let spell = |original, animation| SpellLook {
        original,
        sign: -1,
        value: [0; 2],
        animation: Some(animation),
        ..SpellLook::default()
    };
    for (original, animation) in [(17, 49), (71, 67), (62, 12), (79, 80), (80, 81)] {
        casts.spells.insert(original, spell(original, animation));
    }
    casts
}

/// The timeline of `snapshot` and `commands` with the strikes' spells and animations.
fn play_strikes(
    snapshot: &Observation,
    commands: &[(Option<u32>, Vec<Event>)],
    casts: &Casts,
) -> Timeline {
    let layout = layout();
    let magic = strike_magic();
    let stage = Stage {
        layout: &layout,
        width: 17,
        groups: tables().battle.groups,
        magic: &magic,
        casts,
        bugs: Bugs::Original,
        siege: None,
    };
    let commands: Vec<Command<'_>> = commands
        .iter()
        .map(|(at, events)| Command { at: *at, events })
        .collect();
    timeline(snapshot, &commands, &looks(), stage, Speed::default()).unwrap()
}

/// The animations `t` drew, each by its number as its frame 0 is drawn, in order.
fn animations(t: &Timeline) -> Vec<u8> {
    t.magics
        .iter()
        .filter(|k| k.frame == Some(0))
        .map(|k| k.animation)
        .collect()
}

fn strike_spell(striker: u8, target: u8, spell: u16) -> Event {
    Event::StrikeSpell {
        striker,
        target,
        spell,
    }
}

#[test]
fn spells_after_a_strike_are_cast_at_its_end_and_the_acid_after_its_line() {
    // E17 M3 T7b-2b-3 (`acid`): the Pikemen strike the Vampire Lords; Poison and Acid Breath,
    // pending, are cast at the strike's animation's end: Poison's sound, effect, line, then 67
    // with the stack's hit; Acid Breath's sound, 81, the defense lowered by 3 and its line, no
    // spell set (the box's hue Poison's alone); then the strike's line; then the acid: its sound,
    // its damage's line, 81 again, the stack's reaction.
    let mut lords = at(1, Side::Defender, VAMPIRE_LORD, 100, false);
    lords.defense = 10;
    let o = snapshot(vec![at(0, Side::Attacker, PIKEMAN, 99, false), lords]);
    let melee = flags(false, false, false);
    let both = vec![
        struck(0, 1, 0, melee),
        strike_spell(0, 1, 71),
        applied(1, 71),
        strike_spell(0, 1, 80),
        applied(1, 80),
        Event::StrikeDamage {
            striker: 0,
            target: 1,
            damage: WireU64(25),
            kills: 1,
        },
        // Acid Breath outside its strike: no spell the hue counts either.
        strike_spell(1, 0, 80),
        applied(0, 80),
    ];
    let t = play_strikes(&o, &[(None, both)], &strike_casts());
    let kinds: Vec<&Logged> = t.log.iter().map(|(_, l)| l).collect();
    let poisoned = Logged::Poisoned {
        stack: 1,
        count: 40,
    };
    let acid = Logged::AcidBreath {
        stack: 1,
        count: 40,
        lowered: 3,
    };
    let damage = Logged::SpellDamage {
        spell: 80,
        hit: (1, 25, 1),
    };
    let at_poison = kinds.iter().position(|l| **l == poisoned);
    let at_acid = kinds.iter().position(|l| **l == acid);
    let at_strike = kinds
        .iter()
        .position(|l| matches!(l, Logged::Strike { .. }));
    let at_damage = kinds.iter().position(|l| **l == damage);
    assert!(at_poison < at_acid && at_acid < at_strike && at_strike < at_damage);
    assert!(at_poison.is_some(), "{:?}", t.log);
    assert_eq!(animations(&t), [67, ACID, ACID]);
    // Poison's first frame beside the stack's hit group's frame 0.
    let first = t.magics.iter().find(|k| k.animation == 67).map(|k| k.shown);
    let first = first.unwrap_or_default();
    assert_eq!(t.at(first)[&1].group, 3);
    let sounds: Vec<Sound> = cues(&t).iter().map(|c| c.2).collect();
    for s in [Sound::Spell(71), Sound::Spell(80), Sound::SpellFile(80)] {
        assert!(sounds.contains(&s), "{sounds:?}");
    }
    // The hue: Poison's alone, as without Acid Breath.
    let poison_only = vec![
        struck(0, 1, 0, melee),
        strike_spell(0, 1, 71),
        applied(1, 71),
    ];
    let p = play_strikes(&o, &[(None, poison_only)], &strike_casts());
    assert_eq!(t.hues(20_000), p.hues(20_000));
    // The kill shows once the acid's reaction is over.
    assert_eq!(t.counts(20_000)[&1], 39);
    // Each cast outlines the turn's stack at its end (`cast_spell`'s tail), the second clearing
    // it as it starts: outlined as Acid Breath's line is written, not while it plays.
    let written = t.log.iter().find(|(_, l)| *l == acid).map(|(at, _)| *at);
    let written = written.unwrap_or_default();
    let started = t
        .magics
        .iter()
        .filter(|k| k.animation == ACID)
        .map(|k| k.shown)
        .min();
    assert_eq!(t.outlined(written), Some(0));
    assert_eq!(t.outlined(started.unwrap_or_default()), None);
}

#[test]
fn the_thunderbirds_bolt_and_a_heros_lightning_bolt_fall_from_the_sky() {
    // E17 M3 T7b-2b-3 (`thunder`, `lightning`): after the strike's line, the Thunderbird's bolt:
    // its line, its sound, 1 for 10 ms, the damage's line, then 49 over the marked stack. A
    // hero's Lightning Bolt: 1, then 49, its line after it.
    let o = snapshot(vec![
        at(0, Side::Attacker, PIKEMAN, 99, false),
        at(1, Side::Defender, VAMPIRE_LORD, 100, false),
    ]);
    let events = vec![
        struck(0, 1, 0, flags(false, false, false)),
        strike_spell(0, 1, 17),
        spell_damage(1, 20, 1),
    ];
    let t = play_strikes(&o, &[(None, events)], &strike_casts());
    assert_eq!(animations(&t), [BOLT, THUNDERBOLT]);
    let bolt: Vec<(u32, u32)> = t
        .magics
        .iter()
        .filter(|k| k.animation == BOLT)
        .map(|k| (k.drawn, k.shown))
        .collect();
    let (drawn, shown) = bolt[0];
    assert_eq!(bolt[1].1, shown + BOLT_TIME);
    let line = Logged::Thunderbolt {
        stack: 1,
        count: 40,
    };
    assert!(t.log.contains(&(drawn, line)), "{:?}", t.log);
    // Its damage's line once the erasing redraw shows.
    let damage = (
        bolt[1].1,
        Logged::SpellDamage {
            spell: 17,
            hit: (1, 20, 1),
        },
    );
    assert!(t.log.contains(&damage), "{:?}", t.log);
    assert!(cues(&t).contains(&(drawn, 1, Sound::Thunderbolt, false)));
    // The hero's.
    let o = snapshot(vec![
        at(0, Side::Attacker, MARKSMAN, 86, false),
        at(1, Side::Defender, SWORDSMAN, 83, false),
    ]);
    let events = vec![cast(17, 1), spell_damage(1, 35, 1)];
    let t = play_strikes(&o, &[(None, events)], &strike_casts());
    assert_eq!(animations(&t), [BOLT, THUNDERBOLT]);
    let last = t.magics.iter().rfind(|k| k.animation == THUNDERBOLT);
    let erased = last.map(|k| k.shown).unwrap_or_default();
    let line = Logged::SpellDamage {
        spell: 17,
        hit: (1, 35, 1),
    };
    assert!(
        t.log.iter().any(|(at, l)| *l == line && *at >= erased),
        "{:?}",
        t.log
    );
}

#[test]
fn life_drain_and_death_stare_follow_the_strike() {
    // E17 M3 T7b-2b-3 (`thunder`, `stare`): the Vampire Lords strike back and drain the Pikemen:
    // the line by their number before the raised and the Pikemen's before the strike, the sound,
    // 74 over the drinkers, their box their new number from its first frame; then Death Stare
    // (as if theirs): the line, Death Stare's sound, 80 over the Pikemen, then their reaction.
    let o = snapshot(vec![
        at(0, Side::Attacker, PIKEMAN, 99, false),
        at(1, Side::Defender, VAMPIRE_LORD, 100, false),
    ]);
    let events = vec![
        struck(0, 1, 1, flags(false, false, false)),
        struck(1, 0, 2, flags(false, true, false)),
        Event::LifeDrained {
            stack: 1,
            hit_points: WireU64(50),
            resurrected: 1,
        },
        Event::DeathStared {
            attacker: 1,
            target: 0,
            kills: 1,
        },
    ];
    let t = play_strikes(&o, &[(None, events)], &strike_casts());
    let drain = Logged::Drain {
        stack: 1,
        count: 39,
        hit_points: 50,
        raised: 1,
        target: 0,
        target_count: 40,
    };
    let stare = Logged::DeathStare {
        stack: 1,
        count: 40,
        target: 0,
        kills: 1,
    };
    assert!(t.log.iter().any(|(_, l)| *l == drain), "{:?}", t.log);
    assert!(t.log.iter().any(|(_, l)| *l == stare), "{:?}", t.log);
    assert_eq!(animations(&t), [LIFE_DRAIN, DEATH_STARE]);
    let first = |n: u8| t.magics.iter().find(|k| k.animation == n).map(|k| k.shown);
    let drink = first(LIFE_DRAIN).unwrap_or_default();
    assert_eq!(t.counts(drink - 1)[&1], 39);
    assert_eq!(t.counts(drink)[&1], 40);
    let sounds: Vec<Sound> = cues(&t).iter().map(|c| c.2).collect();
    assert!(sounds.contains(&Sound::Drain) && sounds.contains(&Sound::SpellFile(79)));
    assert_eq!(t.counts(20_000)[&0], 37);
}

#[test]
fn strike_effects_without_their_spell_or_stack_do_what_they_can() {
    // E17 M3 T7b-2b-3: a pending spell not built (Blind) and one on a stack the timeline does
    // not follow take effect without animations; another striker's pending spell, a spell that
    // is no strike's and a bolt without its damage wait for the event loop; the acid and Death
    // Stare without their spells in the record play their animations without a sound or line;
    // Life Drain without a melee strike before it, or by a stack the timeline does not follow,
    // does nothing.
    let o = snapshot(vec![
        at(0, Side::Attacker, PIKEMAN, 99, false),
        at(1, Side::Defender, VAMPIRE_LORD, 100, false),
    ]);
    let drained = Event::LifeDrained {
        stack: 1,
        hit_points: WireU64(5),
        resurrected: 0,
    };
    let melee = flags(false, false, false);
    let commands = [
        (None, vec![drained.clone()]),
        (
            Some(0),
            vec![
                struck(0, 1, 0, melee),
                strike_spell(0, 1, 62),
                applied(1, 62),
                strike_spell(0, 9, 71),
                strike_spell(1, 0, 71),
                applied(0, 71),
                strike_spell(0, 1, 45),
                strike_spell(0, 1, 17),
                strike_spell(0, 9, 17),
                spell_damage(9, 20, 1),
                Event::StrikeDamage {
                    striker: 0,
                    target: 1,
                    damage: WireU64(25),
                    kills: 0,
                },
                Event::StrikeDamage {
                    striker: 0,
                    target: 9,
                    damage: WireU64(25),
                    kills: 0,
                },
                Event::DeathStared {
                    attacker: 1,
                    target: 0,
                    kills: 1,
                },
                Event::DeathStared {
                    attacker: 9,
                    target: 0,
                    kills: 1,
                },
                Event::LifeDrained {
                    stack: 9,
                    hit_points: WireU64(5),
                    resurrected: 0,
                },
            ],
        ),
    ];
    let mut casts = strike_casts();
    casts.spells.remove(&79);
    casts.spells.remove(&80);
    let t = play_strikes(&o, &commands, &casts);
    assert_eq!(animations(&t), [ACID, DEATH_STARE]);
    assert!(
        t.log.iter().all(|(_, l)| !matches!(
            l,
            Logged::Drain { .. }
                | Logged::Thunderbolt { .. }
                | Logged::SpellDamage { .. }
                | Logged::Poisoned { stack: 9, .. }
        )),
        "{:?}",
        t.log
    );
    let sounds: Vec<Sound> = cues(&t).iter().map(|c| c.2).collect();
    assert!(!sounds.iter().any(|s| matches!(
        s,
        Sound::SpellFile(_) | Sound::Drain | Sound::Spell(_) | Sound::Thunderbolt
    )));
    // Blind's and the other striker's Poison took effect: the boxes' hues changed.
    assert_ne!(t.hues(20_000), BTreeMap::new());
}

#[test]
fn spells_after_a_strike_without_animations_and_a_drain_raising_none() {
    // E17 M3 T7b-2b-3: Poison and Acid Breath whose records name no animation write their lines
    // all the same; a drain that raises nobody leaves the box's number as it is.
    let o = snapshot(vec![
        at(0, Side::Attacker, PIKEMAN, 99, false),
        at(1, Side::Defender, VAMPIRE_LORD, 100, false),
    ]);
    let mut casts = strike_casts();
    for spell in [71, 80] {
        if let Some(look) = casts.spells.get_mut(&spell) {
            look.animation = None;
        }
    }
    let events = vec![
        struck(0, 1, 0, flags(false, false, false)),
        strike_spell(0, 1, 71),
        strike_spell(0, 1, 80),
        struck(1, 0, 0, flags(false, true, false)),
        Event::LifeDrained {
            stack: 1,
            hit_points: WireU64(3),
            resurrected: 0,
        },
    ];
    let t = play_strikes(&o, &[(None, events)], &casts);
    assert_eq!(animations(&t), [LIFE_DRAIN]);
    let lines: Vec<&Logged> = t.log.iter().map(|(_, l)| l).collect();
    assert!(lines.iter().any(|l| matches!(l, Logged::Poisoned { .. })));
    assert!(lines.iter().any(|l| matches!(l, Logged::AcidBreath { .. })));
    assert!(t.counts.iter().all(|k| k.stack != 1));
}

/// The stage of the spells aimed at no stack (E17 M3 T7c-1): the casts' with Death Ripple (24,
/// animation 8, 15 frames: `C04spE0.def`), Destroy Undead (25, 29), Fireball (21, 53: 13 frames
/// centred 37 above the anchor), Inferno (22, 9: 16 frames, here from the hex's left), Meteor
/// Shower (23, 16: on the cell's lower edge) and Frost Ring (20, no animation).
fn area_magic() -> BTreeMap<u8, MagicLook> {
    let entry = |def: &str, frames, size, place| MagicLook {
        def: def.to_string(),
        frames,
        size,
        place,
        translucent: false,
    };
    let mut all = spell_magic();
    all.insert(8, entry("C04spE0.def", 15, [100, 100], 1));
    all.insert(29, entry("C14spA0.def", 10, [80, 80], 1));
    all.insert(53, entry("C13spF.def", 13, [120, 110], 1));
    all.insert(9, entry("C04spF0.def", 16, [90, 70], 4));
    all.insert(16, entry("C08spE0.def", 4, [60, 50], 0));
    all
}

fn area_casts() -> Casts {
    let mut casts = casts();
    let spell = |original, animation| SpellLook {
        original,
        sign: -1,
        value: [0; 2],
        animation,
        ..SpellLook::default()
    };
    casts.spells.extend([
        (24, spell(24, Some(8))),
        (25, spell(25, Some(29))),
        (21, spell(21, Some(53))),
        (22, spell(22, Some(9))),
        (23, spell(23, Some(16))),
        (20, spell(20, None)),
    ]);
    casts
}

/// A Faerie Dragon (two hexes): standing 1 frame, its hit 7, its casting groups 17 to 19 of 12
/// each, its attack time 480 (40 ms a frame); and the casts' creatures.
fn area_looks() -> BTreeMap<u16, Look> {
    let mut looks = looks();
    looks.insert(
        FAERIE_DRAGON,
        look(
            [9000, 400, 480, 300],
            &[
                (2, 1),
                (3, 7),
                (7, 2),
                (8, 2),
                (9, 2),
                (10, 2),
                (17, 12),
                (18, 12),
                (19, 12),
            ],
            100,
        ),
    );
    looks
}

fn play_area(snapshot: &Observation, commands: &[(Option<u32>, Vec<Event>)]) -> Timeline {
    play_area_with(snapshot, commands, Bugs::Original)
}

/// As [`play_area`], the original's drawing bugs reproduced or fixed by `bugs` (E17 M3 T7d).
fn play_area_with(
    snapshot: &Observation,
    commands: &[(Option<u32>, Vec<Event>)],
    bugs: Bugs,
) -> Timeline {
    let layout = layout();
    let magic = area_magic();
    let casts = area_casts();
    let stage = Stage {
        layout: &layout,
        width: 17,
        groups: tables().battle.groups,
        magic: &magic,
        casts: &casts,
        bugs,
        siege: None,
    };
    let commands: Vec<Command<'_>> = commands
        .iter()
        .map(|(at, events)| Command { at: *at, events })
        .collect();
    timeline(snapshot, &commands, &area_looks(), stage, Speed::default())
        .unwrap()
        .with_heroes(cast_heroes())
}

fn cast_at(spell: u16, target: Target) -> Event {
    Event::SpellCast {
        side: Side::Attacker,
        spell,
        target,
        second: Target::None {},
    }
}

fn hurt(target: u8, caster: u64, damage: u64, kills: u32) -> Event {
    Event::SpellDamage {
        target,
        value: WireU64(caster),
        caster: WireU64(caster),
        damage: WireU64(damage),
        kills,
    }
}

#[test]
fn the_mass_sequence_plays_as_recorded() {
    let (o, commands) = mass_sequence();
    let t = play_area(&o, &commands);
    assert_eq!(t.lines_with_heroes(5300), until(&screenless(MASS), 5300));
    // The lines: the casts' before their animations, Death Ripple's damage after its cast's.
    let lines: Vec<(u32, &Logged)> = t
        .log
        .iter()
        .filter(|(_, l)| matches!(l, Logged::CastAll { .. } | Logged::MassDamage { .. }))
        .map(|(at, l)| (*at, l))
        .collect();
    assert_eq!(
        lines,
        [
            (500, &Logged::CastAll { side: 0, spell: 54 }),
            (3300, &Logged::CastAll { side: 0, spell: 24 }),
            (
                3300,
                &Logged::MassDamage {
                    spell: 24,
                    damage: 35,
                    undead: false
                }
            ),
        ]
    );
    // The animation shows its frame 18 twice and goes with the hero's frame 5; the boxes show the
    // kills with it too, after Death Ripple (the cast's tail).
    assert_eq!(t.magic(2400).map(|m| m.frame), Some(18));
    assert_eq!(t.magic(2599).map(|m| m.frame), Some(18));
    assert_eq!(t.magic(2600), None);
    assert_eq!(t.counts(4899).get(&1), None);
    assert_eq!(t.counts(4900).get(&1), Some(&39));
    assert_eq!(t.counts(4900).get(&0), Some(&37));
}

/// A command's time (none: after the one before) and its events, as the players here take them.
type Played = (Option<u32>, Vec<Event>);

/// E17 M3 T7c-1 (`seq-mass.toml`): Orrin's Slow at Expert on the four Swordsmen (round 1), their
/// defends, Death Ripple on every stack with their reactions (round 2).
fn mass_sequence() -> (Observation, Vec<Played>) {
    let o = snapshot(vec![
        at(0, Side::Attacker, MARKSMAN, 86, false),
        at(1, Side::Defender, SWORDSMAN, 15, false),
        at(2, Side::Defender, SWORDSMAN, 83, false),
        at(3, Side::Defender, SWORDSMAN, 117, false),
        at(4, Side::Defender, SWORDSMAN, 185, false),
    ]);
    let none = Target::None {};
    let mut slow = vec![cast_at(54, none)];
    slow.extend((1..5).map(|t| applied(t, 54)));
    let defend = |stack| {
        vec![
            Event::Defended { stack },
            Event::TurnStarted { stack: stack + 1 },
        ]
    };
    let mut round = vec![
        Event::Defended { stack: 4 },
        Event::RoundStarted { round: 2 },
    ];
    round.extend((1..5).map(|target| Event::EffectEnded { target, spell: 54 }));
    round.push(Event::TurnStarted { stack: 0 });
    let mut ripple = vec![cast_at(24, none), hurt(0, 35, 35, 3)];
    ripple.extend((1..5).map(|t| hurt(t, 35, 35, 1)));
    let commands = vec![
        (Some(0), slow),
        (Some(2800), defend(0)),
        (Some(2800), defend(1)),
        (Some(2800), defend(2)),
        (Some(2800), defend(3)),
        (Some(2800), round),
        (Some(2800), ripple),
    ];
    (o, commands)
}

#[test]
fn mass_animations_with_the_bugs_fixed() {
    // E17 M3 T9d: `mass`'s casts with the original's slips fixed. CLT-Q9: Slow's animation shows
    // its last frame (19) once, where the original shows the one before it twice. CLT-Q10: the
    // Marksmen's hit by Death Ripple ends on the standing group's frame 0; the original's runs on
    // into the standing group.
    let (o, commands) = mass_sequence();
    let original = play_area(&o, &commands);
    let fixed = play_area_with(&o, &commands, Bugs::Fixed);
    let frame = |t: &Timeline, at| t.magic(at).map(|m| m.frame);
    assert_eq!(
        [2400, 2500, 2600].map(|at| frame(&original, at)),
        [Some(18), Some(18), None]
    );
    assert_eq!(
        [2400, 2500, 2600].map(|at| frame(&fixed, at)),
        [Some(18), Some(19), None]
    );
    let pose = |t: &Timeline| t.at(t.end).get(&0).map(|p| (p.group, p.frame));
    assert_eq!(pose(&fixed), Some((2, 0)));
    assert!(pose(&original).is_some_and(|(group, frame)| group == 2 && frame > 0));
    assert_eq!(original.end, fixed.end);
}

#[test]
fn spells_at_no_stack_play_what_they_strike() {
    // E17 M3 T7c-1: Death Ripple on one stack (its own damage in the line), on none (no line, no
    // animation), killing one (its corpse, a redraw); Destroy Undead's line after its animation;
    // a timed spell without effects, an area spell aimed at no hex and Frost Ring without an
    // animation take effect at once or play their reactions alone.
    let o = snapshot(vec![
        at(0, Side::Attacker, MARKSMAN, 86, false),
        at(1, Side::Defender, SWORDSMAN, 15, false),
        at(2, Side::Defender, SWORDSMAN, 83, false),
    ]);
    let none = Target::None {};
    let commands = [
        (Some(0), vec![cast_at(24, none), hurt(1, 35, 30, 1)]),
        (Some(4000), vec![cast_at(24, none)]),
        (
            Some(8000),
            vec![cast_at(25, none), hurt(1, 20, 20, 40), hurt(2, 20, 20, 1)],
        ),
        (Some(12000), vec![cast_at(54, none)]),
        (Some(16000), vec![cast_at(21, none), hurt(2, 10, 10, 0)]),
        (
            Some(20000),
            vec![cast_at(20, Target::Hex { hex: 83 }), hurt(2, 10, 10, 0)],
        ),
    ];
    let t = play_area(&o, &commands);
    let damage: Vec<(u64, bool)> = t
        .log
        .iter()
        .filter_map(|(_, l)| match l {
            Logged::MassDamage { damage, undead, .. } => Some((*damage, *undead)),
            _ => None,
        })
        .collect();
    assert_eq!(damage, [(30, false), (20, true)]);
    // Destroy Undead: its line after its animation; the Swordsmen at 15 die and lie as a corpse.
    let undead = t
        .log
        .iter()
        .find(|(_, l)| matches!(l, Logged::MassDamage { undead: true, .. }))
        .map(|(at, _)| *at)
        .unwrap();
    let last = t
        .magics
        .iter()
        .filter(|k| k.animation == 29 && k.frame.is_some())
        .map(|k| k.drawn)
        .max();
    assert!(last.is_some_and(|at| at < undead));
    let corpse = t.keys.iter().rev().find(|k| k.stack == 1).unwrap();
    assert_eq!((corpse.pose.group, corpse.pose.layer), (5, CORPSE));
    // Frost Ring: the reactions alone, its line on the one stack it struck.
    assert!(t.log.iter().any(|(_, l)| *l
        == Logged::AreaDamage {
            spell: 20,
            damage: 10,
            kills: 0,
            target: Some(2)
        }));
    assert!(t.spots.is_empty());
}

#[test]
fn an_area_spell_plays_at_its_hex_then_its_line_and_the_reactions() {
    // E17 M3 T7c-1 (0x5A4970): a hero's Fireball at 83 (centred 37 above the anchor), Inferno at
    // 117 (from the hex's left on the cell's lower edge), Meteor Shower at 15 (centred on that
    // edge); a defending stack's group plays on after the loop, a hit stands after its last frame.
    let o = snapshot(vec![
        at(0, Side::Attacker, MARKSMAN, 86, false),
        at(1, Side::Defender, SWORDSMAN, 15, false),
        at(2, Side::Defender, SWORDSMAN, 83, false),
    ]);
    let hex = |hex| Target::Hex { hex };
    let commands = [
        (
            Some(0),
            vec![
                Event::Defended { stack: 0 },
                Event::TurnStarted { stack: 1 },
            ],
        ),
        (
            Some(100),
            vec![
                Event::Defended { stack: 1 },
                Event::TurnStarted { stack: 2 },
            ],
        ),
        (
            Some(200),
            vec![
                Event::Defended { stack: 2 },
                Event::TurnStarted { stack: 0 },
            ],
        ),
        (
            Some(300),
            vec![cast_at(21, hex(83)), hurt(1, 50, 50, 1), hurt(2, 50, 40, 2)],
        ),
        (Some(8000), vec![cast_at(22, hex(117))]),
        (Some(14000), vec![cast_at(23, hex(15)), hurt(1, 9, 9, 40)]),
    ];
    let t = play_area(&o, &commands);
    // Meteor Shower kills the Swordsmen at 15: they lie as a corpse on their death's last frame.
    let corpse = t.keys.iter().rev().find(|k| k.stack == 1).unwrap();
    assert_eq!(
        (corpse.pose.group, corpse.pose.frame, corpse.pose.layer),
        (5, 5, CORPSE)
    );
    let layout = layout();
    let anchor = layout.anchor(4, 15);
    let at = |shown| t.spot(shown).map(|s| (s.def, s.frame, s.at));
    // Fireball's frames from the hero's fifth frame on, each 100 ms, erased after the last.
    let first = t.spots.iter().find(|k| k.animation == 53).unwrap().shown;
    assert_eq!(
        at(first),
        Some((
            "C13spF.def".to_string(),
            0,
            [anchor[0] - 60, anchor[1] - 55 - 37]
        ))
    );
    assert_eq!(at(first + 1200).map(|s| s.1), Some(12));
    assert_eq!(at(first + 1300), None);
    // Its line: several stacks, the caster's damage and every kill, before the reactions.
    let line = t
        .log
        .iter()
        .find(|(_, l)| matches!(l, Logged::AreaDamage { spell: 21, .. }))
        .unwrap();
    assert_eq!(
        *line,
        (
            first + 1300,
            Logged::AreaDamage {
                spell: 21,
                damage: 50,
                kills: 3,
                target: None
            }
        )
    );
    // The defending Swordsmen's group 4 (11 frames) plays on after the six of the loop.
    let poses = |stack| -> Vec<(u32, u32, u32)> {
        t.keys
            .iter()
            .filter(|k| k.stack == stack && k.drawn >= first + 1300 && k.drawn < 8000)
            .map(|k| (k.drawn, k.pose.group, k.pose.frame))
            .collect()
    };
    let swordsmen = poses(2);
    assert_eq!(swordsmen.first(), Some(&(first + 1300, 4, 0)));
    assert!(swordsmen.contains(&(first + 1300 + 1000, 4, 10)));
    assert_eq!(swordsmen.last().map(|p| (p.1, p.2)), Some((2, 0)));
    // Inferno at no stack: its animation alone, from the hex's left.
    let [left, top] = layout.hex_box(6, 15);
    let inferno = t.spots.iter().find(|k| k.animation == 9).unwrap();
    assert_eq!(inferno.at, [left, top + 52 - 70]);
    let shower = t.spots.iter().find(|k| k.animation == 16).unwrap();
    let [x, _] = layout.anchor(0, 15);
    let [_, top] = layout.hex_box(0, 15);
    assert_eq!(shower.at, [x - 30, top + 52 - 50]);
}

/// A creature's cast (`StackCast`).
fn stack_cast(stack: u8, spell: u16, target: Target) -> Event {
    Event::StackCast {
        stack,
        spell,
        target,
    }
}

#[test]
fn a_creature_casts_by_its_angle_and_stands_at_its_casts_tail() {
    // E17 M3 T7c-1 (0x448260, `dragon`): the Faerie Dragon beside its target takes group 17, its
    // frames at trunc(480 / 12), the outline kept until the cast starts; Fireball at the hex, the
    // dragon struck with the defending Swordsmen; it stands after the loop as their group 4 plays
    // on, outlined once nothing plays (the cast's tail).
    let o = snapshot(vec![
        at(0, Side::Attacker, FAERIE_DRAGON, 81, true),
        at(1, Side::Defender, SWORDSMAN, 83, false),
        at(2, Side::Defender, SWORDSMAN, 15, false),
        at(3, Side::Defender, SWORDSMAN, 100, false),
    ]);
    let hex = |hex| Target::Hex { hex };
    let commands = [
        (
            Some(0),
            vec![
                Event::Defended { stack: 1 },
                Event::TurnStarted { stack: 2 },
            ],
        ),
        (
            Some(0),
            vec![
                Event::Defended { stack: 2 },
                Event::TurnStarted { stack: 0 },
            ],
        ),
        (
            Some(1000),
            vec![
                stack_cast(0, 21, hex(83)),
                hurt(0, 20, 20, 0),
                hurt(1, 20, 20, 0),
                hurt(3, 20, 20, 0),
                Event::TurnStarted { stack: 1 },
            ],
        ),
        (
            Some(8000),
            vec![
                Event::Defended { stack: 1 },
                Event::TurnStarted { stack: 0 },
            ],
        ),
        (
            Some(8000),
            vec![stack_cast(0, 21, hex(15)), hurt(2, 20, 20, 0)],
        ),
    ];
    let t = play_area(&o, &commands);
    let frames = |group| -> Vec<u32> {
        t.keys
            .iter()
            .filter(|k| k.stack == 0 && k.pose.group == group)
            .map(|k| k.drawn)
            .collect()
    };
    let cast = frames(17);
    assert_eq!(cast.len(), 12);
    assert_eq!(cast.get(2).zip(cast.get(1)).map(|(b, a)| b - a), Some(40));
    // The outline kept while it casts, gone as the cast starts, back at its tail.
    let last = cast.last().copied().unwrap();
    assert_eq!(t.outlined(last), Some(0));
    assert_eq!(t.outlined(last + 41), None);
    let hit = frames(3);
    assert_eq!(hit.len(), 7);
    let stood = t
        .keys
        .iter()
        .find(|k| k.stack == 0 && k.pose.group == 2 && k.drawn > hit[0])
        .map(|k| k.drawn)
        .unwrap();
    assert_eq!(stood, hit[0] + 700);
    assert_eq!(t.outlined(stood + 100), None);
    // The Swordsmen at 100, not defending: their hit's six frames, standing inside the loop.
    let hit_100: Vec<(u32, u32)> = t
        .keys
        .iter()
        .filter(|k| k.stack == 3 && k.drawn >= hit[0] && k.drawn < stood)
        .map(|k| (k.pose.group, k.pose.frame))
        .collect();
    assert_eq!(hit_100.last(), Some(&(2, 0)));
    assert_eq!(hit_100.len(), 7);
    // The Swordsmen go on after the dragon stands; the outline is back as they stand.
    let defended: Vec<u32> = t
        .keys
        .iter()
        .filter(|k| k.stack == 1 && k.drawn > stood)
        .map(|k| k.drawn)
        .collect();
    assert_eq!(defended.len(), 5);
    // Its frames 8 to 10, standing, then the end's whole redraw.
    let end = defended.last().copied().unwrap();
    assert_eq!(t.outlined(end - 1), None);
    assert!(t.outlines.contains(&(end, Some(0))));
    // The next turn's stack the same moment.
    assert_eq!(t.outlined(end), Some(1));
    // Four rows above: group 19, the dragon not struck, standing in memory after the loop (six),
    // on the screen only with the battle's whole redraw once the Swordsmen stand.
    let above = frames(19);
    assert_eq!(above.len(), 12);
    let reacts = t
        .keys
        .iter()
        .find(|k| k.stack == 2 && k.pose.group == 4)
        .unwrap()
        .drawn;
    let stood = t
        .keys
        .iter()
        .find(|k| k.stack == 0 && k.pose.group == 2 && k.drawn > above[0])
        .map(|k| (k.drawn, k.shown));
    assert_eq!(stood, Some((reacts + 600, reacts + 1200)));
    assert_eq!(t.at(reacts + 1199).get(&0).map(|p| p.group), Some(19));
}

#[test]
fn a_creature_casts_turned_toward_its_target_and_by_its_strike_groups() {
    // E17 M3 T7c-1: straight below the dragon (Δx = 0) group 19, straight above 17; the
    // Swordsmen, without groups 17 to 19, cast by their strike groups (11 to 13), turned toward a
    // hex on their left and back; a cast at a stack, by an unknown stack or of another spell, and
    // an area spell that strikes nothing (the caster stands at its tail all the same).
    let o = snapshot(vec![
        at(0, Side::Attacker, FAERIE_DRAGON, 81, true),
        at(1, Side::Attacker, SWORDSMAN, 100, false),
        at(2, Side::Defender, SWORDSMAN, 150, false),
    ]);
    let hex = |hex| Target::Hex { hex };
    let commands = [
        (
            Some(0),
            vec![
                stack_cast(0, 23, hex(133)),
                Event::SpellResisted { target: 1 },
                hurt(2, 5, 5, 0),
            ],
        ),
        (Some(4000), vec![stack_cast(0, 23, hex(31))]),
        (Some(8000), vec![stack_cast(1, 23, hex(97))]),
        (
            Some(12000),
            vec![
                stack_cast(0, 54, Target::Stack { stack: 2 }),
                applied(2, 54),
            ],
        ),
        (Some(13000), vec![stack_cast(9, 23, hex(97))]),
        (Some(14000), vec![stack_cast(0, 54, hex(97))]),
    ];
    let t = play_area(&o, &commands);
    let groups: Vec<(u8, u32)> = t
        .keys
        .iter()
        .filter(|k| matches!(k.pose.group, 11..=13 | 17..=19) && k.pose.frame == 0)
        .map(|k| (k.stack, k.pose.group))
        .collect();
    assert_eq!(groups, [(0, 19), (0, 17), (1, 12)]);
    // The Swordsmen turned left (group 7) before and right (group 9) after.
    let turns: Vec<u32> = t
        .keys
        .iter()
        .filter(|k| k.stack == 1 && matches!(k.pose.group, 7 | 9) && k.pose.frame == 0)
        .map(|k| k.pose.group)
        .collect();
    assert_eq!(turns, [7, 9]);
    assert!(t.tints.iter().any(|k| k.stack == 2));
}

#[test]
fn a_creatures_cast_with_the_bugs_fixed() {
    // E17 M3 T7d (CLT-Q8): the casts of `a_creature_casts_by_its_angle_and_stands_at_its_casts_tail`
    // with the original's drawing bugs fixed. The group by the rows' angle, y up: beside its
    // target the Faerie Dragon looks straight (18, the original's 17), four rows above it looks up
    // (17, the original's 19), a row below down (19, the original's 17); missed by its Fireball
    // it shows standing at once, not with the end's redraw; its action unmarks the mouse's stack
    // at its start, as a hero's.
    let o = snapshot(vec![
        at(0, Side::Attacker, FAERIE_DRAGON, 81, true),
        at(1, Side::Defender, SWORDSMAN, 83, false),
        at(2, Side::Defender, SWORDSMAN, 15, false),
        at(3, Side::Defender, SWORDSMAN, 100, false),
    ]);
    let hex = |hex| Target::Hex { hex };
    let commands = [
        (
            Some(0),
            vec![
                Event::Defended { stack: 1 },
                Event::TurnStarted { stack: 2 },
            ],
        ),
        (
            Some(0),
            vec![
                Event::Defended { stack: 2 },
                Event::TurnStarted { stack: 0 },
            ],
        ),
        (
            Some(1000),
            vec![stack_cast(0, 21, hex(83)), hurt(1, 20, 20, 0)],
        ),
        (
            Some(8000),
            vec![stack_cast(0, 21, hex(15)), hurt(2, 20, 20, 0)],
        ),
        (Some(16000), vec![stack_cast(0, 23, hex(98))]),
    ];
    let groups = |t: &Timeline| -> Vec<u32> {
        t.keys
            .iter()
            .filter(|k| k.stack == 0 && matches!(k.pose.group, 17..=19) && k.pose.frame == 0)
            .map(|k| k.pose.group)
            .collect()
    };
    let fixed = play_area_with(&o, &commands, Bugs::Fixed);
    assert_eq!(groups(&fixed), [18, 17, 19]);
    assert_eq!(groups(&play_area(&o, &commands)), [17, 19, 17]);
    // Missed: standing after the loop's six frames as the defending Swordsmen play on, on the
    // screen with that frame (the original: its cast group's last frame until the end's redraw,
    // 600 ms later).
    let after_loop = |t: &Timeline| {
        let reacts = t
            .keys
            .iter()
            .find(|k| k.stack == 1 && k.pose.group == 4 && k.drawn > 1000)
            .unwrap()
            .drawn;
        t.at(reacts + 700).get(&0).map(|p| p.group)
    };
    assert_eq!(after_loop(&fixed), Some(2));
    assert_eq!(after_loop(&play_area(&o, &commands)), Some(17));
    // The casts' actions unmark the mouse's stack at their start.
    let unmarks: Vec<bool> = fixed.actions.iter().map(|a| a.unmarks).collect();
    assert_eq!(unmarks, [false, false, true, true, true]);
}

#[test]
fn magic_animations_at_a_hex_and_over_several_stacks_without_their_look() {
    // E17 M3 T7c-1: an area spell or a mass spell whose animation the stage lacks plays only its
    // line and reactions; a mass animation over a stack the timeline does not know skips it.
    let o = snapshot(vec![
        at(0, Side::Attacker, MARKSMAN, 86, false),
        at(1, Side::Defender, SWORDSMAN, 15, false),
    ]);
    let mut casts = area_casts();
    for spell in [21, 24] {
        if let Some(look) = casts.spells.get_mut(&spell) {
            look.animation = Some(200);
        }
    }
    let layout = layout();
    let magic = area_magic();
    let stage = Stage {
        layout: &layout,
        width: 17,
        groups: tables().battle.groups,
        magic: &magic,
        casts: &casts,
        bugs: Bugs::Original,
        siege: None,
    };
    let events = [
        vec![cast_at(21, Target::Hex { hex: 15 }), hurt(1, 9, 9, 0)],
        vec![
            cast_at(24, Target::None {}),
            hurt(1, 9, 9, 0),
            hurt(7, 9, 9, 0),
        ],
    ];
    let commands = [
        Command {
            at: Some(0),
            events: &events[0],
        },
        Command {
            at: Some(4000),
            events: &events[1],
        },
    ];
    let t = timeline(&o, &commands, &area_looks(), stage, Speed::default()).unwrap();
    assert!(t.spots.is_empty() && t.magics.is_empty());
    assert!(t.keys.iter().any(|k| k.stack == 1 && k.pose.group == 3));
}

/// The magic animations of the spell objects' tests (E17 M3 T7c-2): the placing and vanish ones
/// of Fire Wall (65, 66), Quicksand (55, 58), Force Field (32, 33, 60, 61) and Land Mine (47, 59),
/// three frames each, and Armageddon's 12 (20 frames).
fn object_magic() -> BTreeMap<u8, MagicLook> {
    let entry = |def: &str, frames, size, place| MagicLook {
        def: def.to_string(),
        frames,
        size,
        place,
        translucent: false,
    };
    let mut all = spell_magic();
    for (number, def, place) in [
        (65, "C07spF60.def", 4),
        (66, "C07spF62.def", 4),
        (55, "C17spE0.def", 4),
        (58, "C17spE2.def", 4),
        (32, "C15spE0.def", 0),
        (33, "C15spE9.def", 0),
        (60, "C15spE2.def", 4),
        (61, "C15spE11.def", 4),
        (47, "C09spF0.def", 4),
        (59, "C09spF2.def", 4),
    ] {
        all.insert(number, entry(def, 3, [44, 52], place));
    }
    all.insert(12, entry("C06spF0.def", 20, [200, 200], 15));
    all
}

/// The spells that place objects (Quicksand 10, Land Mine 11, Force Field 12, Fire Wall 13: the
/// asset tables' objects, invented frames) and Armageddon (26).
fn object_casts() -> Casts {
    let mut casts = casts();
    let object = |def: &str, (hexes, rows, before_stacks), (placed, vanish)| ObjectLook {
        def: def.to_string(),
        frames: 4,
        hexes,
        rows,
        before_stacks,
        placed,
        vanish,
    };
    let spell = |original, animation, objects, object_sound| SpellLook {
        original,
        animation,
        objects,
        object_sound,
        ..SpellLook::default()
    };
    casts.spells.extend([
        (
            10,
            spell(
                10,
                Some(55),
                vec![object("C17SPE1.DEF", (1, 1, true), (55, 58))],
                true,
            ),
        ),
        (
            11,
            spell(
                11,
                Some(47),
                vec![object("C09spF1.def", (1, 1, true), (47, 59))],
                true,
            ),
        ),
        (
            12,
            spell(
                12,
                None,
                vec![
                    object("C15spE1.def", (2, 3, false), (32, 60)),
                    object("C15spE10.def", (3, 4, false), (33, 61)),
                ],
                false,
            ),
        ),
        (
            13,
            spell(
                13,
                Some(65),
                vec![object("C07spF61.def", (1, 3, false), (65, 66))],
                false,
            ),
        ),
        (26, spell(26, Some(12), Vec::new(), false)),
    ]);
    casts
}

fn play_objects_with(
    snapshot: &Observation,
    commands: &[(Option<u32>, Vec<Event>)],
    looks: (&BTreeMap<u8, MagicLook>, Casts),
) -> Timeline {
    play_objects_bugs(snapshot, commands, looks, Bugs::Original)
}

/// As [`play_objects_with`], the original's drawing bugs reproduced or fixed by `bugs` (E17 M3
/// T9d).
fn play_objects_bugs(
    snapshot: &Observation,
    commands: &[(Option<u32>, Vec<Event>)],
    (magic, casts): (&BTreeMap<u8, MagicLook>, Casts),
    bugs: Bugs,
) -> Timeline {
    let layout = layout();
    let stage = Stage {
        layout: &layout,
        width: 17,
        groups: tables().battle.groups,
        magic,
        casts: &casts,
        bugs,
        siege: None,
    };
    let commands: Vec<Command<'_>> = commands
        .iter()
        .map(|(at, events)| Command { at: *at, events })
        .collect();
    timeline(snapshot, &commands, &area_looks(), stage, Speed::default())
        .unwrap()
        .with_heroes(cast_heroes())
}

fn placed(spell: u16, hexes: &[u8]) -> Event {
    Event::ObstaclePlaced {
        side: Side::Attacker,
        spell,
        hexes: hexes.to_vec(),
    }
}

fn removed(spell: u16, hexes: &[u8]) -> Event {
    Event::ObstacleRemoved {
        spell,
        hexes: hexes.to_vec(),
    }
}

/// When the spots of animation `number` show their frames, and where.
fn spots_of(t: &Timeline, number: u8) -> Vec<(u32, Option<u32>, [i32; 2])> {
    t.spots
        .iter()
        .filter(|k| k.animation == number)
        .map(|k| (k.shown, k.frame, k.at))
        .collect()
}

#[test]
fn the_obstacles_sequence_plays_as_recorded() {
    // E17 M3 T7c-2 (`seq-obstacles.toml`): Orrin's Fire Wall at 146 (round 1), Quicksand's four
    // pits (round 2), the Fire Wall gone at round 3's start, a Force Field at 63 (round 3),
    // Armageddon (round 4); the Swordsmen and the Pikemen defend.
    let o = snapshot(vec![
        at(1, Side::Defender, SWORDSMAN, 49, false),
        at(2, Side::Defender, SWORDSMAN, 151, false),
        at(0, Side::Attacker, PIKEMAN, 86, false),
    ]);
    let defend = |stack, next| {
        vec![
            Event::Defended { stack },
            Event::TurnStarted { stack: next },
        ]
    };
    let round = |round, gone: Vec<Event>| {
        let mut events = vec![Event::Defended { stack: 0 }, Event::RoundStarted { round }];
        events.extend(gone);
        events.push(Event::TurnStarted { stack: 1 });
        events
    };
    let none = Target::None {};
    let pits = [154, 99, 177, 71];
    let mut quicksand = vec![cast_at(10, none)];
    quicksand.extend(pits.iter().map(|&h| placed(10, &[h])));
    let commands = [
        (Some(0), defend(1, 2)),
        (Some(0), defend(2, 0)),
        (
            Some(0),
            vec![
                cast_at(13, Target::Hex { hex: 146 }),
                placed(13, &[146]),
                placed(13, &[130]),
            ],
        ),
        (Some(1400), round(2, Vec::new())),
        (Some(1400), defend(1, 2)),
        (Some(1400), defend(2, 0)),
        (Some(1400), quicksand),
        (
            Some(3400),
            round(3, vec![removed(13, &[146]), removed(13, &[130])]),
        ),
        (Some(4200), defend(1, 2)),
        (Some(4200), defend(2, 0)),
        (
            Some(4200),
            vec![cast_at(12, Target::Hex { hex: 63 }), placed(12, &[46, 63])],
        ),
        (Some(5300), round(4, Vec::new())),
        (Some(5300), defend(1, 2)),
        (Some(5300), defend(2, 0)),
        (
            Some(5300),
            vec![
                cast_at(26, none),
                hurt(0, 80, 80, 8),
                hurt(1, 80, 80, 2),
                hurt(2, 80, 80, 2),
            ],
        ),
    ];
    let t = play_objects_with(&o, &commands, (&object_magic(), object_casts()));
    assert_eq!(
        t.lines_with_heroes(7700),
        until(&screenless(OBSTACLES), 7700)
    );
    // The Fire Wall: both hexes' 65 at the hex aimed at, each object with the redraw after its
    // animation; both go at round 3's start, each with 66 at its own hex, before the round's line.
    let walls = spots_of(&t, 65);
    assert_eq!(walls.iter().filter(|w| w.1.is_some()).count(), 6);
    assert_eq!(walls.len(), 8);
    assert!(walls.iter().all(|w| w.2 == walls[0].2));
    assert_eq!(t.objects(800).len(), 1);
    assert_eq!(
        t.objects(1100).keys().copied().collect::<Vec<_>>(),
        [130, 146]
    );
    let vanish = spots_of(&t, 66);
    assert_eq!(vanish.len(), 8);
    assert_ne!(vanish[0].2, vanish[4].2);
    let round_3 = t
        .log
        .iter()
        .filter(|(_, l)| *l == Logged::Round)
        .map(|(at, _)| *at)
        .nth(1)
        .unwrap();
    assert!(vanish.iter().all(|v| v.0 <= round_3));
    assert!(!t.objects(vanish[0].0).contains_key(&146));
    // The pits: each its own place and its sound; the cast's line after the last.
    let sunk = spots_of(&t, 55);
    assert_eq!(sunk.iter().filter(|p| p.1.is_some()).count(), 12);
    assert_eq!(sunk.len(), 16);
    assert_eq!(
        t.cues
            .iter()
            .filter(|c| c.sound == Sound::Object(10))
            .count(),
        4
    );
    let cast_lines: Vec<u32> = t
        .log
        .iter()
        .filter(|(_, l)| matches!(l, Logged::CastAll { .. }))
        .map(|(at, _)| *at)
        .collect();
    assert_eq!(cast_lines.len(), 3);
    assert!(sunk.iter().all(|p| p.0 <= cast_lines[1]));
    // The Force Field at 63 (two hexes, rows 3): it shows with the hero's next frame, no redraw of
    // its own.
    let field = t.objects(t.end);
    assert_eq!(
        field.keys().copied().collect::<Vec<_>>(),
        [63, 71, 99, 154, 177]
    );
    assert_eq!(
        (field[&63].rows, field[&63].blocked.clone()),
        (3, vec![46, 63])
    );
    assert!(field[&154].before_stacks && field[&154].blocked.is_empty());
    let shown = t.objects.iter().find(|k| k.anchor == 63).unwrap().shown;
    assert!(t.hero_keys.iter().any(|k| k.shown == shown));
    // Armageddon: its 20 frames tiled, then its line; the last frame stays to the next redraw.
    let frames: Vec<Option<u32>> = t.fields.iter().map(|k| k.frame).collect();
    let mut expected: Vec<Option<u32>> = (0..20).map(Some).collect();
    expected.push(None);
    assert_eq!(frames, expected);
    assert_eq!(t.field(t.fields[19].shown).map(|f| f.frame), Some(19));
    assert_eq!(t.field(t.fields[20].shown), None);
    let line = t
        .log
        .iter()
        .find(|(_, l)| matches!(l, Logged::Armageddon { .. }))
        .unwrap();
    assert_eq!(line.1, Logged::Armageddon { damage: 80 });
    assert!(line.0 >= t.fields[19].shown);
    assert!(t.changes(t.end).contains(&t.fields[0].shown));
}

#[test]
fn a_fire_wall_and_armageddon_with_the_bugs_fixed() {
    // E17 M3 T9d: Orrin's Fire Wall at 146 (its second object at 130), then Armageddon, whose
    // value the caster's bonuses raise from 80 to 100. CLT-Q12: each object's placing animation
    // at its own hex, where the original plays both at the hex aimed at. CLT-Q16: Armageddon's
    // line with the caster's bonuses. CLT-Q10: a stack it struck ends on its standing frame 0.
    let o = snapshot(vec![
        at(1, Side::Defender, SWORDSMAN, 49, false),
        at(2, Side::Defender, SWORDSMAN, 151, false),
        at(0, Side::Attacker, PIKEMAN, 86, false),
    ]);
    let hurt = |target, kills| Event::SpellDamage {
        target,
        value: WireU64(80),
        caster: WireU64(100),
        damage: WireU64(100),
        kills,
    };
    let commands = [
        (
            Some(0),
            vec![
                cast_at(13, Target::Hex { hex: 146 }),
                placed(13, &[146]),
                placed(13, &[130]),
            ],
        ),
        (
            None,
            vec![
                cast_at(26, Target::None {}),
                hurt(0, 8),
                hurt(1, 2),
                hurt(2, 2),
            ],
        ),
    ];
    let magic = object_magic();
    let original = play_objects_with(&o, &commands, (&magic, object_casts()));
    let fixed = play_objects_bugs(&o, &commands, (&magic, object_casts()), Bugs::Fixed);
    let places =
        |t: &Timeline| -> BTreeSet<[i32; 2]> { spots_of(t, 65).iter().map(|w| w.2).collect() };
    assert_eq!(places(&original).len(), 1);
    assert_eq!(places(&fixed).len(), 2);
    assert!(places(&fixed).is_superset(&places(&original)));
    let line = |t: &Timeline| {
        t.log.iter().find_map(|(_, l)| match *l {
            Logged::Armageddon { damage } => Some(damage),
            _ => None,
        })
    };
    assert_eq!((line(&original), line(&fixed)), (Some(80), Some(100)));
    let pose = |t: &Timeline| t.at(t.end).get(&1).map(|p| (p.group, p.frame));
    assert_eq!(pose(&fixed), Some((2, 0)));
    assert!(pose(&original).is_some_and(|(group, frame)| group == 2 && frame > 0));
}

#[test]
fn objects_outside_casts_and_without_their_looks() {
    // E17 M3 T7c-2: the snapshot's pit at 0; a Land Mine laid outside a cast and one revealed,
    // shown with the next frame (one already known not twice); a removal outside a round's
    // start plays its vanish, one the timeline does not know none; a placement without hexes or a
    // look is skipped; a Force Field of three hexes takes its three-hex object, a spell whose sizes
    // do not match its first; Armageddon with a two-frame animation shows the field plain after
    // it, kills a stack (its corpse, a redraw), strikes none (no line) or lacks its look.
    let mut o = snapshot(vec![
        at(1, Side::Defender, SWORDSMAN, 49, false),
        at(0, Side::Attacker, PIKEMAN, 86, false),
    ]);
    o.spell_obstacles = vec![h3_protocol::wire::SpellObstacle {
        spell: 10,
        kind: h3_protocol::wire::ObstacleKind::Pit {},
        side: Side::Attacker,
        hexes: vec![50],
        rounds: 0,
        value: WireU64(0),
        revealed: true,
    }];
    let revealed = |spell, hexes: &[u8]| Event::ObstacleRevealed {
        side: Side::Attacker,
        spell,
        hexes: hexes.to_vec(),
    };
    let none = Target::None {};
    let commands = [
        (
            Some(0),
            vec![
                placed(11, &[60]),
                revealed(10, &[50]),
                revealed(10, &[70]),
                placed(99, &[80]),
                placed(10, &[]),
            ],
        ),
        (
            Some(1000),
            vec![
                Event::Defended { stack: 1 },
                removed(11, &[60]),
                removed(11, &[90]),
                removed(11, &[]),
            ],
        ),
        (
            Some(2000),
            vec![
                cast_at(12, Target::Hex { hex: 63 }),
                Event::SpellResisted { target: 1 },
                placed(12, &[29, 46, 63]),
                placed(10, &[]),
            ],
        ),
        (Some(4000), vec![cast_at(11, none), placed(11, &[100])]),
        (
            Some(6000),
            vec![
                cast_at(26, none),
                Event::SpellResisted { target: 0 },
                hurt(1, 50, 50, 40),
                hurt(9, 50, 50, 1),
            ],
        ),
        (Some(9000), vec![cast_at(26, none)]),
    ];
    let mut magic = object_magic();
    magic.insert(
        12,
        MagicLook {
            frames: 2,
            ..magic[&12].clone()
        },
    );
    let t = play_objects_with(&o, &commands, (&magic, object_casts()));
    assert_eq!(t.objects(0).keys().copied().collect::<Vec<_>>(), [50]);
    assert_eq!(
        t.objects(999).keys().copied().collect::<Vec<_>>(),
        [50, 60, 70]
    );
    assert_eq!(spots_of(&t, 59).len(), 4);
    assert!(!t.objects(1999).contains_key(&60));
    assert_eq!(t.objects(5000)[&63].rows, 4);
    assert!(t.cues.iter().any(|c| c.sound == Sound::Object(11)));
    // Armageddon: frames 0 and 1, then the field plain through the Swordsmen's death group.
    let frames: Vec<Option<u32>> = t.fields.iter().map(|k| k.frame).collect();
    assert_eq!(&frames[..3], &[Some(0), Some(1), None]);
    let corpse = t.keys.iter().rev().find(|k| k.stack == 1).unwrap();
    assert_eq!(corpse.pose.layer, CORPSE);
    let lines = t
        .log
        .iter()
        .filter(|(_, l)| matches!(l, Logged::Armageddon { .. }))
        .count();
    assert_eq!(lines, 1);
    // Without its look Armageddon plays the reactions alone; a Fire Wall without its objects'
    // looks places nothing and writes its line.
    let mut bare = object_magic();
    bare.remove(&12);
    let mut casts = object_casts();
    casts.spells.entry(13).and_modify(|s| s.objects.clear());
    let commands = [
        (Some(0), vec![cast_at(26, none), hurt(1, 50, 50, 1)]),
        (
            Some(4000),
            vec![cast_at(13, Target::Hex { hex: 63 }), placed(13, &[63])],
        ),
    ];
    let t = play_objects_with(&o, &commands, (&bare, casts));
    assert!(t.fields.is_empty() && !t.objects(t.end).contains_key(&63));
    assert!(
        t.log
            .iter()
            .any(|(_, l)| *l == Logged::CastAll { side: 0, spell: 13 })
    );
    assert!(t.keys.iter().any(|k| k.stack == 1 && k.pose.group == 3));
}

mod raised;
mod reviewed;
mod siege;
