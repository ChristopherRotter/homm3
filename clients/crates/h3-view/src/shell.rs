//! The shell's frame (E27 M2 T3a; CLT-Q103, decision 21: its pure part here, its element and
//! adapters in `h3-battle-client`): the screens outside play and the state between them, as
//! `game_main`'s menu loop runs them (`docs/epics/E27-design.md` §2.2, from 0x4EE842). A screen
//! ends with an [`Ended`]; [`Shell::end`] takes it to the next screen as the loop's switches do,
//! and refuses one the screen cannot end with. [`text`] is its headless driver: the `h3-view
//! shell` command's words (the text client first; the window follows, M2 T4).
//!
//! **The loop** (§2.2): at its top the screen asked for (`screen_request` 0x6976D8: the request
//! play ended with, [`PlayEnd::Request`]) is taken by the main menu's switch (0x4EEC0D): none or
//! Main Menu (108) the main menu; New Game (101) and Load Game (102) their menus (§2.4); High
//! Scores (103) and Credits (104), then the top again (§2.8, §2.9); Quit (105) the end; the
//! restart (107) play again (§4.6). The main menu's commands (101 to 105, Quit's after its
//! question, §2.3) take the same switch at once.
//!
//! **New Game's menu** (switch 0x4EECF5): Single the scenario screen (§3), Multiplayer its dialog
//! (§2.6) and then the scenario screen, Campaign the campaigns' selection (E20: decision 7's
//! stub), Tutorial the tutorial (CLT-Q84: a stub), Back the main menu; a back from any of them New
//! Game's menu again. **Load Game's menu** (switch 0x4EF02D): Single, Campaign and Tutorial the
//! scenario screen in its load mode (a campaign's saves, a saved tutorial), Multiplayer its dialog
//! and then the load; Back the main menu; a cancelled load Load Game's menu again, a loaded game
//! play.
//!
//! **Play's end** (§5.3, §4.3): with a request, the top takes it; a lost scenario the lost screen
//! (§5.4), a won one the won screen and its entry (§5.5, §5.6), then High Scores
//! (`high_scores_due` 0x69955C, always set by the entry), then the top. The campaigns' path (§5.7)
//! is E20's, the lobby's start (0x4EE903) not built (§2.10).
//!
//! Each screen names the task that builds it ([`Screen::task`]): the frame holds them all, the
//! screens themselves come with T5 to T7, M3, M4, E20 and E24 M5.

use core::fmt;

pub mod text;

/// The screen play's end or a main menu command asks for (`screen_request` 0x6976D8; the main
/// menu's commands): the original's ids.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Request {
    /// New Game's menu (101).
    NewGame = 101,
    /// Load Game's menu (102).
    LoadGame = 102,
    /// High Scores (103).
    HighScores = 103,
    /// Credits (104).
    Credits = 104,
    /// The end (105).
    Quit = 105,
    /// The restart (107, §4.6: the players' setup kept; CLT-Q97).
    Restart = 107,
    /// The main menu (108).
    MainMenu = 108,
}

/// New Game's or Load Game's: the game type menu's argument (0 or 1), the scenario screen's mode
/// (0 a new game, 1 a load).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Game {
    /// A new game.
    New,
    /// A saved game.
    Load,
}

/// A game type menu's command (§2.4): the original's ids.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Choice {
    /// Single Scenario (100).
    Single = 100,
    /// Campaign (101).
    Campaign = 101,
    /// Multiplayer (102).
    Multiplayer = 102,
    /// Tutorial (103).
    Tutorial = 103,
    /// Back (104; Load Game's 0x7801 too).
    Back = 104,
}

/// What the scenario screen lists and begins: a single scenario, a campaign's (Load Game's
/// Campaign: `campaign_saves_flag` 0x69FD90), a multiplayer one (the hot seat's names) or a saved
/// tutorial (the tutorial flag, game +0x1F69D).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// A single scenario.
    Single,
    /// A campaign's saved game.
    Campaign,
    /// A multiplayer game.
    Multiplayer,
    /// A saved tutorial.
    Tutorial,
}

