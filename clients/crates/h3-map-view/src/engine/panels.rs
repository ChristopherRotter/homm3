//! A record's start: the panels' live part (E24 M4 T3b1, *provisional*; `docs/epics/E24-design.md`
//! §5.10): the kingdom, the status window and what choosing each hero and town shows, from the
//! record's first observation, as a capture's live part keeps them (`h3map.py`'s `kingdom`,
//! `status`, `realm` and `[chosen]`), so that `render --record` draws the whole screen.
//!
//! - **The heroes** are the viewer's on the map, in their types' order: the original's new game
//!   lists a player's heroes so (every capture's, `hero_list`), the later hires appended (M4 T4).
//!   Each one's portrait the map's (its file's hero at its tile) or its type's, its points and
//!   mana the view's. The first selected, else the first town (the original's start).
//! - **Not in the record** (MCL-Q36, the engine lanes' inboxes): a hero's morale and luck as the
//!   status window shows them (`H3Hero::GetMoraleBonus` 0x4E39B0, `GetLuckBonus` 0x4E36C0), shown
//!   0; whether a hero can still move (`hero_can_move` 0x4E5F30), shown as having points left.
//!   No rule is copied into the client for them (the user, 2026-11-19).
//! - **The towns** are the viewer's in the view's order (one a player in the comparisons so far:
//!   a shipped map's with several, T3b2's). A town's name the map's own, else its faction's first
//!   (the client's: the engine draws none, as the original's new game does); the status window's
//!   hall and fort (all it draws of the buildings) by the view's hall and fort levels; its silo
//!   the income's resources but gold (the town part's income has no other, TWN-INC-002).
//! - **The kingdom's window:** the viewer's towns by their halls, the players in the game by the
//!   map's teams (each its own without them).
//! - **The date** by the engine's calendar (`CalendarParams`): the month, the week of the month and
//!   the day of the week of the view's day.

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use h3_game::GameRuleset;
use h3_map::RawMap;
use h3_map::model::Body;
use h3_protocol::wire::game::{ArmyStack, GameObservation, HeroInfo, MapHero, Town};
use toml::{Table, Value};

use super::{Refused, int, row};
use crate::art::MapArt;
use crate::live::{HERO_CLASS, TOWN_CLASS, TOWN_NAMES};
use crate::objects;

/// The army's slots.
const SLOTS: usize = 7;
/// The resources a treasury holds, in the original's order (the packs' too: T3a's piles).
const RESOURCES: usize = 7;
/// Gold, the last of them.
const GOLD: u16 = 6;
/// The status window's kinds (`live::Status`): the kingdom's, a hero's, a town's.
const KINGDOM: i64 = 2;
/// A hero's.
const HERO: i64 = 3;
/// A town's.
const TOWN: i64 = 4;
/// The game's ids of the Village Hall, Town Hall, City Hall and Capitol (`tables`' halls).
const HALLS: [u32; 4] = [10, 11, 12, 13];
/// Of the Fort, Citadel and Castle.
const FORTS: [u32; 3] = [7, 8, 9];

