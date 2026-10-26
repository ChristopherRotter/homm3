use super::*;
use alloc::string::ToString;
use alloc::vec;
use h3_protocol::wire::Side;

use crate::anim::TurretPose;
use crate::scene::tests::{layout, stack, tables};
use crate::tables::{Turrets, WallPart};

/// A town's tables: the drawbridge's three pictures, the rope, the moat and its lip, the back
/// wall, the upper tower (255) and its cover, three walls (45, 62, 78: whole in slot 2, hit in
/// 1), the arch (112), the lower tower and the keep with their covers, and a part on row 0's hex
/// 5 (for the row-0 rule); `moat`: the moat's picture.
fn town(moat: &str) -> Town {
    let part = |at: [i16; 2], hex: Option<u8>, pictures: [&str; 5]| WallPart {
        at,
        hex,
        pictures: pictures.map(ToString::to_string),
    };
    let none = ["", "", "", "", ""];
    fn tower<'a>(name: &'a str, whole: &'a str) -> [&'a str; 5] {
        [name, whole, whole, whole, whole]
    }
    fn wall<'a>(hit: &'a str, whole: &'a str) -> [&'a str; 5] {
        ["", hit, whole, whole, whole]
    }
    fn cover(name: &str) -> [&str; 5] {
        ["", name, "", "", ""]
    }
    let parts = vec![
        part(
            [400, 276],
            None,
            ["Drw3.pcx", "Drw2.pcx", "Drw1.pcx", "", ""],
        ),
        part([400, 276], None, cover("DrwC.pcx")),
        part([410, 90], None, [moat, "", "", "", ""]),
        part([403, 80], None, ["Mlip.pcx", "", "", "", ""]),
        part([600, 49], None, ["TpWl.pcx", "", "", "", ""]),
        part([569, 35], Some(255), tower("Tw22.pcx", "Tw21.pcx")),
        part([524, 32], Some(45), wall("Wa62.pcx", "Wa61.pcx")),
        part([489, 79], Some(62), ["Wa5.pcx", "", "", "", ""]),
        part([470, 127], Some(78), wall("Wa42.pcx", "Wa41.pcx")),
        part([477, 238], Some(112), tower("Arch.pcx", "Arch.pcx")),
        part([10, 10], Some(5), ["Row0.pcx"; 5]),
        part([1, 1], Some(9), none),
        part([1, 1], None, wall("Wa12.pcx", "Wa11.pcx")),
        part([602, 500], Some(251), tower("Tw12.pcx", "Tw11.pcx")),
        part([720, 158], Some(135), tower("Man2.pcx", "Man1.pcx")),
        part([720, 158], Some(135), cover("ManC.pcx")),
        part([602, 500], Some(251), cover("Tw1C.pcx")),
        part([557, 24], Some(255), cover("Tw2C.pcx")),
    ];
    Town {
        key: "castle".into(),
        original: 0,
        background: "SgCsBack.pcx".into(),
        turrets: Turrets {
            shooter: 26,
            shift: 39,
            missile: "plcbowx.def".into(),
            keep: [780, 238],
            lower: [648, 566],
            upper: [596, 80],
        },
        parts,
    }
}

/// The test tables with towns: `base:castle` (the moat drawn) and `base:tower` (no moat
/// picture), their shooter the dragon (26); `base:bad` names no creature.
fn siege_tables() -> Tables {
    let mut t = tables();
    t.towns.insert("base:castle".into(), town("Moat.pcx"));
    t.towns.insert("base:tower".into(), town(""));
    let mut bad = town("Moat.pcx");
    bad.turrets.shooter = 999;
    t.towns.insert("base:bad".into(), bad);
    t
}

/// A record's siege: `fort`, the moat damaging, the gate closed, every target whole.
fn siege(fort: u8) -> Siege {
    Siege {
        faction: 0,
        fort,
        moat: 2,
        gate_open: false,
        walls: vec![12, 29, 45, 62, 78, 96, 112, 130, 147, 165, 182],
        moat_hexes: vec![11, 28, 44, 61, 77, 95, 111, 129, 146, 164, 181],
        parts: vec![2, 3, 3, 2, 3, 3, 2, 2],
    }
}

