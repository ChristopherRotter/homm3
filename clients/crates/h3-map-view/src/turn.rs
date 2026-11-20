//! The turn's end and the next day (E24 M4 T1a, *provisional*; `docs/epics/E24-design.md` §5.1):
//! the tables' `[map.turn]`, End Turn's question's text and the week's and the month's message's
//! (`player_turn_start` 0x4CC7D0 at 0x4CCAE5, ArrayTxt's records as `arraytxt_load` 0x5B9CC0
//! copies them), the keys and families they name. The timeline (`crate::timeline`) opens the boxes
//! and plays the turn's change; the adapter lays the boxes out (`h3_ui::message`) with the texts
//! given here. Since E24 M4 T1b2 (design §5.2) what a turn's facts give the timeline of the map's
//! heroes: those standing ([`placed`]) and a computer player's walks in sight ([`walks`]); since
//! E24 M4 T4c (design §5.16) a record's ([`record_heroes`], [`record_walks`]).

use alloc::format;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use h3_ui::text::{Args, Catalogue};
use serde::Deserialize;

use h3_map::model::Pos;

use crate::engine::OtherTurn;
use crate::live::{Calendar, Date, LiveBoat, LiveHero, Selected, Status, Turn};
use crate::timeline::{Direction, Hero, Steps};

/// The turn's numbers and sounds (the tables' `[map.turn]`).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TurnArt {
    /// How long a new turn asks the date's status window for, ms (`player_turn_start`'s
    /// `status_window_set_kind(1, 3000)`).
    pub asked: u32,
    /// The week's message's sound (`newweek.wav`).
    pub week: String,
    /// A month's first week's (`newmonth.wav`).
    pub month: String,
    /// The week of the Deity's two creatures (the Imp, 42, and the Familiar, 43: ArrayTxt 135's
    /// names).
    pub deity: [u16; 2],
    /// The days a player with heroes and no town has to take one (`scenario_condition_met`
    /// 0x4F2CE0: the player's +0x3D −1 become 7; E24 M4 T2a2, design §5.3).
    pub town_days: u8,
    /// A computer player's turn's music (E24 M4 T4c, design §5.1: one of `AITheme0` to
    /// `AITheme2`, `random_between(1, 3)` − 1 at the computer's start; the client's turn by
    /// turn, *provisional*); none, no music asked.
    #[serde(default)]
    pub computer: Vec<String>,
}

/// The string catalogue's keys the turn shows (`clients/text/keys.toml`); the names are the
/// `week-name`, `month-name` and `creature` families' ([`week_key`], [`month_key`]).
pub const KEYS: [&str; 7] = [
    "map.turn.end",
    "map.turn.month",
    "map.turn.month-creature",
    "map.turn.plague",
    "map.turn.week",
    "map.turn.week-creature",
    "map.turn.week-deity",
];

/// A normal week's names (`random_between(0, 14)` at 0x4C87A6; the catalogue's `week-name`
/// family).
pub const WEEK_NAMES: u16 = 15;

/// A normal month's names (`random_between(0, 9)` at 0x4C8FCC; `month-name`).
pub const MONTH_NAMES: u16 = 10;

/// The key of a normal week's name `n` (ArrayTxt's record 43 + n, `random_between(0, 14)` at
/// 0x4C87A6).
#[must_use]
pub fn week_key(n: u16) -> String {
    format!("week-name.{n}")
}

/// The key of a normal month's name `n` (ArrayTxt's record 32 + n, `random_between(0, 9)` at
/// 0x4C8FCC).
#[must_use]
pub fn month_key(n: u16) -> String {
    format!("month-name.{n}")
}

/// Whether `date` is a week's first day but the game's first (`player_turn_start` 0x4CCAE5, as
/// the date's window's 0x450EB4): the week's message shows, the week's picture in the window.
#[must_use]
pub fn starts_week(date: Date) -> bool {
    date.day == 1 && !(date.week == 1 && date.month == 1)
}

/// The week's message's sound on `date` (0x4CCAE5): a month's first week's `newmonth.wav`, else
/// `newweek.wav`.
#[must_use]
pub fn sound(art: &TurnArt, date: Date) -> &str {
    if date.week == 1 {
        &art.month
    } else {
        &art.week
    }
}

/// End Turn's question while a hero can still move (GENRLTXT 56, a box of style 2).
#[must_use]
pub fn question(texts: &Catalogue) -> String {
    texts.text("map.turn.end", &Args::new())
}

