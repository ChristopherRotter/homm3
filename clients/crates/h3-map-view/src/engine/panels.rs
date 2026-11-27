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
use h3_map::model::{Body, HeroBody};
use h3_protocol::wire::game::{ArmyStack, GameObservation, HeroInfo, MapHero, Town};
use toml::{Table, Value};

use super::{Creatures, int, row};
use crate::art::MapArt;
use crate::live::{HERO_CLASS, TOWN_CLASS};
use crate::picks::town_name;

/// A random town's class: its template's entrance is a town's.
const RANDOM_TOWN: u32 = 77;
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
/// `table` for `observation` on the map `raw` (its art `art`) under `rules`. A chosen hero is
/// named by its place among the view's map heroes, the order of the table's `heroes`; every one
/// has its type (`super::start` refused the record otherwise). An unnamed town's name by the
/// client's picks' seed `picks` (E24 M5 T4b, `crate::picks::town_name`; none: its faction's
/// first).
pub(super) fn insert(
    table: &mut Table,
    observation: &GameObservation,
    (raw, picks): (&RawMap, Option<u64>),
    art: &MapArt,
    (rules, creatures): (&GameRuleset, &Creatures),
) {
    let viewer = observation.adventure.viewer;
    let info = |id: u16| observation.heroes.iter().find(|h| h.id == id);
    // The viewer's heroes on the map in its hero list's order (GAM-HER-003; E24 M5 T3b: a live
    // start's), at a new game their types' order (the original's), a hero the list lacks after.
    let mut heroes: Vec<(usize, &MapHero, &HeroInfo)> = observation
        .adventure
        .heroes
        .iter()
        .enumerate()
        .filter(|(_, h)| h.owner == viewer)
        .filter_map(|(i, h)| info(h.id).map(|info| (i, h, info)))
        .collect();
    heroes.sort_by_key(|&(i, hero, info)| (listed(observation, hero.id), info.hero_type, i));
    let towns = viewer_towns(observation);
    let hero_status: Vec<Table> = heroes
        .iter()
        .map(|&(_, hero, info)| hero_status(hero, info, art, creatures))
        .collect();
    let town_status: Vec<Table> = towns
        .iter()
        .map(|t| town_status(t, picks, creatures))
        .collect();
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
        Value::Table(realm(observation, raw, &towns)),
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
}

/// Hero `id`'s place in the viewer's hero list (GAM-HER-003), past its end when it is not listed.
pub(super) fn listed(observation: &GameObservation, id: u16) -> usize {
    observation
        .hero_list
        .iter()
        .position(|&h| h == id)
        .unwrap_or(usize::MAX)
}

/// The viewer's towns in its town list's order at a new game (E24 M4 T3b2, *provisional*): the
/// view's order, as the writer's maps' captures list them (`lists-towns`: the map's order). A
/// shipped map's list need not be (`start-hoard`'s Fortress before its Castle, neither the map's
/// main town: MCL-Q38).
pub(super) fn viewer_towns(observation: &GameObservation) -> Vec<&Town> {
    let viewer = observation.adventure.viewer;
    // stub: MCL-Q38 (a shipped map's town list at a new game: not the map's order)
    observation
        .towns
        .iter()
        .filter(|t| t.owner == Some(viewer))
        .collect()
}

/// The map file's hero (its body) standing on `hero`'s tile.
fn file_hero<'a>(raw: &'a RawMap, art: &MapArt, hero: &MapHero) -> Option<&'a HeroBody> {
    let p = hero.pos;
    let on = |i: usize| {
        objects::template(art, i).is_some_and(|t| t.class == HERO_CLASS)
            && art.objects.get(i).is_some_and(|o| o.anchor.z == p.level)
            && objects::entrance(art, i) == Some((i32::from(p.x), i32::from(p.y)))
    };
    raw.objects
        .iter()
        .enumerate()
        .find_map(|(i, o)| match &o.body {
            Body::Hero(body) if on(i) => Some(&**body),
            _ => None,
        })
}

