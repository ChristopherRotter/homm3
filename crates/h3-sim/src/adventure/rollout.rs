//! The uniformly random legal agent of the adventure (E08 T9a; T9b's benchmark workload and
//! golden replays play it; it answers an open decision since T14b, plays its battles since T19b1
//! and the neutral seat's since T20b3a; a town's build since E09 T19c1, encoding v9's entry; the
//! listed builds skipped before it, E09 T18b2b, decision 56; a town's hire since E09 T38a2,
//! encoding v12; the listed casts skipped from E08 T144a1 to encoding v21, T144a2, which draws
//! them).

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
/// head. The other choices are the commands [`legal_commands`] lists: `EndTurn`, each Shipyard's
/// `BuildBoat` (by its anchor, E08 T85b2b; before encoding v6 a drawn `BuildBoat` had no action
/// and ended the turn), each hero's `Revisit` (E08 T105b2, encoding v8: at 0 points too, so a
/// turn's revisits end only by the draw of another entry) and each town's build (by its town's
/// entrance and its building, E09 T19c1, encoding v9; from E09 T18b2b to T19c1 the listed builds
/// were left out of the draw, decision 56), its recruitments and hero moves (E09 T27, T28a) and
/// its hires (by its entrance and the slot, E09 T38a2, encoding v12) and its heroes' casts of
/// Summon Boat, Fly and Water Walk (encoding v21, E08 T144a2; left out of the draw from T144a1 to
/// it), or with an open decision its answers alone (`Decline`, and `Accept` while the offer
/// stands; a level-up's `First` and `Second`, E08 T30), each at even odds with each hero's move
/// (GAM-DEC-001, GAM-DEC-002; E08 T14b, T15a) and each hero's cast of Scuttle Boat or Dimension
/// Door that has a target (E08 T144a2: the hero's query, [`action::targeted_casts`]), whose target
/// is then uniform among the queried ones, as a move's tile (the adventure lane's proposal to the
/// AI lane, which chooses the bots' weights). `EndTurn` when the seat has
/// no legal entry (the neutral seat while no battle waits for it). In the battle phase: the
/// battle's random agent for
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
    // E08 T144a2: the targeted casts by their heroes' queries, after the moves (so a game
    // without them draws as before).
    let casts = action::targeted_casts(state, rules, seat);
    let entries = commands.len() + moves.len() + casts.len();
    let i = rng.below(u32::try_from(entries).unwrap_or(u32::MAX)) as usize;
    if let Some(command) = commands.get(i) {
        return action::encode(state, rules, seat, command).unwrap_or_else(Action::end_turn);
    }
    let m = i - commands.len();
    let Some((slot, reach)) = moves.get(m) else {
        return casts.get(m - moves.len()).map_or_else(
            Action::end_turn,
            |(slot, spell, targets)| {
                let count = u32::try_from(targets.len()).unwrap_or(u32::MAX);
                let target = targets[rng.below(count) as usize];
                tile_index(state, target)
                    .map_or_else(Action::end_turn, |tile| Action::cast(*slot, *spell, tile))
            },
        );
    };
    let count = u32::try_from(reach.tiles().count()).unwrap_or(u32::MAX);
    let k = rng.below(count) as usize;
    reach
        .tiles()
        .nth(k)
        .and_then(|(pos, _)| tile_index(state, pos))
        .map_or_else(Action::end_turn, |tile| Action::move_to(*slot, tile))
}
