//! The turn's frame (E31 T2a; survey §1.1 to §1.4): the passes of `ai_player_turn` 0x525E80
//! (CPU-TRN-002), the order of heroes of `ai_next_hero` 0x526A90 and its copy in the second pass
//! (CPU-TRN-003), a hero's turn of `ai_hero_turn` 0x5261F0 with its garrison step (CPU-TRN-004).
//! One call gives one command: the frame runs on from its memory ([`Turn`]) until a step gives
//! one, and every step after a command reads the state the command made.
//!
//! What the frame calls and the engine or E31 has not built yet is stubbed: the middle
//! (`AI-STUB-3`, increment 3; the start's records since T2d, `super::start`), the town visit
//! (`AI-STUB-4`, increment 3),
//! the patrol (`AI-STUB-5`: no patrol in the game state), and each hero's walk (`AI-STUB-2`,
//! T2j): since T2i each hero plans (`super::plan`) and is then done.

use h3_adventure::{HeroOnMap, army_value};
use h3_content::Ruleset;
use h3_contracts::{ArmyStack, HeroId, PlayerId, TownId};
use h3_heroes::HeroDetails;
use h3_kernel::Rng;
use h3_towns::{TownCommand, TownHeroChange};

use super::artifacts::Valued;
use super::fight::Fighter;
use super::memory::{Memory, Pass, Plans, Turn};
use super::plan::{self, Planner};
use super::spells;
use crate::{GameCommand, GameRuleset, GameState};

/// The player's heroes on the map from which `ai_next_hero` 0x526A90 sends no garrison hero out
/// (its test of the player record's count +0x1 against 8, as read; the engine's own bound, the
/// town part's `most_heroes`, refuses the move too: [`h3_adventure::AdventureState::town_hero_moves`]).
const HEROES_SENT_OUT_BELOW: usize = 8;

/// The command of `player`'s turn now, its frame in `memory`: a hero's garrison step or a
/// garrison hero sent out, else, once both passes find no hero, `EndTurn` (CPU-TRN-002; survey
/// §1.1). A turn of another day than the memory's starts anew ([`start`]), the kept plans of the
/// heroes `player` no longer owns (on the map or in a town's garrison) dropped. The frame draws
/// nothing (survey §1.1); the planning draws from `rng` (T2i).
// rule: CPU-TRN-002
pub(super) fn step(
    state: &GameState,
    rules: &GameRuleset,
    player: PlayerId,
    memory: &mut Memory,
    rng: &mut Rng,
) -> GameCommand {
    let adventure = state.adventure();
    let owns = |hero| {
        let on_map = adventure.heroes().iter().filter(|h| h.owner == player);
        let held = towns_of(state, player).filter_map(|town| adventure.town_hero(town));
        on_map.chain(held).any(|h| h.id == hero)
    };
    let (turn, plans) = memory.turn_of(state.day(), || start(state, rules, player), owns);
    loop {
        let pass = turn.pass;
        if pass == Pass::Over {
            return GameCommand::EndTurn;
        }
        match next_hero(state, rules, player, turn) {
            Pick::Hero(hero, alone) => {
                let frame = (&mut *turn, &mut *plans);
                if let Some(command) = hero_turn(state, rules, player, frame, (hero, alone), rng) {
                    return command;
                }
            }
            Pick::SentOut(town, hero) => {
                turn.next = Some(hero);
                return heroes(town, TownHeroChange::Leave);
            }
            Pick::None if pass == Pass::First => {
                middle(state, player, turn);
                turn.pass = Pass::Second;
            }
            Pick::None => turn.pass = Pass::Over,
        }
    }
}

/// The turn's start (`ai_turn_start` 0x4297C0, survey §1.5; CPU-TRN-002): no hero done (every
/// hero's and garrison hero's +0x11C cleared), the first pass, the turn's flag ([`turn_flag`])
/// and since T2d the start's records ([`super::start::record`], CPU-STA-001).
fn start(state: &GameState, rules: &GameRuleset, player: PlayerId) -> Turn {
    Turn {
        day: state.day(),
        pass: Pass::First,
        flag: turn_flag(state, player),
        next: None,
        asleep: alloc::vec::Vec::new(),
        record: super::start::record(state, rules, player),
    }
}

/// The middle between the passes (`ai_turn_middle` 0x428DD0, survey §1.5): after it the turn's
/// flag is set again as at the start (`ai_player_turn` 0x525FD5), so a reveal of the first pass
/// that cleared it counts again in the second.
fn middle(state: &GameState, player: PlayerId, turn: &mut Turn) {
    // stub: AI-STUB-3
    turn.flag = turn_flag(state, player);
}

/// The turn's flag F (survey §1.1 step 4): the game's difficulty above Easy (the game's +0x1F6D8
/// not 0) and a town owned (the player record's +0x3E not 0).
fn turn_flag(state: &GameState, player: PlayerId) -> bool {
    state.adventure().difficulty() > 0 && towns_of(state, player).next().is_some()
}

