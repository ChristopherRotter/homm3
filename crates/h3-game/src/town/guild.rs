//! The mage guilds (E09 increment 5; design note `E09-design.md` §84 to §89): at the game's build
//! the setup's town spell sets checked (TWN-SET-012), then every town's guild drawn from the
//! towns' stream and its Aurora set computed (TWN-GLD-002, TWN-GLD-004's set), each with the
//! town's required and banned spells and the scenario's bans (GAM-SET-003; E09 T47a2); since E09
//! T48a the town's heroes learning its spells at the town's triggers ([`learn`], TWN-GLD-005);
//! since E09 T49a1 the spell points a town gives its heroes: the night's refill ([`refilled`],
//! TWN-GLD-008) and the Mana Vortex once a week a town ([`vortex`], [`visit`], TWN-GLD-009);
//! since E09 T49a2 the town screen's opening ([`open`], listed by [`opens`], TWN-GLD-006).

use alloc::vec::Vec;

use h3_content::{SpellCategory, SpellId};
use h3_contracts::{HeroId, PlayerId, TownId};
use h3_heroes::HeroChange;
use h3_kernel::Rng;
use h3_towns::{GuildSets, GuildSlots, SpellSet, Town, TownCommandError, TownStatic};

use super::Told;
use crate::command::{GameCommandError, Sink};
use crate::{GameRuleset, GameSetup, GameSetupError, GameState};

/// The setup's first town spell that is no hero spell of the ruleset (TWN-SET-012; design §84): a
/// spell the ruleset lacks or a creature ability, in `TownId` order, each town's required spells
/// before its banned ones; `None` when every one is a hero spell (one granted by name only, Titan's
/// Lightning Bolt, is allowed: no guild draws it, TWN-GLD-002).
// rule: TWN-SET-012
pub(crate) fn town_spell_refused(setup: &GameSetup, rules: &GameRuleset) -> Option<GameSetupError> {
    let hero_spell = |spell: SpellId| {
        rules
            .core()
            .spell(spell)
            .is_some_and(|s| s.category != SpellCategory::Ability)
    };
    setup
        .towns
        .iter()
        .zip((0..=u16::MAX).map(TownId::new))
        .find_map(|(town, id)| {
            town.required_spells
                .iter()
                .chain(&town.banned_spells)
                .find(|&&spell| !hero_spell(spell))
                .map(|&spell| GameSetupError::TownSpell(id, spell))
        })
}

/// Every town's guild and Aurora set at the build (TWN-GLD-002, TWN-GLD-004's set; design §85,
/// §86): in `TownId` order, after the towns' records are built, each town's slots drawn by
/// `h3_towns::draw_guild` from `draws` (the towns' stream, E09 decision 179: its first draws) with
/// its setup's required and banned spells and the scenario's bans, and its Aurora set by
/// `h3_towns::aurora_set` with the same sets (no draw). `towns` are the built towns by `TownId`,
/// one per setup town (the adventure refused a setup without, `SetupError::Garrisons`); a town
/// whose faction has no record (never after the setup's checks) keeps its empty slots.
// rule: TWN-GLD-002
// rule: TWN-GLD-004
pub(crate) fn draw_guilds(
    setup: &GameSetup,
    rules: &GameRuleset,
    towns: &mut [(Town, TownStatic)],
    draws: &mut Rng,
) {
    let scenario = SpellSet::of(&setup.banned_spells);
    for ((town, town_static), town_setup) in towns.iter_mut().zip(&setup.towns) {
        let sets = GuildSets {
            required: SpellSet::of(&town_setup.required_spells),
            banned: SpellSet::of(&town_setup.banned_spells),
            scenario,
        };
        let part = rules.towns();
        let slots = part
            .town(town.faction())
            .zip(part.params())
            .map(|(record, params)| {
                h3_towns::draw_guild(record, params, rules.core(), &sets, draws)
            });
        *town = town.with_guild(slots.unwrap_or(GuildSlots::EMPTY));
        town_static.aurora = h3_towns::aurora_set(rules.core(), &sets);
    }
}

