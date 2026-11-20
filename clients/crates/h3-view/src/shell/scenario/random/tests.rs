//! The random map tab's tests (E27 M3 T5a): the choices, the tab's state (the selection, the
//! caps), the presses, the own record, and the screen's tab as `adv-random` recorded it.

use alloc::vec;
use alloc::vec::Vec;

use h3_render::Bugs;
use h3_ui::ui::{Draw, Shown};

use super::super::advanced::tests::with_options_in;
use super::super::tests::{map, ok_button, scenario, scenario_of};
use super::super::{
    DESCRIPTION, Did, Entry, LIST, LOSS, MAP_DIFFICULTY, NAME, Over, RANDOM, Scenario, VICTORY,
};
use super::*;
use crate::shell::keys::{Command, Context};
use crate::shell::menu::Hovers;

/// The tab shown with `choices`.
fn tab(choices: Choices) -> (RandomMap, Shown) {
    let mut random = RandomMap {
        choices,
        ..RandomMap::default()
    };
    let mut shown = Shown::default();
    RandomMap::hide_items(&mut shown);
    random.show(&mut shown);
    (random, shown)
}

/// The selected button of the group of `count` from `first`, if one.
fn selected(shown: &Shown, first: u16, count: u16) -> Option<u16> {
    (0..count).find(|&k| shown.state(first + k).selected)
}

/// The disabled buttons of the group of `count` from `first`.
fn disabled(shown: &Shown, first: u16, count: u16) -> Vec<u16> {
    (0..count)
        .filter(|&k| shown.state(first + k).disabled)
        .collect()
}

#[test]
fn the_choices_start_as_the_constructor_s() {
    let c = Choices::default();
    assert_eq!((c.size, c.levels), (72, 2));
    assert_eq!((c.players, c.teams, c.computers), (None, None, Some(0)));
    assert_eq!((c.computer_teams, c.water, c.monsters), (None, None, None));
    // Random players: every colour a human's and anyone's.
    assert_eq!((c.humans(), c.total()), (8, 8));
    let two = Choices {
        players: Some(2),
        computers: Some(3),
        ..c
    };
    assert_eq!((two.humans(), two.total()), (2, 5));
    let random = Choices {
        computers: None,
        ..two
    };
    assert_eq!(random.total(), 8);
}

#[test]
fn the_tab_s_state_selects_each_choice() {
    let (_, shown) = tab(Choices::default());
    assert_eq!(selected(&shown, 281, 4), Some(1));
    assert!(shown.state(LEVELS).selected);
    assert_eq!(selected(&shown, PLAYERS, 9), Some(8));
    assert_eq!(selected(&shown, TEAMS, 9), Some(8));
    assert_eq!(selected(&shown, COMPUTERS, 9), Some(0));
    assert_eq!(selected(&shown, COMPUTER_TEAMS, 8), Some(7));
    assert_eq!(selected(&shown, WATER, 4), Some(3));
    assert_eq!(selected(&shown, MONSTERS, 4), Some(3));
    // Random players count 8 for the teams (none disabled), 1 for the computer only players
    // (7 of them); no computer only players, so none of their teams but 0.
    assert!(disabled(&shown, TEAMS, 9).is_empty());
    assert!(disabled(&shown, COMPUTERS, 9).is_empty());
    assert_eq!(disabled(&shown, COMPUTER_TEAMS, 8), [1, 2, 3, 4, 5, 6]);
    // Every item shown.
    assert!(RandomMap::ids().all(|id| !shown.state(id).hidden));
    let (_, shown) = tab(Choices {
        size: 144,
        levels: 1,
        water: Some(2),
        monsters: Some(0),
        ..Choices::default()
    });
    assert_eq!(selected(&shown, 281, 4), Some(3));
    assert!(!shown.state(LEVELS).selected);
    assert_eq!(selected(&shown, WATER, 4), Some(2));
    assert_eq!(selected(&shown, MONSTERS, 4), Some(0));
}

