//! The adventure context's commands, their errors, outcomes and events (ADR-0017 point 1:
//! `h3-game` checks turn and phase, then passes a player's command here with the player as actor;
//! E08 T7b: `MoveHero`; T13b: its visits and their outcome, ADR-0017 point 10; T14b: the
//! Cartographer's offer; T15a: the observatories, the keys and the border guard's offer; T15b: the
//! border gates; T19a: the visit of another player's hero, which starts a battle). A
//! rejected command changes nothing and emits nothing. An event carries the players who may see
//! it, resolved when it is emitted (ADR-0016 point 6; E08 T8a).

use alloc::vec::Vec;
use core::fmt;

use h3_content::{ContentKey, CreatureTypeId, Formation, ResourceId, TerrainId};
use h3_contracts::{ArmyStack, HeroId, PlayerId};

use crate::ruleset::ObjectTypeId;
use crate::setup::{Pos, StandError};

/// A command to the adventure context, issued by a player.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdventureCommand {
    /// Moves one of the player's heroes along `path` today (ADV-MOV-003 to ADV-MOV-007): each
    /// tile a neighbour of the one before (the first of the hero's), each step paid from the
    /// points left. The path is validated step by step and rejected whole: a step the hero
    /// cannot pay is a rejection, not the original's stop that wastes the rest (survey §1.3).
    /// Its last step may visit an object (ADV-OBJ-003): a step toward a pickup's, a border
    /// guard's or a border gate's visitable tile visits it from the tile before, which the hero
    /// does not leave (E08 T13b, T15a, T15b); a step onto a Cartographer's, an observatory's or a
    /// Keymaster's tent's enters it (T14a, T15a). A border gate whose key the player holds is
    /// crossed as any tile, anywhere on the path (ADV-VIS-005, T15b). A step toward another
    /// player's hero visits it from the tile before, as a neighbour-visited object, and starts a
    /// battle (GAM-BAT-004, E08 T19a); an own hero's tile blocks.
    MoveHero {
        /// The hero.
        hero: HeroId,
        /// The tiles it steps onto, in order, without the one it stands on.
        path: Vec<Pos>,
    },
}

/// Why an adventure command was rejected; the state is unchanged. `step` counts a path's steps
/// from 0.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdventureCommandError {
    /// The hero is not on the map or not the player's (one error for both, so a command tells
    /// nothing about other players' heroes).
    NotYourHero(HeroId),
    /// The path has no step.
    EmptyPath,
    /// A step's tile is not one of the eight around the tile before it on its level
    /// (ADV-MOV-007).
    NotNeighbour {
        /// The step.
        step: usize,
    },
    /// A step's tile is on the map but the player has not explored it (ADV-MOV-008): checked
    /// before anything about the tile, so the error tells nothing about a hidden tile.
    Unexplored {
        /// The step.
        step: usize,
    },
    /// A step's tile may not be stood on (ADV-MOV-007, ADV-MAP-001).
    Blocked {
        /// The step.
        step: usize,
        /// Why.
        error: StandError,
    },
    /// A step's tile is one an object is visited from whose visit is not built: a blocker's (a
    /// class without a type, ADR-0016 point 4; every typed kind's visit is built since E08 T15b).
    Visit {
        /// The step.
        step: usize,
    },
    /// A step goes down onto the visitable tile of an object not visited from above, or up off
    /// such a tile (ADV-OBJ-004).
    FromAbove {
        /// The step.
        step: usize,
    },
    /// A step follows a visit: a visit ends the move (ADV-OBJ-003).
    PastVisit {
        /// The step after the visit.
        step: usize,
    },
    /// A flying or water-walking hero's path ends on a tile it only passes over (blocked, water,
    /// an object's tile flown over, a tile entered by a step up off the visitable tile of an
    /// object not visited from above), or a visit is made from such a tile (ADV-MOV-009,
    /// ADV-MOV-010; E08 T41): a move ends only where the hero may stop (D-25).
    NoStop {
        /// The step.
        step: usize,
    },
    /// A step costs more than the points left before it (ADV-MOV-006).
    TooFewPoints {
        /// The step.
        step: usize,
        /// Its price.
        cost: u32,
        /// The points left before it.
        points: u32,
    },
}

