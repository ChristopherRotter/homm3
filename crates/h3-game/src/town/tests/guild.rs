//! Tests of the mage guilds at the game's build (E09 T47a2, design note §84 to §86): the setup's
//! town spell sets checked (TWN-SET-012), every town's guild drawn from the towns' stream in
//! `TownId` order with its required and banned spells and the scenario's bans (TWN-GLD-002), its
//! Aurora set computed (TWN-GLD-004's set), what it shows read from the town's record
//! (TWN-GLD-003).

use alloc::vec;
use alloc::vec::Vec;

use h3_content::{ContentPack, Ruleset, SpellCategory, SpellDef, SpellId};
use h3_contracts::TownId;
use h3_kernel::{Rng, StreamId};
use h3_towns::{GuildSets, SpellSet, TownPack};

use super::{faction, refused, rules_of, town_pack, towns_core_pack, towns_setup};
use crate::tests::key;
use crate::town::Draws;
use crate::{GameRuleset, GameSetupError, GameState};

const T0: TownId = TownId::new(0);
const T1: TownId = TownId::new(1);

/// The town tests' rules with the core's spells `t:gust` (level 1, 0), `t:gate` (level 3, 1),
/// `t:spark` (level 1, 2), the creature ability `t:breath` (3) and `t:bolt` (level 1, granted by
/// name only, 4), the three new ones adventure spells (the battle part needs no record of them);
/// `t:town`'s guild weights gust 3, spark 1, gate 2, bolt 5.
fn guild_rules() -> GameRuleset {
    let (core, towns) = guild_packs();
    rules_of(
        Ruleset::compile(&[core], "t:faithful").unwrap(),
        &towns,
        false,
        4,
    )
}

/// The core pack and the town part of [`guild_rules`] (E09 T47b: the view's tests add the guild's
/// buildings).
pub(super) fn guild_packs() -> (ContentPack, TownPack) {
    let mut core = towns_core_pack(false);
    let spell = |k: &str, category, level, by_name| SpellDef {
        key: key(k),
        category,
        level,
        granted_by_name_only: by_name,
        ..core.spells[0].clone()
    };
    let extra = [
        spell("t:spark", SpellCategory::Adventure, 1, false),
        SpellDef {
            schools: vec![],
            ..spell("t:breath", SpellCategory::Ability, 1, false)
        },
        spell("t:bolt", SpellCategory::Adventure, 1, true),
    ];
    core.spells.extend(extra);
    let mut towns = town_pack(false);
    towns.towns[0].guild = ["t:gust", "t:spark", "t:gate", "t:bolt"]
        .into_iter()
        .zip([3, 1, 2, 5])
        .map(|(k, weight)| (key(k), weight))
        .collect();
    (core, towns)
}

/// The rules' spell of key `k`.
fn spell(rules: &GameRuleset, k: &str) -> SpellId {
    rules.core().spell_by_key(k).unwrap()
}

/// The spells of `town`'s guild at `level`, in slot order (every slot, not what it shows).
fn slots(game: &GameState, town: TownId, level: u8) -> Vec<SpellId> {
    let record = game.towns().get(town).unwrap();
    record.guild().level(level).flatten().collect()
}

// covers: TWN-GLD-002, TWN-GLD-003, TWN-GLD-004, TWN-SET-012
#[test]
fn every_town_s_guild_is_drawn_at_the_build_from_the_towns_stream() {
    let rules = guild_rules();
    let [gust, gate, spark, bolt] =
        ["t:gust", "t:gate", "t:spark", "t:bolt"].map(|k| spell(&rules, k));
    let mut setup = towns_setup(&rules);
    // Town 0 requires spark and bolt (granted by name only: never drawn); town 1 bans gust.
    setup.towns[0].required_spells = vec![spark, bolt];
    setup.towns[1].banned_spells = vec![gust];
    let mut game = GameState::new(&rules, &setup, 9).unwrap();
    // Town 0: spark first without a draw, then gust (the only weight left) at level 1; gate at 3.
    assert_eq!(slots(&game, T0, 1), [spark, gust]);
    assert_eq!(slots(&game, T0, 3), [gate]);
    assert!(
        [2, 4, 5]
            .iter()
            .all(|&level| slots(&game, T0, level).is_empty())
    );
    // Town 1 (random, aligned to blue: `t:town` too): spark, the gust banned; gate.
    assert_eq!(
        (slots(&game, T1, 1), slots(&game, T1, 3)),
        (vec![spark], vec![gate])
    );
    // The towns' stream: the guilds' draws in `TownId` order, its first ones.
    let part = rules.towns();
    let (record, params) = part
        .town(faction(&rules, "t:town"))
        .zip(part.params())
        .unwrap();
    let mut stream = Rng::for_stream(9, StreamId::TOWNS);
    let sets = |required: &[SpellId], banned: &[SpellId]| GuildSets {
        required: SpellSet::of(required),
        banned: SpellSet::of(banned),
        scenario: SpellSet::EMPTY,
    };
    for (town, sets) in [(T0, sets(&[spark, bolt], &[])), (T1, sets(&[], &[gust]))] {
        let drawn = h3_towns::draw_guild(record, params, rules.core(), &sets, &mut stream);
        assert_eq!(game.towns().get(town).unwrap().guild(), &drawn);
    }
    assert_eq!(game.town_draws(Draws::Towns).2, &stream);
    // The Aurora sets: every hero spell but those granted by name only and the town's bans.
    let aurora = |town| -> Vec<SpellId> {
        game.towns()
            .town_static(town)
            .unwrap()
            .aurora
            .iter()
            .collect()
    };
    assert_eq!(
        (aurora(T0), aurora(T1)),
        (vec![gust, gate, spark], vec![gate, spark])
    );
    // No Mage Guild stands in the test towns: they show nothing; every flag set from the build.
    let town = game.towns().get(T0).unwrap();
    assert_eq!(h3_towns::guild_level(town, part), 0);
    assert_eq!(h3_towns::shown(town.guild(), town, part, 1).count(), 0);
    assert!(town.vortex());
    // Two games of a seed draw alike; a fork shares its parent's guilds.
    let again = GameState::new(&rules, &setup, 9).unwrap();
    assert_eq!(again.state_hash(), game.state_hash());
    let fork = game.clone();
    assert_eq!(
        fork.towns().get(T1).unwrap().guild(),
        game.towns().get(T1).unwrap().guild()
    );
}

