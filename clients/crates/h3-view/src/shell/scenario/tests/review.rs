//! The scenario screen's fixes of M3's review (E27 M3 T7): a release under a box (A1), the list's
//! record's caps (A3, CLT-Q151 (b)), the list's keys under a held button (A4), the ASCII fold
//! (A8), the double press's map (A9), the advanced options' rows over another map (B1, C1,
//! CLT-Q152 (b)), Begin with the name's edit open (B2), a colour only a human may play (B3,
//! CLT-Q93 (b)), a row's press from x 26 (CLT-Q150 (b)) and Enter's begin and stub (C7).

use h3_ui::edit;

use super::super::advanced::tests::with_options_in;
use super::*;
use crate::shell::Ended;

/// A seat a human or the computer may play, every faction offered at random, its main town
/// generating its hero.
fn open() -> Seat {
    Seat {
        human: true,
        computer: true,
        factions: 0x1FF,
        random_town: true,
        generates: true,
        ..Seat::default()
    }
}

/// A map named `name` of 36 tiles whose colours `colours` are each [`open`], the others no one's.
fn seated(file: &str, name: &str, colours: &[usize]) -> Entry {
    let mut e = map(file, name);
    e.seats = [Seat::default(); 8];
    for &c in colours {
        e.seats[c] = open();
    }
    e
}

/// The files in order.
fn files(entries: &[Entry]) -> Vec<&str> {
    entries.iter().map(|e| e.file.as_str()).collect()
}

/// Show Advanced Options' place, Show Available Scenarios', Random Map's, Begin's; the first
/// row's town's right arrow; Show Random Maps' (the tab's).
const ADVANCED_AT: [i32; 2] = [514, 519];
const LIST_AT: [i32; 2] = [514, 91];
const RANDOM_AT: [i32; 2] = [514, 115];
const BEGIN_AT: [i32; 2] = [497, 555];
const TOWN_AT: [i32; 2] = [230, 145];
const SHOW_AT: [i32; 2] = [225, 555];

#[test]
fn a_row_or_a_name_held_under_a_box_goes_without_its_click() {
    // A row held, a column's help opened by the right press: the release is the popup's (it
    // closes), no row selected; the original's would click the row anywhere.
    let mut s = shown(many(40), Bugs::Original);
    s.clock(2000);
    s.moved([216, 235]);
    assert_eq!(s.press([216, 235]), Did::default());
    assert!(s.right([41, 106]).changed);
    assert_eq!(s.over, Over::Help(190));
    assert!(s.release().changed);
    assert_eq!((&s.over, s.row), (&Over::Nothing, None));
    assert_eq!(s.rows(), (Some(0), 0));
    s.clock(3000);
    click(&mut s, [216, 235]);
    assert_eq!(s.rows(), (Some(4), 0));
    // A name held under a help popup: no edit opens at the release; without the box it does.
    let mut s = with_options_in(vec![map("Arrogance.h3m", "Arrogance")], Bugs::Fixed);
    click(&mut s, ADVANCED_AT);
    s.moved([110, 138]);
    s.press([110, 138]);
    assert!(s.right(LIST_AT).changed);
    assert!(s.release().changed);
    assert!(s.right_release() == Did::default() && s.name.is_none());
    assert!(!s.typing());
    click(&mut s, [110, 138]);
    assert!(s.typing());
}

