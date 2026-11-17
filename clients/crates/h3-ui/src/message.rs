//! The message box (`message_box` 0x4F6570, `message_box_build` 0x4F6990; E17 design §7.19), the
//! part both viewers share (E24 M4 T1a, *provisional*): a text in a framed dialog sized to it, with
//! an OK, or an OK and a Cancel. [`layout`] lays the box out as the executable's 0x4F5D80 does and
//! builds its items as `ui` items; [`Open`] is the box on the screen, taking presses, moves and keys
//! until a button's click closes it.
//!
//! Origin: a copy of the client lane's `h3_view::message` (E17 M5 T4a, M5 T8b2, M7 T2), trimmed to
//! what the adventure map's boxes need (E24 M4 T1a: End Turn's question, the week's and the month's
//! message): no pictures, no popup (style 4), no frame colours by the acting side (CLT-Q28 (2) is
//! the battle's). The battle's own stays the client lane's; whether it moves here is theirs to say.
//!
//! *Not built* (no caller needs them): a text above 200 pixels high (the executable's scrolling
//! text), the styles 3 to 10, the pictures (a slot of 0x4F4EB0), the network game's timeout.
//!
//! *The client's* (E17 M5 T8b2): the text's width stops at 578 when a word is wider (the
//! executable's box takes the word whole, past the screen) and the box at 768 × 576, so a server's
//! long name cannot make a box of unbounded items; none of the game's texts reaches either.

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use h3_assets::fnt::Font;
use h3_render::Bugs;
use h3_render::text::{line_count, widest_line, widest_word, wrapped_width};
use serde::Deserialize;

use crate::text::encode_1252;
use crate::ui::{Button, Def, Dialog, Frames, Item, Label, Picture, Shown, State};

/// The text's item.
pub const TEXT: u16 = 1;
/// The first background tile's id (the frame's builder 0x48FF00 counts from 200).
pub const BACKGROUND: u16 = 200;
/// A style-1 box's OK.
pub const OK: u16 = 0x7802;
/// A style-2 box's OK.
pub const YES: u16 = 0x7805;
/// A style-2 box's Cancel.
pub const NO: u16 = 0x7806;

/// The box's assets and fonts (a layout's `[message]`): the executable's literals.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MessageLayout {
    /// The background's tile (`diboxbck.pcx`, 256 × 256).
    pub background: String,
    /// The frame's pieces (`dialgbox.def`, 64 × 64: frames 0 to 3 the corners, 4 to 7 the left,
    /// right, top and bottom edges).
    pub frame: String,
    /// A button's back (`Box64x30.pcx`, 66 × 32).
    pub button_back: String,
    /// The OK button (`iOkay.def`).
    pub ok: String,
    /// The Cancel button (`iCancel.def`).
    pub cancel: String,
    /// The text's font (`medfont.fnt`, 0x698A0C).
    pub font: String,
    /// The palette the font's colours come from (`game.pal`).
    pub font_palette: String,
}

/// The buttons a box has (the style, 0x4F6C93).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Style {
    /// Style 1: an OK alone ([`OK`], Enter).
    Ok,
    /// Style 2: an OK ([`YES`], Enter) and a Cancel ([`NO`], Escape).
    OkCancel,
}

/// What a key does in an open box (its keys a viewer's table's: the buttons' hotkeys
/// `message_box_build` 0x4F6990 gives, Enter the OK's and Escape the Cancel's).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Key {
    /// The OK ([`OK`] or [`YES`]) pressed.
    Ok,
    /// The Cancel ([`NO`]) pressed, when the box has one.
    Cancel,
}

/// A laid-out box: its dialog (shadow and items) and what its items show.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageBox {
    /// The dialog: its place, its shadow and its items in the executable's order.
    pub dialog: Dialog,
    /// What the items show: the text, the frame's colours, the buttons' states.
    pub shown: Shown,
}