/// The week's or the month's message on `date` (none on another day; 0x4CCAE5): on a month's
/// first week with a normal week the month's (a normal month ArrayTxt 130 with its name, a
/// creature's 131 with the creature's singular name, its first letter upper-cased (0x618B5F),
/// twice, the Plague 132), else the week's (a normal week 133 with its name, a creature's 134 with
/// the creature's name so, twice, the Deity's 135 with its first creature's name twice, `growth`
/// (its creature record's +0x44, the Imp's) signed, its second's name and `growth` again).
#[must_use]
pub fn message(
    texts: &Catalogue,
    art: &TurnArt,
    (date, week, month): (Date, Calendar, Calendar),
    growth: i32,
) -> Option<String> {
    if !starts_week(date) {
        return None;
    }
    let name = |key: String| texts.text(&key, &Args::new());
    let creature =
        |c: u16| upper_first(&texts.text(&format!("creature.{c}"), &Args::new().count(1)));
    let text = match (date.week == 1, week, month) {
        (true, Calendar::Named(_), Calendar::Named(n)) => {
            let args = Args::new().text("name", &name(month_key(n)));
            texts.text("map.turn.month", &args)
        }
        (true, Calendar::Named(_), Calendar::Creature(c)) => {
            let args = Args::new().text("creature", &creature(c));
            texts.text("map.turn.month-creature", &args)
        }
        (true, Calendar::Named(_), Calendar::Special) => {
            texts.text("map.turn.plague", &Args::new())
        }
        (_, Calendar::Named(n), _) => {
            let args = Args::new().text("name", &name(week_key(n)));
            texts.text("map.turn.week", &args)
        }
        (_, Calendar::Creature(c), _) => {
            let args = Args::new().text("creature", &creature(c));
            texts.text("map.turn.week-creature", &args)
        }
        (_, Calendar::Special, _) => {
            // The names as the creature table holds them (*provisional*: the reading names no
            // upper-casing for the Deity's).
            let [first, second] = art
                .deity
                .map(|c| texts.text(&format!("creature.{c}"), &Args::new().count(1)));
            let args = Args::new()
                .text("creature", &first)
                .text("growth", &format!("{growth:+}"))
                .text("other", &second);
            texts.text("map.turn.week-deity", &args)
        }
    };
    Some(text)
}

/// `name` with its first letter upper-cased (`toupper` of its first byte, 0x618B5F: an ASCII
/// letter's alone).
fn upper_first(name: &str) -> String {
    let mut chars = name.chars();
    chars.next().map_or_else(String::new, |first| {
        let mut out = String::from(first.to_ascii_uppercase());
        out.push_str(chars.as_str());
        out
    })
}

#[cfg(test)]
mod tests;

/// A hero of the live part on its tile and facing, as the timeline holds it.
fn hero_of(h: &LiveHero) -> Hero {
    Hero {
        at: h.at,
        facing: Direction::ALL[usize::from(h.facing.number()) % Direction::ALL.len()],
    }
}

/// The map's heroes `turn`'s facts hold standing (E24 M4 T1b2, *provisional*: the tables name no
/// hero), each by its index among the live part's `map` heroes at 0 (the timeline's
/// [`crate::timeline::Start::heroes`]): the only one of the same owner and class. None for a table
/// without them (T1a's).
///
/// # Errors
/// A hero standing that matches no map hero or several, one whose boat changed, or boats other
/// than the map's `boats` at 0 (a boat's move is not built).
pub fn placed(
    map: &[LiveHero],
    boats: &[LiveBoat],
    turn: &Turn,
) -> Result<Vec<(usize, Hero)>, String> {
    if turn.boats.as_ref().is_some_and(|b| b != boats) {
        return Err(format!(
            "the turn at {} ms: a boat moved (not built)",
            turn.at
        ));
    }
    let Some(standing) = &turn.heroes else {
        return Ok(Vec::new());
    };
    standing
        .iter()
        .map(|h| {
            let same: Vec<usize> = map
                .iter()
                .enumerate()
                .filter(|(_, m)| (m.owner, m.class) == (h.owner, h.class))
                .map(|(i, _)| i)
                .collect();
            match same[..] {
                [i] if map[i].boat == h.boat => Ok((i, hero_of(h))),
                [_] => Err(format!(
                    "the turn at {} ms: a hero's boat changed (not built)",
                    turn.at
                )),
                _ => Err(format!(
                    "the turn at {} ms: {} map heroes of owner {} and class {}",
                    turn.at,
                    same.len(),
                    h.owner,
                    h.class
                )),
            }
        })
        .collect()
}

