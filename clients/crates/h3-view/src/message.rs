//! The message box (`message_box` 0x4F6570, `message_box_build` 0x4F6990; E17 M5 T4a,
//! `docs/epics/E17-design.md` §7.19): a text in a framed dialog sized to it, with an OK, or an OK
//! and a Cancel, and at most one picture with its caption (an adventure spell's description).
//! [`layout`] lays the box out as the executable's 0x4F5D80 does and builds its items as
//! `ui` items; [`Open`] is the box on the screen, taking presses, moves and keys until a button's
//! click closes it.
//!
//! *Not built* (no battle caller needs them): a text above 200 pixels high (the executable's
//! scrolling text), two to eight pictures, the styles other than 1 and 2 (3 and 10's OK and Cancel
//! of other ids, 4's right-button popup, 5 to 9), the hidden frame around a picture (a choice's
//! mark), the network game's timeout.

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use h3_assets::fnt::Font;
use h3_render::Bugs;
use h3_render::text::{line_count, widest_line, widest_word, wrapped_width};
use serde::Deserialize;

use h3_ui::text::encode_1252;
use h3_ui::ui::{Button, Def, Dialog, Frames, Item, Label, Picture as Image, Shown, State};

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
/// The picture's DEF item and its caption (the executable's −1).
pub const PICTURE: u16 = 0xFFFF;

/// The box's assets and fonts (the layout's `[message]`; E17 M5 T4a): the executable's literals.
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
    /// A picture's caption's font (`smalfont.fnt`, 0x698A08).
    pub caption_font: String,
    /// The palette the fonts' colours come from (`game.pal`).
    pub font_palette: String,
    /// The spells' pictures (`spellScr.def`, a picture slot of type 9; its frame the spell's
    /// number).
    pub spell_pictures: String,
    /// The cursor while the box is open (`crdeflt.def`'s frame 0: `mouse_set_cursor(0, 0)` at
    /// 0x4F74AC).
    pub cursor: String,
}

/// The buttons a box has (the style, 0x4F6C93).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Style {
    /// Style 1: an OK alone ([`OK`], Enter).
    Ok,
    /// Style 2: an OK ([`YES`], Enter) and a Cancel ([`NO`], Escape).
    OkCancel,
}

/// A picture under the text (a slot of 0x4F4EB0): its DEF's frame and caption.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Picture {
    /// The DEF.
    pub def: String,
    /// Its frame of group 0.
    pub frame: u32,
    /// The DEF's width and height.
    pub size: [i32; 2],
    /// The caption under it.
    pub caption: String,
}

/// A laid-out box: its dialog (shadow and items) and what its items show.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageBox {
    /// The dialog: its place, its shadow and its items in the executable's order.
    pub dialog: Dialog,
    /// What the items show: the texts, the frame's colours.
    pub shown: Shown,
}

/// The widest a text gets before its height alone grows (0x242).
const TEXT_WIDTH_MOST: i64 = 578;
/// The narrowest text (0x100).
const TEXT_WIDTH_LEAST: i64 = 256;
/// The highest text; above it the executable scrolls it (not built).
const TEXT_HEIGHT_MOST: i64 = 200;
/// The box's width beyond its text's (50) and the text's left edge (20).
const TEXT_MARGIN: i64 = 50;
const TEXT_LEFT: i64 = 20;
/// The text's top before the box's spare height is shared (30).
const TEXT_TOP: i64 = 30;
/// The height the buttons take (50) and a picture's caption's distance from the box's bottom
/// before them (30).
const BUTTONS: i64 = 50;
const CAPTION_BOTTOM: i64 = 30;
/// A box's least width and height (256, 128).
const BOX_LEAST: [i64; 2] = [256, 128];
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
/// A caption's least and most width (50, 110).
const CAPTION_WIDTH: [i64; 2] = [50, 110];

