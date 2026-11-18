//! A hero that leaves the game (TWN-TAV-006; E09 T38b1, design note `E09-design.md` §67; survey
//! `E09-tavern.md` §4): every path off the map, so a battle's (a loser, a retreat, a surrender,
//! both after a draw, a winner left with stacks created in the battle only: [`after_battle`]),
//! the capture's garrison hero without a stack (`TWN-QUIRK-11`, `capture.rs`) and since E09
//! T38b2 the dismissal ([`dismiss`], GAM-MAN-003, design note §69, survey §12), once the
//! adventure took the hero off the map (its marks gone, its boat freed, `HeroRemoved` told) and,
//! after a battle, after the heroes' part, so that the leave's own writes come last, as the
//! original's after the battle. The original's `hero_leave_game` 0x4D9EC0, read whole; batch
//! TWN-4's rows TWN-4-3 and TWN-4-4 observed it, TWN-4-5 the dismissal's artifacts.

use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;

use h3_adventure::{AdventureState, HeroOnMap};
use h3_content::{HeroType, LEAVE_POINTS_OVER_CLEARED_ARMY};
use h3_contracts::{ArmyStack, BattleResult, BattleSide, HeroId, PlayerId};

use super::pool::Away;
use super::tavern;
use crate::command::{AdventureSink, Sink};
use crate::manage::ManageError;
use crate::{GameRuleset, GameState};

/// How a hero leaves the game (design note §67).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Leave<'a> {
    /// After its side's retreat: back into its owner's tavern, with one creature.
    Retreated,
    /// After its side's surrender: back into its owner's tavern with its side's survivors.
    Surrendered(&'a [ArmyStack]),
    /// Every other leave (a loss, a draw, a winner left with created stacks only, the capture's,
    /// the dismissal): its type free.
    Gone,
}

/// The leave of `hero`, as the adventure took it off the map (TWN-TAV-006; design note §67), by
/// `how`. Its record keeps its level, experience, skills, spells and the artifacts it holds now;
/// its away record keeps its formation and tactics flag (no battle bonuses, no timed marks: the
/// adventure dropped them).
///
/// - [`Leave::Gone`]: its type free; its army one creature of its type's first starting stack
///   (GAM-SET-001's first stack at a count of 1, no draw: a later draw replaces it; none for a
///   type without one); its spell points full (HER-MAN-001); its points left and the day's
///   maximum the land maximum (`h3_adventure::day_points`) over the army cleared before the one
///   creature is placed (`TWN-QUIRK-12`, T-16, faithful: an empty army counts as the fastest, the
///   land table's most, as the original's empty slots' speed 20 reaches it), else over the one
///   creature.
/// - [`Leave::Retreated`], [`Leave::Surrendered`]: into its owner's tavern at once (TD-19): the
///   slot one draw of 0 to 1 from **its owner's stream** (its side's command ended the battle;
///   ADR-0017 point 4), the other slot when that one's returning mark is set (so a third
///   returning hero replaces one of two); that slot's hero, if any, free; the type offered, the
///   slot holding it, its returning mark set (TD-18). A surrendered hero keeps its side's
///   survivors (the adventure's end left the army it fought with: the outcome's are the
///   survivors), a retreated hero one creature as above (and a surrendered one whose survivors
///   are none: its surviving war machines or created stacks paid for it; the agent's call, the
///   original's empty army not kept, so that a hire never places a hero without a stack); both
///   keep their points left, the day's maximum and the spell points the battle left them, the
///   points stale for its owner's next turn start (TWN-TAV-005).
///
/// A hero that is not its type's record in the pool (a setup that repeats a type names its first
/// hero, TWN-SET-005; no shipped map places a type twice) becomes the type's record as it leaves
/// (the agent's call: its type goes free as before E09 T38b1, now with its away record), unless
/// the type is a tavern hero's now: then it leaves the pool as it is, so a slot keeps its view.
/// Nothing is told: a return's view is the caller's to tell ([`after_battle`]).
// rule: TWN-TAV-006, GAM-SET-001
pub(crate) fn leave(state: &mut GameState, rules: &GameRuleset, hero: &HeroOnMap, how: Leave<'_>) {
    let core = rules.core();
    // A hero the adventure held has its record in the store, of a type of the ruleset.
    let hero_type = state.heroes()[usize::from(hero.id.raw())].hero_type;
    let ty = &core.hero_types()[hero_type.index()];
    if state.pool().record(hero_type) != Some(hero.id) {
        if state.pool().is_offered(hero_type) {
            return;
        }
        Arc::make_mut(state.taverns().pool).set_record(hero_type, hero.id);
    }
    let one = one_creature(ty);
    match how {
        Leave::Gone => {
            let parts = state.taverns();
            let details = refilled(parts.heroes, hero.id, rules);
            let cleared = if core.quirks().is_enabled(LEAVE_POINTS_OVER_CLEARED_ARMY) {
                &[][..]
            } else {
                &one[..]
            };
            let points = h3_adventure::day_points(core, rules.adventure(), details, cleared, 0);
            let away = Away {
                movement: points,
                max_movement: points,
                formation: hero.formation,
                tactics: hero.tactics,
                army: one,
                stale: false,
                full: true,
            };
            Arc::make_mut(parts.pool).leave(hero_type, away);
        }
        Leave::Retreated | Leave::Surrendered(_) => {
            let details = state.hero(hero.id);
            let full = details.is_some_and(|d| d.mana >= h3_heroes::max_mana(d, core));
            let army = match how {
                Leave::Surrendered(survivors) if !survivors.is_empty() => survivors.to_vec(),
                _ => one,
            };
            let away = Away {
                movement: hero.movement,
                max_movement: hero.max_movement,
                formation: hero.formation,
                tactics: hero.tactics,
                army,
                stale: true,
                full,
            };
            let returning = state
                .pool()
                .tavern(hero.owner)
                .map_or([false; 2], |tavern| tavern.returning);
            let parts = state.taverns_for(hero.owner);
            let drawn = parts.draws.range_inclusive(0, 1);
            let slot = usize::from(drawn == 1);
            let slot = if returning[slot] { 1 - slot } else { slot };
            Arc::make_mut(parts.pool).return_to((hero.owner, slot), hero_type, away);
        }
    }
}

