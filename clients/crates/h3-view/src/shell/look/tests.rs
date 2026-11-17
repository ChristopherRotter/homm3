//! The shell's looks' tests (E27 M2 T4, T6a, T6b1): the layout file and the menus.

use alloc::boxed::Box;
use alloc::string::String;

use h3_render::Bugs;
use h3_ui::ui::Draw;

use super::{look, names, parse_layout};
use crate::shell::menu::Front;
use crate::shell::menu::tests::{game_type, main_menu};
use crate::shell::{Game, Screen};

const LAYOUT: &str = include_str!("../../../../../layouts/shell.toml");

#[test]
fn the_main_menu_is_its_background_and_five_buttons_in_their_first_frame() {
    let layout = parse_layout(LAYOUT).unwrap();
    assert_eq!(
        (layout.cursor.def.as_str(), layout.cursor.frame),
        ("crdeflt.def", 0)
    );
    let mut menu = main_menu(Bugs::Original);
    let shown = look(
        &layout,
        Screen::MainMenu,
        Some(&Front::Menu(Box::new(menu.clone()))),
    )
    .unwrap();
    assert_eq!(shown.background, "GamSelBk.pcx");
    assert!(shown.cursor);
    let button = |def: &str, x, y| Draw::Button {
        def: def.into(),
        frame: 0,
        x,
        y,
        mirrored: false,
        player: None,
    };
    assert_eq!((shown.stale.len(), shown.over.len()), (0, 0));
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
    // The help popup over it, the cursor hidden (E27 M2 T6a).
    let _ = menu.right([643, 70]);
    let popup = look(
        &layout,
        Screen::MainMenu,
        Some(&Front::Menu(Box::new(menu.clone()))),
    )
    .unwrap();
    assert!(!popup.cursor);
    assert!(!popup.over.is_empty());
    assert_eq!(popup.draws.len(), shown.draws.len());
    // The screens not built yet show nothing, nor a menu without its state or with another's.
    assert_eq!(
        look(
            &layout,
            Screen::Start,
            Some(&Front::Menu(Box::new(menu.clone())))
        ),
        None
    );
    assert_eq!(look(&layout, Screen::GameType(Game::New), None), None);
    assert_eq!(
        look(
            &layout,
            Screen::GameType(Game::New),
            Some(&Front::Menu(Box::new(menu.clone())))
        ),
        None
    );
    assert_eq!(look(&layout, Screen::MainMenu, None), None);
}

#[test]
fn a_game_type_menu_is_its_background_its_title_and_its_buttons() {
    // E27 M2 T6b1: Load Game's title Load Game's; Single's top rows stale once hovered in the
    // original's profile (CLT-Q118).
    let layout = parse_layout(LAYOUT).unwrap();
    let mut menu = game_type(Game::Load, Bugs::Original);
    let shown = look(
        &layout,
        Screen::GameType(Game::Load),
        Some(&Front::Menu(Box::new(menu.clone()))),
    )
    .unwrap();
    assert_eq!(shown.background, "GamSelBk.pcx");
    assert_eq!(
        shown.draws[0],
        Draw::Picture {
            picture: "loadgame.pcx".into(),
            x: 114,
            y: 312,
            player: None,
            size: Some([300, 48]),
        }
    );
    assert_eq!(shown.draws.len(), 6);
    assert_eq!(
        look(
            &layout,
            Screen::GameType(Game::New),
            Some(&Front::Menu(Box::new(menu.clone())))
        ),
        None
    );
    let _ = menu.moved([649, 65]);
    let hovered = look(
        &layout,
        Screen::GameType(Game::Load),
        Some(&Front::Menu(Box::new(menu.clone()))),
    )
    .unwrap();
    assert_eq!(hovered.stale.len(), 1);
    // The main menu's layout has no titles: its dialog is its own for either game.
    assert_eq!(
        layout.main_menu.dialog_for(Game::Load),
        layout.main_menu.dialog
    );
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
            "newgame.pcx",
            "gtsingl.def",
            "gtcampn.def",
            "gttutor.def",
            "gtmulti.def",
            "gtback.def",
            "loadgame.pcx",
            "mupopup.pcx",
            "muBhot.def",
            "muBipx.def",
            "muBtcp.def",
            "muBmodm.def",
            "muBdrct.def",
            "mubonl.def",
            "muBcanc.def",
            "smalfont.fnt",
            "game.pal",
            "mumap.pcx",
            "muhotsea.pcx",
            "bigfont.fnt",
            "mubchck.def",
            "diboxbck.pcx",
            "dialgbox.def",
            "Box64x30.pcx",
            "iOkay.def",
            "iCancel.def",
            "medfont.fnt",
            "PLAYERS.PAL",
        ]
    );
    // A name given twice in another case is kept once.
    let mut twice = layout;
    twice.cursor.def = String::from("MMENUNG.DEF");
    assert_eq!(names(&twice).len(), 34);
}

#[test]
fn a_layout_not_the_shell_s_is_refused() {
    assert!(parse_layout("[cursor]\ndef = \"a.def\"\nframe = 0\n").is_err());
    assert!(parse_layout(&LAYOUT.replace("[main_menu]", "[main_menu]\nextra = 1")).is_err());
}

#[test]
fn the_multiplayer_dialog_shows_over_its_menu() {
    // E27 M2 T7: the menu as it ended, the dialog over it, a popup over them; since M2's review
    // the dialog over the menu's stale rows too (the original's profile: Single hovered).
    let layout = parse_layout(LAYOUT).unwrap();
    let players = &layout.player_palette;
    let mut menu = game_type(Game::New, Bugs::Original);
    let _ = menu.moved([649, 65]);
    let mut dialog = crate::shell::multiplayer::tests::multiplayer(Bugs::Original)
        .opened(menu.clone(), [400, 300]);
    let screen = Screen::Multiplayer(Game::New);
    let front = Front::Multiplayer(Box::new(dialog.clone()));
    let shown = look(&layout, screen, Some(&front)).unwrap();
    assert_eq!(shown.background, "GamSelBk.pcx");
    assert_eq!(shown.draws, menu.dialog_draws(players));
    assert_eq!(shown.stale.len(), 1);
    assert_eq!(shown.stale, menu.stale(players));
    assert_eq!(shown.over, dialog.draws(players));
    assert!(!shown.over.is_empty() && shown.cursor);
    dialog.right([578, 157]);
    let front = Front::Multiplayer(Box::new(dialog.clone()));
    let popup = look(&layout, screen, Some(&front)).unwrap();
    let mut over = dialog.draws(players);
    over.extend(dialog.box_draws(players));
    assert!(over.len() > shown.over.len());
    assert!(popup.over == over && !popup.cursor);
    // Not on another screen; none without a menu under it.
    assert_eq!(
        look(&layout, Screen::GameType(Game::New), Some(&front)),
        None
    );
    let bare = Front::Multiplayer(Box::new(crate::shell::multiplayer::tests::multiplayer(
        Bugs::Fixed,
    )));
    assert_eq!(look(&layout, screen, Some(&bare)), None);
}
