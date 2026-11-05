//! Tests of the main towns' heroes (E09 T7b2b, design note `E09-design.md` §14 step 3):
//! TWN-SET-009, the unused picks freed after it (TWN-SET-005) and `TWN-QUIRK-6`.

use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

use h3_adventure::{Pos, SetupError, StandError, TileMask};
use h3_content::{Formation, HeroTypeId};
use h3_contracts::{HeroId, PlayerId, TownId};
use h3_kernel::{Rng, StreamId};
use h3_towns::{TownFaction, town_setup};

use super::{faction, hero, start, town, town_object, towns_rules, towns_rules_with, towns_setup};
use crate::town::pool::{HeroPool, HeroStatus};
use crate::town::{
    FactionChoice, FactionSet, MainTown, check_starts, draw_heroes, draw_towns, main_town_heroes,
};
use crate::{GameHeroSetup, GameRuleset, GameSetup, GameSetupError, GameState, HeroArmy};

const RED: PlayerId = PlayerId::new(0);
const BLUE: PlayerId = PlayerId::new(1);
/// Red's main town's entrance in [`generating`].
const ENTRANCE: Pos = Pos::new(1, 1, 0);
/// The first hero after [`towns_setup`]'s two.
const FIRST: HeroId = HeroId::new(2);

/// [`towns_setup`] whose red's main town (town 0, its entrance [`ENTRANCE`]) generates its hero,
/// the pool's free types two with a starting army that fits the part's two slots (the gunner and
/// the sage; the two placed heroes are Orrin), red human without a pick.
fn generating(rules: &GameRuleset) -> GameSetup {
    let mut setup = towns_setup(rules);
    setup.players[0].start = start(faction(rules, "t:town"), true);
    setup.allowed_heroes = types(rules, &["t:gunner", "t:sage"]);
    setup
}

/// The types of `keys`.
fn types(rules: &GameRuleset, keys: &[&str]) -> Vec<HeroTypeId> {
    keys.iter().map(|k| hero(rules, k)).collect()
}

/// Step 3 alone on `setup` with `seed` (the build's steps before it run as the build runs them):
/// its heroes or its error, the pool after it and whether it drew from the stream.
fn step_3(
    rules: &GameRuleset,
    setup: &GameSetup,
    seed: u64,
) -> (Result<Vec<GameHeroSetup>, GameSetupError>, HeroPool, bool) {
    let (owners, entrances) = check_starts(setup, rules).unwrap();
    let mut draws = Rng::for_stream(seed, StreamId::SETUP);
    let mut drawn = draw_heroes(setup, rules, &owners, &mut draws).unwrap();
    draw_towns(setup, rules, &owners, &drawn.factions, &mut draws);
    let before = draws.clone();
    let heroes = main_town_heroes(setup, rules, &entrances, &mut drawn, &mut draws);
    (heroes, drawn.pool, draws != before)
}

/// The type of the first hero step 3 gives `setup` with `seed`.
fn first_type(rules: &GameRuleset, setup: &GameSetup, seed: u64) -> HeroTypeId {
    step_3(rules, setup, seed).0.unwrap()[0].hero_type.unwrap()
}

// covers: TWN-SET-009, GAM-SET-001, GAM-HER-001
#[test]
fn a_main_towns_hero_stands_on_its_entrance_after_the_setups_heroes() {
    let rules = towns_rules();
    let setup = generating(&rules);
    let free = types(&rules, &["t:gunner", "t:sage"]);
    let mut drawn = Vec::new();
    for seed in 0..16 {
        let game = GameState::new(&rules, &setup, seed).unwrap();
        assert_eq!(game.heroes().len(), 3);
        let on_map = game.adventure().hero(FIRST).unwrap();
        assert_eq!(
            (on_map.owner, on_map.pos, on_map.formation),
            (RED, ENTRANCE, Formation::Loose)
        );
        // The town's entrance as the adventure has it: the hero is its visitor.
        let map = game.adventure().map();
        assert_eq!(map.objects()[0].entrance(map), Some(ENTRANCE));
        // A type drawn over red's faction, taken by red's colour.
        let details = game.hero(FIRST).unwrap();
        let ty = details.hero_type;
        assert!(free.contains(&ty));
        assert_eq!(game.hero_pool().status(ty), Some(HeroStatus::Taken(0)));
        // Its type's starting army (GAM-SET-001) and the default details (GAM-HER-001).
        let first = &rules.core().hero_type(ty).unwrap().army[0];
        assert_eq!(on_map.army[0].creature, first.creature);
        assert_eq!(details.level, 1);
        assert!((40..=90).contains(&details.experience));
        drawn.push(ty);
    }
    // The draw varies with the seed.
    assert!(drawn.iter().any(|&t| t != drawn[0]));
    // One hero more per generating town counts against the heroes' ids.
    let mut many = setup.clone();
    many.heroes = vec![setup.heroes[0].clone(); usize::from(u16::MAX) + 1];
    let error = many.validate(&rules).unwrap_err();
    assert_eq!(error, GameSetupError::Heroes(usize::from(u16::MAX) + 2));
}

