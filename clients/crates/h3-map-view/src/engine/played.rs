//! A record's own walks and turns (E24 M4 T4a2, *provisional*; `docs/epics/E24-design.md`
//! §5.13): the record folded from its first observation (`GameObservation::fold` each event,
//! `reveal` each reveal) and each of the viewer's own commands turned into the timeline's facts,
//! as a recording gives them.
//!
//! - **A walk** (its `MoveHero` echoed, `GameAccepted`): the path's tiles, the points its hero's
//!   `MovementChanged` left and the tiles the command's `GameReveal` explored for the viewer, as a
//!   [`Move`] closed by the next `GameTurn`. A record tells neither a step's points nor a step's
//!   tiles (MCL-Q39): the walk's points at its last step, its whole reveal at its first (MCL-Q39
//!   (c)).
//! - **A turn's start** (each of the viewer's `TurnStarted`: its turns after the first, and since
//!   E24 M4 T6 its first where the record starts in another player's turn, [`Played::waits`]): a
//!   live part's `[[turn]]` table as a recording's (`crate::live::Turn::read`): the kingdom by the panels
//!   (`super::panels`) of the observation folded to it, the date's window at its first frame
//!   (`player_turn_start` 0x4CC7D0 asks it, design §5.1), the week's and the month's kind as the
//!   fold holds them, a normal one's name the client's (MCL-Q40 (b): the week's number less one
//!   modulo the names, the month's likewise; the original draws it). The table holds no
//!   millisecond (`at` 0) and no heroes or boats: nothing moves out of the viewer's own turns in
//!   T4a2's games (the other players' turns are T4c's). Since E24 M4 T4b2 the table is built once
//!   the message's events are folded: the engine tells the heroes' new points after
//!   `TurnStarted`.
//! - **A visit** (E24 M4 T4b1, design §5.14): each of the viewer's `MoveHero` echoed as a
//!   [`Visit`] at its path's last tile (the visited object's entrance, or its own tile visited
//!   from beside) with the command's events, then each answer the record gives after it with its
//!   own ([`super::visits`] tells what it shows).
//! - **A boat** (E24 M4 T4b2, design §5.15): a walk's `Boarded` or `Landed` its [`Move::sea`],
//!   the timeline's boarding or landing at its last step in place of Part A's boats and water.
//! - **A level-up** ([`Leveled`]): a `LevelUpOpened` (two offers) or a `LevelGained` without one
//!   (none or one), its dialog's facts as a recording's `[[level_up]]` gives them
//!   ([`crate::levels::LevelUp`]: the hero's type, the level, the primary skill, each offer's
//!   skill and the level it reaches by the hero's skills then) and the offer the record took.
//! - **Another player's turn** (E24 M4 T4c, design §5.16): its `TurnStarted` to its `TurnEnded`,
//!   the steps of its heroes the record shows (a step the viewer sees, by its start or its end
//!   tile or a neighbour of either explored: the engine's `seen_by` is the original's
//!   `step_seen`), each hero's run of steps an [`OtherWalk`]; a turn with any is a computer's turn
//!   in sight, a `[[turn]]` table of kind 5 (the computer's colour, the sand and the glass at 0)
//!   among the viewer's ([`OtherTurn::turn`]). The record tells neither the computer's heroes'
//!   points nor kind 5's value (MCL-Q43: the value 1 000, no points left) nor a step's end the
//!   viewer does not see (MCL-Q44: none of its tiles), nor which of its heroes take a turn
//!   without a step in sight (MCL-Q45). Since E24 M4 T6 (the reviewer's R1-1, R1-3, R1-5): a turn
//!   under way at the record's start or at its end is read too ([`Played::waits`],
//!   [`OtherTurn::open`]); each command's steps a move of their own ([`OtherEnd`]), with what the
//!   command then takes, removes or flags; a hero the record's start does not show is left out
//!   (MCL-Q53: the record tells nothing of its look), its commands' results as a stepless
//!   command's ([`Played::between`]).
//! - **The viewer's commands** ([`Command`], E24 M4 T4c): each one echoed, in order, for a record
//!   played without a recording (`crate::director`).

use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

use h3_adventure::{ObjectKind, ObjectTypeId};
use h3_game::GameRuleset;
use h3_kernel::Handle;
use h3_map::model::Pos;
use h3_protocol::wire::ServerMessage;
use h3_protocol::wire::game::{
    AdventureCommand, AdventureEvent, Answer, Boat, GameCommand, GameEvent, GameObservation, Month,
    Pos as WirePos, PrimarySkill, SystemCommand, Week,
};
use toml::{Table, Value};

use super::{Loaded, Refused, int, panels, row};
use crate::art::MapArt;
use crate::levels::{LevelUp, Offer};
use crate::live::{Facing, LiveBoat};
use crate::timeline::{Move, Sea};
use crate::turn::{MONTH_NAMES, WEEK_NAMES};

/// The date's window (`Status`'s kind 1) at its first frame.
const DATE_WINDOW: i64 = 1;

