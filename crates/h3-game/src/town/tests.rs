//! Tests of a player's start and of the build's refusal of a setup that needs a hero drawn (E09
//! T7a1, `TWN-STUB-3`).

use alloc::string::ToString;
use alloc::vec;

use h3_content::{FactionId, HeroTypeId};
use h3_contracts::{HeroId, PlayerId, TownId};
use h3_towns::{BuildingsSetup, TownFaction, TownSetup};

use super::{FactionChoice, FactionSet, HeroDraw, MainTown, PlayerStart};
use crate::tests::{rules, setup};
use crate::{GameSetupError, GameState};

#[test]
fn a_faction_set_holds_the_factions_below_its_capacity() {
    let faction = |index| FactionId::from_index(index).unwrap();
    assert!(FactionSet::EMPTY.is_empty());
    assert_eq!(FactionSet::default(), FactionSet::EMPTY);
    let set = FactionSet::EMPTY
        .with(faction(8))
        .with(faction(0))
        .with(faction(8));
    assert_eq!((set.len(), set.is_empty()), (2, false));
    assert!(set.contains(faction(0)) && set.contains(faction(8)) && !set.contains(faction(1)));
    assert_eq!(
        set.iter().collect::<vec::Vec<_>>(),
        [faction(0), faction(8)]
    );
    let last = faction(FactionSet::CAPACITY - 1);
    assert_eq!(FactionSet::EMPTY.with(last).iter().last(), Some(last));
    // A faction past the capacity is in no set.
    let past = faction(FactionSet::CAPACITY);
    assert_eq!(set.with(past), set);
    assert!(!set.with(past).contains(past));
}

#[test]
fn a_players_start_defaults_to_none_of_it() {
    assert_eq!(
        PlayerStart::default(),
        PlayerStart {
            factions: FactionSet::EMPTY,
            choice: None,
            pick: None,
            main_town: None,
            random_hero: false,
        }
    );
}

/// A start with a main town, generating a hero or not, and everything else set.
fn start(generates_hero: bool) -> PlayerStart {
    let castle = FactionId::from_raw(0);
    PlayerStart {
        factions: FactionSet::EMPTY.with(castle),
        choice: Some(FactionChoice::Fixed(castle)),
        pick: Some(HeroTypeId::from_raw(0)),
        main_town: Some(MainTown {
            town: TownId::new(0),
            generates_hero,
        }),
        random_hero: true,
    }
}

// covers: TWN-SET-001
#[test]
fn the_build_ignores_the_towns_and_the_pools_inputs() {
    let plain = setup();
    let mut full = setup();
    full.players[1].start = start(false);
    full.players[0].start.choice = Some(FactionChoice::Random);
    full.towns = vec![TownSetup {
        faction: TownFaction::Random { alignment: Some(1) },
        buildings: BuildingsSetup::Standard { fort: true },
        garrison: None,
        name: Some("Steadwick".to_string()),
        events: vec![],
    }];
    full.allowed_heroes = vec![HeroTypeId::from_raw(0)];
    full.hero_masks = vec![(HeroTypeId::from_raw(0), 0b10)];
    full.prisoners = vec![HeroTypeId::from_raw(0)];
    assert_eq!(full.validate(&rules()), Ok(()));
    let built = |s| GameState::new(&rules(), s, 7).unwrap().state_hash();
    assert_eq!(built(&full), built(&plain));
}

#[test]
fn the_build_refuses_a_setup_that_needs_a_hero_drawn() {
    let refused = |setup: &crate::GameSetup| setup.validate(&rules()).unwrap_err();
    let mut random = setup();
    random.heroes[1].hero_type = None;
    let error = refused(&random);
    assert_eq!(
        error,
        GameSetupError::HeroDraw(HeroDraw::RandomHero(HeroId::new(1)))
    );
    assert_eq!(error.to_string(), "not built yet: hero 1 is a random hero");
    // A main town generating a hero is refused first, whatever hero is random.
    let mut generating = random.clone();
    generating.players[1].start = start(true);
    let error = refused(&generating);
    assert_eq!(
        error,
        GameSetupError::HeroDraw(HeroDraw::MainTown(PlayerId::new(1)))
    );
    assert_eq!(
        error.to_string(),
        "not built yet: player 1's main town generates a hero"
    );
    // Checked before any hero's type is read: an unknown type before the random hero is not
    // reached.
    random.heroes[0].hero_type = Some(HeroTypeId::from_raw(u16::MAX));
    assert_eq!(
        refused(&random),
        GameSetupError::HeroDraw(HeroDraw::RandomHero(HeroId::new(1)))
    );
    random.heroes[1].hero_type = Some(HeroTypeId::from_raw(0));
    assert_eq!(
        refused(&random),
        GameSetupError::UnknownHeroType(HeroId::new(0))
    );
}
