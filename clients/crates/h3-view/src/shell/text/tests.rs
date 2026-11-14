//! The shell's headless driver's tests (E27 M2 T3a).

use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::vec::Vec;

use super::{answer, describe, ending, name, word};
use crate::shell::{Choice, Ended};
use crate::shell::{Game, Kind, Screen, Shell};

#[test]
fn a_screen_is_described_by_its_name_its_task_and_its_words() {
    assert_eq!(
        describe(Screen::MainMenu),
        "main-menu [E27 M2 T6]: new-game load-game high-scores credits quit"
    );
    assert_eq!(
        describe(Screen::GameType(Game::Load)),
        "load-game-menu [E27 M2 T6]: single multiplayer campaign tutorial back"
    );
    assert_eq!(
        describe(Screen::Multiplayer(Game::New)),
        "multiplayer (new game) [E27 M2 T7]: hot-seat cancel"
    );
    assert_eq!(
        describe(Screen::Scenario(Game::Load, Kind::Tutorial)),
        "scenario (load game, tutorial) [E27 M3]: begin back"
    );
    assert_eq!(
        describe(Screen::Play { restart: true }),
        "play (restart) [E24 M5]: main-menu new-game load-game restart quit lost won"
    );
    assert_eq!(describe(Screen::End), "end");
}

#[test]
fn every_screen_has_a_name_of_its_own_and_words_unique_among_its_endings() {
    let mut screens = Vec::from([
        Screen::Start,
        Screen::MainMenu,
        Screen::GameType(Game::New),
        Screen::GameType(Game::Load),
        Screen::Campaigns,
        Screen::Tutorial,
        Screen::HighScores,
        Screen::Credits,
        Screen::Play { restart: false },
        Screen::Play { restart: true },
        Screen::Lost,
        Screen::Won,
        Screen::End,
    ]);
    for game in [Game::New, Game::Load] {
        screens.push(Screen::Multiplayer(game));
        for kind in [
            Kind::Single,
            Kind::Campaign,
            Kind::Multiplayer,
            Kind::Tutorial,
        ] {
            screens.push(Screen::Scenario(game, kind));
        }
    }
    let names: BTreeSet<String> = screens.iter().map(|&s| name(s)).collect();
    assert_eq!(names.len(), screens.len());
    for screen in screens {
        let words: BTreeSet<&str> = screen.endings().iter().map(|&e| word(e)).collect();
        assert_eq!(words.len(), screen.endings().len(), "{screen:?}");
    }
}

#[test]
fn a_line_ends_the_screen_by_its_word() {
    let mut shell = Shell::new(true);
    let transcript: Vec<String> = [
        "",
        "done",
        "new-game",
        "multiplayer",
        "cancel",
        "campaign",
        "back",
        "tutorial",
        "back",
        "single",
        "begin",
        "restart",
        "won",
        " done ",
        "done",
        "load-game",
        "single",
        "back",
        "back",
        "credits",
        "done",
        "quit",
    ]
    .iter()
    .map(|line| answer(&mut shell, line))
    .collect();
    let screens: Vec<&str> = transcript
        .iter()
        .map(|t| t.split(" [").next().unwrap_or(""))
        .collect();
    assert_eq!(
        screens,
        [
            "start",
            "main-menu",
            "new-game-menu",
            "multiplayer (new game)",
            "new-game-menu",
            "campaigns",
            "new-game-menu",
            "tutorial",
            "new-game-menu",
            "scenario (new game, single)",
            "play",
            "play (restart)",
            "won",
            "high-scores",
            "main-menu",
            "load-game-menu",
            "scenario (load game, single)",
            "load-game-menu",
            "main-menu",
            "credits",
            "main-menu",
            "end",
        ]
    );
    assert_eq!(shell.screen(), Screen::End);
}

#[test]
fn a_word_the_screen_does_not_take_is_refused_with_the_words_it_takes() {
    let mut shell = Shell::new(false);
    assert_eq!(
        answer(&mut shell, "begin"),
        "not on main-menu: begin (main-menu [E27 M2 T6]: new-game load-game high-scores \
         credits quit)"
    );
    assert_eq!(shell.screen(), Screen::MainMenu);
    answer(&mut shell, "quit");
    assert_eq!(answer(&mut shell, "done"), "not on end: done (end)");
    assert_eq!(answer(&mut shell, ""), "end");
}

#[test]
fn the_lost_screen_and_the_start_end_with_done() {
    let mut shell = Shell::new(false);
    for line in ["new-game", "single", "begin", "lost"] {
        answer(&mut shell, line);
    }
    assert_eq!(shell.screen(), Screen::Lost);
    assert!(answer(&mut shell, "done").starts_with("main-menu "));
}

#[test]
fn a_word_names_its_ending_on_its_screen_alone() {
    let menu = Screen::GameType(Game::New);
    assert_eq!(ending(menu, " back "), Some(Ended::GameType(Choice::Back)));
    assert_eq!(ending(menu, "begin"), None);
    assert_eq!(ending(Screen::End, "done"), None);
}