/// The next hero to play ([`next_hero`]).
pub(super) enum Pick<'s> {
    /// A hero on the map, and the alone flag: it alone qualified.
    Hero(&'s HeroOnMap, bool),
    /// A garrison hero to send out onto its town's entrance; it plays next, without the alone
    /// flag.
    SentOut(TownId, HeroId),
    /// No hero: the pass ends.
    None,
}

/// The next hero of `player`'s pass (`ai_next_hero` 0x526A90, the second pass's copy at
/// 0x526000; survey §1.2; CPU-TRN-003): the hero a garrison just sent out, if it stands on the
/// map; else among its heroes on the map with points left and not done, the least sum of the
/// primary skills ([`primary_sum`]), a tie to the earlier in its list of heroes (GAM-HER-003), the
/// alone flag when it alone qualified. A free hero before a patrolling one: the engine has no
/// patrol, every hero is free (`AI-STUB-5`). With none and fewer than eight heroes on the map,
/// the first of its towns (in `TownId` order, the engine's order of a player's towns) whose
/// garrison hero carries creatures, has points and is not done, with no hero on its entrance:
/// that hero sent out, if the engine allows the move (`TownHeroChange::Leave`; the original's
/// `town_garrison_to_visitor` 0x5BE390 tests nothing more).
// rule: CPU-TRN-003
pub(super) fn next_hero<'s>(
    state: &'s GameState,
    rules: &GameRuleset,
    player: PlayerId,
    turn: &mut Turn,
) -> Pick<'s> {
    let adventure = state.adventure();
    if let Some(hero) = turn.next.take().and_then(|hero| adventure.hero(hero)) {
        return Pick::Hero(hero, false);
    }
    // stub: AI-STUB-5
    let mut best: Option<((u16, u32), &HeroOnMap)> = None;
    let mut qualifying = 0_usize;
    for hero in adventure
        .heroes()
        .iter()
        .filter(|hero| hero.owner == player && hero.movement > 0 && !turn.is_asleep(hero.id))
    {
        qualifying += 1;
        let key = (primary_sum(state, rules, hero.id), hero.listed);
        if best.is_none_or(|(least, _)| key < least) {
            best = Some((key, hero));
        }
    }
    if let Some((_, hero)) = best {
        return Pick::Hero(hero, qualifying == 1);
    }
    let on_map = adventure
        .heroes()
        .iter()
        .filter(|hero| hero.owner == player)
        .count();
    if on_map >= HEROES_SENT_OUT_BELOW {
        return Pick::None;
    }
    let most = most_heroes(rules);
    towns_of(state, player)
        .find_map(|town| {
            let held = adventure.town_hero(town)?;
            let sent = adventure.town_visitor(town).is_none()
                && carries_creatures(&held.army)
                && held.movement > 0
                && !turn.is_asleep(held.id);
            sent.then_some((town, held.id))
        })
        .filter(|&(town, _)| adventure.town_hero_moves(rules.adventure(), player, (town, most))[1])
        .map_or(Pick::None, |(town, hero)| Pick::SentOut(town, hero))
}

/// A hero's primary skills as `ai_next_hero` 0x526A90 sums them (survey §1.2): each of the four
/// effective values the adventure map reads (`h3_heroes::primary_skills`, the original's +0x476
/// to +0x479 with the worn artifacts) counted by [`counted`], Attack and Defense from 0, Spell
/// Power and Knowledge from 1.
fn primary_sum(state: &GameState, rules: &GameRuleset, hero: HeroId) -> u16 {
    state
        .hero(hero)
        .map_or(0, |details| primary_sum_of(details, rules.core()))
}

/// [`primary_sum`] of a hero's details (`hero_primary_sum` 0x4E5960 sums the same: each held to
/// 0 to 99, Spell Power and Knowledge at least 1; E31 T2f, the Ballista's value).
pub(super) fn primary_sum_of(details: &HeroDetails, core: &Ruleset) -> u16 {
    let [attack, defense, power, knowledge] = h3_heroes::primary_skills(details, core);
    counted(attack, 0) + counted(defense, 0) + counted(power, 1) + counted(knowledge, 1)
}

/// One primary skill as `ai_next_hero` 0x526A90 counts it: its signed byte above 99 as 99,
/// above 0 as itself, else `floor` (0 for Attack and Defense, 1 for Spell Power and Knowledge).
pub(super) fn counted(value: u8, floor: u16) -> u16 {
    match i8::from_ne_bytes([value]) {
        100.. => 99,
        skill @ 1..=99 => skill.unsigned_abs().into(),
        _ => floor,
    }
}

