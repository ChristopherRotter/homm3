use super::*;
use alloc::string::ToString;
use alloc::vec;
use h3_protocol::wire::{DamageRoll, KeyTables, RemovableObstacle};

use crate::layout::parse_battle;
use crate::ray::Stroke;
use crate::record::tests::{manifest, observation};
use crate::tables::{load, parse};

pub(crate) const TABLE: &str = r#"
[table]
id = "base"
depends = []

[battle]
groups = { moving = 0, mouse_over = 1, standing = 2, hit = 3, defend = 4, death = 5, strike = [11, 12, 13], shoot = [14, 15, 16], breath = [17, 18, 19], move_start = 20, move_end = 21, turn_left = [7, 10], turn_right = [9, 8] }
hero_groups = { standing = 0, idle = 1, defeat = 2, victory = 3, cast = 4 }
flags = ["CmFlagL.def", "CmFlagR.def"]
magic = [{ def = "Heal.def", place = 1, translucent = true }]
heal_sound = "Regener"

[[hero]]
key = "orrin"
original = 0
def = "CH00.DEF"

[[hero]]
key = "crag_hack"
original = 102
def = "CH013.DEF"

[[creature]]
key = "pikeman"
original = 0
def = "CPKMAN.DEF"
times = [9000, 575, 500, 115]

[[creature]]
key = "dragon"
original = 26
def = "CBDRGN.DEF"
times = [9000, 575, 500, 115]

[[battlefield]]
key = "grass"
backgrounds = ["CmBkGrDd.pcx", "CmBkGrMt.pcx", "CmBkGrTr.pcx"]

[[battlefield]]
key = "shore"
backgrounds = ["CmBkBch.pcx"]

[[obstacle]]
key = "obstacle_1"
def = "Back.def"
rows = 1
layer = 1

[[obstacle]]
key = "obstacle_2"
def = "Front.def"
rows = 2
layer = 0

[[obstacle]]
key = "obstacle_100"
pcx = "Abs.pcx"
at = [124, 254]
"#;

pub(crate) fn tables() -> Tables {
    load(&[parse(TABLE).unwrap()], "base").unwrap()
}

pub(crate) fn layout() -> BattleLayout {
    parse_battle(include_str!("../../../../layouts/battle.toml")).unwrap()
}

/// The manifest's handles: each table's last key has no entry in [`TABLE`].
pub(crate) fn keys() -> Manifest {
    let list = |keys: &[&str]| keys.iter().map(|k| (*k).to_string()).collect();
    manifest(KeyTables {
        terrains: list(&["base:grass", "base:lava"]),
        battlefields: list(&["base:shore", "base:deck"]),
        creatures: list(&["base:pikeman", "base:dragon", "base:ghost"]),
        hero_types: list(&["base:orrin", "base:crag_hack", "base:sandro"]),
        obstacles: list(&[
            "base:obstacle_1",
            "base:obstacle_2",
            "base:obstacle_100",
            "base:obstacle_9",
        ]),
        ..KeyTables::default()
    })
}

/// A stack with every field but the ones that place it at its defaults.
pub(crate) fn stack(id: u8, side: Side, creature: u16, hex: u8, two_hex: bool) -> Stack {
    Stack {
        id,
        side,
        controller: side,
        slot: id,
        creature,
        hex,
        alive: true,
        count: 10,
        base_count: 10,
        top_hp: 10,
        max_hp: 10,
        attack: 1,
        defense: 1,
        damage_min: 1,
        damage_max: 3,
        speed: 4,
        morale: 0,
        luck: 0,
        shots: 0,
        retaliations: 1,
        retaliations_left: 1,
        regeneration: 0,
        acted: false,
        waited: false,
        retaliated: false,
        defending: false,
        had_morale: false,
        started: false,
        two_hex,
        flies: false,
        shooter: false,
        war_machine: false,
        no_distance_penalty: false,
        shoots_when_adjacent: false,
        native: false,
        created: false,
        clone: false,
        raised: false,
        clone_rounds: 0,
        clone_link: None,
        temporary: 0,
        corpse: 0,
        bound_by: Vec::new(),
        poison: 0,
        start_hp: 10,
        casts_left: 0,
        cast_candidates: 0,
        melee_attack_bonus: 0,
        ranged_attack_bonus: 0,
        damage_roll: DamageRoll::Random {},
        speed_percent: 100,
        effects: Vec::new(),
    }
}

fn removable(anchor: u8, hexes: &[u8], kind: Option<u16>) -> RemovableObstacle {
    RemovableObstacle {
        anchor,
        hexes: hexes.to_vec(),
        kind,
    }
}

/// Hex `column` of `row` on the 17-hex rows.
const fn hex(row: u8, column: u8) -> u8 {
    17 * row + column
}

