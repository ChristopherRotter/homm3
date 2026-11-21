//! The menus' tests (E27 M2 T6a, T6b1): their hover, presses, hotkeys, help popups and the main
//! menu's Quit's question, as `menu-main`, `menu-quit`, `menu-new`, `menu-load` and `menu-sound`
//! recorded them; the stubs' notices over them (T6b2).

use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::vec;
use alloc::vec::Vec;

use h3_assets::fnt::{Font, Glyph};
use h3_render::Bugs;
use h3_ui::ui::{Dialog, Draw};

use super::{
    BOX_PLAYER, BUTTONS, Boxes, Did, Front, GAME_TYPE_BUTTONS, Hovers, Kind, Menu, Menus, QUIT,
    STUBS, Stale, TITLE, outside, stub_name,
};
use crate::message::{self, Key, Style};
use crate::shell::Kind as ScenarioKind;
use crate::shell::keys::{Command, Context};
use crate::shell::look::{ShellLayout, parse_layout};
use crate::shell::{Choice, Ended, Game, Request, Screen};

const LAYOUT: &str = include_str!("../../../../../layouts/shell.toml");

/// A font of `advance`-wide glyphs `height` high.
pub(crate) fn font(advance: i32, height: u8) -> Font {
    Font {
        height,
        y_offset: 0,
        glyphs: vec![
            Glyph {
                left: 0,
                width: advance,
                right: 0,
                pixels: Vec::new(),
            };
            256
        ],
    }
}

/// A menu's boxes laid out in fonts of 8-wide glyphs: each of `buttons`' help, and Quit's
/// question when `question`.
fn boxes(layout: &ShellLayout, bugs: Bugs, buttons: [u16; 5], question: bool) -> Boxes {
    let (medium, small) = (font(8, 16), font(5, 10));
    let laid = |text: &str, style| {
        message::layout(
            &layout.message,
            (&medium, &small),
            text,
            style,
            None,
            (BOX_PLAYER, bugs),
        )
    };
    let help: BTreeMap<u16, _> = buttons
        .iter()
        .map(|&id| (id, laid("{Help}\n\nThe help.", Style::Popup)))
        .collect();
    Boxes {
        question: question.then(|| laid("Quit?", Style::OkCancel)),
        help,
    }
}

/// The main menu as the shell's layout has it, its boxes laid out in fonts of 8-wide glyphs.
pub(crate) fn main_menu(bugs: Bugs) -> Menu {
    let layout = parse_layout(LAYOUT).unwrap();
    let menu = &layout.main_menu;
    Menu::new(
        Kind::Main,
        menu.dialog.clone(),
        menu.redrawn.clone(),
        boxes(&layout, bugs, BUTTONS, true),
        [400, 300],
        bugs,
    )
}

/// New Game's or Load Game's menu as the shell's layout has it.
pub(crate) fn game_type(game: Game, bugs: Bugs) -> Menu {
    let layout = parse_layout(LAYOUT).unwrap();
    let menu = &layout.game_type;
    Menu::new(
        Kind::GameType(game),
        menu.dialog_for(game),
        menu.redrawn.clone(),
        boxes(&layout, bugs, GAME_TYPE_BUTTONS, false),
        [400, 300],
        bugs,
    )
}

/// The three menus as they open, and the four stubs' notices (E27 M2 T6b2).
pub(crate) fn menus(bugs: Bugs) -> Menus {
    let layout = parse_layout(LAYOUT).unwrap();
    let (medium, small) = (font(8, 16), font(5, 10));
    let notices = STUBS
        .iter()
        .map(|&(screen, _, _)| {
            let notice = message::layout(
                &layout.message,
                (&medium, &small),
                "Not yet.",
                Style::Ok,
                None,
                (BOX_PLAYER, bugs),
            );
            (screen, notice)
        })
        .collect();
    Menus {
        main: main_menu(bugs),
        new_game: game_type(Game::New, bugs),
        load_game: game_type(Game::Load, bugs),
        notices,
        multiplayer: crate::shell::multiplayer::tests::multiplayer(bugs),
        scenario: crate::shell::scenario::tests::scenario(bugs),
        load: crate::shell::scenario::tests::load(bugs),
        scores: crate::shell::scores::tests::scores(bugs),
        credits: crate::shell::credits::tests::credits(),
    }
}

/// The main menu's DEFs in their order.
const MAIN_DEFS: [&str; 5] = [
    "mmenung.def",
    "mmenulg.def",
    "mmenuhs.def",
    "mmenucr.def",
    "mmenuqt.def",
];
/// A game type menu's DEFs by id (Single, Campaign, Multiplayer, Tutorial, Back).
const GAME_TYPE_DEFS: [&str; 5] = [
    "gtsingl.def",
    "gtcampn.def",
    "gtmulti.def",
    "gttutor.def",
    "gtback.def",
];

