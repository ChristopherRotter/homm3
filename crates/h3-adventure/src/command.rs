//! The adventure context's commands, their errors, outcomes and events (ADR-0017 point 1:
//! `h3-game` checks turn and phase, then passes a player's command here with the player as actor;
//! E08 T7b: `MoveHero`; T13b: its visits and their outcome, ADR-0017 point 10; T14b: the
//! Cartographer's offer; T15a: the observatories, the keys and the border guard's offer; T15b: the
//! border gates; T19a: the visit of another player's hero, which starts a battle). A
//! rejected command changes nothing and emits nothing. An event carries the players who may see
//! it, resolved when it is emitted (ADR-0016 point 6; E08 T8a).

use alloc::vec::Vec;
use core::fmt;

use h3_content::{ContentKey, CreatureTypeId, Formation, ResourceId, SkillId, TerrainId};
use h3_contracts::{ArmyStack, HeroId, PlayerId, TownId};
use h3_heroes::{HeroChange, PrimarySkill};

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
    /// hero visits it from the tile before, as a neighbour-visited object: another player's
    /// starts a battle (GAM-BAT-004, E08 T19a), an own one makes a meeting (GAM-MET-001, T50).
    MoveHero {
        /// The hero.
        hero: HeroId,
        /// The tiles it steps onto, in order, without the one it stands on.
        path: Vec<Pos>,
    },
    /// Builds a boat at one of the player's or an ally's Shipyards without a visit (ADV-SHY-003,
    /// E08 T84b2; increment 9's design note §8; survey `E08-economy.md` §5.3: the original's
    /// click on an own Shipyard): an empty boat of the Shipyard's type on its boat tile, for the
    /// kind's price, which the game takes (it checks the treasury first). No hero, no movement
    /// points, no visit. The Shipyard is named as the observation names it, by type and anchor.
    BuildBoat {
        /// The Shipyard's type.
        object_type: ObjectTypeId,
        /// Its anchor.
        anchor: Pos,
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
    /// A step onto an empty boat's tile from a boat, or from water by a water walk (ADV-SEA-002,
    /// ADV-SEA-004; E08 T73): a boat is boarded only from a stop node.
    Boat {
        /// The step.
        step: usize,
    },
    /// A boat's step onto land that is no free coastal tile (ADV-SEA-003, D-42; E08 T73): an
    /// object's, an event's, a hole's or a hero's tile, or a tile off the coast.
    Landing {
        /// The step.
        step: usize,
    },
    /// A boat's diagonal step between two water tiles past a side tile that is not water
    /// (ADV-SEA-004; E08 T73).
    Coast {
        /// The step.
        step: usize,
    },
    /// `BuildBoat` names no Shipyard at that anchor with that type, or one with no tile the player
    /// has explored (ADV-SHY-003, E08 T84b2): one error for both, so a command reveals no hidden
    /// Shipyard.
    UnknownObject,
    /// `BuildBoat`'s Shipyard is neither the player's nor an ally's (ADV-SHY-003).
    NotOwner,
    /// `BuildBoat`'s Shipyard has no boat tile (ADV-SHY-001: none was free water when the game was
    /// built), so it never builds (ADV-SHY-003).
    NoBoatTile,
    /// `BuildBoat`'s boat tile holds a boat or a hero (ADV-SHY-003).
    BoatTileTaken,
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
            Self::Boat { step } => write!(f, "step {step} goes onto an empty boat from water"),
            Self::Landing { step } => {
                write!(f, "step {step} goes ashore where no boat may land")
            }
            Self::UnknownObject => f.write_str("no Shipyard the player knows of is there"),
            Self::NotOwner => f.write_str("the Shipyard is neither the player's nor an ally's"),
            Self::NoBoatTile => f.write_str("the Shipyard has no boat tile"),
            Self::BoatTileTaken => f.write_str("a boat or a hero is on the Shipyard's boat tile"),
            Self::Coast { step } => {
                write!(f, "step {step} sails diagonally past the coast")
            }
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
    /// Each resource lost with its amount, in the order the rule takes them (a reward record's
    /// negative resources, ADV-RWD-001, E08 T62b1): the game takes them from the player's
    /// treasury, never below 0.
    pub losses: Vec<(ResourceId, u32)>,
    /// The offer the command's visit makes its player, if any: a Cartographer's whose type the
    /// player has not bought (ADV-VIS-001), a border guard's whose key the player holds
    /// (ADV-VIS-004).
    pub offer: Option<Offer>,
    /// The battle the command's visit starts, if any: another player's hero's (GAM-BAT-004, E08
    /// T19a), which the game opens at once (ADR-0017 point 11).
    pub battle: Option<BattleStart>,
    /// The hero whose tile's entered object is visited after the visit of the monster guarding
    /// the tile (ADV-MON-006, GAM-DEC-007; E08 T103d, decision 279): the step entered a guarded
    /// tile with an object visited from it, and the outcome is the guard's visit (its offer or
    /// its battle). The original's step tests the guard (`hero_move_step`, 0x48169D) and the walk
    /// visits the tile's object after it (`hero_move_loop`, 0x4080E9), once the guard's answer,
    /// battle and levels are over, for a hero still on the tile (batch E11's E11-19, E11c's
    /// E11-21 and E11-22: a mine flagged and a Learning Stone's experience after the won battle).
    /// The game holds it as a rest of the visit and asks [`crate::AdventureState::visit_held`]
    /// once the player has no open decision (until then the hero cannot move). From E08 T20b1 to
    /// T103d the object was visited first and the guard after it (`guard_after`).
    pub visit_after: Option<HeroId>,
    /// The meeting the command's step toward an own hero makes (GAM-MET-001, E08 T50): the game
    /// opens it as the owner's decision (T51b).
    pub meeting: Option<Meeting>,
    /// The group that found no room in its hero's army, which the game opens the army dialog with
    /// (GAM-DEC-004, E08 T51b): an accepted join's (ADV-MON-008, E08 T50), its treasure given and
    /// the monster gone as for any join; since E08 T62b1 a reward record's stacks that found none
    /// (ADV-RWD-001).
    pub no_room: Option<NoRoom>,
    /// The Leave Guards dialog an own mine's visit opens (ADV-MIN-002, GAM-DEC-004 amended; E08
    /// T84a): the game opens the army dialog between the hero's army and the mine's guards, whose
    /// group its owner's `Close` keeps on the mine ([`crate::AdventureState::set_guards`]).
    pub garrison: Option<Garrison>,
    /// What the visit gives its hero (ADR-0017 point 10, increment 7; E08 T62a): the artifacts,
    /// spells, skills, primary skills, spell points, bonuses and experience of a pickup, a
    /// teacher or a find, checked against the details the visit read; the game applies them.
    pub hero: Option<HeroGains>,
    /// The choice the visit opens for the player (E08 T62a): a treasure chest's gold or
    /// experience (ADV-PIK-005), the Arena's or a School's primary skill (ADV-TCH-001,
    /// ADV-TCH-002). The game opens it as the player's decision when its treasury can pay the
    /// price, and gives the answer to [`crate::AdventureState::choose`].
    pub choice: Option<Choice>,
    /// The University the visit opens (ADV-TCH-007, E08 T62a): the game opens it as the player's
    /// decision, its purchases made through `h3_heroes::university_slot` (GAM-DEC-006).
    pub university: Option<UniversityVisit>,
    /// The hire dialog an accepted hire question opens (ADV-HIR-002, E08 T95b): at the visitor's
    /// own dwelling with a creature above the part's `free_level`, after its free creatures, and
    /// at a Refugee Camp. The game opens it as the player's decision (GAM-DEC-008), checks each
    /// purchase with [`crate::AdventureState::check_hire`], pays its price and makes it with
    /// [`crate::AdventureState::hire`].
    pub hire: Option<HireDialog>,
    /// The steps of the move's path not taken (ADV-MOV-011, E08 T62b2): an event fired on the
    /// tile of an earlier step and ended the move there; 0 for every other outcome.
    pub dropped: usize,
    /// The hero and the tile of a won battle against an object's guards whose reward comes after
    /// the winner's levels (GAM-DEC-007, E08 T63b; survey `E08-rewards.md` §8.1, the original's
    /// visit after 0x4DA720): a guarded artifact's, scroll's or pile's pickup, a Pandora's Box's
    /// or an event's record, a Pyramid's loot and spell; a mine's flag (E08 T84a), a dwelling's
    /// visit (E08 T95a2); since E09 T8b1 a town's capture after a siege won against its garrison
    /// (TWN-CAP-003). The game holds it as a rest of the visit and gives it with
    /// [`crate::AdventureState::give_won`] once the player has no open decision.
    pub reward_after: Option<(HeroId, Pos)>,
    /// The hero and the tile of an event that fires after the visit of the monster guarding its
    /// tile (ADV-OBJ-007, `ADV-QUIRK-29`; E08 T63b): the step onto it ended the move, the guard's
    /// visit is the outcome's. The game holds it as a rest of the visit (GAM-DEC-007) and fires
    /// it with [`crate::AdventureState::fire_held_event`] once the player has no open decision.
    pub event_after: Option<(HeroId, Pos)>,
    /// The town the outcome captures for its player, which the game applies at once (TWN-CAP-001;
    /// E09 T8a2, the town epic's design note §9 and §11): `h3_towns::reveal`, then
    /// [`crate::AdventureState::capture_town`], then the Capitol's limit (TWN-CAP-002): the
    /// undefended town a step entered (TWN-VIS-001); since E09 T8b1 a town whose siege the
    /// player's hero won, given after the winner's levels
    /// ([`crate::AdventureState::give_won`], TWN-CAP-003). The outcome changes no owner itself.
    pub capture: Option<TownId>,
}

