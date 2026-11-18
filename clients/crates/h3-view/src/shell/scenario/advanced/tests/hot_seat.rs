//! The hot seat's advanced options as they open (E27 M3 T4a): a human record a name, each colour a
//! human may play to the next record without one, the names on their rows, the handicap a caption
//! button, every bonus's arrows, the list of the maps two humans may play and the flags' strip from
//! the first player's colour, as `adv-hotseat` and the panels of `adv-hotseat-rows` and
//! `adv-hotseat-two` recorded them.

use super::super::super::{ALLIES, ENEMIES};
use super::*;
use crate::shell::menu::Hovers;

/// The names `A`, `B`, … of `n` players.
fn names(n: u8) -> Vec<String> {
    (0..n).map(|k| String::from(char::from(b'A' + k))).collect()
}

/// The hot seat's screen over `entries` with `n` players, its options shown, in the profile `bugs`.
fn hot_seat_in(entries: Vec<Entry>, n: u8, bugs: Bugs) -> Scenario {
    let mut s = with_options_in(entries, bugs).opened_hot_seat(Hovers::default(), &names(n));
    assert!(s.hot_seat());
    assert!(click(&mut s, [514, 519]));
    assert!(s.advanced_showing());
    s
}

/// The hot seat's screen over `entries` with `n` players, its options shown.
fn hot_seat(entries: Vec<Entry>, n: u8) -> Scenario {
    hot_seat_in(entries, n, Bugs::Fixed)
}

/// Warlords: eight colours, each [`open`].
fn warlords() -> Entry {
    map_of("Warlords.h3m", [Some(open()); 8])
}

/// The names drawn on each row, `rows` of them.
fn row_names(s: &Scenario, rows: i32) -> Vec<String> {
    let draws = s.draws("P");
    (0..rows)
        .flat_map(|r| texts_at(&draws, [62, 130 + dy(r)]))
        .collect()
}

#[test]
fn each_name_takes_the_next_colour_a_human_may_play() {
    // `adv-hotseat` players-2 to players-8: the names on the first rows, the computers' after.
    for n in 2..=8u8 {
        let s = hot_seat(vec![warlords()], n);
        let mut want: Vec<String> = names(n)
            .into_iter()
            .flat_map(|name| [name.clone(), name])
            .collect();
        for _ in n..8 {
            want.extend([String::from("Computer"), String::from("Computer")]);
        }
        assert_eq!(row_names(&s, 8), want, "{n}");
        let records = s.records().unwrap();
        for c in 0..8u8 {
            let held = (c < n).then_some(c);
            assert_eq!(records[usize::from(c)].colour, held, "{n} {c}");
        }
    }
}

#[test]
fn a_colour_only_the_computer_may_play_is_passed_over() {
    // Blue the computer's alone: A red, B purple; blue's and teal's rows the computer's.
    let mut map = arrogance();
    map.seats[1].human = false;
    let s = hot_seat(vec![map], 2);
    assert_eq!(
        row_names(&s, 4),
        [
            "A", "A", "Computer", "Computer", "B", "B", "Computer", "Computer"
        ]
    );
    let records = s.records().unwrap();
    assert_eq!(
        records.map(|r| r.colour),
        [Some(0), None, None, None, None, Some(5), None, None]
    );
    // More names than colours: the last has none (none placed on a row).
    let s = hot_seat(vec![arrogance()], 5);
    assert_eq!(row_names(&s, 4), ["A", "A", "B", "B", "C", "C", "D", "D"]);
}

#[test]
fn the_handicap_is_a_caption_button_taking_the_mouse() {
    // `adv-hotseat-rows` panel: each row's `adopb2%c.def` frame 0 with "None" in tiny colour 4,
    // centred, one pixel up (the dialog's items, then the row's draw again); the single
    // player's has no caption.
    let s = hot_seat(vec![arrogance()], 3);
    let draws = s.draws("P");
    assert_eq!(frames(&draws, "adopb2r.def"), [([110, 151], 0); 2]);
    let mut captions: Vec<_> = draws
        .iter()
        .filter_map(|d| match d {
            Draw::Label {
                item, x, y, text, ..
            } if item.font == "tiny.fnt" && text == "None" => {
                Some((item.id, [*x, *y], item.size, item.colour, item.align))
            }
            _ => None,
        })
        .collect();
    captions.sort_unstable();
    captions.dedup();
    let want: Vec<_> = [(0u16, 0), (1, 1), (5, 2), (6, 3)]
        .into_iter()
        .map(|(c, r)| (HANDICAP + c, [110, 150 + dy(r)], [50, 24], 4, 5))
        .collect();
    assert_eq!(captions, want);
    assert!(s.takes(HANDICAP));
    let single = shown(vec![arrogance()]);
    assert!(!single.takes(HANDICAP));
    let draws = single.draws("P");
    assert!(
        !draws
            .iter()
            .any(|d| matches!(d, Draw::Label { text, .. } if text == "None"))
    );
}

