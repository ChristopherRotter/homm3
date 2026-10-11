//! Tests of the adventure environment on the committed generated map `fixtures/maps/e08-duel-36.h3m`
//! (`h3-map`'s `tests/fixtures.rs` lays it out): the masks against the game's legal commands and
//! moves, steps against `apply`, refused actions, forks, determinism, the day limit.

use std::sync::{Arc, LazyLock};

use h3_adventure::AdventureCommand;
use h3_contracts::PlayerId;
use h3_game::{
    Actor, GameCommand, GameCommandError, GameRuleset, GameSetup, apply, legal_moves, observe,
};
use h3_kernel::{Rng, StreamId};
use proptest::prelude::*;

use super::action::{self, hero_in_slot, tile_index, tile_pos};
use super::rollout::choose;
use super::*;

const DUEL: &[u8] = include_bytes!("../../../../fixtures/maps/e08-duel-36.h3m");

fn rules(profile: &str) -> GameRuleset {
    let (core, part) =
        h3_pack_adventure::compile(&h3_pack_adventure::ORIGINAL_PACKS, profile).unwrap();
    GameRuleset::new(Arc::new(core), Arc::new(part)).unwrap()
}

static FAITHFUL: LazyLock<GameRuleset> = LazyLock::new(|| rules("base:faithful"));
static STRICT: LazyLock<GameRuleset> = LazyLock::new(|| rules("base:strict-fixed"));

fn setup(rules: &GameRuleset) -> Arc<GameSetup> {
    let m = h3_map::read(DUEL, &h3_map::Limits::default()).unwrap();
    Arc::new(h3_map::import(&m, rules).unwrap())
}

static DUEL_SETUP: LazyLock<Arc<GameSetup>> = LazyLock::new(|| setup(&FAITHFUL));

fn env(seed: u64, max_days: u32) -> AdventureEnv {
    AdventureEnv::new(FAITHFUL.clone(), Arc::clone(&DUEL_SETUP), seed, max_days).unwrap()
}

fn acting(env: &AdventureEnv) -> PlayerId {
    env.to_move().next().unwrap()
}

#[test]
fn the_duel_starts_with_red_to_move_and_both_its_heroes_free() {
    let env = env(1, 28);
    assert_eq!(env.tile_count(), 36 * 36 * 2);
    assert_eq!(env.to_move().collect::<Vec<_>>(), [PlayerId::new(0)]);
    assert_eq!((env.day(), env.steps(), env.is_done()), (1, 0, false));
    let mut mask = [true; ENTRY_COUNT];
    env.legal_mask(PlayerId::new(0), &mut mask);
    assert_eq!(
        mask,
        [true, true, true, false, false, false, false, false, false]
    );
    env.legal_mask(PlayerId::new(1), &mut mask);
    assert_eq!(mask, [false; ENTRY_COUNT]);
    let mut tiles = Vec::new();
    env.tile_mask(PlayerId::new(0), END_TURN, &mut tiles);
    assert_eq!(tiles.len(), env.tile_count());
    assert_eq!(tiles.iter().filter(|&&t| t).count(), 1);
    assert!(tiles[0]);
    env.tile_mask(PlayerId::new(1), END_TURN, &mut tiles);
    assert!(!tiles.iter().any(|&t| t));
    assert_eq!(
        env.observation(PlayerId::new(1)),
        observe(
            env.state(),
            env.rules(),
            h3_game::Viewer::Player(PlayerId::new(1))
        )
    );
}

#[test]
fn tiles_and_actions_encode_as_documented() {
    let env = env(1, 0);
    let state = env.state();
    let pos = h3_adventure::Pos::new(5, 7, 1);
    assert_eq!(tile_index(state, pos), Some((36 + 7) * 36 + 5));
    assert_eq!(tile_pos(state, (36 + 7) * 36 + 5), Some(pos));
    assert_eq!(tile_index(state, h3_adventure::Pos::new(36, 0, 0)), None);
    assert_eq!(tile_index(state, h3_adventure::Pos::new(0, 36, 0)), None);
    assert_eq!(tile_index(state, h3_adventure::Pos::new(0, 0, 2)), None);
    assert_eq!(tile_pos(state, 36 * 36 * 2), None);
    assert_eq!(Action::end_turn(), Action::default());
    assert_eq!(Action::move_to(2, 9), Action { entry: 3, tile: 9 });
    assert_eq!(hero_in_slot(state, PlayerId::new(0), HERO_SLOTS), None);
    assert_eq!(hero_in_slot(state, PlayerId::new(1), 2), None);
    assert!(hero_in_slot(state, PlayerId::new(1), 1).is_some());
    for (error, text) in [
        (StepError::InvalidAction, "invalid action"),
        (StepError::EpisodeOver, "the episode is over"),
        (
            StepError::Rejected(GameCommandError::NotYourTurn(PlayerId::new(1))),
            "rejected: it is not player 1's turn",
        ),
    ] {
        assert_eq!(error.to_string(), text);
    }
}

