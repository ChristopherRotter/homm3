//! The scenario screen's tests (E27 M3 T1a): its information panel, its buttons, hotkeys and
//! popups, as `scn-new` recorded them.

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

use h3_render::Bugs;
use h3_ui::ui::Draw;

use super::*;
use crate::message::{self, Style};
use crate::shell::keys::{Command, Context};
use crate::shell::look::parse_layout;
use crate::shell::menu::tests::font;
use crate::shell::menu::{Arrow, BOX_PLAYER, Hovers};

const LAYOUT: &str = include_str!("../../../../../layouts/shell.toml");

/// The screen's texts: each label's name, numbered texts for the tables, a popup for each item
/// with help.
pub(crate) fn texts(bugs: Bugs) -> Texts {
    let layout = parse_layout(LAYOUT).unwrap();
    let (medium, small) = (font(8, 16), font(5, 10));
    let laid = |text: &str| {
        message::layout(
            &layout.message,
            (&medium, &small),
            text,
            Style::Popup,
            None,
            (BOX_PLAYER, bugs),
        )
    };
    let numbered = |what: &str, n: usize| (0..n).map(|k| format!("{what} {k}")).collect();
    Texts {
        labels: LABELS.iter().map(|&(id, n)| (id, n.to_string())).collect(),
        unnamed: String::from("Unnamed"),
        difficulties: numbered("difficulty", 5),
        ratings: RATINGS.iter().map(|r| format!("{r}%")).collect(),
        victories: numbered("victory", 12),
        with_standard: numbered("victory or standard", 11),
        losses: numbered("loss", 4),
        help: HELPED.iter().map(|&(id, n)| (id, laid(n))).collect(),
    }
}

/// A map of 36 tiles, red and blue a human's or the computer's, tan the computer's alone, red
/// and tan allied.
fn map(file: &str, name: &str) -> Entry {
    let both = Seat {
        human: true,
        computer: true,
    };
    let computer = Seat {
        human: false,
        computer: true,
    };
    let mut seats = [Seat::default(); 8];
    seats[0] = both;
    seats[1] = both;
    seats[2] = computer;
    Entry {
        file: file.to_string(),
        format: Some(Format::SoD),
        name: name.to_string(),
        description: String::from("A map."),
        size: 36,
        difficulty: 2,
        victory: None,
        standard: false,
        loss: None,
        seats,
        teams: [0, 1, 0, 3, 4, 5, 6, 7],
    }
}

/// The screen as the shell's layout has it, over `entries`.
pub(crate) fn scenario_of(entries: Vec<Entry>, bugs: Bugs) -> Scenario {
    let layout = parse_layout(LAYOUT).unwrap();
    Scenario::new(layout.scenario.dialog, texts(bugs), entries, bugs)
}

/// The screen over two maps, the kept one second.
pub(crate) fn scenario(bugs: Bugs) -> Scenario {
    scenario_of(
        vec![map("Other.h3m", "Other"), map("arrogance.H3M", "Arrogance")],
        bugs,
    )
}

/// The label drawn for item `id`, if any: its text.
fn label(draws: &[Draw], id: u16) -> Option<String> {
    draws.iter().find_map(|d| match d {
        Draw::Label { item, text, .. } if item.id == id => Some(text.clone()),
        _ => None,
    })
}

/// The frame drawn for the DEF item or button `def` at `at`, if any.
fn frame(draws: &[Draw], def: &str, at: [i32; 2]) -> Option<u32> {
    draws.iter().find_map(|d| match d {
        Draw::Button {
            def: d,
            frame,
            x,
            y,
            ..
        } if d == def && [*x, *y] == at => Some(*frame),
        _ => None,
    })
}

/// The flags drawn, by their place: (x, colour).
fn flags(draws: &[Draw]) -> Vec<(i32, u32)> {
    draws
        .iter()
        .filter_map(|d| match d {
            Draw::Button {
                def, frame, x, y, ..
            } if def == "itgflags.def" && *y == 405 => Some((*x, *frame)),
            _ => None,
        })
        .collect()
}

