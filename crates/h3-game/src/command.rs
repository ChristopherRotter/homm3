//! The whole game's entry point (ADR-0017 point 1, ADR-0004): actors, the game's commands, its
//! events and their sink. A game command wraps a context's command or is the game's own; the game
//! checks turn and phase, the context its rules (E08 T6b: `EndTurn`; T7b: the adventure's
//! `MoveHero`; `observe` and `project` are in `observe.rs`, T8b1; `legal_commands` in
//! `legal.rs`, T8b2a; T14b: the pending decisions and their answers, in `state.rs`; T19a: battles,
//! in `state/battle.rs`; T30: the level-ups, in `state/levels.rs`; T51a: the management commands,
//! [`Manage`], in `state/manage.rs`; E09 T18b1: the town context's commands, [`TownCommand`], in
//! `town/command.rs`).

use core::fmt;

use h3_adventure::{
    ALTAR_SLOTS, AdventureCommand, AdventureCommandError, AdventureEvent, AltarError, AltarPlaces,
    ArmySlot, CastError, CastFailure, HillFortError, HireError, HireTarget, MarketError,
    ObjectTypeId, Pick, Pos, Viewers,
};
use h3_content::{CreatureTypeId, Formation, HeroTypeId, ResourceId, SkillId, SpellId};
use h3_contracts::{ArmyStack, BattleResult, FortLevel, HeroId, PlayerId, TownId};
use h3_heroes::{Choice, HeroChange, PrimarySkill};
use h3_towns::{BuildingId, TownCommand, TownCommandError};

use crate::manage::{Manage, ManageError};
use crate::observe::HireSeen;
use crate::system::SystemCommand;
use crate::town::hire::{HiredPart, TavernError};
use crate::town::tavern::{TavernHero, TavernPart};
use crate::town::thieves::GuildPart;
use crate::town::view::{GarrisonStackView, TownBuildingsView};
use crate::{GameRuleset, GameState, MonthKind, OpenBattle, WeekKind};

/// Who issues a command (ADR-0017 point 2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Actor {
    /// A player, human or bot: the identity the session layer vouches for, never one taken from
    /// the command's payload.
    Player(PlayerId),
    /// The trusted layer that hosts the game (the server's session layer, the env), never a
    /// player (E08 T20b2): it plays a battle's unowned side (a wandering monster's) with a policy
    /// of its own, and commands an owned side only for its player ([`GameCommand::System`]). Its
    /// own commands ([`SystemCommand`], E16 M5 T14) end a player's turn, answer its decision with
    /// its default and play its battle side when its time runs out; removing a player comes with
    /// E10.
    System,
}

/// A command to the whole game.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameCommand {
    /// Ends the actor's turn.
    EndTurn,
    /// A command to the adventure context, given in the actor's turn.
    Adventure(AdventureCommand),
    /// Answers the actor's first open decision (ADR-0017 point 6, GAM-DEC-001; E08 T14b), in its
    /// turn or not.
    Answer(Answer),
    /// A command to the battle of the actor's first open decision, for the side it owns while that
    /// side is to act, in its turn or not (ADR-0017 point 6, GAM-BAT-001; E08 T19a).
    Battle(h3_battle::Command),
    /// A management command on the actor's heroes on the map, in its turn (ADR-0017 points 1 and
    /// 6, GAM-MAN-001; E08 T51a).
    Manage(Manage),
    /// A command to the town context, given in the actor's turn (ADR-0028 point 6, TWN-BLD-001;
    /// E09 decision 45, T18b1): checked and applied as an adventure command is, with no hero and
    /// no visit (the original's town list opens the hall).
    Town(TownCommand),
    /// A cast of a spell on the adventure map by one of the actor's heroes on the map, in its turn
    /// with no decision open (GAM-SPL-001, ADV-SPL-001; E08 T142a, increment 15's design note
    /// §14): `target` names Scuttle Boat's boat and Dimension Door's arrival, and no other spell
    /// takes one. Checked whole before anything changes ([`MapCastError`]); the spell points are
    /// the game's to spend.
    Cast {
        /// The casting hero.
        hero: HeroId,
        /// The core spell.
        spell: SpellId,
        /// The target tile, for a spell that names one.
        target: Option<Pos>,
    },
    /// A command of the system actor for a player ([`SystemCommand`]; ADR-0017 point 2, E16 M5
    /// T14): the system's alone, refused from a player (`SystemOnly`).
    System(SystemCommand),
}

/// An answer to a pending decision, typed by the decision's kind (ADR-0017 point 6; E08 decision
/// 138, T63a): each variant names a kind of decision and carries that kind's own answer. An answer
/// of another kind than the actor's first open decision is rejected (`WrongAnswer`) and changes
/// nothing. The system actor's default for each kind is [`crate::default_answer`]'s.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Answer {
    /// A yes-or-no offer's (GAM-DEC-001): the Cartographer's, the border guard's, a wandering
    /// monster's join, join for gold and pursuit, a guarded object's fight; increment 7's open a
    /// Pandora's Box, search a Pyramid or a Warrior's Tomb, pay a Tree of Knowledge.
    Offer(Consent),
    /// A level-up's with two offers (GAM-DEC-002, E08 T30).
    LevelUp(SkillOffer),
    /// A treasure chest's choice of its gold or its experience (GAM-DEC-005, ADV-PIK-005).
    Chest(ChestChoice),
    /// The Arena's choice of its two primary skills, without leaving (GAM-DEC-005, ADV-TCH-001).
    Arena(ArenaChoice),
    /// A School's choice of its two primary skills for its price, or leaving (GAM-DEC-005,
    /// ADV-TCH-002).
    School(SchoolChoice),
    /// A University's purchase or leaving (GAM-DEC-006, ADV-TCH-007).
    University(UniversityAnswer),
    /// A hire dialog's purchase or closing (GAM-DEC-008, ADV-HIR-002; E08 T95b).
    Hire(HireAnswer),
    /// The map Tavern's hire or closing (GAM-DEC-010, ADV-VIS-007; E09 T39a, the town epic's
    /// design note §66).
    Tavern(TavernAnswer),
    /// A market's trade, transfer, sale, purchase or leaving (GAM-DEC-011, ADV-MKT-003 to -007;
    /// E08 T133b, increment 14's design note §13).
    Market(MarketAnswer),
    /// An Altar of Sacrifice's sacrifice or leaving (GAM-DEC-012, ADV-ALT-001 to -003; E08 T134a,
    /// increment 14's design note §14).
    Altar(AltarAnswer),
    /// A Hill Fort's upgrade of a row or of all, or leaving (GAM-DEC-013, ADV-HFT-001, -002; E08
    /// T134b1, increment 14's design note §15).
    HillFort(HillFortAnswer),
    /// Closes a meeting (GAM-DEC-003), an army dialog (GAM-DEC-004; increment 6's `Decline`) or,
    /// since E08 T114a2, a map garrison's exchange (GAM-DEC-009).
    Close,
}

/// An offer's answer (ADR-0017 point 6; GAM-DEC-001).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Consent {
    /// Yes: the offer's price is paid and the offer carried out.
    Accept,
    /// No: the decision closes; nothing else changes but a wandering monster's answer, which
    /// goes on (its next offer or a fight after a declined join, its leaving after a declined
    /// flight; ADV-MON-007, ADV-MON-008). A later visit asks again.
    Decline,
}

/// A level-up's answer (GAM-DEC-002, E08 T30): which of its two offers the hero learns.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkillOffer {
    /// The first offer (the original's dialog's left).
    First,
    /// The second offer (the dialog's right).
    Second,
}

impl SkillOffer {
    /// The level-up's offer this answer takes.
    pub(crate) const fn choice(self) -> Choice {
        match self {
            Self::First => Choice::First,
            Self::Second => Choice::Second,
        }
    }

    /// The answer that takes `choice`'s offer.
    pub(crate) const fn of(choice: Choice) -> Self {
        match choice {
            Choice::First => Self::First,
            Choice::Second => Self::Second,
        }
    }
}