/// The battle of the order test: obstacles of both layers and stacks on rows 1, 3 and 4.
pub(crate) fn battle_view() -> Observation {
    let mut o = observation();
    o.terrain = Some(0);
    o.absolute_obstacle = Some(2);
    o.active = Some(0);
    o.removable_obstacles = vec![
        // Layer 1 in column 0: the layer-1 pass reads columns 1-15 only, so never drawn.
        removable(hex(2, 0), &[hex(2, 0)], Some(0)),
        removable(hex(3, 5), &[hex(3, 5)], Some(0)),
        removable(hex(4, 8), &[hex(4, 8), hex(3, 8)], Some(1)),
        // No type: not drawn.
        removable(hex(6, 6), &[hex(6, 6)], None),
        // Its hexes in the cleared set (an anchor another removal cleared, BAT-QUIRK-3): still
        // listed, so still on the field and drawn (a removed obstacle leaves the list).
        removable(hex(7, 6), &[hex(7, 6), hex(7, 7)], Some(1)),
    ];
    o.obstacles_cleared = vec![hex(7, 6), hex(7, 7)];
    let mut dead = stack(3, Side::Defender, 0, hex(5, 16), false);
    dead.alive = false;
    o.stacks = vec![
        stack(0, Side::Attacker, 0, hex(4, 0), false),
        // Two-hex: the engine's position is the rear hex, drawn from (BAT-GEO-002, protocol v23).
        stack(1, Side::Attacker, 1, hex(4, 9), true),
        stack(2, Side::Defender, 1, hex(3, 16), true),
        dead,
        // Above the layer-1 obstacles' rows: still drawn after them (§4.6 step 1).
        stack(4, Side::Attacker, 0, hex(1, 2), false),
    ];
    o
}

#[test]
fn the_battle_is_drawn_in_the_executables_order() {
    let layout = layout();
    let list = battle(&battle_view(), &keys(), &tables(), &layout).unwrap();
    let creature = |def: &str, row, column, mirrored, outline| Draw::Creature {
        def: def.to_string(),
        group: 2,
        frame: 0,
        anchor: layout.anchor(row, column),
        mirrored,
        outline,
    };
    // Each stack's count box after it (M3 T4a): beside it, the hexes ahead free.
    let counted = |stack, row, column, [dx, dy]: [i32; 2]| {
        let [x, y] = layout.anchor(row, column);
        Draw::CountBox {
            stack,
            x: x + dx,
            y: y + dy,
            count: 10,
            tint: [0x3F40_0000, 0x3F4C_CCCD],
        }
    };
    let [bx, by] = layout.hex_box(3, 5);
    let [fx, fy] = layout.hex_box(4, 8);
    let [gx, gy] = layout.hex_box(7, 6);
    assert_eq!(
        list,
        [
            Draw::Background {
                picture: "CmBkGrTr.pcx".to_string(),
                x: 0,
                y: 0
            },
            Draw::AbsoluteObstacle {
                picture: "Abs.pcx".to_string(),
                x: 124,
                y: 254
            },
            Draw::Obstacle {
                def: "Back.def".to_string(),
                group: 0,
                frame: 0,
                x: bx,
                y: by
            },
            // Row 1: a stack above the layer-1 obstacle's row, drawn after it all the same.
            creature("CPKMAN.DEF", 1, 2, false, None),
            counted(4, 1, 2, [22, -15]),
            // Row 3: the defender's dragon, from its position (its rear hex).
            creature("CBDRGN.DEF", 3, 16, true, None),
            counted(2, 3, 16, [-52 - 44, -30]),
            // Row 4: the layer-0 obstacle in pass 2, raised by its second row; then the stacks
            // in pass 4, left to right, the attacker's dragon from its rear hex.
            Draw::Obstacle {
                def: "Front.def".to_string(),
                group: 0,
                frame: 0,
                x: fx,
                y: fy - 42
            },
            creature("CPKMAN.DEF", 4, 0, false, Some(0x60)),
            counted(0, 4, 0, [22, -15]),
            creature("CBDRGN.DEF", 4, 9, false, None),
            counted(1, 4, 9, [22 + 44, -15]),
            // Row 7: the layer-0 obstacle whose hexes are in the cleared set.
            Draw::Obstacle {
                def: "Front.def".to_string(),
                group: 0,
                frame: 0,
                x: gx,
                y: gy - 42
            },
        ]
    );
}

