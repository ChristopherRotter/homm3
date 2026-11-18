//! A level-up's choice (E24 M4 T2c, *provisional*; `docs/epics/E24-design.md` §5.5): the dialog
//! `level_up_dialog` 0x4F8880 builds for a human's hero that reaches a level (`hero_level_up`
//! 0x4DA720), one dialog a level. [`LevelUp`] is a level-up's facts in the shape of the engine's
//! `h3_game::LevelUpView` (the hero, its new level, the primary skill raised and its offers, none
//! to two, each a secondary skill and the level it reaches); [`LevelArt::level_of`] the level an
//! experience reaches; [`dialog`] the dialog as the layout's `[level_up]` places it, with what
//! its items show through the string catalogue (GENRLTXT 445, 446, 204, 205 and 5, the
//! executable's `%s +1` and `%s\n%s`, the `hero`, `hero-class`, `primary-skill`,
//! `secondary-skill` and `skill-level` families); [`offer_at`] and [`choose`] its procedure's
//! choice (`level_up_dialog_proc` 0x4F9780: the keys 1 and 2, a click on a picture or its frame),
//! [`frame_draws`] the frame shown.
//!
//! *Provisional* (the viewer has no engine): Part A takes the facts from a recording's live part
//! ([`LevelUp::read_all`]: its `[[level_up]]` tables, the original's random draws), as a
//! computer's turns come from its `[[turn]]`; Part B takes them from the engine.
//!
//! *Not built:* a right-click's popup of an offer's description (`message_box` style 4, SSTRAITS'
//! columns: not recorded), the dialog's deadline (a network game's), the computer's choice (9999).

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

use h3_ui::message::{self, MessageBox, OK};
use h3_ui::text::{Args, Catalogue};
use h3_ui::ui::{Dialog, Draw, Item, Shown, State};
use serde::Deserialize;

/// The background's item (`lvlupbkg.pcx` in the hero's owner's colours).
pub const BACKGROUND: u16 = 2000;
/// The hero's large portrait's item.
pub const PORTRAIT: u16 = 2001;
/// The text "… has gained a level" (GENRLTXT 445).
pub const GAINED: u16 = 2002;
/// The text "… is now a level … " (GENRLTXT 446).
pub const NOW: u16 = 2003;
/// The primary skill's text (the executable's `%s +1`).
pub const RAISED: u16 = 2004;
/// The sentence naming the offers (GENRLTXT 205, or 204 for one: never shown, MCL-Q34).
pub const SENTENCE: u16 = 2005;
/// The "or" between two offers (GENRLTXT 5).
pub const OR: u16 = 2006;
/// The offers' captions, left and right (the executable's `%s\n%s`).
pub const CAPTIONS: [u16; 2] = [2007, 2008];
/// The primary skill's picture (`pskil42.def`, its frame the skill).
pub const PRIMARY: u16 = 2009;
/// The offers' pictures, left and right (`secskill.def`, [`Offer::frame`]).
pub const PICTURES: [u16; 2] = [2010, 2011];
/// The offers' frames, left and right (`dialog_frame_create`, the game palette's entry 45).
pub const FRAMES: [u16; 2] = [2012, 2013];

/// The string catalogue's keys the dialog shows (`clients/text/keys.toml`; the "or" is
/// `crate::asks::KEYS`'s `map.ask.or`); the names are the families `hero`, `hero-class`,
/// `primary-skill`, `secondary-skill` and `skill-level`'s.
pub const KEYS: [&str; 6] = [
    "map.level-up.gained",
    "map.level-up.now",
    "map.level-up.primary",
    "map.level-up.learned",
    "map.level-up.either",
    "map.level-up.caption",
];

/// The levels' numbers (the tables' `[map.level_up]`; design §5.5).
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LevelArt {
    /// The experience each level from 1 starts at (`hero_level_up` 0x4DA720's table 0x679C88 to
    /// level 12: 0, 1 000, 2 000, … 20 600; the engine's packs' `experience`).
    pub experience: Vec<u32>,
    /// The sound the dialog's build plays (`nwherolv.82m`, `play_sound_async` at 0x4DAA26).
    pub sound: String,
    /// The secondary skill whose every level learned adds a morale point the status window
    /// shows (Leadership, 6: +1, +2, +3; recorded, `level-up-twice`'s Advanced Leadership at
    /// 5260); none: the morale shown stays.
    #[serde(default)]
    pub morale: Option<u8>,
    /// The one whose every level adds a luck point (Luck, 9).
    #[serde(default)]
    pub luck: Option<u8>,
}