/// A record's own walks and turns' starts, in order.
#[derive(Debug, Clone, PartialEq)]
pub struct Played {
    /// The viewer's walks (the timeline's `Start::moves`).
    pub moves: Vec<Move>,
    /// Where each walk left its hero (E24 M4 T4b1: its last step's tile), none where it took no
    /// step (a visit from beside).
    pub ends: Vec<Option<Pos>>,
    /// The viewer's turns' starts after the first (since E24 M4 T6 its first too where the record
    /// starts in another player's turn, [`Played::waits`]) and the other players' turns in sight,
    /// each a `[[turn]]` table.
    pub turns: Vec<Value>,
    /// The viewer's heroes in its kingdom's list as each of [`Played::turns`] starts, by their
    /// record ids, slot by slot (E24 M4 T6, the reviewer's R1-4: the next day's selection and
    /// choices follow the record's lists, a hero lost or hired; a computer's turn's the viewer's
    /// then).
    pub lists: Vec<Vec<u16>>,
    /// The record starts in another player's turn (E24 M4 T6, the reviewer's R1-1: its viewer not
    /// acting at its first observation, as every player's record starts at the game's start): the
    /// map plays the turn's change to the first of [`Played::turns`] at its start.
    pub waits: bool,
    /// The viewer's visits (E24 M4 T4b1), one a walk, in order.
    pub visits: Vec<Visit>,
    /// The viewer's heroes' level-ups (E24 M4 T4b1), in order: each visit's its own
    /// ([`Visit::levels`]; since E24 M4 T6, the reviewer's R2-3, the record's start's experience
    /// is no longer followed through the tables' thresholds).
    pub levels: Vec<Leveled>,
    /// Each of [`Played::moves`]' hero, the record's id (E24 M4 T4c: a hero's walks chain from
    /// where its last left it).
    pub walkers: Vec<u16>,
    /// The viewer's commands, in order (E24 M4 T4c), but a battle's (E24 M4 T6, R5-6: the battle
    /// handed over whole takes none of the map's time).
    pub commands: Vec<Command>,
    /// The other players' turns in sight (E24 M4 T4c), in order.
    pub others: Vec<OtherTurn>,
    /// The heroes the record's start shows, by their ids, on their tiles (E24 M4 T4c: a record's
    /// heroes matched with the live part's).
    pub shown: Vec<(u16, Pos)>,
    /// The viewer's heroes' places in its kingdom's list at the start, by their ids (E24 M4 T4c:
    /// the panels' order, their types'; a hero hired later not listed).
    pub slots: Vec<(u16, usize)>,
    /// What other players' commands without a step in sight changed (E24 M4 T4c2: a pickup from
    /// beside, `ObjectRemoved` alone; MCL-Q46 (c), *provisional*: the record does not tell the
    /// hero; since E24 M4 T6, R1-5, an object flagged and a hero lost too, and a left-out hero's,
    /// MCL-Q53), each with the place among [`Played::turns`] of the next turn table: changed as
    /// that turn starts, nothing shown (its pickups off at once with the rest).
    pub between: Vec<(usize, OtherEnd)>,
}

/// One of the viewer's commands in a record (E24 M4 T4c, *provisional*), as a record played
/// without a recording gives it to the map (`crate::director`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    /// A hero's walk (`MoveHero`): the hero, the path's last tile.
    Walk {
        /// The record's hero id.
        hero: u16,
        /// The path's last tile.
        to: Pos,
    },
    /// End Turn.
    EndTurn,
    /// A question's or a choice's answer.
    Answer(Answer),
    /// A command of a screen the map has not (a town's, a hero's, an army's: M5's): not shown. A
    /// battle's is none of the map's commands (E24 M4 T6, R5-6).
    Other,
    /// The battle the command before began (E24 M4 T5b1, design §5.20, *provisional*: an answer's,
    /// a guards' question accepted; a walk's is its [`crate::timeline::Move::battle`]): the
    /// record's number of the `GameEvents` whose `BattleStarted` began it. No command of the
    /// record's: the map hands the battle over where that command's box closes.
    Battle(u32),
}

/// Another player's turn in the viewer's sight (E24 M4 T4c, design §5.16, *provisional*): its
/// place among [`Played::turns`], its colour and its heroes' walks the record shows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OtherTurn {
    /// Its `[[turn]]` table's place among [`Played::turns`].
    pub turn: usize,
    /// The player's colour.
    pub player: u8,
    /// Its heroes' runs of steps, in order.
    pub walks: Vec<OtherWalk>,
    /// The record ends in it (E24 M4 T6, the reviewer's R1-1, *provisional*): its walks so far,
    /// no end; a table even without a walk, so that the End Turn before it has a turn to show.
    pub open: bool,
}

/// A run of one of another player's heroes' steps the record shows (E24 M4 T4c).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OtherWalk {
    /// The record's hero id.
    pub hero: u16,
    /// Its steps.
    pub steps: Vec<OtherStep>,
    /// Each of its moves' ends (E24 M4 T6, the reviewer's R1-3: one command's steps a move, the
    /// last at its last step), in order, with what the command changes after its steps.
    pub ends: Vec<OtherEnd>,
}

/// A move's end of another player's hero (E24 M4 T6, the reviewer's R1-3, R1-5, *provisional*):
/// its command's last step, and what the command changes on the map after its steps, each by the
/// record's facts alone (the original's way of showing a computer's flag raised, a monster it beat
/// or a hero lost in sight not read).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OtherEnd {
    /// The walk's steps up to it.
    pub after: usize,
    /// The pickups taken after it (`ObjectRemoved` of a pile, a campfire, an artifact, a scroll, a
    /// chest or a find: [`pickup`]), by their anchors: the hero turned to the first, its dissolve.
    pub taken: Vec<Pos>,
    /// The other objects off the map after it (`ObjectRemoved` of any other kind: a monster
    /// beaten), by their anchors: off at once, nothing dissolved.
    pub removed: Vec<Pos>,
    /// The objects flagged after it (`ObjectFlagged`), by their anchors, each with its owner's
    /// colour.
    pub flagged: Vec<(Pos, u8)>,
    /// The heroes off the map after it (`HeroRemoved`: a hero lost in sight), by their record ids.
    pub lost: Vec<u16>,
    /// The empty boats the viewer is told of after it (E24 M4 T6, the reviewer's R3-3: a boat
    /// built, `BoatBuilt`; between tables, [`Played::between`], also the viewer's own boats told
    /// with no walk of its after them, a `GameReveal`'s).
    pub boats: Vec<LiveBoat>,
}

impl OtherEnd {
    /// Whether it changes nothing.
    fn is_empty(&self) -> bool {
        self.taken.is_empty()
            && self.removed.is_empty()
            && self.flagged.is_empty()
            && self.lost.is_empty()
            && self.boats.is_empty()
    }
}

/// One step of another player's hero (`HeroStepped`): its two tiles, each none where the viewer
/// does not see it (MCL-Q44), and what it does with a boat (`Boarded`, `Landed`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OtherStep {
    /// The tile it left.
    pub from: Option<Pos>,
    /// The tile it entered.
    pub to: Option<Pos>,
    /// A boat boarded or left there.
    pub sea: Option<Sea>,
}

/// Another player's turn being read.
struct OtherOpen {
    player: u8,
    colour: u8,
    walks: Vec<OtherWalk>,
    /// Whether the command's events under way have a step (E24 M4 T4c2: an object removed in a
    /// command without one is taken without a walk; E24 M4 T6, R1-3: its first step a new move).
    stepped: bool,
}

