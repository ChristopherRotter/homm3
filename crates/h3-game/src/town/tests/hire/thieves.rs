//! Tests of the Thieves' Guild (E09 T39b1, design note `E09-design.md` §73; TWN-TAV-007 to -009,
//! TD-23, `TWN-QUIRK-13`): the query in its viewer's turn by its towns' Taverns (or at a map
//! Tavern), the rows each level shows as places, each row's value, the best hero, its skills and
//! the best creature. On the hire tests' game: red's hero 0 on (0, 0) with 9 imps, blue's hero 1 on
//! (2, 2), red's T0 on (1, 1) and blue's T1 on (3, 0), each with a Tavern; the guild's first
//! resource row the core's one resource (gold), the imp worth 50 and the gog 100.

use h3_adventure::AdventureCommand;
use h3_content::ArtifactTypeId;
use h3_contracts::HeroId;
use h3_towns::TownHeroChange;

use super::super::town_object;
use super::*;
use crate::town::thieves::{places, thieves_guild_at, value};
use crate::{
    DEN_LEVEL, GUILD_LEVELS, GuildRanking, GuildRow, HeroArmy, HeroArtifactsSetup,
    HeroDetailsSetup, thieves_guild,
};

/// The core's edit: the profile `t:guild` with `TWN-QUIRK-13`'s flag on, the imp's and the gog's
/// AI values 50 and 100 (`valued`, else every creature worth 0), the primary skills to 120.
fn guild_core(core: &mut ContentPack, valued: bool) {
    core.profiles.push(ProfileDef {
        key: key("t:guild"),
        quirks: vec![("quirk.town.guild_artifacts_by_backpacks".to_string(), true)],
    });
    if valued {
        core.creatures[0].info.ai_value = 50;
        core.creatures[1].info.ai_value = 100;
    }
    if let Some(params) = &mut core.hero_params {
        params.primary_max = 120;
    }
}

/// The guild tests' rules under `profile` (`t:faithful`, no flag on, or `t:guild`).
fn guild_rules(profile: &str) -> GameRuleset {
    guild_rules_valued(profile, true)
}

fn guild_rules_valued(profile: &str, valued: bool) -> GameRuleset {
    guild_rules_with(profile, valued, 0)
}

/// The guild tests' rules with the core's gold in the guild's resource row `row` (0 to 2; 3: in
/// none).
fn guild_rules_with(profile: &str, valued: bool, row: usize) -> GameRuleset {
    rules_in_core(
        (profile, false),
        |core| guild_core(core, valued),
        |towns| {
            let mut rows: [Vec<_>; 3] = Default::default();
            rows.get_mut(row)
                .into_iter()
                .for_each(|r| r.push(key("t:gold")));
            towns.params.as_mut().unwrap().guild_resources = rows;
        },
    )
}

fn creature(rules: &GameRuleset, k: &str) -> h3_content::CreatureTypeId {
    rules.core().creature_by_key(k).unwrap()
}

fn artifact(rules: &GameRuleset, k: &str) -> ArtifactTypeId {
    rules.core().artifact_by_key(k).unwrap()
}

fn stack(slot: u8, creature: h3_content::CreatureTypeId, count: u32) -> ArmyStack {
    ArmyStack {
        slot,
        creature,
        count,
    }
}

/// [`towns_setup`] with red's town without a standing Tavern.
fn bare(rules: &GameRuleset) -> GameSetup {
    let mut setup = towns_setup(rules);
    setup.towns[0].buildings = BuildingsSetup::Custom {
        built: SlotSet::EMPTY,
        forbidden: SlotSet::EMPTY,
    };
    setup
}

/// [`towns_setup`] with `more` towns of red's beside T0, each with a Tavern.
fn red_towns(rules: &GameRuleset, more: usize) -> GameSetup {
    let mut setup = towns_setup(rules);
    for &at in &[(0, 3), (1, 3), (2, 3), (3, 2), (3, 1)][..more] {
        setup.towns.push(setup.towns[0].clone());
        setup.map.objects.push(town_object(rules, at, Some(0)));
    }
    setup
}

