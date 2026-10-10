//! The whole game's setup (ADR-0015 point 4, E08 T6a): plain owned data a game is built and
//! validated from. A map is imported into it (`h3-map`); `h3-game` checks its own part (the
//! players, the heroes' types and starting armies) and hands the adventure context its setup,
//! which the context validates with the checks its commands use.

use alloc::vec::Vec;
use core::fmt;

use h3_adventure::{AdventureSetup, AdventureState, HeroSetup, MapSetup, Pos, SetupError};
use h3_content::{HeroType, HeroTypeId};
use h3_contracts::{ArmyStack, HeroId, PlayerId};

use crate::GameRuleset;

/// A player of the game.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlayerSetup {
    /// The player's colour in the original's order (0 red to 7 pink): what a map's ownership
    /// and conditions name it by. Players act in colour order (sequential play).
    pub colour: u8,
}

/// Where a hero's army comes from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HeroArmy {
    /// The stacks the setup gives.
    Custom(Vec<ArmyStack>),
    /// The hero type's starting army, drawn when the game is built (E08 T6b): a random
    /// placeholder of the setup (ADR-0015 point 4).
    Starting,
}

/// A hero on the map as the game's setup places it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameHeroSetup {
    /// The hero type.
    pub hero_type: HeroTypeId,
    /// The player owning it.
    pub owner: PlayerId,
    /// The tile it stands on.
    pub pos: Pos,
    /// Its army.
    pub army: HeroArmy,
}

/// Everything a game is built from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameSetup {
    /// The players in colour order; a player's [`PlayerId`] is its index here.
    pub players: Vec<PlayerSetup>,
    /// The static map.
    pub map: MapSetup,
    /// The heroes on the map; a hero's [`HeroId`] is its index here (the game's store of heroes).
    pub heroes: Vec<GameHeroSetup>,
}

/// Why a [`GameSetup`] cannot start a game.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameSetupError {
    /// More players than a [`PlayerId`] can name; the number.
    Players(usize),
    /// The players' colours are not strictly increasing or reach past the ruleset's players;
    /// the index of the first offending player.
    Colour(usize),
    /// More heroes than a [`HeroId`] can name; the number.
    Heroes(usize),
    /// The hero's type is not part of the ruleset.
    UnknownHeroType(HeroId),
    /// The hero takes its type's starting army, which has no stacks (the core checks each
    /// stack's counts).
    StartingArmy(HeroId),
    /// The adventure context rejects its part.
    Adventure(SetupError),
}

impl fmt::Display for GameSetupError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Players(n) => write!(f, "{n} players are too many"),
            Self::Colour(player) => write!(f, "player {player}'s colour is out of order"),
            Self::Heroes(n) => write!(f, "{n} heroes are too many"),
            Self::UnknownHeroType(hero) => {
                write!(f, "hero {} has an unknown hero type", hero.raw())
            }
            Self::StartingArmy(hero) => {
                write!(f, "hero {}'s starting army has no stacks", hero.raw())
            }
            Self::Adventure(error) => write!(f, "adventure: {error}"),
        }
    }
}

impl core::error::Error for GameSetupError {}

impl GameSetup {
    /// Checks the setup: the game's part (players in colour order within the ruleset's players,
    /// known hero types, starting armies that can be drawn), then the adventure context's part
    /// with its own checks. A starting army is checked as its hero type's whole starting army at
    /// the lowest counts, in slots from 0: every draw takes stacks from its front at counts
    /// between their bounds (VCMI `CGHeroInstance::initArmy`; the draw's rule comes with T6b),
    /// so each draw is valid exactly when this army is.
    ///
    /// # Errors
    ///
    /// The first [`GameSetupError`] found, in that order.
    pub fn validate(&self, rules: &GameRuleset) -> Result<(), GameSetupError> {
        self.adventure_setup(rules)
            .and_then(|setup| {
                AdventureState::new(rules.core(), rules.adventure(), &setup)
                    .map_err(GameSetupError::Adventure)
            })
            .map(drop)
    }

    /// The adventure context's setup, with each starting army stood in for as
    /// [`Self::validate`] says.
    fn adventure_setup(&self, rules: &GameRuleset) -> Result<AdventureSetup, GameSetupError> {
        let count = self.players.len();
        let players = u8::try_from(count).map_err(|_| GameSetupError::Players(count))?;
        let max = rules.adventure().params().max_players();
        for (index, player) in self.players.iter().enumerate() {
            let after_previous = index
                .checked_sub(1)
                .is_none_or(|previous| self.players[previous].colour < player.colour);
            if !(after_previous && player.colour < max) {
                return Err(GameSetupError::Colour(index));
            }
        }
        if self.heroes.len() > usize::from(u16::MAX) + 1 {
            return Err(GameSetupError::Heroes(self.heroes.len()));
        }
        let heroes = self
            .heroes
            .iter()
            .zip(0..=u16::MAX)
            .map(|(hero, index)| {
                let id = HeroId::new(index);
                let hero_type = rules
                    .core()
                    .hero_type(hero.hero_type)
                    .ok_or(GameSetupError::UnknownHeroType(id))?;
                let army = match &hero.army {
                    HeroArmy::Custom(stacks) => stacks.clone(),
                    HeroArmy::Starting => starting_army(hero_type, id)?,
                };
                Ok(HeroSetup {
                    id,
                    owner: hero.owner,
                    pos: hero.pos,
                    army,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(AdventureSetup {
            players,
            map: self.map.clone(),
            heroes,
        })
    }
}

/// The hero type's whole starting army at its lowest counts, slots from 0.
fn starting_army(hero_type: &HeroType, hero: HeroId) -> Result<Vec<ArmyStack>, GameSetupError> {
    let stacks = &hero_type.army;
    if stacks.is_empty() {
        return Err(GameSetupError::StartingArmy(hero));
    }
    Ok(stacks
        .iter()
        .zip(0..=u8::MAX)
        .map(|(s, slot)| ArmyStack {
            slot,
            creature: s.creature,
            count: u32::from(s.count[0]),
        })
        .collect())
}
