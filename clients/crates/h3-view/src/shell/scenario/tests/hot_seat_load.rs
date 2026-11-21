//! The hot seat's load (E27 M4 T5b1; design §41): the screen over saves of maps two humans may
//! play, its panel's buttons and flags, Random Map in the original's profile alone (CLT-Q161 (b))
//! and doing nothing; the advanced options the save's (its choices, no arrows, the handicap taking
//! no press, the duration's bar dimmed and taking no press), the names placed by the save's humans
//! first (CLT-Q160 (a)), as `adv-load` and `adv-load-more` recorded them; a flag's press moving
//! a name onto its colour's saved choices, among the save's humans in the client's profile
//! (CLT-Q110 (b)), and Random Map's whole redraw (E27 M4 T5b2; design §43).

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
    // More names than colours a human may play: the last keeps none (its record the save's
    // nowhere).
    let five = ["A", "B", "C", "D", "E"];
    let mut s = hot_load(vec![saved("Four.GM4", &[0, 1, 5, 6])], &five, Bugs::Fixed);
    options(&mut s);
    let names = s.seated().unwrap();
    assert_eq!(names.iter().flatten().count(), 4);
    assert!(!names.iter().flatten().any(|n| n == "E"));
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

/// Row `r`'s flag.
const fn flag(r: i32) -> [i32; 2] {
    [35, 155 + 50 * r]
}

/// Each colour's (town, hero) the rows hold, red first.
fn choices(s: &Scenario) -> [(Option<u8>, Option<u8>); 8] {
    s.records().unwrap().map(|r| (r.town, r.hero))
}

#[test]
fn a_flag_s_press_moves_the_name_onto_the_colour_s_saved_choices() {
    let saved_choices: [(Option<u8>, Option<u8>); 8] = core::array::from_fn(|c| {
        let k = u8::try_from(c % 4).unwrap();
        (Some(k), Some(16 * k))
    });
    let computer = || Some(String::from("Computer"));
    let a = || Some(String::from("A"));
    let b = || Some(String::from("B"));
    for bugs in [Bugs::Original, Bugs::Fixed] {
        let mut s = hot_load(vec![saved("Four.GM4", &[0, 1, 5, 6])], &["A", "B"], bugs);
        options(&mut s);
        let opening = s.draws("P");
        // `adv-load` flag-3: purple's press moves A there, red the computer's; every row the
        // save's choices of its colour, whoever holds it (both are the save's humans:
        // CLT-Q110 (b) takes it).
        assert!(click(&mut s, flag(2)).changed);
        assert_eq!(
            row_names(&s),
            [computer(), b(), None, None, None, a(), computer(), None]
        );
        assert_eq!(s.records().unwrap()[5].colour, Some(5));
        assert_eq!(choices(&s), saved_choices, "{bugs:?}");
        // flag-1: red's press gives the opening's rows back.
        assert!(click(&mut s, flag(0)).changed);
        assert_eq!(
            row_names(&s),
            [a(), b(), None, None, None, computer(), computer(), None]
        );
        assert_eq!(choices(&s), saved_choices, "{bugs:?}");
        let mut seated: [Option<String>; 8] = Default::default();
        seated[0] = a();
        seated[1] = b();
        assert_eq!(s.seated(), Some(seated));
        // random: in the original's profile the flags' strip the presses left stale until Random
        // Map's press, which changes nothing but the whole redraw (0x586FA7); the client's draws
        // whole at every press.
        s.moved([790, 300]);
        assert_eq!(s.draws("P"), opening, "{bugs:?}");
        assert_eq!(
            s.stale_draws("P").is_some(),
            bugs == Bugs::Original,
            "{bugs:?}"
        );
        click(&mut s, [514, 115]);
        assert_eq!(s.draws("P"), opening, "{bugs:?}");
        assert!(s.stale_draws("P").is_none(), "{bugs:?}");
    }
}

