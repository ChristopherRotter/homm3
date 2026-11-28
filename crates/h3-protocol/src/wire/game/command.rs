//! The whole game's commands and rejections (E16 M5 T3a): what a player sends
//! ([`GameCommand`], the engine's `h3_game::GameCommand` one to one) and why the engine refused
//! it ([`GameReject`], `h3_game::GameCommandError` one to one, a battle's refusal as the battle
//! wire's [`RejectCode`]). A command names no player: the actor is the connection's seat (ADR-0018
//! point 6), and the engine validates every value against the state and the actor (ADR-0008).
//! The one that does is the system actor's ([`SystemCommand`], E16 M5 T14, v56): the host's, in
//! its journal and its records; the engine refuses it from a seat ([`GameReject::SystemOnly`]).
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
    /// The system actor's own command for a player (E16 M5 T14, v56): the host's alone.
    System {
        /// The command.
        command: SystemCommand,
    },
    /// A cast of a spell on the adventure map by one of the actor's heroes (GAM-SPL-001; E08
    /// T142a, v82).
    Cast {
        /// The casting hero.
        hero: u16,
        /// The core spell.
        spell: u16,
        /// The target tile, for Scuttle Boat and Dimension Door.
        target: Option<Pos>,
    },
}

/// The system actor's own command for a player (`h3_game::SystemCommand`; E16 M5 T14, v56).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum SystemCommand {
    /// Ends the player's turn (GAM-SYS-001).
    EndTurn {
        /// The player.
        player: u8,
    },
    /// Answers the player's first open decision with its default (GAM-SYS-002).
    Answer {
        /// The player.
        player: u8,
    },
    /// A command to the player's battle side while it is to act (GAM-SYS-003).
    Battle {
        /// The player.
        player: u8,
        /// The battle's command.
        command: Command,
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
    /// Opens the hire dialog of what a building of a town of the actor's sells: a standing
    /// dwelling, horde or Portal of Summoning (TWN-REC-001, -002; E09 T27).
    Recruit {
        /// The town.
        town: u16,
        /// The building, as [`Self::Build`]'s.
        building: u8,
    },
    /// Moves a hero between the two bars of a town of the actor's (TWN-ROW-002 to -004; E09
    /// T28a, v64).
    Heroes {
        /// The town.
        town: u16,
        /// The move.
        change: TownHeroChange,
    },
    /// Hires the actor's tavern hero in a slot at a town of the actor's with a standing Tavern
    /// (TWN-TAV-003; E09 T38a1, v71).
    Hire {
        /// The town.
        town: u16,
        /// The slot: 0 or 1 (another is refused, [`TownReject::UnknownSlot`]).
        slot: u8,
    },
    /// Opens the town screen of a town of the actor's: its Mana Vortex to its garrison hero, then
    /// its visitor, then both learning its guild's spells (TWN-GLD-006; E09 T49a2, v79).
    Open {
        /// The town.
        town: u16,
    },
    /// Buys a spell book at the Mage Guild of a town of the actor's for its visitor, else its
    /// garrison hero, for the town part's price; then the town's heroes learn its guild's spells
    /// (TWN-GLD-007; E09 T49b, v80).
    BuySpellBook {
        /// The town.
        town: u16,
    },
    /// Opens the market window of a town of the actor's, with the town as its place: E08's
    /// market decision at the actor's count of Marketplaces, its hero the town's (TWN-MKT-002;
    /// E09 T58a1, v85).
    Market {
        /// The town.
        town: u16,
    },
    /// Buys the war machine a building of a town of the actor's sells, for the town's hero, at
    /// the price of the machine's creature (TWN-SMT-001: the Blacksmith, the Ballista Yard; E09
    /// T59a, v87).
    BuyWarMachine {
        /// The town.
        town: u16,
        /// The building, as [`Self::Build`]'s.
        building: u8,
    },
    /// Builds a boat on the spot of a town of the actor's at its Shipyard, for the town part's
    /// price (TWN-SHY-001; E09 T59a, v87).
    BuildBoat {
        /// The town.
        town: u16,
    },
    /// Opens the Magic University of a town of the actor's for its visitor or garrison hero, the
    /// actor's: E08's University decision with the town as its place (TWN-UNI-001; E09 T59b1,
    /// v88).
    University {
        /// The town.
        town: u16,
        /// The student.
        hero: u16,
    },
    /// Transforms the stack in a slot of an army of a town of the actor's at its Skeleton
    /// Transformer, the part's target of its creature, its count kept (TWN-TRF-001; E09 T59b2,
    /// v89).
    Transform {
        /// The town.
        town: u16,
        /// The army: the visitor's or the upper bar's.
        army: TownArmy,
        /// The army's slot.
        slot: u8,
    },
}

