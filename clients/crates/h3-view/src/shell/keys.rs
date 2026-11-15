//! The shell's keys (E27 M2 T6a; `h3_ui::keys`: no key hard-coded in the client): its menus'
//! hotkeys and its boxes' keys, by command, in the table `[shell]` of a user's key file, their
//! defaults the original's. A hotkey's key down presses its button (drawn pressed with its
//! sample), its key up clicks it (`def_button_message` 0x45659F, 0x4564D6).
//!
//! - **The main menu** (`main_menu_create` 0x4FB2A0, the buttons' +0x48 scan codes): N New Game
//!   (0x31), L Load Game (0x26), H High Scores (0x23), C Credits (0x2E), Escape Quit (1).
//! - **New Game's and Load Game's menus** (`game_type_menu_create` 0x4D54C0; E27 M2 T6b1): S
//!   Single (0x1F), M Multiplayer (0x32), C Campaign (0x2E), T Tutorial (0x14), Escape Back (1).
//! - **A message box over a screen** (Quit's question, §2.3): its buttons' hotkeys
//!   (`message_box_build` 0x4F6990), Enter its OK and Escape its Cancel, as the battle's box.
//!
//! The window's own keys (`h3_ui::keys::Window`: the music, fullscreen, …) are not the shell's
//! yet (*provisional*): their Escape is the main menu's Quit here.

use alloc::string::String;
use alloc::vec::Vec;

use h3_ui::keys::{self, Chord};

use super::Choice;
use crate::message;

/// Where the shell takes its keys: the screen on, or a box over it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Context {
    /// The main menu with nothing over it.
    MainMenu,
    /// New Game's or Load Game's menu with nothing over it (E27 M2 T6b1).
    GameType,
    /// A message box over a screen (Quit's question).
    Question,
}

/// The shell's commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Command {
    /// The main menu's button pressed while the key is down and clicked as it comes up (its id).
    Button(u16),
    /// A game type menu's button, as the main menu's (E27 M2 T6b1).
    Choice(Choice),
    /// A message box's.
    Message(message::Key),
}

/// The main menu's buttons with a hotkey, by id, with their names and the original's keys.
pub const MAIN_MENU: [(u16, &str, &str); 5] = [
    (101, "new-game", "N"),
    (102, "load-game", "L"),
    (103, "high-scores", "H"),
    (104, "credits", "C"),
    (105, "quit", "Escape"),
];

/// The game type menus' buttons with a hotkey, with their names and the original's keys.
pub const GAME_TYPE: [(Choice, &str, &str); 5] = [
    (Choice::Single, "single", "S"),
    (Choice::Multiplayer, "multiplayer", "M"),
    (Choice::Campaign, "campaign", "C"),
    (Choice::Tutorial, "tutorial", "T"),
    (Choice::Back, "back", "Escape"),
];

/// A message box's keys with their names and the original's (as the battle's,
/// `crate::keys`).
const MESSAGE: [(message::Key, &str, &str); 2] = [
    (message::Key::Ok, "box-ok", "Return"),
    (message::Key::Cancel, "box-cancel", "Escape"),
];

impl Command {
    /// Its name and its default key.
    fn entry(self) -> (String, &'static str) {
        let found = match self {
            Self::Button(id) => MAIN_MENU
                .iter()
                .find(|&&(b, _, _)| b == id)
                .map(|&(_, name, key)| (name, key)),
            Self::Choice(c) => GAME_TYPE
                .iter()
                .find(|&&(g, _, _)| g == c)
                .map(|&(_, name, key)| (name, key)),
            Self::Message(k) => MESSAGE
                .iter()
                .find(|&&(m, _, _)| m == k)
                .map(|&(_, name, key)| (name, key)),
        };
        let (name, key) = found.unwrap_or(("", ""));
        (String::from(name), key)
    }
}

impl keys::Command for Command {
    type Context = Context;
    const TABLE: &'static str = "shell";

    /// Every command, in the table's order: the main menu's buttons, the game type menus', the
    /// box's.
    fn all() -> Vec<Self> {
        let mut all: Vec<Self> = MAIN_MENU.map(|(id, _, _)| Self::Button(id)).into();
        all.extend(GAME_TYPE.map(|(c, _, _)| Self::Choice(c)));
        all.extend(MESSAGE.map(|(k, _, _)| Self::Message(k)));
        all
    }

    /// Its name in a user's file: `new-game`, `quit`, `single`, `box-ok`, …
    fn name(self) -> String {
        self.entry().0
    }

    /// The original's keys.
    fn defaults() -> Vec<(Self, Vec<Chord>)> {
        Self::all()
            .into_iter()
            .map(|c| (c, alloc::vec![Chord::new(false, c.entry().1)]))
            .collect()
    }

    /// The main menu's buttons over the main menu, a game type menu's over it, the box's keys
    /// over a box.
    fn context(self) -> Option<Context> {
        Some(match self {
            Self::Button(_) => Context::MainMenu,
            Self::Choice(_) => Context::GameType,
            Self::Message(_) => Context::Question,
        })
    }
}

#[cfg(test)]
mod tests;
