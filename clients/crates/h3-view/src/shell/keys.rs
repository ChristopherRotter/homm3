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
//!   Escape its Cancel (520).
//! - **The scenario screen** (`scenario_dialog_create` 0x579960; E27 M3 T1a): S Show Available
//!   Scenarios (128), R Random Map (130), A Show Advanced Options (129), Enter Begin (186),
//!   Escape Back (188); its list's (`scenario_dialog_proc` 0x587D00, its scan codes 0x48, 0x50,
//!   0x49 and 0x51; E27 M3 T1b1): Up and Down the selection by a row, Page Up and Page Down the
//!   top by a page, at the key's press; the description's scroll bar's (E27 M3 T2c, its arrows'
//!   keys: 0x48 to 0x51) at the key's release: Up and Left a line back, Down and Right a line
//!   on, Page Up and Page Down a page.
//! - **An edit's own keys** (E27 M2 T7b, decision 39: the client's profile): Home, End, Left,
//!   Right, Delete and BackSpace, taken while an edit has the focus ([`Context::Edit`], asked
//!   before its dialog's); **the hot seat's ring** (`hot_seat_edit_key` 0x50DF60): Tab and Down to
//!   the next edit, Shift and Tab and Up to the previous. The keypad's navigation keys are left
//!   unbound (*provisional*: with Num Lock on they type their digits). In the original's profile
//!   the edits take their keys by scan code instead ([`super::multiplayer`], `h3_ui::edit`:
//!   [`scan`] reads a recipe's or the adapter's name so), these commands unused.
//!
//! A key down's command ([`command_for`]): the chord with Shift as held, else, with Shift held
//! and that chord unbound, the key alone (*provisional*: Shift and a hotkey pressed its button
//! before T7b too, when the shell read no Shift); Ctrl is not read. A user's file binding one
//! chord to an edit's key and to its dialog's is refused, as within one context (M2's review).
//!
//! The window's own keys (`h3_ui::keys::Window`: the music, fullscreen, …) are not the shell's
//! yet (*provisional*): their Escape is the main menu's Quit here.

use alloc::string::String;
use alloc::vec::Vec;

use h3_ui::edit::{self, Caret, Scan};
use h3_ui::keys::{self, Chord, Keys};

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
    /// An edit with the focus, in the client's profile (E27 M2 T7b): its own keys, asked before
    /// its dialog's.
    Edit,
    /// The scenario screen with nothing over it (E27 M3 T1a).
    Scenario,
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
    /// An edit's own key (E27 M2 T7b: the client's profile).
    Edit(Caret),
    /// The hot seat's ring: to the next edit (`true`) or the previous (E27 M2 T7b).
    Ring(bool),
    /// The scenario list's move at the key's press, the description's bar's step at its release
    /// (E27 M3 T1b1, T2c).
    List(Scroll),
}

/// A move of the scenario list by a key at its press (E27 M3 T1b1), and of the description's
/// scroll bar at its release (E27 M3 T2c: the bar's own keys, `scrollbar_key_step` 0x596230).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Scroll {
    /// The selection up a row (Up); the description a line back.
    Up,
    /// The selection down a row (Down); the description a line on.
    Down,
    /// The top up a page (Page Up); the description a page back.
    PageUp,
    /// The top down a page (Page Down); the description a page on.
    PageDown,
    /// The description a line back (Left); the list nothing.
    Left,
    /// The description a line on (Right); the list nothing.
    Right,
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
/// codes 1 and 0x1C; E27 M3 T1a, `scenario_dialog_create` 0x579960: the panel buttons' 0x1F, 0x13
/// and 0x1E, Begin's 0x1C, Back's 1) and their dialog.
pub const DIALOGS: [(u16, &str, &str, Context); 8] = [
    (124, "multiplayer-cancel", "Escape", Context::Multiplayer),
    (519, "hot-seat-ok", "Return", Context::HotSeat),
    (520, "hot-seat-cancel", "Escape", Context::HotSeat),
    (128, "scenario-list", "S", Context::Scenario),
    (130, "scenario-random", "R", Context::Scenario),
    (129, "scenario-advanced", "A", Context::Scenario),
    (186, "scenario-begin", "Return", Context::Scenario),
    (188, "scenario-back", "Escape", Context::Scenario),
];

/// The scenario list's keys (E27 M3 T1b1) and the description's bar's (T2c: Left and Right its
/// own), with their names and the original's keys by the adapter's names.
pub const LIST: [(Scroll, &str, &str); 6] = [
    (Scroll::Up, "scenario-up", "Up"),
    (Scroll::Down, "scenario-down", "Down"),
    (Scroll::PageUp, "scenario-page-up", "PageUp"),
    (Scroll::PageDown, "scenario-page-down", "PageDown"),
    (Scroll::Left, "scenario-left", "Left"),
    (Scroll::Right, "scenario-right", "Right"),
];

/// A message box's keys with their names and the original's (as the battle's,
/// `crate::keys`).
const MESSAGE: [(message::Key, &str, &str); 2] = [
    (message::Key::Ok, "box-ok", "Return"),
    (message::Key::Cancel, "box-cancel", "Escape"),
];

/// An edit's own keys (E27 M2 T7b), with their names and keys (the original's scan codes', by the
/// adapter's names).
pub const EDIT: [(Caret, &str, &str); 6] = [
    (Caret::Home, "edit-home", "Home"),
    (Caret::End, "edit-end", "End"),
    (Caret::Left, "edit-left", "Left"),
    (Caret::Right, "edit-right", "Right"),
    (Caret::Delete, "edit-delete", "Delete"),
    (Caret::BackSpace, "edit-backspace", "BackSpace"),
];

