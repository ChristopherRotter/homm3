//! Tests of the town's heroes learning its guild's spells at the town's own triggers (E09 T48a,
//! design note `E09-design.md` §88, §89; TWN-GLD-004, -005): a build (the command's and a town
//! event's), a move between the bars that leaves a visitor, a town's hire (the hired visitor, then
//! the garrison hero) and the start's main towns (TD-10); each spell `HeroChanged` to the hero's
//! owner, nothing without a spell book, nothing at a turn's start or a new day. On the town tests'
//! game: red's T0 on (1, 1) and blue's T1 on (3, 0), each built by its setup's slots only; every
//! hero type with a spell book and a starting army; `t:town`'s Mage Guild 1 (map slot 0), Library, Aurora Borealis and
//! Tavern (map slot 5), its guild one slot a level, so level 1 draws its slot and its spare (gust
//! and spark) and level 3 gate alone.

use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

use h3_adventure::Pos;
use h3_content::{Ruleset, SpellId, StartingStackDef};
use h3_contracts::{HeroId, PlayerId, TownId};
use h3_heroes::HeroChange;
use h3_towns::{
    BuildingDef, BuildingId, BuildingRoles, BuildingsSetup, SlotSet, TownCommand, TownEventSetup,
    TownHeroChange,
};

use super::command::{BLUE, RED, T0, T1, end_turn, run};
use super::{faction, rules_of, towns_setup};
use crate::tests::key;
use crate::{
    GameCommand, GameEvent, GameRuleset, GameSetup, GameState, HeroArtifactsSetup,
    HeroDetailsSetup, MainTown,
};

/// T0's entrance (its one tile) and T1's.
const ENTRANCE: Pos = Pos::new(1, 1, 0);
const T1_ENTRANCE: Pos = Pos::new(3, 0, 0);
/// The map slots of the Mage Guild and the Tavern (the h3m's).
const GUILD_SLOT: u8 = 0;
const TAVERN_SLOT: u8 = 5;

/// The learning tests' rules (above): `super::guild::guild_packs`' spells and weights, every hero
/// type with a spell book (and a type without an army 2 to 3 imps: a main town's hero takes its
/// type's), the four buildings after the record's, free and without requirements.
fn rules() -> GameRuleset {
    let (mut core, mut towns) = super::guild::guild_packs();
    for ty in &mut core.heroes {
        ty.spell_book = true;
        if ty.army.is_empty() {
            ty.army = vec![StartingStackDef {
                creature: key("t:imp"),
                count: [2, 3],
            }];
        }
    }
    let building = |k: &str, slot, roles| BuildingDef {
        key: k.to_string(),
        name: "Building".into(),
        slot,
        supersedes: vec![],
        forbidden_with: None,
        auto_with: vec![],
        requires: vec![],
        cost: vec![],
        roles,
    };
    let roles = |edit: fn(&mut BuildingRoles)| {
        let mut roles = BuildingRoles::default();
        edit(&mut roles);
        roles
    };
    towns.towns[0].buildings.extend([
        building("guild", Some(GUILD_SLOT), roles(|r| r.guild = Some(1))),
        building("library", None, roles(|r| r.library = true)),
        building("aurora", None, roles(|r| r.aurora = true)),
        building("tavern", Some(TAVERN_SLOT), roles(|r| r.tavern = true)),
    ]);
    towns.towns[0].tavern = vec![(key("t:knight"), 3), (key("t:cleric"), 2)];
    towns.params.as_mut().unwrap().guild_slots = [1; 5];
    rules_of(
        Ruleset::compile(&[core], "t:faithful").unwrap(),
        &towns,
        false,
        4,
    )
}

/// `t:town`'s building of key `k`.
fn building(rules: &GameRuleset, k: &str) -> BuildingId {
    let record = rules.towns().town(faction(rules, "t:town")).unwrap();
    record.building_by_key(k).unwrap()
}

