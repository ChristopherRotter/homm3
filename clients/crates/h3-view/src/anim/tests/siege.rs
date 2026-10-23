//! E17 M4 T2b: a siege's shots over time (design §10.2), replayed from the recorded sequences'
//! events with their creatures' numbers (the asset tables' times, the DEFs' frame counts and
//! canvases, read from the user's archives: numbers only, ADR-0009), and the slips of CLT-Q19
//! fixed.

use alloc::collections::BTreeMap;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

use h3_protocol::wire::{Event, Observation, Side, Siege, Stack, WireU64};
use h3_render::Bugs;

use super::{at, flags, layout, look, screenless, snapshot, tables, until};
use crate::anim::{
    Blast, Burst, Casts, Command, Logged, Look, Missile, Projectile, SiegeStage, Sound, Speed,
    Stage, Timeline, TurretPose, timeline, turret_record, unfidgeted,
};
use crate::siege::Walls;

const KEEP: &str = include_str!("../../../../../situations/timelines/keep.txt");
const HIT: &str = include_str!("../../../../../situations/timelines/hit.txt");
const MISSED: &str = include_str!("../../../../../situations/timelines/missed.txt");
const AUTOMATIC: &str = include_str!("../../../../../situations/timelines/automatic.txt");

/// The siege tests' creatures (invented handles).
const PIKEMAN: u16 = 40;
const GRIFFIN: u16 = 41;
const ARCHER: u16 = 42;
const CATAPULT: u16 = 43;
const GOBLIN: u16 = 44;
const WOLF_RIDER: u16 = 45;
const ORC: u16 = 46;
/// The Arrow Tower: a turret, its look the town's shooter's (a Castle's Archer).
const TURRET: u16 = 47;

/// The Archer's numbers (`clcbow.def`; its missile `plcbowx.def`, 30 × 30, nine frames).
fn archer() -> Look {
    let mut l = look(
        [9000, 500, 500, 115],
        &[
            (0, 8),
            (1, 9),
            (2, 8),
            (3, 6),
            (4, 9),
            (5, 6),
            (14, 8),
            (15, 8),
            (16, 8),
        ],
        190,
    );
    l.climax = 7;
    l.missile_offsets = [[38, -65], [36, -59], [21, -50]];
    l.missile = Some(Missile {
        def: "plcbowx.def".to_string(),
        size: [30, 30],
        frames: 9,
        angles: vec![90, 72, 45, 27, 0, -27, -45, -72, -90],
    });
    l
}

/// The Catapult's numbers (`smcata.def`, its groups 14 to 16 the same 13 frames, climax 4; its
/// rock `SMCatx.def`, 30 × 22, eight frames).
fn catapult() -> Look {
    let mut l = look(
        [9000, 500, 500, 115],
        &[(2, 1), (3, 8), (4, 8), (5, 9), (14, 13), (15, 13), (16, 13)],
        176,
    );
    l.climax = 4;
    l.missile_offsets = [[69, -119]; 3];
    l.missile = Some(Missile {
        def: "SMCatx.def".to_string(),
        size: [30, 22],
        frames: 8,
        angles: vec![0; 8],
    });
    l
}

/// The siege's creatures.
fn siege_looks() -> BTreeMap<u16, Look> {
    BTreeMap::from([
        (
            PIKEMAN,
            look(
                [9000, 575, 500, 115],
                &[(0, 6), (1, 4), (2, 7), (3, 6), (4, 13), (5, 5)],
                171,
            ),
        ),
        (
            GRIFFIN,
            look(
                [9000, 500, 500, 115],
                &[(0, 4), (1, 8), (2, 8), (3, 7), (4, 8), (5, 9)],
                177,
            ),
        ),
        (ARCHER, archer()),
        (CATAPULT, catapult()),
        (
            GOBLIN,
            look(
                [9000, 605, 500, 115],
                &[(0, 9), (1, 9), (2, 8), (3, 7), (4, 9), (5, 9)],
                200,
            ),
        ),
        (
            WOLF_RIDER,
            look(
                [9000, 465, 500, 115],
                &[(0, 6), (1, 9), (2, 8), (3, 7), (4, 8), (5, 7)],
                179,
            ),
        ),
        (
            ORC,
            look(
                [9000, 500, 500, 115],
                &[(0, 8), (1, 11), (2, 8), (3, 6), (4, 11), (5, 8)],
                195,
            ),
        ),
        (TURRET, archer()),
    ])
}

