//! The army info windows' tests (E17 M6 T4b2b1, T4b2b2, design §12.15), on the committed layout
//! and an invented pack (the original's text never enters git, ADR-0009).

use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use h3_protocol::wire::{Command, Hero, Observation, Side};
use h3_render::Bugs;
use h3_ui::text::Catalogue;
use h3_ui::text::pack::Pack;
use h3_ui::ui::Item;

use super::{
    Drawn, Hover, Kind, STACK_BACKGROUND, STACK_COUNT, STACK_LUCK, STACK_LUCK_LABEL, STACK_MORALE,
    STACK_MORALE_LABEL, STACK_NO_SPELLS, STACK_PORTRAIT, STACK_ROWS, STACK_SPELLS, StackInfo,
    StackShown, Windows, on_field, stack_window,
};
use crate::layout::BattleLayout;
use crate::mouse::{Facing, Field, Legal, Spot, slot, slots};
use crate::record::tests::observation;
use crate::scene::tests::{layout, stack};
use crate::stats::{Effect, Pair, Stats};

const PACK: &str = r#"{
    "battle": {
        "hero-info": {
            "label": "{{word}}:",
            "attack": "A",
            "defense": "D",
            "morale": "M",
            "luck": "L"
        },
        "army-info": {
            "damage": "G",
            "health": "H",
            "pair": "{{own}}({{now}})",
            "range": "{{low}}-{{high}}",
            "no-spells": "none"
        },
        "hint": {
            "number": "{{value}}"
        }
    }
}"#;

fn catalogue() -> Catalogue {
    Catalogue {
        packs: vec![Pack::from_json("en", PACK).unwrap()],
    }
}

fn hero() -> Hero {
    Hero {
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
    }
}

/// A battle with the attacker's hero, and the defender's when `defender`; the attacker's stack 1
/// on hex 90, the defender's two-hex stack 2 on hex 100 (its front 99, facing left) and its dead
/// stack 3 on hex 40.
fn battle(defender: bool) -> Observation {
    let mut o = observation();
    o.sides[0].hero = Some(hero());
    o.sides[1].hero = defender.then(hero);
    let mut dead = stack(3, Side::Defender, 0, 40, false);
    dead.alive = false;
    o.stacks = vec![
        stack(1, Side::Attacker, 0, 90, false),
        stack(2, Side::Defender, 0, 100, true),
        dead,
    ];
    o
}

/// The mouse's battle over `o`, each side facing its own way.
fn field<'a>(o: &'a Observation, legal: &'a Legal) -> Field<'a> {
    Field {
        observation: o,
        legal,
        facing: &Facing::Sides,
        bugs: Bugs::Fixed,
    }
}

/// Stack `stack`'s window shown on `side`'s place, of `kind`, by turn `turn`.
const fn shown(side: Side, kind: Kind, stack: u8, turn: usize) -> StackShown {
    StackShown {
        side,
        kind,
        stack,
        turn,
    }
}

/// The middle of side `index`'s hero's rectangle.
fn on_hero(l: &BattleLayout, index: usize) -> [i32; 2] {
    let [left, top, right, bottom] = l.mouse.heroes[index];
    [(left + right) / 2, (top + bottom) / 2]
}

/// A hex's centre on the screen.
fn centre(l: &BattleLayout, hex: u8) -> [i32; 2] {
    let [x, y] = l.hex_box(hex / 17, hex % 17);
    [x + 22, y + 26]
}

/// The heroes' windows drawn, each side with its turn.
fn drawn(windows: &Windows) -> Vec<(Side, usize)> {
    windows
        .drawn()
        .filter_map(|d| match d {
            Drawn::Hero(side, turn) => Some((side, turn)),
            Drawn::Stack(_) => None,
        })
        .collect()
}

