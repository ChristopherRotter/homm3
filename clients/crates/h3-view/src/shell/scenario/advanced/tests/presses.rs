//! The hot seat's presses (E27 M3 T4b): the handicap's, the flags' (the original's cycle as
//! `adv-hotseat-rows`, `adv-hotseat-two` and `adv-hotseat-begin` recorded it, the client's
//! exchange of CLT-Q94), a hero another player holds passed over, and Begin's messages.

use super::hot_seat::{hot_seat, hot_seat_in, names};
use super::*;
use crate::message::Key;
use crate::shell::keys::Command;
use crate::shell::menu::Hovers;

/// Row `r`'s flag.
const fn flag(r: i32) -> [i32; 2] {
    [35, 155 + dy(r)]
}

/// Row `r`'s handicap.
const fn handicap(r: i32) -> [i32; 2] {
    [135, 163 + dy(r)]
}

/// Begin.
const BEGIN: [i32; 2] = [497, 555];

/// The name on each of `colours`' rows.
fn players(s: &Scenario, colours: &[u8]) -> Vec<String> {
    colours
        .iter()
        .map(|&c| s.shown.texts[&(PLAYER + u16::from(c))].clone())
        .collect()
}

/// A Warm and Familiar Place: red and green, each [`open`].
fn warm() -> Entry {
    let mut seats = [None; 8];
    for c in [0, 3] {
        seats[c] = Some(open());
    }
    map_of("A Warm and Familiar Place.h3m", seats)
}

/// The text of the box Begin shows ("Unplaced", "Too few"), none without one.
fn notice(s: &Scenario) -> Option<String> {
    s.box_draws("P").iter().find_map(|d| match d {
        Draw::Label { text, .. } if text == "Unplaced" || text == "Too few" => Some(text.clone()),
        _ => None,
    })
}

/// Begin clicked: the box it shows (closed again by Enter's OK).
fn begin(s: &mut Scenario) -> Option<String> {
    click(s, BEGIN);
    let shown = notice(s);
    if shown.is_some() {
        s.key(None, Some(Command::Message(Key::Ok)), "Return");
        assert!(s.key_up("Return").changed);
        assert_eq!(notice(s), None);
    }
    shown
}

#[test]
fn the_handicap_steps_through_its_three_names() {
    // `adv-hotseat-rows` handicap-1 to handicap-3: Mild, Severe, back to None; the record's too.
    for bugs in [Bugs::Original, Bugs::Fixed] {
        let mut s = hot_seat_in(vec![arrogance()], 3, bugs);
        for (k, want) in ["Mild", "Severe", "None"].into_iter().enumerate() {
            assert!(click(&mut s, handicap(0)), "{bugs:?} {k}");
            assert_eq!(s.shown.texts[&HANDICAP], want, "{bugs:?} {k}");
            let handicap = s.records().unwrap()[0].handicap;
            assert_eq!(handicap, u8::try_from(k + 1).unwrap() % 3);
        }
        // A computer's row too.
        assert!(click(&mut s, handicap(3)));
        assert_eq!(s.shown.texts[&(HANDICAP + 6)], "Mild");
        assert_eq!(s.records().unwrap()[6].handicap, 1);
    }
}

#[test]
fn the_handicap_s_caption_follows_the_player_in_the_client_s_profile() {
    // A (red) Mild, then teal's flag: A takes teal, red the computer's. The original's captions
    // stay with the colours' items (red Mild, teal None: CLT-Q144); the client's follow the
    // records (red None, teal Mild).
    let captions = |s: &Scenario| [0u16, 6].map(|c| s.shown.texts[&(HANDICAP + c)].clone());
    for (bugs, want) in [
        (Bugs::Original, ["Mild", "None"]),
        (Bugs::Fixed, ["None", "Mild"]),
    ] {
        let mut s = hot_seat_in(vec![arrogance()], 3, bugs);
        click(&mut s, handicap(0));
        click(&mut s, flag(3));
        assert_eq!(players(&s, &[0, 6]), ["Computer", "A"], "{bugs:?}");
        assert_eq!(captions(&s), want, "{bugs:?}");
        assert_eq!(s.records().unwrap()[6].handicap, 1);
    }
}

