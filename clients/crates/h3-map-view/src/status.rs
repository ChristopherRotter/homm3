//! The adventure screen's status window (E24 M3 T5b1, *provisional*; `docs/epics/E24-design.md`
//! §4.8, §4.9): what it shows of the live part's [`Status`], a hero's (`AdStatHr.pcx`, built by
//! 0x451AB0), a town's (`AdStatCs.pcx`, 0x4521F0), since T5b2 the date's (`AdStatOt.pcx`,
//! 0x450DD0) or the kingdom's (`AdStatin.pcx`, 0x452B80), as the layout's `status_hero`,
//! `status_town`, `status_date` and `status_kingdom` panels at the dialog's area 2
//! (`status_window_choose` 0x415DE0 chooses the kind: a kind asked for a time, else the selected
//! hero's, else the selected town's, else the date's; when each shows over time is
//! [`crate::timeline`]'s). The other player's turn and the messages are M4's.
//!
//! The date's: the day's picture (`NewDay.def`) and "Day n", or on a week's first day but the
//! game's first the week's (`NewWeek1.def` to `NewWeek4.def` by the week of the month) and "Week
//! n" (the executable's "%s %d" with `GENRLTXT`'s words), the picture's frame the window's step.
//! The kingdom's: the viewer's towns counted by their halls under the four halls' pictures (a count
//! only where there are towns), "Allies:" and "Enemies:" (the executable's "%s:") and the flags of
//! the players in the game, the viewer's team's on the first row, the others' on the second.
//!
//! A hero's: its large portrait, its name, its four primary skills (each cut to 99 at the top and
//! to 0, power and knowledge to 1, at the bottom: 0x451CE4), its spell points, its morale and luck
//! (`imrl22.def`, `ilck22.def`: the bonus + 3) and its army's filled slots (`cprsmall.def`'s
//! frame the creature + 2 over the count, from 10 000 in thousands, truncated: "%dk"). A town's:
//! its picture (`itpt.def`: `town_list_frame` 0x5BD700 without the list's 2), its name, its hall
//! (`itmtls.def`: 1 to 3 for the Town Hall, City Hall and Capitol standing, tested in that order,
//! else 0) and fort (`itmcls.def`: 0 to 2 for the Fort, Citadel and Castle, else 3), the garrison
//! hero's mark (`townqkgh.pcx`), its silo's one or two resources (`smalres.def`; three or more
//! show none), its gold income (its low 16 bits, signed: MCL-Q16) and the army it shows, as a
//! hero's but its filled slots one after another from the first place (a hero's each at its
//! slot's).

use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use h3_ui::text::{Args, Catalogue};
use h3_ui::ui::{Dialog, Shown, State};
use serde::Deserialize;

use crate::layout::AdventureLayout;
use crate::lists::ListArt;
use crate::live::{
    Date, DateStatus, HeroStatus, KingdomStatus, Live, Selected, Stack, Status, TownStatus,
};

/// The status window's control ids (ours, design §4.8; the original numbers its items from 2000
/// in the order it adds them, the army's a picture and its count per filled slot).
pub mod id {
    /// The window's picture.
    pub const PICTURE: u16 = 2000;
    /// The hero's portrait or the town's picture.
    pub const PORTRAIT: u16 = 2001;
    /// The name.
    pub const NAME: u16 = 2002;
    /// A hero's attack, the first of its four primary skills.
    pub const SKILLS: u16 = 2003;
    /// A town's hall.
    pub const HALL: u16 = 2003;
    /// A town's fort.
    pub const FORT: u16 = 2004;
    /// A town's gold income.
    pub const INCOME: u16 = 2006;
    /// A town's silo: the first of two resources.
    pub const SILO: u16 = 2007;
    /// A town's silo: the second of two.
    pub const SILO_SECOND: u16 = 2018;
    /// A town's silo: its only resource.
    pub const SILO_ONLY: u16 = 2019;
    /// A hero's morale.
    pub const MORALE: u16 = 2008;
    /// A town's garrison hero's mark.
    pub const GARRISON: u16 = 2008;
    /// A hero's luck.
    pub const LUCK: u16 = 2009;
    /// A hero's spell points.
    pub const MANA: u16 = 2010;
    /// The army's pictures, the first of seven (the slots in order).
    pub const STACKS: u16 = 2011;
    /// The army's counts, the first of seven.
    pub const COUNTS: u16 = 2021;
    /// The date's picture (E24 M3 T5b2).
    pub const DATE_PICTURE: u16 = 2001;
    /// The date's text.
    pub const DATE_TEXT: u16 = 2002;
    /// The kingdom's halls, the first of four (the villages, Town Halls, City Halls, Capitols).
    pub const HALLS: u16 = 2001;
    /// The kingdom's counts of towns by hall, the first of four (the original's have no id).
    pub const HALL_COUNTS: u16 = 2021;
    /// The kingdom's label of the viewer's team (no id in the original).
    pub const ALLIES: u16 = 2031;
    /// The kingdom's label of the others.
    pub const ENEMIES: u16 = 2032;
    /// The flags of the viewer's team, the first of eight places.
    pub const ALLY_FLAGS: u16 = 2041;
    /// The flags of the others, the first of eight places.
    pub const ENEMY_FLAGS: u16 = 2051;
}