#[test]
fn the_record_keeps_a_name_s_60_bytes_and_a_description_s_300_in_the_original_s_profile() {
    // CLT-Q151 (b): the original's panel, row and description cut, the client's whole; in
    // Windows-1252 a character is a byte.
    let mut long = map("Long.h3m", &"é".repeat(70));
    long.description = "d".repeat(400);
    for (bugs, name, description) in [(Bugs::Original, 60, 300), (Bugs::Fixed, 70, 400)] {
        let s = shown(vec![long.clone()], bugs);
        let draws = s.draws("PLAYERS.PAL");
        let count = |t: &str| t.chars().count();
        assert_eq!(label(&draws, NAME).as_deref().map(count), Some(name));
        assert_eq!(count(&names(&draws)[0]), name);
        assert_eq!(count(&s.shown.texts[&DESCRIPTION]), description);
        assert_eq!(s.entry().map(|e| count(&e.name)), Some(70));
    }
    // A chosen random map's record (short) whole in either.
    let mut s = scenario(Bugs::Original);
    click(&mut s, RANDOM_AT);
    assert_eq!(s.shown.texts[&DESCRIPTION], "Randomly made");
    // The sort by the name: tied on the record's 60 bytes in the original's, the files then.
    let x = "X".repeat(NAME_BYTES);
    let mut entries = vec![
        map("a.h3m", &format!("{x}b")),
        map("b.h3m", &format!("{x}a")),
    ];
    sort_by(&mut entries, 0, false, Bugs::Fixed);
    assert_eq!(files(&entries), ["b.h3m", "a.h3m"]);
    sort_by(&mut entries, 0, false, Bugs::Original);
    assert_eq!(files(&entries), ["a.h3m", "b.h3m"]);
    sort_by(&mut entries, BUILT_KEY, true, Bugs::Original);
    assert_eq!(files(&entries), ["a.h3m", "b.h3m"]);
}

#[test]
fn the_names_fold_the_ascii_letters_alone() {
    // The review's A8: an accented letter is its own byte, not its case's other.
    let mut entries = vec![
        map("1.h3m", "éa"),
        map("2.h3m", "Éz"),
        map("3.h3m", "MAP b"),
        map("4.h3m", "map a"),
    ];
    sort_by(&mut entries, 0, false, Bugs::Fixed);
    assert_eq!(files(&entries), ["4.h3m", "3.h3m", "2.h3m", "1.h3m"]);
}

#[test]
fn the_list_s_keys_do_nothing_while_the_mouse_holds_a_button() {
    // The review's A4: the button's own loop takes every message (the S filter held).
    let mut s = shown(many(40), Bugs::Fixed);
    s.moved([183, 68]);
    s.press([183, 68]);
    let down = Some(Command::List(Scroll::Down));
    assert_eq!(s.key(None, down, "Down"), Did::default());
    assert_eq!(s.rows(), (Some(0), 0));
    s.release();
    assert!(s.key(None, down, "Down").changed);
    assert_eq!(s.rows(), (Some(1), 0));
}

#[test]
fn the_client_s_double_press_begins_the_map_pressed_twice() {
    // The review's A9: a key between the presses, the second on the first's row: that map,
    // selected first, begins.
    let mut s = shown(many(40), Bugs::Fixed);
    s.clock(2000);
    click(&mut s, [216, 210]);
    assert_eq!(s.rows(), (Some(3), 0));
    let third = s.entry().unwrap().file.clone();
    s.key(None, Some(Command::List(Scroll::Down)), "Down");
    assert_eq!(s.rows(), (Some(4), 0));
    s.clock(2100);
    assert_eq!(click(&mut s, [216, 210]).ended, Some(Ended::Scenario(true)));
    assert_eq!(s.begun().map(|b| b.file), Some(third));
    // A sort between them: the row holds another map, which the second press selects alone.
    let mut s = shown(many(40), Bugs::Fixed);
    s.clock(2000);
    click(&mut s, [216, 210]);
    let third = s.entry().unwrap().file.clone();
    click(&mut s, [74, 106]);
    s.clock(2100);
    assert_eq!(click(&mut s, [216, 210]).ended, None);
    assert_eq!(s.rows(), (Some(3), 0));
    assert_ne!(s.entry().map(|e| &e.file), Some(&third));
}

/// The rows' records Begin hands on over `s`: each colour's (colour, town).
fn begun(s: &Scenario) -> Vec<(Option<u8>, Option<u8>)> {
    let begun = s.begun().unwrap();
    begun.records.iter().map(|r| (r.colour, r.town)).collect()
}

/// Red the single player's with Castle, the others the computer's, Random.
fn red_castle() -> Vec<(Option<u8>, Option<u8>)> {
    let mut records = vec![(None, None); 8];
    records[0] = (Some(0), Some(0));
    records
}

/// Colour `c` the single player's, every town Random.
fn on(c: usize) -> Vec<(Option<u8>, Option<u8>)> {
    let mut records = vec![(None, None); 8];
    records[c] = (u8::try_from(c).ok(), None);
    records
}

