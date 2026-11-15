//! The main menu taking its inputs (E27 M2 T6a; `docs/epics/E27-design.md` §2.3, §13): its five
//! buttons' hover, press and release (`h3_ui::input`, decision 20), their hotkeys
//! ([`super::keys`]), the right press's help popup and Quit's question.
//!
//! - **The hover** (`main_menu_proc` 0x4FB710, a move: type 4): the item under the mouse against
//!   the last (`main_menu_hovered` 0x67FA60); on a change the five lose their state 0x10 and the
//!   new one gains it (drawn in its frame 3, `h3_ui::ui::Button::frame`). A button the mouse
//!   holds runs its own loop (`def_button_message` 0x45641C): the moves are its own, the hover
//!   does not follow them. *Recorded, not read* (`menu-main`'s `*-left`, `menu-quit`'s
//!   `cancelled`): once a hold ends (the release, a key's release) and once the question
//!   closes, the hover is the item under the pointer again, as at a move there.
//! - **A press** (`button_press` 0x456860): the button drawn pressed with its sample
//!   (`button.wav`, [`Did::sample`]); the command at the release over it (0x200/0xD), none off it;
//!   a hotkey's key down presses it, its key up clicks it.
//! - **The commands** (0x4FB9BF): New Game, Load Game, High Scores and Credits end the menu with
//!   their id ([`Did::ended`]); Quit asks GENRLTXT 70 in a box of style 2 (the videos paused
//!   meanwhile: nothing shown, CLT-Q86): its OK ends the menu with Quit, its Cancel stays.
//! - **A right press** on a button (0x4FB92D: the message's +0xC bit 0x200, subtype 0xE, the
//!   button not going down): a popup of style 4 with the button's `Help.txt` right-click text
//!   (`main_menu_help` 0x6A6C20), shown while the right button is held (`popup_run_held`
//!   0x602A40): the cursor hidden, the moves and keys dropped, a left press or release, or the
//!   right release, closing it. None while the mouse holds a button (its own loop drops it).
//!
//! - **The redraws** (recorded, `menu-main`'s `load-press`): a press, a hold's move off or on and
//!   a release redraw the button alone (`button_press`, 0x4563BB, 0x456467), the hover's change
//!   all five in their order (0x4FBAEB); Load Game's frames reach two rows into High Scores',
//!   so a pressed Load Game shows over High Scores until the next hover change. The client draws
//!   the five in their order always (CLT-Q126, decision 36: fixed, as CLT-Q118's); the original's
//!   ([`Bugs::Original`]) draws the button redrawn alone last, for the comparisons only.
//!
//! The boxes are laid out by the caller ([`Boxes`]: their texts are the user's tables'), framed in
//! red (the viewing player at the main menu, recorded at M1 T2: `menu-quit`, `menu-main`).
//! *Not built* (design §13): the first pass's disk check (GENRLTXT 708) and the CD notice, a
//! DirectPlay lobby's start; the menus' clip 33 draws nothing (CLT-Q86).

use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use h3_render::Bugs;
use h3_ui::input::{Held, Input};
use h3_ui::ui::{Dialog, Draw, Shown, State};

use super::Request;
use super::keys::{Command, Context};
use crate::message::{self, MessageBox, Open, Outcome};

/// The player whose colours frame the main menu's boxes: the viewing player there, red (0).
pub const BOX_PLAYER: u8 = 0;

/// The main menu's buttons, by id (101 to 105).
pub const BUTTONS: [u16; 5] = [101, 102, 103, 104, 105];
/// Quit's button.
pub const QUIT: u16 = 105;

/// The boxes the main menu shows, laid out ([`message::layout`]): Quit's question (GENRLTXT 70,
/// style 2) and each button's help (its `Help.txt` right-click text, style 4), by id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Boxes {
    /// Quit's question.
    pub question: MessageBox,
    /// The buttons' help popups.
    pub help: BTreeMap<u16, MessageBox>,
}

/// What is open over the menu.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Over {
    Nothing,
    /// A button's help popup, while the right button is held.
    Help(u16),
    /// Quit's question.
    Question(Box<Open>),
}

/// What an input did.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Did {
    /// What the menu shows changed: draw it again.
    pub changed: bool,
    /// A button went down: its press sample (`button.wav`).
    pub sample: bool,
    /// The menu ended with this command.
    pub ended: Option<Request>,
}

impl Did {
    const CHANGED: Self = Self {
        changed: true,
        sample: false,
        ended: None,
    };

    fn with(self, other: Self) -> Self {
        Self {
            changed: self.changed || other.changed,
            sample: self.sample || other.sample,
            ended: self.ended.or(other.ended),
        }
    }
}

/// The main menu on the screen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MainMenu {
    dialog: Dialog,
    shown: Shown,
    input: Input,
    hovered: Option<u16>,
    pointer: [i32; 2],
    over: Over,
    boxes: Boxes,
    bugs: Bugs,
    /// The button redrawn alone since the five were last (the original's profile draws it last).
    top: Option<u16>,
}