/// The frames the buttons `defs` are drawn in (whatever the order drawn).
fn frames_of(menu: &Menu, defs: [&str; 5]) -> Vec<u32> {
    let draws = menu.draws("PLAYERS.PAL");
    defs.iter()
        .map(|name| {
            draws
                .iter()
                .find_map(|d| match d {
                    Draw::Button { def, frame, .. } if def == name => Some(*frame),
                    _ => None,
                })
                .unwrap()
        })
        .collect()
}

/// The main menu's five buttons' frames, in their order.
fn frames(menu: &Menu) -> Vec<u32> {
    frames_of(menu, MAIN_DEFS)
}

const CHANGED: Did = Did {
    changed: true,
    sample: false,
    ended: None,
};
const SAMPLE: Did = Did {
    changed: true,
    sample: true,
    ended: None,
};
const NOTHING: Did = Did {
    changed: false,
    sample: false,
    ended: None,
};

fn ended(request: Request) -> Did {
    Did {
        changed: false,
        sample: false,
        ended: Some(Ended::MainMenu(request)),
    }
}

/// The middle of the button `id` in the box over the menu (the question or a notice).
fn box_button(menu: &Menu, id: u16) -> [i32; 2] {
    let open = match &menu.over {
        super::Over::Question(open) | super::Over::Notice(open) => open,
        _ => panic!("no box"),
    };
    let b = open
        .message
        .dialog
        .items
        .iter()
        .find_map(|i| match i {
            h3_ui::ui::Item::Button(b) if b.id == id => Some(b),
            _ => None,
        })
        .unwrap();
    let [x, y] = open.message.dialog.place;
    [x + b.at[0] + b.size[0] / 2, y + b.at[1] + b.size[1] / 2]
}

/// The OK's and Cancel's middles in the question's box.
fn box_buttons(menu: &Menu) -> ([i32; 2], [i32; 2]) {
    (
        box_button(menu, message::YES),
        box_button(menu, message::NO),
    )
}

#[test]
fn the_hover_draws_the_button_under_the_mouse_in_its_frame_3() {
    let mut menu = main_menu(Bugs::Original);
    assert_eq!(frames(&menu), [0, 0, 0, 0, 0]);
    assert_eq!(menu.moved([643, 70]), CHANGED);
    assert_eq!(menu.hovered(), Some(101));
    assert_eq!(frames(&menu), [3, 0, 0, 0, 0]);
    // The same button again: nothing changes.
    assert_eq!(menu.moved([650, 80]), NOTHING);
    assert_eq!(menu.moved([643, 520]), CHANGED);
    assert_eq!(frames(&menu), [0, 0, 0, 0, 3]);
    assert_eq!(menu.moved([300, 560]), CHANGED);
    assert_eq!((menu.hovered(), frames(&menu)), (None, vec![0; 5]));
    assert_eq!(menu.context(), Context::MainMenu);
    assert!(menu.cursor() && !menu.asking());
}

#[test]
fn a_press_draws_the_button_pressed_with_its_sample_and_the_release_over_it_ends_the_menu() {
    for (point, request) in [
        ([643, 70], Request::NewGame),
        ([645, 192], Request::LoadGame),
        ([643, 304], Request::HighScores),
        ([643, 414], Request::Credits),
    ] {
        let mut menu = main_menu(Bugs::Original);
        let _ = menu.moved(point);
        assert_eq!(menu.press(point), SAMPLE);
        assert!(frames(&menu).contains(&1));
        let did = menu.release();
        assert_eq!(did.ended, Some(Ended::MainMenu(request)), "{point:?}");
        assert!(did.changed);
    }
}

#[test]
fn a_release_off_the_button_gives_no_command_and_the_hover_is_taken_again() {
    let mut menu = main_menu(Bugs::Original);
    let _ = menu.moved([643, 70]);
    let _ = menu.press([643, 70]);
    // Held, the button follows the mouse off (drawn up) and on (pressed again, its sample again);
    // the hover does not follow.
    assert_eq!(menu.moved([300, 560]), CHANGED);
    assert_eq!(menu.hovered(), Some(101));
    assert_eq!(frames(&menu)[0], 3);
    assert_eq!(menu.moved([643, 70]), SAMPLE);
    assert_eq!(menu.moved([300, 560]), CHANGED);
    // Released off it: no command, the hover taken at the pointer (`menu-main`'s `new-left`).
    assert_eq!(menu.release(), CHANGED);
    assert_eq!((menu.hovered(), frames(&menu)), (None, vec![0; 5]));
    // A release with nothing held changes nothing.
    assert_eq!(menu.release(), NOTHING);
    assert_eq!(menu.right_release(), NOTHING);
    // The right button's release ends a hold as the left's does (CLT-Q72 (a)).
    let _ = menu.press([643, 414]);
    assert_eq!(
        menu.right_release().ended,
        Some(Ended::MainMenu(Request::Credits))
    );
}