/// A Castle's turret points and the wall shots' explosions (`SGEXPL.DEF` 254 × 203,
/// `CSGRCK.DEF` 180 × 230, ten frames each).
fn castle() -> SiegeStage {
    SiegeStage {
        turrets: [[780, 238], [648, 566], [596, 80]],
        hit: Burst {
            def: "SGEXPL.DEF".to_string(),
            size: [254, 203],
            frames: 10,
        },
        miss: Burst {
            def: "CSGRCK.DEF".to_string(),
            size: [180, 230],
            frames: 10,
        },
    }
}

/// A war machine (a turret, the Catapult) in `slot`.
fn machine_at(id: u8, side: Side, creature: u16, (hex, slot): (u8, u8)) -> Stack {
    let mut s = at(id, side, creature, hex, false);
    s.war_machine = true;
    s.slot = slot;
    s
}

/// `keep`'s siege (`seq-keep.toml`): a Castle town with a Castle, the attacker's Pikemen,
/// Griffins, Archers and Catapult, the defender's Goblins, Wolf Riders, Orcs and turrets; the
/// keep's turn opens it.
fn keep_siege() -> Observation {
    let mut catapult = machine_at(3, Side::Attacker, CATAPULT, (119, 19));
    catapult.two_hex = true;
    let mut griffins = at(1, Side::Attacker, GRIFFIN, 86, true);
    griffins.flies = true;
    let mut o = snapshot(vec![
        machine_at(7, Side::Defender, TURRET, (254, 20)),
        at(0, Side::Attacker, PIKEMAN, 35, false),
        griffins,
        at(2, Side::Attacker, ARCHER, 137, false),
        catapult,
        at(4, Side::Defender, GOBLIN, 49, false),
        at(5, Side::Defender, WOLF_RIDER, 100, true),
        at(6, Side::Defender, ORC, 151, false),
        machine_at(8, Side::Defender, TURRET, (255, 21)),
        machine_at(9, Side::Defender, TURRET, (251, 22)),
    ]);
    o.siege = Some(Siege {
        faction: 0,
        fort: 3,
        moat: 2,
        gate_open: false,
        walls: vec![12, 29, 45, 62, 78, 96, 112, 130, 147, 165, 182],
        moat_hexes: vec![11, 28, 44, 61, 77, 95, 111, 129, 146, 164, 181],
        parts: vec![2, 3, 3, 2, 3, 3, 2, 2],
    });
    o
}

/// The timeline of `snapshot` and `commands` (each with its time) in a siege, the original's
/// drawing bugs reproduced or fixed by `bugs`.
fn play_siege(
    snapshot: &Observation,
    commands: &[(Option<u32>, Vec<Event>)],
    bugs: Bugs,
    siege: Option<&SiegeStage>,
) -> Timeline {
    let layout = layout();
    let magic = BTreeMap::new();
    let casts = Casts::default();
    let stage = Stage {
        layout: &layout,
        width: 17,
        groups: tables().battle.groups,
        magic: &magic,
        casts: &casts,
        bugs,
        siege,
    };
    let commands: Vec<Command<'_>> = commands
        .iter()
        .map(|(at, events)| Command { at: *at, events })
        .collect();
    timeline(snapshot, &commands, &siege_looks(), stage, Speed::default()).unwrap()
}

/// A turret's shot of `damage` killing `kills`.
fn turret_shot(turret: u8, target: u8, damage: u64, kills: u32) -> Event {
    Event::Struck {
        attacker: turret,
        target,
        damage: WireU64(damage),
        kills,
        flags: flags(true, false, false),
    }
}

/// A wall shot of the Catapult (3) aimed at `aimed` that hit `part` for `damage`.
fn wall_shot(aimed: u8, part: u8, damage: u8) -> Event {
    Event::WallShot {
        stack: 3,
        aimed,
        part,
        damage,
    }
}

/// The recorded `lines` without their screens and the original's fidgets, up to `ms`.
fn recorded(lines: &str, ms: u32) -> alloc::string::String {
    until(
        &unfidgeted(&screenless(lines), &alloc::collections::BTreeSet::new()),
        ms,
    )
}

