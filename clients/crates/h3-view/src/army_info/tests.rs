//! The army info windows' tests (E17 M6 T4b2b1, design §12.15), on the committed layout.

use alloc::vec::Vec;

use h3_protocol::wire::{Hero, Observation, Side};
use h3_render::Bugs;

use super::{Hover, Windows, on_field};
use crate::layout::BattleLayout;
use crate::mouse::Spot;
use crate::record::tests::observation;
use crate::scene::tests::layout;

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

/// A battle with the attacker's hero, and the defender's when `defender`.
fn battle(defender: bool) -> Observation {
    let mut o = observation();
    o.sides[0].hero = Some(hero());
    o.sides[1].hero = defender.then(hero);
    o
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

fn drawn(windows: &Windows) -> Vec<(Side, usize)> {
    windows.drawn().collect()
}

#[test]
fn a_new_hex_shows_at_most_one_hero_s_window_at_levels_1_and_2() {
    let o = battle(true);
    for level in 0..3 {
        let mut w = Windows::at_level(level);
        w.enter(Spot::Hero(Side::Attacker), &o, 3);
        let shown = (level != 0).then_some(3);
        assert_eq!(w.heroes, [shown, None], "level {level}");
        // Another hex hides it first; the defender's shows alone.
        w.enter(Spot::Hero(Side::Defender), &o, 4);
        assert_eq!(w.heroes, [None, (level != 0).then_some(4)], "level {level}");
        // A stack's hex (T4b2b2's windows not built), the keep, nothing: none.
        for spot in [Spot::Hex(90), Spot::Keep, Spot::Tower, Spot::Nothing] {
            let mut w = Windows::at_level(level);
            w.heroes = [Some(1), None];
            w.enter(spot, &o, 5);
            assert_eq!(w.heroes, [None, None], "{spot:?} at level {level}");
        }
    }
    // Over a side without a hero: none.
    let mut w = Windows::at_level(1);
    w.enter(Spot::Hero(Side::Defender), &battle(false), 0);
    assert_eq!(w, Windows::at_level(1));
}

#[test]
fn shown_windows_are_drawn_only_while_the_level_is_not_0() {
    let mut w = Windows::at_level(2);
    w.heroes = [Some(1), Some(2)];
    assert_eq!(drawn(&w), [(Side::Attacker, 1), (Side::Defender, 2)]);
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
    hover.moved(&l, &o, (attacker, true));
    assert_eq!(drawn(&hover.windows), [(Side::Attacker, 0)]);
    // A left press over the battlefield hides it, the hex kept: a move on the same rectangle
    // shows nothing (oddity 5); a press over the panel hides nothing.
    hover.pressed(&l, attacker);
    assert_eq!(drawn(&hover.windows), []);
    hover.moved(&l, &o, ([attacker[0] + 1, attacker[1]], true));
    assert_eq!(drawn(&hover.windows), []);
    // The hex forgotten (a dialog's close posts the move): back.
    hover.forget();
    hover.moved(&l, &o, (attacker, true));
    assert_eq!(drawn(&hover.windows), [(Side::Attacker, 0)]);
    hover.pressed(&l, [618, 593]);
    assert_eq!(drawn(&hover.windows), [(Side::Attacker, 0)]);
    // The keys C, F, S, R and O: the original's keeps them (CLT-Q49), the client's hides them.
    hover.key(&l, attacker, Bugs::Original);
    assert_eq!(drawn(&hover.windows), [(Side::Attacker, 0)]);
    hover.key(&l, attacker, Bugs::Fixed);
    assert_eq!(drawn(&hover.windows), []);
    // A new hex: the defender's; an empty hex: none; out of the battlefield: none.
    hover.moved(&l, &o, (on_hero(&l, 1), true));
    assert_eq!(drawn(&hover.windows), [(Side::Defender, 0)]);
    hover.moved(&l, &o, (centre(&l, 40), true));
    assert_eq!(drawn(&hover.windows), []);
    hover.moved(&l, &o, (on_hero(&l, 1), true));
    hover.moved(&l, &o, ([618, 593], true));
    assert_eq!(drawn(&hover.windows), []);
    // F5 cycles the level and changes no window until a new hex (§12.14 oddity 6).
    hover.moved(&l, &o, (attacker, true));
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
    hover.moved(&l, &o, (attacker, true));
    assert_eq!(hover.windows, Windows::default());
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

impl Hover {
    /// The same hover at `level`.
    fn with_level(mut self, level: u8) -> Self {
        self.windows.level = level;
        self
    }
}