impl fmt::Display for AdventureCommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotYourHero(hero) => write!(f, "hero {} is not the player's", hero.raw()),
            Self::EmptyPath => f.write_str("the path has no step"),
            Self::NotNeighbour { step } => {
                write!(f, "step {step} does not go to a neighbouring tile")
            }
            Self::Unexplored { step } => write!(f, "step {step} goes to an unexplored tile"),
            Self::Blocked { step, error } => write!(f, "step {step}: {error}"),
            Self::Visit { step } => {
                write!(f, "step {step} goes to a tile no visit is built for")
            }
            Self::FromAbove { step } => {
                write!(
                    f,
                    "step {step} crosses the top of an object's visitable tile"
                )
            }
            Self::PastVisit { step } => write!(f, "step {step} follows a visit"),
            Self::NoStop { step } => write!(f, "step {step} lands where the hero may not stop"),
            Self::TooFewPoints { step, cost, points } => {
                write!(f, "step {step} costs {cost} with {points} points left")
            }
        }
    }
}

impl core::error::Error for AdventureCommandError {}

/// What an accepted command gives its player beyond the map (ADR-0017 point 10): the resources it
/// gains, which the game adds to the player's treasury (the context never sees a treasury), and
/// the offer it opens, which the game opens as a pending decision if the player can pay its price
/// (E08 T14b; a border guard's has none, T15a).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AdventureOutcome {
    /// Each resource gained with its amount, in the order the rule gives them (a campfire's
    /// resource, then its gold).
    pub gains: Vec<(ResourceId, u32)>,
    /// The offer the command's visit makes its player, if any: a Cartographer's whose type the
    /// player has not bought (ADV-VIS-001), a border guard's whose key the player holds
    /// (ADV-VIS-004).
    pub offer: Option<Offer>,
    /// The battle the command's visit starts, if any: another player's hero's (GAM-BAT-004, E08
    /// T19a), which the game opens at once (ADR-0017 point 11).
    pub battle: Option<BattleStart>,
    /// Whether the visit of a monster guarding the hero's tile waits for the answer to
    /// [`Self::offer`] (ADV-MON-006, E08 T20b1): the step entered a guarded tile whose own object
    /// made the offer. The original asks the object's question first and tests the guard after
    /// it; a player has at most one open decision (ADR-0017 point 6), so the game asks
    /// [`crate::AdventureState::visit_guard`] once the offer is answered, or when it does not
    /// open.
    pub guard_after: bool,
}

/// A battle a visit starts (ADR-0017 points 10 and 11; GAM-BAT-001, E08 T19a): its two sides as
/// the adventure map has them and where it is fought, from which the game builds the battle's
/// setup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BattleStart {
    /// The attacking side: the visiting hero's.
    pub attacker: Combatant,
    /// The defending side: another player's hero, a wandering monster (E08 T20b2) or a guarded
    /// pile's guards (E08 T21a).
    pub defender: Defender,
    /// The tile the battle is fought at: the defender's (survey `E04c-battlefields.md` §4,
    /// batch 3b), a monster's its own, a guarded pile's the pile's (the visited tile, 0x4A4886).
    pub tile: Pos,
    /// The battle's terrain by its tile ([`crate::BattleTerrains`]): the underground's, else the
    /// coast's, else the tile's own.
    pub terrain: TerrainId,
    /// The special battlefield the tile gives, if any: the coast's.
    pub battlefield: Option<ContentKey>,
}

/// The defending side of a battle on the map (ADR-0017 point 10).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Defender {
    /// Another player's hero (GAM-BAT-004, E08 T19a).
    Hero(Combatant),
    /// A wandering monster (ADV-MON-009, E08 T20b2), unowned: its creature and its count as it
    /// stands, unsplit; the game splits it ([`crate::split`]) with the draws its profile gives
    /// (`ADV-QUIRK-6`).
    Monster {
        /// The creature.
        creature: CreatureTypeId,
        /// The count.
        count: u32,
    },
    /// A guarded pile's guards (ADV-PIK-004, E08 T21a), unowned: their stacks by slot as they
    /// stand, which defend unsplit (the original passes the map's army, 0x4A4876).
    Guards {
        /// The stacks by slot.
        army: Vec<ArmyStack>,
    },
}