/// The turrets' stacks (keep, upper, lower) among two field stacks.
fn stacks() -> Vec<Stack> {
    vec![
        stack(0, Side::Attacker, 0, 35, false),
        stack(1, Side::Defender, 0, 49, false),
        stack(7, Side::Defender, 0, 254, false),
        stack(8, Side::Defender, 0, 255, false),
        stack(9, Side::Defender, 0, 251, false),
    ]
}

fn look<'t>(
    tables: &'t Tables,
    siege: &Siege,
    faction: &str,
    walls: Option<Walls>,
    stacks: &[Stack],
    active: Option<(u8, u8)>,
) -> Option<SiegeLook<'t>> {
    let context = Context {
        walls: walls.unwrap_or_else(|| Walls::of(siege)),
        stacks,
        active,
        turrets: [None; 3],
    };
    SiegeLook::new(siege, faction, tables, context, 17).unwrap()
}

fn wall(picture: &str, at: [i32; 2], clip: Option<[i32; 4]>) -> Draw {
    Draw::Wall {
        picture: picture.into(),
        x: at[0],
        y: at[1],
        clip,
    }
}

#[test]
fn the_states_follow_the_hit_points_and_the_gate() {
    let whole = Walls::of(&siege(3));
    assert_eq!(whole.gate, GATE_CLOSED);
    for part in [5, 6, 8, 9, 10, 12, 13, 14] {
        assert_eq!(whole.parts[part], 2, "{part}");
    }
    for cover in [15, 16, 17] {
        assert_eq!(whole.parts[cover], 1, "{cover}");
    }
    assert_eq!(whole.parts[0..5], [0; 5]);
    let mut s = siege(3);
    s.parts = vec![0, 3, 3, 0, 3, 3, 2, 0];
    let broken = Walls::of(&s);
    assert_eq!(
        (
            broken.parts[5],
            broken.parts[17],
            broken.parts[14],
            broken.parts[15]
        ),
        (0, 0, 0, 0)
    );
    assert_eq!(
        (broken.gate, broken.parts[13], broken.parts[16]),
        (GATE_DESTROYED, 2, 1)
    );
    s.parts = vec![2; 8];
    s.gate_open = true;
    assert_eq!(Walls::of(&s).gate, GATE_OPEN);
    // Missing hit points (a record's short list) count as none.
    s.parts.clear();
    assert_eq!(
        Walls::of(&s),
        Walls {
            parts: [0; 18],
            gate: GATE_DESTROYED
        }
    );
}

#[test]
fn standing_walls_block_a_count_box() {
    // E17 M4 T2c-1 (`moat`): the record's wall hexes, the gate's included, but those of the
    // targets the states show destroyed (the mid-upper wall's 78, the gate's 96).
    let s = siege(3);
    let whole = Walls::of(&s);
    assert_eq!(
        whole.blocking(&s).collect::<Vec<u8>>(),
        [12, 29, 45, 62, 78, 96, 112, 130, 147, 165, 182]
    );
    let mut broken = whole;
    broken.parts[8] = 0;
    broken.parts[9] = 0;
    assert_eq!(
        broken.blocking(&s).collect::<Vec<u8>>(),
        [12, 29, 45, 62, 112, 130, 147, 165, 182]
    );
}

#[test]
fn a_town_without_a_fort_or_tables_draws_no_siege() {
    let t = siege_tables();
    let stacks = stacks();
    assert!(look(&t, &siege(0), "base:castle", None, &stacks, None).is_none());
    let context = Context {
        walls: Walls::default(),
        stacks: &stacks,
        active: None,
        turrets: [None; 3],
    };
    let err = SiegeLook::new(&siege(3), "base:rampart", &t, context, 17).unwrap_err();
    assert_eq!(err, SceneError::Key("base:rampart".into()));
    let err = SiegeLook::new(&siege(3), "base:bad", &t, context, 17).unwrap_err();
    assert_eq!(err, SceneError::Key("creature 999".into()));
}