#[test]
fn a_count_box_shows_by_its_stack_and_the_hex_ahead() {
    // E17 M3 T4a (§4.3): a box for a living stack standing or under the mouse, not a war
    // machine's; beside the stack when the hex ahead is free, else over it (a stack's hex, a
    // two-hex stack's other hex or a blocked one).
    let layout = layout();
    let mut o = observation();
    o.terrain = Some(0);
    let mut machine = stack(5, Side::Attacker, 0, hex(8, 2), false);
    machine.war_machine = true;
    let mut shown = stack(6, Side::Attacker, 0, hex(9, 2), false);
    shown.count = 1234;
    o.stacks = vec![
        // Ahead of 0 stands 1's other hex (it faces left: its rear on 4/4, its other on 4/3).
        stack(0, Side::Attacker, 0, hex(4, 2), false),
        stack(1, Side::Defender, 1, hex(4, 4), true),
        // Ahead of 2 a blocked hex; ahead of 3 nothing.
        stack(2, Side::Attacker, 0, hex(6, 2), false),
        stack(3, Side::Attacker, 0, hex(7, 2), false),
        machine,
        shown,
    ];
    o.obstacles = vec![hex(6, 3)];
    // 3 strikes (group 11): no box; 6 is under the mouse (group 1): its box.
    let pose = |group, hex| Pose {
        group,
        frame: 0,
        hex,
        offset: [0, 0],
        mirrored: false,
        layer: STANDING,
        boxless: false,
    };
    let moment = Moment {
        poses: [(3, pose(11, hex(7, 2))), (6, pose(1, hex(9, 2)))]
            .into_iter()
            .collect(),
        ..Moment::default()
    };
    let list = battle_at(&o, &moment, &keys(), &tables(), &layout).unwrap();
    let boxes: Vec<_> = list
        .iter()
        .filter_map(|d| match d {
            Draw::CountBox { x, y, count, .. } => Some(([*x, *y], *count)),
            _ => None,
        })
        .collect();
    let at = |row, column, [dx, dy]: [i32; 2]| {
        let [x, y] = layout.anchor(row, column);
        [x + dx, y + dy]
    };
    assert_eq!(
        boxes,
        [
            (at(4, 2, [-15, -15]), 10),
            // The dragon faces left: the hex ahead of its other hex is 4/2, taken.
            (at(4, 4, [-(52 + 44 - 37), -15]), 10),
            (at(6, 2, [-15, -15]), 10),
            (at(9, 2, [22, -15]), 1234),
        ]
    );
    // The moment's counts (E17 M3 T5b: a strike's kills from its end) override the snapshot's;
    // its hues (E17 M3 T7b-2a: a stack's spells) the layout's, its saturation kept.
    let moment = Moment {
        counts: [(1, 7), (6, 1200)].into_iter().collect(),
        hues: [(2, 0x3EAA_B368)].into_iter().collect(),
        ..moment
    };
    let list = battle_at(&o, &moment, &keys(), &tables(), &layout).unwrap();
    let counts: Vec<_> = list
        .iter()
        .filter_map(|d| match d {
            Draw::CountBox {
                stack, count, tint, ..
            } => Some((*stack, *count, *tint)),
            _ => None,
        })
        .collect();
    let plain = layout.count_box.tint;
    let green = [0x3EAA_B368, plain[1]];
    assert_eq!(
        counts,
        [
            (0, 10, plain),
            (1, 7, plain),
            (2, 10, green),
            (6, 1200, plain)
        ]
    );
    // A stack on the field's edge: nothing ahead of hex 0 facing left.
    assert_eq!(ahead(0, false, 0), None);
    assert_eq!(ahead(255, true, 0), None);
    assert_eq!(ahead(3, false, 1), Some(1));
}

#[test]
fn a_count_box_follows_the_moments_stacks() {
    // E17 M3 T5a (§4.3): a stack whose box a walk or a flight hides shows none and leaves its
    // hex free for another's box (0x43E61C); a stack that died since the snapshot (among the
    // corpses) has left its hexes (`stack_death` 0x444400).
    let layout = layout();
    let mut o = observation();
    o.terrain = Some(0);
    o.stacks = vec![
        stack(0, Side::Attacker, 0, hex(4, 2), false),
        stack(1, Side::Attacker, 0, hex(4, 3), false),
    ];
    let boxes = |one: Option<Pose>| {
        let moment = Moment {
            poses: one.map(|p| (1, p)).into_iter().collect(),
            ..Moment::default()
        };
        let list = battle_at(&o, &moment, &keys(), &tables(), &layout).unwrap();
        list.iter()
            .filter_map(|d| match d {
                Draw::CountBox { x, y, .. } => Some([*x, *y]),
                _ => None,
            })
            .collect::<Vec<_>>()
    };
    let at = |row, column, [dx, dy]: [i32; 2]| {
        let [x, y] = layout.anchor(row, column);
        [x + dx, y + dy]
    };
    let standing = Pose {
        group: 2,
        frame: 0,
        hex: hex(4, 3),
        offset: [0, 0],
        mirrored: false,
        layer: STANDING,
        boxless: false,
    };
    assert_eq!(
        boxes(Some(standing)),
        [at(4, 2, [-15, -15]), at(4, 3, [22, -15])]
    );
    let hidden = Pose {
        boxless: true,
        ..standing
    };
    assert_eq!(boxes(Some(hidden)), [at(4, 2, [22, -15])]);
    let dead = Pose {
        group: 5,
        layer: 1,
        ..standing
    };
    assert!(!dead.holds_hex() && !hidden.holds_hex() && standing.holds_hex());
    assert_eq!(boxes(Some(dead)), [at(4, 2, [22, -15])]);
}

#[test]
fn a_magic_animation_is_drawn_over_its_stacks_count_box() {
    // E17 M3 T5b-2b (§4.3, §7.15): `draw_battle_stack`'s part 0 draws the sprite, the count box
    // (0x43E568) and then the magic animation (0x43E795); the First Aid Tent's heal lies over the
    // patient's box in `machines`.
    let layout = layout();
    let mut o = observation();
    o.terrain = Some(0);
    o.stacks = vec![stack(0, Side::Attacker, 0, hex(4, 2), false)];
    let moment = Moment {
        magic: Some(Magic {
            def: "Heal.def".to_string(),
            frame: 0,
            translucent: true,
            over: vec![(0, [7, 8], false)],
        }),
        ..Moment::default()
    };
    let list = battle_at(&o, &moment, &keys(), &tables(), &layout).unwrap();
    let order: Vec<_> = list
        .iter()
        .filter_map(|d| match d {
            Draw::Creature { .. } => Some("sprite"),
            Draw::CountBox { .. } => Some("box"),
            Draw::Magic { .. } => Some("magic"),
            _ => None,
        })
        .collect();
    assert_eq!(order, ["sprite", "box", "magic"]);
}

