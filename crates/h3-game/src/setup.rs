//! The whole game's setup (ADR-0015 point 4, E08 T6a): plain owned data a game is built and
//! validated from. A map is imported into it (`h3-map`); `h3-game` checks its own part (the
//! players, the heroes' types and starting armies) and hands the adventure context its setup,
//! which the context validates with the checks its commands use.

use alloc::vec;
use alloc::vec::Vec;
use core::fmt;

use h3_adventure::{AdventureSetup, HeroSetup, MapSetup, Pos, SetupError};
use h3_content::{HeroType, HeroTypeId, StartingStack};
use h3_contracts::{ArmyStack, HeroId, PlayerId};
use h3_kernel::Rng;

use crate::{GameRuleset, GameState};

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
    /// The hero takes its type's starting army, which has no stacks or more stacks than the
    /// army's slots (the core checks each stack's counts).
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
                write!(
                    f,
                    "hero {}'s starting army does not fit an army",
                    hero.raw()
                )
            }
            Self::Adventure(error) => write!(f, "adventure: {error}"),
        }
    }
}

impl core::error::Error for GameSetupError {}

impl GameSetup {
    /// Checks the setup: the game's part (players in colour order within the ruleset's players,
    /// known hero types, starting armies that can be drawn), then the adventure context's part
    /// with its own checks. It builds the game as [`GameState::new`] does: whether a setup is
    /// valid does not depend on the seed, since every starting army it can draw is valid when its
    /// hero type's army has a first stack and no more stacks than the army's slots (the core
    /// checks each stack's counts).
    ///
    /// # Errors
    ///
    /// The first [`GameSetupError`] found, in that order.
    pub fn validate(&self, rules: &GameRuleset) -> Result<(), GameSetupError> {
        GameState::new(rules, self, 0).map(drop)
    }

    /// The adventure context's setup, with each starting army drawn from `draws` in hero order.
    pub(crate) fn adventure_setup(
        &self,
        rules: &GameRuleset,
        draws: &mut Rng,
    ) -> Result<AdventureSetup, GameSetupError> {
        let count = self.players.len();
        let players = u8::try_from(count).map_err(|_| GameSetupError::Players(count))?;
        let params = rules.adventure().params();
        for (index, player) in self.players.iter().enumerate() {
            let after_previous = index
                .checked_sub(1)
                .is_none_or(|previous| self.players[previous].colour < player.colour);
            if !(after_previous && player.colour < params.max_players()) {
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
                    HeroArmy::Starting => {
                        let first = hero_type
                            .army
                            .split_first()
                            .filter(|_| hero_type.army.len() <= usize::from(params.army_slots()))
                            .ok_or(GameSetupError::StartingArmy(id))?;
                        draw_starting_army(first, hero_type, params.starting_stack_chances(), draws)
                    }
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

/// Draws a hero's starting army from its type's starting slots (GAM-SET-001): the first slot's
/// stack always; each later slot with its chance, a roll of 1 to 100 at most the chance, slots
/// past the chances never. The given stacks take the army's slots from 0 in order, each with a
/// count drawn between its bounds. The original's starting slots are the first stack, the war
/// machines, then the other stacks (its hero table has every war machine in the second of three
/// slots; the packs keep the war machines apart, `HeroType::war_machines`). A war machine's slot
/// is rolled like the others but gives no stack; the war machine itself is not modelled on the
/// map yet (ADV-Q6). Draws in the original's order (0x4C9730): the first count, then per later
/// slot its roll and, when given, its count.
// rule: GAM-SET-001
fn draw_starting_army(
    (first, rest): (&StartingStack, &[StartingStack]),
    hero_type: &HeroType,
    chances: &[u8],
    rng: &mut Rng,
) -> Vec<ArmyStack> {
    let mut army = vec![draw_stack(0, first, rng)];
    let later = hero_type
        .war_machines
        .iter()
        .map(|_| None)
        .chain(rest.iter().map(Some));
    for (slot, &chance) in later.zip(chances) {
        let given = rng.range_inclusive(1, 100) <= u32::from(chance);
        if let Some(stack) = slot.filter(|_| given) {
            let next = u8::try_from(army.len()).unwrap_or(u8::MAX);
            army.push(draw_stack(next, stack, rng));
        }
    }
    army
}

/// One stack of a starting army in army slot `slot`, its count drawn between its bounds.
fn draw_stack(slot: u8, stack: &StartingStack, rng: &mut Rng) -> ArmyStack {
    let [low, high] = stack.count;
    ArmyStack {
        slot,
        creature: stack.creature,
        count: rng.range_inclusive(u32::from(low), u32::from(high)),
    }
}
