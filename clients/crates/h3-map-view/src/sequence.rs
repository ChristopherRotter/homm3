//! The map's recorded sequences (E24 M2, *provisional*; the epic's M2 decision):
//! `clients/map/sequences.toml`, as the harness plays them in the original (`h3map.py frames`,
//! the runbook's "The map's sequences"): each a situation of `clients/map/situations/`, the time
//! it runs to, the options the original holds and the inputs at their milliseconds, the keys by
//! xdotool's names. [`Sequence::actions`] turns the inputs into the viewer's actions
//! ([`crate::timeline::Action`]) by the original's default keys (design §3.4): Ctrl and a direction
//! key scroll the view, a direction key alone steps the selected hero, a key's release does
//! nothing; since E24 M3 T6b1 a panel button's letter presses the button and its release clicks
//! it, and the left button's press and release come apart. A walk's paths are the original's, from the recorded timeline ([`Sequence::paths`]),
//! and its hero the live part's ([`selected`]): M2's walks are scripted (the epic's M2 decision).

use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use h3_map::model::Pos;
use serde::Deserialize;

use crate::live::{Arrow, Live, LiveHero, Selected};
use crate::tables::{HERO_SPEEDS, SCROLL_SPEEDS};
use crate::timeline::{Action, Asked, Direction, Hero, Hotkey};

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
    /// The status window's kind the harness asks at 0 and its end, [kind, ms] (E24 M3 T5b2,
    /// design §4.9: 1 the date's, 2 the kingdom's; built afresh by 0's redraw).
    pub status: Option<[u32; 2]>,
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
    /// The left button pressed at a point (E24 M3 T6b1).
    pub press: Option<[i32; 2]>,
    /// The left button released at a point (E24 M3 T6b1).
    pub release: Option<[i32; 2]>,
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

    /// The status window's kind asked at 0 (E24 M3 T5b2), checked as the harness checks it: 1 or
    /// 2, its end from 1 to 600 000 ms.
    ///
    /// # Errors
    /// Another kind or end.
    pub fn asked(&self) -> Result<Option<Asked>, String> {
        self.status
            .map(|[kind, end]| match (u8::try_from(kind), end) {
                (Ok(kind @ (1 | 2)), 1..=600_000) => Ok(Asked { kind, end }),
                _ => Err(format!(
                    "{}: `status` [{kind}, {end}] is not [1 or 2, 1 to 600000]",
                    self.id
                )),
            })
            .transpose()
    }

    /// Whether the sequence walks a hero (a click, a direction key without Ctrl, the hero's
    /// options): its timeline needs the selected hero ([`selected`]) and its paths, its draws
    /// lift and move that hero ([`selected_index`]).
    #[must_use]
    pub fn walks(&self) -> bool {
        self.hero_speed.is_some()
            || self.points.is_some()
            || self.inputs.iter().any(|i| {
                i.click.is_some()
                    || i.press.is_some()
                    || i.keydown
                        .as_deref()
                        .is_some_and(|k| matches!(key(k), Some((false, _))) || k == SPACE)
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
                + usize::from(input.click.is_some())
                + usize::from(input.press.is_some())
                + usize::from(input.release.is_some());
            if kinds != 1 {
                return Err(failed(
                    "not exactly one of keydown, keyup, pointer, click, press, release",
                ));
            }
            if !(last..=self.ms).contains(&input.at) {
                return Err(failed(&format!("out of order or outside 1 to {}", self.ms)));
            }
            last = input.at;
            let action = if let Some(name) = &input.keydown {
                down.push(name);
                Some(match (key(name), hotkey(name)) {
                    (Some((true, direction)), _) => Action::Scroll(direction),
                    (Some((false, direction)), _) => Action::Step(direction),
                    (None, Some(hotkey)) => Action::Key(hotkey),
                    (None, None) if name == SPACE => Action::Visit,
                    (None, None) => {
                        return Err(failed(&format!("`{name}`: no key of the map's")));
                    }
                })
            } else if let Some(name) = &input.keyup {
                let Some(at) = down.iter().position(|d| d == name) else {
                    return Err(failed(&format!("`{name}` released, never pressed")));
                };
                down.remove(at);
                // Another key's release is a message that does nothing (E24 M3 T6b3: it counts
                // in the input ring).
                Some(hotkey(name).map_or(Action::Up, Action::KeyUp))
            } else {
                input
                    .pointer
                    .map(Action::Pointer)
                    .or(input.click.map(Action::Click))
                    .or(input.press.map(Action::Press))
                    .or(input.release.map(Action::Release))
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
        for input in self
            .inputs
            .iter()
            .filter(|i| i.click.is_some() || i.press.is_some())
        {
            let shown = lines
                .iter()
                .find(|(t, arrows)| *t >= input.at && !arrows.is_empty());
            if let Some((_, path)) = shown.filter(|(_, p)| !paths.contains(p)) {
                paths.push(path.clone());
            }
        }
        Ok(paths)
    }

    /// The selected hero's movement points memory held, as its recorded timeline shows them (E24
    /// M3 T4b, M3's scripted walks: `h3map.py frames`' `<ms> points <n>` lines), each with its
    /// millisecond, in order: the timeline's [`crate::timeline::Start::points`].
    ///
    /// # Errors
    /// A `points` line that does not read.
    pub fn points(&self, recorded: &str) -> Result<Vec<(u32, i32)>, String> {
        let mut points = Vec::new();
        for line in recorded.lines() {
            let mut words = line.split_whitespace();
            let (Some(t), Some("points")) = (words.next(), words.next()) else {
                continue;
            };
            let read = match (t.parse(), words.next().map(str::parse), words.next()) {
                (Ok(t), Some(Ok(n)), None) => Some((t, n)),
                _ => None,
            };
            points
                .push(read.ok_or_else(|| format!("{}: the line `{line}` does not read", self.id))?);
        }
        Ok(points)
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
    let hero = &heroes[selected_index(heroes, viewer)?];
    let facing = usize::from(hero.facing.number());
    Ok(Hero {
        at: hero.at,
        facing: Direction::ALL[facing % Direction::ALL.len()],
    })
}

/// The index in `heroes` of the hero [`selected`] gives (the one a walk's draws lift and move).
///
/// # Errors
/// None or several of the viewer's colour.
pub fn selected_index(heroes: &[LiveHero], viewer: u8) -> Result<usize, String> {
    let own: Vec<usize> = heroes
        .iter()
        .enumerate()
        .filter(|(_, h)| h.owner == viewer)
        .map(|(i, _)| i)
        .collect();
    let [index] = own[..] else {
        return Err(format!(
            "{} heroes of the viewer's colour {viewer}: a walk needs one, its selected hero",
            own.len()
        ));
    };
    Ok(index)
}

/// The index in `live`'s heroes of the current player's selected hero: with the mouse-over's
/// facts (E24 M3 T6a1) the kingdom's selected hero by its record index; else [`selected_index`]'s.
///
/// # Errors
/// As [`selected_index`] without the facts; with them, no hero selected or none of its index.
pub fn selected_in(live: &Live) -> Result<usize, String> {
    let Some(facts) = &live.hover else {
        return selected_index(&live.heroes, live.viewer);
    };
    let Selected::Hero(slot) = live.kingdom.selected else {
        return Err("no hero selected".into());
    };
    let id = facts
        .hero_list
        .get(slot)
        .map(|h| h.0)
        .ok_or("the selected hero is past the facts' list")?;
    facts
        .heroes
        .iter()
        .position(|h| h.0 == id)
        .filter(|&i| i < live.heroes.len())
        .ok_or_else(|| format!("the selected hero {id} is not on the view's level"))
}

/// The hero [`selected_in`] gives, on its tile and facing as the live part at 0 holds it.
///
/// # Errors
/// As [`selected_in`].
pub fn selected_hero(live: &Live) -> Result<Hero, String> {
    let hero = &live.heroes[selected_in(live)?];
    let facing = usize::from(hero.facing.number());
    Ok(Hero {
        at: hero.at,
        facing: Direction::ALL[facing % Direction::ALL.len()],
    })
}

/// A panel button's hotkey by xdotool's name (E24 M3 T6b1: its letter in lower case, `k` …).
fn hotkey(name: &str) -> Option<Hotkey> {
    Hotkey::ALL
        .into_iter()
        .find(|h| h.letter().to_ascii_lowercase() == name)
}

/// Space by xdotool's name (E24 M3 T6b3: [`Action::Visit`]).
const SPACE: &str = "space";

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
