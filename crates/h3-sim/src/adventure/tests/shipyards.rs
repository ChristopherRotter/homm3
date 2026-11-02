//! The env's `BuildBoat` (E08 T85b2b, action encoding v6; ADV-SHY-003, ADR-0017 point 8): on the
//! duel with a Shipyard of each player's beside its first hero ([`shipyard_setup`]), the entry
//! [`BUILD_BOAT`] is legal while the game lists a `BuildBoat`, its tile head the anchors of the
//! listed Shipyards; another tile decodes to a `BuildBoat` only at a Shipyard the player may use
//! (which the game refuses out of its turn, with a decision open or without the price), else to
//! nothing; a built boat on the Shipyard's tile masks it; the random agent builds.

use h3_adventure::{AdventureEvent, AdventureEventKind, Pos};
use h3_game::GameEvent;

use super::decisions::legal_entries;
use super::*;

const RED: PlayerId = PlayerId::new(0);
const BLUE: PlayerId = PlayerId::new(1);

/// The Shipyard's boat tile's offset from its entrance in this setup: the original's first,
/// two columns left (`packs/base/adventure.toml`).
const OFFSET: (i16, i16) = (-2, 0);

/// The duel at `difficulty` with a Shipyard of each player's beside its first hero: on the
/// nearest free tile the hero reaches on day 1 whose tile [`OFFSET`] away is free and reached too,
/// made water (free: no object blocks it or is visited from it, the reach also lists objects'
/// entrances); the Shipyard's boat tile is then that water tile (ADV-SHY-001).
/// The Shipyards come after the duel's objects, red's first.
pub(super) fn shipyard_setup(rules: &GameRuleset, difficulty: u8) -> Arc<GameSetup> {
    let mut setup = GameSetup::clone(&setup(rules));
    setup.difficulty = difficulty;
    let shipyard = rules.adventure().object_type_by_key("base:shipyard");
    let water = rules.core().terrain_by_key("base:water").unwrap();
    assert!(shipyard.is_some(), "the original packs' Shipyard");
    let state = GameState::new(rules, &setup, 0).unwrap();
    let mut used: Vec<Pos> = Vec::new();
    for player in [RED, BLUE] {
        let hero = hero_in_slot(&state, player, 0).unwrap();
        let reach = state
            .adventure()
            .reach(rules.core(), rules.adventure(), state.heroes(), hero)
            .unwrap();
        let reached: Vec<(usize, Pos)> = reach
            .tiles()
            .filter_map(|(pos, _)| Some((reach.path(pos)?.len(), pos)))
            .collect();
        let map = state.adventure().map();
        let free = |pos: Pos| map.tile(pos).is_some_and(|t| !t.blocked && !t.visitable);
        let near =
            |pos: Pos| reached.iter().any(|&(_, p)| p == pos) && free(pos) && !used.contains(&pos);
        let (_, anchor, boat) = reached
            .iter()
            .filter_map(|&(steps, anchor)| {
                let x = anchor.x.checked_add_signed(OFFSET.0)?;
                let y = anchor.y.checked_add_signed(OFFSET.1)?;
                let boat = Pos::new(x, y, anchor.level);
                (near(anchor) && near(boat)).then_some((steps, anchor, boat))
            })
            .min_by_key(|&(steps, anchor, _)| (steps, tile_index(&state, anchor)))
            .unwrap();
        used.extend([anchor, boat]);
        let index = tile_index(&state, boat).unwrap() as usize;
        setup.map.tiles[index].terrain = water;
        setup.map.objects.push(ObjectSetup {
            scenery: Default::default(),
            anchor,
            blocked: TileMask::from_bits(1),
            visitable: TileMask::from_bits(1),
            object_type: shipyard,
            amount: None,
            drawn: TileMask::default(),
            monster: None,
            guards: None,
            holds: Default::default(),
            owner: Some(player),
        });
    }
    Arc::new(setup)
}

/// The anchors of [`shipyard_setup`]'s Shipyards: red's, then blue's.
pub(super) fn shipyards(setup: &GameSetup) -> Vec<Pos> {
    let n = setup.map.objects.len() - 2;
    setup.map.objects[n..].iter().map(|o| o.anchor).collect()
}

