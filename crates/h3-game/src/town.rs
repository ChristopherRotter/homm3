//! A player's start as the game's setup gives it (E09 T7a1, design note `E09-design.md` §13): the
//! factions it may start with, its choice among them, its starting hero's pick, its main town and
//! whether its starting hero is random. A map's import fills it (`h3-map`, TWN-SET-001); whoever
//! seats the players may change the choice and the pick before the game is built (E09 decision
//! 6's seam), the pick from the player's offer ([`GameSetup::offer`], TWN-SET-006). The game's
//! build checks the starts and the towns (E09 T7a2: [`check_starts`]) but draws nothing yet: it
//! refuses a setup that needs a hero drawn ([`HeroDraw`], `TWN-STUB-3`; the draws are E09 T7b's,
//! design note §14).

use alloc::vec::Vec;
use core::fmt;

use h3_content::{FactionId, HeroTypeId};
use h3_contracts::{HeroId, PlayerId, TownId};
use h3_towns::TownFaction;

use crate::{GameRuleset, GameSetup, GameSetupError, PlayerSetup};

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

impl GameSetup {
    /// The most hero types a player's offer holds: the start dialog's list of 16
    /// (`scenario_hero_lists` 0x583890; survey `E09-towns.md` §4.3, TWN-SET-006).
    pub const OFFERED_HEROES: usize = 16;

    /// The hero types `player` may pick its starting hero from (TWN-SET-006; survey
    /// `E09-towns.md` §4.3): none unless its choice is a fixed faction and its main town generates
    /// its starting hero or its starting hero is random; then the first
    /// [`GameSetup::OFFERED_HEROES`] in the core's type order that the map allows, that no placed
    /// hero and no prisoner has, whose class is of that faction and that the player's colour may
    /// take (each of the type's [`GameSetup::hero_masks`] has the colour's bit; a type without
    /// one, every colour). None for a player not of the setup. Whoever seats the players chooses
    /// a pick among them before the game is built.
    pub fn offer(&self, rules: &GameRuleset, player: PlayerId) -> Vec<HeroTypeId> {
        let Some(seat) = self.players.get(player.index()) else {
            return Vec::new();
        };
        let start = &seat.start;
        let Some(FactionChoice::Fixed(faction)) = start.choice else {
            return Vec::new();
        };
        if !(start.random_hero || start.main_town.is_some_and(|t| t.generates_hero)) {
            return Vec::new();
        }
        let core = rules.core();
        let colour = u32::from(seat.colour);
        // rule: TWN-SET-006
        core.hero_types()
            .iter()
            .enumerate()
            .filter_map(|(index, ty)| HeroTypeId::from_index(index).map(|id| (id, ty)))
            .filter(|&(id, ty)| {
                self.allowed_heroes.contains(&id)
                    && !self.heroes.iter().any(|hero| hero.hero_type == Some(id))
                    && !self.prisoners.contains(&id)
                    && core
                        .hero_class(ty.class)
                        .is_some_and(|class| class.faction == faction)
                    && self
                        .hero_masks
                        .iter()
                        .filter(|&&(masked, _)| masked == id)
                        .all(|&(_, mask)| mask.checked_shr(colour).is_some_and(|m| m & 1 == 1))
            })
            .map(|(id, _)| id)
            .take(Self::OFFERED_HEROES)
            .collect()
    }
}

