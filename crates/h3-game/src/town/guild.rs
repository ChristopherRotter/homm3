//! The mage guilds at the game's build (E09 T47a2; design note `E09-design.md` §84 to §86): the
//! setup's town spell sets checked (TWN-SET-012), then every town's guild drawn from the towns'
//! stream and its Aurora set computed (TWN-GLD-002, TWN-GLD-004's set), each with the town's
//! required and banned spells and the scenario's bans (GAM-SET-003).

use h3_content::{SpellCategory, SpellId};
use h3_contracts::TownId;
use h3_kernel::Rng;
use h3_towns::{GuildSets, GuildSlots, SpellSet, Town, TownStatic};

use crate::{GameRuleset, GameSetup, GameSetupError};

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
