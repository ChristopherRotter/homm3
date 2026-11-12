//! The whole game's commands and rejections (E16 M5 T3a): what a player sends
//! ([`GameCommand`], the engine's `h3_game::GameCommand` one to one) and why the engine refused
//! it ([`GameReject`], `h3_game::GameCommandError` one to one, a battle's refusal as the battle
//! wire's [`RejectCode`]). A command names no player: the actor is the connection's seat (ADR-0018
//! point 6), and the engine validates every value against the state and the actor (ADR-0008).
//!
//! The same rules as [`super`]'s; the variants' meaning is documented on the engine's types,
//! named in each variant's comment. Provisional (E16 M5 part 1's review): the shapes.

use alloc::vec::Vec;

use serde::{Deserialize, Serialize};

use super::{Formation, Pos};
use crate::wire::{Command, RejectCode};

/// A command to the whole game (`h3_game::GameCommand`), in both directions: a client sends one,
/// the server lists the legal ones and echoes the one it applied.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum GameCommand {
    /// Ends the actor's turn.
    EndTurn {},
    /// A command to the adventure map, in the actor's turn.
    Adventure {
        /// The command.
        command: AdventureCommand,
    },
    /// Answers the actor's first open decision.
    Answer {
        /// The answer.
        answer: Answer,
    },
    /// A command to the battle of the actor's first open decision, for the side it owns.
    Battle {
        /// The battle's command.
        command: Command,
    },
    /// A management command on the actor's heroes on the map (GAM-MAN-001).
    Manage {
        /// The command.
        manage: Manage,
    },
    /// A command to one of the actor's towns (E09 T18b1).
    Town {
        /// The command.
        command: TownCommand,
    },
}

/// A command to a town (`h3_towns::TownCommand`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum TownCommand {
    /// Builds a building in a town of the actor's, as its hall offers it (TWN-BLD-001).
    Build {
        /// The town.
        town: u16,
        /// The building: its index in the town's faction's record, below 64 (a building beyond
        /// is refused as not of the record, [`TownReject::UnknownBuilding`]).
        building: u8,
    },
}

/// A command to the adventure map (`h3_adventure::AdventureCommand`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum AdventureCommand {
    /// Moves a hero of the actor's along `path`, step by step (ADV-MOV-001).
    MoveHero {
        /// The hero.
        hero: u16,
        /// The tiles it steps onto, in order, its own tile not included.
        path: Vec<Pos>,
    },
    /// Builds a boat at a Shipyard the actor or an ally owns (ADV-SHY-003).
    BuildBoat {
        /// The Shipyard's type handle.
        object_type: u16,
        /// Its anchor.
        anchor: Pos,
    },
    /// Visits the entered object under a hero of the actor's again, without points (ADV-TEL-008,
    /// E08 T105b2; the original's space key).
    Revisit {
        /// The hero.
        hero: u16,
    },
}

/// An answer to a pending decision, typed by the decision's kind (`h3_game::Answer`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Answer {
    /// A yes-or-no offer's (GAM-DEC-001).
    Offer {
        /// Yes or no.
        consent: Consent,
    },
    /// A level-up's with two offers (GAM-DEC-002).
    LevelUp {
        /// The offer the hero learns.
        offer: SkillOffer,
    },
    /// A treasure chest's choice (GAM-DEC-005).
    Chest {
        /// Its gold or its experience.
        choice: ChestChoice,
    },
    /// The Arena's choice (GAM-DEC-005).
    Arena {
        /// The primary skill.
        choice: ArenaChoice,
    },
    /// A School's choice (GAM-DEC-005).
    School {
        /// The primary skill, or leaving.
        choice: SchoolChoice,
    },
    /// A University's purchase or leaving (GAM-DEC-006).
    University {
        /// The purchase or leaving.
        answer: UniversityAnswer,
    },
    /// A hire dialog's purchase or closing (GAM-DEC-008).
    Hire {
        /// The purchase or closing.
        answer: HireAnswer,
    },
    /// Closes a meeting (GAM-DEC-003) or an army dialog (GAM-DEC-004).
    Close {},
}

