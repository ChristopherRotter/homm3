//! The hero pool (TWN-SET-005; E09 T7b1, design note `E09-design.md` §14, ADR-0017's E09
//! paragraph) and the build's draws of the heroes: each player's random faction choice
//! (TWN-SET-001), then each random hero's type (TWN-SET-008: its owner's pick, else the draw,
//! TWN-SET-007), from the setup stream before every other draw of the build (the random towns'
//! factions next, [`super::build`], then the main towns' heroes, [`super::heroes`]).

use alloc::vec;
use alloc::vec::Vec;

use h3_content::{FactionId, HeroType, HeroTypeId, Ruleset};
use h3_contracts::{HeroId, PlayerId};
use h3_kernel::{CanonicalHash, Rng, StateHasher};

use super::{FactionChoice, needs_faction};
use crate::{GameRuleset, GameSetup, GameSetupError};

/// Where a hero type stands in the pool (TWN-SET-005).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HeroStatus {
    /// Free: a draw may give it.
    Free,
    /// Taken by the player of this colour: placed, drawn or reserved as its pick.
    Taken(u8),
    /// Taken as a prisoner's.
    Prisoner,
    /// Out of the game: the map does not allow it.
    Out,
}

/// The hero pool (TWN-SET-005; ADR-0017's E09 paragraph): per hero type of the core ruleset, by
/// its index, where it stands and its colour mask (bit `c`: colour `c` may take it; 0xFF unless
/// the setup's masks restrict it). Built with the game ([`HeroPool::new`]), changed by the
/// build's draws and when a hero leaves the map ([`HeroPool::free`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HeroPool {
    types: Vec<(HeroStatus, u8)>,
}

impl HeroPool {
    /// The pool before any draw (TWN-SET-005; `hero_pool_init` 0x4C4E30, survey `E09-towns.md`
    /// §4.2): each type the setup allows free, the rest out; each type's mask the AND of the
    /// setup's masks of it (0xFF without one). Then, whatever the allowed set, every placed hero's
    /// type taken by its owner's colour (an owner that is no player is the adventure context's
    /// check: skipped), every prisoner's taken as a prisoner, and last each player's pick
    /// reserved, taken by its colour (T-2, T-3 fixed: `TWN-QUIRK-2`, `TWN-QUIRK-3`). A type the
    /// ruleset does not have is skipped.
    // rule: TWN-SET-005
    pub(crate) fn new(core: &Ruleset, setup: &GameSetup) -> Self {
        let mut pool = Self {
            types: vec![(HeroStatus::Out, 0xFF); core.hero_types().len()],
        };
        for &id in &setup.allowed_heroes {
            pool.set(id, HeroStatus::Free);
        }
        for &(id, mask) in &setup.hero_masks {
            if let Some(entry) = pool.types.get_mut(id.index()) {
                entry.1 &= mask;
            }
        }
        // The placed heroes' and prisoners' types before any draw (T-3 fixed, `TWN-QUIRK-3`).
        for hero in &setup.heroes {
            let owner = setup.players.get(hero.owner.index());
            if let (Some(id), Some(owner)) = (hero.hero_type, owner) {
                pool.set(id, HeroStatus::Taken(owner.colour));
            }
        }
        for &id in &setup.prisoners {
            pool.set(id, HeroStatus::Prisoner);
        }
        // The picks reserved before any draw (T-2 fixed, `TWN-QUIRK-2`).
        for player in &setup.players {
            if let Some(pick) = player.start.pick {
                pool.set(pick, HeroStatus::Taken(player.colour));
            }
        }
        pool
    }

    /// Where `id` stands; `None` for a type the ruleset does not have.
    #[cfg(test)]
    pub(crate) fn status(&self, id: HeroTypeId) -> Option<HeroStatus> {
        self.types.get(id.index()).map(|&(status, _)| status)
    }

    /// `id`'s colour mask; `None` for a type the ruleset does not have.
    #[cfg(test)]
    pub(crate) fn mask(&self, id: HeroTypeId) -> Option<u8> {
        self.types.get(id.index()).map(|&(_, mask)| mask)
    }