/// What a visit gives its hero (ADR-0017 point 10, increment 7; E08 T62a): each change passes
/// `h3_heroes::check_change` once the earlier ones are applied, against the details the visit
/// read; then the experience, which the game gives last as `HeroChange::GainExperience` under the
/// scenario's level limit, with the levels it brings (GAM-DEC-002).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeroGains {
    /// The hero.
    pub hero: HeroId,
    /// The changes, in order.
    pub changes: Vec<HeroChange>,
    /// The experience, 0 for none.
    pub experience: u32,
}

impl HeroGains {
    /// These changes to `hero`, without experience.
    pub(crate) fn changes(hero: HeroId, changes: Vec<HeroChange>) -> Self {
        Self {
            hero,
            changes,
            experience: 0,
        }
    }

    /// This experience to `hero`, without changes.
    pub(crate) const fn experience(hero: HeroId, experience: u32) -> Self {
        Self {
            hero,
            changes: Vec::new(),
            experience,
        }
    }
}

/// A choice a visit opens for its hero's owner (ADR-0017 point 6, increment 7; E08 T62a), named
/// by the object's type and anchor as an offer names it ([`Offer`]); the answer is a [`Pick`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Choice {
    /// The hero whose visit opened it.
    pub hero: HeroId,
    /// The object's type.
    pub object_type: ObjectTypeId,
    /// The object's anchor.
    pub anchor: Pos,
    /// What is chosen between.
    pub kind: ChoiceKind,
}

