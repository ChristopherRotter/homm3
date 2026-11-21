//! The hot seat's load (E27 M4 T5b1; design §41): the screen over saves of maps two humans may
//! play, its panel's buttons and flags, Random Map in the original's profile alone (CLT-Q161 (b))
//! and doing nothing; the advanced options the save's (its choices, no arrows, the handicap taking
//! no press, the duration's bar dimmed and taking no press), the names placed by the save's humans
//! first (CLT-Q160 (a)), as `adv-load` and `adv-load-more` recorded them.

use super::super::advanced::tests::with_options_in;
use super::super::advanced::{FLAG, HANDICAP, HERO_LEFT, PLAYER, TOWN_LEFT};
use super::*;
use crate::shell::look::parse_layout;
use crate::shell::menu::Hovers;

/// A seat a human or the computer may play, every faction at random, its main town generating
/// its hero.
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

/// Arrogance (red, blue, purple, teal) saved as `file`, the save's humans `humans` (by colour),
/// each colour's town its colour's index mod 4 and its hero sixteen times that, the artifact, the
/// duration 20 minutes (7).
fn saved(file: &str, humans: &[usize]) -> Entry {
    let mut map = map("Arrogance.h3m", "Arrogance");
    map.seats = [Seat::default(); 8];
    for c in [0, 1, 5, 6] {
        map.seats[c] = open();
    }
    let mut saved = Saved {
        difficulty: 0,
        time: 1_003_826_100,
        duration: 7,
        ..Saved::default()
    };
    for &c in humans {
        saved.humans[c] = true;
    }
    for (c, record) in saved.records.iter_mut().enumerate() {
        let k = u8::try_from(c % 4).unwrap();
        record.town = Some(k);
        record.hero = Some(16 * k);
        record.bonus = Bonus::Artifact;
    }
    Entry {
        file: file.to_string(),
        saved: Some(saved),
        ..map
    }
}

/// The load mode's screen over `entries` with the advanced options, opened from the hot seat's
/// names `names` in the profile `bugs`.
fn hot_load(entries: Vec<Entry>, names: &[&str], bugs: Bugs) -> Scenario {
    let layout = parse_layout(LAYOUT).unwrap();
    let names: Vec<String> = names.iter().map(|n| (*n).to_string()).collect();
    with_options_in(entries, bugs)
        .loading(&layout.scenario.load_begin, KOLKATA)
        .opened_hot_seat(Hovers::default(), &names)
}

/// The options shown (Show Advanced Options clicked).
fn options(s: &mut Scenario) {
    assert!(click(s, [514, 519]).changed);
    assert!(s.advanced_showing());
}

/// Each colour's row's name, red first, none for a colour without a row.
fn row_names(s: &Scenario) -> Vec<Option<String>> {
    let draws = s.draws("P");
    (0..8u16).map(|c| label(&draws, PLAYER + c)).collect()
}

#[test]
fn a_hot_seat_load_lists_the_saves_two_humans_may_play_with_its_panel() {
    let mut single = map("Single.h3m", "Single");
    single.seats[1].human = false;
    let one = Entry {
        file: String::from("One.GM1"),
        saved: Some(Saved::default()),
        ..single
    };
    for bugs in [Bugs::Original, Bugs::Fixed] {
        let s = hot_load(
            vec![one.clone(), saved("Four.GM4", &[0, 1, 5, 6])],
            &["A", "B"],
            bugs,
        );
        assert!(s.is_load() && s.hot_seat() && s.showing());
        let draws = s.draws("P");
        // The list: the save of a map one human alone may play left out (`adv-load` screen).
        assert_eq!(names(&draws), ["Four.GM4"]);
        assert_eq!(label(&draws, TITLE).as_deref(), Some("Load a game"));
        // Show Saved Games, Show Advanced Options and the flags' strip; Random Map the
        // original's alone (CLT-Q161 (b)).
        assert_eq!(label(&draws, LIST).as_deref(), Some("Show Saved Games"));
        for id in [LIST, ADVANCED, ALLIES_LABEL, ENEMIES_LABEL, TEAMS] {
            assert!(!s.shown.state(id).hidden, "{id}");
        }
        assert_eq!(s.shown.state(RANDOM).hidden, bugs == Bugs::Fixed);
        // Red the first name's (the save's first human): its ally, the others its enemies.
        assert_eq!(flags(&draws), [(460, 0), (640, 1), (655, 5), (670, 6)]);
    }
}

#[test]
fn the_flags_follow_the_save_s_first_human_before_the_rows_are_made() {
    // Blue the save's first human: A takes it, so blue is the allies' flag.
    let s = hot_load(vec![saved("Blue.GM2", &[1, 5])], &["A", "B"], Bugs::Fixed);
    assert_eq!(
        flags(&s.draws("P")),
        [(460, 1), (640, 0), (655, 5), (670, 6)]
    );
}

