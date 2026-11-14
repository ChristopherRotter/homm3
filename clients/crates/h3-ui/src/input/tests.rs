//! The dialogs' input's tests (E27 M2 T3a): on an invented dialog; the combat options' own tests
//! (`h3_view::options`) check the same rules through that dialog.

use alloc::string::String;

use h3_render::Bugs;

use super::{Did, Held, Input};
use crate::ui::{Dialog, Shown, State};

/// OK, a button, a box, a hidden bar and an area (none of the kinds that take the mouse here).
const DIALOG: &str = r#"
place = [100, 50]
items = [
    { kind = "button", id = 1, def = "ok.def", at = [0, 0], size = [20, 20] },
    { kind = "button", id = 2, def = "b.def", at = [30, 0], size = [20, 20] },
    { kind = "def", id = 3, at = [0, 30], size = [20, 20] },
    { kind = "def", id = 4, at = [30, 30], size = [20, 20] },
    { kind = "area", id = 5, at = [60, 0], size = [20, 20] },
]
"#;

const OK: u16 = 1;
const BUTTON: u16 = 2;
const BOX: u16 = 3;
const BAR: u16 = 4;
const AREA: u16 = 5;
/// Off every item.
const OFF: [i32; 2] = [0, 0];

fn dialog() -> Dialog {
    toml::from_str(DIALOG).unwrap()
}

/// The bar hidden.
fn shown() -> Shown {
    let mut shown = Shown::default();
    shown.states.insert(
        BAR,
        State {
            hidden: true,
            ..State::default()
        },
    );
    shown
}

/// Item `id`'s middle on the screen.
fn centre(dialog: &Dialog, id: u16) -> [i32; 2] {
    let ([x, y], [w, h]) = dialog.item(id).unwrap().area().unwrap();
    [dialog.place[0] + x + w / 2, dialog.place[1] + y + h / 2]
}

const CHANGED: Did = Did {
    changed: true,
    button_sound: false,
    pressed: None,
    clicked: None,
};
const DOWN: Did = Did {
    button_sound: true,
    ..CHANGED
};

fn clicked(id: u16) -> Did {
    Did {
        clicked: Some(id),
        ..CHANGED
    }
}

fn picture_click(id: u16) -> Did {
    Did {
        clicked: Some(id),
        ..Did::default()
    }
}

#[test]
fn a_button_follows_the_mouse_off_and_on_and_clicks_at_a_release_over_it() {
    let (d, mut s) = (dialog(), shown());
    let mut input = Input::new(Bugs::Fixed);
    assert_eq!(input.press(&d, &mut s, centre(&d, OK)), DOWN);
    assert!(s.state(OK).pressed && input.holds_button());
    assert_eq!(input.held(), Some(&Held::Button { id: OK, over: true }));
    // A second press, a key and another button while it is held: nothing.
    assert_eq!(input.press(&d, &mut s, centre(&d, BUTTON)), Did::default());
    assert_eq!(input.key(&mut s, BUTTON, "Return"), Did::default());
    assert_eq!(input.moved(&d, &mut s, OFF), CHANGED);
    assert!(!s.state(OK).pressed);
    assert_eq!(input.moved(&d, &mut s, OFF), Did::default());
    assert_eq!(input.moved(&d, &mut s, centre(&d, BUTTON)), Did::default());
    // Back over it: pressed again with its sample.
    assert_eq!(input.moved(&d, &mut s, centre(&d, OK)), DOWN);
    assert_eq!(input.moved(&d, &mut s, centre(&d, OK)), Did::default());
    assert_eq!(input.release(&mut s), clicked(OK));
    assert!(!s.state(OK).pressed && input.held().is_none());
    assert_eq!(input.release(&mut s), Did::default());
}

#[test]
fn a_button_released_off_it_goes_up_without_its_click() {
    let (d, mut s) = (dialog(), shown());
    let mut input = Input::new(Bugs::Original);
    input.press(&d, &mut s, centre(&d, BUTTON));
    input.moved(&d, &mut s, OFF);
    assert_eq!(input.release(&mut s), CHANGED);
    assert!(!s.state(BUTTON).pressed && input.held().is_none());
}

#[test]
fn a_disabled_button_an_area_and_nothing_take_no_press() {
    let (d, mut s) = (dialog(), shown());
    s.states.insert(
        BUTTON,
        State {
            disabled: true,
            ..State::default()
        },
    );
    let mut input = Input::new(Bugs::Fixed);
    for point in [centre(&d, BUTTON), centre(&d, AREA), OFF] {
        assert_eq!(input.press(&d, &mut s, point), Did::default());
        assert_eq!(input.held(), None);
    }
    assert!(!s.state(BUTTON).pressed);
}

#[test]
fn a_press_keeps_the_buttons_other_state_bits() {
    let (d, mut s) = (dialog(), shown());
    let selected = State {
        selected: true,
        ..State::default()
    };
    s.states.insert(OK, selected);
    let mut input = Input::new(Bugs::Fixed);
    input.press(&d, &mut s, centre(&d, OK));
    assert_eq!(
        s.state(OK),
        State {
            pressed: true,
            ..selected
        }
    );
    input.release(&mut s);
    assert_eq!(s.state(OK), selected);
}