#[test]
fn a_moment_draws_each_stack_in_its_pose() {
    // E17 M2 T4a: the poses of a timeline. In row 4 the Pikemen walk (frame 3 of group 0, moved
    // by their offset) and lose the outline to the active defender; the dragon on 4/9 died: its
    // corpse's layer (T4b-2a) puts it in pass 1, before the layer-0 obstacle of pass 2; the dead
    // defender, with a pose, is drawn as well (its corpse, turned to face right).
    let layout = layout();
    let pose = |group, frame, hex, offset, layer| Pose {
        group,
        frame,
        hex,
        offset,
        mirrored: false,
        layer,
        boxless: false,
    };
    let ray = Ray {
        strokes: vec![Stroke {
            x: 10,
            y: 20,
            lo: -2,
            hi: 2,
            column: true,
        }],
        colours: vec![[1, 2, 3]],
    };
    let moment = Moment {
        poses: [
            (0, pose(0, 3, hex(4, 0), [16, 0], 4)),
            (1, pose(5, 7, hex(4, 9), [0, 0], 1)),
            (3, pose(5, 8, hex(5, 16), [0, 0], 1)),
        ]
        .into_iter()
        .collect(),
        active: Some(2),
        pulse: 3,
        projectile: Some(Projectile {
            def: "PLCBOWX.DEF".to_string(),
            frame: 4,
            at: [100, 50],
            mirrored: true,
        }),
        ray: Some(ray.clone()),
        heroes: [None; 2],
        // E17 M3 T3b: a magic animation over the walking Pikemen (stack 0) and over a stack
        // that is not in the battle.
        magic: Some(Magic {
            def: "Heal.def".to_string(),
            frame: 1,
            translucent: true,
            over: vec![(0, [7, 8], false), (9, [1, 1], true)],
        }),
        spot: None,
        objects: BTreeMap::new(),
        field: None,
        counts: BTreeMap::new(),
        hues: BTreeMap::new(),
        // E17 M3 T6c-1: the mouse marks the walking Pikemen.
        marked: Some(0),
        walls: None,
        turrets: [None; 3],
        blast: None,
        bridge: None,
        shake: None,
    };
    let list = battle_at(&battle_view(), &moment, &keys(), &tables(), &layout).unwrap();
    // The magic animation right after its stack's sprite, once.
    let walker = list
        .iter()
        .position(|d| {
            matches!(
                d,
                Draw::Creature {
                    group: 0,
                    frame: 3,
                    ..
                }
            )
        })
        .unwrap();
    assert_eq!(
        list[walker + 1],
        Draw::Magic {
            def: "Heal.def".to_string(),
            frame: 1,
            x: 7,
            y: 8,
            mirrored: false,
            translucent: true,
        }
    );
    assert_eq!(
        list.iter()
            .filter(|d| matches!(d, Draw::Magic { .. }))
            .count(),
        1
    );
    // T4b-1: the projectile over everything, then the ray (M3 T1b-1); the outline three idle
    // steps on (§4.2).
    assert_eq!(list.last(), Some(&Draw::Ray(ray)));
    assert_eq!(
        list.iter().rev().nth(1),
        Some(&Draw::Projectile {
            def: "PLCBOWX.DEF".to_string(),
            frame: 4,
            x: 100,
            y: 50,
            mirrored: true,
        })
    );
    assert!(list.iter().any(|d| matches!(
        d,
        Draw::Creature {
            outline: Some(0x65),
            ..
        }
    )));
    // The stack the mouse marks: the mouse's range, three steps on (0x70 + 5).
    assert!(list.iter().any(|d| matches!(
        d,
        Draw::Creature {
            group: 0,
            frame: 3,
            outline: Some(0x75),
            ..
        }
    )));
    let creatures: Vec<(String, u32, u32, [i32; 2], bool)> = list
        .iter()
        .filter_map(|d| match d {
            Draw::Creature {
                def,
                group,
                frame,
                anchor,
                outline,
                ..
            } => Some((def.clone(), *group, *frame, *anchor, outline.is_some())),
            _ => None,
        })
        .collect();
    let [x, y] = layout.anchor(4, 0);
    assert_eq!(
        creatures,
        [
            ("CPKMAN.DEF".to_string(), 2, 0, layout.anchor(1, 2), false),
            ("CBDRGN.DEF".to_string(), 2, 0, layout.anchor(3, 16), true),
            ("CBDRGN.DEF".to_string(), 5, 7, layout.anchor(4, 9), false),
            ("CPKMAN.DEF".to_string(), 0, 3, [x + 16, y], true),
            ("CPKMAN.DEF".to_string(), 5, 8, layout.anchor(5, 16), false),
        ]
    );
    let corpse = list.iter().position(|d| {
        matches!(
            d,
            Draw::Creature {
                group: 5,
                frame: 7,
                ..
            }
        )
    });
    let front = list
        .iter()
        .position(|d| matches!(d, Draw::Obstacle { def, .. } if def == "Front.def"));
    assert!(corpse < front);
}

