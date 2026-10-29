//! What the adventure screen's dialog and resource bar show (E24 M3 T2a, *provisional*;
//! `docs/epics/E24-design.md` §4.1): the buttons' states at the human player's turn from the
//! live part's kingdom and buttons, the map's levels and the view's level; the buttons 3 to 12 and
//! the resource bar's picture in the viewer's colours. The lists' contents (T4), the status
//! window, the amounts and the date (T5) and the hints (T6) come later; the minimap is
//! [`crate::minimap`]'s (T3a).

use alloc::collections::BTreeMap;

use h3_ui::ui::{Shown, State};

use crate::layout::AdventureLayout;
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
}

/// The slots of each list (`hero_list_update` 0x4032E0, `town_list_update` 0x403420).
pub const SLOTS: usize = 5;

/// What the dialog's items show for `live` on a map of `levels` levels with the view on level
/// `z` (design §4.1): the underground disabled on one level and its DEF by `z`; the quest log
/// disabled without a quest; sleep and cast a spell disabled without a hero selected, sleep's
/// DEF the asleep one while it sleeps; move hero and next hero as the live part's buttons (or as
/// [`Buttons::of`] gives them); each list's up arrow disabled at its top, its down arrow when no
/// more than its slots lie below the top; the buttons the layout names in the viewer's colours.
#[must_use]
pub fn dialog(layout: &AdventureLayout, live: &Live, levels: usize, z: u8) -> Shown {
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
            kingdom.towns <= towns_top.saturating_add(SLOTS),
        ),
    ];
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
        .collect();
    let panels = &layout.panels;
    let underground = &panels.underground[usize::from(z.min(1))];
    let sleep = &panels.sleep[usize::from(hero.is_some_and(|h| h.asleep))];
    let defs = BTreeMap::from([
        (id::UNDERGROUND, underground.clone()),
        (id::SLEEP, sleep.clone()),
    ]);
    Shown {
        states,
        players: panels
            .coloured
            .iter()
            .map(|&id| (id, live.viewer))
            .collect(),
        defs,
        ..Shown::default()
    }
}

/// What the resource bar shows (design §4.1): its picture in the viewer's colours (0x558C99).
#[must_use]
pub fn resource_bar(layout: &AdventureLayout, live: &Live) -> Shown {
    Shown {
        players: layout
            .resource_bar
            .items
            .iter()
            .filter(|i| matches!(i, h3_ui::ui::Item::Picture(_)))
            .map(|i| (i.id(), live.viewer))
            .collect(),
        ..Shown::default()
    }
}

#[cfg(test)]
mod tests;
