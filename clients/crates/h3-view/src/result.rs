//! The battle's result window (`battle_result_dialog` 0x4702D0; E17 M5 T4b1,
//! `docs/epics/E17-design.md` §7.20): the two sides' heroes (or each side's strongest creature),
//! each side's fate, the result's text, each side's losses and an OK, over the whole screen
//! darkened (`battle_end` 0x47782C). [`layout`] builds its items as `ui` items in the executable's
//! order from what it shows; [`text`] picks the result's text, [`shown_side`] the side whose view it
//! shows, [`experience`] the points the text names and [`losses`] a side's row of losses. The
//! window takes input as a message box does (`crate::message::Open`): its OK (Enter or Escape)
//! closes it.
//!
//! *Not built:* the clip it plays in its slot at (107, 70), 256 × 120 (`video_play` 0x597570 from
//! 0x471A24: `WIN3`, `LBSTART`, `RTSTART`, `surrender`, `defendall`, `LOSECSTL` by the shown side's
//! fate, from `VIDEO.VID`; T4b2): the background's slot is drawn bare, as the original shows it
//! without its videos; a network game's timeout (0x471BBB).

use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use h3_content::CreatureTypeId;
use h3_contracts::{StackOrigin, StackRecord};
use serde::Deserialize;

use crate::message::MessageBox;
use crate::scene::{Draw, Moment};
use crate::text::{Args, Catalogue};
use crate::ui::{Button, Def, Dialog, Frames, Item, Label, Picture, Shown};

/// The OK (`iOkay.def`, 0x4715C3: id 0x7802, Enter and Escape).
pub const OK: u16 = 0x7802;
/// The background (`CPResult.pcx`, id 200), in the viewing player's colours (0x4703A0).
pub const BACKGROUND: u16 = 200;
/// The attacker's name and portrait (0xC9, 0xCA), the text item beside it the executable leaves
/// empty (0xCB); the defender's (0xCC, 0xCD, 0xCE); the result's text (0xCF).
pub const ATTACKER_NAME: u16 = 0xC9;
/// The attacker's portrait.
pub const ATTACKER_PORTRAIT: u16 = 0xCA;
/// The defender's name.
pub const DEFENDER_NAME: u16 = 0xCC;
/// The defender's portrait.
pub const DEFENDER_PORTRAIT: u16 = 0xCD;
/// The result's text.
pub const TEXT: u16 = 0xCF;
/// The items the executable gives the id 200 (none) take ids of their own here, from this one,
/// so that their texts can be told apart ([`Shown::texts`]): the title, the two sides' titles,
/// the two sides' fates, the OK's back, then each side's row of losses from
/// [`LOSSES`] + 0x10 × the side.
const OWN_IDS: u16 = 0x100;
/// A side's row of losses: its "None", or each loss's picture and count (two ids each).
const LOSSES: u16 = 0x110;
/// The most losses a row shows (0x47102D).
pub const LOSSES_SHOWN: usize = 7;
/// The combat cursor's frame while the window is open (the pointer: `battle_end` sets it,
/// `SetCursor(6, 2)` at 0x4777FC, and the window sets none).
pub const CURSOR_FRAME: u32 = 6;

/// The window's assets and fonts (the layout's `[result]`): the executable's literals.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResultLayout {
    /// The background (`CPResult.pcx`, 470 × 561).
    pub picture: String,
    /// The titles' font (`BigFont.fnt`).
    pub title_font: String,
    /// The other texts' font (`smalfont.fnt`).
    pub font: String,
    /// The palette the fonts' colours come from (`game.pal`).
    pub font_palette: String,
    /// A side without a hero's portrait: its strongest creature's (`TwCrPort.def`, 58 × 64, the
    /// frame its original id + 2).
    pub creatures: String,
    /// A loss's picture (`cprsmall.def`, 32 × 32, the frame its original id + 2).
    pub losses: String,
    /// The OK's back (`Box64x30.pcx`).
    pub button_back: String,
    /// The OK (`iOkay.def`).
    pub ok: String,
}