/// The leaves of a battle's heroes that `left` (as the adventure's end returned them, in its
/// order: the attacker's, then the defender's; TWN-TAV-006, GAM-BAT-003), after the battle's
/// outcome whole: the hero of the side that retreated or surrendered (`result`) returns
/// ([`Leave::Retreated`], [`Leave::Surrendered`] with its side's `survivors`), every other is
/// [`Leave::Gone`]; then each slot whose view a return changed is told to its owner
/// ([`tavern::tell`]: `TavernOffered` with its parts, the freed hero's slot by the same event).
// rule: TWN-TAV-006, GAM-BAT-003
pub(crate) fn after_battle<S: Sink>(
    state: &mut GameState,
    rules: &GameRuleset,
    (left, heroes): (&[HeroOnMap], [Option<HeroId>; 2]),
    (result, survivors): (BattleResult, [&[ArmyStack]; 2]),
    sink: &mut S,
) {
    if left.is_empty() {
        return;
    }
    let fled = match result {
        BattleResult::Retreat(side) => Some((side, false)),
        BattleResult::Surrender(side) => Some((side, true)),
        _ => None,
    };
    let returns = fled.is_some_and(|(side, _)| {
        left.iter()
            .any(|hero| heroes[side.index()] == Some(hero.id))
    });
    let before = (S::RECORDS && returns).then(|| tavern::all_views(state));
    for hero in left {
        let side = BattleSide::BOTH
            .into_iter()
            .find(|side| heroes[side.index()] == Some(hero.id));
        let how = match (fled, side) {
            (Some((fled, false)), Some(side)) if fled == side => Leave::Retreated,
            (Some((fled, true)), Some(side)) if fled == side => {
                Leave::Surrendered(survivors[side.index()])
            }
            _ => Leave::Gone,
        };
        leave(state, rules, hero, how);
    }
    if let Some(before) = before {
        tavern::tell(state, &before, sink);
    }
}

