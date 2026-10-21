//! The uniformly random legal agent of the adventure (E08 T9a; T9b's benchmark workload and
//! golden replays play it; it answers an open decision since T14b, plays its battles since T19b1
//! and the neutral seat's since T20b3a).

use h3_adventure::Reach;
use h3_battle::LegalCommands;
use h3_game::legal_commands;
use h3_kernel::{Rng, StreamId};

use super::action::{self, Action, HERO_SLOTS, Seat, tile_index};
use super::env::AdventureEnv;

/// Result of a rollout.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RolloutResult {
    /// Steps played.
    pub steps: u32,
    /// The day the rollout stopped on.
    pub day: u32,
}

/// Plays every seat with uniformly random legal actions ([`choose`]) until the episode ends,
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
        let Some(seat) = env.to_move().next() else {
            break;
        };
        if max_steps != 0 && steps >= max_steps {
            break;
        }
        let action = choose(env, seat, &mut rng);
        let result = env.step(seat, action);
        // A legal action is always accepted (`tests.rs`).
        debug_assert!(result.is_ok(), "{action:?} of {seat:?}: {result:?}");
        steps += 1;
    }
    RolloutResult {
        steps,
        day: env.day(),
    }
}

/// The agent's choice for `seat`. In the map phase: a uniform legal entry, then for a move a
/// uniform tile of the hero's tile mask (its reachable tiles, other players' heroes' included:
/// such a move starts a battle, E08 T19b1), as an agent samples the factorized action head by
/// head. The entries without a tile are the commands [`legal_commands`] lists: `EndTurn`, or with
/// an open decision its answers alone (`Decline`, and `Accept` while the offer stands; a level-up's
/// `First` and `Second`, E08 T30), each at even odds (GAM-DEC-001, GAM-DEC-002; E08 T14b, T15a). `EndTurn` when the seat has no legal entry (the
/// neutral seat while no battle waits for it). In the battle phase: the battle's random agent for
/// the seat's side (`crate::rollout::choose`: a uniform entry among the commands that stay in the
/// battle and the casts, never a retreat or a surrender, so a battle ends by a victory or a draw;
/// the neutral seat's stand-in for the original's battle AI, decision 27).
pub fn choose(env: &AdventureEnv, seat: impl Into<Seat>, rng: &mut Rng) -> Action {
    let seat = seat.into();
    let (state, rules) = (env.state(), env.rules());
    if let Some((battle, side)) = action::battle_of(state, seat) {
        let mut legal = LegalCommands::new();
        h3_battle::legal_commands(battle, rules.battle(), side, &mut legal);
        let command = crate::rollout::choose(battle, rules.battle(), side, &legal, rng);
        return crate::action::encode(battle, rules.battle(), command)
            .map_or_else(Action::end_turn, Action::Battle);
    }
    let commands = legal_commands(state, rules, seat.actor());
    let moves: Vec<(usize, Reach)> = (0..HERO_SLOTS)
        .filter_map(|slot| action::moves(state, rules, seat, slot).map(|reach| (slot, reach)))
        .collect();
    let entries = commands.len() + moves.len();
    let i = rng.below(u32::try_from(entries).unwrap_or(u32::MAX)) as usize;
    let Some((slot, reach)) = i.checked_sub(commands.len()).and_then(|m| moves.get(m)) else {
        return commands
            .get(i)
            .and_then(|command| action::encode(state, rules, seat, command))
            .unwrap_or_else(Action::end_turn);
    };
    let count = u32::try_from(reach.tiles().count()).unwrap_or(u32::MAX);
    let k = rng.below(count) as usize;
    reach
        .tiles()
        .nth(k)
        .and_then(|(pos, _)| tile_index(state, pos))
        .map_or_else(Action::end_turn, |tile| Action::move_to(*slot, tile))
}