/// A treasure chest's answer (GAM-DEC-005, ADV-PIK-005).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChestChoice {
    /// Its gold to the player (the original's plain OK).
    Gold,
    /// Its experience to the hero.
    Experience,
}

/// The Arena's answer (GAM-DEC-005, ADV-TCH-001): the first or the second of its type's two
/// primary skills (the base pack's Attack, then Defense); it cannot be left.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArenaChoice {
    /// The first skill (the base pack's Attack; the original's Cancel takes it).
    Attack,
    /// The second skill (the base pack's Defense).
    Defense,
}

/// A School's answer (GAM-DEC-005, ADV-TCH-002): the first or the second of its type's two
/// primary skills, for its price, or leaving without paying.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchoolChoice {
    /// The first skill.
    First,
    /// The second skill.
    Second,
    /// Neither: nothing paid, no mark.
    Leave,
}

/// A University's answer (GAM-DEC-006, ADV-TCH-007): a purchase, which keeps it open, or leaving.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UniversityAnswer {
    /// Buys the skill of this slot (its index among the University's skills, in the draw's order)
    /// for the type's price, learned at Basic, while the slot is open for the hero
    /// (`h3_heroes::university_slot`) and the treasury can pay.
    Buy(u8),
    /// Closes the University.
    Leave,
}

/// A hire dialog's answer (GAM-DEC-008, ADV-HIR-002; E08 T95b, increment 10's design note §13;
/// the original's `H3RecruitMgr`): a purchase, which keeps a dialog of several creatures open and
/// closes one of a single creature, or closing it (the original's Close; the kind's own, as the
/// University's `Leave`: [`Answer::Close`] stays the exchange dialogs').
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HireAnswer {
    /// Buys `amount` `creature`s of the dialog's source for their price (the creature's cost
    /// times the amount, by resource), joined to the dialog's army: refused when the creature is
    /// not the source's, the amount is 0 or above its count, the army has no room for it
    /// ([`GameCommandError::Hire`]) or the treasury cannot pay ([`GameCommandError::CannotPay`]).
    Buy {
        /// The creature, one of the source's.
        creature: CreatureTypeId,
        /// How many, from 1 to the most ([`crate::hire_range`]).
        amount: u32,
    },
    /// Closes the dialog.
    Close,
}

/// The map Tavern's answer (GAM-DEC-010; E09 T39a, the town epic's design note §66; the
/// original's tavern dialog, `map_tavern_visit` 0x5D7E90): a hire of one of the visitor's owner's
/// two tavern heroes, which closes the decision (one hire a visit), or closing it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TavernAnswer {
    /// Hires the tavern hero in this slot (0 or 1) for the part's price, placed on the Tavern's
    /// visitable tile: refused while the slot holds none, the map holds the most heroes of the
    /// player's, a hero stands on the tile ([`GameCommandError::Tavern`]) or the treasury cannot
    /// pay ([`GameCommandError::CannotPay`]), the decision then left open.
    Hire {
        /// The slot.
        slot: u8,
    },
    /// Closes the Tavern.
    Close,
}

/// A market's answer (GAM-DEC-011; E08 T133b, increment 14's design note §13; survey
/// `E08-trades.md` §5 to §7, the original's market dialog): each but [`Self::Leave`] keeps the
/// window open for more (the original's stays open), each refused for a screen the market's kind
/// lacks (`MarketError::Screen`). Nothing in it is trusted: the rate, the price and the most are
/// the state's answer (ADR-0004, `docs/06-security.md`); a refused answer changes nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarketAnswer {
    /// Trades `lots` lots of `give` for `take` at the market's rate (screen `Trade`,
    /// ADV-MKT-003): each lot pays its `give` of the given (`ResourcePaid`) and gains its `take`
    /// of the wanted (`ResourceGained`, saturating, GAM-RES-001). The amount as lots, the
    /// original's slider unit (survey §5), so every accepted amount is one the original could
    /// trade. Refused for a resource for itself (`SameResource`), one the market does not value
    /// (`NoPrice`), then lots of 0 or above the most the treasury pays (`Amount`).
    Trade {
        /// The resource given.
        give: ResourceId,
        /// The resource wanted.
        take: ResourceId,
        /// The lots, from 1 to what the treasury holds of `give` divided by the lot's `give`.
        lots: u32,
    },
    /// Gives `amount` of `resource` to the player `to`, 1 : 1, ally or enemy (screen `Transfer`,
    /// ADV-MKT-004): paid by the actor (`ResourcePaid`), gained by `to` (`ResourceGained`, to it
    /// alone, saturating). Refused for a recipient that is the actor or no player of the game
    /// (`Recipient`; until E10's elimination every player is still in it), then an amount of 0 or
    /// above the treasury's (`Amount`).
    Transfer {
        /// The resource.
        resource: ResourceId,
        /// The amount, from 1 to the treasury's.
        amount: u32,
        /// The recipient.
        to: PlayerId,
    },
    /// Sells `lots` lots of the creatures in the hero's army's slot `slot` for `take` (screen
    /// `Sell`, ADV-MKT-005): the adventure's check (`h3_adventure::AdventureState::check_sale`:
    /// the slot, the lot, the amount, the army's last creature kept), then the stack lowered
    /// (`StackChanged`) and the gain (`ResourceGained`).
    Sell {
        /// The army's slot.
        slot: u8,
        /// The resource wanted.
        take: ResourceId,
        /// The lots.
        lots: u32,
    },
    /// Buys the Black Market's entry `entry` paying in `pay` (screen `Buy`, ADV-MKT-007): the
    /// adventure's check (`h3_adventure::AdventureState::check_buy`: the entry, the price, the
    /// hero's room, A-46 fixed), then the price against the treasury (`CannotPay`), then paid
    /// (`ResourcePaid`), the artifact given (`HeroChanged`) and the entry sold.
    Buy {
        /// The stock's entry.
        entry: u8,
        /// The resource paid in.
        pay: ResourceId,
    },
    /// Leaves the market (`DecisionClosed`), losing nothing.
    Leave,
}

/// An Altar of Sacrifice's answer (GAM-DEC-012; E08 T134a, increment 14's design note §14;
/// survey `E08-trades.md` §12, the original's altar dialog): a sacrifice names everything it takes
/// in one answer (decision 366: A-48 fixed as `ADV-QUIRK-50`, nothing lies on an altar between
/// commands, so leaving loses nothing), and keeps the window open for more (the original's stays
/// open) unless its levels open a level-up, before which the Altar closes and returns after it
/// (GAM-DEC-007's `Rest::Visit`; decision 371). Nothing in it is trusted: the experience is the
/// state's answer (ADR-0004); a refused answer changes nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AltarAnswer {
    /// Sacrifices the hero's artifacts at these places (ADV-ALT-002): the adventure's check
    /// (`h3_adventure::AdventureState::check_artifact_sacrifice`: the screen, the places, the kept
    /// artifacts, a total above 0), then each removed (`HeroChanged`, the worn ones by position,
    /// then the backpack's from the last index down), then the experience (`HeroChanged`) and the
    /// levels.
    Artifacts(AltarPlaces),
    /// Sacrifices so many creatures of each slot of the hero's army, 0 none (ADV-ALT-003): the
    /// adventure's check (`h3_adventure::AdventureState::check_creature_sacrifice`: the screen,
    /// each amount at most its stack, the army keeping a creature, a total above 0), then each
    /// stack lowered (`StackChanged`), then the experience and the levels.
    Creatures([u32; ALTAR_SLOTS]),
    /// Leaves the Altar (`DecisionClosed`), losing nothing.
    Leave,
}

