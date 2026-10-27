//! The map's recorded sequences (E24 M2, *provisional*; the epic's M2 decision):
//! `clients/map/sequences.toml`, as the harness plays them in the original (`h3map.py frames`,
//! the runbook's "The map's sequences"): each a situation of `clients/map/situations/`, the time
//! it runs to, the options the original holds and the inputs at their milliseconds, the keys by
//! xdotool's names. [`Sequence::actions`] turns the inputs into the viewer's actions
//! ([`crate::timeline::Action`]) by the original's default keys (design §3.4): Ctrl and a direction
//! key scroll the view, a direction key alone steps the selected hero, a key's release does
//! nothing. A walk's paths are the original's, from the recorded timeline ([`Sequence::paths`]),
//! and its hero the live part's ([`selected`]): M2's walks are scripted (the epic's M2 decision).

use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use h3_map::model::Pos;
use serde::Deserialize;

use crate::live::{Arrow, LiveHero};
use crate::tables::{HERO_SPEEDS, SCROLL_SPEEDS};
use crate::timeline::{Action, Direction, Hero};

/// The file.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Sequences {
    /// The sequences in the file's order.
    pub sequence: Vec<Sequence>,
}

/// One sequence.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Sequence {
    /// Its id (its timeline is `timelines/<id>.txt`).
    pub id: String,
    /// Its situation's id (`situations/<situation>.toml`).
    pub situation: String,
    /// The millisecond it runs to.
    pub ms: u32,
    /// What it shows.
    pub why: String,
    /// The scroll speed the original holds (0 to 2; the original's 1 when its settings hold none).
    pub scroll_speed: Option<usize>,
    /// The hero speed (1 to 4; the original's 2 when its settings hold none: the walk).
    pub hero_speed: Option<usize>,
    /// The selected hero's movement points (the harness writes them; the path's arrows the
    /// original shows tell the walk where they end).
    pub points: Option<u32>,
    /// The inputs in time order.
    #[serde(default)]
    pub inputs: Vec<Input>,
}

/// One input: at a millisecond, exactly one of a key pressed or released, the pointer moved, a
/// click.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Input {
    /// The millisecond (1 to the sequence's `ms`).
    pub at: u32,
    /// A key pressed (xdotool's name, `ctrl+` holding Control with it).
    pub keydown: Option<String>,
    /// A key released.
    pub keyup: Option<String>,
    /// The pointer moved to a point of the frame.
    pub pointer: Option<[i32; 2]>,
    /// The left button clicked at a point.
    pub click: Option<[i32; 2]>,
}

/// The scroll speed when a sequence gives none (`settings_read` 0x50B7B0: any value but 0 to 2 is
/// 1).
pub const DEFAULT_SCROLL_SPEED: usize = 1;

/// The hero speed when a sequence gives none (`option_walk_speed` 0x69875C: any value but 1 to 4
/// is 2).
pub const DEFAULT_HERO_SPEED: usize = 2;

/// The direction keys by xdotool's names (X keysyms), the original's scan codes 72, 73, 77, 81, 80,
/// 79, 75, 71 (`adventure_key_down` 0x408C40, design §3.4): north clockwise to north-west.
const DIRECTION_KEYS: [(&str, Direction); 8] = [
    ("Up", Direction::North),
    ("Prior", Direction::NorthEast),
    ("Right", Direction::East),
    ("Next", Direction::SouthEast),
    ("Down", Direction::South),
    ("End", Direction::SouthWest),
    ("Left", Direction::West),
    ("Home", Direction::NorthWest),
];

impl Sequences {
    /// Parses the file.
    ///
    /// # Errors
    /// The parser's message.
    pub fn parse(text: &str) -> Result<Self, String> {
        toml::from_str(text).map_err(|e| e.to_string())
    }

    /// The sequence of that id.
    #[must_use]
    pub fn get(&self, id: &str) -> Option<&Sequence> {
        self.sequence.iter().find(|s| s.id == id)
    }
}

impl Sequence {
    /// The scroll speed (its own or the default), checked.
    ///
    /// # Errors
    /// A speed out of 0 to 2.
    pub fn scroll_speed(&self) -> Result<usize, String> {
        let speed = self.scroll_speed.unwrap_or(DEFAULT_SCROLL_SPEED);
        if speed >= SCROLL_SPEEDS {
            return Err(format!(
                "{}: the scroll speed {speed} is not 0 to 2",
                self.id
            ));
        }
        Ok(speed)
    }

    /// The hero speed (its own or the default), checked.
    ///
    /// # Errors
    /// A speed out of 1 to 4.
    pub fn hero_speed(&self) -> Result<usize, String> {
        let speed = self.hero_speed.unwrap_or(DEFAULT_HERO_SPEED);
        if !(1..=HERO_SPEEDS).contains(&speed) {
            return Err(format!("{}: the hero speed {speed} is not 1 to 4", self.id));
        }
        Ok(speed)
    }

    /// Whether the sequence walks a hero (a click, a direction key without Ctrl, the hero's
    /// options): its timeline needs the selected hero ([`selected`]) and its paths, its frames
    /// the walk drawn (M2 T3b).
    #[must_use]
    pub fn walks(&self) -> bool {
        self.hero_speed.is_some()
            || self.points.is_some()
            || self.inputs.iter().any(|i| {
                i.click.is_some()
                    || i.keydown
                        .as_deref()
                        .is_some_and(|k| matches!(key(k), Some((false, _))))
            })
    }

