//! The whole game's entry point (ADR-0017 point 1, ADR-0004): actors, the game's commands, its
//! events and their sink. A game command wraps a context's command or is the game's own; the game
//! checks turn and phase, the context its rules (E08 T6b: `EndTurn`; T7b: the adventure's
//! `MoveHero`; `observe` and `project` are in `observe.rs`, T8b1; `legal_commands` in
//! `legal.rs`, T8b2a; T14b: the pending decisions and their answers, in `state.rs`; T19a: battles,
//! in `state/battle.rs`; T30: the level-ups, in `state/levels.rs`; T51a: the management commands,
//! [`Manage`], in `state/manage.rs`).

use core::fmt;

use h3_adventure::{
    AdventureCommand, AdventureCommandError, AdventureEvent, ArmySlot, ObjectTypeId, Pick, Pos,
    Viewers,
};
use h3_content::{CreatureTypeId, Formation, ResourceId, SkillId};
use h3_contracts::{ArmyStack, BattleResult, HeroId, PlayerId};
use h3_heroes::{Choice, HeroChange, PrimarySkill};

use crate::manage::{Manage, ManageError};
use crate::{GameRuleset, GameState};

/// Who issues a command (ADR-0017 point 2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Actor {
    /// A player, human or bot: the identity the session layer vouches for, never one taken from
    /// the command's payload.
    Player(PlayerId),
    /// The trusted layer that hosts the game (the server's session layer, the env), never a
    /// player (E08 T20b2): it plays a battle's unowned side (a wandering monster's) with a policy
    /// of its own, and commands no owned side. Its other commands (ending a player's turn,
    /// answering a decision with its default, removing a player) come with the server lane's M5.
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
    /// Closes a meeting (GAM-DEC-003) or an army dialog (GAM-DEC-004; increment 6's `Decline`).
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
    /// commands (E08 T20b2).
    NotForSystem,
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
    /// A University's purchase the treasury cannot pay (GAM-DEC-006, E08 T63a).
    CannotPay,
}

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
                f.write_str("the system actor issues only an unowned side's commands")
            }
            Self::WrongAnswer => f.write_str("the answer does not fit the open decision"),
            Self::WaitingForLevelUp(p) => {
                write!(f, "the game waits for player {}'s level-up", p.raw())
            }
            Self::Manage(error) => write!(f, "{error}"),
            Self::SlotClosed => f.write_str("the University's slot is not open for the hero"),
            Self::CannotPay => f.write_str("the treasury cannot pay the price"),
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
    /// A player's turn started.
    TurnStarted {
        /// The player.
        player: PlayerId,
    },
    /// Something happened on the adventure map.
    Adventure(AdventureEvent),
    /// A player's treasury gained a resource (GAM-RES-001): one event per resource of a gain,
    /// in the gain's order, after the context's events; or a surrender's price, which the other
    /// side's owner receives (GAM-BAT-005, E08 T19a).
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
    /// T62b1): one event per resource of the loss, in the record's order, after the gains; never
    /// below 0, so the amount actually taken. Seen by the owner.
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
    /// an army dialog's group too, `ArmyRef::Group`: T51b). Seen by the owner.
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
    /// The player's meeting or army dialog closed (its `Close`; E08 T51b); since E08 T63a also
    /// its choice once answered (the answer's payment and effects follow) and its University
    /// left. Seen by the owner.
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
}

/// Receiver of the game's events.
pub trait Sink {
    /// Whether the sink keeps what it receives, passed to a battle's sink (`h3_battle::Sink`'s
    /// own: work done only to report something, a walker's path hex by hex, is skipped for a sink
    /// that does not; the state is the same either way; E08 T19b1).
    const RECORDS: bool = true;

    /// Receives one event.
    fn emit(&mut self, event: GameEvent);
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
/// unowned side's battle commands ([`Actor::System`], E08 T20b2). While a level-up is open the
/// game waits for its owner: every other actor's command is rejected (E08 T30, GAM-DEC-002).
/// After the command the rests whose owners have no open decision run (GAM-DEC-007, E08 T63b).
// rule: GAM-DEC-002, GAM-DEC-007
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
    state.check_held(actor)?;
    run(state, rules, actor, command, sink)?;
    state.run_rests(rules, sink);
    Ok(())
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
        (Actor::System, _) => return Err(GameCommandError::NotForSystem),
        (Actor::Player(player), command) => (player, command),
    };
    match command {
        GameCommand::EndTurn => state.end_turn(rules, player, sink),
        GameCommand::Answer(answer) => state.answer(rules, player, answer, sink),
        GameCommand::Battle(command) => state.battle_command(rules, player, command, sink),
        GameCommand::Manage(command) => state.manage(rules, player, command, sink),
        GameCommand::Adventure(command) => {
            state.check_ready(player)?;
            let (adventure, heroes) = state.adventure_with_heroes();
            let outcome = adventure
                .apply(
                    rules.core(),
                    rules.adventure(),
                    heroes,
                    player,
                    &command,
                    &mut AdventureSink(sink),
                )
                .map_err(GameCommandError::Adventure)?;
            state.settle_outcome(rules, player, outcome, sink);
            Ok(())
        }
    }
}
