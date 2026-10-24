//! The whole game's state (ADR-0017, E08 T6b): the players with their RNG streams, the day's
//! schedule of turn groups, the calendar, the game's own stream, the treasuries (T13), the pending
//! decisions (T14b; battles since T19a, `state/battle.rs`; level-ups since T30,
//! `state/levels.rs`) and the contexts' states (the
//! adventure; a battle lives in its decision). Built from a validated
//! [`GameSetup`]; the steps only the game triggers (the start of a day, the start of a turn) are
//! infallible, and commands go through [`crate::apply`].

use alloc::sync::Arc;
use alloc::vec::Vec;

use h3_adventure::{AdventureOutcome, AdventureState, Offer, OfferKind};
use h3_content::{ResourceId, SkillId};
use h3_contracts::{HeroId, PlayerId};
use h3_heroes::{ArtifactInstance, HeroDetails, LevelUp, OfferLevels, Offers};
use h3_kernel::hash::hash_of;
use h3_kernel::{CanonicalHash, Rng, StateHasher, StreamId};

use crate::command::{Actor, AdventureSink, Answer, GameCommandError, GameEvent, NoopSink, Sink};
use crate::observe::LevelUpView;
use crate::treasury::Treasuries;
use crate::{GAME_STATE_HASH_VERSION, GameRuleset, GameSetup, GameSetupError};
use levels::Progression;

mod battle;
mod levels;
mod manage;

pub use battle::OpenBattle;
#[cfg(test)]
pub(crate) use battle::{raised_count, seen_by};

/// A player of a running game.
#[derive(Debug, Clone, PartialEq, Eq)]
struct PlayerState {
    /// Its colour (0 red to 7 pink).
    colour: u8,
    /// Its stream: randomness its commands cause draws only from it (ADR-0017 point 4).
    rng: Rng,
    /// Whether its turn is running: its group's turn has started and it has not ended its turn.
    acting: bool,
}

/// A pending decision (ADR-0017 point 6, GAM-DEC-001; E08 T14b): what its owners must answer
/// before anything else. Increment 2's are offers (ADV-VIS-001, ADV-VIS-004), increment 3's the
/// wandering monsters' too (ADV-MON-008, E08 T20b1); since E08 T19a a battle, owned by the
/// players of its sides, who play it; since E08 T30 a level-up with two offers (GAM-DEC-002).
#[derive(Debug, Clone, PartialEq, Eq)]
enum Decision {
    /// An offer, answered by its owner.
    Offer {
        /// The player who answers it.
        owner: PlayerId,
        /// The offer.
        offer: Offer,
        /// Whether the visit of a monster guarding the hero's tile waits for the answer
        /// (`AdventureOutcome::guard_after`, ADV-MON-006; E08 T20b1).
        guard_after: bool,
    },
    /// A battle (GAM-BAT-001), shared between forks and copied on its first write.
    Battle(Arc<OpenBattle>),
    /// A level-up with two offers (GAM-DEC-002, E08 T30), answered by the hero's owner.
    LevelUp {
        /// The hero's owner, who answers it.
        owner: PlayerId,
        /// The level drawn when the decision opened (GAM-HER-002) as its owner sees it: the
        /// hero, the new level, the primary skill, the two offers.
        view: LevelUpView,
        /// The offers' levels after the level (HER-LVL-004), whatever the answer.
        offered: OfferLevels,
    },
}

/// The drawn level of a level-up decision: `view`'s level with its two offers and `offered`.
const fn drawn(view: &LevelUpView, offered: OfferLevels) -> LevelUp {
    let [first, second] = view.offers;
    LevelUp {
        level: view.level,
        primary: view.primary,
        offers: Offers::Two(first, second),
        offered,
    }
}

impl Decision {
    /// Whether `player` owns the decision.
    fn owned_by(&self, player: PlayerId) -> bool {
        match self {
            Self::Offer { owner, .. } | Self::LevelUp { owner, .. } => *owner == player,
            Self::Battle(battle) => battle.owners().contains(&Some(player)),
        }
    }

    /// The battle, if the decision is one.
    fn battle(&self) -> Option<&Arc<OpenBattle>> {
        match self {
            Self::Battle(battle) => Some(battle),
            Self::Offer { .. } | Self::LevelUp { .. } => None,
        }
    }

