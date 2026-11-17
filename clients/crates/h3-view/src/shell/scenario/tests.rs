//! The scenario screen's tests (E27 M3 T1a): its information panel, its buttons, hotkeys and
//! popups, as `scn-new` recorded them; the list (T1b1, T1b2); the difficulty, the empty list and
//! Begin's message (T2a, T2b).

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
        players: (0..9)
            .flat_map(|p| (0..9).map(move |h| format!("{p}/{h}")))
            .collect(),
        sizes: ["S", "M", "L", "XL"].map(String::from).into(),
        teams: Some(Teams {
            look: layout.scenario.teams.clone(),
            title: String::from("Teams"),
            names: (1..=8).map(|k| format!("Team {k}")).collect(),
        }),
        no_maps: Some(message::layout(
            &layout.message,
            (&medium, &small),
            "No maps",
            Style::Ok,
            None,
            (BOX_PLAYER, bugs),
        )),
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
    Scenario::new(
        layout.scenario.dialog,
        layout.scenario.list,
        texts(bugs),
        entries,
        bugs,
    )
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
    assert_eq!(s.difficulty(), OPENING_DIFFICULTY);
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
    assert_eq!(frame(&draws, "scnrvict.def", [420, 308]), None);
    assert!(flags(&draws).is_empty());
    // E27 M3 T2b: no rating; the size mark's frame 4 (CLT-Q115 (b)), Normal chosen (CLT-Q139).
    assert_eq!(label(&draws, RATING).as_deref(), Some(""));
    assert_eq!(frame(&draws, "scnrmpsz.def", [714, 28]), Some(4));
    assert_eq!(chosen(&draws), Some(1));
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
    // E27 M3 T2a: each opening takes Normal again, the kept map kept.
    let mut hard = scenario(Bugs::Original);
    click(&mut hard, [585, 478]);
    assert_eq!(hard.difficulty(), 2);
    let again = hard.opened(default);
    assert_eq!(again.difficulty(), OPENING_DIFFICULTY);
    assert_eq!(again.entry(), hard.entry());
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
    // The name drawn by the redraw takes no right press.
    assert_eq!(s.right([580, 57]), Did::default());
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

// E27 M3 T1b1: the list.

/// `n` maps named "Map 00" on, their sizes in turn 36, 72, 108, 144, all SoD's, and the kept map
/// "Arrogance" (36 tiles) among them in the name's order.
pub(crate) fn many(n: usize) -> Vec<Entry> {
    let mut entries: Vec<Entry> = (0..n)
        .map(|k| {
            let mut e = map(&format!("m{k:02}.h3m"), &format!("Map {k:02}"));
            e.size = [36, 72, 108, 144][k % 4];
            e
        })
        .collect();
    entries.push(map("Arrogance.h3m", "Arrogance"));
    sort(&mut entries);
    entries
}

/// A label's text or a flag's colour, with its place.
type Placed<T> = Vec<(T, i32, i32)>;

/// The teams' dialog's labels and flags drawn: (text, x, y) and (colour, x, y).
fn teams_drawn(draws: &[Draw]) -> (Placed<String>, Placed<u32>) {
    let mut labels = Vec::new();
    let mut flags = Vec::new();
    for d in draws {
        match d {
            Draw::Label { text, x, y, .. } => labels.push((text.clone(), *x, *y)),
            Draw::Button {
                def, frame, x, y, ..
            } if def == "itgflags.def" => flags.push((*frame, *x, *y)),
            _ => {}
        }
    }
    (labels, flags)
}

