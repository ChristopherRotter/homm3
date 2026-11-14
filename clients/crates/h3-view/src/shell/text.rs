//! The shell's headless driver (E27 M2 T3a): the `h3-view shell` command reads a word a line, each
//! the ending of the screen on, and answers with the next screen: its name, the task that builds
//! it and the words it takes (a text client before the window, as the battle's `connect` came
//! before its window). The binary does the reading and the printing; the words are here.

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use super::{Choice, Ended, Game, Kind, PlayEnd, Request, Screen, Shell};

/// The screen's name.
#[must_use]
pub fn name(screen: Screen) -> String {
    let game = |game| match game {
        Game::New => "new game",
        Game::Load => "load game",
    };
    let kind = |kind| match kind {
        Kind::Single => "single",
        Kind::Campaign => "campaign",
        Kind::Multiplayer => "multiplayer",
        Kind::Tutorial => "tutorial",
    };
    match screen {
        Screen::Start => String::from("start"),
        Screen::MainMenu => String::from("main-menu"),
        Screen::GameType(Game::New) => String::from("new-game-menu"),
        Screen::GameType(Game::Load) => String::from("load-game-menu"),
        Screen::Multiplayer(g) => format!("multiplayer ({})", game(g)),
        Screen::Scenario(g, k) => format!("scenario ({}, {})", game(g), kind(k)),
        Screen::Campaigns => String::from("campaigns"),
        Screen::Tutorial => String::from("tutorial"),
        Screen::HighScores => String::from("high-scores"),
        Screen::Credits => String::from("credits"),
        Screen::Play { restart: false } => String::from("play"),
        Screen::Play { restart: true } => String::from("play (restart)"),
        Screen::Lost => String::from("lost"),
        Screen::Won => String::from("won"),
        Screen::End => String::from("end"),
    }
}

/// The word of an ending (unique among each screen's).
#[must_use]
pub const fn word(ended: Ended) -> &'static str {
    match ended {
        Ended::Start | Ended::Done => "done",
        Ended::MainMenu(request) | Ended::Play(PlayEnd::Request(request)) => request_word(request),
        Ended::GameType(Choice::Single) => "single",
        Ended::GameType(Choice::Multiplayer) => "multiplayer",
        Ended::GameType(Choice::Campaign) => "campaign",
        Ended::GameType(Choice::Tutorial) => "tutorial",
        Ended::GameType(Choice::Back) | Ended::Scenario(false) | Ended::Back => "back",
        Ended::Multiplayer(true) => "hot-seat",
        Ended::Multiplayer(false) => "cancel",
        Ended::Scenario(true) => "begin",
        Ended::Play(PlayEnd::Lost) => "lost",
        Ended::Play(PlayEnd::Won) => "won",
    }
}

const fn request_word(request: Request) -> &'static str {
    match request {
        Request::NewGame => "new-game",
        Request::LoadGame => "load-game",
        Request::HighScores => "high-scores",
        Request::Credits => "credits",
        Request::Quit => "quit",
        Request::Restart => "restart",
        Request::MainMenu => "main-menu",
    }
}

/// The screen described: its name, the task that builds it and the words it takes (the end's
/// name alone).
#[must_use]
pub fn describe(screen: Screen) -> String {
    if screen == Screen::End {
        return name(screen);
    }
    let words: Vec<&str> = screen.endings().iter().map(|&e| word(e)).collect();
    format!("{} [{}]: {}", name(screen), screen.task(), words.join(" "))
}

/// The ending of `screen` whose word is `line` (trimmed), if it has one.
#[must_use]
pub fn ending(screen: Screen, line: &str) -> Option<Ended> {
    let line = line.trim();
    screen.endings().iter().copied().find(|&e| word(e) == line)
}

/// One line read: a word of the screen on ends it (the next screen described); an empty line
/// describes the screen on; another word is refused with the words it takes.
pub fn answer(shell: &mut Shell, line: &str) -> String {
    let screen = shell.screen();
    if line.trim().is_empty() {
        return describe(screen);
    }
    match ending(screen, line).map(|e| shell.end(e)) {
        Some(Ok(next)) => describe(next),
        Some(Err(_)) | None => {
            let line = line.trim();
            format!("not on {}: {line} ({})", name(screen), describe(screen))
        }
    }
}

#[cfg(test)]
mod tests;