/// The panels' keys of a live part's table (`kingdom`, `status`, `realm`, `chosen`), inserted into
/// `table` for `observation` on the map `raw` (its art `art`) under `rules`; `colour` turns a
/// player into its colour. `heroes` is the view's map heroes in the order of the table's
/// `heroes` (the record's).
///
/// # Errors
/// A player the view does not list; a hero of no known type.
pub(super) fn insert(
    table: &mut Table,
    observation: &GameObservation,
    raw: &RawMap,
    art: &MapArt,
    rules: &GameRuleset,
    colour: &dyn Fn(u8) -> Result<i64, Refused>,
) -> Result<(), Refused> {
    let viewer = observation.adventure.viewer;
    let info = |id: u16| observation.heroes.iter().find(|h| h.id == id);
    // The viewer's heroes on the map, in their types' order (the original's new game).
    let mut heroes: Vec<(usize, &MapHero, &HeroInfo)> = observation
        .adventure
        .heroes
        .iter()
        .enumerate()
        .filter(|(_, h)| h.owner == viewer)
        .map(|(i, h)| info(h.id).map(|info| (i, h, info)).ok_or(Refused::Hero(h.id)))
        .collect::<Result<_, _>>()?;
    heroes.sort_by_key(|&(i, _, info)| (info.hero_type, i));
    let towns: Vec<&Town> = observation
        .towns
        .iter()
        .filter(|t| t.owner == Some(viewer))
        .collect();
    let hero_status: Vec<Table> = heroes
        .iter()
        .map(|&(_, hero, info)| hero_status(hero, info, art))
        .collect();
    let town_status: Vec<Table> = towns.iter().map(|t| town_status(t)).collect();
    let kingdom_heroes = heroes
        .iter()
        .map(|&(_, hero, info)| {
            let own = hero.own.as_ref();
            let points = own.map_or(0, |o| o.movement);
            let mana = info.details.as_ref().map_or(0, |d| d.mana);
            row(&[
                0,
                // stub: MCL-Q36 (whether a hero can still move: shown as having points left)
                i64::from(points > 0),
                portrait(raw, art, hero, info),
                i64::from(points),
                i64::from(mana),
            ])
        })
        .collect();
    let kingdom_towns = towns
        .iter()
        .map(|t| {
            row(&[
                i64::from(t.faction),
                i64::from(t.fort > 0),
                i64::from(t.built_today),
            ])
        })
        .collect();
    let selected = match (heroes.len(), towns.len()) {
        (0, 0) => [0, 0],
        (0, _) => [2, 0],
        _ => [1, 0],
    };
    let resources: Vec<i64> = (0..RESOURCES)
        .map(|r| i64::from(observation.treasury.get(r).copied().unwrap_or(0)))
        .collect();
    let calendar = rules.adventure().params().calendar();
    let (week_days, month_weeks) = (
        u32::from(calendar.week_days.get()),
        u32::from(calendar.month_weeks.get()),
    );
    let since = observation.day.saturating_sub(1);
    let date = [
        since / (week_days * month_weeks) + 1,
        since / week_days % month_weeks + 1,
        calendar.weekday(observation.day).get().into(),
    ];
    let mut kingdom = Table::new();
    kingdom.insert("heroes".into(), Value::Array(kingdom_heroes));
    kingdom.insert("towns".into(), Value::Array(kingdom_towns));
    kingdom.insert("selected".into(), row(&selected));
    kingdom.insert("tops".into(), row(&[0, 0]));
    kingdom.insert("quests".into(), Value::Integer(0));
    kingdom.insert("resources".into(), row(&resources));
    kingdom.insert("date".into(), row(&date.map(i64::from)));
    table.insert("kingdom".into(), Value::Table(kingdom));
    let status = match selected[0] {
        1 => hero_status.first().cloned(),
        2 => town_status.first().cloned(),
        _ => None,
    };
    if let Some(status) = status {
        table.insert("status".into(), Value::Table(status));
    }
    table.insert(
        "realm".into(),
        Value::Table(realm(observation, raw, &towns, colour)?),
    );
    let mut chosen = Table::new();
    chosen.insert(
        "heroes".into(),
        Value::Array(
            heroes
                .iter()
                .zip(hero_status)
                .map(|(&(index, _, _), status)| {
                    let mut t = Table::new();
                    t.insert("hero".into(), Value::Integer(int(index)));
                    t.insert("status".into(), Value::Table(status));
                    t.insert("path".into(), Value::Array(Vec::new()));
                    Value::Table(t)
                })
                .collect(),
        ),
    );
    chosen.insert(
        "towns".into(),
        Value::Array(
            towns
                .iter()
                .zip(town_status)
                .map(|(town, status)| {
                    let a = town.anchor;
                    let at = town_entrance(art, town).unwrap_or([
                        i64::from(a.x),
                        i64::from(a.y),
                        i64::from(a.level),
                    ]);
                    let mut t = Table::new();
                    t.insert("at".into(), row(&at));
                    t.insert("status".into(), Value::Table(status));
                    Value::Table(t)
                })
                .collect(),
        ),
    );
    table.insert("chosen".into(), Value::Table(chosen));
    Ok(())
}

