//! The shell's looks' tests (E27 M2 T4, T6a, T6b1): the layout file and the menus.

use alloc::boxed::Box;
use alloc::string::String;

use h3_render::Bugs;
use h3_ui::ui::Draw;

use super::{look, names, parse_layout};
use crate::shell::menu::tests::{game_type, main_menu};
use crate::shell::menu::{Arrow, Front};
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
            "cradvntr.def",
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
            "gamselb0.pcx",
            "gamselb1.pcx",
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
            "GSelPop1.pcx",
            "SCSelBck.pcx",
            "medfont.fnt",
            "AdvOptBk.pcx",
            "scnrmpsz.def",
            "itgflags.def",
            "scsmbut.def",
            "scmdbut.def",
            "sclgbut.def",
            "scxlbut.def",
            "scalbut.def",
            "scbutt1.def",
            "scbutt2.def",
            "ScButCp.def",
            "scbutt3.def",
            "scbutt4.def",
            "scbutt5.def",
            "gspbutt.def",
            "gspbut3.def",
            "gspbut4.def",
            "gspbut5.def",
            "gspbut6.def",
            "gspbut7.def",
            "scnrbeg.def",
            "scnrback.def",
            "scnrvict.def",
            "scnrloss.def",
            "ScSelC.def",
            "SlideBuV.def",
            "sliderV.pcx",
            "diboxbck.pcx",
            "dialgbox.def",
            "AOFLGBr.DEF",
            "adopb2r.def",
            "adoprpnl.pcx",
            "AOFLGBb.DEF",
            "adopb2b.def",
            "adopbpnl.pcx",
            "AOFLGBy.DEF",
            "adopb2y.def",
            "adopypnl.pcx",
            "AOFLGBg.DEF",
            "adopb2g.def",
            "adopgpnl.pcx",
            "AOFLGBo.DEF",
            "adopb2o.def",
            "adopopnl.pcx",
            "AOFLGBp.DEF",
            "adopb2p.def",
            "adopppnl.pcx",
            "AOFLGBt.DEF",
            "adopb2t.def",
            "adoptpnl.pcx",
            "AOFLGBs.DEF",
            "adopb2s.def",
            "adopspnl.pcx",
            "adoplfa.def",
            "adoprta.def",
            "itpa.def",
            "hpsrand0.pcx",
            "hpsrand1.pcx",
            "hpsrand6.pcx",
            "ScnrStar.def",
            "tiny.fnt",
            "SlideBuH.def",
            "slider.pcx",
            "cprsmall.def",
            "Box64x30.pcx",
            "iOkay.def",
            "iCancel.def",
            "PLAYERS.PAL",
        ]
    );
    // A name given twice in another case is kept once.
    let mut twice = layout;
    twice.cursor.def = String::from("MMENUNG.DEF");
    assert_eq!(names(&twice).len(), 101);
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

#[test]
fn the_scenario_screen_shows_its_drawn_background_and_its_arrow() {
    // E27 M3 T1a: the background by the draw (another index the first), the adventure map's
    // arrow after its busy cursor (CLT-Q137), the popup over it hiding the cursor.
    let layout = parse_layout(LAYOUT).unwrap();
    let screen = Screen::Scenario(Game::New, crate::shell::Kind::Single);
    let scenario = crate::shell::scenario::tests::scenario(Bugs::Original);
    let hovers = crate::shell::menu::Hovers {
        arrow: Arrow::Adventure,
        ..Default::default()
    };
    let front = |s: &crate::shell::scenario::Scenario| Front::Scenario(Box::new(s.clone()));
    let shown = look(&layout, screen, Some(&front(&scenario.opened(hovers)))).unwrap();
    assert_eq!(shown.background, "gamselb0.pcx");
    assert_eq!(shown.arrow, Arrow::Adventure);
    assert!(shown.cursor && shown.over.is_empty() && shown.stale.is_empty());
    // E27 M3 T1b1: the original's partial redraw leaves the flags' strip as a stale rectangle
    // (CLT-Q138): after a sort.
    let mut sorted = scenario.clone();
    for point in [[514, 91], [216, 106]] {
        sorted.moved(point);
        sorted.press(point);
        sorted.release();
    }
    let shown = look(&layout, screen, Some(&front(&sorted))).unwrap();
    assert_eq!(shown.stale.len(), 1);
    assert_eq!(shown.stale[0].rects, [crate::shell::scenario::STRIP]);
    let drawn = scenario.clone().with_background(1);
    let shown = look(&layout, screen, Some(&front(&drawn))).unwrap();
    assert_eq!(
        (shown.background.as_str(), shown.arrow),
        ("gamselb1.pcx", Arrow::Default)
    );
    let mut one = layout.clone();
    one.scenario.backgrounds.truncate(1);
    assert_eq!(
        look(&one, screen, Some(&front(&drawn))).unwrap().background,
        "gamselb0.pcx"
    );
    let mut none = layout.clone();
    none.scenario.backgrounds.clear();
    assert_eq!(look(&none, screen, Some(&front(&drawn))), None);
    assert_eq!(look(&layout, Screen::MainMenu, Some(&front(&drawn))), None);
    let mut held = scenario;
    held.right([514, 91]);
    let shown = look(&layout, screen, Some(&front(&held))).unwrap();
    assert!(!shown.cursor && !shown.over.is_empty());
}