    /// The battle, if the decision is one, to write.
    fn battle_mut(&mut self) -> Option<&mut Arc<OpenBattle>> {
        match self {
            Self::Battle(battle) => Some(battle),
            Self::Offer { .. } | Self::LevelUp { .. } => None,
        }
    }
}

/// The state of a whole game.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameState {
    /// The master seed (ADR-0017 point 5: server-only, never in an observation).
    seed: u64,
    players: Vec<PlayerState>,
    /// The players' resources (GAM-RES-001, E08 T13), by player id.
    treasuries: Treasuries,
    /// The open decisions in opening order (GAM-DEC-001, E08 T14b); a player has at most one in
    /// sequential play (no command but its answer runs while it is open; a battle's owners both
    /// have it). None while no decision is
    /// open, never an empty list (equal content compares equal), and shared by forks: a fork
    /// costs nothing more without a decision and a reference count with one (`bench-check`; a
    /// list per fork, or one per player, cost the XL fork benchmark 3 to 4 %).
    decisions: Option<Arc<[Decision]>>,
    /// The day's schedule: turn groups in order, each the players who act together
    /// (ADR-0017 point 3). Sequential play, the only schedule of increment 1, is one player per
    /// group in player order.
    groups: Vec<Vec<PlayerId>>,
    /// The group whose turn is running.
    group: usize,
    /// The day, from 1.
    day: u32,
    /// The game's own stream (the day advance; ADR-0017 point 4).
    rng: Rng,
    /// The store of heroes (GAM-HER-001, E08 T18b): every hero's details by id, each shared
    /// between forks and copied on its first write (ADR-0017 point 1). The list too is shared, so
    /// a fork costs one reference count (`bench-check`: a list per fork cost the XL fork
    /// benchmark about 10 %); a write copies the list of references and the one hero.
    heroes: Arc<Vec<Arc<HeroDetails>>>,
    /// What the heroes' levels read that never changes after the build (E08 T29): the scenario's
    /// level limit and banned skills, each hero's level seed (GAM-HER-002; server-only, ADR-0017
    /// point 5). Shared by forks.
    progression: Arc<Progression>,
    adventure: AdventureState,
}

impl GameState {
    /// Builds a game from its setup and master seed: the setup's checks
    /// ([`GameSetup::validate`]), its random placeholders resolved from the setup stream (the
    /// starting armies, GAM-SET-001, each default experience, GAM-HER-001, the map objects'
    /// contents, ADR-0016 point 4, then each hero's level seed in hero id order, GAM-HER-002;
    /// last of the stream's draws, so the earlier ones do not depend on the heroes' levels), the
    /// levels at load of each hero with a custom experience in hero id order (GAM-SET-002: its
    /// levels' draws from the setup stream in `strict-fixed`), the adventure's start of the game
    /// after them (the heroes' day's points and reveals from their details, E08 T40), a stream
    /// and the difficulty's human row of starting resources per player (GAM-RES-002), sequential
    /// play (GAM-TRN-001) on day 1 (GAM-CAL-001), day 1's start-of-day step and the first group's
    /// turn started, so the first day starts as every later one (ADR-0017 point 3).
    ///
    /// # Errors
    ///
    /// The first [`GameSetupError`] found, in the order of [`GameSetup::validate`].
    pub fn new(rules: &GameRuleset, setup: &GameSetup, seed: u64) -> Result<Self, GameSetupError> {
        // rule: GAM-RES-002
        let start = rules
            .adventure()
            .starting_resources()
            .get(usize::from(setup.difficulty))
            .ok_or(GameSetupError::Difficulty(setup.difficulty))?;
        let mut draws = Rng::for_stream(seed, StreamId::SETUP);
        let (part, mut heroes) = setup.adventure_setup(rules, &mut draws)?;
        let mut adventure = AdventureState::new(rules.core(), rules.adventure(), &part, &mut draws)
            .map_err(GameSetupError::Adventure)?;
        let progression = Progression::new(
            setup.level_limit,
            &setup.banned_skills,
            heroes.len(),
            &mut draws,
        );
        let shares = adventure.map().terrain_shares();
        for ((details, hero), index) in heroes.iter_mut().zip(&setup.heroes).zip(0..=u16::MAX) {
            if let Some(experience) = hero.details.experience {
                let hero = (HeroId::new(index), details);
                progression.levels_at_load(rules.core(), hero, experience, shares, &mut draws);
            }
        }
        let heroes: Arc<Vec<Arc<HeroDetails>>> =
            Arc::new(heroes.into_iter().map(Arc::new).collect());
        // ADR-0017 point 1 (E08 T40): the adventure's start of the game after the levels at
        // load, then day 1's start-of-day step (ADV-MOV-002, ADV-FOG-003).
        adventure.start_game(rules.core(), rules.adventure(), &heroes);
        adventure.start_day(rules.core(), rules.adventure(), &heroes);
        let players: Vec<PlayerState> = setup
            .players
            .iter()
            .zip(0..=u8::MAX)
            .map(|(player, index)| PlayerState {
                colour: player.colour,
                rng: Rng::for_stream(seed, StreamId::player(index)),
                acting: false,
            })
            .collect();
        // rule: GAM-TRN-001
        let groups = (0..=u8::MAX)
            .take(players.len())
            .map(|index| alloc::vec![PlayerId::new(index)])
            .collect();
        let mut state = Self {
            seed,
            treasuries: Treasuries::new(players.len(), start.human()),
            decisions: None,
            players,
            groups,
            group: 0,
            day: 1,
            rng: Rng::for_stream(seed, StreamId::GAME),
            heroes,
            progression: Arc::new(progression),
            adventure,
        };
        state.start_group(rules, &mut NoopSink);
        Ok(state)
    }