#[test]
fn a_right_press_shows_the_button_s_help_while_held() {
    let mut menu = main_menu(Bugs::Original);
    let _ = menu.moved([643, 70]);
    let under = menu.draws("PLAYERS.PAL").len();
    assert_eq!(menu.right([643, 70]), CHANGED);
    // The popup over the hovered button, the cursor hidden; moves and keys dropped.
    assert!(menu.draws("PLAYERS.PAL").len() > under);
    assert_eq!(frames(&menu)[0], 3);
    assert!(!menu.cursor());
    assert_eq!(menu.context(), Context::MainMenu);
    assert_eq!(menu.moved([643, 520]), NOTHING);
    assert_eq!(menu.key(Command::Button(105), "Escape"), NOTHING);
    assert_eq!(menu.right([643, 520]), NOTHING);
    assert_eq!(menu.right_release(), CHANGED);
    assert_eq!(menu.draws("PLAYERS.PAL").len(), under);
    assert!(menu.cursor());
    // A left press or release closes it too.
    let _ = menu.right([643, 70]);
    assert_eq!(menu.press([643, 70]), CHANGED);
    let _ = menu.right([643, 70]);
    assert_eq!(menu.release(), CHANGED);
    assert_eq!(menu.draws("PLAYERS.PAL").len(), under);
    // Nothing under the mouse: no popup; a button the mouse holds: its loop drops the press.
    assert_eq!(menu.right([300, 560]), NOTHING);
    let _ = menu.press([643, 70]);
    assert_eq!(menu.right([643, 70]), NOTHING);
    // A button without a help box gives none.
    let mut bare = main_menu(Bugs::Original);
    bare.boxes.help.clear();
    assert_eq!(bare.right([643, 70]), NOTHING);
}

#[test]
fn the_hotkeys_press_their_buttons_and_click_at_the_key_up() {
    let mut menu = main_menu(Bugs::Original);
    assert_eq!(menu.key(Command::Button(101), "N"), SAMPLE);
    assert_eq!(frames(&menu)[0], 1);
    assert_eq!(menu.key_up("X"), NOTHING);
    assert_eq!(
        menu.key_up("N").ended,
        Some(Ended::MainMenu(Request::NewGame))
    );
    // Not a menu's button, or a box's key over the menu: nothing.
    let mut menu = main_menu(Bugs::Original);
    assert_eq!(menu.key(Command::Button(1), "Q"), NOTHING);
    assert_eq!(menu.key(Command::Message(Key::Ok), "Return"), NOTHING);
    // A key's button whose release the popup takes goes without its click.
    assert_eq!(menu.key(Command::Button(104), "C"), SAMPLE);
    let _ = menu.right([643, 70]);
    assert_eq!(menu.key_up("C"), CHANGED);
    assert_eq!(menu.key_up("C"), NOTHING);
    let _ = menu.right_release();
    assert_eq!(frames(&menu), [0; 5]);
}

#[test]
fn quit_asks_its_question_and_its_cancel_stays() {
    let mut menu = main_menu(Bugs::Original);
    let _ = menu.moved([643, 520]);
    let _ = menu.press([643, 520]);
    // `menu-quit`'s `question`: the box over the menu, Quit hovered under it.
    assert_eq!(menu.release(), CHANGED);
    assert!(menu.asking() && menu.cursor());
    assert_eq!(menu.context(), Context::Question);
    assert_eq!(frames(&menu)[4], 3);
    let (ok, cancel) = box_buttons(&menu);
    // No hover in the box; the menu's keys do nothing there.
    assert_eq!(menu.moved(ok), NOTHING);
    assert_eq!(menu.key(Command::Button(QUIT), "Escape"), NOTHING);
    assert_eq!(menu.right(cancel), NOTHING);
    assert_eq!(menu.right_release(), NOTHING);
    assert_eq!(menu.press([0, 0]), NOTHING);
    // Cancel pressed (its sample), released: the box gone, the hover taken at the pointer.
    assert_eq!(menu.press(cancel), SAMPLE);
    assert_eq!(menu.release(), CHANGED);
    assert!(!menu.asking());
    assert_eq!((menu.hovered(), frames(&menu)), (None, vec![0; 5]));
    // Escape asks again (Quit's hotkey), Escape cancels (the box's).
    assert_eq!(menu.key(Command::Button(QUIT), "Escape"), SAMPLE);
    assert_eq!(menu.key_up("Escape"), CHANGED);
    assert!(menu.asking());
    assert_eq!(menu.key(Command::Message(Key::Cancel), "Escape"), SAMPLE);
    assert_eq!(menu.key_up("Return"), NOTHING);
    assert_eq!(menu.key_up("Escape"), CHANGED);
    assert!(!menu.asking());
}

#[test]
fn the_question_s_ok_ends_the_menu_with_quit() {
    let ask = || {
        let mut menu = main_menu(Bugs::Fixed);
        let _ = menu.key(Command::Button(QUIT), "Escape");
        let _ = menu.key_up("Escape");
        menu
    };
    let mut menu = ask();
    let (ok, _) = box_buttons(&menu);
    let _ = menu.moved(ok);
    assert_eq!(menu.press(ok), SAMPLE);
    // Held, the OK follows the mouse off and on; released off it nothing closes.
    assert_eq!(menu.moved([0, 0]), CHANGED);
    assert_eq!(menu.release(), CHANGED);
    assert!(menu.asking());
    let _ = menu.press(ok);
    assert_eq!(menu.release(), ended(Request::Quit));
    // By Enter, and by the right button's release ending the OK's hold.
    let mut menu = ask();
    assert_eq!(menu.key(Command::Message(Key::Ok), "Return"), SAMPLE);
    assert_eq!(menu.key_up("Return"), ended(Request::Quit));
    let mut menu = ask();
    let _ = menu.press(ok);
    assert_eq!(menu.right_release(), ended(Request::Quit));
}

