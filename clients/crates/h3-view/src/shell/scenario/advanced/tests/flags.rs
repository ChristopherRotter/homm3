//! The advanced options' flags, handicap and name's edit (E27 M3 T3c1): the hover and press of
//! the flags, the name's edit by scan code (the original's profile) and by the key table and the
//! text input (the client's), as `adv-single-rest`'s first twelve shots recorded them.

use h3_ui::edit::{self, Caret, Scan};

use super::super::super::{ADVANCED, ALLIES, BACK, BEGIN, ENEMIES, LIST};
use super::*;
use crate::shell::keys::{Command, Context};
use crate::shell::menu::Did;

/// The edits drawn: corner, text, caret.
fn edits(draws: &[Draw]) -> Vec<([i32; 2], String, Option<u32>)> {
    draws
        .iter()
        .filter_map(|d| match d {
            Draw::Edit {
                x, y, text, caret, ..
            } => Some(([*x, *y], text.clone(), *caret)),
            _ => None,
        })
        .collect()
}

/// The edit drawn (the dialog's items and its row draw it alike), none.
fn edit_of(draws: &[Draw]) -> Option<([i32; 2], String, Option<u32>)> {
    let all = edits(draws);
    assert!(all.windows(2).all(|w| w[0] == w[1]));
    all.first().cloned()
}

/// A character's key as the original reads it (its US key's scan code).
fn scan(c: char) -> Option<Scan> {
    edit::scan_of(u8::try_from(c).unwrap())
}

/// `text` typed by scan code (the original's profile), each key down and up.
fn type_scans(s: &mut Scenario, text: &str) {
    for c in text.chars() {
        let name = String::from(c).to_uppercase();
        s.key(scan(c), None, &name);
        s.key_up(&name);
    }
}

#[test]
fn a_flag_s_hover_highlights_it_until_the_mouse_leaves() {
    let mut s = shown(vec![arrogance()]);
    // Over red's flag: its highlight (frame 3, `AOflgbRh.pcx`).
    assert!(s.moved([35, 155]).changed);
    assert!(s.shown.state(FLAG).selected);
    let red = frames(&s.draws("P"), "AOFLGBr.DEF");
    assert!(!red.is_empty() && red.iter().all(|&(at, f)| at == [14, 130] && f == 3));
    // On it again nothing changes; over blue's, red's cleared and blue's set; off them none.
    assert!(!s.moved([36, 156]).changed);
    s.moved([35, 205]);
    assert!(!s.shown.state(FLAG).selected && s.shown.state(FLAG + 1).selected);
    s.moved([790, 300]);
    assert!(!s.shown.state(FLAG + 1).selected);
    // A button the mouse holds runs its own loop: its moves highlight nothing.
    s.moved([230, 145]);
    s.press([230, 145]);
    s.moved([35, 155]);
    assert!(!s.shown.state(FLAG).selected);
    s.release();
    // Hidden by its key over a flag, the options keep its highlight; the next move clears it.
    s.moved([35, 155]);
    s.key(None, Some(Command::Dialog(ADVANCED)), "A");
    s.key_up("A");
    assert!(!s.advanced_showing());
    assert!(s.moved([790, 300]).changed);
    assert!(!s.shown.state(FLAG).selected);
}

#[test]
fn a_flag_s_press_moves_the_single_player_to_its_colour() {
    let mut s = shown_in(vec![arrogance()], Bugs::Original);
    // Its own flag moves nothing; the strip restored (CLT-Q138's colon) until a whole redraw.
    assert!(s.stale_draws("P").is_none());
    click(&mut s, [35, 155]);
    assert_eq!(s.records().unwrap()[0].colour, Some(0));
    assert!(s.stale_draws("P").is_some());
    // A town chosen, then blue's flag: the single player blue, its town and hero Random; red a
    // computer's; the allies blue, red an enemy.
    click(&mut s, [230, 145]);
    assert_eq!(s.records().unwrap()[0].town, Some(0));
    click(&mut s, [35, 205]);
    let records = s.records().unwrap();
    assert_eq!(records[1], Record::NEW.with_colour(1));
    assert_eq!(records[0], Record::NEW);
    let draws = s.draws("P");
    assert_eq!(texts_at(&draws, [62, 130]), ["Computer", "Computer"]);
    assert_eq!(texts_at(&draws, [62, 130 + dy(1)]), ["", ""]);
    assert_eq!(s.shown.frames.get(&ALLIES), Some(&1));
    assert_eq!(s.shown.frames.get(&ENEMIES), Some(&0));
    // A whole redraw (the options hidden) draws the strip's label again.
    click(&mut s, [514, 519]);
    assert!(s.stale_draws("P").is_none());
    // The client's profile draws whole.
    let mut f = shown(vec![arrogance()]);
    click(&mut f, [35, 205]);
    assert!(f.stale_draws("P").is_none());
    assert_eq!(f.records().unwrap()[1].colour, Some(1));
}