/// A Hill Fort's answer (GAM-DEC-013; E08 T134b1, increment 14's design note §15; survey
/// `E08-trades.md` §13, the original's `hill_fort_dialog_proc` 0x4E8850): a row's upgrade or all
/// of them, each keeping the window open for more (the original's stays open, its rows computed
/// again), or leaving. Nothing in it is trusted: the rows and their prices are the state's answer
/// (ADR-0004); a refused answer changes nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HillFortAnswer {
    /// Upgrades the stack at this slot of the hero's army (ADV-HFT-002; the original's row of
    /// state 1, `hill_fort_upgrade_slot` 0x4E8500): the adventure's check
    /// (`h3_adventure::AdventureState::check_upgrade`: a stack with an upgrade), then the
    /// treasury ([`GameCommandError::CannotPay`]), then its price paid (`ResourcePaid` per
    /// resource, the core's order; nothing for a free one) and the stack upgraded
    /// (`StackChanged`).
    Upgrade(u8),
    /// Upgrades every stack that upgrades, in slot order (the original's All of the whole's state
    /// 1): the adventure's check (`h3_adventure::AdventureState::check_upgrade_all`: some stack
    /// upgrades), then the sum of their prices against the treasury
    /// ([`GameCommandError::CannotPay`]), then each paid and upgraded as [`Self::Upgrade`]'s.
    All,
    /// Leaves the Hill Fort (`DecisionClosed`).
    Leave,
}

impl ChestChoice {
    /// The adventure's pick of this answer.
    pub(crate) const fn pick(self) -> Pick {
        match self {
            Self::Gold => Pick::First,
            Self::Experience => Pick::Second,
        }
    }
}

impl ArenaChoice {
    /// The adventure's pick of this answer.
    pub(crate) const fn pick(self) -> Pick {
        match self {
            Self::Attack => Pick::First,
            Self::Defense => Pick::Second,
        }
    }
}

impl SchoolChoice {
    /// The adventure's pick of this answer.
    pub(crate) const fn pick(self) -> Pick {
        match self {
            Self::First => Pick::First,
            Self::Second => Pick::Second,
            Self::Leave => Pick::Leave,
        }
    }
}

/// Why a command was rejected; a rejected command changes nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameCommandError {
    /// The actor is not a player of the game.
    UnknownPlayer(PlayerId),
    /// The actor's turn is not running.
    NotYourTurn(PlayerId),
    /// The actor has an open decision: its only legal commands are the answers (GAM-DEC-001),
    /// and with a meeting or an army dialog open the management commands on the heroes it holds
    /// (GAM-MAN-001, E08 T51b): a command naming another hero is refused with this.
    DecisionOpen,
    /// An answer, but the actor has no open decision.
    NoDecision,
    /// An acceptance of an offer that no longer stands (ADR-0017 point 10): a border guard
    /// another player passed has left the map (E08 T15a); declining is the only legal answer.
    OfferGone,
    /// The adventure context rejected the command.
    Adventure(AdventureCommandError),
    /// An answer, but the actor's first open decision is a battle: its commands are the actor's
    /// only legal ones (E08 T19a).
    BattleOpen,
    /// A battle command, but the actor's first open decision is no battle (E08 T19a).
    NoBattle,
    /// A battle command, but the side to act is not the actor's (E08 T19a).
    NotYourSide,
    /// The battle rejected the command.
    Battle(h3_battle::CommandError),
    /// A command the system actor does not issue: its commands are an unowned side's battle
    /// commands (E08 T20b2) and its own ([`GameCommand::System`], E16 M5 T14).
    NotForSystem,
    /// A system command ([`GameCommand::System`]) from a player: the system actor's alone
    /// (ADR-0017 point 2, E16 M5 T14).
    SystemOnly,
    /// An answer of another kind than the actor's first open decision (E08 T30; typed by kind
    /// since T63a): a level-up's for an offer, an offer's for a level-up, anything but `Close`
    /// for a meeting or an army dialog.
    WrongAnswer,
    /// The game waits for this player's answer to its level-up (ADR-0017 point 6, GAM-DEC-002;
    /// E08 T30): until it answers, no other actor's command is accepted.
    WaitingForLevelUp(PlayerId),
    /// The management command was refused (GAM-MAN-001; E08 T51a).
    Manage(ManageError),
    /// A University's purchase of a slot that is not open for the hero (GAM-DEC-006, E08 T63a):
    /// the hero knows its skill, knows eight skills, or its class weighs the skill 0
    /// (`h3_heroes::university_slot`), or the University has no such slot.
    SlotClosed,
    /// A University's purchase the treasury cannot pay (GAM-DEC-006, E08 T63a); since E08 T84b2
    /// also a `BuildBoat` (ADV-SHY-003), checked after the adventure's own checks; since E08 T95b
    /// a hire dialog's purchase (GAM-DEC-008), checked after the map's ([`Self::Hire`]).
    CannotPay,
    /// A hire dialog's purchase the map refused (GAM-DEC-008, ADV-HIR-002; E08 T95b), in this
    /// order: the creature not the source's, the amount 0 or above its count, no room in the army
    /// (`h3_adventure::AdventureState::check_hire`).
    Hire(HireError),
    /// The town command was refused (TWN-BLD-001; E09 T18b1): the town not in the store, not the
    /// actor's, or the hall's refusal (`h3_towns::check_build`), in that order; the treasury's
    /// is [`Self::CannotPay`], checked after them.
    Town(TownCommandError),
    /// A map Tavern's hire the game refused (GAM-DEC-010; E09 T39a): the slot, the room on the
    /// map or the Tavern's tile, in that order; the treasury's is [`Self::CannotPay`], checked
    /// after them.
    Tavern(TavernError),
    /// A market's answer refused (GAM-DEC-011, ADV-MKT-003 to -007; E08 T133b): the
    /// adventure's checks (the visit, the screen, the resources, the slot, the last creature, the
    /// entry, the price, the hero's room) and the game's (a trade's lots and a transfer's amount
    /// against the treasury, `Amount`; a transfer's recipient, `Recipient`); a purchase's
    /// treasury is [`Self::CannotPay`], checked after them.
    Market(MarketError),
    /// An Altar's sacrifice refused (GAM-DEC-012, ADV-ALT-002, -003; E08 T134a): the adventure's
    /// checks (the visit, the screen, the places, a kept artifact, an amount, the last creature,
    /// a total of 0).
    Altar(AltarError),
    /// A Hill Fort's upgrade refused (GAM-DEC-013, ADV-HFT-002; E08 T134b1): the adventure's
    /// checks (the visit, a row without an upgrade, no row that upgrades); the treasury is
    /// [`Self::CannotPay`], checked after them.
    HillFort(HillFortError),
    /// A cast on the map refused (GAM-SPL-001; E08 T142a): [`MapCastError`]'s tests in order.
    Cast(MapCastError),
}

/// Why the game refuses a cast on the adventure map (GAM-SPL-001, ADV-SPL-001; E08 T142a,
/// increment 15's design note §14, ADR-0017 point 4): its tests in order, each changing nothing
/// and drawing nothing. The cost comes before the ground's refusals as in the original (the book's
/// click tests the points, then `hero_special_ground` 0x4E4EC0 the ground: survey
/// `E08-spells.md` §4.1; E08 decision 414).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MapCastError {
    /// The hero is not one of the actor's heroes on the map (a town's garrison hero casts
    /// nothing).
    NotYourHero(HeroId),
    /// The hero may not cast the spell (`h3_heroes::can_cast_on_map`: not among its available
    /// spells, learned, a worn scroll's or a worn artifact's; no book test, D-79).
    Unknown(SpellId),
    /// The spell does not act on the map: the adventure part binds no behaviour to it (a battle
    /// spell, or an adventure spell not built: the four information spells, Town Portal).
    NotOnMap(SpellId),
    /// A target given for a spell that names none, or none for one that does.
    Target,
    /// The hero's spell points are below the cast's cost (`h3_heroes::map_spell_cost`; the
    /// original's GENRLTXT 207).
    SpellPoints,
    /// The special ground under the hero allows no spell of the spell's level (Cursed Ground's
    /// level 1, BAT-TER-004's data; the original's GENRLTXT 748).
    Ground,
    /// The hero stands in or on an anti-magic garrison (ADV-GAR-005; the original's GENRLTXT
    /// 685).
    AntiMagic,
    /// The spell's own test refused it ([`CastError`]).
    Adventure(CastError),
}