#[test]
fn the_menu_opens_again_as_it_first_opened() {
    let first = main_menu(Bugs::Original);
    let mut menu = first.clone();
    let _ = menu.moved([643, 70]);
    let _ = menu.right([643, 70]);
    let again = menu.opened([400, 300], Hovers::default());
    assert_eq!(again, first);
    assert_eq!(menu.opened([1, 2], Hovers::default()).pointer, [1, 2]);
}

/// The DEFs of the five buttons in the order they are drawn.
fn order(menu: &Menu) -> Vec<alloc::string::String> {
    menu.draws("PLAYERS.PAL")
        .iter()
        .filter_map(|d| match d {
            Draw::Button { def, .. } => Some(def.clone()),
            _ => None,
        })
        .take(5)
        .collect()
}

#[test]
fn a_button_redrawn_alone_shows_over_its_neighbours_in_the_original_s_profile() {
    // `load-press`: Load Game pressed is drawn alone, over High Scores' top rows, until the
    // hover's change draws the five in their order (CLT-Q126).
    let in_order = MAIN_DEFS;
    let load_last = [
        "mmenung.def",
        "mmenuhs.def",
        "mmenucr.def",
        "mmenuqt.def",
        "mmenulg.def",
    ];
    let mut original = main_menu(Bugs::Original);
    let mut fixed = main_menu(Bugs::Fixed);
    for menu in [&mut original, &mut fixed] {
        let _ = menu.moved([645, 192]);
        let _ = menu.press([645, 192]);
    }
    assert_eq!(order(&original), load_last);
    assert_eq!(order(&fixed), in_order);
    // Moved off (the button alone again), released off: still last; the hover's change after it
    // draws the five in order.
    let _ = original.moved([645, 400]);
    assert_eq!(order(&original), load_last);
    let _ = original.release();
    assert_eq!(order(&original), in_order);
    // A key's press and release, and a release the popup takes, draw the button alone.
    let _ = original.key(Command::Button(104), "C");
    assert_eq!(order(&original)[4], "mmenucr.def");
    let _ = original.right([643, 414]);
    let _ = original.key_up("C");
    assert_eq!(order(&original)[4], "mmenucr.def");
    let _ = original.right_release();
    let _ = original.moved([643, 70]);
    assert_eq!(order(&original), in_order);
    let _ = original.key(Command::Button(101), "N");
    let _ = original.key_up("N");
    assert_eq!(order(&original)[4], "mmenung.def");
    // The right button's release of a hold draws it alone too.
    let mut menu = main_menu(Bugs::Original);
    let _ = menu.press([643, 304]);
    let _ = menu.moved([300, 560]);
    let _ = menu.right_release();
    assert_eq!(order(&menu)[4], "mmenuhs.def");
}

#[test]
fn a_hold_names_its_item() {
    use h3_ui::input::Held;
    // The menu has no DEF pictures; a picture's hold names its item as a button's does.
    assert_eq!(super::held_id(&Held::Picture { id: 7, over: true }), 7);
    assert_eq!(
        super::held_id(&Held::Button {
            id: 101,
            over: false
        }),
        101
    );
}

#[test]
fn a_main_menu_without_its_question_quits_at_once() {
    // As a lobby's start does (0x4FB9F0: no question there).
    let mut menu = main_menu(Bugs::Fixed);
    menu.boxes.question = None;
    let _ = menu.key(Command::Button(QUIT), "Escape");
    assert_eq!(
        menu.key_up("Escape").ended,
        Some(Ended::MainMenu(Request::Quit))
    );
    assert!(!menu.asking());
}

/// A game type menu's five buttons' frames, by id.
fn game_frames(menu: &Menu) -> Vec<u32> {
    frames_of(menu, GAME_TYPE_DEFS)
}

/// The items in the order a game type menu draws them: its title, then its DEFs.
fn drawn(menu: &Menu) -> Vec<alloc::string::String> {
    menu.dialog_draws("PLAYERS.PAL")
        .iter()
        .map(|d| match d {
            Draw::Button { def, .. } => def.clone(),
            Draw::Picture { picture, .. } => picture.clone(),
            other => panic!("{other:?}"),
        })
        .collect()
}

