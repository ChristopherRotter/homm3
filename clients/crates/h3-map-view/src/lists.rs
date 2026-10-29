//! The adventure screen's hero and town lists (E24 M3 T4a, *provisional*;
//! `docs/epics/E24-design.md` §4.5): what each of their five slots shows of the viewer's kingdom
//! from the lists' first entries on (the dialog's +0x64 and +0x68). A hero's slot (`hero_list_slot`
//! 0x403560): its small portrait by its record's portrait (+0x34; `hero_info` 0x679DD0's +0x30),
//! its movement bar's frame by its points left (`hero_movement_frame` 0x4E5330) and its mana bar's
//! by its mana (`hero_mana_frame` 0x4E5380); an empty slot `hpsxxx.pcx` and both bars at frame 0.
//! A town's slot (`town_list_slot` 0x4037A0): its picture in `itpa.def` by its faction, its fort
//! and whether it was built in today (`town_list_frame` 0x5BD700); an empty slot frame 0. The
//! selections (`lists_selection_frames` 0x4038C0, after every item is drawn): the selected hero's
//! frame `hpsyyy.pcx` over its slot, the selected town's mark (`itpa.def` frame 1) over its
//! picture.

use alloc::string::String;
use alloc::vec::Vec;

use serde::Deserialize;

use crate::live::{Kingdom, KingdomHero, KingdomTown, Selected};

/// The slots of each list (`hero_list_update` 0x4032E0, `town_list_update` 0x403420).
pub const SLOTS: usize = 5;

/// The lists' art and frames (the tables' `[map.lists]`).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ListArt {
    /// A hero's small portrait by its record's portrait (`hero_info` 0x679DD0, 0x5C bytes an
    /// entry, its +0x30; 163 in the SoD executable).
    pub portraits: Vec<String>,
    /// The movement bar's frame by the hero's points left (+0x4D).
    pub movement: Bar,
    /// The mana bar's frame by the hero's mana (+0x18).
    pub mana: Bar,
    /// The town list's frames.
    pub towns: TownFrames,
}

/// A bar's frame by a value (`hero_movement_frame` 0x4E5330, `hero_mana_frame` 0x4E5380): 0 at
/// or below 0; below the first of `top` (value + `round`) / `step`, truncated; from `top`'s
/// first, `from`, one more from each further entry it reaches.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Bar {
    /// The value a frame stands for below `top`'s first.
    pub step: i32,
    /// What the value is rounded up by before the division (the mana bar's 4: any mana shows).
    pub round: i32,
    /// The values from which the last frames are drawn, ascending.
    pub top: Vec<i32>,
    /// The frame from `top`'s first.
    pub from: u32,
}

impl Bar {
    /// The frame for `value`.
    #[must_use]
    pub fn frame(&self, value: i32) -> u32 {
        let reached = self.top.iter().filter(|&&t| value >= t).count();
        if value <= 0 {
            0
        } else if reached == 0 {
            let frame = value.saturating_add(self.round) / self.step.max(1);
            u32::try_from(frame).unwrap_or(0)
        } else {
            let more = u32::try_from(reached - 1).unwrap_or(u32::MAX);
            self.from.saturating_add(more)
        }
    }

    /// The frames it draws: one past the highest.
    #[must_use]
    pub fn frames(&self) -> u32 {
        let more = u32::try_from(self.top.len()).unwrap_or(u32::MAX);
        self.from.saturating_add(more)
    }

    fn check(&self) -> Result<(), String> {
        if self.step < 1 || self.round < 0 || self.round >= self.step {
            return Err(alloc::format!(
                "a step of {} rounded by {}: a step of at least 1, a rounding below it",
                self.step,
                self.round
            ));
        }
        if self.top.is_empty() || self.top.windows(2).any(|w| w[0] >= w[1]) || self.top[0] < 1 {
            return Err("`top` empty, below 1 or not ascending".into());
        }
        // The frame just below the first top value must not pass `from`.
        if self.frame(self.top[0] - 1) > self.from {
            return Err(alloc::format!(
                "the frame below {} passes `from` ({})",
                self.top[0],
                self.from
            ));
        }
        Ok(())
    }
}

