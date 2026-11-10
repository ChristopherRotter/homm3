//! The revisit (E08 T105b2; ADV-TEL-008, decision 268, increment 11's design note §13): the
//! entered object under a hero visited again as a step's last step onto it visits it, with no
//! points spent or required, no step and no guard test (`revisits.rs`).

use super::teleporters::{
    BLUE, HERO, ONTO, RED, Rules, at, jump_state, jumps, losses, p, red, stack, teleporter_rules,
    walk, whirl, whirlpool_state,
};
use super::*;
use crate::testing::{PIKEMAN, hero_details, stream};

/// `player`'s hero `hero` revisits with `draws` as the player's stream: the outcome and the
/// events, or the error (a rejected revisit changes neither the state nor the stream and emits
/// nothing).
fn revisit(
    state: &mut AdventureState,
    rules: &Rules,
    (player, hero): (PlayerId, HeroId),
    draws: &mut Rng,
) -> Result<(AdventureOutcome, Vec<AdventureEvent>), AdventureCommandError> {
    let (before, stream) = (state.clone(), draws.clone());
    let store = [Arc::new(hero_details(&[]))];
    let mut events = Vec::new();
    let result = state.apply(
        &rules.0,
        &rules.1,
        &store,
        (player, draws),
        &AdventureCommand::Revisit { hero },
        &mut events,
    );
    if result.is_err() {
        assert_eq!((&*state, &*draws), (&before, &stream));
        assert!(events.is_empty());
    }
    result.map(|outcome| (outcome, events))
}

/// The heroes of `player` whose revisit is legal now.
fn revisitable(state: &AdventureState, rules: &Rules, player: PlayerId) -> Vec<HeroId> {
    state.revisitable(&rules.1, player).collect()
}

// covers: ADV-TEL-008, ADV-TEL-005
#[test]
fn a_revisit_jumps_through_a_gate_and_back_at_0_points_without_a_step() {
    let rules = teleporter_rules();
    let (gate, pair) = (p(2, 1), at(20, 1, 1));
    let gates = [("t:underground_gate", gate), ("t:underground_gate", pair)];
    let mut state = jump_state(&rules, (2, None), &[], &gates);
    walk(&mut state, &rules, &ONTO, &mut stream()).unwrap();
    assert_eq!(red(&state).0, pair);
    assert_eq!(revisitable(&state, &rules, RED), [HERO]);
    // At 0 points the hero jumps back to the gate, then again to the pair, and so on (the
    // original's space key, decision 268): each time the jump alone (no step, no draw), the
    // points and the day's maximum as they were.
    state.heroes[0].movement = 0;
    let max = red(&state).2;
    let mut draws = stream();
    for (from, to) in [(pair, gate), (gate, pair), (pair, gate)] {
        let (outcome, events) = revisit(&mut state, &rules, (RED, HERO), &mut draws).unwrap();
        assert_eq!(outcome, AdventureOutcome::default());
        assert_eq!(jumps(&events), [(Some(from), Some(to))]);
        assert_eq!(events.len(), 1);
        assert_eq!(red(&state), (to, 0, max));
        assert_eq!(revisitable(&state, &rules, RED), [HERO]);
    }
    assert_eq!(draws, stream());
}

// covers: ADV-TEL-008, ADV-TEL-004, ADV-TEL-003
#[test]
fn a_revisit_of_a_two_way_monolith_draws_its_exit_again_from_the_players_stream() {
    let rules = teleporter_rules();
    let members = [p(2, 1), p(12, 1), p(22, 1)];
    let objects = members.map(|m| ("t:monolith", m));
    let mut seen = [false; 2];
    for seed in 0..16 {
        let mut state = jump_state(&rules, (1, None), &[], &objects);
        let mut draws = Rng::for_stream(seed, StreamId::player(0));
        walk(&mut state, &rules, &ONTO, &mut draws).unwrap();
        let (on, points, max) = red(&state);
        // The channel's other members in the walk's order: one drawn from the stream.
        let others: Vec<Pos> = members.into_iter().filter(|&m| m != on).collect();
        let mut expected = draws.clone();
        let place = usize::try_from(expected.below(2)).unwrap();
        seen[place] = true;
        let (_, events) = revisit(&mut state, &rules, (RED, HERO), &mut draws).unwrap();
        assert_eq!(draws, expected, "seed {seed}");
        assert_eq!(red(&state), (others[place], points, max));
        assert_eq!(jumps(&events), [(Some(on), Some(others[place]))]);
    }
    assert_eq!(seen, [true; 2]);
}

// covers: ADV-TEL-008, ADV-TEL-007, ADV-MON-006
#[test]
fn a_revisit_tests_no_guard() {
    let rules = teleporter_rules();
    // The one-way exit (20, 1) beside an aggressive monster, which guards it: the arrival meets no
    // guard (ADV-TEL-007), nor does the revisit, which leads nowhere (the original tests a guard
    // only at a step's end, 0x4800A5).
    let exit = p(20, 1);
    let objects = [("t:entrance", p(2, 1)), ("t:exit", exit)];
    let mut state = jump_state(&rules, (1, Some(p(21, 1))), &[], &objects);
    walk(&mut state, &rules, &ONTO, &mut stream()).unwrap();
    assert_eq!(red(&state).0, exit);
    assert!(state.guard_of(&rules.1, exit).is_some());
    let (outcome, events) = revisit(&mut state, &rules, (RED, HERO), &mut stream()).unwrap();
    assert_eq!(outcome, AdventureOutcome::default());
    let nowhere = AdventureEvent {
        kind: AdventureEventKind::LeadsNowhere {
            hero: HERO,
            object_type: rules.1.object_type_by_key("t:exit").unwrap(),
            anchor: exit,
        },
        seen_by: Viewers::NONE.with(RED),
    };
    assert_eq!(events, [nowhere]);
    assert_eq!(red(&state).0, exit);
}