impl LevelArt {
    /// The level `experience` reaches (`hero_level_up` 0x4DA720): the last of the table's
    /// whose start it reaches; beyond the table each level's step a fifth more than the one
    /// before (`crt_ftol` of × 1.2: level 13 at 24 320). *Provisional:* in integers (the game's
    /// f32 steps not reproduced where they lose digits, past 2^24), capped at level 255; level 1
    /// without a table.
    #[must_use]
    pub fn level_of(&self, experience: u32) -> u8 {
        let mut level: u8 = 1;
        let mut last = 0;
        let mut step = 0;
        for &start in self.experience.iter().skip(1) {
            if experience < start {
                return level;
            }
            step = start.saturating_sub(last);
            last = start;
            level = level.saturating_add(1);
        }
        while level < u8::MAX {
            step = step.saturating_add(step / 5);
            let next = last.saturating_add(step);
            if step == 0 || experience < next {
                break;
            }
            last = next;
            level += 1;
        }
        level
    }
}

/// An offer: a secondary skill and the level it reaches (1 Basic to 3 Expert).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Offer {
    /// The skill (0 Pathfinding to 27 First Aid).
    pub skill: u8,
    /// The level it reaches.
    pub level: u8,
}

impl Offer {
    /// The offer `secskill.def`'s frame shows (`hero_level_up` 0x4DAABD: 3 × the skill + 3 + the
    /// hero's level of it before); none for −1 (no offer) or another frame below 3.
    #[must_use]
    pub fn of_frame(frame: i64) -> Option<Self> {
        let past = u8::try_from(frame).ok()?.checked_sub(3)?;
        Some(Self {
            skill: past / 3,
            level: past % 3 + 1,
        })
    }

    /// Its frame of `secskill.def`.
    #[must_use]
    pub fn frame(self) -> u32 {
        3 * u32::from(self.skill) + 2 + u32::from(self.level.clamp(1, 3))
    }

    /// Its level's name and its skill's (the `skill-level` and `secondary-skill` families; empty
    /// past them).
    fn names(self, texts: &Catalogue) -> (String, String) {
        let named = |key: String| texts.known(&key, &Args::new()).unwrap_or_default();
        (
            named(alloc::format!("skill-level.{}", self.level.clamp(1, 3) - 1)),
            named(alloc::format!("secondary-skill.{}", self.skill)),
        )
    }
}

/// A hero's level-up (*provisional*: a recording's `[[level_up]]` table, Part B the engine's
/// `h3_game::LevelUpView`): what its dialog shows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LevelUp {
    /// The hero (its type: the status window's hero).
    pub hero: u8,
    /// The new level.
    pub level: u8,
    /// The primary skill raised by 1 (0 Attack to 3 Knowledge).
    pub primary: u8,
    /// The offers, the dialog's left first (none to two).
    pub offers: Vec<Offer>,
}

impl LevelUp {
    /// A live part's `[[level_up]]` table as the harness writes it (`h3map.py`'s
    /// `level_up_toml`): `hero`, `level`, `offers` (the left and the right picture's
    /// `secskill.def` frame, −1 none) and `pictures` in the items' last-first order (the
    /// harness's: the offers', then the primary skill's `[def, frame]`, the portrait's and the
    /// background's), the primary skill the third from the end; none for anything else.
    #[must_use]
    pub fn read(value: &toml::Value) -> Option<Self> {
        let table = value.as_table()?;
        let byte = |key: &str| {
            table
                .get(key)
                .and_then(toml::Value::as_integer)
                .and_then(|n| u8::try_from(n).ok())
        };
        let list = |key: &str| table.get(key).and_then(toml::Value::as_array);
        let pictures = list("pictures")?;
        let primary = pictures
            .len()
            .checked_sub(3)
            .and_then(|i| pictures.get(i))
            .and_then(toml::Value::as_array)
            .and_then(|p| p.get(1))
            .and_then(toml::Value::as_integer)
            .and_then(|p| u8::try_from(p).ok())
            .filter(|&p| p < 4)?;
        let frames: Vec<i64> = list("offers")?
            .iter()
            .map(toml::Value::as_integer)
            .collect::<Option<_>>()?;
        let offers: Vec<Offer> = frames
            .into_iter()
            .filter(|&frame| frame >= 0)
            .map(Offer::of_frame)
            .collect::<Option<_>>()?;
        Some(Self {
            hero: byte("hero")?,
            level: byte("level")?,
            primary,
            offers,
        })
        .filter(|up| up.offers.len() <= 2)
    }