    /// The actions at their milliseconds, checked as the harness checks them (`h3map.py`'s
    /// `sequence_inputs`): each input one kind at 1 to `ms`, in time order, a key released only
    /// once pressed; here also a key the map takes (a direction key, with or without Ctrl).
    ///
    /// # Errors
    /// What breaks a rule.
    pub fn actions(&self) -> Result<BTreeMap<u32, Vec<Action>>, String> {
        let mut actions: BTreeMap<u32, Vec<Action>> = BTreeMap::new();
        let mut down: Vec<&str> = Vec::new();
        let mut last = 1;
        for input in &self.inputs {
            let failed = |why: &str| format!("{}: the input at {}: {why}", self.id, input.at);
            let kinds = usize::from(input.keydown.is_some())
                + usize::from(input.keyup.is_some())
                + usize::from(input.pointer.is_some())
                + usize::from(input.click.is_some());
            if kinds != 1 {
                return Err(failed("not exactly one of keydown, keyup, pointer, click"));
            }
            if !(last..=self.ms).contains(&input.at) {
                return Err(failed(&format!("out of order or outside 1 to {}", self.ms)));
            }
            last = input.at;
            let action = if let Some(name) = &input.keydown {
                let (ctrl, direction) =
                    key(name).ok_or_else(|| failed(&format!("`{name}`: no key of the map's")))?;
                down.push(name);
                Some(if ctrl {
                    Action::Scroll(direction)
                } else {
                    Action::Step(direction)
                })
            } else if let Some(name) = &input.keyup {
                let Some(at) = down.iter().position(|d| d == name) else {
                    return Err(failed(&format!("`{name}` released, never pressed")));
                };
                down.remove(at);
                None
            } else {
                input
                    .pointer
                    .map(Action::Pointer)
                    .or(input.click.map(Action::Click))
            };
            if let Some(action) = action {
                actions.entry(input.at).or_default().push(action);
            }
        }
        Ok(actions)
    }

    /// The paths its clicks show, as the original showed them (M2's scripted walks): for each
    /// click, the first `arrows` line at or after it that lists any, from its recorded timeline
    /// (`h3map.py frames`: `<ms> arrows <x>,<y>,<level>,<arrow> ...`), each path once.
    ///
    /// # Errors
    /// An `arrows` line that does not read.
    pub fn paths(&self, recorded: &str) -> Result<Vec<Vec<Arrow>>, String> {
        let mut lines = Vec::new();
        for line in recorded.lines() {
            let mut words = line.split_whitespace();
            let (Some(t), Some("arrows")) = (words.next(), words.next()) else {
                continue;
            };
            let failed = || format!("{}: the line `{line}` does not read", self.id);
            let t: u32 = t.parse().map_err(|_| failed())?;
            let arrows: Option<Vec<Arrow>> = words.map(arrow).collect();
            lines.push((t, arrows.ok_or_else(failed)?));
        }
        let mut paths: Vec<Vec<Arrow>> = Vec::new();
        for input in self.inputs.iter().filter(|i| i.click.is_some()) {
            let shown = lines
                .iter()
                .find(|(t, arrows)| *t >= input.at && !arrows.is_empty());
            if let Some((_, path)) = shown.filter(|(_, p)| !paths.contains(p)) {
                paths.push(path.clone());
            }
        }
        Ok(paths)
    }
}

/// An arrow of an `arrows` line: `<x>,<y>,<level>,<arrow>`.
fn arrow(word: &str) -> Option<Arrow> {
    let parts: Vec<&str> = word.split(',').collect();
    let [x, y, z, arrow] = parts[..] else {
        return None;
    };
    match (x.parse(), y.parse(), z.parse(), arrow.parse()) {
        (Ok(x), Ok(y), Ok(z), Ok(arrow)) => Some(Arrow {
            at: Pos { x, y, z },
            arrow,
        }),
        _ => None,
    }
}

/// The hero a walking sequence moves: the live part's only hero of the viewing player's colour
/// (the current player's selected hero), on its tile and facing as the live part at 0 holds it.
///
/// # Errors
/// None or several of that colour.
pub fn selected(heroes: &[LiveHero], viewer: u8) -> Result<Hero, String> {
    let own: Vec<&LiveHero> = heroes.iter().filter(|h| h.owner == viewer).collect();
    let [hero] = own[..] else {
        return Err(format!(
            "{} heroes of the viewer's colour {viewer}: a walk needs one, its selected hero",
            own.len()
        ));
    };
    let facing = usize::from(hero.facing.number());
    Ok(Hero {
        at: hero.at,
        facing: Direction::ALL[facing % Direction::ALL.len()],
    })
}

/// A key's name: with Ctrl or not, and its direction.
fn key(name: &str) -> Option<(bool, Direction)> {
    let (ctrl, bare) = name
        .strip_prefix("ctrl+")
        .map_or((false, name), |rest| (true, rest));
    DIRECTION_KEYS
        .iter()
        .find(|(k, _)| *k == bare)
        .map(|&(_, d)| (ctrl, d))
}

#[cfg(test)]
mod tests;