    /// Ends `player`'s turn (`EndTurn`). When every player of the group has ended its turn the
    /// next group's turn starts; after the last group the day advances (GAM-TRN-002) and the
    /// first group's turn starts. A rejected command changes nothing.
    pub(crate) fn end_turn<S: Sink>(
        &mut self,
        rules: &GameRuleset,
        player: PlayerId,
        sink: &mut S,
    ) -> Result<(), GameCommandError> {
        self.check_ready(player)?;
        self.players[player.index()].acting = false;
        sink.emit(GameEvent::TurnEnded { player });
        if self.players.iter().any(|p| p.acting) {
            return Ok(());
        }
        self.group += 1;
        if self.group == self.groups.len() {
            self.group = 0;
            self.advance_day(rules, sink);
        }
        self.start_group(rules, sink);
        Ok(())
    }

    /// Checks that `player` is a player of the game whose turn is running (the game's part of
    /// every player command, ADR-0017 point 1).
    pub(crate) fn check_acting(&self, player: PlayerId) -> Result<(), GameCommandError> {
        let state = self
            .players
            .get(player.index())
            .ok_or(GameCommandError::UnknownPlayer(player))?;
        if !state.acting {
            return Err(GameCommandError::NotYourTurn(player));
        }
        Ok(())
    }

    /// Checks that `player` may give a command other than an answer: its turn is running
    /// ([`Self::check_acting`]), it has no open decision, whose answers are then its only legal
    /// commands (ADR-0017 point 6), and the game waits for no other player's level-up
    /// ([`Self::check_held`]).
    // rule: GAM-DEC-001, GAM-DEC-002
    pub(crate) fn check_ready(&self, player: PlayerId) -> Result<(), GameCommandError> {
        self.check_acting(player)?;
        if self.has_decision(player) {
            return Err(GameCommandError::DecisionOpen);
        }
        self.check_held(Actor::Player(player))
    }

    /// Checks that the game waits for no level-up but `actor`'s own (ADR-0017 point 6,
    /// GAM-DEC-002; E08 T30): an open level-up holds the game, as the original's modal dialog,
    /// so until its owner answers no other actor's command is accepted.
    // rule: GAM-DEC-002
    pub(crate) fn check_held(&self, actor: Actor) -> Result<(), GameCommandError> {
        match self.waiting_for() {
            Some(owner) if actor != Actor::Player(owner) => {
                Err(GameCommandError::WaitingForLevelUp(owner))
            }
            _ => Ok(()),
        }
    }

    /// The player whose level-up the game waits for, if one is open (ADR-0017 point 6,
    /// GAM-DEC-002; E08 T30): the hosting layer hands the game to it (hot seat) or pauses the
    /// other players' turn timers and answers for it with [`crate::default_answer`] after its
    /// time (online). Another player learns only this, never the level-up.
    pub fn waiting_for(&self) -> Option<PlayerId> {
        self.decisions
            .as_deref()
            .unwrap_or_default()
            .iter()
            .find_map(|d| match d {
                Decision::LevelUp { owner, .. } => Some(*owner),
                Decision::Offer { .. } | Decision::Battle(_) => None,
            })
    }

