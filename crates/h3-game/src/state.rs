//! The whole game's state (ADR-0017, E08 T6b): the players with their RNG streams, the day's
//! schedule of turn groups, the calendar, the game's own stream, the treasuries (T13), the pending
//! decisions (T14b) and the contexts' states (the adventure now). Built from a validated [`GameSetup`]; the steps only the game triggers
//! (the start of a day, the start of a turn) are infallible, and commands go through
//! [`crate::apply`].

use alloc::vec::Vec;

use h3_adventure::{AdventureState, Offer};
use h3_content::ResourceId;
use h3_contracts::PlayerId;
use h3_kernel::hash::hash_of;
use h3_kernel::{CanonicalHash, Rng, StateHasher, StreamId};

use crate::command::{AdventureSink, Answer, GameCommandError, GameEvent, NoopSink, Sink};
use crate::treasury::Treasuries;
use crate::{GAME_STATE_HASH_VERSION, GameRuleset, GameSetup, GameSetupError};

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

/// A pending decision (ADR-0017 point 6, GAM-DEC-001): what its owner must answer before
/// anything else. Increment 2's are offers (ADV-VIS-001).
#[derive(Debug, Clone, PartialEq, Eq)]
struct Decision {
    /// The player who answers it.
    owner: PlayerId,
    /// The offer.
    offer: Offer,
}

/// The state of a whole game.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameState {
    /// The master seed (ADR-0017 point 5: server-only, never in an observation).
    seed: u64,
    players: Vec<PlayerState>,
    /// The players' resources (GAM-RES-001, E08 T13), by player id.
    treasuries: Treasuries,
    /// The open decisions, in opening order (GAM-DEC-001, E08 T14b); a player has at most one in
    /// increment 2 (no command but its answer runs while it is open). Empty most of the time, so
    /// a fork copies nothing more.
    decisions: Vec<Decision>,
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
    adventure: AdventureState,
}

impl GameState {
    /// Builds a game from its setup and master seed: the setup's checks
    /// ([`GameSetup::validate`]), its random placeholders resolved from the setup stream (the
    /// starting armies, GAM-SET-001, then the map objects' contents, ADR-0016 point 4), a stream
    /// and the difficulty's human row of starting resources per player (GAM-RES-002), sequential
    /// play (GAM-TRN-001) on day 1
    /// (GAM-CAL-001), and the first group's turn started, so the first day starts as every later
    /// one (ADR-0017 point 3; no context has a rule at the start of a day yet).
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
        let part = setup.adventure_setup(rules, &mut draws)?;
        let adventure = AdventureState::new(rules.core(), rules.adventure(), &part, &mut draws)
            .map_err(GameSetupError::Adventure)?;
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
            decisions: Vec::new(),
            players,
            groups,
            group: 0,
            day: 1,
            rng: Rng::for_stream(seed, StreamId::GAME),
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
            self.advance_day(sink);
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
    /// ([`Self::check_acting`]) and it has no open decision, whose answers are then its only legal
    /// commands (ADR-0017 point 6).
    // rule: GAM-DEC-001
    pub(crate) fn check_ready(&self, player: PlayerId) -> Result<(), GameCommandError> {
        self.check_acting(player)?;
        if self.decision(player).is_some() {
            return Err(GameCommandError::DecisionOpen);
        }
        Ok(())
    }

    /// Opens the offer a command of `player` made (ADR-0017 point 10): as a pending decision of
    /// the player if its treasury can pay the price (GAM-DEC-001), told to it alone; otherwise
    /// the offer does not open, and the player alone is told so (ADV-VIS-001: the original's
    /// message).
    // rule: GAM-DEC-001, ADV-VIS-001
    pub(crate) fn open<S: Sink>(&mut self, player: PlayerId, offer: Offer, sink: &mut S) {
        let (hero, object_type, anchor) = (offer.hero, offer.object_type, offer.anchor);
        if self.treasuries.can_pay(player, &offer.price) {
            self.decisions.push(Decision {
                owner: player,
                offer,
            });
            sink.emit(GameEvent::OfferOpened {
                player,
                hero,
                object_type,
                anchor,
            });
        } else {
            sink.emit(GameEvent::OfferRefused {
                player,
                hero,
                object_type,
                anchor,
            });
        }
    }