#[test]
fn a_flag_moves_only_to_a_colour_a_human_may_play() {
    let mut advanced = Advanced::new(advanced_texts(), Bugs::Fixed);
    let mut shown = Shown::default();
    let mut seats = [Seat::default(); 8];
    seats[0] = open();
    seats[2] = Seat {
        human: false,
        ..open()
    };
    let map = Map {
        seats,
        heroes: Heroes::default(),
        local: Some(0),
        saved: None,
    };
    let mut dialog = Dialog {
        place: [0, 0],
        shadow: None,
        items: Advanced::items(&advanced_texts().look),
    };
    advanced.show(&map, &mut dialog, &mut shown);
    advanced.flag_click(2, &map, &mut shown);
    assert_eq!(advanced.record(0).colour, Some(0));
    // A single player holding no colour takes the one pressed (no old flag to clear).
    let mut free = Advanced::new(advanced_texts(), Bugs::Fixed);
    let unheld = Map {
        local: None,
        ..map.clone()
    };
    free.show(&unheld, &mut dialog, &mut shown);
    free.flag_click(0, &unheld, &mut shown);
    assert_eq!(free.record(0).colour, Some(0));
    // Neither shown nor without an edit, a name opens nothing and closes nothing; no edit takes
    // a key.
    let mut hidden = Advanced::new(advanced_texts(), Bugs::Fixed);
    assert!(!hidden.name_click(0, &mut shown));
    assert!(!hidden.close_edit(&mut shown));
    assert!(!hidden.edit_key(Scan::plain(edit::RETURN), &mut shown));
    assert!(!hidden.edit_caret(Caret::Home) && !hidden.edit_typed('a'));
    assert_eq!(hidden.piece(&dialog, &shown, "P"), None);
    // A computer's name opens nothing.
    assert!(!advanced.name_click(2, &mut shown));
    assert!(!advanced.editing());
}