// covers: TWN-SET-009
#[test]
fn the_main_towns_heroes_come_in_player_order() {
    let rules = towns_rules();
    let castle = faction(&rules, "t:town");
    let mut setup = generating(&rules);
    // Blue's main town is town 1, its random town aligned to blue (its faction blue's, castle,
    // fixed: nothing drawn before the heroes).
    setup.players[1].start.choice = Some(FactionChoice::Fixed(castle));
    setup.players[1].start.main_town = Some(MainTown {
        town: TownId::new(1),
        generates_hero: true,
    });
    for seed in 0..8 {
        let game = GameState::new(&rules, &setup, seed).unwrap();
        let on_map = |id| game.adventure().hero(HeroId::new(id)).unwrap();
        assert_eq!((on_map(2).owner, on_map(2).pos), (RED, ENTRANCE));
        assert_eq!((on_map(3).owner, on_map(3).pos), (BLUE, Pos::new(3, 0, 0)));
        // Red's draw first, then blue's, each type taken by its colour at once.
        let mut replay = Rng::for_stream(seed, StreamId::SETUP);
        let mut pool = HeroPool::new(rules.core(), &setup);
        for (colour, id) in [(0, 2), (1, 3)] {
            let (ty, _) = pool
                .draw(rules.core(), Some(castle), colour, &mut replay)
                .unwrap();
            pool.take(ty, colour);
            assert_eq!(game.hero(HeroId::new(id)).unwrap().hero_type, ty);
            assert_eq!(game.hero_pool().status(ty), Some(HeroStatus::Taken(colour)));
        }
    }
}

// covers: TWN-SET-009, TWN-SET-002, TWN-SET-004
#[test]
fn the_main_towns_heroes_draw_after_the_random_towns_and_before_the_towns_buildings() {
    let rules = towns_rules();
    let castle = faction(&rules, "t:town");
    let mut setup = generating(&rules);
    // Blue's choice fixed (nothing drawn for it); a neutral random town of no alignment, whose
    // faction step 2 draws and whose garrison step 4 draws.
    setup.players[1].start.choice = Some(FactionChoice::Fixed(castle));
    setup
        .towns
        .push(town(TownFaction::Random { alignment: None }));
    setup.map.objects.push(town_object(&rules, (0, 3), None));
    for seed in 0..16 {
        let game = GameState::new(&rules, &setup, seed).unwrap();
        let mut replay = Rng::for_stream(seed, StreamId::SETUP);
        // Step 2: the neutral town's faction, over the one faction with a town record.
        replay.below(1);
        // Step 3: red's hero.
        let pool = HeroPool::new(rules.core(), &setup);
        let (ty, _) = pool
            .draw(rules.core(), Some(castle), 0, &mut replay)
            .unwrap();
        assert_eq!(game.hero(FIRST).unwrap().hero_type, ty);
        // Step 4: the towns in `TownId` order.
        for (index, owned) in [(0, true), (1, true), (2, false)] {
            let built = town_setup(
                &setup.towns[index],
                castle,
                owned,
                rules.towns(),
                &mut replay,
            );
            let built = built.unwrap();
            let id = TownId::new(u16::try_from(index).unwrap());
            assert_eq!(game.towns().get(id), Some(&built.town));
            assert_eq!(
                game.adventure().town_garrison(index),
                Some(&built.garrison[..])
            );
        }
    }
}

