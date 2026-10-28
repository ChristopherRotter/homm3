//! Dialog items as reusable components (E17 M3 T6a; `docs/epics/E17-design.md` §9.7). The
//! executable builds every screen's controls from a few kinds of dialog item, each drawn by its
//! kind's own drawer whatever the screen: a picture (H3API's `H3DlgPcx`), a button of DEF frames
//! (`H3DlgDefButton`, and its kin `H3DlgCustomButton` and `H3DlgCaptionButton`, which share its
//! drawer 0x456940), a text over a picture (`H3DlgTextPcx`); since E17 M5 T3a a DEF's frame
//! (`H3DlgDef`), a text alone (`H3DlgText`) and an area that takes the mouse and draws nothing
//! (`H3DlgTransparentItem`). A layout file lists a dialog's items in the order the executable adds
//! them, which is the order they are drawn, each with the executable's control id (a loop's items
//! as one entry with its `repeat`, [`Dialog::expanded`]); a screen's logic sets the items' states
//! by id ([`Shown`]), and [`Dialog::draws`] turns both into the draw list, of the dialog's own draw
//! items ([`Draw`]: a viewer's scene takes them into its list and its compositor draws them). The
//! battle's panel is the first dialog (`h3-view`'s `panel`), the spell book the second (its
//! `book`). The format is *provisional* (E17's M3 review list). Common to both viewers since E24
//! M3 T1a (the adventure screen's dialog, E24 M3 T2).

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use serde::Deserialize;

/// One thing a dialog draws, in the order of its list (E17 M3 T6a, E17-design §9.7; the battle
/// scene's until E24 M3 T1a, which made them the dialog's own).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Draw {
    /// A dialog's picture (`H3DlgPcx`): index 0 transparent, in its own colours or a player's, at
    /// most its item's size of it from its corner (`dialog_picture_draw` 0x450450; E17 M5 T4a: a
    /// message box's background tiles cut to the box).
    Picture {
        /// The picture.
        picture: String,
        /// Its left edge.
        x: i32,
        /// Its top edge.
        y: i32,
        /// The player's colours over its entries 224–255, if any.
        player: Option<PlayerColours>,
        /// The item's width and height, none for the whole picture.
        size: Option<[i32; 2]>,
    },
    /// A dialog's button (`H3DlgDefButton`) or DEF item (`H3DlgDef`, E17 M5 T3a): a frame of its
    /// DEF's group 0 (frame 0 when the group has no such frame), by the plain drawer with runs
    /// skipped; a DEF item in a player's colours once the screen gives it one (the message
    /// 0x200/0xD, `dialog_def_message` 0x4EA810; E17 M5 T4a: a message box's frame).
    Button {
        /// The DEF.
        def: String,
        /// The frame in group 0 its state shows.
        frame: u32,
        /// The canvas's left edge.
        x: i32,
        /// The canvas's top edge.
        y: i32,
        /// The player's colours over the DEF's entries 224–255, if any.
        player: Option<PlayerColours>,
    },
    /// A dialog's text over a picture (`H3DlgTextPcx`; the battle log's item, E17-design §9.5; E17
    /// M3 T4b-2b-1, T6a): the item's picture, then the text in its font over it.
    Text {
        /// The item: its picture, size, font and how the text is drawn.
        item: Text,
        /// Its left edge.
        x: i32,
        /// Its top edge.
        y: i32,
        /// The text (the battle log's: its shown entries, one per line, 0x472C30; the status
        /// line's text instead when it shows).
        text: String,
    },
    /// A rectangle of the screen darkened (`screen_darken` 0x44E5F0: each channel halved; E17 M5
    /// T3a: a dialog's shadow, [`Dialog::shadow_draws`]).
    Darken {
        /// Its left edge, top edge, width and height.
        rect: [i32; 4],
    },
    /// A dialog's text alone (`H3DlgText`, E17 M5 T3a: the spell book's): the text in the item's
    /// box in its font, its colour 2 more while disabled (0x5BC5F0).
    Label {
        /// The item: its size, font and how the text is drawn.
        item: Label,
        /// Its left edge.
        x: i32,
        /// Its top edge.
        y: i32,
        /// The text.
        text: String,
        /// The item is disabled (its state's bit 8).
        disabled: bool,
    },
}