/// A side of a battle on the map as the adventure has it: a hero on the map, its owner, its army
/// by slot and the army's formation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Combatant {
    /// The hero.
    pub hero: HeroId,
    /// The player owning it.
    pub owner: PlayerId,
    /// Its army, by slot.
    pub army: Vec<ArmyStack>,
    /// The army's formation.
    pub formation: Formation,
}

/// An offer a visit makes to its visitor's owner, answered yes or no (ADR-0017 points 6 and 10;
/// E08 T14b, T15a). The object is named as the observation names it, by type and anchor, never by
/// index (ADR-0016 point 6), so the offer is also what its owner sees of it. Only the context
/// makes offers; [`crate::AdventureState::accept_offer`] trusts the one it is given, once
/// [`crate::AdventureState::offer_stands`] says it still stands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Offer {
    /// The hero whose visit made it.
    pub hero: HeroId,
    /// The object's type.
    pub object_type: ObjectTypeId,
    /// The object's anchor (the bottom right tile its masks are relative to).
    pub anchor: Pos,
    /// What is on offer.
    pub kind: OfferKind,
}

impl Offer {
    /// The price, by resource in the core's order: a Cartographer's, none for a border guard's.
    pub fn price(&self) -> &[(ResourceId, u32)] {
        match &self.kind {
            OfferKind::Reveal { price } | OfferKind::JoinForGold { price } => price,
            OfferKind::Pass | OfferKind::Join | OfferKind::Pursue | OfferKind::Fight => &[],
        }
    }
}

/// What an offer offers (ADR-0017 point 6: increment 2's two kinds of decision, increment 3's
/// wandering monsters' three and a guarded pile's question).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OfferKind {
    /// A Cartographer's reveal of its type's terrains, for the type's price, each amount at least
    /// 1 (ADV-VIS-001, E08 T14b).
    Reveal {
        /// The price, by resource in the core's order.
        price: Vec<(ResourceId, u32)>,
    },
    /// A border guard's passage, without a price: accepted, the guard leaves the map for everyone
    /// (ADV-VIS-004, E08 T15a; D-7 in ADV-Q10).
    Pass,
    /// A wandering monster's offer to join the hero's army for free (ADV-MON-008, E08 T20b1):
    /// accepted, the group joins as one stack, its treasure's resources go to the player and it
    /// leaves the map; declined, its answer goes on (ADV-MON-007: a fight, or its offer to flee).
    Join,
    /// A wandering monster's offer to join for a price (ADV-MON-008, E08 T20b1): its creature's
    /// cost in the monsters' price resource times its count (empty for a creature that costs
    /// none of it); otherwise as [`Self::Join`]. Made only when the player can pay: else its
    /// answer goes on as after a decline.
    JoinForGold {
        /// The price, by resource.
        price: Vec<(ResourceId, u32)>,
    },
    /// A wandering monster's offer to flee (ADV-MON-008, E08 T20b1): accepted, the hero pursues
    /// and fights it; declined, it leaves the map without its treasure.
    Pursue,
    /// A guarded pile's question (ADV-PIK-004, E08 T21a): accepted, the hero fights its guards;
    /// declined, nothing happens.
    Fight,
}

/// The players who may see an event (ADR-0016 point 6): a set of players, resolved when the event
/// is emitted with the state at hand, so projecting an event needs no state. A game has at most
/// eight players (the ruleset part's bound, [`crate::AdventureParams::max_players`]).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Viewers(u8);

impl Viewers {
    /// No player.
    pub const NONE: Self = Self(0);

    /// This set and `player` (a player id of eight or more is not a game's: it is left out).
    #[must_use]
    pub const fn with(self, player: PlayerId) -> Self {
        Self(self.0 | bit(player))
    }

    /// Whether `player` may see the event.
    pub const fn contains(self, player: PlayerId) -> bool {
        self.0 & bit(player) != 0
    }

    /// The set as bits, bit `n` for the player with id `n`.
    pub const fn bits(self) -> u8 {
        self.0
    }

    /// The players in this set or in `other`.
    #[must_use]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    /// The players in both this set and `other`.
    #[must_use]
    pub const fn intersection(self, other: Self) -> Self {
        Self(self.0 & other.0)
    }
}

/// The bit of `player` in [`Viewers`], in the players who bought a type (E08 T14b) and in those
/// who hold a key (T15a), none for an id of eight or more.
pub(crate) const fn bit(player: PlayerId) -> u8 {
    match 1u8.checked_shl(player.raw() as u32) {
        Some(bit) => bit,
        None => 0,
    }
}

