//! The shell's frame's tests (E27 M2 T3a): the menu loop's switches (design §2.2 to §2.4, §4.3,
//! §5.3).

use alloc::format;
use alloc::vec::Vec;

use super::{Choice, Ended, Game, Kind, PlayEnd, Refused, Request, Screen, Shell};

/// Every screen.
fn screens() -> Vec<Screen> {
    let mut all = Vec::from([
        Screen::Start,
        Screen::MainMenu,
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
        all.extend([Screen::GameType(game), Screen::Multiplayer(game)]);
        for kind in [
            Kind::Single,
            Kind::Campaign,
            Kind::Multiplayer,
            Kind::Tutorial,
        ] {
            all.push(Screen::Scenario(game, kind));
        }
    }
    all
}

const REQUESTS: [Request; 7] = [
    Request::NewGame,
    Request::LoadGame,
    Request::HighScores,
    Request::Credits,
    Request::Quit,
    Request::Restart,
    Request::MainMenu,
];

/// Every ending.
fn endings() -> Vec<Ended> {
    let mut all = Vec::from([
        Ended::Start,
        Ended::Multiplayer(true),
        Ended::Multiplayer(false),
        Ended::Scenario(true),
        Ended::Scenario(false),
        Ended::Back,
        Ended::Done,
        Ended::Play(PlayEnd::Lost),
        Ended::Play(PlayEnd::Won),
    ]);
    for request in REQUESTS {
        all.extend([
            Ended::MainMenu(request),
            Ended::Play(PlayEnd::Request(request)),
        ]);
    }
    all.extend(
        [
            Choice::Single,
            Choice::Campaign,
            Choice::Multiplayer,
            Choice::Tutorial,
            Choice::Back,
        ]
        .map(Ended::GameType),
    );
    all
}

/// The shell on `screen`.
const fn at(screen: Screen) -> Shell {
    Shell { screen }
}

/// The screen `ended` leads to from `screen`.
fn next(screen: Screen, ended: Ended) -> Screen {
    at(screen).end(ended).unwrap()
}

#[test]
fn the_start_is_the_clips_with_show_intro_else_the_main_menu() {
    assert_eq!(Shell::new(true).screen(), Screen::Start);
    assert_eq!(Shell::new(false).screen(), Screen::MainMenu);
    let mut shell = Shell::new(true);
    assert_eq!(shell.end(Ended::Start), Ok(Screen::MainMenu));
    assert_eq!(shell.screen(), Screen::MainMenu);
}

#[test]
fn a_screen_takes_its_own_endings_alone_and_a_refusal_changes_nothing() {
    for screen in screens() {
        for ended in endings() {
            let mut shell = at(screen);
            let result = shell.end(ended);
            assert_eq!(
                result.is_ok(),
                screen.endings().contains(&ended),
                "{screen:?} {ended:?}"
            );
            match result {
                Ok(next) => assert_eq!(shell.screen(), next),
                Err(refused) => {
                    assert_eq!(refused, Refused { screen, ended });
                    assert_eq!(shell, at(screen));
                }
            }
        }
    }
}

#[test]
fn the_main_menu_s_commands_take_the_loop_s_switch() {
    let menu = |request| next(Screen::MainMenu, Ended::MainMenu(request));
    assert_eq!(menu(Request::NewGame), Screen::GameType(Game::New));
    assert_eq!(menu(Request::LoadGame), Screen::GameType(Game::Load));
    assert_eq!(menu(Request::HighScores), Screen::HighScores);
    assert_eq!(menu(Request::Credits), Screen::Credits);
    assert_eq!(menu(Request::Quit), Screen::End);
    for screen in [Screen::HighScores, Screen::Credits] {
        assert_eq!(next(screen, Ended::Done), Screen::MainMenu);
    }
    assert!(Screen::End.endings().is_empty());
}

#[test]
fn new_game_s_menu_and_a_back_from_its_screens() {
    let new = |choice| next(Screen::GameType(Game::New), Ended::GameType(choice));
    assert_eq!(
        new(Choice::Single),
        Screen::Scenario(Game::New, Kind::Single)
    );
    assert_eq!(new(Choice::Multiplayer), Screen::Multiplayer(Game::New));
    assert_eq!(new(Choice::Campaign), Screen::Campaigns);
    assert_eq!(new(Choice::Tutorial), Screen::Tutorial);
    assert_eq!(new(Choice::Back), Screen::MainMenu);
    assert_eq!(
        next(Screen::Multiplayer(Game::New), Ended::Multiplayer(true)),
        Screen::Scenario(Game::New, Kind::Multiplayer)
    );
    for (screen, ended) in [
        (Screen::Multiplayer(Game::New), Ended::Multiplayer(false)),
        (Screen::Campaigns, Ended::Back),
        (Screen::Tutorial, Ended::Back),
        (
            Screen::Scenario(Game::New, Kind::Single),
            Ended::Scenario(false),
        ),
        (
            Screen::Scenario(Game::New, Kind::Multiplayer),
            Ended::Scenario(false),
        ),
    ] {
        assert_eq!(next(screen, ended), Screen::GameType(Game::New));
    }
    assert_eq!(
        next(
            Screen::Scenario(Game::New, Kind::Single),
            Ended::Scenario(true)
        ),
        Screen::Play { restart: false }
    );
}

#[test]
fn load_game_s_menu_opens_the_scenario_screen_in_its_load_mode() {
    let load = |choice| next(Screen::GameType(Game::Load), Ended::GameType(choice));
    assert_eq!(
        load(Choice::Single),
        Screen::Scenario(Game::Load, Kind::Single)
    );
    assert_eq!(
        load(Choice::Campaign),
        Screen::Scenario(Game::Load, Kind::Campaign)
    );
    assert_eq!(
        load(Choice::Tutorial),
        Screen::Scenario(Game::Load, Kind::Tutorial)
    );
    assert_eq!(load(Choice::Multiplayer), Screen::Multiplayer(Game::Load));
    assert_eq!(load(Choice::Back), Screen::MainMenu);
    assert_eq!(
        next(Screen::Multiplayer(Game::Load), Ended::Multiplayer(true)),
        Screen::Scenario(Game::Load, Kind::Multiplayer)
    );
    assert_eq!(
        next(Screen::Multiplayer(Game::Load), Ended::Multiplayer(false)),
        Screen::GameType(Game::Load)
    );
    let saved = Screen::Scenario(Game::Load, Kind::Campaign);
    assert_eq!(
        next(saved, Ended::Scenario(false)),
        Screen::GameType(Game::Load)
    );
    assert_eq!(
        next(saved, Ended::Scenario(true)),
        Screen::Play { restart: false }
    );
}

#[test]
fn play_s_requests_are_taken_at_the_loop_s_top() {
    for restart in [false, true] {
        let play = |request| {
            next(
                Screen::Play { restart },
                Ended::Play(PlayEnd::Request(request)),
            )
        };
        assert_eq!(play(Request::MainMenu), Screen::MainMenu);
        assert_eq!(play(Request::NewGame), Screen::GameType(Game::New));
        assert_eq!(play(Request::LoadGame), Screen::GameType(Game::Load));
        assert_eq!(play(Request::Quit), Screen::End);
        assert_eq!(play(Request::Restart), Screen::Play { restart: true });
    }
}

#[test]
fn a_won_scenario_shows_high_scores_after_its_screen_a_lost_one_does_not() {
    let play = Screen::Play { restart: false };
    assert_eq!(next(play, Ended::Play(PlayEnd::Lost)), Screen::Lost);
    assert_eq!(next(Screen::Lost, Ended::Done), Screen::MainMenu);
    assert_eq!(next(play, Ended::Play(PlayEnd::Won)), Screen::Won);
    assert_eq!(next(Screen::Won, Ended::Done), Screen::HighScores);
}

#[test]
fn each_screen_names_the_task_that_builds_it() {
    let tasks: Vec<(Screen, &str)> = screens().into_iter().map(|s| (s, s.task())).collect();
    for (screen, task) in tasks {
        assert_eq!(task.is_empty(), screen == Screen::End, "{screen:?}");
    }
    assert_eq!(Screen::Start.task(), "E27 M2 T5");
    assert_eq!(Screen::Multiplayer(Game::Load).task(), "E27 M2 T7");
    assert_eq!(Screen::Scenario(Game::New, Kind::Single).task(), "E27 M3");
    assert_eq!(Screen::Play { restart: true }.task(), "E24 M5");
    assert_eq!(Screen::HighScores.task(), "E27 M4");
}

#[test]
fn a_refusal_reads_as_the_screen_and_the_ending() {
    let refused = at(Screen::End).end(Ended::Done).unwrap_err();
    assert_eq!(format!("{refused}"), "End cannot end with Done");
}

#[test]
fn the_ids_are_the_originals() {
    let ids: Vec<u16> = REQUESTS.iter().map(|&r| r as u16).collect();
    assert_eq!(ids, [101, 102, 103, 104, 105, 107, 108]);
    assert_eq!(Choice::Single as u16, 100);
    assert_eq!(Choice::Back as u16, 104);
}