// covers: TWN-GLD-002, TWN-GLD-004, GAM-SET-003
#[test]
fn the_scenario_s_bans_leave_every_guild_and_the_aurora_set_by_the_quirk() {
    let rules = guild_rules();
    let [gust, gate, spark] = ["t:gust", "t:gate", "t:spark"].map(|k| spell(&rules, k));
    let mut setup = towns_setup(&rules);
    // The scenario bans gate; town 0 requires it: never drawn (as a town's own ban, TD-25).
    setup.banned_spells = vec![gate];
    setup.towns[0].required_spells = vec![gate];
    let game = GameState::new(&rules, &setup, 3).unwrap();
    for town in [T0, T1] {
        assert!(slots(&game, town, 3).is_empty());
        let mut level_1 = slots(&game, town, 1);
        level_1.sort_unstable();
        assert_eq!(level_1, [gust, spark]);
        let sets = GuildSets {
            required: SpellSet::of(&setup.towns[town.index()].required_spells),
            banned: SpellSet::EMPTY,
            scenario: SpellSet::of(&[gate]),
        };
        let aurora = game.towns().town_static(town).unwrap().aurora;
        assert_eq!(aurora, h3_towns::aurora_set(rules.core(), &sets));
        // `TWN-QUIRK-14`: the scenario's ban in the set only with the flag on.
        let flag = rules
            .core()
            .quirks()
            .is_enabled(h3_content::AURORA_IGNORES_MAP_BANS);
        assert_eq!(aurora.contains(gate), flag);
    }
}

// covers: TWN-SET-012
#[test]
fn a_town_spell_is_a_hero_spell_of_the_ruleset() {
    let rules = guild_rules();
    let breath = spell(&rules, "t:breath");
    let unknown = SpellId::from_raw(u16::MAX);
    // A creature ability or a spell the ruleset lacks; a town's required spells before its
    // banned ones, the towns in `TownId` order.
    let error = refused(
        &rules,
        "town 1's spell 3 is not a hero spell of the ruleset",
        |s| {
            s.towns[1].banned_spells = vec![unknown];
            s.towns[1].required_spells = vec![spell(&rules, "t:gust"), breath];
        },
    );
    assert_eq!(error, GameSetupError::TownSpell(T1, breath));
    assert_eq!(
        refused(
            &rules,
            "town 0's spell 65535 is not a hero spell of the ruleset",
            |s| {
                s.towns[0].banned_spells = vec![unknown];
                s.towns[1].required_spells = vec![breath];
            }
        ),
        GameSetupError::TownSpell(T0, unknown)
    );
    // After the scenario's bans; a spell granted by name only is allowed.
    assert_eq!(
        refused(&rules, "banned spell 65535 is not the ruleset's", |s| {
            s.banned_spells = vec![unknown];
            s.towns[0].required_spells = vec![unknown];
        }),
        GameSetupError::BannedSpell(unknown)
    );
    let mut setup = towns_setup(&rules);
    let bolt = spell(&rules, "t:bolt");
    setup.towns[0].required_spells = vec![bolt];
    setup.towns[1].banned_spells = vec![bolt];
    assert_eq!(setup.validate(&rules), Ok(()));
}
