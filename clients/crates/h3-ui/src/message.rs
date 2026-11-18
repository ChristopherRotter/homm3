//! The message box (`message_box` 0x4F6570, `message_box_build` 0x4F6990; E17 design §7.19), the
//! part both viewers share (E24 M4 T1a, *provisional*): a text in a framed dialog sized to it, with
//! an OK, or an OK and a Cancel. [`layout`] lays the box out as the executable's 0x4F5D80 does and
//! builds its items as `ui` items; [`Open`] is the box on the screen, taking presses, moves and keys
//! until a button's click closes it.
//!
//! The pictures (E24 M4 T2a2, design §5.3: the finds told): [`told`], a box of 0x4F6570 with up to
//! three pictures under its text (a slot of 0x4F4EB0 each, [`Picture`]), and [`pictures`], the box
//! of `message_box_pictures` 0x4F7690 with up to eight in up to two rows of four, the first row the
//! larger share, laid out from the bottom (E17 design §12.21). A box with pictures holds a hidden
//! frame around each (a choice's mark, [`CHOICE`] on: hidden at once, never drawn, so not built).
//!
//! Origin: a copy of the client lane's `h3_view::message` (E17 M5 T4a, M5 T8b2, M6 T5c, M7 T1b,
//! M7 T2), trimmed to what the adventure map's boxes need (E24 M4 T1a: End Turn's question, the
//! week's and the month's message; E24 M4 T2a2: the pictures, its `Slot`, `Rows` and `row_xs`
//! copied): no popup (style 4), no frame colours by the acting side (CLT-Q28 (2) is the battle's).
//! The battle's own stays the client lane's; whether it moves here is theirs to say.
//!
//! The choices (E24 M4 T2b2 part B, design §5.4, *provisional*): [`Style::Choose`] and
//! [`Style::ChooseCancel`] (styles 7 and 10), their OK disabled until a picture is chosen, each
//! picture's hidden frame ([`CHOICE`] + its slot: an [`Area`] the box's [`Open::press`] chooses,
//! drawn as [`MessageBox::frame_draws`] says while shown) and [`choice`]'s "or" between the
//! pictures.
//!
//! *Not built* (no caller needs them): a text above 200 pixels high (the executable's scrolling
//! text), the styles 3 to 6, 8 and 9, the hidden frames around the pictures of the other styles
//! (never shown there), a choice frame's right-click popup (`choice_frame_popup` 0x4F0B20), the
//! network game's timeout.
//!
//! *The client's* (E17 M5 T8b2): the text's width stops at 578 when a word is wider (the
//! executable's box takes the word whole, past the screen) and the box at 768 × 576, so a server's
//! long name cannot make a box of unbounded items; none of the game's texts reaches either.

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use h3_assets::fnt::Font;
use h3_render::Bugs;
use h3_render::text::{line_count, widest_line, widest_word, wrapped_width};
use serde::Deserialize;

use crate::text::encode_1252;
use crate::ui::{
    Area, Button, Def, Dialog, Draw, Frames, Item, Label, Picture as Image, Shown, State,
};

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
/// A choice box's Cancel (styles 3 and 10, 0x4F6E5B; E24 M4 T2b2 part B).
pub const CANCEL: u16 = 0x7801;
/// The first picture's hidden frame (a choice's mark, 0x4F6990: slot n's `CHOICE + n`, hidden at
/// once; E17 design §7.19, E24 design §5.3 and §5.4): built in the choice styles alone (E24 M4
/// T2b2 part B: [`Style::Choose`], [`Style::ChooseCancel`]), where a click shows it.
pub const CHOICE: u16 = 0x7809;
/// The "or" between a choice's pictures (GENRLTXT 5, 0x4F73FC: the executable's id −1), an id of
/// its own so that its text is told apart (E24 M4 T2b2 part B, *provisional*).
pub const OR: u16 = 0xFEFF;
/// The game palette's entry a choice's frame is drawn in (45: `game_palette16` +0x76 at 0x4F7313,
/// its colours from +0x1C; recorded, the frames' (237, 213, 122)).
pub const FRAME_ENTRY: u32 = 45;
/// The most pictures a box of [`pictures`] shows (its eight slots, 0x4F5DA6).
pub const SLOTS: usize = 8;
/// The most pictures a box of [`told`] shows (`message_box` 0x4F6570's three types and subtypes).
pub const TOLD_SLOTS: usize = 3;
/// The first of the ids the pictures' items take, which the executable gives −1 (E17 M6 T5c):
/// slot n's DEF item `PICTURES + 2n`, its caption `PICTURES + 2n + 1`, so that each caption's text
/// is told apart ([`Shown::texts`]).
pub const PICTURES: u16 = 0xFF00;

