//! A record's start: the mouse-over's facts (E24 M4 T3b2, *provisional*;
//! `docs/epics/E24-design.md` §5.11): the live part's `[hover]` table (`crate::hover::Facts`) from
//! the record's first observation, as a capture's keeps it (`h3mem.Mem.hover`), so that the map's
//! hint and cursor read the engine's game.
//!
//! - **Per map object** (the map's order): its entrance's word as the hint reads it, a monster's
//!   count from the word the view gives (its first count: the original's thresholds,
//!   `hover::SIZES`, are the view's, ADV-FOG-006) and a town's index among the towns written;
//!   whether a town or a garrison defends itself (its garrison as the viewer's level shows it, a
//!   hero standing on its entrance); what the viewer knows of it (the kinds its team marked, the
//!   selected hero's marks, the player's visits) and its contents (a mine's resource and guards,
//!   a Shrine's spell, a Tree's price, a Witch Hut's skill as the player's visits learnt them).
//! - **The towns** the viewer sees, in the view's order: each one's name the map's own, else its
//!   faction's first (the client's: the engine draws none), and its faction; the viewer's towns
//!   among them in its list's order (the panels': the view's, MCL-Q38).
//! - **The heroes** in the view's order (the live part's `heroes`): each one's type and whether
//!   the map names it; the viewer's in their types' order (the panels', `super::panels`).
//! - **The teams** the map's, each colour its own without them.
//! - **Not in the record** (MCL-Q37, the engine lanes' inboxes): the selected hero's costs over
//!   several days (the cursor's days; a `Moves` query gives today's and no record holds an
//!   answer), the tiles a monster guards (the attack cursor), whether another team's town or
//!   garrison holds an army at the viewer's level 0, a visited bank's guards: none written (no
//!   guarded tile, no reach, no army, no guards). A Seer's Hut's and a Quest Guard's quest neither
//!   (the engine's views show neither object yet: their hint is their name, as before a visit).
//!   No rule is copied into the client for them (the user, 2026-11-19).

use alloc::collections::BTreeSet;
use alloc::vec::Vec;

use h3_adventure::{ObjectKind, ObjectTypeId};
use h3_game::GameRuleset;
use h3_kernel::Handle;
use h3_map::RawMap;
use h3_protocol::wire::game::{
    GameObservation, Garrison, HeroDetails, Known, MapHero, Object, ObjectRef, OwnMapHero, Pos,
};
use toml::{Table, Value};

use super::{Creatures, int, row};
use crate::art::MapArt;
use crate::hover::{SEVERAL, SIZES, VISITS};
use crate::live::TOWN_CLASS;
use crate::objects;
use crate::picks::town_name;

/// A monster (class 54).
const MONSTER: u32 = 54;
/// A map garrison (class 33).
const GARRISON: u32 = 33;
/// A mine (class 53).
const MINE: u32 = 53;
/// The abandoned mine's subclass, whose resource the game draws (the name's eighth).
const ABANDONED: u32 = 7;
/// The creature banks' classes (the hint's case 0x40D3F0).
const BANKS: [u32; 5] = [16, 24, 25, 84, 85];
/// The Shrines' classes.
const SHRINES: [u32; 3] = [88, 89, 90];
/// A Tree of Knowledge.
const TREE: u32 = 102;
/// A Witch Hut.
const WITCH_HUT: u32 = 113;
/// A Keymaster's Tent (its mark a `visits` rule, design §4.12).
const TENT: u32 = 10;
/// The Fountain of Fortune: its bonus known by the player's own visit (design §4.11).
const FORTUNE: u32 = 30;
/// The classes of [`VISITS`] whose mark the hint shows only with a hero selected (design §4.11).
const BY_HERO: [u32; 27] = [
    4, 11, 14, 22, 23, 28, 30, 31, 32, 38, 39, 41, 47, 48, 49, 51, 52, 56, 61, 63, 64, 92, 94, 96,
    100, 107, 110,
];
/// Of those, the ones a player's visit marks (the player record's bits, the Pyramid's word).
const BY_PLAYER: [u32; 4] = [22, 39, 48, 63];
/// The Water Wheel and the Windmill: a mark only once the player visited them.
const ONCE_VISITED: [u32; 2] = [109, 112];
/// The Tree of Knowledge's prices' resources: gold (its price 1) and gems (2).
const GOLD: u16 = 6;
/// Gems.
const GEMS: u16 = 5;

