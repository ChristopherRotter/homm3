//! The mouse's tests (E17 M3 T6c-1, design §9.9), on the committed layout and an invented pack
//! (the original's text never enters git, ADR-0009).

use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

use h3_protocol::wire::{Command, Hero, Observation, Side, Stack, Tactics};

use super::{Hint, Kind, Legal, Pointer, Spot, distance, neighbour, pointer, slot, slots, spot};
use crate::layout::{BattleLayout, parse_battle};
use crate::record::tests::observation;
use crate::scene::Draw;
use crate::scene::tests::stack;
use crate::text::Catalogue;
use crate::text::pack::Pack;

const PACK: &str = r#"{
    "battle": {
        "hint": {
            "move": "Walk {{creature}}.",
            "fly": "Fly {{creature}}.",
            "attack": "Hit {{creature}}",
            "view": "See {{creature}}.",
            "hero": "Hero.",
            "enemy-hero": "Foe.",
            "heal": "Mend {{creature}}"
        },
        "control": { "wait": "Hold" }
    },
    "creature": { "t:ant_one": "Ant", "t:ant_other": "Ants" }
}"#;

fn layout() -> BattleLayout {
    parse_battle(include_str!("../../../../layouts/battle.toml")).unwrap()
}

fn catalogue() -> Catalogue {
    Catalogue {
        packs: vec![Pack::from_json("en", PACK).unwrap()],
    }
}

fn hero() -> Option<Hero> {
    Some(Hero {
        hero_type: 0,
        attack: 1,
        defense: 1,
        power: 1,
        knowledge: 1,
        mana: 0,
        max_mana: 0,
        casts: 0,
        commands_ballista: false,
        commands_tent: false,
        commands_catapult: false,
        spell_book: false,
    })
}

/// The attacker's stack 0 acting on hex 86 (row 5, column 1), the defender's 1 on 90, the
/// attacker's 2 on 120, the defender's 3 far away on 100; both heroes.
fn battle() -> Observation {
    let mut o = observation();
    o.stacks = vec![
        stack(0, Side::Attacker, 0, 86, false),
        stack(1, Side::Defender, 0, 90, false),
        stack(2, Side::Attacker, 0, 120, false),
        stack(3, Side::Defender, 0, 100, false),
    ];
    o.active = Some(0);
    o.sides[0].hero = hero();
    o.sides[1].hero = hero();
    o
}

/// A hex's centre on the screen.
fn centre(hex: u8) -> [i32; 2] {
    let [x, y] = layout().hex_box(hex / 17, hex % 17);
    [x + 22, y + 26]
}

fn at(o: &Observation, commands: &[Command], point: [i32; 2]) -> Pointer {
    pointer(o, &Legal::new(commands), &layout(), point)
}

#[test]
fn the_mouse_finds_hexes_heroes_and_nothing() {
    let l = layout();
    assert_eq!(spot(&l, [0, 16]), Spot::Hero(Side::Attacker));
    assert_eq!(spot(&l, [57, 127]), Spot::Hero(Side::Attacker));
    assert_eq!(spot(&l, [799, 16]), Spot::Hero(Side::Defender));
    assert_eq!(spot(&l, [58, 127]), Spot::Hex(0));
    for hex in [0, 18, 86, 90, 186] {
        assert_eq!(spot(&l, centre(hex)), Spot::Hex(hex));
    }
    // Above a box's slanted edge: from odd row 1 into even row 0 on the left (a column less), from
    // even row 2 into odd row 1 on the right (a column more); on the right of row 1 and the left
    // of row 2 the column stays.
    assert_eq!(spot(&l, [60, 130]), Spot::Hex(0));
    assert_eq!(spot(&l, [100, 130]), Spot::Hex(1));
    assert_eq!(spot(&l, [120, 172]), Spot::Hex(19));
    assert_eq!(spot(&l, [82, 172]), Spot::Hex(18));
    // Below the edge the hex's own row.
    assert_eq!(spot(&l, [80, 137]), Spot::Hex(18));
    // Off the field: above row 0's edge, left of row 1's first box's edge, an even row's first
    // 22 pixels, outside the boxes.
    assert_eq!(spot(&l, [70, 87]), Spot::Nothing);
    assert_eq!(spot(&l, [16, 130]), Spot::Nothing);
    assert_eq!(spot(&l, [5, 200]), Spot::Nothing);
    assert_eq!(spot(&l, [20, 200]), Spot::Nothing);
    assert_eq!(spot(&l, [100, 60]), Spot::Nothing);
    assert_eq!(spot(&l, [100, 86 + 472]), Spot::Nothing);
    assert_eq!(spot(&l, [14 + 748, 300]), Spot::Nothing);
}

