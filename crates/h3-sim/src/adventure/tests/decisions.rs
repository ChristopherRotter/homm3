//! The env with open decisions (E08 T14b, action encoding v2): a Cartographer's offer opens as
//! its visitor's owner's decision, the entry mask then allows its two answers alone, each with
//! tile 0 alone, the answers step through the env, a fork carries the open decision, the
//! random agent answers, and a replay records the answers and the game's difficulty.

use h3_game::{Answer, GameEvent, GameSetupError};

use super::*;

const RED: PlayerId = PlayerId::new(0);
const BLUE: PlayerId = PlayerId::new(1);

/// The duel with [`chart_setup`]'s Cartographers at `difficulty`, seed 1, and the tile of the
/// Cartographer beside red's hero in slot 0.
fn chart_env(difficulty: u8) -> (AdventureEnv, u32) {
    let setup = chart_setup(&FAITHFUL, difficulty);
    let chart = charts(&setup)[0];
    let env = AdventureEnv::new(FAITHFUL.clone(), setup, 1, 0).unwrap();
    let tile = tile_index(env.state(), chart).unwrap();
    (env, tile)
}

/// [`chart_env`] after red's hero in slot 0 entered its Cartographer: the step's events.
fn entered(env: &mut AdventureEnv, tile: u32) -> Vec<GameEvent> {
    let mut events = Vec::new();
    env.step_with(RED, Action::move_to(0, tile), &mut events)
        .unwrap();
    events
}

fn legal_entries(env: &AdventureEnv, player: PlayerId) -> Vec<usize> {
    let mut mask = [false; ENTRY_COUNT];
    env.legal_mask(player, &mut mask);
    (0..ENTRY_COUNT).filter(|&e| mask[e]).collect()
}

/// The tiles of each entry's tile mask (one past the last entry too).
fn legal_tiles(env: &AdventureEnv, player: PlayerId) -> Vec<Vec<u32>> {
    let mut tiles = Vec::new();
    (0..=ENTRY_COUNT)
        .map(|entry| {
            env.tile_mask(player, entry, &mut tiles);
            (0u32..)
                .zip(&tiles)
                .filter(|&(_, &t)| t)
                .map(|(i, _)| i)
                .collect()
        })
        .collect()
}

/// Decoding and encoding agree: `EndTurn` and the answers with tile 0 alone, each
/// legal move by its hero's slot and destination along the query's path; a move of another
/// player's hero, without a step or off the map names no action of the player.
#[test]
fn actions_and_commands_round_trip() {
    let e = env(1, 0);
    let (state, rules) = (e.state(), e.rules());
    for (action, command) in [
        (Action::end_turn(), GameCommand::EndTurn),
        (Action::accept(), GameCommand::Answer(Answer::Accept)),
        (Action::decline(), GameCommand::Answer(Answer::Decline)),
    ] {
        assert_eq!(
            action::decode(state, rules, RED, action),
            Some(command.clone())
        );
        let Action::Adventure(map) = action else {
            panic!("{action:?} is no map action")
        };
        let off = Action::map(map.entry, 77);
        assert_eq!(action::decode(state, rules, RED, off), None);
        assert_eq!(action::encode(state, rules, RED, &command), Some(action));
    }
    let past = Action::map(ENTRY_COUNT, 0);
    assert_eq!(action::decode(state, rules, RED, past), None);
    let mut tiles = Vec::new();
    let mut moves = 0;
    for slot in 0..HERO_SLOTS {
        e.tile_mask(RED, MOVE_BASE + slot, &mut tiles);
        for tile in (0u32..).zip(&tiles).filter(|&(_, &t)| t).map(|(i, _)| i) {
            let action = Action::move_to(slot, tile);
            let command = action::decode(state, rules, RED, action).unwrap();
            assert_eq!(action::encode(state, rules, RED, &command), Some(action));
            moves += 1;
        }
    }
    assert!(moves > 2, "{moves}");
    let blue = hero_in_slot(state, BLUE, 0).unwrap();
    let red = hero_in_slot(state, RED, 1).unwrap();
    let at = state.adventure().hero(red).unwrap().pos;
    let off = h3_adventure::Pos::new(36, at.y, at.level);
    for (hero, path) in [(blue, vec![at]), (red, Vec::new()), (red, vec![off])] {
        let command = GameCommand::Adventure(AdventureCommand::MoveHero { hero, path });
        assert_eq!(action::encode(state, rules, RED, &command), None);
    }
}