/// The widest a text gets before its height alone grows (0x242).
const TEXT_WIDTH_MOST: i64 = 578;
/// The narrowest text (0x100).
const TEXT_WIDTH_LEAST: i64 = 256;
/// The highest text; above it the executable scrolls it (not built).
const TEXT_HEIGHT_MOST: i64 = 200;
/// The box's width beyond its text's (50) and the text's left edge (20): the executable's text
/// sits 5 pixels left of the box's centre, the client's centred (CLT-Q28 (1), [`layout`]).
const TEXT_MARGIN: i64 = 50;
const TEXT_LEFT: i64 = 20;
/// The text's top before the box's spare height is shared (30).
const TEXT_TOP: i64 = 30;
/// The height the buttons take (50).
const BUTTONS: i64 = 50;
/// A box's least width and height (256, 128).
const BOX_LEAST: [i64; 2] = [256, 128];
/// The client's largest box (E17 M5 T8b2): the screen's multiples of 64.
const BOX_MOST: [i64; 2] = [768, 576];
/// The screen the box is centred on (800 × 600).
const SCREEN: [i64; 2] = [800, 600];
/// A frame piece's and a size step's side (64).
const PIECE: i64 = 64;
/// A background tile's side (256).
const TILE: i64 = 256;
/// A button's back's size (66 × 32) and its button's (64 × 30, one in from it), and the backs'
/// distance from the box's bottom (62).
const BACK: [i32; 2] = [66, 32];
const BUTTON: [i32; 2] = [64, 30];
const BACK_BOTTOM: i32 = 62;

/// Lays out a box of `style` showing `text` (in the medium font `medium`) as the executable does
/// (0x4F5D80, 0x48FF00, 0x4F6990), its frame in the colours of `player` (the player whose view the
/// screen shows, `viewing_player` 0x4CEA20), centred on the screen, in the profile `bugs`: the
/// executable's text at x 20, 5 pixels left of the box's centre, the client's centred on the box
/// (CLT-Q28 (1), answered (c) at E17 M5 T8c with the user's directive: a text the original sets off
/// its centre is centred).
#[must_use]
pub fn layout(
    layout: &MessageLayout,
    medium: &Font,
    text: &str,
    style: Style,
    (player, bugs): (u8, Bugs),
) -> MessageBox {
    let bytes = encode_1252(text).0;
    let font_height = i64::from(medium.height);
    // The text's width: its widest word, widened while it wraps too high, then its widest line.
    // A word wider than 578 is broken there (the client's: the executable's box takes it whole).
    let widest = widest_line(medium, &bytes);
    let mut text_width = widest_word(medium, &bytes).clamp(TEXT_WIDTH_LEAST, TEXT_WIDTH_MOST);
    let mut text_height;
    loop {
        text_height = font_height * line_count(medium, &bytes, text_width, Bugs::Original);
        if widest <= text_width {
            text_width = widest;
            break;
        }
        if text_width >= TEXT_WIDTH_MOST
            || (text_height <= TEXT_HEIGHT_MOST && 3 * text_height <= 2 * text_width)
        {
            text_width = wrapped_width(medium, &bytes, text_width);
            break;
        }
        text_width = (3 * text_width / 2).min(TEXT_WIDTH_MOST);
    }
    let text_height = text_height.min(TEXT_HEIGHT_MOST);
    // The box's width (at least 40 beside the text's, 0x4F5EC0) and height.
    let width = up64(text_width + TEXT_MARGIN).clamp(BOX_LEAST[0], BOX_MOST[0]);
    let text_width = width - TEXT_MARGIN;
    let used = text_height + 60 + BUTTONS;
    let height = up64(used).clamp(BOX_LEAST[1], BOX_MOST[1]);
    let text_top = TEXT_TOP + (height - used) / 2;
    let place = [(SCREEN[0] - width) / 2, (SCREEN[1] - height) / 2];
    let mut items = frame(layout, [width, height]);
    let last_piece = items.iter().map(Item::id).max().unwrap_or(BACKGROUND);
    let text_left = match bugs {
        Bugs::Fixed => (width - text_width) / 2,
        Bugs::Original => TEXT_LEFT,
    };
    items.push(Item::Label(Label {
        id: TEXT,
        at: [i32_of(text_left), i32_of(text_top)],
        size: [i32_of(text_width), i32_of(text_height)],
        font: layout.font.clone(),
        font_palette: layout.font_palette.clone(),
        colour: 1,
        align: 1,
        hint: None,
        repeat: None,
    }));
    let back_y = i32_of(height) - BACK_BOTTOM;
    let half = i32_of(width) / 2;
    match style {
        Style::Ok => {
            let x = (i32_of(width) - BACK[0]) / 2;
            items.extend(button(layout, (0x7866, OK), &layout.ok, [x, back_y]));
        }
        Style::OkCancel => {
            let left = (half - 74).max(1);
            let right = (half + 8).min(i32_of(width) - BACK[0]);
            items.extend(button(layout, (0x7869, YES), &layout.ok, [left, back_y]));
            items.extend(button(
                layout,
                (0x786A, NO),
                &layout.cancel,
                [right, back_y],
            ));
        }
    }
    let mut texts = BTreeMap::new();
    texts.insert(TEXT, String::from(text));
    // The frame's pieces in the viewing player's colours (0x4F6B5D); the tiles keep theirs.
    let players = items
        .iter()
        .filter_map(|item| match item {
            Item::Def(d) if d.id <= last_piece => Some((d.id, player)),
            _ => None,
        })
        .collect();
    MessageBox {
        dialog: Dialog {
            place: [i32_of(place[0]), i32_of(place[1])],
            shadow: Some([i32_of(width), i32_of(height)]),
            items,
        },
        shown: Shown {
            texts,
            players,
            ..Shown::default()
        },
    }
}

