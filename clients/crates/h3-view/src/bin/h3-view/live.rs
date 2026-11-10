//! A battle fought as it is played (E17 M5 T1b): a scenario's battle hosted by `h3-protocol`'s
//! session in-process ([`crate::scenario::hosted`]), both seats held here and a spectator joined.
//! Each command is asked for when the battle's next one is needed: the computer's ([`Computer`])
//! for a side no human plays here, a human's (E17 M5 T2b: the window's press) for the others,
//! sent from that side's seat as a client sends it (the session validates it as a server does);
//! what the spectator is sent for it makes the record grow. The computer decides on a copy of the
//! battle kept here by the same commands (as [`crate::scenario::fought`] does), which also gives
//! the mouse each turn's battle and legal commands, and every side's events with the side that
//! alone sees each (the screen of a battle a human plays shows the side to act's hidden objects).
//! E17 M6 T5b1 (design §12.18): the window's computer is the engine's original one
//! (`h3_battle_ai`, its whole choice for its side), which under Auto Combat also plays a human's
//! side by its parts: the hero's cast as the battle asks it ([`Live::set_cast`]), the action of a
//! stack that plays itself; each side draws from a stream of its own, the seat's. E17 M6 T5b2a
//! (design §12.19): the parts as the original asks them of a human's side ([`Decision::human`]),
//! the retreat test before each such action and, on its yes, the human asked once a battle
//! ([`Question`], [`Live::answer`]). An adapter.

use h3_battle::{
    BattleSetup, BattleSide, BattleState, EventKind, LegalCommands, Viewer, Visibility,
};
use h3_battle_ai::{Decision, Leaving, Sight};
use h3_content_battle::BattleRuleset;
use h3_kernel::{Rng, StreamId};
use h3_protocol::session::Bot as _;
use h3_protocol::session::Session;
use h3_protocol::wire::{
    ClientMessage, Command, Event, Observation, RejectCode, Seat, ServerMessage, Side,
    SpellObstacle,
};
use h3_protocol::{codec, map};

use crate::bot::Greedy;
use crate::scenario::{FOUGHT_LIMIT, Turned, VIEWER, fetch_record, hosted, seat_conn};

/// Who plays the computer's sides here (E17 M6 T5b1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Computer {
    /// The client's fixed bot ([`Greedy`], *provisional*: M2's review list), a fixture's: `h3-view
    /// play`'s battles, fought by it on both sides. It casts nothing, leaves never and plays a
    /// human's side under Auto Combat whole.
    Greedy,
    /// The engine's original computer (ADR-0027) on the battle [`Sight`] names: the window's. Its
    /// whole choice for its side (`h3_battle_ai::choose_fair`, `choose`: it may cast and
    /// retreat); a human's side under Auto Combat by its parts ([`Decision`]).
    Original(Sight),
}

/// What asked for each command played here, and for each cast the computer was asked for a
/// human's hero, in order (E17 M6 T5b1): what [`refight`] asks again.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ask {
    /// A human's command ([`Live::play`]; an unattended run's: the presser's choice,
    /// [`Live::choice`]).
    Pressed,
    /// The computer's whole choice for its side.
    Computer,
    /// The computer asked for a human's hero's cast (Auto Combat with Combat Auto Spells): a
    /// command, or none.
    Cast,
    /// Auto Combat's action for a human's stack that plays itself (E17 M6 T5b2a: the retreat test
    /// before it, which may ask the human a [`Question`] instead).
    Action,
    /// The human's answer to Auto Combat's question or offer (E17 M6 T5b2a, [`Live::answer`]):
    /// its OK (true) or its Cancel.
    Answered(bool),
}

/// What Auto Combat asks the human whose stack plays itself when the computer's retreat test says
/// yes (E17 M6 T5b2a, design §12.19; `ai_battle_action` 0x477F56 to 0x47828E), a box of OK and
/// Cancel over the battle standing still.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Question {
    /// Whether to retreat (GENRLTXT's record 414 with the side's hero's name), once a battle
    /// (+0x1402F): OK the computer's leaving ([`Decision::leaving`]: the Retreat, or the offer),
    /// Cancel the stack's action and no question again this battle.
    Retreat,
    /// Whether to save the army for `price` (GENRLTXT's record 130 with the hero's name and the
    /// price, the price's gold as its picture; BAT-AI-026): OK the Surrender, Cancel the Retreat.
    Offer {
        /// The battle's surrender price (the side's private part, its own to see).
        price: u32,
    },
}

/// The computer's cast for a human's hero at the turn under way (E17 M6 T5b1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum Cast {
    /// Not asked (Auto Combat off as the turn started, or Combat Auto Spells clear).
    #[default]
    No,
    /// To ask before anything else in the turn.
    Due,
    /// Asked (a cast or none): the turn goes on.
    Asked,
}

/// The battle being fought: the session, the copy the computer decides on, the commands sent.
#[derive(Debug)]
pub struct Live<'r> {
    ruleset: &'r BattleRuleset,
    session: Session<'r>,
    state: BattleState,
    /// The computer's sides' player (E17 M6 T5b1).
    computer: Computer,
    /// Each side's stream (E17 M6 T5b1): the seat's, from the battle's seed, as the engine's
    /// batches seed theirs (`StreamId::seat`); the computer's choices for the side draw from it.
    seats: [Rng; 2],
    /// The presser's bot (E17 M5 T2b): what an unattended run presses for a human
    /// ([`Live::choice`], *provisional*: a fixed player that draws from no stream).
    bot: Greedy,
    legal: LegalCommands,
    /// The sides a human plays here (the computer plays the others).
    humans: [bool; 2],
    /// Auto Combat plays the turn to come (E17 M5 T4c1): the computer plays the humans' sides too.
    /// E17 M6 T4b1 (*provisional*): the battle says so by the stack's box (`Battle::auto_turn`),
    /// so a human's turn under Auto Combat whose box is off waits here as with it off.
    auto: bool,
    /// The computer's cast for the human's hero at the turn under way (E17 M6 T5b1).
    cast: Cast,
    /// What asked for each command, in order (E17 M6 T5b1).
    asks: Vec<Ask>,
    /// Auto Combat's question may be asked (E17 M6 T5b2a: the original's +0x1402F, set as the
    /// battle starts, cleared by the question's Cancel).
    may_ask: bool,
    /// Auto Combat's question or offer waiting for the human's answer (E17 M6 T5b2a), with the
    /// side's stream as it was before the decision it came from: the decision made again from it
    /// sees the same world (`Decision::new` draws the fair world's seed from a copy), the parts
    /// going on drawing from the side's stream.
    question: Option<(Question, Rng)>,
    sent: u32,
    /// The computer's commands since a human's (E17 M5 T8a, R3-4): [`FOUGHT_LIMIT`] stops the
    /// computer's endless fights, not a long battle a human plays.
    bot_run: usize,
}