#[test]
fn a_stack_gone_without_a_corpse_is_not_drawn() {
    // E17 M3 T9d: a sacrificed stack (`anim::GONE`, beyond the passes) is drawn by no pass and
    // not as a flier; the living stack on 4/9 without its pose would be.
    let layout = layout();
    let gone = Pose {
        group: 5,
        frame: 5,
        hex: hex(4, 9),
        offset: [0, 0],
        mirrored: false,
        layer: GONE,
        boxless: false,
    };
    let creatures = |moment: &Moment| {
        battle_at(&battle_view(), moment, &keys(), &tables(), &layout)
            .unwrap()
            .iter()
            .filter(|d| matches!(d, Draw::Creature { .. }))
            .count()
    };
    let moment = Moment {
        poses: [(1, gone)].into_iter().collect(),
        ..Moment::default()
    };
    assert_eq!(creatures(&moment) + 1, creatures(&Moment::default()));
}

#[test]
fn a_magic_animation_at_a_hex_is_drawn_over_everything() {
    // E17 M3 T7c-1 (0x496A10): after the stacks, the flier and the projectile, never mirrored.
    let layout = layout();
    let moment = Moment {
        spot: Some(Spot {
            def: "C13spF.def".to_string(),
            frame: 4,
            translucent: false,
            at: [600, 200],
        }),
        projectile: Some(Projectile {
            def: "PLCBOWX.DEF".to_string(),
            frame: 0,
            at: [100, 50],
            mirrored: false,
        }),
        ..Moment::default()
    };
    let list = battle_at(&battle_view(), &moment, &keys(), &tables(), &layout).unwrap();
    assert_eq!(
        list.last(),
        Some(&Draw::Magic {
            def: "C13spF.def".to_string(),
            frame: 4,
            x: 600,
            y: 200,
            mirrored: false,
            translucent: false,
        })
    );
}

#[test]
fn a_flier_between_its_hexes_is_drawn_over_every_row() {
    // E17 M2 T5a: off its hexes (0x468730) the flier is drawn by itself after the redraw
    // (0x4B4D60): over the lower rows' stacks and obstacles, though its position is row 0.
    let layout = layout();
    let flying = Pose {
        group: 0,
        frame: 2,
        hex: hex(0, 3),
        offset: [5, 90],
        mirrored: false,
        layer: crate::anim::FLYING,
        boxless: true,
    };
    let moment = Moment {
        poses: [(0, flying)].into_iter().collect(),
        ..Moment::default()
    };
    let list = battle_at(&battle_view(), &moment, &keys(), &tables(), &layout).unwrap();
    let [x, y] = layout.anchor(0, 3);
    assert_eq!(
        list.last(),
        Some(&Draw::Creature {
            def: "CPKMAN.DEF".to_string(),
            group: 0,
            frame: 2,
            anchor: [x + 5, y + 90],
            mirrored: false,
            outline: None,
        })
    );
    let creatures = list
        .iter()
        .filter(|d| matches!(d, Draw::Creature { .. }))
        .count();
    assert_eq!(creatures, 4);
}

#[test]
fn an_empty_battle_draws_nothing() {
    let list = battle(&observation(), &keys(), &tables(), &layout()).unwrap();
    assert_eq!(list, []);
}

#[test]
fn a_special_battlefield_has_its_own_picture() {
    // 0x4646D0 (E17 M3 T2): a special battlefield's picture, whatever the terrain under it; a
    // battle without a terrain or a special battlefield has no background.
    let (tables, layout) = (tables(), layout());
    let background = |terrain, battlefield| {
        let mut o = observation();
        (o.terrain, o.battlefield) = (terrain, battlefield);
        let list = battle(&o, &keys(), &tables, &layout).unwrap();
        list.into_iter().find_map(|d| match d {
            Draw::Background { picture, .. } => Some(picture),
            _ => None,
        })
    };
    assert_eq!(background(Some(0), Some(0)).as_deref(), Some("CmBkBch.pcx"));
    assert_eq!(background(None, Some(0)).as_deref(), Some("CmBkBch.pcx"));
    assert_eq!(background(Some(0), None).as_deref(), Some("CmBkGrTr.pcx"));
    assert_eq!(background(None, None), None);
}

