//! The Conflux's Magic University in the game (TWN-UNI-001; E09 T59b1, the town epic's design note
//! §112, decision 210; survey `E09-trades.md` §6): `TownCommand::University` opens E08's University
//! decision (GAM-DEC-006, amended) for the town's hero the command names, with the town as its
//! place (the original's town University runs the map's dialog functions, survey §6.3): its
//! answers, slots, price and payer are the map's University's (`GameState`'s `buy`). The legal
//! listing offers each opening `open` accepts whose University sells a slot the treasury pays
//! (`town::services`).

use h3_adventure::{UniversityPlace, UniversityVisit};
use h3_content::{SkillId, UNIVERSITY_IGNORES_MAP_BANS};
use h3_contracts::{HeroId, PlayerId, TownId};
use h3_heroes::SlotState;
use h3_towns::{TownCommandError, UniversitySale};

use super::command::owner;
use super::guild::heroes;
use crate::command::{GameCommandError, Sink};
use crate::{GameRuleset, GameState};

/// Opens the Magic University of `player`'s `town` for its `hero` (TWN-UNI-001; E09 T59b1, design
/// §112; survey §6.1 to §6.5), `player`'s turn without an open decision checked by
/// `GameState::check_ready`, in this order, each refusal changing nothing:
///
/// 1. the town in the store of towns, else [`TownCommandError::UnknownTown`];
/// 2. its owner `player`, else [`TownCommandError::NotOwner`] (the original tests no owner, survey
///    §6.5: an ally's town waits for E10's teams, `TWN-STUB-21`);
/// 3. [`h3_towns::check_university`]: a standing University, else
///    [`TownCommandError::NoUniversity`]; the part's skills and price;
/// 4. `hero` the town's visitor or its garrison hero, `player`'s, else
///    [`TownCommandError::NotTownHero`] (decision 207: the original's student, the visitor unless
///    a stack of the upper bar is selected and a garrison hero stands there, survey §6.2, the
///    command's choice; not `TWN-QUIRK-15`'s visitor alone: the University takes the garrison
///    hero).
///
/// Then, infallibly: E08's University decision opened with the town as its place
/// (`GameState::open_university`, `UniversityOpened` to `player`), its skills [`taught`] (the
/// part's, a school the scenario bans left out with `TWN-QUIRK-16`'s flag off), its price the
/// part's, whatever the treasury holds (the original's dialog opens and disables the purchase it
/// cannot pay). No movement point, no mark, any number a turn (survey §6.6). No field of the
/// command is trusted but its two ids: the skills and the price are the part's (ADR-0004).
///
/// # Errors
///
/// [`GameCommandError::Town`] with the refusal.
// rule: TWN-UNI-001
pub(crate) fn open<S: Sink>(
    state: &mut GameState,
    rules: &GameRuleset,
    player: PlayerId,
    (town, hero): (TownId, HeroId),
    sink: &mut S,
) -> Result<(), GameCommandError> {
    let record = state
        .towns()
        .get(town)
        .ok_or(GameCommandError::Town(TownCommandError::UnknownTown))?;
    if owner(state, town) != Some(player) {
        return Err(GameCommandError::Town(TownCommandError::NotOwner));
    }
    let sale = h3_towns::check_university(record, rules.towns()).map_err(GameCommandError::Town)?;
    if !heroes(state, town).contains(&Some((hero, player))) {
        return Err(GameCommandError::Town(TownCommandError::NotTownHero));
    }
    let visit = UniversityVisit {
        hero,
        place: UniversityPlace::Town(town),
        skills: taught(state, rules, sale.skills).collect(),
        price: sale.price.to_vec(),
    };
    state.open_university(player, visit, sink);
    Ok(())
}

/// The skills a town's University offers of the part's `skills` (TWN-UNI-001; design §112;
/// `TWN-QUIRK-16`, T-23): with the flag `quirk.town.university_ignores_map_bans` on (`faithful`)
/// all of them, a magic school the scenario bans offered and taught (the original's slots and
/// purchase read no ban, survey §6.3, batch TWN-6's TWN-6-7); off (`strict-fixed`) those the
/// scenario bans left out (GAM-SET-002's banned skills, `GameState::banned_skills`), in the
/// part's order.
// rule: TWN-UNI-001
pub(super) fn taught<'a>(
    state: &'a GameState,
    rules: &GameRuleset,
    skills: &'a [SkillId],
) -> impl Iterator<Item = SkillId> + 'a {
    let bans = !rules
        .core()
        .quirks()
        .is_enabled(UNIVERSITY_IGNORES_MAP_BANS);
    skills
        .iter()
        .copied()
        .filter(move |skill| !bans || !state.banned_skills().contains(skill))
}

/// Whether the legal listing offers the University of a town to `hero`, `player`'s (TWN-UNI-001;
/// E09 T59b1, design §115: "listed where a slot is open and the treasury holds the price"): the
/// treasury pays the part's price and one of the skills [`taught`] has a slot open for the hero
/// (`GameState::slot_state`, HER-CHG-004), so that the opened University offers a purchase the
/// listing gives inside it. The caller found `hero` on the town's entrance or in its garrison.
// rule: TWN-UNI-001
pub(super) fn sells(
    state: &GameState,
    rules: &GameRuleset,
    (player, hero): (PlayerId, HeroId),
    sale: UniversitySale<'_>,
) -> bool {
    state.can_pay(player, sale.price)
        && taught(state, rules, sale.skills)
            .any(|skill| state.slot_state(rules, hero, skill) == SlotState::Open)
}