#[test]
fn the_background_holds_the_moat_and_its_lip() {
    let t = siege_tables();
    let stacks = stacks();
    let castle = look(&t, &siege(3), "base:castle", None, &stacks, None).unwrap();
    let background = Draw::Background {
        picture: "SgCsBack.pcx".into(),
        x: 0,
        y: 0,
    };
    assert_eq!(
        castle.background(),
        [
            background.clone(),
            wall("Moat.pcx", [410, 90], None),
            wall("Mlip.pcx", [403, 80], None)
        ]
    );
    assert_eq!(castle.back_wall(), Some(wall("TpWl.pcx", [600, 49], None)));
    // No moat: none drawn (a Tower's pictures; a moat absent).
    let tower = look(&t, &siege(3), "base:tower", None, &stacks, None).unwrap();
    assert_eq!(tower.background(), core::slice::from_ref(&background));
    let mut dry = siege(2);
    dry.moat = 0;
    let dry = look(&t, &dry, "base:castle", None, &stacks, None).unwrap();
    assert_eq!(dry.background(), [background]);
}

#[test]
fn the_rows_turn_at_six_and_the_gate_draws_by_its_state() {
    let t = siege_tables();
    let stacks = stacks();
    let castle = look(&t, &siege(3), "base:castle", None, &stacks, None).unwrap();
    let (columns, d) = castle.columns(5);
    assert_eq!((columns[0], columns[16], d), (0, 16, 1));
    let (columns, d) = castle.columns(6);
    assert_eq!((columns[0], columns[16], d), (16, 0, -1));
    assert_eq!((castle.drawbridge(), castle.rope()), (None, None));
    let gate = |state| {
        let walls = Walls {
            gate: state,
            ..Walls::of(&siege(3))
        };
        let look = look(&t, &siege(3), "base:castle", Some(walls), &stacks, None).unwrap();
        (look.drawbridge(), look.rope())
    };
    let bridge = |name| Some(wall(name, [400, 276], None));
    assert_eq!(gate(GATE_OPEN), (bridge("Drw2.pcx"), bridge("DrwC.pcx")));
    assert_eq!(gate(2), (bridge("Drw1.pcx"), None));
    assert_eq!(gate(GATE_DESTROYED), (bridge("Drw3.pcx"), None));
    // A state past the pictures draws nothing.
    assert_eq!(gate(7), (None, None));
}

#[test]
fn towers_draw_whole_with_their_shooters_by_the_fort_level() {
    let t = siege_tables();
    let stacks = stacks();
    let shooter = |anchor: [i32; 2], outline| Draw::Shooter {
        def: "CBDRGN.DEF".into(),
        group: 2,
        frame: 0,
        anchor,
        mirrored: true,
        outline,
        rows: SHOOTER_ROWS,
    };
    let lay = layout();
    // A Castle: the upper tower, its shooter (39 right: the shift) and cover; the keep's
    // shooter outlined while its turret (7, at 254) is active.
    let castle = look(&t, &siege(3), "base:castle", None, &stacks, Some((7, 96))).unwrap();
    assert_eq!(
        castle.parts_at(255, 1, &lay),
        [
            wall("Tw21.pcx", [569, 35], None),
            shooter([635, 80], None),
            wall("Tw2C.pcx", [557, 24], None)
        ]
    );
    assert_eq!(
        castle.parts_at(135, 1, &lay),
        [
            wall("Man1.pcx", [720, 158], None),
            shooter([819, 238], Some(96)),
            wall("ManC.pcx", [720, 158], None)
        ]
    );
    // A Citadel: the keep's shooter only; a Fort: none, the towers and covers drawn all the same.
    let citadel = look(&t, &siege(2), "base:castle", None, &stacks, None).unwrap();
    assert_eq!(citadel.parts_at(135, 1, &lay)[1], shooter([819, 238], None));
    assert_eq!(citadel.parts_at(251, -1, &lay).len(), 2);
    let fort = look(&t, &siege(1), "base:castle", None, &stacks, None).unwrap();
    assert_eq!(
        fort.parts_at(251, -1, &lay),
        [
            wall("Tw11.pcx", [602, 500], None),
            wall("Tw1C.pcx", [602, 500], None)
        ]
    );
    // A destroyed tower: its destroyed picture, no cover, no shooter.
    let mut s = siege(3);
    s.parts[6] = 0;
    let broken = look(&t, &s, "base:castle", None, &stacks, None).unwrap();
    assert_eq!(
        broken.parts_at(251, -1, &lay),
        [wall("Tw12.pcx", [602, 500], None)]
    );
    // A dead turret is not outlined.
    let mut dead = stacks.clone();
    dead[2].alive = false;
    let keep = look(&t, &siege(3), "base:castle", None, &dead, Some((7, 96))).unwrap();
    assert_eq!(keep.parts_at(135, 1, &lay)[1], shooter([819, 238], None));
}