#[test]
fn what_the_tables_or_the_manifest_lack_is_an_error() {
    let (tables, layout) = (tables(), layout());
    let run = |change: &dyn Fn(&mut Observation)| {
        let mut o = observation();
        change(&mut o);
        battle(&o, &keys(), &tables, &layout).unwrap_err()
    };
    let key = |k: &str| SceneError::Key(k.to_string());
    assert_eq!(
        run(&|o| o.terrain = Some(2)),
        SceneError::Handle("terrains", 2)
    );
    assert_eq!(run(&|o| o.terrain = Some(1)), key("base:lava"));
    assert_eq!(
        run(&|o| o.battlefield = Some(2)),
        SceneError::Handle("battlefields", 2)
    );
    assert_eq!(run(&|o| o.battlefield = Some(1)), key("base:deck"));
    assert_eq!(
        run(&|o| o.absolute_obstacle = Some(4)),
        SceneError::Handle("obstacles", 4)
    );
    assert_eq!(
        run(&|o| o.absolute_obstacle = Some(3)),
        key("base:obstacle_9")
    );
    assert_eq!(
        run(&|o| o.absolute_obstacle = Some(0)),
        SceneError::Art("base:obstacle_1".to_string())
    );
    assert_eq!(
        run(&|o| o.removable_obstacles = vec![removable(5, &[5], Some(2))]),
        SceneError::Art("base:obstacle_100".to_string())
    );
    assert_eq!(
        run(&|o| o.removable_obstacles = vec![removable(5, &[5], Some(3))]),
        key("base:obstacle_9")
    );
    assert_eq!(
        run(&|o| o.stacks = vec![stack(0, Side::Attacker, 3, 5, false)]),
        SceneError::Handle("creatures", 3)
    );
    assert_eq!(
        run(&|o| o.stacks = vec![stack(0, Side::Attacker, 2, 5, false)]),
        key("base:ghost")
    );
    for (e, shown) in [
        (
            SceneError::Handle("terrains", 2),
            "the manifest's terrains have no handle 2",
        ),
        (key("k"), "the asset tables have no entry for `k`"),
        (
            SceneError::Art("k".to_string()),
            "`k` is not drawn that way",
        ),
    ] {
        assert_eq!(e.to_string(), shown);
    }
}

/// A side's hero of type `hero_type`.
fn led(hero_type: u16) -> Option<h3_protocol::wire::Hero> {
    Some(h3_protocol::wire::Hero {
        hero_type,
        attack: 1,
        defense: 1,
        power: 1,
        knowledge: 1,
        mana: 10,
        max_mana: 10,
        casts: 0,
        commands_ballista: false,
        commands_tent: false,
        commands_catapult: false,
        spell_book: false,
        sex: None,
    })
}

#[test]
fn each_hero_and_its_flag_are_drawn_after_the_layer_1_obstacles() {
    // §4.5, 0x494701 (E17 M3 T3a): the left flag and hero, then the right ones, the right hero
    // mirrored; a flag in its player's colours (red, blue: CLT-Q6).
    let (tables, layout) = (tables(), layout());
    let mut o = battle_view();
    o.sides[0].hero = led(0);
    o.sides[1].hero = led(1);
    let flag = |side: usize, frame| Draw::Flag {
        def: ["CmFlagL.def", "CmFlagR.def"][side].to_string(),
        frame,
        x: layout.flag_canvas[side][0],
        y: layout.flag_canvas[side][1],
        palette: "PLAYERS.PAL".to_string(),
        player: u8::try_from(side).unwrap(),
    };
    let hero = |side: usize, group, frame| Draw::Hero {
        def: ["CH00.DEF", "CH013.DEF"][side].to_string(),
        group,
        frame,
        x: layout.hero_canvas[side][0],
        y: layout.hero_canvas[side][1],
        mirrored: side == 1,
    };
    // Without poses: standing, each flag at its start (0x463F29).
    let list = battle(&o, &keys(), &tables, &layout).unwrap();
    let back = list
        .iter()
        .position(|d| matches!(d, Draw::Obstacle { def, .. } if def == "Back.def"))
        .unwrap();
    assert_eq!(
        list[back + 1..back + 5],
        [flag(0, 0), hero(0, 0, 0), flag(1, 3), hero(1, 0, 0)]
    );
    assert!(matches!(list[back + 5], Draw::Creature { .. }));
    // A moment's poses; a side without a hero draws none.
    o.sides[0].hero = None;
    let moment = Moment {
        heroes: [
            None,
            Some(HeroPose {
                group: 1,
                frame: 2,
                flag: 5,
            }),
        ],
        ..Moment::default()
    };
    let list = battle_at(&o, &moment, &keys(), &tables, &layout).unwrap();
    assert_eq!(list[back + 1..back + 3], [flag(1, 5), hero(1, 1, 2)]);
    // A handle the manifest lacks, a key the tables lack.
    o.sides[1].hero = led(3);
    assert_eq!(
        battle(&o, &keys(), &tables, &layout),
        Err(SceneError::Handle("hero types", 3))
    );
    o.sides[1].hero = led(2);
    assert_eq!(
        battle(&o, &keys(), &tables, &layout),
        Err(SceneError::Key("base:sandro".to_string()))
    );
}