impl fmt::Display for MapCastError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotYourHero(hero) => {
                write!(f, "hero {} is not the player's on the map", hero.raw())
            }
            Self::Unknown(spell) => write!(f, "the hero cannot cast spell {}", spell.raw()),
            Self::NotOnMap(spell) => write!(f, "spell {} is not cast on the map", spell.raw()),
            Self::Target => f.write_str("the cast's target does not fit its spell"),
            Self::SpellPoints => f.write_str("the hero's spell points are below the cost"),
            Self::Ground => f.write_str("the ground allows no spell of that level"),
            Self::AntiMagic => f.write_str("no spell is cast in an anti-magic garrison"),
            Self::Adventure(error) => write!(f, "{error}"),
        }
    }
}

impl core::error::Error for MapCastError {}

impl fmt::Display for GameCommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownPlayer(p) => write!(f, "player {} is not in the game", p.raw()),
            Self::NotYourTurn(p) => write!(f, "it is not player {}'s turn", p.raw()),
            Self::DecisionOpen => f.write_str("an open decision waits for its answer"),
            Self::NoDecision => f.write_str("no decision is open"),
            Self::OfferGone => f.write_str("the offer no longer stands"),
            Self::Adventure(error) => write!(f, "{error}"),
            Self::BattleOpen => f.write_str("an open battle waits for its commands"),
            Self::NoBattle => f.write_str("no battle is open"),
            Self::NotYourSide => f.write_str("the side to act is not the player's"),
            Self::Battle(error) => write!(f, "{error}"),
            Self::NotForSystem => {
                f.write_str("the system actor issues only an unowned side's and its own commands")
            }
            Self::SystemOnly => f.write_str("only the system actor issues a system command"),
            Self::WrongAnswer => f.write_str("the answer does not fit the open decision"),
            Self::WaitingForLevelUp(p) => {
                write!(f, "the game waits for player {}'s level-up", p.raw())
            }
            Self::Manage(error) => write!(f, "{error}"),
            Self::SlotClosed => f.write_str("the University's slot is not open for the hero"),
            Self::CannotPay => f.write_str("the treasury cannot pay the price"),
            Self::Hire(error) => write!(f, "{error}"),
            Self::Town(error) => write!(f, "{error}"),
            Self::Tavern(error) => write!(f, "{error}"),
            Self::Market(error) => write!(f, "{error}"),
            Self::Altar(error) => write!(f, "{error}"),
            Self::HillFort(error) => write!(f, "{error}"),
            Self::Cast(error) => write!(f, "{error}"),
        }
    }
}

impl core::error::Error for GameCommandError {}