#[test]
fn walls_draw_in_three_strips_across_the_rows() {
    let t = siege_tables();
    let stacks = stacks();
    let lay = layout();
    let castle = look(&t, &siege(3), "base:castle", None, &stacks, None).unwrap();
    // Hex 62 (row 3): the upper wall's left strip (45's left edge 520), the buttress's middle
    // (62's, 498), the mid-upper wall's right strip (78's left edge 476, from 520).
    let (min, max) = (i32::MIN, i32::MAX);
    assert_eq!(
        castle.parts_at(62, 1, &lay),
        [
            wall("Wa61.pcx", [524, 32], Some([min, min, 520, max])),
            wall("Wa5.pcx", [489, 79], Some([498, min, 542, max])),
            wall("Wa41.pcx", [470, 127], Some([520, min, max, max]))
        ]
    );
    // Hex 45: the upper wall's middle strip, the buttress's right one; hex 29 the wall's right
    // strip.
    assert_eq!(
        castle.parts_at(45, 1, &lay),
        [
            wall("Wa61.pcx", [524, 32], Some([520, min, 564, max])),
            wall("Wa5.pcx", [489, 79], Some([542, min, max, max]))
        ]
    );
    assert_eq!(
        castle.parts_at(29, 1, &lay),
        [wall("Wa61.pcx", [524, 32], Some([564, min, max, max]))]
    );
    // A hit wall shows its slot 1.
    let walls = Walls {
        parts: {
            let mut p = Walls::of(&siege(3)).parts;
            p[6] = 1;
            p
        },
        gate: GATE_CLOSED,
    };
    let hit = look(&t, &siege(3), "base:castle", Some(walls), &stacks, None).unwrap();
    assert_eq!(
        hit.parts_at(29, 1, &lay),
        [wall("Wa62.pcx", [524, 32], Some([564, min, max, max]))]
    );
    // On row 0 a part's left strip is drawn at its own hex too, before the middle one (hex 5's
    // left edge 256); the arch whole at 112; nothing elsewhere.
    assert_eq!(
        castle.parts_at(5, 1, &lay),
        [
            wall("Row0.pcx", [10, 10], Some([min, min, 256, max])),
            wall("Row0.pcx", [10, 10], Some([256, min, 300, max]))
        ]
    );
    assert_eq!(
        castle.parts_at(112, -1, &lay),
        [wall("Arch.pcx", [477, 238], None)]
    );
    assert!(castle.parts_at(100, -1, &lay).is_empty());
}

