//! The grid's and the shadows' tests (E17 M6 T4c, design §12.16).

use alloc::collections::BTreeSet;
use alloc::vec;
use alloc::vec::Vec;

use h3_protocol::wire::{Command, Observation, Side, Tactics};
use h3_render::Bugs;

use super::*;
use crate::mouse::aim::{Aim, Area, BERSERK, Target};
use crate::mouse::{Facing, Field, Legal};
use crate::record::tests::observation;
use crate::scene::tests::stack;

/// The attacker's stack 0 acting on hex 86, a human's; the defender's 1 on 90, two hexes wide
/// (89 and 90); the attacker's 2 on 18; the defender's 3 on 1.
fn battle() -> Observation {
    let mut o = observation();
    o.stacks = vec![
        stack(0, Side::Attacker, 0, 86, false),
        stack(1, Side::Defender, 0, 90, true),
        stack(2, Side::Attacker, 0, 18, false),
        stack(3, Side::Defender, 0, 1, false),
    ];
    o.active = Some(0);
    o.sides[0].human = true;
    o
}

fn field<'a>(o: &'a Observation, legal: &'a Legal, bugs: Bugs) -> Field<'a> {
    Field {
        observation: o,
        legal,
        facing: &Facing::Sides,
        bugs,
    }
}

/// An aim numbered `original` with `area`.
fn aim(original: u16, area: Option<Area>) -> Aim {
    Aim {
        spell: 1,
        original,
        creature: false,
        targets: vec![Target::Hex(86)],
        area,
        stacks: false,
        polarity: -1,
        level: 3,
        anti_magic: None,
        mastery: 0,
    }
}

const fn area(radius: u8, centre: bool) -> Area {
    Area { radius, centre }
}

#[test]
fn the_options_come_from_the_settings_and_their_keys_toggle_them() {
    let mut settings = Settings::CAPTURE;
    settings.cursor_shade = true;
    let options = Options::of(&settings);
    assert_eq!(
        options,
        Options {
            grid: false,
            cursor: true,
            movement: false
        }
    );
    let all = options
        .toggled(Toggle::Grid)
        .toggled(Toggle::Movement)
        .toggled(Toggle::Cursor)
        .toggled(Toggle::Cursor);
    assert_eq!((all.grid, all.cursor, all.movement), (true, true, true));
    all.set_in(&mut settings);
    assert!(settings.grid && settings.cursor_shade && settings.movement_shade);
    // An aim with both shadows on turns the movement shadow off; Teleport's and Sacrifice's none,
    // and none with either shadow off.
    let off = Options {
        movement: false,
        ..all
    };
    assert_eq!(all.aiming(21), Some(off));
    assert_eq!(all.aiming(FIRE_WALL), Some(off));
    assert_eq!(all.aiming(TELEPORT), None);
    assert_eq!(all.aiming(SACRIFICE), None);
    assert_eq!(off.aiming(21), None);
    assert_eq!(options.toggled(Toggle::Movement).aiming(21), Some(options));
    assert_eq!(
        Options::default().toggled(Toggle::Movement).aiming(21),
        None
    );
}

#[test]
fn the_grid_and_the_mouse_s_hex_leave_the_side_columns_out() {
    let gridded_hexes: Vec<u8> = (0..=255).filter(|&h| gridded(h)).collect();
    assert_eq!(gridded_hexes.len(), 11 * 15);
    assert_eq!((gridded_hexes[0], gridded_hexes[164]), (1, 185));
    assert!(!gridded(16) && !gridded(17) && !gridded(187));
    assert_eq!(mouse_hexes(90), (90, vec![90]));
    for off in [0, 16, 17, 186, 187, 0xFC, 0xFD, -1] {
        assert_eq!(mouse_hexes(off), (-1, Vec::new()), "{off}");
    }
}

#[test]
fn an_area_keeps_the_inner_columns_and_its_centre_by_its_shape() {
    let of =
        |centre: i32, a: Area| -> Vec<u8> { (0..187).filter(|&h| in_area(centre, a, h)).collect() };
    // Radius 1 around 90 (row 5, odd): the hex and its six neighbours; Frost Ring's without it.
    assert_eq!(of(90, area(1, true)), [72, 73, 89, 90, 91, 106, 107]);
    assert_eq!(of(90, area(1, false)), [72, 73, 89, 91, 106, 107]);
    assert_eq!(of(90, area(2, true)).len(), 19);
    assert_eq!(of(90, area(0, true)), [90]);
    // A side column's hex: its area's part in columns 1 to 15, never itself.
    assert_eq!(of(34, area(1, true)), [18, 35, 52]);
    // Off the field: −1 is row 0's column −1 (oddity 5): hexes 1 and 18 at radius 2, none at
    // radius 1; the heroes' rectangles none.
    assert_eq!(of(-1, area(2, true)), [1, 18]);
    assert!(of(-1, area(1, true)).is_empty());
    assert!(of(0xFC, area(2, true)).is_empty());
}