/// A picture's or a DEF item's player colours (0x450520, 0x4EAA4A): the players' palette and the
/// player's colour (0 red to 7 pink), whose 32 entries of that palette replace its 224–255.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerColours {
    /// The players' palette (`PLAYERS.PAL`).
    pub palette: String,
    /// The player's colour.
    pub player: u8,
}

/// A dialog: its corner on the screen and its items.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Dialog {
    /// The dialog's top-left corner on the screen; each item's place is from it.
    pub place: [i32; 2],
    /// The dialog's width and height when it casts a shadow (its flag 0x10, `vShow` 0x5FEAE0; E17
    /// M5 T3a): none for none.
    #[serde(default)]
    pub shadow: Option<[i32; 2]>,
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
    /// A DEF's frame (`H3DlgDef`; E17 M5 T3a).
    Def(Def),
    /// A text alone (`H3DlgText`; E17 M5 T3a).
    Label(Label),
    /// An area that takes the mouse and draws nothing (`H3DlgTransparentItem`; E17 M5 T3a).
    Area(Area),
}

impl Item {
    /// Where the item takes the mouse, from the dialog's corner: its corner and size (a picture
    /// only with its `size`: *provisional*, design §9.9).
    #[must_use]
    pub const fn area(&self) -> Option<([i32; 2], [i32; 2])> {
        match self {
            Self::Picture(Picture {
                at,
                size: Some(size),
                ..
            }) => Some((*at, *size)),
            Self::Picture(_) => None,
            Self::Button(b) => Some((b.at, b.size)),
            Self::Text(t) => Some((t.at, t.size)),
            Self::Def(d) => Some((d.at, d.size)),
            Self::Label(l) => Some((l.at, l.size)),
            Self::Area(a) => Some((a.at, a.size)),
        }
    }

    /// The key of the item's hint, if it has one.
    #[must_use]
    pub fn hint(&self) -> Option<&str> {
        match self {
            Self::Picture(p) => p.hint.as_deref(),
            Self::Button(b) => b.hint.as_deref(),
            Self::Text(t) => t.hint.as_deref(),
            Self::Def(_) => None,
            Self::Label(l) => l.hint.as_deref(),
            Self::Area(a) => a.hint.as_deref(),
        }
    }

    /// The item's control id.
    #[must_use]
    pub const fn id(&self) -> u16 {
        match self {
            Self::Picture(p) => p.id,
            Self::Button(b) => b.id,
            Self::Text(t) => t.id,
            Self::Def(d) => d.id,
            Self::Label(l) => l.id,
            Self::Area(a) => a.id,
        }
    }

    /// The items a layout entry stands for: itself, or its `repeat`'s copies ([`Repeat::places`]).
    fn copies(self) -> Vec<Self> {
        match self {
            Self::Def(d) => match d.repeat {
                Some(r) => r
                    .places(d.id, d.at)
                    .into_iter()
                    .map(|(id, at)| {
                        Self::Def(Def {
                            id,
                            at,
                            repeat: None,
                            ..d.clone()
                        })
                    })
                    .collect(),
                None => alloc::vec![Self::Def(d)],
            },
            Self::Label(l) => match l.repeat {
                Some(r) => r
                    .places(l.id, l.at)
                    .into_iter()
                    .map(|(id, at)| {
                        Self::Label(Label {
                            id,
                            at,
                            repeat: None,
                            ..l.clone()
                        })
                    })
                    .collect(),
                None => alloc::vec![Self::Label(l)],
            },
            other => alloc::vec![other],
        }
    }
}

/// A layout entry's loop (E17 M5 T3a): its copies' columns and rows and the steps between them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Repeat {
    /// The columns and the rows.
    pub count: [i32; 2],
    /// A column's step in x and a row's in y.
    pub step: [i32; 2],
}