/// The army of a town a transformation names (`h3_towns::TownArmy`; E09 T59b2, v89).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum TownArmy {
    /// The army of the actor's hero on the town's entrance.
    Visitor {},
    /// The upper bar's: the garrison hero's army, else the town's garrison.
    Garrison {},
}

/// A hero's move between a town's two bars (`h3_towns::TownHeroChange`; E09 T28a, v64).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum TownHeroChange {
    /// The visitor into the garrison (TWN-ROW-002).
    Enter {},
    /// The garrison hero out onto the entrance (TWN-ROW-003).
    Leave {},
    /// The two change places (TWN-ROW-004).
    Swap {},
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
    /// A map Tavern's hire or closing (GAM-DEC-010; E09 T39a, v73).
    Tavern {
        /// The hire or closing.
        answer: TavernAnswer,
    },
    /// A market's trade, transfer, sale, purchase or leaving (GAM-DEC-011; E08 T133b, v75).
    Market {
        /// The answer.
        answer: MarketAnswer,
    },
    /// An Altar of Sacrifice's sacrifice or leaving (GAM-DEC-012; E08 T134a, v76).
    Altar {
        /// The answer.
        answer: AltarAnswer,
    },
    /// A Hill Fort's upgrade of a row or of all, or leaving (GAM-DEC-013; E08 T134b1, v77).
    HillFort {
        /// The answer.
        answer: HillFortAnswer,
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

/// A map Tavern's answer (`h3_game::TavernAnswer`, GAM-DEC-010; E09 T39a, v73).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum TavernAnswer {
    /// Hires the tavern hero of this slot (0 or 1), which closes the Tavern.
    Hire {
        /// The slot.
        slot: u8,
    },
    /// Closes the Tavern.
    Close {},
}

/// A market's answer (`h3_game::MarketAnswer`, GAM-DEC-011; E08 T133b, v75): resources by their
/// handle. The rate, the price and the most are the server's (ADR-0004).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum MarketAnswer {
    /// Trades whole lots of one resource for another at the market's rate (ADV-MKT-003).
    Trade {
        /// The resource given.
        give: u16,
        /// The resource wanted.
        take: u16,
        /// The lots.
        lots: u32,
    },
    /// Gives a resource to another player, 1 : 1 (ADV-MKT-004).
    Transfer {
        /// The resource.
        resource: u16,
        /// The amount.
        amount: u32,
        /// The recipient.
        to: u8,
    },
    /// Sells whole lots of a stack of the hero's army (ADV-MKT-005).
    Sell {
        /// The army's slot.
        slot: u8,
        /// The resource wanted.
        take: u16,
        /// The lots.
        lots: u32,
    },
    /// Buys a Black Market's entry (ADV-MKT-007).
    Buy {
        /// The stock's entry.
        entry: u8,
        /// The resource paid in.
        pay: u16,
    },
    /// Sells the window's hero's artifact at a place, at a town with the Artifact Merchants
    /// (TWN-MKT-005; E09 T58b, v86).
    SellArtifact {
        /// The artifact's place.
        place: ArtifactPlace,
        /// The resource wanted.
        take: u16,
    },
    /// Leaves the market.
    Leave {},
}

/// An Altar of Sacrifice's answer (`h3_game::AltarAnswer`, GAM-DEC-012; E08 T134a, v76): a
/// sacrifice names everything it takes. The experience is the server's (ADR-0004).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum AltarAnswer {
    /// Sacrifices the hero's artifacts at these places (ADV-ALT-002; `h3_game::AltarPlaces`).
    Artifacts {
        /// Bit `p`: the worn position `p`.
        worn: u32,
        /// Bit `i`: the backpack's index `i`.
        backpack: u64,
    },
    /// Sacrifices so many creatures of each slot of the hero's army (ADV-ALT-003).
    Creatures {
        /// The amounts by slot, 0 none.
        amounts: [u32; 7],
    },
    /// Leaves the Altar.
    Leave {},
}