#[test]
fn an_area_aim_shades_its_hexes_and_off_the_field_the_original_s_alone() {
    let (o, legal) = (battle(), Legal::default());
    let fixed = field(&o, &legal, Bugs::Fixed);
    let original = field(&o, &legal, Bugs::Original);
    let fireball = aim(21, Some(area(1, true)));
    // The defender's two-hex stack's hexes stay: the marks hold it.
    assert_eq!(
        area_shade(&fireball, &fixed, 90),
        [72, 73, 89, 90, 91, 106, 107]
    );
    assert!(area_shade(&aim(15, None), &fixed, 90).is_empty());
    // Inferno off the field: the original's hexes 1 and 18 and its marks of their stacks; the
    // client's none (CLT-Q55).
    let inferno = aim(22, Some(area(2, true)));
    assert_eq!(area_shade(&inferno, &original, -1), [1, 18]);
    assert_eq!(inferno.marks(&original, -1), BTreeSet::from([2, 3]));
    assert!(area_shade(&inferno, &fixed, -1).is_empty());
    assert!(inferno.marks(&fixed, -1).is_empty());
    // Berserk at Expert off the field: the original's hexes but its stacks', its marks not read
    // (none).
    let berserk = aim(BERSERK, Some(area(2, true)));
    assert!(berserk.marks(&original, -1).is_empty());
    assert!(area_shade(&berserk, &original, -1).is_empty());
    let mut alone = battle();
    alone.stacks.truncate(2);
    let alone_field = field(&alone, &legal, Bugs::Original);
    assert_eq!(area_shade(&berserk, &alone_field, -1), [1, 18]);
}

#[test]
fn the_wall_s_aim_shades_fire_wall_s_hexes_alone() {
    assert!(wall(FORCE_FIELD) && wall(FIRE_WALL) && !wall(21));
    assert!(wall_shade(FORCE_FIELD, 90, (Side::Attacker, 3)).is_empty());
    // Row 5 (odd): the upper right neighbour 17 lower for the attacker, the upper left 18 lower
    // for the defender; from Advanced 34 lower.
    assert_eq!(wall_shade(FIRE_WALL, 90, (Side::Attacker, 1)), [90, 73]);
    assert_eq!(wall_shade(FIRE_WALL, 90, (Side::Defender, 1)), [90, 72]);
    assert_eq!(wall_shade(FIRE_WALL, 90, (Side::Attacker, 2)), [90, 73, 56]);
    // Row 6 (even): 16 and 17 lower.
    assert_eq!(wall_shade(FIRE_WALL, 104, (Side::Attacker, 0)), [104, 88]);
    assert_eq!(
        wall_shade(FIRE_WALL, 104, (Side::Defender, 3)),
        [104, 87, 70]
    );
    // Off the field above row 0: left out.
    assert_eq!(wall_shade(FIRE_WALL, 10, (Side::Attacker, 3)), [10]);
}

#[test]
fn the_movement_shadow_shades_the_acting_stack_s_reach() {
    let mut o = battle();
    let legal = Legal::new(&[
        Command::Move { to: 87 },
        Command::Move { to: 70 },
        Command::Attack {
            target: 1,
            from: 88,
        },
        Command::Attack {
            target: 9,
            from: 88,
        },
    ]);
    let on = field(&o, &legal, Bugs::Fixed);
    assert_eq!(reach(&on), BTreeSet::from([70, 86, 87, 89, 90]));
    assert_eq!(movement(&on, true), reach(&on));
    assert!(movement(&on, false).is_empty());
    // A two-hex mover's front hex at each position, by its facing now.
    o.stacks[0].two_hex = true;
    let wide = field(&o, &legal, Bugs::Fixed);
    assert_eq!(reach(&wide), BTreeSet::from([70, 71, 86, 87, 88, 89, 90]));
    let left = Facing::Left(BTreeSet::from([0, 1]));
    let turned = Field {
        facing: &left,
        ..wide
    };
    assert_eq!(reach(&turned), BTreeSet::from([69, 70, 85, 86, 87, 89, 90]));
    // None for a stack no human plays, a dead one or none.
    o.sides[0].human = false;
    assert!(reach(&field(&o, &legal, Bugs::Fixed)).is_empty());
    o.sides[0].human = true;
    o.stacks[0].alive = false;
    assert!(reach(&field(&o, &legal, Bugs::Fixed)).is_empty());
    o.active = None;
    assert!(reach(&field(&o, &legal, Bugs::Fixed)).is_empty());
    // The tactics phase's own whatever the option.
    let mut phase = battle();
    phase.tactics = Some(Tactics {
        side: Side::Attacker,
        columns: 3,
    });
    let in_phase = field(&phase, &legal, Bugs::Fixed);
    assert_eq!(movement(&in_phase, false), crate::tactics::shade(&phase));
    assert!(!movement(&in_phase, false).is_empty());
}