impl Repeat {
    /// The copies' ids and places from the first's: row by row, columns inside rows (a
    /// constructor's loops: the spell book's places, 0x59BF90), the ids following on.
    #[must_use]
    pub fn places(self, id: u16, at: [i32; 2]) -> Vec<(u16, [i32; 2])> {
        let mut out = Vec::new();
        for row in 0..self.count[1] {
            for column in 0..self.count[0] {
                let place = [at[0] + self.step[0] * column, at[1] + self.step[1] * row];
                out.push((
                    id.wrapping_add(u16::try_from(out.len()).unwrap_or(u16::MAX)),
                    place,
                ));
            }
        }
        out
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
    /// Its size where it takes the mouse (its picture's; E17 M4 T3b: the battle's panel picture
    /// is entered as the hovered item, `hovered_item` 0x6AAC68), and the most of its picture it
    /// draws from the picture's corner (`dialog_picture_draw` 0x450450; E17 M5 T4a: a message
    /// box's background tiles); none: it takes none and draws the whole picture
    /// (*provisional*, design §9.9).
    #[serde(default)]
    pub size: Option<[i32; 2]>,
    /// The key of the status line's text while the mouse is over it, none for none (E17 M5 T3a:
    /// the spell book's corners).
    #[serde(default)]
    pub hint: Option<String>,
}

/// A button of DEF frames (`H3DlgDefButton` 0x455EF0, drawn by 0x456940): a frame of the DEF's
/// group 0 by the button's state, its canvas's corner at the button's, by the plain drawer with
/// runs skipped (0x47BF00 → 0x47C570). A frame beyond the group's is drawn as frame 0. Its
/// hotkeys (its +0x48, the screens' scan codes) are the viewer's table of keys' (`keys`; E24 M3
/// T1b: until then the layout's `keys`).
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

/// A DEF's frame (`H3DlgDef`, 0x4EA720; E17 M5 T3a, design §7.17): its frame of group 0 at its
/// corner, drawn by the DEF's type (0x4EAB40 with the flags 0x10): an interface DEF's (type 0x47,
/// the spell book's) by the buttons' frame drawer (0x47BF00), runs skipped. A screen may give it
/// another DEF (0x4EB030) and frame (0x4EAFF0) by id ([`Shown`]).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Def {
    /// The control id (a `repeat`'s first).
    pub id: u16,
    /// The DEF, none until the screen gives one.
    #[serde(default)]
    pub def: Option<String>,
    /// Its top-left corner from the dialog's.
    pub at: [i32; 2],
    /// Its width and height, where it takes the mouse.
    pub size: [i32; 2],
    /// The frame of group 0.
    #[serde(default)]
    pub frame: u32,
    /// The constructor's loop, if the entry stands for several.
    #[serde(default)]
    pub repeat: Option<Repeat>,
}

/// A text alone (`H3DlgText`, 0x5BC280, drawn by 0x5BC5F0; E17 M5 T3a): the text in its box by the
/// text drawer (§3.6) with the item's alignment and its colour, 2 more while it is disabled (its
/// state's bit 8, 0x5BC641). Its background colour, 0 in every one built, fills nothing.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Label {
    /// The control id (a `repeat`'s first).
    pub id: u16,
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
    /// The constructor's loop, if the entry stands for several.
    #[serde(default)]
    pub repeat: Option<Repeat>,
}

/// An area that takes the mouse and draws nothing (`H3DlgTransparentItem`, 0x44FF10; E17 M5 T3a:
/// the spell book's tabs, bookmarks and close).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Area {
    /// The control id.
    pub id: u16,
    /// Its top-left corner from the dialog's.
    pub at: [i32; 2],
    /// Its width and height.
    pub size: [i32; 2],
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

/// What a dialog's items show, by control id; an item without an entry shows its default: in the
/// default [`State`], a picture in its own colours, a text empty, a DEF item its own DEF and frame.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Shown {
    /// The items' states (a button's frame; any item hidden; a text alone's colour when
    /// disabled).
    pub states: BTreeMap<u16, State>,
    /// The pictures and DEF items in a player's colours: the player's colour (0 red to 7 pink; a
    /// DEF item's since E17 M5 T4a, the message 0x200/0xD of `dialog_def_message` 0x4EA810).
    pub players: BTreeMap<u16, u8>,
    /// The texts.
    pub texts: BTreeMap<u16, String>,
    /// The DEF items' DEFs the screen gave them (0x4EB030; E17 M5 T3a).
    pub defs: BTreeMap<u16, String>,
    /// The DEF items' frames the screen gave them (0x4EAFF0; E17 M5 T3a).
    pub frames: BTreeMap<u16, u32>,
}

