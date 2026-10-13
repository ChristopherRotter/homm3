use super::*;
use alloc::string::ToString;
use alloc::vec;
use h3_protocol::wire::{DamageRoll, KeyTables, RemovableObstacle};

use crate::layout::parse_battle;
use crate::record::tests::{manifest, observation};
use crate::tables::{load, parse};

pub(crate) const TABLE: &str = r#"
[table]
id = "base"
depends = []

[battle]
groups = { moving = 0, mouse_over = 1, standing = 2 }

[[creature]]
key = "pikeman"
def = "CPKMAN.DEF"
times = [9000, 575, 500, 115]

[[creature]]
key = "dragon"
def = "CBDRGN.DEF"
times = [9000, 575, 500, 115]

[[battlefield]]
key = "grass"
backgrounds = ["CmBkGrDd.pcx", "CmBkGrMt.pcx", "CmBkGrTr.pcx"]

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
        creatures: list(&["base:pikeman", "base:dragon", "base:ghost"]),
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
            // Row 3: the defender's dragon, from its position (its rear hex).
            creature("CBDRGN.DEF", 3, 16, true, None),
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
            creature("CBDRGN.DEF", 4, 9, false, None),
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
fn an_empty_battle_draws_nothing() {
    let list = battle(&observation(), &keys(), &tables(), &layout()).unwrap();
    assert_eq!(list, []);
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
