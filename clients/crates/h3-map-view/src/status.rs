//! The adventure screen's status window (E24 M3 T5b1, *provisional*; `docs/epics/E24-design.md`
//! §4.8): what it shows of the live part's [`Status`], a hero's (`AdStatHr.pcx`, built by
//! 0x451AB0) or a town's (`AdStatCs.pcx`, 0x4521F0), as the layout's `status_hero` and
//! `status_town` panels at the dialog's area 2 (`status_window_choose` 0x415DE0 chooses the kind:
//! the selected hero's, else the selected town's). The date's and the kingdom's windows and when
//! each shows are T5b2's; the other player's turn and the messages M4's.
//!
//! A hero's: its large portrait, its name, its four primary skills (each cut to 99 at the top and
//! to 0, power and knowledge to 1, at the bottom: 0x451CE4), its spell points, its morale and luck
//! (`imrl22.def`, `ilck22.def`: the bonus + 3) and its army's filled slots (`cprsmall.def`'s
//! frame the creature + 2 over the count, from 10 000 in thousands, truncated: "%dk"). A town's:
//! its picture (`itpt.def`: `town_list_frame` 0x5BD700 without the list's 2), its name, its hall
//! (`itmtls.def`: 1 to 3 for the Town Hall, City Hall and Capitol standing, tested in that order,
//! else 0) and fort (`itmcls.def`: 0 to 2 for the Fort, Citadel and Castle, else 3), the garrison
//! hero's mark (`townqkgh.pcx`), its silo's one or two resources (`smalres.def`; three or more
//! show none), its gold income and the army it shows, as a hero's but its filled slots one after
//! another from the first place (a hero's each at its slot's).

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use h3_ui::text::{Args, Catalogue};
use h3_ui::ui::{Dialog, Shown, State};
use serde::Deserialize;

use crate::layout::AdventureLayout;
use crate::lists::ListArt;
use crate::live::{HeroStatus, Live, Selected, Stack, Status, TownStatus};

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
    /// A hero's morale; a town's garrison hero's mark.
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
}

/// The army's slots.
pub const SLOTS: u16 = 7;

/// The string catalogue's keys the window shows numbers by (`clients/text/keys.toml`); the names
/// are the `hero` and `town-name` families' ([`hero_key`], [`town_key`]).
pub const KEYS: [&str; 2] = ["map.status.number", "map.status.thousands"];

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
/// `status_town`) and what its items show, the texts in `texts`. None when it shows nothing yet
/// (no status, another kind, a hero's or a town's window without that selection); an error for a
/// portrait past the tables'.
///
/// # Errors
/// A hero's portrait the tables lack.
pub fn window<'l>(
    layout: &'l AdventureLayout,
    art: &StatusArt,
    lists: &ListArt,
    live: &Live,
    texts: &Catalogue,
) -> Result<Option<(&'l Dialog, Shown)>, String> {
    let kingdom = &live.kingdom;
    Ok(match (&kingdom.status, kingdom.selected) {
        (Some(Status::Hero(status)), Selected::Hero(at)) => match kingdom.heroes.get(at) {
            Some(hero) => {
                let portrait = art
                    .portraits
                    .get(usize::from(hero.portrait))
                    .ok_or("a hero's portrait is past the tables'")?;
                let shown = hero_window(art, status, portrait, hero.mana, texts);
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
    texts: &Catalogue,
) -> Shown {
    let mut shown = Shown::default();
    shown.pictures.insert(id::PORTRAIT, portrait.into());
    shown
        .texts
        .insert(id::NAME, texts.text(&hero_key(status.hero), &Args::new()));
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
    shown
        .texts
        .insert(id::INCOME, number(texts, i64::from(status.income)));
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
