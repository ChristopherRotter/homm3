//! Tests of the Shipyards (E08 T84b2; increment 9's design note §7, §8): a visit flags a neutral
//! Shipyard or another team's, then offers a boat on its free boat tile or tells its visitor's
//! owner why it cannot; the offer stands while the tile stays free and builds an empty boat of the
//! Shipyard's type there; `BuildBoat` builds at an own Shipyard, checked in order without
//! revealing a hidden one.

use super::economy::{GRASS_ROWS, Rules, object, owned, rules, setup};
use super::*;
use crate::ShipyardBlock;
use crate::testing::started;

const RED: PlayerId = PlayerId::new(0);
const BLUE: PlayerId = PlayerId::new(1);
const HERO: HeroId = HeroId::new(1);
const GOLD: ResourceId = ResourceId::from_raw(1);
/// The Shipyards' entrance in these tests, two steps right of red's hero at (0, 7).
const ENTRANCE: Pos = Pos::new(2, 7, 0);
/// Its boat tile, the original's second offset (2, 0): the bottom row's only water.
const TILE: Pos = Pos::new(4, 7, 0);

/// The grass map with water on [`TILE`] (an empty boat there with `boat`).
fn rows(boat: bool) -> Vec<&'static str> {
    let mut rows = GRASS_ROWS.to_vec();
    rows[7] = if boat {
        "ggggbggggggggggg"
    } else {
        "gggg~ggggggggggg"
    };
    rows
}

fn shipyard(rules: &Rules) -> ObjectTypeId {
    rules.1.object_type_by_key("t:shipyard").unwrap()
}

/// Red's hero's walk onto [`ENTRANCE`]: the outcome and the events after its steps.
fn visit(rules: &Rules, state: &mut AdventureState) -> (AdventureOutcome, Vec<AdventureEvent>) {
    let mut events = Vec::new();
    let walk = AdventureCommand::MoveHero {
        hero: HERO,
        path: vec![Pos::new(1, 7, 0), ENTRANCE],
    };
    let outcome = state
        .apply(&rules.0, &rules.1, &[], RED, &walk, &mut events)
        .unwrap();
    events.retain(|e| !matches!(e.kind, AdventureEventKind::HeroStepped { .. }));
    (outcome, events)
}

fn kinds(events: &[AdventureEvent]) -> Vec<AdventureEventKind> {
    events.iter().map(|e| e.kind).collect()
}

fn told(kind: AdventureEventKind, player: PlayerId) -> AdventureEvent {
    AdventureEvent {
        kind,
        seen_by: Viewers::NONE.with(player),
    }
}

fn accept(rules: &Rules, state: &mut AdventureState, offer: &Offer) -> Vec<AdventureEvent> {
    let mut events = Vec::new();
    let outcome = state.accept_offer((&rules.0, &rules.1, &[]), RED, offer, &mut events);
    assert_eq!(outcome, AdventureOutcome::default());
    events
}