#[test]
fn the_client_moves_a_load_s_colours_among_the_save_s_humans_only() {
    // `adv-load-more`'s save: red its one human, B on blue (a computer's in the save). A press on
    // purple (the computer's in the save): the original's moves A there, the client's takes none
    // (CLT-Q110 (b)); a press on red (the save's) the client's passes on.
    let mut s = hot_load(
        vec![saved("Week_one.GM1", &[0])],
        &["A", "B"],
        Bugs::Original,
    );
    options(&mut s);
    click(&mut s, flag(2));
    assert_eq!(s.records().unwrap()[5].colour, Some(5));
    let mut s = hot_load(vec![saved("Week_one.GM1", &[0])], &["A", "B"], Bugs::Fixed);
    options(&mut s);
    let before = s.records().unwrap();
    click(&mut s, flag(2));
    assert_eq!(s.records().unwrap(), before);
    click(&mut s, flag(0));
    let held = s.records().unwrap().map(|r| r.colour);
    assert_eq!(held[0], Some(0));
    assert_eq!(held[1], Some(1));
    let names = s.seated().unwrap();
    assert_eq!(
        [&names[0], &names[1]].map(|n| n.as_deref()),
        [Some("B"), Some("A")]
    );
}

/// Delete's question about the selected save, answered OK by Enter.
fn delete_selected(s: &mut Scenario) {
    s.key(None, Some(Command::Delete), "Delete");
    s.key_up("Delete");
    s.key(None, Some(Command::Message(message::Key::Ok)), "Return");
    s.key_up("Return");
}

/// E27 M4 T6 (the review's B6 = C3): leaving a hot-seat load takes the deleted save alone out of
/// the load's list, not the hot seat's filtered list: the next single load lists the save of a
/// map one human alone may play still.
#[test]
fn a_hot_seat_load_s_delete_leaves_the_single_load_its_own_list() {
    let mut single = map("Single.h3m", "Single");
    single.seats[1].human = false;
    let one = Entry {
        file: String::from("One.GM1"),
        saved: Some(Saved::default()),
        ..single
    };
    let layout = parse_layout(LAYOUT).unwrap();
    let saves = vec![
        one,
        saved("Four.GM4", &[0, 1, 5, 6]),
        saved("Two.GM2", &[0, 1]),
    ];
    for bugs in [Bugs::Original, Bugs::Fixed] {
        let mut base = with_options_in(saves.clone(), bugs).loading(&layout.scenario.load_begin, 0);
        let names = [String::from("A"), String::from("B")];
        let mut hot = base.opened_hot_seat(Hovers::default(), &names);
        assert_eq!(names_of(&hot), ["Four.GM4", "Two.GM2"]);
        delete_selected(&mut hot);
        assert_eq!(hot.take_deleted().as_deref(), Some("Four.GM4"));
        assert_eq!(names_of(&hot), ["Two.GM2"]);
        base.keep(&hot);
        let single = base.opened(Hovers::default());
        assert_eq!(names_of(&single), ["One.GM1", "Two.GM2"], "{bugs:?}");
        let hot = base.opened_hot_seat(Hovers::default(), &names);
        assert_eq!(names_of(&hot), ["Two.GM2"], "{bugs:?}");
    }
}

/// The rows' files' names drawn.
fn names_of(s: &Scenario) -> Vec<String> {
    names(&s.draws("P"))
}

/// The names Load hands on, by colour, red's and blue's.
fn red_blue(s: &Scenario) -> [Option<String>; 2] {
    let names = s.seated().unwrap();
    [names[0].clone(), names[1].clone()]
}

/// E27 M4 T6 (the review's C1, item 41): in the client's profile a load's press passes past the
/// last name on to the first (always an exchange), never to the computer: `adv-load`'s Blue (the
/// last name's) gives A blue and B red; a one-human save's red pressed twice gives each name its
/// colour back.
#[test]
fn a_load_s_press_past_the_last_name_hands_it_to_the_first() {
    let [a, b] = [Some(String::from("A")), Some(String::from("B"))];
    let mut s = hot_load(
        vec![saved("Four.GM4", &[0, 1, 5, 6])],
        &["A", "B"],
        Bugs::Fixed,
    );
    options(&mut s);
    assert!(click(&mut s, flag(1)).changed);
    assert_eq!(red_blue(&s), [b.clone(), a.clone()]);
    assert_eq!(s.seated().unwrap().iter().flatten().count(), 2);
    // `adv-load-more`'s save: red its one human, B on blue (a computer's in the save).
    let mut s = hot_load(vec![saved("Week_one.GM1", &[0])], &["A", "B"], Bugs::Fixed);
    options(&mut s);
    click(&mut s, flag(0));
    assert_eq!(red_blue(&s), [b.clone(), a.clone()]);
    click(&mut s, flag(0));
    assert_eq!(red_blue(&s), [a, b]);
    // Each row the save's choices of its colour, whoever holds it.
    assert_eq!(s.records().unwrap()[0].town, Some(0));
}