#[test]
fn a_game_type_menu_draws_its_title_and_its_buttons_in_the_order_it_added_them() {
    let new = game_type(Game::New, Bugs::Fixed);
    assert_eq!(
        drawn(&new),
        [
            "newgame.pcx",
            "gtsingl.def",
            "gtcampn.def",
            "gttutor.def",
            "gtmulti.def",
            "gtback.def"
        ]
    );
    // Load Game's the same dialog, its title Load Game's.
    assert_eq!(
        drawn(&game_type(Game::Load, Bugs::Fixed))[0],
        "loadgame.pcx"
    );
    assert_eq!(new.kind(), Kind::GameType(Game::New));
    assert_eq!(new.context(), Context::GameType);
    assert_eq!(new.box_draws("PLAYERS.PAL"), []);
}

#[test]
fn a_game_type_menu_s_hover_takes_the_item_under_the_mouse_its_title_too() {
    // `menu-new`'s hovers: Single, Multiplayer, Campaign, Tutorial and Back in frame 3.
    let mut menu = game_type(Game::New, Bugs::Original);
    assert_eq!(game_frames(&menu), [0; 5]);
    for (point, id, at) in [
        ([649, 65], 100, 0),
        ([649, 180], 102, 2),
        ([646, 298], 101, 1),
        ([647, 412], 103, 3),
        ([645, 517], 104, 4),
    ] {
        assert_eq!(menu.moved(point), CHANGED);
        assert_eq!(menu.hovered(), Some(id));
        let mut want = vec![0; 5];
        want[at] = 3;
        assert_eq!(game_frames(&menu), want);
    }
    // The title is an item under the mouse: the buttons lose their hover.
    assert_eq!(menu.moved([200, 330]), CHANGED);
    assert_eq!(
        (menu.hovered(), game_frames(&menu)),
        (Some(TITLE), vec![0; 5])
    );
    assert_eq!(menu.moved([300, 560]), CHANGED);
    assert_eq!(menu.hovered(), None);
}

#[test]
fn a_game_type_menu_s_buttons_end_it_with_their_command() {
    for game in [Game::New, Game::Load] {
        for (point, choice) in [
            ([649, 65], Choice::Single),
            ([649, 180], Choice::Multiplayer),
            ([646, 298], Choice::Campaign),
            ([647, 412], Choice::Tutorial),
            ([645, 517], Choice::Back),
        ] {
            let mut menu = game_type(game, Bugs::Original);
            let _ = menu.moved(point);
            assert_eq!(menu.press(point), SAMPLE);
            let did = menu.release();
            assert_eq!(did.ended, Some(Ended::GameType(choice)), "{point:?}");
        }
    }
    // Released off: no command, the hover taken again (`single-left`).
    let mut menu = game_type(Game::New, Bugs::Original);
    let _ = menu.moved([649, 65]);
    let _ = menu.press([649, 65]);
    let _ = menu.moved([300, 560]);
    assert_eq!(menu.release(), CHANGED);
    assert_eq!((menu.hovered(), game_frames(&menu)), (None, vec![0; 5]));
}

#[test]
fn a_game_type_menu_s_hotkeys_press_and_click_its_buttons() {
    let mut menu = game_type(Game::Load, Bugs::Original);
    // `menu-load`'s Escape: Back.
    assert_eq!(menu.key(Command::Choice(Choice::Back), "Escape"), SAMPLE);
    assert_eq!(game_frames(&menu)[4], 1);
    assert_eq!(
        menu.key_up("Escape").ended,
        Some(Ended::GameType(Choice::Back))
    );
    let mut menu = game_type(Game::New, Bugs::Original);
    assert_eq!(menu.key(Command::Choice(Choice::Tutorial), "T"), SAMPLE);
    assert_eq!(
        menu.key_up("T").ended,
        Some(Ended::GameType(Choice::Tutorial))
    );
    // The main menu's commands do nothing here, nor a game type menu's on the main menu.
    let mut menu = game_type(Game::New, Bugs::Original);
    assert_eq!(menu.key(Command::Button(101), "C"), NOTHING);
    let mut main = main_menu(Bugs::Original);
    assert_eq!(main.key(Command::Choice(Choice::Campaign), "C"), NOTHING);
}

#[test]
fn a_game_type_menu_s_right_press_shows_its_button_s_help() {
    let mut menu = game_type(Game::New, Bugs::Original);
    assert_eq!(menu.right([649, 65]), CHANGED);
    assert!(!menu.cursor());
    assert_eq!(menu.context(), Context::GameType);
    assert!(!menu.box_draws("PLAYERS.PAL").is_empty());
    assert_eq!(menu.right_release(), CHANGED);
    // The title has no help.
    assert_eq!(menu.right([200, 330]), NOTHING);
}

#[test]
fn a_game_type_menu_s_button_without_a_command_ends_nothing() {
    // A layout's button whose id is no game type command.
    let layout = parse_layout(LAYOUT).unwrap();
    let mut dialog: Dialog = layout.game_type.dialog_for(Game::New);
    for item in &mut dialog.items {
        if let h3_ui::ui::Item::Button(b) = item
            && b.id == 100
        {
            b.id = 99;
        }
    }
    let mut menu = Menu::new(
        Kind::GameType(Game::New),
        dialog,
        Vec::new(),
        boxes(&layout, Bugs::Fixed, GAME_TYPE_BUTTONS, false),
        [0, 0],
        Bugs::Fixed,
    );
    let _ = menu.press([649, 65]);
    assert_eq!(menu.release().ended, None);
}