/// What the view shows of the map's objects, by the map's order (none for an object it does not
/// show), with each one's class as the game loads it (`Records::game_class` of the live part's:
/// the class the hint dispatches on, 219 a garrison's 33, 220 a mine's 53) and subclass.
pub(super) struct Sight<'a> {
    /// The original's number of each creature handle (E24 M5 T4c: a mine's guards).
    pub creatures: &'a Creatures,
    /// The view's object of each map object.
    pub views: Vec<Option<&'a Object>>,
    /// Each map object's class as the game loads it, and its subclass.
    pub classes: Vec<(u32, u32)>,
}

/// The `[hover]` table of `observation` on the map `raw` (its art `art`) under `rules`, the map's
/// objects as `sight` shows them.
pub(super) fn table(
    observation: &GameObservation,
    (raw, picks): (&RawMap, Option<u64>),
    art: &MapArt,
    rules: &GameRuleset,
    sight: &Sight<'_>,
) -> Table {
    let adventure = &observation.adventure;
    let viewer = adventure.viewer;
    let info = |id: u16| observation.heroes.iter().find(|h| h.id == id);
    let own_name =
        |hero_type: u16| u8::try_from(hero_type).is_ok_and(|h| art.hero_names.contains_key(&h));
    let class_of = |hero_type: u16| {
        rules
            .core()
            .hero_type(Handle::from_raw(hero_type))
            .map_or(0, |t| int(t.class.index()))
    };
    // The towns the viewer sees, in the view's order; the viewer's among them.
    let towns: Vec<Value> = observation
        .towns
        .iter()
        .map(|t| {
            let name = t.name.as_ref().map_or_else(
                || Value::Integer(i64::from(town_name(t.faction, t.town, picks))),
                |own| Value::String(own.clone()),
            );
            Value::Array(alloc::vec![name, Value::Integer(i64::from(t.faction))])
        })
        .collect();
    let town_list: Vec<i64> = super::panels::viewer_towns(observation)
        .into_iter()
        .filter_map(|t| observation.towns.iter().position(|o| o.town == t.town))
        .map(int)
        .collect();
    let hero_ids: Vec<Value> = adventure
        .heroes
        .iter()
        .map(|h| {
            let t = info(h.id).map_or(0, |i| i.hero_type);
            row(&[i64::from(t), i64::from(own_name(t))])
        })
        .collect();
    // The viewer's heroes in their types' order (the panels' list); the first the selected one.
    let mut mine: Vec<(u16, usize, &MapHero)> = adventure
        .heroes
        .iter()
        .enumerate()
        .filter(|(_, h)| h.owner == viewer)
        .filter_map(|(i, h)| info(h.id).map(|info| (info.hero_type, i, h)))
        .collect();
    mine.sort_by_key(|&(t, i, _)| (t, i));
    let hero_list: Vec<Value> = mine
        .iter()
        .map(|&(t, _, _)| row(&[i64::from(t), class_of(t), i64::from(own_name(t))]))
        .collect();
    let selected = mine.first().map(|&(_, _, h)| h);
    let own = selected.and_then(|h| h.own.as_ref());
    let details = selected
        .and_then(|h| info(h.id))
        .and_then(|i| i.details.as_ref());
    let teams: Vec<i64> = (0..8u8)
        .map(|c| {
            raw.teams
                .as_ref()
                .and_then(|t| t.of_player.get(usize::from(c)))
                .map_or(i64::from(c), |&code| i64::from(code))
        })
        .collect();
    let kinds: BTreeSet<u16> = adventure.kinds.iter().copied().collect();
    let mut words = Vec::with_capacity(sight.views.len());
    let mut armed = Vec::with_capacity(sight.views.len());
    let mut visits = Vec::with_capacity(sight.views.len());
    let mut contents = Contents::default();
    for (index, (view, &(class, subclass))) in sight.views.iter().zip(&sight.classes).enumerate() {
        let entrance = objects::entrance(art, index);
        let Some((object, (ex, ey))) = view.zip(entrance) else {
            words.push(Value::Integer(-1));
            armed.push(Value::Integer(0));
            visits.push(row(&[0, -1]));
            continue;
        };
        let level = object.anchor.level;
        let at_entrance = |p: &Pos| (i32::from(p.x), i32::from(p.y), p.level) == (ex, ey, level);
        let reference = ObjectRef {
            object_type: object.object_type,
            anchor: object.anchor,
        };
        let word = match class {
            MONSTER => object.monster.map_or(0, |m| first_count(m.count_word)),
            TOWN_CLASS => object
                .town
                .and_then(|id| observation.towns.iter().position(|t| t.town == id))
                .map_or(0, int),
            _ => 0,
        };
        words.push(Value::Integer(word));
        let visitor = adventure.heroes.iter().any(|h| at_entrance(&h.pos));
        let defended = match class {
            TOWN_CLASS => object
                .town
                .and_then(|id| observation.towns.iter().find(|t| t.town == id))
                .is_some_and(|t| visitor || holds(&t.garrison)),
            GARRISON => observation
                .garrisons
                .iter()
                .find(|g| (g.object_type, g.anchor) == (object.object_type, object.anchor))
                .is_some_and(|g| holds(&g.garrison)),
            _ => false,
        };
        armed.push(Value::Integer(i64::from(defended)));
        let kind = rules
            .adventure()
            .object_type(ObjectTypeId::from_raw(object.object_type))
            .map(|t| &t.kind);
        let known = adventure
            .known
            .iter()
            .find(|k| (k.object_type, k.anchor) == (object.object_type, object.anchor))
            .map(|k| &k.known);
        let player_visited = known.is_some() || object.visited;
        let hero_visited = own.is_some_and(|o| marked(o, reference, kind));
        let bonus = if class == FORTUNE {
            known.is_some()
        } else {
            kinds.contains(&object.object_type)
        };
        let mark = if class == TENT {
            let colour = u32::from(adventure.keys) >> subclass & 1;
            i64::from(colour)
        } else if BY_PLAYER.contains(&class) {
            selected.map_or(-1, |_| i64::from(player_visited || hero_visited))
        } else if BY_HERO.contains(&class) {
            selected.map_or(-1, |_| i64::from(hero_visited))
        } else if ONCE_VISITED.contains(&class) {
            if player_visited { 1 } else { -1 }
        } else if VISITS.contains(&class) && ![35, 104].contains(&class) {
            i64::from(player_visited)
        } else {
            -1
        };
        visits.push(row(&[i64::from(bonus), mark]));
        let numbers = sight.creatures;
        contents.add(
            index, class, subclass, object, known, bonus, own, details, numbers,
        );
    }
    let rows = art.size.saturating_mul(art.levels);
    // stub: MCL-Q37 (the tiles a monster guards: none in the record)
    let guarded: Vec<Value> = (0..rows)
        .map(|_| Value::String("0".repeat(art.size)))
        .collect();
    let mut t = Table::new();
    t.insert("words".into(), Value::Array(words));
    t.insert("armed".into(), Value::Array(armed));
    t.insert("towns".into(), Value::Array(towns));
    t.insert("hero_ids".into(), Value::Array(hero_ids));
    t.insert("hero_list".into(), Value::Array(hero_list));
    t.insert("town_list".into(), row(&town_list));
    t.insert("teams".into(), row(&teams));
    t.insert("guarded".into(), Value::Array(guarded));
    // stub: MCL-Q37 (the selected hero's costs over several days: none in the record)
    t.insert("reach".into(), Value::Array(Vec::new()));
    let day_points = own.map_or(0, |o| i64::from(o.max_movement));
    t.insert("day_points".into(), Value::Integer(day_points));
    t.insert("visits".into(), Value::Array(visits));
    contents.insert(&mut t);
    t
}

