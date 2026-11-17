//! The turn's end and the next day (E24 M4 T1a, *provisional*; `docs/epics/E24-design.md` §5.1):
//! the tables' `[map.turn]`, End Turn's question's text and the week's and the month's message's
//! (`player_turn_start` 0x4CC7D0 at 0x4CCAE5, ArrayTxt's records as `arraytxt_load` 0x5B9CC0
//! copies them), the keys and families they name. The timeline (`crate::timeline`) opens the boxes
//! and plays the turn's change; the adapter lays the boxes out (`h3_ui::message`) with the texts
//! given here. Since E24 M4 T1b2 (design §5.2) what a turn's facts give the timeline of the map's
//! heroes: those standing ([`placed`]) and a computer player's walks in sight ([`walks`]).

use alloc::format;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use h3_ui::text::{Args, Catalogue};
use serde::Deserialize;

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
    Ok(vec![Steps { hero, slot, steps }])
}