/// What a command played sent: the spectator's messages, in order (its `Events`, then
/// `GameEnded` when it ended the battle), and every side's events, each with the side that alone
/// sees it (none: both); and whether it started a turn (E17 M6 T5b1: a hero's cast does not, its
/// stack's turn goes on).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Played {
    pub messages: Vec<ServerMessage>,
    pub events: Vec<(Event, Option<Side>)>,
    pub turned: bool,
}

impl<'r> Live<'r> {
    /// The battle `setup` sets up with `seed` (its `ruleset` compiled from the scenario's packs),
    /// hosted and started, no command played; a human plays here the sides `humans` names,
    /// `computer` the others.
    pub fn new(
        ruleset: &'r BattleRuleset,
        setup: BattleSetup,
        seed: u64,
        (humans, computer): ([bool; 2], Computer),
    ) -> Result<Self, String> {
        let state =
            BattleState::new(ruleset, &setup, seed).map_err(|e| format!("the setup: {e:?}"))?;
        Ok(Self {
            ruleset,
            session: hosted(ruleset, setup, seed, true)?,
            state,
            computer,
            seats: [0, 1].map(|s| Rng::for_stream(seed, StreamId::seat(s))),
            bot: Greedy,
            legal: LegalCommands::new(),
            humans,
            auto: false,
            cast: Cast::No,
            asks: Vec::new(),
            may_ask: true,
            question: None,
            sent: 0,
            bot_run: 0,
        })
    }

    /// The spectator's record so far: its header and every message it was sent.
    pub fn record(&mut self) -> Result<String, String> {
        fetch_record(&mut self.session, VIEWER)
    }

    /// The engine's outcome once the battle is over (E17 M5 T4b1: the result window), and the
    /// ruleset it was fought by.
    pub fn outcome(&self) -> Option<(h3_contracts::BattleOutcome, &'r BattleRuleset)> {
        Some((self.state.outcome()?, self.ruleset))
    }

    /// Whether the battle is over (no side is to act).
    pub fn over(&self) -> bool {
        self.state.side_to_act().is_none()
    }

    /// Whether the side to act is a human's here and Auto Combat does not play it: the battle
    /// waits for its command ([`Live::play`]); E17 M6 T5b1: once the computer was asked for its
    /// hero's cast, when the battle asks it at the turn ([`Live::set_cast`]).
    pub fn waiting(&self) -> bool {
        !self.auto
            && self.cast != Cast::Due
            && self
                .state
                .side_to_act()
                .is_some_and(|side| self.humans[side.index()])
    }

    /// Whether Auto Combat plays the turn to come (E17 M5 T4c1): while it does, the computer plays
    /// the humans' sides too ([`Live::next`]). E17 M6 T4b1: the battle asks the stack's box first
    /// (`h3_view::auto::plays_itself`), so Auto Combat on over a stack it does not play sets it
    /// off here.
    pub fn set_auto(&mut self, on: bool) {
        self.auto = on;
    }

    /// Whether the computer is asked for a human's hero's cast as the turn under way started (E17
    /// M6 T5b1, design §12.18: Auto Combat on then and Combat Auto Spells set,
    /// `h3_view::auto::Auto::asks_cast`, the battle's to say): before anything else in the turn,
    /// the human's or the computer's. Once a turn: taken until the cast is asked, a turn started
    /// clearing it ([`Live::next`] asks it).
    pub fn set_cast(&mut self, ask: bool) {
        if self.cast != Cast::Asked {
            self.cast = if ask { Cast::Due } else { Cast::No };
        }
    }

