//! Tests of a hired hero's placement (TWN-TAV-003; E09 T38a1, the town epic's design note §64):
//! the room and the entrance checked in order, each refusal changing nothing; the hero placed on
//! the entrance as the town's visitor, in id order, nothing revealed, told to the tile's viewers.

extern crate std;

use std::vec;
use std::vec::Vec;

use h3_content::{FactionId, Formation, Ruleset};
use h3_contracts::{HeroId, PlayerId, TownId};

use crate::ruleset::{AdventurePack, AdventureRuleset, ObjectKindDef, ObjectTypeDef, Visit};
use crate::setup::{AdventureSetup, Holds, ObjectSetup, Pos, Scenery, TileMask};
use crate::state::{AdventureState, HeroOnMap};
use crate::testing::{PEASANT, hero_at, key, movement_rules_with, started, world};
use crate::town::TownHeroError;

type Rules = (Ruleset, AdventureRuleset);

const RED: PlayerId = PlayerId::new(0);
const BLUE: PlayerId = PlayerId::new(1);
const TOWN: TownId = TownId::new(0);
const ENTRANCE: Pos = Pos::new(8, 3, 0);

/// [`movement_rules_with`] with a town type `t:town`.
fn rules() -> Rules {
    movement_rules_with(
        true,
        16,
        |_| {},
        |part: &mut AdventurePack| {
            part.object_types.push(ObjectTypeDef {
                key: key("t:town"),
                name: "Town".into(),
                visit: Visit::Enter,
                from_above: false,
                kind_mark: false,
                kind: ObjectKindDef::Town {
                    boat_offsets: Vec::new(),
                },
            });
        },
    )
}

/// A grass map of 16 × 8 with red's town anchored on [`ENTRANCE`] (its gate), red's hero 1 on
/// `first`, red's hero 4 on (1, 1) and blue's hero 3 on (14, 6).
fn game(rules: &Rules, first: Pos) -> AdventureState {
    let mut setup: AdventureSetup = world(
        &["gggggggggggggggg"; 8],
        1,
        vec![
            hero_at(1, 0, first, &[PEASANT]),
            hero_at(4, 0, Pos::new(1, 1, 0), &[PEASANT]),
            hero_at(3, 1, Pos::new(14, 6, 0), &[PEASANT]),
        ],
    );
    setup.map.objects = vec![ObjectSetup {
        scenery: Scenery::Ownable,
        anchor: ENTRANCE,
        blocked: TileMask::from_bits(0b11),
        visitable: TileMask::from_bits(1),
        object_type: rules.1.object_type_by_key("t:town"),
        amount: None,
        drawn: TileMask::default(),
        monster: None,
        guards: None,
        holds: Holds::Town {
            formation: Formation::Loose,
        },
        owner: Some(RED),
    }];
    setup.town_factions = vec![FactionId::from_raw(0)];
    setup.garrisons = vec![Vec::new()];
    started(rules, &setup).unwrap()
}

/// A hired hero of red's on [`ENTRANCE`] with an away record of 1 200 of 1 500 points.
fn hired(id: u16) -> HeroOnMap {
    HeroOnMap {
        id: HeroId::new(id),
        owner: RED,
        pos: ENTRANCE,
        army: vec![h3_contracts::ArmyStack {
            slot: 0,
            creature: PEASANT,
            count: 1,
        }],
        formation: Formation::Tight,
        tactics: false,
        movement: 1200,
        boat: None,
        max_movement: 1500,
    }
}

// covers: TWN-TAV-003
#[test]
fn a_hire_needs_room_on_the_map_and_a_free_entrance() {
    let rules = rules();
    let away = game(&rules, Pos::new(5, 5, 0));
    assert_eq!(away.check_tavern_hire(RED, (TOWN, 8)), Ok(ENTRANCE));
    // Red's two heroes on the map fill a most of 2: the room comes before the entrance.
    assert_eq!(
        away.check_tavern_hire(RED, (TOWN, 2)),
        Err(TownHeroError::TooManyHeroes)
    );
    // Blue's one hero leaves it room; the check is the caller's player's.
    assert_eq!(away.check_tavern_hire(BLUE, (TOWN, 2)), Ok(ENTRANCE));
    // A visitor on the entrance, red's or another player's, holds it.
    let visited = game(&rules, ENTRANCE);
    assert_eq!(
        visited.check_tavern_hire(RED, (TOWN, 8)),
        Err(TownHeroError::EntranceHeld)
    );
    assert_eq!(
        visited.check_tavern_hire(BLUE, (TOWN, 8)),
        Err(TownHeroError::EntranceHeld)
    );
    assert_eq!(
        away.check_tavern_hire(RED, (TownId::new(1), 8)),
        Err(TownHeroError::UnknownTown)
    );
}

// covers: TWN-TAV-003, TWN-VIS-002
#[test]
fn a_hired_hero_stands_on_the_entrance_as_the_visitor_with_its_away_record() {
    let rules = rules();
    let mut state = game(&rules, Pos::new(5, 5, 0));
    let explored: Vec<u32> = [RED, BLUE]
        .map(|p| state.explored(p).unwrap().count())
        .into();
    let seen_by = state.place_hired(&rules.0, hired(2));
    assert_eq!(seen_by, state.viewers(&rules.0, RED, ENTRANCE));
    assert!(seen_by.contains(RED));
    // In id order among the heroes on the map, with the record as given, on land.
    let ids: Vec<u16> = state.heroes().iter().map(|h| h.id.raw()).collect();
    assert_eq!(ids, [1, 2, 3, 4]);
    assert_eq!(state.hero(HeroId::new(2)), Some(&hired(2)));
    assert_eq!(state.town_visitor(TOWN), Some(HeroId::new(2)));
    // Nothing is revealed.
    let after: Vec<u32> = [RED, BLUE]
        .map(|p| state.explored(p).unwrap().count())
        .into();
    assert_eq!(after, explored);
    // The entrance is held now.
    assert_eq!(
        state.check_tavern_hire(RED, (TOWN, 8)),
        Err(TownHeroError::EntranceHeld)
    );
}

// covers: GAM-DEC-010
#[test]
fn a_map_tavern_s_hire_needs_room_on_the_map_and_its_tile_free() {
    // E09 T39a (the town epic's design note §66): the room first, then any hero on the Tavern's
    // visitable tile, another player's included; no town and no visitor are read.
    let rules = rules();
    let tile = Pos::new(11, 2, 0);
    let away = game(&rules, Pos::new(5, 5, 0));
    assert!(away.tile_free(tile));
    assert_eq!(away.check_map_tavern_hire(RED, (tile, 8)), Ok(()));
    assert_eq!(
        away.check_map_tavern_hire(RED, (tile, 2)),
        Err(TownHeroError::TooManyHeroes)
    );
    assert_eq!(away.check_map_tavern_hire(BLUE, (tile, 2)), Ok(()));
    let held = game(&rules, tile);
    assert!(!held.tile_free(tile));
    for player in [RED, BLUE] {
        assert_eq!(
            held.check_map_tavern_hire(player, (tile, 8)),
            Err(TownHeroError::EntranceHeld)
        );
    }
    // A hired hero placed there holds it.
    let mut placed = game(&rules, Pos::new(5, 5, 0));
    placed.place_hired(
        &rules.0,
        HeroOnMap {
            pos: tile,
            ..hired(2)
        },
    );
    assert_eq!(placed.hero(HeroId::new(2)).map(|h| h.pos), Some(tile));
    assert!(!placed.tile_free(tile));
}