/// The frame's items (0x48FF00) of a box `size` (multiples of 64): the background's tiles from
/// [`BACKGROUND`] on, each cut to the box, then the frame's pieces: the four corners, the top
/// and bottom edges, the left and right edges. A box of at most [`BOX_MOST`] has at most 47 (ids
/// 200 to 246: none reaches the buttons').
fn frame(layout: &MessageLayout, [width, height]: [i64; 2]) -> Vec<Item> {
    let mut items = Vec::new();
    let mut next = BACKGROUND;
    let mut id = || {
        let this = next;
        next = next.wrapping_add(1);
        this
    };
    let (columns, rows) = (width / TILE, height / TILE);
    let (rest_w, rest_h) = (width - columns * TILE, height - rows * TILE);
    let tile = |at: [i64; 2], size: [i64; 2], id: u16| {
        Item::Picture(Picture {
            id,
            picture: layout.background.clone(),
            at: [i32_of(at[0]), i32_of(at[1])],
            size: Some([i32_of(size[0]), i32_of(size[1])]),
            hint: None,
        })
    };
    for r in 0..rows {
        for c in 0..columns {
            items.push(tile([c * TILE, r * TILE], [TILE, TILE], id()));
        }
    }
    if rest_w > 0 {
        for r in 0..rows {
            items.push(tile([columns * TILE, r * TILE], [rest_w, TILE], id()));
        }
    }
    if rest_h > 0 {
        for c in 0..columns {
            items.push(tile([c * TILE, rows * TILE], [TILE, rest_h], id()));
        }
    }
    if rest_w > 0 && rest_h > 0 {
        items.push(tile([columns * TILE, rows * TILE], [rest_w, rest_h], id()));
    }
    let piece = |at: [i64; 2], frame: u32, id: u16| {
        Item::Def(Def {
            id,
            def: Some(layout.frame.clone()),
            at: [i32_of(at[0]), i32_of(at[1])],
            size: [i32_of(PIECE), i32_of(PIECE)],
            frame,
            mirrored: false,
            group: 0,
            creature: false,
            frames_of: None,
            repeat: None,
        })
    };
    let (right, bottom) = (width - PIECE, height - PIECE);
    for (at, f) in [
        ([0, 0], 0),
        ([right, 0], 1),
        ([0, bottom], 2),
        ([right, bottom], 3),
    ] {
        items.push(piece(at, f, id()));
    }
    let mut x = PIECE;
    while x < right {
        items.push(piece([x, 0], 6, id()));
        items.push(piece([x, bottom], 7, id()));
        x += PIECE;
    }
    let mut y = PIECE;
    while y < bottom {
        items.push(piece([0, y], 4, id()));
        items.push(piece([right, y], 5, id()));
        y += PIECE;
    }
    items
}

/// A button's back (`Box64x30.pcx`) at `at` and its DEF button one in, frames 0 and 1 (its hotkey
/// a viewer's table's, [`Key`]).
fn button(layout: &MessageLayout, (back, id): (u16, u16), def: &str, at: [i32; 2]) -> [Item; 2] {
    [
        Item::Picture(Picture {
            id: back,
            picture: layout.button_back.clone(),
            at,
            size: Some(BACK),
            hint: None,
        }),
        Item::Button(Button {
            id,
            def: String::from(def),
            at: [at[0] + 1, at[1] + 1],
            size: BUTTON,
            frames: Frames::default(),
            hint: None,
        }),
    ]
}

/// `value` rounded up to a multiple of 64 (`add 0x3F; and 0xC0` of the low byte).
const fn up64(value: i64) -> i64 {
    (value + PIECE - 1) & !(PIECE - 1)
}

/// A layout's number as a screen coordinate (they all fit).
fn i32_of(value: i64) -> i32 {
    i32::try_from(value).unwrap_or(i32::MAX)
}

