//! A player's start as the game's setup gives it (E09 T7a1, design note `E09-design.md` §13): the
//! factions it may start with, its choice among them, its starting hero's pick, its main town and
//! whether its starting hero is random. A map's import fills it (`h3-map`, TWN-SET-001); whoever
//! seats the players may change the choice and the pick before the game is built (E09 decision
//! 6's seam), the pick from the player's offer ([`GameSetup::offer`], TWN-SET-006). The game's
//! build checks the starts and the towns (E09 T7a2: [`check_starts`]), then builds the hero pool
//! and draws the random factions and the random heroes' types (E09 T7b1: [`pool`], design note
//! §14), then the random towns' factions (E09 T7b2a: [`build`]), then the main towns' heroes (E09
//! T7b2b: [`heroes`]), then each town's garrison and buildings ([`build`]). In play the game
//! composes a town's capture ([`capture`], E09 T8a2) and builds a siege's battle ([`siege`],
//! E09 T8b1); since E09 T8c1 it pays the towns' income in the day's advance ([`income`]) and
//! tells each viewer what it sees of the towns ([`view`]).

use alloc::vec::Vec;

use h3_adventure::{MapSetup, ObjectKind, ObjectSetup, Pos};
use h3_content::{FactionId, HeroTypeId};
use h3_contracts::{HeroId, PlayerId, TownId};
use h3_towns::TownFaction;

use crate::{GameRuleset, GameSetup, GameSetupError, PlayerSetup};

pub(crate) mod build;
pub(crate) mod capture;
pub(crate) mod heroes;
pub(crate) mod income;
pub(crate) mod pool;
pub(crate) mod siege;
pub(crate) mod view;

pub(crate) use build::{adventure_reveal, build_towns, draw_towns};
pub(crate) use capture::capture;
pub(crate) use heroes::main_town_heroes;
pub(crate) use income::add_income;
pub(crate) use pool::{HeroPool, draw_heroes};
pub(crate) use siege::{mana_after, merge, siege_experience, siege_setup};

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
    /// Drawn among the player's factions when the game is built (TWN-SET-001, E09 T7b1).
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

/// What the setup's checks of the starts give the build ([`check_starts`]): the towns' owners at
/// the start by `TownId`, and by player id the tile its main town's hero stands on when its main
/// town generates one (E09 T7b2b: the town's entrance, [`entrance`]).
pub(crate) type Starts = (Vec<Option<PlayerId>>, Vec<Option<Pos>>);