/// The three turrets' defends that open `hit`, `missed` and `automatic` (the keep's turn opens).
fn defends(last: Vec<Event>) -> Vec<(Option<u32>, Vec<Event>)> {
    let turn = |stack| Event::TurnStarted { stack };
    let mut third = vec![Event::Defended { stack: 9 }, turn(3)];
    third.extend(last);
    vec![
        (Some(0), vec![Event::Defended { stack: 7 }, turn(8)]),
        (Some(0), vec![Event::Defended { stack: 8 }, turn(9)]),
        (Some(0), third),
    ]
}

/// The walls' states as the snapshot's siege gives them, `part` at `state`.
fn walls_with(part: usize, state: u8) -> Walls {
    let mut w = Walls::of(keep_siege().siege.as_ref().unwrap());
    w.parts[part] = state;
    w
}

#[test]
fn the_keep_sequence_plays_as_recorded() {
    // `keep`: the keep shoots the Pikemen, the upper tower the Griffins, the lower tower the
    // Archers, each commanded (the defender's Artillery).
    let o = keep_siege();
    let turn = |stack| Event::TurnStarted { stack };
    let commands = [
        (Some(0), vec![turret_shot(7, 0, 13, 1), turn(8)]),
        (Some(1654), vec![turret_shot(8, 1, 11, 0), turn(9)]),
        (Some(3313), vec![turret_shot(9, 2, 11, 1), turn(3)]),
    ];
    let castle = castle();
    let t = play_siege(&o, &commands, Bugs::Original, Some(&castle));
    assert_eq!(t.lines(), recorded(KEEP, 5439));
    // The keep's frames 0 to 6 of group 15 facing left, shown on the deadline; its rest (frame
    // 7) drawn at 1 554 never shows (CLT-Q19 (5)); it stands at 1 654.
    let keep = |ms| t.turrets(ms)[0];
    let shooting = |frame| {
        Some(TurretPose {
            group: 15,
            frame,
            right: false,
        })
    };
    assert_eq!(keep(99), shooting(0).map(|p| TurretPose { group: 2, ..p }));
    assert_eq!((keep(100), keep(526)), (shooting(0), shooting(6)));
    assert_eq!(keep(1653), shooting(6));
    assert_eq!(keep(1654).map(|p| (p.group, p.frame)), Some((2, 0)));
    // The arrow: its 16 steps every 33 ms from 526, frame 4 (straight), toward the left.
    assert_eq!(
        t.projectile(526),
        Some(Projectile {
            def: "plcbowx.def".to_string(),
            frame: 4,
            at: [729, 164],
            mirrored: true,
        })
    );
    assert_eq!(t.projectile(1054), None);
    // The line names the arrow tower before the reactions (0x465D9E).
    let lines: Vec<(u32, &Logged)> = t
        .log
        .iter()
        .filter(|(_, l)| matches!(l, Logged::Turret { .. }))
        .map(|(at, l)| (*at, l))
        .collect();
    assert_eq!(lines[0], (1054, &Logged::Turret { hit: (0, 13, 1) }));
    // The turret's shot sound by its file (category 2).
    assert!(
        t.cues
            .iter()
            .any(|c| c.stack == 7 && c.sound == Sound::TurretShot)
    );
    // The slip fixed: the rest shows, on the deadline (frame 7 at 1 654, as it stands).
    let fixed = play_siege(&o, &commands, Bugs::Fixed, Some(&castle));
    let rest = |t: &Timeline| {
        t.turrets
            .iter()
            .find(|k| k.pose.frame == 7)
            .map(|k| k.shown)
    };
    assert_eq!((rest(&t), rest(&fixed)), (Some(u32::MAX), Some(1654)));
}

