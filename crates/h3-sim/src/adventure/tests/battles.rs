//! The env with battles (E08 T19b1, action encoding v3): a move onto another player's hero is a
//! legal destination of the masks, the decoder and the random agent, and opens a battle both
//! players own (GAM-BAT-004); its owners act in the battle phase (the map masks empty, the battle
//! encoding's masks for the side to act), `to_move` yields the owner of the side to act, the
//! random agent plays it to its end, a fork carries it, and a replay records and verifies its
//! commands with their actors.

use h3_adventure::Pos;
use h3_contracts::BattleResult;
use h3_game::GameEvent;

use super::*;

const RED: PlayerId = PlayerId::new(0);
const BLUE: PlayerId = PlayerId::new(1);

/// The duel at `difficulty` with blue's first hero two steps from red's hero in slot 0 (on the
/// first tile red's day-1 query reaches by a path of two steps), and that tile.
pub(super) fn battle_setup(rules: &GameRuleset, difficulty: u8) -> (Arc<GameSetup>, Pos) {
    let mut setup = GameSetup::clone(&setup(rules));
    setup.difficulty = difficulty;
    let state = GameState::new(rules, &setup, 1).unwrap();
    let hero = hero_in_slot(&state, RED, 0).unwrap();
    let reach = legal_moves(&state, rules, Actor::Player(RED), hero).unwrap();
    let (near, _) = reach
        .tiles()
        .find(|&(pos, _)| reach.path(pos).unwrap().len() == 2)
        .unwrap();
    let blue = setup.heroes.iter().position(|h| h.owner == BLUE).unwrap();
    setup.heroes[blue].pos = near;
    (Arc::new(setup), near)
}

/// [`battle_setup`]'s env (faithful, normal, seed 1, no day limit) and the tile index of blue's
/// hero.
fn battle_env() -> (AdventureEnv, u32) {
    let (setup, near) = battle_setup(&FAITHFUL, h3_game::DEFAULT_DIFFICULTY);
    let env = AdventureEnv::new(FAITHFUL.clone(), setup, 1, 0).unwrap();
    let tile = tile_index(env.state(), near).unwrap();
    (env, tile)
}

/// [`battle_env`] after red's attack: the step's events.
fn attacked() -> (AdventureEnv, Vec<GameEvent>) {
    let (mut env, tile) = battle_env();
    let mut events = Vec::new();
    env.step_with(RED, Action::move_to(0, tile), &mut events)
        .unwrap();
    (env, events)
}

/// Every battle mask of `player` is empty.
pub(super) fn assert_no_battle_masks(env: &AdventureEnv, player: PlayerId) {
    let mut legal = LegalCommands::new();
    let mut entries = [true; battle::ACTION_COUNT];
    env.battle_mask(player, &mut legal, &mut entries);
    let mut spells = [true; battle::SPELL_COUNT];
    env.spell_mask(player, 0, &mut legal, &mut spells);
    let mut targets = [true; battle::TARGET_COUNT];
    env.target_mask(player, (0, 0), &mut legal, &mut targets);
    let mut seconds = [true; battle::TARGET_COUNT];
    env.second_mask(player, (0, 0, 0), &mut legal, &mut seconds);
    assert!(!entries.contains(&true) && !spells.contains(&true));
    assert!(!targets.contains(&true) && !seconds.contains(&true));
}