// covers: GAM-DEC-001
/// While red's offer is open its answers are its only legal entries, each with tile 0 alone;
/// `EndTurn` and moves are refused, blue has nothing; a fork carries the decision, and each
/// answer steps through the env and gives the masks back.
#[test]
fn an_open_offer_is_answered_through_the_env() {
    let (mut env, tile) = chart_env(h3_game::DEFAULT_DIFFICULTY);
    let hero = hero_in_slot(env.state(), RED, 0).unwrap();
    let home = env.state().adventure().hero(hero).unwrap().pos;
    let mut tiles = Vec::new();
    env.tile_mask(RED, MOVE_BASE, &mut tiles);
    assert!(tiles[tile as usize]);
    let events = entered(&mut env, tile);
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::OfferOpened { player: RED, .. })),
        "{events:?}"
    );
    let offer = env.observation(RED).decision.unwrap();
    assert_eq!(offer.price().len(), 1);
    assert_eq!(env.observation(BLUE).decision, None);
    assert_eq!(legal_entries(&env, RED), [ACCEPT, DECLINE]);
    for (entry, tiles) in legal_tiles(&env, RED).into_iter().enumerate() {
        let answer = entry == ACCEPT || entry == DECLINE;
        assert_eq!(tiles, if answer { vec![0] } else { Vec::new() }, "{entry}");
    }
    assert_eq!(legal_entries(&env, BLUE), []);
    assert!(legal_tiles(&env, BLUE).iter().all(Vec::is_empty));
    assert_eq!(env.to_move().collect::<Vec<_>>(), [RED]);
    let (hash, steps) = (env.state_hash(), env.steps());
    let open = Err(StepError::Rejected(GameCommandError::DecisionOpen));
    assert_eq!(env.step(RED, Action::end_turn()), open);
    let back = GameCommand::Adventure(AdventureCommand::MoveHero {
        hero,
        path: vec![home],
    });
    assert_eq!(env.step_command(RED, back, &mut h3_game::NoopSink), open);
    let home = tile_index(env.state(), home).unwrap();
    assert!(env.step(RED, Action::move_to(0, home)).is_err());
    assert_eq!(
        env.step(BLUE, Action::accept()),
        Err(StepError::Rejected(GameCommandError::NoDecision))
    );
    assert_eq!((env.state_hash(), env.steps()), (hash, steps));
    // A fork during the decision: it declines, its parent is still asked, then accepts.
    let mut fork = env.fork();
    assert_eq!(fork.observation(RED).decision, Some(offer.clone()));
    let mut declined = Vec::new();
    fork.step_with(RED, Action::decline(), &mut declined)
        .unwrap();
    assert!(declined.iter().any(|e| matches!(
        e,
        GameEvent::OfferAnswered {
            accepted: false,
            ..
        }
    )));
    assert_eq!(env.state_hash(), hash);
    assert_eq!(env.observation(RED).decision, Some(offer.clone()));
    let gold = offer.price()[0].0.index();
    let before = env.observation(RED).treasury[gold];
    let mut accepted = Vec::new();
    let off = Action::map(ACCEPT, 5);
    assert_eq!(env.step(RED, off), Err(StepError::InvalidAction));
    env.step_with(RED, Action::accept(), &mut accepted).unwrap();
    assert!(
        accepted
            .iter()
            .any(|e| matches!(e, GameEvent::ResourcePaid { amount: 1000, .. }))
    );
    assert_eq!(env.observation(RED).treasury[gold], before - 1000);
    assert_eq!(fork.observation(RED).treasury[gold], before);
    for e in [&mut env, &mut fork] {
        assert_eq!(e.observation(RED).decision, None);
        assert_eq!(legal_entries(e, RED)[0], END_TURN);
        assert!(!legal_tiles(e, RED)[MOVE_BASE].is_empty());
        assert_eq!(
            e.step(RED, Action::decline()).unwrap_err(),
            StepError::Rejected(GameCommandError::NoDecision)
        );
    }
    assert_ne!(env.state_hash(), fork.state_hash());
    assert_eq!(env.observation(RED).adventure.bought, [offer.object_type]);
    assert_eq!(fork.observation(RED).adventure.bought, []);
}

// covers: ADV-VIS-001
/// At impossible difficulty red has no gold: the offer does not open and the masks stay.
#[test]
fn an_offer_its_treasury_cannot_pay_opens_no_decision() {
    let (mut env, tile) = chart_env(4);
    let events = entered(&mut env, tile);
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::OfferRefused { player: RED, .. }))
    );
    assert_eq!(env.observation(RED).decision, None);
    assert_eq!(legal_entries(&env, RED)[0], END_TURN);
    assert!(!legal_entries(&env, RED).contains(&ACCEPT));
}

/// The random agent answers an open decision, either way, from its own stream; a rollout plays
/// on through it.
#[test]
fn the_random_agent_answers_an_open_decision() {
    let (mut env, tile) = chart_env(h3_game::DEFAULT_DIFFICULTY);
    entered(&mut env, tile);
    let answers: Vec<Action> = (0..16)
        .map(|seed| choose(&env, RED, &mut Rng::for_stream(seed, StreamId::AGENT)))
        .collect();
    for answer in [Action::accept(), Action::decline()] {
        assert!(answers.contains(&answer), "{answers:?}");
    }
    assert!(
        answers
            .iter()
            .all(|&a| a == Action::accept() || a == Action::decline())
    );
    let played = random_rollout(&mut env, 3, 20);
    assert_eq!(played.steps, 20);
    assert_eq!(env.observation(RED).decision, None);
}

