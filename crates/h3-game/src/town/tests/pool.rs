//! Tests of the hero pool and the build's draws (E09 T7b1, design note `E09-design.md` §14):
//! TWN-SET-005, TWN-SET-007, a random choice's draw (TWN-SET-001) and a random hero's type
//! (TWN-SET-008); `TWN-QUIRK-2`, `TWN-QUIRK-3` and the draw's empty case of `TWN-QUIRK-4`.

use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

use h3_adventure::Pos;
use h3_content::{FactionId, HeroTypeId};
use h3_contracts::{HeroId, PlayerId};
use h3_kernel::{Rng, StreamId};

use super::{faction, hero, towns_rules, towns_rules_with, towns_setup};
use crate::tests::setup as setup_without_towns;
use crate::town::pool::{HeroPool, HeroStatus, draw_heroes};
use crate::town::{FactionChoice, FactionSet};
use crate::{GameRuleset, GameSetup, GameSetupError, GameState};

const RED: PlayerId = PlayerId::new(0);
const BLUE: PlayerId = PlayerId::new(1);

/// Makes hero `index` random, with hero 0's army (the knights have no starting army to draw).
fn random(setup: &mut GameSetup, index: usize) {
    setup.heroes[index].hero_type = None;
    setup.heroes[index].army = setup.heroes[0].army.clone();
}

/// Adds a random hero of `owner` at `(x, 3)`, hero 0's army.
fn add_random(setup: &mut GameSetup, owner: PlayerId, x: u16) {
    let mut added = setup.heroes[0].clone();
    added.hero_type = None;
    added.owner = owner;
    added.pos = Pos::new(x, 3, 0);
    setup.heroes.push(added);
}

/// The types of `keys`.
fn types(rules: &GameRuleset, keys: &[&str]) -> Vec<HeroTypeId> {
    keys.iter().map(|k| hero(rules, k)).collect()
}

/// Hero `index`'s type in the game `setup` builds with `seed`.
fn drawn(rules: &GameRuleset, setup: &GameSetup, seed: u64, index: u16) -> HeroTypeId {
    let game = GameState::new(rules, setup, seed).unwrap();
    game.hero(HeroId::new(index)).unwrap().hero_type
}

/// The faction of `id`'s class.
fn faction_of(rules: &GameRuleset, id: HeroTypeId) -> FactionId {
    let core = rules.core();
    core.hero_class(core.hero_type(id).unwrap().class)
        .unwrap()
        .faction
}

// covers: TWN-SET-005
#[test]
fn the_pool_frees_the_allowed_types_and_takes_the_placed_the_prisoners_and_the_picks() {
    let rules = towns_rules();
    let core = rules.core();
    let id = |k| hero(&rules, k);
    let unknown = HeroTypeId::from_raw(u16::MAX);
    let mut setup = towns_setup(&rules);
    // Blue's hero is the crowd, which the map does not allow; the cleric is not allowed either.
    setup.heroes[1].hero_type = Some(id("t:crowd"));
    setup
        .allowed_heroes
        .retain(|&t| t != id("t:crowd") && t != id("t:c0"));
    setup.allowed_heroes.push(unknown);
    setup.prisoners = vec![id("t:sage"), unknown];
    setup.hero_masks = vec![
        (id("t:gunner"), 0b0111),
        (id("t:gunner"), 0b1101),
        (unknown, 0),
    ];
    setup.players[0].start.pick = Some(id("t:loner"));
    // A placed hero whose owner is no player takes nothing (the adventure context refuses it).
    let mut stray = setup.heroes[0].clone();
    stray.hero_type = Some(id("t:k5"));
    stray.owner = PlayerId::new(5);
    setup.heroes.push(stray);
    let pool = HeroPool::new(core, &setup);
    let status = |k| pool.status(id(k));
    assert_eq!(status("t:orrin"), Some(HeroStatus::Taken(0)));
    assert_eq!(status("t:crowd"), Some(HeroStatus::Taken(1)));
    assert_eq!(status("t:c0"), Some(HeroStatus::Out));
    assert_eq!(status("t:sage"), Some(HeroStatus::Prisoner));
    assert_eq!(status("t:loner"), Some(HeroStatus::Taken(0)));
    assert_eq!(status("t:k0"), Some(HeroStatus::Free));
    assert_eq!(status("t:k5"), Some(HeroStatus::Free));
    assert_eq!(pool.mask(id("t:gunner")), Some(0b0101));
    assert_eq!(pool.mask(id("t:k0")), Some(0xFF));
    assert_eq!((pool.status(unknown), pool.mask(unknown)), (None, None));
    // A prisoner's type is taken whatever the allowed set.
    setup.prisoners.push(id("t:c0"));
    assert_eq!(
        HeroPool::new(core, &setup).status(id("t:c0")),
        Some(HeroStatus::Prisoner)
    );
}