/// A screen of the shell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    /// The clips before the main menu (§2.1).
    Start,
    /// The main menu (§2.3), its Quit's question among it.
    MainMenu,
    /// New Game's or Load Game's menu (§2.4).
    GameType(Game),
    /// The multiplayer dialog over that menu (§2.6), the hot seat's names its Hot Seat (§2.7).
    Multiplayer(Game),
    /// The scenario screen, a new game's or a load's (§3).
    Scenario(Game, Kind),
    /// The campaigns' selection (`campaign_selection_run` 0x4F00A0; E20).
    Campaigns,
    /// The tutorial (§2.4, 0x4EED32; CLT-Q84).
    Tutorial,
    /// High Scores (§2.8).
    HighScores,
    /// Credits (§2.9).
    Credits,
    /// Play: the game handed over (E24 M5 hosts it); a restart's (§4.6) or not.
    Play {
        /// The restart's play.
        restart: bool,
    },
    /// The lost scenario's screen (§5.4).
    Lost,
    /// The won scenario's screen, its score and the high score's entry (§5.5, §5.6).
    Won,
    /// The end (Quit).
    End,
}

/// How play ended (§5.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayEnd {
    /// With the screen it asks for (the system options' Load, Quit, Restart and Main Menu, §4.3;
    /// the adventure map's keys, M4).
    Request(Request),
    /// The scenario lost (`scenario_result` 0).
    Lost,
    /// The scenario won.
    Won,
}

/// How a screen ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ended {
    /// The start's clips ran out or were skipped.
    Start,
    /// The main menu with its command (101 to 105; Quit's after its question's OK).
    MainMenu(Request),
    /// A game type menu with its command.
    GameType(Choice),
    /// The multiplayer dialog: closed by the hot seat's names' OK (true) or cancelled (0x7801).
    Multiplayer(bool),
    /// The scenario screen: a game begun or loaded (true), or Back (a load cancelled).
    Scenario(bool),
    /// The campaigns' selection or the tutorial: back to New Game's menu (their stubs').
    Back,
    /// High Scores, Credits, the lost screen or the won one: done.
    Done,
    /// Play's end.
    Play(PlayEnd),
}

/// An ending the screen cannot have: the shell unchanged.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Refused {
    /// The screen.
    pub screen: Screen,
    /// The ending refused.
    pub ended: Ended,
}

impl fmt::Display for Refused {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} cannot end with {:?}", self.screen, self.ended)
    }
}

impl Screen {
    /// The task that builds the screen (the frame's stand-ins until then; T6 draws High Scores'
    /// and Credits' stubs, M4 the screens).
    #[must_use]
    pub const fn task(self) -> &'static str {
        match self {
            Self::Start => "E27 M2 T5",
            Self::MainMenu | Self::GameType(_) => "E27 M2 T6",
            Self::Multiplayer(_) => "E27 M2 T7",
            Self::Scenario(..) => "E27 M3",
            Self::Campaigns => "E20",
            Self::Tutorial => "CLT-Q84",
            Self::Play { .. } => "E24 M5",
            Self::HighScores | Self::Credits | Self::Lost | Self::Won => "E27 M4",
            Self::End => "",
        }
    }

    /// The endings the screen can have, in the order its driver lists them.
    #[must_use]
    pub const fn endings(self) -> &'static [Ended] {
        match self {
            Self::Start => &[Ended::Start],
            Self::MainMenu => &[
                Ended::MainMenu(Request::NewGame),
                Ended::MainMenu(Request::LoadGame),
                Ended::MainMenu(Request::HighScores),
                Ended::MainMenu(Request::Credits),
                Ended::MainMenu(Request::Quit),
            ],
            Self::GameType(_) => &[
                Ended::GameType(Choice::Single),
                Ended::GameType(Choice::Multiplayer),
                Ended::GameType(Choice::Campaign),
                Ended::GameType(Choice::Tutorial),
                Ended::GameType(Choice::Back),
            ],
            Self::Multiplayer(_) => &[Ended::Multiplayer(true), Ended::Multiplayer(false)],
            Self::Scenario(..) => &[Ended::Scenario(true), Ended::Scenario(false)],
            Self::Campaigns | Self::Tutorial => &[Ended::Back],
            Self::HighScores | Self::Credits | Self::Lost | Self::Won => &[Ended::Done],
            Self::Play { .. } => &[
                Ended::Play(PlayEnd::Request(Request::MainMenu)),
                Ended::Play(PlayEnd::Request(Request::NewGame)),
                Ended::Play(PlayEnd::Request(Request::LoadGame)),
                Ended::Play(PlayEnd::Request(Request::Restart)),
                Ended::Play(PlayEnd::Request(Request::Quit)),
                Ended::Play(PlayEnd::Lost),
                Ended::Play(PlayEnd::Won),
            ],
            Self::End => &[],
        }
    }
}