/// The town tests' setup with both towns built by `slots` alone and red's hero (0) on T0's
/// entrance; blue's (1) stays on (2, 2).
fn setup(rules: &GameRuleset, slots: &[u8]) -> GameSetup {
    let mut setup = towns_setup(rules);
    let built = slots
        .iter()
        .fold(SlotSet::EMPTY, |set, &slot| set.with(slot));
    for town in &mut setup.towns {
        town.buildings = BuildingsSetup::Custom {
            built,
            forbidden: SlotSet::EMPTY,
        };
    }
    setup.heroes[0].pos = ENTRANCE;
    setup
}

/// The game of `setup` with red holding 10 000 gold.
fn game(rules: &GameRuleset, setup: &GameSetup) -> GameState {
    let mut game = GameState::new(rules, setup, 9).unwrap();
    game.set_resource(RED, h3_content::ResourceId::from_raw(0), 10_000);
    game
}

/// T0's guild's level-1 slots: its slot, then its spare.
fn level_1(game: &GameState, town: TownId) -> [SpellId; 2] {
    let slots: Vec<SpellId> = game
        .towns()
        .get(town)
        .unwrap()
        .guild()
        .level(1)
        .flatten()
        .collect();
    [slots[0], slots[1]]
}

/// The rules' spell of key `k`.
fn spell(rules: &GameRuleset, k: &str) -> SpellId {
    rules.core().spell_by_key(k).unwrap()
}

/// The spells `events` teach, as (owner, hero, spell) in order.
fn taught(events: &[GameEvent]) -> Vec<(PlayerId, HeroId, SpellId)> {
    events
        .iter()
        .filter_map(|e| match *e {
            GameEvent::HeroChanged {
                player,
                hero,
                change: HeroChange::LearnSpell(spell),
            } => Some((player, hero, spell)),
            _ => None,
        })
        .collect()
}

fn spells(game: &GameState, hero: HeroId) -> Vec<SpellId> {
    game.hero(hero).unwrap().spells.clone()
}

fn town_command(command: TownCommand) -> GameCommand {
    GameCommand::Town(command)
}

/// The next day's red turn: both players' turns ended.
fn next_day(rules: &GameRuleset, game: &mut GameState) -> Vec<GameEvent> {
    end_turn(rules, game, RED);
    let mut events = Vec::new();
    crate::apply(
        game,
        rules,
        crate::Actor::Player(BLUE),
        GameCommand::EndTurn,
        &mut events,
    )
    .unwrap();
    events
}

const RED_HERO: HeroId = HeroId::new(0);
const BLUE_HERO: HeroId = HeroId::new(1);

// covers: TWN-GLD-005, TWN-BLD-005, TWN-GLD-004
#[test]
fn each_build_teaches_the_visitor_what_the_guild_shows_then() {
    let rules = rules();
    let mut game = game(&rules, &setup(&rules, &[]));
    let [slot, spare] = level_1(&game, T0);
    let before = spells(&game, RED_HERO);
    let build = |k| {
        town_command(TownCommand::Build {
            town: T0,
            building: building(&rules, k),
        })
    };
    // Mage Guild 1: its one slot, told to red after the build's own events.
    let events = run(&rules, &mut game, RED, build("guild")).unwrap();
    assert_eq!(taught(&events), [(RED, RED_HERO, slot)]);
    let built = events
        .iter()
        .position(|e| matches!(e, GameEvent::TownBuilt { .. }))
        .unwrap();
    assert!(matches!(events.last(), Some(GameEvent::HeroChanged { .. })));
    assert!(built < events.len() - 1);
    assert_eq!(spells(&game, RED_HERO), [&before[..], &[slot]].concat());
    // Nothing at a new day's or a turn's start; the Library's build its spare at once.
    let day = next_day(&rules, &mut game);
    assert_eq!(taught(&day), []);
    let events = run(&rules, &mut game, RED, build("library")).unwrap();
    assert_eq!(taught(&events), [(RED, RED_HERO, spare)]);
    // The Aurora Borealis: its set up to Wisdom + 2 (gate is level 3), the known spells skipped.
    next_day(&rules, &mut game);
    let events = run(&rules, &mut game, RED, build("aurora")).unwrap();
    assert_eq!(taught(&events), []);
    assert_eq!(
        spells(&game, RED_HERO),
        [&before[..], &[slot, spare]].concat()
    );
}