/// The dismissal of `player`'s hero `hero` (GAM-MAN-003; E09 T38b2, design note §69; survey
/// `E09-tavern.md` §12, the hero screen's button 129, `hero_leave_game` 0x4D9EC0 with (1, 0)):
/// checked first, in this order, a refusal changing nothing: the player's own hero
/// (`NotYourHero`: another player's, a tavern hero, one out of the game), on the map (`NotOnMap`:
/// its garrison hero, whom the town screen offers no dismissal), not a town's visitor on its
/// entrance (`InTown`: the original's `hero_screen_refresh` 0x4E1A6B tests the tile, TD-22's
/// reading, decision 118; VCMI's button does not), not the player's one hero on the map while it
/// owns no town (`LastHero`). The caller checked the turn and that no decision is open. Then the
/// adventure takes it off the map (`AdventureState::remove_hero`: `HeroRemoved` to its owner and
/// the players that see its tile, its boat freed, its marks gone) and it leaves without return
/// ([`Leave::Gone`]): its type free with its record and every artifact it holds (decision 105,
/// row TWN-4-5), one creature, its spell points full, T-16's points. No draw; nothing more told
/// (the pool is hidden).
///
/// # Errors
///
/// The refusals above, as [`ManageError`]; nothing changed.
// rule: GAM-MAN-003, TWN-TAV-006
pub(crate) fn dismiss<S: Sink>(
    state: &mut GameState,
    rules: &GameRuleset,
    (player, hero): (PlayerId, HeroId),
    sink: &mut S,
) -> Result<(), ManageError> {
    let adventure = state.adventure();
    if adventure
        .garrisoned(hero)
        .is_some_and(|(_, held)| held.owner == player)
    {
        return Err(ManageError::NotOnMap);
    }
    if let Some(on) = adventure.hero(hero).filter(|on| on.owner == player) {
        check_dismissal(adventure, player, on)?;
    }
    let (adventure, _) = state.adventure_with_heroes();
    let removed = adventure
        .remove_hero(rules.core(), (player, hero), &mut AdventureSink(&mut *sink))
        .ok_or(ManageError::NotYourHero(hero))?;
    leave(state, rules, &removed, Leave::Gone);
    Ok(())
}

/// Checks that `player`'s hero on the map `on` may be dismissed (GAM-MAN-003; [`dismiss`]'s map
/// half, and the listing's, `crate::legal_management`): not a town's visitor on its entrance
/// (`InTown`, TD-22), not the player's one hero on the map while it owns no town (`LastHero`).
///
/// # Errors
///
/// `InTown`, then `LastHero`.
// rule: GAM-MAN-003
pub(crate) fn check_dismissal(
    adventure: &AdventureState,
    player: PlayerId,
    on: &HeroOnMap,
) -> Result<(), ManageError> {
    let map = adventure.map();
    if map.town_at(on.pos).is_some() {
        return Err(ManageError::InTown);
    }
    // stub: TWN-STUB-16 (the loss condition "lose this hero", `hero_mission_critical` 0x5F2A40:
    // E10's scenarios)
    let others = adventure
        .heroes()
        .iter()
        .any(|other| other.owner == player && other.id != on.id);
    let towns = map
        .towns()
        .iter()
        .any(|&index| adventure.owner(index) == Some(player));
    if others || towns {
        Ok(())
    } else {
        Err(ManageError::LastHero)
    }
}

/// One creature of `ty`'s first starting stack, in slot 0 (GAM-SET-001's first stack at a count
/// of 1, no draw); none for a type without one. A leave's and a hire's replacement's army
/// (TWN-TAV-004, -006).
pub(super) fn one_creature(ty: &HeroType) -> Vec<ArmyStack> {
    ty.army.first().map_or(vec![], |first| {
        vec![ArmyStack {
            slot: 0,
            creature: first.creature,
            count: 1,
        }]
    })
}

/// `hero`'s record in `store` with its spell points full (HER-MAN-001), written only when they
/// are not (a fork shares the store).
fn refilled<'s>(
    store: &'s mut Arc<Vec<Arc<h3_heroes::HeroDetails>>>,
    hero: HeroId,
    rules: &GameRuleset,
) -> Option<&'s h3_heroes::HeroDetails> {
    let core = rules.core();
    let at = usize::from(hero.raw());
    let max = store.get(at).map(|d| h3_heroes::max_mana(d, core));
    if let Some(max) = max.filter(|&max| store[at].mana != max) {
        let details = Arc::make_mut(&mut Arc::make_mut(store)[at]);
        details.mana = max;
    }
    store.get(at).map(|d| &**d)
}