/// One step of the property test's play.
#[derive(Debug, Clone)]
enum Pick {
    /// The acting player's legal action, as the random agent picks it from this stream.
    Legal(u64),
    /// An arbitrary action of the acting player (`None`) or of a player of the game or the one
    /// after the last: the entry and tile modulo one more than their counts.
    Free {
        player: Option<u8>,
        entry: u8,
        tile: u32,
    },
}

fn pick() -> impl Strategy<Value = Pick> {
    prop_oneof![
        4 => any::<u64>().prop_map(Pick::Legal),
        1 => (proptest::option::of(any::<u8>()), any::<u8>(), any::<u32>())
            .prop_map(|(player, entry, tile)| Pick::Free { player, entry, tile }),
    ]
}

/// The masks of `player` agree with the game: an entry is legal exactly when its tile mask has a
/// tile; a move's tiles are its hero's legal moves but its own tile, and each decodes to the
/// query's path.
fn check_masks(env: &AdventureEnv, player: PlayerId) -> Result<(), TestCaseError> {
    let (state, rules) = (env.state(), env.rules());
    let mut entries = [false; ENTRY_COUNT];
    env.legal_mask(player, &mut entries);
    let mut tiles = Vec::new();
    for (entry, &legal) in entries.iter().enumerate() {
        env.tile_mask(player, entry, &mut tiles);
        prop_assert_eq!(tiles.len(), env.tile_count());
        prop_assert_eq!(legal, tiles.iter().any(|&t| t), "entry {}", entry);
        let Some(hero) = entry
            .checked_sub(MOVE_BASE)
            .and_then(|slot| hero_in_slot(state, player, slot))
        else {
            continue;
        };
        let reach = legal_moves(state, rules, Actor::Player(player), hero);
        let listed: Vec<u32> = reach
            .iter()
            .flat_map(|r| r.tiles())
            .map(|(pos, _)| tile_index(state, pos).unwrap())
            .collect();
        let masked: Vec<u32> = (0u32..)
            .zip(&tiles)
            .filter(|&(_, &t)| t)
            .map(|(i, _)| i)
            .collect();
        prop_assert_eq!(&masked, &listed);
        for &tile in masked.iter().step_by(17) {
            let command = action::decode(state, rules, player, Action { entry, tile });
            let path = reach
                .as_ref()
                .and_then(|r| r.path(tile_pos(state, tile).unwrap()));
            prop_assert_eq!(
                command,
                path.map(|path| GameCommand::Adventure(AdventureCommand::MoveHero { hero, path }))
            );
        }
    }
    Ok(())
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 16, ..ProptestConfig::default() })]

    /// Random play of legal and arbitrary actions: the masks agree with the game for every
    /// player; a legal action is accepted and does what `apply` does with its command; an action
    /// outside the masks is refused and leaves the environment as it was.
    #[test]
    fn steps_agree_with_the_masks_and_the_game(
        seed in any::<u64>(),
        strict in any::<bool>(),
        picks in proptest::collection::vec(pick(), 1..60),
    ) {
        let rules = if strict { &*STRICT } else { &*FAITHFUL };
        let mut env = AdventureEnv::new(rules.clone(), setup(rules), seed, 0).unwrap();
        let players = u8::try_from(env.state().players()).unwrap();
        for pick in &picks {
            for player in (0..players).map(PlayerId::new) {
                check_masks(&env, player)?;
            }
            let actor = acting(&env);
            let (player, action) = match *pick {
                Pick::Legal(stream) => {
                    let mut rng = Rng::for_stream(stream, StreamId::AGENT);
                    (actor, choose(&env, actor, &mut rng))
                }
                Pick::Free { player, entry, tile } => {
                    let player = player.map_or(actor, |p| PlayerId::new(p % (players + 1)));
                    let entry = usize::from(entry) % (ENTRY_COUNT + 1);
                    let tile = tile % (u32::try_from(env.tile_count()).unwrap() + 1);
                    (player, Action { entry, tile })
                }
            };
            let mut entries = [false; ENTRY_COUNT];
            env.legal_mask(player, &mut entries);
            let mut tiles = Vec::new();
            env.tile_mask(player, action.entry, &mut tiles);
            let legal = entries.get(action.entry) == Some(&true)
                && tiles.get(action.tile as usize) == Some(&true);
            let command = action::decode(env.state(), env.rules(), player, action);
            let mut expected = env.state().clone();
            let applied = command.clone().map(|command| {
                apply(&mut expected, env.rules(), Actor::Player(player), command, &mut Vec::new())
            });
            let (hash, steps) = (env.state_hash(), env.steps());
            let mut events = Vec::new();
            let result = env.step_with(player, action, &mut events);
            if legal {
                prop_assert_eq!(result, Ok(StepResult { done: false, truncated: false }));
            }
            match result {
                Ok(_) => {
                    prop_assert!(legal, "{:?} {:?} accepted outside the masks", player, action);
                    prop_assert_eq!(applied, Some(Ok(())));
                    prop_assert_eq!(env.state(), &expected);
                    prop_assert_eq!(env.steps(), steps + 1);
                    prop_assert!(!events.is_empty());
                }
                Err(error) => {
                    prop_assert!(!legal);
                    prop_assert_eq!((env.state_hash(), env.steps()), (hash, steps));
                    prop_assert!(events.is_empty());
                    match error {
                        StepError::InvalidAction => prop_assert_eq!(command, None),
                        StepError::Rejected(e) => prop_assert_eq!(applied, Some(Err(e))),
                        StepError::EpisodeOver => prop_assert!(false, "no day limit"),
                    }
                }
            }
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 12, ..ProptestConfig::default() })]

    /// Clone equivalence: a fork playing on leaves its parent as it was, and a fork and its
    /// parent evolve identically under the same agent.
    #[test]
    fn forks_evolve_as_their_parent(seed in any::<u64>(), agent in any::<u64>(), prefix in 0u32..80) {
        let mut parent = env(seed, 0);
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
        prop_assert_eq!(parent.state_hash(), fork.state_hash());
    }
}