#[test]
fn the_leftward_rows_draw_their_strips_mirrored() {
    // E17 M4 T3a, R1-4: on the rows that run right to left (d = −1) a wall's left strip is drawn
    // at h − 17 − o and its right strip at h + 17 − o + 1: a buttress on 165 (row 9) and the
    // lower wall on 182 (row 10), whose right strip falls on 200 (`AFTER_ROWS`).
    let mut t = siege_tables();
    let mut lower = town("Moat.pcx");
    lower.parts[11] = WallPart {
        at: [500, 400],
        hex: Some(165),
        pictures: ["", "Bt2.pcx", "Bt1.pcx", "Bt1.pcx", "Bt1.pcx"].map(ToString::to_string),
    };
    lower.parts[12].hex = Some(182);
    t.towns.insert("base:lower".into(), lower);
    let stacks = stacks();
    let lay = layout();
    let mut walls = Walls::of(&siege(3));
    walls.parts[11] = 2;
    walls.parts[12] = 2;
    let look = look(&t, &siege(3), "base:lower", Some(walls), &stacks, None).unwrap();
    let (min, max) = (i32::MIN, i32::MAX);
    let edge = |hex: u8| lay.hex_box(hex / 17, hex % 17)[0];
    let (b, w) = (edge(165), edge(182));
    let buttress = |clip| wall("Bt1.pcx", [500, 400], Some(clip));
    let lower_wall = |clip| wall("Wa11.pcx", [1, 1], Some(clip));
    assert_eq!(look.parts_at(147, -1, &lay), [buttress([min, min, b, max])]);
    assert_eq!(
        look.parts_at(165, -1, &lay),
        [
            buttress([b, min, b + 44, max]),
            lower_wall([min, min, w, max])
        ]
    );
    assert_eq!(
        look.parts_at(182, -1, &lay),
        [
            buttress([b + 44, min, max, max]),
            lower_wall([w, min, w + 44, max])
        ]
    );
    assert_eq!(AFTER_ROWS[0], (200, -1));
    assert_eq!(
        look.parts_at(200, -1, &lay),
        [lower_wall([w + 44, min, max, max])]
    );
    // The same hexes drawn rightward take none of these strips.
    assert!(look.parts_at(200, 1, &lay).is_empty());
}

#[test]
fn the_moat_covers_a_stack_s_feet_and_its_box_is_drawn_again() {
    let t = siege_tables();
    let stacks = stacks();
    let lay = layout();
    let castle = look(&t, &siege(3), "base:castle", None, &stacks, None).unwrap();
    let count = Draw::CountBox {
        stack: 1,
        x: 1,
        y: 2,
        count: 3,
        tint: [0, 0],
    };
    // Hex 44 (row 2, its box at 476, 170): the moat's 44 × 6 strip, then the box again.
    let strip = |x: i32, y: i32| wall("Moat.pcx", [410, 90], Some([x, y + 36, x + 44, y + 42]));
    assert_eq!(
        castle.over_stack((44, false, false), 4, &lay, Some(&count)),
        [strip(476, 170), count.clone()]
    );
    // Not in the moat: the box again only; a war machine's none.
    assert_eq!(
        castle.over_stack((40, false, false), 4, &lay, Some(&count)),
        core::slice::from_ref(&count)
    );
    assert!(
        castle
            .over_stack((40, false, false), 4, &lay, None)
            .is_empty()
    );
    // A two-hex stack's other hex: ahead facing right, behind facing left.
    assert_eq!(
        castle.over_stack((43, true, false), 4, &lay, None),
        [strip(476, 170)]
    );
    assert_eq!(
        castle.over_stack((45, true, true), 4, &lay, None),
        [strip(476, 170)]
    );
    assert_eq!(castle.over_stack((0, true, true), 4, &lay, None), []);
    // Neither in the last pass, nor for a flier between its hexes (R1-2: drawn off them), nor
    // on the gate row while the gate is not closed.
    for pass in [7, crate::anim::FLYING] {
        assert!(
            castle
                .over_stack((44, false, false), pass, &lay, Some(&count))
                .is_empty()
        );
    }
    let open = Walls {
        gate: GATE_OPEN,
        ..Walls::of(&siege(3))
    };
    let opened = look(&t, &siege(3), "base:castle", Some(open), &stacks, None).unwrap();
    assert!(
        opened
            .over_stack((95, false, false), 4, &lay, Some(&count))
            .is_empty()
    );
    assert_eq!(
        opened.over_stack((44, false, false), 4, &lay, None),
        [strip(476, 170)]
    );
    // Without the moat flag nothing.
    let tower = look(&t, &siege(3), "base:tower", None, &stacks, None).unwrap();
    assert!(
        tower
            .over_stack((44, false, false), 4, &lay, Some(&count))
            .is_empty()
    );
}