#[test]
fn a_new_hex_shows_at_most_one_hero_s_window_at_levels_1_and_2() {
    let (o, legal) = (battle(true), Legal::default());
    let f = field(&o, &legal);
    for level in 0..3 {
        let mut w = Windows::at_level(level);
        w.enter(Spot::Hero(Side::Attacker), &f, 3);
        let shown = (level != 0).then_some(3);
        assert_eq!(w.heroes, [shown, None], "level {level}");
        // Another hex hides it first; the defender's shows alone.
        w.enter(Spot::Hero(Side::Defender), &f, 4);
        assert_eq!(w.heroes, [None, (level != 0).then_some(4)], "level {level}");
        // A stack's hex, the keep, nothing: no hero's.
        for spot in [Spot::Hex(90), Spot::Keep, Spot::Tower, Spot::Nothing] {
            let mut w = Windows::at_level(level);
            w.heroes = [Some(1), None];
            w.enter(spot, &f, 5);
            assert_eq!(w.heroes, [None, None], "{spot:?} at level {level}");
        }
    }
    // Over a side without a hero: none.
    let o = battle(false);
    let mut w = Windows::at_level(1);
    w.enter(Spot::Hero(Side::Defender), &field(&o, &legal), 0);
    assert_eq!(w, Windows::at_level(1));
}

#[test]
fn a_stack_s_hex_shows_its_window_by_its_owner_s_side_and_the_level() {
    // E17 M6 T4b2b2 (0x475709): kind 1 at level 1, kind 2 at level 2, none at 0; the attacker's
    // stack on its side's place, the defender's on its.
    let (mut o, legal) = (battle(true), Legal::default());
    for (level, kind) in [(1, Kind::All), (2, Kind::Spells)] {
        let mut w = Windows::at_level(level);
        w.heroes = [Some(0), None];
        w.enter(Spot::Hex(90), &field(&o, &legal), 4);
        assert_eq!(w.heroes, [None, None]);
        assert_eq!(w.stack, Some(shown(Side::Attacker, kind, 1, 4)));
        w.enter(Spot::Hex(100), &field(&o, &legal), 5);
        assert_eq!(w.stack, Some(shown(Side::Defender, kind, 2, 5)));
        // Its front hex too (oddity 10: moving between them hides and shows it again, the same
        // window); a hero's rectangle hides it.
        w.enter(Spot::Hex(99), &field(&o, &legal), 5);
        assert_eq!(w.stack, Some(shown(Side::Defender, kind, 2, 5)));
        w.enter(Spot::Hero(Side::Attacker), &field(&o, &legal), 6);
        assert_eq!((w.stack, w.heroes), (None, [Some(6), None]));
        // An empty hex, a dead stack's: none.
        for hex in [91, 40] {
            w.enter(Spot::Hex(hex), &field(&o, &legal), 7);
            assert_eq!(w, Windows::at_level(level), "hex {hex}");
        }
    }
    let mut w = Windows::at_level(0);
    w.enter(Spot::Hex(90), &field(&o, &legal), 1);
    assert_eq!(w, Windows::at_level(0));
    // By its owner's side, not its controller's (oddity 8: a hypnotized stack).
    o.stacks[0].controller = Side::Defender;
    let mut w = Windows::at_level(1);
    w.enter(Spot::Hex(90), &field(&o, &legal), 2);
    assert_eq!(w.stack.map(|s| s.side), Some(Side::Attacker));
    // Where the stack faces: the defender's front hex 101 once it faces right.
    let facing = Facing::Left(alloc::collections::BTreeSet::new());
    let turned = Field {
        facing: &facing,
        ..field(&o, &legal)
    };
    w.enter(Spot::Hex(101), &turned, 3);
    assert_eq!(w.stack, Some(shown(Side::Defender, Kind::All, 2, 3)));
}

#[test]
fn shown_windows_are_drawn_only_while_the_level_is_not_0() {
    let mut w = Windows::at_level(2);
    w.heroes = [Some(1), Some(2)];
    assert_eq!(drawn(&w), [(Side::Attacker, 1), (Side::Defender, 2)]);
    // A stack's after the heroes', its kind kept through a change of the level (§12.14 oddity 6).
    let stack = shown(Side::Defender, Kind::All, 4, 3);
    w.stack = Some(stack);
    assert_eq!(w.drawn().last(), Some(Drawn::Stack(stack)));
    w.level = 0;
    assert_eq!(w.drawn().count(), 0);
    w.level = 2;
    w.stack = None;
    // F5 to 0: still shown (flagged), drawn no more; back to 1: drawn again (§12.14 oddity 6).
    w.level = 0;
    assert_eq!(drawn(&w), []);
    assert_eq!(w.heroes, [Some(1), Some(2)]);
    w.level = 1;
    assert_eq!(drawn(&w).len(), 2);
    w.hide();
    assert_eq!(w, Windows::at_level(1));
}

