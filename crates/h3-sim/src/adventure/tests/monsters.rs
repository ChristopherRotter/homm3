//! The env with a wandering monster's battle (E08 T20b3a, ADR-0017 point 8): `to_move` yields the
//! neutral seat while the monster's side is to act, the seat plays it through the battle's
//! encoding as the system actor (its map masks empty, its observation the monster's side's), the
//! players' seats cannot, the random agent plays both sides to the battle's end, and a replay
//! records and verifies the system actor's commands.

use h3_adventure::{MonsterSetup, Pos};
use h3_content::CreatureTypeId;
use h3_contracts::BattleSide;
use h3_game::GameEvent;

use super::*;

const RED: PlayerId = PlayerId::new(0);

/// The duel at `difficulty` with 30 savage pikemen that never flee (so they fight: survey
/// `E08-battles.md` §3.2) on the first free tile beside red's hero in slot 0 (one step on its
/// day-1 query, visited from no object), and that tile.
pub(super) fn monster_setup(rules: &GameRuleset, difficulty: u8) -> (Arc<GameSetup>, Pos) {
    let mut setup = GameSetup::clone(&setup(rules));
    setup.difficulty = difficulty;
    let state = GameState::new(rules, &setup, 1).unwrap();
    let hero = hero_in_slot(&state, RED, 0).unwrap();
    let reach = legal_moves(&state, rules, Actor::Player(RED), hero).unwrap();
    let map = state.adventure().map();
    let (near, _) = reach
        .tiles()
        .find(|&(pos, _)| reach.path(pos).unwrap().len() == 1 && map.visited_from(pos).is_empty())
        .unwrap();
    let monster = rules.adventure().object_type_by_key("base:monster");
    assert!(monster.is_some(), "the original packs' monster");
    setup.map.objects.push(ObjectSetup {
        anchor: near,
        blocked: TileMask::from_bits(1),
        visitable: TileMask::from_bits(1),
        object_type: monster,
        amount: None,
        drawn: TileMask::default(),
        monster: Some(MonsterSetup {
            creature: Some(CreatureTypeId::from_raw(0)),
            count: 30,
            disposition: 4,
            never_flees: true,
            no_growth: false,
            treasure: vec![],
        }),
        guards: None,
    });
    (Arc::new(setup), near)
}

/// [`monster_setup`]'s env (faithful, normal, seed 1, no day limit) after red's hero attacks
/// the monster: the step's events.
fn attacked() -> (AdventureEnv, Vec<GameEvent>) {
    let (setup, near) = monster_setup(&FAITHFUL, h3_game::DEFAULT_DIFFICULTY);
    let mut env = AdventureEnv::new(FAITHFUL.clone(), setup, 1, 0).unwrap();
    let tile = tile_index(env.state(), near).unwrap();
    let mut events = Vec::new();
    env.step_with(RED, Action::move_to(0, tile), &mut events)
        .unwrap();
    (env, events)
}

#[test]
fn a_seat_names_its_actor_and_its_player() {
    assert_eq!(Seat::from(RED), Seat::Player(RED));
    assert_eq!(Seat::Player(RED).actor(), Actor::Player(RED));
    assert_eq!(Seat::Neutral.actor(), Actor::System);
    assert_eq!(
        (Seat::Player(RED).player(), Seat::Neutral.player()),
        (Some(RED), None)
    );
}