/// The town's heroes learn its guild's spells (TWN-GLD-005; design §89; survey `E09-guild.md`
/// §11, §12: the original's `town_heroes_learn_spells` 0x5BE030 with 0): its visitor, then its
/// garrison hero, each that stands there, learns [`h3_towns::teaches`]' spells in their order,
/// each applied as `HeroChange::LearnSpell` (HER-CHG-001) and told to the hero's owner
/// (`HeroChanged`). No owner is tested (an allied visitor learns) and nothing is drawn; nothing for
/// a town the store or the map lacks. Called at the town's own triggers (E09 T48a): every build,
/// a command's and a town event's ([`super::command::build_told`]), a town's hire
/// ([`super::hire::hire`]), a move between the bars that leaves a visitor
/// ([`super::rows::heroes`]) and the start's main towns ([`at_start`], TD-10); since E09 T48b the
/// visit's end (a step or a `Revisit` entering a town of the team, an undefended capture: the
/// outcome's `town_entered`, after the capture, in `GameState::settle_outcome`) and a siege's end
/// whoever won (in `GameState::end_battle`, before the winner's levels); since E09 T49a1 the
/// visit's end through [`visit`], after the Vortex; since E09 T49a2 the opening ([`open`]); the
/// book is T49b's.
// rule: TWN-GLD-005
pub(crate) fn learn<S: Sink>(
    state: &mut GameState,
    rules: &GameRuleset,
    town: TownId,
    sink: &mut S,
) {
    let [visitor, garrison] = heroes(state, town);
    for (hero, owner) in [visitor, garrison].into_iter().flatten() {
        let towns = state.towns();
        let spells = towns
            .get(town)
            .zip(towns.town_static(town))
            .zip(state.hero(hero))
            .map(|((record, statics), details)| {
                h3_towns::teaches(record, statics, rules.towns(), rules.core(), details)
            })
            .unwrap_or_default();
        for spell in spells {
            let change = HeroChange::LearnSpell(spell);
            state.change_hero(rules.core(), hero, Some(owner), change, sink);
        }
    }
}

/// The start's learning (TWN-GLD-005, TWN-SET-009 amended; TD-10, the original's: design §89,
/// survey `E09-guild.md` §10, §12, the original's `main_town_heroes` 0x4CA040 at 0x4CA1A2): at the
/// game's build, once its heroes stand and its guilds are drawn, for each player in id order (the
/// colours' order) whose main town generates its starting hero, that town's heroes [`learn`] (the
/// new hero, its visitor; no garrison hero stands at the start). No event: the build tells nothing
/// (a client reads the start's state). A placed hero on another town's entrance learns nothing
/// here (survey §12), but at its first trigger.
// rule: TWN-GLD-005, TWN-SET-009
pub(crate) fn at_start(state: &mut GameState, rules: &GameRuleset, setup: &GameSetup) {
    let towns = setup
        .players
        .iter()
        .filter_map(|player| player.start.main_town)
        .filter(|main| main.generates_hero);
    for main in towns {
        learn(state, rules, main.town, &mut crate::command::NoopSink);
    }
}

/// The town's visitor, then its garrison hero, each with its owner, where one stands (the
/// adventure's records: ADR-0028 point 2).
pub(crate) fn heroes(state: &GameState, town: TownId) -> [Option<(HeroId, PlayerId)>; 2] {
    let adventure = state.adventure();
    let visitor = adventure
        .town_visitor(town)
        .and_then(|hero| adventure.hero(hero));
    [visitor, adventure.town_hero(town)].map(|on| on.map(|on| (on.id, on.owner)))
}