#[test]
fn the_counts_cap_each_other_and_a_choice_past_its_cap_shows_the_cap() {
    // Four players: their teams from 4 on disabled, the computer only players above 4; five teams
    // shown as 4 (a disabled button selected), seven computer players as 4.
    let (_, shown) = tab(Choices {
        players: Some(4),
        teams: Some(5),
        computers: Some(7),
        computer_teams: Some(6),
        ..Choices::default()
    });
    assert_eq!(selected(&shown, PLAYERS, 9), Some(3));
    assert_eq!(disabled(&shown, TEAMS, 9), [4, 5, 6, 7]);
    assert_eq!(selected(&shown, TEAMS, 9), Some(4));
    assert_eq!(disabled(&shown, COMPUTERS, 9), [5, 6, 7]);
    assert_eq!(selected(&shown, COMPUTERS, 9), Some(4));
    assert_eq!(disabled(&shown, COMPUTER_TEAMS, 8), [4, 5, 6]);
    assert_eq!(selected(&shown, COMPUTER_TEAMS, 8), Some(4));
    // One player: no teams but 0, every computer only player; Random computer players count as
    // their cap (7) for their teams.
    let (_, shown) = tab(Choices {
        players: Some(1),
        computers: None,
        ..Choices::default()
    });
    assert_eq!(disabled(&shown, TEAMS, 9), [1, 2, 3, 4, 5, 6, 7]);
    assert!(disabled(&shown, COMPUTERS, 9).is_empty());
    assert!(disabled(&shown, COMPUTER_TEAMS, 8).is_empty());
    // Eight players: no computer only player but 0, none of their teams.
    let (_, shown) = tab(Choices {
        players: Some(8),
        ..Choices::default()
    });
    assert!(disabled(&shown, TEAMS, 9).is_empty());
    assert_eq!(disabled(&shown, COMPUTERS, 9), [1, 2, 3, 4, 5, 6, 7]);
    // A hidden tab's state is left as it was.
    let (mut random, mut shown) = tab(Choices::default());
    random.hide(&mut shown);
    random.choices.size = 36;
    random.states(&mut shown);
    assert_eq!(selected(&shown, 281, 4), Some(1));
    assert!(RandomMap::ids().all(|id| shown.state(id).hidden));
}

#[test]
fn a_press_sets_its_choice_and_says_whether_the_record_is_built_again() {
    let mut random = RandomMap::default();
    let presses = [
        (283, Pressed::Chosen),
        (LEVELS, Pressed::Chosen),
        (PLAYERS + 2, Pressed::Counted),
        (TEAMS + 1, Pressed::Counted),
        (COMPUTERS + 4, Pressed::Counted),
        (COMPUTER_TEAMS + 3, Pressed::Chosen),
        (WATER + 1, Pressed::Chosen),
        (MONSTERS + 2, Pressed::Chosen),
    ];
    for (id, pressed) in presses {
        assert_eq!(random.click(id), Some(pressed), "{id}");
    }
    let c = random.choices();
    assert_eq!((c.size, c.levels), (108, 1));
    assert_eq!(
        (c.players, c.teams, c.computers),
        (Some(3), Some(1), Some(4))
    );
    assert_eq!(
        (c.computer_teams, c.water, c.monsters),
        (Some(3), Some(1), Some(2))
    );
    // The levels' switch back; each group's Random (the teams' builds no record: 0x586E94).
    let randoms = [
        (LEVELS, Pressed::Chosen),
        (PLAYERS + 8, Pressed::Counted),
        (TEAMS + 8, Pressed::Chosen),
        (COMPUTERS + 8, Pressed::Counted),
        (COMPUTER_TEAMS + 7, Pressed::Chosen),
        (WATER + 3, Pressed::Chosen),
        (MONSTERS + 3, Pressed::Chosen),
    ];
    for (id, pressed) in randoms {
        assert_eq!(random.click(id), Some(pressed), "{id}");
    }
    let c = random.choices();
    assert_eq!(c.levels, 2);
    assert_eq!((c.players, c.teams, c.computers), (None, None, None));
    assert_eq!((c.computer_teams, c.water, c.monsters), (None, None, None));
    // Not a choice: the labels, Show Random Maps (T5b), another item.
    for id in [280, 286, SHOW, LIST] {
        assert_eq!(random.click(id), None, "{id}");
    }
}