// covers: GAM-BAT-004
/// Another player's hero's tile is a destination of red's tile mask, decoding to the query's
/// path and chosen by the random agent; the step opens the battle both players own, whose owners
/// then act in the battle phase: the owner of the side to act moves, with the battle's masks.
#[test]
fn a_move_onto_another_players_hero_opens_a_battle_in_the_battle_phase() {
    let (env, tile) = battle_env();
    let (state, rules) = (env.state(), env.rules());
    let mut tiles = Vec::new();
    env.tile_mask(RED, MOVE_BASE, &mut tiles);
    assert!(tiles[tile as usize]);
    let hero = hero_in_slot(state, RED, 0).unwrap();
    let reach = legal_moves(state, rules, Actor::Player(RED), hero).unwrap();
    assert_eq!(tiles.iter().filter(|&&t| t).count(), reach.tiles().count());
    let near = tile_pos(state, tile).unwrap();
    let path = reach.path(near).unwrap();
    assert_eq!(
        action::decode(state, rules, RED, Action::move_to(0, tile)),
        Some(GameCommand::Adventure(AdventureCommand::MoveHero { hero, path }))
    );
    let attacks = (0..400)
        .map(|seed| choose(&env, RED, &mut Rng::for_stream(seed, StreamId::AGENT)))
        .filter(|&a| a == Action::move_to(0, tile))
        .count();
    assert!(attacks > 0);
    for player in [RED, BLUE] {
        assert_eq!(env.phase(player), Phase::Adventure);
        assert_no_battle_masks(&env, player);
    }
    let (env, events) = attacked();
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::BattleStarted { .. }))
    );
    let owner = env.state().battle(RED).unwrap().owner_to_act().unwrap();
    assert_eq!(env.to_move().collect::<Vec<_>>(), [owner]);
    for player in [RED, BLUE] {
        assert_eq!(env.phase(player), Phase::Battle);
        check_masks(&env, player).unwrap();
        let mut entries = [true; ENTRY_COUNT];
        env.legal_mask(player, &mut entries);
        assert_eq!(entries, [false; ENTRY_COUNT]);
    }
}

/// In a battle the owner of the side to act steps its battle actions; the other owner's are
/// refused (`NotYourSide`), as is a map action of either (the battle waits), and an action of the
/// other phase decodes to nothing: a battle action outside a battle, a battle command of a player
/// outside it.
#[test]
fn only_the_owner_of_the_side_to_act_steps_its_battle() {
    let (mut env, _) = attacked();
    let owner = acting(&env);
    let other = if owner == RED { BLUE } else { RED };
    let action = choose(&env, owner, &mut Rng::for_stream(1, StreamId::AGENT));
    assert!(matches!(action, Action::Battle(_)));
    let hash = env.state_hash();
    assert_eq!(
        env.step(other, action),
        Err(StepError::Rejected(GameCommandError::NotYourSide))
    );
    for player in [RED, BLUE] {
        assert!(matches!(
            env.step(player, Action::end_turn()),
            Err(StepError::Rejected(_))
        ));
    }
    let past = Action::Battle(battle::Action::entry(battle::ACTION_COUNT));
    assert_eq!(env.step(owner, past), Err(StepError::InvalidAction));
    assert_eq!(env.state_hash(), hash);
    env.step(owner, action).unwrap();
    // Before the battle: no battle action decodes, no battle command encodes.
    let (before, _) = battle_env();
    let (state, rules) = (before.state(), before.rules());
    assert_eq!(action::decode(state, rules, RED, action), None);
    let command = GameCommand::Battle(h3_battle::Command::Wait);
    assert_eq!(action::encode(state, rules, RED, &command), None);
}