    /// A live part's `[[level_up]]` tables, one level-up each dialog: the harness appends a
    /// table at each choice and change of the OK's state, so a table of the same hero and level
    /// as the one before is its dialog's again; none if one is not a level-up's.
    #[must_use]
    pub fn read_all(value: &toml::Value) -> Option<Vec<Self>> {
        let mut out: Vec<Self> = Vec::new();
        for table in value.as_array()? {
            let up = Self::read(table)?;
            if out
                .last()
                .is_none_or(|last| (last.hero, last.level) != (up.hero, up.level))
            {
                out.push(up);
            }
        }
        Some(out)
    }

    /// What the dialog's texts show, by item, for a hero named `name` of the class named
    /// `class` (0x4F8880): GENRLTXT 445 with the name, 446 with the name, the level and the
    /// class, the executable's `%s +1` with the primary skill's name; with offers the sentence
    /// naming them (204 for one, 205 for two: each its level's name and its skill's) and each
    /// one's caption (`%s\n%s`), with two the "or" (GENRLTXT 5).
    #[must_use]
    pub fn texts(&self, texts: &Catalogue, name: &str, class: &str) -> BTreeMap<u16, String> {
        let named = |key: String| texts.known(&key, &Args::new()).unwrap_or_default();
        let mut out = BTreeMap::new();
        let named_args = Args::new().text("name", name);
        out.insert(GAINED, texts.text("map.level-up.gained", &named_args));
        let now = named_args
            .number("level", i64::from(self.level))
            .text("class", class);
        out.insert(NOW, texts.text("map.level-up.now", &now));
        let skill = named(alloc::format!("primary-skill.{}", self.primary));
        let raised = Args::new().text("skill", &skill);
        out.insert(RAISED, texts.text("map.level-up.primary", &raised));
        let names: Vec<(String, String)> = self.offers.iter().map(|o| o.names(texts)).collect();
        for ((level, skill), id) in names.iter().zip(CAPTIONS) {
            let args = Args::new().text("level", level).text("skill", skill);
            out.insert(id, texts.text("map.level-up.caption", &args));
        }
        match names.as_slice() {
            [(level, skill)] => {
                let args = Args::new().text("level", level).text("skill", skill);
                out.insert(SENTENCE, texts.text("map.level-up.learned", &args));
            }
            [(level, skill), (second_level, second)] => {
                let args = Args::new()
                    .text("level", level)
                    .text("skill", skill)
                    .text("second_level", second_level)
                    .text("second", second);
                out.insert(SENTENCE, texts.text("map.level-up.either", &args));
                out.insert(OR, texts.text("map.ask.or", &Args::new()));
            }
            _ => {}
        }
        out
    }
}

/// The dialog's layout (the adventure layout's `[level_up]`; design §5.5): its items as
/// `level_up_dialog` 0x4F8880 builds them without an offer (the background, the portrait, the
/// three texts, the primary skill's picture, the OK last) and the items it builds before the OK
/// with two offers (the sentence, the "or", the frames, the pictures, the captions) and with one
/// (the sentence, the left's frame, picture and caption, placed in the middle).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LevelUpLayout {
    /// The dialog: its place (205, 65), its shadow (385 × 470, the message boxes') and its items
    /// with no offer, the OK last.
    pub dialog: Dialog,
    /// The items two offers add.
    pub two: Vec<Item>,
    /// The items one offer adds.
    pub one: Vec<Item>,
}