/// The shell: the screen on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Shell {
    screen: Screen,
}

impl Shell {
    /// The shell at its start: the clips when `intro` (the setting `Show Intro`, 0x6993C0; the
    /// client has no first run, CLT-Q85), else the main menu.
    #[must_use]
    pub const fn new(intro: bool) -> Self {
        Self {
            screen: if intro {
                Screen::Start
            } else {
                Screen::MainMenu
            },
        }
    }

    /// The screen on.
    #[must_use]
    pub const fn screen(&self) -> Screen {
        self.screen
    }

    /// The screen on ended with `ended`: the next screen, or [`Refused`] (the shell unchanged)
    /// when the screen cannot end so.
    ///
    /// # Errors
    /// An ending not among the screen's ([`Screen::endings`]).
    pub fn end(&mut self, ended: Ended) -> Result<Screen, Refused> {
        let refused = Refused {
            screen: self.screen,
            ended,
        };
        let next = match (self.screen, ended) {
            (Screen::Start, Ended::Start)
            | (Screen::HighScores | Screen::Credits | Screen::Lost, Ended::Done) => {
                Screen::MainMenu
            }
            (Screen::MainMenu, Ended::MainMenu(request))
                if !matches!(request, Request::Restart | Request::MainMenu) =>
            {
                taken(request)
            }
            (Screen::GameType(game), Ended::GameType(choice)) => game_type(game, choice),
            (Screen::Multiplayer(game), Ended::Multiplayer(true)) => {
                Screen::Scenario(game, Kind::Multiplayer)
            }
            (Screen::Multiplayer(game), Ended::Multiplayer(false))
            | (Screen::Scenario(game, _), Ended::Scenario(false)) => Screen::GameType(game),
            (Screen::Scenario(..), Ended::Scenario(true)) => Screen::Play { restart: false },
            (Screen::Campaigns | Screen::Tutorial, Ended::Back) => Screen::GameType(Game::New),
            (Screen::Won, Ended::Done) => Screen::HighScores,
            (Screen::Play { .. }, Ended::Play(PlayEnd::Request(request)))
                if !matches!(request, Request::HighScores | Request::Credits) =>
            {
                taken(request)
            }
            (Screen::Play { .. }, Ended::Play(PlayEnd::Lost)) => Screen::Lost,
            (Screen::Play { .. }, Ended::Play(PlayEnd::Won)) => Screen::Won,
            _ => return Err(refused),
        };
        self.screen = next;
        Ok(next)
    }
}

/// The main menu's switch (0x4EEC0D) on a command or a request (the loop's top takes play's at
/// once: none is left between).
const fn taken(request: Request) -> Screen {
    match request {
        Request::NewGame => Screen::GameType(Game::New),
        Request::LoadGame => Screen::GameType(Game::Load),
        Request::HighScores => Screen::HighScores,
        Request::Credits => Screen::Credits,
        Request::Quit => Screen::End,
        Request::Restart => Screen::Play { restart: true },
        Request::MainMenu => Screen::MainMenu,
    }
}

/// A game type menu's switch (New Game's 0x4EECF5, Load Game's 0x4EF02D).
const fn game_type(game: Game, choice: Choice) -> Screen {
    match (game, choice) {
        (_, Choice::Single) => Screen::Scenario(game, Kind::Single),
        (_, Choice::Multiplayer) => Screen::Multiplayer(game),
        (_, Choice::Back) => Screen::MainMenu,
        (Game::New, Choice::Campaign) => Screen::Campaigns,
        (Game::New, Choice::Tutorial) => Screen::Tutorial,
        (Game::Load, Choice::Campaign) => Screen::Scenario(game, Kind::Campaign),
        (Game::Load, Choice::Tutorial) => Screen::Scenario(game, Kind::Tutorial),
    }
}

#[cfg(test)]
mod tests;