/// What a side's corner shows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Face {
    /// Its hero: its name and its large portrait (`hero_table`'s +0x34 by the hero's portrait).
    Hero {
        /// The name.
        name: String,
        /// The portrait (`HPL*.pcx`).
        portrait: String,
    },
    /// Its strongest creature ([`strongest`]): its name, the plural when the side had more than
    /// one stack, and its original id.
    Creature {
        /// The name.
        name: String,
        /// The original id.
        original: u32,
    },
}

/// What the window shows of a side.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Side {
    /// Its corner.
    pub face: Face,
    /// Whether it won (its word "Victorious", else "Defeated").
    pub victorious: bool,
    /// Its losses ([`losses`]): each creature's original id and how many it lost.
    pub losses: Vec<(u32, u32)>,
}

/// The window's texts ([`text`] for the result's).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Texts {
    /// "Battlefield Casualties".
    pub title: String,
    /// "Attacker".
    pub attacker: String,
    /// "Defender".
    pub defender: String,
    /// "Victorious".
    pub victorious: String,
    /// "Defeated".
    pub defeated: String,
    /// A row without losses' "None".
    pub none: String,
    /// The result's text.
    pub result: String,
}

impl Texts {
    /// The window's own texts from `catalogue`, the result's `result`.
    #[must_use]
    pub fn of(catalogue: &Catalogue, result: String) -> Self {
        let text = |key: &str| catalogue.text(key, &Args::new());
        Self {
            title: text("result.title"),
            attacker: text("result.attacker"),
            defender: text("result.defender"),
            victorious: text("result.victorious"),
            defeated: text("result.defeated"),
            none: text("result.none"),
            result,
        }
    }
}

/// The window's place on the screen (0x4702FE: 165, 19) and size (470 × 561); it casts a shadow
/// (the flag 0x10).
const PLACE: [i32; 2] = [165, 19];
const SIZE: [i32; 2] = [470, 561];
/// The top of a side's row of losses (0x470E62: 343 for the attacker, 440 for the defender).
const LOSSES_TOP: [i32; 2] = [343, 440];
/// A loss's step and picture size (42, 32 × 32), the row's width it is centred in (468) and its
/// left margin (11): 0x471044 to 0x471060.
const LOSS_STEP: i32 = 42;
const LOSS_SIZE: i32 = 32;
const LOSS_ROW: i32 = 468;
const LOSS_LEFT: i32 = 11;