/// What happened in the game. The game's own events are public facts (whose turn it is and the
/// day) but a treasury's change and an offer's, which their owner alone sees (E08 T13b, T14b); an
/// adventure event carries the players who may see it (E08 T8a). An offer is named as the
/// observation names its object, by type and anchor, never by index (ADR-0016 point 6).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameEvent {
    /// A player ended its turn.
    TurnEnded {
        /// The player.
        player: PlayerId,
    },
    /// A new day started.
    DayStarted {
        /// The day, from 1.
        day: u32,
    },
    /// A new week started (GAM-CAL-002, E08 T85b): on its first day after the first week's, after
    /// [`Self::DayStarted`] and before the adventure's week step's events, since E09 T26a2 with
    /// the kind it drew (TWN-GRO-002). Public, as the original's week's message.
    WeekStarted {
        /// The week, from 1 (the first one starts with the game, without an event).
        week: u32,
        /// Its kind.
        kind: WeekKind,
    },
    /// A new month started (GAM-CAL-002, GAM-CAL-003, E08 T85b): on its first day after the first
    /// month's, after the week's events, with the kind it drew, before a creature's month's
    /// monsters' events. Public, as the original's month's message.
    MonthStarted {
        /// The month, from 1.
        month: u32,
        /// Its kind.
        kind: MonthKind,
    },
    /// A player's turn started.
    TurnStarted {
        /// The player.
        player: PlayerId,
    },
    /// A map's timed event fired at a player's turn start (GAM-CAL-004, E08 T85b): after
    /// [`Self::TurnStarted`] and the changes of its treasury (its gains as
    /// [`Self::ResourceGained`], then its losses as [`Self::ResourceLost`]), so a client shows
    /// its message. Seen by the player.
    TimedEventFired {
        /// The player.
        player: PlayerId,
        /// The event, by its message (`crate::TimedEventSetup::message`: its index in the map's
        /// list).
        event: u16,
    },
    /// Something happened on the adventure map.
    Adventure(AdventureEvent),
    /// A player's treasury gained a resource (GAM-RES-001): one event per resource of a gain,
    /// in the gain's order, after the context's events; or a surrender's price, which the other
    /// side's owner receives (GAM-BAT-005, E08 T19a); since E08 T85b a new day's income
    /// (GAM-RES-003) and a timed event's gains (GAM-CAL-004).
    ResourceGained {
        /// The player.
        player: PlayerId,
        /// The resource.
        resource: ResourceId,
        /// The amount the treasury actually gained (a gain saturates at 2³¹ − 1).
        amount: u32,
    },
    /// A visit's offer opened as a pending decision of its player (GAM-DEC-001, ADV-VIS-001):
    /// its treasury can pay the price, which the object's type gives.
    OfferOpened {
        /// The player.
        player: PlayerId,
        /// The hero whose visit made the offer.
        hero: HeroId,
        /// The object's type.
        object_type: ObjectTypeId,
        /// The object's anchor.
        anchor: Pos,
    },
    /// A visit's offer did not open: its player's treasury cannot pay the price (ADR-0017 point
    /// 10; ADV-VIS-001, the original's message); since E08 T63a also a School's choice
    /// (GAM-DEC-005, ADV-TCH-002).
    OfferRefused {
        /// The player.
        player: PlayerId,
        /// The hero whose visit made the offer.
        hero: HeroId,
        /// The object's type.
        object_type: ObjectTypeId,
        /// The object's anchor.
        anchor: Pos,
    },
    /// A player answered its open decision, which closed (GAM-DEC-001). An accepted offer's
    /// payment and effect follow.
    OfferAnswered {
        /// The player.
        player: PlayerId,
        /// The object's type.
        object_type: ObjectTypeId,
        /// The object's anchor.
        anchor: Pos,
        /// Whether the player accepted.
        accepted: bool,
    },
    /// A battle on the map started (GAM-BAT-001, E08 T19a): its heroes by side (attacker first)
    /// and the tile it is fought at. Seen by its owners.
    BattleStarted {
        /// The heroes leading the sides, by side index.
        heroes: [Option<HeroId>; 2],
        /// The tile the battle is fought at: the defender's.
        tile: Pos,
        /// Its owners (the viewer alone in a projection).
        seen_by: Viewers,
    },
    /// Something happened in a battle on the map (E08 T19a): a public event, seen by both owners;
    /// a side's, by that side's owner.
    Battle {
        /// The battle's event.
        event: h3_battle::Event,
        /// Who sees it (the viewer alone in a projection).
        seen_by: Viewers,
    },
    /// A battle on the map ended (GAM-BAT-001, E08 T19a): its heroes by side and the result; its
    /// outcome's effects follow. Seen by its owners.
    BattleEnded {
        /// The heroes that led the sides, by side index.
        heroes: [Option<HeroId>; 2],
        /// How it ended.
        result: BattleResult,
        /// Its owners (the viewer alone in a projection).
        seen_by: Viewers,
    },
    /// A hero's details changed (E08 T19a2a): a battle's outcome, one event per change in the
    /// order applied (GAM-BAT-002, GAM-BAT-004). Seen by the hero's owner.
    HeroChanged {
        /// The hero's owner.
        player: PlayerId,
        /// The hero.
        hero: HeroId,
        /// The change, as `h3_heroes` applied it.
        change: HeroChange,
    },
    /// A hero's Necromancy raised creatures into its army after its side won a battle
    /// (GAM-BAT-006, E08 T19a2b): those that joined, the upgrade at two thirds when the army was
    /// full. Seen by the hero's owner.
    CreaturesRaised {
        /// The hero's owner.
        player: PlayerId,
        /// The hero.
        hero: HeroId,
        /// The creature that joined.
        creature: CreatureTypeId,
        /// How many joined.
        count: u32,
    },
    /// A hero's level-up with two offers opened as a pending decision of its owner (GAM-DEC-002,
    /// E08 T30): the new level, the primary skill it raises and the two offers, answered `First`
    /// or `Second`. Seen by the owner.
    LevelUpOpened {
        /// The hero's owner.
        player: PlayerId,
        /// The hero.
        hero: HeroId,
        /// The new level.
        level: u8,
        /// The primary skill it raises by 1.
        primary: PrimarySkill,
        /// The offers, the dialog's left and right.
        offers: [SkillId; 2],
    },
    /// A hero gained a level in play (GAM-DEC-002, GAM-BAT-008; E08 T30): at once with one offer
    /// or none, or after its decision's answer. Seen by the owner.
    LevelGained {
        /// The hero's owner.
        player: PlayerId,
        /// The hero.
        hero: HeroId,
        /// The new level.
        level: u8,
        /// The primary skill it raised by 1.
        primary: PrimarySkill,
        /// The secondary skill it learned or advanced; none when the level offered none.
        skill: Option<SkillId>,
    },
    /// A player's treasury lost a resource a reward record takes (GAM-RES-001, ADV-RWD-001; E08
    /// T62b1), or since E08 T85b a timed event (GAM-CAL-004): one event per resource of the loss,
    /// in the record's order, after the gains; never below 0, so the amount actually taken. Seen
    /// by the owner.
    ResourceLost {
        /// The player.
        player: PlayerId,
        /// The resource.
        resource: ResourceId,
        /// The amount the treasury actually lost.
        amount: u32,
    },
    /// A player's treasury paid a resource of an accepted offer's price (GAM-RES-001): one event
    /// per resource of the price, in its order; or a surrender's price (GAM-BAT-005, E08 T19a).
    ResourcePaid {
        /// The player.
        player: PlayerId,
        /// The resource.
        resource: ResourceId,
        /// The amount paid.
        amount: u32,
    },
    /// A slot of a player's army changed by an army command (GAM-MAN-001, E08 T51a): one event per
    /// slot the command names, the stack's, then its target's, with what it holds now (a slot of
    /// an army dialog's group too, `ArmyRef::Group`: T51b). Since E09 T8b2b also each slot the
    /// garrison's merge into a town's defending visitor's army changed before the siege, the
    /// visitor's, then the garrison's (`ArmyRef::Garrison`; TWN-SIE-002; a neutral town's slots
    /// to nobody). Seen by the owner.
    StackChanged {
        /// The army's owner.
        player: PlayerId,
        /// The slot.
        at: ArmySlot,
        /// The stack there now (at `at.slot`); none: the slot is empty.
        stack: Option<ArmyStack>,
    },
    /// A hero's formation was set (GAM-MAN-001, ADV-ARM-006; E08 T51a). Seen by the owner.
    FormationSet {
        /// The hero's owner.
        player: PlayerId,
        /// The hero.
        hero: HeroId,
        /// The formation.
        formation: Formation,
    },
    /// A hero's tactics flag was set (GAM-MAN-001, ADV-ARM-006; E08 T51a). Seen by the owner.
    TacticsSet {
        /// The hero's owner.
        player: PlayerId,
        /// The hero.
        hero: HeroId,
        /// Whether the flag is on.
        on: bool,
    },
    /// A meeting of two of the player's heroes opened (GAM-DEC-003; E08 T51b), after the spells
    /// Scholar taught each (their `HeroChanged`, the visitor's first). Seen by the owner.
    MeetingOpened {
        /// The heroes' owner.
        player: PlayerId,
        /// The visitor, then the visited hero.
        heroes: [HeroId; 2],
    },
    /// An army dialog opened (GAM-DEC-004; E08 T51b): one event per stack of its group at its
    /// opening, in slot order, the first opening it: an accepted join's group its hero's army had
    /// no room for, as one stack in the group's slot 0; since E08 T62b1 a reward record's stacks
    /// that found no room, at their slots (ADV-RWD-001). Seen by the owner.
    ArmyDialogOpened {
        /// The hero's owner.
        player: PlayerId,
        /// The hero the group joined.
        hero: HeroId,
        /// A stack of the group, at its slot.
        stack: ArmyStack,
    },
    /// An own mine's Leave Guards dialog opened (GAM-DEC-004 amended, ADV-MIN-002; E08 T84a): the
    /// army dialog between the visiting hero's army and the mine's guards, which its `Close` keeps
    /// on the mine; the guards follow as [`Self::ArmyDialogOpened`], one event per stack in slot
    /// order (none for a mine without guards). Seen by the owner.
    LeaveGuardsOpened {
        /// The mine's owner.
        player: PlayerId,
        /// The visiting hero, on the mine's entrance.
        hero: HeroId,
        /// The mine's type.
        object_type: ObjectTypeId,
        /// The mine's anchor.
        anchor: Pos,
    },
    /// A map garrison's exchange opened (GAM-DEC-009, ADV-GAR-003; E08 T114a2, increment 12's
    /// design note §13): its owner's army commands name the hero's army and the garrison's
    /// (`h3_adventure::ArmyRef::MapGarrison`) until its `Close`; the garrison's stacks follow as
    /// [`Self::StackChanged`], one event per stack in slot order (none for an empty garrison), and
    /// its commands' changes as theirs. Seen by the owner.
    ExchangeOpened {
        /// The hero's owner.
        player: PlayerId,
        /// The visiting hero, on the garrison's entrance.
        hero: HeroId,
        /// The garrison's object index, which the army commands name.
        garrison: u32,
        /// The garrison's type.
        object_type: ObjectTypeId,
        /// The garrison's anchor.
        anchor: Pos,
        /// Whether its owner may take its troops back (the map's flag; the one owner rule: a
        /// non-removable garrison only takes stacks into its empty slots).
        removable: bool,
    },
    /// A visit's choice opened as a pending decision of its player (GAM-DEC-005, E08 T63a): a
    /// treasure chest's, the Arena's, or a School's when the treasury can pay its price; what it
    /// chooses between is its view ([`crate::DecisionView::Choice`]). Seen by the owner.
    ChoiceOpened {
        /// The player.
        player: PlayerId,
        /// The hero whose visit opened it.
        hero: HeroId,
        /// The object's type.
        object_type: ObjectTypeId,
        /// The object's anchor.
        anchor: Pos,
    },
    /// A University opened as a pending decision of its player (GAM-DEC-006, E08 T63a): its
    /// skills and their slots' states are its view ([`crate::DecisionView::University`]). Seen
    /// by the owner.
    UniversityOpened {
        /// The player.
        player: PlayerId,
        /// The hero whose visit opened it.
        hero: HeroId,
        /// The object's type.
        object_type: ObjectTypeId,
        /// The object's anchor.
        anchor: Pos,
    },
    /// A hire dialog opened as a pending decision of its player (GAM-DEC-008, E08 T95b) after an
    /// accepted hire question, or since E09 T27 a town's recruitment (TWN-REC-001, -002): its
    /// creatures, their counts and costs are its view ([`crate::DecisionView::Hire`]). The source
    /// is named as [`HireSeen`] says, never by an object's index (ADR-0016 point 6). Seen by the
    /// owner.
    HireOpened {
        /// The player.
        player: PlayerId,
        /// The army the bought creatures join: the visiting hero's, or a town's garrison.
        target: HireTarget,
        /// The source.
        source: HireSeen,
    },
    /// The map Tavern's decision opened for the visitor's owner (GAM-DEC-010, ADV-VIS-007; E09
    /// T39a, the town epic's design note §75): its offers are the owner's tavern heroes
    /// ([`crate::GameObservation::tavern`]), its answers [`TavernAnswer`]'s. Seen by the owner.
    TavernOpened {
        /// The visitor's owner.
        player: PlayerId,
        /// The visiting hero.
        hero: HeroId,
        /// The Tavern's type.
        object_type: ObjectTypeId,
        /// The Tavern's anchor.
        anchor: Pos,
    },
    /// A market's decision opened for the visitor's owner (GAM-DEC-011, ADV-MKT-001; E08 T133b,
    /// increment 14's design note §13, §17): what its window offers is its view
    /// ([`crate::DecisionView::Market`]), its answers [`MarketAnswer`]'s. Seen by the owner.
    MarketOpened {
        /// The visitor's owner.
        player: PlayerId,
        /// The visiting hero.
        hero: HeroId,
        /// The market's type.
        object_type: ObjectTypeId,
        /// The market's anchor.
        anchor: Pos,
    },
    /// An Altar of Sacrifice's decision opened for the visitor's owner (GAM-DEC-012, ADV-ALT-001;
    /// E08 T134a, increment 14's design note §14, §17), at the visit and again after the
    /// level-ups a sacrifice opened (decision 371): what its window offers is its view
    /// ([`crate::DecisionView::Altar`]), its answers [`AltarAnswer`]'s. Seen by the owner.
    AltarOpened {
        /// The visitor's owner.
        player: PlayerId,
        /// The visiting hero.
        hero: HeroId,
        /// The Altar's type.
        object_type: ObjectTypeId,
        /// The Altar's anchor.
        anchor: Pos,
    },
    /// A Hill Fort's decision opened for the visitor's owner (GAM-DEC-013, ADV-HFT-001; E08
    /// T134b1, increment 14's design note §15, §17): its rows are its view
    /// ([`crate::DecisionView::HillFort`]), its answers [`HillFortAnswer`]'s. Seen by the owner.
    HillFortOpened {
        /// The visitor's owner.
        player: PlayerId,
        /// The visiting hero.
        hero: HeroId,
        /// The Hill Fort's type.
        object_type: ObjectTypeId,
        /// The Hill Fort's anchor.
        anchor: Pos,
    },
    /// The player's meeting or army dialog closed (its `Close`; E08 T51b); since E08 T63a also
    /// its choice once answered (the answer's payment and effects follow) and its University
    /// left; since E08 T95b its hire dialog closed or a single creature's bought; since E09 T39a
    /// a map Tavern closed or its hire made; since E08 T133b a market left; since T134a an Altar,
    /// since T134b1 a Hill Fort. Seen by the owner.
    DecisionClosed {
        /// Its owner.
        player: PlayerId,
    },
    /// A stack left in a closed army dialog's group, lost (GAM-DEC-004; E08 T51b), told after
    /// the dialog's `DecisionClosed`, one per stack in slot order. Seen by the owner.
    GroupLost {
        /// The hero's owner.
        player: PlayerId,
        /// The hero the group had joined.
        hero: HeroId,
        /// The lost stack.
        stack: ArmyStack,
    },
    /// A player's view of a town changed (TWN-VEW-001 to -004; E09 T8a2's Capitol, since E09
    /// T18b2a each viewer's whole view, SRV-Q26): the town's facts as `player` now sees it
    /// (`TownView`'s), followed by a [`Self::TownStackSeen`] for each stack its garrison's level
    /// shows and a [`Self::TownIncomeSeen`] for each line of the top level's income; its view
    /// after them is the told one, its anchor, faction and name as they were. Told after the
    /// change, to each player that sees the town (an explored tile of it) and whose view of it
    /// the change made differ: a build (after its `TownBuilt`; a new Tavern raises its owner's
    /// garrison level of every other town it sees, TWN-VEW-002), a capture (after its
    /// `TownCaptured` and `TilesRevealed`: the owner, the day's mark, the Capitol's limit, the
    /// garrison emptied, a Tavern's two owners' levels), a siege's merge (after its
    /// `StackChanged`), a garrison's battle (after its `BattleEnded`), the day's start (the marks
    /// cleared). Not told: a change of Visions' reach (`TWN-STUB-13`). Seen by `player`.
    TownChanged {
        /// The player told: the viewer.
        player: PlayerId,
        /// The town.
        town: TownId,
        /// Its owner, none for a neutral town.
        owner: Option<PlayerId>,
        /// Its fort level.
        fort: FortLevel,
        /// Whether it has a Capitol (a standing building of role `hall` 4).
        capitol: bool,
        /// Whether it was built in today (TWN-VEW-004).
        built_today: bool,
        /// The viewer's level of its garrison (`GarrisonView::level`).
        level: u8,
        /// Whether the viewer sees its garrison by slot (`GarrisonView::army`, the owner's team):
        /// then each [`Self::TownStackSeen`] gives its slot, and the garrison by slot is those
        /// stacks with their slots and counts.
        slots: bool,
        /// The hall level of the popup's top level (`TownOwnerView::hall`), for a viewer at
        /// `OWNER_LEVEL`; none for every other viewer.
        hall: Option<u8>,
        /// Whether a hero stands in its garrison, the popup's top level's mark
        /// (`TownOwnerView::garrison_hero`, E09 T28b2), for a viewer at `OWNER_LEVEL`; false for
        /// every other viewer.
        garrison_hero: bool,
        /// Its buildings, for its owner alone (`TownView::buildings`).
        buildings: Option<TownBuildingsView>,
    },
    /// A stack of a town's garrison as the viewer's level shows it (TWN-VEW-002, -003; E09
    /// T18b2a), told after the viewer's [`Self::TownChanged`], one per stack in slot order. Seen
    /// by `player`.
    TownStackSeen {
        /// The player told: the viewer.
        player: PlayerId,
        /// The town.
        town: TownId,
        /// The stack as the level shows it.
        stack: GarrisonStackView,
        /// Its slot, for a viewer that sees the garrison by slot (`TownChanged::slots`).
        slot: Option<u8>,
    },
    /// A line of a town's day's income as the popup's top level shows it (TWN-VEW-003; E09
    /// T18b2a), told after the viewer's [`Self::TownChanged`] and its stacks, one per resource
    /// in the core's order (`TownOwnerView::income`). Seen by `player`.
    TownIncomeSeen {
        /// The player told: the viewer.
        player: PlayerId,
        /// The town.
        town: TownId,
        /// The resource.
        resource: ResourceId,
        /// Its amount a day.
        amount: u32,
    },
    /// A player's tavern hero in a slot changed (TWN-TAV-002; E09 T37a, design note §75,
    /// decisions 134, 135): the slot's new hero as `player` sees it without its lists
    /// ([`crate::TavernHeroView::hero`]), none when the slot was emptied, followed by a
    /// [`Self::TavernPartSeen`] for each part of its view ([`crate::TavernHeroView::parts`]); its
    /// view after them is the told one. Told after the week's release and fill, each changed slot
    /// in player then slot order, after the towns' week, before the adventure's week step. The
    /// game's start's fill is the first observation's. Seen by `player`.
    TavernOffered {
        /// The player told: the slot's owner.
        player: PlayerId,
        /// The slot: 0 or 1.
        slot: u8,
        /// Its hero, none for an emptied slot.
        hero: Option<TavernHero>,
    },
    /// A part of a tavern hero's view (TWN-TAV-002; E09 T37a, decision 135), told after its
    /// slot's [`Self::TavernOffered`], one per part in [`TavernPart`]'s order. Seen by `player`.
    TavernPartSeen {
        /// The player told: the slot's owner.
        player: PlayerId,
        /// The slot: 0 or 1.
        slot: u8,
        /// The part.
        part: TavernPart,
    },
    /// A player hired its tavern hero (TWN-TAV-003; E09 T38a1, design note §75, ADR-0017 point 2):
    /// the hero placed on `pos`, a town's entrance, as its visitor. Told after the price's
    /// `ResourcePaid`, before the owner's [`Self::HiredPartSeen`] and the slot's
    /// [`Self::TavernOffered`] (its replacement, or its emptying). Seen by its owner and by every
    /// player that sees the tile (`seen_by`, as a step's `seen_to`, ADV-FOG-004, -005), each
    /// projected naming the viewer alone; the slot to the owner alone (none to another viewer).
    /// A viewer's fold adds the hero on the map (owner, tile, type; on land); its owner's moves
    /// the slot's view into its heroes (details, army, formation, tactics), completed by the parts,
    /// its points by the command's `MovementChanged`.
    HeroHired {
        /// The hirer: the hero's owner.
        player: PlayerId,
        /// The hero.
        hero: HeroId,
        /// Its type: identity and class.
        hero_type: HeroTypeId,
        /// The tavern slot it was hired from, for its owner alone.
        slot: Option<u8>,
        /// Its tile.
        pos: Pos,
        /// The players that see it (the owner among them).
        seen_by: Viewers,
    },
    /// A part of a hired hero's details its tavern slot's view did not show (E09 T38a1, decision
    /// 141): told after its [`Self::HeroHired`], one per part in [`HiredPart`]'s order, so that
    /// the owner's fold of the slot's view gives the hero's details whole; since E08 T141a also
    /// after a [`Self::PrisonerFreed`]'s parts, for the freed hero. Seen by `player`.
    HiredPartSeen {
        /// The hirer: the hero's owner.
        player: PlayerId,
        /// The hero.
        hero: HeroId,
        /// The part.
        part: HiredPart,
    },
    /// A Prison's prisoner freed for the visitor's owner (ADV-PRI-002, GAM-SET-001; E08 T141a,
    /// increment 15's design note §15, §16; survey `E08-spells.md` §2.2 point 3): the hero placed
    /// on the Prison's visitable tile, last in the owner's list, after the Prison's
    /// `ObjectRemoved`. Seen by its owner and by every player that sees the tile (`seen_by`, as a
    /// hire's [`Self::HeroHired`]), each projected naming the viewer alone; the hero's `view` (the
    /// tavern hero screen's parts, decision 134's) for the owner alone, its lists following as
    /// [`Self::PrisonerPartSeen`] and its other parts as [`Self::HiredPartSeen`], its points by
    /// the command's `MovementChanged`. A viewer's fold adds the hero on the map (owner, tile,
    /// type; on land); its owner's makes its view its hero's.
    PrisonerFreed {
        /// The visitor's owner: the hero's.
        player: PlayerId,
        /// The hero.
        hero: HeroId,
        /// Its type: identity and class.
        hero_type: HeroTypeId,
        /// Its tile.
        pos: Pos,
        /// The players that see it (the owner among them).
        seen_by: Viewers,
        /// The hero as its owner sees it first, for its owner alone.
        view: Option<TavernHero>,
    },
    /// A part of a freed prisoner's view that is one of a list (E08 T141a; [`TavernPart`]'s,
    /// in its order: the army's stacks by slot, the secondary skills, the worn artifacts by
    /// position, the backpack's first places): told after its [`Self::PrisonerFreed`], before its
    /// [`Self::HiredPartSeen`]. Seen by `player`.
    PrisonerPartSeen {
        /// The hero's owner.
        player: PlayerId,
        /// The hero.
        hero: HeroId,
        /// The part.
        part: TavernPart,
    },
    /// A Prison's visit found the visitor's owner at the town part's most heroes on the map
    /// (ADV-PRI-002; E08 T141a, the original's ADVEVENT 103): nothing changed. Seen by `player`.
    PrisonFull {
        /// The visitor's owner.
        player: PlayerId,
        /// The visiting hero.
        hero: HeroId,
    },
    /// A Prison's visit found a prisoner that is no prisoner any more (ADV-PRI-002; E08 T141a, the
    /// original's ADVEVENT 104; unreachable without campaigns): the Prison taken off the map, its
    /// `ObjectRemoved` before. Seen by `player`.
    PrisonEmptied {
        /// The visitor's owner.
        player: PlayerId,
        /// The visiting hero.
        hero: HeroId,
    },
    /// A spell cast on the adventure map (GAM-SPL-001, ADV-SPL-001; E08 T142a, increment 15's
    /// design note §16): accepted, its effect made, before the spell points' `HeroChanged`; the
    /// changed day spells follow after the command (`DaySpellsChanged`). Seen by the caster's
    /// owner (a viewer sees the effects only, §13).
    SpellCast {
        /// The caster's owner.
        player: PlayerId,
        /// The caster.
        hero: HeroId,
        /// The core spell.
        spell: SpellId,
        /// The mastery it was cast at on the hero's ground (0 to 3).
        mastery: u8,
        /// The spell points it cost (`h3_heroes::map_spell_cost`).
        cost: u16,
    },
    /// An accepted cast on the adventure map failed, its spell points spent (GAM-SPL-001,
    /// ADV-SPL-003, ADV-SPL-004; E08 T143a, increment 15's design note §14, §16): a charged
    /// failure, told after its [`Self::SpellCast`] and before the points' `HeroChanged` (the
    /// original's GENRLTXT 336 to 338). Seen by the caster's owner.
    CastFailed {
        /// The caster's owner.
        player: PlayerId,
        /// The caster.
        hero: HeroId,
        /// The core spell.
        spell: SpellId,
        /// Why it failed.
        reason: CastFailure,
    },
    /// A Den of Thieves' visit showed the visitor's owner the Thieves' Guild at everything
    /// (ADV-VIS-008, TWN-TAV-007; E09 T39b2, design note §74, §75): its level
    /// ([`crate::DEN_LEVEL`]), followed by a [`Self::GuildPartSeen`] for each part of its view
    /// ([`crate::ThievesGuildView::parts`]); the view they make is the told one, a snapshot (the
    /// guild is not in the observation). Told after the visit's step, every visit alike (a
    /// `Revisit` too). Seen by `player`.
    ThievesGuildSeen {
        /// The player told: the visitor's owner.
        player: PlayerId,
        /// The guild's level.
        level: u8,
    },
    /// A part of the Thieves' Guild a Den of Thieves' visit shows (TWN-TAV-008, -009; E09
    /// T39b2), told after its [`Self::ThievesGuildSeen`], one per part in [`GuildPart`]'s order.
    /// Seen by `player`.
    GuildPartSeen {
        /// The player told: the visitor's owner.
        player: PlayerId,
        /// The part.
        part: GuildPart,
    },
    /// A player's view of a map garrison changed (ADV-GAR-006; E08 T114b, increment 12's design
    /// note §10 and §15; SRV-Q26): the garrison's facts as `player` now sees it
    /// (`MapGarrisonView`'s), followed by a [`Self::GarrisonStackSeen`] for each stack its level
    /// shows; its view after them is the told one. Told after the change, to each player that sees
    /// the garrison (an explored tile of it) and whose view of it the change made differ: its
    /// battle's end (after its `BattleEnded`: the survivors, or none after the visitor's win), an
    /// exchange's army command (after its `StackChanged`), its capture (after its `ObjectFlagged`
    /// and `TilesRevealed`, before its `ExchangeOpened`: every viewer's), and a change of a
    /// viewer's level by its Taverns (a Tavern's build, a town's capture: after the towns'
    /// `TownChanged`). Not told: a change of Visions' reach (`TWN-STUB-13`). Seen by `player`.
    GarrisonChanged {
        /// The player told: the viewer.
        player: PlayerId,
        /// The garrison's type.
        object_type: ObjectTypeId,
        /// Its anchor.
        anchor: Pos,
        /// Its owner, none for a neutral garrison.
        owner: Option<PlayerId>,
        /// The viewer's level of its army (`GarrisonView::level`).
        level: u8,
        /// Whether the viewer sees its army by slot (`GarrisonView::army`, the owner's team):
        /// then each [`Self::GarrisonStackSeen`] gives its slot, and the army by slot is those
        /// stacks with their slots and counts.
        slots: bool,
        /// Its removable flag, for a viewer of its owner's team (`MapGarrisonView::removable`).
        removable: Option<bool>,
    },
    /// A stack of a map garrison's army as the viewer's level shows it (ADV-GAR-006; E08 T114b),
    /// told after the viewer's [`Self::GarrisonChanged`], one per stack in slot order. Seen by
    /// `player`.
    GarrisonStackSeen {
        /// The player told: the viewer.
        player: PlayerId,
        /// The garrison's type.
        object_type: ObjectTypeId,
        /// Its anchor.
        anchor: Pos,
        /// The stack as the level shows it.
        stack: GarrisonStackView,
        /// Its slot, for a viewer that sees the army by slot (`GarrisonChanged::slots`).
        slot: Option<u8>,
    },
    /// A building was built in a player's town (TWN-BLD-001, TWN-BLD-005; E09 T18b1, the town
    /// epic's design note §31): by the owner's build command, after its price's `ResourcePaid`
    /// events. Seen by the owner.
    TownBuilt {
        /// The town's owner.
        player: PlayerId,
        /// The town.
        town: TownId,
        /// The building built (of the town's faction's record).
        building: BuildingId,
        /// The town event that built it, by its place in the town's list (`h3_towns::TownStatic`'s
        /// events); none for the owner's command.
        event: Option<u16>,
    },
    /// A town event fired at its town's owner's turn start (TWN-EVT-001, TWN-EVT-002; E09 T18c,
    /// the town epic's design note §30, §31): after the map's timed events and the changes of the
    /// owner's treasury it made (its gains as [`Self::ResourceGained`], then its losses as
    /// [`Self::ResourceLost`]), before its buildings' [`Self::TownBuilt`], so a client shows its
    /// message. Seen by the owner.
    TownEventFired {
        /// The town's owner.
        player: PlayerId,
        /// The town.
        town: TownId,
        /// The event, by its place in the town's list (`h3_towns::TownStatic`'s events), which
        /// names its message (the text is the client's).
        event: u16,
    },
}