/// Something that happened on the adventure map, with the players who may see it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AdventureEvent {
    /// What happened.
    pub kind: AdventureEventKind,
    /// Who may see it.
    pub seen_by: Viewers,
}

/// What happened on the adventure map.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdventureEventKind {
    /// A hero made one step of a move (one event per step: the fog changes during a move, so a
    /// viewer learns of the steps it can see, ADR-0016 point 6). Seen by the players that see the
    /// hero on the tile it leaves or on the tile it enters: its owner, every player that has
    /// explored the tile, and under `ADV-QUIRK-1` every player with the tile on its fringe
    /// (ADV-FOG-004, ADV-FOG-005). A player that sees the hero on one of the two tiles only is
    /// told that one only: the other is `None` in its projection (the hero comes into sight, or
    /// goes out of it).
    HeroStepped {
        /// The hero.
        hero: HeroId,
        /// The tile it left (always given when emitted; `None` in a projection for a viewer that
        /// does not see the hero there).
        from: Option<Pos>,
        /// The tile it entered (likewise).
        to: Option<Pos>,
        /// The players that see the hero on the tile it left (the viewer alone, or none, in a
        /// projection).
        seen_from: Viewers,
        /// The players that see the hero on the tile it entered (likewise).
        seen_to: Viewers,
    },
    /// An object left the map (ADV-OBJ-005; a pickup in E08 T13b, an accepted border guard in
    /// T15a). Seen by the players that see
    /// it there, as the observation shows it: those that have explored a tile it covers and,
    /// under `ADV-QUIRK-1`, those with such a tile on their fringe. Named as the observation
    /// names it, by type and anchor, never by index (ADR-0016 point 6).
    ObjectRemoved {
        /// Its object type.
        object_type: ObjectTypeId,
        /// The bottom right tile its masks are relative to.
        anchor: Pos,
    },
    /// Tiles became explored for a player by an object's reveal (ADV-VIS-001: an accepted
    /// Cartographer's, E08 T14b; ADV-VIS-002: an observatory's at every visit, T15a). Seen by that
    /// player alone, as its explored tiles are.
    TilesRevealed {
        /// The revealing object's type: for a Cartographer, the kind its player now has bought.
        object_type: ObjectTypeId,
        /// How many tiles the player had not explored before.
        count: u32,
    },
    /// A player got the key of a colour by a Keymaster's tent's visit (ADV-VIS-003, E08 T15a);
    /// not told when it already held it. Seen by that player alone, as its keys are.
    KeyGained {
        /// The key's colour, below [`crate::KEY_COLOURS`].
        key: u8,
    },
    /// A hero left the map by a battle's outcome (GAM-BAT-003, E08 T19a). Seen by its owner and the
    /// players that saw it there (ADV-FOG-004, ADV-FOG-005, as a step's).
    HeroRemoved {
        /// The hero.
        hero: HeroId,
        /// The tile it stood on.
        pos: Pos,
    },
    /// A wandering monster's group joined a hero's army (ADV-MON-008, E08 T20b1): its stack of
    /// the creature, else its first free slot. Seen by the hero's owner, as its army is; the
    /// monster's leaving the map follows ([`Self::ObjectRemoved`]).
    CreaturesJoined {
        /// The hero.
        hero: HeroId,
        /// The creature.
        creature: CreatureTypeId,
        /// How many joined.
        count: u32,
    },
    /// The word of a wandering monster's count that a viewer sees changed (ADV-FOG-006,
    /// ADV-MON-010; E08 T20b3b1): a battle it survived left it fewer creatures. Seen by the
    /// players that see the monster, as its removal would be ([`Self::ObjectRemoved`]); not told
    /// when the word stays.
    MonsterChanged {
        /// The monster's type.
        object_type: ObjectTypeId,
        /// Its anchor.
        anchor: Pos,
        /// The word of its count now ([`crate::MonsterParams::count_word`]).
        count_word: u8,
    },
}

/// Receiver of the adventure context's events.
pub trait Sink {
    /// Receives one event.
    fn emit(&mut self, event: AdventureEvent);
}

impl Sink for Vec<AdventureEvent> {
    fn emit(&mut self, event: AdventureEvent) {
        self.push(event);
    }
}

#[cfg(test)]
mod tests;