/// The setup's checks of the towns and the players' starts (E09 T7a2, design note §13), one pass
/// per invariant (ADR-0015 point 4): since E09 T8a1 one town record per town of the static map,
/// then a town's faction (a fixed one's town record; since E09 T7b2a a random town needs a faction
/// with one), a random town's alignment, each player's faction choice, main town (since E09 T8a1
/// a town it owns at the start, §8; since E09 T7b2b one visited from a tile of the map when it
/// generates a hero, TWN-SET-009), pick and a pick of no earlier player, then a random hero's
/// owner. The towns' owners at the start by `TownId` and the main towns' heroes' tiles by player
/// ([`Starts`]) when every check passes.
///
/// # Errors
///
/// The first [`GameSetupError`] found, in that order: [`GameSetupError::TownCount`],
/// [`GameSetupError::TownFaction`], [`GameSetupError::TownAlignment`],
/// [`GameSetupError::Choice`], [`GameSetupError::MainTown`], [`GameSetupError::Pick`],
/// [`GameSetupError::PickTaken`], [`GameSetupError::RandomHeroOwner`].
pub(crate) fn check_starts(
    setup: &GameSetup,
    rules: &GameRuleset,
) -> Result<Starts, GameSetupError> {
    let objects = town_objects(setup, rules);
    let owners: Vec<Option<PlayerId>> = objects.iter().map(|object| object.owner).collect();
    // rule: TWN-SET-009
    let entrances: Vec<Option<Pos>> = setup
        .players
        .iter()
        .map(|player| {
            player
                .start
                .main_town
                .filter(|main| main.generates_hero)
                .and_then(|main| objects.get(main.town.index()))
                .and_then(|object| entrance(&setup.map, object))
        })
        .collect();
    if owners.len() != setup.towns.len() {
        return Err(GameSetupError::TownCount(setup.towns.len()));
    }
    let towns = || setup.towns.iter().zip((0..=u16::MAX).map(TownId::new));
    // rule: TWN-SET-002
    let no_records = rules.towns().towns().is_empty();
    if let Some((_, town)) = towns().find(|(town, _)| match town.faction {
        TownFaction::Fixed(faction) => rules.towns().town(faction).is_none(),
        TownFaction::Random { .. } => no_records,
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
    if let Some(player) = first_player(setup, |id, player| {
        !chooses(setup, rules, &owners, id, player)
    }) {
        return Err(GameSetupError::Choice(player));
    }
    // rule: TWN-SET-001, TWN-SET-009
    if let Some(player) = first_player(setup, |id, player| {
        player.start.main_town.is_some_and(|main| {
            owners.get(main.town.index()) != Some(&Some(id))
                || main.generates_hero && entrances[id.index()].is_none()
        })
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
    Ok((owners, entrances))
}

/// The static map's towns by `TownId` (E09 T8a1, design note §8), each with its owner at the
/// start: the objects whose type is of kind [`ObjectKind::Town`], in object order, as the static
/// map lists them (`h3_adventure::StaticMap::towns`; no random dwelling becomes a town); an
/// object of an unknown type is none (the adventure's own check refuses it).
fn town_objects<'s>(setup: &'s GameSetup, rules: &GameRuleset) -> Vec<&'s ObjectSetup> {
    setup
        .map
        .objects
        .iter()
        .filter(|object| {
            object
                .object_type
                .and_then(|id| rules.adventure().object_type(id))
                .is_some_and(|ty| ty.kind == ObjectKind::Town)
        })
        .collect()
}

/// A town's entrance on `map` (TWN-SET-009, E09 T7b2b): the first tile of the map `object` is
/// visited from, in its visitable mask's order (row by row up from its anchor's, each row from
/// the anchor's column leftwards), as the adventure's `MapObject::entrance` finds it on the
/// static map it builds; none when no visitable tile of it lies on the map.
fn entrance(map: &MapSetup, object: &ObjectSetup) -> Option<Pos> {
    let anchor = object.anchor;
    object.visitable.offsets().find_map(|(column, row)| {
        let x = anchor.x.checked_sub(u16::from(column))?;
        let y = anchor.y.checked_sub(u16::from(row))?;
        (x < map.width && y < map.height && anchor.level < map.levels).then_some(Pos::new(
            x,
            y,
            anchor.level,
        ))
    })
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
/// random among a set that is not empty, or none while its start needs no faction
/// ([`needs_faction`]): no main town, no random-hero flag, no random hero of its own, no random
/// town aligned to its colour and, since E09 T7b2a, no random town it owns at the start
/// (`owners`, by `TownId`).
fn chooses(
    setup: &GameSetup,
    rules: &GameRuleset,
    owners: &[Option<PlayerId>],
    id: PlayerId,
    player: &PlayerSetup,
) -> bool {
    let start = &player.start;
    start
        .factions
        .iter()
        .all(|faction| rules.towns().town(faction).is_some())
        && match start.choice {
            Some(FactionChoice::Fixed(faction)) => start.factions.contains(faction),
            Some(FactionChoice::Random) => !start.factions.is_empty(),
            None => !needs_faction(setup, owners, id, player),
        }
}

/// Whether `player`'s start reads its faction (TWN-SET-001, TWN-SET-002): it has a main town, the
/// random-hero flag, a random hero of its own, a random town aligned to its colour or, since E09
/// T7b2a, a random town it owns at the start (`owners`, the towns' owners by `TownId`; aligned to
/// a colour or not). Only such a player's `Random` choice is drawn when the game is built
/// ([`draw_heroes`]).
pub(crate) fn needs_faction(
    setup: &GameSetup,
    owners: &[Option<PlayerId>],
    id: PlayerId,
    player: &PlayerSetup,
) -> bool {
    let start = &player.start;
    start.main_town.is_some()
        || start.random_hero
        || setup
            .heroes
            .iter()
            .any(|hero| hero.hero_type.is_none() && hero.owner == id)
        || setup.towns.iter().zip(owners).any(|(town, &owner)| {
            let random = matches!(town.faction, TownFaction::Random { .. });
            town.faction
                == TownFaction::Random {
                    alignment: Some(player.colour),
                }
                || random && owner == Some(id)
        })
}

#[cfg(test)]
mod tests;