impl Choice {
    /// The price the first or second pick costs, by resource: a School's, none for the others.
    pub fn price(&self) -> &[(ResourceId, u32)] {
        match &self.kind {
            ChoiceKind::Chest { .. } => &[],
            ChoiceKind::Primary { price, .. } => price,
        }
    }

    /// Whether `pick` answers the choice: the first and the second always, leaving only a
    /// choice that may be left (a School's; the chest's and the Arena's may not).
    pub const fn allows(&self, pick: Pick) -> bool {
        match (&self.kind, pick) {
            (_, Pick::First | Pick::Second) => true,
            (ChoiceKind::Primary { leave, .. }, Pick::Leave) => *leave,
            (ChoiceKind::Chest { .. }, Pick::Leave) => false,
        }
    }

    /// The pick the system actor gives (ADR-0017 point 6's defaults): leaving where the choice
    /// may be left (a School), else the first (the chest's gold, the original's plain OK; the
    /// Arena's first skill, its Cancel).
    pub const fn default_pick(&self) -> Pick {
        if self.allows(Pick::Leave) {
            Pick::Leave
        } else {
            Pick::First
        }
    }
}

/// What a [`Choice`] chooses between.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChoiceKind {
    /// A treasure chest's (ADV-PIK-005): first its gold to the player, second the experience to
    /// the hero (the gold less the type's offset, times the hero's Learning). The chest left the
    /// map with the visit.
    Chest {
        /// The gold: its resource and amount.
        gold: (ResourceId, u32),
        /// The experience.
        experience: u32,
    },
    /// A once-per-hero gift's (ADV-TCH-001: the Arena; ADV-TCH-002: the Schools): `amount` added
    /// to the first or the second skill for the price, which marks the hero; with `leave`, the
    /// player may take neither, without paying or a mark.
    Primary {
        /// The two skills, in the dialog's order.
        skills: [PrimarySkill; 2],
        /// The gain.
        amount: u8,
        /// The price, by resource (empty: free).
        price: Vec<(ResourceId, u32)>,
        /// Whether the player may leave the choice.
        leave: bool,
    },
}

/// An answer to a [`Choice`] (E08 T62a): its first or second option, or leaving without either.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pick {
    /// The first option.
    First,
    /// The second option.
    Second,
    /// Neither (only a choice that [`Choice::allows`] it).
    Leave,
}

/// A University a visit opens (ADV-TCH-007, E08 T62a): its skills in the draw's order, each sold
/// at the price for the hero to learn at Basic while its slot is open
/// (`h3_heroes::university_slot`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UniversityVisit {
    /// The hero whose visit opened it.
    pub hero: HeroId,
    /// The object's type.
    pub object_type: ObjectTypeId,
    /// The object's anchor.
    pub anchor: Pos,
    /// The skills.
    pub skills: Vec<SkillId>,
    /// The price of each, by resource.
    pub price: Vec<(ResourceId, u32)>,
}

