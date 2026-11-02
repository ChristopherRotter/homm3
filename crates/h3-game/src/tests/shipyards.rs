//! Tests of the Shipyards through the game (E08 T84b2; increment 9's design note §7, §8, §14): a
//! visit's boat offer opens as its visitor's owner's decision when the treasury can pay, and an
//! accepted one pays the price and builds the boat; `BuildBoat` builds at an own Shipyard without
//! a visit, checked by the adventure first, then against the treasury, then paid; the legal
//! commands list the Shipyards that can build now.

use h3_adventure::{
    AdventureCommand, AdventureCommandError, AdventureEvent, AdventureEventKind, BoatView,
    ObjectSetup, ObjectTypeId, OfferKind, ShipyardBlock, TileMask, Viewers,
};

use super::decisions::{ACCEPT, DECLINE, GOLD, after_steps, move_hero, play};
use super::*;

const RED: PlayerId = PlayerId::new(0);
const BLUE: PlayerId = PlayerId::new(1);
const RED_HERO: HeroId = HeroId::new(0);
/// The test part's Shipyard's price in gold (`tests/mod.rs`).
const PRICE: u32 = 1500;
/// A Shipyard's entrance on the top row and its boat tile, the original's second offset (2, 0).
const TOP: (Pos, Pos) = (Pos::new(1, 0, 0), Pos::new(3, 0, 0));
/// A Shipyard's entrance on the bottom row and its boat tile.
const BOTTOM: (Pos, Pos) = (Pos::new(1, 3, 0), Pos::new(3, 3, 0));

/// A Shipyard at `anchor`, `owner`'s or neutral, entered there.
fn shipyard(rules: &GameRuleset, anchor: Pos, owner: Option<PlayerId>) -> ObjectSetup {
    ObjectSetup {
        scenery: Default::default(),
        anchor,
        blocked: TileMask::from_bits(1),
        visitable: TileMask::from_bits(1),
        object_type: rules.adventure().object_type_by_key("t:shipyard"),
        amount: None,
        drawn: TileMask::default(),
        monster: None,
        guards: None,
        holds: h3_adventure::Holds::Nothing,
        owner,
    }
}

/// [`setup`] (difficulty 1: 2 000 gold) with these Shipyards and the sea on `water`; red's hero
/// at (0, 0), blue's at (2, 2), each exploring the whole 4 × 4 map.
fn game(rules: &GameRuleset, shipyards: Vec<ObjectSetup>, water: &[Pos]) -> GameState {
    let mut setup = setup();
    for pos in water {
        let index = usize::from(pos.y) * 4 + usize::from(pos.x);
        setup.map.tiles[index].terrain = TerrainId::from_raw(2);
    }
    setup.map.objects = shipyards;
    GameState::new(rules, &setup, 7).unwrap()
}

fn shipyard_type(rules: &GameRuleset) -> ObjectTypeId {
    rules.adventure().object_type_by_key("t:shipyard").unwrap()
}

fn build(object_type: ObjectTypeId, anchor: Pos) -> GameCommand {
    GameCommand::Adventure(AdventureCommand::BuildBoat {
        object_type,
        anchor,
    })
}

fn gold(state: &GameState, player: PlayerId) -> u32 {
    state.treasury(player).unwrap()[usize::from(GOLD.raw())]
}

fn adventure(kind: AdventureEventKind, seen_by: Viewers) -> GameEvent {
    GameEvent::Adventure(AdventureEvent { kind, seen_by })
}