#[test]
fn the_right_press_keeps_its_window_at_levels_1_and_2() {
    // 0x476A1D: hidden as the button is released at level 0 only.
    let mut w = Windows::at_level(0);
    w.held(Side::Defender, 2);
    assert_eq!(w.heroes, [None, None]);
    // Shown already at level 0 (the level set to 0 under it): the release hides it.
    w.heroes = [None, Some(1)];
    w.held(Side::Defender, 2);
    assert_eq!(w.heroes, [None, None]);
    for level in [1, 2] {
        let mut w = Windows::at_level(level);
        w.held(Side::Attacker, 2);
        assert_eq!(w.heroes, [Some(2), None]);
        // Filled again, the same seen (oddity 6).
        w.held(Side::Attacker, 2);
        assert_eq!(drawn(&w), [(Side::Attacker, 2)]);
    }
}

#[test]
fn the_held_press_over_a_window_flagged_at_level_0_shows_by_profile() {
    // CLT-Q50: the original's shows nothing while its window is flagged at level 0; the
    // client's shows it whatever.
    let mut w = Windows::at_level(0);
    w.heroes = [Some(0), None];
    assert!(!w.held_shows(Side::Attacker, Bugs::Original));
    assert!(w.held_shows(Side::Attacker, Bugs::Fixed));
    // The other side's, not flagged, shows in both; at level 1 the flagged one too.
    assert!(w.held_shows(Side::Defender, Bugs::Original));
    w.level = 1;
    assert!(w.held_shows(Side::Attacker, Bugs::Original));
}

#[test]
fn a_still_s_mouse_shows_hides_and_keeps_the_windows() {
    let (l, o) = (layout(), battle(true));
    let mut hover = Hover::new(1);
    assert_eq!(hover, Hover::default().with_level(1));
    let attacker = on_hero(&l, 0);
    hover.moved(&l, (&o, &Legal::default()), (attacker, true));
    assert_eq!(drawn(&hover.windows), [(Side::Attacker, 0)]);
    // A left press over the battlefield hides it, the hex kept: a move on the same rectangle
    // shows nothing (oddity 5); a press over the panel hides nothing.
    hover.pressed(&l, attacker);
    assert_eq!(drawn(&hover.windows), []);
    hover.moved(
        &l,
        (&o, &Legal::default()),
        ([attacker[0] + 1, attacker[1]], true),
    );
    assert_eq!(drawn(&hover.windows), []);
    // The hex forgotten (a dialog's close posts the move): back.
    hover.forget();
    hover.moved(&l, (&o, &Legal::default()), (attacker, true));
    assert_eq!(drawn(&hover.windows), [(Side::Attacker, 0)]);
    hover.pressed(&l, [618, 593]);
    assert_eq!(drawn(&hover.windows), [(Side::Attacker, 0)]);
    // The keys C, F, S, R and O: the original's keeps them (CLT-Q49), the client's hides them.
    hover.key(&l, attacker, Bugs::Original);
    assert_eq!(drawn(&hover.windows), [(Side::Attacker, 0)]);
    hover.key(&l, attacker, Bugs::Fixed);
    assert_eq!(drawn(&hover.windows), []);
    // A new hex: the defender's; an empty hex: none; out of the battlefield: none.
    hover.moved(&l, (&o, &Legal::default()), (on_hero(&l, 1), true));
    assert_eq!(drawn(&hover.windows), [(Side::Defender, 0)]);
    hover.moved(&l, (&o, &Legal::default()), (centre(&l, 40), true));
    assert_eq!(drawn(&hover.windows), []);
    hover.moved(&l, (&o, &Legal::default()), (on_hero(&l, 1), true));
    hover.moved(&l, (&o, &Legal::default()), ([618, 593], true));
    assert_eq!(drawn(&hover.windows), []);
    // F5 cycles the level and changes no window until a new hex (§12.14 oddity 6).
    hover.moved(&l, (&o, &Legal::default()), (attacker, true));
    hover.cycle();
    assert_eq!(hover.windows.level, 2);
    assert_eq!(drawn(&hover.windows), [(Side::Attacker, 0)]);
    hover.cycle();
    assert_eq!(
        (hover.windows.level, drawn(&hover.windows)),
        (0, Vec::new())
    );
    assert_eq!(hover.windows.heroes, [Some(0), None]);
    hover.cycle();
    assert_eq!(hover.windows.level, 1);
    // At level 0 a new hex shows nothing.
    let mut hover = Hover::new(0);
    hover.moved(&l, (&o, &Legal::default()), (attacker, true));
    assert_eq!(hover.windows, Windows::default());
}