/// The hire dialog an accepted hire question opens (ADV-HIR-002; E08 T95b, increment 10's design
/// note §8, §13; survey `E08-dwellings.md` §5, the original's `H3RecruitMgr`): the army the
/// creatures join and the source they are bought from, by name only. Its creatures and counts are
/// the state's, read at each view and purchase ([`crate::AdventureState::hire_stock`]); a
/// purchase is [`crate::AdventureState::hire`]. Both halves are enums so E09's town recruitment
/// names a town's dwelling and a garrison through the same dialog (design note §13, "E09's
/// reuse"). *Agent's call:* no read-only flag yet: an ally's dialog (E10's teams, the original
/// disabling every button) is unreachable while `allied` is the player alone, so its branch is
/// left out until then (design note §13).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HireDialog {
    /// The army the bought creatures join.
    pub target: HireTarget,
    /// Where they are bought.
    pub source: HireSource,
}

/// The army a hire dialog's creatures join ([`HireDialog`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HireTarget {
    /// The army of the hero on the source's entrance.
    Hero(HeroId),
}

/// Where a hire dialog's creatures are bought ([`HireDialog`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HireSource {
    /// An object on the map: a dwelling (its kind's creatures and their counts) or a Refugee Camp
    /// (its week's creature and count), named by index, type and anchor.
    Object {
        /// The object's index.
        index: usize,
        /// The object's type.
        object_type: ObjectTypeId,
        /// The object's anchor.
        anchor: Pos,
    },
}

/// Why a purchase in a hire dialog was refused ([`crate::AdventureState::check_hire`], in this
/// order; ADV-HIR-002, ADV-ARM-007); nothing changed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HireError {
    /// The dialog's hero is not on the map, or its source is not the dwelling or Refugee Camp it
    /// names.
    NoSource,
    /// The creature is not one of the source's.
    NotOffered(CreatureTypeId),
    /// The amount is 0 or above the source's count of the creature.
    Amount(u32),
    /// The army has no room for the amount (ADV-ARM-007: no stack of the creature takes it
    /// within a stack's most, and no slot is free; the original's GENRLTXT 426).
    NoRoom,
}

impl fmt::Display for HireError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoSource => f.write_str("the hire dialog's hero or source is not on the map"),
            Self::NotOffered(creature) => {
                write!(f, "creature {} is not the source's", creature.raw())
            }
            Self::Amount(amount) => write!(f, "{amount} is not an amount the source holds"),
            Self::NoRoom => f.write_str("the army has no room for the creatures"),
        }
    }
}

impl core::error::Error for HireError {}

/// Two heroes of one player met by a step (GAM-MET-001, E08 T50): the visitor's move ended on the
/// tile beside the visited hero, at the step's price (ADR-0016 point 4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Meeting {
    /// The hero whose move made the visit.
    pub visitor: HeroId,
    /// The hero it visited.
    pub visited: HeroId,
}

/// A group its hero's army had no room for (ADV-ARM-007; E08 T50): for each of its stacks, no
/// stack of its creature takes it whole within a stack's most and no slot is free. A joining
/// group's (ADV-MON-008) is one stack in slot 0; a reward record's (ADV-RWD-001, E08 T62b1) its
/// stacks that found no room, at their slots in the record (the original's copy of the record's
/// army, each stack that joined cleared, 0x4A09BB).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoRoom {
    /// The hero it joined.
    pub hero: HeroId,
    /// Its stacks by slot, at least one.
    pub group: Vec<ArmyStack>,
}

