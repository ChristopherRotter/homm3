//! The scroll bar's tests (E27 M3 T1b1): the scenario list's bar as the original builds it (at
//! (375, 92), 16 × 480, `SlideBuV.def`'s 16-pixel buttons, a page of 18 rows) over the 159
//! shipped maps' 142 ticks.

use alloc::string::String;
use alloc::vec::Vec;

use super::*;

fn look() -> Look {
    Look {
        id: 337,
        at: [375, 92],
        size: [16, 480],
        def: String::from("SlideBuV.def"),
        track: String::from("sliderV.pcx"),
        button: 16,
        step: 18,
    }
}

/// The bar with 142 ticks at `value`.
fn at_value(value: i32) -> Scrollbar {
    let mut bar = Scrollbar::new(look());
    bar.reset(142);
    bar.set_value(value);
    let _ = bar.report();
    bar
}

/// The DEF frames drawn and the thumb's top.
fn frames(bar: &Scrollbar) -> (Vec<u32>, i32) {
    let draws = bar.draws([0, 0]);
    let mut frames = Vec::new();
    let mut thumb = 0;
    for draw in &draws {
        if let Draw::Button { frame, y, .. } = draw {
            frames.push(*frame);
            if *frame == 4 {
                thumb = *y;
            }
        }
    }
    (frames, thumb)
}

#[test]
fn the_value_places_the_thumb_on_its_travel() {
    let mut bar = Scrollbar::new(look());
    assert_eq!((bar.ticks(), bar.value(), bar.look().id), (1, 0, 337));
    assert_eq!(frames(&bar), (Vec::from([0, 2, 4]), 92 + 16));
    bar.reset(142);
    // The travel is 480 - 3 × 16 = 432: the kept map's top row 107 puts the thumb at
    // 432 × 107 / 141 + 16 = 343 (mid-bar, as `scn-list-keys` list shows it).
    bar.set_value(107);
    assert_eq!((bar.value(), frames(&bar).1), (107, 92 + 343));
    bar.set_value(141);
    assert_eq!(frames(&bar).1, 92 + 432 + 16);
    // Clamped to the ticks.
    bar.set_value(500);
    assert_eq!(bar.value(), 141);
    bar.set_value(-3);
    assert_eq!((bar.value(), frames(&bar).1), (0, 92 + 16));
    // A reset with no ticks keeps one; a single tick's thumb stays at the top.
    bar.reset(0);
    bar.set_value(5);
    assert_eq!((bar.ticks(), bar.value(), frames(&bar).1), (1, 0, 92 + 16));
}

#[test]
fn an_arrow_steps_at_the_release_drawn_pressed_meanwhile() {
    let mut bar = at_value(50);
    // The bottom arrow (its top at 92 + 480 - 16 = 556): pressed, nothing changes until the
    // release.
    assert_eq!(bar.press([383, 562]), None);
    assert!(bar.held());
    assert_eq!(frames(&bar).0, Vec::from([0, 3, 4]));
    assert_eq!(bar.release(), Some(51));
    assert!(!bar.held());
    assert_eq!(frames(&bar).0, Vec::from([0, 2, 4]));
    // The top arrow.
    assert_eq!(bar.press([383, 99]), None);
    assert_eq!(frames(&bar).0, Vec::from([1, 2, 4]));
    assert_eq!(bar.release(), Some(50));
    // At either end it stays.
    let mut top = at_value(0);
    let _ = top.press([383, 99]);
    assert_eq!(top.release(), None);
    let mut end = at_value(141);
    let _ = end.press([383, 570]);
    assert_eq!(end.release(), None);
    // The arrow is the one the mouse was last on, whatever its x (0x596B16 tests no x).
    let mut bar = at_value(10);
    let _ = bar.press([383, 99]);
    let _ = bar.moved([700, 100]);
    assert_eq!(bar.release(), Some(9));
    // A release with nothing held does nothing; so does a move.
    assert_eq!((bar.release(), bar.moved([383, 300])), (None, None));
}

#[test]
fn the_bottom_arrow_s_first_row_presses_but_neither_draws_pressed_nor_steps() {
    // Offset 464 (= 480 - 16) is the arrow's for the press (no page) but not for the draw or the
    // release, which want more than 464 (0x596A11 against 0x596ED0 and 0x596B85).
    let mut bar = at_value(20);
    assert_eq!(bar.press([383, 92 + 464]), None);
    assert_eq!(frames(&bar).0, Vec::from([0, 2, 4]));
    assert_eq!(bar.release(), None);
}

