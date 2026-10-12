//! Tests of the game state (E08 T6b): the starting armies' draw, the players' turns in sequential
//! play and in a larger turn group, the calendar, `EndTurn` and its rejections, and the hash.

use h3_adventure::{AdventureState, MAX_GAIN, ObjectSetup, TileMask};
use h3_content::ResourceId;
use h3_kernel::hash::hash_of;
use h3_kernel::{Rng, StreamId};
use proptest::prelude::{prop_assert, prop_assert_eq, proptest};

use super::*;

const GOG: CreatureTypeId = CreatureTypeId::from_raw(1);
const CROWD: HeroTypeId = HeroTypeId::from_raw(2);
const GUNNER: HeroTypeId = HeroTypeId::from_raw(3);
const RED: PlayerId = PlayerId::new(0);
const BLUE: PlayerId = PlayerId::new(1);

fn stack(slot: u8, creature: CreatureTypeId, count: u32) -> ArmyStack {
    ArmyStack {
        slot,
        creature,
        count,
    }
}

fn build(rules: &GameRuleset, setup: &GameSetup, seed: u64) -> GameState {
    GameState::new(rules, setup, seed).unwrap()
}

/// Blue's hero's drawn army when it is of `hero_type`.
fn drawn(rules: &GameRuleset, hero_type: HeroTypeId, seed: u64) -> Vec<ArmyStack> {
    let mut setup = setup();
    setup.heroes[1].hero_type = hero_type;
    let state = build(rules, &setup, seed);
    state.adventure().hero(HeroId::new(1)).unwrap().army.clone()
}

fn end_turn(
    state: &mut GameState,
    rules: &GameRuleset,
    player: PlayerId,
) -> Result<Vec<GameEvent>, GameCommandError> {
    let mut events = Vec::new();
    apply(
        state,
        rules,
        Actor::Player(player),
        GameCommand::EndTurn,
        &mut events,
    )
    .map(|()| events)
}

// covers: GAM-SET-001
#[test]
fn building_draws_the_starting_armies_and_keeps_custom_ones() {
    let rules = rules();
    let state = build(&rules, &setup(), 7);
    let heroes = state.adventure().heroes();
    assert_eq!(heroes[0].army, [stack(1, IMP, 9)]);
    // Orrin's first stack always comes, 2 to 5 imps; gogs, if any, follow in slot 1.
    let army = &heroes[1].army;
    assert_eq!((army[0].slot, army[0].creature), (0, IMP));
    assert!((2..=5).contains(&army[0].count));
    assert!(
        army[1..]
            .iter()
            .all(|s| s.slot == 1 && s.creature == GOG && (1..=3).contains(&s.count))
    );
    // The same seed draws the same armies; the draw is the setup stream's alone.
    assert_eq!(build(&rules, &setup(), 7), state);
    let armies: Vec<Vec<ArmyStack>> = (0..64).map(|seed| drawn(&rules, ORRIN, seed)).collect();
    assert!(armies.iter().any(|a| a.len() == 1) && armies.iter().any(|a| a.len() == 2));
    assert!(armies.iter().any(|a| a[0].count == 2) && armies.iter().any(|a| a[0].count == 5));
}

// covers: GAM-SET-001, ADV-PIK-001
#[test]
fn the_setup_stream_draws_the_objects_contents_after_the_armies() {
    let rules = rules();
    let mut setup = setup();
    setup.map.objects.push(ObjectSetup {
        anchor: Pos::new(2, 2, 0),
        blocked: TileMask::default(),
        visitable: TileMask::from_bits(1),
        object_type: rules.adventure().object_type_by_key("t:pile"),
        amount: None,
    });
    let mut amounts = Vec::new();
    for seed in 0..40 {
        let state = build(&rules, &setup, seed);
        let mut draws = Rng::for_stream(seed, StreamId::SETUP);
        let part = setup.adventure_setup(&rules, &mut draws).unwrap();
        let alone =
            AdventureState::new(rules.core(), rules.adventure(), &part, &mut draws).unwrap();
        assert_eq!(state.adventure(), &alone);
        let [(_, gold)] = state.adventure().map().objects()[0].contents[..] else {
            panic!("one resource");
        };
        amounts.push(gold);
    }
    assert!(
        amounts
            .iter()
            .all(|a| (500..=1000).contains(a) && a % 100 == 0)
    );
    assert!(amounts.contains(&500) && amounts.contains(&1000));
}