// covers: ADV-SHY-002, ADV-OWN-001, GAM-DEC-001
#[test]
fn a_shipyard_s_visit_offers_its_boat_which_the_game_builds_for_its_price() {
    let rules = rules();
    let ty = shipyard_type(&rules);
    let (anchor, tile) = TOP;
    let both = Viewers::NONE.with(RED).with(BLUE);
    let mut state = game(&rules, vec![shipyard(&rules, anchor, None)], &[tile]);
    let before = gold(&state, RED);
    // The neutral Shipyard flagged for red, then its boat offered for the price, which red can
    // pay: the offer opens.
    let events = after_steps(play(&mut state, &rules, RED, move_hero(0, &[anchor])).unwrap());
    assert_eq!(
        events[0],
        adventure(
            AdventureEventKind::ObjectFlagged {
                object_type: ty,
                anchor,
                owner: RED,
            },
            both
        )
    );
    assert_eq!(
        events.last(),
        Some(&GameEvent::OfferOpened {
            player: RED,
            hero: RED_HERO,
            object_type: ty,
            anchor,
        })
    );
    assert_eq!(
        state.decision(RED).map(|offer| offer.kind.clone()),
        Some(OfferKind::Boat {
            price: vec![(GOLD, PRICE)]
        })
    );
    assert_eq!(
        legal_commands(&state, &rules, Actor::Player(RED)),
        [ACCEPT, DECLINE]
    );
    // The open offer is hashed with its price: declined, the state hashes otherwise.
    let mut declined = state.clone();
    play(&mut declined, &rules, RED, DECLINE).unwrap();
    assert_ne!(state.state_hash(), declined.state_hash());
    // Accepted: the price paid, then the boat built, seen by both players (each sees the tile).
    let events = play(&mut state, &rules, RED, ACCEPT).unwrap();
    assert_eq!(
        events,
        [
            GameEvent::OfferAnswered {
                player: RED,
                object_type: ty,
                anchor,
                accepted: true,
            },
            GameEvent::ResourcePaid {
                player: RED,
                resource: GOLD,
                amount: PRICE,
            },
            adventure(AdventureEventKind::BoatBuilt { tile, boat_type: 1 }, both),
        ]
    );
    assert_eq!(gold(&state, RED), before - PRICE);
    let boat = BoatView {
        tile,
        kind: 1,
        facing: 2,
    };
    for player in [RED, BLUE] {
        let view = observe(&state, &rules, Viewer::Player(player)).adventure;
        assert_eq!(view.boats, [boat]);
    }
    // The tile taken: no `BuildBoat` listed and the command refused whole; a second visit tells
    // red why, no offer.
    assert_eq!(
        legal_commands(&state, &rules, Actor::Player(RED)),
        [GameCommand::EndTurn]
    );
    assert_eq!(
        play(&mut state, &rules, RED, build(ty, anchor)),
        Err(GameCommandError::Adventure(
            AdventureCommandError::BoatTileTaken
        ))
    );
    play(&mut state, &rules, RED, move_hero(0, &[Pos::new(0, 0, 0)])).unwrap();
    let events = after_steps(play(&mut state, &rules, RED, move_hero(0, &[anchor])).unwrap());
    assert_eq!(
        events,
        [adventure(
            AdventureEventKind::ShipyardBlocked {
                object_type: ty,
                anchor,
                reason: ShipyardBlock::Taken,
            },
            Viewers::NONE.with(RED)
        )]
    );
    assert!(!state.has_decision(RED));
}

// covers: ADV-SHY-002, ADV-SHY-003, GAM-DEC-001
#[test]
fn build_boat_needs_an_own_shipyard_a_free_tile_and_the_price() {
    let rules = rules();
    let ty = shipyard_type(&rules);
    let both = Viewers::NONE.with(RED).with(BLUE);
    let shipyards = vec![
        shipyard(&rules, TOP.0, Some(RED)),
        shipyard(&rules, BOTTOM.0, Some(RED)),
    ];
    let mut state = game(&rules, shipyards, &[TOP.1, BOTTOM.1]);
    assert_eq!(gold(&state, RED), 2000);
    // Both of red's Shipyards can build: listed after `EndTurn`, in object order.
    assert_eq!(
        legal_commands(&state, &rules, Actor::Player(RED)),
        [GameCommand::EndTurn, build(ty, TOP.0), build(ty, BOTTOM.0)]
    );
    // Built without a hero: the boat, then the price.
    let events = play(&mut state, &rules, RED, build(ty, TOP.0)).unwrap();
    assert_eq!(
        events,
        [
            adventure(
                AdventureEventKind::BoatBuilt {
                    tile: TOP.1,
                    boat_type: 1
                },
                both
            ),
            GameEvent::ResourcePaid {
                player: RED,
                resource: GOLD,
                amount: PRICE,
            },
        ]
    );
    assert_eq!(gold(&state, RED), 2000 - PRICE);
    assert!(state.adventure().is_empty_boat(TOP.1));
    // The other's price is more than red holds now: not listed, refused whole.
    assert_eq!(
        legal_commands(&state, &rules, Actor::Player(RED)),
        [GameCommand::EndTurn]
    );
    assert_eq!(
        play(&mut state, &rules, RED, build(ty, BOTTOM.0)),
        Err(GameCommandError::CannotPay)
    );
    // A visit of the other: red's own, so no flag; its offer does not open (`OfferRefused`).
    let path = [Pos::new(0, 1, 0), Pos::new(0, 2, 0), BOTTOM.0];
    let events = after_steps(play(&mut state, &rules, RED, move_hero(0, &path)).unwrap());
    assert_eq!(
        events,
        [GameEvent::OfferRefused {
            player: RED,
            hero: RED_HERO,
            object_type: ty,
            anchor: BOTTOM.0,
        }]
    );
    assert!(!state.has_decision(RED));
    // Blue, in its turn, builds at none of red's.
    play(&mut state, &rules, RED, GameCommand::EndTurn).unwrap();
    assert_eq!(
        legal_commands(&state, &rules, Actor::Player(BLUE)),
        [GameCommand::EndTurn]
    );
    assert_eq!(
        play(&mut state, &rules, BLUE, build(ty, BOTTOM.0)),
        Err(GameCommandError::Adventure(AdventureCommandError::NotOwner))
    );
}
