//! The advanced options' duration's bar and hints (E27 M3 T3c2): the bar's arrows, track and drag
//! (`scenario_duration_scrolled` 0x57C7F0), the right press's help on the headers, the bar and
//! each colour's items, as `adv-single-rest`'s other 26 shots recorded them.

use super::super::super::{ADVANCED, Over};
use super::*;
use crate::shell::keys::Command;
use crate::shell::menu::Did;

/// The duration's text drawn and the bar's frames (left arrow, right arrow, thumb) by x.
fn duration_of(s: &Scenario) -> (Vec<String>, Vec<([i32; 2], u32)>) {
    let draws = s.draws("P");
    (texts_at(&draws, [256, 556]), frames(&draws, "SlideBuH.def"))
}

/// The bar's frames with the left arrow's `left`, the right one's `right` and the thumb at the
/// duration `value` (its travel 194 − 3 × 16 = 146, from 58 + 16).
fn bar_at(left: u32, right: u32, value: i32) -> Vec<([i32; 2], u32)> {
    vec![
        ([58, 557], left),
        ([58 + 194 - 16, 557], right),
        ([58 + 16 + 146 * value / 10, 557], 4),
    ]
}

#[test]
fn the_duration_s_arrows_step_it_at_the_release_the_screen_redrawn_whole() {
    let mut s = shown_in(vec![arrogance()], Bugs::Original);
    assert_eq!(s.duration(), Some(UNLIMITED));
    assert_eq!(
        duration_of(&s),
        (vec![String::from("duration 10")], bar_at(0, 2, 10))
    );
    // A flag's press leaves the strip stale (CLT-Q138's colon); a step of the duration redraws
    // the screen whole.
    click(&mut s, [35, 155]);
    assert!(s.stale_draws("P").is_some());
    // The left arrow (`adv-single-rest` duration-9 to duration-0): drawn pressed while held, the
    // step at the release.
    s.moved([66, 565]);
    assert_eq!(s.press([66, 565]), Did::CHANGED);
    assert_eq!(duration_of(&s).1, bar_at(1, 2, 10));
    assert_eq!(s.duration(), Some(10));
    assert_eq!(s.release(), Did::CHANGED);
    assert_eq!(
        duration_of(&s),
        (vec![String::from("duration 9")], bar_at(0, 2, 9))
    );
    assert!(s.stale_draws("P").is_none());
    for value in (0..9).rev() {
        click(&mut s, [66, 565]);
        assert_eq!(s.duration(), Some(value));
        assert_eq!(duration_of(&s).1, bar_at(0, 2, i32::from(value)));
    }
    // At 0 the left arrow steps nothing (duration-0-again); the right one steps on
    // (duration-1-right), drawn pressed while held.
    assert!(click(&mut s, [66, 565]));
    assert_eq!(duration_of(&s).0, [String::from("duration 0")]);
    s.moved([244, 565]);
    s.press([244, 565]);
    assert_eq!(duration_of(&s).1, bar_at(0, 3, 0));
    s.release();
    assert_eq!(
        duration_of(&s),
        (vec![String::from("duration 1")], bar_at(0, 2, 1))
    );
    // The options hidden and shown again keep it.
    click(&mut s, [514, 519]);
    click(&mut s, [514, 519]);
    assert_eq!(duration_of(&s).0, [String::from("duration 1")]);
}

#[test]
fn the_duration_s_track_drags_its_thumb_while_held() {
    let mut s = shown(vec![arrogance()]);
    // No page: a press in the track drags the thumb's middle there, the nearest tick
    // ((10 × (130 − 58 − 16 − 8) + 73) / 146 = 3).
    s.moved([130, 565]);
    assert_eq!(s.press([130, 565]), Did::CHANGED);
    assert_eq!(s.duration(), Some(3));
    assert_eq!(duration_of(&s).0, [String::from("duration 3")]);
    // Its own loop: a move drags it (within 40 pixels above or below the bar too), no hover, no
    // key, no right press; the right button's release lets it go as the left's.
    assert_eq!(s.moved([200, 540]), Did::CHANGED);
    assert_eq!(s.duration(), Some(8));
    assert_eq!(s.moved([90, 600]), Did::CHANGED);
    assert_eq!(s.duration(), Some(1));
    assert_eq!(s.moved([35, 155]), Did::CHANGED);
    assert!(!s.shown.state(FLAG).selected);
    assert_eq!(s.duration(), Some(1));
    assert_eq!(s.moved([150, 500]), Did::CHANGED);
    assert_eq!(s.duration(), Some(1));
    assert_eq!(
        s.key(None, Some(Command::Dialog(ADVANCED)), "A"),
        Did::default()
    );
    assert_eq!(s.right([110, 108]), Did::default());
    assert_eq!(s.right_release(), Did::CHANGED);
    assert_eq!(s.over, Over::Nothing);
    assert_eq!(s.duration(), Some(1));
    // Let go, a move drags nothing.
    s.moved([240, 565]);
    assert_eq!(s.duration(), Some(1));
}

#[test]
fn a_right_press_on_the_options_shows_each_item_s_help() {
    let mut s = shown(vec![arrogance()]);
    // Red's town chosen: its hero's arrows show (`adv-single-rest` popup-hero-arrow, on the single
    // player's row).
    click(&mut s, [230, 145]);
    for (point, id) in [
        ([110, 108], 339),
        ([200, 108], 342),
        ([276, 108], 343),
        ([352, 108], 344),
        ([225, 544], 340),
        ([155, 565], BAR),
        ([35, 155], FLAG),
        ([35, 205], FLAG + 1),
        ([169, 145], TOWN_LEFT),
        ([230, 145], TOWN_RIGHT),
        ([245, 145], HERO_LEFT),
        ([306, 145], HERO_RIGHT),
        ([321, 145], BONUS_LEFT),
        ([382, 195], BONUS_RIGHT + 1),
        ([85, 163], KIND),
        ([85, 313], KIND + 6),
    ] {
        assert_eq!(s.right(point), Did::CHANGED, "{point:?}");
        assert_eq!(s.over, Over::Help(id), "{point:?}");
        assert!(!s.box_draws("PLAYERS.PAL").is_empty());
        assert!(s.right_release().changed);
    }
    // A single player's handicap is not enabled, a name and a hidden arrow (a computer's hero's)
    // have no hint: no popup, the screen redrawn whole (popup-handicap, popup-name).
    for point in [[135, 163], [110, 138], [245, 195]] {
        assert_eq!(s.right(point), Did::CHANGED, "{point:?}");
        assert_eq!(s.over, Over::Nothing, "{point:?}");
    }
    // The bar's help only while the options show.
    click(&mut s, [514, 519]);
    assert_eq!(s.right([155, 565]), Did::CHANGED);
    assert_eq!(s.over, Over::Nothing);
}
