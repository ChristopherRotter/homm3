//! A dialog's input (E27 M2 T3a, decision 20): the press, the hold and the release of a dialog's
//! DEF buttons and DEF pictures (its boxes and bars) by the mouse, its default button held by a
//! key, and the item whose help the right press asks for. One implementation for the battle's
//! dialogs and the shell's (CLT-Q102: "the code is shared, not built twice"); moved from the combat
//! options dialog (`h3_view::options`, E17 M6 T4a to M7 T2), which keeps its clicks.
//!
//! The original's rules, as read for the combat options (E17 design §12.12) and the system options
//! (E27 design §4.2):
//! - **A DEF button** runs its own loop from the press (`def_button_message` 0x456338 to
//!   0x456451): drawn pressed with its own press sample (`button_press` 0x456860), it follows the
//!   mouse off and on (pressed again with its sample as it comes back, 0x4563A2), and it clicks at
//!   the release over it (the left button's or the right's: CLT-Q72 (a)); the right press is
//!   dropped meanwhile, so no help opens over it. A disabled button takes no press.
//! - **A DEF picture** (a box or a bar) has no loop of its own: the press goes to the dialog's
//!   procedure (its sample is the dialog's), the release clicks it. In the original's profile at a
//!   release anywhere (`dialog_def_message` tests no box: the combat options' oddity 2, the system
//!   options' CLT-Q102); in the client's over it alone (CLT-Q43 (2), decision 20). A picture takes
//!   the mouse hidden too (its bit 2 alone: the combat options' hidden bars).
//! - **A key** (Enter or Escape for the default button; the key whose release clicks): the button
//!   goes down with its press sample (0x4565E7) and clicks at that key's release (0x45659F).
//! - **A release a loop of its own took** (the right press's help popup, `popup_run_held`
//!   0x602A40): what the mouse or that key holds goes without its click (CLT-Q39's rule, E17 M6
//!   T7b, R4-2); a button the mouse holds is never let go so, its own loop dropping the right
//!   press.
//!
//! The dialog's own effects (what a click does, a picture's press sample) stay with the dialog:
//! [`Did`] names the item.

use alloc::string::String;

use h3_render::Bugs;

use crate::ui::{Dialog, Item, Shown, State};

/// What holds an item down.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Held {
    /// A DEF button by the mouse, and whether the mouse is over it.
    Button {
        /// The button's control id.
        id: u16,
        /// The mouse is over it.
        over: bool,
    },
    /// A DEF picture (a box or a bar) by the mouse, and whether the mouse is over it.
    Picture {
        /// The picture's control id.
        id: u16,
        /// The mouse is over it.
        over: bool,
    },
    /// A button by a key.
    Key {
        /// The button's control id.
        id: u16,
        /// The key whose release clicks it (the adapter's name).
        by: String,
    },
}

/// What an input did.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Did {
    /// A button went down or up: draw the dialog again.
    pub changed: bool,
    /// A DEF button's own press sample (`button_press` 0x456860).
    pub button_sound: bool,
    /// The DEF picture the left button pressed (the press the dialog's procedure gets: its sample,
    /// if any, is the dialog's).
    pub pressed: Option<u16>,
    /// The item clicked (the procedure's 0x200/0xD with its id); a button is drawn up first.
    pub clicked: Option<u16>,
}

/// A dialog's input: what the mouse or a key holds.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Input {
    held: Option<Held>,
    /// The original's profile: a picture clicks at a release anywhere.
    anywhere: bool,
}

impl Input {
    /// Nothing held, in the profile `bugs` (the original's: a picture clicks at a release
    /// anywhere).
    #[must_use]
    pub fn new(bugs: Bugs) -> Self {
        Self {
            held: None,
            anywhere: bugs == Bugs::Original,
        }
    }

    /// What the mouse or a key holds, if anything.
    #[must_use]
    pub const fn held(&self) -> Option<&Held> {
        self.held.as_ref()
    }

    /// Whether the mouse holds a DEF button: its own loop runs until a release, the right press
    /// dropped.
    #[must_use]
    pub const fn holds_button(&self) -> bool {
        matches!(self.held, Some(Held::Button { .. }))
    }

    /// The left button pressed at `point` (on the screen): a DEF button there goes down with its
    /// press sample (not a disabled one), a DEF picture there is held unchanged (its press named
    /// for the dialog). Nothing while something is held.
    pub fn press(&mut self, dialog: &Dialog, shown: &mut Shown, point: [i32; 2]) -> Did {
        if self.held.is_some() {
            return Did::default();
        }
        let item = item_at(dialog, point).map(|i| (i.id(), matches!(i, Item::Button(_))));
        let Some((id, button)) = item else {
            return Did::default();
        };
        if !button {
            self.held = Some(Held::Picture { id, over: true });
            return Did {
                pressed: Some(id),
                ..Did::default()
            };
        }
        if shown.state(id).disabled {
            return Did::default();
        }
        self.held = Some(Held::Button { id, over: true });
        set_pressed(shown, id, true);
        Did {
            changed: true,
            button_sound: true,
            ..Did::default()
        }
    }