/// The setup's checks of the towns and the players' starts (E09 T7a2, design note §13), one pass
/// per invariant (ADR-0015 point 4): a fixed town's faction, a random town's alignment, each
/// player's faction choice, main town, pick and a pick of no earlier player, then a random hero's
/// owner.
///
/// # Errors
///
/// The first [`GameSetupError`] found, in that order: [`GameSetupError::TownFaction`],
/// [`GameSetupError::TownAlignment`], [`GameSetupError::Choice`], [`GameSetupError::MainTown`],
/// [`GameSetupError::Pick`], [`GameSetupError::PickTaken`], [`GameSetupError::RandomHeroOwner`].
pub(crate) fn check_starts(setup: &GameSetup, rules: &GameRuleset) -> Result<(), GameSetupError> {
    let towns = || setup.towns.iter().zip((0..=u16::MAX).map(TownId::new));
    // rule: TWN-SET-002
    if let Some((_, town)) = towns().find(|(town, _)| {
        matches!(town.faction, TownFaction::Fixed(faction) if rules.towns().town(faction).is_none())
    }) {
        return Err(GameSetupError::TownFaction(town));
    }
    // rule: TWN-SET-002
    if let Some((_, town)) = towns().find(|(town, _)| {
        matches!(town.faction, TownFaction::Random { alignment: Some(colour) }
            if !setup.players.iter().any(|player| player.colour == colour))
    }) {
        return Err(GameSetupError::TownAlignment(town));
    }
    // rule: TWN-SET-001
    if let Some(player) = first_player(setup, |id, player| !chooses(setup, rules, id, player)) {
        return Err(GameSetupError::Choice(player));
    }
    // rule: TWN-SET-001
    if let Some(player) = first_player(setup, |_, player| {
        player
            .start
            .main_town
            .is_some_and(|main| main.town.index() >= setup.towns.len())
    }) {
        return Err(GameSetupError::MainTown(player));
    }
    // rule: TWN-SET-006
    if let Some(player) = first_player(setup, |id, player| {
        player
            .start
            .pick
            .is_some_and(|pick| !setup.offer(rules, id).contains(&pick))
    }) {
        return Err(GameSetupError::Pick(player));
    }
    // rule: TWN-SET-006
    if let Some(player) = first_player(setup, |id, player| {
        player.start.pick.is_some_and(|pick| {
            setup.players[..id.index()]
                .iter()
                .any(|earlier| earlier.start.pick == Some(pick))
        })
    }) {
        return Err(GameSetupError::PickTaken(player));
    }
    // rule: TWN-SET-008
    if let Some((_, hero)) = setup
        .heroes
        .iter()
        .zip((0..=u16::MAX).map(HeroId::new))
        .find(|(hero, _)| hero.hero_type.is_none() && hero.owner.index() >= setup.players.len())
    {
        return Err(GameSetupError::RandomHeroOwner(hero));
    }
    Ok(())
}

/// The first player in id order that `refused` refuses (the players at most a [`PlayerId`]'s).
fn first_player(
    setup: &GameSetup,
    mut refused: impl FnMut(PlayerId, &PlayerSetup) -> bool,
) -> Option<PlayerId> {
    setup
        .players
        .iter()
        .zip((0..=u8::MAX).map(PlayerId::new))
        .find(|&(player, id)| refused(id, player))
        .map(|(_, id)| id)
}

/// Whether `player`'s faction choice is valid (TWN-SET-001): every faction of its set has a town
/// record (a draw gives one of them, `TWN-QUIRK-4`), and its choice is a faction of the set,
/// random among a set that is not empty, or none while its start needs no faction: no main town,
/// no random-hero flag, no random hero of its own and no random town aligned to its colour.
fn chooses(setup: &GameSetup, rules: &GameRuleset, id: PlayerId, player: &PlayerSetup) -> bool {
    let start = &player.start;
    start
        .factions
        .iter()
        .all(|faction| rules.towns().town(faction).is_some())
        && match start.choice {
            Some(FactionChoice::Fixed(faction)) => start.factions.contains(faction),
            Some(FactionChoice::Random) => !start.factions.is_empty(),
            None => {
                start.main_town.is_none()
                    && !start.random_hero
                    && !setup
                        .heroes
                        .iter()
                        .any(|hero| hero.hero_type.is_none() && hero.owner == id)
                    && !setup.towns.iter().any(|town| {
                        town.faction
                            == TownFaction::Random {
                                alignment: Some(player.colour),
                            }
                    })
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