#[test]
fn the_panel_draws_its_items_as_the_turn_sets_them() {
    // E17 M3 T6a: the layout's panel, the attacker (red, human) acting, its log's text.
    let layout = layout();
    let mut o = battle_view();
    o.sides[0].human = true;
    let kept = crate::panel::Kept {
        colours: Side::Attacker,
        scroll: false,
    };
    let list = panel(&o, Some(Side::Attacker), kept, "a\nb", &layout);
    let button = |def: &str, frame, x| Draw::Button {
        def: def.to_string(),
        frame,
        x,
        y: 561,
    };
    let scroll = |frame, y| Draw::Button {
        def: "ComSlide.def".to_string(),
        frame,
        x: 624,
        y,
    };
    assert_eq!(
        list,
        [
            Draw::Picture {
                picture: "cbar.pcx".to_string(),
                x: 0,
                y: 556,
                player: Some(PlayerColours {
                    palette: "PLAYERS.PAL".to_string(),
                    player: 0
                })
            },
            // No heroes: no surrender, no retreat, no book.
            button("icm001.def", 2, 54),
            button("icm002.def", 2, 105),
            button("icm003.def", 0, 3),
            button("icm004.def", 0, 156),
            button("icm005.def", 2, 645),
            button("icm006.def", 0, 696),
            button("icm007.def", 0, 747),
            Draw::Text {
                item: layout.log().unwrap().clone(),
                x: 214,
                y: 563,
                text: "a\nb".to_string()
            },
            scroll(0, 561),
            scroll(2, 580),
        ]
    );
}

#[test]
fn the_panel_over_time_follows_the_turns_and_the_log() {
    // E17 M3 T6b: stack 0 the attacker's (Red, human), stack 2 the defender's (a computer).
    let layout = layout();
    let mut o = battle_view();
    o.sides[0].human = true;
    let events = [
        Event::TurnStarted { stack: 2 },
        Event::TurnStarted { stack: 0 },
    ];
    let texts = vec![(0, String::new()), (150, "hit".to_string())];
    let panels = Panels::new(&o, &events, texts);
    let started = [(100, 2), (300, 0)];
    let picture = |list: &[Draw]| match &list[0] {
        Draw::Picture { player, .. } => player.as_ref().map(|p| p.player),
        _ => None,
    };
    let wait = |list: &[Draw]| match &list[6] {
        Draw::Button { frame, .. } => *frame,
        _ => 9,
    };
    // Red's opening: lit; the computer's turn: dimmed in Red's colours, the log's line shown.
    let opening = panels.at(&o, &started, 0, &layout);
    assert_eq!((picture(&opening), wait(&opening)), (Some(0), 0));
    let computer = panels.at(&o, &started, 200, &layout);
    assert_eq!((picture(&computer), wait(&computer)), (Some(0), 2));
    assert_eq!(panels.text(200), "hit");
    assert!(matches!(&computer[8], Draw::Text { text, .. } if text == "hit"));
    assert_eq!(wait(&panels.at(&o, &started, 300, &layout)), 0);
    // E17 M3 T6c-2: Wait pressed shows its click frame; a disabled one takes no press.
    let pressed = panels.at_pressed(&o, &started, 300, &layout, "", Some(2009));
    assert_eq!(wait(&pressed), 1);
    let dimmed = panels.at_pressed(&o, &started, 200, &layout, "", Some(2009));
    assert_eq!(wait(&dimmed), 2);
    // More turns started than the events name: the last named; no opening stack: the attacker's
    // colours.
    assert_eq!(
        wait(&panels.at(&o, &[(0, 2), (1, 0), (2, 2)], 5, &layout)),
        0
    );
    o.active = None;
    let none = Panels::new(&o, &[], Vec::new());
    assert_eq!(picture(&none.at(&o, &[], 0, &layout)), Some(0));
    assert_eq!(none.text(0), "");
    // E17 M3 T9b, the reviewer: the scroll buttons take a click once a human's turn enabled them
    // (the opening's own active stack none: Red's turn at 300 is the first).
    let later = Panels::new(&o, &events, Vec::new());
    assert!(later.scrolls(&started, 300));
    assert!(!later.scrolls(&started, 200));
    assert!(!none.scrolls(&[], 0));
    // Red's opening is a human's turn: enabled from the start.
    assert!(panels.scrolls(&started, 0));
    // The log's texts by time: the last of the list shown by then, whatever their order (a
    // shot's line has its start's time).
    let shot = vec![
        (0, String::new()),
        (100, "a".to_string()),
        (50, "b".to_string()),
    ];
    let shot = Panels::new(&o, &[], shot);
    assert_eq!(
        [shot.text(49), shot.text(60), shot.text(120)],
        ["", "b", "b"]
    );
}

#[test]
fn a_heros_cast_disables_its_spell_book_for_the_turn() {
    // E17 M3 T7b-1 (the recorded `cast`): from the cast's start, the side's hero has cast this
    // round; the other side's cast leaves this turn's book as it is.
    let layout = layout();
    let mut o = battle_view();
    o.sides[0].human = true;
    o.sides[0].hero = led(0).map(|h| h3_protocol::wire::Hero {
        spell_book: true,
        ..h
    });
    let events = [Event::TurnStarted { stack: 0 }];
    let casts = vec![(150, Side::Defender), (200, Side::Attacker)];
    let panels = Panels::new(&o, &events, Vec::new()).with_casts(casts);
    let started = [(100, 0)];
    let book = |t| match &panels.at(&o, &started, t, &layout)[5] {
        Draw::Button { frame, .. } => *frame,
        _ => 9,
    };
    assert_eq!([book(100), book(199), book(200)], [0, 0, 2]);
}