/// A draw's name for the order tests: a wall's picture, a shooter, a creature's stack hex, a
/// count box's stack.
fn name(d: &Draw) -> String {
    match d {
        Draw::Background { picture, .. } | Draw::Wall { picture, .. } => picture.clone(),
        Draw::Shooter { outline, .. } => alloc::format!("shooter {outline:?}"),
        Draw::Creature { anchor, .. } => alloc::format!("creature {anchor:?}"),
        Draw::CountBox { stack, .. } => alloc::format!("box {stack}"),
        other => alloc::format!("{other:?}"),
    }
}

#[test]
fn the_scene_draws_the_siege_in_the_redraws_order() {
    use crate::record::tests::{manifest, observation};
    use crate::scene::{Moment, battle_at};
    use h3_protocol::wire::KeyTables;
    let t = siege_tables();
    let lay = layout();
    let keys = manifest(KeyTables {
        creatures: vec!["base:pikeman".into()],
        factions: vec!["base:castle".into(), "base:rampart".into()],
        ..KeyTables::default()
    });
    let mut o = observation();
    o.siege = Some(siege(3));
    // A stack on the moat's hex 44 (row 2), two on row 6 (drawn right to left), the turrets.
    o.stacks = vec![
        stack(0, Side::Attacker, 0, 44, false),
        stack(1, Side::Attacker, 0, 102, false),
        stack(2, Side::Defender, 0, 110, false),
        stack(7, Side::Defender, 0, 254, false),
        stack(8, Side::Defender, 0, 255, false),
        stack(9, Side::Defender, 0, 251, false),
    ];
    let moment = Moment {
        active: Some(7),
        ..Moment::default()
    };
    let names: Vec<String> = battle_at(&o, &moment, &keys, &t, &lay)
        .unwrap()
        .iter()
        .map(name)
        .collect();
    let at = |n: &str| names.iter().position(|x| x == n).unwrap();
    assert_eq!(
        names[..4],
        ["SgCsBack.pcx", "Moat.pcx", "Mlip.pcx", "TpWl.pcx"]
    );
    // The upper tower before the rows, its shooter, the keep's outlined in the active turret's
    // pulse; the lower tower after them, last.
    assert_eq!(names[4..7], ["Tw21.pcx", "shooter None", "Tw2C.pcx"]);
    let keep = alloc::format!("shooter {:?}", Some(lay.outline_after(0)));
    assert!(names.contains(&keep));
    assert_eq!(
        names[names.len() - 3..],
        ["Tw11.pcx", "shooter None", "Tw1C.pcx"]
    );
    // The moat's strip and the box again after the stack on 44; row 6 right to left.
    let box0 = at("box 0");
    assert_eq!(names[box0 + 1..box0 + 3], ["Moat.pcx", "box 0"]);
    assert!(at("box 2") < at("box 1"));
    // The turrets are no stacks drawn: three creatures in all.
    assert_eq!(
        names.iter().filter(|n| n.starts_with("creature")).count(),
        3
    );
    // The gate open: the drawbridge before row 5's passes, its rope after them.
    let open = Moment {
        walls: Some(Walls {
            gate: GATE_OPEN,
            ..Walls::of(&siege(3))
        }),
        ..Moment::default()
    };
    let names: Vec<String> = battle_at(&o, &open, &keys, &t, &lay)
        .unwrap()
        .iter()
        .map(name)
        .collect();
    let (bridge, rope) = (at_in(&names, "Drw2.pcx"), at_in(&names, "DrwC.pcx"));
    assert!(bridge < at_in(&names, "box 1") && at_in(&names, "box 1") > rope);
    assert!(bridge < rope);
    // Below a Fort an ordinary battlefield: no siege drawn; a town the tables lack: an error.
    o.siege.as_mut().unwrap().fort = 0;
    let none = battle_at(&o, &moment, &keys, &t, &lay).unwrap();
    assert!(none.iter().all(|d| !matches!(d, Draw::Wall { .. })));
    o.siege.as_mut().unwrap().faction = 1;
    o.siege.as_mut().unwrap().fort = 3;
    assert!(battle_at(&o, &moment, &keys, &t, &lay).is_err());
    // A faction handle the manifest lacks.
    o.siege.as_mut().unwrap().faction = 9;
    assert!(battle_at(&o, &moment, &keys, &t, &lay).is_err());
}

