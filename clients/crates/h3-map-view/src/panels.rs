//! What the adventure screen's dialog and resource bar show (E24 M3 T2a, *provisional*;
//! `docs/epics/E24-design.md` §4.1): the buttons' states at the human player's turn from the
//! live part's kingdom and buttons, the map's levels and the view's level; the buttons 3 to 12 and
//! the resource bar's picture in the viewer's colours; the lists' contents and their selections
//! (T4a, [`crate::lists`]); the resource bar's amounts and the date (T5a, design §4.7). The status
//! window (T5b) and the hints (T6) come later; the minimap is [`crate::minimap`]'s (T3a).

use alloc::collections::BTreeMap;
use alloc::string::ToString;
use alloc::vec::Vec;

use h3_ui::text::{Args, Catalogue};
use h3_ui::ui::{Draw, Item, Shown, State};

use crate::layout::AdventureLayout;
use crate::lists::{self, ListArt, SLOTS};
use crate::live::{Buttons, Live, Selected};

/// The dialog's control ids the panels' logic sets (design §4.1).
pub mod id {
    /// The minimap's area.
    pub const MINIMAP: u16 = 1;
    /// The underground.
    pub const UNDERGROUND: u16 = 4;
    /// The quest log.
    pub const QUESTS: u16 = 5;
    /// Sleep.
    pub const SLEEP: u16 = 6;
    /// Move hero.
    pub const MOVE_HERO: u16 = 7;
    /// Cast a spell.
    pub const CAST: u16 = 8;
    /// Next hero.
    pub const NEXT_HERO: u16 = 11;
    /// The hero list's arrows, up and down.
    pub const HEROES: [u16; 2] = [13, 14];
    /// The town list's arrows, up and down.
    pub const TOWNS: [u16; 2] = [30, 31];
    /// The hero list's portraits, the first of five (E24 M3 T4a).
    pub const PORTRAITS: u16 = 15;
    /// The hero list's movement bars, the first of five.
    pub const MOVEMENT: u16 = 20;
    /// The hero list's mana bars, the first of five.
    pub const MANA: u16 = 25;
    /// The town list's pictures, the first of five.
    pub const TOWN_PICTURES: u16 = 32;
    /// The hero list's selection frames, the first of five.
    pub const FRAMES: u16 = 39;
    /// The resource bar's amounts, the first of seven (E24 M3 T5a).
    pub const AMOUNTS: u16 = 1001;
    /// The resource bar's date.
    pub const DATE: u16 = 1008;
}

/// What the dialog's items show for `live` on a map of `levels` levels with the view on level
/// `z` (design §4.1): the underground disabled on one level and its DEF by `z`; the quest log
/// disabled without a quest; sleep and cast a spell disabled without a hero selected, sleep's
/// DEF the asleep one while it sleeps; move hero and next hero as the live part's buttons (or as
/// [`Buttons::of`] gives them); each list's up arrow disabled at its top, its down arrow when no
/// more than its slots lie below the top; the buttons the layout names in the viewer's colours.
/// The lists' slots by [`lists`] (design §4.5): each hero's portrait and its bars' frames, each
/// town's picture, an empty slot the layout's picture and frame 0; the selection frames hidden
/// ([`selections`] draws the selected hero's after the items, as 0x4038C0 does). None when a
/// hero's portrait is past the tables'.
#[must_use]
pub fn dialog(
    layout: &AdventureLayout,
    lists: &ListArt,
    live: &Live,
    levels: usize,
    z: u8,
) -> Option<Shown> {
    let kingdom = &live.kingdom;
    let buttons = live
        .buttons
        .unwrap_or_else(|| Buttons::of(kingdom, &live.path));
    let hero = match kingdom.selected {
        Selected::Hero(i) => Some(kingdom.heroes.get(i).copied().unwrap_or_default()),
        Selected::None | Selected::Town(_) => None,
    };
    let [heroes_top, towns_top] = kingdom.tops;
    let disabled = [
        (id::UNDERGROUND, levels < 2),
        (id::QUESTS, !kingdom.quests),
        (id::SLEEP, hero.is_none()),
        (id::MOVE_HERO, !(hero.is_some() && buttons.move_hero)),
        (id::CAST, hero.is_none()),
        (id::NEXT_HERO, !buttons.next_hero),
        (id::HEROES[0], heroes_top == 0),
        (
            id::HEROES[1],
            kingdom.heroes.len() <= heroes_top.saturating_add(SLOTS),
        ),
        (id::TOWNS[0], towns_top == 0),
        (
            id::TOWNS[1],
            kingdom.towns.len() <= towns_top.saturating_add(SLOTS),
        ),
    ];
    let hidden = State {
        hidden: true,
        ..State::default()
    };
    let states = disabled
        .into_iter()
        .filter(|&(_, off)| off)
        .map(|(id, _)| {
            let state = State {
                disabled: true,
                ..State::default()
            };
            (id, state)
        })
        .chain((id::FRAMES..).take(SLOTS).map(|id| (id, hidden)))
        .collect();
    let heroes = lists::hero_slots(lists, kingdom)?;
    let towns = lists::town_slots(lists, kingdom);
    let mut frames = BTreeMap::new();
    let mut pictures = BTreeMap::new();
    for ((slot, hero), town) in (0u16..).zip(&heroes).zip(towns) {
        frames.insert(id::MOVEMENT + slot, hero.movement);
        frames.insert(id::MANA + slot, hero.mana);
        frames.insert(id::TOWN_PICTURES + slot, town);
        if let Some(portrait) = &hero.picture {
            pictures.insert(id::PORTRAITS + slot, portrait.clone());
        }
    }
    let panels = &layout.panels;
    let underground = &panels.underground[usize::from(z.min(1))];
    let sleep = &panels.sleep[usize::from(hero.is_some_and(|h| h.asleep))];
    let defs = BTreeMap::from([
        (id::UNDERGROUND, underground.clone()),
        (id::SLEEP, sleep.clone()),
    ]);
    Some(Shown {
        states,
        players: panels
            .coloured
            .iter()
            .map(|&id| (id, live.viewer))
            .collect(),
        texts: BTreeMap::new(),
        defs,
        frames,
        pictures,
    })
}