/// E27 M4 T6 (CLT-Q171 (a), the review's C2): the client's hot-seat Load refuses as Begin does
/// where a name holds no colour (GENRLTXT 531: more names than colours a human may play); the
/// original's ends the screen (its begin after a load drops the name, *inferred*).
#[test]
fn a_hot_seat_load_refuses_a_name_without_a_colour_in_the_client_s_profile() {
    let five = ["A", "B", "C", "D", "E"];
    let four = || vec![saved("Four.GM4", &[0, 1, 5, 6])];
    let mut s = hot_load(four(), &five, Bugs::Fixed);
    let did = click(&mut s, [497, 555]);
    assert_eq!(did.ended, None);
    assert!(s.question().is_some());
    assert_eq!(s.asked, Asked::Told);
    let mut s = hot_load(four(), &five, Bugs::Original);
    assert_eq!(click(&mut s, [497, 555]).ended, Some(Ended::Scenario(true)));
    // Every name placed: Load ends the screen; without the message's text nothing.
    let mut s = hot_load(four(), &["A", "B"], Bugs::Fixed);
    assert_eq!(click(&mut s, [497, 555]).ended, Some(Ended::Scenario(true)));
    let mut s = hot_load(four(), &five, Bugs::Fixed);
    s.texts.unplaced = None;
    let did = click(&mut s, [497, 555]);
    assert_eq!(did.ended, None);
    assert!(s.question().is_none());
}

/// E27 M4 T6 (the review's C4): Delete while the advanced options show. The original takes it
/// (0x587E85 reads no list's showing), the rows left the deleted save's; the client's profile
/// hides the options and shows the list again, Load then handing on names over the save now
/// selected.
#[test]
fn delete_over_the_advanced_options_shows_the_list_again_in_the_client_s_profile() {
    let two = || vec![saved("Four.GM4", &[0, 1, 5, 6]), saved("Two.GM2", &[5, 6])];
    for bugs in [Bugs::Fixed, Bugs::Original] {
        let mut s = hot_load(two(), &["A", "B"], bugs);
        options(&mut s);
        delete_selected(&mut s);
        assert_eq!(s.take_deleted().as_deref(), Some("Four.GM4"));
        let fixed = bugs == Bugs::Fixed;
        assert_eq!(s.advanced_showing(), !fixed, "{bugs:?}");
        assert_eq!(s.showing(), fixed, "{bugs:?}");
        if fixed {
            assert_eq!(names_of(&s), ["Two.GM2"]);
            // Two's saved humans, purple and teal, take the names.
            let names = s.seated().unwrap();
            assert_eq!(
                [&names[5], &names[6]].map(|n| n.as_deref()),
                [Some("A"), Some("B")]
            );
        }
    }
}

/// E27 M4 T6 (the review's C10): Show Advanced Options over no saves (`mp-load` load-closed: the
/// button enabled) shows an empty panel: no rows, nothing handed on.
#[test]
fn the_advanced_options_over_no_saves_show_no_rows() {
    for bugs in [Bugs::Fixed, Bugs::Original] {
        let mut s = hot_load(Vec::new(), &["A", "B"], bugs);
        assert_eq!(s.asked, Asked::NoSaves);
        s.key(None, Some(Command::Message(message::Key::Ok)), "Return");
        s.key_up("Return");
        assert!(s.question().is_none());
        options(&mut s);
        assert_eq!(
            row_names(&s),
            [None, None, None, None, None, None, None, None]
        );
        assert_eq!(s.seated(), None);
        assert_eq!(s.duration(), Some(UNLIMITED));
        // Load stays disabled: nothing.
        assert_eq!(click(&mut s, [497, 555]).ended, None);
    }
}
