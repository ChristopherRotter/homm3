//! The mouse's tests (E17 M3 T6c-1, design §9.9), on the committed layout and an invented pack
//! (the original's text never enters git, ADR-0009).

use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

use h3_protocol::wire::{Command, Hero, Observation, Side, Siege, Stack, Tactics};
use h3_render::Bugs;

use super::track::Look;
use super::{
    DEFEND, Facing, Field, Hint, Kind, Legal, Pointer, Slot, Spot, WAIT, attack_slots, button,
    click, cursor, distance, look_cursor, neighbour, pointer, pointer_in, slot, slots, spot,
    walls_halve,
};
use crate::layout::{BattleLayout, parse_battle};
use crate::record::tests::observation;
use crate::scene::Draw;
use crate::scene::tests::stack;
use h3_ui::text::Catalogue;
use h3_ui::text::pack::Pack;

const PACK: &str = r#"{
    "battle": {
        "hint": {
            "move": "Walk {{creature}}.",
            "fly": "Fly {{creature}}.",
            "attack": "Hit {{creature}}",
            "view": "See {{creature}}.",
            "hero": "Hero.",
            "enemy-hero": "Foe.",
            "heal": "Mend {{creature}}",
            "no-shots": "Empty!",
            "cast-on": "Zap {{creature}} with {{spell}}",
            "cast": "Zap {{spell}}",
            "turret": "Tower.",
            "remove-obstacle": "Clear.",
            "select-target": "Pick.",
            "protects": "{{spell}} keeps {{creature}}",
            "hostile-only": "{{spell}}: foes",
            "friendly-only": "{{spell}}: friends",
            "not-on": "{{spell}} not on {{creature}}",
            "immune": "{{creature}} shrug {{spell}}"
        },
        "control": { "wait": "Hold" }
    },
    "creature": { "t:ant_one": "Ant", "t:ant_other": "Ants" },
    "spell": { "t:slow": "Slow" },
    "wall": { "t:castle": { "6": "Top Wall" } }
}"#;

pub(super) fn layout() -> BattleLayout {
    parse_battle(include_str!("../../../../layouts/battle.toml")).unwrap()
}

pub(super) fn catalogue() -> Catalogue {
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
        sex: None,
    })
}