/// The window laid out as the executable builds it (0x4702D0, the items in its order), its
/// background in the colours of `player` (the viewing player's, `viewing_player` 0x4CEA20).
#[must_use]
pub fn layout(layout: &ResultLayout, texts: &Texts, sides: &[Side; 2], player: u8) -> MessageBox {
    let mut items = Vec::new();
    let mut shown_texts = BTreeMap::new();
    let mut label = |id: u16, at: [i32; 2], size: [i32; 2], (font, colour, align), text: &str| {
        shown_texts.insert(id, String::from(text));
        Item::Label(Label {
            id,
            at,
            size,
            font: String::from(font),
            font_palette: layout.font_palette.clone(),
            colour,
            align,
            hint: None,
            repeat: None,
        })
    };
    let (big, small) = (layout.title_font.as_str(), layout.font.as_str());
    items.push(Item::Picture(Picture {
        id: BACKGROUND,
        picture: layout.picture.clone(),
        at: [0, 0],
        size: Some(SIZE),
        hint: None,
    }));
    // The titles (0x4703BA to 0x4704BC): records 408 to 410, the big font.
    items.push(label(OWN_IDS, [0, 290], [466, 100], (big, 8, 1), &texts.title));
    items.push(label(OWN_IDS + 1, [0, 320], [466, 100], (big, 4, 1), &texts.attacker));
    items.push(label(OWN_IDS + 2, [0, 412], [466, 100], (big, 4, 1), &texts.defender));
    // Each side's name and portrait (0x4704DD to 0x470962), and the text item beside it the
    // executable leaves empty.
    let [attacker, defender] = sides;
    items.push(label(
        ATTACKER_NAME,
        [89, 37],
        [115, 20],
        (small, 4, 0),
        face_name(&attacker.face),
    ));
    items.push(portrait(layout, ATTACKER_PORTRAIT, [21, 38], &attacker.face, false));
    items.push(label(0xCB, [83, 112], [65, 20], (small, 4, 2), ""));
    items.push(label(
        DEFENDER_NAME,
        [266, 37],
        [115, 20],
        (small, 4, 2),
        face_name(&defender.face),
    ));
    items.push(portrait(layout, DEFENDER_PORTRAIT, [392, 38], &defender.face, true));
    items.push(label(0xCE, [266, 38], [115, 20], (small, 4, 0), ""));
    // Each side's word (0x4709D5, 0x470A49).
    let word = |s: &Side| {
        if s.victorious {
            &texts.victorious
        } else {
            &texts.defeated
        }
    };
    items.push(label(OWN_IDS + 3, [17, 116], [84, 20], (small, 1, 1), word(attacker)));
    items.push(label(OWN_IDS + 4, [367, 116], [84, 20], (small, 1, 1), word(defender)));
    // The result's text (0x470D1E): centred both ways.
    items.push(label(TEXT, [68, 202], [332, 70], (small, 4, 5), &texts.result));
    // Each side's losses (0x470E5C to 0x47140C): "None", or at most seven pictures centred in the
    // row, each with its count under it.
    for (index, side) in sides.iter().enumerate() {
        let top = LOSSES_TOP[index];
        let first = LOSSES + 0x10 * u16::try_from(index).unwrap_or(0);
        if side.losses.is_empty() {
            items.push(label(first, [42, top + 10], [384, 36], (small, 1, 1), &texts.none));
            continue;
        }
        let shown = side.losses.len().min(LOSSES_SHOWN);
        let count = i32::try_from(shown).unwrap_or(0);
        let mut x = (LOSS_ROW - LOSS_STEP * count) / 2 + LOSS_LEFT;
        for (k, &(original, lost)) in side.losses.iter().take(shown).enumerate() {
            let id = first + 2 * u16::try_from(k).unwrap_or(0);
            items.push(Item::Def(Def {
                id,
                def: Some(layout.losses.clone()),
                at: [x, top],
                size: [LOSS_SIZE, LOSS_SIZE],
                frame: original + 2,
                mirrored: false,
                repeat: None,
            }));
            items.push(label(
                id + 1,
                [x - 5, top + 38],
                [LOSS_SIZE, LOSS_SIZE],
                (small, 1, 1),
                &format!("{lost}"),
            ));
            x += LOSS_STEP;
        }
    }
    // The OK (0x471427, 0x4715B4): its back, the button with Enter and Escape.
    items.push(Item::Picture(Picture {
        id: OWN_IDS + 5,
        picture: layout.button_back.clone(),
        at: [384, 506],
        size: Some([66, 32]),
        hint: None,
    }));
    items.push(Item::Button(Button {
        id: OK,
        def: layout.ok.clone(),
        at: [385, 507],
        size: [64, 30],
        frames: Frames::default(),
        hint: None,
        keys: alloc::vec![String::from("Return"), String::from("Escape")],
    }));
    let mut players = BTreeMap::new();
    players.insert(BACKGROUND, player);
    MessageBox {
        dialog: Dialog {
            place: PLACE,
            shadow: Some(SIZE),
            items,
        },
        shown: Shown {
            texts: shown_texts,
            players,
            ..Shown::default()
        },
    }
}