#[test]
fn the_record_lets_humans_play_below_the_players_and_anyone_below_the_total() {
    let mut random = RandomMap::default();
    assert_eq!(random.record(), None);
    random.click(PLAYERS + 1);
    random.click(COMPUTERS + 3);
    random.click(281);
    let offers: [Vec<u8>; 9] = core::array::from_fn(|f| vec![u8::try_from(16 * f).unwrap()]);
    random.build("Random Map", "Randomly made", &offers);
    assert!(random.chosen());
    let record = random.record().unwrap();
    assert_eq!(record.name, "Random Map");
    assert_eq!(record.description, "Randomly made");
    assert_eq!((record.size, record.difficulty), (36, 1));
    assert_eq!((record.victory, record.loss), (None, None));
    assert_eq!(record.format, Some(Format::SoD));
    let humans: Vec<bool> = record.seats.iter().map(|s| s.human).collect();
    let anyone: Vec<bool> = record.seats.iter().map(|s| s.plays()).collect();
    assert_eq!(
        humans,
        [true, true, false, false, false, false, false, false]
    );
    assert_eq!(anyone, [true, true, true, true, true, false, false, false]);
    assert!(
        record
            .seats
            .iter()
            .all(|s| s.factions == 0x1FF && s.random_town)
    );
    assert_eq!(record.teams, [0, 1, 2, 3, 4, 5, 6, 7]);
    assert_eq!((record.players(), record.humans()), (5, 2));
    // Each colour offered each faction's heroes (E27 M3 T7).
    assert!(record.heroes.offers.iter().all(|o| *o == offers));
    random.unchoose();
    assert_eq!(random.record(), None);
}

// The screen's tab.

/// A click at `point`: the move, the press and the release.
fn click(s: &mut Scenario, point: [i32; 2]) -> Did {
    s.moved(point);
    s.press(point);
    s.release()
}

/// The centre of the tab's button `id` (its group's place: the sizes at y 81, the counts 32 apart
/// from x 70, Random at x 326, the water's and the monsters' 85 apart).
fn at(id: u16) -> [i32; 2] {
    let x = |k: u16, step: i32| 70 + step * i32::from(k) + 15;
    match id {
        281..=285 => [161 + 47 * i32::from(id - 281) + 22, 97],
        287..=294 => [x(id - PLAYERS, 32), 169],
        297..=304 => [x(id - TEAMS, 32), 230],
        307..=314 => [x(id - COMPUTERS, 32), 301],
        317..=323 => [x(id - COMPUTER_TEAMS, 32), 367],
        326..=328 => [x(id - WATER, 85) + 25, 435],
        331..=333 => [x(id - MONSTERS, 85) + 25, 501],
        295 | 305 | 315 | 324 | 329 | 334 => {
            let y = [
                (295, 169),
                (305, 235),
                (315, 301),
                (324, 367),
                (329, 435),
                (334, 501),
            ];
            [
                353,
                y.iter().find(|&&(i, _)| i == id).map_or(0, |&(_, y)| y),
            ]
        }
        _ => [225, 555],
    }
}

/// The label drawn for item `id`, if any: its text.
fn label(draws: &[Draw], id: u16) -> Option<&str> {
    draws.iter().find_map(|d| match d {
        Draw::Label { item, text, .. } if item.id == id => Some(text.as_str()),
        _ => None,
    })
}

/// The size mark's frame drawn, if any.
fn size_mark(draws: &[Draw]) -> Option<u32> {
    draws.iter().find_map(|d| match d {
        Draw::Button { def, frame, .. } if def == "scnrmpsz.def" => Some(*frame),
        _ => None,
    })
}

/// The flags drawn: (allies, enemies) by colour.
fn flags(draws: &[Draw]) -> (Vec<u32>, Vec<u32>) {
    let mut allies = Vec::new();
    let mut enemies = Vec::new();
    for d in draws {
        if let Draw::Button {
            def,
            frame,
            x,
            y: 405,
            ..
        } = d
            && def == "itgflags.def"
        {
            if *x < 640 {
                allies.push(*frame);
            } else {
                enemies.push(*frame);
            }
        }
    }
    (allies, enemies)
}