#[test]
fn the_strip_s_right_press_shows_the_teams_while_held() {
    // Red and tan of one team, blue its own: two teams, ((100 + 56) / 64 + 1) × 64 = 192 high.
    let mut s = scenario(Bugs::Original);
    assert!(s.right([600, 414]).changed);
    assert!(s.cursor(), "the teams' dialog runs with the cursor");
    let draws = s.box_draws("PLAYERS.PAL");
    let (labels, flags) = teams_drawn(&draws);
    let (x, y) = (272, (600 - 192) / 2);
    assert_eq!(
        labels,
        vec![
            (String::from("Teams"), x + 10, y + 20),
            (String::from("Team 1"), x + 10, y + 56),
            (String::from("Team 2"), x + 10, y + 106),
        ]
    );
    // A team's flags centred: (256 - (18 × 2 - 3)) / 2 = 111, then (256 - 15) / 2 = 120.
    assert_eq!(
        flags,
        vec![
            (0, x + 111, y + 76),
            (2, x + 129, y + 76),
            (1, x + 120, y + 126)
        ]
    );
    // The frame in the DEF's own colours, a shadow under it.
    assert!(draws.iter().any(|d| matches!(d, Draw::Darken { .. })));
    // A left press, a release, a move and a key change nothing; the right release closes it.
    assert_eq!(s.press([1, 1]), Did::default());
    assert_eq!(s.release(), Did::default());
    assert_eq!(s.moved([2, 2]), Did::default());
    assert_eq!(s.key(Some(Command::Dialog(BACK)), "Escape"), Did::default());
    assert_eq!(s.right([514, 91]), Did::default());
    assert!(s.right_release().changed);
    assert!(s.box_draws("PLAYERS.PAL").is_empty());
    // Every colour its own team: one flag a team, as many teams as colours that play.
    let mut solo = map("Arrogance.h3m", "Arrogance");
    solo.teams = [0, 1, 2, 3, 4, 5, 6, 7];
    let mut s = scenario_of(vec![solo], Bugs::Fixed);
    s.right([600, 414]);
    let (labels, flags) = teams_drawn(&s.box_draws("PLAYERS.PAL"));
    let y = (600 - 256) / 2;
    assert_eq!(labels.len(), 4);
    assert_eq!(flags[2], (2, 272 + 120, y + 176));
    // No dialog without its texts, nor without a selection in the client's profile (T2b).
    let mut s = scenario(Bugs::Original);
    s.texts.teams = None;
    assert_eq!(s.right([600, 414]), Did::default());
    let mut s = scenario_of(Vec::new(), Bugs::Fixed);
    assert_eq!(s.right([600, 414]), Did::default());
}

#[test]
fn the_list_s_items_and_bar_show_their_help() {
    let mut s = shown(many(30), Bugs::Original);
    // The columns, the filters, the labels and the bar; a row none.
    for (point, id) in [
        ([41, 106], 190),
        ([216, 106], 193),
        ([358, 106], 195),
        ([183, 68], 137),
        ([371, 68], 141),
        ([383, 300], BAR),
        ([459, 444], 134),
        ([580, 444], 132),
        ([707, 444], 133),
    ] {
        assert!(s.right(point).changed, "{point:?}");
        assert_eq!(s.over, Over::Help(id), "{point:?}");
        assert!(s.right_release().changed);
    }
    assert_eq!(s.right([216, 160]), Did::default());
    // The bar's help only while the list shows.
    let mut s = scenario_of(many(30), Bugs::Original);
    assert_eq!(s.right([383, 300]), Did::default());
}

/// A click at `point`: the move, the press and the release.
fn click(s: &mut Scenario, point: [i32; 2]) -> Did {
    s.moved(point);
    s.press(point);
    s.release()
}

/// The list shown (Show Available Scenarios clicked).
fn shown(entries: Vec<Entry>, bugs: Bugs) -> Scenario {
    let mut s = scenario_of(entries, bugs);
    s.clock(1000);
    assert!(click(&mut s, [514, 91]).changed);
    assert!(s.showing());
    s
}

/// The rows' names drawn, from the top.
fn names(draws: &[Draw]) -> Vec<String> {
    draws
        .iter()
        .filter_map(|d| match d {
            Draw::Label { item, text, .. } if item.id == 1103 => Some(text.clone()),
            _ => None,
        })
        .collect()
}

/// The selected name's row: its colour 5.
fn selected_row(draws: &[Draw]) -> Option<i32> {
    draws.iter().find_map(|d| match d {
        Draw::Label { item, y, .. } if item.id == 1103 && item.colour == 5 => Some((y - 122) / 25),
        _ => None,
    })
}

/// The thumb's top on the screen.
fn thumb(draws: &[Draw]) -> Option<i32> {
    draws.iter().find_map(|d| match d {
        Draw::Button {
            def, frame: 4, y, ..
        } if def == "SlideBuV.def" => Some(*y),
        _ => None,
    })
}