#[test]
fn the_client_s_options_are_made_again_over_another_map() {
    // CLT-Q152 (b), the review's B1: a sort leaving the selection on row 0 with another map
    // there. X's red chosen Castle; Y's single player blue.
    let x = seated("x.h3m", "A first", &[0, 1]);
    let y = seated("y.h3m", "B second", &[1, 2]);
    for bugs in [Bugs::Original, Bugs::Fixed] {
        let mut s = with_options_in(vec![x.clone(), y.clone()], bugs);
        click(&mut s, ADVANCED_AT);
        click(&mut s, TOWN_AT);
        assert_eq!(begun(&s), red_castle());
        click(&mut s, LIST_AT);
        click(&mut s, [216, 106]);
        click(&mut s, [216, 106]);
        assert_eq!(s.entry().map(|e| e.file.as_str()), Some("y.h3m"));
        // The original's keeps the rows (the select's index the same, 0x585574).
        let records = if bugs == Bugs::Original {
            red_castle()
        } else {
            on(1)
        };
        assert_eq!(begun(&s), records, "{bugs:?}");
    }
    // A filter leaving row 0 with another map there (Y of 72 tiles, its single player red).
    let mut y72 = seated("y.h3m", "B second", &[0, 3]);
    y72.size = 72;
    let mut s = with_options_in(vec![seated("x.h3m", "A first", &[1, 2]), y72], Bugs::Fixed);
    click(&mut s, ADVANCED_AT);
    click(&mut s, TOWN_AT);
    assert_eq!(begun(&s)[1], (Some(1), Some(0)));
    click(&mut s, LIST_AT);
    click(&mut s, [230, 68]);
    assert_eq!(s.entry().map(|e| e.file.as_str()), Some("y.h3m"));
    assert_eq!(begun(&s), on(0));
    // Shown again over the same map, the rows stay as made.
    click(&mut s, ADVANCED_AT);
    click(&mut s, TOWN_AT);
    click(&mut s, ADVANCED_AT);
    click(&mut s, ADVANCED_AT);
    assert_eq!(begun(&s), red_castle());
}

#[test]
fn the_client_s_options_are_made_again_leaving_a_random_map() {
    // CLT-Q152 (b), the review's C1: the options over a chosen random map (red the single
    // player's, Castle), then Show Available Scenarios: the selection's records (blue's).
    let x = seated("x.h3m", "A first", &[1, 2]);
    let mut s = with_options_in(vec![x.clone()], Bugs::Fixed);
    click(&mut s, RANDOM_AT);
    click(&mut s, ADVANCED_AT);
    click(&mut s, TOWN_AT);
    let red = s.records().unwrap()[0];
    assert_eq!((red.colour, red.town), (Some(0), Some(0)));
    click(&mut s, LIST_AT);
    assert_eq!(begun(&s), on(1));
    // Show Random Maps, the options over the random record the game holds, then the files'
    // list back (the kind's change: its first row given to the game).
    let mut s = with_options_in(vec![x], Bugs::Fixed);
    click(&mut s, RANDOM_AT);
    click(&mut s, SHOW_AT);
    assert_eq!(s.listed().count(), 0);
    click(&mut s, ADVANCED_AT);
    click(&mut s, TOWN_AT);
    assert_eq!(s.records().unwrap()[0].town, Some(0));
    click(&mut s, LIST_AT);
    assert_eq!(s.entry().map(|e| e.file.as_str()), Some("x.h3m"));
    assert_eq!(begun(&s), on(1));
}

#[test]
fn the_client_s_begin_takes_the_name_s_open_edit() {
    // The review's B2: Begin by the mouse with the edit open takes its text, as Enter's close
    // does; the original's record keeps its name (its path not read, design §26.5).
    for (bugs, name) in [(Bugs::Fixed, "Zed"), (Bugs::Original, "")] {
        let mut s = with_options_in(vec![seated("x.h3m", "A first", &[0, 1])], bugs);
        click(&mut s, ADVANCED_AT);
        click(&mut s, [110, 138]);
        for c in "Zed".chars() {
            let key = String::from(c);
            if bugs == Bugs::Fixed {
                s.typed(c);
            } else {
                s.key(edit::scan_of(u8::try_from(c).unwrap()), None, &key);
                s.key_up(&key);
            }
        }
        assert_eq!(
            click(&mut s, BEGIN_AT).ended,
            Some(Ended::Scenario(true)),
            "{bugs:?}"
        );
        let names = s.begun().unwrap().names;
        assert_eq!(names[0].as_deref(), Some(name), "{bugs:?}");
    }
}

