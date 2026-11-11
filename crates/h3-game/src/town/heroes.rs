//! The main towns' heroes at the game's build (TWN-SET-009; E09 T7b2b, design note
//! `E09-design.md` §14 step 3; survey `E09-towns.md` §4.3, §4.5): after the random towns'
//! factions ([`super::draw_towns`]) and before each town's garrison and buildings
//! ([`super::build_towns`]), from the setup stream.

use alloc::vec::Vec;

use h3_adventure::Pos;
use h3_content::Formation;
use h3_contracts::PlayerId;
use h3_kernel::Rng;

use super::pool::Drawn;
use crate::{GameHeroSetup, GameRuleset, GameSetup, GameSetupError, HeroArmy, HeroDetailsSetup};

/// Each main town's hero (TWN-SET-009; `main_town_heroes` 0x4CA040): for each player in id order
/// (the players' colours strictly increase, so colour order, as the original's loop over colours
/// 0 to 7) whose main town generates its starting hero, a hero of its own on the town's entrance
/// (`entrances`, by player: [`super::check_starts`]), the town's visitor by standing there. Its
/// type: the player's pick when the player is human and no random hero took the pick
/// ([`Drawn::picks`]); else the draw (TWN-SET-007, [`super::HeroPool::draw`]) over the player's
/// resolved faction (the colour's, not the town's: survey §4.5) for its colour, taken by the
/// colour at once. A computer player's pick goes only to its first random hero (survey §4.3),
/// and stays out of its own draw here. Then every pick no hero took goes back free
/// (TWN-SET-005, the agent's call, design note §14). Each hero takes its type's starting army
/// ([`HeroArmy::Starting`], GAM-SET-001), the default formation and every default detail
/// (GAM-HER-001), drawn later with the setup's heroes in hero id order; the heroes are returned
/// in player order, their ids after the setup's heroes, and their types are appended to
/// [`Drawn::types`]. A placed hero already on the entrance (TWN-SET-010) is left to the
/// adventure's check of the heroes' tiles, which refuses the second hero there
/// (`SetupError::HeroTile` with `StandError::Occupied`; the original draws the new hero over the
/// visitor, `TWN-QUIRK-6`). The original then teaches the town's visitor and garrison hero the
/// guild's spells (0x5BE030 at 0x4CA1A2), which this does not (`TWN-STUB-6`, increment 5).
///
/// # Errors
///
/// [`GameSetupError::NoFreeHero`] for the first player whose draw finds no type (`TWN-QUIRK-4`).
// rule: TWN-SET-009
// stub: TWN-STUB-6 (the town's guild spells at the start: increment 5)
pub(crate) fn main_town_heroes<'r>(
    setup: &GameSetup,
    rules: &'r GameRuleset,
    entrances: &[Option<Pos>],
    drawn: &mut Drawn<'r>,
    draws: &mut Rng,
) -> Result<Vec<GameHeroSetup>, GameSetupError> {
    let core = rules.core();
    let mut heroes = Vec::new();
    let players = setup.players.iter().zip(entrances);
    for ((player, &entrance), id) in players.zip((0..=u8::MAX).map(PlayerId::new)) {
        let Some(pos) = entrance else {
            continue;
        };
        let index = id.index();
        let pick = if player.human {
            drawn.picks[index].take()
        } else {
            None
        };
        let picked = pick.and_then(|t| core.hero_types().get(t.index()).map(|ty| (t, ty)));
        let colour = player.colour;
        let found = picked
            .or_else(|| drawn.pool.draw(core, drawn.factions[index], colour, draws))
            .ok_or(GameSetupError::NoFreeHero(id))?;
        drawn.pool.take(found.0, colour);
        drawn.types.push(found);
        heroes.push(GameHeroSetup {
            hero_type: Some(found.0),
            owner: id,
            pos,
            army: HeroArmy::Starting,
            formation: Formation::default(),
            details: HeroDetailsSetup::default(),
        });
    }
    for pick in drawn.picks.drain(..).flatten() {
        drawn.pool.free(pick);
    }
    Ok(heroes)
}