/// The kingdom's halls.
pub const HALLS: u16 = 4;
/// The places of a row of flags.
pub const FLAGS: u16 = 8;

/// The army's slots.
pub const SLOTS: u16 = 7;

/// The string catalogue's keys the window shows numbers by (`clients/text/keys.toml`), since T5b2
/// the date (with the panels' `word.day` and `word.week`, [`crate::panels::KEYS`]) and the
/// kingdom's labels with their words; the names are the `hero` and `town-name` families'
/// ([`hero_key`], [`town_key`]).
pub const KEYS: [&str; 6] = [
    "map.status.number",
    "map.status.thousands",
    "map.status.date",
    "map.status.side",
    "word.allies",
    "word.enemies",
];

/// The key of the hero type `hero`'s name (the `hero` family over `HOTRAITS.TXT`, keyed by the
/// type in the map viewer: *provisional*, the battle client keys its asset tables' heroes).
#[must_use]
pub fn hero_key(hero: u8) -> String {
    format!("hero.{hero}")
}

/// The key of `TownName.txt`'s record `name` (the `town-name` family).
#[must_use]
pub fn town_key(name: u16) -> String {
    format!("town-name.{name}")
}

/// The window's art and rules (the tables' `[map.status]`).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StatusArt {
    /// A hero's large portrait by its record's portrait (`hero_info` 0x679DD0's +0x34).
    pub portraits: Vec<String>,
    /// What the town list's frame of a town adds to the window's (`town_list_frame` 0x5BD700's
    /// argument: 2).
    pub town_list: u32,
    /// What a creature's number adds to give its frame in `cprsmall.def` (2).
    pub stack: u32,
    /// What a morale or luck bonus adds to give its frame (3).
    pub bonus: i32,
    /// A primary skill's highest shown and the lowest of each (99; 0, 0, 1, 1).
    pub skills: Skills,
    /// The count from which an army's slot shows thousands (10 000).
    pub thousands: i32,
    /// The halls whose standing gives the hall's frames 1 on, in the order tested (the Town Hall
    /// 11, City Hall 12, Capitol 13); none of them 0.
    pub halls: Vec<u32>,
    /// The forts whose standing gives the fort's frames 0 on, in the order tested (the Fort 7,
    /// Citadel 8, Castle 9); none of them the next.
    pub forts: Vec<u32>,
    /// The date's window (E24 M3 T5b2): its pictures and their timing.
    pub date: DateArt,
    /// How long a click on the window asks the kingdom's or the date's window for, ms (3 000:
    /// `adventure_command` 0x40A030; E24 M3 T6b2, design §4.14).
    pub asked: u32,
}

/// The date's window's pictures and timing (the tables' `[map.status.date]`; E24 M3 T5b2, design
/// §4.9).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DateArt {
    /// The day's picture (`NewDay.def`).
    pub day: String,
    /// The weeks' pictures by the week of the month, 1 to 4 (the table 0x660BA0).
    pub weeks: Vec<String>,
    /// The frames of each picture's group 0: the day's, then the weeks'.
    pub frames: Vec<u32>,
    /// The least time between two frames' steps, ms (the panel's +0x40).
    pub step: u32,
    /// The sound the day's window plays when built (`newday.wav`; the week's plays none).
    pub sound: String,
}

