//! The shell's looks' tests (E27 M2 T4): the layout file and the main menu at rest.

use alloc::string::String;

use h3_ui::ui::Draw;

use super::{look, names, parse_layout};
use crate::shell::{Game, Screen};

const LAYOUT: &str = include_str!("../../../../../layouts/shell.toml");

#[test]
fn the_main_menu_is_its_background_and_five_buttons_in_their_first_frame() {
    let layout = parse_layout(LAYOUT).unwrap();
    assert_eq!(
        (layout.cursor.def.as_str(), layout.cursor.frame),
        ("crdeflt.def", 0)
    );
    let shown = look(&layout, Screen::MainMenu).unwrap();
    assert_eq!(shown.background, "GamSelBk.pcx");
    let button = |def: &str, x, y| Draw::Button {
        def: def.into(),
        frame: 0,
        x,
        y,
        mirrored: false,
        player: None,
    };
    assert_eq!(
        shown.draws,
        [
            button("mmenung.def", 540, 10),
            button("mmenulg.def", 532, 132),
            button("mmenuhs.def", 524, 251),
            button("mmenucr.def", 557, 359),
            button("mmenuqt.def", 586, 469),
        ]
    );
    // The screens not built yet show nothing.
    assert_eq!(look(&layout, Screen::Start), None);
    assert_eq!(look(&layout, Screen::GameType(Game::New)), None);
}

#[test]
fn the_shell_s_assets_are_named_once() {
    let layout = parse_layout(LAYOUT).unwrap();
    let all = names(&layout);
    assert_eq!(
        all,
        [
            "crdeflt.def",
            "GamSelBk.pcx",
            "mmenung.def",
            "mmenulg.def",
            "mmenuhs.def",
            "mmenucr.def",
            "mmenuqt.def",
        ]
    );
    // A name given twice in another case is kept once.
    let mut twice = layout;
    twice.cursor.def = String::from("MMENUNG.DEF");
    assert_eq!(names(&twice).len(), 6);
}

#[test]
fn a_layout_not_the_shell_s_is_refused() {
    assert!(parse_layout("[cursor]\ndef = \"a.def\"\nframe = 0\n").is_err());
    assert!(parse_layout(&LAYOUT.replace("[main_menu]", "[main_menu]\nextra = 1")).is_err());
}