/// Receiver of the game's events.
pub trait Sink {
    /// Whether the sink keeps what it receives, passed to a battle's sink (`h3_battle::Sink`'s
    /// own: work done only to report something, a walker's path hex by hex, is skipped for a sink
    /// that does not; the state is the same either way; E08 T19b1).
    const RECORDS: bool = true;

    /// Receives one event.
    fn emit(&mut self, event: GameEvent);

    /// Receives a battle as it ended, when the command that ended it closes it, before its
    /// outcome's events (`BattleEnded` first): its state, owners, heroes and tile (SRV-Q27 (1),
    /// the server lane's `SRV-STUB-14`). A server sends each owner the battle as its side saw it
    /// last (`h3_battle::observe` of its side: the own hero's Eagle Eye picks, which no event
    /// shows), as a battle session's last snapshot. Server-only, as [`GameState::battle`]'s state
    /// is: never in an event. The default ignores it, so the state keeps no closed battle and a
    /// fork pays nothing for it.
    fn battle_ended(&mut self, battle: &OpenBattle) {
        let _ = battle;
    }
}

/// A sink that drops every event.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoopSink;

impl Sink for NoopSink {
    const RECORDS: bool = false;

    #[inline]
    fn emit(&mut self, _event: GameEvent) {}
}

impl Sink for alloc::vec::Vec<GameEvent> {
    fn emit(&mut self, event: GameEvent) {
        self.push(event);
    }
}