/// The map file's hero standing on `hero`'s tile, its index among the objects.
fn file_hero(art: &MapArt, hero: &MapHero) -> Option<usize> {
    let p = hero.pos;
    (0..art.objects.len()).find(|&i| {
        objects::template(art, i).is_some_and(|t| t.class == HERO_CLASS)
            && art.objects[i].anchor.z == p.level
            && objects::entrance(art, i) == Some((i32::from(p.x), i32::from(p.y)))
    })
}

/// A hero's portrait: its file's own, else its type's.
fn portrait(raw: &RawMap, art: &MapArt, hero: &MapHero, info: &HeroInfo) -> i64 {
    let own = file_hero(art, hero)
        .and_then(|i| raw.objects.get(i))
        .and_then(|o| match &o.body {
            Body::Hero(body) => body.portrait,
            _ => None,
        });
    own.map_or_else(|| i64::from(info.hero_type), i64::from)
}

/// An army by slot, as a live part's `army`: `[creature, count]` each, `[-1, 0]` an empty slot.
fn army(stacks: &[ArmyStack]) -> Value {
    Value::Array(
        (0..SLOTS)
            .map(|slot| {
                stacks
                    .iter()
                    .find(|s| usize::from(s.slot) == slot)
                    .map_or_else(
                        || row(&[-1, 0]),
                        |s| row(&[i64::from(s.creature), i64::from(s.count)]),
                    )
            })
            .collect(),
    )
}

/// A hero's status window (`live::HeroStatus`): its type, whether the map names it, its base
/// primary skills, its army; its morale and luck 0 (MCL-Q36).
fn hero_status(hero: &MapHero, info: &HeroInfo, art: &MapArt) -> Table {
    let mut t = Table::new();
    t.insert("kind".into(), Value::Integer(HERO));
    t.insert("hero".into(), Value::Integer(i64::from(info.hero_type)));
    let own_name = u8::try_from(info.hero_type).is_ok_and(|h| art.hero_names.contains_key(&h));
    t.insert("own_name".into(), Value::Integer(i64::from(own_name)));
    let skills = info
        .details
        .as_ref()
        .map_or([0; 4], |d| d.primary.map(i64::from));
    t.insert("skills".into(), row(&skills));
    // stub: MCL-Q36 (a hero's morale and luck as the status window shows them)
    t.insert("morale".into(), Value::Integer(0));
    t.insert("luck".into(), Value::Integer(0));
    let stacks = hero.own.as_ref().map_or(&[][..], |o| &o.army);
    t.insert("army".into(), army(stacks));
    t
}

/// A town's status window (`live::TownStatus`): its name, its hall and fort, its garrison's
/// mark and army, its silo and its gold.
fn town_status(town: &Town) -> Table {
    let mut t = Table::new();
    t.insert("kind".into(), Value::Integer(TOWN));
    let name = town.name.as_ref().map_or_else(
        || Value::Integer(i64::from(town.faction.saturating_mul(TOWN_NAMES))),
        |own| Value::String(own.clone()),
    );
    t.insert("name".into(), name);
    let facts = town.owner_facts.as_ref();
    let hall = facts
        .and_then(|f| usize::from(f.hall).checked_sub(1))
        .and_then(|i| HALLS.get(i));
    let fort = usize::from(town.fort)
        .checked_sub(1)
        .and_then(|i| FORTS.get(i));
    let buildings: Vec<i64> = [fort, hall].into_iter().flatten().map(|&b| b.into()).collect();
    t.insert("buildings".into(), row(&buildings));
    let garrison = facts.is_some_and(|f| f.garrison_hero);
    t.insert("garrison".into(), Value::Integer(i64::from(garrison)));
    let stacks = town.garrison.army.as_deref().unwrap_or(&[]);
    t.insert("army".into(), army(stacks));
    let income = facts.map_or(&[][..], |f| &f.income);
    let mut silo: Vec<i64> = income
        .iter()
        .filter(|a| a.resource != GOLD && a.amount > 0)
        .map(|a| i64::from(a.resource))
        .collect();
    silo.sort_unstable();
    t.insert("silo".into(), row(&silo));
    let gold = income
        .iter()
        .filter(|a| a.resource == GOLD)
        .map(|a| i64::from(a.amount))
        .sum::<i64>();
    t.insert("income".into(), Value::Integer(gold));
    t
}