fn at_in(names: &[String], n: &str) -> usize {
    names.iter().position(|x| x == n).unwrap()
}

#[test]
fn a_shooting_turret_takes_its_record_s_pose() {
    // E17 M4 T2b (design §10.2): the keep's record shooting (group 15, frame 3) facing right is
    // drawn unmirrored, its point the shift left of its own (`siege_parts_at_hex` 0x494EB3), not
    // outlined while it shoots; the lower tower's facing left, as at rest.
    let t = siege_tables();
    let stacks = stacks();
    let lay = layout();
    let pose = |group, frame, right| {
        Some(TurretPose {
            group,
            frame,
            right,
        })
    };
    let context = Context {
        walls: Walls::of(&siege(3)),
        stacks: &stacks,
        active: Some((7, 96)),
        turrets: [pose(15, 3, true), pose(14, 1, false), None],
    };
    let castle = SiegeLook::new(&siege(3), "base:castle", &t, context, 17)
        .unwrap()
        .unwrap();
    let shooter = |group, frame, anchor, mirrored| Draw::Shooter {
        def: "CBDRGN.DEF".into(),
        group,
        frame,
        anchor,
        mirrored,
        outline: None,
        rows: SHOOTER_ROWS,
    };
    assert_eq!(
        castle.parts_at(135, 1, &lay)[1],
        shooter(15, 3, [741, 238], false)
    );
    assert_eq!(
        castle.parts_at(251, -1, &lay)[1],
        shooter(14, 1, [687, 566], true)
    );
}

#[test]
fn an_explosion_draws_in_its_rectangle_over_the_redraws() {
    // E17 M4 T2b (design §10.2, CLT-Q19 (4)): a wall shot's explosion is a list drawn within its
    // rectangle: the battle with the background, the first frame; the battle again without it
    // (the wall hit from frame 5), the next frame over it.
    use crate::anim::{Blast, Landing};
    use crate::record::tests::{manifest, observation};
    use crate::scene::{Moment, battle_at};
    use h3_protocol::wire::KeyTables;
    let t = siege_tables();
    let lay = layout();
    let keys = manifest(KeyTables {
        creatures: vec!["base:pikeman".into()],
        factions: vec!["base:castle".into()],
        ..KeyTables::default()
    });
    let mut o = observation();
    o.siege = Some(siege(3));
    o.stacks = stacks();
    let whole = Walls::of(&siege(3));
    let mut hit = whole;
    hit.parts[6] = 1;
    let moment = Moment {
        walls: Some(whole),
        blast: Some(Blast {
            def: "SGEXPL.DEF".into(),
            landings: vec![Landing {
                at: [437, 27],
                rect: [437, 27, 691, 230],
            }],
            frames: vec![(0, whole), (5, hit)],
        }),
        ..Moment::default()
    };
    let list = battle_at(&o, &moment, &keys, &t, &lay).unwrap();
    let Some(Draw::Within { rect, list: inner }) = list.last() else {
        panic!("no explosion drawn: {list:?}");
    };
    assert_eq!(*rect, [437, 27, 691, 230]);
    let names: Vec<String> = inner.iter().map(name).collect();
    assert_eq!(
        names
            .iter()
            .filter(|n| n.as_str() == "SgCsBack.pcx")
            .count(),
        1
    );
    let blasts: Vec<usize> = inner
        .iter()
        .enumerate()
        .filter(|(_, d)| matches!(d, Draw::Explosion { def, rows: 203, .. } if def == "SGEXPL.DEF"))
        .map(|(i, _)| i)
        .collect();
    assert_eq!(blasts.len(), 2);
    let (whole_at, hit_at) = (at_in(&names, "Wa61.pcx"), at_in(&names, "Wa62.pcx"));
    assert!(whole_at < blasts[0] && blasts[0] < hit_at && hit_at < blasts[1]);
    // Outside it the battle as the moment's walls show it: whole.
    assert!(list.iter().any(|d| name(d) == "Wa61.pcx"));
}