/// The lists' selections drawn after the dialog's items (`lists_selection_frames` 0x4038C0,
/// design §4.5): the selected town's mark (`itpa.def`'s frame 1) over its slot's picture, or the
/// selected hero's frame (`hpsyyy.pcx`) over its slot, when the selection lies among the slots
/// shown; nothing else. The original leaves the town's item at the mark's frame: drawn again by
/// itself, it would show the mark over what lies under it (a redraw artifact, not copied).
#[must_use]
pub fn selections(layout: &AdventureLayout, lists: &ListArt, live: &Live) -> Vec<Draw> {
    let Some((selected, slot)) = lists::selected_slot(&live.kingdom) else {
        return Vec::new();
    };
    let slot = u16::try_from(slot).unwrap_or(u16::MAX);
    let [x, y] = layout.dialog.place;
    let first = match selected {
        Selected::Town(_) => id::TOWN_PICTURES,
        Selected::None | Selected::Hero(_) => id::FRAMES,
    };
    match layout.dialog.item(first.saturating_add(slot)) {
        Some(Item::Picture(p)) => Vec::from([Draw::Picture {
            picture: p.picture.clone(),
            x: x.saturating_add(p.at[0]),
            y: y.saturating_add(p.at[1]),
            player: None,
            size: p.size,
        }]),
        Some(Item::Def(d)) => d
            .def
            .iter()
            .map(|def| Draw::Button {
                def: def.to_string(),
                frame: lists.towns.selected,
                x: x.saturating_add(d.at[0]),
                y: y.saturating_add(d.at[1]),
                mirrored: false,
                player: None,
            })
            .collect(),
        _ => Vec::new(),
    }
}

/// The string catalogue's keys the panels show texts by (E24 M3 T5a; `clients/text/keys.toml`).
pub const KEYS: [&str; 5] = [
    "map.resources.amount",
    "map.resources.date",
    "word.month",
    "word.week",
    "word.day",
];

/// What the resource bar shows (design §4.1, §4.7): its picture in the viewer's colours
/// (0x558C99); each of the kingdom's seven amounts by the key `map.resources.amount` (0x558F55)
/// and the date by `map.resources.date` with the words `word.month`, `word.week` and `word.day`
/// (0x558F8B), in `texts`.
#[must_use]
pub fn resource_bar(layout: &AdventureLayout, live: &Live, texts: &Catalogue) -> Shown {
    let kingdom = &live.kingdom;
    let mut shown_texts: BTreeMap<u16, _> = (id::AMOUNTS..)
        .zip(kingdom.resources)
        .map(|(id, amount)| {
            let args = Args::new().number("amount", i64::from(amount));
            (id, texts.text("map.resources.amount", &args))
        })
        .collect();
    let date = kingdom.date;
    let word = |key: &str| texts.text(key, &Args::new());
    let args = Args::new()
        .text("month_word", &word("word.month"))
        .number("month", i64::from(date.month))
        .text("week_word", &word("word.week"))
        .number("week", i64::from(date.week))
        .text("day_word", &word("word.day"))
        .number("day", i64::from(date.day));
    shown_texts.insert(id::DATE, texts.text("map.resources.date", &args));
    Shown {
        players: layout
            .resource_bar
            .items
            .iter()
            .filter(|i| matches!(i, Item::Picture(_)))
            .map(|i| (i.id(), live.viewer))
            .collect(),
        texts: shown_texts,
        ..Shown::default()
    }
}

#[cfg(test)]
mod tests;