    /// Auto Combat switched on over the turn waiting with Combat Auto Spells set (E17 M6 T5b1,
    /// design §12.18: the switch runs the turn's start again, 0x474E73): the computer is asked
    /// for the hero's cast once more.
    pub fn cast_again(&mut self) {
        self.cast = Cast::Due;
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

    /// The battle at the turn to come, a spectator's view: [`Self::turn`]'s, without its legal
    /// commands, aims and damage previews (E17 M6 T7b, R5-4: what Auto Combat asks of the turn to
    /// come, its acting stack); none when the battle is over.
    pub fn observation(&self) -> Option<Observation> {
        self.state.side_to_act()?;
        Some(map::observe(&self.state, self.ruleset, Viewer::Spectator))
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

    /// Each side's hero's private part now (E17 M6 T5a: the Orb's and the Cloak's refusals know
    /// both heroes here, design §12.17).
    pub fn magic(&self) -> [Option<h3_protocol::wire::OwnHero>; 2] {
        crate::aims::magic(&self.state, self.ruleset)
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

    /// What asked for each command played so far, and for each cast asked (E17 M6 T5b1).
    pub fn asks(&self) -> &[Ask] {
        &self.asks
    }

    /// The sides a human plays here and the computer's player (E17 M6 T5b1: what [`refight`]
    /// fights with).
    pub const fn players(&self) -> ([bool; 2], Computer) {
        (self.humans, self.computer)
    }

    /// Plays the next command if the computer's, sent from its seat: for a side no human plays
    /// here, its whole choice; for a human's (E17 M6 T5b1, design §12.18), first the hero's cast
    /// when the battle asks it at the turn ([`Live::set_cast`]: the hero's cast, else nothing),
    /// then under Auto Combat the stack's action ([`Live::set_auto`]; E17 M6 T5b2a: the retreat
    /// test first, whose yes may ask the human instead, [`Live::question`]). None when the battle
    /// is over or waits for a human ([`Live::waiting`]; also once a cast asked gave none and the
    /// turn is the human's, and while a question waits for its answer). One too many of the
    /// computer's in a row ([`FOUGHT_LIMIT`] since a human's) is an error that changes nothing;
    /// for the others, [`Live::play`]'s.
    pub fn next(&mut self) -> Result<Option<Played>, String> {
        let Some(side) =
            (self.state.side_to_act()).filter(|_| !self.waiting() && self.question.is_none())
        else {
            return Ok(None);
        };
        if self.bot_run >= FOUGHT_LIMIT {
            return Err(format!("no end after {FOUGHT_LIMIT} commands of the bot's"));
        }
        let (choice, ask) = if !self.humans[side.index()] {
            (self.whole(side), Ask::Computer)
        } else if self.cast == Cast::Due {
            self.cast = Cast::Asked;
            self.asks.push(Ask::Cast);
            match self.hero_cast(side) {
                Some(cast) => (cast, Ask::Cast),
                None if self.auto => match self.action(side) {
                    Some(action) => (action, Ask::Action),
                    None => return Ok(self.asked()),
                },
                None => return Ok(None),
            }
        } else {
            match self.action(side) {
                Some(action) => (action, Ask::Action),
                None => return Ok(self.asked()),
            }
        };
        let command = map::command(choice);
        let played = self
            .send(side, command)?
            .map_err(|code| format!("the bot's command {command:?}: rejected, {code:?}"))?;
        if ask != Ask::Cast {
            self.asks.push(ask);
        }
        self.bot_run += 1;
        Ok(Some(played))
    }

    /// The computer's whole choice for `side`, the side to act no human plays here (E17 M6
    /// T5b1): the engine's original computer's by the sight (from the side's stream), or the
    /// fixture's bot.
    fn whole(&mut self, side: BattleSide) -> h3_battle::Command {
        let seat = &mut self.seats[side.index()];
        match self.computer {
            Computer::Original(Sight::Fair) => {
                h3_battle_ai::choose_fair(&self.state, self.ruleset, side, seat)
            }
            Computer::Original(Sight::AllSeeing) => {
                h3_battle_ai::choose(&self.state, self.ruleset, side, seat)
            }
            Computer::Greedy => self.greedy(side),
        }
    }

    /// The computer's cast for `side`'s hero, a human's, at the turn under way (E17 M6 T5b1,
    /// `ai_hero_cast_turn` 0x422DA0: [`Decision::hero_cast`], from the side's stream, asked as of
    /// a human's side, E17 M6 T5b2a); none: no cast (the hero cast this round, cannot or decides
    /// not to). The fixture's bot casts nothing.
    fn hero_cast(&mut self, side: BattleSide) -> Option<h3_battle::Command> {
        let Computer::Original(sight) = self.computer else {
            return None;
        };
        let seat = &mut self.seats[side.index()];
        Decision::new(&self.state, self.ruleset, side, sight, seat)?
            .human()
            .hero_cast(seat)
    }

    /// Auto Combat's action for `side`'s stack, a human's that plays itself (E17 M6 T5b1, T5b2a;
    /// `ai_battle_action` 0x477EE0): the retreat test ([`Decision::retreat_test`] as of a human's
    /// side, its draws at every such action), then on its yes, while the question may be asked,
    /// none: the question waits for the human ([`Live::question`]); else the stack's action
    /// ([`Decision::action`]), both from the side's stream; Defend for a world the battle refuses,
    /// as the fair actor's. The fixture's bot asks nothing.
    fn action(&mut self, side: BattleSide) -> Option<h3_battle::Command> {
        let Computer::Original(sight) = self.computer else {
            return Some(self.greedy(side));
        };
        let before = self.seats[side.index()].clone();
        let seat = &mut self.seats[side.index()];
        let Some(decision) = Decision::new(&self.state, self.ruleset, side, sight, seat) else {
            return Some(h3_battle::Command::Defend);
        };
        let decision = decision.human();
        if decision.retreat_test(seat) && self.may_ask {
            self.question = Some((Question::Retreat, before));
            return None;
        }
        Some(decision.action(seat))
    }

    /// The question asked for the action (E17 M6 T5b2a): logged as its action's ask, its answer
    /// to come ([`Live::answer`]); no command.
    fn asked(&mut self) -> Option<Played> {
        self.asks.push(Ask::Action);
        None
    }

    /// Auto Combat's question or offer waiting for the human's answer (E17 M6 T5b2a, design
    /// §12.19): the battle stands still behind it, taking no command ([`Live::next`] and
    /// [`Live::play`] give none) until [`Live::answer`].
    pub fn question(&self) -> Option<Question> {
        self.question.as_ref().map(|&(question, _)| question)
    }

    /// The human's answer to the question or offer waiting (E17 M6 T5b2a, design §12.19;
    /// 0x477F56 to 0x47828E), `ok` its OK, sent from the side's seat as the computer's: the
    /// question's Cancel clears the flag (no question again this battle) and plays the stack's
    /// action ([`Decision::action`], on the world of the decision the test was asked in), its OK
    /// the computer's leaving ([`Decision::leaving`]): the Retreat at once, or the offer, which
    /// waits in turn (none played); the offer's OK the Surrender, its Cancel the Retreat. None
    /// when no question waits; an error as [`Live::next`]'s.
    pub fn answer(&mut self, ok: bool) -> Result<Option<Played>, String> {
        let (Some((question, before)), Some(side)) =
            (self.question.clone(), self.state.side_to_act())
        else {
            return Ok(None);
        };
        let Computer::Original(sight) = self.computer else {
            return Ok(None);
        };
        self.question = None;
        self.asks.push(Ask::Answered(ok));
        let choice = match (question, ok) {
            (Question::Retreat, false) => {
                self.may_ask = false;
                let seat = &mut self.seats[side.index()];
                Decision::new(&self.state, self.ruleset, side, sight, &before)
                    .map_or(h3_battle::Command::Defend, |d| d.human().action(seat))
            }
            (Question::Retreat, true) => {
                let leaving = Decision::new(&self.state, self.ruleset, side, sight, &before)
                    .map_or(Leaving::Retreat, |d| d.human().leaving());
                match leaving {
                    Leaving::Retreat => h3_battle::Command::Retreat,
                    Leaving::Offer { price } => {
                        self.question = Some((Question::Offer { price }, before));
                        return Ok(None);
                    }
                }
            }
            (Question::Offer { .. }, true) => h3_battle::Command::Surrender,
            (Question::Offer { .. }, false) => h3_battle::Command::Retreat,
        };
        let command = map::command(choice);
        let played = self
            .send(side, command)?
            .map_err(|code| format!("the computer's command {command:?}: rejected, {code:?}"))?;
        self.bot_run = 0;
        Ok(Some(played))
    }

    /// The fixture's bot's choice for `side`, the side to act.
    fn greedy(&mut self, side: BattleSide) -> h3_battle::Command {
        h3_battle::legal_commands(&self.state, self.ruleset, side, &mut self.legal);
        self.bot
            .command(&self.state, self.ruleset, side, &self.legal)
    }

    /// The presser's choice for the side to act, as the wire's command (E17 M5 T2b: what the
    /// window's unattended run presses for a human; the fixed bot's, *provisional*: E17 M6 T5b1
    /// keeps it, a player that draws from no stream and leaves never); none when the battle is
    /// over.
    pub fn choice(&mut self) -> Option<Command> {
        let side = self.state.side_to_act()?;
        Some(map::command(self.greedy(side)))
    }

    /// Plays a human's `command` for the side to act (E17 M5 T2b), sent from its seat, however
    /// many came before (E17 M5 T8a, R3-4: a long hot-seat battle): the session validates it as
    /// a server; one it rejects changes nothing and is none (E17 M5 T8b2, R4b-4: the turn waits
    /// on, as a server's rejection leaves it). An error comes after the session took the
    /// command: the session and the copy here no longer agree, the battle cannot go on (the
    /// window quits on it); or the battle is over.
    pub fn play(&mut self, command: Command) -> Result<Option<Played>, String> {
        let side = self.state.side_to_act().ok_or("the battle is over")?;
        // Auto Combat's question waits for its answer, not a command (E17 M6 T5b2a).
        if self.question.is_some() {
            return Ok(None);
        }
        let Ok(played) = self.send(side, command)? else {
            return Ok(None);
        };
        self.asks.push(Ask::Pressed);
        self.bot_run = 0;
        Ok(Some(played))
    }

    /// Sends `command` from `side`'s seat; the copy here plays it after the session took it. A
    /// rejection changes nothing (its code); a frame not read or the copy's failure to play it
    /// come after the session took it (fatal to the battle, [`Live::play`]). A turn it starts
    /// asks no cast until the battle says so ([`Live::set_cast`]).
    fn send(
        &mut self,
        side: BattleSide,
        command: Command,
    ) -> Result<Result<Played, RejectCode>, String> {
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
                return Ok(Err(code));
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
        let turned = (emitted.iter()).any(|e| matches!(e.kind, EventKind::TurnStarted { .. }));
        if turned {
            self.cast = Cast::No;
        }
        let mut events = Vec::new();
        for e in &emitted {
            let (viewer, seen) = match e.visibility {
                Visibility::Public => (Viewer::Spectator, None),
                Visibility::Side(side) => (Viewer::Side(side), Some(map::side(side))),
            };
            let mapped = map::events(core::slice::from_ref(e), viewer);
            events.extend(mapped.into_iter().map(|event| (event, seen)));
        }
        Ok(Ok(Played {
            messages,
            events,
            turned,
        }))
    }

    /// Asks `ask` again (E17 M6 T5b1, [`refight`]): a human's command the presser's choice, the
    /// computer's as [`Live::next`] asks them. A command not given (rejected, or none) is
    /// nothing here: the records differ.
    fn again(&mut self, ask: Ask) -> Result<(), String> {
        match ask {
            Ask::Pressed => {
                if let Some(command) = self.choice() {
                    self.play(command)?;
                }
                return Ok(());
            }
            Ask::Computer => self.auto = false,
            Ask::Cast => (self.auto, self.cast) = (false, Cast::Due),
            Ask::Action => {
                self.auto = true;
                if self.cast == Cast::Due {
                    self.cast = Cast::Asked;
                }
            }
            Ask::Answered(ok) => {
                self.answer(ok)?;
                return Ok(());
            }
        }
        self.next()?;
        Ok(())
    }
}

/// The record of the battle `setup` sets up with `seed` fought again here by `asks`, with the
/// sides `humans` names a human's and `computer` the others' (E17 M6 T5b1; E17 M5 T2b's: what a
/// battle pressed by the presser's choices must equal): each human's command the presser's
/// choice ([`Live::choice`]), each of the computer's asked again of it in the same order (its
/// whole choice, a cast asked for a human's hero, Auto Combat's action and its retreat test,
/// E17 M6 T5b2a: a question's answers as given), each side from its stream afresh. A battle the
/// bot fights on both sides: `asks` all [`Ask::Computer`].
pub fn refight(
    ruleset: &BattleRuleset,
    setup: BattleSetup,
    seed: u64,
    players: ([bool; 2], Computer),
    asks: &[Ask],
) -> Result<String, String> {
    let mut live = Live::new(ruleset, setup, seed, players)?;
    for &ask in asks {
        live.again(ask)?;
    }
    live.record()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};

    fn repo() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
    }

    /// The duel's battle (a synthetic one, no hero) with `seed`.
    fn duel(seed: u64) -> (BattleRuleset, BattleSetup) {
        let (scenario, packs) = (
            repo().join("packs/synthetic/scenarios/duel.toml"),
            repo().join("packs"),
        );
        crate::scenario::battle(&scenario, &packs, seed).unwrap()
    }

    /// The battle prepared for the recordings of the computer's casts for the human's hero (E17
    /// M6 T5b1, `seq-auto-cast.toml`): Orrin's book holding Bless, the Champions' turn first.
    fn auto_cast() -> (BattleRuleset, BattleSetup) {
        let scenario = repo().join("clients/situations/seq-auto-cast.toml");
        let (ruleset, mut setup) =
            crate::scenario::battle(&scenario, &repo().join("packs"), 1).unwrap();
        setup.human = [true, false];
        (ruleset, setup)
    }

    #[test]
    fn a_battle_fought_as_it_is_played_records_what_the_bot_fights_at_once() {
        // The duel fought command by command: the spectator's messages, each command's as it is
        // sent, are the record of the battle the bot fights at once (`scenario::fought`).
        let (scenario, packs) = (
            repo().join("packs/synthetic/scenarios/duel.toml"),
            repo().join("packs"),
        );
        let (ruleset, setup) = duel(1);
        let mut live = Live::new(&ruleset, setup, 1, ([false; 2], Computer::Greedy)).unwrap();
        let opening = h3_view::record::parse(&live.record().unwrap()).unwrap();
        let mut messages = opening.messages.clone();
        let mut turns = 0;
        assert!(!live.human() && !live.waiting());
        while live.turn().is_some() {
            // The turn to come's battle alone is its turn's (E17 M6 T7b, R5-4).
            assert_eq!(live.observation(), live.turn().map(|t| t.observation));
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
        assert_eq!(live.observation(), None);
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
        // Each command the computer's whole choice, asked again so.
        assert_eq!(live.asks(), vec![Ask::Computer; turns]);
    }

    #[test]
    fn the_bot_s_limit_leaves_a_human_s_commands_alone() {
        // E17 M5 T8a (R3-4): past `FOUGHT_LIMIT` commands a hot-seat battle's are taken; the
        // bot stops after as many in a row (Auto Combat on), a human's command starting the run
        // afresh.
        let (ruleset, mut setup) = duel(1);
        setup.human = [true, true];
        let mut live = Live::new(&ruleset, setup, 1, ([true, true], Computer::Greedy)).unwrap();
        live.sent = u32::try_from(FOUGHT_LIMIT).unwrap();
        for _ in 0..2 {
            let choice = live.choice().unwrap();
            live.play(choice).unwrap();
        }
        assert_eq!(live.sent, u32::try_from(FOUGHT_LIMIT + 2).unwrap());
        live.set_auto(true);
        live.bot_run = FOUGHT_LIMIT;
        let before = live.record().unwrap();
        assert!(live.next().unwrap_err().starts_with("no end after"));
        assert_eq!(live.record().unwrap(), before);
        live.set_auto(false);
        let choice = live.choice().unwrap();
        live.play(choice).unwrap();
        assert_eq!(live.bot_run, 0);
        live.set_auto(true);
        live.next().unwrap().unwrap();
        assert_eq!(live.bot_run, 1);
    }

    #[test]
    fn a_human_s_commands_wait_for_their_seat_and_are_validated() {
        // E17 M5 T2b: the attacker a human's, the defender the computer's. The battle waits on
        // the attacker's turns (the computer plays none for it); the human's command, the
        // presser's choice given, plays as the computer's would: fought again by what asked for
        // each command, the battle is the same (E17 M6 T5b1), and with the fixed bot on both
        // sides the bot's at once. A command the session rejects changes nothing.
        let (ruleset, mut setup) = duel(1);
        setup.human = [true, false];
        let players = ([true, false], Computer::Greedy);
        let mut live = Live::new(&ruleset, setup.clone(), 1, players).unwrap();
        assert!(live.human());
        let mut waits = 0;
        while !live.over() {
            if live.waiting() {
                waits += 1;
                assert_eq!(live.next().unwrap(), None);
                let before = live.record().unwrap();
                assert_eq!(live.play(Command::Move { to: 255 }), Ok(None));
                assert_eq!(live.record().unwrap(), before);
                let choice = live.choice().unwrap();
                live.play(choice).unwrap();
            } else {
                live.next().unwrap().unwrap();
            }
        }
        assert!(waits > 0 && live.asks().contains(&Ask::Pressed));
        let asks = live.asks().to_vec();
        let again = refight(&ruleset, setup.clone(), 1, players, &asks).unwrap();
        assert_eq!(live.record().unwrap(), again);
        let computer = vec![Ask::Computer; asks.len()];
        let bot = refight(
            &ruleset,
            setup.clone(),
            1,
            ([false; 2], Computer::Greedy),
            &computer,
        );
        assert_eq!(live.record().unwrap(), bot.unwrap());
        assert!(live.obstacles().is_empty());
        // E17 M5 T4c1: Auto Combat on, the bot plays the human's side too; off, it waits again.
        let mut auto = Live::new(&ruleset, setup, 1, players).unwrap();
        assert!(auto.waiting() && auto.next().unwrap().is_none());
        auto.set_auto(true);
        assert!(!auto.waiting());
        auto.next().unwrap().unwrap();
        auto.set_auto(false);
        while !auto.waiting() && !auto.over() {
            auto.next().unwrap().unwrap();
        }
        assert!(auto.over() || auto.next().unwrap().is_none());
        auto.set_auto(true);
        while auto.next().unwrap().is_some() {}
        assert!(auto.over());
        assert_eq!(auto.record().unwrap(), again);
    }

    #[test]
    fn the_original_computer_plays_its_side_from_the_seat_s_stream() {
        // E17 M6 T5b1: the window's computer is the engine's original one, its whole choice by
        // the sight (the fair actor's, or the all-seeing original's), each side drawing from the
        // seat's stream of the battle's seed; fought again by the asks, the same battle.
        for (sight, seed) in [(Sight::Fair, 3), (Sight::AllSeeing, 4)] {
            let (ruleset, setup) = duel(seed);
            let players = ([false; 2], Computer::Original(sight));
            let mut live = Live::new(&ruleset, setup.clone(), seed, players).unwrap();
            while live.next().unwrap().is_some() {}
            assert!(live.over());
            let mut state = BattleState::new(&ruleset, &setup, seed).unwrap();
            let mut seats = [0, 1].map(|s| Rng::for_stream(seed, StreamId::seat(s)));
            let mut commands = 0;
            while let Some(side) = state.side_to_act() {
                let seat = &mut seats[side.index()];
                let command = match sight {
                    Sight::Fair => h3_battle_ai::choose_fair(&state, &ruleset, side, seat),
                    Sight::AllSeeing => h3_battle_ai::choose(&state, &ruleset, side, seat),
                };
                h3_battle::apply(
                    &mut state,
                    &ruleset,
                    side,
                    command,
                    &mut h3_battle::NoopSink,
                )
                .unwrap();
                commands += 1;
            }
            assert_eq!(live.state, state);
            assert_eq!(live.asks(), vec![Ask::Computer; commands]);
            let asks = live.asks().to_vec();
            let again = refight(&ruleset, setup, seed, players, &asks).unwrap();
            assert_eq!(live.record().unwrap(), again);
        }
    }

    /// The computer's decision for `side` in `live`'s battle now, on copies (the battle and the
    /// side's stream), its cast and then its action as [`Live::next`] asks them.
    fn decided(live: &Live<'_>, side: BattleSide) -> (Option<h3_battle::Command>, Rng) {
        let mut seat = live.seats[side.index()].clone();
        let decision = Decision::new(&live.state, live.ruleset, side, Sight::Fair, &seat).unwrap();
        (decision.hero_cast(&mut seat), seat)
    }

    #[test]
    fn auto_combat_asks_the_computer_for_the_human_s_hero_s_cast_then_plays_on() {
        // E17 M6 T5b1 (design §12.18), the shape of `auto-cast-human` (Creatures cleared): asked
        // at the Champions' turn (the switch's), the computer casts nothing and the turn waits for
        // the human; asked as the Marksmen's turn starts, it casts Orrin's Bless from the
        // attacker's stream and the turn goes on, the human's: it waits. Asked once a turn: set
        // again, nothing; the switch asks again (none: cast this round). Auto Combat then plays
        // the stack: the action, from the stream where the cast left it.
        let (ruleset, setup) = auto_cast();
        let players = ([true, false], Computer::Original(Sight::Fair));
        let mut live = Live::new(&ruleset, setup.clone(), 1, players).unwrap();
        let side = BattleSide::Attacker;
        assert!(live.waiting());
        live.set_cast(true);
        assert!(!live.waiting());
        assert_eq!(decided(&live, side).0, None);
        assert_eq!(live.next().unwrap(), None);
        assert!(live.waiting());
        let pressed = live.choice().unwrap();
        assert!(live.play(pressed).unwrap().unwrap().turned);
        assert_eq!(live.state.side_to_act(), Some(side));
        assert!(live.waiting());
        live.set_cast(true);
        let (cast, after) = decided(&live, side);
        let cast = cast.expect("the computer casts Bless as the Marksmen's turn starts");
        assert!(matches!(cast, h3_battle::Command::Cast { .. }), "{cast:?}");
        let played = live.next().unwrap().unwrap();
        assert!(!played.turned && live.waiting());
        assert_eq!(live.seats[0], after);
        assert!(live.state.hero(side).is_some_and(|h| h.casts == 1));
        live.set_cast(true);
        assert!(live.waiting() && live.next().unwrap().is_none());
        live.cast_again();
        assert_eq!(live.next().unwrap(), None);
        assert!(live.waiting());
        live.set_auto(true);
        let mut seat = live.seats[0].clone();
        let action = Decision::new(&live.state, &ruleset, side, Sight::Fair, &seat)
            .unwrap()
            .action(&mut seat);
        let played = live.next().unwrap().unwrap();
        assert!(played.turned && live.seats[0] == seat);
        assert!(!matches!(action, h3_battle::Command::Cast { .. }));
        let asks = [Ask::Cast, Ask::Pressed, Ask::Cast, Ask::Cast, Ask::Action];
        assert_eq!(live.asks(), asks);
        // The computer's side's turns ask no cast of it; a turn started asks none until set.
        live.set_auto(false);
        while live.state.side_to_act() == Some(BattleSide::Defender) {
            live.next().unwrap().unwrap();
        }
        assert!(live.waiting());
        let again = refight(&ruleset, setup, 1, players, live.asks()).unwrap();
        assert_eq!(live.record().unwrap(), again);
    }

    /// `auto_cast`'s battle with the defender's stacks a hundred times as many and Orrin worth
    /// keeping (5000 experience, a Tavern town, 100 000 gold): the human's side under Auto Combat
    /// is overwhelmed, the computer's retreat test says yes (E17 M6 T5b2a).
    fn overwhelmed() -> (BattleRuleset, BattleSetup) {
        let (ruleset, mut setup) = auto_cast();
        for stack in &mut setup.defender.stacks {
            stack.count *= 100;
        }
        setup.retreat[0] = h3_contracts::RetreatFacts {
            experience: 5000,
            tavern: true,
            ..h3_contracts::RetreatFacts::default()
        };
        setup.funds[0] = 100_000;
        (ruleset, setup)
    }

    #[test]
    fn auto_combat_asks_the_human_once_a_battle_when_the_computer_would_leave() {
        // E17 M6 T5b2a (design §12.19): the retreat test (as of a human's side) before the
        // action of the human's stack that plays itself; its yes asks the human, no command
        // played. Cancel: the stack's action on the same decision's world, from the stream the
        // test left; no question again this battle, the test still asked (and drawing) at each
        // such action. Fought again by the asks, the same battle.
        let (ruleset, setup) = overwhelmed();
        let players = ([true, false], Computer::Original(Sight::Fair));
        let mut live = Live::new(&ruleset, setup.clone(), 1, players).unwrap();
        let side = BattleSide::Attacker;
        live.set_auto(true);
        let before = live.seats[0].clone();
        assert_eq!(live.next().unwrap(), None);
        assert_eq!(live.question(), Some(Question::Retreat));
        assert_eq!(live.asks(), [Ask::Action]);
        // The battle stands still: no command, the human's either.
        assert_eq!(live.next().unwrap(), None);
        let pressed = live.choice().unwrap();
        assert_eq!(live.play(pressed).unwrap(), None);
        // The decision made again from the stream before it: the test, then the action.
        let mut seat = before.clone();
        let decision = Decision::new(&live.state, &ruleset, side, Sight::Fair, &seat)
            .unwrap()
            .human();
        assert!(decision.retreat_test(&mut seat));
        assert_eq!(live.seats[0], seat);
        let action = decision.action(&mut seat);
        assert!(matches!(decision.leaving(), Leaving::Offer { .. }));
        let played = live.answer(false).unwrap().unwrap();
        assert!(played.turned && live.seats[0] == seat && !live.may_ask);
        assert_eq!(live.asks(), [Ask::Action, Ask::Answered(false)]);
        assert!(!matches!(action, h3_battle::Command::Retreat));
        // No question waits: an answer gives nothing.
        assert_eq!(live.answer(true).unwrap(), None);
        // The human's next stack: the test asked again, no question.
        live.set_auto(false);
        while live.state.side_to_act() == Some(BattleSide::Defender) {
            live.next().unwrap().unwrap();
        }
        live.set_auto(true);
        let mut seat = live.seats[0].clone();
        let decision = Decision::new(&live.state, &ruleset, side, Sight::Fair, &seat)
            .unwrap()
            .human();
        let yes = decision.retreat_test(&mut seat);
        let _ = decision.action(&mut seat);
        live.next().unwrap().unwrap();
        assert!(yes && live.seats[0] == seat && live.question().is_none());
        let again = refight(&ruleset, setup, 1, players, live.asks()).unwrap();
        assert_eq!(live.record().unwrap(), again);
    }

    #[test]
    fn the_question_s_ok_leaves_by_the_offer_or_by_the_retreat() {
        // E17 M6 T5b2a: OK gives the computer's leaving: the offer (both heroes, the gold
        // enough), whose OK surrenders and whose Cancel retreats; without the enemy's hero the
        // Retreat at once. Each fought again by the asks.
        let (ruleset, setup) = overwhelmed();
        let players = ([true, false], Computer::Original(Sight::Fair));
        for (offer_ok, ends) in [(true, Command::Surrender {}), (false, Command::Retreat {})] {
            let mut live = Live::new(&ruleset, setup.clone(), 1, players).unwrap();
            live.set_auto(true);
            assert_eq!(live.next().unwrap(), None);
            assert_eq!(live.answer(true).unwrap(), None);
            let Some(Question::Offer { price }) = live.question() else {
                panic!("the offer: {:?}", live.question());
            };
            assert!(price > 0 && live.next().unwrap().is_none());
            let played = live.answer(offer_ok).unwrap().unwrap();
            assert!(live.over() && !played.messages.is_empty());
            let asks = [Ask::Action, Ask::Answered(true), Ask::Answered(offer_ok)];
            assert_eq!(live.asks(), asks);
            let again = refight(&ruleset, setup.clone(), 1, players, &asks).unwrap();
            assert_eq!(live.record().unwrap(), again);
            use h3_protocol::wire::Outcome;
            let left = played.events.iter().find_map(|(e, _)| match e {
                Event::BattleEnded { result } => Some(*result),
                _ => None,
            });
            let side = Side::Attacker;
            let wanted = match ends {
                Command::Surrender {} => Outcome::Surrender { side },
                _ => Outcome::Retreat { side },
            };
            assert_eq!(left, Some(wanted));
        }
        let (ruleset, mut setup) = overwhelmed();
        setup.heroes[1] = None;
        let mut live = Live::new(&ruleset, setup, 1, players).unwrap();
        live.set_auto(true);
        assert_eq!(live.next().unwrap(), None);
        live.answer(true).unwrap().unwrap();
        assert!(live.over() && live.question().is_none());
        // The fixture's bot asks nothing.
        let (ruleset, setup) = overwhelmed();
        let greedy = ([true, false], Computer::Greedy);
        let mut live = Live::new(&ruleset, setup, 1, greedy).unwrap();
        live.set_auto(true);
        assert!(live.next().unwrap().is_some() && live.question().is_none());
    }

    #[test]
    fn a_stack_that_plays_itself_acts_in_the_same_ask_when_the_hero_casts_nothing() {
        // E17 M6 T5b1, the shape of `auto-cast` (Creatures set): under Auto Combat a cast asked
        // that gives none goes on to the stack's action at once (the cast's draws first, the
        // Champions'); one that gives a cast plays it alone (the Marksmen's Bless), the action
        // next. The fixed bot casts nothing: a turn it does not play waits.
        let (ruleset, setup) = auto_cast();
        let players = ([true, false], Computer::Original(Sight::Fair));
        let mut live = Live::new(&ruleset, setup.clone(), 1, players).unwrap();
        live.set_auto(true);
        live.set_cast(true);
        assert!(live.next().unwrap().unwrap().turned);
        assert_eq!(live.asks(), [Ask::Cast, Ask::Action]);
        assert_eq!(live.state.side_to_act(), Some(BattleSide::Attacker));
        live.set_cast(true);
        assert!(!live.next().unwrap().unwrap().turned);
        assert!(live.next().unwrap().unwrap().turned);
        let asks = [Ask::Cast, Ask::Action, Ask::Cast, Ask::Action];
        assert_eq!(live.asks(), asks);
        let again = refight(&ruleset, setup.clone(), 1, players, &asks).unwrap();
        assert_eq!(live.record().unwrap(), again);
        let greedy = ([true, false], Computer::Greedy);
        let mut asked = Live::new(&ruleset, setup, 1, greedy).unwrap();
        asked.set_cast(true);
        assert!(!asked.waiting());
        assert_eq!(asked.next().unwrap(), None);
        assert!(asked.waiting() && asked.asks() == [Ask::Cast]);
    }

    #[test]
    fn a_stop_read_at_the_cast_s_end_leaves_the_stack_s_action_to_the_human() {
        // E17 M6 T5b2b (design §12.20, CLT-Q61), the shape of `auto-cast-stop`: under Auto
        // Combat the computer casts Bless as the Marksmen's turn starts; a press during the cast
        // stops it at the cast's end (the battle sets it off), so the turn waits for the human
        // and the action is the human's. Fought again by the asks, the same battle.
        let (ruleset, setup) = auto_cast();
        let players = ([true, false], Computer::Original(Sight::Fair));
        let mut live = Live::new(&ruleset, setup.clone(), 1, players).unwrap();
        live.set_auto(true);
        live.set_cast(true);
        assert!(live.next().unwrap().unwrap().turned);
        live.set_cast(true);
        assert!(!live.next().unwrap().unwrap().turned);
        live.set_auto(false);
        assert!(live.waiting() && live.next().unwrap().is_none());
        let pressed = live.choice().unwrap();
        assert!(live.play(pressed).unwrap().unwrap().turned);
        let asks = [Ask::Cast, Ask::Action, Ask::Cast, Ask::Pressed];
        assert_eq!(live.asks(), asks);
        let again = refight(&ruleset, setup, 1, players, &asks).unwrap();
        assert_eq!(live.record().unwrap(), again);
    }

    /// The battle prepared for Auto Combat's question at a later turn and before a human's turn
    /// (E17 M6 T5b2b, `seq-auto-retreat-machine.toml`): 20 Dwarves and a Ballista Orrin commands
    /// against 3000 Wolf Riders and 1500 Goblins.
    fn machine() -> (BattleRuleset, BattleSetup) {
        let scenario = repo().join("clients/situations/seq-auto-retreat-machine.toml");
        let (ruleset, mut setup) =
            crate::scenario::battle(&scenario, &repo().join("packs"), 1).unwrap();
        setup.human = [true, false];
        (ruleset, setup)
    }

    /// The acting stack of `live`'s battle now.
    fn active(live: &Live<'_>) -> Option<u8> {
        live.turn().and_then(|t| t.observation.active)
    }

    #[test]
    fn the_machine_battle_asks_at_a_later_turn_and_before_a_human_s_turn() {
        // E17 M6 T5b2b (design §12.20), the sequences `auto-retreat-later` and `-human`: on grass
        // the Wolf Riders (2) and the Goblins (3) walk first, out of the Dwarves' reach, then the
        // Dwarves (0), then the Ballista (1), Orrin's to command. `auto-retreat-later`: the
        // Dwarves wait, the Ballista defends (a human's: its box clear under Auto Combat), the
        // Dwarves' waited turn plays itself and asks, Cancel walks them to 37.
        // `auto-retreat-human`: the Dwarves play themselves at once and ask, Cancel has them
        // wait, and the Ballista's turn is the human's, waiting.
        let (ruleset, setup) = machine();
        let players = ([true, false], Computer::Original(Sight::Fair));
        let walks = |live: &mut Live<'_>| {
            for (stack, to) in [(2, 43), (3, 146)] {
                assert_eq!(active(live), Some(stack));
                let played = live.next().unwrap().unwrap();
                let walked = |e: &Event| match *e {
                    Event::Moved {
                        stack: s, to: t, ..
                    } => (s, t) == (stack, to),
                    _ => false,
                };
                assert!(played.events.iter().any(|(e, _)| walked(e)), "{stack}");
            }
            assert!(live.waiting() && active(live) == Some(0));
        };
        let mut later = Live::new(&ruleset, setup.clone(), 1, players).unwrap();
        walks(&mut later);
        later.play(Command::Wait {}).unwrap().unwrap();
        assert!(later.waiting() && active(&later) == Some(1));
        later.play(Command::Defend {}).unwrap().unwrap();
        assert_eq!(active(&later), Some(0));
        later.set_auto(true);
        assert_eq!(later.next().unwrap(), None);
        assert_eq!(later.question(), Some(Question::Retreat));
        let played = later.answer(false).unwrap().unwrap();
        let walked = |e: &Event| {
            matches!(
                *e,
                Event::Moved {
                    stack: 0,
                    to: 37,
                    ..
                }
            )
        };
        assert!(played.events.iter().any(|(e, _)| walked(e)));
        let mut human = Live::new(&ruleset, setup.clone(), 1, players).unwrap();
        walks(&mut human);
        human.set_auto(true);
        assert_eq!(human.next().unwrap(), None);
        assert_eq!(human.question(), Some(Question::Retreat));
        let played = human.answer(false).unwrap().unwrap();
        let waited = |e: &Event| matches!(*e, Event::Waited { stack: 0 });
        assert!(played.events.iter().any(|(e, _)| waited(e)));
        assert_eq!(active(&human), Some(1));
        human.set_auto(false);
        assert!(human.waiting());
        let again = refight(&ruleset, setup, 1, players, human.asks()).unwrap();
        assert_eq!(human.record().unwrap(), again);
    }
}