    /// Applies what the adventure left of a command of `player` or of its answer (ADR-0017
    /// point 10): the gains (GAM-RES-001), then the offer ([`Self::open`]) with whatever follows
    /// one that does not open, then the battle ([`Self::start_battle`]; an outcome has an offer or
    /// a battle, not both).
    // rule: GAM-DEC-001
    pub(crate) fn settle_outcome<S: Sink>(
        &mut self,
        rules: &GameRuleset,
        player: PlayerId,
        outcome: AdventureOutcome,
        sink: &mut S,
    ) {
        let mut next = Some(outcome);
        while let Some(outcome) = next.take() {
            self.gain(player, &outcome.gains, sink);
            next = outcome
                .offer
                .and_then(|offer| self.open(rules, player, offer, outcome.guard_after, sink));
            if let Some(battle) = outcome.battle {
                self.start_battle(rules, player, battle, sink);
            }
        }
    }

    /// Opens the offer a command of `player` made (ADR-0017 point 10): as a pending decision of
    /// the player if its treasury can pay the price (GAM-DEC-001; a border guard's has none, so it
    /// always opens, ADV-VIS-004), told to it alone. Otherwise the offer does not open: a
    /// wandering monster's offer to join for its price counts as declined, without an event, and
    /// the monster's answer goes on ([`AdventureState::decline_offer`]; ADV-MON-008, E08 T20b1);
    /// any other's refusal is told to the player alone (ADV-VIS-001: the original's message), and
    /// the visit of a monster guarding the hero's tile that waited for it follows
    /// ([`AdventureState::visit_guard`], ADV-MON-006). Returns that outcome, for the caller to
    /// apply.
    // rule: GAM-DEC-001, ADV-VIS-001, ADV-VIS-004, ADV-MON-006, ADV-MON-008
    pub(crate) fn open<S: Sink>(
        &mut self,
        rules: &GameRuleset,
        player: PlayerId,
        offer: Offer,
        guard_after: bool,
        sink: &mut S,
    ) -> Option<AdventureOutcome> {
        let (hero, object_type, anchor) = (offer.hero, offer.object_type, offer.anchor);
        if self.treasuries.can_pay(player, offer.price()) {
            self.push_decision(Decision::Offer {
                owner: player,
                offer,
                guard_after,
            });
            sink.emit(GameEvent::OfferOpened {
                player,
                hero,
                object_type,
                anchor,
            });
            return None;
        }
        let (adventure, heroes) = self.adventure_with_heroes();
        let context = (rules.core(), rules.adventure(), heroes);
        if matches!(offer.kind, OfferKind::JoinForGold { .. }) {
            return Some(adventure.decline_offer(context, &offer, &mut AdventureSink(sink)));
        }
        sink.emit(GameEvent::OfferRefused {
            player,
            hero,
            object_type,
            anchor,
        });
        guard_after.then(|| adventure.visit_guard(context, hero))
    }

    /// `Answer`: closes `player`'s first open decision (GAM-DEC-001), in its turn or not: an offer
    /// ([`Self::answer_offer`]) or a level-up ([`Self::answer_level_up`], GAM-DEC-002, E08 T30).
    ///
    /// # Errors
    ///
    /// `UnknownPlayer`, then `NoDecision`, then `BattleOpen` when the first is a battle (E08
    /// T19a), then `WrongAnswer` for an answer of the other kind (E08 T30), then `OfferGone` for
    /// an acceptance; the state is then unchanged.
    // rule: GAM-DEC-001, GAM-DEC-002
    pub(crate) fn answer<S: Sink>(
        &mut self,
        rules: &GameRuleset,
        player: PlayerId,
        answer: Answer,
        sink: &mut S,
    ) -> Result<(), GameCommandError> {
        self.players
            .get(player.index())
            .ok_or(GameCommandError::UnknownPlayer(player))?;
        let at = self.first(player).ok_or(GameCommandError::NoDecision)?;
        match self.decisions.as_deref().unwrap_or_default()[at].clone() {
            Decision::Battle(_) => Err(GameCommandError::BattleOpen),
            Decision::LevelUp { view, offered, .. } => {
                let choice = answer.choice().ok_or(GameCommandError::WrongAnswer)?;
                let level_up = drawn(&view, offered);
                self.answer_level_up(rules, ((view.hero, player), at), (&level_up, choice), sink);
                Ok(())
            }
            Decision::Offer {
                offer, guard_after, ..
            } => {
                if answer.choice().is_some() {
                    return Err(GameCommandError::WrongAnswer);
                }
                let accepted = answer == Answer::Accept;
                if accepted && !self.adventure.offer_stands(&offer) {
                    return Err(GameCommandError::OfferGone);
                }
                self.close(at);
                self.answer_offer(rules, player, (&offer, guard_after), accepted, sink);
                Ok(())
            }
        }
    }