impl OtherOpen {
    /// Player `player`'s turn started, its colour by `observation`.
    fn new(observation: &GameObservation, player: u8) -> Self {
        Self {
            player,
            colour: colour(observation, player),
            walks: Vec::new(),
            stepped: false,
        }
    }

    /// Its turn as the place `turn` among the tables, `open` if the record ends in it.
    fn turn(self, turn: usize, open: bool) -> OtherTurn {
        OtherTurn {
            turn,
            player: self.colour,
            walks: self.walks,
            open,
        }
    }
}

/// Player `player`'s colour in `observation` (its own number where the list has none).
fn colour(observation: &GameObservation, player: u8) -> u8 {
    observation
        .players
        .get(usize::from(player))
        .map_or(player, |p| p.colour)
}

/// Whether an object of the record's type `object_type` is a pickup under `rules` (E24 M4 T6, the
/// reviewer's R1-5, *provisional*: a pile, a campfire, an artifact, a scroll, a chest or a find,
/// what a hero takes from beside with the pickup's dissolve, `visit_resource_pile` 0x4A4BE0 as the
/// viewer's own); any other removal (a monster beaten) is not.
#[must_use]
pub fn pickup(rules: &GameRuleset, object_type: u16) -> bool {
    let kind = rules
        .adventure()
        .object_type(ObjectTypeId::from_raw(object_type))
        .map(|t| &t.kind);
    matches!(
        kind,
        Some(
            ObjectKind::Pickup(_)
                | ObjectKind::Artifact { .. }
                | ObjectKind::Scroll
                | ObjectKind::Chest(_)
                | ObjectKind::Find(_)
        )
    )
}

/// One of the viewer's visits in a record (E24 M4 T4b1, design §5.14, *provisional*): a walk's
/// last tile, the visited object's entrance or its own tile visited from beside, the command's
/// events and the answers the record gives after it, each with its events. Since E24 M4 T6 (the
/// reviewer's R2-1, R2-3, R2-4) its walk, its hero, what the hero held as it began and the visit's
/// level-ups; the visit goes on through its army dialog (a reward's creatures with no room).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Visit {
    /// The walk's last tile.
    pub at: Pos,
    /// The walk's place among [`Played::moves`] (R2-1: each visit to a tile played at its own
    /// walk's end).
    pub walk: usize,
    /// The walk's hero (the record's id) and its type (none without details: R2-3, R2-4, its
    /// level-ups' dialog and Pandora's Box's boxes name it).
    pub hero: u16,
    /// The hero's type.
    pub hero_type: Option<u8>,
    /// What the hero held as the walk began (R2-4): a record tells its spell points and its
    /// battle bonuses as they are after a change, so what a visit changed is told against this.
    pub held: Held,
    /// The visit's level-ups (R2-3: a `LevelUpOpened` or a `LevelGained` without one while the
    /// visit's events are read, its battle's end's too), their places among [`Played::levels`].
    pub levels: Vec<usize>,
    /// The command's events.
    pub events: Vec<GameEvent>,
    /// The answers after it, in order, each with its events.
    pub answers: Vec<(Answer, Vec<GameEvent>)>,
    /// The battle it began (E24 M4 T5b2, design §5.21): its command's or an answer's.
    pub battle: Option<Battled>,
    /// The colour of the flag it raises (E24 M4 T6, the reviewers' R3-1 and R3-4): the first
    /// `ObjectFlagged` of its events, its answers' or its battle's end, its owner (a player id)
    /// in that player's colour by the observation then; none without one.
    pub flag: Option<u8>,
}

/// What a visit's hero held as its walk began (E24 M4 T6, the reviewer's R2-4): its spell points
/// and the morale and luck no once-mark names (`Bonuses`), as the record's observation holds
/// them.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Held {
    /// The spell points.
    pub mana: u16,
    /// The morale.
    pub morale: i8,
    /// The luck.
    pub luck: i8,
}

/// The battle a visit began (E24 M4 T5b2, design §5.21, *provisional*): the record's number of
/// the `GameEvents` whose `BattleStarted` began it, and the events from its `BattleEnded` on in
/// the batch that ends it (its result, then its effects on the map); none before its end.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Battled {
    /// The `GameEvents`' number.
    pub started: u32,
    /// The events from its end on.
    pub ended: Vec<GameEvent>,
}

/// One of the viewer's heroes' level-ups in a record (E24 M4 T4b1, *provisional*): its dialog's
/// facts and the offer the record's answer took (none: no offer, or none taken yet); since E24 M4
/// T6 (the reviewer's R2-3: any hero's dialog) the hero's class, which the dialog names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Leveled {
    /// The dialog's facts.
    pub up: LevelUp,
    /// The offer taken, the dialog's left first.
    pub chosen: Option<usize>,
    /// The hero type's class in the record's rules (0 for a type they do not hold).
    pub class: u8,
}

/// A walk being read: its tiles, the points it left and the tiles explored before it.
struct Open {
    hero: u16,
    tiles: Vec<Pos>,
    points: Option<i32>,
    before: Vec<Pos>,
    end: Option<Pos>,
    sea: Option<Sea>,
    /// The record's number of its `GameEvents` whose `BattleStarted` began a battle (E24 M4 T5b1).
    battle: Option<u32>,
    /// The empty boats the viewer is told of since the walk before (E24 M4 T6, R3-3).
    boats: Vec<LiveBoat>,
}