/// The screen with its tab opened (Random Map clicked) in the profile `bugs`.
fn opened(bugs: Bugs) -> Scenario {
    let mut s = scenario(bugs);
    assert!(click(&mut s, [514, 115]).changed);
    s
}

#[test]
fn random_map_shows_the_tab_and_a_random_map_s_record() {
    let s = opened(Bugs::Original);
    assert!(s.random_showing());
    assert!(!s.showing());
    assert_eq!(s.random_map(), Some(Choices::default()));
    let draws = s.draws("PLAYERS.PAL");
    assert!(
        draws
            .iter()
            .any(|d| matches!(d, Draw::Picture { picture, .. } if picture == "RanMapBk.pcx"))
    );
    assert_eq!(label(&draws, TITLE), Some("random-title"));
    assert_eq!(label(&draws, 280), Some("random-size"));
    // The panel: a random map's name and description, the standard victory and loss, Normal.
    assert_eq!(label(&draws, NAME), Some("Random Map"));
    assert_eq!(label(&draws, DESCRIPTION), Some("Randomly made"));
    assert_eq!(label(&draws, VICTORY), Some("victory 0"));
    assert_eq!(label(&draws, LOSS), Some("loss 0"));
    assert_eq!(label(&draws, MAP_DIFFICULTY), Some("difficulty 1"));
    // Red the single player's, the seven others its enemies; the size mark Arrogance's (S).
    assert_eq!(flags(&draws), (vec![0], vec![1, 2, 3, 4, 5, 6, 7]));
    assert_eq!(size_mark(&draws), Some(0));
    // The chosen buttons selected: M's frame 3, Random's 2.
    let frame = |def: &str, x: i32, y: i32| {
        draws.iter().find_map(|d| match d {
            Draw::Button {
                def: d,
                frame,
                x: dx,
                y: dy,
                ..
            } if d == def && (*dx, *dy) == (x, y) => Some(*frame),
            _ => None,
        })
    };
    assert_eq!(frame("RanSizM.def", 208, 81), Some(3));
    assert_eq!(frame("RanRand.def", 326, 153), Some(2));
    // Pressed again it hides, the random map still chosen.
    let mut s = s;
    assert!(click(&mut s, [514, 115]).changed);
    assert!(!s.random_showing());
    assert!(s.random_map().is_some());
    let draws = s.draws("PLAYERS.PAL");
    assert_eq!(label(&draws, TITLE), None);
    assert_eq!(label(&draws, NAME), Some("Random Map"));
}

#[test]
fn a_count_s_press_builds_the_record_again_the_others_do_not() {
    let mut s = opened(Bugs::Original);
    // One player, no computer only one: red alone.
    assert!(click(&mut s, at(PLAYERS)).changed);
    let draws = s.draws("PLAYERS.PAL");
    assert_eq!(flags(&draws), (vec![0], vec![]));
    // Two computer only players: blue and tan the enemies.
    click(&mut s, at(COMPUTERS + 2));
    assert_eq!(flags(&s.draws("PLAYERS.PAL")), (vec![0], vec![1, 2]));
    // A size's press builds none: the record keeps its size; the size mark stays (the original's).
    click(&mut s, at(284));
    assert_eq!(s.random_map().map(|c| c.size), Some(144));
    assert_eq!(s.random.record().map(|r| r.size), Some(72));
    assert_eq!(size_mark(&s.draws("PLAYERS.PAL")), Some(0));
    // Nor do Random teams.
    click(&mut s, at(TEAMS + 8));
    assert_eq!(s.random.record().map(|r| r.size), Some(72));
    // A team's count does.
    click(&mut s, at(TEAMS));
    assert_eq!(s.random.record().map(|r| r.size), Some(144));
    // A disabled button takes no press: with one player, two teams.
    let before = s.random_map();
    let did = click(&mut s, at(TEAMS + 2));
    assert!(!did.changed);
    assert_eq!(s.random_map(), before);
}