/// The button a hold names.
const fn held_id(held: &Held) -> u16 {
    match held {
        Held::Button { id, .. } | Held::Picture { id, .. } | Held::Key { id, .. } => *id,
    }
}

impl MainMenu {
    /// The menu as it opens (`main_menu_create`: none hovered, pressed or disabled), its dialog
    /// the layout's, the pointer at `pointer` (taken at the first move), in the profile `bugs`.
    #[must_use]
    pub fn new(dialog: Dialog, boxes: Boxes, pointer: [i32; 2], bugs: Bugs) -> Self {
        Self {
            dialog,
            shown: Shown::default(),
            input: Input::new(bugs),
            hovered: None,
            pointer,
            over: Over::Nothing,
            boxes,
            bugs,
            top: None,
        }
    }

    /// The menu opened again with the pointer at `pointer`: none hovered, nothing held or open.
    #[must_use]
    pub fn opened(&self, pointer: [i32; 2]) -> Self {
        Self::new(self.dialog.clone(), self.boxes.clone(), pointer, self.bugs)
    }

    /// Where the menu takes its keys: itself, or the question over it (the popup takes none).
    #[must_use]
    pub const fn context(&self) -> Context {
        match self.over {
            Over::Question(_) => Context::Question,
            Over::Nothing | Over::Help(_) => Context::MainMenu,
        }
    }

    /// The button hovered (its state 0x10), if any.
    #[must_use]
    pub const fn hovered(&self) -> Option<u16> {
        self.hovered
    }

    /// Whether the cursor shows (the help popup hides it).
    #[must_use]
    pub const fn cursor(&self) -> bool {
        !matches!(self.over, Over::Help(_))
    }

    /// Whether Quit's question is open.
    #[must_use]
    pub const fn asking(&self) -> bool {
        matches!(self.over, Over::Question(_))
    }

    /// The draw list: the buttons as they are, then the box open over them with its shadow; a
    /// player's colours from the players' palette `players`.
    #[must_use]
    pub fn draws(&self, players: &str) -> Vec<Draw> {
        let mut dialog = self.dialog.clone();
        if self.bugs == Bugs::Original
            && let Some(at) =
                (self.top).and_then(|id| dialog.items.iter().position(|i| i.id() == id))
        {
            let item = dialog.items.remove(at);
            dialog.items.push(item);
        }
        let mut list = dialog.draws(&self.shown, players);
        let open = match &self.over {
            Over::Nothing => None,
            Over::Help(id) => self.boxes.help.get(id),
            Over::Question(open) => Some(&open.message),
        };
        if let Some(m) = open {
            list.extend(m.dialog.draws(&m.shown, players));
        }
        list
    }

    /// The mouse moved to `point`.
    pub fn moved(&mut self, point: [i32; 2]) -> Did {
        self.pointer = point;
        match &mut self.over {
            Over::Help(_) => Did::default(),
            Over::Question(open) => changed(open.moved(point)),
            Over::Nothing if self.input.holds_button() => {
                let held = self.held();
                let did = self.input.moved(&self.dialog, &mut self.shown, point);
                self.alone(did.changed, held);
                Did {
                    changed: did.changed,
                    sample: did.button_sound,
                    ended: None,
                }
            }
            Over::Nothing => self.hover(),
        }
    }

    /// The left button pressed at `point`.
    pub fn press(&mut self, point: [i32; 2]) -> Did {
        self.pointer = point;
        match &mut self.over {
            Over::Help(_) => self.close_help(),
            Over::Question(open) => pressed(open.press(point)),
            Over::Nothing => {
                let did = self.input.press(&self.dialog, &mut self.shown, point);
                let held = self.held();
                self.alone(did.changed, held);
                Did {
                    changed: did.changed,
                    sample: did.button_sound,
                    ended: None,
                }
            }
        }
    }

    /// The left button released.
    pub fn release(&mut self) -> Did {
        match &mut self.over {
            Over::Help(_) => self.close_help(),
            Over::Question(open) => {
                let outcome = open.release();
                self.answered(outcome)
            }
            Over::Nothing => {
                let held = self.held();
                let did = self.input.release(&mut self.shown);
                self.alone(did.changed, held);
                self.after_hold(did.changed, did.clicked)
            }
        }
    }

    /// The right button pressed at `point`: a button's help, unless the mouse holds a button.
    pub fn right(&mut self, point: [i32; 2]) -> Did {
        self.pointer = point;
        if self.over != Over::Nothing {
            return Did::default();
        }
        match self.input.help_at(&self.dialog, point) {
            Some(id) if self.boxes.help.contains_key(&id) => {
                self.over = Over::Help(id);
                Did::CHANGED
            }
            _ => Did::default(),
        }
    }