/// An own mine's Leave Guards dialog (ADV-MIN-002, GAM-DEC-004 amended; E08 T84a; survey
/// `E08-economy.md` §3.3, the original's `army_dialog_with` 0x5D12B0 over the mine record's
/// army): the visiting hero and the mine, named by its object index for the game's decision and
/// by type and anchor for its owner's view, with its guards as they stand, by slot (none at
/// all is a dialog too: the owner may leave guards there).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Garrison {
    /// The visiting hero, on the mine's entrance.
    pub hero: HeroId,
    /// The mine's object index.
    pub mine: usize,
    /// The mine's type.
    pub object_type: ObjectTypeId,
    /// The mine's anchor.
    pub anchor: Pos,
    /// Its guards as they stand, by slot ([`crate::AdventureState::mine_guards`]).
    pub guards: Vec<ArmyStack>,
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
    /// The battle's terrain by its tile ([`crate::BattleTerrains`]): the sea's when either side's
    /// hero is in a boat (E08 T74b), else the underground's, else the coast's, else the tile's
    /// own.
    pub terrain: TerrainId,
    /// The special battlefield, if any: a special ground's on the tile, else the deck with one
    /// hero in a boat and the two boats with both (E08 T74b), else the coast's
    /// ([`crate::AdventureState::sea_battle_ground`], [`crate::AdventureState::battle_ground`]).
    pub battlefield: Option<ContentKey>,
    /// The battlefield picture's variant, 0 to 2, by the scenery around the tile
    /// ([`crate::AdventureState::background`], GAM-BAT-011; E08 T54): for viewers.
    pub background: u8,
    /// The special battlefield of the special ground under the attacking hero, if any
    /// ([`crate::AdventureState::special_ground`], GAM-BAT-013; E08 T54): the original's computer
    /// values that hero's spells at the mastery of its own tile.
    pub attacker_ground: Option<ContentKey>,
    /// The town of a siege, if the battle is one (E09 T8b1; the town epic's design note §10 and
    /// §15): the battle at a defended town's entrance, `tile`. The game builds the siege's setup
    /// from the town (its facts, the defending hero's bonuses, its terrain; TWN-SIE-003 to -005)
    /// and an attacker's win captures it (TWN-CAP-003). None for every other battle.
    pub town: Option<TownId>,
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
    /// stand, which defend unsplit (the original passes the map's army, 0x4A4876). Since E08
    /// T62b1 a Pandora's Box's too (ADV-PIK-013); since E08 T95a2 a dwelling's, whatever its
    /// owner (ADV-DWL-004; D-51: the original's owner −1).
    Guards {
        /// The stacks by slot.
        army: Vec<ArmyStack>,
    },
    /// Monsters of two creatures (ADV-MON-012, E08 T62b1: the Pyramid's guards), unowned: the
    /// first split as a monster's less the second's stacks, the second in its stacks, the slots
    /// in the original's order; the game splits them ([`crate::split_with_second`]) with the
    /// draws its profile gives (`ADV-QUIRK-6`). Their losses are not kept.
    Monsters {
        /// The first creature and its count.
        split: (CreatureTypeId, u32),
        /// The second creature and its stacks' counts, in order.
        fixed: (CreatureTypeId, Vec<u32>),
    },
    /// An owned mine's guards (GAM-BAT-001 amended, ADV-MIN-002; E08 T84a): an army without a
    /// hero, its side the mine's owner's, who plays it as a hero battle's defender, outside its
    /// turn (the original hands its human owner the guards' side, each stack waiting for its
    /// order: batch E9's E9-2); their stacks by slot as they stand, unsplit.
    Garrison {
        /// The mine's owner.
        owner: PlayerId,
        /// The stacks by slot.
        army: Vec<ArmyStack>,
    },
}

impl Defender {
    /// The player owning the defending side: a hero's owner, an owned mine's (E08 T84a); none for
    /// a monster's or guards' side.
    pub const fn owner(&self) -> Option<PlayerId> {
        match self {
            Self::Hero(combatant) => Some(combatant.owner),
            Self::Garrison { owner, .. } => Some(*owner),
            Self::Monster { .. } | Self::Guards { .. } | Self::Monsters { .. } => None,
        }
    }
}

/// A side of a battle on the map as the adventure has it: a hero on the map, its owner, its army
/// by slot, the army's formation and the hero's tactics flag.
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
    /// The hero's tactics flag (ADV-ARM-006): off, the game denies the tactics phase the side
    /// would get (GAM-BAT-009; E08 T51a).
    pub tactics: bool,
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
            OfferKind::Reveal { price }
            | OfferKind::JoinForGold { price }
            | OfferKind::Gift { price }
            | OfferKind::Boat { price } => price,
            OfferKind::Pass
            | OfferKind::Join
            | OfferKind::Pursue
            | OfferKind::Fight
            | OfferKind::Search
            | OfferKind::OpenBox
            | OfferKind::SearchPyramid
            | OfferKind::Hire { .. }
            | OfferKind::Guarded { .. } => &[],
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
    /// declined, nothing happens. Since E08 T62a a guarded artifact's (ADV-PIK-007) and a guarded
    /// scroll's with a text (ADV-PIK-008) too; since E08 T84a a guarded mine's, neutral or another
    /// player's (ADV-MIN-002: the original's ADVEVENT 84 or 187): accepted, its guards' battle, or
    /// the flag when none stand any more.
    Fight,
    /// A once-per-hero gift's price (ADV-TCH-003, E08 T62a: the Tree of Knowledge's drawn one),
    /// each amount at least 1: accepted, the gift is given and marks the hero; declined, nothing
    /// (no mark).
    Gift {
        /// The price, by resource in the core's order.
        price: Vec<(ResourceId, u32)>,
    },
    /// A find's question (ADV-PIK-011, E08 T62a: the Warrior's Tomb's "search?"): accepted, its
    /// contents and its bonus; declined, nothing.
    Search,
    /// Pandora's Box's question (ADV-PIK-013, E08 T62b1: "open the box?"): accepted, its guards
    /// fight at once, else its reward record is given and it leaves the map; declined, nothing.
    OpenBox,
    /// The Pyramid's question (ADV-TCH-008, E08 T62b1: "search it?", at every visit): accepted,
    /// its owner learns whether it is looted, a looted one costs the hero luck once, else its
    /// guards fight; declined, nothing.
    SearchPyramid,
    /// A Shipyard's boat for its kind's price (ADV-SHY-002, E08 T84b2; the original's boat
    /// dialog): made when its boat tile is free, opened by the game only when the treasury can
    /// pay, standing while the tile stays free; accepted, an empty boat on the tile
    /// ([`crate::AdventureState::accept_offer`]); declined, nothing.
    Boat {
        /// The price, by resource in the core's order.
        price: Vec<(ResourceId, u32)>,
    },
    /// The hire question (ADV-DWL-004, ADV-RFG-001, ADV-HIR-001; E08 T95a1, increment 10's
    /// design note §7, §13: the original's ADVEVENT 35, or 36 for four creatures, and a Refugee
    /// Camp's 112), without a price: a dwelling's, after its flag when it was not the visitor's
    /// team's, and a camp's with creatures left. Accepted, on the visitor's own dwelling, each
    /// creature of the part's `free_level` or less joins free where its army has room
    /// ([`crate::AdventureState::accept_offer`]); then the hire dialog for the rest, and a camp's
    /// ([`AdventureOutcome::hire`], E08 T95b). Declined, nothing (the flag stays).
    Hire {
        /// The creatures it names: the dwelling's, in its kind's order; a camp's week's one.
        creatures: Vec<CreatureTypeId>,
    },
    /// A dwelling's guards' question (ADV-DWL-004; E08 T95a2, increment 10's design note §7 step
    /// 2, §12: the original's GENRLTXT 422), without a price: a neutral dwelling's or another
    /// team's whose guards stand. It names the creature of the guards' first occupied slot and the
    /// word of their total count (VCMI reads the first stack's: a text difference only).
    /// Accepted, the hero fights the guards, or, when none stand any more, the visit goes on (the
    /// flag, then [`Self::Hire`]); declined, nothing (the hero stays on the entrance).
    Guarded {
        /// The creature of the guards' first occupied slot.
        creature: CreatureTypeId,
        /// The word of the guards' total count ([`crate::MonsterParams::count_word`]).
        count_word: u8,
    },
}

