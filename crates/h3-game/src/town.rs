//! A player's start as the game's setup gives it (E09 T7a1, design note `E09-design.md` §13): the
//! factions it may start with, its choice among them, its starting hero's pick, its main town and
//! whether its starting hero is random. A map's import fills it (`h3-map`, TWN-SET-001); whoever
//! seats the players may change the choice and the pick before the game is built (E09 decision
//! 6's seam). The game's build reads none of it yet but the refusal of a setup that needs a hero
//! drawn ([`HeroDraw`], `TWN-STUB-3`): the draws are E09 T7b's (design note §14).

use alloc::vec::Vec;
use core::fmt;

use h3_content::{FactionId, HeroTypeId};
use h3_contracts::{HeroId, PlayerId, TownId};

use crate::{GameSetup, GameSetupError};

/// A set of the core's factions, a bit per faction below [`FactionSet::CAPACITY`] (the map's
/// allowed towns, read through the ruleset's factions).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct FactionSet(u64);

impl FactionSet {
    /// No faction.
    pub const EMPTY: Self = Self(0);

    /// The factions a set can hold: those whose index is below it.
    pub const CAPACITY: usize = 64;

    /// The set with `faction` added; a faction from [`FactionSet::CAPACITY`] up is in no set,
    /// the set is unchanged.
    #[must_use]
    pub const fn with(self, faction: FactionId) -> Self {
        if faction.index() < Self::CAPACITY {
            Self(self.0 | 1 << faction.index())
        } else {
            self
        }
    }

    /// Whether the set holds `faction`.
    pub const fn contains(self, faction: FactionId) -> bool {
        faction.index() < Self::CAPACITY && self.0 & 1 << faction.index() != 0
    }

    /// The number of factions in the set.
    pub const fn len(self) -> u32 {
        self.0.count_ones()
    }

    /// Whether the set holds no faction.
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// The set's factions in index order.
    pub fn iter(self) -> impl Iterator<Item = FactionId> {
        (0..Self::CAPACITY)
            .filter(move |&index| self.0 & 1 << index != 0)
            .filter_map(FactionId::from_index)
    }
}

/// A player's choice of its faction before the game is built.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FactionChoice {
    /// Drawn among the player's factions when the game is built (T7b).
    Random,
    /// This faction.
    Fixed(FactionId),
}

/// A player's main town (the map's; survey `E09-towns.md` §1.2 and finding 3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MainTown {
    /// The town.
    pub town: TownId,
    /// Whether the game generates the player's starting hero at it (the map's flag; an RoE map's
    /// always).
    pub generates_hero: bool,
}

/// A player's start (design note §13); the default is none of it, for a setup without towns.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PlayerStart {
    /// The factions it may start with (TWN-SET-001).
    pub factions: FactionSet,
    /// Its faction's choice; `None` without one (no faction to start with).
    pub choice: Option<FactionChoice>,
    /// Its starting hero's pick; `None` for the draw's.
    pub pick: Option<HeroTypeId>,
    /// Its main town, if the map names one.
    pub main_town: Option<MainTown>,
    /// Whether its starting hero is random (the map's flag).
    pub random_hero: bool,
}

/// What a setup needs drawn that the game cannot build yet (`TWN-STUB-3`; the draws: E09 T7b).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeroDraw {
    /// This hero is random: its type is drawn.
    RandomHero(HeroId),
    /// This player's main town generates its starting hero.
    MainTown(PlayerId),
}

impl fmt::Display for HeroDraw {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RandomHero(hero) => write!(f, "hero {} is a random hero", hero.raw()),
            Self::MainTown(player) => {
                write!(f, "player {}'s main town generates a hero", player.index())
            }
        }
    }
}

/// Each hero's type in hero order, the setup's heroes at most a [`HeroId`]'s.
///
/// # Errors
///
/// [`GameSetupError::HeroDraw`]: the first player in id order whose main town generates a hero,
/// else the first random hero.
// stub: TWN-STUB-3
pub(crate) fn hero_types(setup: &GameSetup) -> Result<Vec<HeroTypeId>, GameSetupError> {
    if let Some((_, player)) = setup
        .players
        .iter()
        .zip(0..=u8::MAX)
        .find(|(player, _)| player.start.main_town.is_some_and(|t| t.generates_hero))
    {
        return Err(GameSetupError::HeroDraw(HeroDraw::MainTown(PlayerId::new(
            player,
        ))));
    }
    setup
        .heroes
        .iter()
        .zip(0..=u16::MAX)
        .map(|(hero, index)| {
            hero.hero_type
                .ok_or(GameSetupError::HeroDraw(HeroDraw::RandomHero(HeroId::new(
                    index,
                ))))
        })
        .collect()
}

#[cfg(test)]
mod tests;