/// The attacker's stack 0 acting on hex 86 (row 5, column 1), the defender's 1 on 90, the
/// attacker's 2 on 120, the defender's 3 far away on 100; both heroes.
pub(super) fn battle() -> Observation {
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
pub(super) fn centre(hex: u8) -> [i32; 2] {
    let [x, y] = layout().hex_box(hex / 17, hex % 17);
    [x + 22, y + 26]
}

fn at(o: &Observation, commands: &[Command], point: [i32; 2]) -> Pointer {
    pointer(o, &Legal::new(commands), &layout(), (point, Bugs::Fixed))
}

#[test]
fn the_mouse_finds_hexes_heroes_and_nothing() {
    let l = layout();
    assert_eq!(spot(&l, [0, 16], false), Spot::Hero(Side::Attacker));
    assert_eq!(spot(&l, [57, 127], false), Spot::Hero(Side::Attacker));
    assert_eq!(spot(&l, [799, 16], false), Spot::Hero(Side::Defender));
    assert_eq!(spot(&l, [58, 127], false), Spot::Hex(0));
    for hex in [0, 18, 86, 90, 186] {
        assert_eq!(spot(&l, centre(hex), false), Spot::Hex(hex));
    }
    // Above a box's slanted edge: from odd row 1 into even row 0 on the left (a column less), from
    // even row 2 into odd row 1 on the right (a column more); on the right of row 1 and the left
    // of row 2 the column stays.
    assert_eq!(spot(&l, [60, 130], false), Spot::Hex(0));
    assert_eq!(spot(&l, [100, 130], false), Spot::Hex(1));
    assert_eq!(spot(&l, [120, 172], false), Spot::Hex(19));
    assert_eq!(spot(&l, [82, 172], false), Spot::Hex(18));
    // Below the edge the hex's own row.
    assert_eq!(spot(&l, [80, 137], false), Spot::Hex(18));
    // Off the field: above row 0's edge, left of row 1's first box's edge, an even row's first
    // 22 pixels, outside the boxes.
    assert_eq!(spot(&l, [70, 87], false), Spot::Nothing);
    assert_eq!(spot(&l, [16, 130], false), Spot::Nothing);
    assert_eq!(spot(&l, [5, 200], false), Spot::Nothing);
    assert_eq!(spot(&l, [20, 200], false), Spot::Nothing);
    assert_eq!(spot(&l, [100, 60], false), Spot::Nothing);
    assert_eq!(spot(&l, [100, 86 + 472], false), Spot::Nothing);
    assert_eq!(spot(&l, [14 + 748, 300], false), Spot::Nothing);
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
    // At the field's edge (hex 100, column 15): no hex right of it, its slots copied.
    assert_eq!(slots(&o, &legal(&[(3, 99)]), 100), [8; 12]);
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
    let moves = [
        Command::Move { to: 87 },
        Command::Move { to: 103 },
        Command::Wait {},
    ];
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
    // The cursor at the mouse with its frame's hot spot; none beyond the table.
    assert_eq!(
        hit.cursor(&l, [301, 200]),
        Draw::Cursor {
            def: "CRCOMBAT.DEF".to_string(),
            frame: 8,
            x: 301,
            y: 200,
            hot: [30, 6],
        }
    );
    let beyond = Pointer { frame: 99, ..hit };
    assert!(matches!(
        beyond.cursor(&l, [300, 200]),
        Draw::Cursor { hot: [0, 0], .. }
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
    let spell = |h: u16| (h == 54).then(|| "spell.t:slow".to_string());
    let text = |h: Hint| h.text(&c, &o, (&name, &spell, &|_, _| None));
    // E17 M3 T7b-1: a spell aimed at a stack, at no stack; an unknown spell named by nothing.
    assert_eq!(text(Hint::Cast(54, Some(1))), "Zap Ants with Slow");
    assert_eq!(text(Hint::Cast(54, None)), "Zap Slow");
    assert_eq!(text(Hint::Cast(7, None)), "Zap ");
    assert!(!Hint::Cast(54, Some(1)).forced());
    // E17 M5 T3b: the aim's other lines; the reasons name a stack by its number (Anti-Magic's)
    // or by its plural name whatever its number, a stack without a name by nothing.
    assert_eq!(text(Hint::RemoveObstacle), "Clear.");
    assert_eq!(text(Hint::SelectTarget), "Pick.");
    assert_eq!(text(Hint::Protects(54, 1)), "Slow keeps Ants");
    assert_eq!(text(Hint::HostileOnly(54)), "Slow: foes");
    assert_eq!(text(Hint::FriendlyOnly(7)), ": friends");
    assert_eq!(text(Hint::NotOn(54, 1)), "Slow not on Ants");
    assert_eq!(text(Hint::Immune(1, 54)), "Ants shrug Slow");
    assert_eq!(text(Hint::Immune(2, 54)), " shrug Slow");
    assert_eq!(text(Hint::Immune(9, 54)), " shrug Slow");
    assert_eq!(text(Hint::Move(0)), "Walk Ants.");
    assert_eq!(text(Hint::Fly(0)), "Fly Ants.");
    assert_eq!(text(Hint::Attack(1)), "Hit Ants");
    assert_eq!(text(Hint::View(1)), "See Ant.");
    assert_eq!(text(Hint::Heal(1)), "Mend Ants");
    assert_eq!(text(Hint::Hero), "Hero.");
    assert_eq!(text(Hint::EnemyHero), "Foe.");
    assert_eq!(text(Hint::Control("battle.control.wait".into())), "Hold");
    assert_eq!(text(Hint::Empty), "");
    assert_eq!(text(Hint::NoShots), "Empty!");
    // A stack without a name, or none.
    assert_eq!(text(Hint::View(2)), "See .");
    assert_eq!(text(Hint::View(9)), "See .");
    assert!(!Hint::Move(0).forced());
}

#[test]
fn a_look_draws_its_cursor() {
    // E17 M5 T3b: the combat cursor's frame with its hot spot, the spell cursor's with its one.
    let l = layout();
    assert_eq!(
        look_cursor(&l, Look::Combat(6), [5, 7]),
        cursor(&l, 6, [5, 7])
    );
    assert_eq!(
        look_cursor(&l, Look::Spell(3), [5, 7]),
        Draw::Cursor {
            def: "CRSPELL.DEF".into(),
            frame: 3,
            x: 5,
            y: 7,
            hot: [18, 27],
        }
    );
}

/// A siege of a town of `fort` (E17 M4 T2c-2): the attacker's Catapult (4, its war machine on
/// 119) and Archers (0, on 88), the defender's Hobgoblins (1, on 98) inside the walls; the gate
/// closed, every wall standing.
fn siege(fort: u8) -> Observation {
    let mut o = battle();
    let mut catapult = stack(4, Side::Attacker, 0, 119, true);
    catapult.war_machine = true;
    let mut archers = stack(0, Side::Attacker, 0, 88, false);
    archers.shooter = true;
    archers.shots = 12;
    o.stacks = vec![archers, stack(1, Side::Defender, 0, 98, false), catapult];
    o.siege = Some(Siege {
        faction: 0,
        fort,
        moat: 0,
        gate_open: false,
        walls: vec![12, 29, 45, 62, 78, 96, 112, 130, 147, 165, 182],
        moat_hexes: Vec::new(),
        parts: vec![2; 8],
    });
    o
}

/// The Catapult's legal commands: a shot at each target in `standing`.
fn catapult(standing: &[u8]) -> Legal {
    let shots: Vec<Command> = standing
        .iter()
        .map(|&part| Command::ShootWall { part })
        .collect();
    let mut legal = Legal::new(&shots);
    legal.catapult = true;
    legal
}

/// The mouse at `point` in `o` for `legal`, the profile `bugs`.
fn siege_at(o: &Observation, legal: &Legal, point: [i32; 2], bugs: Bugs) -> Pointer {
    let field = Field {
        observation: o,
        legal,
        facing: &Facing::Sides,
        bugs,
    };
    pointer_in(&field, &layout(), point, &mut Some(1))
}

const KEEP: [i32; 2] = [770, 250];
const TOWER: [i32; 2] = [600, 40];

#[test]
fn the_siege_rectangles_take_the_mouse_in_a_siege_or_the_original_s() {
    // E17 M4 T2c-2 (CLT-Q21): the keep's 742–799 × 160–337, the upper tower's 564–651 × 0–85,
    // bounds inclusive, after the heroes'.
    let l = layout();
    assert_eq!(spot(&l, [742, 160], true), Spot::Keep);
    assert_eq!(spot(&l, [799, 337], true), Spot::Keep);
    assert_eq!(spot(&l, [564, 0], true), Spot::Tower);
    assert_eq!(spot(&l, [651, 85], true), Spot::Tower);
    assert_eq!(spot(&l, [799, 127], true), Spot::Hero(Side::Defender));
    assert_ne!(spot(&l, [741, 200], true), Spot::Keep);
    assert_ne!(spot(&l, [652, 40], true), Spot::Tower);
    // Outside a siege the client finds the hex below; the original's rectangles take the mouse
    // in every battle (kind 0 there), marking no stack.
    let mut o = battle();
    o.stacks.push(stack(5, Side::Defender, 0, 84, false));
    let point = centre(84);
    assert_eq!(spot(&l, point, true), Spot::Keep);
    assert_eq!(spot(&l, point, false), Spot::Hex(84));
    let legal = Legal::new(&[]);
    let fixed = siege_at(&o, &legal, point, Bugs::Fixed);
    assert_eq!(fixed.marked, Some(5));
    let original = siege_at(&o, &legal, point, Bugs::Original);
    assert_eq!((original.kind, original.marked), (Kind::Nothing, None));
    // A town without a fort is no siege (R2-2): the hex below, as outside a town.
    let mut town = siege(0);
    town.stacks.push(stack(5, Side::Defender, 0, 84, false));
    assert_eq!(siege_at(&town, &legal, point, Bugs::Fixed).marked, Some(5));
    let original = siege_at(&town, &legal, point, Bugs::Original);
    assert_eq!((original.kind, original.marked), (Kind::Nothing, None));
}

#[test]
fn the_catapult_aims_at_the_standing_targets() {
    // E17 M4 T2c-2: a Castle, every target standing: the rectangles and the aim hexes give the
    // Catapult's target (16) and "Attack %s" with the part; the stack target cleared.
    let o = siege(3);
    let all = catapult(&[0, 1, 2, 3, 4, 5, 6, 7]);
    let aim = |point| siege_at(&o, &all, point, Bugs::Fixed);
    for (point, t) in [
        (TOWER, 0),
        (centre(29), 1),
        (centre(62), 2),
        (centre(96), 3),
        (centre(130), 4),
        (centre(182), 5),
        (centre(183), 6),
        (KEEP, 7),
    ] {
        let p = aim(point);
        assert_eq!(
            (p.kind, p.frame, p.hint),
            (Kind::Wall, 16, Hint::Wall(t)),
            "{t}"
        );
    }
    // Over a stack, the Catapult aims as over an empty hex: nothing to do on the Archers.
    let p = aim(centre(88));
    assert_eq!((p.kind, p.marked), (Kind::Nothing, Some(0)));
    // A target down: its rectangle shows the turret's info (22, the query's frame), its aim hex
    // nothing but the lower tower's hex, which shows its info.
    let some = catapult(&[1, 2, 3, 4, 5]);
    for point in [TOWER, KEEP, centre(183)] {
        let p = siege_at(&o, &some, point, Bugs::Fixed);
        assert_eq!((p.kind, p.frame, p.hint), (Kind::Turret, 5, Hint::Turret));
    }
    let gone = catapult(&[1, 3, 4, 5]);
    assert_eq!(
        siege_at(&o, &gone, centre(62), Bugs::Fixed).kind,
        Kind::Nothing
    );
    // A Citadel: no upper tower (its rectangle nothing), the keep a target; a Fort: neither.
    let citadel = siege(2);
    let p = |o: &Observation, point| siege_at(o, &all, point, Bugs::Fixed).kind;
    assert_eq!(
        (
            p(&citadel, TOWER),
            p(&citadel, KEEP),
            p(&citadel, centre(183))
        ),
        (Kind::Nothing, Kind::Wall, Kind::Nothing)
    );
    let fort = siege(1);
    assert_eq!(
        (p(&fort, TOWER), p(&fort, KEEP)),
        (Kind::Nothing, Kind::Nothing)
    );
    assert_eq!(p(&fort, centre(29)), Kind::Wall);
    // The kinds' numbers: the cursor's frame for 16, the query's for 22.
    assert_eq!((Kind::Wall.code(), Kind::Turret.code()), (16, 22));
}

#[test]
fn the_walls_halve_a_shot_through_them() {
    // E17 M4 T2c-2 (`shot_wall_test` 0x467510): the Archers on 88 outside, the Hobgoblins on 98
    // inside, 10 hexes apart: only the closed gate on the line halves the shot, the broken
    // arrow (15).
    let mut o = siege(3);
    o.active = Some(0);
    let mut legal = Legal::new(&[Command::Shoot { target: 1, at: 98 }]);
    let over = |o: &Observation, legal: &Legal| siege_at(o, legal, centre(98), Bugs::Fixed);
    let p = over(&o, &legal);
    assert_eq!(
        (p.kind, p.frame, p.hint),
        (Kind::BrokenArrow, 15, Hint::Attack(1))
    );
    // The gate open: a shot in full; a shooter the walls never halve, too.
    let mut open = o.clone();
    open.siege.as_mut().unwrap().gate_open = true;
    assert_eq!(over(&open, &legal).kind, Kind::Shoot);
    legal.wall_exempt = true;
    assert_eq!(over(&o, &legal).kind, Kind::Shoot);
    // The test itself: outside to inside, a fort, a wall on the line.
    let s = o.siege.clone().unwrap();
    assert!(walls_halve(&s, 88, 98));
    assert!(!walls_halve(&s, 88, 95));
    assert!(!walls_halve(&s, 97, 98));
    let mut none = s.clone();
    none.fort = 0;
    assert!(!walls_halve(&none, 88, 98));
    // Hex 187 and beyond have no row's wall.
    assert!(!walls_halve(&s, 200, 98));
    assert!(!walls_halve(&s, 88, 200));
    // A turret's shot is never halved (the arrow tower, 0x4767B4).
    let mut turret = stack(5, Side::Defender, 0, 254, false);
    turret.war_machine = true;
    let mut t = siege(3);
    t.stacks.push(turret);
    t.active = Some(5);
    let shoot = Legal::new(&[Command::Shoot { target: 0, at: 88 }]);
    assert_eq!(
        siege_at(&t, &shoot, centre(88), Bugs::Fixed).kind,
        Kind::Shoot
    );
}

#[test]
fn the_siege_s_hints_name_the_walls_and_the_turrets() {
    // E17 M4 T2c-2: "Attack %s" with the target's part's name for the town's type (the upper
    // wall, target 1, is part 6); "View arrow tower info".
    let o = siege(3);
    let c = catalogue();
    let part = |_, p: usize| Some(alloc::format!("wall.t:castle.{p}"));
    let named = |h: Hint| h.text(&c, &o, (&|_| None, &|_| None, &part));
    assert_eq!(named(Hint::Wall(1)), "Hit Top Wall");
    assert_eq!(named(Hint::Turret), "Tower.");
    assert!(!Hint::Wall(1).forced() && !Hint::Turret.forced());
    // Outside a siege, or a target without a part, no name.
    let plain = battle();
    let text = |o: &Observation, h: Hint| h.text(&c, o, (&|_| None, &|_| None, &|_, _| None));
    assert_eq!(text(&plain, Hint::Wall(1)), "Hit ");
    assert_eq!(text(&o, Hint::Wall(9)), "Hit ");
}

/// A point of `hex` in its slot `s` (the first of its box, row by row).
pub(super) fn slot_point(hex: u8, s: usize) -> [i32; 2] {
    let l = layout();
    let [left, top] = l.hex_box(hex / 17, hex % 17);
    (top..top + 52)
        .flat_map(|y| (left..left + 44).map(move |x| [x, y]))
        .find(|&p| spot(&l, p, false) == Spot::Hex(hex) && slot(&l, p, hex) == s)
        .unwrap()
}

/// The command a press at `point` gives in `o` for `commands`, each stack facing its side's way.
fn press(o: &Observation, commands: &[Command], point: [i32; 2]) -> Option<Command> {
    let legal = Legal::new(commands);
    let field = Field {
        observation: o,
        legal: &legal,
        facing: &Facing::Sides,
        bugs: Bugs::Fixed,
    };
    click(&field, &layout(), point)
}

#[test]
fn a_press_gives_the_command_the_cursor_shows() {
    // E17 M5 T2a (`battle_click_action` 0x476BD0, design §9.11).
    let o = battle();
    let moves = [Command::Move { to: 87 }, Command::Move { to: 103 }];
    // A walk or a flight to the hex; nothing where the cursor shows nothing, itself, outside the
    // battlefield or with no stack acting.
    assert_eq!(
        press(&o, &moves, centre(87)),
        Some(Command::Move { to: 87 })
    );
    let mut flier = o.clone();
    flier.stacks[0].flies = true;
    assert_eq!(
        press(&flier, &moves, centre(103)),
        Some(Command::Move { to: 103 })
    );
    assert_eq!(press(&o, &moves, centre(88)), None);
    assert_eq!(press(&o, &moves, centre(86)), None);
    assert_eq!(press(&o, &moves, [700, 570]), None);
    assert_eq!(press(&o, &moves, [20, 60]), None);
    let mut idle = o.clone();
    idle.active = None;
    assert_eq!(press(&idle, &moves, centre(87)), None);
    // A two-hex stack facing right: a reached position's front hex walks to the position, even
    // where the hex is a reached position too (0x476C5F: the reach map's +0x20F first).
    let mut wide = o.clone();
    wide.stacks[0].two_hex = true;
    assert_eq!(
        press(&wide, &moves, centre(104)),
        Some(Command::Move { to: 103 })
    );
    let both = [Command::Move { to: 103 }, Command::Move { to: 104 }];
    assert_eq!(
        press(&wide, &both, centre(104)),
        Some(Command::Move { to: 103 })
    );
    assert_eq!(
        press(&wide, &both, centre(103)),
        Some(Command::Move { to: 103 })
    );
    assert_eq!(press(&wide, &both, centre(87)), None);
    // A shot at the stack: the hex aimed at where the target may be aimed at there (an area
    // shot's), else the target's.
    let mut wide_target = o.clone();
    wide_target.stacks[1].two_hex = true;
    let front = [Command::Shoot { target: 1, at: 90 }];
    assert_eq!(
        press(&wide_target, &front, centre(89)),
        Some(Command::Shoot { target: 1, at: 90 })
    );
    let area = [
        Command::Shoot { target: 1, at: 90 },
        Command::Shoot { target: 1, at: 89 },
    ];
    assert_eq!(
        press(&wide_target, &area, centre(89)),
        Some(Command::Shoot { target: 1, at: 89 })
    );
    assert_eq!(
        press(&o, &[Command::Shoot { target: 3, at: 100 }], centre(100)),
        Some(Command::Shoot { target: 3, at: 100 })
    );
    // A strike from the slot's position.
    let strikes = [
        Command::Attack {
            target: 1,
            from: 73,
        },
        Command::Attack {
            target: 1,
            from: 106,
        },
    ];
    assert_eq!(
        press(&o, &strikes, slot_point(90, 0)),
        Some(Command::Attack {
            target: 1,
            from: 73
        })
    );
    assert_eq!(
        press(&o, &strikes, slot_point(90, 7)),
        Some(Command::Attack {
            target: 1,
            from: 106
        })
    );
    // The tent's heal; its own side's stack without one, nothing.
    assert_eq!(
        press(&o, &[Command::Heal { target: 2 }], centre(120)),
        Some(Command::Heal { target: 2 })
    );
    assert_eq!(press(&o, &[], centre(120)), None);
    // A wall shot at a standing target.
    let s = siege(3);
    let all = catapult(&[0, 1, 2, 3, 4, 5, 6, 7]);
    let field = Field {
        observation: &s,
        legal: &all,
        facing: &Facing::Sides,
        bugs: Bugs::Fixed,
    };
    let l = layout();
    assert_eq!(
        click(&field, &l, KEEP),
        Some(Command::ShootWall { part: 7 })
    );
    assert_eq!(
        click(&field, &l, centre(29)),
        Some(Command::ShootWall { part: 1 })
    );
    assert_eq!(click(&field, &l, centre(88)), None);
}

#[test]
fn a_slot_strikes_from_the_position_the_executable_takes() {
    // E17 M5 T2a (`cursor_attack_slots` 0x474690 whole): each slot's frame with its position, a
    // slot without one copying both.
    let o = battle();
    let legal = |c: &[(u8, u8)]| {
        Legal::new(
            &c.iter()
                .map(|&(target, from)| Command::Attack { target, from })
                .collect::<Vec<_>>(),
        )
    };
    let field = |o: &Observation, legal: &Legal| -> [Slot; 12] {
        let f = Field {
            observation: o,
            legal,
            facing: &Facing::Sides,
            bugs: Bugs::Fixed,
        };
        attack_slots(&f, 90)
    };
    let froms = |slots: [Slot; 12]| slots.map(|s| s.from.unwrap_or(0));
    let one = field(&o, &legal(&[(1, 73), (1, 106)]));
    let by_frame = one.map(|s| if s.frame == 10 { 73 } else { 106 });
    assert_eq!(froms(one), by_frame);
    assert_eq!(field(&o, &legal(&[])), [Slot::default(); 12]);
    // A two-hex stack facing right: up right from 73 (the sword) or 72 (the vertical one).
    let mut wide = o.clone();
    wide.stacks[0].two_hex = true;
    let both = field(&wide, &legal(&[(1, 73), (1, 72)]));
    assert_eq!(
        both[..2],
        [
            Slot {
                frame: 14,
                from: Some(72)
            },
            Slot {
                frame: 10,
                from: Some(73)
            }
        ]
    );
    // From the left: the position beside (88) when taken, else the neighbour (89).
    assert_eq!(froms(field(&wide, &legal(&[(1, 88)]))), [88; 12]);
    assert_eq!(froms(field(&wide, &legal(&[(1, 88), (1, 89)])))[8], 88);
    assert_eq!(froms(field(&wide, &legal(&[(1, 89)]))), [89; 12]);
    // Facing left: down right only the neighbour (107, vertical), down left both (slot 6 the
    // vertical from 107, slot 7 the sword from 106).
    let mut left = wide.clone();
    left.stacks[0].side = Side::Defender;
    left.stacks[0].controller = Side::Defender;
    left.stacks[1].side = Side::Attacker;
    left.stacks[1].controller = Side::Attacker;
    let slots = field(&left, &legal(&[(1, 106), (1, 107)]));
    assert_eq!(
        slots.map(|s| s.frame),
        [7, 7, 7, 7, 13, 13, 13, 7, 7, 7, 7, 7]
    );
    assert_eq!(
        froms(slots),
        [106, 106, 106, 106, 107, 107, 107, 106, 106, 106, 106, 106]
    );
    // From the right facing left: the position beside (92) when taken.
    assert_eq!(froms(field(&left, &legal(&[(1, 91), (1, 92)])))[2], 92);
}

#[test]
fn the_panel_s_wait_and_defend_give_their_commands_when_listed() {
    let both = Legal::new(&[Command::Wait {}, Command::Defend {}]);
    assert_eq!(button(&both, WAIT), Some(Command::Wait {}));
    assert_eq!(button(&both, DEFEND), Some(Command::Defend {}));
    assert_eq!(button(&both, 2001), None);
    // Waited already (or the Catapult): no Wait; neither without a turn.
    let defend = Legal::new(&[Command::Defend {}]);
    assert_eq!(button(&defend, WAIT), None);
    assert_eq!(button(&Legal::default(), DEFEND), None);
    assert_eq!(button(&Legal::new(&[Command::Retreat {}]), DEFEND), None);
}