// covers: GAM-SET-001
#[test]
fn each_later_slot_comes_with_its_chance() {
    // The gunner's slots: imp 1-1, the cart (a war machine, no stack), gog 2-2.
    let gunner = |chances: Vec<u8>| drawn(&rules_with(chances, 2), GUNNER, 3);
    assert_eq!(gunner(vec![100, 100]), [stack(0, IMP, 1), stack(1, GOG, 2)]);
    assert_eq!(gunner(vec![0, 100]), [stack(0, IMP, 1), stack(1, GOG, 2)]);
    assert_eq!(gunner(vec![100, 0]), [stack(0, IMP, 1)]);
    // Slots past the chances never come.
    assert_eq!(gunner(vec![100]), [stack(0, IMP, 1)]);
    assert_eq!(gunner(vec![]), [stack(0, IMP, 1)]);
    let orrin = |chances: Vec<u8>| drawn(&rules_with(chances, 2), ORRIN, 3).len();
    assert_eq!((orrin(vec![100]), orrin(vec![0, 100])), (2, 1));
}

// covers: GAM-SET-001
#[test]
fn the_second_and_third_slots_are_rolled_apart() {
    // The crowd (imp, imp, gog) in three slots with the original's chances, over 4000 seeds:
    // the second stack comes about 88 % of the time, the third about 25 %, and the third
    // without the second about 3 % (one roll for both, as VCMI has it, would never give it).
    let rules = rules_with(vec![88, 25], 3);
    let (mut second, mut third, mut alone) = (0, 0, 0);
    for seed in 0..4000 {
        let army = drawn(&rules, CROWD, seed);
        let has_second = army.get(1).is_some_and(|s| s.creature == IMP);
        let has_third = army.last().is_some_and(|s| s.creature == GOG);
        second += u32::from(has_second);
        third += u32::from(has_third);
        alone += u32::from(has_third && !has_second);
    }
    assert!((3400..=3640).contains(&second), "{second}");
    assert!((880..=1120).contains(&third), "{third}");
    assert!((80..=170).contains(&alone), "{alone}");
}

// covers: GAM-TRN-001, GAM-TRN-002, GAM-CAL-001, ADV-MOV-002
#[test]
fn players_take_their_turns_in_order_and_the_day_advances() {
    let rules = rules();
    let mut state = build(&rules, &setup(), 7);
    assert_eq!((state.day(), state.players()), (1, 2));
    assert_eq!(state.turn_group(), [RED]);
    assert!(state.is_acting(RED) && !state.is_acting(BLUE));
    assert_eq!(
        (state.colour(BLUE), state.colour(PlayerId::new(2))),
        (Some(1), None)
    );
    assert_eq!(
        end_turn(&mut state, &rules, RED),
        Ok(vec![
            GameEvent::TurnEnded { player: RED },
            GameEvent::TurnStarted { player: BLUE },
        ])
    );
    assert_eq!((state.day(), state.turn_group()), (1, &[BLUE][..]));
    assert!(!state.is_acting(RED) && state.is_acting(BLUE));
    assert_eq!(
        end_turn(&mut state, &rules, BLUE),
        Ok(vec![
            GameEvent::TurnEnded { player: BLUE },
            GameEvent::DayStarted { day: 2 },
            GameEvent::TurnStarted { player: RED },
        ])
    );
    assert_eq!((state.day(), state.turn_group()), (2, &[RED][..]));
    // Each hero has the day's points for its army: imps alone 1500, gogs slow it to 1430.
    for hero in state.adventure().heroes() {
        let gogs = hero.army.iter().any(|s| s.creature == GOG);
        assert_eq!(hero.movement, if gogs { 1430 } else { 1500 });
    }
}

#[test]
fn a_rejected_end_turn_changes_nothing() {
    let rules = rules();
    let mut state = build(&rules, &setup(), 7);
    let before = state.clone();
    assert_eq!(
        end_turn(&mut state, &rules, BLUE),
        Err(GameCommandError::NotYourTurn(BLUE))
    );
    let stranger = PlayerId::new(2);
    assert_eq!(
        end_turn(&mut state, &rules, stranger),
        Err(GameCommandError::UnknownPlayer(stranger))
    );
    assert_eq!(state, before);
    assert_eq!(state.state_hash(), before.state_hash());
}