    /// Sets where `id` stands (a type the ruleset does not have: nothing).
    fn set(&mut self, id: HeroTypeId, status: HeroStatus) {
        if let Some(entry) = self.types.get_mut(id.index()) {
            entry.0 = status;
        }
    }

    /// `id` goes back free (TWN-SET-005: a hero that leaves the map, the original's 0x4D9EC0;
    /// an unused pick after the main towns' heroes, [`super::main_town_heroes`]).
    pub(crate) fn free(&mut self, id: HeroTypeId) {
        self.set(id, HeroStatus::Free);
    }

    /// `id` is taken by the player of `colour` (a drawn or picked hero's type, TWN-SET-007).
    pub(super) fn take(&mut self, id: HeroTypeId, colour: u8) {
        self.set(id, HeroStatus::Taken(colour));
    }

    /// The draw (TWN-SET-007; `hero_draw_of_faction` 0x4BB400, survey `E09-towns.md` §4.4): one
    /// of the free types that `colour` may take, uniform, among those of `faction`'s hero classes
    /// (the core's classes of the faction) when there is one, else among them all; drawn from
    /// `draws` only when there is one. The type is not taken here.
    // rule: TWN-SET-007
    pub(super) fn draw<'r>(
        &self,
        core: &'r Ruleset,
        faction: Option<FactionId>,
        colour: u8,
        draws: &mut Rng,
    ) -> Option<(HeroTypeId, &'r HeroType)> {
        let takeable = || {
            core.hero_types()
                .iter()
                .zip(&self.types)
                .zip(0..)
                .filter(move |&((_, &(status, mask)), _)| {
                    status == HeroStatus::Free
                        && mask
                            .checked_shr(u32::from(colour))
                            .is_some_and(|m| m & 1 == 1)
                })
                .filter_map(|((ty, _), index)| HeroTypeId::from_index(index).map(|id| (id, ty)))
        };
        let of_faction: Vec<_> = takeable()
            .filter(|(_, ty)| {
                core.hero_class(ty.class)
                    .is_some_and(|class| Some(class.faction) == faction)
            })
            .collect();
        let listed = if of_faction.is_empty() {
            takeable().collect()
        } else {
            of_faction
        };
        // `below(0)` draws nothing and gives 0, past an empty list.
        let count = u32::try_from(listed.len()).unwrap_or(u32::MAX);
        let drawn = usize::try_from(draws.below(count)).unwrap_or(usize::MAX);
        listed.get(drawn).copied()
    }
}

impl CanonicalHash for HeroPool {
    /// The game state hash's encoding (v23, E09 T7b1): the number of types, then each type in
    /// index order: where it stands (0 free, 1 taken and its colour, 2 a prisoner's, 3 out) and
    /// its colour mask.
    fn hash_into(&self, h: &mut StateHasher) {
        h.write_len(self.types.len());
        for &(status, mask) in &self.types {
            match status {
                HeroStatus::Free => h.write_u8(0),
                HeroStatus::Taken(colour) => {
                    h.write_u8(1);
                    h.write_u8(colour);
                }
                HeroStatus::Prisoner => h.write_u8(2),
                HeroStatus::Out => h.write_u8(3),
            }
            h.write_u8(mask);
        }
    }
}

/// Each hero's type in hero order, with the ruleset's record of it.
pub(crate) type HeroTypes<'r> = Vec<(HeroTypeId, &'r HeroType)>;

/// What the heroes' draws give the rest of the build ([`draw_heroes`], then
/// [`super::main_town_heroes`]).
pub(crate) struct Drawn<'r> {
    /// The pool.
    pub(crate) pool: HeroPool,
    /// Each hero's type in hero order: the setup's heroes, then the main towns' heroes.
    pub(crate) types: HeroTypes<'r>,
    /// Each player's resolved faction by id (`None` without a choice, or a `Random` one its start
    /// does not read).
    pub(crate) factions: Vec<Option<FactionId>>,
    /// Each player's pick by id that no hero has taken yet (still taken in the pool by its
    /// colour until the main towns' heroes free it).
    pub(crate) picks: Vec<Option<HeroTypeId>>,
}