/// The adventure context's sink inside a game's: its events become game events.
pub(crate) struct AdventureSink<'a, S>(pub(crate) &'a mut S);

impl<S: Sink> h3_adventure::Sink for AdventureSink<'_, S> {
    fn emit(&mut self, event: AdventureEvent) {
        self.0.emit(GameEvent::Adventure(event));
    }
}

/// Applies a command of `actor` to the game (ADR-0017 point 1): validated against the state and
/// the trusted actor, then applied; the events go to `sink`. The system actor issues only an
/// unowned side's battle commands ([`Actor::System`], E08 T20b2) and its own for a player
/// ([`GameCommand::System`], E16 M5 T14), which no player issues. While a level-up is open the
/// game waits for its owner: every other actor's command is rejected (E08 T30, GAM-DEC-002), but
/// the system's command for the owner, judged as the owner's (its default answer past the hold).
/// After the command the rests whose owners have no open decision run (GAM-DEC-007, E08 T63b).
/// Each battle the command ends goes to the sink as it ended ([`Sink::battle_ended`]; SRV-Q27
/// (1)). Last, a sink that keeps events ([`Sink::RECORDS`]) is told each hero's movement points
/// and day's maximum that the command changed (`AdventureEventKind::MovementChanged`, to its
/// owner; SRV-Q26 (b)) and since E08 T115a its battle bonuses and timed marks
/// (`AdventureEventKind::BonusChanged`, `OnceMarkChanged`, `TimedMarkChanged`; `h3_adventure::
/// OwnHeroes`).
// rule: GAM-DEC-002, GAM-DEC-007, GAM-SYS-002
///
/// # Errors
///
/// A [`GameCommandError`]; the state is then unchanged and no event was emitted.
pub fn apply<S: Sink>(
    state: &mut GameState,
    rules: &GameRuleset,
    actor: Actor,
    command: GameCommand,
    sink: &mut S,
) -> Result<(), GameCommandError> {
    state.check_held(held_as(actor, &command))?;
    // What the command changes of the heroes' movement (SRV-Q26 (b)) and, since E08 T115a, of
    // their battle bonuses and timed marks is told after it.
    let own = S::RECORDS.then(|| state.adventure().own_heroes());
    run(state, rules, actor, command, sink)?;
    state.run_rests(rules, sink);
    if let Some(own) = own {
        own.tell(state.adventure(), &mut AdventureSink(sink));
    }
    Ok(())
}

