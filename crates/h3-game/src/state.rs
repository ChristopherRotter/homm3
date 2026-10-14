//! The whole game's state (ADR-0017, E08 T6b): the players with their RNG streams, the day's
//! schedule of turn groups, the calendar, the game's own stream, the treasuries (T13), the pending
//! decisions (T14b; battles since T19a, `state/battle.rs`) and the contexts' states (the
//! adventure; a battle lives in its decision). Built from a validated
//! [`GameSetup`]; the steps only the game triggers (the start of a day, the start of a turn) are
//! infallible, and commands go through [`crate::apply`].

use alloc::sync::Arc;
use alloc::vec::Vec;

use h3_adventure::{AdventureState, Offer, OfferKind};
use h3_content::ResourceId;
use h3_contracts::{HeroId, PlayerId};
use h3_heroes::{ArtifactInstance, HeroDetails};
use h3_kernel::hash::hash_of;
use h3_kernel::{CanonicalHash, Rng, StateHasher, StreamId};

use crate::command::{AdventureSink, Answer, GameCommandError, GameEvent, NoopSink, Sink};
use crate::treasury::Treasuries;
use crate::{GAME_STATE_HASH_VERSION, GameRuleset, GameSetup, GameSetupError};

mod battle;

pub use battle::OpenBattle;
#[cfg(test)]
pub(crate) use battle::seen_by;

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
/// before anything else. Increment 2's are offers (ADV-VIS-001, ADV-VIS-004); since E08 T19a a
/// battle, owned by the players of its sides, who play it.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Decision {
    /// An offer, answered by its owner.
    Offer {
        /// The player who answers it.
        owner: PlayerId,
        /// The offer.
        offer: Offer,
    },
    /// A battle (GAM-BAT-001), shared between forks and copied on its first write.
    Battle(Arc<OpenBattle>),
}

impl Decision {
    /// Whether `player` owns the decision.
    fn owned_by(&self, player: PlayerId) -> bool {
        match self {
            Self::Offer { owner, .. } => *owner == player,
            Self::Battle(battle) => battle.owners().contains(&Some(player)),
        }
    }

    /// The battle, if the decision is one.
    fn battle(&self) -> Option<&Arc<OpenBattle>> {
        match self {
            Self::Battle(battle) => Some(battle),
            Self::Offer { .. } => None,
        }
    }

