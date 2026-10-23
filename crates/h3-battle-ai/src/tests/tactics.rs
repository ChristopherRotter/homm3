//! The computer's tactics phase (BAT-AI-023): its end, the shooters' placement, the walkers'
//! walks to the zone's edge and their threat records over the whole field; and the commanded
//! First Aid Tent's patient (BAT-AI-022).

use std::vec;

use h3_battle::{BattleState, Command, Field, Hex, StackId, computer};
use h3_contracts::BattleSide;

use super::{
    ARCHER, CAVALIER, EFREET, GOG, IMP, RUNNER, TENT, cast_at, checked, creature, decide, hero,
    hero_with, on, play, ruleset, setup,
};
use crate::gain::records;
use crate::walker::Walker;

/// The battle of the attacker's hero with Tactics `tactics` leading `attacker` against
/// `defender` (no hero) on `difficulty`, in the attacker's tactics phase.
fn phase(
    tactics: u8,
    attacker: &[(usize, u32)],
    defender: &[(usize, u32)],
    difficulty: u8,
) -> BattleState {
    let r = ruleset();
    let mut setup = on(difficulty, setup(attacker, defender));
    setup.heroes = [Some(hero(tactics)), None];
    let state = BattleState::new(&r, &setup, 5).unwrap();
    assert!(state.tactics().is_some());
    state
}

/// The column and row of `hex`.
fn xy(hex: Hex) -> (i32, i32) {
    Field::new(ruleset().battle().field).coords(hex)
}

// covers: BAT-AI-023
#[test]
fn the_side_ends_its_phase_after_a_round_without_a_placement() {
    let r = ruleset();
    let mut state = phase(2, &[(IMP, 3)], &[(GOG, 2)], 1);
    // Round 1: the imp walks towards the gogs along its row to the zone's last column (5 of the
    // difference of 2).
    let start = xy(state.stacks()[0].hex);
    let Command::Move { to } = decide(&state, &r) else {
        panic!("no walk");
    };
    assert_eq!(xy(to), (5, start.1));
    play(&mut state, &r, Command::Move { to });
    let rounds = computer::tactics_rounds(&state);
    assert_eq!((rounds.completed, rounds.placed_last), (1, true));
    // Round 2: its walk ends where it stands: Defend, which the phase takes as Wait; below Hard
    // a walker that has waited gives Wait again, which ends its turn.
    assert_eq!(decide(&state, &r), Command::Wait);
    play(&mut state, &r, Command::Wait);
    assert!(computer::wait_phase(&state));
    assert_eq!(decide(&state, &r), Command::Wait);
    play(&mut state, &r, Command::Wait);
    let rounds = computer::tactics_rounds(&state);
    assert_eq!((rounds.completed, rounds.placed_last), (2, false));
    // The round placed nothing: the phase ends.
    assert_eq!(decide(&state, &r), Command::EndTactics);
    play(&mut state, &r, Command::EndTactics);
    assert!(state.tactics().is_none());
}

// covers: BAT-AI-023
#[test]
fn the_side_ends_its_phase_after_three_rounds() {
    let r = ruleset();
    let mut state = phase(2, &[(IMP, 3)], &[(GOG, 2)], 1);
    let start = state.stacks()[0].hex;
    let next = Hex(start.0 + 1);
    for (round, to) in [next, start, next].into_iter().enumerate() {
        assert_ne!(decide(&state, &r), Command::EndTactics, "round {round}");
        play(&mut state, &r, Command::Move { to });
    }
    assert_eq!(computer::tactics_rounds(&state).completed, 3);
    assert_eq!(decide(&state, &r), Command::EndTactics);
    // In the phase a choice the engine refuses falls back to Wait, which ends no turn.
    let side = state.side_to_act().unwrap();
    assert_eq!(checked(&state, &r, side, Command::Defend), Command::Wait);
}

// covers: BAT-AI-023
#[test]
fn a_shooter_takes_the_hex_of_the_zone_with_the_fewest_free_neighbours() {
    let r = ruleset();
    let mut state = phase(1, &[(ARCHER, 5), (ARCHER, 5), (IMP, 3)], &[(GOG, 2)], 1);
    let at: vec::Vec<Hex> = state.stacks().iter().map(|s| s.hex).collect();
    assert_eq!(at[..3], [Hex(1), Hex(52), Hex(120)]);
    // The imp acts first and walks.
    assert_eq!(state.active(), Some(StackId(2)));
    let walk = decide(&state, &r);
    assert!(matches!(walk, Command::Move { .. }), "{walk:?}");
    play(&mut state, &r, walk);
    // The first archer stands in the field's corner, four free neighbours (two lie off the
    // field): no hex of the zone has fewer, so it stays (Wait).
    assert_eq!(state.active(), Some(StackId(0)));
    assert_eq!(decide(&state, &r), Command::Wait);
    play(&mut state, &r, Command::Wait);
    // The second takes the first hex with four, on the top row: not hex 2, next to the first
    // archer (a shooter sets the score to 1000), but hex 3.
    assert_eq!(state.active(), Some(StackId(1)));
    assert_eq!(decide(&state, &r), Command::Move { to: Hex(3) });
    play(&mut state, &r, Command::Move { to: Hex(3) });
    // The first, among the waited stacks, still stays: its Wait ends its turn.
    assert_eq!(state.active(), Some(StackId(0)));
    assert_eq!(decide(&state, &r), Command::Wait);
}