// covers: TWN-GLD-004, TWN-GLD-005
#[test]
fn the_aurora_teaches_past_the_guild_s_level_and_a_hero_without_a_book_nothing() {
    let rules = rules();
    let mut setup = setup(&rules, &[GUILD_SLOT]);
    // Red's hero with Basic Sense (spell level 3); blue's without a book on T1's entrance.
    let sense = rules.core().skill_by_key("t:sense").unwrap();
    setup.heroes[0].details.skills = Some(vec![(sense, 1)]);
    setup.heroes[1].pos = T1_ENTRANCE;
    setup.heroes[1].details.artifacts = Some(HeroArtifactsSetup {
        worn: h3_heroes::wearing(rules.core(), &[]),
        backpack: vec![],
    });
    let mut game = game(&rules, &setup);
    // The placed heroes learned nothing at the start (no main town generates one).
    let [red_before, blue_before] = [RED_HERO, BLUE_HERO].map(|hero| spells(&game, hero));
    let [gust, gate, spark] = ["t:gust", "t:gate", "t:spark"].map(|k| spell(&rules, k));
    let aurora = town_command(TownCommand::Build {
        town: T0,
        building: building(&rules, "aurora"),
    });
    let events = run(&rules, &mut game, RED, aurora).unwrap();
    // The Aurora set in spell order (gust, gate, spark), gate past the guild's level 1.
    let known = |s: &SpellId| !red_before.contains(s);
    let expected: Vec<_> = [gust, gate, spark].into_iter().filter(known).collect();
    assert_eq!(
        taught(&events),
        expected
            .iter()
            .map(|&s| (RED, RED_HERO, s))
            .collect::<Vec<_>>()
    );
    // Blue's hero without a book at its own T1's build learns nothing.
    end_turn(&rules, &mut game, RED);
    let events = run(
        &rules,
        &mut game,
        BLUE,
        town_command(TownCommand::Build {
            town: T1,
            building: building(&rules, "aurora"),
        }),
    )
    .unwrap();
    assert_eq!(taught(&events), []);
    assert_eq!(spells(&game, BLUE_HERO), blue_before);
}

// covers: TWN-GLD-005, TWN-EVT-003
#[test]
fn a_town_event_s_build_teaches_at_the_owner_s_turn_start() {
    let rules = rules();
    let mut setup = setup(&rules, &[]);
    setup.towns[0].events = vec![TownEventSetup {
        players: vec![RED],
        human: true,
        computer: false,
        first_day: 2,
        interval: 0,
        resources: vec![],
        buildings: SlotSet::EMPTY.with(GUILD_SLOT),
        creatures: [0; 7],
    }];
    let mut game = game(&rules, &setup);
    let [slot, _] = level_1(&game, T0);
    let events = next_day(&rules, &mut game);
    assert_eq!(taught(&events), [(RED, RED_HERO, slot)]);
    let built = events
        .iter()
        .position(|e| matches!(e, GameEvent::TownBuilt { event: Some(0), .. }))
        .unwrap();
    let learned = events
        .iter()
        .position(|e| matches!(e, GameEvent::HeroChanged { .. }))
        .unwrap();
    assert!(built < learned);
}