/// Before any battle the neutral seat does not move, has no legal action, sees nothing, and
/// nothing decodes or encodes for it; its steps are refused and change nothing.
#[test]
fn the_neutral_seat_waits_outside_a_monsters_battle() {
    let (setup, near) = monster_setup(&FAITHFUL, h3_game::DEFAULT_DIFFICULTY);
    let mut env = AdventureEnv::new(FAITHFUL.clone(), setup, 1, 0).unwrap();
    assert_eq!(env.to_move().collect::<Vec<_>>(), [Seat::Player(RED)]);
    assert_eq!(env.phase(Seat::Neutral), Phase::Battle);
    assert_eq!(env.neutral_observation(), None);
    check_masks(&env, Seat::Neutral).unwrap();
    let tile = tile_index(env.state(), near).unwrap();
    let (state, rules) = (env.state(), env.rules());
    let mut tiles = Vec::new();
    action::tile_mask(state, rules, Seat::Neutral, MOVE_BASE, &mut tiles);
    assert!(!tiles.contains(&true));
    assert_eq!(action::moves(state, rules, Seat::Neutral, 0), None);
    for action in [
        Action::end_turn(),
        Action::move_to(0, tile),
        Action::decline(),
    ] {
        assert_eq!(action::decode(state, rules, Seat::Neutral, action), None);
    }
    for command in [
        GameCommand::EndTurn,
        GameCommand::Answer(h3_game::Answer::Decline),
    ] {
        assert_eq!(action::encode(state, rules, Seat::Neutral, &command), None);
    }
    let walk = action::decode(state, rules, RED, Action::move_to(0, tile)).unwrap();
    assert_eq!(action::encode(state, rules, Seat::Neutral, &walk), None);
    // With nothing to play the random agent's choice is `EndTurn`, which it cannot issue.
    let mut rng = Rng::for_stream(1, StreamId::AGENT);
    assert_eq!(choose(&env, Seat::Neutral, &mut rng), Action::end_turn());
    let hash = env.state_hash();
    assert_eq!(
        env.step(Seat::Neutral, Action::end_turn()),
        Err(StepError::InvalidAction)
    );
    let wait = Action::Battle(battle::Action::entry(0));
    assert_eq!(env.step(Seat::Neutral, wait), Err(StepError::InvalidAction));
    assert_eq!(
        env.step_command(Seat::Neutral, GameCommand::EndTurn, &mut h3_game::NoopSink),
        Err(StepError::Rejected(GameCommandError::NotForSystem))
    );
    assert_eq!((env.state_hash(), env.steps()), (hash, 0));
}

// covers: ADV-MON-009, GAM-BAT-001
/// Red's step toward the monster opens its battle; from then on exactly one seat moves at a
/// time: red while its side is to act, the neutral seat while the monster's is, with the
/// battle's masks for that side, the monster's side's observation, and every action of the
/// other seat refused; the random agent plays both to the battle's end, after which red's turn
/// goes on.
#[test]
fn the_neutral_seat_plays_the_monsters_side_to_the_battles_end() {
    let (mut env, events) = attacked();
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::BattleStarted { .. }))
    );
    assert_eq!(env.state().battle(RED).unwrap().owners(), [Some(RED), None]);
    let mut rng = Rng::for_stream(3, StreamId::AGENT);
    let mut events = Vec::new();
    let (mut neutral, mut red) = (0, 0);
    while env.phase(RED) == Phase::Battle {
        let movers: Vec<Seat> = env.to_move().collect();
        let unowned = env.state().unowned_battle().is_some();
        let (seat, other) = if unowned {
            (Seat::Neutral, Seat::Player(RED))
        } else {
            (Seat::Player(RED), Seat::Neutral)
        };
        assert_eq!(movers, [seat]);
        check_masks(&env, Seat::Neutral).unwrap();
        check_masks(&env, RED).unwrap();
        let observation = env.neutral_observation();
        assert_eq!(observation.is_some(), unowned);
        if let Some(observation) = observation {
            assert_eq!(observation.viewer, BattleSide::Defender);
            assert!(!observation.spectator);
        }
        let action = choose(&env, seat, &mut rng);
        assert!(matches!(action, Action::Battle(_)));
        let hash = env.state_hash();
        assert!(matches!(
            env.step(other, action),
            Err(StepError::InvalidAction | StepError::Rejected(GameCommandError::NotYourSide))
        ));
        assert_eq!(env.state_hash(), hash);
        env.step_with(seat, action, &mut events).unwrap();
        if unowned {
            neutral += 1;
        } else {
            red += 1;
        }
        assert!(neutral + red < 5000, "the battle did not end");
    }
    assert!(neutral > 0 && red > 0, "{neutral} {red}");
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::BattleEnded { .. }))
    );
    assert_eq!(env.neutral_observation(), None);
    let movers: Vec<Seat> = env.to_move().collect();
    assert!(
        movers.is_empty() || movers == [Seat::Player(RED)],
        "{movers:?}"
    );
}