/// Why a Shipyard's visit makes no boat offer (ADV-SHY-002, E08 T84b2; survey `E08-economy.md`
/// §5.2, the original's messages).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShipyardBlock {
    /// It has no boat tile (ADV-SHY-001: the original's "the Ways are blocked", GENRLTXT 135).
    NoTile,
    /// A boat or a hero is on its boat tile (the original's "Cannot build another boat",
    /// GENRLTXT 52).
    Taken,
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
    /// monster's leaving the map follows ([`Self::ObjectRemoved`]). Since E08 T95a1 a dwelling's
    /// free creatures too (ADV-HIR-001: the original's GENRLTXT 424), all of its count, which
    /// the dwelling then holds no more of.
    CreaturesJoined {
        /// The hero.
        hero: HeroId,
        /// The creature.
        creature: CreatureTypeId,
        /// How many joined.
        count: u32,
    },
    /// An event fired for a hero that stepped onto its tile (ADV-OBJ-007, ADV-FOG-007; E08 T62b2):
    /// the move ended there, and its guards' battle or its reward record follows. Seen by the
    /// hero's owner alone: no other player is told of an event, nor sees one.
    EventFired {
        /// The hero.
        hero: HeroId,
        /// The event's tile, where the hero stands.
        tile: Pos,
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
    /// A hero boarded an empty boat (ADV-SEA-002, E08 T73; increment 8's design note §12): the boat
    /// is no longer an empty one on the tile, the hero is in it there (its step came before as
    /// [`Self::HeroStepped`]). Seen by the players that see heroes on the tile, as they see the
    /// empty boat there (ADV-FOG-008). Named by the boat's type and tile, never its index (ADR-0016
    /// point 6), owner or captain.
    Boarded {
        /// The hero.
        hero: HeroId,
        /// The boat's type (appearance only).
        boat: u8,
        /// The boat's tile, where the hero stands.
        at: Pos,
    },
    /// A hero in a boat landed (ADV-SEA-003, E08 T73; increment 8's design note §12): its boat lies
    /// empty on the water tile it left, facing the landing's way (its step came before as
    /// [`Self::HeroStepped`], which tells the landing tile to those that see it). Seen by the
    /// players that see heroes on the water tile, as they then see the empty boat there
    /// (ADV-FOG-008); never the boat's index, owner or captain.
    Landed {
        /// The hero.
        hero: HeroId,
        /// The boat's type (appearance only).
        boat: u8,
        /// The water tile the boat lies on now.
        at: Pos,
        /// The boat's facing, in the original's order of eight directions from up, clockwise.
        facing: u8,
    },
    /// A hero read an object's message (the Ocean Bottle, ADV-PIK-017, E08 T74a; increment 8's
    /// design note §12): the map's text, or a default one when it has none, is the client's to
    /// show, found by the object's type and anchor (never its index, ADR-0016 point 6). Seen by the
    /// hero's owner alone; a bottle's leaving follows as [`Self::ObjectRemoved`].
    MessageRead {
        /// The hero.
        hero: HeroId,
        /// The object's type.
        object_type: ObjectTypeId,
        /// Its anchor.
        anchor: Pos,
    },
    /// A boat left the map with its hero (ADV-SEA-006, E08 T74b; increment 8's design note §13): a
    /// battle took a hero in a boat off the map, and its boat with it (the hero's removal came
    /// before as [`Self::HeroRemoved`], at the same tile). Seen by the players that see heroes on
    /// the tile, as they saw the hero in its boat there (ADV-FOG-006, ADV-FOG-008); never the
    /// boat's index, owner or captain.
    BoatRemoved {
        /// The hero.
        hero: HeroId,
        /// The boat's type (appearance only).
        boat: u8,
        /// The water tile it left the map from, its hero's.
        at: Pos,
    },
    /// An ownable object got a new owner (ADV-OWN-001, E08 T84a; increment 9's design note §7): a
    /// visit flagged it, at once or after its guards' battle. Seen by the players that see the
    /// object, as its removal would be ([`Self::ObjectRemoved`]), and by its new owner; named by
    /// type and anchor, never by index (ADR-0016 point 6). Its reveal follows, told to the owner
    /// alone ([`Self::TilesRevealed`]).
    ObjectFlagged {
        /// The object's type.
        object_type: ObjectTypeId,
        /// Its anchor.
        anchor: Pos,
        /// Its new owner.
        owner: PlayerId,
    },
    /// A town got a new owner by its capture (TWN-CAP-001, E09 T8a2; the town epic's design note
    /// §11): an undefended town entered ([`crate::AdventureState::capture_town`]). Seen by the
    /// players that see the town, as a flag ([`Self::ObjectFlagged`]), and by its new owner. Its
    /// reveal follows, told to the owner alone ([`Self::TilesRevealed`]).
    TownCaptured {
        /// The town.
        town: TownId,
        /// Its anchor.
        anchor: Pos,
        /// Its new owner.
        owner: PlayerId,
    },
    /// A mine's guards' hint as every viewer sees it changed (ADV-OWN-003, ADV-MIN-002; E08 T84a):
    /// a battle left fewer or beat them, or its owner's Leave Guards dialog changed them, and the
    /// creature it names or the word of their total count changed, or they are gone
    /// ([`crate::GuardsView`]'s public part). Seen by the players that see the mine, as its flag
    /// would be ([`Self::ObjectFlagged`]); not told when the hint stays (as a monster's word,
    /// [`Self::MonsterChanged`]). Their stacks, the owner's team's knowledge, are told to it by
    /// the dialog's or the battle's own events.
    GuardsChanged {
        /// The mine's type.
        object_type: ObjectTypeId,
        /// Its anchor.
        anchor: Pos,
        /// Their creature when all their stacks are of one; none for several or none at all.
        creature: Option<CreatureTypeId>,
        /// The word of their total count; none when no guards are left.
        count_word: Option<u8>,
    },
    /// A weekly object's visit set visited marks (ADV-VIS-006 amended, E08 T84b1; increment 9's
    /// design note §9): for the visitor's team (a Garden's too since T84b2, `ADV-QUIRK-35`), kept
    /// through the weeks ([`crate::ObjectView::visited`]). Seen by the
    /// players whose mark it set, whose view of the object it changes; not told to a player that
    /// held its mark already. Its gain or [`Self::ObjectEmpty`] follows.
    ObjectMarked {
        /// The object's type.
        object_type: ObjectTypeId,
        /// Its anchor.
        anchor: Pos,
    },
    /// A weekly object's visit found it emptied this week (ADV-WEK-001 to -003, E08 T84b1;
    /// increment 9's design note §9: the original's "nothing" message, the Garden's "already"):
    /// nothing given. Since E08 T95a1 a Refugee Camp's visit with no creature left (ADV-RFG-001:
    /// the original's ADVEVENT 44, "deserted"): no question. Seen by the visitor's owner alone,
    /// as the gain its contents would have been.
    ObjectEmpty {
        /// The object's type.
        object_type: ObjectTypeId,
        /// Its anchor.
        anchor: Pos,
    },
    /// A Shipyard's visit made no boat offer (ADV-SHY-002, E08 T84b2; increment 9's design note
    /// §7: the original's message): it has no boat tile, or a boat or a hero is on it. Seen by the
    /// visitor's owner alone (the tile's content is what the owner sees there, or the static map's).
    ShipyardBlocked {
        /// The Shipyard's type.
        object_type: ObjectTypeId,
        /// Its anchor.
        anchor: Pos,
        /// Why.
        reason: ShipyardBlock,
    },
    /// A Shipyard built a boat (ADV-SHY-002, ADV-SHY-003; E08 T84b2; increment 9's design note §7):
    /// an empty boat of its type now lies on its boat tile, facing 2 as the map's boats. Seen by
    /// its builder and by the players that see heroes on the tile, as they see empty boats there
    /// (ADV-FOG-008); never the boat's index or owner.
    BoatBuilt {
        /// The boat tile.
        tile: Pos,
        /// The boat's type (appearance only).
        boat_type: u8,
    },
    /// An accepted hire question found none of a free creature at its owner's dwelling
    /// (ADV-HIR-001; E08 T95a1, increment 10's design note §7: the original's GENRLTXT 423,
    /// "There are no %s here to recruit."): nothing joins. Seen by the dwelling's owner alone.
    NoneToRecruit {
        /// The dwelling's type.
        object_type: ObjectTypeId,
        /// Its anchor.
        anchor: Pos,
        /// The free creature.
        creature: CreatureTypeId,
    },
    /// A dwelling's free creatures found no room in the hero's army (ADV-HIR-001, ADV-ARM-007; E08
    /// T95a1, increment 10's design note §7: the original's GENRLTXT 426, "not enough
    /// provisions"): none of them joins and the dwelling keeps them (D-52 as the original: the
    /// hero's stacks are not merged to make room). Seen by the hero's owner alone, as its army
    /// is.
    JoinRefused {
        /// The hero.
        hero: HeroId,
        /// The creature.
        creature: CreatureTypeId,
        /// How many would have joined.
        count: u32,
    },
    /// A month of a creature added a wandering monster on a free land tile (ADV-MON-014; E08
    /// T85a; increment 9's design note §11): an object added in play, one tile blocked and visited
    /// from it, seen and visited as a placed monster from then on. Seen by the players that see
    /// it, as a placed monster (ADV-FOG-004: its tile explored, or on the fringe), with what they
    /// see of it ([`crate::MonsterView`]); never its index, exact count or aggression.
    MonsterAdded {
        /// The monster's type (the months' parameters' `monster`).
        object_type: ObjectTypeId,
        /// Its anchor, its one tile.
        anchor: Pos,
        /// The month's creature.
        creature: CreatureTypeId,
        /// The word of its count ([`crate::MonsterParams::count_word`]).
        count_word: u8,
    },
    /// A hero jumped through a teleporter (ADV-TEL-007; E08 T105a1, increment 11's design note
    /// §10): from the entrance it stepped onto (or stood on, for a visit after its guard) to the
    /// exit, without a step. Seen as a step's two ends ([`Self::HeroStepped`]'s fields and
    /// projection, ADV-FOG-004, ADV-FOG-005): by the players that see the hero on the entrance or
    /// on the exit, a viewer that sees one end only told that one, the other `None`; so no viewer
    /// learns a hidden exit (A-41 not reproduced, `ADV-QUIRK-43`). Its own event, not a step's,
    /// whose two tiles a client takes for neighbours. The exit's reveal is told by the explored
    /// tiles alone.
    HeroTeleported {
        /// The hero.
        hero: HeroId,
        /// The entrance (always given when emitted; `None` in a projection for a viewer that does
        /// not see the hero there).
        from: Option<Pos>,
        /// The exit (likewise).
        to: Option<Pos>,
        /// The players that see the hero on the entrance (the viewer alone, or none, in a
        /// projection).
        seen_from: Viewers,
        /// The players that see the hero on the exit (likewise).
        seen_to: Viewers,
    },
    /// A hero's teleporter leads nowhere (ADV-TEL-004, ADV-TEL-005; E08 T105a1, increment 11's
    /// design note §10): a one-way exit (the original's ADVEVENT 70, a human's only: every
    /// owner's here, decision 16) or a Subterranean Gate without a pair (ADVEVENT 153); the hero
    /// stays on it. Named by type and anchor, as the observation names the object (the client
    /// chooses the text by its kind). Seen by the hero's owner alone. A monolith without a
    /// candidate tells nothing (the original's silence).
    LeadsNowhere {
        /// The hero.
        hero: HeroId,
        /// The teleporter's type.
        object_type: ObjectTypeId,
        /// Its anchor.
        anchor: Pos,
    },
    /// A Whirlpool took a hero's troops before its jump (ADV-TEL-006; E08 T105a2, increment 11's
    /// design note §8): its weakest stack, by fight value × count, kept its count divided by the
    /// Whirlpool's divisor, the slot emptied at 0 when the army holds other stacks, a lone stack
    /// keeping one (the original's ADVEVENT 168, shown also when a lone creature lost nothing).
    /// Told before the jump; not told to a hero the Whirlpool spares (a Sea Captain's Hat). Seen
    /// by the hero's owner alone, as its army is.
    WhirlpoolLoss {
        /// The hero.
        hero: HeroId,
        /// The stack's army slot.
        slot: u8,
        /// Its creature.
        creature: CreatureTypeId,
        /// How many it lost: 0 for a lone creature kept, all of its count for a slot emptied.
        count: u32,
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