#[test]
fn a_still_s_mouse_shows_a_stack_s_window_and_writes_the_line_at_the_level() {
    // E17 M6 T4b2b2: the stacks' windows in a still, each side facing its own way; the line's
    // level that of the last new hex (the epic's review item 42), F5 after it rewriting nothing.
    let (l, o) = (layout(), battle(true));
    let mut hover = Hover::new(1);
    hover.moved(&l, (&o, &Legal::default()), (centre(&l, 100), true));
    assert_eq!(
        hover.windows.stack,
        Some(shown(Side::Defender, Kind::All, 2, 0))
    );
    assert_eq!(hover.line_level, 1);
    hover.cycle();
    hover.moved(&l, (&o, &Legal::default()), (centre(&l, 100), true));
    assert_eq!(hover.windows.stack.map(|s| s.kind), Some(Kind::All));
    assert_eq!(hover.line_level, 1);
    hover.moved(&l, (&o, &Legal::default()), (centre(&l, 99), true));
    assert_eq!(
        hover.windows.stack,
        Some(shown(Side::Defender, Kind::Spells, 2, 0))
    );
    assert_eq!(hover.line_level, 2);
    hover.cycle();
    hover.moved(&l, (&o, &Legal::default()), ([618, 593], true));
    assert_eq!((hover.windows, hover.line_level), (Windows::at_level(0), 0));
}

#[test]
fn a_still_s_line_is_written_again_at_an_attack_s_new_slot() {
    // E17 M6 T4b2b2 (0x475553, as `mouse::track`'s): over the hex the mouse is over already the
    // line is written again only as an attack's slot gives the cursor another frame, at the level
    // then; the windows unchanged. A shot's hex, or a stack only to look at, never.
    let (l, mut o) = (layout(), battle(true));
    o.active = Some(1);
    let strikes: Vec<Command> = (0..187)
        .map(|from| Command::Attack { target: 2, from })
        .collect();
    let legal = Legal::new(&strikes);
    let c = centre(&l, 100);
    // Left of the hex's centre one slot, above it another; a point beside the first.
    let (a, beside, b) = ([c[0] - 10, c[1]], [c[0] - 11, c[1]], [c[0], c[1] - 10]);
    let frame = |p: [i32; 2]| slots(&o, &legal, 100)[slot(&l, p, 100)];
    assert_eq!(frame(a), frame(beside));
    assert_ne!(frame(a), frame(b));
    let kind = |o: &Observation, legal: &Legal, p: [i32; 2]| {
        crate::mouse::pointer(o, legal, &l, (p, Bugs::Original, 0)).kind
    };
    assert_eq!(kind(&o, &legal, a), crate::mouse::Kind::Attack);
    let mut shooting = o.clone();
    shooting.stacks[0].shots = 5;
    let moved = |hover: &mut Hover, battle: (&Observation, &Legal), to: [i32; 2]| {
        hover.moved(&l, battle, (to, true));
    };
    // Level 1, F5 twice to 0: the same slot keeps the line's level, another slot writes it at 0.
    let mut hover = Hover::new(1);
    moved(&mut hover, (&o, &legal), a);
    let windows = hover.windows;
    hover.cycle();
    hover.cycle();
    moved(&mut hover, (&o, &legal), beside);
    assert_eq!(hover.line_level, 1);
    moved(&mut hover, (&o, &legal), b);
    assert_eq!(hover.line_level, 0);
    assert_eq!(
        hover.windows,
        Windows {
            level: 0,
            ..windows
        }
    );
    // The reverse: F5 to 1, back to the first slot: at 1.
    hover.cycle();
    moved(&mut hover, (&o, &legal), a);
    assert_eq!(hover.line_level, 1);
    // A shot's hex, a stack only to look at: the slots change nothing.
    let shot = Legal::new(&[Command::Shoot { target: 2, at: 100 }]);
    assert_eq!(kind(&shooting, &shot, a), crate::mouse::Kind::Shoot);
    for (o, legal) in [(&shooting, &shot), (&o, &Legal::default())] {
        let mut hover = Hover::new(1);
        moved(&mut hover, (o, legal), a);
        hover.cycle();
        moved(&mut hover, (o, legal), b);
        assert_eq!(hover.line_level, 1, "{legal:?}");
    }
}