/// An offer's answer (`h3_game::Consent`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Consent {
    /// Yes: the price is paid and the offer carried out.
    Accept,
    /// No: the decision closes.
    Decline,
}

/// A level-up's answer (`h3_game::SkillOffer`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SkillOffer {
    /// The first offer (the dialog's left).
    First,
    /// The second offer (the dialog's right).
    Second,
}

/// A treasure chest's answer (`h3_game::ChestChoice`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChestChoice {
    /// Its gold to the player.
    Gold,
    /// Its experience to the hero.
    Experience,
}

/// The Arena's answer (`h3_game::ArenaChoice`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArenaChoice {
    /// The type's first primary skill (the base pack's Attack).
    Attack,
    /// Its second (the base pack's Defense).
    Defense,
}

/// A School's answer (`h3_game::SchoolChoice`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SchoolChoice {
    /// The type's first primary skill, for its price.
    First,
    /// Its second, for its price.
    Second,
    /// Neither: nothing paid.
    Leave,
}

/// A University's answer (`h3_game::UniversityAnswer`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum UniversityAnswer {
    /// Buys the skill of a slot.
    Buy {
        /// The slot: its index among the University's skills.
        slot: u8,
    },
    /// Closes the University.
    Leave {},
}

/// A hire dialog's answer (`h3_game::HireAnswer`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum HireAnswer {
    /// Buys creatures of the dialog's source for their price.
    Buy {
        /// The creature's handle, one of the source's.
        creature: u16,
        /// How many.
        amount: u32,
    },
    /// Closes the dialog.
    Close {},
}

/// A management command (`h3_game::Manage`, GAM-MAN-001).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Manage {
    /// An army command: a stack moved, split or dismissed (ADV-ARM-001 to -005).
    Army {
        /// The command.
        command: ArmyCommand,
    },
    /// Sets a hero's formation (ADV-ARM-006).
    Formation {
        /// The hero.
        hero: u16,
        /// The formation.
        formation: Formation,
    },
    /// Sets a hero's tactics flag (ADV-ARM-006).
    Tactics {
        /// The hero.
        hero: u16,
        /// Whether the flag is on.
        on: bool,
    },
    /// Moves an artifact between two places of a hero, or of the hero it meets (HER-ART-005).
    Artifact {
        /// Where it is taken from.
        from: ArtifactAt,
        /// Where it goes; the artifact there is swapped back.
        to: ArtifactAt,
    },
    /// Assembles the combination the artifact worn at `at` completes (HER-ART-006).
    Assemble {
        /// The hero.
        hero: u16,
        /// The position of a part.
        at: u16,
    },
    /// Disassembles the combination worn at `at` (HER-ART-006).
    Disassemble {
        /// The hero.
        hero: u16,
        /// The combination's position.
        at: u16,
    },
}

/// An army command (`h3_adventure::ArmyCommand`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum ArmyCommand {
    /// Moves a stack onto a slot: onto an empty one, merged with the same creature, or swapped.
    Move {
        /// The stack's slot.
        from: ArmySlot,
        /// The target slot.
        to: ArmySlot,
    },
    /// Splits `count` creatures off a stack onto a slot that is empty or holds the same creature.
    Split {
        /// The stack's slot.
        from: ArmySlot,
        /// The target slot.
        to: ArmySlot,
        /// How many go.
        count: u32,
    },
    /// Dismisses a stack.
    Dismiss {
        /// Its slot.
        at: ArmySlot,
    },
}

/// A slot of an army (`h3_adventure::ArmySlot`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArmySlot {
    /// The army.
    pub army: ArmyRef,
    /// The slot, from 0.
    pub slot: u8,
}

/// An army a command names (`h3_adventure::ArmyRef`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum ArmyRef {
    /// A hero's army.
    Hero {
        /// The hero.
        hero: u16,
    },
    /// The group of the actor's open army dialog.
    Group {},
    /// A town's garrison.
    Garrison {
        /// The town.
        town: u16,
    },
}

/// A place of a hero's artifacts: the hero and where.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArtifactAt {
    /// The hero.
    pub hero: u16,
    /// The place.
    pub place: ArtifactPlace,
}

