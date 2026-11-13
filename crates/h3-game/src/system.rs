//! The system actor's own commands (ADR-0017 point 2 as amended by E16 M5 T14; SRV-Q31): what
//! the trusted host does for a player whose time ran out, each a command of its own so that a
//! journal replays it: the player's turn ended, its first open decision answered by
//! [`crate::default_answer`] (a level-up past the game's hold too), a command to its battle side.
//! Each is the player's own command checked and applied as the player's, refused unless it
//! applies; no player issues one (`GameCommandError::SystemOnly`). The host's policy (when a
//! turn's time runs out, which battle command its computer plays) stays outside the state.

use h3_contracts::PlayerId;

use crate::command::{GameCommandError, Sink};
use crate::{GameRuleset, GameState};

/// A command of the system actor for a player (ADR-0017 point 2; E16 M5 T14), issued only as
/// `GameCommand::System` by `Actor::System`. Removing a player comes with E10.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemCommand {
    /// Ends `player`'s turn, as its own `EndTurn` (GAM-SYS-001): refused unless its turn is
    /// running and it has no open decision (the host answers them first).
    EndTurn {
        /// The player whose turn ends.
        player: PlayerId,
    },
    /// Answers `player`'s first open decision with its default ([`crate::default_answer`],
    /// GAM-SYS-002), in its turn or not; a level-up's too while the game waits for it: the hold
    /// admits its owner's answer from the system. Refused for a battle (its commands are
    /// [`Self::Battle`]) and without an open decision.
    Answer {
        /// The player whose decision is answered.
        player: PlayerId,
    },
    /// A command to the battle of `player`'s first open decision, for the side it owns while
    /// that side is to act, as its own `Battle` (GAM-SYS-003): a timed-out player's side played
    /// by the host's computer.
    Battle {
        /// The player whose side acts.
        player: PlayerId,
        /// The battle command.
        command: h3_battle::Command,
    },
}

impl SystemCommand {
    /// The player the command acts for: the game's hold judges it as that player's.
    pub const fn player(self) -> PlayerId {
        match self {
            Self::EndTurn { player } | Self::Answer { player } | Self::Battle { player, .. } => {
                player
            }
        }
    }
}

/// Applies the system's `command` (GAM-SYS-001 to GAM-SYS-003): the player's own command, with
/// the default answer for [`SystemCommand::Answer`].
///
/// # Errors
///
/// The player's command's errors; for an answer without a default `UnknownPlayer`, then
/// `BattleOpen` when the first decision is a battle, else `NoDecision`. The state is then
/// unchanged.
// rule: GAM-SYS-001, GAM-SYS-002, GAM-SYS-003
pub(crate) fn run<S: Sink>(
    state: &mut GameState,
    rules: &GameRuleset,
    command: SystemCommand,
    sink: &mut S,
) -> Result<(), GameCommandError> {
    match command {
        SystemCommand::EndTurn { player } => state.end_turn(rules, player, sink),
        SystemCommand::Answer { player } => match crate::default_answer(state, rules, player) {
            Some(answer) => state.answer(rules, player, answer, sink),
            None => {
                state.check_player(player)?;
                Err(if state.battle(player).is_some() {
                    GameCommandError::BattleOpen
                } else {
                    GameCommandError::NoDecision
                })
            }
        },
        SystemCommand::Battle { player, command } => {
            state.battle_command(rules, player, command, sink)
        }
    }
}