/// A hero's portrait: its file's own, else its type's.
fn portrait(raw: &RawMap, art: &MapArt, hero: &MapHero, info: &HeroInfo) -> i64 {
    file_hero(raw, art, hero)
        .and_then(|body| body.portrait)
        .map_or_else(|| i64::from(info.hero_type), i64::from)
}

/// An army by slot, as a live part's `army`: `[creature, count]` each (the creature by the
/// original's number, `creatures`: E24 M5 T4c), `[-1, 0]` an empty slot.
fn army(stacks: &[ArmyStack], creatures: &Creatures) -> Value {
    Value::Array(
        (0..SLOTS)
            .map(|slot| {
                stacks
                    .iter()
                    .find(|s| usize::from(s.slot) == slot)
                    .map_or_else(
                        || row(&[-1, 0]),
                        |s| row(&[i64::from(creatures.number(s.creature)), i64::from(s.count)]),
                    )
            })
            .collect(),
    )
}

/// A hero's status window (`live::HeroStatus`): its type, whether the map names it, its base
/// primary skills, its army (by `creatures`' numbers); its morale and luck 0 (MCL-Q36).
fn hero_status(hero: &MapHero, info: &HeroInfo, art: &MapArt, creatures: &Creatures) -> Table {
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
    t.insert("army".into(), army(stacks, creatures));
    t
}

/// A town's status window (`live::TownStatus`): its name, its hall and fort, its garrison's
/// mark and army (by `creatures`' numbers), its silo and its gold.
fn town_status(town: &Town, picks: Option<u64>, creatures: &Creatures) -> Table {
    let mut t = Table::new();
    t.insert("kind".into(), Value::Integer(TOWN));
    let name = town.name.as_ref().map_or_else(
        || Value::Integer(i64::from(town_name(town.faction, town.town, picks))),
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
    let buildings: Vec<i64> = [fort, hall]
        .into_iter()
        .flatten()
        .map(|&b| b.into())
        .collect();
    t.insert("buildings".into(), row(&buildings));
    let garrison = facts.is_some_and(|f| f.garrison_hero);
    t.insert("garrison".into(), Value::Integer(i64::from(garrison)));
    let stacks = town.garrison.army.as_deref().unwrap_or(&[]);
    t.insert("army".into(), army(stacks, creatures));
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
        objects::template(art, i).is_some_and(|t| matches!(t.class, TOWN_CLASS | RANDOM_TOWN))
            && (u16::from(o.anchor.x), u16::from(o.anchor.y), o.anchor.z) == (a.x, a.y, a.level)
    })?;
    objects::entrance(art, index).map(|(x, y)| [i64::from(x), i64::from(y), i64::from(a.level)])
}

/// The kingdom's window (`live::KingdomStatus`): the viewer's towns by their halls, the players
/// in the game by the map's teams.
fn realm(observation: &GameObservation, raw: &RawMap, towns: &[&Town]) -> Table {
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
    let team = |c: u8| {
        raw.teams
            .as_ref()
            .and_then(|t| t.of_player.get(usize::from(c)))
            .map_or(usize::from(c), |&code| usize::from(code))
    };
    let colours: Vec<u8> = observation.players.iter().map(|p| p.colour).collect();
    // The viewer is among the players (`super::start` refused the record otherwise).
    let viewer = colours
        .get(usize::from(observation.adventure.viewer))
        .copied()
        .unwrap_or_default();
    let by_colour: BTreeMap<i64, bool> = colours
        .iter()
        .map(|&c| (i64::from(c), team(c) == team(viewer)))
        .collect();
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
    t
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
fn status_differences(
    name: &str,
    ours: Option<&Value>,
    theirs: Option<&Value>,
    found: &mut Vec<String>,
) {
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
        .map_or_else(Vec::new, |a| {
            a.iter().filter_map(Value::as_integer).collect()
        });
    let highest = |of: &[u32]| {
        ids.iter()
            .copied()
            .filter(|&b| of.iter().any(|&o| i64::from(o) == b))
            .max()
    };
    [highest(&HALLS), highest(&FORTS)]
}