// covers: ADV-OWN-001, ADV-SHY-002
#[test]
fn a_shipyard_s_visit_flags_it_then_offers_a_boat_on_its_tile() {
    let rules = rules();
    let ty = shipyard(&rules);
    let mut s = started(
        &rules,
        &setup(&rows(false), vec![object(&rules, 2, 7, "t:shipyard")]),
    )
    .unwrap();
    assert_eq!(s.boats().count(), 0);
    // The neutral Shipyard flagged for red and revealed around, then the boat offered for the
    // kind's price; entered, so the walk ends on it.
    let (outcome, events) = visit(&rules, &mut s);
    assert_eq!(s.hero(HERO).unwrap().pos, ENTRANCE);
    assert_eq!(s.owner(0), Some(RED));
    let told_kinds = kinds(&events);
    assert!(matches!(
        told_kinds[..],
        [
            AdventureEventKind::ObjectFlagged { owner: RED, .. },
            AdventureEventKind::TilesRevealed { .. }
        ]
    ));
    let offer = Offer {
        hero: HERO,
        object_type: ty,
        anchor: ENTRANCE,
        kind: OfferKind::Boat {
            price: vec![(GOLD, 1000)],
        },
    };
    assert_eq!(outcome.offer.as_ref(), Some(&offer));
    assert_eq!(offer.price(), [(GOLD, 1000)]);
    assert!(s.offer_stands(&offer));
    // Accepted: the map's first boat, empty on the tile, of the Shipyard's type, facing 2, red its
    // owner, no captain; told to red, not to blue, which does not see the tile.
    let before = s.state_hash();
    let events = accept(&rules, &mut s, &offer);
    assert_eq!(
        events,
        [told(
            AdventureEventKind::BoatBuilt {
                tile: TILE,
                boat_type: 1
            },
            RED
        )]
    );
    assert!(!s.is_explored(BLUE, TILE));
    assert_eq!(
        s.boat(0),
        Some(&Boat {
            tile: TILE,
            kind: 1,
            facing: 2,
            owner: Some(RED),
            captain: None,
            aboard: false,
            gone: false,
        })
    );
    assert_eq!(s.boats().count(), 1);
    assert!(s.is_empty_boat(TILE) && s.check_boats(&rules.1));
    assert_ne!(s.state_hash(), before);
    // The tile is taken now: the offer no longer stands, and red's next visit of its own Shipyard
    // flags nothing and tells it why there is no offer.
    assert!(!s.offer_stands(&offer));
    let back = AdventureCommand::MoveHero {
        hero: HERO,
        path: vec![Pos::new(1, 7, 0)],
    };
    s.apply(&rules.0, &rules.1, &[], RED, &back, &mut Vec::new())
        .unwrap();
    let again = AdventureCommand::MoveHero {
        hero: HERO,
        path: vec![ENTRANCE],
    };
    let mut events = Vec::new();
    let outcome = s
        .apply(&rules.0, &rules.1, &[], RED, &again, &mut events)
        .unwrap();
    events.retain(|e| !matches!(e.kind, AdventureEventKind::HeroStepped { .. }));
    assert_eq!(outcome, AdventureOutcome::default());
    assert_eq!(
        events,
        [told(
            AdventureEventKind::ShipyardBlocked {
                object_type: ty,
                anchor: ENTRANCE,
                reason: ShipyardBlock::Taken,
            },
            RED
        )]
    );
}

// covers: ADV-SHY-002
#[test]
fn a_shipyard_without_a_free_tile_tells_its_visitor_why() {
    let rules = rules();
    let ty = shipyard(&rules);
    let blocked = |reason| {
        told(
            AdventureEventKind::ShipyardBlocked {
                object_type: ty,
                anchor: ENTRANCE,
                reason,
            },
            RED,
        )
    };
    // No water among its offsets: flagged all the same, then no tile.
    let mut land = started(
        &rules,
        &setup(&GRASS_ROWS, vec![object(&rules, 2, 7, "t:shipyard")]),
    )
    .unwrap();
    let (outcome, events) = visit(&rules, &mut land);
    assert_eq!(outcome, AdventureOutcome::default());
    assert_eq!(land.owner(0), Some(RED));
    assert_eq!(events.len(), 3);
    assert_eq!(events[2], blocked(ShipyardBlock::NoTile));
    // A hero in a boat on the tile: taken.
    let mut s = started(
        &rules,
        &setup(&rows(true), vec![object(&rules, 2, 7, "t:shipyard")]),
    )
    .unwrap();
    s.put_aboard(HeroId::new(2), 0);
    assert!(!s.is_empty_boat(TILE));
    let (outcome, events) = visit(&rules, &mut s);
    assert_eq!(outcome, AdventureOutcome::default());
    assert_eq!(events.last(), Some(&blocked(ShipyardBlock::Taken)));
}