#[test]
fn the_name_s_edit_takes_the_keys_until_enter_in_the_original_s_profile() {
    let mut s = shown_in(vec![arrogance()], Bugs::Original);
    // A click on the single player's name: the edit over it, empty with its caret, the name
    // hidden; the bar's piece pasted at its corner (CLT-Q143).
    click(&mut s, [110, 138]);
    let draws = s.draws("P");
    assert_eq!(edit_of(&draws), Some(([62, 130], String::new(), Some(0))));
    assert!(texts_at(&draws, [62, 130]).is_empty());
    let (rect, piece) = s.piece_draws("P").unwrap();
    assert_eq!(rect, [62, 130, 92, 15]);
    assert!(pictures(&piece).contains(&(String::from("adoprpnl.pcx"), [52, 126])));
    assert_eq!(edit_of(&piece), edit_of(&draws));
    assert!(!s.typing() && s.contexts() == [Context::Scenario]);
    // Typed by scan code; the list's key S typed too (no hotkey); Escape and Tab are not the
    // edit's (Tab has no command; Escape is Back's, its click at the release).
    type_scans(&mut s, "ZedS");
    assert!(!s.showing());
    s.key(Some(Scan::plain(edit::BACKSPACE)), None, "BackSpace");
    assert_eq!(edit_of(&s.draws("P")).unwrap().1, "Zed");
    assert!(!s.key(Some(Scan::plain(edit::TAB)), None, "Tab").changed);
    // A key the adapter gives no scan code: not the edit's.
    assert!(!s.key(None, None, "F13").changed);
    // Its row drawn again (the hover): the name shown under the open edit, the piece gone.
    s.moved([35, 155]);
    assert!(s.piece_draws("P").is_none());
    assert_eq!(texts_at(&s.draws("P"), [62, 130]), ["", ""]);
    // Enter: the edit closed, its text the name; no Begin.
    assert!(
        s.key(Some(Scan::plain(edit::RETURN)), None, "Return")
            .changed
    );
    s.key_up("Return");
    let draws = s.draws("P");
    assert_eq!(edit_of(&draws), None);
    assert_eq!(texts_at(&draws, [62, 130]), ["Zed", "Zed"]);
    // The name moves with the player (`name-after-flag`).
    s.moved([790, 300]);
    click(&mut s, [35, 255]);
    let draws = s.draws("P");
    assert_eq!(texts_at(&draws, [62, 130 + dy(2)]), ["Zed", "Zed"]);
    assert_eq!(texts_at(&draws, [62, 130]), ["Computer", "Computer"]);
    // Opened again, it holds the name, the caret at its end; a flag's press closes it.
    click(&mut s, [110, 138 + dy(2)]);
    assert_eq!(
        edit_of(&s.draws("P")),
        Some(([62, 130 + dy(2)], String::from("Zed"), Some(3)))
    );
    type_scans(&mut s, "a");
    click(&mut s, [35, 155]);
    assert_eq!(texts_at(&s.draws("P"), [62, 130]), ["Zeda", "Zeda"]);
    // Hiding the options closes it too.
    click(&mut s, [110, 138]);
    assert!(click(&mut s, [514, 519]));
    click(&mut s, [514, 519]);
    assert_eq!(edit_of(&s.draws("P")), None);
    // The name's click at the release wherever the mouse is.
    s.moved([110, 138]);
    s.press([110, 138]);
    s.moved([790, 300]);
    assert!(s.release().changed);
    assert!(edit_of(&s.draws("P")).is_some());
}

#[test]
fn the_name_s_edit_types_through_the_text_input_in_the_client_s_profile() {
    let mut s = shown(vec![arrogance()]);
    // The name's click only over it at the release.
    s.moved([110, 138]);
    s.press([110, 138]);
    s.moved([790, 300]);
    assert!(!s.release().changed);
    assert!(!s.typing() && s.contexts() == [Context::Scenario]);
    assert_eq!(s.typed('Z'), Did::default());
    click(&mut s, [110, 138]);
    assert!(s.typing());
    assert_eq!(s.contexts(), [Context::Edit, Context::Scenario]);
    // No piece; the name stays hidden under the edit when its row is drawn again.
    assert!(s.piece_draws("P").is_none());
    for c in "Zedx".chars() {
        assert!(s.typed(c).changed);
    }
    assert!(
        s.key(None, Some(Command::Edit(Caret::BackSpace)), "BackSpace")
            .changed
    );
    s.moved([35, 155]);
    assert!(texts_at(&s.draws("P"), [62, 130]).is_empty());
    // The screen's other keys are the edit's (nothing), Back's its own; Begin's closes the edit.
    assert!(!s.key(None, Some(Command::Dialog(LIST)), "S").changed);
    assert!(!s.showing());
    assert!(!s.key(None, None, "F1").changed);
    s.key(None, Some(Command::Dialog(BEGIN)), "Return");
    s.key_up("Return");
    assert!(!s.typing());
    assert_eq!(texts_at(&s.draws("P"), [62, 130]), ["Zed", "Zed"]);
    // Opened again, Back's key is the screen's.
    click(&mut s, [110, 138]);
    s.key(None, Some(Command::Dialog(BACK)), "Escape");
    assert!(s.key_up("Escape").ended.is_some());
}

#[test]
fn over_no_map_a_flag_s_press_moves_no_screen_flag() {
    // The options shown over no map (no row, no flag to press): a flag's click changes no record
    // and no screen flag.
    let mut s = with_options(Vec::new());
    assert!(click(&mut s, [514, 519]));
    assert!(s.advanced_click(FLAG));
    assert_eq!(s.records().unwrap(), [Record::NEW; 8]);
}
