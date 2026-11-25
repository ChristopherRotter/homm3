//! The original's computer player hosted (E31 T2a; ADR-0030 points 2, 3, 7): a game every player
//! of which `h3_game::computer::choose` plays, each with its memory and its own stream, the sides
//! of its battles by the original's battle AI (`h3_battle_ai::choose`, the faithful variant's
//! battle seat), the sides no player owns by the same as the system actor. For the property tests
//! (every command legal, a turn ending within a bound of commands, the same command for the same
//! state, memory and stream: [`play`] with `checked`), the shipped maps' check
//! (`examples/original_data.rs`) and the computer's benchmark (`h3 ai computer`). The env's and
//! the server's computer seats are E31 increment 5's.

use h3_contracts::PlayerId;
use h3_game::computer::{Memory, choose};
use h3_game::{Actor, GameCommand, GameCommandError, GameRuleset, GameState, NoopSink, apply};
use h3_kernel::{Rng, StreamId};
use h3_towns::{TownCommand, TownHeroChange};

/// What a game [`play`]ed gave.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Played {
    /// The day the game stopped on: the first past the limit.
    pub day: u32,
    /// The turns ended.
    pub turns: u64,
    /// Every command applied: the players' in their turns, their answers, the battles'.
    pub commands: u64,
    /// The open decisions answered.
    pub answers: u64,
    /// The battle commands, the system actor's included.
    pub battle_commands: u64,
    /// The heroes' moves between a town's two bars, by `TownHeroChange` (`Enter`, `Leave`, `Swap`).
    pub garrison_moves: [u64; 3],
    /// The most commands one turn took in its frame (its answers and battles not counted), its
    /// `EndTurn` included.
    pub longest_turn: u32,
    /// The game state's hash at the end (`GameState::state_hash`).
    pub hash: u64,
}

/// Why [`play`] stopped before its day limit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlayError {
    /// The game refused a command (ADR-0030 point 3: never).
    Refused {
        /// The day.
        day: u32,
        /// Whose command it was.
        actor: Actor,
        /// The command.
        command: GameCommand,
        /// The game's refusal.
        error: GameCommandError,
    },
    /// Two calls on copies of one state, memory and stream disagreed (ADR-0030 point 2).
    Unequal {
        /// The day.
        day: u32,
        /// The player.
        player: PlayerId,
    },
    /// A turn took more commands than its bound ([`turn_bound`]).
    TooLong {
        /// The day.
        day: u32,
        /// The player.
        player: PlayerId,
        /// The bound.
        bound: u32,
    },
    /// No player and no battle had a command: the game cannot go on.
    Stalled {
        /// The day.
        day: u32,
    },
}

/// The most commands of `player`'s turn in its frame, `EndTurn` included, as the frame is built
/// (E31 T2a: each hero done at once): one each for its heroes on the map and its garrison
/// heroes at the turn's start (a garrison step), one more for each garrison hero (sent out),
/// and the end.
pub fn turn_bound(state: &GameState, player: PlayerId) -> u32 {
    let adventure = state.adventure();
    let mine = |owner: PlayerId| owner == player;
    let on_map = adventure.heroes().iter().filter(|h| mine(h.owner)).count();
    let garrisoned = adventure
        .town_heroes()
        .iter()
        .filter(|(_, h)| mine(h.owner))
        .count();
    u32::try_from(on_map + 2 * garrisoned + 1).unwrap_or(u32::MAX)
}

