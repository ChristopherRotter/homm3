//! Dialog items as reusable components (E17 M3 T6a; `docs/epics/E17-design.md` §9.7). The
//! executable builds every screen's controls from a few kinds of dialog item, each drawn by its
//! kind's own drawer whatever the screen: a picture (H3API's `H3DlgPcx`), a button of DEF frames
//! (`H3DlgDefButton`, and its kin `H3DlgCustomButton` and `H3DlgCaptionButton`, which share its
//! drawer 0x456940) and a text over a picture (`H3DlgTextPcx`). A layout file lists a dialog's
//! items in the order the executable adds them, which is the order they are drawn, each with the
//! executable's control id; a screen's logic sets the items' states by id ([`Shown`]), and
//! [`Dialog::draws`] turns both into the draw list. The battle's panel is the first dialog
//! (`crate::panel`). The format is *provisional* (the M3 review list).

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use serde::Deserialize;

use crate::scene::{Draw, PlayerColours};

/// A dialog: its corner on the screen and its items.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Dialog {
    /// The dialog's top-left corner on the screen; each item's place is from it.
    pub place: [i32; 2],
    /// Its items, in the order the executable adds them: the order they are drawn.
    pub items: Vec<Item>,
}

/// A dialog item: its kind and its fields.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Item {
    /// A picture (`H3DlgPcx`).
    Picture(Picture),
    /// A button of DEF frames (`H3DlgDefButton` and its kin).
    Button(Button),
    /// A text over a picture (`H3DlgTextPcx`).
    Text(Text),
}

impl Item {
    /// Where the item takes the mouse, from the dialog's corner: its corner and size (a picture
    /// takes none: *provisional*, design §9.9).
    #[must_use]
    pub const fn area(&self) -> Option<([i32; 2], [i32; 2])> {
        match self {
            Self::Picture(_) => None,
            Self::Button(b) => Some((b.at, b.size)),
            Self::Text(t) => Some((t.at, t.size)),
        }
    }

    /// The key of the item's hint, if it has one.
    #[must_use]
    pub fn hint(&self) -> Option<&str> {
        match self {
            Self::Picture(_) => None,
            Self::Button(b) => b.hint.as_deref(),
            Self::Text(t) => t.hint.as_deref(),
        }
    }

    /// The item's control id.
    #[must_use]
    pub const fn id(&self) -> u16 {
        match self {
            Self::Picture(p) => p.id,
            Self::Button(b) => b.id,
            Self::Text(t) => t.id,
        }
    }
}

/// A picture (`H3DlgPcx`, 0x4502D0): drawn from its corner with index 0 transparent (0x450450 →
/// 0x44FDB0 with its last argument 1), in its own colours or, once the screen's logic gives it a
/// player, with the palette's entries 224–255 in that player's (0x450520).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Picture {
    /// The control id.
    pub id: u16,
    /// The picture (`.pcx`).
    pub picture: String,
    /// Its top-left corner from the dialog's.
    pub at: [i32; 2],
}

/// A button of DEF frames (`H3DlgDefButton` 0x455EF0, drawn by 0x456940): a frame of the DEF's
/// group 0 by the button's state, its canvas's corner at the button's, by the plain drawer with
/// runs skipped (0x47BF00 → 0x47C570). A frame beyond the group's is drawn as frame 0.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Button {
    /// The control id.
    pub id: u16,
    /// The DEF.
    pub def: String,
    /// Its top-left corner from the dialog's.
    pub at: [i32; 2],
    /// Its width and height, where it takes the mouse.
    pub size: [i32; 2],
    /// The frames of its states.
    #[serde(default)]
    pub frames: Frames,
    /// The key of the status line's text while the mouse is over it (`SetHints` 0x5FE840; E17
    /// M3 T6c-1), none for none.
    #[serde(default)]
    pub hint: Option<String>,
    /// Its hotkeys, by name (`S`, `Space`; its +0x48, the screens' scan codes, E17 M3 T6c-2): a
    /// key down presses it, the key up releases it and clicks (0x45659F, 0x4564D6).
    #[serde(default)]
    pub keys: Vec<String>,
}

/// A button's frame in each state: the constructor's frame (+0x34) and click frame (+0x38), the
/// disabled frame (+0x3C, 2 unless the screen changes it) and the selected one (+0x40, 3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct Frames {
    /// Released and enabled.
    pub normal: u32,
    /// Pressed.
    pub pressed: u32,
    /// Disabled.
    pub disabled: u32,
    /// Selected (a switch that is on) and not pressed.
    pub selected: u32,
}

impl Default for Frames {
    fn default() -> Self {
        Self {
            normal: 0,
            pressed: 1,
            disabled: 2,
            selected: 3,
        }
    }
}