#[test]
fn the_panel_shows_the_kept_map() {
    let s = scenario(Bugs::Original);
    assert_eq!(s.entry().map(|e| e.name.as_str()), Some("Arrogance"));
    assert_eq!(s.entries().len(), 2);
    assert_eq!(s.difficulty(), KEPT_DIFFICULTY);
    let draws = s.draws("PLAYERS.PAL");
    assert_eq!(label(&draws, NAME).as_deref(), Some("Arrogance"));
    assert_eq!(label(&draws, DESCRIPTION).as_deref(), Some("A map."));
    assert_eq!(label(&draws, VICTORY).as_deref(), Some("victory 0"));
    assert_eq!(label(&draws, LOSS).as_deref(), Some("loss 0"));
    assert_eq!(
        label(&draws, MAP_DIFFICULTY).as_deref(),
        Some("difficulty 2")
    );
    assert_eq!(label(&draws, RATING).as_deref(), Some("100%"));
    assert_eq!(label(&draws, 132).as_deref(), Some("player-difficulty"));
    assert_eq!(frame(&draws, "scnrmpsz.def", [714, 28]), Some(0));
    assert_eq!(frame(&draws, "scnrvict.def", [420, 308]), Some(11));
    assert_eq!(frame(&draws, "scnrloss.def", [420, 365]), Some(3));
    // Red (the local colour) and tan allied, blue the enemy.
    assert_eq!(flags(&draws), vec![(460, 0), (475, 2), (640, 1)]);
    // Normal selected (frame 3), the others in frame 0.
    let difficulty: Vec<_> = (0..5)
        .map(|k| frame(&draws, &format!("gspbut{}.def", k + 3), [506 + 32 * k, 456]))
        .collect();
    assert_eq!(difficulty, [Some(0), Some(3), Some(0), Some(0), Some(0)]);
    // The captions over their buttons, one pixel up while released.
    let caption = draws.iter().find_map(|d| match d {
        Draw::Label { item, x, y, .. } if item.id == LIST => Some([*x, *y]),
        _ => None,
    });
    assert_eq!(caption, Some([414, 80]));
}

#[test]
fn the_victory_and_the_loss_take_their_texts_and_marks() {
    let mut special = map("Arrogance.h3m", "");
    special.victory = Some(4);
    special.loss = Some(2);
    special.size = 144;
    let s = scenario_of(vec![special.clone()], Bugs::Fixed);
    let draws = s.draws("PLAYERS.PAL");
    assert_eq!(label(&draws, NAME).as_deref(), Some("Unnamed"));
    assert_eq!(label(&draws, VICTORY).as_deref(), Some("victory 5"));
    assert_eq!(label(&draws, LOSS).as_deref(), Some("loss 3"));
    assert_eq!(frame(&draws, "scnrvict.def", [420, 308]), Some(4));
    assert_eq!(frame(&draws, "scnrloss.def", [420, 365]), Some(2));
    assert_eq!(frame(&draws, "scnrmpsz.def", [714, 28]), Some(3));
    special.standard = true;
    let s = scenario_of(vec![special], Bugs::Fixed);
    let draws = s.draws("PLAYERS.PAL");
    assert_eq!(
        label(&draws, VICTORY).as_deref(),
        Some("victory or standard 4")
    );
}

#[test]
fn without_the_kept_map_the_first_shows_and_without_maps_none() {
    let s = scenario_of(vec![map("A.h3m", "A"), map("B.h3m", "B")], Bugs::Fixed);
    assert_eq!(s.entry().map(|e| e.file.as_str()), Some("A.h3m"));
    let s = scenario_of(Vec::new(), Bugs::Fixed);
    assert_eq!(s.entry(), None);
    let draws = s.draws("PLAYERS.PAL");
    assert_eq!(label(&draws, NAME).as_deref(), Some(""));
    assert_eq!(frame(&draws, "scnrmpsz.def", [714, 28]), None);
    assert_eq!(frame(&draws, "scnrvict.def", [420, 308]), None);
    assert!(flags(&draws).is_empty());
    // The rating still shows the chosen difficulty's.
    assert_eq!(label(&draws, RATING).as_deref(), Some("100%"));
}

#[test]
fn the_local_colour_and_the_flags_follow_the_seats() {
    let mut entry = map("A.h3m", "A");
    assert_eq!(entry.local(), Some(0));
    // Blue only a human's comes first.
    entry.seats[1].computer = false;
    assert_eq!(entry.local(), Some(1));
    // No human: the first anyone plays.
    entry.seats = [Seat::default(); 8];
    entry.seats[5].computer = true;
    assert_eq!(entry.local(), Some(5));
    entry.seats[5].computer = false;
    assert_eq!(entry.local(), None);
    assert!(!Seat::default().plays());
    // The computer's colours alone: the first the local one, the other its enemy.
    entry.seats[3].computer = true;
    entry.seats[4].computer = true;
    let draws = scenario_of(vec![entry], Bugs::Fixed).draws("PLAYERS.PAL");
    assert_eq!(flags(&draws), vec![(460, 3), (640, 4)]);
}

#[test]
fn the_list_s_order_is_the_format_then_the_name_then_the_file() {
    let mut a = map("b.h3m", "Same");
    a.format = Some(Format::RoE);
    let b = map("a.h3m", "same");
    let c = map("A.h3m", "Same");
    let d = map("c.h3m", "Alpha");
    let mut old = map("z.h3m", "Old");
    old.format = None;
    let mut entries = vec![b.clone(), c.clone(), d.clone(), a.clone(), old.clone()];
    sort(&mut entries);
    let files: Vec<&str> = entries.iter().map(|e| e.file.as_str()).collect();
    assert_eq!(files, ["z.h3m", "b.h3m", "c.h3m", "A.h3m", "a.h3m"]);
    assert!(Format::RoE < Format::Ab && Format::Ab < Format::SoD);
}

#[test]
fn the_size_mark_s_frames() {
    let frames: Vec<u32> = [36, 72, 108, 144, 50].into_iter().map(size_frame).collect();
    assert_eq!(frames, [0, 1, 2, 3, 4]);
    assert_eq!(text_at(&[String::from("a")], 3), "");
}