/// The first count of the size word `word` (the view's, ADV-FOG-006): what the hint's
/// thresholds read back as that word.
fn first_count(word: u8) -> i64 {
    usize::from(word)
        .checked_sub(1)
        .map_or(1, |i| SIZES.get(i).map_or(i64::MAX, |&s| i64::from(s)))
}

/// Whether a garrison as the viewer's level shows it holds an army: a stack at level 1 and up,
/// or by slot for its owner's team; at level 0 the view tells nothing (MCL-Q37: none written).
fn holds(garrison: &Garrison) -> bool {
    // stub: MCL-Q37 (another team's town or garrison at level 0: whether it holds an army)
    !garrison.stacks.is_empty() || garrison.army.as_ref().is_some_and(|a| !a.is_empty())
}

/// Whether the selected hero `own` holds the mark of the object `object` of the kind `kind`: its
/// once-per-hero mark of the object, its type's day or week mark, or its kind's once-mark.
fn marked(own: &OwnMapHero, object: ObjectRef, kind: Option<&ObjectKind>) -> bool {
    let once = match kind {
        Some(
            ObjectKind::Bonus { mark, .. }
            | ObjectKind::FountainOfFortune { mark, .. }
            | ObjectKind::Sirens { mark, .. },
        ) => Some(*mark),
        _ => None,
    };
    own.marks.contains(&object)
        || own.timed.contains(&object.object_type)
        || once.is_some_and(|m| own.bonuses.once.iter().any(|b| b.mark == m))
}