#[test]
fn the_rows_a_redraw_does_not_show_keep_the_frame_last_shown_whole() {
    // CLT-Q118 (`menu-new`): Single's rows 4 to 6 and Back's 567 to 569 lie outside the
    // redraw's rectangle; they keep the frame its own loop last showed whole.
    let single = |frame: u32| Draw::Button {
        def: "gtsingl.def".into(),
        frame,
        x: 545,
        y: 4,
        mirrored: false,
        player: None,
    };
    let mut menu = game_type(Game::New, Bugs::Original);
    assert_eq!(menu.stale("PLAYERS.PAL"), []);
    // The hover's redraw: Single in frame 3 inside the rectangle, its top rows still frame 0.
    let _ = menu.moved([649, 65]);
    let strip = |frame| Stale {
        rects: vec![[545, 4, 209, 3]],
        draws: vec![single(frame)],
    };
    assert_eq!(menu.stale("PLAYERS.PAL"), [strip(0)]);
    // The press shows its box whole: nothing stale.
    let _ = menu.press([649, 65]);
    assert_eq!(menu.stale("PLAYERS.PAL"), []);
    // Moved off (drawn up, hovered: frame 3, shown whole), released: the hover gone, the rows
    // keep frame 3 (`single-left`).
    let _ = menu.moved([300, 560]);
    let _ = menu.release();
    assert_eq!(menu.stale("PLAYERS.PAL"), [strip(3)]);
    // Back alike at its bottom rows.
    let _ = menu.moved([645, 517]);
    let _ = menu.press([645, 517]);
    let _ = menu.moved([300, 560]);
    let _ = menu.release();
    let stale = menu.stale("PLAYERS.PAL");
    assert_eq!(stale.len(), 2);
    assert_eq!(stale[1].rects, [[582, 567, 127, 4]]);
    // The client's profile draws every button whole; the main menu's buttons all lie inside.
    let mut fixed = game_type(Game::New, Bugs::Fixed);
    let _ = fixed.moved([649, 65]);
    assert_eq!(fixed.stale("PLAYERS.PAL"), []);
    let mut main = main_menu(Bugs::Original);
    let _ = main.moved([643, 70]);
    assert_eq!(main.stale("PLAYERS.PAL"), []);
}

#[test]
fn a_rectangle_s_parts_outside_others_are_its_rows_and_columns_around_them() {
    // Apart: whole; covered: nothing; a hole: above, below, left and right of it.
    assert_eq!(outside([0, 0, 10, 10], &[[20, 20, 5, 5]]), [[0, 0, 10, 10]]);
    assert!(outside([0, 0, 10, 10], &[[-1, -1, 20, 20]]).is_empty());
    assert_eq!(
        outside([0, 0, 10, 10], &[[2, 3, 4, 5]]),
        [[0, 0, 10, 3], [0, 8, 10, 2], [0, 3, 2, 5], [6, 3, 4, 5]]
    );
    // Two rectangles in turn; one touching an edge leaves no empty part.
    assert_eq!(
        outside([0, 0, 10, 10], &[[0, 2, 10, 10], [0, 0, 5, 2]]),
        [[5, 0, 5, 2]]
    );
    assert_eq!(outside([0, 0, 10, 10], &[]), [[0, 0, 10, 10]]);
}

#[test]
fn the_menus_open_by_their_screen() {
    let all = menus(Bugs::Original);
    for (screen, kind) in [
        (Screen::MainMenu, Kind::Main),
        (Screen::GameType(Game::New), Kind::GameType(Game::New)),
        (Screen::GameType(Game::Load), Kind::GameType(Game::Load)),
    ] {
        let menu = all.opened(screen, [1, 2], Hovers::default()).unwrap();
        assert_eq!((menu.kind(), menu.kind().screen()), (kind, screen));
        assert_eq!(menu.pointer, [1, 2]);
    }
    assert_eq!(all.opened(Screen::Lost, [0, 0], Hovers::default()), None);
    assert_eq!(Kind::Main.buttons(), BUTTONS);
    assert_eq!(Kind::GameType(Game::Load).buttons(), GAME_TYPE_BUTTONS);
}