/// What the date's window shows for a date (0x450EB4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DateShows<'a> {
    /// Its picture.
    pub def: &'a str,
    /// The picture's frames.
    pub frames: u32,
    /// The week's (its text the week's, no sound), not the day's.
    pub week: bool,
}

impl DateArt {
    /// What the window shows for `date`: on a week's first day but the game's first the week's
    /// picture, else the day's (a week past the table's the day's too).
    #[must_use]
    pub fn shows(&self, date: Date) -> DateShows<'_> {
        let first = date.week == 1 && date.month == 1;
        let week = usize::from(date.week)
            .checked_sub(1)
            .filter(|_| date.day == 1 && !first)
            .and_then(|i| self.weeks.get(i).map(|def| (i, def)));
        let frames = |i: usize| self.frames.get(i).copied().unwrap_or(1);
        match week {
            Some((i, def)) => DateShows {
                def,
                frames: frames(i.saturating_add(1)),
                week: true,
            },
            None => DateShows {
                def: &self.day,
                frames: frames(0),
                week: false,
            },
        }
    }
}

/// The primary skills' bounds as shown (0x451CE4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Skills {
    /// The highest shown.
    pub top: i8,
    /// The lowest shown of each.
    pub bottom: [i8; 4],
}

impl StatusArt {
    /// Checks that there are portraits and that a skill's lowest lies below its highest.
    ///
    /// # Errors
    /// What breaks a rule.
    pub fn check(&self) -> Result<(), String> {
        if self.portraits.is_empty() {
            return Err("no portraits".into());
        }
        if self.skills.bottom.iter().any(|&b| b > self.skills.top) {
            return Err("a skill's lowest above its highest".into());
        }
        if self.thousands < 1 {
            return Err("thousands below 1".into());
        }
        let date = &self.date;
        if date.frames.len() != date.weeks.len().saturating_add(1) || date.frames.contains(&0) {
            return Err("the date's pictures and their frames do not match".into());
        }
        Ok(())
    }

    /// The place in `tested` of the first building standing in `buildings`, if one stands.
    fn standing(tested: &[u32], buildings: u64) -> Option<u32> {
        tested
            .iter()
            .position(|&b| b < 64 && buildings >> b & 1 == 1)
            .and_then(|i| u32::try_from(i).ok())
    }

    /// The hall's frame for `buildings` standing: the first hall's place + 1, else 0.
    #[must_use]
    pub fn hall(&self, buildings: u64) -> u32 {
        Self::standing(&self.halls, buildings).map_or(0, |i| i.saturating_add(1))
    }

    /// The fort's frame for `buildings` standing: the first fort's place, else the forts'
    /// number.
    #[must_use]
    pub fn fort(&self, buildings: u64) -> u32 {
        Self::standing(&self.forts, buildings)
            .unwrap_or_else(|| u32::try_from(self.forts.len()).unwrap_or(u32::MAX))
    }
}