#[test]
fn an_earthquake_s_explosions_land_on_every_target_and_its_shake_shifts_the_screen() {
    // E17 M4 T2c-2 (design §10.2): each frame's redraw, then the frame over every target hit in
    // turn; the list within each landing's rectangle. The shake: the whole frame's list shifted by
    // the offsets of the steps shown, the battlefield shown.
    use crate::anim::{Blast, Landing, SHAKE, shake_offsets};
    use crate::record::tests::{manifest, observation};
    use crate::scene::{Moment, battle_at, shaken};
    use h3_protocol::wire::KeyTables;
    let t = siege_tables();
    let lay = layout();
    let keys = manifest(KeyTables {
        creatures: vec!["base:pikeman".into()],
        factions: vec!["base:castle".into()],
        ..KeyTables::default()
    });
    let mut o = observation();
    o.siege = Some(siege(3));
    o.stacks = stacks();
    let whole = Walls::of(&siege(3));
    let landings = vec![
        Landing {
            at: [393, 111],
            rect: [393, 111, 647, 314],
        },
        Landing {
            at: [393, 279],
            rect: [393, 279, 647, 482],
        },
    ];
    let moment = Moment {
        walls: Some(whole),
        blast: Some(Blast {
            def: "SGEXPL.DEF".into(),
            landings,
            frames: vec![(0, whole), (1, whole)],
        }),
        ..Moment::default()
    };
    let list = battle_at(&o, &moment, &keys, &t, &lay).unwrap();
    let within: Vec<(&[i32; 4], &Vec<Draw>)> = list
        .iter()
        .filter_map(|d| match d {
            Draw::Within { rect, list } => Some((rect, list)),
            _ => None,
        })
        .collect();
    assert_eq!(within.len(), 2);
    assert_eq!(
        (within[0].0, within[1].0),
        (&[393, 111, 647, 314], &[393, 279, 647, 482])
    );
    assert_eq!(within[0].1, within[1].1);
    let blasts: Vec<(i32, i32, u32)> = within[0]
        .1
        .iter()
        .filter_map(|d| match d {
            Draw::Explosion { x, y, frame, .. } => Some((*x, *y, *frame)),
            _ => None,
        })
        .collect();
    assert_eq!(
        blasts,
        [(393, 111, 0), (393, 279, 0), (393, 111, 1), (393, 279, 1)]
    );
    // The shake: 16 steps, the 15 offsets and the first again.
    let offsets = shake_offsets(16);
    assert_eq!(
        (offsets.len(), offsets[0], offsets[15]),
        (16, SHAKE[0], SHAKE[0])
    );
    let Draw::Shaken {
        offsets: o16,
        rect,
        list: inner,
    } = shaken(list.clone(), 16, &lay)
    else {
        panic!("not shaken");
    };
    assert_eq!((o16, rect, inner), (offsets, [0, 0, 800, 556], list));
}