    /// The mouse moved to `point`: the DEF button it holds follows it off and on, pressed again
    /// with its sample as it comes back; the picture it holds notes whether the mouse is over it,
    /// nothing drawn.
    pub fn moved(&mut self, dialog: &Dialog, shown: &mut Shown, point: [i32; 2]) -> Did {
        let over_id = |id: u16| item_at(dialog, point).is_some_and(|i| i.id() == id);
        let (id, over) = match self.held {
            Some(Held::Picture { id, .. }) => {
                self.held = Some(Held::Picture {
                    id,
                    over: over_id(id),
                });
                return Did::default();
            }
            Some(Held::Button { id, over }) => (id, over),
            Some(Held::Key { .. }) | None => return Did::default(),
        };
        let now = over_id(id);
        if now == over {
            return Did::default();
        }
        self.held = Some(Held::Button { id, over: now });
        set_pressed(shown, id, now);
        Did {
            changed: true,
            button_sound: now,
            ..Did::default()
        }
    }

    /// The left button released: the DEF button held clicks when the mouse is still over it
    /// (drawn up first), else goes up; the picture held clicks when the mouse is over it, in the
    /// original's profile wherever it is.
    pub fn release(&mut self, shown: &mut Shown) -> Did {
        let anywhere = self.anywhere;
        match self.held.take_if(|h| !matches!(h, Held::Key { .. })) {
            Some(Held::Button { id, over }) => {
                set_pressed(shown, id, false);
                Did {
                    changed: true,
                    clicked: over.then_some(id),
                    ..Did::default()
                }
            }
            Some(Held::Picture { id, over }) => Did {
                clicked: (over || anywhere).then_some(id),
                ..Did::default()
            },
            _ => Did::default(),
        }
    }

    /// The right button released: it ends a DEF button's hold by the mouse as the left's release
    /// does (its own loop, `def_button_message` 0x456338; E17 M7 T2, CLT-Q72 (a)); a picture the
    /// mouse holds (no loop of its own) and a button its key holds are kept.
    pub fn right_release(&mut self, shown: &mut Shown) -> Did {
        if self.holds_button() {
            self.release(shown)
        } else {
            Did::default()
        }
    }

    /// A key down: button `id` goes down with its press sample, held by the key `by` (whose
    /// release clicks it). Nothing while something is held.
    pub fn key(&mut self, shown: &mut Shown, id: u16, by: &str) -> Did {
        if self.held.is_some() {
            return Did::default();
        }
        self.held = Some(Held::Key {
            id,
            by: String::from(by),
        });
        set_pressed(shown, id, true);
        Did {
            changed: true,
            button_sound: true,
            ..Did::default()
        }
    }

    /// A key up: the button key `by` held clicks, drawn up first.
    pub fn key_up(&mut self, shown: &mut Shown, by: &str) -> Did {
        match self
            .held
            .take_if(|h| matches!(h, Held::Key { by: key, .. } if key == by))
        {
            Some(Held::Key { id, .. }) => {
                set_pressed(shown, id, false);
                Did {
                    changed: true,
                    clicked: Some(id),
                    ..Did::default()
                }
            }
            _ => Did::default(),
        }
    }

    /// The item whose help the right button pressed at `point` asks for: a DEF button or a DEF
    /// picture there, whatever its state; none while the mouse holds a DEF button (its own loop
    /// drops the right press: `def_button_message` 0x456338 to 0x45641C; E17 M6 T7b, R4-2).
    #[must_use]
    pub fn help_at(&self, dialog: &Dialog, point: [i32; 2]) -> Option<u16> {
        if self.holds_button() {
            return None;
        }
        item_at(dialog, point).map(Item::id)
    }

    /// Lets what the mouse (`by` none) or the key `by` holds go without its click, a key's button
    /// drawn up again: the release the right button's help popup took. A DEF button the mouse
    /// holds is never let go here.
    pub fn let_go(&mut self, shown: &mut Shown, by: Option<&str>) -> Did {
        let held = self.held.take_if(|h| match h {
            Held::Key { by: key, .. } => by == Some(key.as_str()),
            Held::Picture { .. } => by.is_none(),
            Held::Button { .. } => false,
        });
        let Some(Held::Key { id, .. }) = held else {
            return Did::default();
        };
        set_pressed(shown, id, false);
        Did {
            changed: true,
            ..Did::default()
        }
    }
}

/// The item at `point` (on the screen) of the kinds that take the mouse here: a DEF button or a
/// DEF picture, hidden ones too (a DEF picture takes the mouse by its bit 2 alone).
fn item_at(dialog: &Dialog, point: [i32; 2]) -> Option<&Item> {
    dialog
        .item_at(point)
        .filter(|i| matches!(i, Item::Button(_) | Item::Def(_)))
}

/// Item `id` drawn pressed or released in `shown`, its other state bits kept.
fn set_pressed(shown: &mut Shown, id: u16, pressed: bool) {
    let state = shown.state(id);
    shown.states.insert(id, State { pressed, ..state });
}

#[cfg(test)]
mod tests;