/// A Hill Fort's answer (`h3_game::HillFortAnswer`, GAM-DEC-013; E08 T134b1, v77). The rows and
/// their prices are the server's (ADR-0004).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum HillFortAnswer {
    /// Upgrades the stack at this slot of the hero's army (ADV-HFT-002).
    Upgrade {
        /// The slot.
        slot: u8,
    },
    /// Upgrades every stack that upgrades.
    All {},
    /// Leaves the Hill Fort.
    Leave {},
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
    /// Dismisses a hero (GAM-MAN-003): it leaves the game with every artifact it holds.
    Dismiss {
        /// The hero.
        hero: u16,
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
    /// A map garrison's army, by the garrison's object index (E08 T114a2), while its exchange is
    /// open (its `ExchangeOpened` names the index).
    MapGarrison {
        /// The garrison's object index.
        garrison: u32,
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

/// Why the game rejected a command (`h3_game::GameCommandError`, and the session's last four, E16
/// M5 T8a and T8b); a rejected command changed nothing. The text is the client's.
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
    /// A system command from a player (E16 M5 T14, v56): the system actor's alone.
    SystemOnly {},
    /// An answer of another kind than the open decision.
    WrongAnswer {},
    /// The game waits for another player's answer to its level-up: whose is not told (E16 M5
    /// T19, v63; SRV-Q31, as `GameClock`'s `waits`).
    WaitingForLevelUp {},
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
    /// A map Tavern's hire the game refused (E09 T39a, v73).
    Tavern {
        /// Why.
        error: TavernReject,
    },
    /// A market's answer the game refused (E08 T133b, v75); the treasury's is [`Self::CannotPay`].
    Market {
        /// Why.
        error: MarketReject,
    },
    /// An Altar's sacrifice the game refused (E08 T134a, v76).
    Altar {
        /// Why.
        error: AltarReject,
    },
    /// A Hill Fort's upgrade the game refused (E08 T134b1, v77); the treasury's is
    /// [`Self::CannotPay`].
    HillFort {
        /// Why.
        error: HillFortReject,
    },
    /// The session's, not the engine's (E16 M5 T8a, v45): the connection holds no seat of the
    /// game.
    NotSeated {},
    /// The session's (E16 M5 T8a, v45): the game does not run: not every seat was held yet, or
    /// (E16 M5 T8b, v46) it is paused, or (E16 M5 T20, v65) it is void (a battle the system
    /// played reached the host's bound on its commands).
    NotStarted {},
    /// The session's (E16 M5 T8b, v46): the seat asked for a pause already since the game last
    /// moved on.
    AlreadyAsked {},
    /// The session's (E16 M5 T8b, v46): the command's number is at or below the last its seat's
    /// accepted commands took, and the session keeps no answer for it (a number rejected or
    /// skipped, or one older than the seat's last [`crate::session::game::KEPT_ANSWERS`]
    /// accepted): a resend, nothing applied.
    Stale {},
    /// The session's (E16 M5 T18, v62): the seat's map commands in its running turn (an
    /// adventure's, a management one, a town's) reached the host's bound
    /// ([`crate::session::game::CommandBounds::per_turn`]): it may still end its turn, answer its
    /// decisions and fight its battles.
    TurnLimit {},
    /// The session's (E16 M5 T18, v62): the seat's commands in its open battle reached the
    /// host's bound ([`crate::session::game::CommandBounds::per_battle`]): the computer plays its
    /// side for the rest of the battle.
    BattleLimit {},
    /// The session's (E16 M5 T22a, v66): `SetSpectators` from a seat that is not the host's (the
    /// creator's).
    NotHost {},
    /// The session's (E16 M5 T22a, v66): `SetSpectators` once the game started, when its
    /// setting is fixed.
    AlreadyStarted {},
    /// A cast on the map the game refused (`h3_game::MapCastError`, GAM-SPL-001; E08 T142a,
    /// v82).
    Cast {
        /// Why.
        error: CastReject,
    },
}

/// Why a cast on the map was refused (`h3_game::MapCastError`, GAM-SPL-001; E08 T142a, v82), in
/// the engine's order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum CastReject {
    /// The hero is not one of the actor's heroes on the map.
    NotYourHero {
        /// The hero.
        hero: u16,
    },
    /// The hero may not cast the spell.
    Unknown {
        /// The spell.
        spell: u16,
    },
    /// The spell does not act on the map.
    NotOnMap {
        /// The spell.
        spell: u16,
    },
    /// A target given for a spell that names none, or none for one that does.
    Target {},
    /// The spell points are below the cost.
    SpellPoints {},
    /// The ground allows no spell of that level (Cursed Ground).
    Ground {},
    /// The hero stands in or on an anti-magic garrison.
    AntiMagic {},
    /// The spell's own test refused it.
    Adventure {
        /// Why.
        error: SpellReject,
    },
}