// covers: TWN-TAV-007
#[test]
fn the_guild_answers_in_its_viewers_turn_by_its_towns_taverns() {
    let rules = guild_rules("t:faithful");
    let mut game = game(&rules, &towns_setup(&rules), 5000);
    let guild = thieves_guild(&game, &rules, RED).unwrap();
    assert_eq!(guild.level, 1);
    assert_eq!(guild, thieves_guild_at(&game, &rules, 1));
    // Outside the viewer's turn, and for a player outside the game, none.
    assert_eq!(thieves_guild(&game, &rules, BLUE), None);
    assert_eq!(thieves_guild(&game, &rules, PlayerId::new(5)), None);
    run(&rules, &mut game, RED, GameCommand::EndTurn).unwrap();
    assert_eq!(thieves_guild(&game, &rules, RED), None);
    assert_eq!(thieves_guild(&game, &rules, BLUE).map(|g| g.level), Some(1));
    // Without a standing Tavern, none.
    let bare = self::game(&rules, &bare(&rules), 5000);
    assert_eq!(thieves_guild(&bare, &rules, RED), None);
    // Each own town with a Tavern counts, to five.
    for (more, level) in [(1, 2), (3, 4), (4, 5), (5, GUILD_LEVELS)] {
        let game = self::game(&rules, &red_towns(&rules, more), 5000);
        assert_eq!(
            thieves_guild(&game, &rules, RED).map(|g| g.level),
            Some(level)
        );
    }
}

// covers: TWN-TAV-007
#[test]
fn a_map_taverns_decision_opens_the_guild_at_its_owners_taverns() {
    let rules = guild_rules("t:faithful");
    // Without a town's Tavern, level 0 while the map Tavern's decision is open.
    let mut setup = map::inn_setup(&rules);
    setup.towns[0].buildings = BuildingsSetup::Custom {
        built: SlotSet::EMPTY,
        forbidden: SlotSet::EMPTY,
    };
    let (mut game, _) = map::visited(&rules, &setup, 0);
    assert!(game.tavern(RED).is_some());
    let guild = thieves_guild(&game, &rules, RED).unwrap();
    assert_eq!((guild.level, guild.rows.len()), (0, 2));
    assert_eq!(guild.players[0].hero, None);
    let close = GameCommand::Answer(crate::Answer::Tavern(crate::TavernAnswer::Close));
    run(&rules, &mut game, RED, close).unwrap();
    assert_eq!(thieves_guild(&game, &rules, RED), None);
    // With T0's, its level.
    let (game, _) = map::visited(&rules, &map::inn_setup(&rules), 0);
    assert_eq!(thieves_guild(&game, &rules, RED).map(|g| g.level), Some(1));
}

// covers: TWN-TAV-008, TWN-TAV-009
#[test]
fn each_level_shows_its_rows_and_parts() {
    let rules = guild_rules("t:faithful");
    let game = game(&rules, &towns_setup(&rules), 5000);
    let rows = [2, 2, 4, 6, 8, 9];
    for level in 0..=GUILD_LEVELS {
        let guild = thieves_guild_at(&game, &rules, level);
        assert_eq!(guild.level, level);
        let shown: Vec<GuildRow> = guild.rows.iter().map(|r| r.row).collect();
        assert_eq!(shown, GuildRow::ALL[..rows[usize::from(level)]]);
        assert!(guild.rows.iter().all(|r| r.places.len() == 2));
        assert_eq!(guild.players.len(), 2);
        let red = guild.players[0];
        assert_eq!(red.hero.is_some(), level >= 1, "{level}");
        assert_eq!(red.hero.and_then(|h| h.skills).is_some(), level >= 2);
        assert_eq!(red.creature.is_some(), level >= 4);
    }
    // A Den of Thieves shows everything; past five the same.
    let all = thieves_guild_at(&game, &rules, DEN_LEVEL);
    assert_eq!(all.rows.len(), 9);
    assert_eq!(thieves_guild_at(&game, &rules, 99).rows, all.rows);
    // Red's best hero and creature: its one hero and its imps.
    let red = all.players[0];
    let details = game.hero(HeroId::new(0)).unwrap();
    assert_eq!(red.hero.map(|h| h.hero_type), Some(details.hero_type));
    let skills = h3_heroes::primary_skills(details, rules.core());
    assert_eq!(red.hero.and_then(|h| h.skills), Some(skills));
    assert_eq!(red.creature, Some(creature(&rules, "t:imp")));
}