/// The hot seat's ring (E27 M2 T7b), with its names and the original's keys
/// (`hot_seat_edit_key` 0x50DF60).
pub const RING: [(bool, &str, [&str; 2]); 2] = [
    (true, "hot-seat-next", ["Tab", "Down"]),
    (false, "hot-seat-previous", ["Shift+Tab", "Up"]),
];

impl Command {
    /// Its name and its default chords.
    fn entry(self) -> (String, Vec<Chord>) {
        let one = |key: &str| alloc::vec![Chord::new(false, key)];
        let found = match self {
            Self::Button(id) => MAIN_MENU
                .iter()
                .find(|&&(b, _, _)| b == id)
                .map(|&(_, name, key)| (name, one(key))),
            Self::Choice(c) => GAME_TYPE
                .iter()
                .find(|&&(g, _, _)| g == c)
                .map(|&(_, name, key)| (name, one(key))),
            Self::Message(k) => MESSAGE
                .iter()
                .find(|&&(m, _, _)| m == k)
                .map(|&(_, name, key)| (name, one(key))),
            Self::Dialog(id) => DIALOGS
                .iter()
                .find(|&&(d, _, _, _)| d == id)
                .map(|&(_, name, key, _)| (name, one(key))),
            Self::Edit(c) => EDIT
                .iter()
                .find(|&&(e, _, _)| e == c)
                .map(|&(_, name, key)| (name, one(key))),
            Self::Ring(next) => RING
                .iter()
                .find(|&&(r, _, _)| r == next)
                .map(|(_, name, keys)| {
                    let chords = keys.iter().filter_map(|k| Chord::parse(k).ok()).collect();
                    (*name, chords)
                }),
            Self::List(scroll) => LIST
                .iter()
                .find(|&&(l, _, _)| l == scroll)
                .map(|&(_, name, key)| (name, one(key))),
        };
        let (name, chords) = found.unwrap_or(("", Vec::new()));
        (String::from(name), chords)
    }
}

impl keys::Command for Command {
    type Context = Context;
    const TABLE: &'static str = "shell";

    /// Every command, in the table's order: the main menu's buttons, the game type menus', the
    /// box's, the dialogs', an edit's own keys, the hot seat's ring, the scenario list's.
    fn all() -> Vec<Self> {
        let mut all: Vec<Self> = MAIN_MENU.map(|(id, _, _)| Self::Button(id)).into();
        all.extend(GAME_TYPE.map(|(c, _, _)| Self::Choice(c)));
        all.extend(MESSAGE.map(|(k, _, _)| Self::Message(k)));
        all.extend(DIALOGS.map(|(id, _, _, _)| Self::Dialog(id)));
        all.extend(EDIT.map(|(c, _, _)| Self::Edit(c)));
        all.extend(RING.map(|(next, _, _)| Self::Ring(next)));
        all.extend(LIST.map(|(scroll, _, _)| Self::List(scroll)));
        all
    }

    /// Its name in a user's file: `new-game`, `quit`, `single`, `box-ok`, …
    fn name(self) -> String {
        self.entry().0
    }

    /// The original's keys.
    fn defaults() -> Vec<(Self, Vec<Chord>)> {
        Self::all().into_iter().map(|c| (c, c.entry().1)).collect()
    }

    /// The main menu's buttons over the main menu, a game type menu's over it, the box's keys
    /// over a box, a dialog's over it, an edit's while it has the focus, the ring over the hot
    /// seat's names, the list's over the scenario screen.
    fn context(self) -> Option<Context> {
        Some(match self {
            Self::Button(_) => Context::MainMenu,
            Self::Choice(_) => Context::GameType,
            Self::Message(_) => Context::Question,
            Self::Dialog(id) => DIALOGS.iter().find(|&&(d, _, _, _)| d == id)?.3,
            Self::Edit(_) => Context::Edit,
            Self::Ring(_) => Context::HotSeat,
            Self::List(_) => Context::Scenario,
        })
    }

    /// An edit's own keys and its dialog's, the Multiplayer dialog's or the hot seat's: one key
    /// down asks both ([`command_for`] in `Front::contexts`' order), so a user's file binding a
    /// chord in both is refused (M2's review of E27: `edit-home = ["Tab"]` shadowed the ring).
    fn asked_together(a: Context, b: Context) -> bool {
        let dialog = |c| matches!(c, Context::Multiplayer | Context::HotSeat);
        a == b || (a == Context::Edit && dialog(b)) || (b == Context::Edit && dialog(a))
    }
}

/// The command a key down gives (E27 M2 T7b) where `contexts` take keys, the first first (an
/// edit's own keys before its dialog's): the key `key` (the adapter's name) with Shift as held;
/// with Shift held and no context binding that chord, the key alone (*provisional*).
#[must_use]
pub fn command_for(
    keys: &Keys<Command>,
    contexts: &[Context],
    shift: bool,
    key: &str,
) -> Option<Command> {
    let chord = Chord::new(false, key);
    let find = |chord: &Chord| {
        contexts
            .iter()
            .find_map(|&context| keys.chord_command(context, chord))
    };
    find(&chord.clone().with_shift(shift)).or_else(|| shift.then(|| find(&chord)).flatten())
}

/// A key's name as a recipe (`xdotool`) or the adapter gives it, `shift+` (any case) holding
/// Shift (E27 M2 T7b): whether Shift is held, and the key's own name.
#[must_use]
pub fn shifted(name: &str) -> (bool, &str) {
    match name.get(..6) {
        Some(prefix) if prefix.eq_ignore_ascii_case("shift+") => (true, &name[6..]),
        _ => (false, name),
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
    let (shift, key) = shifted(name);
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