#[test]
fn rollouts_are_deterministic_and_end_at_the_day_limit() {
    let mut a = env(7, 4);
    let mut b = env(7, 4);
    let ra = random_rollout(&mut a, 11, 0);
    assert_eq!(ra, random_rollout(&mut b, 11, 0));
    assert_eq!(a.state_hash(), b.state_hash());
    assert_eq!(ra.day, 5);
    assert!(ra.steps > 8, "{ra:?}");
    assert!(a.is_done());
    assert_eq!(a.to_move().count(), 0);
    let mut mask = [true; ENTRY_COUNT];
    a.legal_mask(PlayerId::new(0), &mut mask);
    assert_eq!(mask, [false; ENTRY_COUNT]);
    let mut tiles = vec![true];
    a.tile_mask(PlayerId::new(0), END_TURN, &mut tiles);
    assert!(!tiles.iter().any(|&t| t));
    assert_eq!(
        a.step(PlayerId::new(0), Action::end_turn()),
        Err(StepError::EpisodeOver)
    );
    // The heroes moved: random play is not only ending turns.
    let start = env(7, 4);
    let moved = a
        .state()
        .adventure()
        .heroes()
        .iter()
        .zip(start.state().adventure().heroes())
        .filter(|(now, then)| now.pos != then.pos)
        .count();
    assert!(moved >= 2, "{moved} heroes moved");
    a.reset(7).unwrap();
    assert_eq!((a.day(), a.steps(), a.is_done()), (1, 0, false));
    assert_eq!(a.state(), start.state());
    // A step limit stops a rollout early.
    let mut c = env(7, 4);
    assert_eq!(random_rollout(&mut c, 11, 5).steps, 5);
}