/// The walks and turns of `loaded`'s record (the module's documentation), its map's art `art`
/// under `rules`.
///
/// # Errors
/// [`Refused::NoStart`]; a path's tile off the map's positions ([`Refused::Line`]: the message's
/// line counted from the header's, blank lines left out).
pub fn played(loaded: &Loaded, art: &MapArt, rules: &GameRuleset) -> Result<Played, Refused> {
    let mut observation = loaded.first()?.clone();
    let viewer = observation.adventure.viewer;
    // A record starts at the game's start (E24 M4 T6, R1-1): a viewer not acting there waits for
    // the player that acts, whose turn is under way.
    let waits = !observation
        .players
        .get(usize::from(viewer))
        .is_some_and(|p| p.acting);
    let mut moves = Vec::new();
    let mut ends = Vec::new();
    let mut turns = Vec::new();
    let mut open: Option<Open> = None;
    let mut week = (1, observation.week);
    let mut month = (1, observation.month);
    let mut visits: Vec<Visit> = Vec::new();
    // The visit the next events belong to (the last one, from its walk to another command), and
    // whether its army dialog is open (E24 M4 T6, R2-4).
    let mut visiting = false;
    let mut army = false;
    let mut levels = Vec::new();
    // The level-up opened and not answered yet (its place among `levels`).
    let mut pending = None;
    let mut walkers = Vec::new();
    let mut commands = Vec::new();
    let mut others = Vec::new();
    let acting = observation
        .players
        .iter()
        .position(|p| p.acting)
        .and_then(|p| u8::try_from(p).ok());
    let mut other: Option<OtherOpen> = acting
        .filter(|_| waits)
        .map(|p| OtherOpen::new(&observation, p));
    // What other players' commands without a step changed in the turn under way (E24 M4 T6,
    // R1-6: given to the first turn table after its end), what is due at the next table, and
    // what each table was given.
    let mut untold = OtherEnd::default();
    let mut due = OtherEnd::default();
    let mut between = Vec::new();
    let mut lists = Vec::new();
    // The empty boats the viewer is told of with no walk of its open (E24 M4 T6, R3-3: a boat
    // built, a reveal's), the next walk's or the next table's, whichever comes first.
    let mut told_boats: Vec<LiveBoat> = Vec::new();
    let shown: Vec<(u16, Pos)> = observation
        .adventure
        .heroes
        .iter()
        .filter_map(|h| tile(h.pos.x, h.pos.y, h.pos.level).map(|at| (h.id, at)))
        .collect();
    let known: Vec<u16> = shown.iter().map(|&(id, _)| id).collect();
    let slots = slots(&observation, viewer);
    // Whether the last command accepted is an answer that has begun no battle yet (E24 M4 T5b1).
    let mut answering = false;
    // The visit whose battle is under way (E24 M4 T5b2).
    let mut fighting: Option<usize> = None;
    for (i, message) in loaded.record.messages.iter().enumerate().skip(1) {
        match message {
            ServerMessage::GameAccepted { command, .. } => {
                close(
                    &mut open,
                    &observation,
                    art,
                    (&mut moves, &mut ends, &mut walkers),
                );
                // A battle's command takes none of the map's time (E24 M4 T6, the reviewer's
                // R5-6): the battle is handed over whole, or not at all (MCL-STUB-3).
                if !in_battle(command) {
                    commands.push(command_of(command));
                }
                answering = matches!(command, GameCommand::Answer { .. });
                match command {
                    GameCommand::Adventure {
                        command: AdventureCommand::MoveHero { hero, path },
                    } => {
                        let tiles = path
                            .iter()
                            .map(|p| tile(p.x, p.y, p.level))
                            .collect::<Option<Vec<_>>>()
                            .ok_or_else(|| {
                                Refused::Line(i + 2, "a path's tile off the map".into())
                            })?;
                        visiting = match tiles.last() {
                            Some(&at) => {
                                let (hero_type, held) = held(&observation, *hero);
                                visits.push(Visit {
                                    at,
                                    walk: moves.len(),
                                    hero: *hero,
                                    hero_type,
                                    held,
                                    levels: Vec::new(),
                                    events: Vec::new(),
                                    answers: Vec::new(),
                                    battle: None,
                                    flag: None,
                                });
                                true
                            }
                            None => false,
                        };
                        open = Some(Open {
                            hero: *hero,
                            tiles,
                            points: None,
                            before: explored(&observation, art),
                            end: None,
                            sea: None,
                            battle: None,
                            boats: core::mem::take(&mut told_boats),
                        });
                    }
                    GameCommand::Answer { answer } => {
                        if let Some(visit) = visits.last_mut().filter(|_| visiting) {
                            visit.answers.push((*answer, Vec::new()));
                        }
                    }
                    // The visit's army dialog's commands (E24 M4 T6, R2-4: a reward's creatures
                    // with no room, GAM-DEC-004): the visit goes on to the dialog's close and what
                    // follows it (the reward's experience, its level-ups).
                    GameCommand::Manage { .. } if army => {}
                    _ => visiting = false,
                }
            }
            ServerMessage::GameEvents { n, events } => {
                if let Some(visit) = visits.last_mut().filter(|_| visiting) {
                    match visit.answers.last_mut() {
                        Some((_, after)) => after.extend(events.iter().cloned()),
                        None => visit.events.extend(events.iter().cloned()),
                    }
                }
                // The viewer's turn's start (E24 M4 T4b2): its table once the batch is folded (the
                // engine tells the heroes' new points after `TurnStarted`: `game-boats` at 1460).
                let mut starts = false;
                if let Some(o) = &mut other {
                    o.stepped = false;
                }
                // A visit's battle ended (E24 M4 T5b2): its result and its effects on the map.
                let mut ended = None;
                if let Some(k) = fighting
                    && let Some(end) = events
                        .iter()
                        .position(|e| matches!(e, GameEvent::BattleEnded { .. }))
                    && let Some(battle) = visits.get_mut(k).and_then(|v| v.battle.as_mut())
                {
                    battle.ended = events[end..].to_vec();
                    fighting = None;
                    ended = Some(k);
                }
                // The visit the batch's level-ups and army dialog are of (E24 M4 T6, R2-3, R2-4):
                // the one under way, else the one whose battle the batch ends.
                let owner = if visiting {
                    visits.len().checked_sub(1)
                } else {
                    ended
                };
                for event in events {
                    let leveled = levels.len();
                    level(&mut levels, &mut pending, (&observation, rules), event);
                    if levels.len() > leveled
                        && let Some(visit) = owner.and_then(|k| visits.get_mut(k))
                    {
                        visit.levels.push(leveled);
                    }
                    match event {
                        GameEvent::ArmyDialogOpened { .. } => army = owner.is_some(),
                        GameEvent::DecisionClosed { .. } => army = false,
                        // The visit's flag (E24 M4 T6, the reviewers' R3-1, R3-4): the record
                        // names its owner by its player id, the map draws its colour.
                        GameEvent::Adventure {
                            event: AdventureEvent::ObjectFlagged { owner: player, .. },
                        } => {
                            if let Some(visit) = owner.and_then(|k| visits.get_mut(k)) {
                                visit.flag.get_or_insert(colour(&observation, *player));
                            }
                        }
                        _ => {}
                    }
                    observation.fold(event);
                    if let (Some(o), GameEvent::Adventure { event }) = (&mut other, event) {
                        let seen = Seen {
                            known: &known,
                            observation: &observation,
                            rules,
                        };
                        other_event(o, event, &seen, &mut untold);
                    } else if let GameEvent::Adventure {
                        event: AdventureEvent::BoatBuilt { tile: at, .. },
                    } = event
                    {
                        // A boat the viewer built (E24 M4 T6, R3-3): the open walk's, else the
                        // next walk's or table's.
                        let boat = built(&observation, *at);
                        match open.as_mut() {
                            Some(walk) => walk.boats.extend(boat),
                            None => told_boats.extend(boat),
                        }
                    }
                    match event {
                        // The viewer's walk began a battle (E24 M4 T5b1, design §5.20): its end
                        // hands the battle over; an answer's, the box it closes.
                        GameEvent::BattleStarted { .. } => {
                            // The visit's battle (E24 M4 T5b2): its end's events kept below.
                            if visiting && let Some(visit) = visits.last_mut() {
                                visit.battle = Some(Battled {
                                    started: *n,
                                    ended: Vec::new(),
                                });
                                fighting = Some(visits.len() - 1);
                            }
                            if let Some(walk) = open.as_mut() {
                                walk.battle.get_or_insert(*n);
                            } else if core::mem::take(&mut answering) {
                                commands.push(Command::Battle(*n));
                            }
                            // stub: MCL-STUB-3 (a battle the viewer defends: not handed over)
                        }
                        GameEvent::Adventure {
                            event: AdventureEvent::HeroStepped { hero, to, .. },
                        } => {
                            if let Some(walk) = open.as_mut().filter(|w| w.hero == *hero) {
                                walk.end = to.and_then(|to| tile(to.x, to.y, to.level));
                            }
                        }
                        GameEvent::Adventure {
                            event: AdventureEvent::MovementChanged { hero, movement, .. },
                        } => {
                            if let Some(walk) = open.as_mut().filter(|w| w.hero == *hero) {
                                walk.points = Some(i32::try_from(*movement).unwrap_or(i32::MAX));
                            }
                        }
                        GameEvent::Adventure {
                            event: AdventureEvent::Boarded { hero, boat, .. },
                        } => {
                            if let Some(walk) = open.as_mut().filter(|w| w.hero == *hero) {
                                walk.sea = Some(Sea::Board(*boat));
                            }
                        }
                        GameEvent::Adventure {
                            event:
                                AdventureEvent::Landed {
                                    hero,
                                    boat,
                                    at,
                                    facing,
                                },
                        } => {
                            if let Some(walk) = open.as_mut().filter(|w| w.hero == *hero) {
                                walk.sea = landed(*at, *boat, *facing).or(walk.sea);
                            }
                        }
                        GameEvent::WeekStarted { week: n, kind } => week = (*n, *kind),
                        GameEvent::MonthStarted { month: n, kind } => month = (*n, *kind),
                        GameEvent::TurnStarted { player } if *player == viewer => starts = true,
                        GameEvent::TurnStarted { player } => {
                            other = Some(OtherOpen::new(&observation, *player));
                        }
                        GameEvent::TurnEnded { player } => {
                            if let Some(o) = other.take_if(|o| o.player == *player) {
                                if o.walks.is_empty() {
                                    extend(&mut due, core::mem::take(&mut untold));
                                } else {
                                    turns.push(computer_turn(
                                        &observation,
                                        loaded,
                                        art,
                                        rules,
                                        (week, month),
                                        (o.colour, heroes_of(&o.walks)),
                                    ));
                                    lists.push(list(&observation, viewer));
                                    // What came before it at its start, its own after it
                                    // (R1-6); the boats told meanwhile (R3-3).
                                    due.boats.append(&mut told_boats);
                                    give(&mut between, &mut due, turns.len());
                                    due = core::mem::take(&mut untold);
                                    others.push(o.turn(turns.len() - 1, false));
                                }
                            }
                        }
                        _ => {}
                    }
                }
                if starts {
                    turns.push(turn(&observation, loaded, art, rules, week, month));
                    lists.push(list(&observation, viewer));
                    extend(&mut due, core::mem::take(&mut untold));
                    due.boats.append(&mut told_boats);
                    give(&mut between, &mut due, turns.len());
                }
            }
            ServerMessage::GameReveal { reveal, .. } => {
                observation.reveal(reveal);
                // The empty boats it shows (E24 M4 T6, the reviewer's R3-3): the open walk's, else
                // the next walk's or table's.
                let boats = reveal.adventure.boats.iter().filter_map(live_boat);
                match open.as_mut() {
                    Some(walk) => walk.boats.extend(boats),
                    None => told_boats.extend(boats),
                }
            }
            ServerMessage::GameTurn { .. } => close(
                &mut open,
                &observation,
                art,
                (&mut moves, &mut ends, &mut walkers),
            ),
            ServerMessage::GameSnapshot {
                observation: snapshot,
                ..
            } => observation = snapshot.clone(),
            _ => {}
        }
    }
    close(
        &mut open,
        &observation,
        art,
        (&mut moves, &mut ends, &mut walkers),
    );
    // A turn under way at the record's end (E24 M4 T6, R1-1): its walks so far, no end.
    if let Some(o) = other {
        due.boats.append(&mut told_boats);
        turns.push(computer_turn(
            &observation,
            loaded,
            art,
            rules,
            (week, month),
            (o.colour, heroes_of(&o.walks)),
        ));
        lists.push(list(&observation, viewer));
        give(&mut between, &mut due, turns.len());
        others.push(o.turn(turns.len() - 1, true));
    }
    Ok(Played {
        moves,
        ends,
        turns,
        lists,
        waits,
        visits,
        levels,
        walkers,
        commands,
        others,
        shown,
        slots,
        between,
    })
}