/// The whole screen darkened under the window (`battle_end` 0x47782C: `screen_darken` over
/// 800 × 600), drawn before the window's own draws.
#[must_use]
pub fn backdrop(frame: [u32; 2]) -> Draw {
    let [w, h] = frame.map(|v| i32::try_from(v).unwrap_or(i32::MAX));
    Draw::Darken { rect: [0, 0, w, h] }
}

/// The battle behind the window (`battle_end` darkens the screen as shown): the moment at the
/// battle's end `now`, its heroes' flags as the moment just before showed them (`before`): the
/// end's last action is drawn, but the idle step that would turn the flags then never runs.
pub fn behind(now: &mut Moment, before: &Moment) {
    for (hero, shown) in now.heroes.iter_mut().zip(&before.heroes) {
        if let (Some(hero), Some(shown)) = (hero.as_mut(), shown) {
            hero.flag = shown.flag;
        }
    }
}

/// A side's name.
fn face_name(face: &Face) -> &str {
    match face {
        Face::Hero { name, .. } | Face::Creature { name, .. } => name,
    }
}

/// A side's portrait at `at`: its hero's picture (0x47054A, 0x4707BB), or its creature's frame of
/// `TwCrPort.def` (0x470681, 0x4708F2), the defender's mirrored.
fn portrait(layout: &ResultLayout, id: u16, at: [i32; 2], face: &Face, defender: bool) -> Item {
    match face {
        Face::Hero { portrait, .. } => Item::Picture(Picture {
            id,
            picture: portrait.clone(),
            at,
            size: Some([58, 64]),
            hint: None,
        }),
        Face::Creature { original, .. } => Item::Def(Def {
            id,
            def: Some(layout.creatures.clone()),
            at,
            size: [58, 64],
            frame: original + 2,
            mirrored: defender,
            repeat: None,
        }),
    }
}

/// How a side gave up the battle, if one did (the flags 0x6985A3 and 0x697744, either side's).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GaveUp {
    /// A side retreated.
    Retreat,
    /// A side surrendered.
    Surrender,
}

/// The side whose view the window shows (`battle_end` 0x4778B1): the winner's when its player is
/// a human here, else the other one; after a draw the defender's (1 − (−1) taken as the
/// defender's hero).
#[must_use]
pub fn shown_side(winner: Option<usize>, human: [bool; 2]) -> usize {
    match winner {
        Some(w) if human.get(w).copied().unwrap_or(false) => w,
        Some(w) => 1 - w.min(1),
        None => 1,
    }
}

/// The result's text (0x470AB4 to 0x470CFF): the shown side won (`won`): the enemy surrendered,
/// fled or fell, then the shown side's hero's `experience` when it has a hero (`hero`, its name);
/// it lost: it surrendered, fled or fell, its hero named when it has one.
#[must_use]
pub fn text(
    catalogue: &Catalogue,
    won: bool,
    gave_up: Option<GaveUp>,
    hero: Option<&str>,
    experience: u32,
) -> String {
    let plain = |key: &str| catalogue.text(key, &Args::new());
    if won {
        let mut text = plain(match gave_up {
            Some(GaveUp::Surrender) => "result.won.surrendered",
            Some(GaveUp::Retreat) => "result.won.fled",
            None => "result.won",
        });
        if let Some(name) = hero {
            let args = Args::new()
                .text("hero", name)
                .number("experience", i64::from(experience));
            text.push_str(&catalogue.text("result.experience", &args));
        }
        return text;
    }
    match hero {
        Some(name) => {
            let key = match gave_up {
                Some(GaveUp::Surrender) => "result.lost.surrendered",
                Some(GaveUp::Retreat) => "result.lost.fled",
                None => "result.lost",
            };
            catalogue.text(key, &Args::new().text("hero", name))
        }
        None => plain(match gave_up {
            Some(GaveUp::Surrender) => "result.lost.surrendered-army",
            Some(GaveUp::Retreat) => "result.lost.fled-army",
            None => "result.lost-army",
        }),
    }
}

