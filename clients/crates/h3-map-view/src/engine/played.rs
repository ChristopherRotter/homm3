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
//! - **A turn's start** (the viewer's `TurnStarted` after the first): a live part's `[[turn]]`
//!   table as a recording's (`crate::live::Turn::read`): the kingdom by the panels
//!   (`super::panels`) of the observation folded to it, the date's window at its first frame
//!   (`player_turn_start` 0x4CC7D0 asks it, design §5.1), the week's and the month's kind as the
//!   fold holds them, a normal one's name the client's (MCL-Q40 (b): the week's number less one
//!   modulo the names, the month's likewise; the original draws it). The table holds no
//!   millisecond (`at` 0) and no heroes or boats: nothing moves out of the viewer's own turns in
//!   T4a2's games (the other players' turns are T4c's).

use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

use h3_game::GameRuleset;
use h3_map::model::Pos;
use h3_protocol::wire::ServerMessage;
use h3_protocol::wire::game::{
    AdventureCommand, AdventureEvent, GameCommand, GameEvent, GameObservation, Month, Week,
};
use toml::{Table, Value};

use super::{Loaded, Refused, int, panels, row};
use crate::art::MapArt;
use crate::timeline::Move;
use crate::turn::{MONTH_NAMES, WEEK_NAMES};

/// The date's window (`Status`'s kind 1) at its first frame.
const DATE_WINDOW: i64 = 1;

/// A record's own walks and turns' starts, in order.
#[derive(Debug, Clone, PartialEq)]
pub struct Played {
    /// The viewer's walks (the timeline's `Start::moves`).
    pub moves: Vec<Move>,
    /// The viewer's turns' starts after the first, each a `[[turn]]` table.
    pub turns: Vec<Value>,
}

/// A walk being read: its tiles, the points it left and the tiles explored before it.
struct Open {
    hero: u16,
    tiles: Vec<Pos>,
    points: Option<i32>,
    before: Vec<Pos>,
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
    let mut moves = Vec::new();
    let mut turns = Vec::new();
    let mut open: Option<Open> = None;
    let mut week = (1, observation.week);
    let mut month = (1, observation.month);
    for (i, message) in loaded.record.messages.iter().enumerate().skip(1) {
        match message {
            ServerMessage::GameAccepted { command, .. } => {
                close(&mut open, &observation, art, &mut moves);
                if let GameCommand::Adventure {
                    command: AdventureCommand::MoveHero { hero, path },
                } = command
                {
                    let tiles = path
                        .iter()
                        .map(|p| tile(p.x, p.y, p.level))
                        .collect::<Option<Vec<_>>>()
                        .ok_or_else(|| Refused::Line(i + 2, "a path's tile off the map".into()))?;
                    open = Some(Open {
                        hero: *hero,
                        tiles,
                        points: None,
                        before: explored(&observation, art),
                    });
                }
            }
            ServerMessage::GameEvents { events, .. } => {
                for event in events {
                    observation.fold(event);
                    match event {
                        GameEvent::Adventure {
                            event: AdventureEvent::MovementChanged { hero, movement, .. },
                        } => {
                            if let Some(walk) = open.as_mut().filter(|w| w.hero == *hero) {
                                walk.points = Some(i32::try_from(*movement).unwrap_or(i32::MAX));
                            }
                        }
                        GameEvent::WeekStarted { week: n, kind } => week = (*n, *kind),
                        GameEvent::MonthStarted { month: n, kind } => month = (*n, *kind),
                        GameEvent::TurnStarted { player } if *player == viewer => {
                            turns.push(turn(&observation, loaded, art, rules, week, month));
                        }
                        _ => {}
                    }
                }
            }
            ServerMessage::GameReveal { reveal, .. } => observation.reveal(reveal),
            ServerMessage::GameTurn { .. } => close(&mut open, &observation, art, &mut moves),
            ServerMessage::GameSnapshot {
                observation: snapshot,
                ..
            } => observation = snapshot.clone(),
            _ => {}
        }
    }
    close(&mut open, &observation, art, &mut moves);
    Ok(Played { moves, turns })
}

/// The walk read so far closed: its [`Move`] with the tiles explored since it opened (MCL-Q39
/// (c): its points at its last step, its whole reveal at its first).
fn close(
    open: &mut Option<Open>,
    observation: &GameObservation,
    art: &MapArt,
    out: &mut Vec<Move>,
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
    });
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
