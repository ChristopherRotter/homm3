//! The walker's ends (BAT-AI-005's besieger's walk, BAT-AI-010, BAT-AI-011): the besieger's walk
//! to its row's wall, holding the walls with and without the original's slip 2, guarding a
//! shooter and the step to safety.

use h3_battle::{BattleState, Command, Field, Hex, StackId};
use h3_content_battle::WallKind;
use h3_contracts::{BattleSide, FortLevel, Moat};

use super::{
    ARCHER, CAVALIER, GOG, IMP, MAGOG, RUNNER, active, at_town, decide, on, play, ruleset, setup,
    touches,
};
use crate::ends::breaches;
use crate::walker::Walker;

/// The walker `id` of `side` in `state`.
fn walker<'a>(
    state: &'a BattleState,
    r: &'a h3_content_battle::BattleRuleset,
    (id, side): (u8, BattleSide),
) -> Walker<'a> {
    Walker::new(state, r, (StackId(id), side))
}

// covers: BAT-AI-005
#[test]
fn a_besieger_without_a_target_walks_to_its_rows_wall() {
    let r = ruleset();
    // The gogs stay behind the walls and the closed gate: the imps reach none and walk along
    // row 0 towards its wall (hex 12): its hex before the moat, 10 (11 is the moat), five steps.
    for (fort, moat) in [
        (FortLevel::Fort, Moat::Absent),
        (FortLevel::Citadel, Moat::Damaging),
    ] {
        let setup = at_town(&r, (&[(IMP, 10)], &[(GOG, 10)]), (fort, moat, 0));
        let state = BattleState::new(&r, &setup, 1).unwrap();
        assert_eq!(active(&state), IMP);
        assert_eq!(state.stacks()[0].hex, Hex(1));
        let want = if fort == FortLevel::Fort { 11 } else { 10 };
        let to = match decide(&state, &r) {
            Command::Move { to } => to,
            other => panic!("{fort:?}: {other:?}"),
        };
        let field = Field::new(r.battle().field);
        assert_eq!(field.distance(Hex(1), to), 5, "{fort:?}");
        assert_eq!(
            field.distance(to, Hex(want)),
            field.distance(Hex(1), Hex(want)) - 5,
            "{fort:?}"
        );
    }
    // A two-hex walker keeps both hexes out of the moat: the hex before it is no end for its
    // rear (its front would stand in the moat); it walks towards hex 9, two steps.
    let setup = at_town(
        &r,
        (&[(CAVALIER, 2)], &[(GOG, 10)]),
        (FortLevel::Citadel, Moat::Harmless, 0),
    );
    let state = BattleState::new(&r, &setup, 1).unwrap();
    let me = state.stacks()[0].hex;
    let w = walker(&state, &r, (0, BattleSide::Attacker));
    assert!(w.besieging());
    let Command::Move { to } = w.besiege() else {
        panic!("no walk");
    };
    let field = Field::new(r.battle().field);
    assert_eq!(field.distance(me, to), 2);
    assert_eq!(field.distance(to, Hex(9)), field.distance(me, Hex(9)) - 2);
    // The defender never besieges; on the open field nobody does.
    let state = BattleState::new(&r, &setup_open(), 1).unwrap();
    assert!(!walker(&state, &r, (0, BattleSide::Attacker)).besieging());
}

/// Imps against gogs on the open field.
fn setup_open() -> h3_contracts::BattleSetup {
    setup(&[(IMP, 10)], &[(GOG, 10)])
}

// covers: BAT-AI-011
#[test]
fn the_walls_test_counts_breaches_with_or_without_the_slip() {
    let walls = [
        (0, WallKind::Tower),
        (1, WallKind::Wall),
        (2, WallKind::Wall),
        (3, WallKind::Gate),
        (4, WallKind::Wall),
        (5, WallKind::Wall),
        (6, WallKind::Tower),
        (7, WallKind::Keep),
    ];
    for (t, kind) in walls {
        let wall = matches!(kind, WallKind::Wall | WallKind::Gate);
        // Standing parts and the turrets' never breach.
        assert!(!breaches((t, Some(kind)), true, true));
        assert!(!breaches((t, Some(kind)), true, false));
        // Without the slip every destroyed wall or the gate does; with it not the Mid-Upper or
        // the Lower Wall (the wrong records' hex 12, always blocked).
        assert_eq!(breaches((t, Some(kind)), false, false), wall, "{t}");
        assert_eq!(
            breaches((t, Some(kind)), false, true),
            wall && t != 2 && t != 5,
            "{t}"
        );
    }
    assert!(!breaches((9, None), false, false));
}