#[test]
fn a_stub_s_screen_is_its_menu_opened_anew_under_its_notice() {
    // E27 M2 T6b2: Campaign and Tutorial over New Game's menu, High Scores and Credits over the
    // main menu; none hovered, the cursor shown, the box's keys taken.
    let all = menus(Bugs::Fixed);
    let layout = parse_layout(LAYOUT).unwrap();
    for (screen, kind, _) in STUBS {
        let menu = all.opened(screen, [5, 6], Hovers::default()).unwrap();
        assert_eq!((menu.kind(), menu.screen()), (kind, screen));
        assert_eq!((menu.pointer, menu.hovered()), ([5, 6], None));
        assert_eq!(menu.context(), Context::Question);
        assert!(menu.cursor() && !menu.asking());
        assert!(!menu.box_draws(&layout.player_palette).is_empty());
        let shown =
            crate::shell::look::look(&layout, screen, Some(&Front::Menu(Box::new(menu.clone()))))
                .unwrap();
        assert_eq!(shown.over, menu.box_draws(&layout.player_palette));
        // Not the menu's own screen.
        assert_eq!(
            crate::shell::look::look(
                &layout,
                kind.screen(),
                Some(&Front::Menu(Box::new(menu.clone())))
            ),
            None
        );
    }
    // A stub without its notice opens nothing (the frame's stand-in); neither does a screen no
    // longer a stub (the credits since E27 M4 T2).
    assert_eq!(all.opened(Screen::Credits, [0, 0], Hovers::default()), None);
    let without = Menus {
        notices: Vec::new(),
        ..all
    };
    assert_eq!(
        without.opened(Screen::Tutorial, [0, 0], Hovers::default()),
        None
    );
}

#[test]
fn a_stub_s_notice_takes_the_box_s_inputs_and_its_ok_ends_the_screen() {
    let all = menus(Bugs::Original);
    for (screen, _, end) in STUBS {
        let ended = Did {
            ended: Some(end),
            ..NOTHING
        };
        let mut menu = all.opened(screen, [400, 300], Hovers::default()).unwrap();
        let ok = box_button(&menu, message::OK);
        // The menu's own inputs do nothing under it: its buttons, its hotkeys, the help.
        assert_eq!(menu.moved([643, 304]), NOTHING);
        assert_eq!(menu.hovered(), None);
        assert_eq!(menu.key(Command::Button(103), "H"), NOTHING);
        assert_eq!(menu.key(Command::Choice(Choice::Back), "Escape"), NOTHING);
        assert_eq!(menu.right(ok), NOTHING);
        assert_eq!(menu.right_release(), NOTHING);
        assert_eq!(menu.press([0, 0]), NOTHING);
        // No Cancel: Escape does nothing.
        assert_eq!(menu.key(Command::Message(Key::Cancel), "Escape"), NOTHING);
        // The OK pressed (its sample), off and on, released off it: still open.
        assert_eq!(menu.press(ok), SAMPLE);
        assert_eq!(menu.moved([0, 0]), CHANGED);
        assert_eq!(menu.release(), CHANGED);
        assert_eq!(menu.release(), NOTHING);
        // Released over it: the screen ends.
        let _ = menu.press(ok);
        assert_eq!(menu.release(), ended, "{screen:?}");
        // By Enter, and by the right button's release ending the OK's hold.
        let mut menu = all.opened(screen, [400, 300], Hovers::default()).unwrap();
        assert_eq!(menu.key(Command::Message(Key::Ok), "Return"), SAMPLE);
        assert_eq!(menu.key_up("Return"), ended);
        let mut menu = all.opened(screen, [400, 300], Hovers::default()).unwrap();
        let _ = menu.press(ok);
        assert_eq!(menu.right_release(), ended);
    }
}

#[test]
fn the_front_is_a_menu_or_the_multiplayer_dialog_over_its_menu() {
    // E27 M2 T7.
    let mut all = menus(Bugs::Fixed);
    let mut front = all
        .front(Screen::GameType(Game::New), None, [400, 300])
        .unwrap();
    assert_eq!(front.context(), Context::GameType);
    assert!(front.moved([649, 180]).changed);
    assert_eq!(front.hovers().game_type, Some(102));
    // A key without a command does nothing to a menu, nor a character.
    assert_eq!(front.key(None, None, "Q"), Did::default());
    assert!(!front.typing());
    assert_eq!(front.contexts(), [Context::GameType]);
    assert_eq!(front.typed('q'), Did::default());
    assert!(front.press([649, 180]).changed);
    let did = front.release();
    assert_eq!(did.ended, Some(Ended::GameType(Choice::Multiplayer)));
    // The dialog opens over the menu as it ended; none without a menu before it.
    assert_eq!(
        all.front(Screen::Multiplayer(Game::New), None, [0, 0]),
        None
    );
    let mut dialog = all
        .front(Screen::Multiplayer(Game::New), Some(front), [649, 180])
        .unwrap();
    assert_eq!(dialog.context(), Context::Multiplayer);
    assert_eq!(dialog.hovers().game_type, Some(102));
    assert!(dialog.moved([578, 157]).changed);
    assert!(dialog.right([578, 157]).changed);
    assert!(dialog.right_release().changed);
    // Its name types (E27 M2 T7b: the client's profile, the window's text input).
    assert!(dialog.typing());
    assert_eq!(dialog.contexts(), [Context::Edit, Context::Multiplayer]);
    assert!(dialog.typed('a').changed);
    assert_eq!(dialog.key(None, None, "A"), Did::default());
    assert_eq!(dialog.key_up("A"), Did::default());
    assert!(dialog.press([578, 503]).changed);
    assert_eq!(dialog.release().ended, Some(Ended::Multiplayer(false)));
    // New Game's menu again: the pointer over Multiplayer drawn hovered at the first move
    // (CLT-Q128, fixed); in the original's profile its procedure's last item is still
    // Multiplayer, so that move is no change.
    let mut original = menus(Bugs::Original);
    let mut kept = original
        .front(
            Screen::GameType(Game::New),
            Some(dialog.clone()),
            [649, 180],
        )
        .unwrap();
    assert!(!kept.moved([649, 180]).changed);
    let mut menu = all
        .front(Screen::GameType(Game::New), Some(dialog), [649, 180])
        .unwrap();
    assert!(menu.moved([649, 180]).changed);
    assert!(menu.moved([643, 70]).changed);
    let Front::Menu(m) = &menu else {
        panic!("a menu")
    };
    assert_eq!(m.hovered(), Some(100));
    // Its right press and release through the front.
    assert!(menu.right([643, 70]).changed);
    assert!(menu.right_release().changed);
    assert_eq!(menu.key_up("S"), Did::default());
    // The main menu's static is its own.
    let main = all.front(Screen::MainMenu, Some(menu), [0, 0]).unwrap();
    assert_eq!(
        main.hovers(),
        Hovers {
            main: None,
            game_type: Some(100),
            arrow: super::Arrow::Default,
        }
    );
    let Front::Menu(m) = main else {
        panic!("a menu")
    };
    assert_eq!(m.hovered(), None);
}