#[test]
fn opened_anew_it_keeps_the_choices_and_carries_the_hovers() {
    let s = scenario(Bugs::Original);
    let hovers = Hovers {
        main: Some(101),
        game_type: Some(100),
        arrow: Arrow::Adventure,
    };
    let opened = s.opened(hovers);
    assert_eq!(opened.hovers(), hovers);
    // CLT-Q137 (b): the adventure map's arrow after the busy cursor in the original's profile
    // alone.
    let default = Hovers::default();
    assert_eq!(s.opened(default).hovers().arrow, Arrow::Adventure);
    let fixed = scenario(Bugs::Fixed).opened(default);
    assert_eq!(fixed.hovers().arrow, Arrow::Default);
    assert_eq!(opened.entry(), s.entry());
    assert_eq!(opened.background(), 0);
    assert_eq!(opened.clone().with_background(1).background(), 1);
    assert_eq!(opened.with_background(7).background(), 0);
    assert_eq!(s.context(), Context::Scenario);
    assert!(s.cursor());
}

#[test]
fn a_press_holds_a_button_and_back_ends_the_screen() {
    let mut s = scenario(Bugs::Original);
    let did = s.press([667, 555]);
    assert!(did.changed && did.sample);
    let draws = s.draws("PLAYERS.PAL");
    assert_eq!(frame(&draws, "scnrback.def", [584, 535]), Some(1));
    // Off and on again: up, then down with its sample.
    assert!(s.moved([790, 300]).changed);
    assert!(s.moved([667, 555]).sample);
    let did = s.release();
    assert_eq!(did.ended, Some(Ended::Scenario(false)));
    // A caption button's press: its text one pixel right, no ending at the release.
    let mut s = scenario(Bugs::Original);
    s.press([514, 91]);
    let caption = s.draws("PLAYERS.PAL").iter().find_map(|d| match d {
        Draw::Label { item, x, y, .. } if item.id == LIST => Some([*x, *y]),
        _ => None,
    });
    assert_eq!(caption, Some([415, 81]));
    assert_eq!(s.release().ended, None);
    // A move without a hold changes nothing (no hover).
    assert_eq!(s.moved([514, 115]), Did::default());
}

#[test]
fn a_right_press_shows_the_item_s_help_while_held() {
    let mut s = scenario(Bugs::Original);
    // The name drawn by the redraw takes no right press; nor does the flags' strip yet.
    assert_eq!(s.right([580, 57]), Did::default());
    assert_eq!(s.right([600, 414]), Did::default());
    assert!(s.box_draws("PLAYERS.PAL").is_empty());
    assert!(s.right([514, 91]).changed);
    assert!(!s.cursor());
    assert!(!s.box_draws("PLAYERS.PAL").is_empty());
    // Nothing else while it shows; a second right press neither.
    assert_eq!(s.right([497, 555]), Did::default());
    assert_eq!(s.moved([1, 1]), Did::default());
    assert_eq!(s.key(Some(Command::Dialog(BACK)), "Escape"), Did::default());
    assert!(!s.key_up("Escape").changed);
    assert!(s.right_release().changed);
    assert!(s.cursor());
    // Closed by a left press and by a left release too.
    s.right([497, 555]);
    assert!(s.press([1, 1]).changed);
    s.right([667, 555]);
    assert!(s.release().changed);
    // None while the mouse holds a button.
    s.press([667, 555]);
    assert_eq!(s.right([514, 91]), Did::default());
    // The right release lets the button go, off the button no click.
    s.moved([790, 300]);
    let did = s.right_release();
    assert!(did.changed && did.ended.is_none());
    // A hidden item takes none: the enemies' last flag's place.
    assert_eq!(s.item_at([745, 410]), Some(387));
}

#[test]
fn the_hotkeys_press_and_click_their_buttons() {
    let mut s = scenario(Bugs::Original);
    let did = s.key(Some(Command::Dialog(BACK)), "Escape");
    assert!(did.changed && did.sample);
    assert_eq!(s.key_up("Escape").ended, Some(Ended::Scenario(false)));
    // Begin's and the panel buttons' click nothing yet (their tasks').
    let mut s = scenario(Bugs::Original);
    s.key(Some(Command::Dialog(BEGIN)), "Return");
    assert_eq!(s.key_up("Return").ended, None);
    // Another dialog's button, or no command, takes nothing.
    assert_eq!(s.key(Some(Command::Dialog(124)), "Escape"), Did::default());
    assert_eq!(s.key(None, "B"), Did::default());
    let mut s = scenario(Bugs::Original);
    s.press([514, 519]);
    // A key while the mouse holds a button takes nothing.
    assert!(!s.key(Some(Command::Dialog(RANDOM)), "R").changed);
    assert!(s.release().changed);
    // An item's state kept as the screen shows it: the difficulties' selection.
    let draws = s.draws("PLAYERS.PAL");
    assert!(
        draws
            .iter()
            .any(|d| matches!(d, Draw::Button { def, frame: 3, .. } if def == "gspbut4.def"))
    );
}
