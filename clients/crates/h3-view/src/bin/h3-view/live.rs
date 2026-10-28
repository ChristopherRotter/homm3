//! A battle fought as it is played (E17 M5 T1b): a scenario's battle hosted by `h3-protocol`'s
//! session in-process ([`crate::scenario::hosted`]), both seats held here and a spectator joined.
//! Each command is asked for when the battle's next one is needed: the Greedy bot's
//! ([`crate::bot`]) for a side no human plays here, a human's (E17 M5 T2b: the window's press)
//! for the others, sent from that side's seat as a client sends it (the session validates it as
//! a server does); what the spectator is sent for it makes the record grow. The bot decides on a
//! copy of the battle kept here by the same commands (as [`crate::scenario::fought`] does), which
//! also gives the mouse each turn's battle and legal commands, and every side's events with the
//! side that alone sees each (the screen of a battle a human plays shows the side to act's hidden
//! objects). An adapter.

use h3_battle::{BattleSetup, BattleSide, BattleState, LegalCommands, Viewer, Visibility};
use h3_content_battle::BattleRuleset;
use h3_protocol::session::Bot as _;
use h3_protocol::session::Session;
use h3_protocol::wire::{
    ClientMessage, Command, Event, Observation, Seat, ServerMessage, Side, SpellObstacle,
};
use h3_protocol::{codec, map};

use crate::bot::Greedy;
use crate::scenario::{FOUGHT_LIMIT, Turned, VIEWER, fetch_record, hosted, seat_conn};

/// The battle being fought: the session, the copy the bot decides on, the commands sent.
#[derive(Debug)]
pub struct Live<'r> {
    ruleset: &'r BattleRuleset,
    session: Session<'r>,
    state: BattleState,
    bot: Greedy,
    legal: LegalCommands,
    /// The sides a human plays here (the bot plays the others).
    humans: [bool; 2],
    sent: u32,
}

/// What a command played sent: the spectator's messages, in order (its `Events`, then
/// `GameEnded` when it ended the battle), and every side's events, each with the side that alone
/// sees it (none: both).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Played {
    pub messages: Vec<ServerMessage>,
    pub events: Vec<(Event, Option<Side>)>,
}