fn masked(tiles: &[bool]) -> Vec<u32> {
    (0u32..)
        .zip(tiles)
        .filter(|&(_, &t)| t)
        .map(|(i, _)| i)
        .collect()
}

fn boat_built(events: &[GameEvent]) -> bool {
    events.iter().any(|event| {
        matches!(
            event,
            GameEvent::Adventure(AdventureEvent {
                kind: AdventureEventKind::BoatBuilt { .. },
                ..
            })
        )
    })
}

// covers: ADV-SHY-003
#[test]
fn build_boat_is_an_entry_whose_tile_is_the_shipyard_s_anchor() {
    let rules = &*FAITHFUL;
    let setup = shipyard_setup(rules, 1);
    let mut env = AdventureEnv::new(rules.clone(), Arc::clone(&setup), 1, 0).unwrap();
    let state = env.state();
    let [red, blue] = shipyards(&setup)[..] else {
        panic!("two Shipyards")
    };
    let (red, blue) = (
        tile_index(state, red).unwrap(),
        tile_index(state, blue).unwrap(),
    );
    // Red's turn: the entry is legal, its tile head red's Shipyard's anchor alone.
    assert!(legal_entries(&env, RED).contains(&BUILD_BOAT));
    let mut tiles = Vec::new();
    env.tile_mask(RED, BUILD_BOAT, &mut tiles);
    assert_eq!(masked(&tiles), [red]);
    let command = action::decode(state, env.rules(), RED, Action::build_boat(red)).unwrap();
    assert!(matches!(
        command,
        GameCommand::Adventure(AdventureCommand::BuildBoat { .. })
    ));
    assert_eq!(
        action::encode(state, env.rules(), RED, &command),
        Some(Action::build_boat(red))
    );
    // Blue's own Shipyard decodes for blue, which the game refuses out of its turn; red's
    // Shipyard, a tile without one and one past the map decode to nothing; the neutral seat
    // builds nowhere.
    env.tile_mask(BLUE, BUILD_BOAT, &mut tiles);
    assert!(masked(&tiles).is_empty());
    assert_eq!(
        env.step(BLUE, Action::build_boat(blue)),
        Err(StepError::Rejected(GameCommandError::NotYourTurn(BLUE)))
    );
    for (seat, tile) in [
        (Seat::Player(BLUE), red),
        (Seat::Player(RED), blue),
        (Seat::Player(RED), 0),
        (Seat::Player(RED), u32::try_from(env.tile_count()).unwrap()),
        (Seat::Neutral, red),
    ] {
        assert_eq!(
            env.step(seat, Action::build_boat(tile)),
            Err(StepError::InvalidAction),
            "{seat:?} {tile}"
        );
    }
    assert_eq!(
        action::encode(env.state(), env.rules(), Seat::Neutral, &command),
        None
    );
    // Red builds: the boat on the Shipyard's tile, which then masks the entry.
    let mut events = Vec::new();
    env.step_with(RED, Action::build_boat(red), &mut events)
        .unwrap();
    assert!(boat_built(&events), "{events:?}");
    assert!(!legal_entries(&env, RED).contains(&BUILD_BOAT));
    env.tile_mask(RED, BUILD_BOAT, &mut tiles);
    assert!(masked(&tiles).is_empty());
    assert_eq!(
        env.step(RED, Action::build_boat(red)),
        Err(StepError::InvalidAction)
    );
}

#[test]
fn the_random_agent_builds_boats() {
    let rules = &*FAITHFUL;
    let mut env = AdventureEnv::new(rules.clone(), shipyard_setup(rules, 0), 3, 8).unwrap();
    let mut rng = Rng::for_stream(5, StreamId::AGENT);
    let mut built = 0;
    loop {
        let Some(seat) = env.to_move().next() else {
            break;
        };
        let action = choose(&env, seat, &mut rng);
        let mut events = Vec::new();
        let (_, command) = env.step_decoded(seat, action, &mut events).unwrap();
        if matches!(
            command,
            GameCommand::Adventure(AdventureCommand::BuildBoat { .. })
        ) {
            assert!(boat_built(&events), "{events:?}");
            built += 1;
        }
    }
    assert!(built > 0, "no boat built in a week");
}