/// How many heroes `walks` are of.
fn heroes_of(walks: &[OtherWalk]) -> usize {
    walks
        .iter()
        .fold(Vec::new(), |mut seen, w| {
            if !seen.contains(&w.hero) {
                seen.push(w.hero);
            }
            seen
        })
        .len()
}

/// Whether `command` is a battle's (the viewer's own or the system's for it, E24 M4 T6): none of
/// the map's [`Played::commands`].
fn in_battle(command: &GameCommand) -> bool {
    matches!(
        command,
        GameCommand::Battle { .. }
            | GameCommand::System {
                command: SystemCommand::Battle { .. }
            }
    )
}

/// A command echoed as the map's [`Command`].
fn command_of(command: &GameCommand) -> Command {
    match command {
        GameCommand::Adventure {
            command: AdventureCommand::MoveHero { hero, path },
        } => path
            .last()
            .and_then(|p| tile(p.x, p.y, p.level))
            .map_or(Command::Other, |to| Command::Walk { hero: *hero, to }),
        GameCommand::EndTurn {} => Command::EndTurn,
        GameCommand::Answer { answer } => Command::Answer(*answer),
        _ => Command::Other,
    }
}

/// What [`other_event`] reads: the heroes the record's start shows (by their ids), the
/// observation folded so far (the players' colours) and the rules (the objects' kinds).
struct Seen<'a> {
    known: &'a [u16],
    observation: &'a GameObservation,
    rules: &'a GameRuleset,
}