impl<'r> Live<'r> {
    /// The battle `setup` sets up with `seed` (its `ruleset` compiled from the scenario's packs),
    /// hosted and started, no command played; a human plays here the sides `humans` names, the
    /// bot the others.
    pub fn new(
        ruleset: &'r BattleRuleset,
        setup: BattleSetup,
        seed: u64,
        humans: [bool; 2],
    ) -> Result<Self, String> {
        let state =
            BattleState::new(ruleset, &setup, seed).map_err(|e| format!("the setup: {e:?}"))?;
        Ok(Self {
            ruleset,
            session: hosted(ruleset, setup, seed, true)?,
            state,
            bot: Greedy,
            legal: LegalCommands::new(),
            humans,
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

    /// Whether the side to act is a human's here: the battle waits for its command
    /// ([`Live::play`]).
    pub fn waiting(&self) -> bool {
        self.state
            .side_to_act()
            .is_some_and(|side| self.humans[side.index()])
    }

    /// Whether a human plays here any side.
    pub fn human(&self) -> bool {
        self.humans.contains(&true)
    }

    /// The battle at the turn to come, a spectator's view, and the legal commands of the side to
    /// act (E17 M3 T6c-2: what the mouse's cursor asks); none when the battle is over.
    pub fn turn(&self) -> Option<Turned> {
        let side = self.state.side_to_act()?;
        Some(Turned::of(&self.state, self.ruleset, side, &[]))
    }

    /// The engine's part of the side to act's hero's aim of `spell` now (E17 M5 T3b): none when
    /// the engine casts it without a target or at nothing.
    pub fn hero_aim(&self, spell: u16) -> Option<h3_view::mouse::aim::Aim> {
        let side = self.state.side_to_act()?;
        crate::aims::hero(&self.state, self.ruleset, side, spell)
    }

    /// Whether the side to act's hero may cast `spell` without a target now (E17 M5 T3a: the
    /// spell book's choice cast at once; the engine's `cast_targets`, BAT-MAG-001).
    pub fn casts_untargeted(&self, spell: u16) -> bool {
        let Some(side) = self.state.side_to_act() else {
            return false;
        };
        let mut targets = h3_battle::CastTargets::new();
        let id = h3_kernel::Handle::from_raw(spell);
        h3_battle::cast_targets(&self.state, self.ruleset, side, id, &mut targets);
        targets.iter().any(|t| *t == h3_battle::CastTarget::None)
    }

    /// The side to act's own observation (E17 M5 T3a: its hero's spell book, as its seat is sent
    /// it), none once the battle is over.
    pub fn own_view(&self) -> Option<Observation> {
        let side = self.state.side_to_act()?;
        Some(map::observe(&self.state, self.ruleset, Viewer::Side(side)))
    }

    /// The spell obstacles on the battlefield, each side's own hidden ones among them (E17 M5
    /// T2b): the attacker's view, then the defender's hidden ones.
    pub fn obstacles(&self) -> Vec<SpellObstacle> {
        let view = |side| map::observe(&self.state, self.ruleset, Viewer::Side(side));
        let mut all = view(BattleSide::Attacker).spell_obstacles;
        all.extend(
            view(BattleSide::Defender)
                .spell_obstacles
                .into_iter()
                .filter(|o| o.side == Side::Defender && !o.revealed),
        );
        all
    }

    /// Plays the next command if a bot's: the bot's for the side to act, sent from its seat;
    /// none when the battle is over or waits for a human ([`Live::waiting`]). A command the
    /// session rejects, or one too many ([`FOUGHT_LIMIT`]), is an error.
    pub fn next(&mut self) -> Result<Option<Played>, String> {
        let Some(side) = self.state.side_to_act().filter(|s| !self.humans[s.index()]) else {
            return Ok(None);
        };
        h3_battle::legal_commands(&self.state, self.ruleset, side, &mut self.legal);
        let choice = self
            .bot
            .command(&self.state, self.ruleset, side, &self.legal);
        self.send(side, map::command(choice)).map(Some)
    }

    /// The bot's choice for the side to act, as the wire's command (E17 M5 T2b: what the
    /// window's unattended run presses for a human); none when the battle is over.
    pub fn choice(&mut self) -> Option<Command> {
        let side = self.state.side_to_act()?;
        h3_battle::legal_commands(&self.state, self.ruleset, side, &mut self.legal);
        let choice = self
            .bot
            .command(&self.state, self.ruleset, side, &self.legal);
        Some(map::command(choice))
    }

    /// Plays a human's `command` for the side to act (E17 M5 T2b), sent from its seat: the
    /// session validates it as a server; one it rejects is an error and changes nothing.
    pub fn play(&mut self, command: Command) -> Result<Played, String> {
        let side = self.state.side_to_act().ok_or("the battle is over")?;
        self.send(side, command)
    }

    /// Sends `command` from `side`'s seat; the copy here plays it after the session took it.
    fn send(&mut self, side: BattleSide, command: Command) -> Result<Played, String> {
        if usize::try_from(self.sent).unwrap_or(usize::MAX) >= FOUGHT_LIMIT {
            return Err(format!("no end after {FOUGHT_LIMIT} commands"));
        }
        let seq = self.sent + 1;
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
        self.sent = seq;
        let choice = map::command_to_engine(command);
        let mut emitted: Vec<h3_battle::Event> = Vec::new();
        h3_battle::apply(&mut self.state, self.ruleset, side, choice, &mut emitted)
            .map_err(|e| format!("command {seq} ({choice:?}): {e:?}"))?;
        let mut events = Vec::new();
        for e in &emitted {
            let (viewer, seen) = match e.visibility {
                Visibility::Public => (Viewer::Spectator, None),
                Visibility::Side(side) => (Viewer::Side(side), Some(map::side(side))),
            };
            let mapped = map::events(core::slice::from_ref(e), viewer);
            events.extend(mapped.into_iter().map(|event| (event, seen)));
        }
        Ok(Played { messages, events })
    }
}

/// The record of the battle `setup` sets up with `seed` when the bot plays every side, fought as
/// it is played (E17 M5 T2b: what a battle pressed by the bot's choices must equal).
pub fn bot_record(
    ruleset: &BattleRuleset,
    setup: BattleSetup,
    seed: u64,
) -> Result<String, String> {
    let mut live = Live::new(ruleset, setup, seed, [false; 2])?;
    while live.next()?.is_some() {}
    live.record()
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
        let mut live = Live::new(&ruleset, setup, 1, [false; 2]).unwrap();
        let opening = h3_view::record::parse(&live.record().unwrap()).unwrap();
        let mut messages = opening.messages.clone();
        let mut turns = 0;
        assert!(!live.human() && !live.waiting());
        while live.turn().is_some() {
            turns += 1;
            let played = live.next().unwrap().unwrap();
            // Every side's events: the public ones are the spectator's.
            let public: Vec<Event> = played
                .events
                .iter()
                .filter(|(_, seen)| seen.is_none())
                .map(|(e, _)| e.clone())
                .collect();
            assert!(matches!(
                played.messages.first(),
                Some(ServerMessage::Events { events, .. }) if *events == public
            ));
            messages.extend(played.messages);
        }
        assert!(live.over() && turns > 0);
        assert_eq!(live.next().unwrap(), None);
        assert_eq!(live.choice(), None);
        assert!(live.play(Command::Wait {}).is_err());
        let (text, commands) = crate::scenario::fought(&scenario, &packs, 1).unwrap();
        let fought = h3_view::record::parse(&text).unwrap();
        assert_eq!(commands.len(), turns);
        assert_eq!(messages, fought.messages);
        let whole = h3_view::record::parse(&live.record().unwrap()).unwrap();
        assert_eq!(whole.messages, fought.messages);
        assert_eq!(whole.header, opening.header);
    }

    #[test]
    fn a_human_s_commands_wait_for_their_seat_and_are_validated() {
        // E17 M5 T2b: the attacker a human's, the defender the bot's. The battle waits on the
        // attacker's turns (the bot plays none for it); the human's command, the bot's choice
        // given, plays as the bot's would: the battle is the bot's at once with the same setup.
        // A command the session rejects is an error and changes nothing.
        let (scenario, packs) = (
            repo().join("packs/synthetic/scenarios/duel.toml"),
            repo().join("packs"),
        );
        let (ruleset, mut setup) = crate::scenario::battle(&scenario, &packs, 1).unwrap();
        setup.human = [true, false];
        let mut live = Live::new(&ruleset, setup.clone(), 1, [true, false]).unwrap();
        assert!(live.human());
        let mut waits = 0;
        while !live.over() {
            if live.waiting() {
                waits += 1;
                assert_eq!(live.next().unwrap(), None);
                let before = live.record().unwrap();
                assert!(live.play(Command::Move { to: 255 }).is_err());
                assert_eq!(live.record().unwrap(), before);
                let choice = live.choice().unwrap();
                live.play(choice).unwrap();
            } else {
                live.next().unwrap().unwrap();
            }
        }
        assert!(waits > 0);
        let bot = bot_record(&ruleset, setup, 1).unwrap();
        assert_eq!(live.record().unwrap(), bot);
        assert!(live.obstacles().is_empty());
    }
}