/// A town's entrance: its file's object at its anchor's first visitable cell.
fn town_entrance(art: &MapArt, town: &Town) -> Option<[i64; 3]> {
    let a = town.anchor;
    let index = (0..art.objects.len()).find(|&i| {
        let o = &art.objects[i];
        objects::template(art, i).is_some_and(|t| t.class == TOWN_CLASS)
            && (u16::from(o.anchor.x), u16::from(o.anchor.y), o.anchor.z) == (a.x, a.y, a.level)
    })?;
    objects::entrance(art, index).map(|(x, y)| [i64::from(x), i64::from(y), i64::from(a.level)])
}

/// The kingdom's window (`live::KingdomStatus`): the viewer's towns by their halls, the players
/// in the game by the map's teams.
fn realm(
    observation: &GameObservation,
    raw: &RawMap,
    towns: &[&Town],
    colour: &dyn Fn(u8) -> Result<i64, Refused>,
) -> Result<Table, Refused> {
    let mut halls = [0i64; 4];
    for town in towns {
        let hall = town
            .owner_facts
            .as_ref()
            .and_then(|f| usize::from(f.hall).checked_sub(1));
        if let Some(count) = hall.and_then(|h| halls.get_mut(h)) {
            *count += 1;
        }
    }
    let team = |c: i64| {
        usize::try_from(c).map_or(usize::MAX, |c| {
            raw.teams
                .as_ref()
                .and_then(|t| t.of_player.get(c))
                .map_or(c, |&code| usize::from(code))
        })
    };
    let viewer = colour(observation.adventure.viewer)?;
    let mut by_colour = BTreeMap::new();
    for (id, _) in observation.players.iter().enumerate() {
        let c = colour(u8::try_from(id).unwrap_or(u8::MAX))?;
        by_colour.insert(c, team(c) == team(viewer));
    }
    let side = |ally: bool| -> Vec<i64> {
        by_colour
            .iter()
            .filter(|&(_, &a)| a == ally)
            .map(|(&c, _)| c)
            .collect()
    };
    let (allies, enemies) = (side(true), side(false));
    let mut t = Table::new();
    t.insert("kind".into(), Value::Integer(KINGDOM));
    t.insert("halls".into(), row(&halls));
    t.insert("allies".into(), row(&allies));
    t.insert("enemies".into(), row(&enemies));
    Ok(t)
}