#[test]
fn the_scenario_screen_is_a_stub_over_its_game_type_menu() {
    // E27 M2 T7, until M3: from Single, Campaign, Tutorial or the Multiplayer dialog; a new
    // single scenario's built since M3 T1a, the hot seat's since M3 T4a, a single load's since M4
    // T4b1, the hot seat's load since M4 T5b1.
    assert_eq!(
        stub_name(Screen::Scenario(Game::Load, ScenarioKind::Tutorial)),
        "scenario"
    );
    assert_eq!(stub_name(Screen::Tutorial), "tutorial");
    assert_eq!(
        super::scenario(Game::Load, ScenarioKind::Campaign),
        STUBS[2]
    );
    for built in [ScenarioKind::Single, ScenarioKind::Multiplayer] {
        assert!(
            !STUBS
                .iter()
                .any(|&(s, _, _)| s == Screen::Scenario(Game::Load, built))
        );
    }
    for built in [ScenarioKind::Single, ScenarioKind::Multiplayer] {
        assert!(
            !STUBS
                .iter()
                .any(|&(s, _, _)| s == Screen::Scenario(Game::New, built))
        );
    }
    let all = menus(Bugs::Fixed);
    for (screen, game) in [
        (
            Screen::Scenario(Game::Load, ScenarioKind::Tutorial),
            Game::Load,
        ),
        (
            Screen::Scenario(Game::Load, ScenarioKind::Campaign),
            Game::Load,
        ),
    ] {
        let mut menu = all.opened(screen, [0, 0], Hovers::default()).unwrap();
        assert_eq!((menu.kind(), menu.screen()), (Kind::GameType(game), screen));
        let ended = [
            menu.key(Command::Message(Key::Ok), "Return"),
            menu.key_up("Return"),
        ];
        assert_eq!(ended[1].ended, Some(Ended::Scenario(false)));
    }
}

#[test]
fn a_new_single_scenario_s_screen_takes_its_inputs_through_the_front() {
    // E27 M3 T1a: opened from New Game's menu, the adventure map's arrow (CLT-Q137) and the
    // menus' statics carried.
    let mut all = menus(Bugs::Original);
    let screen = Screen::Scenario(Game::New, ScenarioKind::Single);
    let new_game = all.front(Screen::GameType(Game::New), None, [0, 0]);
    let mut front = all.front(screen, new_game, [790, 300]).unwrap();
    let Front::Scenario(_) = &front else {
        panic!("the scenario screen")
    };
    assert_eq!(front.hovers().arrow, super::Arrow::Adventure);
    assert_eq!(front.context(), Context::Scenario);
    assert_eq!(front.contexts(), vec![Context::Scenario]);
    assert!(!front.typing());
    assert_eq!(front.typed('a'), Did::default());
    assert_eq!(front.moved([1, 1]), Did::default());
    assert!(front.press([667, 555]).sample);
    assert_eq!(front.release().ended, Some(Ended::Scenario(false)));
    assert!(front.right([667, 555]).changed);
    assert!(front.right_release().changed);
    assert!(
        front
            .key(None, Some(Command::Dialog(188)), "Escape")
            .changed
    );
    assert_eq!(front.key_up("Escape").ended, Some(Ended::Scenario(false)));
    // The menu loop's top sets the default arrow again before the main menu.
    let back = all
        .front(Screen::GameType(Game::New), Some(front), [0, 0])
        .unwrap();
    assert_eq!(back.hovers().arrow, super::Arrow::Adventure);
    let main = all.front(Screen::MainMenu, Some(back), [0, 0]).unwrap();
    assert_eq!(main.hovers().arrow, super::Arrow::Default);
}