/// The status window drawn for `live` (design §4.8): its panel (the layout's `status_hero` or
/// `status_town`) and what its items show, the texts in `texts`, a hero's name of its own in
/// `names` (the map's, [`crate::art::MapArt::hero_names`]; one the map lacks shows its type's).
/// None when it shows nothing yet (no status, another kind, a hero's or a town's window without
/// that selection); an error for a portrait past the tables'.
///
/// # Errors
/// A hero's portrait the tables lack.
pub fn window<'l>(
    layout: &'l AdventureLayout,
    art: &StatusArt,
    lists: &ListArt,
    live: &Live,
    names: &BTreeMap<u8, String>,
    texts: &Catalogue,
) -> Result<Option<(&'l Dialog, Shown)>, String> {
    let kingdom = &live.kingdom;
    Ok(match (&kingdom.status, kingdom.selected) {
        (Some(Status::Date(status)), _) => Some((
            &layout.status_date,
            date_window(art, *status, kingdom.date, texts),
        )),
        (Some(Status::Kingdom(status)), _) => {
            Some((&layout.status_kingdom, kingdom_window(status, texts)))
        }
        (Some(Status::Hero(status)), Selected::Hero(at)) => match kingdom.heroes.get(at) {
            Some(hero) => {
                let portrait = art
                    .portraits
                    .get(usize::from(hero.portrait))
                    .ok_or("a hero's portrait is past the tables'")?;
                let name = match names.get(&status.hero) {
                    Some(own) if status.own_name => own.clone(),
                    _ => texts.text(&hero_key(status.hero), &Args::new()),
                };
                let shown = hero_window(art, status, portrait, hero.mana, name, texts);
                Some((&layout.status_hero, shown))
            }
            None => None,
        },
        (Some(Status::Town(status)), Selected::Town(at)) => match kingdom.towns.get(at) {
            Some(&town) => {
                let picture = lists.towns.frame(town).saturating_sub(art.town_list);
                let shown = town_window(art, status, picture, texts);
                Some((&layout.status_town, shown))
            }
            None => None,
        },
        _ => None,
    })
}

/// The date's window (0x450DD0): its picture and frame, and the day's or the week's text
/// ("%s %d", 0x660C98, with `GENRLTXT`'s word).
fn date_window(art: &StatusArt, status: DateStatus, date: Date, texts: &Catalogue) -> Shown {
    let shows = art.date.shows(date);
    let (word, number) = if shows.week {
        ("word.week", date.week)
    } else {
        ("word.day", date.day)
    };
    let args = Args::new()
        .text("word", &texts.text(word, &Args::new()))
        .number("number", i64::from(number));
    let mut shown = Shown::default();
    shown.defs.insert(id::DATE_PICTURE, shows.def.into());
    shown.frames.insert(id::DATE_PICTURE, status.frame);
    shown
        .texts
        .insert(id::DATE_TEXT, texts.text("map.status.date", &args));
    shown
}

/// The kingdom's window (0x452B80): the halls' frames, the counts of towns where there are any,
/// the labels ("%s:", 0x660D28) and each row's flags from its first place, the places left over
/// hidden.
fn kingdom_window(status: &KingdomStatus, texts: &Catalogue) -> Shown {
    let hidden = State {
        hidden: true,
        ..State::default()
    };
    let mut shown = Shown::default();
    for ((hall, &count), frame) in (0..HALLS).zip(&status.halls).zip(0u32..) {
        shown.frames.insert(id::HALLS + hall, frame);
        let count_id = id::HALL_COUNTS + hall;
        if count > 0 {
            shown
                .texts
                .insert(count_id, number(texts, i64::from(count)));
        } else {
            shown.states.insert(count_id, hidden);
        }
    }
    for (label, word) in [(id::ALLIES, "word.allies"), (id::ENEMIES, "word.enemies")] {
        let args = Args::new().text("word", &texts.text(word, &Args::new()));
        shown
            .texts
            .insert(label, texts.text("map.status.side", &args));
    }
    for (first, colours) in [
        (id::ALLY_FLAGS, &status.allies),
        (id::ENEMY_FLAGS, &status.enemies),
    ] {
        for place in 0..FLAGS {
            match colours.get(usize::from(place)) {
                Some(&colour) => {
                    shown.frames.insert(first + place, u32::from(colour));
                }
                None => {
                    shown.states.insert(first + place, hidden);
                }
            }
        }
    }
    shown
}

/// A number as the window writes it (`%d`, 0x660A1C).
fn number(texts: &Catalogue, value: i64) -> String {
    texts.text("map.status.number", &Args::new().number("number", value))
}