// covers: BAT-AI-011
#[test]
fn a_defender_inside_its_walls_holds_them_when_it_is_the_stronger() {
    let r = ruleset();
    // At a Citadel the keep makes the defender the stronger at range against walkers: its imps,
    // inside, hold the walls. Archers outside make the attacker the stronger instead.
    let holds = |attacker: &[(usize, u32)], fort: FortLevel| {
        let setup = at_town(&r, (attacker, &[(IMP, 10)]), (fort, Moat::Absent, 0));
        let mut state = BattleState::new(&r, &setup, 1).unwrap();
        while state.stacks()[state.active().unwrap().index()].side != BattleSide::Defender {
            play(&mut state, &r, Command::Defend);
        }
        let id = u8::try_from(state.active().unwrap().index()).unwrap();
        walker(&state, &r, (id, BattleSide::Defender)).holds_walls()
    };
    assert!(holds(&[(GOG, 10)], FortLevel::Citadel));
    assert!(!holds(&[(GOG, 10), (ARCHER, 30)], FortLevel::Citadel));
    // At a Fort there is no keep: nobody is the stronger at range, nobody holds.
    assert!(!holds(&[(GOG, 10)], FortLevel::Fort));
    // Defenders with shooters hold at a Fort too.
    let setup = at_town(
        &r,
        (&[(GOG, 10)], &[(IMP, 10), (ARCHER, 10)]),
        (FortLevel::Fort, Moat::Absent, 0),
    );
    let mut state = BattleState::new(&r, &setup, 1).unwrap();
    assert_eq!(active(&state), IMP);
    let defender_imps = u8::try_from(state.active().unwrap().index()).unwrap();
    assert!(walker(&state, &r, (defender_imps, BattleSide::Defender)).holds_walls());
    // Holding, the imps take no enemy outside: nothing to attack, they guard their archers (or
    // stay beside them).
    let command = decide(&state, &r);
    assert!(
        matches!(command, Command::Move { .. } | Command::Defend),
        "{command:?}"
    );
    if let Command::Move { to } = command {
        assert!(touches(&state, &r, to, StackId(defender_imps + 1)));
    }
    play(&mut state, &r, command);
    // The attacker never holds: the walls are the defender's.
    let state = BattleState::new(&r, &setup_open(), 1).unwrap();
    assert!(!walker(&state, &r, (0, BattleSide::Attacker)).holds_walls());
}

// covers: BAT-AI-010
#[test]
fn a_losing_walker_guards_its_shooters_when_its_side_is_the_stronger() {
    let r = ruleset();
    // Ten imps would lose against thirty gogs (a negative value, not desperate with a hundred
    // archers); with the archers the attacker is the stronger at range, so the imps guard the
    // archers instead, from the side they face: the hex right of them, in one turn.
    let state = BattleState::new(
        &r,
        &on(1, setup(&[(IMP, 10), (ARCHER, 100)], &[(GOG, 30)])),
        1,
    )
    .unwrap();
    assert_eq!(active(&state), IMP);
    let archers = StackId(1);
    let command = decide(&state, &r);
    let Command::Move { to } = command else {
        panic!("{command:?}");
    };
    let field = Field::new(r.battle().field);
    let a = state.stacks()[1].hex;
    let (col, row) = field.coords(a);
    assert!(touches(&state, &r, to, archers), "{to:?}");
    assert_eq!(field.coords(to), (col + 1, row));
    // Without the archers the attacker is not the stronger: the imps attack after all (they walk
    // towards the gogs).
    let state = BattleState::new(&r, &on(1, setup(&[(IMP, 10)], &[(GOG, 30)])), 1).unwrap();
    let Command::Move { to } = decide(&state, &r) else {
        panic!("no walk");
    };
    assert!(
        field.distance(to, state.stacks()[1].hex) < field.distance(Hex(1), state.stacks()[1].hex)
    );
    // With an area threat on them (the magogs' fireballs) they do not guard: they attack.
    let state = BattleState::new(
        &r,
        &on(
            1,
            setup(&[(IMP, 10), (ARCHER, 100)], &[(GOG, 30), (MAGOG, 5)]),
        ),
        1,
    )
    .unwrap();
    assert_eq!(active(&state), IMP);
    let command = decide(&state, &r);
    let Command::Move { to } = command else {
        panic!("{command:?}");
    };
    assert!(!touches(&state, &r, to, archers));
}