// covers: TWN-TAV-008
#[test]
fn a_rows_places_go_from_the_highest_equal_values_sharing_one() {
    assert_eq!(places(&[5, 9, 5, 1]), [1, 0, 1, 2]);
    assert_eq!(places(&[0, 0]), [0, 0]);
    assert_eq!(places(&[]), [] as [u8; 0]);
    let rules = guild_rules("t:faithful");
    let mut game = game(&rules, &towns_setup(&rules), 5000);
    game.set_resource(BLUE, GOLD, 7000);
    let guild = thieves_guild_at(&game, &rules, 2);
    let ranking = |row: GuildRow, places: [u8; 2]| GuildRanking {
        row,
        places: places.to_vec(),
    };
    // A town each shares the first place; blue's gold above red's; no resource of the second
    // row, a shared place.
    assert_eq!(
        guild.rows,
        [
            ranking(GuildRow::Towns, [0, 0]),
            ranking(GuildRow::Heroes, [0, 0]),
            ranking(GuildRow::Gold, [1, 0]),
            ranking(GuildRow::Common, [0, 0]),
        ]
    );
}

// covers: TWN-TAV-008
#[test]
fn each_row_counts_the_originals_value() {
    let rules = guild_rules("t:faithful");
    let core = rules.core();
    let mut game = game(&rules, &towns_setup(&rules), 5000);
    let gog = creature(&rules, "t:gog");
    game.adventure_with_heroes()
        .0
        .set_town_garrison(T0, vec![stack(0, gog, 2)]);
    let red = |game: &GameState, row| value(game, &rules, row, RED);
    assert_eq!(red(&game, GuildRow::Towns), 1);
    assert_eq!(red(&game, GuildRow::Heroes), 1);
    assert_eq!(red(&game, GuildRow::Gold), 5000);
    assert_eq!(red(&game, GuildRow::Common), 0);
    assert_eq!(red(&game, GuildRow::Rare), 0);
    // `TWN-STUB-17`: the obelisks visited are 0 until they are placed.
    assert_eq!(red(&game, GuildRow::Obelisks), 0);
    // The army: its hero's power on the map and T0's garrison's value, 2 gogs at 100, no factor.
    let hero = game.adventure().hero(HeroId::new(0)).unwrap().clone();
    let power = h3_adventure::power(core, &hero.army, game.hero(hero.id));
    assert_eq!(red(&game, GuildRow::Army), power + 200);
    // The income: the gold the next day's start pays.
    let income = red(&game, GuildRow::Income);
    assert!(income > 0);
    for player in [RED, BLUE] {
        run(&rules, &mut game, player, GameCommand::EndTurn).unwrap();
    }
    assert_eq!(gold(&game), 5000 + u32::try_from(income).unwrap());
    // Each resource row sums the treasury's resources the part puts in it; in none, nothing.
    let rows = [GuildRow::Gold, GuildRow::Common, GuildRow::Rare];
    for at in 0..=rows.len() {
        let rules = guild_rules_with("t:faithful", true, at);
        let game = self::game(&rules, &towns_setup(&rules), 5000);
        for (row, &kind) in rows.iter().enumerate() {
            let held = if row == at { 5000 } else { 0 };
            assert_eq!(value(&game, &rules, kind, RED), held, "{at} {row}");
        }
    }
}

// covers: TWN-TAV-008, TWN-TAV-009
#[test]
fn a_garrison_hero_counts_by_its_towns_army_not_as_a_hero() {
    let rules = guild_rules("t:faithful");
    let mut game = game(&rules, &towns_setup(&rules), 5000);
    let step = AdventureCommand::MoveHero {
        hero: HeroId::new(0),
        path: vec![ENTRANCE],
    };
    run(&rules, &mut game, RED, GameCommand::Adventure(step)).unwrap();
    let enter = TownCommand::Heroes {
        town: T0,
        change: TownHeroChange::Enter,
    };
    run(&rules, &mut game, RED, GameCommand::Town(enter)).unwrap();
    assert!(game.adventure().heroes().iter().all(|h| h.owner != RED));
    // Not a hero on the map; its 9 imps as T0's army, without a factor; no best hero, its imp
    // the best creature.
    assert_eq!(value(&game, &rules, GuildRow::Heroes, RED), 0);
    assert_eq!(value(&game, &rules, GuildRow::Army, RED), 450);
    let red = thieves_guild_at(&game, &rules, DEN_LEVEL).players[0];
    assert_eq!(red.hero, None);
    assert_eq!(red.creature, Some(creature(&rules, "t:imp")));
}