#[test]
fn neighbours_follow_the_executables_table() {
    // Odd row 5, even row 4.
    let around = |h| (0..6).map(|d| neighbour(h, d)).collect::<Vec<_>>();
    assert_eq!(
        around(90),
        [Some(73), Some(91), Some(107), Some(106), Some(89), Some(72)]
    );
    assert_eq!(
        around(72),
        [Some(56), Some(73), Some(90), Some(89), Some(71), Some(55)]
    );
    // A side column has its inner neighbours only; a neighbour in a side column or off the field
    // is none.
    assert_eq!(around(68), [Some(52), Some(69), Some(86), None, None, None]);
    assert_eq!(around(85), [None, Some(86), None, None, None, None]);
    assert_eq!(
        around(101),
        [None, None, None, Some(117), Some(100), Some(83)]
    );
    assert_eq!(around(86)[3..5], [None, None]);
    assert_eq!(around(1)[0], None);
    assert_eq!(around(180)[2], None);
}

#[test]
fn distances_are_the_executables() {
    assert_eq!(distance(86, 86), 0);
    assert_eq!(distance(86, 90), 4);
    assert_eq!(distance(86, 100), 14);
    assert_eq!(distance(90, 73), 1);
    assert_eq!(distance(90, 56), 2);
    assert_eq!(distance(0, 186), 21);
    assert_eq!(distance(186, 0), 21);
    assert_eq!(distance(16, 170), 21);
    assert_eq!(distance(86, 96), 10);
}

#[test]
fn kinds_have_the_executables_numbers() {
    let codes = [
        Kind::Nothing,
        Kind::Move,
        Kind::Fly,
        Kind::Shoot,
        Kind::Hero,
        Kind::Query,
        Kind::Pointer,
        Kind::Attack,
        Kind::BrokenArrow,
        Kind::Heal,
        Kind::EnemyHero,
    ]
    .map(Kind::code);
    assert_eq!(codes, [0, 1, 2, 3, 4, 5, 6, 7, 15, 17, 21]);
}

#[test]
fn the_slot_is_the_mouses_angle_from_the_hexs_centre() {
    let l = layout();
    let [x, y] = centre(90);
    let at = |dx: i32, dy: i32| slot(&l, [x + dx, y + dy], 90);
    // Clockwise from the top: right of the top, the upper right, the right's upper half.
    assert_eq!([at(1, -20), at(10, -10), at(20, -1)], [0, 1, 2]);
    // The right's lower half (and dy 0), the lower right, right of the bottom.
    assert_eq!([at(20, 0), at(10, 10), at(1, 20), at(0, 0)], [3, 4, 5, 3]);
    // Left of the bottom, the lower left, the left's lower half.
    assert_eq!(
        [at(-1, 20), at(-10, 10), at(-20, 1), at(-20, 0)],
        [6, 7, 8, 8]
    );
    // The left's upper half, the upper left, left of the top.
    assert_eq!([at(-20, -1), at(-10, -10), at(-1, -20)], [9, 10, 11]);
    // The thresholds: 0.58 and 1.73 (29/50 and 173/100).
    assert_eq!(
        [at(29, -50), at(30, -50), at(173, -100), at(174, -100)],
        [0, 1, 1, 2]
    );
}

