//! A battle fought as it is played (E17 M5 T1b): a scenario's battle hosted by `h3-protocol`'s
//! session in-process ([`crate::scenario::hosted`]), both seats held here and a spectator joined.
//! Each command is asked for when the battle's next one is needed: the Greedy bot's
//! ([`crate::bot`]) for the side to act, sent from that side's seat as a client sends it (the
//! session validates it as a server does); what the spectator is sent for it makes the record
//! grow. The bot decides on a copy of the battle kept here by the same commands (as
//! [`crate::scenario::fought`] does), which also gives the mouse each turn's battle and legal
//! commands. An adapter.

use h3_battle::{BattleSetup, BattleSide, BattleState, LegalCommands, NoopSink, Viewer};
use h3_content_battle::BattleRuleset;
use h3_protocol::session::Bot as _;
use h3_protocol::session::Session;
use h3_protocol::wire::{ClientMessage, Command, Observation, Seat, ServerMessage};
use h3_protocol::{codec, map};

use crate::bot::Greedy;
use crate::scenario::{FOUGHT_LIMIT, VIEWER, fetch_record, hosted, seat_conn};

/// The battle being fought: the session, the copy the bot decides on, the commands sent.
#[derive(Debug)]
pub struct Live<'r> {
    ruleset: &'r BattleRuleset,
    session: Session<'r>,
    state: BattleState,
    bot: Greedy,
    legal: LegalCommands,
    sent: u32,
}

impl<'r> Live<'r> {
    /// The battle `setup` sets up with `seed` (its `ruleset` compiled from the scenario's packs),
    /// hosted and started, no command played.
    pub fn new(ruleset: &'r BattleRuleset, setup: BattleSetup, seed: u64) -> Result<Self, String> {
        let state =
            BattleState::new(ruleset, &setup, seed).map_err(|e| format!("the setup: {e:?}"))?;
        Ok(Self {
            ruleset,
            session: hosted(ruleset, setup, seed, true)?,
            state,
            bot: Greedy,
            legal: LegalCommands::new(),
            sent: 0,
        })
    }

    /// The spectator's record so far: its header and every message it was sent.
    pub fn record(&mut self) -> Result<String, String> {
        fetch_record(&mut self.session, VIEWER)
    }

    /// Whether the battle is over (no side is to act).
    pub fn over(&self) -> bool {
        self.state.side_to_act().is_none()
    }

    /// The battle at the turn to come, a spectator's view, and the legal commands of the side to
    /// act (E17 M3 T6c-2: what the mouse's cursor asks); none when the battle is over.
    pub fn turn(&self) -> Option<(Observation, Vec<Command>)> {
        let side = self.state.side_to_act()?;
        let observation = map::observe(&self.state, self.ruleset, Viewer::Spectator);
        Some((
            observation,
            map::legal_commands(&self.state, self.ruleset, side),
        ))
    }

    /// Plays the next command, the bot's for the side to act, sent from its seat: what the
    /// spectator was sent for it, in order (its `Events`, then `GameEnded` when it ended the
    /// battle); none when the battle is over. A command the session rejects, or one too many
    /// ([`FOUGHT_LIMIT`]), is an error.
    pub fn next(&mut self) -> Result<Option<Vec<ServerMessage>>, String> {
        let Some(side) = self.state.side_to_act() else {
            return Ok(None);
        };
        if usize::try_from(self.sent).unwrap_or(usize::MAX) >= FOUGHT_LIMIT {
            return Err(format!("no end after {FOUGHT_LIMIT} commands"));
        }
        h3_battle::legal_commands(&self.state, self.ruleset, side, &mut self.legal);
        let choice = self
            .bot
            .command(&self.state, self.ruleset, side, &self.legal);
        self.sent += 1;
        let (seq, command) = (self.sent, map::command(choice));
        h3_battle::apply(&mut self.state, self.ruleset, side, choice, &mut NoopSink)
            .map_err(|e| format!("command {seq} ({choice:?}): {e:?}"))?;
        let seat = match side {
            BattleSide::Attacker => Seat::Attacker,
            BattleSide::Defender => Seat::Defender,
        };
        let mut out = Vec::new();
        let sent = ClientMessage::Command { seq, command };
        self.session.handle(seat_conn(seat), sent, &mut out);
        let mut messages = Vec::new();
        for frame in out {
            let message = codec::decode_server(frame.frame.as_bytes(), usize::MAX)
                .map_err(|e| format!("command {seq}: the session's frame: {e:?}"))?;
            if let ServerMessage::Rejected { code, .. } = message {
                return Err(format!("command {seq} ({command:?}): rejected, {code:?}"));
            }
            if frame.to == VIEWER {
                messages.push(message);
            }
        }
        Ok(Some(messages))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};

    fn repo() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
    }

    #[test]
    fn a_battle_fought_as_it_is_played_records_what_the_bot_fights_at_once() {
        // The duel fought command by command: the spectator's messages, each command's as it is
        // sent, are the record of the battle the bot fights at once (`scenario::fought`).
        let (scenario, packs) = (
            repo().join("packs/synthetic/scenarios/duel.toml"),
            repo().join("packs"),
        );
        let (ruleset, setup) = crate::scenario::battle(&scenario, &packs, 1).unwrap();
        let mut live = Live::new(&ruleset, setup, 1).unwrap();
        let opening = h3_view::record::parse(&live.record().unwrap()).unwrap();
        let mut messages = opening.messages.clone();
        let mut turns = 0;
        while live.turn().is_some() {
            turns += 1;
            messages.extend(live.next().unwrap().unwrap());
        }
        assert!(live.over() && turns > 0);
        assert_eq!(live.next().unwrap(), None);
        let (text, commands) = crate::scenario::fought(&scenario, &packs, 1).unwrap();
        let fought = h3_view::record::parse(&text).unwrap();
        assert_eq!(commands.len(), turns);
        assert_eq!(messages, fought.messages);
        let whole = h3_view::record::parse(&live.record().unwrap()).unwrap();
        assert_eq!(whole.messages, fought.messages);
        assert_eq!(whole.header, opening.header);
    }
}