/// The ids of slot `slot`'s DEF item and caption in a box of [`told`] or [`pictures`].
#[must_use]
pub fn slot_ids(slot: usize) -> [u16; 2] {
    let def = PICTURES.saturating_add(u16::try_from(2 * slot).unwrap_or(u16::MAX));
    [def, def.saturating_add(1)]
}

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
    /// A picture's caption's font (`smalfont.fnt`, 0x698A08; E24 M4 T2a2).
    pub caption_font: String,
    /// The palette the fonts' colours come from (`game.pal`).
    pub font_palette: String,
}

/// The buttons a box has (the style, 0x4F6C93).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Style {
    /// Style 1: an OK alone ([`OK`], Enter).
    Ok,
    /// Style 2: an OK ([`YES`], Enter) and a Cancel ([`NO`], Escape).
    OkCancel,
    /// Style 7 (E24 M4 T2b2 part B): the pictures to choose, each with its hidden frame, and an
    /// OK ([`OK`], Enter) disabled until one is chosen (0x4F7083), placed as style 1's.
    Choose,
    /// Style 10: as style 7 with a Cancel ([`CANCEL`], Escape), built before the OK (0x4F6DF6)
    /// and placed as style 2's (the OK's disabling at 0x4F6F2E).
    ChooseCancel,
}

impl Style {
    /// Whether its pictures are chosen (styles 7 and 10).
    #[must_use]
    pub const fn chooses(self) -> bool {
        matches!(self, Self::Choose | Self::ChooseCancel)
    }
}

/// What a key does in an open box (its keys a viewer's table's: the buttons' hotkeys
/// `message_box_build` 0x4F6990 gives, Enter the OK's and Escape the Cancel's).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Key {
    /// The OK ([`OK`] or [`YES`]) pressed.
    Ok,
    /// The Cancel ([`NO`] or [`CANCEL`]) pressed, when the box has one.
    Cancel,
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
    /// What the items show: the texts, the frame's colours, the buttons' states.
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
/// The height the buttons take (50) and a picture's caption's distance from the box's bottom
/// before them (30).
const BUTTONS: i64 = 50;
const CAPTION_BOTTOM: i64 = 30;
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
/// A caption's least and most width (50, 110).
const CAPTION_WIDTH: [i64; 2] = [50, 110];

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
    // No picture: no caption measured in the small font.
    build(
        layout,
        (medium, medium),
        text,
        (style, bugs == Bugs::Fixed, bugs),
        (&[], None),
        player,
    )
}

/// Lays out the box `message_box` 0x4F6570 shows with pictures (E24 M4 T2a2, design §5.3: a find
/// told): as [`layout`], `text` above the first [`TOLD_SLOTS`] of `pictures` in one row (their
/// captions in the small font `small`; 0x4F5D80's pictures, [`Rows`]), the text at x 20 in the
/// executable's profile and centred in the client's, the rows in the profile `bugs`. Each
/// picture's items take their slot's ids ([`slot_ids`]).
#[must_use]
pub fn told(
    layout: &MessageLayout,
    (medium, small): (&Font, &Font),
    text: &str,
    style: Style,
    pictures: &[Picture],
    (player, bugs): (u8, Bugs),
) -> MessageBox {
    let pictures: Vec<(&Picture, [u16; 2])> = pictures
        .iter()
        .take(TOLD_SLOTS)
        .enumerate()
        .map(|(slot, p)| (p, slot_ids(slot)))
        .collect();
    build(
        layout,
        (medium, small),
        text,
        (style, bugs == Bugs::Fixed, bugs),
        (&pictures, None),
        player,
    )
}

/// Lays out the box `message_box_pictures` 0x4F7690 shows (E17 M6 T5c, design §12.21): an OK
/// (style 1), `text` and the first [`SLOTS`] of `pictures` in up to two rows (0x4F5D80's
/// pictures), as [`layout`] otherwise, the rows in the profile `bugs` (CLT-Q77: [`Rows::places`]);
/// the text's corner at x 25 (the call's own, in both profiles: the text centred on the box).
/// Each picture's items take their slot's ids ([`slot_ids`]).
#[must_use]
pub fn pictures(
    layout: &MessageLayout,
    (medium, small): (&Font, &Font),
    text: &str,
    pictures: &[Picture],
    (player, bugs): (u8, Bugs),
) -> MessageBox {
    let pictures: Vec<(&Picture, [u16; 2])> = pictures
        .iter()
        .take(SLOTS)
        .enumerate()
        .map(|(slot, p)| (p, slot_ids(slot)))
        .collect();
    build(
        layout,
        (medium, small),
        text,
        (Style::Ok, true, bugs),
        (&pictures, None),
        player,
    )
}

