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

/// The army command `command` of red on `state` (no group), the state unchanged when refused.
fn manage(
    state: &mut crate::AdventureState,
    command: crate::ArmyCommand,
) -> Result<(), crate::ArmyCommandError> {
    let before = state.clone();
    let result = state.manage_army(&crate::testing::rules().1, RED, &command, None);
    if result.is_err() {
        assert_eq!(*state, before);
    }
    result
}

// covers: GAM-TEM-003, ADV-GAR-003, ADV-ARM-002
#[test]
fn a_command_s_target_may_be_an_ally_s_hero_onto_its_empty_slots_alone() {
    use crate::{ArmyCommand, ArmyCommandError, ArmyRef, ArmySlot};
    use h3_contracts::HeroId;

    use super::{GOG, IMP, stack};
    // Red's hero 1 with imps in slot 0 and gogs in slot 1, blue's hero 5 with imps in slot 0, of
    // the rules' three slots. E10 T3b1: blue on red's team, its hero is a bar not red's, so a
    // command onto it is a gift into an empty slot (the one owner rule); the game holds the
    // meeting that names it.
    let (red, blue) = (HeroId::new(1), HeroId::new(5));
    let at = |hero, slot| ArmySlot {
        army: ArmyRef::Hero(hero),
        slot,
    };
    let armies = |mut setup: crate::setup::AdventureSetup| {
        setup.heroes[0].army = alloc::vec![stack(0, IMP, 12)];
        setup.heroes[1].army = alloc::vec![stack(0, IMP, 5), stack(1, GOG, 2)];
        setup
    };
    let ally = allies(armies(setup()));
    let army = |state: &crate::AdventureState, hero| state.hero(hero).unwrap().army.clone();
    let mut state = build(&ally).unwrap();
    let give = ArmyCommand::Move {
        from: at(red, 1),
        to: at(blue, 1),
    };
    manage(&mut state, give).unwrap();
    assert_eq!(army(&state, red), [stack(0, IMP, 5)]);
    assert_eq!(army(&state, blue), [stack(0, IMP, 12), stack(1, GOG, 2)]);
    // Red's last stack never leaves (ADV-ARM-002).
    let last = ArmyCommand::Move {
        from: at(red, 0),
        to: at(blue, 2),
    };
    assert_eq!(manage(&mut state, last), Err(ArmyCommandError::LastStack));
    // A split into an empty slot gives too.
    let mut state = build(&ally).unwrap();
    let split = ArmyCommand::Split {
        from: at(red, 0),
        to: at(blue, 2),
        count: 2,
    };
    manage(&mut state, split).unwrap();
    assert_eq!(army(&state, red), [stack(0, IMP, 3), stack(1, GOG, 2)]);
    assert_eq!(army(&state, blue), [stack(0, IMP, 12), stack(2, IMP, 2)]);
    // Onto an occupied slot, of the same creature (a merge) or another (a swap), or a split onto
    // one: `OnlyEmpty`. From blue's hero, or a dismissal there: not red's hero.
    let mut state = build(&ally).unwrap();
    for from in [0, 1] {
        let command = ArmyCommand::Move {
            from: at(red, from),
            to: at(blue, 0),
        };
        assert_eq!(
            manage(&mut state, command),
            Err(ArmyCommandError::OnlyEmpty)
        );
    }
    let onto = ArmyCommand::Split {
        from: at(red, 0),
        to: at(blue, 0),
        count: 1,
    };
    assert_eq!(manage(&mut state, onto), Err(ArmyCommandError::OnlyEmpty));
    for command in [
        ArmyCommand::Move {
            from: at(blue, 0),
            to: at(red, 2),
        },
        ArmyCommand::Dismiss { at: at(blue, 0) },
    ] {
        assert_eq!(
            manage(&mut state, command),
            Err(ArmyCommandError::NotYourHero(blue))
        );
    }
    // The split's range: into an empty slot of blue's, none onto a stack of it.
    let part = crate::testing::rules().1;
    assert_eq!(
        state.split_range(&part, RED, (at(red, 0), at(blue, 1)), None),
        Some(1..=4)
    );
    assert_eq!(
        state.split_range(&part, RED, (at(red, 0), at(blue, 0)), None),
        None
    );
    // The listed commands onto blue's army are the moves into its two empty slots alone.
    let listed = state.army_commands(
        &part,
        RED,
        (ArmyRef::Hero(red), Some(ArmyRef::Hero(blue))),
        None,
    );
    let onto_blue: alloc::vec::Vec<_> = listed
        .iter()
        .filter_map(|command| match *command {
            ArmyCommand::Move { from, to } if to.army == ArmyRef::Hero(blue) => {
                Some((from.slot, to.slot))
            }
            _ => None,
        })
        .collect();
    assert_eq!(onto_blue, [(0, 1), (0, 2), (1, 1), (1, 2)]);
    // Not allied: blue's hero is no target at all.
    let mut apart = build(&armies(setup())).unwrap();
    assert_eq!(
        manage(&mut apart, give),
        Err(ArmyCommandError::NotYourHero(blue))
    );
}