/// `hero`'s turn (`ai_hero_turn` 0x5261F0; survey §1.3; CPU-TRN-004), `alone` its alone flag:
/// first a visit of the town on whose entrance it stands (`ai_town_visit` 0x5253D0, increment 3:
/// `AI-STUB-4`) and the patrol of radius 0 that never moves (`AI-STUB-5`); then with points left
/// (every hero picked has them until the visit and the walk spend points) the selection, the
/// danger map, the boat marks and the search and the walk ([`search_and_walk`]); last
/// [`garrison`]'s step.
// rule: CPU-TRN-004
fn hero_turn(
    state: &GameState,
    rules: &GameRuleset,
    player: PlayerId,
    (turn, plans): (&mut Turn, &mut Plans),
    (hero, alone): (&HeroOnMap, bool),
    rng: &mut Rng,
) -> Option<GameCommand> {
    // stub: AI-STUB-4
    // stub: AI-STUB-5
    search_and_walk(state, rules, (turn, plans), (hero, alone), rng);
    garrison(state, rules, player, turn, hero)
}

/// A hero's danger map and its search and walk (`ai_hero_turn` 0x5261F0 at 0x5263B1 and
/// 0x526571; survey §1.3, §1.4): the danger map (`plan::danger_map`), on every hero's turn (the
/// original's only for a hero newly selected: a hero is not picked twice in a turn until the walk,
/// T2j), the boat marks (`AI-STUB-2`) and the planning (`plan::search_and_walk`, on the hero's
/// values' reads: the turn start's record and weights, its caster record); then the hero is done,
/// whatever the planning gives (the walk, T2j: `AI-STUB-2`).
fn search_and_walk(
    state: &GameState,
    rules: &GameRuleset,
    (turn, plans): (&mut Turn, &mut Plans),
    (hero, alone): (&HeroOnMap, bool),
    rng: &mut Rng,
) {
    state.hero(hero.id).into_iter().for_each(|details| {
        let fighter = Fighter {
            on_map: hero,
            details,
        };
        let danger = plan::danger_map(state, rules, fighter);
        // stub: AI-STUB-2
        let record = &turn.record;
        let weights = record.weights(rules);
        let caster = spells::record(state, rules, fighter);
        let valued = Valued::with(state, rules, (record, &weights), (fighter, caster));
        let planner = Planner::new((state, rules), &valued, (record, &danger, turn.flag));
        let _ = plan::search_and_walk(&planner, plans, alone, rng);
    });
    turn.sleep(hero.id);
}

/// The garrison step at the end of `hero`'s turn (`ai_hero_turn` 0x52667F; survey §1.3 step 7):
/// a hero without points or done (every hero is done until the walk, T2j) on an own town's
/// entrance goes into the garrison when no hero holds it (`TownHeroChange::Enter`: the original's
/// `town_visitor_to_garrison` 0x4B9FC0 does nothing when the armies do not merge, as the engine
/// refuses); with a garrison hero the two change places when that hero has points and is not
/// done, or carries creatures worth less than the visitor's army (`army_value` 0x44AC80,
/// `h3_adventure::army_value`; `TownHeroChange::Swap`). No step where the engine refuses it.
fn garrison(
    state: &GameState,
    rules: &GameRuleset,
    player: PlayerId,
    turn: &Turn,
    hero: &HeroOnMap,
) -> Option<GameCommand> {
    let adventure = state.adventure();
    let town = adventure
        .map()
        .town_at(hero.pos)
        .filter(|&town| owner(state, town) == Some(player))?;
    let (change, index) = match adventure.town_hero(town) {
        None => (TownHeroChange::Enter, 0),
        Some(held) => {
            let awake = held.movement > 0 && !turn.is_asleep(held.id);
            let core = rules.core();
            let weaker = carries_creatures(&held.army)
                && army_value(core, &held.army) < army_value(core, &hero.army);
            if !(awake || weaker) {
                return None;
            }
            (TownHeroChange::Swap, 2)
        }
    };
    let legal = adventure.town_hero_moves(rules.adventure(), player, (town, most_heroes(rules)));
    legal[index].then(|| heroes(town, change))
}

/// Whether an army holds a creature (`army_creature_count` 0x44ADA0 above 0).
fn carries_creatures(army: &[ArmyStack]) -> bool {
    army.iter().any(|stack| stack.count > 0)
}

/// The hero move `change` at `town` (TWN-ROW-002 to -004).
const fn heroes(town: TownId, change: TownHeroChange) -> GameCommand {
    GameCommand::Town(TownCommand::Heroes { town, change })
}

/// The town part's most heroes on the map (TWN-ROW-003), the bound of the engine's moves.
fn most_heroes(rules: &GameRuleset) -> u8 {
    rules
        .towns()
        .params()
        .map_or(0, |params| params.most_heroes)
}

/// The owner of `town`, none for a neutral town.
fn owner(state: &GameState, town: TownId) -> Option<PlayerId> {
    let adventure = state.adventure();
    adventure
        .map()
        .town_object(town)
        .and_then(|index| adventure.owner(index))
}

/// `player`'s towns in `TownId` order: the engine's order of a player's towns (its upper bars',
/// `AdventureState::upper_bars`), for the original's town list (player +0x40).
pub(super) fn towns_of(
    state: &GameState,
    player: PlayerId,
) -> impl Iterator<Item = TownId> + use<'_> {
    let count = state.adventure().map().towns().len();
    (0..=u16::MAX)
        .take(count)
        .map(TownId::new)
        .filter(move |&town| owner(state, town) == Some(player))
}