    /// The right button released: the help popup closed; a button the mouse holds released as by
    /// the left button (its own loop, CLT-Q72 (a)), in the question's box too.
    pub fn right_release(&mut self) -> Did {
        match &mut self.over {
            Over::Help(_) => self.close_help(),
            Over::Question(open) if open.holds_mouse() => {
                let outcome = open.release();
                self.answered(outcome)
            }
            Over::Question(_) => Did::default(),
            Over::Nothing => {
                let held = self.held();
                let did = self.input.right_release(&mut self.shown);
                self.alone(did.changed, held);
                self.after_hold(did.changed, did.clicked)
            }
        }
    }

    /// A key down giving `command` (the key `by`, whose release clicks).
    pub fn key(&mut self, command: Command, by: &str) -> Did {
        match (&mut self.over, command) {
            (Over::Question(open), Command::Message(key)) => pressed(open.key(key, by)),
            (Over::Nothing, Command::Button(id)) if BUTTONS.contains(&id) => {
                let did = self.input.key(&mut self.shown, id, by);
                self.alone(did.changed, Some(id));
                Did {
                    changed: did.changed,
                    sample: did.button_sound,
                    ended: None,
                }
            }
            _ => Did::default(),
        }
    }

    /// The key `by` released.
    pub fn key_up(&mut self, by: &str) -> Did {
        match &mut self.over {
            // The popup takes the release: the button the key held goes without its click.
            Over::Help(_) => {
                let held = self.held();
                let did = self.input.let_go(&mut self.shown, Some(by));
                self.alone(did.changed, held);
                Did {
                    changed: did.changed,
                    ..Did::default()
                }
            }
            Over::Question(open) => {
                let outcome = open.key_up(by);
                self.answered(outcome)
            }
            Over::Nothing => {
                let held = self.held();
                let did = self.input.key_up(&mut self.shown, by);
                self.alone(did.changed, held);
                self.after_hold(did.changed, did.clicked)
            }
        }
    }

    /// The button the mouse or a key holds.
    fn held(&self) -> Option<u16> {
        self.input.held().map(held_id)
    }

    /// The button `id` redrawn alone when `changed`.
    fn alone(&mut self, changed: bool, id: Option<u16>) {
        if changed {
            self.top = id;
        }
    }

    /// The hover taken at the pointer (a move there); a change redraws the five in their order.
    fn hover(&mut self) -> Did {
        let now = self.dialog.button_at(self.pointer);
        if now == self.hovered {
            return Did::default();
        }
        self.hovered = now;
        self.top = None;
        for id in BUTTONS {
            let state = self.shown.state(id);
            self.shown.states.insert(
                id,
                State {
                    selected: Some(id) == now,
                    ..state
                },
            );
        }
        Did::CHANGED
    }

    /// A hold ended (`changed`: a button went up), with the click `clicked` if any: the hover
    /// taken again, then the command.
    fn after_hold(&mut self, changed: bool, clicked: Option<u16>) -> Did {
        let did = Did {
            changed,
            ..Did::default()
        }
        .with(self.hover());
        match clicked {
            Some(id) => did.with(self.command(id)),
            None => did,
        }
    }

    /// A button's command (0x4FB9BF): Quit asks its question, the others end the menu.
    fn command(&mut self, id: u16) -> Did {
        let ended = match id {
            101 => Request::NewGame,
            102 => Request::LoadGame,
            103 => Request::HighScores,
            104 => Request::Credits,
            _ => {
                self.over = Over::Question(Box::new(Open::new(self.boxes.question.clone())));
                return Did::CHANGED;
            }
        };
        Did {
            ended: Some(ended),
            ..Did::default()
        }
    }

    /// The question's box after an input: its OK ends the menu with Quit, its Cancel closes it
    /// (the hover taken again).
    fn answered(&mut self, outcome: Outcome) -> Did {
        match outcome {
            Outcome::Nothing => Did::default(),
            Outcome::Changed => Did::CHANGED,
            Outcome::Closed(message::YES) => Did {
                ended: Some(Request::Quit),
                ..Did::default()
            },
            Outcome::Closed(_) => {
                self.over = Over::Nothing;
                Did::CHANGED.with(self.hover())
            }
        }
    }

    /// The help popup closed.
    fn close_help(&mut self) -> Did {
        self.over = Over::Nothing;
        Did::CHANGED
    }
}

/// A box's outcome as the menu's: drawn again.
fn changed(outcome: Outcome) -> Did {
    Did {
        changed: outcome != Outcome::Nothing,
        ..Did::default()
    }
}

/// A box's button going down: drawn again with its press sample.
fn pressed(outcome: Outcome) -> Did {
    let down = outcome == Outcome::Changed;
    Did {
        changed: down,
        sample: down,
        ended: None,
    }
}

#[cfg(test)]
pub(crate) mod tests;