#[test]
fn in_the_client_s_profile_the_size_mark_follows_and_each_opening_builds_the_record() {
    // CLT-Q115 (b): the size mark the chosen size's.
    let mut s = opened(Bugs::Fixed);
    assert_eq!(size_mark(&s.draws("PLAYERS.PAL")), Some(1));
    click(&mut s, at(283));
    assert_eq!(size_mark(&s.draws("PLAYERS.PAL")), Some(2));
    // CLT-Q111 (b): the record built again at the next opening; the original's keeps it.
    for (bugs, size) in [(Bugs::Fixed, 108), (Bugs::Original, 72)] {
        let mut s = opened(bugs);
        click(&mut s, at(283));
        click(&mut s, [514, 115]);
        click(&mut s, [514, 115]);
        assert_eq!(s.random.record().map(|r| r.size), Some(size), "{bugs:?}");
    }
}

#[test]
fn the_list_s_showing_hides_the_tab_and_chooses_no_random_map() {
    let mut s = opened(Bugs::Original);
    assert!(click(&mut s, [514, 91]).changed);
    assert!(s.showing());
    assert!(!s.random_showing());
    assert_eq!(s.random_map(), None);
    // The tab's hidden buttons take no press over the list's: a column's sort under a size.
    let key = s.key;
    click(&mut s, at(281));
    assert_ne!(s.key, key);
    // Random Map hides the list.
    assert!(click(&mut s, [514, 115]).changed);
    assert!(!s.showing());
    assert!(s.random_showing());
}

#[test]
fn the_advanced_options_hide_the_tab_and_show_the_random_map_s_players() {
    let mut s = with_options_in(vec![super::super::tests::many(1).remove(0)], Bugs::Original);
    assert!(click(&mut s, [514, 115]).changed);
    click(&mut s, at(PLAYERS + 2));
    assert!(click(&mut s, [514, 519]).changed);
    assert!(s.advanced_showing());
    assert!(!s.random_showing());
    assert!(s.random_map().is_some());
    // The options read the random map's seats: three colours a human's.
    assert_eq!(
        s.players().seats.iter().filter(|seat| seat.human).count(),
        3
    );
    // Random Map hides the options again.
    assert!(click(&mut s, [514, 115]).changed);
    assert!(!s.advanced_showing());
    assert!(s.random_showing());
}

#[test]
fn a_random_map_s_row_shows_the_hero_random_and_its_arrows_once_a_town_is_chosen() {
    // E27 M3 T7, the review's C2: the record's generate flag set (+0xB, design §30.2), so the
    // hero is Random (`hpsrand1.pcx`, not None's `hpsrand6.pcx`), its arrows shown with a town
    // (§3.14); each faction's free heroes offered (recorded, `adv-random-options`: Castle's
    // arrow to Orrin, its first), so the arrow steps to the first; kept by an opening anew.
    let castle: Vec<u8> = (0..16).collect();
    let offers: [Vec<u8>; 9] =
        core::array::from_fn(|f| if f == 0 { castle.clone() } else { Vec::new() });
    let mut s = with_options_in(vec![map("Arrogance.h3m", "Arrogance")], Bugs::Fixed)
        .with_random_offers(offers.clone())
        .opened(Hovers::default());
    click(&mut s, [514, 115]);
    click(&mut s, [514, 519]);
    assert!(s.players().seats.iter().all(|seat| seat.generates));
    let hero = |s: &Scenario| {
        s.draws("P").into_iter().find_map(|d| match d {
            Draw::Picture { picture, x, y, .. } if [x, y] == [252, 130] => Some(picture),
            _ => None,
        })
    };
    assert_eq!(hero(&s).as_deref(), Some("hpsrand1.pcx"));
    assert!(s.shown.state(231).hidden);
    click(&mut s, [230, 145]);
    assert!(!s.shown.state(231).hidden && !s.shown.state(239).hidden);
    click(&mut s, [306, 145]);
    assert_ne!(hero(&s).as_deref(), Some("hpsrand1.pcx"));
    assert_eq!(s.records().unwrap()[0].hero, Some(0));
}

