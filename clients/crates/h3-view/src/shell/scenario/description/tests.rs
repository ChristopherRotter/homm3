//! The description's tests (E27 M3 T2c): its split, its bar and its keys, and what the screen
//! last drew, in a font of 5-wide glyphs 16 high (59 to a scrolling line, 63 to a fitting one).

use alloc::string::String;
use alloc::vec::Vec;

use h3_ui::ui::Draw;

use super::*;
use crate::shell::look::parse_layout;
use crate::shell::menu::tests::font;

const LAYOUT: &str = include_str!("../../../../../../layouts/shell.toml");

fn description() -> Description {
    let layout = parse_layout(LAYOUT).unwrap();
    Description::new(layout.scenario.description, Some(font(5, 16)))
}

/// `n` lines of their numbers.
fn numbered(n: usize) -> String {
    (0..n)
        .map(|k| alloc::format!("line {k}"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// The thumb's top, if the bar draws.
fn thumb(draws: &[Draw]) -> Option<i32> {
    draws.iter().find_map(|d| match d {
        Draw::Button { frame: 4, y, .. } => Some(*y),
        _ => None,
    })
}

#[test]
fn a_text_that_fits_shows_whole_without_its_bar() {
    let mut d = description();
    // 63 glyphs fit at the width less 3 (316), the split at less 24 (295) takes 59.
    let long_line = "x".repeat(60);
    d.set(&alloc::format!("{long_line}\nshort"));
    assert_eq!(d.text(), alloc::format!("{long_line}\nshort"));
    assert!(d.bar_draws([0, 0], false).is_empty());
    // Seven lines still fit; the description's own look.
    d.set(&numbered(7));
    assert_eq!(d.text(), numbered(7));
    assert!(d.bar_draws([0, 0], false).is_empty());
    assert_eq!(d.look().size, [319, 115]);
    assert_eq!(
        description_names(d.look()),
        ["smalfont.fnt", "SlideBuV.def", "sliderV.pcx"]
    );
}

#[test]
fn a_longer_text_scrolls_its_lines_by_the_bar() {
    let mut d = description();
    d.set(&numbered(18));
    // 18 lines, 7 shown: 12 ticks, the first 7 lines at value 0.
    assert_eq!(d.text(), numbered(7));
    let draws = d.bar_draws([0, 0], false);
    assert_eq!(thumb(&draws), Some(155 + 16));
    // The bottom arrow (733, 262) steps at the release, the lines drawn again.
    assert!(d.press([733, 262]));
    assert!(d.held());
    d.moved([733, 262]);
    d.release();
    assert!(!d.held());
    let from_one: Vec<String> = (1..8).map(|k| alloc::format!("line {k}")).collect();
    assert_eq!(d.text(), from_one.join("\n"));
    assert_eq!(d.drawn_text(), d.text());
    // A press beside the bar is not its.
    assert!(!d.press([700, 262]));
    // A text set again starts at its first line.
    d.set(&numbered(18));
    assert_eq!(d.text(), numbered(7));
}

#[test]
fn the_keys_step_the_bar_at_their_release() {
    let mut d = description();
    d.set(&numbered(18));
    d.key_down(Scroll::Down, "Down");
    // Another key's release does nothing; the key's own steps the bar.
    assert!(!d.key_up("Up", true));
    assert!(d.key_up("Down", true));
    assert!(d.text().starts_with("line 1\n"));
    // Taken once.
    assert!(!d.key_up("Down", true));
    d.key_down(Scroll::PageDown, "Next");
    assert!(d.key_up("Next", true));
    assert!(d.text().starts_with("line 8\n"));
    d.key_down(Scroll::Right, "Right");
    assert!(d.key_up("Right", true));
    d.key_down(Scroll::Left, "Left");
    assert!(d.key_up("Left", true));
    d.key_down(Scroll::PageUp, "Prior");
    assert!(d.key_up("Prior", true));
    assert!(d.text().starts_with("line 1\n"));
    d.key_down(Scroll::Up, "Up");
    assert!(d.key_up("Up", true));
    // At the top Up takes nothing; with something over the screen nothing either.
    d.key_down(Scroll::Up, "Up");
    assert!(!d.key_up("Up", true));
    d.key_down(Scroll::Down, "Down");
    assert!(!d.key_up("Down", false));
    assert!(d.text().starts_with("line 0\n"));
    // A text that fits takes no key.
    d.set("short");
    d.key_down(Scroll::Down, "Down");
    assert!(!d.key_up("Down", true));
}

#[test]
fn the_screen_keeps_what_it_last_drew() {
    let mut d = description();
    d.set(&numbered(18));
    d.drew();
    d.key_down(Scroll::PageDown, "Next");
    assert!(d.key_up("Next", true));
    let scrolled = d.text();
    // The select again (`scenario_to_game`) resets the text and the bar, undrawn.
    d.set(&numbered(18));
    assert_eq!(d.drawn_text(), scrolled);
    let stale = d.bar_draws([0, 0], true);
    let thumb_at_7 = 155 + 16 + 67 * 7 / 11;
    assert_eq!(thumb(&stale), Some(thumb_at_7));
    assert_eq!(thumb(&d.bar_draws([0, 0], false)), Some(155 + 16));
    // The top arrow at 0 draws the bar, not the lines (no change).
    assert!(d.press([733, 163]));
    d.release();
    assert_eq!(thumb(&d.bar_draws([0, 0], true)), Some(155 + 16));
    assert_eq!(d.drawn_text(), scrolled);
    // A whole redraw draws both; a text that fits draws no bar.
    d.set("short");
    d.drew();
    assert_eq!(d.drawn_text(), "short");
    assert!(d.bar_draws([0, 0], true).is_empty());
}

#[test]
fn a_line_left_highlighted_ends_its_highlight() {
    let mut d = description();
    // Each line is a text item of its own: a highlight runs to its line's end.
    d.set("{lit\nplain} text");
    assert_eq!(d.text(), "{lit}\nplain} text");
}

#[test]
fn without_its_font_the_description_is_one_label() {
    let layout = parse_layout(LAYOUT).unwrap();
    let mut d = Description::new(layout.scenario.description, None);
    d.set(&numbered(18));
    assert_eq!(d.text(), numbered(18));
    assert!(d.bar_draws([0, 0], false).is_empty());
    assert!(!d.press([733, 262]));
}
