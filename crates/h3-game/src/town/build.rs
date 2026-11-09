//! The towns at the game's build (E09 T7b2a, design note `E09-design.md` §14, §5): step 2's random
//! towns' factions (TWN-SET-002), after the random heroes' draws ([`super::draw_heroes`]), then
//! step 4, each town's garrison and buildings (TWN-SET-004, TWN-SET-003: `h3_towns::town_setup`),
//! both from the setup stream; since E09 T18a the Shipyards without water once the static map fixed
//! the towns' spots (TWN-SET-011); and each town's reveal at the start (TWN-CAP-001).

use alloc::vec::Vec;

use h3_adventure::{StaticMap, TownReveal};
use h3_content::FactionId;
use h3_contracts::{PlayerId, TownId};
use h3_kernel::Rng;
use h3_towns::{BuiltTown, MAX_TOWNS, Reveal, Town, TownFaction};

use crate::{GameRuleset, GameSetup};

/// Step 2's random towns (TWN-SET-002; design note §14 step 2, TD-1, TD-2): each town's faction in
/// `TownId` order. A fixed town's own, drawing nothing; a random town aligned to a colour, the
/// resolved faction of that colour's player; else, owned at the start by a player (`owners`, by
/// `TownId`), its owner's; else (neutral) one drawn from `draws`, uniform over the factions with a
/// town record in faction order (eight with the base pack, nine with AB). `factions` are the
/// players' resolved factions by id ([`super::draw_heroes`]). After the setup's checks
/// ([`super::check_starts`]) every town's is a faction with a town record: a fixed one has one, an
/// aligned colour is a player's, a player whose colour a random town is aligned to or that owns
/// one has a choice among factions with town records ([`super::needs_faction`]), and a random town
/// needs a faction with one (an owner that is no player, which the adventure refuses later, draws
/// as a neutral town's).
// rule: TWN-SET-002
pub(crate) fn draw_towns(
    setup: &GameSetup,
    rules: &GameRuleset,
    owners: &[Option<PlayerId>],
    factions: &[Option<FactionId>],
    draws: &mut Rng,
) -> Vec<Option<FactionId>> {
    let records = rules.towns().towns();
    setup
        .towns
        .iter()
        .zip(owners)
        .map(|(town, owner)| match town.faction {
            TownFaction::Fixed(faction) => Some(faction),
            TownFaction::Random { alignment } => {
                let player = alignment.map_or(owner.map(PlayerId::index), |colour| {
                    setup.players.iter().position(|p| p.colour == colour)
                });
                player
                    .and_then(|index| factions.get(index).copied().flatten())
                    .or_else(|| {
                        let count = u32::try_from(records.len()).unwrap_or(u32::MAX);
                        let drawn = usize::try_from(draws.below(count)).unwrap_or(usize::MAX);
                        records.get(drawn).map(|record| record.faction)
                    })
            }
        })
        .collect()
}

/// Step 4 (TWN-SET-004, TWN-SET-003; design note §14 step 4, §5): each town in `TownId` order
/// built by `h3_towns::town_setup` with its faction from step 2 ([`draw_towns`]), owned when it
/// has an owner at the start (`owners`), drawing from `draws` (its garrison's draws, then its
/// buildings'). After the setup's checks every town has a faction with a town record
/// ([`draw_towns`]), which `town_setup` refuses for nothing else (a part with a town record has
/// its parameters), so every town is built; one without would be left out, and the adventure
/// would refuse the setup for its garrisons' number (`SetupError::Garrisons`).
// rule: TWN-SET-003, TWN-SET-004
pub(crate) fn build_towns(
    setup: &GameSetup,
    rules: &GameRuleset,
    owners: &[Option<PlayerId>],
    factions: &[Option<FactionId>],
    draws: &mut Rng,
) -> Vec<BuiltTown> {
    setup
        .towns
        .iter()
        .zip(owners)
        .zip(factions)
        .take(MAX_TOWNS)
        .filter_map(|((town, owner), faction)| {
            faction.and_then(|faction| {
                let owned = owner.is_some();
                h3_towns::town_setup(town, faction, owned, rules.towns(), draws).ok()
            })
        })
        .collect()
}

/// The towns without water (TWN-SET-011; E09 T18a, design note §28): after the adventure built
/// the static map, which fixed each town's spot (`StaticMap::town_boat_spot`), and before the
/// store is made, each town (by `TownId`, its place in `towns`) whose spot is none gets its
/// Shipyard forbidden and out of its ever-built set (`h3_towns::without_water`); no draw.
// rule: TWN-SET-011
pub(crate) fn without_water<S>(towns: &mut [(Town, S)], map: &StaticMap, rules: &GameRuleset) {
    for ((town, _), id) in towns.iter_mut().zip((0..=u16::MAX).map(TownId::new)) {
        if map.town_boat_spot(id).is_none() {
            *town = h3_towns::without_water(town, rules.towns());
        }
    }
}

/// What `town` reveals to its owner (TWN-CAP-001; design note §8 and §11), as the adventure takes
/// it: at the start (`start_game` by `TownId`) and at a capture (`capture_town`):
/// `h3_towns::reveal`'s radius, or for [`Reveal::All`] (a Tower's Grail) every tile of every
/// level (E09 T8a2).
// rule: TWN-CAP-001
pub(crate) fn adventure_reveal(town: &Town, rules: &GameRuleset) -> TownReveal {
    match h3_towns::reveal(town, rules.towns()) {
        Reveal::Radius(radius) => TownReveal::Radius(radius),
        Reveal::All => TownReveal::All,
    }
}