/// The records the window reads of a side: its first 20 (the executable's loops run over 20 of a
/// side's 21 stack slots).
const RECORDS: usize = 20;
/// The arrow tower (149), which a side's strongest creature never is (0x4705BD).
const ARROW_TOWER: u32 = 149;
/// The points a beaten hero gives (0x46A3DF), taken back after a retreat or a surrender
/// (0x46A3FC), and given to an attacker that took a town (0x46A416).
const HERO_POINTS: i64 = 500;

/// A side's losses (0x470D89 to 0x470E48): over its records' (`stacks`, the engine's, in stack
/// order) creatures lost (`start − end`), each creature's sum in the order it first lost some, by
/// its original id (`original`; a creature without one is left out).
#[must_use]
pub fn losses(stacks: &[StackRecord], original: impl Fn(CreatureTypeId) -> Option<u32>) -> Vec<(u32, u32)> {
    let mut out: Vec<(u32, u32)> = Vec::new();
    for record in stacks.iter().take(RECORDS) {
        let lost = record.start.saturating_sub(record.end);
        let Some(id) = original(record.creature).filter(|_| lost > 0) else {
            continue;
        };
        match out.iter_mut().find(|(o, _)| *o == id) {
            Some((_, sum)) => *sum = sum.saturating_add(lost),
            None => out.push((id, lost)),
        }
    }
    out
}

/// A side without a hero's face (0x4705A2 to 0x470627): of its records but the arrow towers, the
/// first with the highest AI value (`ai_value`), its name plural when there are several
/// (`more`), as the executable reads them; its creature and whether there are several.
#[must_use]
pub fn strongest(
    stacks: &[StackRecord],
    original: impl Fn(CreatureTypeId) -> Option<u32>,
    ai_value: impl Fn(CreatureTypeId) -> u32,
) -> Option<(CreatureTypeId, bool)> {
    let mut best: Option<(CreatureTypeId, u32)> = None;
    let mut count = 0;
    for record in stacks.iter().take(RECORDS) {
        if original(record.creature) == Some(ARROW_TOWER) {
            continue;
        }
        count += 1;
        let value = ai_value(record.creature);
        if best.is_none_or(|(_, b)| value > b) {
            best = Some((record.creature, value));
        }
    }
    best.map(|(creature, _)| (creature, count > 1))
}

/// The points the result's text says the winner's hero receives (`battle_experience` 0x46A350):
/// the hit points (`hit_points`) of the creatures the loser's army and raised stacks lost
/// (`loser`, its records; not its war machines nor the stacks the battle created, which the
/// engine's [`StackOrigin`] tells as the executable's flags do), 500 more when the loser had a
/// hero (`beaten_hero`) unless a side gave up (`gave_up`), 500 more when the attacker took a town
/// (`town_taken`). *Not built:* the winner's Learning (×, `crt_ftol`): the client's heroes have
/// none (*provisional*: the scenarios name no Learning).
#[must_use]
pub fn experience(
    loser: &[StackRecord],
    hit_points: impl Fn(CreatureTypeId) -> u32,
    (beaten_hero, gave_up, town_taken): (bool, bool, bool),
) -> u32 {
    let killed: i64 = loser
        .iter()
        .take(RECORDS)
        .filter(|s| matches!(s.origin, StackOrigin::Army(_) | StackOrigin::Raised(_)))
        .map(|s| i64::from(s.start.saturating_sub(s.end)) * i64::from(hit_points(s.creature)))
        .sum();
    let mut points = killed;
    if beaten_hero {
        points += HERO_POINTS;
    }
    if gave_up {
        points -= HERO_POINTS;
    }
    if town_taken {
        points += HERO_POINTS;
    }
    u32::try_from(points.max(0)).unwrap_or(u32::MAX)
}

#[cfg(test)]
mod tests;