#[test]
fn a_picture_is_held_unchanged_and_clicks_at_a_release_over_it() {
    let (d, mut s) = (dialog(), shown());
    for bugs in [Bugs::Fixed, Bugs::Original] {
        let mut input = Input::new(bugs);
        let did = input.press(&d, &mut s, centre(&d, BOX));
        assert_eq!(
            did,
            Did {
                pressed: Some(BOX),
                ..Did::default()
            }
        );
        assert_eq!(
            input.held(),
            Some(&Held::Picture {
                id: BOX,
                over: true
            })
        );
        assert!(!input.holds_button());
        // Off and on again: nothing drawn.
        assert_eq!(input.moved(&d, &mut s, OFF), Did::default());
        assert_eq!(input.moved(&d, &mut s, centre(&d, BOX)), Did::default());
        assert_eq!(input.release(&mut s), picture_click(BOX));
        assert_eq!(s, shown());
    }
}

#[test]
fn a_picture_released_off_it_clicks_in_the_originals_profile_alone() {
    let (d, mut s) = (dialog(), shown());
    // The hidden bar takes the press too.
    for (bugs, then) in [
        (Bugs::Fixed, Did::default()),
        (Bugs::Original, picture_click(BAR)),
    ] {
        let mut input = Input::new(bugs);
        input.press(&d, &mut s, centre(&d, BAR));
        input.moved(&d, &mut s, centre(&d, BOX));
        assert_eq!(
            input.held(),
            Some(&Held::Picture {
                id: BAR,
                over: false
            })
        );
        assert_eq!(input.release(&mut s), then);
        assert_eq!(input.held(), None);
    }
}

#[test]
fn the_right_release_ends_a_buttons_hold_alone() {
    let (d, mut s) = (dialog(), shown());
    let mut input = Input::new(Bugs::Fixed);
    assert_eq!(input.right_release(&mut s), Did::default());
    input.press(&d, &mut s, centre(&d, OK));
    assert_eq!(input.right_release(&mut s), clicked(OK));
    input.press(&d, &mut s, centre(&d, BOX));
    assert_eq!(input.right_release(&mut s), Did::default());
    assert_eq!(input.release(&mut s), picture_click(BOX));
    input.key(&mut s, OK, "Return");
    assert_eq!(input.right_release(&mut s), Did::default());
    assert!(s.state(OK).pressed);
}

#[test]
fn a_key_holds_its_button_until_that_keys_release() {
    let (d, mut s) = (dialog(), shown());
    let mut input = Input::new(Bugs::Fixed);
    assert_eq!(input.key_up(&mut s, "Return"), Did::default());
    assert_eq!(input.key(&mut s, OK, "Return"), DOWN);
    assert_eq!(
        input.held(),
        Some(&Held::Key {
            id: OK,
            by: String::from("Return")
        })
    );
    assert!(s.state(OK).pressed && !input.holds_button());
    // The mouse does nothing meanwhile, another key's release neither.
    assert_eq!(input.press(&d, &mut s, centre(&d, BUTTON)), Did::default());
    assert_eq!(input.moved(&d, &mut s, OFF), Did::default());
    assert_eq!(input.release(&mut s), Did::default());
    assert_eq!(input.key(&mut s, OK, "Escape"), Did::default());
    assert_eq!(input.key_up(&mut s, "Escape"), Did::default());
    assert!(s.state(OK).pressed);
    assert_eq!(input.key_up(&mut s, "Return"), clicked(OK));
    assert!(!s.state(OK).pressed && input.held().is_none());
}

#[test]
fn the_help_is_a_button_s_or_a_picture_s_unless_a_button_is_held() {
    let (d, mut s) = (dialog(), shown());
    let mut input = Input::new(Bugs::Fixed);
    assert_eq!(input.help_at(&d, centre(&d, OK)), Some(OK));
    assert_eq!(input.help_at(&d, centre(&d, BAR)), Some(BAR));
    assert_eq!(input.help_at(&d, centre(&d, AREA)), None);
    assert_eq!(input.help_at(&d, OFF), None);
    input.press(&d, &mut s, centre(&d, BOX));
    assert_eq!(input.help_at(&d, centre(&d, OK)), Some(OK));
    input.release(&mut s);
    input.press(&d, &mut s, centre(&d, BUTTON));
    assert_eq!(input.help_at(&d, centre(&d, OK)), None);
}

#[test]
fn letting_go_drops_a_key_s_or_a_picture_s_hold_never_a_button_s() {
    let (d, mut s) = (dialog(), shown());
    let mut input = Input::new(Bugs::Original);
    assert_eq!(input.let_go(&mut s, None), Did::default());
    // A key's: by that key alone, drawn up again.
    input.key(&mut s, OK, "Return");
    assert_eq!(input.let_go(&mut s, None), Did::default());
    assert_eq!(input.let_go(&mut s, Some("Escape")), Did::default());
    assert!(input.held().is_some());
    assert_eq!(input.let_go(&mut s, Some("Return")), CHANGED);
    assert!(!s.state(OK).pressed && input.held().is_none());
    // A picture's: by the mouse alone, with nothing drawn; its release then clicks nothing.
    input.press(&d, &mut s, centre(&d, BOX));
    assert_eq!(input.let_go(&mut s, Some("Return")), Did::default());
    assert!(input.held().is_some());
    assert_eq!(input.let_go(&mut s, None), Did::default());
    assert_eq!(input.held(), None);
    assert_eq!(input.release(&mut s), Did::default());
    // A button's: kept.
    input.press(&d, &mut s, centre(&d, OK));
    assert_eq!(input.let_go(&mut s, None), Did::default());
    assert!(input.holds_button() && s.state(OK).pressed);
}

#[test]
fn the_default_input_is_the_clients_profile() {
    assert_eq!(Input::default(), Input::new(Bugs::Fixed));
}