/// Where a hero holds an artifact (`h3_heroes::ArtifactPlace`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum ArtifactPlace {
    /// Worn at a position of the ruleset's slots.
    Worn {
        /// The position.
        position: u16,
    },
    /// In the backpack at an index.
    Backpack {
        /// The index.
        index: u16,
    },
}

/// Why the game rejected a command (`h3_game::GameCommandError`, and the session's last two, E16
/// M5 T8a); a rejected command changed nothing. The text is the client's.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum GameReject {
    /// The actor is not a player of the game.
    UnknownPlayer {
        /// The player.
        player: u8,
    },
    /// The actor's turn is not running.
    NotYourTurn {
        /// The player.
        player: u8,
    },
    /// The actor has an open decision: its answers are its legal commands.
    DecisionOpen {},
    /// An answer, but no decision is open.
    NoDecision {},
    /// An acceptance of an offer that no longer stands.
    OfferGone {},
    /// The adventure map refused the command.
    Adventure {
        /// Why.
        error: AdventureReject,
    },
    /// An answer, but the open decision is a battle.
    BattleOpen {},
    /// A battle command, but the open decision is no battle.
    NoBattle {},
    /// A battle command, but the side to act is not the actor's.
    NotYourSide {},
    /// The battle refused the command: one of the battle wire's engine codes.
    Battle {
        /// Why.
        code: RejectCode,
    },
    /// A command the system actor does not issue.
    NotForSystem {},
    /// An answer of another kind than the open decision.
    WrongAnswer {},
    /// The game waits for this player's answer to its level-up.
    WaitingForLevelUp {
        /// The player.
        player: u8,
    },
    /// The management command was refused.
    Manage {
        /// Why.
        error: ManageReject,
    },
    /// A University's purchase of a slot not open for the hero.
    SlotClosed {},
    /// The treasury cannot pay the price.
    CannotPay {},
    /// A hire dialog's purchase the map refused.
    Hire {
        /// Why.
        error: HireReject,
    },
    /// A town command the town refused.
    Town {
        /// Why.
        error: TownReject,
    },
    /// The session's, not the engine's (E16 M5 T8a, v45): the connection holds no seat of the
    /// game.
    NotSeated {},
    /// The session's (E16 M5 T8a, v45): the game has not started, not every seat was held yet.
    NotStarted {},
}

/// Why a town command was refused (`h3_towns::TownCommandError`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum TownReject {
    /// The town is not the game's.
    UnknownTown {},
    /// The actor does not own the town.
    NotOwner {},
    /// The building is not of the town's faction's record.
    UnknownBuilding {},
    /// It was ever built: it stands, or a building that supersedes it does.
    Built {},
    /// It is forbidden in this town.
    Forbidden {},
    /// The Grail's building, which the hall never builds.
    Grail {},
    /// Not every building it requires was ever built.
    Requirements {},
    /// A Capitol while the player holds another.
    Capitol {},
    /// A horde the hall offers as another building.
    OfferedAs {
        /// The building the hall offers it as.
        building: u8,
    },
    /// The town was built in today (TWN-BLD-004).
    BuiltToday {},
}

/// Why the adventure map refused a command (`h3_adventure::AdventureCommandError`); a step is
/// the index of a path's tile, from 0.
#[allow(missing_docs)] // the fields are `step`, `cost` and `points` as the engine's
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum AdventureReject {
    /// The hero is not the actor's on the map.
    NotYourHero { hero: u16 },
    /// The path has no step.
    EmptyPath {},
    /// The step does not go to a neighbouring tile.
    NotNeighbour { step: u32 },
    /// The step goes to an unexplored tile.
    Unexplored { step: u32 },
    /// The hero cannot stand on the step's tile.
    Blocked { step: u32, error: StandReject },
    /// The step goes to a tile no visit is built for.
    Visit { step: u32 },
    /// The step crosses the top of an object's visitable tile.
    FromAbove { step: u32 },
    /// The step follows a visit.
    PastVisit { step: u32 },
    /// The step lands where the hero may not stop.
    NoStop { step: u32 },
    /// The step costs more than the points left.
    TooFewPoints { step: u32, cost: u32, points: u32 },
    /// The step goes onto an empty boat from water.
    Boat { step: u32 },
    /// The step goes ashore where no boat may land.
    Landing { step: u32 },
    /// The step sails diagonally past the coast.
    Coast { step: u32 },
    /// No Shipyard the actor knows of is there.
    UnknownObject {},
    /// The Shipyard is neither the actor's nor an ally's.
    NotOwner {},
    /// The Shipyard has no boat tile.
    NoBoatTile {},
    /// A boat or a hero is on the Shipyard's boat tile.
    BoatTileTaken {},
    /// The hero stands on no object to visit again (E08 T105b2).
    NothingToRevisit {},
}