// covers: BAT-AI-010
#[test]
fn the_guard_defends_beside_its_shooter_and_the_step_to_safety_takes_the_best_hex() {
    let r = ruleset();
    let mut state = BattleState::new(
        &r,
        &on(2, setup(&[(RUNNER, 10), (ARCHER, 20)], &[(GOG, 30)])),
        1,
    )
    .unwrap();
    assert_eq!(active(&state), RUNNER);
    // The runners walk to their guard hex beside the archers, then defend there.
    let w = walker(&state, &r, (0, BattleSide::Attacker));
    let Some(Command::Move { to }) = w.guard() else {
        panic!("no guard");
    };
    play(&mut state, &r, Command::Move { to });
    while active(&state) != RUNNER {
        play(&mut state, &r, Command::Defend);
    }
    assert_eq!(
        walker(&state, &r, (0, BattleSide::Attacker)).guard(),
        Some(Command::Defend)
    );
    // The step to safety on Hard: from a hex scored −5 to the best hex it reaches this turn.
    let here = state.stacks()[0].hex;
    let field = Field::new(r.battle().field);
    let mut scores = [0; 256];
    scores[here.index()] = -5;
    let safe = field.neighbors(field.neighbors(here)[0].unwrap())[0].unwrap();
    scores[safe.index()] = 3;
    let w = walker(&state, &r, (0, BattleSide::Attacker)).scored(scores);
    assert_eq!(w.step_to_safety(), Some(Command::Move { to: safe }));
    // A score of 0 or more, or Normal, takes no step.
    let mut calm = scores;
    calm[here.index()] = 0;
    let w = walker(&state, &r, (0, BattleSide::Attacker)).scored(calm);
    assert_eq!(w.step_to_safety(), None);
    let normal = BattleState::new(&r, &on(1, setup(&[(RUNNER, 10)], &[(GOG, 30)])), 1).unwrap();
    let mut scores = [0; 256];
    scores[normal.stacks()[0].hex.index()] = -5;
    let w = walker(&normal, &r, (0, BattleSide::Attacker)).scored(scores);
    assert_eq!(w.step_to_safety(), None);
    // Nowhere better: no step.
    let w = walker(&state, &r, (0, BattleSide::Attacker)).scored([-5; 256]);
    assert_eq!(w.step_to_safety(), None);
}

// covers: BAT-AI-010
#[test]
fn the_guard_defends_on_the_last_shooters_hex_with_the_slip_else_on_the_bests() {
    // The runners stand on the guard hex of the five archers (the last shooter asked, beside it
    // on its right); the fifty archers are worth more, so their guard hex is the best. The
    // original tests the last answer (slip 21): the runners defend; without the slip they go to
    // the fifty archers' side.
    for (profile, slips) in [("t:faithful", true), ("t:fixed", false)] {
        let r = super::ruleset_as(profile);
        let mut state = BattleState::new(
            &r,
            &setup(&[(RUNNER, 10), (ARCHER, 50), (ARCHER, 5)], &[(GOG, 30)]),
            1,
        )
        .unwrap();
        assert_eq!(active(&state), RUNNER);
        let field = Field::new(r.battle().field);
        let [_, many, few] = [0, 1, 2].map(|i| state.stacks()[i].hex);
        let beside = |h: Hex| {
            let (col, row) = field.coords(h);
            field.hex(col + 1, row).unwrap()
        };
        play(&mut state, &r, Command::Move { to: beside(few) });
        while active(&state) != RUNNER {
            play(&mut state, &r, Command::Defend);
        }
        assert_eq!(state.stacks()[0].hex, beside(few));
        let guard = walker(&state, &r, (0, BattleSide::Attacker)).guard();
        let want = if slips {
            Command::Defend
        } else {
            Command::Move { to: beside(many) }
        };
        assert_eq!(guard, Some(want), "{profile}");
    }
}