#[test]
fn random_map_does_nothing_in_a_load() {
    // `adv-load` random, random-key: the original's button and key change nothing.
    let mut s = hot_load(
        vec![saved("Four.GM4", &[0, 1, 5, 6])],
        &["A", "B"],
        Bugs::Original,
    );
    let before = s.draws("P");
    click(&mut s, [514, 115]);
    s.key(None, Some(Command::Dialog(RANDOM)), "R");
    s.key_up("R");
    assert_eq!(s.draws("P"), before);
    assert!(s.showing());
}

#[test]
fn the_rows_are_the_save_s_and_its_humans_take_the_names_first() {
    for bugs in [Bugs::Original, Bugs::Fixed] {
        // `adv-load` advanced: four saved humans under two names, purple and teal the
        // computer's with their saved choices.
        let mut s = hot_load(vec![saved("Four.GM4", &[0, 1, 5, 6])], &["A", "B"], bugs);
        options(&mut s);
        let computer = Some(String::from("Computer"));
        assert_eq!(
            row_names(&s),
            [
                Some(String::from("A")),
                Some(String::from("B")),
                None,
                None,
                None,
                computer.clone(),
                computer,
                None
            ]
        );
        let records = s.records().unwrap();
        assert_eq!(
            records.map(|r| r.colour),
            [Some(0), Some(1), None, None, None, None, None, None]
        );
        assert_eq!(
            records.map(|r| (r.town, r.hero)),
            core::array::from_fn(|c| {
                let k = u8::try_from(c % 4).unwrap();
                (Some(k), Some(16 * k))
            })
        );
        assert!(records.iter().all(|r| r.bonus == Bonus::Artifact));
        // No arrows; the handicap and the duration's bar take no press; the duration the save's,
        // its bar dimmed (`dialog_item_message` 0x5FE54B).
        for c in [0u16, 1, 5, 6] {
            for id in [TOWN_LEFT, HERO_LEFT] {
                assert!(s.shown.state(id + c).hidden, "{id} {c}");
            }
            assert!(!s.takes(HANDICAP + c));
            assert!(s.takes(FLAG + c));
        }
        assert_eq!(s.duration(), Some(7));
        let draws = s.draws("P");
        assert!(draws.iter().any(|d| matches!(
            d,
            Draw::Darken {
                rect: [58, 557, _, _]
            }
        )));
        click(&mut s, [66, 565]);
        assert_eq!(s.duration(), Some(7));
        // Load hands on the names by the colour each holds.
        let mut seated: [Option<String>; 8] = Default::default();
        seated[0] = Some(String::from("A"));
        seated[1] = Some(String::from("B"));
        assert_eq!(s.seated(), Some(seated));
    }
}

#[test]
fn a_name_past_the_save_s_humans_takes_the_next_colour_a_human_may_play() {
    // `adv-load-more` advanced: the save's one human (red) to A, B on blue (a computer's in the
    // save); a saved human no human may play any longer is passed over (a colour lost).
    let mut s = hot_load(vec![saved("Week_one.GM1", &[0])], &["A", "B"], Bugs::Fixed);
    options(&mut s);
    let records = s.records().unwrap();
    assert_eq!(records[0].colour, Some(0));
    assert_eq!(records[1].colour, Some(1));
    let mut lost = saved("Lost.GM2", &[0, 5]);
    lost.seats[0].human = false;
    let mut s = hot_load(vec![lost], &["A", "B", "C"], Bugs::Fixed);
    options(&mut s);
    let held = s.records().unwrap().map(|r| r.colour);
    assert_eq!(
        held,
        [None, Some(1), None, None, None, Some(5), Some(6), None]
    );
    let names = s.seated().unwrap();
    assert_eq!(
        [&names[5], &names[1], &names[6]].map(|n| n.as_deref()),
        [Some("A"), Some("B"), Some("C")]
    );
}

#[test]
fn only_the_hot_seat_s_load_with_a_save_selected_hands_on_names() {
    // A single load, a new hot seat, a hot seat's load over no save: none.
    assert_eq!(load(Bugs::Fixed).seated(), None);
    let new = with_options_in(vec![saved("x.h3m", &[])], Bugs::Fixed)
        .opened_hot_seat(Hovers::default(), &[String::from("A"), String::from("B")]);
    assert_eq!(new.seated(), None);
    let none = hot_load(Vec::new(), &["A", "B"], Bugs::Fixed);
    assert_eq!(none.seated(), None);
    // A load's options over no options' texts hand on nothing either.
    let layout = parse_layout(LAYOUT).unwrap();
    let bare = scenario_of(vec![saved("Four.GM4", &[0, 1])], Bugs::Fixed)
        .loading(&layout.scenario.load_begin, KOLKATA)
        .opened_hot_seat(Hovers::default(), &[String::from("A"), String::from("B")]);
    assert_eq!(bare.seated(), None);
}