/// What differs between the panels' keys of two live parts' tables (`ours` the engine's start,
/// `theirs` a capture's), appended to `found`: the kingdom key by key, the status window and the
/// kingdom's window (a town's buildings by the hall and fort its window draws; a hero's morale
/// and luck named with MCL-Q36), and, when `theirs` holds one, what choosing each hero and town
/// shows (a hero by its tile, its status and path; a town by its tile and status).
pub(super) fn differences(ours: &Table, theirs: &Table, found: &mut Vec<String>) {
    let kingdom = |t: &Table| t.get("kingdom").and_then(Value::as_table).cloned();
    match (kingdom(ours), kingdom(theirs)) {
        (Some(a), Some(b)) => {
            for key in [
                "heroes",
                "towns",
                "selected",
                "tops",
                "quests",
                "resources",
                "date",
            ] {
                if a.get(key) != b.get(key) {
                    found.push(format!("`kingdom.{key}` differs"));
                }
            }
        }
        (a, b) if a != b => found.push("`kingdom` differs".into()),
        _ => {}
    }
    for key in ["status", "realm"] {
        status_differences(key, ours.get(key), theirs.get(key), found);
    }
    let Some(theirs_chosen) = theirs.get("chosen").and_then(Value::as_table) else {
        return;
    };
    let list = |t: Option<&Table>, key: &str| {
        t.and_then(|t| t.get(key))
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default()
    };
    let ours_chosen = ours.get("chosen").and_then(Value::as_table);
    // A chosen hero's tile: its row of the table's `heroes`.
    let tile = |t: &Table, entry: &Value| {
        let index = entry
            .get("hero")
            .and_then(Value::as_integer)
            .and_then(|i| usize::try_from(i).ok());
        let hero = index.and_then(|i| t.get("heroes")?.as_array()?.get(i));
        hero.and_then(Value::as_array)
            .map(|r| r.iter().take(3).cloned().collect::<Vec<_>>())
    };
    let (a, b) = (
        list(ours_chosen, "heroes"),
        list(Some(theirs_chosen), "heroes"),
    );
    if a.len() != b.len() {
        found.push("`chosen.heroes` differ in number".into());
    }
    for (i, (x, y)) in a.iter().zip(&b).enumerate() {
        if tile(ours, x) != tile(theirs, y) || x.get("path") != y.get("path") {
            found.push(format!("`chosen.heroes` {i}: its tile or path differs"));
        }
        status_differences(
            &format!("chosen.heroes {i}"),
            x.get("status"),
            y.get("status"),
            found,
        );
    }
    let (a, b) = (
        list(ours_chosen, "towns"),
        list(Some(theirs_chosen), "towns"),
    );
    if a.len() != b.len() {
        found.push("`chosen.towns` differ in number".into());
    }
    for (i, (x, y)) in a.iter().zip(&b).enumerate() {
        if x.get("at") != y.get("at") {
            found.push(format!("`chosen.towns` {i}: its tile differs"));
        }
        status_differences(
            &format!("chosen.towns {i}"),
            x.get("status"),
            y.get("status"),
            found,
        );
    }
}

/// What differs between two status windows' tables, key by key (a town's buildings by its hall
/// and fort), appended to `found` under `name`.
fn status_differences(name: &str, ours: Option<&Value>, theirs: Option<&Value>, found: &mut Vec<String>) {
    let (Some(a), Some(b)) = (
        ours.and_then(Value::as_table),
        theirs.and_then(Value::as_table),
    ) else {
        if ours != theirs {
            found.push(format!("`{name}` differs"));
        }
        return;
    };
    let keys: BTreeSet<&String> = a.keys().chain(b.keys()).collect();
    for key in keys {
        let (x, y) = (a.get(key), b.get(key));
        let same = if key == "buildings" {
            drawn(x) == drawn(y)
        } else {
            x == y
        };
        if !same {
            let gap = if key == "morale" || key == "luck" {
                " (MCL-Q36: the record has none)"
            } else {
                ""
            };
            found.push(format!("`{name}.{key}` differs{gap}"));
        }
    }
}

/// The hall and fort a town's status window draws of its buildings: the highest of each standing.
fn drawn(buildings: Option<&Value>) -> [Option<i64>; 2] {
    let ids: Vec<i64> = buildings
        .and_then(Value::as_array)
        .map_or_else(Vec::new, |a| a.iter().filter_map(Value::as_integer).collect());
    let highest = |of: &[u32]| {
        ids.iter()
            .copied()
            .filter(|&b| of.iter().any(|&o| i64::from(o) == b))
            .max()
    };
    [highest(&HALLS), highest(&FORTS)]
}