/// Why a hero cannot stand on a tile (`h3_adventure::StandError`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StandReject {
    /// The tile is outside the map.
    Outside,
    /// The terrain is not walked on land.
    Terrain,
    /// An object blocks the tile.
    Blocked,
    /// Another hero stands there.
    Occupied,
    /// The tile's object is visited from a neighbouring tile.
    Visited,
}

/// Why a management command was refused (`h3_game::ManageError`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum ManageReject {
    /// A hero the command names is not the actor's on the map.
    NotYourHero {
        /// The hero.
        hero: u16,
    },
    /// The two armies are not together in a meeting or an army dialog.
    NotTogether {},
    /// The adventure refused the army command.
    Army {
        /// Why.
        error: ArmyReject,
    },
    /// The hero model refused the artifact's move.
    Artifact {
        /// Why.
        error: MoveReject,
    },
    /// The hero model refused the assembly or the disassembly.
    Combine {
        /// Why.
        error: CombineReject,
    },
}

/// Why an army command was refused (`h3_adventure::ArmyCommandError`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum ArmyReject {
    /// The hero is not the actor's.
    NotYourHero {
        /// The hero.
        hero: u16,
    },
    /// No army dialog's group is open.
    NoGroup {},
    /// The slot is beyond the army's slots.
    NoSlot {
        /// The slot.
        slot: u8,
    },
    /// The slot holds no stack.
    NoStack {
        /// The slot.
        slot: u8,
    },
    /// The stack would go onto its own slot.
    SameSlot {},
    /// The stack would hold more than a stack holds.
    TooManyCreatures {},
    /// A hero's last stack would leave its army.
    LastStack {},
    /// The split's target holds another creature.
    OtherCreature {},
    /// A split cannot leave this count in its target.
    SplitCount {
        /// The count.
        count: u32,
    },
    /// The town's garrison is not managed.
    Garrison {
        /// The town.
        town: u16,
    },
}

/// Why an artifact's move was refused (`h3_heroes::MoveError`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum MoveReject {
    /// No artifact is at the place.
    NoArtifact {
        /// The place.
        place: ArtifactPlace,
    },
    /// The artifact never moves.
    Unmovable {
        /// The artifact type's handle.
        artifact: u16,
    },
    /// The artifact is there already.
    SamePlace {},
    /// The artifact cannot go to the place.
    CannotPlace {
        /// The place.
        place: ArtifactPlace,
    },
    /// The target's artifact cannot go back to the place.
    CannotReturn {
        /// The place.
        place: ArtifactPlace,
    },
}

/// Why an assembly or a disassembly was refused (`h3_heroes::CombineError`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum CombineReject {
    /// No artifact is worn at the position.
    NoArtifact {
        /// The position.
        at: u16,
    },
    /// The artifact at the position completes no combination.
    Incomplete {
        /// The position.
        at: u16,
    },
    /// The artifact at the position is no combination.
    NotCombination {
        /// The position.
        at: u16,
    },
}

/// Why a hire dialog's purchase was refused (`h3_adventure::HireError`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum HireReject {
    /// The dialog's hero or source is not on the map.
    NoSource {},
    /// The creature is not the source's.
    NotOffered {
        /// The creature's handle.
        creature: u16,
    },
    /// The amount is not one the source holds.
    Amount {
        /// The amount.
        amount: u32,
    },
    /// The army has no room for the creatures.
    NoRoom {},
}