#[test]
fn slots_point_their_swords_at_the_hex_and_fill_the_rest() {
    let o = battle();
    let legal = |c: &[(u8, u8)]| {
        Legal::new(
            &c.iter()
                .map(|&(target, from)| Command::Attack { target, from })
                .collect::<Vec<_>>(),
        )
    };
    // From up right (73) and down left (106): a slot without a sword copies the nearest by the
    // executable's measure, which makes slot 8 take slot 0's, not its neighbour 7's.
    assert_eq!(
        slots(&o, &legal(&[(1, 73), (1, 106)]), 90),
        [10, 10, 10, 10, 10, 10, 7, 7, 10, 10, 10, 10]
    );
    assert_eq!(slots(&o, &legal(&[(1, 89)]), 90), [8; 12]);
    assert_eq!(slots(&o, &legal(&[]), 90), [0; 12]);
    // No target, no acting stack.
    assert_eq!(slots(&o, &legal(&[(1, 89)]), 91), [0; 12]);
    let mut idle = o.clone();
    idle.active = None;
    assert_eq!(slots(&idle, &legal(&[(1, 89)]), 90), [0; 12]);
    // A two-hex stack facing right: the left side's position beside (88) keeps the sword; up
    // right only beside (72): the vertical sword from above; both: slot 0 vertical, 1 the sword.
    let mut wide = o.clone();
    wide.stacks[0].two_hex = true;
    assert_eq!(slots(&wide, &legal(&[(1, 88)]), 90), [8; 12]);
    assert_eq!(slots(&wide, &legal(&[(1, 72)]), 90), [14; 12]);
    let both = slots(&wide, &legal(&[(1, 73), (1, 72)]), 90);
    assert_eq!(both[..2], [14, 10]);
    // Facing left (the defender's): down right prefers beside, down left the neighbour.
    let mut left = wide.clone();
    left.stacks[0].side = Side::Defender;
    left.stacks[0].controller = Side::Defender;
    left.stacks[1].side = Side::Attacker;
    left.stacks[1].controller = Side::Attacker;
    assert_eq!(
        slots(&left, &legal(&[(1, 106), (1, 107)]), 90),
        [7, 7, 7, 7, 13, 13, 13, 7, 7, 7, 7, 7]
    );
    assert_eq!(slots(&left, &legal(&[(1, 107)]), 90), [13; 12]);
}

#[test]
fn over_the_field_the_cursor_follows_the_hex() {
    let o = battle();
    let l = layout();
    let moves = [Command::Move { to: 87 }, Command::Move { to: 103 }];
    // An empty hex in reach: walk; out of reach or in a side column: nothing.
    let walk = at(&o, &moves, centre(87));
    assert_eq!(
        (walk.kind, walk.frame, &walk.hint),
        (Kind::Move, 1, &Hint::Move(0))
    );
    assert_eq!(at(&o, &moves, centre(88)).kind, Kind::Nothing);
    assert_eq!(at(&o, &moves, centre(85)).kind, Kind::Nothing);
    let mut flier = o.clone();
    flier.stacks[0].flies = true;
    let fly = at(&flier, &moves, centre(103));
    assert_eq!(
        (fly.kind, fly.frame, &fly.hint),
        (Kind::Fly, 2, &Hint::Fly(0))
    );
    // A two-hex stack: a reached position's front hex too.
    let mut wide = o.clone();
    wide.stacks[0].two_hex = true;
    assert_eq!(at(&wide, &moves, centre(104)).kind, Kind::Move);
    assert_eq!(at(&wide, &moves, centre(105)).kind, Kind::Nothing);
    // Itself and its own side's stacks: query, marked.
    let me = at(&o, &moves, centre(86));
    assert_eq!((me.kind, me.frame, me.marked), (Kind::Query, 5, Some(0)));
    assert_eq!(me.hint, Hint::View(0));
    assert_eq!(at(&o, &moves, centre(120)).hint, Hint::View(2));
    // An enemy: struck with the slot's sword, shot, at half damage far away; else nothing.
    let strike = [Command::Attack {
        target: 1,
        from: 89,
    }];
    let hit = at(&o, &strike, centre(90));
    assert_eq!(
        (hit.kind, hit.frame, hit.marked),
        (Kind::Attack, 8, Some(1))
    );
    assert_eq!(hit.hint, Hint::Attack(1));
    assert!(hit.hint.forced());
    let unreached = at(&o, &strike, centre(100));
    assert_eq!(
        (unreached.kind, unreached.hint),
        (Kind::Nothing, Hint::Empty)
    );
    assert_eq!(unreached.marked, Some(3));
    let shots = [
        Command::Shoot { target: 1, at: 90 },
        Command::Shoot { target: 3, at: 100 },
    ];
    assert_eq!(at(&o, &shots, centre(90)).kind, Kind::Shoot);
    let far = at(&o, &shots, centre(100));
    assert_eq!(
        (far.kind, far.frame, far.hint),
        (Kind::BrokenArrow, 15, Hint::Attack(3))
    );
    let mut sharp = o.clone();
    sharp.stacks[0].no_distance_penalty = true;
    assert_eq!(at(&sharp, &shots, centre(100)).kind, Kind::Shoot);
    // A two-hex target's other hex within 10: full damage.
    let mut wide_far = o.clone();
    wide_far.stacks[3].two_hex = true;
    wide_far.stacks[3].hex = 98;
    assert_eq!(at(&wide_far, &shots, centre(97)).kind, Kind::BrokenArrow);
    let mut near = wide_far.clone();
    near.stacks[3].hex = 97;
    assert_eq!(at(&near, &shots, centre(96)).kind, Kind::Shoot);
    // The tent's patient; the tactics phase looks at every stack.
    let heal = at(&o, &[Command::Heal { target: 2 }], centre(120));
    assert_eq!(
        (heal.kind, heal.frame, heal.hint),
        (Kind::Heal, 17, Hint::Heal(2))
    );
    let mut tactics = o.clone();
    tactics.tactics = Some(Tactics {
        side: Side::Attacker,
        columns: 3,
    });
    assert_eq!(at(&tactics, &strike, centre(90)).kind, Kind::Query);
    // The heroes: its own side's, the other's (frame 5), none without one; nothing off the hexes.
    let own = at(&o, &[], [20, 60]);
    assert_eq!((own.kind, own.frame, own.hint), (Kind::Hero, 4, Hint::Hero));
    let foe = at(&o, &[], [760, 60]);
    assert_eq!(
        (foe.kind, foe.frame, foe.hint),
        (Kind::EnemyHero, 5, Hint::EnemyHero)
    );
    let mut alone = o.clone();
    alone.sides[1].hero = None;
    assert_eq!(at(&alone, &[], [760, 60]).kind, Kind::Nothing);
    assert_eq!(at(&o, &[], [100, 60]).kind, Kind::Nothing);
    // No acting stack: nothing; a dead stack's hex is empty.
    let mut idle = o.clone();
    idle.active = None;
    assert_eq!(at(&idle, &moves, centre(87)).kind, Kind::Nothing);
    let mut dead = o.clone();
    dead.stacks[1].alive = false;
    assert_eq!(
        at(&dead, &[Command::Move { to: 90 }], centre(90)).kind,
        Kind::Move
    );
    // The cursor's canvas at the mouse less its hot spot.
    assert_eq!(
        hit.cursor(&l, [300, 200]),
        Draw::Cursor {
            def: "CRCOMBAT.DEF".to_string(),
            frame: 8,
            x: 270,
            y: 194
        }
    );
    let beyond = Pointer { frame: 99, ..hit };
    assert!(matches!(
        beyond.cursor(&l, [300, 200]),
        Draw::Cursor { x: 300, y: 200, .. }
    ));
    // The software cursor's x is even (0x50CE04).
    assert!(matches!(
        beyond.cursor(&l, [301, 200]),
        Draw::Cursor { x: 300, y: 200, .. }
    ));
}