    /// An offer's answer, its decision closed (GAM-DEC-001). Accepted (the offer still stands,
    /// [`AdventureState::offer_stands`], ADR-0017 point 10: a border guard another player of the
    /// turn group passed has left the map, E08 T15a; checked by [`Self::answer`]), the treasury
    /// pays the price and the adventure carries the offer out ([`AdventureState::accept_offer`]:
    /// a Cartographer's reveal and purchase, ADV-VIS-001; a guard's removal, ADV-VIS-004; a
    /// wandering monster's group joining with its treasure, or the pursuit of a fleeing one,
    /// ADV-MON-008, E08 T20b1). Declined, the adventure's answer goes on
    /// ([`AdventureState::decline_offer`]: after a monster's declined offer to join its next
    /// answer, after a declined flight its leaving; nothing for the others). What follows is
    /// applied as a command's outcome ([`Self::settle_outcome`]): a monster's next offer opens in
    /// the same command, so the player still has at most one open decision. Then the visit of a
    /// monster guarding the hero's tile that waited for this answer (ADV-MON-006). ADR-0017 point
    /// 10's other check, that the treasury can pay, cannot fail and is not made: the decision
    /// opened only when the treasury could pay, and while it is open its owner's only legal
    /// commands are its answers and no other player's command changes the owner's treasury
    /// (ownership isolation; `h3-map`'s play properties check it at every step).
    // rule: GAM-DEC-001, ADV-VIS-004, ADV-MON-006, ADV-MON-008
    fn answer_offer<S: Sink>(
        &mut self,
        rules: &GameRuleset,
        player: PlayerId,
        (offer, guard_after): (&Offer, bool),
        accepted: bool,
        sink: &mut S,
    ) {
        sink.emit(GameEvent::OfferAnswered {
            player,
            object_type: offer.object_type,
            anchor: offer.anchor,
            accepted,
        });
        if accepted {
            for &(resource, amount) in offer.price() {
                let amount = self.treasuries.pay(player, resource, amount);
                sink.emit(GameEvent::ResourcePaid {
                    player,
                    resource,
                    amount,
                });
            }
        }
        let (adventure, heroes) = self.adventure_with_heroes();
        let context = (rules.core(), rules.adventure(), heroes);
        let mut events = AdventureSink(&mut *sink);
        let outcome = if accepted {
            adventure.accept_offer(context.0, context.1, player, offer, &mut events)
        } else {
            adventure.decline_offer(context, offer, &mut events)
        };
        self.settle_outcome(rules, player, outcome, sink);
        if guard_after {
            let (adventure, heroes) = self.adventure_with_heroes();
            let context = (rules.core(), rules.adventure(), heroes);
            let guard = adventure.visit_guard(context, offer.hero);
            self.settle_outcome(rules, player, guard, sink);
        }
    }

    /// The offer of `player`'s first open decision, if it has one and it is an offer
    /// (GAM-DEC-001): what its owner sees of it ([`crate::GameObservation::decision`]). It may no
    /// longer stand ([`AdventureState::offer_stands`]): then only declining it is legal. A battle
    /// is [`Self::battle`], a level-up [`Self::level_up`].
    pub fn decision(&self, player: PlayerId) -> Option<&Offer> {
        match &self.decisions.as_deref().unwrap_or_default()[self.first(player)?] {
            Decision::Offer { offer, .. } => Some(offer),
            Decision::Battle(_) | Decision::LevelUp { .. } => None,
        }
    }

    /// The level-up of `player`'s first open decision, if it has one and it is a level-up
    /// (GAM-DEC-002, E08 T30): what its owner sees of it ([`crate::GameObservation::decision`]).
    pub fn level_up(&self, player: PlayerId) -> Option<LevelUpView> {
        self.open_level_up(player).map(|(view, _)| view)
    }