/// Why the spell's own test refused a cast on the map (`h3_adventure::CastError`; E08 T142a,
/// v82).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum SpellReject {
    /// The hero is not one of the actor's heroes on the map.
    NotYourHero {
        /// The hero.
        hero: u16,
    },
    /// Fly, or Summon Boat, in a boat.
    InBoat {},
    /// The spell's level is on already (`ADV-QUIRK-54`).
    AlreadyOn {},
    /// The cast would not raise the hero's level (`ADV-QUIRK-54` off).
    NotHigher {},
    /// Summon Boat without a free water tile around the hero (E08 T143a, v83).
    NoWater {},
    /// A target outside the spell's window or off the map (E08 T143a, v83).
    OutOfWindow {},
    /// Scuttle Boat's target holds no empty boat it sinks (E08 T143a, v83).
    NoBoat {},
    /// Dimension Door with no movement points left (E08 T143b, v84).
    NoPoints {},
    /// Dimension Door cast today as often as the mastery allows (E08 T143b, v84).
    NoCastsLeft {},
    /// Dimension Door's target is blocked or an entrance (E08 T143b, v84).
    NotFree {},
    /// Dimension Door's explored target is land from a boat or water from land
    /// (`ADV-QUIRK-55` off; E08 T143b, v84).
    WrongKind {},
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
    /// The building sells no creatures (TWN-REC-001).
    NotRecruiting {},
    /// The building does not stand to sell creatures (TWN-REC-001).
    NotStanding {},
    /// The Portal of Summoning holds nothing (TWN-REC-002).
    NothingSummoned {},
    /// A hero stands in the town's garrison already (TWN-ROW-002; E09 T28a, v64).
    GarrisonHeld {},
    /// No hero of the actor's stands on the town's entrance (TWN-ROW-002, -004).
    NoVisitor {},
    /// The garrison's stacks find no slot in the visitor's army (TWN-ROW-002).
    ArmiesDoNotFit {},
    /// A stack would hold more than a stack's most (TWN-ROW-002).
    TooManyCreatures {},
    /// No hero stands in the town's garrison (TWN-ROW-003, -004).
    NoGarrisonHero {},
    /// A hero stands on the town's entrance (TWN-ROW-003).
    EntranceHeld {},
    /// The actor has the most heroes on the map already (TWN-ROW-003).
    TooManyHeroes {},
    /// The garrison hero's army holds no stack (TWN-ROW-003).
    EmptyArmy {},
    /// The town has no standing Tavern (TWN-TAV-003; E09 T38a1, v71).
    NoTavern {},
    /// The slot is not one of the actor's two tavern slots (TWN-TAV-003).
    UnknownSlot {},
    /// The slot holds no hero (TWN-TAV-003).
    EmptySlot {},
    /// No Mage Guild stands in the town to sell a spell book (TWN-GLD-007; E09 T49b, v80).
    NoGuild {},
    /// No hero stands in the town, neither a visitor nor a garrison hero (TWN-GLD-007; v80).
    NoHero {},
    /// The town's hero wears a spell book (TWN-GLD-007; v80).
    HasSpellBook {},
    /// No Marketplace, Artifact Merchants or Freelancer's Guild stands in the town (TWN-MKT-002;
    /// v85).
    NoMarket {},
    /// The building sells no war machine (TWN-SMT-001; E09 T59a, v87).
    NoWarMachine {},
    /// The town's hero holds the building's war machine, or has no position free for it
    /// (TWN-SMT-001; v87).
    HasWarMachine {},
    /// An empty boat or a hero stands on the town's spot (TWN-SHY-001; v87).
    BoatSpotTaken {},
    /// No Magic University stands in the town (TWN-UNI-001; E09 T59b1, v88).
    NoUniversity {},
    /// The hero is neither the town's visitor nor its garrison hero, or not the actor's
    /// (TWN-UNI-001; v88).
    NotTownHero {},
    /// No Skeleton Transformer stands in the town (TWN-TRF-001; E09 T59b2, v89).
    NoTransformer {},
    /// The stack's creature is its own target (TWN-TRF-001; v89).
    NothingToTransform {},
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
    /// The hero to dismiss is the actor's garrison hero, not on the map (GAM-MAN-003).
    NotOnMap {},
    /// The hero to dismiss visits a town (GAM-MAN-003).
    InTown {},
    /// The hero to dismiss is the actor's one hero on the map, and it owns no town (GAM-MAN-003).
    LastHero {},
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
    /// The object index holds no map garrison (E08 T114a2).
    NoGarrison {},
    /// The stack's army is not the actor's to take from (the one owner rule, ADV-GAR-003).
    NotOwn {},
    /// An army not the actor's takes stacks only into its empty slots (the one owner rule).
    OnlyEmpty {},
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