/// The pool, each hero's type in hero order and each player's resolved faction by id (`None`
/// without a choice, or a `Random` one its start does not read), the build's first draws from
/// `draws` (design note §14, E09 T7b1): every placed hero's type checked first (before any draw),
/// the pool built ([`HeroPool::new`]); then each player's `Random` faction choice in player order,
/// uniform over its factions (TWN-SET-001; the resolved factions are not kept in the game, the
/// build's random towns read them, [`super::draw_towns`]), only when its start reads it
/// ([`needs_faction`] with the towns' owners at the start, `owners` by `TownId`: a choice nothing
/// reads draws nothing, the agent's call, so a map whose players allow several factions but have
/// no towns and no random hero, as the duel's, keeps its stream); then each random hero in hero
/// order (TWN-SET-008): its owner's pick for the owner's first random hero, else the draw over
/// the owner's faction (TWN-SET-007), the type taken by the owner's colour at once. Each pick no
/// random hero took is given back unused ([`Drawn::picks`], still taken: a human player's main
/// town's hero may take it, and the main towns' heroes free the rest, the agent's call, design
/// note §14). The setup's checks of the starts ran before ([`super::check_starts`]: a random
/// hero's owner is a player with a faction choice, a `Random` choice has factions, a pick is
/// offered and free).
///
/// # Errors
///
/// [`GameSetupError::UnknownHeroType`] for the first placed hero whose type the ruleset does not
/// have; [`GameSetupError::NoFreeHero`] for the owner of the first random hero the draw finds no
/// type for (`TWN-QUIRK-4`).
// rule: TWN-SET-001, TWN-SET-008
pub(crate) fn draw_heroes<'r>(
    setup: &GameSetup,
    rules: &'r GameRuleset,
    owners: &[Option<PlayerId>],
    draws: &mut Rng,
) -> Result<Drawn<'r>, GameSetupError> {
    let core = rules.core();
    let placed: Vec<Option<(HeroTypeId, &HeroType)>> = setup
        .heroes
        .iter()
        .zip((0..=u16::MAX).map(HeroId::new))
        .map(|(hero, id)| {
            hero.hero_type
                .map(|t| {
                    core.hero_type(t)
                        .map(|ty| (t, ty))
                        .ok_or(GameSetupError::UnknownHeroType(id))
                })
                .transpose()
        })
        .collect::<Result<_, _>>()?;
    let mut pool = HeroPool::new(core, setup);
    let factions: Vec<Option<FactionId>> = setup
        .players
        .iter()
        .zip((0..=u8::MAX).map(PlayerId::new))
        .map(|(player, id)| match player.start.choice {
            Some(FactionChoice::Fixed(faction)) => Some(faction),
            Some(FactionChoice::Random) if needs_faction(setup, owners, id, player) => {
                let set = player.start.factions;
                let drawn = usize::try_from(draws.below(set.len())).unwrap_or(usize::MAX);
                set.iter().nth(drawn)
            }
            _ => None,
        })
        .collect();
    let mut picks: Vec<Option<HeroTypeId>> = setup
        .players
        .iter()
        .map(|player| player.start.pick)
        .collect();
    let mut types = Vec::with_capacity(placed.len());
    for (hero, placed) in setup.heroes.iter().zip(placed) {
        let found = match placed {
            Some(found) => found,
            None => {
                let owner = hero.owner.index();
                let colour = setup.players[owner].colour;
                let pick = picks[owner].take();
                let picked = pick.and_then(|t| core.hero_types().get(t.index()).map(|ty| (t, ty)));
                let found = picked
                    .or_else(|| pool.draw(core, factions[owner], colour, draws))
                    .ok_or(GameSetupError::NoFreeHero(hero.owner))?;
                pool.take(found.0, colour);
                found
            }
        };
        types.push(found);
    }
    Ok(Drawn {
        pool,
        types,
        factions,
        picks,
    })
}