// covers: ADV-SHY-002
#[test]
fn a_boat_offer_stands_while_its_tile_is_free() {
    let rules = rules();
    let mut s = started(
        &rules,
        &setup(&rows(false), vec![object(&rules, 2, 7, "t:shipyard")]),
    )
    .unwrap();
    let offer = visit(&rules, &mut s).0.offer.unwrap();
    // Another boat ends on the tile before the answer: the offer no longer stands, and accepting
    // it anyway builds nothing.
    s.launch(TILE, 0, BLUE);
    assert!(!s.offer_stands(&offer));
    assert_eq!(accept(&rules, &mut s, &offer), []);
    assert_eq!(s.boats().count(), 1);
    assert_eq!(s.boat(0).map(|b| (b.kind, b.owner)), Some((0, Some(BLUE))));
    // A Shipyard's offer under another type or anchor names no object.
    let elsewhere = Offer {
        anchor: Pos::new(3, 7, 0),
        ..offer.clone()
    };
    assert!(!s.offer_stands(&elsewhere));
}

// covers: ADV-SHY-003
#[test]
fn build_boat_checks_the_shipyard_its_owner_and_its_tile_in_order() {
    let rules = rules();
    let (core, part) = (&rules.0, &rules.1);
    let ty = shipyard(&rules);
    let windmill = part.object_type_by_key("t:windmill").unwrap();
    // Red's at the entrance with its tile; a neutral one red sees; blue's beyond red's sight,
    // without a tile; red's at (8, 1) without a tile.
    let (neutral, blues, landlocked) = (Pos::new(2, 4, 0), Pos::new(13, 7, 0), Pos::new(8, 1, 0));
    let objects = vec![
        owned(&rules, 2, 7, "t:shipyard", 0),
        object(&rules, 2, 4, "t:shipyard"),
        owned(&rules, 13, 7, "t:shipyard", 1),
        owned(&rules, 8, 1, "t:shipyard", 0),
        object(&rules, 6, 4, "t:windmill"),
    ];
    let mut s = started(&rules, &setup(&rows(false), objects)).unwrap();
    let price = |player, object_type, anchor| s.boat_price(part, player, object_type, anchor);
    assert_eq!(price(RED, ty, ENTRANCE).cloned(), Ok(vec![(GOLD, 1000)]));
    // Not a Shipyard there, not a Shipyard's type, or one red has not seen: one error.
    assert!(!s.is_explored(RED, blues));
    for (object_type, anchor) in [
        (ty, Pos::new(3, 7, 0)),
        (windmill, Pos::new(6, 4, 0)),
        (ty, blues),
    ] {
        assert_eq!(
            price(RED, object_type, anchor),
            Err(AdventureCommandError::UnknownObject)
        );
    }
    // Seen but not red's; red's (blue's for blue) without a tile.
    assert_eq!(
        price(RED, ty, neutral),
        Err(AdventureCommandError::NotOwner)
    );
    assert_eq!(
        price(RED, ty, landlocked),
        Err(AdventureCommandError::NoBoatTile)
    );
    assert_eq!(
        price(BLUE, ty, blues),
        Err(AdventureCommandError::NoBoatTile)
    );
    let listed: Vec<_> = s.buildable_shipyards(part, RED).collect();
    assert_eq!(listed, [(ty, ENTRANCE, &vec![(GOLD, 1000)])]);
    assert_eq!(s.buildable_shipyards(part, BLUE).count(), 0);
    // Built: the boat on the tile, told to red; no hero moves, no points spent.
    let points = s.hero(HERO).unwrap().movement;
    let command = AdventureCommand::BuildBoat {
        object_type: ty,
        anchor: ENTRANCE,
    };
    let mut events = Vec::new();
    let outcome = s
        .apply(core, part, &[], RED, &command, &mut events)
        .unwrap();
    assert_eq!(outcome, AdventureOutcome::default());
    assert_eq!(
        events,
        [told(
            AdventureEventKind::BoatBuilt {
                tile: TILE,
                boat_type: 1
            },
            RED
        )]
    );
    assert_eq!(s.hero(HERO).unwrap().movement, points);
    assert!(s.is_empty_boat(TILE));
    // Again: the tile is taken, rejected whole.
    let before = s.clone();
    let mut events = Vec::new();
    assert_eq!(
        s.apply(core, part, &[], RED, &command, &mut events),
        Err(AdventureCommandError::BoatTileTaken)
    );
    assert!(events.is_empty());
    assert_eq!(s, before);
    assert_eq!(s.buildable_shipyards(part, RED).count(), 0);
}