/// Batch 4b H1 (E23 T7): a computer hero's Archers alone in the defender's zone of Expert Tactics
/// go to hex 15 (row 0, column 15), two free neighbours in the original's table, which has none in
/// the side column; with the field's own neighbours hex 15 had four and the port took hex 9.
// covers: BAT-AI-023
#[test]
fn a_shooter_counts_no_neighbour_in_a_side_column() {
    let r = ruleset();
    let mut setup = setup(&[(IMP, 20)], &[(ARCHER, 20)]);
    setup.heroes = [Some(hero(0)), Some(hero(3))];
    let state = BattleState::new(&r, &setup, 5).unwrap();
    assert!(state.tactics().is_some());
    assert_eq!(state.stacks()[1].hex, Hex(100));
    assert_eq!(state.active(), Some(StackId(1)));
    assert_eq!(decide(&state, &r), Command::Move { to: Hex(15) });
}

// covers: BAT-AI-023
#[test]
fn on_easy_a_computers_shooter_waits_in_the_phase() {
    let r = ruleset();
    let mut state = phase(1, &[(ARCHER, 5), (IMP, 3)], &[(GOG, 2)], 0);
    // The imp acts first and walks; the archer waits.
    assert_eq!(state.active(), Some(StackId(1)));
    let walk = decide(&state, &r);
    assert!(matches!(walk, Command::Move { .. }), "{walk:?}");
    play(&mut state, &r, walk);
    assert_eq!(state.active(), Some(StackId(0)));
    assert_eq!(decide(&state, &r), Command::Wait);
}

// covers: BAT-AI-023
#[test]
fn a_two_hex_walker_ends_with_both_hexes_in_the_zone() {
    let r = ruleset();
    let state = phase(1, &[(CAVALIER, 2)], &[(GOG, 2)], 1);
    let start = xy(state.stacks()[0].hex);
    // The zone's columns are 1 to 3: its rear at 3 would put its front at 4.
    assert_eq!(
        decide(&state, &r),
        Command::Move {
            to: Hex(u8::try_from(start.1 * 17 + 2).unwrap())
        }
    );
}

// covers: BAT-AI-009, BAT-AI-010, BAT-AI-023
#[test]
fn in_the_phase_the_records_count_turns_and_an_attack_is_a_walk() {
    let r = ruleset();
    let state = phase(1, &[(EFREET, 1), (IMP, 3)], &[(GOG, 2)], 2);
    let (efreet, imp, gog) = (StackId(0), StackId(1), StackId(2));
    assert_eq!(state.active(), Some(efreet));
    // The imp's record reaches the gogs over the whole field, more than a turn away: its nearest,
    // not among its enemies within a turn.
    let ctx = computer::context(&state, &r, BattleSide::Attacker);
    let recorded = records(&state, &r, &ctx, (efreet, BattleSide::Attacker))[imp.index()];
    assert_eq!(recorded.nearest, Some(gog));
    assert!(recorded.turns > 1, "{recorded:?}");
    assert_eq!(recorded.mask, 0);
    // The efreet reaches the gogs within its turn, but walks to the zone's edge.
    let Command::Move { to } = decide(&state, &r) else {
        panic!("no walk");
    };
    assert_eq!(xy(to).0, 3);
    // A guard hex is one turn away in the phase (0x420760).
    let w = Walker::new(&state, &r, (efreet, BattleSide::Attacker));
    assert_eq!(w.turns_to(Hex(150)), 1);
}

// covers: BAT-AI-022
#[test]
fn the_tent_heals_the_most_hurt_and_the_later_at_a_tie() {
    let r = ruleset();
    let mut setup = setup(&[(GOG, 10), (GOG, 10), (GOG, 10)], &[(RUNNER, 1)]);
    setup.heroes = [Some(hero(0)), Some(hero_with(&r, &["bolt"], 1, 99))];
    setup.war_machines = [vec![creature(TENT)], vec![]];
    let mut state = BattleState::new(&r, &setup, 3).unwrap();
    let tent = StackId(3);
    let lost = |state: &BattleState, id: usize| {
        let s = &state.stacks()[id];
        s.max_hp - s.top_hp
    };
    // Nobody hurt: Defend.
    assert_eq!(crate::tent(&state, tent), Command::Defend);
    // The same bolt on the first two stacks: the later.
    cast_at(&mut state, &r, BattleSide::Defender, "bolt", StackId(0));
    cast_at(&mut state, &r, BattleSide::Defender, "bolt", StackId(1));
    assert!(lost(&state, 0) > 0 && lost(&state, 0) == lost(&state, 1));
    assert_eq!(
        crate::tent(&state, tent),
        Command::Heal { target: StackId(1) }
    );
    // A third: the one that lost the most, the later at a tie.
    cast_at(&mut state, &r, BattleSide::Defender, "bolt", StackId(2));
    cast_at(&mut state, &r, BattleSide::Defender, "bolt", StackId(0));
    let most = (0..3).max_by_key(|&i| (lost(&state, i), i)).unwrap();
    assert_eq!(
        crate::tent(&state, tent),
        Command::Heal {
            target: StackId(u8::try_from(most).unwrap())
        }
    );
}