// covers: ADV-TEL-008, ADV-TEL-006
#[test]
fn a_revisit_in_a_whirlpool_takes_its_loss_and_draws_its_exit_again() {
    let rules = teleporter_rules();
    // W1 on (2, 0) to (3, 1), W2's one tile (27, 1): the step into W1 jumps to W2 without a draw
    // (one candidate), losing half the Pikemen.
    let whirlpools = [
        ("t:whirlpool", p(3, 1), 0x0303),
        ("t:whirlpool", p(27, 1), 1),
    ];
    let store = [Arc::new(hero_details(&[]))];
    let mut state = whirlpool_state(&rules, vec![stack(0, PIKEMAN, 8)], &whirlpools);
    let mut draws = stream();
    let (_, events) = whirl(&mut state, &rules, &store, &mut draws);
    assert_eq!(losses(&events), [(HERO, 0, PIKEMAN, 4)]);
    assert_eq!(draws, stream());
    let (on, points, max) = red(&state);
    assert_eq!(on, p(27, 1));
    // In its boat on W2 the hero revisits it: the loss again, then one of W1's four tiles drawn,
    // in the walk's order, the hero in its boat with the points it had.
    let mut expected = draws.clone();
    let place = usize::try_from(expected.below(4)).unwrap();
    let (outcome, events) = revisit(&mut state, &rules, (RED, HERO), &mut draws).unwrap();
    assert_eq!(outcome, AdventureOutcome::default());
    assert_eq!(draws, expected);
    assert_eq!(losses(&events), [(HERO, 0, PIKEMAN, 2)]);
    let w1 = [p(2, 0), p(3, 0), p(2, 1), p(3, 1)];
    assert_eq!(red(&state), (w1[place], points, max));
    assert_eq!(jumps(&events), [(Some(on), Some(w1[place]))]);
    assert_eq!(state.hero(HERO).unwrap().boat, Some(0));
    assert!(state.check_boats(&rules.1));
}

// covers: ADV-TEL-008, ADV-VIS-002, ADV-VIS-003, ADV-VIS-005
#[test]
fn nothing_to_revisit_but_an_entered_object_whose_visit_is_built() {
    let rules = teleporter_rules();
    // Red's hero on a free tile, Blue's hero 2 on another.
    let objects = [
        ("t:tower", p(2, 1)),
        ("t:tent", p(6, 1)),
        ("t:gate", p(8, 1)),
    ];
    let mut state = jump_state(&rules, (1, None), &[(2, 1, p(12, 1))], &objects);
    assert!(revisitable(&state, &rules, RED).is_empty());
    assert!(revisitable(&state, &rules, BLUE).is_empty());
    let mut draws = stream();
    for (player, hero, error) in [
        (RED, HERO, AdventureCommandError::NothingToRevisit),
        (
            RED,
            HeroId::new(2),
            AdventureCommandError::NotYourHero(HeroId::new(2)),
        ),
        (
            RED,
            HeroId::new(9),
            AdventureCommandError::NotYourHero(HeroId::new(9)),
        ),
        (BLUE, HERO, AdventureCommandError::NotYourHero(HERO)),
    ] {
        assert_eq!(
            revisit(&mut state, &rules, (player, hero), &mut draws).err(),
            Some(error)
        );
    }
    // On the observatory: its reveal again, told with no tile new.
    walk(&mut state, &rules, &ONTO, &mut draws).unwrap();
    assert_eq!(revisitable(&state, &rules, RED), [HERO]);
    let tower = rules.1.object_type_by_key("t:tower").unwrap();
    let (outcome, events) = revisit(&mut state, &rules, (RED, HERO), &mut draws).unwrap();
    assert_eq!(outcome, AdventureOutcome::default());
    let revealed = AdventureEventKind::TilesRevealed {
        object_type: tower,
        count: 0,
    };
    assert_eq!(
        events.iter().map(|e| e.kind).collect::<Vec<_>>(),
        [revealed]
    );
    // On the Keymaster's tent: the key, then nothing at its revisit (the player holds it).
    let path = [p(3, 1), p(4, 1), p(5, 1), p(6, 1)];
    walk(&mut state, &rules, &path, &mut draws).unwrap();
    assert!(state.has_key(RED, 0));
    let (outcome, events) = revisit(&mut state, &rules, (RED, HERO), &mut draws).unwrap();
    assert_eq!((outcome, events), (AdventureOutcome::default(), Vec::new()));
    // On the border gate, crossed with the key (ADV-VIS-005): nothing to revisit.
    walk(&mut state, &rules, &[p(7, 1), p(8, 1)], &mut draws).unwrap();
    assert_eq!(red(&state).0, p(8, 1));
    assert!(revisitable(&state, &rules, RED).is_empty());
    assert_eq!(
        revisit(&mut state, &rules, (RED, HERO), &mut draws).err(),
        Some(AdventureCommandError::NothingToRevisit)
    );
    assert_eq!(draws, stream());
}