/// The town's visit at its end (TWN-GLD-005, TWN-GLD-009; design §89, §90; survey `E09-guild.md`
/// §12, §15: the original's town screen opened by `town_visit` 0x4AAFD0, its
/// `town_screen_init` 0x5C63C0 visiting the buildings, then the visit's learning): the Mana
/// Vortex to the garrison hero, then the visitor ([`vortex`]), then their learning ([`learn`]).
/// Called at the visit's end (a step or a `Revisit` entering a town of the team, an undefended
/// capture: `GameState::settle_outcome`'s `town_entered`, after the capture's effects) and by the
/// opening ([`open`], E09 T49a2). Not at a siege's end, which teaches alone.
// rule: TWN-GLD-005, TWN-GLD-009
pub(crate) fn visit<S: Sink>(
    state: &mut GameState,
    rules: &GameRuleset,
    town: TownId,
    sink: &mut S,
) {
    vortex(state, rules, town, sink);
    learn(state, rules, town, sink);
}

/// Opens `player`'s `town`'s screen (TWN-GLD-006; E09 T49a2, design §90; decisions 170, 175, 178;
/// survey `E09-guild.md` §12, §15: the original's `town_screen_open` 0x5BE210 from the town list,
/// its `town_screen_init` 0x5C63C0 visiting the buildings, then the screen's bars and the guild
/// window teaching), the turn checked by the caller ([`GameState::check_ready`]: the player's turn,
/// no decision open, no level-up held), in this order, each refusal changing nothing:
///
/// 1. the town in the store of towns, else [`TownCommandError::UnknownTown`];
/// 2. its owner, the adventure's owner of its object, is `player`, else
///    [`TownCommandError::NotOwner`] (TD-11's rule, E09 decision 42: only the owner opens its
///    town screen; an ally's visitor of the town still takes what the owner's opening gives).
///
/// Then [`visit`]: the Mana Vortex to the garrison hero, then the visitor, its cleared flag told
/// to the town's owner (TWN-GLD-009), then the visitor's, then the garrison hero's learning
/// (TWN-GLD-005; decision 178: with or without a visitor), each `HeroChanged` to the hero's owner.
/// No other refusal: an opening that changes nothing is accepted and changes nothing (no event,
/// the state's hash unchanged). Nothing is drawn; no movement point is spent.
///
/// # Errors
///
/// [`GameCommandError::Town`] with [`TownCommandError::UnknownTown`] or
/// [`TownCommandError::NotOwner`].
// rule: TWN-GLD-006
pub(crate) fn open<S: Sink>(
    state: &mut GameState,
    rules: &GameRuleset,
    player: PlayerId,
    town: TownId,
    sink: &mut S,
) -> Result<(), GameCommandError> {
    if state.towns().get(town).is_none() {
        return Err(GameCommandError::Town(TownCommandError::UnknownTown));
    }
    if super::command::owner(state, town) != Some(player) {
        return Err(GameCommandError::Town(TownCommandError::NotOwner));
    }
    visit(state, rules, town, sink);
    Ok(())
}

/// Whether [`open`] of `town`, one of the lister's own, would change something, so that
/// [`crate::legal_commands`] lists it (TWN-GLD-006; design §90, decision 175: never an idle
/// opening, which would flood the env's plans): its garrison hero or its visitor would take the
/// Mana Vortex ([`h3_towns::vortex`]: a standing Vortex, the week's flag set, the hero below the
/// factor times its maximum), or would learn a spell ([`h3_towns::teaches`] not empty): one
/// question to the town's part for both heroes ([`h3_towns::opening_gives`]). `visitor`
/// is the hero on its entrance, the listing's (`AdventureState::town_entrance_hero`, found once a
/// town for the hires, the opening and the moves). False for a town no hero stands in, the
/// cheapest test first (the heroes, then the town's record).
// rule: TWN-GLD-006
pub(crate) fn opens(
    state: &GameState,
    rules: &GameRuleset,
    town: TownId,
    visitor: Option<HeroId>,
) -> bool {
    let adventure = state.adventure();
    let garrison = adventure.town_hero(town).map(|on| on.id);
    if garrison.is_none() && visitor.is_none() {
        return false;
    }
    let towns = state.towns();
    let (Some(record), Some(statics)) = (towns.get(town), towns.town_static(town)) else {
        return false;
    };
    let heroes = garrison
        .into_iter()
        .chain(visitor)
        .filter_map(|hero| state.hero(hero));
    h3_towns::opening_gives(record, statics, rules.towns(), rules.core(), heroes)
}

