//! The mage guilds (E09 increment 5; design note `E09-design.md` §84 to §89): at the game's build
//! the setup's town spell sets checked (TWN-SET-012), then every town's guild drawn from the
//! towns' stream and its Aurora set computed (TWN-GLD-002, TWN-GLD-004's set), each with the
//! town's required and banned spells and the scenario's bans (GAM-SET-003; E09 T47a2); since E09
//! T48a the town's heroes learning its spells at the town's triggers ([`learn`], TWN-GLD-005).

use h3_content::{SpellCategory, SpellId};
use h3_contracts::TownId;
use h3_heroes::HeroChange;
use h3_kernel::Rng;
use h3_towns::{GuildSets, GuildSlots, SpellSet, Town, TownStatic};

use crate::command::Sink;
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
/// ([`super::rows::heroes`]) and the start's main towns ([`at_start`], TD-10); the visit's end, a
/// capture and a siege's end are T48b's, the opening and the book T49's.
// rule: TWN-GLD-005
pub(crate) fn learn<S: Sink>(
    state: &mut GameState,
    rules: &GameRuleset,
    town: TownId,
    sink: &mut S,
) {
    let adventure = state.adventure();
    let visitor = adventure
        .town_visitor(town)
        .and_then(|hero| adventure.hero(hero));
    let heroes = [visitor, adventure.town_hero(town)].map(|on| on.map(|on| (on.id, on.owner)));
    for (hero, owner) in heroes.into_iter().flatten() {
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
