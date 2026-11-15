//! The main menu's tests (E27 M2 T6a): its hover, presses, hotkeys, help popups and Quit's
//! question, as `menu-main`, `menu-quit` and `menu-sound` recorded them.

use alloc::collections::BTreeMap;
use alloc::vec;
use alloc::vec::Vec;

use h3_assets::fnt::{Font, Glyph};
use h3_render::Bugs;
use h3_ui::ui::Draw;

use super::{BOX_PLAYER, BUTTONS, Boxes, Did, MainMenu, QUIT};
use crate::message::{self, Key, Style};
use crate::shell::Request;
use crate::shell::keys::{Command, Context};
use crate::shell::look::parse_layout;

const LAYOUT: &str = include_str!("../../../../../layouts/shell.toml");

/// A font of `advance`-wide glyphs `height` high.
fn font(advance: i32, height: u8) -> Font {
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

/// The main menu as the shell's layout has it, its boxes laid out in fonts of 8-wide glyphs.
pub(crate) fn main_menu(bugs: Bugs) -> MainMenu {
    let layout = parse_layout(LAYOUT).unwrap();
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
    let help: BTreeMap<u16, _> = BUTTONS
        .iter()
        .map(|&id| (id, laid("{Help}\n\nThe help.", Style::Popup)))
        .collect();
    let boxes = Boxes {
        question: laid("Quit?", Style::OkCancel),
        help,
    };
    MainMenu::new(layout.main_menu.dialog.clone(), boxes, [400, 300], bugs)
}

/// The frames the five buttons are drawn in, in their order (whatever the order drawn).
fn frames(menu: &MainMenu) -> Vec<u32> {
    let draws = menu.draws("PLAYERS.PAL");
    [
        "mmenung.def",
        "mmenulg.def",
        "mmenuhs.def",
        "mmenucr.def",
        "mmenuqt.def",
    ]
    .iter()
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
        ended: Some(request),
    }
}

/// The OK's and Cancel's middles in the question's box.
fn box_buttons(menu: &MainMenu) -> ([i32; 2], [i32; 2]) {
    let middle = |id| {
        let open = match &menu.over {
            super::Over::Question(open) => open,
            _ => panic!("no question"),
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
    };
    (middle(message::YES), middle(message::NO))
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
        assert_eq!(did.ended, Some(request), "{point:?}");
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
    assert_eq!(menu.right_release().ended, Some(Request::Credits));
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
    assert_eq!(menu.key_up("N").ended, Some(Request::NewGame));
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
    let again = menu.opened([400, 300]);
    assert_eq!(again, first);
    assert_eq!(menu.opened([1, 2]).pointer, [1, 2]);
}

/// The DEFs of the five buttons in the order they are drawn.
fn order(menu: &MainMenu) -> Vec<alloc::string::String> {
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
    let in_order = [
        "mmenung.def",
        "mmenulg.def",
        "mmenuhs.def",
        "mmenucr.def",
        "mmenuqt.def",
    ];
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