/// The count box's hue under spells (E17 M3 T7b-2a, 0x43E6AA): the executable's f64 and f32
/// arithmetic, checked against it run in floats for every sum of up to 81 signs (all equal); here
/// a few: all positive green, all negative red (0), none or only neutral ones 0.1667, mixes
/// between (r = 1/3, −1/3, 2/3, 1/2, 5/7, −2/5, −79/81).
#[test]
fn a_boxs_hue_follows_its_spells_signs() {
    let hue = |pos: usize, neg: usize, zero: usize| {
        let signs = core::iter::repeat_n(1i8, pos)
            .chain(core::iter::repeat_n(-1, neg))
            .chain(core::iter::repeat_n(0, zero));
        spell_hue(signs)
    };
    assert_eq!(hue(1, 0, 0), 0x3EAA_B368);
    assert_eq!(hue(3, 0, 2), 0x3EAA_B368);
    assert_eq!(hue(0, 1, 0), 0);
    assert_eq!(hue(0, 0, 1), 0x3E2A_B368);
    assert_eq!(hue(1, 1, 0), 0x3E2A_B368);
    assert_eq!(hue(2, 1, 0), 0x3E63_99E0);
    assert_eq!(hue(1, 2, 0), 0x3DE3_99E0);
    assert_eq!(hue(5, 1, 0), 0x3E8E_402C);
    assert_eq!(hue(3, 1, 0), 0x3E80_068E);
    assert_eq!(hue(6, 1, 0), 0x3E92_50A2);
    assert_eq!(hue(3, 7, 0), 0x3DCC_D74A);
    assert_eq!(hue(0, 81, 0), 0);
    assert_eq!(hue(1, 80, 0), 0x3B86_DFED);
}

#[test]
fn spell_objects_go_through_the_obstacles_passes() {
    // E17 M3 T7c-2 (0x494680): a pit (layer 1) before every stack, a Force Field (layer 0, three
    // rows) in its anchor row's pass 2, each at its anchor raised by its rows above, its frame the
    // idle steps' count modulo its frames; the Force Field's hexes keep a count box off them.
    let layout = layout();
    let mut o = observation();
    o.terrain = Some(0);
    o.stacks = vec![stack(0, Side::Attacker, 0, hex(6, 2), false)];
    let object = |def: &str, frames, rows, before_stacks, blocked: Vec<u8>| Object {
        def: def.to_string(),
        frames,
        rows,
        before_stacks,
        blocked,
        owner: None,
    };
    let moment = Moment {
        pulse: 5,
        objects: [
            (
                hex(6, 3),
                object("C15spE10.def", 4, 4, false, vec![hex(6, 3)]),
            ),
            (hex(9, 9), object("C17SPE1.DEF", 1, 1, true, Vec::new())),
        ]
        .into_iter()
        .collect(),
        field: Some(Field {
            def: "C06spF0.def".to_string(),
            frame: 7,
        }),
        ..Moment::default()
    };
    let list = battle_at(&o, &moment, &keys(), &tables(), &layout).unwrap();
    let [px, py] = layout.hex_box(9, 9);
    let [fx, fy] = layout.hex_box(6, 3);
    let pit = list
        .iter()
        .position(|d| {
            *d == Draw::Obstacle {
                def: "C17SPE1.DEF".to_string(),
                group: 0,
                frame: 0,
                x: px,
                y: py,
            }
        })
        .unwrap();
    let field = list
        .iter()
        .position(|d| {
            *d == Draw::Obstacle {
                def: "C15spE10.def".to_string(),
                group: 0,
                frame: 1,
                x: fx,
                y: fy - 3 * 42,
            }
        })
        .unwrap();
    let sprite = list
        .iter()
        .position(|d| matches!(d, Draw::Creature { .. }))
        .unwrap();
    assert!(pit < sprite && field < sprite);
    // The box over its stack: the hex ahead is the Force Field's.
    let [x, y] = layout.anchor(6, 2);
    assert!(
        list.iter()
            .any(|d| matches!(d, Draw::CountBox { x: bx, y: by, .. }
        if [*bx, *by] == [x - 15, y - 15]))
    );
    // Armageddon's frame tiled over the battlefield, last.
    assert_eq!(
        list.last(),
        Some(&Draw::Tiled {
            def: "C06spF0.def".to_string(),
            frame: 7,
            area: layout.mouse.field,
        })
    );
}

#[test]
fn the_drawbridge_s_redraws_show_its_rectangle_alone() {
    // E17 M4 T2c-1 (CLT-Q23): the battle as the screen showed it before, the moment's inside the
    // drawbridge's rectangle (x 365 to 542, y 211 to 380).
    let layout = layout();
    let before = Moment {
        active: Some(1),
        ..Moment::default()
    };
    let moment = Moment {
        bridge: Some(Box::new(before.clone())),
        ..Moment::default()
    };
    let list = battle_at(&battle_view(), &moment, &keys(), &tables(), &layout).unwrap();
    let base = battle_at(&battle_view(), &before, &keys(), &tables(), &layout).unwrap();
    let now = battle_at(
        &battle_view(),
        &Moment::default(),
        &keys(),
        &tables(),
        &layout,
    )
    .unwrap();
    assert_eq!(list[..base.len()], base[..]);
    assert_eq!(
        list.last(),
        Some(&Draw::Within {
            rect: [365, 211, 543, 381],
            list: now,
        })
    );
    assert_eq!(list.len(), base.len() + 1);
}