/// A computer player's walks in the turn `turns[at]` (E24 M4 T1b2, *provisional*: M4's scripted
/// walks; Part B the engine's moves): none unless its window is another player's (kind 5). Its one
/// hero (the map's only hero of its colour, its kingdom's selected one) and its steps, the
/// recording's `hero` lines (`lines`, [`crate::sequence::Sequence::heroes`]) from the turn's
/// millisecond to the next turn's.
///
/// # Errors
/// A computer's turn with other than one hero, or none selected.
pub fn walks(
    turns: &[Turn],
    at: usize,
    map: &[LiveHero],
    lines: &[(u32, Hero)],
) -> Result<Vec<Steps>, String> {
    let Some(turn) = turns
        .get(at)
        .filter(|t| matches!(t.status, Status::Turn(_)))
    else {
        return Ok(Vec::new());
    };
    let own: Vec<usize> = map
        .iter()
        .enumerate()
        .filter(|(_, h)| h.owner == turn.player)
        .map(|(i, _)| i)
        .collect();
    let (&[hero], 1, Selected::Hero(slot)) =
        (&own[..], turn.kingdom.heroes.len(), turn.kingdom.selected)
    else {
        return Err(format!(
            "the computer's turn at {} ms: {} heroes of its colour on the map, {} in its \
             kingdom (one hero, selected, is built)",
            turn.at,
            own.len(),
            turn.kingdom.heroes.len()
        ));
    };
    let end = turns.get(at.saturating_add(1)).map_or(u32::MAX, |t| t.at);
    let steps = lines
        .iter()
        .filter(|(t, _)| (turn.at..end).contains(t))
        .map(|&(_, h)| h)
        .collect();
    Ok(vec![Steps {
        hero,
        slot,
        steps,
        unseen: Vec::new(),
    }])
}

/// A record's heroes matched with the live part's map heroes at 0 (E24 M4 T4c): each one the
/// record's start shows (`shown`, by its id) with the map hero on its tile, by its index.
#[must_use]
pub fn record_heroes(shown: &[(u16, Pos)], map: &[LiveHero]) -> Vec<(u16, usize)> {
    shown
        .iter()
        .filter_map(|&(id, at)| Some((id, map.iter().position(|h| h.at == at)?)))
        .collect()
}

/// Another player's turn in a record as the timeline's walks (E24 M4 T4c, design §5.16,
/// *provisional*): each walk's hero by `ids` ([`record_heroes`]), its place in the computer's
/// kingdom the order its heroes first walk in (the turn's table's), its steps: one with both tiles
/// facing from the one to the other; one into sight (no tile it left: MCL-Q44) taken unseen onto
/// its tile, facing as before; one out of sight (no tile it entered: MCL-Q44) left out, the hero
/// standing where the viewer last saw it.
///
/// # Errors
/// A walk's hero the record's start does not show (not built), a step between tiles that are not
/// neighbours.
pub fn record_walks(
    other: &OtherTurn,
    ids: &[(u16, usize)],
    map: &[LiveHero],
) -> Result<Vec<Steps>, String> {
    let mut order: Vec<u16> = Vec::new();
    let mut facings: Vec<(usize, Direction)> = Vec::new();
    other
        .walks
        .iter()
        .map(|walk| {
            let Some(&(_, hero)) = ids.iter().find(|(id, _)| *id == walk.hero) else {
                return Err(format!(
                    "a record's hero {} walks in sight: not shown at the start (not built)",
                    walk.hero
                ));
            };
            let slot = order
                .iter()
                .position(|&id| id == walk.hero)
                .unwrap_or_else(|| {
                    order.push(walk.hero);
                    order.len() - 1
                });
            let mut facing = facings
                .iter()
                .rev()
                .find(|(h, _)| *h == hero)
                .map(|&(_, f)| f)
                .or_else(|| map.get(hero).map(|h| hero_of(h).facing))
                .unwrap_or(Direction::ALL[0]);
            let mut steps = Vec::new();
            let mut unseen = Vec::new();
            for step in &walk.steps {
                match (step.from, step.to) {
                    (Some(from), Some(to)) => {
                        facing = Direction::between(from, to).ok_or_else(|| {
                            format!("a record's step from {from:?} to {to:?}: no neighbours")
                        })?;
                        steps.push(Hero { at: to, facing });
                    }
                    (None, Some(to)) => {
                        // stub: MCL-Q44 (a step into sight: the tile it left not told)
                        unseen.push(steps.len());
                        steps.push(Hero { at: to, facing });
                    }
                    // stub: MCL-Q44 (a step out of sight: the tile it entered not told)
                    _ => {}
                }
            }
            facings.push((hero, facing));
            Ok(Steps {
                hero,
                slot,
                steps,
                unseen,
            })
        })
        .collect()
}