#[test]
fn the_hit_sequence_plays_as_recorded() {
    // `hit`: the turrets defend, the Catapult hits the upper wall (target 1, part 6) for 1.
    let o = keep_siege();
    let turn = |stack| Event::TurnStarted { stack };
    let commands = defends(vec![]);
    let mut commands = commands;
    commands.push((Some(0), vec![wall_shot(1, 1, 1), turn(1)]));
    let castle = castle();
    let t = play_siege(&o, &commands, Bugs::Original, Some(&castle));
    assert_eq!(t.lines(), recorded(HIT, 4500));
    // The rock from 475: 25 steps of 100 ms on its arc, its frames cycling; the first at the
    // start's canvas, then up and over to the upper wall.
    let rock = |ms: u32| t.projectile(ms).map(|p| (p.frame, p.at));
    assert_eq!(rock(474), None);
    assert_eq!(rock(475), Some((0, [90, 292])));
    assert_eq!(rock(575), Some((1, [108, 249])));
    assert_eq!(rock(2875), Some((0, [530, 88])));
    assert_eq!(rock(2975), None);
    // The explosion's frames 0 and 1 at 2 975, then one each 100 ms; its rectangle the canvas
    // clipped to the battlefield; each frame over the frames before it (CLT-Q19 (4)).
    let blast = |ms| t.blast(ms);
    let whole = Walls::of(o.siege.as_ref().unwrap());
    let hit = walls_with(6, 1);
    assert_eq!(blast(2974), None);
    assert_eq!(
        blast(2975),
        Some(Blast {
            def: "SGEXPL.DEF".to_string(),
            at: [437, 27],
            rect: [437, 27, 691, 230],
            frames: vec![(0, whole), (1, whole)],
        })
    );
    // From frame 5 on the redraws draw the wall hit; the whole screen shows it at the end.
    let frames = blast(3375).unwrap().frames;
    assert_eq!((frames.len(), frames[5]), (6, (5, hit)));
    assert_eq!(blast(3775), None);
    assert_eq!((t.walls(3774), t.walls(3775)), (Some(whole), Some(hit)));
    // The Catapult stands in memory from 475, on the screen from the end's redraw (CLT-Q19 (2)).
    assert_eq!((t.at(3774)[&3].group, t.at(3775)[&3].group), (15, 2));
    // The landing's sound with the explosion.
    assert!(
        t.cues
            .iter()
            .any(|c| c.sound == Sound::WallHit && c.at == 2975)
    );
    // The slips fixed: standing from the flight, the wall hit from frame 5 everywhere, each
    // explosion frame alone.
    let fixed = play_siege(&o, &commands, Bugs::Fixed, Some(&castle));
    assert_eq!(fixed.at(475)[&3].group, 2);
    assert_eq!(fixed.walls(3375), Some(hit));
    assert_eq!(fixed.blast(3375).unwrap().frames, vec![(5, hit)]);
}

#[test]
fn the_missed_sequence_plays_as_recorded() {
    // `missed`: aimed at the gate (target 3), the shot lands on the mid-lower wall (target 4,
    // part 10) for 1.
    let o = keep_siege();
    let mut commands = defends(vec![]);
    commands.push((
        Some(0),
        vec![wall_shot(3, 4, 1), Event::TurnStarted { stack: 1 }],
    ));
    let castle = castle();
    let t = play_siege(&o, &commands, Bugs::Original, Some(&castle));
    assert_eq!(t.lines(), recorded(MISSED, 4500));
    assert_eq!(t.walls(t.end), Some(walls_with(10, 1)));
}

#[test]
fn the_automatic_sequence_plays_as_recorded() {
    // `automatic`: the Catapult's own shot after the lower tower's defend, at the mid-upper wall
    // (target 2), no damage: CSGRCK and `WallMiss`, no state changed.
    let o = keep_siege();
    let commands = defends(vec![wall_shot(2, 2, 0), Event::TurnStarted { stack: 1 }]);
    let castle = castle();
    let t = play_siege(&o, &commands, Bugs::Original, Some(&castle));
    assert_eq!(t.lines(), recorded(AUTOMATIC, 5000));
    assert_eq!(t.walls.len(), 1);
    let blast = t.blasts.first().unwrap();
    assert_eq!((blast.def.as_str(), blast.at), ("CSGRCK.DEF", [430, 97]));
    assert!(t.cues.iter().any(|c| c.sound == Sound::WallMiss));
    // The slip fixed, a shot of no damage changes no walls either.
    let fixed = play_siege(&o, &commands, Bugs::Fixed, Some(&castle));
    assert_eq!(fixed.walls.len(), 1);
}