#[test]
fn a_field_spot_without_a_hex_hides_the_windows_and_forgets_the_hex() {
    let (l, o) = (layout(), battle(true));
    let gap = [400, 2];
    assert!(on_field(&l, gap));
    assert_eq!(crate::mouse::spot(&l, gap, true), Spot::Nothing);
    let mut hover = Hover::new(1);
    hover.moved(&l, (&o, &Legal::default()), (on_hero(&l, 0), true));
    assert_eq!(drawn(&hover.windows), [(Side::Attacker, 0)]);
    // The hex −1 counts as a new hex (the hints' guard, *provisional*): every window hidden.
    hover.moved(&l, (&o, &Legal::default()), (gap, true));
    assert_eq!((hover.hex, drawn(&hover.windows)), (None, Vec::new()));
    assert_eq!(hover.windows.heroes, [None, None]);
}

#[test]
fn the_battlefield_ends_at_799_555() {
    let l = layout();
    assert!(on_field(&l, [0, 0]));
    assert!(on_field(&l, [799, 555]));
    assert!(!on_field(&l, [800, 10]));
    assert!(!on_field(&l, [10, 556]));
    assert!(!on_field(&l, [-1, 10]));
}

/// The stats dialog's numbers of a stack: attack 4 (6 now), defense 3 (3), damage 2 to 3, 25 hit
/// points, morale 5 and luck −4 (beyond the pictures' frames), 12 creatures, the spells `spells`.
fn stats(spells: &[u32]) -> Stats {
    Stats {
        count: 12,
        attack: Pair { own: 4, now: 6 },
        defense: Pair { own: 3, now: 3 },
        damage: [2, 3],
        health: Pair { own: 20, now: 25 },
        morale: 5,
        luck: -4,
        effects: spells
            .iter()
            .map(|&spell| Effect {
                spell,
                name: String::new(),
                rounds: None,
            })
            .collect(),
        ..Stats::default()
    }
}

/// A window's texts and frames by id.
fn texts(w: &crate::message::MessageBox) -> Vec<(u16, &str)> {
    w.shown
        .texts
        .iter()
        .map(|(&id, t)| (id, t.as_str()))
        .collect()
}

#[test]
fn a_stack_s_window_stands_at_its_side_s_and_kind_s_place() {
    // `combat_dialog_create`: kind 1 at (1, 267) and (721, 267), 78 x 288; kind 2 at (1, 429)
    // and (721, 429), 78 x 126; their items the constructor's (ids from 2200).
    let (l, c) = (layout(), catalogue());
    let info = StackInfo::of(&stats(&[]), (5, 2), [2, 3]);
    let places = [
        (Side::Attacker, Kind::All, [1, 267]),
        (Side::Defender, Kind::All, [721, 267]),
        (Side::Attacker, Kind::Spells, [1, 429]),
        (Side::Defender, Kind::Spells, [721, 429]),
    ];
    for (side, kind, place) in places {
        let w = stack_window(&l.army_info, &c, (side, kind), &info, Bugs::Fixed);
        assert_eq!(w.dialog.place, place, "{side:?} {kind:?}");
        assert_eq!(w.dialog.shadow, None);
        assert_eq!(w.shown.players.get(&STACK_BACKGROUND), Some(&2));
        let ids: Vec<u16> = w.dialog.items.iter().map(Item::id).collect();
        let size = match &w.dialog.items[0] {
            Item::Picture(p) => p.size,
            other => panic!("{other:?}"),
        };
        match kind {
            Kind::All => {
                assert_eq!(ids, (2200..=2218).collect::<Vec<u16>>());
                assert_eq!(size, Some([78, 288]));
            }
            Kind::Spells => {
                assert_eq!(ids, [2200, 2215, 2216, 2217, 2218]);
                assert_eq!(size, Some([78, 126]));
            }
        }
    }
}