/// What an input did to the open box.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// Nothing.
    Nothing,
    /// A button went down or up: draw it again.
    Changed,
    /// A button's click closed the box: its id ([`OK`], [`YES`] or [`NO`]).
    Closed(u16),
}

/// The box on the screen: its layout, its buttons' states and the button a press or a key holds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Open {
    /// The laid-out box.
    pub message: MessageBox,
    held: Option<Held>,
}

/// A button held down: by the mouse (whether the mouse is over it) or by its key.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Held {
    id: u16,
    key: Option<String>,
    over: bool,
}

impl Open {
    /// The box shown.
    #[must_use]
    pub const fn new(message: MessageBox) -> Self {
        Self {
            message,
            held: None,
        }
    }

    /// The left button pressed at `point`: a button there goes down (the DEF button's handler
    /// 0x456328).
    pub fn press(&mut self, point: [i32; 2]) -> Outcome {
        if self.held.is_some() {
            return Outcome::Nothing;
        }
        match self.message.dialog.button_at(point) {
            Some(id) => self.hold(Held {
                id,
                key: None,
                over: true,
            }),
            None => Outcome::Nothing,
        }
    }

    /// The mouse moved to `point`: the button it holds follows it on or off.
    pub fn moved(&mut self, point: [i32; 2]) -> Outcome {
        let Some(held) = self.held.as_ref().filter(|h| h.key.is_none()) else {
            return Outcome::Nothing;
        };
        let over = self.message.dialog.button_at(point) == Some(held.id);
        if over == held.over {
            return Outcome::Nothing;
        }
        let id = held.id;
        self.held = Some(Held {
            id,
            key: None,
            over,
        });
        self.set_pressed(id, over);
        Outcome::Changed
    }

    /// The left button released, or the right one while the mouse holds a button (the DEF
    /// button's own loop ends at either, `def_button_message` 0x456451 and 0x456338): the button it
    /// held clicks when the mouse is still over it.
    pub fn release(&mut self) -> Outcome {
        match self.held.take_if(|h| h.key.is_none()) {
            Some(h) if h.over => Outcome::Closed(h.id),
            Some(h) => {
                self.set_pressed(h.id, false);
                Outcome::Changed
            }
            None => Outcome::Nothing,
        }
    }

    /// A key down giving `key` (the name of the key `by`, whose release clicks): its button goes
    /// down.
    pub fn key(&mut self, key: Key, by: &str) -> Outcome {
        if self.held.is_some() {
            return Outcome::Nothing;
        }
        let ids: &[u16] = match key {
            Key::Ok => &[OK, YES],
            Key::Cancel => &[NO],
        };
        let has = |id: &&u16| {
            self.message
                .dialog
                .items
                .iter()
                .any(|i| matches!(i, Item::Button(b) if b.id == **id))
        };
        match ids.iter().find(has) {
            Some(&id) => self.hold(Held {
                id,
                key: Some(String::from(by)),
                over: true,
            }),
            None => Outcome::Nothing,
        }
    }

    /// A key up: the button its key held clicks (0x4564D6).
    pub fn key_up(&mut self, key: &str) -> Outcome {
        match self.held.take_if(|h| h.key.as_deref() == Some(key)) {
            Some(h) => Outcome::Closed(h.id),
            None => Outcome::Nothing,
        }
    }

    /// Whether the mouse holds a button down: its own loop runs until a release (`def_button_message`
    /// 0x45641C), every key and the right press dropped, the moves only its own.
    #[must_use]
    pub fn holds_mouse(&self) -> bool {
        self.held.as_ref().is_some_and(|h| h.key.is_none())
    }

    /// Lets the button held by key `by` (none: by the mouse) go without its click, drawn up again
    /// (a release a loop of its own took: CLT-Q39, MCL-Q24).
    pub fn let_go(&mut self, by: Option<&str>) -> Outcome {
        match self.held.take_if(|h| h.key.as_deref() == by) {
            Some(h) => {
                self.set_pressed(h.id, false);
                Outcome::Changed
            }
            None => Outcome::Nothing,
        }
    }

    fn hold(&mut self, held: Held) -> Outcome {
        self.set_pressed(held.id, true);
        self.held = Some(held);
        Outcome::Changed
    }

    fn set_pressed(&mut self, id: u16, pressed: bool) {
        self.message.shown.states.insert(
            id,
            State {
                pressed,
                ..State::default()
            },
        );
    }
}

#[cfg(test)]
mod tests;