/// The Mana Vortex of `town` to its garrison hero, then its visitor ([`vortex_to`]): the visit's
/// end ([`visit`]) and every build after its learning ([`super::command::build_told`]).
// rule: TWN-GLD-009
pub(crate) fn vortex<S: Sink>(
    state: &mut GameState,
    rules: &GameRuleset,
    town: TownId,
    sink: &mut S,
) {
    let [visitor, garrison] = heroes(state, town);
    vortex_to(state, rules, town, [garrison, visitor], sink);
}

/// The Mana Vortex of `town` given to the first of `heroes`, in order, whose points it raises
/// (TWN-GLD-009; design §93; survey `E09-guild.md` §15: the original's `town_visit_buildings`
/// 0x5BD8E0, its first branch): [`h3_towns::vortex`]'s points set (`HeroChange::SetSpellPoints`,
/// HER-CHG-001; `HeroChanged` to the hero's owner), the town's week flag cleared
/// ([`h3_towns::Towns::take_vortex`]), then the views it changed told to their viewers ([`Told`]:
/// the owner's flag, TWN-VEW-004), so the rest gets nothing until the week's start sets the flag
/// again (`Towns::start_week`, GAM-CAL-002). No owner is tested (an allied visitor takes it) and
/// nothing is drawn; nothing without a hero it raises. Its callers: the visit's end and every
/// build after its learning, a command's and a town event's (the garrison hero, then the visitor:
/// [`vortex`]), and a town's hire (the hired hero, [`super::hire::hire`]); never a turn's start,
/// the night or a siege's end.
// rule: TWN-GLD-009
pub(crate) fn vortex_to<S: Sink>(
    state: &mut GameState,
    rules: &GameRuleset,
    town: TownId,
    heroes: [Option<(HeroId, PlayerId)>; 2],
    sink: &mut S,
) {
    let (core, part) = (rules.core(), rules.towns());
    let Some(record) = state.towns().get(town).filter(|record| record.vortex()) else {
        return;
    };
    let taken = heroes.into_iter().flatten().find_map(|(hero, owner)| {
        state
            .hero(hero)
            .and_then(|details| h3_towns::vortex(record, part, core, details))
            .map(|points| (hero, owner, points))
    });
    let Some((hero, owner, points)) = taken else {
        return;
    };
    let told = Told::before::<S>(state, rules, &[town], &[]);
    let change = HeroChange::SetSpellPoints(points);
    state.change_hero(core, hero, Some(owner), change, sink);
    state.towns_mut().take_vortex(town);
    told.tell(state, rules, sink);
}

/// The heroes the towns' night refills (TWN-GLD-008, GAM-CAL-002 amended; design §92; survey
/// `E09-guild.md` §14: the original's `new_day` 0x4C82E4 to 0x4C8434, after every hero's daily
/// regain): each town in `TownId` order whose night refills ([`h3_towns::refills`]: a building of
/// role `guild` ever built there), its visitor, then its garrison hero, where one stands; no spell
/// book, no owner is tested. Read by the day's regain (`GameState::regain_mana`), which gives each
/// its points or its maximum, whichever is more, in place of the regain (the regain never passes
/// the maximum nor lowers the points, so the night's result is the same, the maximum computed
/// once: `bench-check`'s day-start workloads); silent, as the regain.
// rule: TWN-GLD-008
pub(crate) fn refilled(state: &GameState, rules: &GameRuleset) -> Vec<HeroId> {
    let adventure = state.adventure();
    let towns = state.towns();
    let mut heroes = Vec::new();
    for town in (0..=u16::MAX).take(towns.len()).map(TownId::new) {
        let visitor = adventure.town_visitor(town);
        let garrison = adventure.town_hero(town).map(|on| on.id);
        if visitor.is_none() && garrison.is_none() {
            continue;
        }
        if towns
            .get(town)
            .is_some_and(|record| h3_towns::refills(record, rules.towns()))
        {
            heroes.extend(visitor.into_iter().chain(garrison));
        }
    }
    heroes
}