/// Why a map Tavern's hire was refused (`h3_game::TavernError`, GAM-DEC-010; E09 T39a, v73).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum TavernReject {
    /// The slot is not one of the player's two.
    UnknownSlot {},
    /// The slot holds no hero.
    EmptySlot {},
    /// The player has the most heroes on the map.
    TooManyHeroes {},
    /// A hero stands on the Tavern's visitable tile.
    Occupied {},
}

/// Why an Altar's sacrifice was refused (`h3_game::AltarError`, GAM-DEC-012; E08 T134a, v76).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum AltarReject {
    /// The visit's hero is not on the map, or its object is no Altar.
    Visit {},
    /// The hero's class's faction does not sacrifice that.
    Screen {},
    /// More places than the Altar takes, or a place without an artifact.
    Places {},
    /// An artifact the Altar never takes.
    Kept {
        /// The artifact's handle.
        artifact: u16,
    },
    /// A slot's amount above its stack (or on an empty slot).
    Amount {
        /// The slot.
        slot: u8,
    },
    /// The sacrifice would take the army's last creature.
    LastCreature {},
    /// The sacrifice gives no experience.
    Nothing {},
}

/// Why a Hill Fort's upgrade was refused (`h3_game::HillFortError`, GAM-DEC-013; E08 T134b1, v77).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum HillFortReject {
    /// The visit's hero is not on the map, or its object is no Hill Fort.
    Visit {},
    /// The slot holds no stack that upgrades.
    NoUpgrade {
        /// The slot.
        slot: u8,
    },
    /// No stack of the army upgrades.
    Nothing {},
}

/// Why a market's answer was refused (`h3_game::MarketError`, GAM-DEC-011; E08 T133b, v75).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum MarketReject {
    /// The visit's hero is not on the map, or its object is no market.
    Visit {},
    /// The market's window has no screen for the answer.
    Screen {},
    /// A trade of a resource for itself.
    SameResource {},
    /// The lots (a transfer's amount) are 0 or above the most.
    Amount {
        /// The lots or the amount.
        amount: u32,
    },
    /// A transfer's recipient is the actor or no player of the game.
    Recipient {},
    /// The slot holds no stack of the hero's army.
    Slot {
        /// The slot.
        slot: u8,
    },
    /// The sale would take the army's last creature.
    LastCreature {},
    /// The entry is not one of the stock, or it was bought.
    Entry {
        /// The entry.
        entry: u8,
    },
    /// No lot or price in the resource.
    NoPrice {},
    /// The hero cannot receive the artifact.
    NoRoom {},
    /// The hero holds no artifact at the place (an artifact's sale; v86).
    Place {
        /// The place.
        place: ArtifactPlace,
    },
    /// The artifact at the place is not traded: worn where the backpack does not take its slot's
    /// artifacts (the war machines, the Catapult, the Spell Book; v86).
    NotTradable {},
}
