//! The map window's commands (E24 M2 T4a, *provisional*; the user, 2026-10-26: no key hard-coded
//! in the client), whose keys go through the common table (`h3_ui::keys`, E24 M3 T1a: the chords,
//! a user's file, the lookup; its `[map]` table since T1b). Their defaults are the original's keys
//! on the adventure map (`docs/epics/E24-design.md` §3.4: Ctrl and a direction key scroll the view,
//! a direction key alone steps the selected hero; the keypad's eight give the same scan codes in
//! either NumLock state, §3.10) and the window's own (`h3_ui::keys::Window`, the battle's too: the
//! music, the effects, pause, a frame on, restart, vsync, the cursor's kind, fullscreen, quit),
//! which take the function keys and Esc, keys the original's map leaves to nothing else (pause, a
//! frame on, restart and vsync with Ctrl since E17 M6 T4b2a, the user, 2026-11-04: the original's
//! battle takes F5 to F8 alone, and both windows' own keys stay the same); since
//! E24 M3 T6b1 the panels' buttons' hotkeys (the original's letters K, U, Q, Z, W, M, C, A, O, H
//! and E, design §4.13); since T6b3 Space (the selected hero chosen again and its tile visited,
//! design §4.15); since E24 M4 T1a a message box's OK and Cancel (Enter and Escape, in the box's
//! own [`Context`], the battle's way: Escape is the window's quit with no box open); since E24 M4
//! T2c a level-up's offers (the keys 1 and 2, in the box's context too).

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use h3_ui::keys::{self, Chord, Window};
use h3_ui::message::Key;

use crate::timeline::{Action, Direction, Hotkey};

/// The map window's table of keys.
pub type Keys = keys::Keys<Command>;

/// Where the map takes a key (E24 M4 T1a): what is open over it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Context {
    /// Nothing: the map's own keys and the window's.
    Map,
    /// A message box: its buttons' keys, and the window's own that work everywhere.
    Message,
}

impl Context {
    /// A message box's when one is open, else the map's.
    #[must_use]
    pub const fn of(message: bool) -> Self {
        if message { Self::Message } else { Self::Map }
    }
}

/// A message box's keys.
const MESSAGE: [Key; 2] = [Key::Ok, Key::Cancel];

/// A box key's name and the original's key (its button's hotkey, `message_box_build` 0x4F6990),
/// as the battle's table names them.
const fn message_key(key: Key) -> (&'static str, &'static str) {
    match key {
        Key::Ok => ("box-ok", "Return"),
        Key::Cancel => ("box-cancel", "Escape"),
    }
}

/// A level-up's offer's key name and the original's key (scan codes 2 and 3): the left's 1, the
/// right's 2.
const fn offer_key(n: usize) -> (&'static str, &'static str) {
    if n == 0 {
        ("offer-left", "1")
    } else {
        ("offer-right", "2")
    }
}

/// What a key does.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Command {
    /// The view one scroll step that way (the original's Ctrl and a direction key).
    Scroll(Direction),
    /// The selected hero one step that way (the original's direction key alone).
    Hero(Direction),
    /// A panel button's hotkey (E24 M3 T6b1, design §4.13): its key down presses the button, its
    /// key up clicks it.
    Panel(Hotkey),
    /// The selected hero chosen again and the object it stands on visited (the original's Space,
    /// E24 M3 T6b3).
    Visit,
    /// A message box's OK or Cancel (E24 M4 T1a): its key down presses the button, its key up
    /// clicks it.
    Message(Key),
    /// A level-up's left or right offer chosen (E24 M4 T2c, design §5.5: the original's 1 and 2,
    /// `level_up_dialog_proc` 0x4F9780), 0 or 1.
    Offer(usize),
    /// The window's own (the music and the effects since E24 M2 T5, the cursor's kind since T4b).
    Window(Window),
}

/// The original's direction keys by the adapter's names, north clockwise to north-west: the scan
/// codes 72, 73, 77, 81, 80, 79, 75, 71 (`adventure_key_down` 0x408C40).
const DIRECTION_KEYS: [&str; 8] = [
    "Up", "PageUp", "Right", "PageDown", "Down", "End", "Left", "Home",
];

/// The keypad's keys giving the same scan codes, in the same order: the message's code is
/// lParam's bits 16 to 23 (`input_key_message` 0x4EC0E0), the keypad's in either NumLock state.
const KEYPAD_KEYS: [&str; 8] = [
    "Keypad8", "Keypad9", "Keypad6", "Keypad3", "Keypad2", "Keypad1", "Keypad4", "Keypad7",
];

impl Command {
    /// The map's action (the timeline's) a scroll or a hero's step gives; none for the window's
    /// own commands.
    #[must_use]
    pub fn action(self) -> Option<Action> {
        match self {
            Self::Scroll(d) => Some(Action::Scroll(d)),
            Self::Hero(d) => Some(Action::Step(d)),
            Self::Panel(key) => Some(Action::Key(key)),
            Self::Visit => Some(Action::Visit),
            Self::Message(key) => Some(Action::BoxKey(key)),
            Self::Offer(n) => Some(Action::Offer(n)),
            Self::Window(_) => None,
        }
    }

    /// The map's action its key's release gives: a hotkey's key up (E24 M3 T6b1); for the map's
    /// other keys a message in the input queue that does nothing (E24 M3 T6b3: it counts in the
    /// ring); none for the window's own.
    #[must_use]
    pub const fn release(self) -> Option<Action> {
        match self {
            Self::Panel(key) => Some(Action::KeyUp(key)),
            Self::Message(key) => Some(Action::BoxKeyUp(key)),
            Self::Scroll(_) | Self::Hero(_) | Self::Visit | Self::Offer(_) => Some(Action::Up),
            Self::Window(_) => None,
        }
    }
}