/// Lays out a box of `style` showing `text` (in the medium font `medium`) and `picture` (its
/// caption in the small font `small`) as the executable does (0x4F5D80, 0x4F4EB0's tail,
/// 0x48FF00, 0x4F6990), its frame in the colours of `player` (the viewing player's,
/// `viewing_player` 0x4CEA20), centred on the screen.
#[must_use]
pub fn layout(
    layout: &MessageLayout,
    (medium, small): (&Font, &Font),
    text: &str,
    style: Style,
    picture: Option<&Picture>,
    player: u8,
) -> MessageBox {
    let bytes = encode_1252(text).0;
    let slot = picture.map(|p| Slot::of(small, p));
    let font_height = i64::from(medium.height);
    // The text's width: its widest word, widened while it wraps too high, then its widest line.
    let widest = widest_line(medium, &bytes);
    let mut text_width = widest_word(medium, &bytes).max(TEXT_WIDTH_LEAST);
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
    // The box's width: the text's, or the picture's row's.
    let row = slot.map_or(40, |s| s.size[0].max(s.caption[0]) + 40);
    let width = up64(row.max(text_width + TEXT_MARGIN)).max(BOX_LEAST[0]);
    let text_width = width - TEXT_MARGIN;
    // The box's height: the text's and the picture's, the buttons'.
    let mut used = text_height + 60;
    if let Some(s) = slot {
        used += 20 + s.size[1] + s.caption[1];
    }
    used += BUTTONS;
    let height = up64(used).max(BOX_LEAST[1]);
    let text_top = TEXT_TOP + (height - used) / 2;
    let place = [(SCREEN[0] - width) / 2, (SCREEN[1] - height) / 2];
    let mut items = frame(layout, [width, height]);
    let last_piece = items.iter().map(Item::id).max().unwrap_or(BACKGROUND);
    items.push(Item::Label(Label {
        id: TEXT,
        at: [i32_of(TEXT_LEFT), i32_of(text_top)],
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
            items.extend(button(
                layout,
                (0x7866, OK),
                &layout.ok,
                [x, back_y],
                "Return",
            ));
        }
        Style::OkCancel => {
            let left = (half - 74).max(1);
            let right = (half + 8).min(i32_of(width) - BACK[0]);
            items.extend(button(
                layout,
                (0x7869, YES),
                &layout.ok,
                [left, back_y],
                "Return",
            ));
            items.extend(button(
                layout,
                (0x786A, NO),
                &layout.cancel,
                [right, back_y],
                "Escape",
            ));
        }
    }
    let mut texts = BTreeMap::new();
    texts.insert(TEXT, String::from(text));
    if let (Some(p), Some(s)) = (picture, slot) {
        // One picture: centred, its caption's top 30 above the buttons' room.
        let caption_top = height - s.caption[1] - CAPTION_BOTTOM - BUTTONS;
        let x = (width - s.size[0]) / 2;
        items.push(Item::Def(Def {
            id: PICTURE,
            def: Some(p.def.clone()),
            at: [i32_of(x), i32_of(caption_top - s.size[1])],
            size: [i32_of(s.size[0]), i32_of(s.size[1])],
            frame: p.frame,
            mirrored: false,
            repeat: None,
        }));
        items.push(Item::Label(Label {
            id: PICTURE,
            at: [
                i32_of(x + (s.size[0] - s.caption[0]) / 2),
                i32_of(caption_top),
            ],
            size: [i32_of(s.caption[0]), i32_of(s.caption[1])],
            font: layout.caption_font.clone(),
            font_palette: layout.font_palette.clone(),
            colour: 1,
            align: 1,
            hint: None,
            repeat: None,
        }));
        texts.insert(PICTURE, p.caption.clone());
    }
    // The frame's pieces in the viewing player's colours (0x4F6B5D); the tiles keep theirs.
    let players = items
        .iter()
        .filter_map(|item| match item {
            Item::Def(d) if d.id != PICTURE && d.id <= last_piece => Some((d.id, player)),
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

/// A picture slot's sizes (0x4F4EB0's tail): the DEF's + 2, its caption's.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Slot {
    size: [i64; 2],
    caption: [i64; 2],
}

impl Slot {
    /// The slot of `picture`, its caption in `small` (0x4F5BC5 to 0x4F5CF6): the caption at
    /// least 50 and the picture's width wide, its widest word's (2 more than its glyphs), at most
    /// 110; widened by half while it takes more than one line and more than 2/3 of its width.
    fn of(small: &Font, picture: &Picture) -> Self {
        let size = [
            i64::from(picture.size[0]) + 2,
            i64::from(picture.size[1]) + 2,
        ];
        let bytes = encode_1252(&picture.caption).0;
        if bytes.is_empty() {
            return Self {
                size,
                caption: [0, 0],
            };
        }
        let words = bytes
            .split(|&b| b == b' ')
            .filter(|w| !w.is_empty())
            .map(|w| {
                2 + w
                    .iter()
                    .map(|&c| i64::from(small.glyph(c).advance()))
                    .sum::<i64>()
            });
        let mut width = words
            .fold(CAPTION_WIDTH[0].max(size[0]), i64::max)
            .min(CAPTION_WIDTH[1]);
        let font_height = i64::from(small.height);
        let mut lines = line_count(small, &bytes, width, Bugs::Original);
        while lines > 1 && font_height * lines > 2 * width / 3 && width != CAPTION_WIDTH[1] {
            width = (3 * width / 2).min(CAPTION_WIDTH[1]);
            lines = line_count(small, &bytes, width, Bugs::Original);
        }
        Self {
            size,
            caption: [width, font_height * lines],
        }
    }
}

/// The frame's items (0x48FF00) of a box `size` (multiples of 64): the background's tiles from
/// [`BACKGROUND`] on, each cut to the box, then the frame's pieces: the four corners, the top
/// and bottom edges, the left and right edges.
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
        Item::Picture(Image {
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

/// A button's back (`Box64x30.pcx`) at `at` and its DEF button one in, frames 0 and 1, with its
/// hotkey `key`.
fn button(
    layout: &MessageLayout,
    (back, id): (u16, u16),
    def: &str,
    at: [i32; 2],
    key: &str,
) -> [Item; 2] {
    [
        Item::Picture(Image {
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
            keys: alloc::vec![String::from(key)],
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
    /// 0x4564D6; E17 M3 T6c-2's).
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

    /// The left button released: the button it held clicks when the mouse is still over it.
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

    /// A key down (its name, as the buttons' hotkeys: `Return`, `Escape`): its button goes down.
    pub fn key(&mut self, key: &str) -> Outcome {
        if self.held.is_some() {
            return Outcome::Nothing;
        }
        match self.message.dialog.button_for_key(key) {
            Some(id) => self.hold(Held {
                id,
                key: Some(String::from(key)),
                over: true,
            }),
            None => Outcome::Nothing,
        }
    }

    /// A key up: the button its key held clicks (0x45659F).
    pub fn key_up(&mut self, key: &str) -> Outcome {
        match self.held.take_if(|h| h.key.as_deref() == Some(key)) {
            Some(h) => Outcome::Closed(h.id),
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