#[test]
fn a_screen_without_maps_shows_the_random_map_s_record_in_the_client_s_profile() {
    // The original's panel draws nothing over an empty list (`scenario_info_draw` 0x584128, E27
    // M3 T5b), a random map's record neither; its description shows.
    let mut s = scenario_of(Vec::<Entry>::new(), Bugs::Original);
    assert_eq!(label(&s.draws("PLAYERS.PAL"), NAME), Some(""));
    click(&mut s, [514, 115]);
    let draws = s.draws("PLAYERS.PAL");
    assert_eq!(label(&draws, NAME), Some(""));
    assert_eq!(label(&draws, DESCRIPTION), Some("Randomly made"));
    let mut s = scenario_of(Vec::<Entry>::new(), Bugs::Fixed);
    click(&mut s, [514, 115]);
    assert_eq!(label(&s.draws("PLAYERS.PAL"), NAME), Some("Random Map"));
    // The hotkey R toggles the tab too.
    s.key(None, Some(Command::Dialog(RANDOM)), "R");
    assert!(s.key_up("R").changed);
    assert!(!s.random_showing());
}

#[test]
fn a_difficulty_s_press_keeps_the_random_map_s_panel() {
    let mut s = opened(Bugs::Original);
    // Hard (109) over the tab: no choice of the tab's, the panel still the random map's.
    assert!(click(&mut s, [585, 479]).changed);
    assert_eq!(s.difficulty(), 2);
    assert_eq!(label(&s.draws("PLAYERS.PAL"), NAME), Some("Random Map"));
    assert_eq!(s.random_map(), Some(Choices::default()));
}

// E27 M3 T5b: the right press, CLT-Q146, Show Random Maps, the tab again, Begin.

#[test]
fn every_item_of_the_tab_has_its_help_record() {
    let ids: Vec<u16> = HELPED.iter().map(|&(id, _)| id).collect();
    assert_eq!(ids, (280..=SHOW).collect::<Vec<_>>());
    let s = opened(Bugs::Original);
    assert!(ids.iter().all(|id| s.texts.help.contains_key(id)));
}

#[test]
fn a_right_press_on_the_tab_shows_its_item_s_help_a_disabled_count_s_too() {
    let mut s = opened(Bugs::Original);
    // A size, the size's label, Show Random Maps, Random water.
    for (point, id) in [
        (at(281), 281),
        ([100, 95], 280),
        (at(SHOW), SHOW),
        (at(WATER + 3), WATER + 3),
    ] {
        assert!(s.right(point).changed, "{id}");
        assert_eq!(s.over, Over::Help(id));
        assert!(!s.box_draws("PLAYERS.PAL").is_empty());
        assert!(s.right_release().changed);
    }
    // One player: their teams' 1 disabled, its popup all the same (`adv-random-list`
    // popup-disabled).
    click(&mut s, at(PLAYERS));
    assert!(s.shown.state(TEAMS + 1).disabled);
    assert!(s.right(at(TEAMS + 1)).changed);
    assert_eq!(s.over, Over::Help(TEAMS + 1));
    assert!(s.right_release().changed);
    // Hidden, the tab's items show none (nothing: E27 M4 T4b1).
    click(&mut s, [514, 115]);
    assert_eq!(s.right(at(281)), Did::default());
    assert_eq!(s.over, Over::Nothing);
}

#[test]
fn the_teams_1_and_2_take_the_mouse_32_rows_high_in_the_client_s_profile() {
    // CLT-Q146: 30 × 24 in the original's, their lowest 8 rows (y 243 to 250) taking no press; the
    // client's 32 high like their frames.
    for (bugs, low) in [(Bugs::Original, None), (Bugs::Fixed, Some(1))] {
        let mut s = opened(bugs);
        click(&mut s, [117, 247]);
        assert_eq!(s.random_map().map(|c| c.teams), Some(low), "{bugs:?}");
        click(&mut s, [149, 249]);
        let two = low.map(|_| 2);
        assert_eq!(s.random_map().map(|c| c.teams), Some(two), "{bugs:?}");
        // Their upper rows in both.
        click(&mut s, [149, 222]);
        assert_eq!(s.random_map().map(|c| c.teams), Some(Some(2)), "{bugs:?}");
    }
}