/// Another player's hero's event folded into its turn `other` (E24 M4 T4c): a step a new step
/// (a new walk when another hero steps; since E24 M4 T6, R1-3, a command's first step a new move
/// of the walk), a boarding or a landing its last step's; an object removed or flagged and a hero
/// removed the move's end's (E24 M4 T6, R1-5); one in a command without a step, or of a hero the
/// start does not show (MCL-Q53: left out), `pending`'s (E24 M4 T4c2, MCL-Q46: without a walk).
fn other_event(
    other: &mut OtherOpen,
    event: &AdventureEvent,
    seen: &Seen<'_>,
    pending: &mut OtherEnd,
) {
    match event {
        AdventureEvent::HeroStepped { hero, from, to } => {
            // stub: MCL-Q53 (a hero coming into sight: its owner, type and boat not told)
            if !seen.known.contains(hero) {
                return;
            }
            let first = !core::mem::replace(&mut other.stepped, true);
            let step = OtherStep {
                from: from.and_then(|p| tile(p.x, p.y, p.level)),
                to: to.and_then(|p| tile(p.x, p.y, p.level)),
                sea: None,
            };
            match other.walks.last_mut().filter(|w| w.hero == *hero) {
                Some(walk) => {
                    walk.steps.push(step);
                    let after = walk.steps.len();
                    match walk.ends.last_mut().filter(|_| !first) {
                        Some(end) => end.after = after,
                        None => walk.ends.push(OtherEnd {
                            after,
                            ..OtherEnd::default()
                        }),
                    }
                }
                None => other.walks.push(OtherWalk {
                    hero: *hero,
                    steps: vec![step],
                    ends: vec![OtherEnd {
                        after: 1,
                        ..OtherEnd::default()
                    }],
                }),
            }
        }
        AdventureEvent::Boarded { hero, boat, .. } => {
            if let Some(step) = last_step(other, *hero) {
                step.sea = Some(Sea::Board(*boat));
            }
        }
        AdventureEvent::Landed {
            hero,
            boat,
            at,
            facing,
        } => {
            if let Some(step) = last_step(other, *hero) {
                step.sea = landed(*at, *boat, *facing).or(step.sea);
            }
        }
        AdventureEvent::ObjectRemoved {
            object_type,
            anchor,
        } => {
            if let Some(at) = tile(anchor.x, anchor.y, anchor.level) {
                // stub: MCL-Q46 (the hero that takes an object without a step not told)
                let end = move_end(other).unwrap_or(pending);
                if pickup(seen.rules, *object_type) {
                    end.taken.push(at);
                } else {
                    end.removed.push(at);
                }
            }
        }
        AdventureEvent::ObjectFlagged { anchor, owner, .. } => {
            if let Some(at) = tile(anchor.x, anchor.y, anchor.level) {
                let colour = colour(seen.observation, *owner);
                move_end(other)
                    .unwrap_or(pending)
                    .flagged
                    .push((at, colour));
            }
        }
        AdventureEvent::HeroRemoved { hero, .. } => {
            move_end(other).unwrap_or(pending).lost.push(*hero);
        }
        // A boat built in sight (E24 M4 T6, the reviewer's R3-3).
        AdventureEvent::BoatBuilt { tile: at, .. } => {
            let boats = built(seen.observation, *at);
            move_end(other).unwrap_or(pending).boats.extend(boats);
        }
        _ => {}
    }
}

/// A `Landed`'s boat left on `at`, of type `boat` facing `facing` (its number); a facing past 7
/// the timeline refuses (E24 M4 T6, the reviewer's R3-6: [`Sea::Unfaced`]); none off the map's
/// positions.
fn landed(at: WirePos, boat: u8, facing: u8) -> Option<Sea> {
    let at = tile(at.x, at.y, at.level)?;
    Some(match Facing::from_number(facing) {
        Some(facing) => Sea::Land(LiveBoat {
            at,
            kind: boat,
            facing,
        }),
        None => Sea::Unfaced(facing),
    })
}

/// An empty boat of the record as the timeline draws it ([`LiveBoat`]); none off the map's
/// positions or with a facing past 7.
fn live_boat(boat: &Boat) -> Option<LiveBoat> {
    let p = boat.tile;
    Some(LiveBoat {
        at: tile(p.x, p.y, p.level)?,
        kind: boat.kind,
        facing: Facing::from_number(boat.facing)?,
    })
}

/// The boat `BoatBuilt` put on `at`, as the observation folded with it holds it (its type, the
/// built boats' facing: `GameObservation::fold`'s).
fn built(observation: &GameObservation, at: WirePos) -> Option<LiveBoat> {
    observation
        .adventure
        .boats
        .iter()
        .find(|b| b.tile == at)
        .and_then(live_boat)
}

/// The end of the move under way in `other`, none outside a command with a step.
fn move_end(other: &mut OtherOpen) -> Option<&mut OtherEnd> {
    other
        .walks
        .last_mut()
        .filter(|_| other.stepped)?
        .ends
        .last_mut()
}

/// `from`'s changes added to `into`'s.
fn extend(into: &mut OtherEnd, from: OtherEnd) {
    into.taken.extend(from.taken);
    into.removed.extend(from.removed);
    into.flagged.extend(from.flagged);
    into.lost.extend(from.lost);
    into.boats.extend(from.boats);
}