    /// The battle, if the decision is one, to write.
    fn battle_mut(&mut self) -> Option<&mut Arc<OpenBattle>> {
        match self {
            Self::Battle(battle) => Some(battle),
            Self::Offer { .. } => None,
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
        let (part, heroes) = setup.adventure_setup(rules, &mut draws)?;
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
            decisions: None,
            players,
            groups,
            group: 0,
            day: 1,
            rng: Rng::for_stream(seed, StreamId::GAME),
            heroes: Arc::new(heroes.into_iter().map(Arc::new).collect()),
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
        if self.has_decision(player) {
            return Err(GameCommandError::DecisionOpen);
        }
        Ok(())
    }

    /// Opens the offer a command of `player` made (ADR-0017 point 10): as a pending decision of
    /// the player if its treasury can pay the price (GAM-DEC-001; a border guard's has none, so it
    /// always opens, ADV-VIS-004), told to it alone; otherwise the offer does not open, and the
    /// player alone is told so (ADV-VIS-001: the original's message).
    // rule: GAM-DEC-001, ADV-VIS-001, ADV-VIS-004
    pub(crate) fn open<S: Sink>(&mut self, player: PlayerId, offer: Offer, sink: &mut S) {
        let (hero, object_type, anchor) = (offer.hero, offer.object_type, offer.anchor);
        if self.treasuries.can_pay(player, offer.price()) {
            self.push_decision(Decision::Offer {
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
    /// the offer must still stand ([`AdventureState::offer_stands`], ADR-0017 point 10: a border
    /// guard another player of the turn group passed has left the map, E08 T15a); then the
    /// treasury pays the price and the adventure carries the offer out
    /// ([`AdventureState::accept_offer`]: a Cartographer's reveal and purchase, ADV-VIS-001; a
    /// guard's removal, ADV-VIS-004). Declined, nothing else changes. ADR-0017 point 10's other
    /// check, that the treasury can pay, cannot fail and is not made: the decision opened only
    /// when the treasury could pay, and while it is open its owner's only legal commands are its
    /// answers and no other player's command changes the owner's treasury (ownership isolation;
    /// `h3-map`'s play properties check it at every step).
    ///
    /// # Errors
    ///
    /// `UnknownPlayer`, then `NoDecision`, then `BattleOpen` when the first is a battle (E08
    /// T19a), then `OfferGone` for an acceptance; the state is then unchanged.
    // rule: GAM-DEC-001, ADV-VIS-004
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
        let Decision::Offer { offer, .. } = &self.decisions.as_deref().unwrap_or_default()[at]
        else {
            return Err(GameCommandError::BattleOpen);
        };
        let offer = offer.clone();
        let accepted = answer == Answer::Accept;
        if accepted && !self.adventure.offer_stands(&offer) {
            return Err(GameCommandError::OfferGone);
        }
        self.close(at);
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
            self.adventure.accept_offer(
                rules.core(),
                rules.adventure(),
                player,
                &offer,
                &mut AdventureSink(sink),
            );
        }
        Ok(())
    }

    /// The offer of `player`'s first open decision, if it has one and it is an offer
    /// (GAM-DEC-001): what its owner sees of it ([`crate::GameObservation::decision`]). It may no
    /// longer stand ([`AdventureState::offer_stands`]): then only declining it is legal. A battle
    /// is [`Self::battle`].
    pub fn decision(&self, player: PlayerId) -> Option<&Offer> {
        match &self.decisions.as_deref().unwrap_or_default()[self.first(player)?] {
            Decision::Offer { offer, .. } => Some(offer),
            Decision::Battle(_) => None,
        }
    }

    /// Whether `player` has an open decision, an offer or a battle (GAM-DEC-001).
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
    /// Encoding v6 (E08 T19a; v1 was T6b's, v2 T13's, v3 T14b's, v4 T15a's, v5 T18b's): version;
    /// the master seed; the day; the running group's index; the schedule (groups as length +
    /// player ids); the players (colour, acting, stream state); since v2 the treasuries (the
    /// resource kinds, then every player's amounts as length and entries); since v3 the open
    /// decisions in opening order, as length and entries: since v6 first the decision's kind, 0
    /// an offer, 1 a battle; an offer's owner, hero, object type, anchor, since v4 the offer's
    /// kind, 0 a reveal, 1 a passage, then a reveal's price as length and resource–amount pairs;
    /// a battle's owners and heroes by side (each 0 for none, else 1 and the id), tile and battle
    /// state hash; the game's stream; since v5 the store of heroes (length, then each hero's
    /// details by id, [`hash_details`]); the adventure state's hash.
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
                Decision::Offer { owner, offer } => {
                    h.write_u8(0);
                    h.write_u8(owner.raw());
                    h.write_u16(offer.hero.raw());
                    offer.object_type.hash_into(h);
                    offer.anchor.hash_into(h);
                    match &offer.kind {
                        OfferKind::Reveal { price } => {
                            h.write_u8(0);
                            h.write_len(price.len());
                            for (resource, amount) in price {
                                resource.hash_into(h);
                                h.write_u32(*amount);
                            }
                        }
                        OfferKind::Pass => h.write_u8(1),
                    }
                }
                Decision::Battle(battle) => {
                    h.write_u8(1);
                    battle.hash_into(h);
                }
            }
        }
        self.rng.hash_into(h);
        h.write_len(self.heroes.len());
        for details in self.heroes.iter() {
            hash_details(details, h);
        }
        h.write_u64(self.adventure.state_hash());
    }
}

/// A hero's details in the game state's hash (v5): hero type, level, experience, the primary
/// skills, the secondary skills (length, then skill and level), the worn artifacts and the
/// backpack (each length, then artifact and spell, 0 for none, else 1 and the spell), the
/// learned spells (length, then spell), the spell points.
fn hash_details(details: &HeroDetails, h: &mut StateHasher) {
    details.hero_type.hash_into(h);
    h.write_u8(details.level);
    h.write_u32(details.experience);
    h.write_bytes(&details.primary);
    h.write_len(details.skills.len());
    for (skill, level) in &details.skills {
        skill.hash_into(h);
        h.write_u8(*level);
    }
    for artifacts in [&details.worn, &details.backpack] {
        h.write_len(artifacts.len());
        for ArtifactInstance { artifact, spell } in artifacts {
            artifact.hash_into(h);
            match spell {
                None => h.write_u8(0),
                Some(spell) => {
                    h.write_u8(1);
                    spell.hash_into(h);
                }
            }
        }
    }
    h.write_len(details.spells.len());
    for spell in &details.spells {
        spell.hash_into(h);
    }
    h.write_u16(details.mana);
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

    /// A player's stream as it stands.
    pub(crate) fn stream(&self, player: PlayerId) -> Rng {
        self.players[player.index()].rng.clone()
    }
}