    /// The level-up of `player`'s first open decision, if it is one: as its owner sees it, and the
    /// drawn level its answer gives.
    pub(crate) fn open_level_up(&self, player: PlayerId) -> Option<(LevelUpView, LevelUp)> {
        match &self.decisions.as_deref().unwrap_or_default()[self.first(player)?] {
            Decision::LevelUp { view, offered, .. } => Some((*view, drawn(view, *offered))),
            Decision::Battle(_) | Decision::Offer { .. } => None,
        }
    }

    /// Whether `player` has an open decision, an offer, a level-up or a battle (GAM-DEC-001).
    pub fn has_decision(&self, player: PlayerId) -> bool {
        self.first(player).is_some()
    }

    /// The index of `player`'s first open decision, if it has one.
    fn first(&self, player: PlayerId) -> Option<usize> {
        self.decisions
            .as_deref()
            .unwrap_or_default()
            .iter()
            .position(|d| d.owned_by(player))
    }

    /// Opens a decision after the open ones.
    fn push_decision(&mut self, decision: Decision) {
        let open = self
            .decisions
            .as_deref()
            .unwrap_or_default()
            .iter()
            .cloned();
        self.decisions = Some(open.chain([decision]).collect());
    }

    /// Replaces the decision at `at` with `decision` (E08 T30: a hero's next level-up).
    fn replace(&mut self, at: usize, decision: Decision) {
        let mut list: Vec<Decision> = self.decisions.as_deref().unwrap_or_default().to_vec();
        list[at] = decision;
        self.decisions = Some(list.into());
    }

    /// Closes the decision at `at` (none open: no list).
    fn close(&mut self, at: usize) {
        let rest: Vec<Decision> = self
            .decisions
            .as_deref()
            .unwrap_or_default()
            .iter()
            .enumerate()
            .filter(|&(i, _)| i != at)
            .map(|(_, d)| d.clone())
            .collect();
        self.decisions = (!rest.is_empty()).then(|| rest.into());
    }

    /// The day advance (ADR-0017 point 3): the calendar, then each context's start-of-day step:
    /// the adventure's (E08 T40: in `strict-fixed` each hero's reveal, ADV-FOG-003).
    // rule: GAM-TRN-002, GAM-CAL-001
    fn advance_day<S: Sink>(&mut self, rules: &GameRuleset, sink: &mut S) {
        self.day = self.day.saturating_add(1);
        sink.emit(GameEvent::DayStarted { day: self.day });
        self.adventure
            .start_day(rules.core(), rules.adventure(), &self.heroes);
    }

    /// Starts the turn of the current group: each of its players acts, and each context runs its
    /// start-of-turn step for it (the adventure's movement points, ADV-MOV-002).
    fn start_group<S: Sink>(&mut self, rules: &GameRuleset, sink: &mut S) {
        let group = self.groups[self.group].clone();
        for player in group {
            self.players[player.index()].acting = true;
            self.adventure
                .start_turn(rules.core(), rules.adventure(), &self.heroes, player);
            sink.emit(GameEvent::TurnStarted { player });
        }
    }

    /// The day, from 1.
    pub const fn day(&self) -> u32 {
        self.day
    }

    /// How many players the game has.
    pub fn players(&self) -> usize {
        self.players.len()
    }

    /// A player's colour, if the player exists.
    pub fn colour(&self, player: PlayerId) -> Option<u8> {
        self.players.get(player.index()).map(|p| p.colour)
    }

    /// The players of the group whose turn is running.
    pub fn turn_group(&self) -> &[PlayerId] {
        &self.groups[self.group]
    }

    /// Whether `player`'s turn is running: its group's turn has started and it has not ended its
    /// turn.
    pub fn is_acting(&self, player: PlayerId) -> bool {
        self.players.get(player.index()).is_some_and(|p| p.acting)
    }

    /// Adds a command's gains to `player`'s treasury (ADR-0017 point 10: the context applied the
    /// command, and a gain cannot fail), each told to the player alone as the amount it added.
    pub(crate) fn gain<S: Sink>(
        &mut self,
        player: PlayerId,
        gains: &[(ResourceId, u32)],
        sink: &mut S,
    ) {
        for &(resource, amount) in gains {
            let amount = self.treasuries.gain(player, resource, amount);
            sink.emit(GameEvent::ResourceGained {
                player,
                resource,
                amount,
            });
        }
    }