/// The difficulty is chosen at a reset and kept by later ones; one the rules do not have is the
/// setup's error and leaves the env as it was.
#[test]
fn a_reset_chooses_the_difficulty() {
    let mut e = env(1, 0);
    assert_eq!(e.difficulty(), h3_game::DEFAULT_DIFFICULTY);
    let normal = e.observation(RED).treasury;
    e.reset_with_difficulty(2, 0).unwrap();
    let easy = GameSetup {
        difficulty: 0,
        ..GameSetup::clone(&DUEL_SETUP)
    };
    assert_eq!(e.state(), &GameState::new(&FAITHFUL, &easy, 2).unwrap());
    assert_eq!(e.difficulty(), 0);
    let total = |treasury: Vec<u32>| treasury.iter().sum::<u32>();
    assert!(total(e.observation(RED).treasury) > total(normal.clone()));
    e.step(RED, Action::end_turn()).unwrap();
    e.reset(1).unwrap();
    assert_eq!((e.difficulty(), e.steps()), (0, 0));
    assert_eq!(e.state(), &GameState::new(&FAITHFUL, &easy, 1).unwrap());
    let hash = e.state_hash();
    assert_eq!(
        e.reset_with_difficulty(3, 5),
        Err(GameSetupError::Difficulty(5))
    );
    assert_eq!((e.state_hash(), e.difficulty()), (hash, 0));
    e.reset_with_difficulty(1, 0).unwrap();
    assert_eq!(e.state_hash(), hash);
    e.reset_with_difficulty(1, h3_game::DEFAULT_DIFFICULTY)
        .unwrap();
    assert_eq!(e.observation(RED).treasury, normal);
}

/// A game with answers at easy difficulty, recorded command by command, verifies at the
/// replay's difficulty against the map's import (normal); another difficulty is another game,
/// one the rules do not have the setup's error. The random agent's record keeps its difficulty.
#[test]
fn a_replay_records_the_answers_and_the_difficulty() {
    let setup = chart_setup(&FAITHFUL, 0);
    let charts = charts(&setup);
    let mut env = AdventureEnv::new(FAITHFUL.clone(), Arc::clone(&setup), 1, 0).unwrap();
    let mut commands = Vec::new();
    let mut play = |env: &mut AdventureEnv, player, action| {
        let (_, command) = env
            .step_decoded(player, action, &mut h3_game::NoopSink)
            .unwrap();
        commands.push((Actor::Player(player), command));
    };
    let red_chart = tile_index(env.state(), charts[0]).unwrap();
    play(&mut env, RED, Action::move_to(0, red_chart));
    play(&mut env, RED, Action::accept());
    play(&mut env, RED, Action::end_turn());
    let blue_chart = tile_index(env.state(), charts[1]).unwrap();
    play(&mut env, BLUE, Action::move_to(0, blue_chart));
    play(&mut env, BLUE, Action::decline());
    let answers = [Answer::Accept, Answer::Decline].map(GameCommand::Answer);
    assert_eq!(
        commands.iter().filter(|(_, c)| answers.contains(c)).count(),
        2
    );
    let mut replay = replay::Replay {
        header: replay::ReplayHeader::current(&FAITHFUL),
        map: duel_map(),
        seed: 1,
        difficulty: 0,
        commands,
        checkpoint_every: 2,
        checkpoints: Vec::new(),
        final_day: env.day(),
        final_hash: env.state_hash(),
    };
    let imported = GameSetup {
        difficulty: h3_game::DEFAULT_DIFFICULTY,
        ..GameSetup::clone(&setup)
    };
    let mut state = GameState::new(&FAITHFUL, &setup, 1).unwrap();
    for (i, (actor, command)) in replay.commands.iter().enumerate() {
        apply(
            &mut state,
            &FAITHFUL,
            *actor,
            command.clone(),
            &mut h3_game::NoopSink,
        )
        .unwrap();
        if (i + 1) % 2 == 0 {
            replay.checkpoints.push(state.state_hash());
        }
    }
    assert_eq!(replay.verify(&FAITHFUL, &duel_map(), &imported), Ok(()));
    let mut normal = replay.clone();
    normal.difficulty = h3_game::DEFAULT_DIFFICULTY;
    assert!(matches!(
        normal.verify(&FAITHFUL, &duel_map(), &imported),
        Err(replay::ReplayError::HashMismatch { step: 2, .. })
    ));
    normal.difficulty = 9;
    assert_eq!(
        normal.verify(&FAITHFUL, &duel_map(), &imported),
        Err(replay::ReplayError::Setup(GameSetupError::Difficulty(9)))
    );
    let options = replay::RecordOptions {
        seed: 2,
        agent_seed: 3,
        max_days: 0,
        max_steps: 12,
        checkpoint_every: 4,
    };
    let random = replay::Replay::record_random(&FAITHFUL, setup, duel_map(), options).unwrap();
    assert_eq!(random.difficulty, 0);
    assert_eq!(random.verify(&FAITHFUL, &duel_map(), &imported), Ok(()));
}