#[test]
fn the_track_pages_by_the_step_and_the_thumb_drags() {
    // The thumb at 92 + 432 × 50 / 141 + 16 = 261: the track above it pages up, below it down.
    let mut bar = at_value(50);
    assert_eq!(frames(&bar).1, 261);
    assert_eq!(bar.press([383, 120]), Some(32));
    assert_eq!(bar.release(), None);
    assert_eq!(bar.press([383, 540]), Some(50));
    let _ = bar.release();
    // Paged past either end: clamped.
    let mut low = at_value(5);
    assert_eq!(low.press([383, 110]), Some(0));
    let mut high = at_value(130);
    assert_eq!(high.press([383, 550]), Some(141));
    // On the thumb its middle goes to the mouse: (261 + 8 - 92 - 16 - 8) = 153 of 432, the tick
    // (141 × 153 + 216) / 432 = 50.
    let mut bar = at_value(50);
    assert_eq!(bar.press([383, 261 + 8]), None);
    assert_eq!(bar.press([383, 261 + 15]), Some(52));
    // Held, a move over the track drags it (from an arrow too); off the track it records only.
    let mut bar = at_value(50);
    let _ = bar.press([383, 99]);
    assert_eq!(bar.moved([383, 92 + 16 + 8]), Some(0));
    assert_eq!(bar.moved([383, 95]), None);
    assert_eq!(bar.moved([383, 600]), None);
    // Within 40 pixels either side its x is the bar's; further off it is not over the track.
    assert_eq!(bar.moved([360, 300]), Some(60));
    assert_eq!(bar.moved([420, 400]), Some(93));
    assert_eq!(bar.moved([300, 200]), None);
    assert_eq!(bar.moved([450, 200]), None);
}

#[test]
fn a_bar_without_a_page_drags_from_the_track() {
    let mut no_page = Scrollbar::new(Look { step: 0, ..look() });
    no_page.reset(11);
    assert_eq!(no_page.press([383, 92 + 16 + 8 + 432]), Some(10));
    // A single tick: the thumb stays, the value 0.
    let mut one = Scrollbar::new(look());
    assert_eq!(one.press([383, 300]), None);
    assert_eq!(frames(&one).1, 92 + 16);
    // A bar of no travel keeps its value.
    let mut flat = Scrollbar::new(Look {
        size: [16, 48],
        ..look()
    });
    flat.reset(5);
    flat.set_value(3);
    let _ = flat.report();
    assert_eq!(flat.press([383, 92 + 20]), None);
    assert!(flat.contains([375, 92]) && !flat.contains([391, 92]) && !flat.contains([375, 140]));
}

#[test]
fn a_value_the_caller_set_is_reported_at_the_next_press() {
    // As the original's +0x38: the caller's set keeps the value before, so a press on an arrow
    // reports the new one.
    let mut bar = at_value(10);
    bar.set_value(30);
    assert_eq!(bar.press([383, 99]), Some(30));
    assert_eq!(bar.release(), Some(29));
}

#[test]
fn it_draws_its_arrows_its_track_cut_to_it_and_its_thumb() {
    let bar = at_value(0);
    let draws = bar.draws([10, 20]);
    assert_eq!(draws.len(), 4);
    assert_eq!(
        draws[2],
        Draw::Picture {
            picture: String::from("sliderV.pcx"),
            x: 385,
            y: 128,
            player: None,
            size: Some([16, 448]),
        }
    );
    assert!(matches!(
        &draws[1],
        Draw::Button {
            frame: 2,
            x: 385,
            y: 576,
            ..
        }
    ));
    assert!(
        matches!(&draws[3], Draw::Button { frame: 4, y: 128, def, .. } if def == "SlideBuV.def")
    );
}

/// The scenario screen's description's bar (E27 M3 T2c): (725, 155), 16 × 115, a page of 7 lines,
/// with `ticks` ticks at `value`.
fn description(ticks: i32, value: i32) -> Scrollbar {
    let mut bar = Scrollbar::new(Look {
        id: 1021,
        at: [725, 155],
        size: [16, 115],
        step: 7,
        ..look()
    });
    bar.reset(ticks);
    bar.set_value(value);
    let _ = bar.report();
    bar
}

#[test]
fn the_keys_step_a_tick_or_a_page_within_the_ticks() {
    // `scn-long-text`: 12 ticks; Down and Right a tick on, Page Down a page (clamped), Up and
    // Left a tick back, Page Up a page back.
    let mut bar = description(12, 8);
    assert_eq!(bar.key(KeyStep::On), Some(9));
    assert_eq!(bar.key(KeyStep::PageOn), Some(11));
    assert!(bar.takes_key(KeyStep::PageOn));
    assert_eq!(bar.key(KeyStep::PageOn), None);
    assert!(!bar.takes_key(KeyStep::On));
    assert_eq!(bar.key(KeyStep::On), None);
    assert_eq!(bar.key(KeyStep::Back), Some(10));
    assert_eq!(bar.key(KeyStep::PageBack), Some(3));
    assert_eq!(bar.key(KeyStep::PageBack), Some(0));
    // At the top a tick back is not taken (the step not run), a page back is but changes nothing.
    assert!(!bar.takes_key(KeyStep::Back));
    assert_eq!(bar.key(KeyStep::Back), None);
    assert!(bar.takes_key(KeyStep::PageBack));
    assert_eq!(bar.key(KeyStep::PageBack), None);
    assert_eq!(frames(&bar), (alloc::vec![0, 2, 4], 155 + 16));
    // A value the caller set and not yet reported is reported first (0x5963A3).
    let mut bar = description(12, 0);
    bar.set_value(5);
    assert_eq!(bar.key(KeyStep::PageBack), Some(0));
    let mut bar = description(12, 0);
    bar.set_value(5);
    assert_eq!(bar.key(KeyStep::PageOn), Some(12 - 1));
    let mut bar = description(12, 3);
    bar.set_value(4);
    assert_eq!(bar.key(KeyStep::Back), Some(3));
}