impl Shown {
    /// The state of item `id`.
    #[must_use]
    pub fn state(&self, id: u16) -> State {
        self.states.get(&id).copied().unwrap_or_default()
    }
}

impl Dialog {
    /// The dialog with each entry's `repeat` made into its copies ([`Repeat`]), in order.
    #[must_use]
    pub fn expanded(self) -> Self {
        Self {
            items: self.items.into_iter().flat_map(Item::copies).collect(),
            ..self
        }
    }

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

    /// The item that takes the mouse at `point` (on the screen) as `shown` (`ItemAtPosition`
    /// 0x5FF3E0, read at E17 M5 T3a: the spell book's): the last in the executable's order whose
    /// area holds it and which is shown and enabled (bit 2) and not disabled (bits 8 and 0x1000).
    #[must_use]
    pub fn item_taking(&self, point: [i32; 2], shown: &Shown) -> Option<&Item> {
        let [x, y] = [point[0] - self.place[0], point[1] - self.place[1]];
        self.items.iter().rev().find(|item| {
            let state = shown.state(item.id());
            !state.hidden
                && !state.disabled
                && item.area().is_some_and(|([left, top], [w, h])| {
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

    /// The shadow a dialog with one casts as it shows (`vShow` 0x5FEAE0 with the flag 0x10):
    /// `screen_darken` four times, a strip right of it 8 wide from 8 below its top and one below
    /// it 8 high from 8 right of its left, each again 7 wide or high one further in: their
    /// insides darkened twice.
    #[must_use]
    pub fn shadow_draws(&self) -> Vec<Draw> {
        let Some([w, h]) = self.shadow else {
            return Vec::new();
        };
        let [x, y] = self.place;
        [
            [x + w, y + 9, 7, h - 9],
            [x + w, y + 8, 8, h - 8],
            [x + 9, y + h, w - 2, 7],
            [x + 8, y + h, w, 8],
        ]
        .into_iter()
        .map(|rect| Draw::Darken { rect })
        .collect()
    }

    /// The draw list of the dialog: its shadow ([`Dialog::shadow_draws`]), then its items as
    /// `shown`, in their order; a player's colours come from the players' palette `players`
    /// (`PLAYERS.PAL`, 32 entries a player).
    #[must_use]
    pub fn draws(&self, shown: &Shown, players: &str) -> Vec<Draw> {
        let [x, y] = self.place;
        let place = |at: [i32; 2]| [x.saturating_add(at[0]), y.saturating_add(at[1])];
        let colours = |id: u16| {
            shown.players.get(&id).map(|&player| PlayerColours {
                palette: players.into(),
                player,
            })
        };
        let items = self
            .items
            .iter()
            .filter(|item| !shown.state(item.id()).hidden)
            .filter_map(|item| match item {
                Item::Picture(p) => {
                    let [x, y] = place(p.at);
                    Some(Draw::Picture {
                        picture: p.picture.clone(),
                        x,
                        y,
                        player: colours(p.id),
                        size: p.size,
                    })
                }
                Item::Button(b) => {
                    let state = shown.state(b.id);
                    let [x, y] = place(b.at);
                    b.frame(state).map(|frame| Draw::Button {
                        def: b.def.clone(),
                        frame,
                        x,
                        y,
                        player: None,
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
                Item::Def(d) => {
                    let [x, y] = place(d.at);
                    let def = shown.defs.get(&d.id).or(d.def.as_ref())?;
                    Some(Draw::Button {
                        def: def.clone(),
                        frame: shown.frames.get(&d.id).copied().unwrap_or(d.frame),
                        x,
                        y,
                        player: colours(d.id),
                    })
                }
                Item::Label(l) => {
                    let [x, y] = place(l.at);
                    Some(Draw::Label {
                        item: l.clone(),
                        x,
                        y,
                        text: shown.texts.get(&l.id).cloned().unwrap_or_default(),
                        disabled: shown.state(l.id).disabled,
                    })
                }
                Item::Area(_) => None,
            });
        self.shadow_draws().into_iter().chain(items).collect()
    }
}

#[cfg(test)]
mod tests;