#[test]
fn the_original_passes_a_colour_on_as_recorded() {
    let arrogance_rows = [0, 1, 5, 6];
    let mut s = hot_seat_in(vec![arrogance()], 3, Bugs::Original);
    assert_eq!(players(&s, &arrogance_rows), ["A", "B", "C", "Computer"]);
    // `adv-hotseat-rows`: teal (no holder, everyone placed): the first player takes it, leaving
    // red; teal again: A back to red, B (after A) takes teal, leaving blue; blue (no holder,
    // everyone placed): A takes it, leaving red; red: A takes it, leaving blue.
    let steps = [
        (3, ["Computer", "B", "C", "A"]),
        (3, ["A", "Computer", "C", "B"]),
        (1, ["Computer", "A", "C", "B"]),
        (0, ["A", "Computer", "C", "B"]),
    ];
    for (k, (row, want)) in steps.into_iter().enumerate() {
        assert!(click(&mut s, flag(row)), "{k}");
        assert_eq!(players(&s, &arrogance_rows), want, "{k}");
    }
    // `adv-hotseat-two`: green, its holder B the last: nobody takes it; red (A's): B takes it;
    // red again: B the last, the colour it left none: both the computer's; green (no holder): the
    // first player without a colour, A.
    let mut s = hot_seat_in(vec![warm()], 2, Bugs::Original);
    let steps = [
        (1, ["A", "Computer"]),
        (0, ["B", "Computer"]),
        (0, ["Computer", "Computer"]),
        (1, ["Computer", "A"]),
    ];
    for (k, (row, want)) in steps.into_iter().enumerate() {
        click(&mut s, flag(row));
        assert_eq!(players(&s, &[0, 3]), want, "{k}");
    }
    // `adv-hotseat-begin` (three players): red (A's): B takes it, leaving green; red again: B back
    // to green, C takes red; green (B's): C takes it, leaving red; red (no holder): A, the first
    // without one.
    let mut s = hot_seat_in(vec![warm()], 3, Bugs::Original);
    let steps = [
        (0, ["B", "Computer"]),
        (0, ["C", "B"]),
        (1, ["Computer", "C"]),
        (0, ["A", "C"]),
    ];
    for (k, (row, want)) in steps.into_iter().enumerate() {
        click(&mut s, flag(row));
        assert_eq!(players(&s, &[0, 3]), want, "{k}");
    }
}

#[test]
fn the_options_hidden_forget_the_colour_pressed_last() {
    // Teal, the options hidden and shown (the same selection: the rows kept), teal again: no
    // return to red (0x581D5F), B takes teal from A, A left with none.
    let mut s = hot_seat_in(vec![arrogance()], 3, Bugs::Original);
    click(&mut s, flag(3));
    click(&mut s, [514, 519]);
    assert!(!s.advanced_showing());
    click(&mut s, [514, 519]);
    assert!(s.advanced_showing());
    assert_eq!(players(&s, &[0, 1, 5, 6]), ["Computer", "B", "C", "A"]);
    click(&mut s, flag(3));
    assert_eq!(
        players(&s, &[0, 1, 5, 6]),
        ["Computer", "Computer", "C", "B"]
    );
}