// covers: TWN-SET-005
#[test]
fn a_pick_no_random_hero_takes_goes_back_free() {
    let rules = towns_rules();
    let loner = hero(&rules, "t:loner");
    let mut setup = towns_setup(&rules);
    setup.players[0].start.pick = Some(loner);
    // Red has the random-hero flag but no random hero (and no main town that generates one).
    let game = GameState::new(&rules, &setup, 3).unwrap();
    assert_eq!(game.hero_pool().status(loner), Some(HeroStatus::Free));
    // With a random hero it takes the pick, which stays taken.
    random(&mut setup, 0);
    let game = GameState::new(&rules, &setup, 3).unwrap();
    assert_eq!(game.hero(HeroId::new(0)).unwrap().hero_type, loner);
    assert_eq!(game.hero_pool().status(loner), Some(HeroStatus::Taken(0)));
}

// covers: TWN-SET-001
#[test]
fn a_random_choice_is_drawn_from_its_factions() {
    let rules = towns_rules_with(true);
    let (castle, other) = (faction(&rules, "t:town"), faction(&rules, "t:other"));
    let mut setup = towns_setup(&rules);
    setup.players[1].start.factions = FactionSet::EMPTY.with(castle).with(other);
    random(&mut setup, 1);
    // Blue's random hero draws from its drawn faction's classes: the cleric for the other
    // faction, a knight for the town's; both come up over the seeds.
    let factions: Vec<FactionId> = (0..24)
        .map(|seed| faction_of(&rules, drawn(&rules, &setup, seed, 1)))
        .collect();
    assert!(factions.contains(&castle) && factions.contains(&other));
    let cleric = (0..24).find(|&seed| faction_of(&rules, drawn(&rules, &setup, seed, 1)) == other);
    assert_eq!(
        drawn(&rules, &setup, cleric.unwrap(), 1),
        hero(&rules, "t:c0")
    );
    // A set of one gives it.
    setup.players[1].start.factions = FactionSet::EMPTY.with(other);
    for seed in 0..8 {
        assert_eq!(drawn(&rules, &setup, seed, 1), hero(&rules, "t:c0"));
    }
    // The choice is drawn first of the setup stream: one draw over two factions, then blue's
    // hero's over its faction's classes.
    setup.players[1].start.factions = FactionSet::EMPTY.with(castle).with(other);
    let mut draws = Rng::for_stream(9, StreamId::SETUP);
    let owners = [Some(RED), Some(BLUE)];
    let types = draw_heroes(&setup, &rules, &owners, &mut draws)
        .unwrap()
        .types;
    let mut replay = Rng::for_stream(9, StreamId::SETUP);
    let chosen = [castle, other][usize::try_from(replay.below(2)).unwrap()];
    assert_eq!(faction_of(&rules, types[1].0), chosen);
    // A choice nothing reads draws nothing (no main town, random-hero flag, random hero or
    // aligned random town): the game is the one without it, as on the duel's map.
    let plain = setup_without_towns();
    let mut chosen = plain.clone();
    for player in &mut chosen.players {
        player.start.factions = FactionSet::EMPTY.with(castle).with(other);
        player.start.choice = Some(FactionChoice::Random);
    }
    let built = |s| GameState::new(&rules, s, 5).unwrap();
    assert_eq!(built(&chosen), built(&plain));
}

// covers: TWN-SET-008, TWN-SET-007
#[test]
fn a_random_hero_takes_its_owners_pick_then_a_second_draws() {
    let rules = towns_rules();
    let castle = faction(&rules, "t:town");
    let loner = hero(&rules, "t:loner");
    let mut setup = towns_setup(&rules);
    setup.players[0].start.pick = Some(loner);
    random(&mut setup, 0);
    add_random(&mut setup, RED, 0);
    for seed in 0..16 {
        let game = GameState::new(&rules, &setup, seed).unwrap();
        let second = game.hero(HeroId::new(2)).unwrap().hero_type;
        assert_eq!(game.hero(HeroId::new(0)).unwrap().hero_type, loner);
        assert_ne!(second, loner);
        assert_eq!(faction_of(&rules, second), castle);
        assert_eq!(game.hero_pool().status(second), Some(HeroStatus::Taken(0)));
        assert_eq!(game.hero_pool().status(loner), Some(HeroStatus::Taken(0)));
    }
    // The same seed draws the same; the draws vary with the seed.
    let seconds: Vec<HeroTypeId> = (0..16).map(|seed| drawn(&rules, &setup, seed, 2)).collect();
    assert_eq!(
        seconds,
        (0..16)
            .map(|seed| drawn(&rules, &setup, seed, 2))
            .collect::<Vec<_>>()
    );
    assert!(seconds.iter().any(|&t| t != seconds[0]));
    let built = |seed| GameState::new(&rules, &setup, seed).unwrap();
    assert_eq!(built(4), built(4));
}