/// The random rollout plays the neutral seat too: from the monster's battle it goes on to the
/// day limit (before E08 T20b3a it stopped at the monster's first turn).
#[test]
fn the_random_rollout_plays_through_a_monsters_battle() {
    let (setup, near) = monster_setup(&FAITHFUL, h3_game::DEFAULT_DIFFICULTY);
    let mut env = AdventureEnv::new(FAITHFUL.clone(), setup, 1, 3).unwrap();
    let tile = tile_index(env.state(), near).unwrap();
    env.step(RED, Action::move_to(0, tile)).unwrap();
    assert_eq!(env.phase(RED), Phase::Battle);
    let result = random_rollout(&mut env, 5, 0);
    assert_eq!(result.day, 4);
    assert!(env.is_done());
    assert_eq!(env.state().unowned_battle(), None);
    assert_eq!(env.to_move().count(), 0);
}

/// A game with a monster's battle, recorded with each command's actor (the system actor's for
/// the monster's side), verifies; one of its commands under red is refused at its step.
#[test]
fn a_replay_records_the_system_actors_commands() {
    let (setup, near) = monster_setup(&FAITHFUL, h3_game::DEFAULT_DIFFICULTY);
    let mut env = AdventureEnv::new(FAITHFUL.clone(), Arc::clone(&setup), 1, 0).unwrap();
    let tile = tile_index(env.state(), near).unwrap();
    let mut commands = Vec::new();
    let mut rng = Rng::for_stream(2, StreamId::AGENT);
    let mut play = |env: &mut AdventureEnv, seat: Seat, action| {
        let (_, command) = env
            .step_decoded(seat, action, &mut h3_game::NoopSink)
            .unwrap();
        commands.push((seat.actor(), command));
    };
    play(&mut env, Seat::Player(RED), Action::move_to(0, tile));
    while env.phase(RED) == Phase::Battle {
        let seat = env.to_move().next().unwrap();
        let action = choose(&env, seat, &mut rng);
        play(&mut env, seat, action);
    }
    let system = commands
        .iter()
        .position(|(actor, _)| *actor == Actor::System)
        .expect("the monster's side acted");
    let replay = replay::Replay {
        header: replay::ReplayHeader::current(&FAITHFUL),
        map: duel_map(),
        seed: 1,
        difficulty: h3_game::DEFAULT_DIFFICULTY,
        commands,
        checkpoint_every: 0,
        checkpoints: Vec::new(),
        final_day: env.day(),
        final_hash: env.state_hash(),
    };
    assert_eq!(replay.verify(&FAITHFUL, &duel_map(), &setup), Ok(()));
    let mut wrong = replay.clone();
    wrong.commands[system].0 = Actor::Player(RED);
    assert_eq!(
        wrong.verify(&FAITHFUL, &duel_map(), &setup),
        Err(replay::ReplayError::Rejected {
            step: system,
            error: GameCommandError::NotYourSide
        })
    );
}

/// [`replay::Replay::record_random`] plays the neutral seat as the random rollout does: on the
/// monster's map, from a seed whose random play meets it, the recording holds the system actor's
/// commands and verifies.
#[test]
fn a_random_recording_plays_the_neutral_seat() {
    let (setup, _) = monster_setup(&FAITHFUL, h3_game::DEFAULT_DIFFICULTY);
    let found = (0..64).find_map(|seed| {
        let options = replay::RecordOptions {
            seed,
            agent_seed: seed,
            max_days: 3,
            max_steps: 0,
            checkpoint_every: 50,
        };
        let replay =
            replay::Replay::record_random(&FAITHFUL, Arc::clone(&setup), duel_map(), options)
                .unwrap();
        replay
            .commands
            .iter()
            .any(|(actor, _)| *actor == Actor::System)
            .then_some(replay)
    });
    let replay = found.expect("a seed whose random play fights the monster");
    assert_eq!(replay.verify(&FAITHFUL, &duel_map(), &setup), Ok(()));
}