/// The cursor shadow on, the grid and the movement shadow by `grid` and `movement`.
const fn on(grid: bool, movement: bool) -> Options {
    Options {
        grid,
        cursor: true,
        movement,
    }
}

#[test]
fn the_cursor_shadow_follows_its_key_and_puts_its_blocks_back() {
    let reach = BTreeSet::from([86, 87]);
    let mut cells = Cells::new(Options::default(), Bugs::Original);
    // Off: nothing.
    cells.mouse(90, false);
    assert_eq!(cells.drawn(&reach), Drawn::default());
    cells.compose(on(false, true));
    assert_eq!(cells.options(), on(false, true));
    cells.mouse(90, false);
    let drawn = cells.drawn(&reach);
    assert_eq!(
        (drawn.shade, drawn.overlays.cursor),
        (reach.clone(), vec![90])
    );
    // The same key again does nothing unless forced; at most 19 hexes.
    cells.shade(90, &[1, 2], false);
    assert_eq!(cells.drawn(&reach).overlays.cursor, [90]);
    let many: Vec<u8> = (1..30).collect();
    cells.shade(90, &many, true);
    assert_eq!(cells.drawn(&reach).overlays.cursor.len(), 19);
    // F6 to F8 take the shade off first: nothing put back stale.
    cells.set(on(true, false));
    let drawn = cells.drawn(&reach);
    assert!(drawn.shade.is_empty() && drawn.overlays.grid);
    assert!(drawn.overlays.cursor.is_empty() && drawn.overlays.stale.is_empty());
    cells.mouse(90, false);
    // The options dialog composes without the shade's call (oddity 2): the resting hex unshaded,
    // its key kept; the next hex puts the old block back over the new grid.
    cells.compose(on(false, false));
    cells.mouse(90, false);
    assert!(cells.drawn(&reach).overlays.cursor.is_empty());
    cells.mouse(91, false);
    let overlays = cells.drawn(&reach).overlays;
    assert_eq!(overlays.cursor, [91]);
    assert_eq!(
        overlays.stale,
        [Stale {
            hex: 90,
            grid: true,
            shade: BTreeSet::new(),
        }]
    );
    // Back over the stale block: its save is the old block, put back again as the mouse leaves.
    cells.mouse(90, false);
    cells.mouse(-1, false);
    let overlays = cells.drawn(&reach).overlays;
    assert!(overlays.cursor.is_empty());
    assert_eq!(overlays.stale.len(), 1);
    // An action's start takes the shade off, the old blocks with it.
    cells.mouse(92, false);
    cells.action();
    assert_eq!(cells.drawn(&reach).overlays, Overlays::default());
    // A composition with the movement shadow on draws the reach; a stale block keeps its own.
    cells.compose(on(false, true));
    cells.mouse(92, false);
    cells.compose(on(false, false));
    cells.mouse(93, false);
    let drawn = cells.drawn(&reach);
    assert!(drawn.shade.is_empty());
    assert_eq!(drawn.overlays.stale[0].shade, reach);
}

#[test]
fn the_client_s_composition_forgets_the_cursor_shadow_s_list() {
    // CLT-Q52 (fixed): after the dialog the mouse's hex shades again on the new background, and
    // no block of the old one comes back.
    let reach = BTreeSet::new();
    let mut cells = Cells::new(on(false, false), Bugs::Fixed);
    cells.mouse(90, false);
    cells.compose(on(true, false));
    cells.mouse(90, false);
    let overlays = cells.drawn(&reach).overlays;
    assert_eq!(overlays.cursor, [90]);
    cells.mouse(91, false);
    assert!(cells.drawn(&reach).overlays.stale.is_empty());
}