    /// `Answer`: closes `player`'s first open decision (GAM-DEC-001), in its turn or not. Accepted,
    /// the treasury pays the price and the adventure carries the offer out
    /// ([`AdventureState::accept_offer`]: the reveal and the purchase, ADV-VIS-001); declined,
    /// nothing else changes. ADR-0017 point 10's checks at the answer, that the treasury can pay
    /// and that the offer still stands, cannot fail and are not made: the decision opened only
    /// when the treasury could pay, and while it is open its owner's only legal commands are its
    /// answers and no other player's command changes the owner's treasury or purchases
    /// (ownership isolation), so an accepted answer is never rejected (`h3-map`'s play
    /// properties check it at every step).
    ///
    /// # Errors
    ///
    /// `UnknownPlayer`, then `NoDecision`; the state is then unchanged.
    // rule: GAM-DEC-001
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
        let at = self
            .decisions
            .iter()
            .position(|d| d.owner == player)
            .ok_or(GameCommandError::NoDecision)?;
        let Decision { offer, .. } = self.decisions.remove(at);
        let accepted = answer == Answer::Accept;
        sink.emit(GameEvent::OfferAnswered {
            player,
            object_type: offer.object_type,
            anchor: offer.anchor,
            accepted,
        });
        if accepted {
            for &(resource, amount) in &offer.price {
                let amount = self.treasuries.pay(player, resource, amount);
                sink.emit(GameEvent::ResourcePaid {
                    player,
                    resource,
                    amount,
                });
            }
            self.adventure.accept_offer(
                rules.adventure(),
                player,
                &offer,
                &mut AdventureSink(sink),
            );
        }
        Ok(())
    }

    /// The offer of `player`'s first open decision, if it has one (GAM-DEC-001; increment 2's
    /// decisions are offers): what its owner sees of it ([`crate::GameObservation::decision`]).
    pub fn decision(&self, player: PlayerId) -> Option<&Offer> {
        self.decisions
            .iter()
            .find(|d| d.owner == player)
            .map(|d| &d.offer)
    }

    /// The day advance (ADR-0017 point 3): the calendar, then each context's start-of-day step
    /// (none has a rule there yet).
    // rule: GAM-TRN-002, GAM-CAL-001
    fn advance_day<S: Sink>(&mut self, sink: &mut S) {
        self.day = self.day.saturating_add(1);
        sink.emit(GameEvent::DayStarted { day: self.day });
    }

    /// Starts the turn of the current group: each of its players acts, and each context runs its
    /// start-of-turn step for it (the adventure's movement points, ADV-MOV-002).
    fn start_group<S: Sink>(&mut self, rules: &GameRuleset, sink: &mut S) {
        let group = self.groups[self.group].clone();
        for player in group {
            self.players[player.index()].acting = true;
            self.adventure
                .start_turn(rules.core(), rules.adventure(), player);
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

    /// The adventure context's state.
    pub const fn adventure(&self) -> &AdventureState {
        &self.adventure
    }

    /// The adventure context's state, for the commands routed to it.
    pub(crate) const fn adventure_mut(&mut self) -> &mut AdventureState {
        &mut self.adventure
    }

    /// The canonical state hash (ADR-0003 point 12): content only. It covers the seed and the
    /// streams, so it never reaches a client (ADR-0017 point 5).
    pub fn state_hash(&self) -> u64 {
        hash_of(self)
    }
}

impl CanonicalHash for GameState {
    /// Encoding v3 (E08 T14b; v1 was T6b's, v2 T13's): version; the master seed; the day; the
    /// running group's index; the schedule (groups as length + player ids); the players (colour,
    /// acting, stream state); since v2 the treasuries (the resource kinds, then every player's
    /// amounts as length and entries); since v3 the open decisions in opening order, as length
    /// and entries (owner, hero, object type, anchor, the price as length and resource–amount
    /// pairs); the game's stream; the adventure state's hash.
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
        h.write_len(self.decisions.len());
        for Decision { owner, offer } in &self.decisions {
            h.write_u8(owner.raw());
            h.write_u16(offer.hero.raw());
            offer.object_type.hash_into(h);
            offer.anchor.hash_into(h);
            h.write_len(offer.price.len());
            for (resource, amount) in &offer.price {
                resource.hash_into(h);
                h.write_u32(*amount);
            }
        }
        self.rng.hash_into(h);
        h.write_u64(self.adventure.state_hash());
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
}