#[test]
fn every_colour_s_bonus_and_each_player_s_hero_may_change() {
    // Arrogance's towns Random: every row's town and bonus arrows, no hero's; with a town fixed
    // (red's only Castle) a human's hero arrows show, a computer's not.
    let s = hot_seat(vec![arrogance()], 2);
    let rights = frames(&s.draws("P"), "adoprta.def");
    for r in 0..4 {
        assert!(rights.contains(&([225, 133 + dy(r)], 1)), "{r}");
        assert!(rights.contains(&([377, 133 + dy(r)], 1)), "{r}");
    }
    assert!(!rights.iter().any(|&(at, _)| at[0] == 301));
    let mut map = arrogance_offered();
    for c in [0, 1, 5] {
        map.seats[c].factions = 1;
        map.seats[c].random_town = false;
    }
    map.heroes.offers[1][0] = vec![3, 4];
    map.heroes.offers[5][0] = vec![5];
    let s = hot_seat(vec![map], 2);
    let rights = frames(&s.draws("P"), "adoprta.def");
    let mut heroes: Vec<i32> = rights
        .iter()
        .filter(|&&(at, _)| at[0] == 301)
        .map(|&(at, _)| (at[1] - 133) / 50)
        .collect();
    heroes.sort_unstable();
    heroes.dedup();
    assert_eq!(heroes, [0, 1]);
    // A computer's fixed town keeps its bonus's arrows too (single: a fixed town's).
    assert!(rights.contains(&([377, 133 + dy(2)], 1)));
}

#[test]
fn the_names_take_no_press() {
    // `scenario_name_click` 0x58B620: single only.
    let mut s = hot_seat(vec![arrogance()], 2);
    assert!(!click(&mut s, [110, 138]));
    assert!(!s.typing());
}

#[test]
fn the_list_holds_the_maps_two_humans_may_play() {
    // `scenario_list_build` 0x582D53: the record's +8 at least 2. The kept map (Arrogance) is
    // left out, so the first listed is selected.
    let mut lone = arrogance();
    for c in [1, 5, 6] {
        lone.seats[c].human = false;
    }
    let mut other = warlords();
    other.name = String::from("Warlords");
    let s = with_options(vec![lone.clone(), other.clone()])
        .opened_hot_seat(Hovers::default(), &names(2));
    assert_eq!(s.entries(), [other]);
    assert_eq!(s.entry().map(|e| e.file.as_str()), Some("Warlords.h3m"));
    // Single lists both.
    let single = with_options(vec![lone, warlords()]).opened(Hovers::default());
    assert_eq!(single.entries().len(), 2);
}

#[test]
fn the_flags_strip_reads_the_first_player_s_colour() {
    // Red the computer's alone, blue's team with purple: the first player (A) takes blue, so
    // blue and purple are allies, red and teal enemies, before and after the options show.
    let mut map = arrogance();
    map.seats[0].human = false;
    map.teams = [0, 1, 2, 3, 4, 1, 6, 7];
    let flags = |s: &Scenario| {
        let shown = |id: u16| !s.shown.state(id).hidden;
        let frame = |id: u16| s.shown.frames.get(&id).copied();
        let allies: Vec<_> = (ALLIES..ALLIES + 8)
            .filter(|&i| shown(i))
            .map(frame)
            .collect();
        let enemies: Vec<_> = (ENEMIES..ENEMIES + 8)
            .filter(|&i| shown(i))
            .map(frame)
            .collect();
        (allies, enemies)
    };
    let want = (vec![Some(1), Some(5)], vec![Some(0), Some(6)]);
    let s = with_options(vec![map.clone()]).opened_hot_seat(Hovers::default(), &names(2));
    assert_eq!(flags(&s), want);
    let s = hot_seat(vec![map], 2);
    assert_eq!(flags(&s), want);
}