#[test]
fn show_random_maps_lists_none_and_the_original_s_panel_keeps_its_leftovers() {
    let mut s = opened(Bugs::Original);
    assert!(click(&mut s, at(SHOW)).changed);
    assert!(s.showing() && !s.random_showing());
    assert_eq!(s.random_map(), None);
    assert_eq!((s.listed().count(), s.rows()), (0, (None, 0)));
    // `scenario_info_draw` draws nothing over the empty list; the random map's description and
    // flags stay, the size mark Arrogance's S.
    let draws = s.draws("PLAYERS.PAL");
    assert_eq!(label(&draws, NAME), Some(""));
    assert_eq!(label(&draws, VICTORY), Some(""));
    assert_eq!(label(&draws, DESCRIPTION), Some("Randomly made"));
    assert_eq!(flags(&draws), (vec![0], vec![1, 2, 3, 4, 5, 6, 7]));
    assert_eq!(size_mark(&draws), Some(0));
    // The tab again: the record built anew over the random maps' empty list, the panel still
    // drawing nothing of it (CLT-Q111's recording); hidden again, the same.
    for showing in [true, false] {
        assert!(click(&mut s, [514, 115]).changed);
        assert_eq!(s.random_showing(), showing);
        assert!(!s.showing() && s.random_map().is_some());
        let draws = s.draws("PLAYERS.PAL");
        assert_eq!(label(&draws, NAME), Some(""));
        assert_eq!(label(&draws, LOSS), Some(""));
        assert_eq!(label(&draws, DESCRIPTION), Some("Randomly made"));
        assert_eq!(flags(&draws), (vec![0], vec![1, 2, 3, 4, 5, 6, 7]));
    }
}

#[test]
fn in_the_client_s_profile_the_random_maps_show_nothing_and_the_tab_again_its_record() {
    let mut s = opened(Bugs::Fixed);
    click(&mut s, at(SHOW));
    let draws = s.draws("PLAYERS.PAL");
    assert_eq!(label(&draws, NAME), Some(""));
    assert_eq!(label(&draws, DESCRIPTION), Some(""));
    assert_eq!(flags(&draws), (vec![], vec![]));
    assert_eq!(size_mark(&draws), Some(4));
    // CLT-Q111 (b): the tab again shows the random map's record.
    click(&mut s, [514, 115]);
    let draws = s.draws("PLAYERS.PAL");
    assert_eq!(label(&draws, NAME), Some("Random Map"));
    assert_eq!(label(&draws, VICTORY), Some("victory 0"));
    assert_eq!(label(&draws, DESCRIPTION), Some("Randomly made"));
    assert_eq!(flags(&draws), (vec![0], vec![1, 2, 3, 4, 5, 6, 7]));
    assert_eq!(size_mark(&draws), Some(1));
}

#[test]
fn the_files_list_back_after_the_random_maps_is_sorted_anew() {
    // The list's kind changed back (`scenario_list_toggle` 0x580AB1): sorted by the version, its
    // first row selected and given to the game, not the kept map.
    let mut s = scenario_of(
        vec![map("Arrogance.h3m", "Arrogance"), map("a.h3m", "Alpha")],
        Bugs::Original,
    );
    assert_eq!(s.rows(), (Some(0), 0));
    click(&mut s, [514, 115]);
    click(&mut s, at(SHOW));
    // Shown, the list hides at a press of Show Available Scenarios, its kind kept.
    click(&mut s, [514, 91]);
    assert!(!s.showing());
    assert_eq!(s.listed().count(), 0);
    click(&mut s, [514, 91]);
    assert!(s.showing());
    assert_eq!((s.listed().count(), s.rows()), (2, (Some(0), 0)));
    assert_eq!(s.entry().map(|e| e.name.as_str()), Some("Alpha"));
    let draws = s.draws("PLAYERS.PAL");
    assert_eq!(label(&draws, NAME), Some("Alpha"));
    // The same kind again keeps its order and selection.
    click(&mut s, [514, 91]);
    click(&mut s, [514, 91]);
    assert_eq!(s.rows(), (Some(0), 0));
}