// covers: TWN-GLD-005, TWN-ROW-002, TWN-ROW-003
#[test]
fn a_move_between_the_bars_teaches_only_with_a_visitor_left_and_a_new_day_nothing() {
    let rules = rules();
    let mut game = game(&rules, &setup(&rules, &[GUILD_SLOT]));
    let [slot, _] = level_1(&game, T0);
    let before = spells(&game, RED_HERO);
    // A placed visitor learned nothing at the start, nor at a new day's or its turn's start.
    assert!(!before.contains(&slot));
    assert_eq!(taught(&next_day(&rules, &mut game)), []);
    assert_eq!(spells(&game, RED_HERO), before);
    let heroes = |change| town_command(TownCommand::Heroes { town: T0, change });
    // Into the garrison: no visitor stands after it, nothing taught.
    let events = run(&rules, &mut game, RED, heroes(TownHeroChange::Enter)).unwrap();
    assert_eq!(taught(&events), []);
    assert_eq!(game.adventure().town_hero(T0).unwrap().id, RED_HERO);
    // Out again: the visitor learns, after the move's events.
    let events = run(&rules, &mut game, RED, heroes(TownHeroChange::Leave)).unwrap();
    assert_eq!(taught(&events), [(RED, RED_HERO, slot)]);
    assert!(matches!(events.last(), Some(GameEvent::HeroChanged { .. })));
    assert_eq!(spells(&game, RED_HERO), [&before[..], &[slot]].concat());
}

// covers: TWN-GLD-005, TWN-TAV-003
#[test]
fn a_hire_teaches_the_hired_visitor_then_the_garrison_hero() {
    let rules = rules();
    let mut game = game(&rules, &setup(&rules, &[GUILD_SLOT, TAVERN_SLOT]));
    let [slot, _] = level_1(&game, T0);
    let enter = town_command(TownCommand::Heroes {
        town: T0,
        change: TownHeroChange::Enter,
    });
    run(&rules, &mut game, RED, enter).unwrap();
    let hired_type = game.pool().tavern(RED).unwrap().slots[0].unwrap();
    let hired = game.pool().record(hired_type).unwrap();
    let hire = town_command(TownCommand::Hire { town: T0, slot: 0 });
    let events = run(&rules, &mut game, RED, hire).unwrap();
    let after = |hero| !spells(&game, hero).is_empty();
    assert_eq!(taught(&events), [(RED, hired, slot), (RED, RED_HERO, slot)]);
    assert!(after(hired) && after(RED_HERO));
    // After the hire's own events: the hero placed and told first.
    let placed = events
        .iter()
        .position(|e| matches!(e, GameEvent::HeroHired { .. }))
        .unwrap();
    let learned = events
        .iter()
        .position(|e| matches!(e, GameEvent::HeroChanged { .. }))
        .unwrap();
    assert!(placed < learned);
}

// covers: TWN-GLD-005, TWN-SET-009
#[test]
fn td_10_a_main_town_s_new_hero_learns_at_the_start_a_placed_one_not() {
    let rules = rules();
    let mut setup = setup(&rules, &[GUILD_SLOT]);
    // Red's main town T0 generates its hero; red's placed hero off the entrance, blue's placed on
    // T1's (its main town generates none).
    setup.players[0].start.main_town = Some(MainTown {
        town: T0,
        generates_hero: true,
    });
    setup.heroes[0].pos = Pos::new(0, 0, 0);
    setup.heroes[1].pos = T1_ENTRANCE;
    setup.heroes[1].details = HeroDetailsSetup {
        spells: Some(vec![]),
        ..HeroDetailsSetup::default()
    };
    let game = game(&rules, &setup);
    let [slot, _] = level_1(&game, T0);
    let generated = game.adventure().town_visitor(T0).unwrap();
    assert_eq!(generated, HeroId::new(2));
    assert_eq!(spells(&game, generated), [slot]);
    assert_eq!(spells(&game, BLUE_HERO), []);
    // The start draws nothing for it: a second build of the seed is the same game.
    let again = GameState::new(&rules, &setup, 9).unwrap();
    assert_eq!(again.hero(generated), game.hero(generated));
    // Without the guild the same new hero learns nothing.
    for town in &mut setup.towns {
        town.buildings = BuildingsSetup::Custom {
            built: SlotSet::EMPTY,
            forbidden: SlotSet::EMPTY,
        };
    }
    let bare = GameState::new(&rules, &setup, 9).unwrap();
    assert_eq!(
        bare.hero(generated).unwrap().hero_type,
        game.hero(generated).unwrap().hero_type
    );
    assert_eq!(spells(&bare, generated), []);
}
