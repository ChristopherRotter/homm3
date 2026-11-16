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
//! - **The Multiplayer dialog** (`multiplayer_dialog_init` 0x50E050; E27 M2 T7): Escape its
//!   Cancel (124); **the hot seat's names** (`hot_seat_dialog_init` 0x511E20): Enter its OK (519),
//!   Escape its Cancel (520). Their edits take their keys first, by scan code
//!   ([`super::multiplayer`], `h3_ui::edit`: [`scan`] reads a recipe's or the adapter's name so).
//!
//! The window's own keys (`h3_ui::keys::Window`: the music, fullscreen, …) are not the shell's
//! yet (*provisional*): their Escape is the main menu's Quit here.

use alloc::string::String;
use alloc::vec::Vec;

use h3_ui::edit::{self, Scan};
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
    /// The Multiplayer dialog with nothing over it (E27 M2 T7).
    Multiplayer,
    /// The hot seat's names (E27 M2 T7).
    HotSeat,
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
    /// A dialog's button, as the main menu's (E27 M2 T7: the Multiplayer dialog's and the hot
    /// seat's, by id).
    Dialog(u16),
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

/// The dialogs' buttons with a hotkey (E27 M2 T7), by id, with their names, the original's keys
/// (`multiplayer_dialog_init` 0x50E050, `hot_seat_dialog_init` 0x511E20: the buttons' scan
/// codes 1 and 0x1C) and their dialog.
pub const DIALOGS: [(u16, &str, &str, Context); 3] = [
    (124, "multiplayer-cancel", "Escape", Context::Multiplayer),
    (519, "hot-seat-ok", "Return", Context::HotSeat),
    (520, "hot-seat-cancel", "Escape", Context::HotSeat),
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
            Self::Dialog(id) => DIALOGS
                .iter()
                .find(|&&(d, _, _, _)| d == id)
                .map(|&(_, name, key, _)| (name, key)),
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
        all.extend(DIALOGS.map(|(id, _, _, _)| Self::Dialog(id)));
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
    /// over a box, a dialog's over it.
    fn context(self) -> Option<Context> {
        Some(match self {
            Self::Button(_) => Context::MainMenu,
            Self::Choice(_) => Context::GameType,
            Self::Message(_) => Context::Question,
            Self::Dialog(id) => DIALOGS.iter().find(|&&(d, _, _, _)| d == id)?.3,
        })
    }
}

/// The names of the keys an edit reads besides the characters (E27 M2 T7: the adapter's and the
/// harness's `xdotool` names), with their scan codes.
const NAMED: [(&str, u8); 18] = [
    ("Escape", edit::ESCAPE),
    ("BackSpace", edit::BACKSPACE),
    ("Backspace", edit::BACKSPACE),
    ("Tab", edit::TAB),
    ("Return", edit::RETURN),
    ("space", 0x39),
    ("Space", 0x39),
    ("Home", edit::HOME),
    ("Up", edit::UP),
    ("Prior", 0x49),
    ("PageUp", 0x49),
    ("Left", edit::LEFT),
    ("Right", edit::RIGHT),
    ("End", edit::END),
    ("Down", edit::DOWN),
    ("Next", 0x51),
    ("PageDown", 0x51),
    ("Delete", edit::DELETE),
];

/// The key `name` names as the original's input manager reads it (E27 M2 T7): a recipe's
/// (`xdotool`'s) or the adapter's name, `shift+` (any case) holding Shift; a character by the US
/// key that types it (`h3_ui::edit::scan_of`: a letter without Shift as `xdotool key` gives it,
/// whatever its case); the function keys by their codes; none for another name.
#[must_use]
pub fn scan(name: &str) -> Option<Scan> {
    let (shift, key) = match name.get(..6) {
        Some(prefix) if prefix.eq_ignore_ascii_case("shift+") => (true, &name[6..]),
        _ => (false, name),
    };
    let code = if let Some(&(_, code)) = NAMED.iter().find(|(n, _)| *n == key) {
        code
    } else if let Some(n) = key.strip_prefix('F').and_then(|n| n.parse::<u8>().ok()) {
        match n {
            1..=10 => 0x3A + n,
            11 | 12 => 0x4C + n,
            _ => return None,
        }
    } else {
        let &[c] = key.as_bytes() else {
            return None;
        };
        edit::scan_of(c.to_ascii_lowercase())?.code
    };
    Some(Scan { code, shift })
}

#[cfg(test)]
mod tests;