    /// A player's treasury, by resource in the core's order, if the player exists (GAM-RES-001).
    pub fn treasury(&self, player: PlayerId) -> Option<&[u32]> {
        self.treasuries.of(player)
    }

    /// A hero's details (GAM-HER-001), if the game has the hero.
    pub fn hero(&self, hero: HeroId) -> Option<&HeroDetails> {
        self.heroes
            .get(usize::from(hero.raw()))
            .map(|details| &**details)
    }

    /// The store of heroes (GAM-HER-001): every hero's details, by id. A context reads them by
    /// reference through the hero model (ADR-0017 point 1).
    pub fn heroes(&self) -> &[Arc<HeroDetails>] {
        &self.heroes
    }

    /// The scenario's level limit (HER-EXP-003; E08 T29): 0 none.
    pub fn level_limit(&self) -> u8 {
        self.progression.level_limit()
    }

    /// The secondary skills the scenario bans (HER-LVL-003; E08 T29), in id order.
    pub fn banned_skills(&self) -> &[SkillId] {
        self.progression.banned()
    }

    /// The adventure context's state.
    pub const fn adventure(&self) -> &AdventureState {
        &self.adventure
    }

    /// The adventure state to change, with the store of heroes it reads (E08 T20a2b).
    pub(crate) fn adventure_with_heroes(&mut self) -> (&mut AdventureState, &[Arc<HeroDetails>]) {
        (&mut self.adventure, &self.heroes)
    }

    /// The canonical state hash (ADR-0003 point 12): content only. It covers the seed and the
    /// streams, so it never reaches a client (ADR-0017 point 5).
    pub fn state_hash(&self) -> u64 {
        hash_of(self)
    }
}

impl CanonicalHash for GameState {
    /// Encoding v11 (E08 T30; v1 was T6b's, v2 T13's, v3 T14b's, v4 T15a's, v5 T18b's, v6
    /// T19a's, v7 T20b1's, v8 T21a's, v9 T27's, v10 T29's): version;
    /// the master seed; the day; the running group's index; the schedule (groups as length +
    /// player ids); the players (colour, acting, stream state); since v2 the treasuries (the
    /// resource kinds, then every player's amounts as length and entries); since v3 the open
    /// decisions in opening order, as length and entries: since v6 first the decision's kind, 0
    /// an offer, 1 a battle; an offer's owner, hero, object type, anchor, since v4 the offer's
    /// kind, 0 a reveal, 1 a passage, since v7 2 a join, 3 a join for gold, 4 a flight, since v8
    /// 5 a guarded pile's fight, then a reveal's or a join for gold's price as length and
    /// resource–amount pairs, since v7 whether a guard's visit waits for the answer;
    /// a battle's owners and heroes by side (each 0 for none, else 1 and the id), tile and battle
    /// state hash; since v11 (E08 T30) 2 a level-up: its owner, hero, new level, primary skill
    /// (0 attack to 3 knowledge), its two offers, the offers' levels after it (Wisdom's, a
    /// school's); the game's stream; since v5 the store of heroes (length, then each hero's
    /// details by id, [`hash_details`]); since v10 (E08 T29) the heroes' progression (the level
    /// limit, the banned skills, the level seeds: [`Progression`]'s encoding); the adventure
    /// state's hash.
    fn hash_into(&self, h: &mut StateHasher) {
        h.write_u32(GAME_STATE_HASH_VERSION);
        h.write_u64(self.seed);
        h.write_u32(self.day);
        h.write_len(self.group);
        h.write_len(self.groups.len());
        for group in &self.groups {
            h.write_len(group.len());
            for player in group {
                h.write_u8(player.raw());
            }
        }
        h.write_len(self.players.len());
        for player in &self.players {
            h.write_u8(player.colour);
            h.write_bool(player.acting);
            player.rng.hash_into(h);
        }
        self.treasuries.hash_into(h);
        let decisions = self.decisions.as_deref().unwrap_or_default();
        h.write_len(decisions.len());
        for decision in decisions {
            match decision {
                Decision::Offer {
                    owner,
                    offer,
                    guard_after,
                } => {
                    h.write_u8(0);
                    h.write_u8(owner.raw());
                    h.write_u16(offer.hero.raw());
                    offer.object_type.hash_into(h);
                    offer.anchor.hash_into(h);
                    let (tag, price) = match &offer.kind {
                        OfferKind::Reveal { price } => (0, Some(price)),
                        OfferKind::Pass => (1, None),
                        OfferKind::Join => (2, None),
                        OfferKind::JoinForGold { price } => (3, Some(price)),
                        OfferKind::Pursue => (4, None),
                        OfferKind::Fight => (5, None),
                    };
                    h.write_u8(tag);
                    if let Some(price) = price {
                        h.write_len(price.len());
                        for (resource, amount) in price {
                            resource.hash_into(h);
                            h.write_u32(*amount);
                        }
                    }
                    h.write_bool(*guard_after);
                }
                Decision::Battle(battle) => {
                    h.write_u8(1);
                    battle.hash_into(h);
                }
                Decision::LevelUp {
                    owner,
                    view,
                    offered,
                } => {
                    h.write_u8(2);
                    h.write_u8(owner.raw());
                    h.write_u16(view.hero.raw());
                    h.write_u8(view.level);
                    h.write_u8(view.primary as u8);
                    for skill in view.offers {
                        skill.hash_into(h);
                    }
                    h.write_u8(offered.wisdom);
                    h.write_u8(offered.school);
                }
            }
        }
        self.rng.hash_into(h);
        h.write_len(self.heroes.len());
        for details in self.heroes.iter() {
            hash_details(details, h);
        }
        self.progression.hash_into(h);
        h.write_u64(self.adventure.state_hash());
    }
}

