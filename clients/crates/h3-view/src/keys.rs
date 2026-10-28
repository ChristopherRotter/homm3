//! The battle window's commands (E24 M3 T1b, *provisional*; the user, 2026-10-26: no key hard-coded
//! in the client), whose keys go through the common table (`h3_ui::keys`: the chords, a user's
//! file's `[battle]` table, the lookup). The window's own are both viewers' (`h3_ui::keys::Window`:
//! the function keys and Esc, the user at E24 M3 T0); the battle's defaults are the original's
//! hotkeys, by where the battle takes them ([`Context`]): with nothing over it the log's arrows, the
//! Faerie Dragon's F and the panel's buttons' letters; the spell book's (`book::Key`), a message
//! box's (`message::Key`) and an aim's while one is open. Esc quits only with nothing over the
//! battle: over it, it is the book's close, the box's Cancel and the aim's give-up.

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use h3_ui::keys::{self, Chord, Window};

use crate::{book, message, panel};

/// The battle window's table of keys.
pub type Keys = keys::Keys<Command>;

/// Where the battle takes a key: what is open over it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Context {
    /// Nothing: the battle's own keys.
    Battle,
    /// The hero's spell book (E17 M5 T3a).
    Book,
    /// A message box (E17 M5 T4a), over the book too.
    Message,
    /// A spell aimed (E17 M5 T3b).
    Aim,
}

impl Context {
    /// What takes the keys: a message box when one is open, else the book, else an aim, else the
    /// battle.
    #[must_use]
    pub const fn of(message: bool, book: bool, aim: bool) -> Self {
        if message {
            Self::Message
        } else if book {
            Self::Book
        } else if aim {
            Self::Aim
        } else {
            Self::Battle
        }
    }
}

/// What a key does.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Command {
    /// The window's own.
    Window(Window),
    /// The log one line up (E17 M3 T6c-2).
    LogUp,
    /// The log one line down.
    LogDown,
    /// The Faerie Dragon's drawn spell aimed (E17 M5 T3b).
    DragonSpell,
    /// A panel button pressed while the key is down and clicked as it comes up (its id,
    /// `panel`'s: one of [`BUTTONS`]).
    Button(u16),
    /// The spell book's.
    Book(book::Key),
    /// A message box's.
    Message(message::Key),
    /// The aim given up.
    AimCancel,
}

/// The panel's buttons with a hotkey, by id, with their names and the original's keys
/// (`battle_panel_base` 0x46B610, each button's +0x48: the scan codes 0x1F S, 0x13 R, 0x18 O,
/// 0x1E A, 0x2E C, 0x11 W, 0x20 D and 0x39 Space; a key down presses it, the key up clicks,
/// 0x45659F, 0x4564D6).
pub const BUTTONS: [(u16, &str, &[&str]); 7] = [
    (panel::SURRENDER, "surrender", &["S"]),
    (panel::RETREAT, "retreat", &["R"]),
    (panel::OPTIONS, "options", &["O"]),
    (panel::AUTO_COMBAT, "auto-combat", &["A"]),
    (panel::SPELL_BOOK, "spell-book", &["C"]),
    (panel::WAIT, "wait", &["W"]),
    (panel::DEFEND, "defend", &["D", "Space"]),
];

/// The book's keys with their names and the original's keys (`spell_book_proc` 0x59D1B4, by the
/// screens' scan codes).
const BOOK: [(book::Key, &str, &[&str]); 7] = [
    (book::Key::Close, "book-close", &["Escape", "Return"]),
    (book::Key::Adventure, "book-adventure", &["A"]),
    (book::Key::Combat, "book-combat", &["C"]),
    (book::Key::Previous, "book-previous", &["Left"]),
    (book::Key::Next, "book-next", &["Right"]),
    (book::Key::TabUp, "book-tab-up", &["Up"]),
    (book::Key::TabDown, "book-tab-down", &["Down"]),
];

/// A message box's keys with their names and the original's (its buttons' hotkeys,
/// `message_box_build` 0x4F6990).
const MESSAGE: [(message::Key, &str, &[&str]); 2] = [
    (message::Key::Ok, "box-ok", &["Return"]),
    (message::Key::Cancel, "box-cancel", &["Escape"]),
];

impl Command {
    /// Its name and its default keys.
    fn entry(self) -> (String, Vec<Chord>) {
        let plain = |keys: &[&str]| keys.iter().map(|k| Chord::new(false, k)).collect();
        let (name, keys): (String, Vec<Chord>) = match self {
            Self::Window(w) => (w.name().into(), plain(&[w.key()])),
            // The battle's own keys (0x4758FB: the scan codes 72 and 80).
            Self::LogUp => ("log-up".into(), plain(&["Up"])),
            Self::LogDown => ("log-down".into(), plain(&["Down"])),
            // 0x475C17.
            Self::DragonSpell => ("dragon-spell".into(), plain(&["F"])),
            Self::Button(id) => BUTTONS.iter().find(|&&(b, _, _)| b == id).map_or_else(
                || (format!("button-{id}"), Vec::new()),
                |&(_, name, keys)| (name.into(), plain(keys)),
            ),
            Self::Book(k) => BOOK
                .iter()
                .find(|&&(b, _, _)| b == k)
                .map(|&(_, name, keys)| (name.into(), plain(keys)))
                .unwrap_or_default(),
            Self::Message(k) => MESSAGE
                .iter()
                .find(|&&(m, _, _)| m == k)
                .map(|&(_, name, keys)| (name.into(), plain(keys)))
                .unwrap_or_default(),
            // Escape (0x5A317A) or the right button (0x5A3184) in the aim's loop 0x5A3070.
            Self::AimCancel => ("aim-cancel".into(), plain(&["Escape", "RightButton"])),
        };
        (name, keys)
    }
}

impl keys::Command for Command {
    type Context = Context;
    const TABLE: &'static str = "battle";

    /// Every command, in the table's order: the window's own, the log's, the dragon's, the panel's
    /// buttons, the book's, the box's, the aim's.
    fn all() -> Vec<Self> {
        let mut all: Vec<Self> = Window::ALL.map(Self::Window).into();
        all.extend([Self::LogUp, Self::LogDown, Self::DragonSpell]);
        all.extend(BUTTONS.map(|(id, _, _)| Self::Button(id)));
        all.extend(BOOK.map(|(k, _, _)| Self::Book(k)));
        all.extend(MESSAGE.map(|(k, _, _)| Self::Message(k)));
        all.push(Self::AimCancel);
        all
    }

    /// Its name in a user's file: `pause`, `log-up`, `surrender`, `book-close`, `box-ok`, …
    fn name(self) -> String {
        self.entry().0
    }

    /// The window's own keys and the original's.
    fn defaults() -> Vec<(Self, Vec<Chord>)> {
        Self::all().into_iter().map(|c| (c, c.entry().1)).collect()
    }

    /// Everywhere the window's own that work whatever is open; with nothing over the battle quit,
    /// a frame on, restart, the log's, the dragon's and the buttons'; the book's, the box's and
    /// the aim's in theirs.
    fn context(self) -> Option<Context> {
        match self {
            Self::Window(w) if w.everywhere() => None,
            Self::Window(_) | Self::LogUp | Self::LogDown | Self::DragonSpell | Self::Button(_) => {
                Some(Context::Battle)
            }
            Self::Book(_) => Some(Context::Book),
            Self::Message(_) => Some(Context::Message),
            Self::AimCancel => Some(Context::Aim),
        }
    }
}

#[cfg(test)]
mod tests;