// covers: TWN-TAV-009
#[test]
fn the_best_hero_is_the_first_of_the_greatest_power_above_0() {
    let rules = guild_rules("t:faithful");
    let imp = creature(&rules, "t:imp");
    let mut setup = towns_setup(&rules);
    let second_type = rules.core().hero_type_by_key("t:k0").unwrap();
    let skilled = HeroDetailsSetup {
        primary: Some([110, 2, 3, 4]),
        ..HeroDetailsSetup::default()
    };
    setup.heroes[0].details = skilled.clone();
    let mut second = setup.heroes[0].clone();
    second.hero_type = Some(second_type);
    second.pos = Pos::new(0, 1, 0);
    setup.heroes.push(second);
    // Two heroes of equal power: the first in the adventure's order; its Attack 110 shown as 99.
    let game = game(&rules, &setup, 5000);
    let red = thieves_guild_at(&game, &rules, 2).players[0].hero.unwrap();
    assert_eq!(red.hero_type, game.hero(HeroId::new(0)).unwrap().hero_type);
    assert_eq!(red.skills, Some([99, 2, 3, 4]));
    // A strictly greater power replaces it.
    setup.heroes[2].army = HeroArmy::Custom(vec![stack(1, imp, 10)]);
    let game = self::game(&rules, &setup, 5000);
    let red = thieves_guild_at(&game, &rules, 1).players[0].hero.unwrap();
    assert_eq!((red.hero_type, red.skills), (second_type, None));
    // Every creature worth 0: no best hero, no best creature.
    let worthless = guild_rules_valued("t:faithful", false);
    let game = self::game(&worthless, &towns_setup(&worthless), 5000);
    let red = thieves_guild_at(&game, &worthless, DEN_LEVEL).players[0];
    assert_eq!((red.hero, red.creature), (None, None));
}

// covers: TWN-TAV-009
#[test]
fn the_best_creature_is_the_greatest_over_the_towns_then_the_heroes() {
    let rules = guild_rules("t:faithful");
    let (imp, gog) = (creature(&rules, "t:imp"), creature(&rules, "t:gog"));
    // T0's one gog above its hero's nine imps (TD-23: the towns' armies too).
    let mut game = game(&rules, &towns_setup(&rules), 5000);
    let set =
        |game: &mut GameState, army| game.adventure_with_heroes().0.set_town_garrison(T0, army);
    set(&mut game, vec![stack(1, gog, 1)]);
    let best = |game: &GameState| thieves_guild_at(game, &rules, 4).players[0].creature;
    assert_eq!(best(&game), Some(gog));
    // An imp in T0, a gog with the hero: the hero's, strictly greater.
    let mut setup = towns_setup(&rules);
    setup.heroes[0].army = HeroArmy::Custom(vec![stack(0, imp, 9), stack(1, gog, 1)]);
    let mut game = self::game(&rules, &setup, 5000);
    set(&mut game, vec![stack(0, imp, 4)]);
    assert_eq!(best(&game), Some(gog));
}

// covers: TWN-TAV-008
#[test]
fn the_artifacts_row_counts_the_backpacks_alone_under_twn_quirk_13() {
    // Red's hero wears two rings, blue's carries a charm.
    let artifacts = |rules: &GameRuleset| {
        let mut setup = towns_setup(rules);
        let ring = ArtifactInstance::from(artifact(rules, "t:ring"));
        let charm = ArtifactInstance::from(artifact(rules, "t:charm"));
        setup.heroes[0].details.artifacts = Some(HeroArtifactsSetup {
            worn: h3_heroes::wearing(rules.core(), &[ring, ring]),
            backpack: vec![],
        });
        setup.heroes[1].details.artifacts = Some(HeroArtifactsSetup {
            worn: h3_heroes::wearing(rules.core(), &[]),
            backpack: vec![charm],
        });
        let game = game(rules, &setup, 5000);
        let row = |player| value(&game, rules, GuildRow::Artifacts, player);
        let places = thieves_guild_at(&game, rules, 4).rows[6].places.clone();
        ([row(RED), row(BLUE)], places)
    };
    // The flag on (the original's): the backpacks alone, blue first.
    assert_eq!(artifacts(&guild_rules("t:guild")), ([0, 1], vec![1, 0]));
    // Off: the worn and the backpack's, red first.
    assert_eq!(artifacts(&guild_rules("t:faithful")), ([2, 1], vec![0, 1]));
}