/// The random agent plays the battle to its end (the battle's agent never retreats or
/// surrenders): every step is the owner of the side to act's and is accepted; then both players
/// are back in the map phase and red's turn goes on.
#[test]
fn the_random_agent_plays_a_battle_to_its_end() {
    let (mut env, _) = attacked();
    let mut rng = Rng::for_stream(4, StreamId::AGENT);
    let mut events = Vec::new();
    let mut steps = 0;
    while env.phase(RED) == Phase::Battle {
        let player = acting(&env);
        assert_eq!(env.to_move().count(), 1);
        let action = choose(&env, player, &mut rng);
        env.step_with(player, action, &mut events).unwrap();
        steps += 1;
        assert!(steps < 5000, "the battle did not end");
    }
    let result = events.iter().find_map(|e| match e {
        GameEvent::BattleEnded { result, .. } => Some(*result),
        _ => None,
    });
    assert!(
        matches!(result, Some(BattleResult::Victory(_) | BattleResult::Draw)),
        "{result:?}"
    );
    assert_eq!(env.phase(BLUE), Phase::Adventure);
    assert_eq!(env.to_move().collect::<Vec<_>>(), [RED]);
    for player in [RED, BLUE] {
        check_masks(&env, player).unwrap();
    }
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 8, ..ProptestConfig::default() })]

    /// Clone equivalence with a battle open: a fork playing on leaves its parent as it was, and a
    /// fork and its parent evolve identically under the same agent.
    #[test]
    fn a_fork_with_a_battle_open_evolves_as_its_parent(agent in any::<u64>(), prefix in 0u32..30) {
        let (mut parent, _) = attacked();
        random_rollout(&mut parent, agent, prefix);
        let hash = parent.state_hash();
        let mut wild = parent.fork();
        random_rollout(&mut wild, agent ^ 1, 40);
        prop_assert_eq!(parent.state_hash(), hash);
        let mut fork = parent.fork();
        let a = random_rollout(&mut parent, agent, 40);
        let b = random_rollout(&mut fork, agent, 40);
        prop_assert_eq!(a, b);
        prop_assert_eq!(parent.state(), fork.state());
    }
}

/// A game with a battle, recorded command by command with each command's actor (the attacked
/// player's battle commands outside its turn), verifies; a battle command under the other owner
/// is refused at its step.
#[test]
fn a_replay_records_a_battle_with_its_actors() {
    let (setup, near) = battle_setup(&FAITHFUL, h3_game::DEFAULT_DIFFICULTY);
    let mut env = AdventureEnv::new(FAITHFUL.clone(), Arc::clone(&setup), 1, 0).unwrap();
    let tile = tile_index(env.state(), near).unwrap();
    let mut commands = Vec::new();
    let mut rng = Rng::for_stream(2, StreamId::AGENT);
    let mut play = |env: &mut AdventureEnv, player, action| {
        let (_, command) = env
            .step_decoded(player, action, &mut h3_game::NoopSink)
            .unwrap();
        commands.push((Actor::Player(player), command));
    };
    play(&mut env, RED, Action::move_to(0, tile));
    while env.phase(RED) == Phase::Battle {
        let player = acting(&env);
        let action = choose(&env, player, &mut rng);
        play(&mut env, player, action);
    }
    play(&mut env, RED, Action::end_turn());
    let blue_moved = commands
        .iter()
        .position(|(actor, c)| *actor == Actor::Player(BLUE) && matches!(c, GameCommand::Battle(_)));
    assert!(blue_moved.is_some(), "blue's side never acted");
    let replay = replay::Replay {
        header: replay::ReplayHeader::current(&FAITHFUL),
        map: duel_map(),
        seed: 1,
        difficulty: h3_game::DEFAULT_DIFFICULTY,
        commands,
        checkpoint_every: 7,
        checkpoints: Vec::new(),
        final_day: env.day(),
        final_hash: env.state_hash(),
    };
    let mut state = GameState::new(&FAITHFUL, &setup, 1).unwrap();
    let mut checkpoints = Vec::new();
    for (i, (actor, command)) in replay.commands.iter().enumerate() {
        apply(&mut state, &FAITHFUL, *actor, command.clone(), &mut h3_game::NoopSink).unwrap();
        if (i + 1) % 7 == 0 {
            checkpoints.push(state.state_hash());
        }
    }
    let replay = replay::Replay {
        checkpoints,
        ..replay
    };
    assert_eq!(replay.verify(&FAITHFUL, &duel_map(), &setup), Ok(()));
    let mut wrong = replay.clone();
    let step = blue_moved.unwrap();
    wrong.commands[step].0 = Actor::Player(RED);
    assert_eq!(
        wrong.verify(&FAITHFUL, &duel_map(), &setup),
        Err(replay::ReplayError::Rejected {
            step,
            error: GameCommandError::NotYourSide
        })
    );
}