#[test]
fn over_the_panel_the_pointer_shows_the_controls_hint() {
    let o = battle();
    // Wait (2009, at 696 + 0, 556 + 5): its hint; the log's item: none; the picture: none.
    let wait = at(&o, &[], [700, 570]);
    assert_eq!((wait.kind, wait.frame), (Kind::Pointer, 6));
    assert_eq!(wait.hint, Hint::Control("battle.control.wait".into()));
    assert_eq!(wait.marked, None);
    assert_eq!(at(&o, &[], [400, 580]).hint, Hint::Empty);
    assert_eq!(at(&o, &[], [620, 590]).hint, Hint::Empty);
    assert_eq!(at(&o, &[], [-1, 300]).kind, Kind::Pointer);
    assert_eq!(at(&o, &[], [800, 300]).kind, Kind::Pointer);
}

#[test]
fn hints_are_worded_by_the_catalogue() {
    let o = battle();
    let c = catalogue();
    let name = |s: &Stack| (s.id != 2).then(|| "creature.t:ant".to_string());
    let text = |h: Hint| h.text(&c, &o, &name);
    assert_eq!(text(Hint::Move(0)), "Walk Ants.");
    assert_eq!(text(Hint::Fly(0)), "Fly Ants.");
    assert_eq!(text(Hint::Attack(1)), "Hit Ants");
    assert_eq!(text(Hint::View(1)), "See Ant.");
    assert_eq!(text(Hint::Heal(1)), "Mend Ants");
    assert_eq!(text(Hint::Hero), "Hero.");
    assert_eq!(text(Hint::EnemyHero), "Foe.");
    assert_eq!(text(Hint::Control("battle.control.wait".into())), "Hold");
    assert_eq!(text(Hint::Empty), "");
    // A stack without a name, or none.
    assert_eq!(text(Hint::View(2)), "See .");
    assert_eq!(text(Hint::View(9)), "See .");
    assert!(!Hint::Move(0).forced());
}