/// The changes `due` given to the turn table just pushed (`tables` long: E24 M4 T4c2, since T6
/// [`Played::between`]), none when there are none.
fn give(between: &mut Vec<(usize, OtherEnd)>, due: &mut OtherEnd, tables: usize) {
    if !due.is_empty() {
        between.push((tables - 1, core::mem::take(due)));
    }
}

/// The viewer's heroes in its kingdom's list in `observation`, slot by slot, by their ids
/// ([`Played::lists`]).
fn list(observation: &GameObservation, viewer: u8) -> Vec<u16> {
    slots(observation, viewer)
        .into_iter()
        .map(|(id, _)| id)
        .collect()
}

/// The last step of `hero`'s walk under way in `other`.
fn last_step(other: &mut OtherOpen, hero: u16) -> Option<&mut OtherStep> {
    other
        .walks
        .last_mut()
        .filter(|w| w.hero == hero)?
        .steps
        .last_mut()
}

/// The viewer's heroes' places in its kingdom's list in `observation` (`super::panels`' order:
/// their types', then their places on the map), by their ids.
fn slots(observation: &GameObservation, viewer: u8) -> Vec<(u16, usize)> {
    let mut own: Vec<(u16, usize, u16)> = observation
        .adventure
        .heroes
        .iter()
        .enumerate()
        .filter(|(_, h)| h.owner == viewer)
        .filter_map(|(i, h)| {
            let info = observation.heroes.iter().find(|info| info.id == h.id);
            info.map(|info| (info.hero_type, i, h.id))
        })
        .collect();
    own.sort_unstable();
    own.iter()
        .enumerate()
        .map(|(slot, &(_, _, id))| (id, slot))
        .collect()
}

/// Hero `hero`'s type and what it holds in `observation` (E24 M4 T6, R2-4: [`Held`]); none and
/// nothing for a hero without details or not on the map.
fn held(observation: &GameObservation, hero: u16) -> (Option<u8>, Held) {
    let details = observation
        .heroes
        .iter()
        .find(|i| i.id == hero)
        .and_then(|i| i.details.as_ref());
    let bonuses = observation
        .adventure
        .heroes
        .iter()
        .find(|h| h.id == hero)
        .and_then(|h| h.own.as_ref())
        .map(|o| &o.bonuses);
    let held = Held {
        mana: details.map_or(0, |d| d.mana),
        morale: bonuses.map_or(0, |b| b.morale),
        luck: bonuses.map_or(0, |b| b.luck),
    };
    let hero_type = details.and_then(|d| u8::try_from(d.hero_type).ok());
    (hero_type, held)
}

/// A level-up's facts folded into `levels` before `event` is (E24 M4 T4b1): a `LevelUpOpened`'s
/// dialog with its two offers, `pending` until its `LevelGained` gives the offer taken; a
/// `LevelGained` without one a dialog with none or its one offer, taken. Each offer's level the
/// one it reaches by the hero's skills in `observation`; a hero `observation` holds no details of
/// gives none.
fn level(
    levels: &mut Vec<Leveled>,
    pending: &mut Option<usize>,
    (observation, rules): (&GameObservation, &GameRuleset),
    event: &GameEvent,
) {
    let class = |up: &LevelUp| {
        rules
            .core()
            .hero_type(Handle::from_raw(u16::from(up.hero)))
            .and_then(|t| u8::try_from(t.class.index()).ok())
            .unwrap_or(0)
    };
    match *event {
        GameEvent::LevelUpOpened {
            hero,
            level,
            primary,
            offers,
            ..
        } => {
            if let Some(up) = level_up(observation, hero, level, primary, &offers) {
                *pending = Some(levels.len());
                let class = class(&up);
                levels.push(Leveled {
                    up,
                    chosen: None,
                    class,
                });
            }
        }
        GameEvent::LevelGained {
            hero,
            level,
            primary,
            skill,
            ..
        } => {
            if let Some(open) = pending.take().and_then(|i| levels.get_mut(i)) {
                open.chosen =
                    skill.and_then(|s| open.up.offers.iter().position(|o| u16::from(o.skill) == s));
                return;
            }
            let offers: Vec<u16> = skill.into_iter().collect();
            if let Some(up) = level_up(observation, hero, level, primary, &offers) {
                let chosen = skill.map(|_| 0);
                let class = class(&up);
                levels.push(Leveled { up, chosen, class });
            }
        }
        _ => {}
    }
}

/// The dialog's facts of hero `hero`'s level `level` (its type, the primary skill raised, each of
/// `offers` with the level it reaches by the hero's skills in `observation`); none for a hero
/// without details there or an offer past the skills' bytes.
fn level_up(
    observation: &GameObservation,
    hero: u16,
    level: u8,
    primary: PrimarySkill,
    offers: &[u16],
) -> Option<LevelUp> {
    let details = observation
        .heroes
        .iter()
        .find(|i| i.id == hero)
        .and_then(|i| i.details.as_ref())?;
    let offers = offers
        .iter()
        .map(|&skill| {
            let had = details
                .skills
                .iter()
                .find(|s| s.skill == skill)
                .map_or(0, |s| s.level);
            Some(Offer {
                skill: u8::try_from(skill).ok()?,
                level: had.saturating_add(1),
            })
        })
        .collect::<Option<Vec<_>>>()?;
    Some(LevelUp {
        hero: u8::try_from(details.hero_type).unwrap_or(u8::MAX),
        level,
        primary: match primary {
            PrimarySkill::Attack => 0,
            PrimarySkill::Defense => 1,
            PrimarySkill::Power => 2,
            PrimarySkill::Knowledge => 3,
        },
        offers,
    })
}

/// The walk read so far closed: its [`Move`] with the tiles explored since it opened (MCL-Q39
/// (c): its points at its last step, its whole reveal at its first).
fn close(
    open: &mut Option<Open>,
    observation: &GameObservation,
    art: &MapArt,
    (out, ends, walkers): (&mut Vec<Move>, &mut Vec<Option<Pos>>, &mut Vec<u16>),
) {
    let Some(walk) = open.take() else {
        return;
    };
    let steps = walk.tiles.len();
    let revealed: Vec<Pos> = explored(observation, art)
        .into_iter()
        .filter(|t| !walk.before.contains(t))
        .collect();
    // stub: MCL-Q39 (a step's points and reveal: the walk's at its last and first)
    let mut points = vec![None; steps];
    if let Some(last) = points.last_mut() {
        *last = walk.points;
    }
    let mut per_step = vec![Vec::new(); steps];
    if let Some(first) = per_step.first_mut() {
        *first = revealed;
    }
    out.push(Move {
        tiles: walk.tiles,
        points,
        revealed: per_step,
        sea: walk.sea,
        battle: walk.battle,
        boats: walk.boats,
    });
    ends.push(walk.end);
    walkers.push(walk.hero);
}