/// The contents' rows of the `[hover]` table (`hover::contents`' written lists).
#[derive(Default)]
struct Contents {
    mines: Vec<Value>,
    banks: Vec<Value>,
    shrines: Vec<Value>,
    trees: Vec<Value>,
    witch_huts: Vec<Value>,
}

impl Contents {
    /// The row of the object `index` of `class` and `subclass` (its view `object`, what the
    /// player learnt of it `known`, its kind known `bonus`), the selected hero's `own` view and
    /// `details`, a creature by the original's number (`creatures`: a mine's guards).
    #[allow(clippy::too_many_arguments)]
    fn add(
        &mut self,
        index: usize,
        class: u32,
        subclass: u32,
        object: &Object,
        known: Option<&Known>,
        bonus: bool,
        own: Option<&OwnMapHero>,
        details: Option<&HeroDetails>,
        creatures: &Creatures,
    ) {
        let index = int(index);
        let kind_known = i64::from(bonus);
        if class == MINE {
            let resource = object.production.map_or_else(
                || {
                    if subclass < ABANDONED {
                        i64::from(subclass)
                    } else {
                        0
                    }
                },
                |p| i64::from(p.resource),
            );
            let guards = object.guards.as_ref().map_or_else(Vec::new, |g| {
                let creature = g
                    .creature
                    .map_or(i64::from(SEVERAL), |c| i64::from(creatures.number(c)));
                alloc::vec![row(&[creature, first_count(g.count_word)])]
            });
            self.mines.push(Value::Array(alloc::vec![
                Value::Integer(index),
                Value::Integer(resource),
                Value::Array(guards),
            ]));
        } else if BANKS.contains(&class) {
            // stub: MCL-Q37 (a visited bank's guards: none in the record)
            self.banks.push(Value::Array(alloc::vec![
                Value::Integer(index),
                Value::Integer(i64::from(known.is_some())),
                Value::Array(Vec::new()),
            ]));
        } else if SHRINES.contains(&class) {
            let spell = match known {
                Some(Known::Spell { spell: Some(s) }) => Some(*s),
                _ => None,
            };
            let learned = spell.is_some_and(|s| details.is_some_and(|d| d.spells.contains(&s)));
            self.shrines.push(row(&[
                index,
                kind_known,
                spell.map_or(-1, i64::from),
                i64::from(learned),
            ]));
        } else if class == TREE {
            let price = match known {
                Some(Known::Price { price }) => Some(if price.iter().any(|a| a.resource == GEMS) {
                    2
                } else {
                    i64::from(price.iter().any(|a| a.resource == GOLD))
                }),
                _ => None,
            };
            let reference = ObjectRef {
                object_type: object.object_type,
                anchor: object.anchor,
            };
            let mark = own.map_or(-1, |o| i64::from(o.marks.contains(&reference)));
            self.trees
                .push(row(&[index, kind_known, price.unwrap_or(-1), mark]));
        } else if class == WITCH_HUT {
            let skill = match known {
                Some(Known::Skill { skill: Some(s) }) => Some(*s),
                _ => None,
            };
            let learned = skill
                .is_some_and(|s| details.is_some_and(|d| d.skills.iter().any(|k| k.skill == s)));
            self.witch_huts.push(row(&[
                index,
                kind_known,
                skill.map_or(-1, i64::from),
                i64::from(learned),
            ]));
        }
    }

    /// The lists inserted into `t`.
    fn insert(self, t: &mut Table) {
        for (key, list) in [
            ("mines", self.mines),
            ("banks", self.banks),
            ("shrines", self.shrines),
            ("trees", self.trees),
            ("witch_huts", self.witch_huts),
        ] {
            t.insert(key.into(), Value::Array(list));
        }
    }
}
