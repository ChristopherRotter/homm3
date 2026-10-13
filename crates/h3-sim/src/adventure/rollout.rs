//! The uniformly random legal agent of the adventure (E08 T9a; T9b's benchmark workload and
//! golden replays play it; it answers an open decision since T14b).

use h3_adventure::Reach;
use h3_contracts::PlayerId;
use h3_game::{Actor, legal_commands};
use h3_kernel::{Rng, StreamId};

use super::action::{self, Action, HERO_SLOTS, tile_index};
use super::env::AdventureEnv;

/// Result of a rollout.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RolloutResult {
    /// Steps played.
    pub steps: u32,
    /// The day the rollout stopped on.
    pub day: u32,
}

/// Plays every player with uniformly random legal actions ([`choose`]) until the episode ends,
/// or after `max_steps` steps when `max_steps > 0`. Increment 1's games end only by the env's day
/// limit: with neither a day limit nor `max_steps` the rollout never returns. The agent's
/// randomness comes from the AGENT stream of `agent_seed`, so it never perturbs the game's own
/// streams.
///
/// Never inlined: T9b's `bench-check` workload counts the instructions inside this function.
#[inline(never)]
pub fn random_rollout(env: &mut AdventureEnv, agent_seed: u64, max_steps: u32) -> RolloutResult {
    let mut rng = Rng::for_stream(agent_seed, StreamId::AGENT);
    let mut steps = 0;
    loop {
        let Some(player) = env.to_move().next() else {
            break;
        };
        if max_steps != 0 && steps >= max_steps {
            break;
        }
        let action = choose(env, player, &mut rng);
        let result = env.step(player, action);
        // A legal action is always accepted (`tests.rs`).
        debug_assert!(result.is_ok(), "{action:?} of {player:?}: {result:?}");
        steps += 1;
    }
    RolloutResult {
        steps,
        day: env.day(),
    }
}

/// The agent's choice for `player`: a uniform legal entry, then for a move a uniform tile of the
/// hero's tile mask, as an agent samples the factorized action head by head. The entries without a
/// tile are the commands [`legal_commands`] lists: `EndTurn`, or with an open decision its answers
/// alone (`Decline`, and `Accept` while the offer stands), each at even odds (GAM-DEC-001; E08
/// T14b, T15a). `EndTurn` when the player has no legal entry.
pub fn choose(env: &AdventureEnv, player: PlayerId, rng: &mut Rng) -> Action {
    let (state, rules) = (env.state(), env.rules());
    let commands = legal_commands(state, rules, Actor::Player(player));
    let moves: Vec<(usize, Reach)> = (0..HERO_SLOTS)
        .filter_map(|slot| action::moves(state, rules, player, slot).map(|reach| (slot, reach)))
        .collect();
    let entries = commands.len() + moves.len();
    let i = rng.below(u32::try_from(entries).unwrap_or(u32::MAX)) as usize;
    let Some((slot, reach)) = i.checked_sub(commands.len()).and_then(|m| moves.get(m)) else {
        return commands
            .get(i)
            .and_then(|command| action::encode(state, player, command))
            .unwrap_or_else(Action::end_turn);
    };
    let tiles = u32::try_from(reach.tiles().count()).unwrap_or(u32::MAX);
    let k = rng.below(tiles) as usize;
    reach
        .tiles()
        .nth(k)
        .and_then(|(pos, _)| tile_index(state, pos))
        .map_or_else(Action::end_turn, |tile| Action::move_to(*slot, tile))
}
