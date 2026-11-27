//! The teams in the adventure (GAM-TEM-001, GAM-TEM-002; E10 T3a, E10 design note §1 to §3): the
//! seats' teams checked dense, `allied` by state, one explored layer a team.

use h3_contracts::PlayerId;

use super::{allies, build, error, setup};
use crate::setup::{Pos, SetupError};

const RED: PlayerId = PlayerId::new(0);
const BLUE: PlayerId = PlayerId::new(1);

// covers: GAM-TEM-001
#[test]
fn the_seats_teams_must_be_dense() {
    // A team at or past the number of players.
    assert_eq!(error(|s| s.seats[1].team = 2), SetupError::Team(2));
    // A team below the highest used that no seat holds.
    assert_eq!(
        error(|s| {
            s.seats[0].team = 1;
            s.seats[1].team = 1;
        }),
        SetupError::Team(0)
    );
    // One team for both, or each its own, builds.
    assert!(build(&allies(setup())).is_ok());
    let mut swapped = setup();
    swapped.seats[0].team = 1;
    swapped.seats[1].team = 0;
    assert!(build(&swapped).is_ok());
}

// covers: GAM-TEM-001
#[test]
fn players_of_one_team_are_allies() {
    let state = build(&allies(setup())).unwrap();
    assert!(state.allied(RED, BLUE) && state.allied(BLUE, RED));
    assert_eq!(state.map().team(BLUE), Some(0));
    let apart = build(&setup()).unwrap();
    assert!(!apart.allied(RED, BLUE));
    assert!(apart.allied(BLUE, BLUE));
    assert_eq!(apart.map().team(BLUE), Some(1));
    // A player past the game's has no team and is allied with itself alone.
    let past = PlayerId::new(5);
    assert_eq!(apart.map().team(past), None);
    assert!(apart.allied(past, past));
    assert!(!apart.allied(past, RED) && !apart.allied(RED, past));
    assert!(!apart.allied(PlayerId::new(9), RED));
    assert_eq!(apart.explored(past), None);
    assert_eq!(apart.explored(PlayerId::new(9)), None);
}

// covers: GAM-TEM-002
#[test]
fn a_team_shares_one_explored_layer() {
    // Blue's hero underground, Red's on the surface: each start reveals its own level.
    let mut apart_setup = setup();
    apart_setup.heroes[0].pos = Pos::new(0, 0, 1);
    let below = Pos::new(0, 0, 1);
    let above = Pos::new(0, 0, 0);
    let apart = build(&apart_setup).unwrap();
    assert!(apart.is_explored(BLUE, below) && !apart.is_explored(RED, below));
    assert!(apart.is_explored(RED, above) && !apart.is_explored(BLUE, above));
    let shared = build(&allies(apart_setup)).unwrap();
    // The team's layer holds each member's reveals, and each member reads it.
    assert_eq!(shared.explored(RED), shared.explored(BLUE));
    for player in [RED, BLUE] {
        assert!(shared.is_explored(player, below) && shared.is_explored(player, above));
    }
    // Each viewer test walks its team's layer.
    let layers: alloc::vec::Vec<_> = shared.layers().collect();
    assert_eq!(layers.len(), 2);
    assert!(core::ptr::eq(layers[0].0, layers[1].0));
    assert!(!core::ptr::eq(
        apart.layers().next().unwrap().0,
        apart.layers().nth(1).unwrap().0
    ));
    // The layers and the teams are state (adventure state hash v31).
    assert_ne!(shared.state_hash(), apart.state_hash());
}