/// The actor the game's hold judges `actor`'s `command` by: a system command for a player as
/// the player's own (ADR-0017 point 2, E16 M5 T14: the level-up's owner's default answer past
/// the hold).
fn held_as(actor: Actor, command: &GameCommand) -> Actor {
    match (actor, command) {
        (Actor::System, GameCommand::System(system)) => Actor::Player(system.player()),
        _ => actor,
    }
}

/// [`apply`]'s command, checked by the game's hold.
fn run<S: Sink>(
    state: &mut GameState,
    rules: &GameRuleset,
    actor: Actor,
    command: GameCommand,
    sink: &mut S,
) -> Result<(), GameCommandError> {
    let (player, command) = match (actor, command) {
        (Actor::System, GameCommand::Battle(command)) => {
            return state.system_battle_command(rules, command, sink);
        }
        (Actor::System, GameCommand::System(command)) => {
            return crate::system::run(state, rules, command, sink);
        }
        (Actor::System, _) => return Err(GameCommandError::NotForSystem),
        (Actor::Player(player), command) => (player, command),
    };
    match command {
        GameCommand::EndTurn => state.end_turn(rules, player, sink),
        GameCommand::System(_) => Err(GameCommandError::SystemOnly),
        GameCommand::Answer(answer) => state.answer(rules, player, answer, sink),
        GameCommand::Battle(command) => state.battle_command(rules, player, command, sink),
        GameCommand::Manage(command) => state.manage(rules, player, command, sink),
        GameCommand::Town(command) => {
            state.check_ready(player)?;
            crate::town::command(state, rules, player, command, sink)
        }
        GameCommand::Cast {
            hero,
            spell,
            target,
        } => {
            state.check_ready(player)?;
            state.cast(rules, player, (hero, spell, target), sink)
        }
        GameCommand::Adventure(command) => {
            state.check_ready(player)?;
            // E08 T84b2 (ADV-SHY-003; increment 9's design note §14): `BuildBoat`'s checks and the
            // treasury's before anything changes; its price is taken after the boat is built.
            let price = match &command {
                AdventureCommand::BuildBoat {
                    object_type,
                    anchor,
                } => Some(state.boat_price(rules, player, *object_type, *anchor)?),
                AdventureCommand::MoveHero { .. } | AdventureCommand::Revisit { .. } => None,
            };
            // The acting player's stream (ADR-0017 point 4; E08 T105a1): a visit's draw, a
            // jump's exit, comes from it, after the whole command is checked. The day of the week
            // is the calendar's (E08 T115b: the Idol's and the Temple's amounts).
            let weekday = state.weekday(rules);
            let (adventure, heroes, draws) = state.adventure_for(player);
            let outcome = adventure
                .apply(
                    rules.core(),
                    rules.adventure(),
                    heroes,
                    (player, weekday, draws),
                    &command,
                    &mut AdventureSink(sink),
                )
                .map_err(GameCommandError::Adventure)?;
            if let Some(price) = price {
                state.pay_price(player, price, sink);
            }
            state.settle_outcome(rules, player, outcome, sink);
            Ok(())
        }
    }
}