impl keys::Command for Command {
    /// The map's or a message box's (E24 M4 T1a).
    type Context = Context;
    const TABLE: &'static str = "map";

    /// Every command, in the table's order: the scroll's eight directions, the hero's eight, the
    /// panels' hotkeys, Space's, the box's, a level-up's offers', the window's own.
    fn all() -> Vec<Self> {
        let mut all: Vec<Self> = Direction::ALL.into_iter().map(Self::Scroll).collect();
        all.extend(Direction::ALL.into_iter().map(Self::Hero));
        all.extend(Hotkey::ALL.map(Self::Panel));
        all.push(Self::Visit);
        all.extend(MESSAGE.map(Self::Message));
        all.extend([0, 1].map(Self::Offer));
        all.extend(Window::ALL.map(Self::Window));
        all
    }

    /// Its name in a user's file: `scroll-north`, `hero-south-west`, `next-hero`, `pause`, …
    fn name(self) -> String {
        match self {
            Self::Scroll(d) => format!("scroll-{}", direction_name(d)),
            Self::Hero(d) => format!("hero-{}", direction_name(d)),
            Self::Panel(key) => hotkey_name(key).into(),
            Self::Visit => "visit".into(),
            Self::Message(key) => message_key(key).0.into(),
            Self::Offer(n) => offer_key(n).0.into(),
            Self::Window(w) => w.name().into(),
        }
    }

    /// The original's keys on the map (a direction's key and the keypad's, with Ctrl for the
    /// scroll; the panels' letters, E24 M3 T6b1; Space, T6b3; a box's Enter and Escape, E24 M4
    /// T1a; a level-up's 1 and 2, T2c) and the window's own.
    fn defaults() -> Vec<(Self, Vec<Chord>)> {
        let mut defaults = Vec::new();
        for ((d, key), pad) in Direction::ALL
            .into_iter()
            .zip(DIRECTION_KEYS)
            .zip(KEYPAD_KEYS)
        {
            let chords = |ctrl| Vec::from([key, pad].map(|k| Chord::new(ctrl, k)));
            defaults.push((Self::Scroll(d), chords(true)));
            defaults.push((Self::Hero(d), chords(false)));
        }
        for key in Hotkey::ALL {
            defaults.push((
                Self::Panel(key),
                Vec::from([Chord::new(false, key.letter())]),
            ));
        }
        defaults.push((Self::Visit, Vec::from([Chord::new(false, "Space")])));
        for key in MESSAGE {
            let chord = Chord::new(false, message_key(key).1);
            defaults.push((Self::Message(key), Vec::from([chord])));
        }
        for n in [0, 1] {
            let chord = Chord::new(false, offer_key(n).1);
            defaults.push((Self::Offer(n), Vec::from([chord])));
        }
        for w in Window::ALL {
            defaults.push((Self::Window(w), Vec::from([w.chord()])));
        }
        defaults
    }

    /// The window's own that work whatever is open everywhere; the box's and a level-up's offers'
    /// in the box's context;
    /// with nothing over the map the rest (quit, a frame on and restart among them: Escape is the
    /// box's Cancel while one is open).
    fn context(self) -> Option<Context> {
        match self {
            Self::Window(w) if w.everywhere() => None,
            Self::Message(_) | Self::Offer(_) => Some(Context::Message),
            Self::Scroll(_) | Self::Hero(_) | Self::Panel(_) | Self::Visit | Self::Window(_) => {
                Some(Context::Map)
            }
        }
    }
}

/// Every chord of `keys` the map can tell apart (the client lane's E27 M2 T7b note): the map reads
/// no Shift, as the original's adventure map, whose keys go by their scan code and Ctrl alone
/// (`adventure_key_down` 0x408C40's modifiers 0xC; the hotkeys by scan code, design §4.13), so a
/// key with Shift held gives what the key alone gives and a `Shift+` chord, which `h3_ui::keys`
/// parses, would never match: refused.
///
/// # Errors
/// The first `Shift+` chord, with its command.
pub fn refuse_shift(table: &Keys) -> Result<(), String> {
    use keys::Command as _;
    table
        .chords()
        .into_iter()
        .find(|(_, chord)| chord.shift)
        .map_or(Ok(()), |(command, chord)| {
            Err(format!(
                "`{}`: {chord}: the map reads no Shift (a key with Shift held is the key alone)",
                command.name()
            ))
        })
}

/// A panel's hotkey's name in a user's file.
const fn hotkey_name(key: Hotkey) -> &'static str {
    match key {
        Hotkey::Kingdom => "kingdom-overview",
        Hotkey::Underground => "underground",
        Hotkey::Quests => "quest-log",
        Hotkey::Sleep => "sleep",
        Hotkey::Wake => "wake",
        Hotkey::MoveHero => "move-hero",
        Hotkey::Cast => "cast-spell",
        Hotkey::Adventure => "adventure-options",
        Hotkey::System => "system-options",
        Hotkey::NextHero => "next-hero",
        Hotkey::EndTurn => "end-turn",
    }
}

/// A direction's part of a command's name.
fn direction_name(d: Direction) -> &'static str {
    match d {
        Direction::North => "north",
        Direction::NorthEast => "north-east",
        Direction::East => "east",
        Direction::SouthEast => "south-east",
        Direction::South => "south",
        Direction::SouthWest => "south-west",
        Direction::West => "west",
        Direction::NorthWest => "north-west",
    }
}

#[cfg(test)]
mod tests;