/// A text over a picture (`H3DlgTextPcx` 0x5BC760, drawn by 0x5BC7F0): the picture opaque from
/// the item's corner, at most the item's size of it, then the text in its box by the text drawer
/// (§3.6) with the item's colour and alignment.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Text {
    /// The control id.
    pub id: u16,
    /// The picture behind the text.
    pub picture: String,
    /// Its top-left corner from the dialog's.
    pub at: [i32; 2],
    /// Its width and height: the text's box.
    pub size: [i32; 2],
    /// The font.
    pub font: String,
    /// The palette the font's colours come from.
    pub font_palette: String,
    /// The text drawer's colour argument.
    pub colour: u32,
    /// The text drawer's alignment.
    pub align: u32,
    /// The key of the status line's text while the mouse is over it, none for none.
    #[serde(default)]
    pub hint: Option<String>,
}

/// A control's state, the flags of the item's +0x16 its drawer reads: shown (bit 4), pressed
/// (bit 1), disabled (bit 8, which the state 0x1000 of `AddControlState` sets: 0x5FE560), selected
/// (bit 0x10). The default: shown, released, enabled, not selected.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct State {
    /// Not drawn.
    pub hidden: bool,
    /// Pressed.
    pub pressed: bool,
    /// Disabled.
    pub disabled: bool,
    /// Selected.
    pub selected: bool,
}

impl Button {
    /// The frame drawn in `state`, none when hidden (0x456940): the selected frame when selected
    /// and not pressed, else the disabled one when disabled, else the pressed one when pressed,
    /// else the normal one.
    #[must_use]
    pub const fn frame(&self, state: State) -> Option<u32> {
        let f = &self.frames;
        if state.hidden {
            None
        } else if state.selected && !state.pressed {
            Some(f.selected)
        } else if state.disabled {
            Some(f.disabled)
        } else if state.pressed {
            Some(f.pressed)
        } else {
            Some(f.normal)
        }
    }
}

/// What a dialog's items show, by control id; an item without an entry shows its default: a
/// button in the default [`State`], a picture in its own colours, a text empty.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Shown {
    /// The buttons' states.
    pub states: BTreeMap<u16, State>,
    /// The pictures in a player's colours: the player's colour (0 red to 7 pink).
    pub players: BTreeMap<u16, u8>,
    /// The texts.
    pub texts: BTreeMap<u16, String>,
}

impl Dialog {
    /// The item with control id `id`.
    #[must_use]
    pub fn item(&self, id: u16) -> Option<&Item> {
        self.items.iter().find(|i| i.id() == id)
    }

    /// The text item with control id `id`.
    #[must_use]
    pub fn text(&self, id: u16) -> Option<&Text> {
        match self.item(id) {
            Some(Item::Text(t)) => Some(t),
            _ => None,
        }
    }

    /// The item under the mouse at `point` (on the screen): the last in the executable's order
    /// whose area holds it (*provisional*: the dialog's search was not read; design §9.9).
    #[must_use]
    pub fn item_at(&self, point: [i32; 2]) -> Option<&Item> {
        let [x, y] = [point[0] - self.place[0], point[1] - self.place[1]];
        self.items.iter().rev().find(|item| {
            item.area().is_some_and(|([left, top], [w, h])| {
                (left..left.saturating_add(w)).contains(&x)
                    && (top..top.saturating_add(h)).contains(&y)
            })
        })
    }

    /// The button under the mouse at `point` ([`Dialog::item_at`]), if a button is there (E17
    /// M3 T6c-2: the one a click presses).
    #[must_use]
    pub fn button_at(&self, point: [i32; 2]) -> Option<u16> {
        match self.item_at(point)? {
            Item::Button(b) => Some(b.id),
            _ => None,
        }
    }

    /// The first button whose hotkeys hold `key` (E17 M3 T6c-2).
    #[must_use]
    pub fn button_for_key(&self, key: &str) -> Option<u16> {
        self.items.iter().find_map(|item| match item {
            Item::Button(b) if b.keys.iter().any(|k| k == key) => Some(b.id),
            _ => None,
        })
    }

    /// The draw list of the dialog's items as `shown`, in their order; a player's colours come
    /// from the players' palette `players` (`PLAYERS.PAL`, 32 entries a player).
    #[must_use]
    pub fn draws(&self, shown: &Shown, players: &str) -> Vec<Draw> {
        let [x, y] = self.place;
        let place = |at: [i32; 2]| [x.saturating_add(at[0]), y.saturating_add(at[1])];
        self.items
            .iter()
            .filter_map(|item| match item {
                Item::Picture(p) => {
                    let [x, y] = place(p.at);
                    Some(Draw::Picture {
                        picture: p.picture.clone(),
                        x,
                        y,
                        player: shown.players.get(&p.id).map(|&player| PlayerColours {
                            palette: players.into(),
                            player,
                        }),
                    })
                }
                Item::Button(b) => {
                    let state = shown.states.get(&b.id).copied().unwrap_or_default();
                    let [x, y] = place(b.at);
                    b.frame(state).map(|frame| Draw::Button {
                        def: b.def.clone(),
                        frame,
                        x,
                        y,
                    })
                }
                Item::Text(t) => {
                    let [x, y] = place(t.at);
                    Some(Draw::Text {
                        item: t.clone(),
                        x,
                        y,
                        text: shown.texts.get(&t.id).cloned().unwrap_or_default(),
                    })
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests;