/// Lays out a choice's box (E24 M4 T2b2 part B, design §5.4: `message_box` 0x4F6570 of style 7
/// or 10, [`Style::chooses`]): as [`told`], each picture with its hidden frame and the OK
/// disabled, and `or` (GENRLTXT 5, the call's "or" flag) between the pictures: a text item in
/// the medium font as wide as the text and a line high, centred on the box (halves rounded
/// toward zero), one line above the first picture's caption (0x4F737D to 0x4F7461). Another
/// style lays out as [`told`] with `or`.
#[must_use]
pub fn choice(
    layout: &MessageLayout,
    (medium, small): (&Font, &Font),
    text: &str,
    style: Style,
    pictures: &[Picture],
    or: Option<&str>,
    (player, bugs): (u8, Bugs),
) -> MessageBox {
    let pictures: Vec<(&Picture, [u16; 2])> = pictures
        .iter()
        .take(TOLD_SLOTS)
        .enumerate()
        .map(|(slot, p)| (p, slot_ids(slot)))
        .collect();
    build(
        layout,
        (medium, small),
        text,
        (style, bugs == Bugs::Fixed, bugs),
        (&pictures, or),
        player,
    )
}

/// The box of `style` with `text` and `pictures` (each with its DEF item's and caption's ids) and
/// a choice's `or`, the text centred on the box (`centred`: x 25) or at x 20 (0x4F5D80,
/// 0x48FF00, 0x4F6990), the pictures' rows placed in the profile `bugs` ([`Rows::places`]).
fn build(
    layout: &MessageLayout,
    (medium, small): (&Font, &Font),
    text: &str,
    (style, centred, bugs): (Style, bool, Bugs),
    (pictures, or): (&[(&Picture, [u16; 2])], Option<&str>),
    player: u8,
) -> MessageBox {
    let bytes = encode_1252(text).0;
    let slots: Vec<Slot> = pictures.iter().map(|(p, _)| Slot::of(small, p)).collect();
    let rows = Rows::of(&slots);
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
    // The box's width: the text's (at least 40 beside it, 0x4F5EC0), or the pictures' rows'.
    let width = up64(rows.width().max(text_width + TEXT_MARGIN)).clamp(BOX_LEAST[0], BOX_MOST[0]);
    let text_width = width - TEXT_MARGIN;
    // The box's height: the text's and the pictures' rows', the buttons'.
    let used = text_height + 60 + rows.height() + BUTTONS;
    let height = up64(used).clamp(BOX_LEAST[1], BOX_MOST[1]);
    let text_top = TEXT_TOP + (height - used) / 2;
    let place = [(SCREEN[0] - width) / 2, (SCREEN[1] - height) / 2];
    let mut items = frame(layout, [width, height]);
    let last_piece = items.iter().map(Item::id).max().unwrap_or(BACKGROUND);
    let text_left = if centred {
        (width - text_width) / 2
    } else {
        TEXT_LEFT
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
    let left = (half - 74).max(1);
    let right = (half + 8).min(i32_of(width) - BACK[0]);
    match style {
        Style::Ok | Style::Choose => {
            let x = (i32_of(width) - BACK[0]) / 2;
            items.extend(button(layout, (0x7866, OK), &layout.ok, [x, back_y]));
        }
        Style::OkCancel => {
            items.extend(button(layout, (0x7869, YES), &layout.ok, [left, back_y]));
            items.extend(button(
                layout,
                (0x786A, NO),
                &layout.cancel,
                [right, back_y],
            ));
        }
        Style::ChooseCancel => {
            // The Cancel first (its back 0x7865), then the OK (0x4F6DF6 to 0x4F6F1E).
            items.extend(button(
                layout,
                (0x7865, CANCEL),
                &layout.cancel,
                [right, back_y],
            ));
            items.extend(button(layout, (0x7866, OK), &layout.ok, [left, back_y]));
        }
    }
    let mut states = BTreeMap::new();
    if style.chooses() {
        // The OK disabled until a picture is chosen (0x4F6F2E, 0x4F7083).
        let disabled = State {
            disabled: true,
            ..State::default()
        };
        states.insert(OK, disabled);
    }
    let mut texts = BTreeMap::new();
    texts.insert(TEXT, String::from(text));
    // The pictures in slot order (0x4F7154 to 0x4F7377): each DEF item, then its caption when it
    // has one; in the choice styles its frame one pixel out, hidden (elsewhere never shown, so
    // not built).
    let places = rows.places(&slots, width, height - BUTTONS, bugs);
    let first_caption = places.first().map(|&(_, top)| top);
    for (slot, (((p, [def, caption]), s), (x, caption_top))) in
        pictures.iter().zip(&slots).zip(places).enumerate()
    {
        let frame = style.chooses().then(|| {
            Item::Area(Area {
                id: CHOICE.saturating_add(u16::try_from(slot).unwrap_or(u16::MAX)),
                at: [i32_of(x - 1), i32_of(caption_top - s.size[1] - 1)],
                size: [i32_of(s.size[0] + 2), i32_of(s.size[1] + 2)],
                hint: None,
            })
        });
        items.push(Item::Def(Def {
            id: *def,
            def: Some(p.def.clone()),
            at: [i32_of(x), i32_of(caption_top - s.size[1])],
            size: [i32_of(s.size[0]), i32_of(s.size[1])],
            frame: p.frame,
            mirrored: false,
            group: 0,
            creature: false,
            frames_of: None,
            repeat: None,
        }));
        if !p.caption.is_empty() {
            items.push(Item::Label(Label {
                id: *caption,
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
            texts.insert(*caption, p.caption.clone());
        }
        if let Some(frame) = frame {
            let hidden = State {
                hidden: true,
                ..State::default()
            };
            states.insert(frame.id(), hidden);
            items.push(frame);
        }
    }
    // The "or" (0x4F737D to 0x4F7461): as wide as its text, a line high, one line above the first
    // picture's caption.
    if let (Some(or), Some(top)) = (or, first_caption) {
        let w = widest_line(medium, &encode_1252(or).0);
        items.push(Item::Label(Label {
            id: OR,
            at: [i32_of((width - w) / 2), i32_of(top - font_height)],
            size: [i32_of(w), i32_of(font_height)],
            font: layout.font.clone(),
            font_palette: layout.font_palette.clone(),
            colour: 1,
            align: 1,
            hint: None,
            repeat: None,
        }));
        texts.insert(OR, String::from(or));
    }
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
            states,
            texts,
            players,
            ..Shown::default()
        },
    }
}

impl MessageBox {
    /// The choice frames shown (E24 M4 T2b2 part B, design §5.4): each its one-pixel outline in
    /// the entry [`FRAME_ENTRY`] of the game palette `palette` (`game.pal`), four fills on the
    /// screen (*provisional*: the H3DlgFrame's drawer 0x450130 not read, the recording's
    /// pixels), drawn over the box's items (nothing it outlines overlaps a later item: the
    /// executable draws it after its caption); none while hidden.
    #[must_use]
    pub fn frame_draws(&self, palette: &str) -> Vec<Draw> {
        let [px, py] = self.dialog.place;
        let frames = self.dialog.items.iter().filter_map(|item| match item {
            Item::Area(a) if is_choice(a.id) && !self.shown.state(a.id).hidden => Some(a),
            _ => None,
        });
        frames
            .flat_map(|a| {
                let at = [px.saturating_add(a.at[0]), py.saturating_add(a.at[1])];
                outline(at, a.size, palette)
            })
            .collect()
    }
}

/// A frame's one-pixel outline (E24 M4 T2c, design §5.5: the H3DlgFrame of a choice's box and of
/// the level-up dialog, `dialog_frame_create`): the rectangle of `size` at `at` on the screen,
/// its four sides' fills in the entry [`FRAME_ENTRY`] of the game palette `palette` (the top, the
/// bottom, the left, the right; *provisional*: the frame's drawer 0x450130 not read, the
/// recordings' pixels).
#[must_use]
pub fn outline([x, y]: [i32; 2], [w, h]: [i32; 2], palette: &str) -> [Draw; 4] {
    let (right, bottom) = (x.saturating_add(w - 1), y.saturating_add(h - 1));
    [
        [x, y, w, 1],
        [x, bottom, w, 1],
        [x, y.saturating_add(1), 1, h - 2],
        [right, y.saturating_add(1), 1, h - 2],
    ]
    .map(|rect| Draw::Fill {
        rect,
        palette: String::from(palette),
        entry: FRAME_ENTRY,
    })
}

/// Whether `id` is a choice frame's (the first three slots').
fn is_choice(id: u16) -> bool {
    (CHOICE..CHOICE.saturating_add(3)).contains(&id)
}

/// The pictures' rows (0x4F5D80's pictures, E17 design §12.21): at most four a row, the first row
/// the larger share; over the slots `widest` the largest picture's or caption's width and
/// `overhang` the largest caption's width beyond its picture's (at least 0).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Rows {
    /// How many pictures each row shows (the second 0 with one row; both 0 without pictures).
    counts: [usize; 2],
    widest: i64,
    overhang: i64,
    /// Each row's picture height and caption height: the largest of its slots' (a slot's row its
    /// index divided by the first row's count).
    heights: [[i64; 2]; 2],
}

/// The space between two pictures of a row beside the captions' overhang (40).
const PICTURE_GAP: i64 = 40;
/// The space above the first row and between two rows (20).
const ROW_GAP: i64 = 20;

impl Rows {
    /// The rows of `slots` (0x4F5D9D to 0x4F5E7D).
    fn of(slots: &[Slot]) -> Self {
        let n = slots.len();
        let rows = n.div_ceil(4);
        let first = if rows == 0 { 0 } else { n.div_ceil(rows) };
        let mut out = Self {
            counts: [first, n - first],
            widest: 0,
            overhang: 0,
            heights: [[0; 2]; 2],
        };
        for (i, s) in slots.iter().enumerate() {
            out.widest = out.widest.max(s.size[0]).max(s.caption[0]);
            out.overhang = out.overhang.max(s.caption[0] - s.size[0]);
            // Its row: the first's count divides at most eight slots in two.
            let row = &mut out.heights[(i / first).min(1)];
            row[0] = row[0].max(s.size[1]);
            row[1] = row[1].max(s.caption[1]);
        }
        out
    }

    /// The box's least width for them (0x4F5E83 to 0x4F5EC0): the first row's pictures at the
    /// widest, the gaps with the overhang between them, 40 more (40 without pictures).
    fn width(&self) -> i64 {
        let k = i64::try_from(self.counts[0]).unwrap_or(0);
        if k == 0 {
            return PICTURE_GAP;
        }
        k * self.widest + (k - 1) * (self.overhang + PICTURE_GAP) + PICTURE_GAP
    }

    /// The height they add to the box (0x4F6014 to 0x4F6032): 20, each row's picture and caption
    /// heights, 20 between two rows (0 without pictures).
    fn height(&self) -> i64 {
        let shown = self
            .counts
            .iter()
            .zip(&self.heights)
            .filter(|(k, _)| **k > 0);
        let (rows, sum) = shown.fold((0_i64, 0_i64), |(r, h), (_, [p, c])| (r + 1, h + p + c));
        if rows == 0 {
            return 0;
        }
        ROW_GAP + sum + ROW_GAP * (rows - 1)
    }

    /// Each slot's picture x and caption top in a box `width` wide whose buttons' room starts at
    /// `bottom` (0x4F60BB to 0x4F627C), in the profile `bugs`: the first row's captions 30 above
    /// it, raised by a second row's height and caption height and 20. A later row's captions:
    /// the executable's lower by the **second** row's heights and 20 (0x4F626E: with a second
    /// row's captions higher than the first's they end less than 30 above the room, its pictures
    /// more than 20 under the captions above); the client's by the row above's own caption height
    /// (CLT-Q77 (b), answered at E17 M6's closure; the original's for the comparisons only), so
    /// they end 30 above the room and its pictures start 20 under the captions above. The gaps:
    /// [`row_xs`].
    fn places(&self, slots: &[Slot], width: i64, bottom: i64, bugs: Bugs) -> Vec<(i64, i64)> {
        let [first, second] = self.heights;
        let mut caption_top = bottom - first[1] - CAPTION_BOTTOM;
        if self.counts[1] > 0 {
            caption_top -= ROW_GAP + second[1] + second[0];
        }
        let gap = self.overhang + PICTURE_GAP;
        // The space beside a row's outer pictures (CLT-Q77 (b)): the executable's 40 alone, the
        // client's the overhang's too, as between the middle two.
        let outer = match bugs {
            Bugs::Fixed => gap,
            Bugs::Original => PICTURE_GAP,
        };
        // The caption height the step under a row takes: the executable's the second row's.
        let above = match bugs {
            Bugs::Fixed => first[1],
            Bugs::Original => second[1],
        };
        let mut out = Vec::new();
        let mut row_slots = slots;
        for &k in self.counts.iter().filter(|&&k| k > 0) {
            let (row, rest) = row_slots.split_at(k.min(row_slots.len()));
            row_slots = rest;
            let widths: Vec<i64> = row.iter().map(|s| s.size[0]).collect();
            let xs = row_xs(width, &widths, (gap, outer));
            out.extend(xs.into_iter().map(|x| (x, caption_top)));
            caption_top += above + second[0] + ROW_GAP;
        }
        out
    }
}

/// The pictures' x in a row of the pictures `widths` (one to four) in a box `width` wide, `gap`
/// the space between the middle two (the overhang's and 40), `outer` the space beside the outer
/// ones of three or four (the executable's 40, the client's `gap`: CLT-Q77 (b); the switch at
/// 0x4F6129; halves rounded toward zero).
fn row_xs(width: i64, widths: &[i64], (gap, outer): (i64, i64)) -> Vec<i64> {
    let w = |i: usize| widths.get(i).copied().unwrap_or(0);
    match widths.len() {
        1 => vec![(width - w(0)) / 2],
        2 => vec![(width - gap) / 2 - w(0), (width + gap) / 2],
        3 => {
            let x1 = (width - w(1)) / 2;
            vec![x1 - w(0) - outer, x1, x1 + w(1) + outer]
        }
        _ => {
            let x1 = (width - gap) / 2 - w(1);
            let x2 = (width + gap) / 2;
            vec![x1 - w(0) - outer, x1, x2, x2 + w(2) + outer]
        }
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
/// and bottom edges, the left and right edges. A box of at most [`BOX_MOST`] has at most 47 (ids
/// 200 to 246: none reaches the buttons' or the pictures').
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
            caption: None,
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
    /// A button's click closed the box: its id ([`OK`], [`YES`], [`NO`] or [`CANCEL`]).
    Closed(u16),
}

/// The box on the screen: its layout, its buttons' states, the button a press or a key holds and
/// the picture chosen (styles 7 and 10).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Open {
    /// The laid-out box.
    pub message: MessageBox,
    held: Option<Held>,
    chosen: Option<u16>,
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
            chosen: None,
        }
    }

    /// The picture chosen (E24 M4 T2b2 part B: `message_box_chosen` 0x6993D4, [`CHOICE`] or
    /// [`CHOICE`] + 1), none while none is.
    #[must_use]
    pub const fn chosen(&self) -> Option<u16> {
        self.chosen
    }

    /// The left button pressed at `point`: a button there goes down (the DEF button's handler
    /// 0x456328), unless disabled; since E24 M4 T2b2 part B a choice frame there is chosen
    /// ([`Open::choose`]; *provisional*: at the press, the frame's own message not read).
    pub fn press(&mut self, point: [i32; 2]) -> Outcome {
        if self.held.is_some() {
            return Outcome::Nothing;
        }
        if let Some(Item::Area(a)) = self.message.dialog.item_at(point)
            && is_choice(a.id)
        {
            return self.choose(a.id);
        }
        match self.message.dialog.button_at(point) {
            Some(id) if !self.message.shown.state(id).disabled => self.hold(Held {
                id,
                key: None,
                over: true,
            }),
            _ => Outcome::Nothing,
        }
    }

    /// A choice frame's command (`dialog_default_proc` 0x4F0FC0, styles 7 and 10): frame `id`
    /// shown and the other hidden, the OK enabled, the picture chosen; the box drawn again (whole,
    /// as the dialog's redraw). Only the first two frames are handled (a third does nothing).
    fn choose(&mut self, id: u16) -> Outcome {
        let other = match id {
            CHOICE => CHOICE + 1,
            _ if id == CHOICE + 1 => CHOICE,
            _ => return Outcome::Nothing,
        };
        let states = &mut self.message.shown.states;
        states.entry(id).or_default().hidden = false;
        if let Some(state) = states.get_mut(&other) {
            state.hidden = true;
        }
        states.entry(OK).or_default().disabled = false;
        self.chosen = Some(id);
        Outcome::Changed
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
            Key::Cancel => &[NO, CANCEL],
        };
        // A disabled button takes no key (a choice's OK before its choice).
        let has = |id: &&u16| {
            !self.message.shown.state(**id).disabled
                && self
                    .message
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
        self.message.shown.states.entry(id).or_default().pressed = pressed;
    }
}

#[cfg(test)]
mod tests;