/// Plays `state` to the end of day `days` (until its day is `days + 1`), every player the
/// original's computer ([`choose`], each with a memory and the seat stream of its index under
/// `seed`), the owned battle sides by `h3_battle_ai::choose` as their owners' `Battle` commands
/// and the unowned ones as the system actor's, each battle side drawing from the stream of its
/// owner's seat (the system actor's: [`StreamId::AGENT`]). With `checked` each player's command
/// is asked twice, on copies of its memory and stream, and each turn held to [`turn_bound`].
///
/// # Errors
///
/// [`PlayError`]: a refused command, two answers that differ, a turn past its bound (`checked`),
/// or a game no one can go on with.
pub fn play(
    rules: &GameRuleset,
    mut state: GameState,
    seed: u64,
    days: u32,
    checked: bool,
) -> Result<Played, PlayError> {
    let players = u8::try_from(state.players()).unwrap_or(u8::MAX);
    let mut memories = vec![Memory::new(); usize::from(players)];
    let mut streams: Vec<Rng> = (0..players)
        .map(|index| Rng::for_stream(seed, StreamId::seat(index)))
        .collect();
    let mut system = Rng::for_stream(seed, StreamId::AGENT);
    let mut turn = vec![(0_u32, 0_u32); usize::from(players)];
    let mut played = Played::default();
    while state.day() <= days {
        let day = state.day();
        let (actor, command) = match battle_command(&state, rules, &mut streams, &mut system) {
            Some(found) => {
                played.battle_commands += 1;
                found
            }
            None => {
                let mut found = None;
                for index in 0..players {
                    let player = PlayerId::new(index);
                    let (memory, rng) = (
                        &mut memories[usize::from(index)],
                        &mut streams[usize::from(index)],
                    );
                    let (again, mut other) = (memory.clone(), rng.clone());
                    let Some(command) = choose(&state, rules, player, memory, rng) else {
                        continue;
                    };
                    if checked {
                        let mut again = again;
                        let twice = choose(&state, rules, player, &mut again, &mut other);
                        if twice.as_ref() != Some(&command) || again != *memory || other != *rng {
                            return Err(PlayError::Unequal { day, player });
                        }
                    }
                    found = Some((player, command));
                    break;
                }
                let (player, command) = found.ok_or(PlayError::Stalled { day })?;
                if matches!(command, GameCommand::Answer(_)) {
                    played.answers += 1;
                } else {
                    let (count, bound) = &mut turn[usize::from(player.raw())];
                    if *count == 0 {
                        *bound = turn_bound(&state, player);
                    }
                    *count += 1;
                    if checked && *count > *bound {
                        return Err(PlayError::TooLong {
                            day,
                            player,
                            bound: *bound,
                        });
                    }
                    played.longest_turn = played.longest_turn.max(*count);
                    if command == GameCommand::EndTurn {
                        played.turns += 1;
                        *count = 0;
                    }
                    if let GameCommand::Town(TownCommand::Heroes { change, .. }) = command {
                        played.garrison_moves[move_index(change)] += 1;
                    }
                }
                (Actor::Player(player), command)
            }
        };
        apply(&mut state, rules, actor, command.clone(), &mut NoopSink).map_err(|error| {
            PlayError::Refused {
                day,
                actor,
                command,
                error,
            }
        })?;
        played.commands += 1;
    }
    played.day = state.day();
    played.hash = state.state_hash();
    Ok(played)
}

/// The place of `change` in [`Played::garrison_moves`].
const fn move_index(change: TownHeroChange) -> usize {
    match change {
        TownHeroChange::Enter => 0,
        TownHeroChange::Leave => 1,
        TownHeroChange::Swap => 2,
    }
}

/// The next battle command, if a battle's side is to act: the side an owner owns by its seat's
/// stream, else an unowned side by the system's (the battle AI of the original, `h3_battle_ai`).
fn battle_command(
    state: &GameState,
    rules: &GameRuleset,
    streams: &mut [Rng],
    system: &mut Rng,
) -> Option<(Actor, GameCommand)> {
    let owned = (0..streams.len()).find_map(|index| {
        let player = PlayerId::new(u8::try_from(index).ok()?);
        let battle = state.battle(player)?;
        let side = battle.state().side_to_act()?;
        (battle.owners()[side.index()] == Some(player)).then_some((index, player, side))
    });
    if let Some((index, player, side)) = owned {
        let battle = state.battle(player)?.state();
        let command = h3_battle_ai::choose(battle, rules.battle(), side, &mut streams[index]);
        return Some((Actor::Player(player), GameCommand::Battle(command)));
    }
    let battle = state.unowned_battle()?.state();
    let side = battle.side_to_act()?;
    let command = h3_battle_ai::choose(battle, rules.battle(), side, system);
    Some((Actor::System, GameCommand::Battle(command)))
}

#[cfg(test)]
mod tests;