// covers: GAM-TRN-002
#[test]
fn a_turn_group_ends_when_each_of_its_players_has_ended_its_turn() {
    let rules = rules();
    let mut state = build(&rules, &setup(), 7);
    state.set_schedule(&rules, vec![vec![RED, BLUE]]);
    assert!(state.is_acting(RED) && state.is_acting(BLUE));
    assert_eq!(
        end_turn(&mut state, &rules, BLUE),
        Ok(vec![GameEvent::TurnEnded { player: BLUE }])
    );
    assert_eq!(
        end_turn(&mut state, &rules, BLUE),
        Err(GameCommandError::NotYourTurn(BLUE))
    );
    assert_eq!(
        end_turn(&mut state, &rules, RED),
        Ok(vec![
            GameEvent::TurnEnded { player: RED },
            GameEvent::DayStarted { day: 2 },
            GameEvent::TurnStarted { player: RED },
            GameEvent::TurnStarted { player: BLUE },
        ])
    );
}

// covers: GAM-RES-001, GAM-RES-002
#[test]
fn every_player_starts_with_the_human_row_of_the_difficulty() {
    let rules = rules();
    let gold = |state: &GameState, player| state.treasury(player).unwrap().amounts().to_vec();
    let normal = build(&rules, &setup(), 7);
    assert_eq!(setup().difficulty, DEFAULT_DIFFICULTY);
    assert_eq!(
        (gold(&normal, RED), gold(&normal, BLUE)),
        (vec![2000], vec![2000])
    );
    let mut easy = setup();
    easy.difficulty = 0;
    let easy = build(&rules, &easy, 7);
    assert_eq!(
        (gold(&easy, RED), gold(&easy, BLUE)),
        (vec![3000], vec![3000])
    );
    assert!(normal.treasury(PlayerId::new(2)).is_none());
    let treasury = normal.treasury(RED).unwrap();
    assert_eq!(treasury.amount(ResourceId::from_raw(0)), 2000);
    assert_eq!(treasury.amount(ResourceId::from_raw(1)), 0);
    // A row's amount is capped at 2^31 - 1, as a gain is (the compiler already checks the rows).
    assert_eq!(Treasury::new(&[u32::MAX, 5]).amounts(), [MAX_GAIN, 5]);
}

#[test]
fn the_hash_covers_the_whole_game() {
    let rules = rules();
    let state = build(&rules, &setup(), 7);
    assert_eq!(state.seed(), 7);
    assert_eq!(state.state_hash(), hash_of(&state));
    assert_eq!(state.clone().state_hash(), state.state_hash());
    // Another seed: other streams (and maybe other armies).
    assert_ne!(build(&rules, &setup(), 8).state_hash(), state.state_hash());
    let mut ended = state.clone();
    end_turn(&mut ended, &rules, RED).unwrap();
    assert_ne!(ended.state_hash(), state.state_hash());
    let mut grouped = state.clone();
    grouped.set_schedule(&rules, vec![vec![RED, BLUE]]);
    assert_ne!(grouped.state_hash(), state.state_hash());
    let mut colours = setup();
    colours.players[1].colour = 2;
    assert_ne!(build(&rules, &colours, 7).state_hash(), state.state_hash());
    let mut easy = setup();
    easy.difficulty = 0;
    assert_ne!(build(&rules, &easy, 7).state_hash(), state.state_hash());
}

#[test]
fn the_command_errors_read_well() {
    assert_eq!(
        GameCommandError::UnknownPlayer(PlayerId::new(4)).to_string(),
        "player 4 is not in the game"
    );
    assert_eq!(
        GameCommandError::NotYourTurn(BLUE).to_string(),
        "it is not player 1's turn"
    );
}

proptest! {
    /// Any sequence of `EndTurn`s by any actors: a rejected one changes nothing, an accepted one
    /// passes the turn on, and two runs of the same sequence end in the same state.
    #[test]
    fn end_turns_are_atomic_and_deterministic(seed in 0u64..1000, actors in proptest::collection::vec(0u8..4, 0..40)) {
        let rules = rules();
        let mut state = build(&rules, &setup(), seed);
        let mut accepted = 0u32;
        for &actor in &actors {
            let before = state.clone();
            match end_turn(&mut state, &rules, PlayerId::new(actor)) {
                Ok(events) => {
                    accepted += 1;
                    prop_assert!(!events.is_empty());
                    prop_assert!(state.is_acting(PlayerId::new(actor ^ 1)));
                }
                Err(_) => {
                    prop_assert_eq!(&state, &before);
                    prop_assert_eq!(state.state_hash(), before.state_hash());
                }
            }
        }
        prop_assert_eq!(state.day(), 1 + accepted / 2);
        let mut again = build(&rules, &setup(), seed);
        for &actor in &actors {
            let _ = end_turn(&mut again, &rules, PlayerId::new(actor));
        }
        prop_assert_eq!(again.state_hash(), state.state_hash());
    }
}