#[test]
fn the_client_passes_a_colour_to_the_next_player() {
    // CLT-Q94: red (A's) to B, the two exchanging; again to C (B left with C's none); again past
    // the last to the computer (C none); red (no holder) to the first player, its own (green) the
    // computer's.
    let mut s = hot_seat(vec![warm()], 3);
    assert_eq!(players(&s, &[0, 3]), ["A", "B"]);
    let steps = [
        (0, ["B", "A"]),
        (0, ["C", "A"]),
        (0, ["Computer", "A"]),
        (0, ["A", "Computer"]),
    ];
    for (k, (row, want)) in steps.into_iter().enumerate() {
        assert!(click(&mut s, flag(row)), "{k}");
        assert_eq!(players(&s, &[0, 3]), want, "{k}");
    }
    // A colour only the computer may play takes no press (its flag hidden; the press tests the
    // colour too, 0x58B7FF).
    let mut map = arrogance();
    map.seats[6].human = false;
    let mut s = hot_seat(vec![map], 2);
    click(&mut s, flag(3));
    let map = s.players();
    let (advanced, shown) = (s.advanced.as_mut().unwrap(), &mut s.shown);
    advanced.flag_click(6, &map, shown);
    assert_eq!(
        players(&s, &[0, 1, 5, 6]),
        ["A", "B", "Computer", "Computer"]
    );
}

#[test]
fn the_players_who_move_take_a_random_town_and_hero() {
    // Red's and blue's towns Castle, then red's flag: the taker (B) to Random on red; the
    // original's holder (A) left without a colour keeps its town (+0x24), the client's takes blue
    // (the exchange) with a Random town.
    for (bugs, a) in [
        (Bugs::Original, (None, Some(0))),
        (Bugs::Fixed, (Some(1), None)),
    ] {
        let mut s = hot_seat_in(vec![arrogance()], 2, bugs);
        click(&mut s, [230, 145]);
        click(&mut s, [230, 145 + dy(1)]);
        let towns = |s: &Scenario| s.records().unwrap().map(|r| r.town);
        assert_eq!(towns(&s)[..2], [Some(0), Some(0)]);
        click(&mut s, flag(0));
        assert_eq!(towns(&s)[..2], [None, None], "{bugs:?}");
        let record = s.advanced.as_ref().unwrap().humans[0].record;
        assert_eq!((record.colour, record.town), a, "{bugs:?}");
    }
}

/// Arrogance with red's and blue's towns Castle alone, each offered heroes 3, 4 and 5.
fn castles() -> Entry {
    let mut map = arrogance();
    for c in [0, 1] {
        map.seats[c].factions = 1;
        map.seats[c].random_town = false;
        map.heroes.offers[c][0] = vec![3, 4, 5];
    }
    map
}

#[test]
fn a_hero_another_player_holds_is_passed_over() {
    // `scenario_hero_step` 0x58AF20: A (red) takes hero 3; B (blue) right passes over it to 4;
    // left back over 3 to Random, left again from Random to 5, then 4, then over 3 to Random.
    let mut s = hot_seat(vec![castles()], 2);
    let hero = |s: &Scenario, c: usize| s.records().unwrap()[c].hero;
    click(&mut s, [306, 145]);
    assert_eq!(hero(&s, 0), Some(3));
    click(&mut s, [306, 145 + dy(1)]);
    assert_eq!(hero(&s, 1), Some(4));
    for want in [None, Some(5), Some(4), None] {
        click(&mut s, [245, 145 + dy(1)]);
        assert_eq!(hero(&s, 1), want);
    }
    // A player without a colour (the fifth on four colours) holds nothing.
    let mut s = hot_seat(vec![castles()], 5);
    click(&mut s, [306, 145]);
    s.advanced.as_mut().unwrap().humans[4].record.hero = Some(4);
    click(&mut s, [306, 145 + dy(1)]);
    assert_eq!(hero(&s, 1), Some(4));
}