/// A hero's window (0x451AB0).
fn hero_window(
    art: &StatusArt,
    status: &HeroStatus,
    portrait: &str,
    mana: i32,
    name: String,
    texts: &Catalogue,
) -> Shown {
    let mut shown = Shown::default();
    shown.pictures.insert(id::PORTRAIT, portrait.into());
    shown.texts.insert(id::NAME, name);
    for ((id, &skill), &bottom) in (id::SKILLS..).zip(&status.skills).zip(&art.skills.bottom) {
        let shown_skill = skill.min(art.skills.top).max(bottom);
        shown
            .texts
            .insert(id, number(texts, i64::from(shown_skill)));
    }
    shown.texts.insert(id::MANA, number(texts, i64::from(mana)));
    for (id, bonus) in [(id::MORALE, status.morale), (id::LUCK, status.luck)] {
        let frame = i32::from(bonus).saturating_add(art.bonus);
        shown
            .frames
            .insert(id, u32::try_from(frame).unwrap_or_default());
    }
    army(art, &status.army, Places::BySlot, texts, &mut shown);
    shown
}

/// A town's window (0x4521F0).
fn town_window(art: &StatusArt, status: &TownStatus, picture: u32, texts: &Catalogue) -> Shown {
    let mut shown = Shown::default();
    shown.frames.insert(id::PORTRAIT, picture);
    shown
        .texts
        .insert(id::NAME, texts.text(&town_key(status.name), &Args::new()));
    shown.frames.insert(id::HALL, art.hall(status.buildings));
    shown.frames.insert(id::FORT, art.fort(status.buildings));
    let hidden = State {
        hidden: true,
        ..State::default()
    };
    if !status.garrison {
        shown.states.insert(id::GARRISON, hidden);
    }
    let silo: &[u8] = &status.silo;
    let silo_items: &[u16] = match silo.len() {
        2 => &[id::SILO, id::SILO_SECOND],
        1 => &[id::SILO_ONLY],
        _ => &[],
    };
    for id in [id::SILO, id::SILO_SECOND, id::SILO_ONLY] {
        match silo_items.iter().position(|&i| i == id) {
            Some(at) => {
                let resource = silo.get(at).copied().unwrap_or_default();
                shown.frames.insert(id, u32::from(resource));
            }
            None => {
                shown.states.insert(id, hidden);
            }
        }
    }
    // The income's low 16 bits as a signed number (0x45279F: `movsx edx, ax`; MCL-Q16).
    let [low, high, ..] = status.income.to_le_bytes();
    let income = i16::from_le_bytes([low, high]);
    shown
        .texts
        .insert(id::INCOME, number(texts, i64::from(income)));
    army(art, &status.army, Places::Packed, texts, &mut shown);
    shown
}

/// How a window places an army's filled slots.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Places {
    /// Each at its slot's place (a hero's: 0x4520E9 steps the places with every slot).
    BySlot,
    /// One after another from the first place (a town's: 0x452A38 steps them with a filled slot
    /// only).
    Packed,
}

/// The army's slots (0x451EBB, 0x45280C): each filled slot's creature and count at its place
/// (`places`), the places left over hidden.
fn army(
    art: &StatusArt,
    army: &[Option<Stack>; 7],
    places: Places,
    texts: &Catalogue,
    shown: &mut Shown,
) {
    let hidden = State {
        hidden: true,
        ..State::default()
    };
    let filled: Vec<(u16, Stack)> = (0u16..)
        .zip(army)
        .filter_map(|(slot, stack)| stack.map(|s| (slot, s)))
        .enumerate()
        .map(|(n, (slot, stack))| match places {
            Places::BySlot => (slot, stack),
            Places::Packed => (u16::try_from(n).unwrap_or(u16::MAX), stack),
        })
        .collect();
    for place in 0..SLOTS {
        let (stack_id, count_id) = (id::STACKS + place, id::COUNTS + place);
        match filled.iter().find(|(p, _)| *p == place) {
            Some((_, stack)) => {
                let frame = u32::from(stack.creature).saturating_add(art.stack);
                shown.frames.insert(stack_id, frame);
                let text = if stack.count < art.thousands {
                    number(texts, i64::from(stack.count))
                } else {
                    let thousands = i64::from(stack.count / 1000);
                    texts.text(
                        "map.status.thousands",
                        &Args::new().number("thousands", thousands),
                    )
                };
                shown.texts.insert(count_id, text);
            }
            None => {
                shown.states.insert(stack_id, hidden);
                shown.states.insert(count_id, hidden);
            }
        }
    }
}

#[cfg(test)]
mod tests;