#[test]
fn a_stack_s_window_shows_the_stats_dialog_s_numbers() {
    // `army_info_stack_set`: Att and Def "%d(%d)" even when equal (oddity 9), Dmg "%d-%d",
    // Health "%d", the count; morale's and luck's frames held to −3..=3, + 3; the portrait's
    // frame the creature + 2.
    let (l, c) = (layout(), catalogue());
    let info = StackInfo::of(&stats(&[]), (5, 2), [2, 3]);
    let w = stack_window(
        &l.army_info,
        &c,
        (Side::Attacker, Kind::All),
        &info,
        Bugs::Fixed,
    );
    let rows = [
        (STACK_ROWS, "A:"),
        (STACK_ROWS + 1, "4(6)"),
        (STACK_ROWS + 2, "D:"),
        (STACK_ROWS + 3, "3(3)"),
        (STACK_ROWS + 4, "G:"),
        (STACK_ROWS + 5, "2-3"),
        (STACK_ROWS + 6, "H:"),
        (STACK_ROWS + 7, "25"),
        (STACK_MORALE_LABEL, "M:"),
        (STACK_LUCK_LABEL, "L:"),
        (STACK_COUNT, "12"),
        (STACK_NO_SPELLS, "none"),
    ];
    assert_eq!(texts(&w), rows);
    let frames = [
        (STACK_PORTRAIT, 7),
        (STACK_MORALE, 6),
        (STACK_LUCK, 0),
        (STACK_SPELLS, 0),
        (STACK_SPELLS + 1, 0),
        (STACK_SPELLS + 2, 0),
    ];
    assert_eq!(w.shown.frames, frames.into());
    // One damage: "%d".
    let mut info = info;
    info.damage = [7, 7];
    let w = stack_window(
        &l.army_info,
        &c,
        (Side::Attacker, Kind::All),
        &info,
        Bugs::Fixed,
    );
    assert_eq!(w.shown.texts[&(STACK_ROWS + 5)], "7");
}

#[test]
fn a_ballista_s_damage_is_the_dialog_s_in_the_client_s_profile() {
    // CLT-Q48 (the user's answer, 2026-11-04): the client's window gives the stats dialog's
    // range (each end times the hero's attack plus 1), the original's its own range.
    let (l, c) = (layout(), catalogue());
    let mut ballista = stats(&[]);
    ballista.damage = [10, 15];
    let info = StackInfo::of(&ballista, (146, 0), [2, 3]);
    let row = |bugs| {
        let w = stack_window(&l.army_info, &c, (Side::Attacker, Kind::All), &info, bugs);
        w.shown.texts[&(STACK_ROWS + 5)].clone()
    };
    assert_eq!(row(Bugs::Fixed), "10-15");
    assert_eq!(row(Bugs::Original), "2-3");
}

#[test]
fn a_stack_s_window_shows_the_last_three_spells() {
    // The list's last three from index max(size − 3, 0), each icon's frame the spell + 1, the
    // rest 0; the text empty with any. Kind 2: the spells alone.
    let (l, c) = (layout(), catalogue());
    let spells = |list: &[u32], kind| {
        let info = StackInfo::of(&stats(list), (5, 2), [2, 3]);
        let w = stack_window(
            &l.army_info,
            &c,
            (Side::Defender, kind),
            &info,
            Bugs::Original,
        );
        let icons: Vec<u32> = (0..3)
            .map(|k| w.shown.frames[&(STACK_SPELLS + k)])
            .collect();
        (icons, w)
    };
    let (icons, w) = spells(&[10, 20, 30, 40, 50], Kind::All);
    assert_eq!(icons, [31, 41, 51]);
    assert_eq!(w.shown.texts[&STACK_NO_SPELLS], "");
    let (icons, w) = spells(&[10, 20], Kind::Spells);
    assert_eq!(icons, [11, 21, 0]);
    assert_eq!(texts(&w), [(STACK_NO_SPELLS, "")]);
    assert_eq!(w.shown.frames.len(), 3);
    let (icons, w) = spells(&[], Kind::Spells);
    assert_eq!(icons, [0, 0, 0]);
    assert_eq!(texts(&w), [(STACK_NO_SPELLS, "none")]);
}

#[test]
fn a_stack_s_kind_follows_the_level() {
    assert_eq!(Kind::at_level(0), None);
    assert_eq!(Kind::at_level(1), Some(Kind::All));
    assert_eq!(Kind::at_level(2), Some(Kind::Spells));
}

impl Hover {
    /// The same hover at `level`.
    fn with_level(mut self, level: u8) -> Self {
        self.windows.level = level;
        self.line_level = level;
        self
    }
}
