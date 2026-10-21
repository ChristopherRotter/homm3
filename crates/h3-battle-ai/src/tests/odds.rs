//! The computer's odds (0x420A80; BAT-AI-008): the strength at range, the turrets with and
//! without the original's slip 20. The hero's value: `h3-battle`'s `tests/computer.rs`.

use h3_battle::{BattleState, computer};
use h3_contracts::{BattleSide, FortLevel, Moat};

use super::{ARCHER, GOG, IMP, at_town, ruleset, ruleset_as, setup};
use crate::odds::{stronger, turrets};

/// Whether `side` is the stronger at range in `state`.
fn is_stronger(state: &BattleState, profile: &str, side: BattleSide) -> bool {
    let r = ruleset_as(profile);
    stronger(state, &r, &computer::context(state, &r, side), side)
}

// covers: BAT-AI-008
#[test]
fn the_side_with_more_shooters_is_the_stronger_at_range() {
    let r = ruleset();
    // The defender's archers against walkers: the defender is the stronger, the attacker not.
    let state = BattleState::new(&r, &setup(&[(GOG, 50)], &[(ARCHER, 5)]), 1).unwrap();
    assert!(is_stronger(&state, "t:faithful", BattleSide::Defender));
    assert!(!is_stronger(&state, "t:faithful", BattleSide::Attacker));
    // No shooters: nobody (0 against 0).
    let state = BattleState::new(&r, &setup(&[(GOG, 50)], &[(IMP, 5)]), 1).unwrap();
    assert!(!is_stronger(&state, "t:faithful", BattleSide::Defender));
    assert!(!is_stronger(&state, "t:faithful", BattleSide::Attacker));
}

// covers: BAT-AI-008
#[test]
fn the_turrets_count_as_creature_2_with_the_slip_else_as_themselves() {
    let r = ruleset();
    let value = |profile: &str, fort: FortLevel, structures: u8| {
        let r = ruleset_as(profile);
        let setup = at_town(
            &r,
            (&[(GOG, 10)], &[(IMP, 10)]),
            (fort, Moat::Absent, structures),
        );
        let state = BattleState::new(&r, &setup, 1).unwrap();
        turrets(&state, &r)
    };
    // A Fort has no turrets; a Citadel the keep (4 creatures plus the structures); a Castle also
    // two towers at trunc((a + 1) × v / 2) each. The runner's 126 stands for the Archers'.
    assert_eq!(value("t:faithful", FortLevel::Fort, 0), 0);
    assert_eq!(value("t:faithful", FortLevel::Citadel, 0), 4 * 126);
    assert_eq!(value("t:faithful", FortLevel::Citadel, 3), 7 * 126);
    assert_eq!(
        value("t:faithful", FortLevel::Castle, 0),
        4 * 126 + 2 * (5 * 126 / 2)
    );
    assert_eq!(
        value("t:faithful", FortLevel::Castle, 1),
        5 * 126 + 2 * (6 * 126 / 2)
    );
    // Without the slip the tower's own 400.
    assert_eq!(value("t:fixed", FortLevel::Citadel, 0), 4 * 400);
    assert_eq!(
        value("t:fixed", FortLevel::Castle, 0),
        4 * 400 + 2 * (5 * 400 / 2)
    );
    // On the open field none, and the turrets make the defender the stronger.
    let state = BattleState::new(&r, &setup(&[(GOG, 10)], &[(IMP, 10)]), 1).unwrap();
    assert_eq!(turrets(&state, &r), 0);
    let town = at_town(
        &r,
        (&[(GOG, 10)], &[(IMP, 10)]),
        (FortLevel::Citadel, Moat::Absent, 0),
    );
    let state = BattleState::new(&r, &town, 1).unwrap();
    assert!(is_stronger(&state, "t:faithful", BattleSide::Defender));
}