// covers: TWN-SET-009, TWN-SET-006
#[test]
fn a_human_players_pick_goes_to_its_main_towns_hero() {
    let rules = towns_rules();
    let sage = hero(&rules, "t:sage");
    let mut setup = generating(&rules);
    setup.players[0].start.pick = Some(sage);
    for seed in 0..8 {
        let game = GameState::new(&rules, &setup, seed).unwrap();
        assert_eq!(game.hero(FIRST).unwrap().hero_type, sage);
        assert_eq!(game.hero_pool().status(sage), Some(HeroStatus::Taken(0)));
        // The pick draws nothing from the stream.
        let (heroes, _, drew) = step_3(&rules, &setup, seed);
        assert_eq!(heroes.unwrap()[0].hero_type, Some(sage));
        assert!(!drew);
    }
    // The sage starts with its spell book (GAM-HER-001's default).
    let game = GameState::new(&rules, &setup, 1).unwrap();
    assert!(game.hero(FIRST).unwrap().worn.iter().any(Option::is_some));
}

// covers: TWN-SET-009, TWN-SET-008
#[test]
fn a_pick_its_random_hero_took_leaves_the_main_towns_hero_to_the_draw() {
    let rules = towns_rules();
    let sage = hero(&rules, "t:sage");
    let mut setup = generating(&rules);
    setup.players[0].start.pick = Some(sage);
    // Red's placed hero random, with its custom army: it takes the pick (step 2).
    setup.heroes[0].hero_type = None;
    let gunner = hero(&rules, "t:gunner");
    for seed in 0..8 {
        let game = GameState::new(&rules, &setup, seed).unwrap();
        assert_eq!(game.hero(HeroId::new(0)).unwrap().hero_type, sage);
        assert_eq!(game.hero(FIRST).unwrap().hero_type, gunner);
        assert!(step_3(&rules, &setup, seed).2);
    }
}

// covers: TWN-SET-009, TWN-SET-005
#[test]
fn a_computer_players_pick_stays_out_of_its_main_towns_draw_then_goes_back_free() {
    let rules = towns_rules();
    let sage = hero(&rules, "t:sage");
    let mut setup = generating(&rules);
    setup.players[0].human = false;
    setup.players[0].start.pick = Some(sage);
    let gunner = hero(&rules, "t:gunner");
    for seed in 0..8 {
        let game = GameState::new(&rules, &setup, seed).unwrap();
        assert_eq!(game.hero(FIRST).unwrap().hero_type, gunner);
        assert_eq!(game.hero_pool().status(sage), Some(HeroStatus::Free));
    }
    // With the pick the only free type, the computer's draw finds none (TWN-QUIRK-4).
    setup.allowed_heroes = vec![sage];
    let error = setup.validate(&rules).unwrap_err();
    assert_eq!(error, GameSetupError::NoFreeHero(RED));
    assert_eq!(error.to_string(), "player 0 draws a hero with none free");
    // A human player's draw finds none either without a free type.
    setup.players[0].human = true;
    setup.players[0].start.pick = None;
    setup.allowed_heroes.clear();
    assert_eq!(setup.validate(&rules), Err(GameSetupError::NoFreeHero(RED)));
}

// covers: TWN-SET-009, TWN-SET-007
#[test]
fn the_main_towns_hero_draws_over_its_players_faction_not_its_towns() {
    // Red's faction the other one (with a town record), its main town castle's: the draw gives
    // the other faction's cleric, which has no starting army to draw.
    let rules = towns_rules_with(true);
    let other = faction(&rules, "t:other");
    let mut setup = generating(&rules);
    setup.players[0].start.factions = FactionSet::EMPTY.with(other);
    setup.players[0].start.choice = Some(FactionChoice::Fixed(other));
    setup.allowed_heroes.push(hero(&rules, "t:c0"));
    for seed in 0..8 {
        assert_eq!(first_type(&rules, &setup, seed), hero(&rules, "t:c0"));
    }
    assert_eq!(
        setup.validate(&rules),
        Err(GameSetupError::StartingArmy(FIRST))
    );
}