#[test]
fn a_hero_held_twice_is_stepped_on() {
    // `scenario_heroes_unclash` 0x58BBC0 (a town's arrow) and the showing's loop: colours 7 down
    // to 0, the higher colour's pick stepped right past what is held. The steps never make a
    // clash: set one by hand.
    let clash = |s: &mut Scenario| {
        let advanced = s.advanced.as_mut().unwrap();
        for human in &mut advanced.humans[..2] {
            human.record.hero = Some(3);
        }
    };
    let heroes = |s: &Scenario| s.records().unwrap().map(|r| r.hero)[..2].to_vec();
    // Purple's town arrow.
    let mut s = hot_seat(vec![castles()], 3);
    clash(&mut s);
    click(&mut s, [230, 145 + dy(2)]);
    assert_eq!(heroes(&s), [Some(3), Some(4)]);
    // The options hidden and shown again (the rows kept).
    let mut s = hot_seat(vec![castles()], 3);
    clash(&mut s);
    click(&mut s, [514, 519]);
    click(&mut s, [514, 519]);
    assert_eq!(heroes(&s), [Some(3), Some(4)]);
}

#[test]
fn begin_is_refused_where_a_player_has_no_colour_or_two_are_not_placed() {
    // `adv-hotseat-two`: green's flag leaves A alone placed (83: begin-1); red's gives B red, A
    // left without (the first player: 531); red again leaves both without (531); green places A
    // alone (83). Two placed: nothing yet (the begin is T6's).
    for bugs in [Bugs::Original, Bugs::Fixed] {
        let mut s = hot_seat_in(vec![warm()], 2, bugs);
        assert_eq!(begin(&mut s), None, "{bugs:?}");
        click(&mut s, flag(1));
        // The original's: A alone placed; the client's: green past the last player (B) to the
        // computer, B without a colour.
        let want = match bugs {
            Bugs::Original => "Too few",
            Bugs::Fixed => "Unplaced",
        };
        assert_eq!(begin(&mut s).as_deref(), Some(want), "{bugs:?}");
    }
    let mut s = hot_seat_in(vec![warm()], 2, Bugs::Original);
    let steps = [
        (1, Some("Too few")),
        (0, Some("Unplaced")),
        (0, Some("Unplaced")),
        (1, Some("Too few")),
    ];
    for (k, (row, want)) in steps.into_iter().enumerate() {
        click(&mut s, flag(row));
        assert_eq!(begin(&mut s).as_deref(), want, "{k}");
    }
    // The client's: red past the last player to the computer: A has none (531, as for any named
    // player: CLT-Q109 (b)); the original's 531 only for the first player.
    let mut s = hot_seat(vec![warm()], 2);
    click(&mut s, flag(0));
    click(&mut s, flag(0));
    assert_eq!(players(&s, &[0, 3]), ["Computer", "A"]);
    assert_eq!(begin(&mut s).as_deref(), Some("Unplaced"));
}

#[test]
fn more_players_than_colours_never_begin_in_the_client_s_profile() {
    // Three players on two colours (`adv-hotseat-begin`): the original begins with A and B (C
    // dropped, CLT-Q109's); the client's refuses while C has none (CLT-Q145 (c)), the options
    // shown or not (the rows as the showing will make them).
    let mut s = hot_seat_in(vec![warm()], 3, Bugs::Original);
    assert_eq!(begin(&mut s), None);
    let mut s = hot_seat(vec![warm()], 3);
    assert_eq!(begin(&mut s).as_deref(), Some("Unplaced"));
    let options = || Texts {
        advanced: Some(advanced_texts()),
        ..texts(Bugs::Fixed)
    };
    let layout = parse_layout(LAYOUT).unwrap();
    let opened = |n: u8| {
        Scenario::new(
            layout.scenario.dialog.clone(),
            (
                layout.scenario.list.clone(),
                layout.scenario.description.clone(),
            ),
            options(),
            vec![warm()],
            Bugs::Fixed,
        )
        .opened_hot_seat(Hovers::default(), &names(n))
    };
    let mut s = opened(3);
    assert!(!s.advanced_showing());
    assert_eq!(begin(&mut s).as_deref(), Some("Unplaced"));
    let mut s = opened(2);
    assert_eq!(begin(&mut s), None);
    // Single: the player has its colour.
    let mut s = shown(vec![warm()]);
    assert_eq!(begin(&mut s), None);
}