#[test]
fn the_parts_take_their_damage_as_the_executable() {
    // `wall_part_damage` 0x465990: hit points less the damage, the state 1 while some are left,
    // 0 at none; a tower's or the keep's cover with it, the gate's state destroyed.
    let o = keep_siege();
    let castle = castle();
    let shots = vec![
        wall_shot(0, 0, 1),
        wall_shot(0, 0, 2),
        wall_shot(3, 3, 2),
        wall_shot(7, 7, 2),
        wall_shot(6, 6, 2),
        // No such target: nothing.
        wall_shot(9, 9, 2),
    ];
    let mut commands = defends(vec![]);
    commands.push((Some(0), shots));
    let t = play_siege(&o, &commands, Bugs::Fixed, Some(&castle));
    let w = t.walls(t.end).unwrap();
    // The upper tower (part 5) hit then destroyed, its cover (17) gone; the gate (9) destroyed;
    // the keep (14) and the lower tower (13) and their covers (15, 16).
    let states: Vec<u8> = [5, 17, 9, 14, 15, 13, 16].map(|p| w.parts[p]).into();
    assert_eq!(states, [0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(w.gate, crate::siege::GATE_DESTROYED);
    assert_eq!(t.walls.len(), 6);
    assert_eq!(t.walls[5].walls, w);
    let first = t.walls[1].walls;
    assert_eq!((first.parts[5], first.parts[17]), (1, 1));
}

#[test]
fn a_siege_without_its_shooter_or_stage_still_takes_the_damage() {
    // A wall shot by a stack the timeline does not know: the damage alone, at once. A turret's
    // shot at a stack it does not know: its line and the reactions alone. Without the siege's
    // stage the explosions have no frames.
    let o = keep_siege();
    let commands = [(
        Some(0),
        vec![
            Event::WallShot {
                stack: 30,
                aimed: 1,
                part: 1,
                damage: 1,
            },
            turret_shot(7, 30, 5, 0),
            wall_shot(2, 2, 1),
            // A wall shooter without a projectile: no rock.
            Event::WallShot {
                stack: 0,
                aimed: 4,
                part: 4,
                damage: 0,
            },
        ],
    )];
    let t = play_siege(&o, &commands, Bugs::Original, None);
    assert_eq!(t.walls.len(), 3);
    assert_eq!(t.walls[1].drawn, 0);
    assert!(t.blasts.iter().all(|b| b.frame.is_none()));
    assert!(
        t.log
            .iter()
            .any(|(_, l)| *l == Logged::Turret { hit: (30, 5, 0) })
    );
    // Outside a siege no walls nor turrets: a wall shot changes nothing.
    let mut plain = keep_siege();
    plain.siege = None;
    plain.stacks.retain(|s| s.hex < 187);
    let t = play_siege(&plain, &commands, Bugs::Original, None);
    assert!(t.walls.is_empty() && t.turrets.is_empty());
    assert_eq!(t.walls(0), None);
    // With the town's explosions but no walls: the frames drawn, no states.
    let t = play_siege(&plain, &commands, Bugs::Original, Some(&castle()));
    assert!(t.walls.is_empty());
    assert_eq!(t.blasts.iter().filter(|b| b.frame.is_some()).count(), 20);
}

#[test]
fn a_wall_shot_s_group_follows_the_original_s_angle() {
    use crate::anim::siege::wall_group;
    // Up and right at 45°: the original's down (its Δy unnegated), a shot's up; and the other way.
    assert_eq!(
        (wall_group(100, -100, false), wall_group(100, -100, true)),
        (2, 0)
    );
    assert_eq!(
        (wall_group(-100, 100, false), wall_group(-100, 100, true)),
        (0, 2)
    );
    assert_eq!(wall_group(100, 10, false), 1);
    // Straight above or below: 90° for Δy ≤ 0 in both.
    assert_eq!(
        [
            wall_group(0, -5, false),
            wall_group(0, 0, true),
            wall_group(0, 5, false)
        ],
        [0, 0, 2]
    );
}

#[test]
fn turret_records_follow_their_positions() {
    assert_eq!(
        [254, 251, 255, 200].map(turret_record),
        [Some(0), Some(1), Some(2), None]
    );
}