/// The town list's frames in `itpa.def` (`town_list_frame` 0x5BD700 with its argument 1): a town
/// with a Fort (in its every-built set +0x158; a Citadel or Castle keeps it) `fort`, else
/// `village`, plus `faction` a faction (+0x4), plus `built` when a building was built in it today
/// (+0x2).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TownFrames {
    /// An empty slot's (0x403822).
    pub empty: u32,
    /// The selected town's mark, drawn over its picture (0x403908).
    pub selected: u32,
    /// The first faction's with a fort.
    pub fort: u32,
    /// The first faction's without one.
    pub village: u32,
    /// The frames between two factions'.
    pub faction: u32,
    /// What a building built today adds.
    pub built: u32,
}

impl TownFrames {
    /// The frame of `town`.
    #[must_use]
    pub fn frame(&self, town: KingdomTown) -> u32 {
        let first = if town.fort { self.fort } else { self.village };
        first
            .saturating_add(self.faction.saturating_mul(u32::from(town.faction)))
            .saturating_add(if town.built { self.built } else { 0 })
    }

    /// The frames the towns of `factions` factions draw: one past the highest.
    #[must_use]
    pub fn frames(&self, factions: u8) -> u32 {
        let towns = factions.checked_sub(1).into_iter().flat_map(|last| {
            [true, false].map(|fort| {
                self.frame(KingdomTown {
                    faction: last,
                    fort,
                    built: true,
                })
            })
        });
        let highest = towns.chain([self.empty, self.selected]).max();
        highest.unwrap_or(0).saturating_add(1)
    }
}

impl ListArt {
    /// Checks the bars' rules and that there are portraits.
    ///
    /// # Errors
    /// What breaks a rule: no portraits, a bar's step below 1 or its rounding not below its step,
    /// its `top` empty, below 1 or not ascending, or a frame below `top` past `from`.
    pub fn check(&self) -> Result<(), String> {
        if self.portraits.is_empty() {
            return Err("no portraits".into());
        }
        self.movement
            .check()
            .map_err(|e| alloc::format!("movement: {e}"))?;
        self.mana.check().map_err(|e| alloc::format!("mana: {e}"))
    }
}

/// What one slot of the hero list shows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeroSlot {
    /// The portrait, the empty slot's picture when none.
    pub picture: Option<String>,
    /// The movement bar's frame.
    pub movement: u32,
    /// The mana bar's frame.
    pub mana: u32,
}

/// The hero list's slots for `kingdom` (`hero_list_slot` 0x403560): the heroes from the list's
/// first (+0x64) on, a slot past them empty (no picture: the layout's `hpsxxx.pcx`; both bars at
/// 0); none for a hero whose portrait the table lacks.
#[must_use]
pub fn hero_slots(art: &ListArt, kingdom: &Kingdom) -> Option<[HeroSlot; SLOTS]> {
    let slot = |i: usize| -> Option<HeroSlot> {
        let hero = kingdom.tops[0]
            .checked_add(i)
            .and_then(|at| kingdom.heroes.get(at));
        Some(match hero {
            Some(KingdomHero {
                portrait,
                points,
                mana,
                ..
            }) => HeroSlot {
                picture: Some(art.portraits.get(usize::from(*portrait))?.clone()),
                movement: art.movement.frame(*points),
                mana: art.mana.frame(*mana),
            },
            None => HeroSlot {
                picture: None,
                movement: 0,
                mana: 0,
            },
        })
    };
    Some([slot(0)?, slot(1)?, slot(2)?, slot(3)?, slot(4)?])
}

/// The town list's frames for `kingdom` (`town_list_slot` 0x4037A0): the towns from the list's
/// first (+0x68) on, a slot past them the empty one's.
#[must_use]
pub fn town_slots(art: &ListArt, kingdom: &Kingdom) -> [u32; SLOTS] {
    core::array::from_fn(|i| {
        kingdom.tops[1]
            .checked_add(i)
            .and_then(|at| kingdom.towns.get(at))
            .map_or(art.towns.empty, |&t| art.towns.frame(t))
    })
}

/// The slot the selection shows in (`lists_selection_frames` 0x4038C0): the selected hero's in
/// the hero list or the selected town's in the town list, if it lies among the slots shown.
#[must_use]
pub fn selected_slot(kingdom: &Kingdom) -> Option<(Selected, usize)> {
    let (at, top) = match kingdom.selected {
        Selected::None => return None,
        Selected::Hero(at) => (at, kingdom.tops[0]),
        Selected::Town(at) => (at, kingdom.tops[1]),
    };
    let slot = at.checked_sub(top).filter(|&s| s < SLOTS)?;
    Some((kingdom.selected, slot))
}

#[cfg(test)]
pub(crate) mod tests;