/// Red and tan a human's or the computer's, blue and green a human's alone.
fn human_only() -> Entry {
    let mut e = seated("h.h3m", "Humans", &[0, 1, 2, 3]);
    for c in [1, 2] {
        e.seats[c].computer = false;
    }
    e
}

#[test]
fn a_colour_only_a_human_may_play_seats_the_single_player() {
    // CLT-Q93, decision 13: the single player on blue, the first colour only a human may play.
    // The original's makes blue and every later colour a human may play the computer's alone:
    // the player may move to red alone, never back; the client's (b) only green: it may move to
    // red and tan and back to blue.
    let kinds = |s: &Scenario| s.players().seats.map(|seat| (seat.human, seat.computer));
    let computer = (false, true);
    let both = (true, true);
    let mut s = with_options_in(vec![human_only()], Bugs::Original);
    assert_eq!(kinds(&s)[..4], [both, computer, computer, computer]);
    click(&mut s, ADVANCED_AT);
    assert_eq!(s.records().unwrap()[1].colour, Some(1));
    click(&mut s, [35, 155]);
    assert_eq!(s.records().unwrap()[0].colour, Some(0));
    click(&mut s, [35, 205]);
    assert_eq!(s.records().unwrap()[0].colour, Some(0));
    assert_eq!(s.records().unwrap()[1].colour, None);
    let mut s = with_options_in(vec![human_only()], Bugs::Fixed);
    assert_eq!(kinds(&s)[..4], [both, (true, false), computer, both]);
    click(&mut s, ADVANCED_AT);
    click(&mut s, [35, 155]);
    assert_eq!(s.records().unwrap()[0].colour, Some(0));
    click(&mut s, [35, 255]);
    assert_eq!(s.records().unwrap()[0].colour, Some(0));
    click(&mut s, [35, 205]);
    assert_eq!(s.records().unwrap()[1].colour, Some(1));
    assert_eq!(begun(&s), on(1));
    // The hot seat's colours as the map's.
    let hot = s.opened_hot_seat(Hovers::default(), &[String::from("A"), String::from("B")]);
    assert_eq!(kinds(&hot)[..4], [both, (true, false), (true, false), both]);
}

#[test]
fn the_client_s_row_takes_the_press_from_the_players_column() {
    // CLT-Q150 (b): x 40 on row 4 selects it in the client's profile; the original's row starts
    // at x 57. Released off the row (on the next) nothing.
    let mut f = shown(many(40), Bugs::Fixed);
    f.clock(2000);
    click(&mut f, [40, 235]);
    assert_eq!(f.rows(), (Some(4), 0));
    f.clock(3000);
    f.press([40, 285]);
    f.moved([40, 310]);
    f.release();
    assert_eq!(f.rows(), (Some(4), 0));
    let mut o = shown(many(40), Bugs::Original);
    o.clock(2000);
    click(&mut o, [40, 235]);
    assert_eq!(o.rows(), (Some(0), 0));
}

#[test]
fn enter_begins_over_a_map_and_shows_the_stub_over_a_random_map() {
    // The review's C7: Enter's Begin over the selected map ends the screen with it.
    let mut s = scenario(Bugs::Fixed);
    s.key(None, Some(Command::Dialog(BEGIN)), "Return");
    assert_eq!(s.key_up("Return").ended, Some(Ended::Scenario(true)));
    assert_eq!(
        s.begun().map(|b| b.file),
        Some(String::from("arrogance.H3M"))
    );
    // With a random map chosen its stub's notice, the screen staying.
    let mut s = scenario(Bugs::Fixed);
    click(&mut s, RANDOM_AT);
    s.key(None, Some(Command::Dialog(BEGIN)), "Return");
    assert_eq!(s.key_up("Return").ended, None);
    assert!(s.stubbed());
    assert_eq!(s.begun(), None);
}