/// The dialog of `up` laid out (0x4F8880): the layout's items with its offers' before the OK, its
/// `texts` ([`LevelUp::texts`]), the hero's large `portrait`, the background in the colours of
/// `player` (the hero's owner, `dialog_picture_player_colours`), the primary skill's frame and
/// the offers'. Two offers: both frames hidden and the OK disabled (0x4F9507); one: its frame
/// shown, chosen at once (+0x68 = 2010).
#[must_use]
pub fn dialog(
    layout: &LevelUpLayout,
    up: &LevelUp,
    texts: BTreeMap<u16, String>,
    portrait: &str,
    player: u8,
) -> MessageBox {
    let mut items = layout.dialog.items.clone();
    let added = match up.offers.len() {
        0 => &[][..],
        1 => &layout.one[..],
        _ => &layout.two[..],
    };
    let at = items.len().saturating_sub(1);
    items.splice(at..at, added.iter().cloned());
    let mut shown = Shown {
        texts,
        ..Shown::default()
    };
    shown.players.insert(BACKGROUND, player);
    shown.pictures.insert(PORTRAIT, String::from(portrait));
    shown.frames.insert(PRIMARY, u32::from(up.primary));
    for (offer, id) in up.offers.iter().zip(PICTURES) {
        shown.frames.insert(id, offer.frame());
    }
    if up.offers.len() >= 2 {
        let hidden = State {
            hidden: true,
            ..State::default()
        };
        for id in FRAMES {
            shown.states.insert(id, hidden);
        }
        let disabled = State {
            disabled: true,
            ..State::default()
        };
        shown.states.insert(OK, disabled);
    }
    MessageBox {
        dialog: Dialog {
            items,
            ..layout.dialog.clone()
        },
        shown,
    }
}

/// The offer a press at `point` takes in the open dialog `message` (0x4F9780: a click on 2010 or
/// 2012 the left, on 2011 or 2013 the right; *provisional*: at the press, as a choice box's,
/// `h3_ui::message::Open::press`), its place; none elsewhere or where the dialog has no such
/// offer.
#[must_use]
pub fn offer_at(message: &MessageBox, point: [i32; 2]) -> Option<usize> {
    let id = message.dialog.item_at(point)?.id();
    [PICTURES, FRAMES]
        .iter()
        .find_map(|ids| ids.iter().position(|&i| i == id))
}

/// The offer `n` (0 the left, 1 the right) of the open dialog `message` with `offers` offers
/// chosen (0x4F9780: the key 1 or 2, or a click): its frame shown (`dialog_item_send` (5, 4)),
/// the other's hidden (6, 4), the OK enabled; whether there is such an offer (the dialog then
/// drawn again whole: the same offer chosen again draws the same pixels).
pub fn choose(message: &mut MessageBox, n: usize, offers: usize) -> bool {
    if n >= offers.min(2) {
        return false;
    }
    let states = &mut message.shown.states;
    for (k, id) in FRAMES.into_iter().enumerate().take(offers) {
        states.entry(id).or_default().hidden = k != n;
    }
    states.entry(OK).or_default().disabled = false;
    true
}

/// The offer chosen in the open dialog `message` with `offers` offers (its +0x68): the first whose
/// frame shows (one offer's at once); none while none is.
#[must_use]
pub fn chosen(message: &MessageBox, offers: usize) -> Option<usize> {
    FRAMES
        .iter()
        .take(offers)
        .position(|&id| !message.shown.state(id).hidden)
}

/// The dialog's frames shown (the H3DlgFrames 2012 and 2013): each its one-pixel outline in the
/// game palette `palette`'s entry 45 ([`message::outline`]), over the dialog's items; none for
/// another box.
#[must_use]
pub fn frame_draws(message: &MessageBox, palette: &str) -> Vec<Draw> {
    let [px, py] = message.dialog.place;
    message
        .dialog
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Area(a) if FRAMES.contains(&a.id) && !message.shown.state(a.id).hidden => Some(a),
            _ => None,
        })
        .flat_map(|a| {
            let at = [px.saturating_add(a.at[0]), py.saturating_add(a.at[1])];
            message::outline(at, a.size, palette)
        })
        .collect()
}

#[cfg(test)]
mod tests;