// covers: TWN-SET-009, TWN-SET-001
#[test]
fn a_main_towns_hero_stands_on_the_first_entrance_on_the_map() {
    let rules = towns_rules();
    let text = "player 0's main town is not a town of its own";
    let with = |(x, y): (u16, u16), visitable: u64, generates: bool| {
        let mut setup = generating(&rules);
        let object = &mut setup.map.objects[0];
        object.anchor = Pos::new(x, y, 0);
        object.blocked = TileMask::from_bits(visitable);
        object.visitable = TileMask::from_bits(visitable);
        setup.players[0]
            .start
            .main_town
            .as_mut()
            .unwrap()
            .generates_hero = generates;
        setup
    };
    // The anchor past the right edge: its tile is skipped, the next one west is the entrance.
    let setup = with((4, 1), 0b11, true);
    let game = GameState::new(&rules, &setup, 3).unwrap();
    assert_eq!(game.adventure().hero(FIRST).unwrap().pos, Pos::new(3, 1, 0));
    let map = game.adventure().map();
    assert_eq!(map.objects()[0].entrance(map), Some(Pos::new(3, 1, 0)));
    // No entrance on the map: past the right edge, left of the first column, above the first
    // row. The main town is refused only when it generates a hero.
    for (anchor, visitable) in [((4, 1), 0b1), ((0, 1), 0b10), ((1, 0), 0x100)] {
        let error = with(anchor, visitable, true).validate(&rules).unwrap_err();
        assert_eq!(error, GameSetupError::MainTown(RED));
        assert_eq!(error.to_string(), text);
        assert_eq!(with(anchor, visitable, false).validate(&rules), Ok(()));
    }
}

// covers: TWN-SET-009, TWN-SET-010
#[test]
fn twn_quirk_6_a_main_towns_hero_is_refused_on_a_visitors_entrance() {
    let rules = towns_rules();
    // A placed hero on the entrance (TWN-SET-010's moved hero): the town's visitor, valid alone.
    let mut setup = towns_setup(&rules);
    setup.heroes[0].pos = ENTRANCE;
    assert_eq!(setup.validate(&rules), Ok(()));
    // With the town generating a hero too, the second hero there is refused, red's or blue's.
    for placed in [0, 1] {
        let mut setup = generating(&rules);
        setup.heroes[placed].pos = ENTRANCE;
        let error = setup.validate(&rules).unwrap_err();
        assert_eq!(
            error,
            GameSetupError::Adventure(SetupError::HeroTile {
                hero: FIRST,
                error: StandError::Occupied,
            })
        );
        assert_eq!(
            error.to_string(),
            "adventure: hero 2: another hero stands there"
        );
    }
}

// covers: TWN-SET-009, TWN-SET-005
#[test]
fn a_setup_without_a_generating_main_town_draws_nothing_in_step_3() {
    let rules = towns_rules();
    let loner = hero(&rules, "t:loner");
    let mut setup = towns_setup(&rules);
    // Red's main town generates no hero; its pick, which no random hero takes, goes back free.
    setup.players[0].start.pick = Some(loner);
    let (heroes, pool, drew) = step_3(&rules, &setup, 4);
    assert_eq!(heroes, Ok(Vec::new()));
    assert!(!drew);
    assert_eq!(pool.status(loner), Some(HeroStatus::Free));
    let game = GameState::new(&rules, &setup, 4).unwrap();
    assert_eq!(game.heroes().len(), 2);
    assert_eq!(game.hero_pool().status(loner), Some(HeroStatus::Free));
    // A main town's hero is then built as a placed one: with a human pick (which draws nothing),
    // the game is the one whose setup places that hero last, on the entrance, and generates none.
    let mut generated = generating(&rules);
    generated.players[0].start.pick = Some(hero(&rules, "t:sage"));
    let heroes = step_3(&rules, &generated, 4).0.unwrap();
    assert_eq!(heroes[0].army, HeroArmy::Starting);
    let mut placed = generated.clone();
    placed.players[0].start.pick = None;
    placed.players[0]
        .start
        .main_town
        .as_mut()
        .unwrap()
        .generates_hero = false;
    placed.heroes.extend(heroes);
    for seed in 0..4 {
        let built = |s| GameState::new(&rules, s, seed).unwrap();
        assert_eq!(built(&placed).state_hash(), built(&generated).state_hash());
    }
}