// covers: TWN-SET-007
#[test]
fn the_draw_falls_back_to_every_free_type_then_fails() {
    let rules = towns_rules();
    let id = |k| hero(&rules, k);
    let mut setup = towns_setup(&rules);
    random(&mut setup, 0);
    add_random(&mut setup, RED, 0);
    // One knight and the cleric free: the first random hero takes the knight, the second falls
    // back to the cleric, of no class of red's faction.
    setup.allowed_heroes = types(&rules, &["t:k3", "t:c0"]);
    for seed in 0..8 {
        assert_eq!(drawn(&rules, &setup, seed, 0), id("t:k3"));
        assert_eq!(drawn(&rules, &setup, seed, 2), id("t:c0"));
    }
    // Nothing left for the second: refused, the drawing player named (TWN-QUIRK-4).
    setup.allowed_heroes = types(&rules, &["t:k3"]);
    let error = setup.validate(&rules).unwrap_err();
    assert_eq!(error, GameSetupError::NoFreeHero(RED));
    assert_eq!(error.to_string(), "player 0 draws a hero with none free");
    // A pick is no free type of another player's draw: blue finds none.
    setup.allowed_heroes = types(&rules, &["t:k3", "t:loner"]);
    setup.heroes.truncate(2);
    setup.heroes[0].hero_type = Some(id("t:k3"));
    setup.players[0].start.pick = Some(id("t:loner"));
    random(&mut setup, 1);
    assert_eq!(
        setup.validate(&rules),
        Err(GameSetupError::NoFreeHero(BLUE))
    );
}

// covers: TWN-SET-007, TWN-SET-005
#[test]
fn a_mask_leaves_a_colour_out_of_a_type() {
    let rules = towns_rules();
    let id = |k| hero(&rules, k);
    let mut setup = towns_setup(&rules);
    setup.allowed_heroes = types(&rules, &["t:k0", "t:k1"]);
    setup.hero_masks = vec![(id("t:k0"), 0b10)];
    random(&mut setup, 0);
    random(&mut setup, 1);
    // Red (colour 0) may not take k0: it draws k1 at every seed, blue takes k0.
    for seed in 0..8 {
        let game = GameState::new(&rules, &setup, seed).unwrap();
        let ty = |index| game.hero(HeroId::new(index)).unwrap().hero_type;
        assert_eq!((ty(0), ty(1)), (id("t:k1"), id("t:k0")));
    }
    // Leaving red out of every free type refuses red's draw.
    setup.hero_masks.push((id("t:k1"), 0b10));
    assert_eq!(setup.validate(&rules), Err(GameSetupError::NoFreeHero(RED)));
    // A colour past a mask's bits takes no type.
    setup.hero_masks.clear();
    let core = rules.core();
    let pool = HeroPool::new(core, &setup);
    let mut draws = Rng::for_stream(0, StreamId::SETUP);
    let castle = Some(faction(&rules, "t:town"));
    assert!(pool.draw(core, castle, 0, &mut draws).is_some());
    assert!(pool.draw(core, castle, 9, &mut draws).is_none());
}

// covers: TWN-SET-005, TWN-SET-008
#[test]
fn twn_quirk_2_a_pick_is_reserved_before_any_draw() {
    let rules = towns_rules();
    let id = |k| hero(&rules, k);
    let mut setup = towns_setup(&rules);
    setup.players[1].start.choice = Some(FactionChoice::Fixed(faction(&rules, "t:town")));
    setup.allowed_heroes = types(&rules, &["t:loner", "t:k0"]);
    setup.players[0].start.pick = Some(id("t:loner"));
    // Blue's random hero draws before red's, which takes red's pick: blue never draws it.
    random(&mut setup, 1);
    add_random(&mut setup, RED, 0);
    for seed in 0..16 {
        assert_eq!(drawn(&rules, &setup, seed, 1), id("t:k0"));
        assert_eq!(drawn(&rules, &setup, seed, 2), id("t:loner"));
    }
}

// covers: TWN-SET-005, TWN-SET-008
#[test]
fn twn_quirk_3_a_random_hero_draws_no_later_placed_heros_type() {
    let rules = towns_rules();
    let id = |k| hero(&rules, k);
    let mut setup = towns_setup(&rules);
    // Red's random hero comes first; blue's placed hero (k0) and a prisoner (k1) later.
    random(&mut setup, 0);
    setup.heroes[1].hero_type = Some(id("t:k0"));
    setup.heroes[1].army = setup.heroes[0].army.clone();
    setup.prisoners = vec![id("t:k1")];
    setup.allowed_heroes = types(&rules, &["t:k0", "t:k1", "t:k2"]);
    for seed in 0..16 {
        assert_eq!(drawn(&rules, &setup, seed, 0), id("t:k2"));
    }
}