/// Kind 5's value the record gives none for (MCL-Q43): 1 000, the panel's own part.
const OTHER_VALUE: i64 = 1000;

/// Another player's turn in sight as a `[[turn]]` table (E24 M4 T4c, the module's
/// documentation): the viewer's panels' kingdom with the computer's `heroes` walking (each with
/// no points left, selected the first: MCL-Q43), kind 5 for its `colour` at frame 0 with the
/// value [`OTHER_VALUE`], the week and the month as the fold holds them.
fn computer_turn(
    observation: &GameObservation,
    loaded: &Loaded,
    art: &MapArt,
    rules: &GameRuleset,
    (week, month): ((u32, Week), (u32, Month)),
    (colour, heroes): (u8, usize),
) -> Value {
    let mut table = Table::new();
    panels::insert(&mut table, observation, &loaded.raw, art, rules);
    table.remove("realm");
    table.remove("chosen");
    // stub: MCL-Q43 (the computer's heroes' points and kind 5's value: none told)
    let mut kingdom = table
        .remove("kingdom")
        .and_then(|k| k.as_table().cloned())
        .unwrap_or_default();
    kingdom.insert(
        "heroes".to_string(),
        Value::Array(vec![row(&[0, 1, 0, 0, 0]); heroes]),
    );
    // None selected without a walking hero (E24 M4 T6: a turn under way at the record's end).
    let selected = i64::from(heroes > 0);
    kingdom.insert("selected".to_string(), row(&[selected, 0]));
    table.insert("kingdom".to_string(), Value::Table(kingdom));
    let mut status = Table::new();
    status.insert("kind".to_string(), Value::Integer(5));
    for key in ["sand", "glass"] {
        status.insert(key.to_string(), Value::Integer(0));
    }
    status.insert("crest".to_string(), Value::Integer(i64::from(colour)));
    status.insert("value".to_string(), Value::Integer(OTHER_VALUE));
    table.insert("at".to_string(), Value::Integer(0));
    table.insert("player".to_string(), Value::Integer(i64::from(colour)));
    table.insert("status".to_string(), Value::Table(status));
    table.insert("week".to_string(), week_calendar(week));
    table.insert("month".to_string(), month_calendar(month));
    Value::Table(table)
}

/// A viewer's turn's start as a `[[turn]]` table (the module's documentation).
fn turn(
    observation: &GameObservation,
    loaded: &Loaded,
    art: &MapArt,
    rules: &GameRuleset,
    week: (u32, Week),
    month: (u32, Month),
) -> Value {
    // The panels' kingdom; their status window, kingdom's window and choices are the start's.
    let mut table = Table::new();
    panels::insert(&mut table, observation, &loaded.raw, art, rules);
    table.remove("realm");
    table.remove("chosen");
    let colour = observation
        .players
        .get(usize::from(observation.adventure.viewer))
        .map_or(0, |p| i64::from(p.colour));
    let mut status = Table::new();
    status.insert("kind".to_string(), Value::Integer(DATE_WINDOW));
    status.insert("frame".to_string(), Value::Integer(0));
    table.insert("at".to_string(), Value::Integer(0));
    table.insert("player".to_string(), Value::Integer(colour));
    table.insert("status".to_string(), Value::Table(status));
    table.insert("week".to_string(), week_calendar(week));
    table.insert("month".to_string(), month_calendar(month));
    Value::Table(table)
}

/// A week's `[kind, name or creature]` (`crate::live::Calendar`): a normal one's name the
/// client's (MCL-Q40 (b)), a building's week the Deity's (kind 2).
fn week_calendar((n, kind): (u32, Week)) -> Value {
    match kind {
        // stub: MCL-Q40 (a normal week's name: the client's)
        Week::Normal {} => row(&[0, name(n, WEEK_NAMES)]),
        Week::Creature { creature } => row(&[1, i64::from(creature)]),
        Week::Of { .. } => row(&[2, 0]),
    }
}

/// A month's `[kind, name or creature]` (as [`week_calendar`]; the Plague kind 2).
fn month_calendar((n, kind): (u32, Month)) -> Value {
    match kind {
        // stub: MCL-Q40 (a normal month's name: the client's)
        Month::Normal {} => row(&[0, name(n, MONTH_NAMES)]),
        Month::Creature { creature } => row(&[1, i64::from(creature)]),
        Month::Plague {} => row(&[2, 0]),
    }
}

/// The client's name of the `n`-th week or month among `names` (MCL-Q40 (b)).
fn name(n: u32, names: u16) -> i64 {
    int(usize::try_from(n.saturating_sub(1) % u32::from(names.max(1))).unwrap_or(0))
}

/// The viewer's explored tiles, level by level, row by row (the view's bits; a map is at most 252
/// tiles a side).
fn explored(observation: &GameObservation, art: &MapArt) -> Vec<Pos> {
    let a = &observation.adventure;
    let stride = usize::from(a.stride);
    let size = u8::try_from(art.size).unwrap_or(u8::MAX);
    let levels = u8::try_from(art.levels).unwrap_or(u8::MAX);
    let mut tiles = Vec::new();
    for z in 0..levels {
        for y in 0..size {
            let row = usize::from(z) * usize::from(size) + usize::from(y);
            for x in 0..size {
                let at = usize::from(x);
                let word = a.explored.get(row * stride + at / 64).map_or(0, |w| w.0);
                if word >> (at % 64) & 1 == 1 {
                    tiles.push(Pos { x, y, z });
                }
            }
        }
    }
    tiles
}

/// A wire tile as the map's position; none past a byte.
fn tile(x: u16, y: u16, level: u8) -> Option<Pos> {
    match (u8::try_from(x), u8::try_from(y)) {
        (Ok(x), Ok(y)) => Some(Pos { x, y, z: level }),
        _ => None,
    }
}