#[test]
fn begin_with_a_random_map_chosen_shows_the_client_s_stub() {
    for bugs in [Bugs::Original, Bugs::Fixed] {
        let mut s = opened(bugs);
        assert!(!s.stubbed());
        assert!(click(&mut s, [497, 555]).changed);
        assert!(s.stubbed(), "{bugs:?}");
        assert_eq!(s.context(), Context::Question);
        // Its OK closes it, the random map still chosen.
        let ok = ok_button(&s);
        s.press(ok);
        assert!(s.release().changed);
        assert!(!s.stubbed());
        assert_eq!(s.context(), Context::Scenario);
        assert!(s.random_map().is_some());
    }
    // Over the random maps' empty list no random map is chosen: GENRLTXT 530's box, no stub.
    let mut s = opened(Bugs::Original);
    click(&mut s, at(SHOW));
    assert!(click(&mut s, [497, 555]).changed);
    assert_eq!(s.context(), Context::Question);
    assert!(!s.stubbed());
    // Without the stub's text Begin does nothing with a random map chosen.
    let mut s = opened(Bugs::Fixed);
    s.texts.random_stub = None;
    click(&mut s, [497, 555]);
    assert_eq!(s.context(), Context::Scenario);
    assert!(!s.stubbed());
}

#[test]
fn the_list_over_a_chosen_random_map_mixes_the_two_in_the_original_s_profile() {
    // CLT-Q147: the original's showing gives the game nothing (`adv-random-list` list): the
    // selection's name over the random map's header, description and flags; the client's (b)
    // shows the selection whole.
    let mut s = opened(Bugs::Original);
    click(&mut s, at(PLAYERS));
    click(&mut s, [514, 91]);
    let draws = s.draws("PLAYERS.PAL");
    assert_eq!(label(&draws, NAME), Some("Arrogance"));
    assert_eq!(label(&draws, MAP_DIFFICULTY), Some("difficulty 1"));
    assert_eq!(label(&draws, DESCRIPTION), Some("Randomly made"));
    assert_eq!(flags(&draws), (vec![0], vec![]));
    let mut s = opened(Bugs::Fixed);
    click(&mut s, at(PLAYERS));
    click(&mut s, [514, 91]);
    let draws = s.draws("PLAYERS.PAL");
    assert_eq!(label(&draws, NAME), Some("Arrogance"));
    assert_eq!(label(&draws, MAP_DIFFICULTY), Some("difficulty 2"));
    assert_eq!(label(&draws, DESCRIPTION), Some("A map."));
    assert_eq!(flags(&draws), (vec![0, 2], vec![1]));
}

#[test]
fn the_files_list_back_keeps_the_random_map_s_flags_in_the_original_s_profile() {
    // CLT-Q147: the sort gives the game the first row (its description) but sets no flags
    // (`adv-random-list` list-back); the client's (b) sets them.
    for (bugs, shown) in [
        (Bugs::Original, (vec![0], vec![])),
        (Bugs::Fixed, (vec![0, 2], vec![1])),
    ] {
        let mut s = opened(bugs);
        click(&mut s, at(PLAYERS));
        click(&mut s, at(SHOW));
        click(&mut s, [514, 91]);
        click(&mut s, [514, 91]);
        let draws = s.draws("PLAYERS.PAL");
        assert_eq!(label(&draws, NAME), Some("Arrogance"), "{bugs:?}");
        assert_eq!(label(&draws, DESCRIPTION), Some("A map."), "{bugs:?}");
        assert_eq!(flags(&draws), shown, "{bugs:?}");
    }
}

/// E27 M3 T6: with a random map chosen Begin hands nothing on (its stub's notice, T5b).
#[test]
fn a_random_map_chosen_begins_nothing() {
    let s = opened(Bugs::Fixed);
    assert!(s.random_map().is_some());
    assert_eq!(s.begun(), None);
}