/// A hero's details in the game state's hash (v5; v9: the offers' levels after the experience;
/// v12: the worn artifacts by position): hero type, level, experience, the levels at which
/// Wisdom and a school were last offered, the primary skills, the secondary skills (length, then
/// skill and level), the worn positions (length, then per position 0 for empty, else 1 and the
/// artifact), the backpack (length, then the artifacts), the learned spells (length, then spell),
/// the spell points; an artifact is its type and spell, 0 for none, else 1 and the spell.
fn hash_details(details: &HeroDetails, h: &mut StateHasher) {
    details.hero_type.hash_into(h);
    h.write_u8(details.level);
    h.write_u32(details.experience);
    h.write_u8(details.offered.wisdom);
    h.write_u8(details.offered.school);
    h.write_bytes(&details.primary);
    h.write_len(details.skills.len());
    for (skill, level) in &details.skills {
        skill.hash_into(h);
        h.write_u8(*level);
    }
    h.write_len(details.worn.len());
    for item in &details.worn {
        match item {
            None => h.write_u8(0),
            Some(item) => {
                h.write_u8(1);
                hash_artifact(item, h);
            }
        }
    }
    h.write_len(details.backpack.len());
    for item in &details.backpack {
        hash_artifact(item, h);
    }
    h.write_len(details.spells.len());
    for spell in &details.spells {
        spell.hash_into(h);
    }
    h.write_u16(details.mana);
}

/// An artifact in a hero's details' hash: its type, then 0 for no spell, else 1 and the spell.
fn hash_artifact(&ArtifactInstance { artifact, spell }: &ArtifactInstance, h: &mut StateHasher) {
    artifact.hash_into(h);
    match spell {
        None => h.write_u8(0),
        Some(spell) => {
            h.write_u8(1);
            spell.hash_into(h);
        }
    }
}

#[cfg(test)]
impl GameState {
    /// Replaces the schedule (tests of turn groups larger than sequential play) and restarts the
    /// first group's turn.
    pub(crate) fn set_schedule(&mut self, rules: &GameRuleset, groups: Vec<Vec<PlayerId>>) {
        for player in &mut self.players {
            player.acting = false;
        }
        self.groups = groups;
        self.group = 0;
        self.start_group(rules, &mut NoopSink);
    }

    /// The master seed.
    pub(crate) const fn seed(&self) -> u64 {
        self.seed
    }

    /// A hero's level seed (GAM-HER-002).
    pub(crate) fn level_seed(&self, hero: HeroId) -> u8 {
        self.progression.seed(hero)
    }

    /// Whether two games share their progression (a fork's is shared, not copied).
    pub(crate) fn shares_progression(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.progression, &other.progression)
    }

    /// A player's stream as it stands.
    pub(crate) fn stream(&self, player: PlayerId) -> Rng {
        self.players[player.index()].rng.clone()
    }
}
