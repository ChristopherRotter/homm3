use super::*;
use crate::ui::Label;
use alloc::string::String;
use alloc::vec;

/// `text` typed into `edit` by the US layout's keys ([`scan_of`]).
fn typed(edit: &mut Edit, text: &str) {
    for c in text.bytes() {
        assert!(edit.key(scan_of(c).unwrap()));
    }
}

fn focused(text: &str, limit: usize) -> Edit {
    let mut edit = Edit::new(text, limit);
    edit.set_focus(true);
    edit
}

#[test]
fn the_table_is_the_us_layout_by_scan_code() {
    let at = |code, shift| char::from(character(Scan { code, shift }));
    let row = |from: u8, shift| (from..from + 10).map(|c| at(c, shift)).collect::<String>();
    assert_eq!(row(0x02, false), "1234567890");
    assert_eq!(row(0x02, true), "!@#$%^&*()");
    assert_eq!(row(0x10, false), "qwertyuiop");
    assert_eq!(row(0x10, true), "QWERTYUIOP");
    assert_eq!(row(0x2C, false), "zxcvbnm,./");
    assert_eq!(row(0x2C, true), "ZXCVBNM<>?");
    let marks = [
        0x0C, 0x0D, 0x1A, 0x1B, 0x27, 0x28, 0x2B, 0x37, 0x39, 0x4A, 0x4E,
    ];
    let all = |shift| marks.iter().map(|&c| at(c, shift)).collect::<String>();
    assert_eq!(all(false), "-=[];'\\* -+");
    assert_eq!(all(true), "_+[]:\"|* _+");
    // The keys without a character: Ctrl, the backtick, the Shifts, Alt, Caps Lock, the keypad's
    // digits, the function keys, the codes past the table.
    for code in [
        0x1D, 0x29, 0x2A, 0x36, 0x38, 0x3A, 0x3B, 0x44, 0x47, 0x52, 0x57, 0x58, 0x59,
    ] {
        assert_eq!(character(Scan::plain(code)), 0, "{code:#x}");
    }
    assert_eq!(character(Scan::plain(BACKSPACE)), RUB_OUT);
}

#[test]
fn typing_puts_characters_in_at_the_caret_up_to_the_limit() {
    let mut edit = focused("", 6);
    typed(&mut edit, "Anna");
    assert_eq!((edit.text(), edit.caret()), ("Anna", 4));
    assert!(edit.key(Scan::plain(LEFT)));
    assert!(edit.key(Scan::plain(LEFT)));
    typed(&mut edit, "x");
    assert_eq!((edit.text(), edit.caret()), ("Anxna", 3));
    // The limit holds the terminator: five characters in six.
    typed(&mut edit, "y");
    assert_eq!((edit.text(), edit.caret()), ("Anxna", 3));
    // Taken all the same; no character for F1 or a key without one.
    assert!(edit.key(Scan::plain(0x3B)));
    let mut room = focused("a", 6);
    assert!(room.key(Scan::plain(0x3B)));
    assert!(room.key(Scan::plain(0x29)));
    assert_eq!(room.text(), "a");
}

#[test]
fn the_caret_keys_move_and_delete() {
    let mut edit = focused("Anna", 21);
    let key = |edit: &mut Edit, code| assert!(edit.key(Scan::plain(code)));
    key(&mut edit, HOME);
    assert_eq!(edit.caret(), 0);
    key(&mut edit, LEFT);
    assert_eq!(edit.caret(), 0);
    key(&mut edit, DELETE);
    assert_eq!((edit.text(), edit.caret()), ("nna", 0));
    key(&mut edit, RIGHT);
    assert_eq!(edit.caret(), 1);
    key(&mut edit, END);
    key(&mut edit, RIGHT);
    key(&mut edit, DELETE);
    assert_eq!((edit.text(), edit.caret()), ("nna", 3));
    key(&mut edit, BACKSPACE);
    assert_eq!((edit.text(), edit.caret()), ("nn", 2));
    key(&mut edit, HOME);
    key(&mut edit, BACKSPACE);
    assert_eq!((edit.text(), edit.caret()), ("nn", 0));
}

#[test]
fn the_dialog_keeps_escape_tab_enter_and_every_key_without_the_focus() {
    let mut edit = focused("a", 21);
    for code in [ESCAPE, TAB, RETURN] {
        assert!(!edit.key(Scan::plain(code)));
    }
    edit.set_focus(false);
    assert!(!edit.focused());
    assert!(!edit.key(Scan::plain(0x1E)));
    assert_eq!(edit.text(), "a");
}

#[test]
fn an_edit_draws_as_its_label_with_the_caret_while_focused() {
    let item = Label {
        id: 7,
        at: [1, 2],
        size: [30, 18],
        font: String::from("smalfont.fnt"),
        font_palette: String::from("game.pal"),
        colour: 4,
        align: 0,
        hint: None,
        repeat: None,
    };
    let label = |id| Draw::Label {
        item: Label { id, ..item.clone() },
        x: 3,
        y: 4,
        text: String::new(),
        disabled: false,
    };
    let mut draws = vec![label(7), label(8), Draw::Darken { rect: [0; 4] }];
    let mut edit = Edit::new("ab", 21);
    edit.place(7, &mut draws);
    let edit_draw = |caret| Draw::Edit {
        item: item.clone(),
        x: 3,
        y: 4,
        text: String::from("ab"),
        caret,
    };
    assert_eq!(draws[0], edit_draw(None));
    assert_eq!(draws[1], label(8));
    edit.set_focus(true);
    edit.place(7, &mut draws);
    assert_eq!(draws[0], edit_draw(None), "only a label becomes an edit");
    let mut draws = vec![label(7)];
    edit.place(7, &mut draws);
    assert_eq!(draws[0], edit_draw(Some(2)));
}

#[test]
fn each_character_is_typed_by_its_us_key() {
    let scan = |c: u8| scan_of(c).map(|s| (s.code, s.shift));
    assert_eq!(scan(b'a'), Some((0x1E, false)));
    assert_eq!(scan(b'A'), Some((0x1E, true)));
    assert_eq!(scan(b'!'), Some((0x02, true)));
    assert_eq!(scan(b'-'), Some((0x0C, false)));
    // Without Shift first: the keypad's `*` before Shift and 8.
    assert_eq!(scan(b'*'), Some((0x37, false)));
    assert_eq!(scan(b'~'), None);
    assert_eq!(scan(0), None);
    for c in (b' '..=b'}').filter(|c| !b"`{}~".contains(c)) {
        assert_eq!(scan_of(c).map(character), Some(c), "{}", char::from(c));
    }
}