#[test]
fn the_list_shows_and_hides_with_its_items_and_draws_its_rows() {
    let mut s = scenario_of(many(3), Bugs::Original);
    let hidden = s.draws("PLAYERS.PAL");
    assert!(names(&hidden).is_empty() && thumb(&hidden).is_none());
    assert!(label(&hidden, TITLE).is_none());
    assert!(click(&mut s, [514, 91]).changed);
    let draws = s.draws("PLAYERS.PAL");
    assert_eq!(label(&draws, TITLE).as_deref(), Some("title"));
    assert_eq!(label(&draws, SIZES).as_deref(), Some("sizes"));
    // The kept map on the top row, selected; three rows after it: the list is shorter than a page,
    // the top 0.
    assert_eq!(names(&draws), ["Arrogance", "Map 00", "Map 01", "Map 02"]);
    assert_eq!(s.rows(), (Some(0), 0));
    assert_eq!(selected_row(&draws), Some(0));
    // Each row: "%d/%d" (three colours play, two a human's), the size's letter, the version's
    // mark, the marks.
    assert_eq!(label(&draws, 1100).as_deref(), Some("3/2"));
    assert_eq!(label(&draws, 1101).as_deref(), Some("S"));
    assert_eq!(frame(&draws, "ScSelC.def", [91, 123]), Some(2));
    assert_eq!(frame(&draws, "scnrvict.def", [309, 148]), Some(11));
    assert_eq!(frame(&draws, "scnrloss.def", [342, 148]), Some(3));
    let letters: Vec<String> = draws
        .iter()
        .filter_map(|d| match d {
            Draw::Label { item, text, .. } if item.id == 1101 => Some(text.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(letters, ["S", "S", "M", "L"]);
    // A single tick: the thumb at the top; the filters and the columns shown.
    assert_eq!(thumb(&draws), Some(92 + 16));
    assert_eq!(frame(&draws, "scalbut.def", [349, 52]), Some(0));
    assert_eq!(frame(&draws, "scbutt3.def", [124, 92]), Some(0));
    // Hidden again by a second press, the selection kept.
    assert!(click(&mut s, [514, 91]).changed);
    assert!(!s.showing());
    assert!(names(&s.draws("PLAYERS.PAL")).is_empty());
    // Shown again: not built again.
    s.key(Some(Command::List(Scroll::Down)), "Down");
    click(&mut s, [514, 91]);
    assert_eq!(s.rows(), (Some(0), 0));
}

#[test]
fn a_row_draws_an_old_format_without_its_mark_and_an_unnamed_map_so() {
    let mut old = map("old.h3m", "");
    old.format = None;
    old.size = 50;
    old.victory = Some(3);
    old.loss = Some(1);
    let s = shown(vec![old], Bugs::Fixed);
    let draws = s.draws("PLAYERS.PAL");
    assert_eq!(names(&draws), ["Unnamed"]);
    assert_eq!(label(&draws, 1101).as_deref(), Some("XL"));
    assert_eq!(frame(&draws, "ScSelC.def", [91, 123]), None);
    assert_eq!(frame(&draws, "scnrvict.def", [309, 123]), Some(3));
    assert_eq!(frame(&draws, "scnrloss.def", [342, 123]), Some(1));
    assert_eq!(Format::RoE.mark() + Format::Ab.mark(), 1);
}

#[test]
fn the_list_is_built_with_the_kept_map_on_top_no_further_than_a_page_from_the_end() {
    // 40 maps and Arrogance (the first by name): its row 0 on top.
    let s = shown(many(40), Bugs::Fixed);
    assert_eq!(s.rows(), (Some(0), 0));
    // The kept map near the end: the top the last page's.
    let mut entries = many(40);
    entries.iter_mut().for_each(|e| {
        if e.file == "Arrogance.h3m" {
            e.name = String::from("Zed");
        }
    });
    sort(&mut entries);
    let s = shown(entries, Bugs::Fixed);
    assert_eq!(s.rows(), (Some(40), 41 - 18));
    let draws = s.draws("PLAYERS.PAL");
    assert_eq!(selected_row(&draws), Some(17));
    // The bar's 24 ticks at their last: the thumb at its foot.
    assert_eq!(thumb(&draws), Some(92 + 480 - 32));
    // Without the kept map the first row.
    let s = shown(vec![map("A.h3m", "A"), map("B.h3m", "B")], Bugs::Fixed);
    assert_eq!(s.rows(), (Some(0), 0));
    let s = shown(Vec::new(), Bugs::Fixed);
    assert_eq!(s.rows(), (None, 0));
    assert!(names(&s.draws("PLAYERS.PAL")).is_empty());
}

#[test]
fn the_columns_sort_and_flip_the_size_first_descending_in_the_original_s_profile() {
    let mut entries = many(6);
    entries[1].victory = Some(2);
    entries[2].loss = Some(0);
    let mut s = shown(entries.clone(), Bugs::Original);
    // CLT-Q91: the recorded key after the build is the size's: its first press flips it.
    click(&mut s, [74, 106]);
    let sizes: Vec<u32> = s.listed().map(|e| e.size).collect();
    assert_eq!(sizes, [144, 108, 72, 72, 36, 36, 36]);
    assert_eq!(s.rows(), (Some(0), 0));
    // The ties by name.
    let names_of = |s: &Scenario| -> Vec<String> { s.listed().map(|e| e.name.clone()).collect() };
    assert_eq!(names_of(&s)[4..], ["Arrogance", "Map 00", "Map 04"]);
    click(&mut s, [74, 106]);
    let sizes: Vec<u32> = s.listed().map(|e| e.size).collect();
    assert_eq!(sizes, [36, 36, 36, 72, 72, 108, 144]);
    // CLT-Q91 (c): the client's first press sorts ascending.
    let mut f = shown(entries, Bugs::Fixed);
    click(&mut f, [74, 106]);
    let sizes: Vec<u32> = f.listed().map(|e| e.size).collect();
    assert_eq!(sizes, [36, 36, 36, 72, 72, 108, 144]);
    // The name, descending on the second press.
    click(&mut f, [216, 106]);
    assert_eq!(names_of(&f)[0], "Arrogance");
    click(&mut f, [216, 106]);
    assert_eq!(names_of(&f)[0], "Map 05");
    // The version: the format then the name, reversed whole.
    let mut v = f.clone();
    click(&mut v, [107, 106]);
    assert_eq!(names_of(&v)[0], "Arrogance");
    click(&mut v, [107, 106]);
    assert_eq!(names_of(&v)[0], "Map 05");
    // The players: ten times the colours plus the humans'.
    let mut p = f.clone();
    click(&mut p, [41, 106]);
    assert_eq!(p.rows(), (Some(0), 0));
    // The victory's code (none first), the loss's.
    click(&mut f, [325, 106]);
    assert_eq!(names_of(&f)[6], "Map 00");
    click(&mut f, [358, 106]);
    assert_eq!(names_of(&f)[6], "Map 01");
    // Another key compares nothing: the name then the file.
    let mut all = many(2);
    sort_by(&mut all, 9, true);
    assert_eq!(all[0].name, "Arrogance");
}

#[test]
fn the_filters_keep_a_size_and_the_selection_goes_to_the_first() {
    let mut s = shown(many(8), Bugs::Fixed);
    click(&mut s, [230, 68]);
    assert!(s.listed().all(|e| e.size == 72));
    assert_eq!(s.listed().count(), 2);
    assert_eq!(s.rows(), (Some(0), 0));
    assert_eq!(s.entry().map(|e| e.size), Some(72));
    // XL keeps two, All every one; a size of none leaves the list empty.
    click(&mut s, [324, 68]);
    assert_eq!(s.listed().count(), 2);
    click(&mut s, [371, 68]);
    assert_eq!(s.listed().count(), 9);
    let mut small = shown(vec![map("A.h3m", "A")], Bugs::Fixed);
    click(&mut small, [277, 68]);
    assert_eq!((small.listed().count(), small.rows()), (0, (None, 0)));
    assert_eq!(small.entry(), None);
    // A sort keeps the filter.
    click(&mut s, [183, 68]);
    click(&mut s, [216, 106]);
    assert!(s.listed().all(|e| e.size == 36));
}

#[test]
fn the_keys_move_the_selection_and_the_top() {
    let mut s = shown(many(40), Bugs::Fixed);
    let key = |s: &mut Scenario, scroll| s.key(Some(Command::List(scroll)), "k");
    // Up on the first row: nothing.
    assert!(key(&mut s, Scroll::Up).changed);
    assert_eq!(s.rows(), (Some(0), 0));
    // Down to the last row shown, then past it: the top follows.
    for _ in 0..17 {
        key(&mut s, Scroll::Down);
    }
    assert_eq!(s.rows(), (Some(17), 0));
    key(&mut s, Scroll::Down);
    assert_eq!(s.rows(), (Some(18), 1));
    // Up above the top: the top follows.
    key(&mut s, Scroll::PageUp);
    assert_eq!(s.rows(), (Some(17), 0));
    // Page Down keeps the row; to the last page at most.
    key(&mut s, Scroll::PageDown);
    assert_eq!(s.rows(), (Some(34), 17));
    key(&mut s, Scroll::PageDown);
    assert_eq!(s.rows(), (Some(40), 23));
    key(&mut s, Scroll::PageDown);
    assert_eq!(s.rows(), (Some(40), 23));
    // Down on the last: nothing; Up past the top.
    key(&mut s, Scroll::Down);
    assert_eq!(s.rows(), (Some(40), 23));
    key(&mut s, Scroll::PageUp);
    key(&mut s, Scroll::PageUp);
    assert_eq!(s.rows(), (Some(17), 0));
    for _ in 0..17 {
        key(&mut s, Scroll::Up);
    }
    assert_eq!(s.rows(), (Some(0), 0));
    // The bar's thumb with the top: 24 ticks.
    key(&mut s, Scroll::PageDown);
    assert_eq!(
        thumb(&s.draws("PLAYERS.PAL")),
        Some(92 + 16 + 432 * 17 / 23)
    );
    // A list of one page: no Page Down; an empty list: no key; the list hidden: none either.
    let mut short = shown(many(3), Bugs::Fixed);
    key(&mut short, Scroll::PageDown);
    assert_eq!(short.rows(), (Some(0), 0));
    let mut empty = shown(Vec::new(), Bugs::Fixed);
    key(&mut empty, Scroll::Down);
    assert_eq!(empty.rows(), (None, 0));
    let mut hidden = scenario_of(many(3), Bugs::Fixed);
    assert_eq!(key(&mut hidden, Scroll::Down), Did::default());
    // Selected off the list's page by the bar, Page Up keeps a row above the top none.
    let mut off = shown(many(40), Bugs::Fixed);
    click(&mut off, [383, 540]);
    assert_eq!(off.rows(), (Some(0), 18));
    key(&mut off, Scroll::PageUp);
    assert_eq!(off.rows(), (Some(0), 1));
    // Page Down keeps its distance from the top: one row above it, row 17 of the next page.
    key(&mut off, Scroll::PageDown);
    assert_eq!(off.rows(), (Some(17), 18));
    key(&mut off, Scroll::Down);
    assert_eq!(off.rows(), (Some(18), 18));
}

#[test]
fn the_bar_moves_the_top_and_holds_the_mouse() {
    let mut s = shown(many(40), Bugs::Original);
    // The bottom arrow: one row at the release, drawn pressed while held.
    s.moved([383, 562]);
    assert!(s.press([383, 562]).changed);
    let draws = s.draws("PLAYERS.PAL");
    assert_eq!(frame(&draws, "SlideBuV.def", [375, 556]), Some(3));
    // Held, it takes no key and no right press; its move follows the mouse.
    assert_eq!(
        s.key(Some(Command::List(Scroll::Down)), "Down"),
        Did::default()
    );
    assert_eq!(s.right([514, 91]), Did::default());
    assert!(s.moved([383, 563]).changed);
    assert!(s.release().changed);
    assert_eq!(s.rows(), (Some(0), 1));
    // The track below the thumb: a page of 18; a move over the track drags the thumb.
    s.press([383, 540]);
    assert_eq!(s.rows(), (Some(0), 19));
    s.moved([383, 92 + 24]);
    assert_eq!(s.rows(), (Some(0), 0));
    // The right release ends the bar's hold too.
    assert!(s.right_release().changed);
    assert_eq!(s.moved([383, 300]), Did::default());
    // The list hidden: no bar.
    click(&mut s, [514, 91]);
    assert_eq!(s.press([383, 540]), Did::default());
}

#[test]
fn a_row_s_release_selects_it_and_a_second_soon_begins() {
    let mut s = shown(many(40), Bugs::Fixed);
    s.clock(2000);
    // Row 4 (y 222 to 246).
    assert_eq!(s.press([216, 235]), Did::default());
    assert!(s.release().changed);
    assert_eq!(s.rows(), (Some(4), 0));
    // A second press on another row within 400 ms: the client's needs the same row.
    s.clock(2100);
    click(&mut s, [216, 260]);
    assert_eq!(s.rows(), (Some(5), 0));
    // The same row again within 400 ms: Begin (nothing yet), the selection as it is.
    s.clock(2200);
    click(&mut s, [216, 260]);
    assert_eq!(s.rows(), (Some(5), 0));
    // Later: selected (the same row: the partial redraw).
    s.clock(3000);
    click(&mut s, [216, 260]);
    assert_eq!(s.rows(), (Some(5), 0));
    // Released off the row: the client's selects nothing; past the list: nothing.
    s.clock(4000);
    s.press([216, 285]);
    s.moved([700, 300]);
    s.release();
    assert_eq!(s.rows(), (Some(5), 0));
    let mut short = shown(many(1), Bugs::Fixed);
    short.clock(5000);
    click(&mut short, [216, 285]);
    assert_eq!(short.rows(), (Some(0), 0));
    // The original's: any row within 400 ms of the last, or of the opening, begins; a release
    // anywhere clicks.
    let mut o = scenario_of(many(40), Bugs::Original);
    o.clock(1000);
    click(&mut o, [514, 91]);
    o.clock(1200);
    click(&mut o, [216, 235]);
    assert_eq!(o.rows(), (Some(0), 0));
    o.clock(1700);
    o.press([216, 235]);
    o.moved([700, 300]);
    o.release();
    assert_eq!(o.rows(), (Some(4), 0));
    o.clock(1800);
    click(&mut o, [216, 260]);
    assert_eq!(o.rows(), (Some(4), 0));
}

#[test]
fn a_hidden_item_takes_no_press() {
    // The filters' place with the list hidden.
    let mut s = scenario_of(many(8), Bugs::Fixed);
    assert_eq!(s.press([183, 68]), Did::default());
    assert_eq!(s.release(), Did::default());
    assert_eq!(s.listed().count(), 9);
}

#[test]
fn the_original_s_partial_redraw_leaves_the_size_mark_and_the_strip() {
    // CLT-Q138: after a sort the select's partial redraw: the size mark the previous selection's,
    // the panel over the items it does not redraw, the flags' strip restored.
    let mut entries = many(3);
    entries[3].size = 108;
    let mut s = shown(entries.clone(), Bugs::Original);
    assert!(s.stale_draws("PLAYERS.PAL").is_none());
    click(&mut s, [216, 106]);
    click(&mut s, [216, 106]);
    let draws = s.draws("PLAYERS.PAL");
    assert_eq!(s.entry().map(|e| e.size), Some(108));
    assert_eq!(frame(&draws, "scnrmpsz.def", [714, 28]), Some(0));
    // The panel drawn after Begin.
    let at = |name: &str| {
        draws.iter().position(
            |d| matches!(d, Draw::Picture { picture, .. } | Draw::Button { def: picture, .. } if picture == name),
        )
    };
    assert!(at("SCSelBck.pcx") > at("scnrbeg.def"));
    let (rect, strip) = s.stale_draws("PLAYERS.PAL").unwrap();
    assert_eq!(rect, STRIP);
    assert!(
        strip
            .iter()
            .any(|d| matches!(d, Draw::Picture { picture, .. } if picture == "GSelPop1.pcx"))
    );
    assert_eq!(flags(&strip).len(), 3);
    assert!(label(&strip, 1013).is_none() && label(&strip, ENEMIES_LABEL).is_some());
    // A whole redraw (row 1: another map, 72 tiles) ends it.
    s.clock(9000);
    click(&mut s, [216, 160]);
    assert!(s.stale_draws("PLAYERS.PAL").is_none());
    assert_eq!(
        frame(&s.draws("PLAYERS.PAL"), "scnrmpsz.def", [714, 28]),
        Some(1)
    );
    // The client's draws whole.
    let mut f = shown(entries, Bugs::Fixed);
    click(&mut f, [216, 106]);
    click(&mut f, [216, 106]);
    assert!(f.stale_draws("PLAYERS.PAL").is_none());
    assert_eq!(
        frame(&f.draws("PLAYERS.PAL"), "scnrmpsz.def", [714, 28]),
        Some(2)
    );
    // The list hidden: none; the help popup's close redraws whole.
    let mut s = shown(many(40), Bugs::Original);
    click(&mut s, [383, 562]);
    assert!(s.stale_draws("PLAYERS.PAL").is_some());
    s.right([514, 91]);
    s.right_release();
    assert!(s.stale_draws("PLAYERS.PAL").is_none());
    // A key whose selection stays (Page Down on the last page): partial; Down: whole.
    let key = |s: &mut Scenario, scroll| s.key(Some(Command::List(scroll)), "k");
    key(&mut s, Scroll::PageDown);
    key(&mut s, Scroll::PageDown);
    assert_eq!(s.rows(), (Some(22), 23));
    assert!(s.stale_draws("PLAYERS.PAL").is_none());
    key(&mut s, Scroll::PageDown);
    assert!(s.stale_draws("PLAYERS.PAL").is_some());
    // Up selects the row above: whole.
    key(&mut s, Scroll::Up);
    assert!(s.stale_draws("PLAYERS.PAL").is_none());
    // Up on the first row redraws nothing: the bar's partial redraw stays.
    let mut first = shown(many(40), Bugs::Original);
    click(&mut first, [383, 562]);
    key(&mut first, Scroll::Up);
    assert!(first.stale_draws("PLAYERS.PAL").is_some());
    // The list hidden: none.
    click(&mut first, [514, 91]);
    assert!(first.stale_draws("PLAYERS.PAL").is_none());
}

#[test]
fn the_entry_counts_its_players_and_the_list_names_its_assets() {
    let e = map("A.h3m", "A");
    assert_eq!((e.players(), e.humans()), (3, 2));
    let layout = parse_layout(LAYOUT).unwrap();
    let names = list_names(&layout.scenario.list);
    assert!(names.iter().any(|n| n == "ScSelC.def") && names.iter().any(|n| n == "sliderV.pcx"));
    let names = teams_names(&layout.scenario.teams);
    assert!(names.iter().any(|n| n == "dialgbox.def") && names.iter().any(|n| n == "itgflags.def"));
}

// E27 M3 T2a and T2b: the difficulty, the empty list and Begin's message.

/// The difficulty whose button shows selected (frame 3), if any (0 easy to 4 impossible).
fn chosen(draws: &[Draw]) -> Option<i32> {
    (0..5).find(|&k| {
        frame(draws, &format!("gspbut{}.def", k + 3), [506 + 32 * k, 456]) == Some(3)
    })
}

#[test]
fn a_difficulty_s_press_chooses_it_and_the_list_keeps_it() {
    let mut s = scenario(Bugs::Original);
    // Held, it shows pressed; at the release over it the choice, the screen drawn whole.
    let did = s.press([617, 478]);
    assert!(did.changed && did.sample);
    let did = s.release();
    assert!(did.changed && did.ended.is_none());
    assert_eq!(s.difficulty(), 3);
    let draws = s.draws("PLAYERS.PAL");
    assert_eq!(chosen(&draws), Some(3));
    assert_eq!(label(&draws, RATING).as_deref(), Some("160%"));
    // Each in turn, the rating its own.
    for (k, rating) in [(0, "80%"), (1, "100%"), (2, "130%"), (4, "200%")] {
        click(&mut s, [521 + 32 * k, 478]);
        let draws = s.draws("PLAYERS.PAL");
        assert_eq!(chosen(&draws), Some(k));
        assert_eq!(label(&draws, RATING).as_deref(), Some(rating));
    }
    // The list's first showing keeps it (`scn-difficulty` impossible-list), its rows too.
    click(&mut s, [514, 91]);
    assert!(s.showing());
    assert_eq!(chosen(&s.draws("PLAYERS.PAL")), Some(4));
    click(&mut s, [216, 135]);
    assert_eq!(s.difficulty(), 4);
}

#[test]
fn an_empty_list_s_panel_keeps_the_original_s_leftovers() {
    // A filter that empties the list: the original's size mark the filter's, the last map's
    // flags; no map's texts, no rating; the choice kept.
    let mut s = shown(vec![map("Arrogance.h3m", "Arrogance")], Bugs::Original);
    click(&mut s, [324, 68]);
    assert_eq!(s.entry(), None);
    let draws = s.draws("PLAYERS.PAL");
    assert_eq!(label(&draws, NAME).as_deref(), Some(""));
    assert_eq!(label(&draws, MAP_DIFFICULTY).as_deref(), Some(""));
    assert_eq!(label(&draws, RATING).as_deref(), Some(""));
    assert_eq!(frame(&draws, "scnrvict.def", [420, 308]), None);
    assert_eq!(frame(&draws, "scnrmpsz.def", [714, 28]), Some(3));
    assert_eq!(flags(&draws), vec![(460, 0), (475, 2), (640, 1)]);
    assert_eq!(chosen(&draws), Some(1));
    // All shows the map again.
    click(&mut s, [371, 68]);
    assert_eq!(label(&s.draws("PLAYERS.PAL"), RATING).as_deref(), Some("100%"));
    // The client's: frame 4, no flags (CLT-Q115 (b), CLT-Q139 (b)).
    let mut s = shown(vec![map("Arrogance.h3m", "Arrogance")], Bugs::Fixed);
    click(&mut s, [324, 68]);
    let draws = s.draws("PLAYERS.PAL");
    assert_eq!(frame(&draws, "scnrmpsz.def", [714, 28]), Some(4));
    assert!(flags(&draws).is_empty());
    // No map at all: the original's S, no flags, no choice shown until a press (whose rating
    // stays hidden).
    let mut s = scenario_of(Vec::new(), Bugs::Original);
    let draws = s.draws("PLAYERS.PAL");
    assert_eq!(frame(&draws, "scnrmpsz.def", [714, 28]), Some(0));
    assert!(flags(&draws).is_empty());
    assert_eq!(chosen(&draws), None);
    click(&mut s, [585, 478]);
    let draws = s.draws("PLAYERS.PAL");
    assert_eq!(chosen(&draws), Some(2));
    assert_eq!(label(&draws, RATING).as_deref(), Some(""));
}

#[test]
fn the_teams_dialog_over_an_empty_list() {
    // The original's after a filter: the last map's teams (red and tan, blue).
    let mut s = shown(vec![map("Arrogance.h3m", "Arrogance")], Bugs::Original);
    click(&mut s, [324, 68]);
    assert!(s.right([600, 414]).changed);
    let (labels, flags) = teams_drawn(&s.box_draws("PLAYERS.PAL"));
    assert_eq!(labels.len(), 3);
    assert_eq!(flags.len(), 3);
    s.right_release();
    // With no map at all, the reset game's: one team of the eight colours.
    let mut s = scenario_of(Vec::new(), Bugs::Original);
    assert!(s.right([600, 414]).changed);
    let (labels, flags) = teams_drawn(&s.box_draws("PLAYERS.PAL"));
    assert_eq!(labels.len(), 2);
    let colours: Vec<u32> = flags.iter().map(|&(c, _, _)| c).collect();
    assert_eq!(colours, (0..8).collect::<Vec<_>>());
    // The client's: none (CLT-Q139 (b)).
    let mut s = scenario_of(Vec::new(), Bugs::Fixed);
    assert_eq!(s.right([600, 414]), Did::default());
    assert!(s.box_draws("PLAYERS.PAL").is_empty());
}

#[test]
fn begin_over_an_empty_list_shows_its_message() {
    // With a map Begin does nothing yet (T6).
    let mut s = scenario(Bugs::Original);
    assert_eq!(click(&mut s, [497, 555]).ended, None);
    assert!(s.box_draws("PLAYERS.PAL").is_empty());
    // The original's with no map at all: nothing (CLT-Q114).
    let mut none = scenario_of(Vec::new(), Bugs::Original);
    click(&mut none, [497, 555]);
    assert!(none.box_draws("PLAYERS.PAL").is_empty());
    assert_eq!(none.context(), Context::Scenario);
    // After a filter emptied it: GENRLTXT 530 in an OK box, the cursor showing, its keys.
    let adventure = Hovers {
        arrow: Arrow::Adventure,
        ..Hovers::default()
    };
    let mut s = shown(vec![map("Arrogance.h3m", "Arrogance")], Bugs::Original).opened(adventure);
    click(&mut s, [514, 91]);
    click(&mut s, [324, 68]);
    assert!(click(&mut s, [497, 555]).changed);
    assert!(!s.box_draws("PLAYERS.PAL").is_empty());
    assert_eq!(s.context(), Context::Question);
    assert!(s.cursor());
    // The box's own loop shows the default arrow, the screen's back after it.
    assert_eq!(s.hovers().arrow, Arrow::Default);
    // Nothing else takes an input meanwhile: the screen's keys, a right press.
    assert_eq!(s.key(Some(Command::Dialog(BACK)), "Escape"), Did::default());
    assert_eq!(s.right([600, 414]), Did::default());
    assert_eq!(s.right_release(), Did::default());
    assert_eq!(s.press([10, 10]), Did::default());
    // Enter's OK closes it at the key's release, the screen drawn again.
    let did = s.key(Some(Command::Message(message::Key::Ok)), "Return");
    assert!(did.changed && did.sample);
    assert_eq!(s.key_up("Escape"), Did::default());
    assert!(s.key_up("Return").changed);
    assert!(s.box_draws("PLAYERS.PAL").is_empty());
    assert_eq!(s.context(), Context::Scenario);
    assert_eq!(s.hovers().arrow, Arrow::Adventure);
    // The OK pressed by the mouse: off it and back, then released over it; or the right release
    // while the mouse holds it (CLT-Q72 (a)).
    click(&mut s, [497, 555]);
    let ok = ok_button(&s);
    assert!(s.press(ok).changed);
    assert!(s.moved([10, 10]).changed);
    assert!(s.moved(ok).changed);
    assert!(s.release().changed);
    assert!(s.box_draws("PLAYERS.PAL").is_empty());
    click(&mut s, [497, 555]);
    s.press(ok);
    assert!(s.right_release().changed);
    assert!(s.box_draws("PLAYERS.PAL").is_empty());
    // The client's with no map at all (CLT-Q114 (b)); none without the text.
    let mut fixed = scenario_of(Vec::new(), Bugs::Fixed);
    assert!(click(&mut fixed, [497, 555]).changed);
    assert_eq!(fixed.context(), Context::Question);
    let mut silent = scenario_of(Vec::new(), Bugs::Fixed);
    silent.texts.no_maps = None;
    click(&mut silent, [497, 555]);
    assert_eq!(silent.context(), Context::Scenario);
}

/// The OK button's middle in the open box.
fn ok_button(s: &Scenario) -> [i32; 2] {
    let Over::Notice(open) = &s.over else {
        panic!("no box open");
    };
    let dialog = &open.message.dialog;
    dialog
        .items
        .iter()
        .find_map(|item| match item {
            h3_ui::ui::Item::Button(b) if b.id == message::OK => {
                let ([x, y], [w, h]) = item.area()?;
                Some([dialog.place[0] + x + w / 2, dialog.place[1] + y + h / 2])
            }
            _ => None,
        })
        .unwrap()
}
