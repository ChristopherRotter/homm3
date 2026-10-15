//! Random play on imported maps (E08 T8b2a): a strategy for playable maps (generated, written,
//! read back and imported, heroes of several players) and a random-play driver through
//! `h3-game`, with the properties of what a command may do (`docs/05-testing.md`): atomicity,
//! legality consistency (`legal_commands`, and `legal_moves` per hero), ownership isolation,
//! conservation (armies; since E08 T13b treasuries, which change only by a visit's gains, and
//! since T14b an accepted offer's price), clone equivalence and determinism; since T14b with the
//! pending decisions (a Cartographer's offer) and their answers; since T15a with observatories,
//! Keymaster's tents and their keys, and border guards with the second offer; since T15b with
//! border gates, crossed by their key's holders and visited from a neighbour by the others; since
//! T19a with battles between two players' heroes, which the driver plays as the owner of the side
//! to act (a battle's outcome changes armies, removes heroes and moves a surrender's gold); since
//! T19b2 with runs of fighting steps that bring heroes together and play their battles mostly to
//! a victory, with hero casts, and heroes with Eagle Eye, Necromancy and Wisdom of their own, so
//! that winners capture, raise and learn (`the_plans_play_battles_to_a_victory`). What a viewer
//! may learn is T8b2b's ([`super::view`]), on the same maps and driver.

use std::sync::{Arc, LazyLock};

use h3_adventure::{
    AdventureCommand, AdventureEvent, AdventureEventKind, ObjectKind, Offer, OfferKind,
    Pos as TilePos, Reach, Viewers,
};
use h3_battle::{
    CastTargets, Command, Event, EventKind, cast_targets, castable_spells, second_targets,
};
use h3_content::{ArtifactTypeId, CreatureTypeId, ResourceId};
use h3_contracts::{BattleResult, BattleSide, HeroId, PlayerId};
use h3_game::{
    Actor, Answer, GameCommand, GameCommandError, GameEvent, GameRuleset, GameState, HeroChange,
    OpenBattle, Sink, apply, legal_commands, legal_moves,
};
use h3_pack_adventure::{ORIGINAL_PACKS, compile};
use proptest::collection::vec;
use proptest::prelude::*;
use proptest::sample::subsequence;
use proptest::test_runner::TestCaseError;

use super::encode::encode;
use super::import::{object, template};
use super::sample;
use super::strategy::version;
use crate::import::codes;
use crate::model::{
    Artifacts, Body, HeroBody, Message, MonsterBody, MonsterTreasure, RawMap, Stack, Template,
    Text, Tile, Version,
};
use crate::{Limits, import, parse};

fn rules(profile: &str) -> GameRuleset {
    let (core, part) = compile(&ORIGINAL_PACKS, profile).unwrap();
    let battle = h3_pack_battle::compile(
        &[
            h3_pack_battle::BASE_PACK,
            h3_pack_battle::AB_PACK,
            h3_pack_battle::SOD_PACK,
        ],
        profile,
    )
    .unwrap();
    GameRuleset::new(Arc::new(core), Arc::new(part), Arc::new(battle)).unwrap()
}

pub(super) static FAITHFUL: LazyLock<GameRuleset> = LazyLock::new(|| rules("base:faithful"));
pub(super) static STRICT: LazyLock<GameRuleset> = LazyLock::new(|| rules("base:strict-fixed"));
/// The faithful rules compiled a second time, for the determinism test.
static FAITHFUL_AGAIN: LazyLock<GameRuleset> = LazyLock::new(|| rules("base:faithful"));

/// The largest map side the strategy makes: small maps keep a case fast, and with the scouting
/// radius of 5 a player still starts with part of one hidden (at the end of a case, players have
/// explored a third to all of their map, two thirds on average).
const MAX_SIDE: u8 = 16;

/// The hero template (the editor's: blocked and visited one column left of the anchor), then the
/// blockers': one tile; two tiles, visited on the left; three tiles around a visited corner.
pub(super) const HERO_TEMPLATE: u32 = 0;
pub(super) const BLOCKERS: [(Mask, Mask); 3] = [
    (&[(0, 0)], &[]),
    (&[(0, 0), (1, 0)], &[(1, 0)]),
    (&[(1, 0), (0, 1), (1, 1)], &[(0, 0)]),
];
/// Blockers' object class: any the import keeps as a blocker (not a hero or a town).
pub(super) const BLOCKER_CLASS: u32 = 124;
/// The kinds of objects a plan lays out (E08 T12b): the blocker, then typed objects whose drawn
/// contents no viewer sees, as class, subclass and a resource body's amount: a wood pile of a
/// drawn amount, 300 gold, a campfire, a Redwood Observatory, the red Keymaster's tent, (E08
/// T14a) a land Cartographer, entered and not from above, (T15a) the red border guard and (T15b)
/// the red border gate. No random resource: its drawn pile type shows. The kinds after the first
/// [`PLAYED`] only the committed maps lay out (E08 T16a, `fixtures.rs`): the other piles of a
/// drawn amount, a random resource, the Pillar of Fire, and the green tent, guard and gate. Then
/// (E08 T20b3b1) the plans' wandering monsters ([`MONSTERS`]: goblins, gogs, pikemen, wolf riders
/// and centaurs, class 54 with the creature's code as the subclass) and special grounds (Cursed
/// Ground and Magic Plains), which the strategy draws by their objects ([`MONSTER_OBJECTS`],
/// [`GROUND_OBJECTS`]): appended, so the committed maps keep their templates. No random monster
/// and no drawn count: the drawn creature and its count's word would show (as a random resource's
/// pile type). Then (E08 T21a) the plans' guarded piles ([`GUARDED`]): 500 gold whose guards ask
/// first and four crystals whose guards fight at once.
pub(super) const KINDS: [(u32, u32, Option<u32>); 29] = [
    (BLOCKER_CLASS, 0, None),
    (79, 0, Some(0)),
    (79, 6, Some(3)),
    (12, 0, None),
    (58, 0, None),
    (10, 2, None),
    (13, 1, None),
    (9, 2, None),
    (212, 2, None),
    (79, 1, Some(0)),
    (79, 2, Some(0)),
    (79, 3, Some(0)),
    (79, 4, Some(0)),
    (79, 5, Some(0)),
    (79, 6, Some(0)),
    (76, 0, Some(0)),
    (60, 0, None),
    (10, 1, None),
    (9, 1, None),
    (212, 1, None),
    (54, 84, None),
    (54, 44, None),
    (54, 0, None),
    (54, 86, None),
    (54, 14, None),
    (21, 0, None),
    (46, 0, None),
    (79, 6, Some(5)),
    (79, 4, Some(4)),
];
/// The plans' wandering monsters (E08 T20b3b1), by kind: the count, the disposition (0 compliant,
/// 1 friendly, 2 aggressive, 3 hostile, 4 savage), whether it never flees, and its treasure's
/// gold. Six compliant goblins join most heroes (aggression −4, ADV-MON-007); fourteen hostile
/// gogs with 100 gold mostly fight; five aggressive pikemen are weak, so they offer to flee from
/// most heroes or join one that holds pikemen; eight savage wolf riders never flee, so they fight;
/// four friendly centaurs with 50 gold join, join for gold (Diplomacy), flee or fight.
pub(super) const MONSTERS: [(usize, u16, u8, bool, i32); 5] = [
    (20, 6, 0, false, 0),
    (21, 14, 3, false, 100),
    (22, 5, 2, false, 0),
    (23, 8, 4, true, 0),
    (24, 4, 1, false, 50),
];
/// The monsters' objects (one shape: [`masks`] gives a monster its one tile, blocked and visited).
pub(super) const MONSTER_OBJECTS: [usize; 5] = [
    BLOCKERS.len() * 20,
    BLOCKERS.len() * 21,
    BLOCKERS.len() * 22,
    BLOCKERS.len() * 23,
    BLOCKERS.len() * 24,
];
/// The special grounds' objects (no tile blocked or visited: [`masks`]; the import's ground lies on
/// its anchor, an unknown picture's one tile).
pub(super) const GROUND_OBJECTS: [usize; 2] = [BLOCKERS.len() * 25, BLOCKERS.len() * 26];
/// The plans' guarded piles (E08 T21a, ADV-PIK-004), by kind: whether the guards ask first (the
/// message has a text) and their stacks by slot as creature code and count. Eight gremlins and two
/// gnolls ask before they guard 500 gold, weak enough that most heroes beat them; twenty
/// skeletons fight at once for four crystals, strong enough that many heroes lose.
pub(super) const GUARDED: [(usize, bool, [(u8, u16, u16); 2]); 2] = [
    (27, true, [(0, 28, 8), (3, 98, 2)]),
    (28, false, [(2, 56, 20), (2, 56, 0)]),
];
/// The guarded piles' objects, of a shape with a visitable tile.
pub(super) const GUARDED_OBJECTS: [usize; 4] = [
    BLOCKERS.len() * 27 + 1,
    BLOCKERS.len() * 27 + 2,
    BLOCKERS.len() * 28 + 1,
    BLOCKERS.len() * 28 + 2,
];
/// The kinds the strategy's plans draw: the first nine.
const PLAYED: usize = 9;
/// The objects a plan may lay out: object `i` has the shape `BLOCKERS[i % 3]` and the kind
/// `KINDS[i / 3]`, and the template `1 + i` (the blockers keep templates 1 to 3). The strategy
/// draws the first `OBJECTS`, of the [`PLAYED`] kinds.
pub(super) const OBJECTS: usize = BLOCKERS.len() * PLAYED;
/// The land Cartographers of a shape with a visitable tile (E08 T14b): a plan draws them more
/// often, so that random play enters charts.
const CHARTS: [usize; 2] = [BLOCKERS.len() * 6 + 1, BLOCKERS.len() * 6 + 2];
/// The observatories, red tents, red guards and (E08 T15b) red gates of a shape with a visitable
/// tile (T15a): a plan draws them more often too, and puts a tent, a guard and a gate near its
/// first hero in three cases of four, so that random play takes keys, passes guards and meets
/// gates with and without the key.
const KEYS: [usize; 8] = [
    BLOCKERS.len() * 4 + 1,
    BLOCKERS.len() * 4 + 2,
    BLOCKERS.len() * 5 + 1,
    BLOCKERS.len() * 5 + 2,
    BLOCKERS.len() * 7 + 1,
    BLOCKERS.len() * 7 + 2,
    BLOCKERS.len() * 8 + 1,
    BLOCKERS.len() * 8 + 2,
];

/// A template mask's tiles, as `(column, row)` left of and above the anchor.
pub(super) type Mask = &'static [(u8, u8)];

/// The blocked and visitable masks of object `object` ([`OBJECTS`]): its shape's
/// ([`BLOCKERS`]), but a monster's one tile, blocked and visited, and nothing of a special
/// ground's, which neither blocks nor is visited (E08 T20b3b1).
pub(super) fn masks(object: usize) -> (Mask, Mask) {
    match KINDS[object / BLOCKERS.len()].0 {
        54 => (&[(0, 0)], &[(0, 0)]),
        21 | 46 => (&[], &[]),
        _ => BLOCKERS[object % BLOCKERS.len()],
    }
}

/// The body of an object of kind `kind`: a pile's amount (and since E08 T21a its message with
/// guards, [`GUARDED`]), a monster's data ([`MONSTERS`]; its identifier, which AB and SoD store,
/// `identifier`), else none.
fn body(kind: usize, v: Version, identifier: u32) -> Body {
    if let Some(&(_, count, disposition, never_flees, gold)) = MONSTERS.iter().find(|m| m.0 == kind)
    {
        return Body::Monster(MonsterBody {
            identifier: (v != Version::RoE).then_some(identifier),
            count,
            disposition,
            treasure: (gold > 0).then(|| MonsterTreasure {
                resources: [0, 0, 0, 0, 0, 0, gold],
                ..MonsterTreasure::default()
            }),
            never_flees,
            no_growth: false,
        });
    }
    let message = GUARDED
        .iter()
        .find(|g| g.0 == kind)
        .map(|&(_, asks, stacks)| {
            let mut guards = vec![None; 7];
            for (slot, creature, count) in stacks.into_iter().filter(|s| s.2 > 0) {
                guards[usize::from(slot)] = Some(Stack { creature, count });
            }
            Message {
                text: Text(if asks { b"Fight?".to_vec() } else { Vec::new() }),
                guards: Some(guards),
            }
        });
    KINDS[kind]
        .2
        .map_or(Body::None, |amount| Body::Resource { message, amount })
}
/// A tile as column, row and level.
pub(super) type Spot = (u8, u8, u8);
/// An army's seven slots as creature code and count (none: the hero type's starting army).
pub(super) type Slots = Option<Vec<Option<(u16, u16)>>>;

/// A playable map before it is laid out: what the strategy draws.
#[derive(Debug, Clone)]
pub(super) struct Plan {
    pub(super) version: Version,
    pub(super) side: u8,
    pub(super) two_levels: bool,
    /// Terrain, road and river codes for as many tiles as the largest map has, row by row and
    /// level by level; the map takes the first ones.
    pub(super) tiles: Vec<(u8, u8, u8)>,
    /// The playing colours in order, and whether a human may play each.
    pub(super) players: Vec<(u8, bool)>,
    /// Each hero's tile (modulo the map) and army slots (none: its type's starting army); owners
    /// go round the players.
    pub(super) heroes: Vec<(Spot, Slots)>,
    /// Added to each hero's index times 7 (coprime to 128), so the heroes' types differ.
    pub(super) hero_types: u8,
    /// Artifact codes for each hero's backpack, by the hero's index (none or empty: the type's
    /// defaults); a hero with a backpack wears nothing (E08 T19a2a: a battle's capture) but the
    /// Spell Book that skills of its own come with.
    pub(super) backpacks: Vec<Vec<u16>>,
    /// Secondary skills of its own (skill code and level) for each hero, by the hero's index
    /// (none: its type's; E08 T19b2: Necromancy, Eagle Eye and Wisdom more often than the types
    /// have them, so that battles' winners raise creatures and learn spells).
    pub(super) skills: Vec<Option<Vec<(u8, u8)>>>,
    /// Each object's anchor (modulo the map) and object (shape and kind, [`OBJECTS`]; since E08
    /// T20b3b1 also the monsters' and grounds', [`MONSTER_OBJECTS`], [`GROUND_OBJECTS`]).
    pub(super) blockers: Vec<(Spot, usize)>,
    /// The game's difficulty (E08 T14b), written as the map's (the import's own default is
    /// normal; the tests take the map's): impossible, with no gold to start, in half the cases,
    /// so that Cartographers are refused as well as offered.
    pub(super) difficulty: u8,
}

/// Terrain, road and river codes: mostly land, some water and rock (codes 8 and 9).
pub(super) fn tile() -> impl Strategy<Value = (u8, u8, u8)> {
    (
        prop_oneof![20 => 0u8..=7, 1 => Just(8u8), 1 => Just(9u8)],
        prop_oneof![3 => Just(0u8), 1 => 1u8..=3],
        prop_oneof![5 => Just(0u8), 1 => 1u8..=4],
    )
}

/// Seven army slots, some empty, of the base game's creatures (no war machines).
pub(super) fn slots() -> impl Strategy<Value = Vec<Option<(u16, u16)>>> {
    let creatures: Vec<u16> = codes::CREATURES
        .iter()
        .map(|&(code, _)| code)
        .filter(|&code| code <= 117)
        .collect();
    let stack = (proptest::sample::select(creatures), 1u16..=40);
    vec(proptest::option::weighted(0.4, stack), 7)
}

/// An army of its own (seven slots, [`slots`]) or none (the hero type's starting army).
pub(super) fn army() -> impl Strategy<Value = Slots> {
    proptest::option::of(slots())
}

/// A hero's secondary skills of its own, in one case of two (E08 T19b2): Eagle Eye (code 11) and
/// up to three of Diplomacy (4; E08 T20b3b1: a monster's offer to join for gold), Wisdom (7) and
/// Necromancy (12), each at a level 1 to 3.
fn skills() -> impl Strategy<Value = Option<Vec<(u8, u8)>>> {
    proptest::option::weighted(
        0.5,
        (subsequence(vec![4u8, 7, 12], 0..=3), vec(1u8..=3, 4))
            .prop_map(|(skills, levels)| [11].into_iter().chain(skills).zip(levels).collect()),
    )
}

pub(super) fn plan() -> impl Strategy<Value = Plan> {
    let tiles = 2 * usize::from(MAX_SIDE) * usize::from(MAX_SIDE);
    (
        version(),
        5..=MAX_SIDE,
        any::<bool>(),
        vec(tile(), tiles),
        subsequence((0u8..8).collect::<Vec<_>>(), 2..=4),
        vec(any::<bool>(), 4),
        vec((any::<(u8, u8, u8)>(), army()), 2..=6),
        any::<u8>(),
        vec(
            (
                any::<(u8, u8, u8)>(),
                prop_oneof![
                    3 => 0..OBJECTS,
                    1 => proptest::sample::select(&CHARTS[..]),
                    2 => proptest::sample::select(&KEYS[..]),
                    1 => proptest::sample::select(&MONSTER_OBJECTS[..]),
                    1 => proptest::sample::select(&GROUND_OBJECTS[..]),
                    1 => proptest::sample::select(&GUARDED_OBJECTS[..]),
                ],
            ),
            0..=8,
        ),
        prop_oneof![Just(4), 0u8..4],
        proptest::option::weighted(0.75, ((0u8..7, 0u8..7), 0u8..3)),
        (
            vec(vec(7u16..=40, 0..=3), 0..=6),
            vec(skills(), 0..=6),
            proptest::option::weighted(
                0.75,
                (
                    (0u8..5, 0u8..5),
                    proptest::sample::select(&MONSTER_OBJECTS[..]),
                    proptest::option::of(proptest::sample::select(&GROUND_OBJECTS[..])),
                    any::<bool>(),
                ),
            ),
            proptest::option::weighted(
                0.5,
                (
                    (0u8..5, 0u8..5),
                    proptest::sample::select(&GUARDED_OBJECTS[..]),
                ),
            ),
        ),
    )
        .prop_map(
            |(
                version,
                side,
                two_levels,
                tiles,
                colours,
                humans,
                heroes,
                hero_types,
                mut blockers,
                difficulty,
                keyring,
                (backpacks, mut skills, beside, pile),
            )| {
                // In three plans of four a red tent within three tiles of the first hero, a red
                // guard whose visitable tile is two columns east of the tent, a row up, level or
                // down (E08 T15a), and a red gate whose visitable tile (blocked by its own mask,
                // as the guard's) is the one below the tent's (T15b): the tent is not entered
                // from above, so a hero leaving it downward crosses or stops on the gate with the
                // key, and one coming from below without the key visits it.
                if let Some(((dx, dy), row)) = keyring {
                    let ((hx, hy, hz), _) = heroes[0];
                    let levels = 1 + u8::from(two_levels);
                    let at = |h: u8, d: u8| (h % side + side + d - 3) % side;
                    let (tx, ty) = (at(hx, dx), at(hy, dy));
                    blockers.push(((tx, ty, hz % levels), KEYS[3]));
                    blockers.push(((at(tx, 6), at(ty, row + 2), hz % levels), KEYS[4]));
                    blockers.push(((at(tx, 4), at(ty, 4), hz % levels), KEYS[6]));
                }
                // In three plans of four (E08 T20b3b1) a monster within two tiles of the first
                // hero, on a special ground in one of those cases of two, so that random play
                // meets monsters early and fights some of them on a ground's battlefield; in one
                // of those cases of two the hero has expert Diplomacy and the monster is the
                // friendly centaurs, so that groups offer to join for gold too (ADV-MON-007's
                // third answer: their aggression of 5 to 7 above a weaker hero's charisma).
                if let Some(((dx, dy), monster, ground, diplomat)) = beside {
                    let monster = if diplomat {
                        MONSTER_OBJECTS[4]
                    } else {
                        monster
                    };
                    if diplomat {
                        if skills.is_empty() {
                            skills.push(None);
                        }
                        let own = skills[0].get_or_insert_with(Vec::new);
                        own.retain(|&(skill, _)| skill != 4);
                        own.push((4, 3));
                    }
                    let ((hx, hy, hz), _) = heroes[0];
                    let levels = 1 + u8::from(two_levels);
                    let at = |h: u8, d: u8| (h % side + side + d - 2) % side;
                    let (dx, dy) = if (dx, dy) == (2, 2) { (3, 2) } else { (dx, dy) };
                    let spot = (at(hx, dx), at(hy, dy), hz % levels);
                    blockers.push((spot, monster));
                    blockers.extend(ground.map(|ground| (spot, ground)));
                }
                // In half the plans (E08 T21a) a guarded pile whose visitable tile is within
                // three tiles of the first hero, so that random play meets its guards early.
                if let Some(((dx, dy), pile)) = pile {
                    let ((hx, hy, hz), _) = heroes[0];
                    let levels = 1 + u8::from(two_levels);
                    let at = |h: u8, d: u8| (h % side + side + d - 1) % side;
                    blockers.push(((at(hx, dx), at(hy, dy), hz % levels), pile));
                }
                Plan {
                    version,
                    side,
                    two_levels,
                    tiles,
                    players: colours.into_iter().zip(humans).collect(),
                    heroes,
                    hero_types,
                    backpacks,
                    skills,
                    blockers,
                    difficulty,
                }
            },
        )
}

/// The plan as a raw map of its version: the heroes on distinct land tiles, owners round the
/// players, the blockers that would cover a hero's tile left out, and a typed object whose kind
/// is already on its anchor.
pub(super) fn lay_out(plan: &Plan) -> RawMap {
    let v = plan.version;
    let side = plan.side;
    let levels = 1 + u8::from(plan.two_levels);
    let mut m = sample(v);
    m.header.difficulty = plan.difficulty;
    m.header.size = u32::from(side);
    m.header.two_levels = plan.two_levels;
    m.terrain.size = usize::from(side);
    m.terrain.levels = usize::from(levels);
    let count = usize::from(side) * usize::from(side) * usize::from(levels);
    m.terrain.tiles = plan.tiles[..count]
        .iter()
        .map(|&(terrain, road, river)| Tile {
            terrain,
            road,
            river,
            ..Tile::default()
        })
        .collect();
    let player = m.players[0].clone();
    m.players = Default::default();
    for &(colour, human) in &plan.players {
        m.players[usize::from(colour)] = crate::model::Player {
            human,
            computer: true,
            ..player.clone()
        };
    }
    m.templates = vec![template(34, &[(1, 0)], &[(1, 0)])];
    // The blockers' templates, and the typed objects' up to the last one the plan uses.
    let used = plan.blockers.iter().map(|&(_, o)| o + 1).max().unwrap_or(0);
    m.templates.extend((0..used.max(BLOCKERS.len())).map(|o| {
        let (blocked, visitable) = masks(o);
        let (class, subclass, _) = KINDS[o / BLOCKERS.len()];
        Template {
            subclass,
            ..template(class, blocked, visitable)
        }
    }));
    // The first hero on each tile stays; its tile becomes land if it is water or rock.
    let mut heroes: Vec<(Spot, &Slots)> = Vec::new();
    for ((x, y, z), army) in &plan.heroes {
        let at = (x % side, y % side, z % levels);
        if heroes.iter().all(|&(taken, _)| taken != at) {
            heroes.push((at, army));
        }
    }
    for &((x, y, z), _) in &heroes {
        let index = (usize::from(z) * usize::from(side) + usize::from(y)) * usize::from(side)
            + usize::from(x);
        let tile = &mut m.terrain.tiles[index];
        if tile.terrain >= 8 {
            tile.terrain = 2;
        }
    }
    // The typed objects laid out, by class and subclass (a kind's type; two kinds share the gold
    // pile's, E08 T16a; the monsters' kinds the monster's) and anchor.
    let mut typed: Vec<((u32, u32), Spot)> = Vec::new();
    for &((x, y, z), shape) in &plan.blockers {
        let (ax, ay, az) = (x % side, y % side, z % levels);
        let (blocked, visitable) = masks(shape);
        let covers = |&((hx, hy, hz), _): &(Spot, _)| {
            hz == az
                && blocked.iter().chain(visitable).any(|&(column, row)| {
                    ax.checked_sub(column) == Some(hx) && ay.checked_sub(row) == Some(hy)
                })
        };
        // At most one typed object of a kind on an anchor (E08 T14's review): what a viewer sees
        // and is told names a typed object by type and anchor, never by index (ADR-0016 point 6),
        // so a second object of the type on the anchor would share the name, and a removal could
        // not say which of the two left the view (the T13b view property failed on two campfires
        // of other shapes on one anchor). The engine removes the visited one either way.
        let kind = shape / BLOCKERS.len();
        let (class, subclass, _) = KINDS[kind];
        // Every monster has one type, whatever its creature (E08 T20b3b1).
        let subclass = if class == 54 { 0 } else { subclass };
        let twin = kind > 0 && typed.contains(&((class, subclass), (ax, ay, az)));
        if !twin && !heroes.iter().any(covers) {
            if kind > 0 {
                typed.push(((class, subclass), (ax, ay, az)));
            }
            let template = 1 + u32::try_from(shape).unwrap();
            let identifier = u32::try_from(m.objects.len()).unwrap();
            let mut o = object(ax, ay, template, body(kind, v, identifier));
            o.pos.z = az;
            m.objects.push(o);
        }
    }
    let colours: Vec<u8> = plan.players.iter().map(|&(colour, _)| colour).collect();
    for (index, &((x, y, z), army)) in (0u8..).zip(&heroes) {
        let army = army.as_ref().map(|slots| {
            slots
                .iter()
                .map(|slot| slot.map(|(creature, count)| Stack { creature, count }))
                .collect()
        });
        let hero_type = (u16::from(plan.hero_types) + 7 * u16::from(index)) % 128;
        let backpack = plan
            .backpacks
            .get(usize::from(index))
            .filter(|b| !b.is_empty());
        // A hero with skills of its own wears a Spell Book (artifact 0, in any slot: the import
        // keeps no position, ADR-0013; E08 T19b2): Eagle Eye's picks are learned only with it
        // (HER-SPL-006), and its type may have none.
        let skills = plan.skills.get(usize::from(index)).cloned().flatten();
        let artifacts = (backpack.is_some() || skills.is_some()).then(|| {
            let mut worn = vec![None; v.artifact_slots()];
            if skills.is_some() {
                worn[0] = Some(0);
            }
            Artifacts {
                worn,
                backpack: backpack.into_iter().flatten().copied().map(Some).collect(),
            }
        });
        let body = HeroBody {
            identifier: (v != Version::RoE).then_some(u32::from(index)),
            owner: colours[usize::from(index) % colours.len()],
            hero_type: u8::try_from(hero_type).unwrap(),
            experience: (v != Version::Sod).then_some(0),
            army,
            patrol: u8::MAX,
            sex: u8::MAX,
            spell: (v == Version::Ab).then_some(u8::MAX),
            artifacts,
            secondary_skills: skills,
            ..HeroBody::default()
        };
        let mut o = object(x + 1, y, HERO_TEMPLATE, Body::Hero(Box::new(body)));
        o.pos.z = z;
        m.objects.push(o);
    }
    m
}

/// The plan's map written, read back and imported, and a game started on it with `seed`.
fn start(plan: &Plan, rules: &GameRuleset, seed: u64) -> GameState {
    start_on(&lay_out(plan), rules, seed)
}

/// The map written, read back and imported, and a game started on it with `seed` at the map's
/// difficulty (the header's rating, which the import leaves to the game: E08 T14b's tests take
/// it as the game's).
pub(super) fn start_on(m: &RawMap, rules: &GameRuleset, seed: u64) -> GameState {
    let m = parse(&encode(m), &Limits::default()).unwrap();
    let mut setup = import(&m, rules).unwrap();
    setup.difficulty = m.header.difficulty;
    GameState::new(rules, &setup, seed).unwrap()
}

/// The eight directions, as column and row offsets.
const DIRECTIONS: [(i16, i16); 8] = [
    (0, -1),
    (1, -1),
    (1, 0),
    (1, 1),
    (0, 1),
    (-1, 1),
    (-1, 0),
    (-1, -1),
];

/// The tile in `direction` (modulo 8) of `pos`; off the map's top or left it wraps to a column or
/// row far off the map.
fn toward(pos: TilePos, direction: u8) -> TilePos {
    let (dx, dy) = DIRECTIONS[usize::from(direction % 8)];
    TilePos::new(
        pos.x.wrapping_add_signed(dx),
        pos.y.wrapping_add_signed(dy),
        pos.level,
    )
}

/// One step of random play, resolved against the state it is played on.
#[derive(Debug, Clone)]
pub(super) enum Step {
    /// The acting player's `n`-th option ([`options`]), modulo their number.
    Legal(u16),
    /// The acting player's `n`-th option that moves a hero onto, toward or (E08 T15b: a gate with
    /// its key, a removed object's tile) across an object's visitable tile, or (T19a) toward
    /// another player's hero (modulo their number; T14b), or as `Legal(n)` if none does: visits
    /// are rare among the reachable tiles.
    Visit(u16),
    /// The acting player ends its turn.
    EndTurn,
    /// The acting player's `n`-th legal move (modulo their number), edited; with no legal move,
    /// hero 0 with an empty path.
    Edited(u16, Edit),
    /// The acting player (`None`), or a player of the game or the one after the last, issues an
    /// arbitrary command.
    Free { player: Option<u8>, command: Free },
    /// (E08 T19b2) The acting player fights: on the map it seeks another player's hero
    /// ([`seek`]), in a battle it plays the `n`-th of its fighting commands ([`fight`]).
    Fight(u16),
}

#[derive(Debug, Clone)]
pub(super) enum Edit {
    /// Without its last step.
    DropLast,
    /// With one more step, in this direction.
    Extend(u8),
    /// With one more step onto a tile the actor has not explored around the last, the first
    /// from this direction on (or in this direction if there is none).
    Beyond(u8),
    /// Its `n`-th step (modulo the path's length) replaced by the tile in this direction of the
    /// tile before it.
    Turn(u16, u8),
    /// For the hero with this id, modulo the heroes and one more.
    OtherHero(u8),
    /// Issued by the player with this id, modulo the players and one more.
    OtherPlayer(u8),
}

#[derive(Debug, Clone)]
pub(super) enum Free {
    EndTurn,
    /// The hero walks from its tile (or the map's corner) in these directions; the jump puts a
    /// tile anywhere at a step's index.
    Move {
        hero: Who,
        directions: Vec<u8>,
        jump: Option<(usize, (u16, u16, u8))>,
    },
    /// The hero walks from its tile straight (diagonally first) to this tile of its level:
    /// across explored and hidden tiles alike.
    Line {
        hero: Who,
        to: (u16, u16),
    },
    /// An answer to a pending decision (E08 T14b).
    Answer(Answer),
}

/// A hero of an arbitrary command: the `n`-th of the actor's own (modulo their number; hero 0 if
/// it has none), or the hero with id `n` modulo the heroes and one more.
#[derive(Debug, Clone, Copy)]
pub(super) struct Who {
    own: bool,
    n: u8,
}

impl Who {
    fn hero(self, state: &GameState, actor: PlayerId) -> HeroId {
        let heroes = state.adventure().heroes();
        if self.own {
            let own: Vec<HeroId> = heroes
                .iter()
                .filter(|h| h.owner == actor)
                .map(|h| h.id)
                .collect();
            own.get(usize::from(self.n) % own.len().max(1))
                .copied()
                .unwrap_or(HeroId::new(0))
        } else {
            let all = u16::try_from(heroes.len()).unwrap() + 1;
            HeroId::new(u16::from(self.n) % all)
        }
    }
}

fn who() -> impl Strategy<Value = Who> {
    (any::<bool>(), any::<u8>()).prop_map(|(own, n)| Who { own, n })
}

pub(super) fn step() -> impl Strategy<Value = Step> {
    let edit = prop_oneof![
        Just(Edit::DropLast),
        any::<u8>().prop_map(Edit::Extend),
        any::<u8>().prop_map(Edit::Beyond),
        any::<(u16, u8)>().prop_map(|(n, d)| Edit::Turn(n, d)),
        any::<u8>().prop_map(Edit::OtherHero),
        any::<u8>().prop_map(Edit::OtherPlayer),
    ];
    let free = prop_oneof![
        Just(Free::EndTurn),
        (
            who(),
            vec(any::<u8>(), 0..10),
            proptest::option::of((0usize..10, (0u16..20, 0u16..20, 0u8..3))),
        )
            .prop_map(|(hero, directions, jump)| Free::Move {
                hero,
                directions,
                jump
            }),
        (who(), (0u16..MAX_SIDE.into(), 0u16..MAX_SIDE.into()))
            .prop_map(|(hero, to)| Free::Line { hero, to }),
        any::<bool>().prop_map(|yes| Free::Answer(if yes {
            Answer::Accept
        } else {
            Answer::Decline
        })),
    ];
    prop_oneof![
        6 => any::<u16>().prop_map(Step::Legal),
        2 => any::<u16>().prop_map(Step::Visit),
        1 => Just(Step::EndTurn),
        2 => (any::<u16>(), edit).prop_map(|(n, edit)| Step::Edited(n, edit)),
        2 => (proptest::option::weighted(0.33, any::<u8>()), free)
            .prop_map(|(player, command)| Step::Free { player, command }),
    ]
}

/// Steps of random play (E08 T19b2): single steps and runs of [`Step::Fight`], long enough to
/// bring two players' heroes together and play their battle to its end.
pub(super) fn steps() -> impl Strategy<Value = Vec<Step>> {
    let run = vec(any::<u16>(), 16..96).prop_map(|ns| ns.into_iter().map(Step::Fight).collect());
    vec(
        prop_oneof![12 => step().prop_map(|step| vec![step]), 1 => run],
        0..40,
    )
    .prop_map(|runs: Vec<Vec<Step>>| runs.into_iter().flatten().collect())
}

/// On the map (E08 T19b2): the acting player's open offer declined; else the `n`-th of its
/// options that end nearest to another player's hero on the hero's level, nearer than the hero
/// stands (an attack, on the other hero's tile, is nearest of all; the heroes seek each other as
/// the fork benchmark's do: random play seldom meets); `EndTurn` when none gets nearer.
fn seek(state: &GameState, legal: &[GameCommand], n: u16) -> GameCommand {
    let actor = acting(state);
    if state.decision(actor).is_some() {
        return GameCommand::Answer(Answer::Decline);
    }
    let heroes = state.adventure().heroes();
    let distance = |pos: TilePos| {
        heroes
            .iter()
            .filter(|h| h.owner != actor && h.pos.level == pos.level)
            .map(|h| h.pos.x.abs_diff(pos.x).max(h.pos.y.abs_diff(pos.y)))
            .min()
    };
    let mut best: Vec<&GameCommand> = Vec::new();
    let mut nearest = u16::MAX;
    for command in legal {
        let Some((hero, path)) = as_move(command) else {
            continue;
        };
        let here = state.adventure().hero(hero).and_then(|h| distance(h.pos));
        let there = path.last().and_then(|&to| distance(to));
        if let (Some(here), Some(there)) = (here, there)
            && there < here
            && there <= nearest
        {
            if there < nearest {
                best.clear();
                nearest = there;
            }
            best.push(command);
        }
    }
    best.get(usize::from(n) % best.len().max(1))
        .map_or(GameCommand::EndTurn, |&command| command.clone())
}

/// In a battle (E08 T19b2): the actor's `n`-th fighting command: in three steps of four its
/// hero's cast while it may cast (the spell, its target and a second target by `n`: the game's
/// legal commands list no casts, a separate action head, ADR-0014 point 9; Eagle Eye picks only
/// what the other side's hero casts, BAT-MAG-069), else a strike (an attack or a shot) if its
/// active stack has one, else a command that stays in the battle; in one step of 128 it leaves
/// the battle (a retreat or a surrender, if its side may), so most battles end by a victory. The
/// system actor (E08 T20b3b1) fights a monster's side alike: it has no hero to cast or leave.
fn fight(state: &GameState, rules: &GameRuleset, legal: &[GameCommand], n: u16) -> GameCommand {
    let (battle, side) = battle_of(state, actor(state)).unwrap();
    let (field, ruleset) = (battle.state(), rules.battle());
    let n = usize::from(n);
    let spells = castable_spells(field, ruleset, side);
    let count = spells.iter().count();
    if n % 4 != 3 && count > 0 {
        let spell = spells.iter().nth((n / 4) % count).unwrap();
        let mut targets = CastTargets::new();
        cast_targets(field, ruleset, side, spell, &mut targets);
        let target = targets
            .get((n / 16) % targets.len().max(1))
            .copied()
            .unwrap_or_default();
        second_targets(field, ruleset, side, (spell, target), &mut targets);
        let second = targets
            .get((n / 256) % targets.len().max(1))
            .copied()
            .unwrap_or_default();
        return GameCommand::Battle(Command::Cast {
            spell,
            target,
            second,
        });
    }
    let (staying, leaving): (Vec<&GameCommand>, Vec<&GameCommand>) = legal
        .iter()
        .partition(|c| matches!(c, GameCommand::Battle(c) if !c.leaves_battle()));
    if n % 128 == 127 && !leaving.is_empty() {
        return leaving[(n / 128) % leaving.len()].clone();
    }
    let strikes: Vec<&GameCommand> = staying
        .iter()
        .copied()
        .filter(|c| {
            matches!(
                c,
                GameCommand::Battle(Command::Attack { .. } | Command::Shoot { .. })
            )
        })
        .collect();
    let pick = if strikes.is_empty() { staying } else { strikes };
    pick.get(n % pick.len().max(1))
        .map_or_else(|| legal[n % legal.len()].clone(), |&c| c.clone())
}

fn move_hero(hero: HeroId, path: Vec<TilePos>) -> GameCommand {
    GameCommand::Adventure(AdventureCommand::MoveHero { hero, path })
}

/// The hero and path of a move.
fn as_move(command: &GameCommand) -> Option<(HeroId, &[TilePos])> {
    match command {
        GameCommand::Adventure(AdventureCommand::MoveHero { hero, path }) => Some((*hero, path)),
        GameCommand::EndTurn | GameCommand::Answer(_) | GameCommand::Battle(_) => None,
    }
}

/// Whether another player's hero than `player`'s stands on `pos` in `state`: a step toward it is
/// a visit, a battle (E08 T19a).
fn foe_at(state: &GameState, player: PlayerId, pos: TilePos) -> bool {
    let heroes = state.adventure().heroes();
    heroes.iter().any(|h| h.pos == pos && h.owner != player)
}

/// The heroes of the battles that ended in `events`, with their results.
fn ended(events: &[GameEvent]) -> Vec<([Option<HeroId>; 2], BattleResult)> {
    events
        .iter()
        .filter_map(|e| match *e {
            GameEvent::BattleEnded { heroes, result, .. } => Some((heroes, result)),
            _ => None,
        })
        .collect()
}

/// Where a move that `apply` accepted on `before` left its hero, from `start`, and whether it
/// visited an object or (E08 T19a) another player's hero from the tile before its last: the hero
/// steps onto every tile of the path but a neighbour visit's, the last (ADV-OBJ-003), which is a
/// visitable tile or the other hero's; the move emitted a step event per tile it stepped onto,
/// then at most one adventure event of the visit (and the removals of a battle that ended at its
/// start).
fn landing(
    before: &GameState,
    start: TilePos,
    path: &[TilePos],
    events: &[GameEvent],
) -> Result<(TilePos, bool), TestCaseError> {
    let kinds: Vec<AdventureEventKind> = events
        .iter()
        .filter_map(|e| match e {
            GameEvent::Adventure(AdventureEvent { kind, .. })
                if !matches!(kind, AdventureEventKind::HeroRemoved { .. }) =>
            {
                Some(*kind)
            }
            _ => None,
        })
        .collect();
    let steps = kinds
        .iter()
        .take_while(|k| matches!(k, AdventureEventKind::HeroStepped { .. }))
        .count();
    prop_assert!(kinds.len() <= steps + 1, "{:?}", kinds);
    let stays = steps + 1 == path.len();
    prop_assert!(
        steps == path.len() || stays,
        "{} steps of {:?}",
        steps,
        path
    );
    if stays {
        let to = *path.last().unwrap();
        let visitable = before
            .adventure()
            .map()
            .tile(to)
            .is_some_and(|tile| tile.visitable);
        let mover = before.adventure().heroes().iter().find(|h| h.pos == start);
        let foe = mover.is_some_and(|h| foe_at(before, h.owner, to));
        prop_assert!(visitable || foe);
    }
    let walked = &path[..steps];
    Ok((walked.last().copied().unwrap_or(start), stays))
}

/// The player who acts: the one whose turn is running (sequential play: one at a time), or, while
/// its battle's other side is to act, that side's owner (E08 T19a); while the other side is a
/// monster's, still the player whose battle it is ([`actor`] is then the system).
pub(super) fn acting(state: &GameState) -> PlayerId {
    let player = state.turn_group()[0];
    state
        .battle(player)
        .and_then(OpenBattle::owner_to_act)
        .unwrap_or(player)
}

/// Who acts (E08 T20b3b1): the system actor while a battle's side to act is a monster's (no
/// player owns it: `GameState::unowned_battle`), else the acting player ([`acting`]).
pub(super) fn actor(state: &GameState) -> Actor {
    if state.unowned_battle().is_some() {
        Actor::System
    } else {
        Actor::Player(acting(state))
    }
}

/// The player a command of `actor` is played for: the player itself, or for the system actor the
/// player whose battle it plays ([`acting`]).
pub(super) fn player_of(state: &GameState, actor: Actor) -> PlayerId {
    match actor {
        Actor::Player(player) => player,
        Actor::System => acting(state),
    }
}

/// The battle `actor` plays and its side there: a player's open battle and the side it owns, or
/// the system actor's (the first whose side to act no player owns) and that side.
fn battle_of(state: &GameState, actor: Actor) -> Option<(&OpenBattle, BattleSide)> {
    match actor {
        Actor::Player(player) => {
            let battle = state.battle(player)?;
            Some((battle, battle.side_of(player)?))
        }
        Actor::System => {
            let battle = state.unowned_battle()?;
            Some((battle, battle.state().side_to_act()?))
        }
    }
}

/// The commands the driver picks from: the actor's legal commands, then each of its legal moves
/// with the query's path, by hero and row by row (none for the system actor). The game does not
/// list moves one by one (a hero with many points reaches thousands of tiles); the small maps
/// here allow it.
pub(super) fn options(state: &GameState, rules: &GameRuleset) -> Vec<GameCommand> {
    let actor = actor(state);
    let mut options = legal_commands(state, rules, actor);
    for hero in state.adventure().heroes() {
        if let Some(moves) = legal_moves(state, rules, actor, hero.id) {
            for (to, _) in moves.tiles() {
                options.push(move_hero(hero.id, moves.path(to).unwrap()));
            }
        }
    }
    options
}

/// A step's actor and command, given the actor's [`options`]: the acting player's, or (E08
/// T20b3b1) the system actor's in a monster's battle, whose steps but a fight's and a free
/// command's of a player are refused or its legal commands.
pub(super) fn resolve(
    state: &GameState,
    rules: &GameRuleset,
    legal: &[GameCommand],
    step: &Step,
) -> (Actor, GameCommand) {
    let who = actor(state);
    let actor = acting(state);
    let players = u8::try_from(state.players()).unwrap();
    let heroes = u16::try_from(state.adventure().heroes().len()).unwrap();
    let visits = |command: &&GameCommand| {
        as_move(command).is_some_and(|(_, path)| {
            path.iter().any(|&to| {
                state
                    .adventure()
                    .map()
                    .tile(to)
                    .is_some_and(|tile| tile.visitable)
                    || foe_at(state, actor, to)
            })
        })
    };
    match step {
        Step::Legal(n) => (who, legal[usize::from(*n) % legal.len()].clone()),
        Step::Visit(n) => {
            let visits: Vec<&GameCommand> = legal.iter().filter(visits).collect();
            let command = match visits.len() {
                0 => &legal[usize::from(*n) % legal.len()],
                len => visits[usize::from(*n) % len],
            };
            (who, command.clone())
        }
        Step::EndTurn => (who, GameCommand::EndTurn),
        Step::Fight(n) if battle_of(state, who).is_some() => (who, fight(state, rules, legal, *n)),
        Step::Fight(n) => (who, seek(state, legal, *n)),
        Step::Edited(n, edit) => {
            let moves: Vec<(HeroId, &[TilePos])> = legal.iter().filter_map(as_move).collect();
            let Some(&(mut hero, path)) = moves.get(usize::from(*n) % moves.len().max(1)) else {
                return (who, move_hero(HeroId::new(0), Vec::new()));
            };
            let origin = state.adventure().hero(hero).unwrap().pos;
            let mut path = path.to_vec();
            let mut who = who;
            match edit {
                Edit::DropLast => {
                    path.pop();
                }
                Edit::Extend(direction) => path.push(toward(*path.last().unwrap(), *direction)),
                Edit::Beyond(direction) => {
                    let last = *path.last().unwrap();
                    let adventure = state.adventure();
                    let hidden = (0..8)
                        .map(|i| toward(last, direction.wrapping_add(i)))
                        .find(|&pos| {
                            adventure.map().tile(pos).is_some()
                                && !adventure.is_explored(actor, pos)
                        });
                    path.push(hidden.unwrap_or_else(|| toward(last, *direction)));
                }
                Edit::Turn(n, direction) => {
                    let i = usize::from(*n) % path.len();
                    path[i] = toward(if i == 0 { origin } else { path[i - 1] }, *direction);
                }
                Edit::OtherHero(id) => hero = HeroId::new(u16::from(*id) % (heroes + 1)),
                Edit::OtherPlayer(id) => who = Actor::Player(PlayerId::new(id % (players + 1))),
            }
            (who, move_hero(hero, path))
        }
        Step::Free { player, command } => {
            let who = player.map_or(who, |player| {
                Actor::Player(PlayerId::new(player % (players + 1)))
            });
            let actor = player_of(state, who);
            match command {
                Free::EndTurn => (who, GameCommand::EndTurn),
                Free::Answer(answer) => (who, GameCommand::Answer(*answer)),
                Free::Move {
                    hero,
                    directions,
                    jump,
                } => {
                    let hero = hero.hero(state, actor);
                    let mut at = state
                        .adventure()
                        .hero(hero)
                        .map_or(TilePos::new(0, 0, 0), |h| h.pos);
                    let mut path = Vec::new();
                    for (i, &direction) in directions.iter().enumerate() {
                        at = match jump {
                            Some((j, (x, y, z))) if *j == i => TilePos::new(*x, *y, *z),
                            _ => toward(at, direction),
                        };
                        path.push(at);
                    }
                    (who, move_hero(hero, path))
                }
                Free::Line { hero, to } => {
                    let hero = hero.hero(state, actor);
                    let mut at = state
                        .adventure()
                        .hero(hero)
                        .map_or(TilePos::new(0, 0, 0), |h| h.pos);
                    let mut path = Vec::new();
                    // A step's offset on one axis: -1, 0 or 1.
                    let offset = |from: u16, to: u16| -i16::from(from.cmp(&to) as i8);
                    while (at.x, at.y) != *to {
                        at = TilePos::new(
                            at.x.wrapping_add_signed(offset(at.x, to.0)),
                            at.y.wrapping_add_signed(offset(at.y, to.1)),
                            at.level,
                        );
                        path.push(at);
                    }
                    (who, move_hero(hero, path))
                }
            }
        }
    }
}

/// Plays `step` on `state`: resolved, then applied; the events go to `sink`.
pub(super) fn advance<S: Sink>(
    state: &mut GameState,
    rules: &GameRuleset,
    step: &Step,
    sink: &mut S,
) -> Result<(), GameCommandError> {
    let options = options(state, rules);
    let (actor, command) = resolve(state, rules, &options, step);
    apply(state, rules, actor, command, sink)
}

/// Plays a game from the plan's map: the final state, every event and the hash after each step.
fn play(
    plan: &Plan,
    rules: &GameRuleset,
    seed: u64,
    steps: &[Step],
) -> (GameState, Vec<GameEvent>, Vec<u64>) {
    let mut state = start(plan, rules, seed);
    let mut events = Vec::new();
    let mut hashes = Vec::new();
    for step in steps {
        let _ = advance(&mut state, rules, step, &mut events);
        hashes.push(state.state_hash());
    }
    (state, events, hashes)
}

fn fail(message: String) -> TestCaseError {
    TestCaseError::fail(message)
}

/// The query for each of the heroes, by id, as `state` has them.
fn reaches(state: &GameState, rules: &GameRuleset) -> Vec<(HeroId, Reach)> {
    state
        .adventure()
        .heroes()
        .iter()
        .map(|hero| {
            let reach =
                state
                    .adventure()
                    .reach(rules.core(), rules.adventure(), state.heroes(), hero.id);
            (hero.id, reach.unwrap())
        })
        .collect()
}

fn reach_of(reaches: &[(HeroId, Reach)], hero: HeroId) -> &Reach {
    &reaches.iter().find(|(id, _)| *id == hero).unwrap().1
}

/// The answers to a pending decision, as `legal_commands` lists them (GAM-DEC-001).
const ANSWERS: [GameCommand; 2] = [
    GameCommand::Answer(Answer::Accept),
    GameCommand::Answer(Answer::Decline),
];

/// Legality consistency, before a step: only the actor has legal commands, `EndTurn`, or with an
/// open decision (only ever the acting player's in sequential play) its answers and nothing else
/// (GAM-DEC-001, E08 T14b), `Decline` alone if the offer no longer stands (T15a; never in
/// sequential play), or in a battle (T19a) the battle's commands while its side is to act, each
/// of which applies; while a monster's side is to act (E08 T20b3b1) the actor is the system, its
/// commands that side's, no player's any; the system has none otherwise and never a move; the
/// actor's legal moves are the query of each of its own heroes while no decision is open, and
/// nobody has any for another player's; every listed answer applies and closes the decision;
/// every option's move applies and leaves the hero on its last tile (on the tile before for a
/// visit from a neighbour) with the points the query reports there, one event per step.
fn check_legal(
    state: &GameState,
    rules: &GameRuleset,
    legal: &[GameCommand],
    reaches: &[(HeroId, Reach)],
) -> Result<(), TestCaseError> {
    let who = actor(state);
    let actor = acting(state);
    let fighting = battle_of(state, who).is_some();
    let decided = state.decision(actor).is_some();
    let system = legal_commands(state, rules, Actor::System);
    prop_assert_eq!(system.is_empty(), who != Actor::System);
    prop_assert!(system.iter().all(|c| matches!(c, GameCommand::Battle(_))));
    for (hero, _) in reaches {
        prop_assert_eq!(legal_moves(state, rules, Actor::System, *hero), None);
    }
    let answers = match state.decision(actor) {
        Some(offer) if !state.adventure().offer_stands(offer) => &ANSWERS[1..],
        _ => &ANSWERS[..],
    };
    for player in (0..=state.players()).map(|p| PlayerId::new(u8::try_from(p).unwrap())) {
        let listed = legal_commands(state, rules, Actor::Player(player));
        let me = who == Actor::Player(player);
        prop_assert!(player == actor || state.decision(player).is_none());
        if me && fighting {
            prop_assert!(!listed.is_empty());
            prop_assert!(listed.iter().all(|c| matches!(c, GameCommand::Battle(_))));
        } else {
            let expected = if !me {
                Vec::new()
            } else if decided {
                answers.to_vec()
            } else {
                vec![GameCommand::EndTurn]
            };
            prop_assert_eq!(listed, expected);
        }
        for (hero, reach) in reaches {
            let moves = legal_moves(state, rules, Actor::Player(player), *hero);
            let own = me
                && !decided
                && !fighting
                && state.adventure().hero(*hero).map(|h| h.owner) == Some(actor);
            prop_assert_eq!(moves.as_ref(), own.then_some(reach));
        }
    }
    if fighting {
        prop_assert_eq!(legal, &legal_commands(state, rules, who)[..]);
        for command in legal {
            let mut probe = state.clone();
            let result = apply(&mut probe, rules, who, command.clone(), &mut Vec::new());
            prop_assert!(result.is_ok(), "legal {:?} rejected: {:?}", command, result);
        }
        return Ok(());
    }
    if decided {
        prop_assert_eq!(legal, answers);
        for answer in answers {
            let mut probe = state.clone();
            let result = apply(
                &mut probe,
                rules,
                Actor::Player(actor),
                answer.clone(),
                &mut Vec::new(),
            );
            prop_assert!(result.is_ok(), "legal {:?} rejected: {:?}", answer, result);
            // E08 T20b3b1: a declined offer to join goes on to the monster's offer to flee
            // (ADV-MON-007), and the answer to an object's offer on a guarded tile to the
            // guard's visit, which may offer too (ADV-MON-006); every other answer closes the
            // decision.
            let open = state.decision(actor).unwrap();
            let next = probe.decision(actor);
            let monster = |kind: &OfferKind| {
                matches!(
                    kind,
                    OfferKind::Join | OfferKind::JoinForGold { .. } | OfferKind::Pursue
                )
            };
            let flees = *answer == GameCommand::Answer(Answer::Decline)
                && matches!(open.kind, OfferKind::Join | OfferKind::JoinForGold { .. })
                && next.is_some_and(|n| {
                    (n.object_type, n.anchor, &n.kind)
                        == (open.object_type, open.anchor, &OfferKind::Pursue)
                });
            let guard = state.adventure().hero(open.hero).and_then(|hero| {
                let index = state.adventure().guard_of(rules.adventure(), hero.pos)?;
                let o = &state.adventure().map().objects()[index];
                Some((o.object_type, o.anchor))
            });
            let guarded = !monster(&open.kind)
                && next.is_some_and(|n| {
                    monster(&n.kind) && Some((Some(n.object_type), n.anchor)) == guard
                });
            prop_assert!(
                next.is_none() || flees || guarded,
                "{:?} after {:?}: {:?}",
                open,
                answer,
                next
            );
        }
        return Ok(());
    }
    prop_assert_eq!(legal.first(), Some(&GameCommand::EndTurn));
    for command in &legal[1..] {
        let (hero, path) = as_move(command).ok_or_else(|| fail(format!("{command:?}")))?;
        prop_assert_eq!(state.adventure().hero(hero).map(|h| h.owner), Some(actor));
        let to = *path.last().unwrap();
        let mut probe = state.clone();
        let mut events = Vec::new();
        let result = apply(
            &mut probe,
            rules,
            Actor::Player(actor),
            command.clone(),
            &mut events,
        );
        prop_assert!(result.is_ok(), "legal {:?} rejected: {:?}", command, result);
        let start = state.adventure().hero(hero).unwrap().pos;
        let (at, visited) = landing(state, start, path, &events)?;
        prop_assert_eq!(
            at,
            if visited {
                path.len().checked_sub(2).map_or(start, |i| path[i])
            } else {
                to
            }
        );
        let moved = probe.adventure().hero(hero).unwrap();
        let points = reach_of(reaches, hero).points_left(to);
        prop_assert_eq!((moved.pos, Some(moved.movement)), (at, points));
    }
    Ok(())
}

/// Legality consistency for a command `apply` accepted although it is not an option: a move of
/// the actor's own hero along another path the hero can pay. A one-step move's tile is among its
/// legal moves, with at least the points the move left (every tile one step away is). Or (E08
/// T19b2) a cast of the actor's hero in its battle (the game lists no casts, a separate action
/// head): of a spell the battle's castable spells held, at one of its targets. The system actor
/// (E08 T20b3b1) plays only what it is listed: a monster's side has no hero to cast.
fn check_unlisted(
    rules: &GameRuleset,
    (before, after): (&GameState, &GameState),
    actor: Actor,
    command: &GameCommand,
    reaches: &[(HeroId, Reach)],
) -> Result<(), TestCaseError> {
    let Actor::Player(actor) = actor else {
        return Err(fail(format!("the system's unlisted {command:?} accepted")));
    };
    if let GameCommand::Battle(Command::Cast { spell, target, .. }) = *command {
        let battle = before.battle(actor).unwrap();
        let side = battle.side_of(actor).unwrap();
        let (field, ruleset) = (battle.state(), rules.battle());
        prop_assert!(castable_spells(field, ruleset, side).contains(spell));
        let mut targets = CastTargets::new();
        cast_targets(field, ruleset, side, spell, &mut targets);
        prop_assert!(targets.iter().any(|&t| t == target), "{:?}", command);
        return Ok(());
    }
    let (hero, path) =
        as_move(command).ok_or_else(|| fail(format!("unlisted {command:?} accepted")))?;
    prop_assert_eq!(before.adventure().hero(hero).map(|h| h.owner), Some(actor));
    if let [to] = path {
        let left = after.adventure().hero(hero).unwrap().movement;
        let best = reach_of(reaches, hero).points_left(*to);
        prop_assert!(
            best.is_some_and(|best| best >= left),
            "{:?} left {}, the query {:?}",
            command,
            left,
            best
        );
    }
    Ok(())
}

/// Conservation (`docs/05-testing.md`): increments 1 and 2 have no source or sink of creatures
/// (no recruitment; E08 T10b's audit), so an accepted command leaves every hero's army as it was,
/// the actor's own included, but (E08 T19a) by a battle's end: only the battle's heroes change,
/// each leaving the map or keeping an army of its own stacks. A treasury changes only by a visit's
/// gains (E08 T13b, ADR-0017's consequences): each player's by exactly the gains it was told,
/// which come with an object leaving the map, and (E08 T16a's audit) are exactly the contents
/// drawn for the objects that left, in their order (ADV-PIK-001 to -003; a passed guard has
/// none); since T14b by an accepted offer's price, which its owner is told it paid, exactly the
/// price of the decision it answered; and since T19a by a surrender's price, which the
/// surrendering side's owner pays and the other side's owner gains, as much as was paid. The
/// heroes' details (E08 T19a2a) change only by a battle's end. Since E08 T20b3b1 a wandering
/// monster is a source of creatures and resources: a group that joins goes into its visitor's
/// army (into the stack of its creature, else the first free slot), all of it, told to the
/// visitor's owner as the creatures joining, and its treasure is the owner's gains as it leaves
/// the map; a monster that flees leaves without its treasure.
fn check_conservation(
    rules: &GameRuleset,
    before: &GameState,
    after: &GameState,
    events: &[GameEvent],
) -> Result<(), TestCaseError> {
    let battles = ended(events);
    if !battles.is_empty() {
        return check_battle_end(rules, before, after, events, &battles);
    }
    prop_assert_eq!(before.heroes(), after.heroes());
    let (map, adventure) = (before.adventure().map(), after.adventure());
    let joined: Vec<(HeroId, CreatureTypeId, u32)> = events
        .iter()
        .filter_map(|e| match *e {
            GameEvent::Adventure(AdventureEvent {
                kind:
                    AdventureEventKind::CreaturesJoined {
                        hero,
                        creature,
                        count,
                    },
                ..
            }) => Some((hero, creature, count)),
            _ => None,
        })
        .collect();
    prop_assert!(joined.len() <= 1, "{:?}", joined);
    let mut contents = Vec::new();
    for event in events {
        if let GameEvent::Adventure(AdventureEvent {
            kind:
                AdventureEventKind::ObjectRemoved {
                    object_type,
                    anchor,
                },
            ..
        }) = *event
        {
            let left = map.objects().iter().enumerate().filter(|&(i, o)| {
                (o.object_type, o.anchor) == (Some(object_type), anchor)
                    && !before.adventure().is_removed(i)
                    && adventure.is_removed(i)
            });
            for (i, o) in left {
                contents.extend(o.contents.iter().copied());
                // A monster that joins (all of it, of its creature) gives its treasure; one that
                // flees gives nothing.
                if let Some(monster) = &o.monster
                    && let Some(&(_, creature, count)) = joined.first()
                {
                    prop_assert_eq!(creature, monster.creature);
                    prop_assert_eq!(Some(count), before.adventure().monster_count(i));
                    contents.extend(monster.treasure.iter().copied());
                }
            }
        }
    }
    let gains: Vec<(ResourceId, u32)> = events
        .iter()
        .filter_map(|e| match *e {
            GameEvent::ResourceGained {
                resource, amount, ..
            } => Some((resource, amount)),
            _ => None,
        })
        .collect();
    prop_assert_eq!(&gains, &contents);
    let removed = events.iter().any(|e| {
        matches!(
            e,
            GameEvent::Adventure(AdventureEvent {
                kind: AdventureEventKind::ObjectRemoved { .. },
                ..
            })
        )
    });
    let accepted = events.iter().find_map(|e| match *e {
        GameEvent::OfferAnswered {
            player,
            accepted: true,
            ..
        } => Some(player),
        _ => None,
    });
    for player in (0..before.players()).map(|p| PlayerId::new(u8::try_from(p).unwrap())) {
        let mut treasury = before.treasury(player).unwrap().to_vec();
        let mut paid = Vec::new();
        for event in events {
            match *event {
                GameEvent::ResourceGained {
                    player: owner,
                    resource,
                    amount,
                } => {
                    prop_assert!(removed, "{:?} without a visit", event);
                    if owner == player {
                        treasury[usize::from(resource.raw())] += amount;
                    }
                }
                GameEvent::ResourcePaid {
                    player: owner,
                    resource,
                    amount,
                } => {
                    prop_assert_eq!(accepted, Some(owner), "{:?} without an offer", event);
                    if owner == player {
                        treasury[usize::from(resource.raw())] -= amount;
                        paid.push((resource, amount));
                    }
                }
                _ => {}
            }
        }
        prop_assert_eq!(after.treasury(player).unwrap(), &treasury[..]);
        if accepted == Some(player) {
            prop_assert_eq!(&paid[..], before.decision(player).unwrap().price());
        }
    }
    let mut expected = armies(before);
    for &(hero, creature, count) in &joined {
        let (_, army) = expected.iter_mut().find(|(id, _)| *id == hero).unwrap();
        if let Some(stack) = army.iter_mut().find(|s| s.creature == creature) {
            stack.count += count;
        } else {
            let slot = (0..7).find(|&slot| army.iter().all(|s| s.slot != slot));
            let slot = slot.ok_or_else(|| fail(format!("{hero:?} joined without room")))?;
            let at = army
                .iter()
                .position(|s| s.slot > slot)
                .unwrap_or(army.len());
            army.insert(
                at,
                h3_contracts::ArmyStack {
                    slot,
                    creature,
                    count,
                },
            );
        }
    }
    prop_assert_eq!(expected, armies(after));
    Ok(())
}

/// Every hero's army, by id.
fn armies(state: &GameState) -> Vec<(HeroId, Vec<h3_contracts::ArmyStack>)> {
    let heroes = state.adventure().heroes().iter();
    heroes.map(|h| (h.id, h.army.clone())).collect()
}

/// Conservation for a command that ended a battle (E08 T19a; GAM-BAT-002, -003, -005): only one
/// ends; heroes other than the battle's keep their armies; a battle hero that did not win leaves
/// the map, told as its removal, and one that stays keeps a nonempty army whose stacks fit its
/// slots; a treasury changes only by a surrender's price: the surrendering side's owner pays, the
/// other side's owner gains what was paid. The heroes' details (E08 T19a2a; GAM-BAT-002, -004)
/// change exactly as the changes told to their owners, only the battle's heroes', only the
/// winner that stays gains experience, and artifacts only move between the two or leave with a
/// destroyed war machine. A monster's battle (E08 T20b3b1; ADV-MON-010): beaten, the monster
/// leaves the map, told as its removal, with its treasure for the visitor's owner (a draw gives
/// none); else it stays with at most the creatures it had; no surrender. A guarded pile's (E08
/// T21a; ADV-PIK-004): won, the pile is picked up; else its guards keep their survivors.
fn check_battle_end(
    rules: &GameRuleset,
    before: &GameState,
    after: &GameState,
    events: &[GameEvent],
    battles: &[([Option<HeroId>; 2], BattleResult)],
) -> Result<(), TestCaseError> {
    prop_assert_eq!(battles.len(), 1);
    let (heroes, result) = battles[0];
    let fought = |id: HeroId| heroes.contains(&Some(id));
    let (was, is) = (armies(before), armies(after));
    for (id, army) in &was {
        let now = is.iter().find(|(other, _)| other == id).map(|(_, a)| a);
        if !fought(*id) {
            prop_assert_eq!(now, Some(army));
        }
    }
    let removed: Vec<HeroId> = events
        .iter()
        .filter_map(|e| match *e {
            GameEvent::Adventure(AdventureEvent {
                kind: AdventureEventKind::HeroRemoved { hero, .. },
                ..
            }) => Some(hero),
            _ => None,
        })
        .collect();
    let monster = heroes[BattleSide::Defender.index()].is_none();
    if monster {
        check_unowned_battle_end(before, after, events, heroes, result)?;
    }
    for (side, hero) in heroes.iter().enumerate() {
        let Some(hero) = *hero else {
            continue;
        };
        let won = result.winner().map(|w| w.index()) == Some(side);
        let stays = after.adventure().hero(hero);
        prop_assert!(won || stays.is_none(), "{:?} did not win but stays", hero);
        prop_assert_eq!(stays.is_none(), removed.contains(&hero));
        if let Some(stays) = stays {
            prop_assert!(!stays.army.is_empty());
            prop_assert!(stays.army.iter().all(|s| s.count > 0 && s.slot < 7));
        }
    }
    let owner = |id: Option<HeroId>| before.adventure().hero(id.unwrap()).unwrap().owner;
    let mut paid = None;
    for event in events {
        match *event {
            GameEvent::ResourcePaid { player, amount, .. } => {
                let BattleResult::Surrender(side) = result else {
                    return Err(fail(format!("{event:?} without a surrender")));
                };
                prop_assert_eq!(player, owner(heroes[side.index()]));
                paid = Some(amount);
            }
            GameEvent::ResourceGained { .. } if monster => {}
            GameEvent::ResourceGained { player, amount, .. } => {
                let BattleResult::Surrender(side) = result else {
                    return Err(fail(format!("{event:?} without a surrender")));
                };
                prop_assert_eq!(player, owner(heroes[side.opponent().index()]));
                prop_assert_eq!(Some(amount), paid);
            }
            _ => {}
        }
    }
    for player in (0..before.players()).map(|p| PlayerId::new(u8::try_from(p).unwrap())) {
        let (b, a) = (
            before.treasury(player).unwrap(),
            after.treasury(player).unwrap(),
        );
        if paid.is_none() && !monster {
            prop_assert_eq!(b, a);
        }
    }
    check_heroes_after_battle(rules, (before, after), events, heroes)
}

/// The unowned side's part of [`check_battle_end`]: the object fought is the one visited at the
/// battle's tile, a wandering monster (E08 T20b3b1; ADV-MON-010) or a guarded pile (E08 T21a;
/// ADV-PIK-004). A monster: a victory of the visitor's side removes it, told as its removal, and
/// gives the visitor's owner exactly its treasure; a draw removes it without one; else it stays
/// with between one and the creatures it had. A pile: a victory picks it up, told as its removal,
/// and gives the visitor's owner exactly its contents; else it stays, its guards' survivors by
/// slot each of its stack's creature and at most its count (none after a draw). No treasury
/// changes otherwise; no side surrenders to them or pays them.
fn check_unowned_battle_end(
    before: &GameState,
    after: &GameState,
    events: &[GameEvent],
    heroes: [Option<HeroId>; 2],
    result: BattleResult,
) -> Result<(), TestCaseError> {
    let visitor = heroes[BattleSide::Attacker.index()].unwrap();
    let owner = before.adventure().hero(visitor).unwrap().owner;
    let started = events.iter().find_map(|e| match *e {
        GameEvent::BattleStarted { tile, .. } => Some(tile),
        _ => None,
    });
    let tile = before
        .battle(owner)
        .map(OpenBattle::tile)
        .or(started)
        .unwrap();
    let map = before.adventure().map();
    let index = map
        .visited_from(tile)
        .iter()
        .copied()
        .rev()
        .find(|&i| !before.adventure().is_removed(i))
        .filter(|&i| map.objects()[i].monster.is_some() || map.objects()[i].guards.is_some())
        .ok_or_else(|| fail(format!("no monster or guards fought at {tile:?}")))?;
    let object = &map.objects()[index];
    let removals = events
        .iter()
        .filter(|e| {
            matches!(
                e,
                GameEvent::Adventure(AdventureEvent {
                    kind: AdventureEventKind::ObjectRemoved { anchor, .. },
                    ..
                }) if *anchor == object.anchor
            )
        })
        .count();
    let won = result.winner() == Some(BattleSide::Attacker);
    let (removed, gains) = if let Some(monster) = &object.monster {
        let beaten = won || result == BattleResult::Draw;
        if !beaten {
            let had = before.adventure().monster_count(index).unwrap();
            let left = after.adventure().monster_count(index).unwrap();
            prop_assert!((1..=had).contains(&left), "{} of {}", left, had);
        }
        (beaten, won.then(|| monster.treasure.clone()))
    } else {
        if !won {
            let had = before.adventure().guards_left(index);
            let left = after.adventure().guards_left(index);
            prop_assert!(
                left.iter().all(|s| had
                    .iter()
                    .any(|h| (h.slot, h.creature) == (s.slot, s.creature) && h.count >= s.count)),
                "{:?} of {:?}",
                left,
                had
            );
            prop_assert_eq!(left.is_empty(), result == BattleResult::Draw);
        }
        (won, won.then(|| object.contents.clone()))
    };
    prop_assert_eq!(after.adventure().is_removed(index), removed, "{:?}", result);
    prop_assert_eq!(removals, usize::from(removed));
    prop_assert!(
        !matches!(result, BattleResult::Surrender(_)),
        "{:?}",
        result
    );
    let mut treasury = before.treasury(owner).unwrap().to_vec();
    let mut gained = Vec::new();
    for event in events {
        match *event {
            GameEvent::ResourceGained {
                player,
                resource,
                amount,
            } => {
                prop_assert_eq!(player, owner);
                treasury[usize::from(resource.raw())] += amount;
                gained.push((resource, amount));
            }
            GameEvent::ResourcePaid { .. } => {
                return Err(fail(format!("{event:?} in a battle without an enemy hero")));
            }
            _ => {}
        }
    }
    prop_assert_eq!(gained, gains.unwrap_or_default());
    prop_assert_eq!(after.treasury(owner).unwrap(), &treasury[..]);
    for player in (0..before.players()).map(|p| PlayerId::new(u8::try_from(p).unwrap())) {
        if player != owner {
            prop_assert_eq!(before.treasury(player), after.treasury(player));
        }
    }
    Ok(())
}

/// The heroes' part of [`check_battle_end`] (E08 T19a2a): each change goes to one of the
/// battle's heroes and is told to its owner; every other hero's details stay; a battle hero's
/// experience grows by the gains it is told (only one that stays on the map gains), its spell
/// points become the last it is told, its artifacts change in number by the ones it is told it
/// received and lost, and nothing else of it changes; the two heroes' artifacts only move between
/// them or leave with a destroyed war machine.
fn check_heroes_after_battle(
    rules: &GameRuleset,
    (before, after): (&GameState, &GameState),
    events: &[GameEvent],
    heroes: [Option<HeroId>; 2],
) -> Result<(), TestCaseError> {
    let fought: Vec<HeroId> = heroes.iter().flatten().copied().collect();
    let changes: Vec<(HeroId, HeroChange)> = events
        .iter()
        .filter_map(|e| match *e {
            GameEvent::HeroChanged {
                player,
                hero,
                change,
            } => Some((player, hero, change)),
            _ => None,
        })
        .map(|(player, hero, change)| {
            prop_assert!(fought.contains(&hero), "{:?} changed", hero);
            prop_assert_eq!(player, before.adventure().hero(hero).unwrap().owner);
            Ok((hero, change))
        })
        .collect::<Result<_, TestCaseError>>()?;
    for (i, (b, a)) in before.heroes().iter().zip(after.heroes()).enumerate() {
        let id = HeroId::new(u16::try_from(i).unwrap());
        if !fought.contains(&id) {
            prop_assert_eq!(b, a);
            continue;
        }
        let told = changes
            .iter()
            .filter(|(hero, _)| *hero == id)
            .map(|&(_, c)| c);
        let (mut experience, mut mana, mut items) =
            (b.experience, b.mana, b.worn.len() + b.backpack.len());
        let mut spells = b.spells.clone();
        for change in told {
            match change {
                HeroChange::GainExperience(points) => {
                    prop_assert!(after.adventure().hero(id).is_some(), "{:?} gained", id);
                    experience = experience.saturating_add(points);
                }
                HeroChange::SetSpellPoints(points) => mana = points,
                HeroChange::AddArtifact(_) => items += 1,
                HeroChange::RemoveArtifact(_) => items -= 1,
                // E08 T19a2b: only the winner that stays learns (Eagle Eye's picks).
                HeroChange::LearnSpell(spell) => {
                    prop_assert!(after.adventure().hero(id).is_some(), "{:?} learned", id);
                    spells.push(spell);
                }
            }
        }
        prop_assert_eq!((a.experience, a.mana), (experience, mana));
        prop_assert_eq!(a.worn.len() + a.backpack.len(), items);
        prop_assert_eq!(&a.spells, &spells);
        prop_assert_eq!(
            (a.hero_type, a.level, a.primary, &a.skills),
            (b.hero_type, b.level, b.primary, &b.skills)
        );
    }
    // E08 T19a2b: creatures are raised into the army of a battle hero that stays, told to its
    // owner, as a stack of the creature told with at least that many.
    for event in events {
        if let GameEvent::CreaturesRaised {
            player,
            hero,
            creature,
            count,
        } = *event
        {
            prop_assert!(fought.contains(&hero), "{:?} raised", hero);
            prop_assert_eq!(player, before.adventure().hero(hero).unwrap().owner);
            let stays = after.adventure().hero(hero);
            prop_assert!(
                stays.is_some_and(|h| h
                    .army
                    .iter()
                    .any(|s| s.creature == creature && s.count >= count)),
                "{:?} raised {} of {:?}",
                hero,
                count,
                creature
            );
        }
    }
    let items = |state: &GameState| {
        let mut all: Vec<_> = fought
            .iter()
            .flat_map(|&id| {
                let d = state.hero(id).unwrap();
                d.worn
                    .iter()
                    .chain(&d.backpack)
                    .copied()
                    .collect::<Vec<_>>()
            })
            .map(|item| (item.artifact.raw(), item.spell.map(|s| s.raw())))
            .collect();
        all.sort_unstable();
        all
    };
    let mut gone = items(before);
    for item in items(after) {
        let at = gone.iter().position(|&i| i == item);
        prop_assert!(at.is_some(), "{:?} appeared", item);
        gone.remove(at.unwrap());
    }
    for (artifact, _) in gone {
        let lost = rules
            .core()
            .artifact(ArtifactTypeId::from_raw(artifact))
            .unwrap();
        prop_assert!(lost.war_machine.is_some(), "{:?} vanished", lost.key);
    }
    Ok(())
}

/// The pending decisions (GAM-DEC-001, ADV-VIS-001, ADV-VIS-004; E08 T14b, T15a) and the keys
/// (ADV-VIS-003), after an accepted command of `actor` (for the system's, the player whose battle
/// it plays): an offer opens, as the actor's decision, only when it was told so and could pay the
/// price, the object's type's (the actor's treasury after the command holds it), or is refused
/// when it could not; a Cartographer's offers its reveal for its price to a player that has not
/// bought its type, a border guard's its passage without a price to a holder of its key, (E08
/// T20b3b1) a monster's its group to join (for free, or for gold the actor holds) or its flight,
/// never refused (an offer to join for gold the actor cannot pay does not open), (E08 T21a) a
/// guarded pile's question while its guards stand and its message asks, never refused; an answer
/// closes the decision it names; only an accepted Cartographer's adds the type to the actor's purchases;
/// an accepted guard's removes the guard, an accepted join (told as the creatures joining) and a
/// declined flight the monster, each told as its removal, and an accepted pursuit or question
/// starts its battle; the actor's keys grow only by the keys it is told it gained, each one it did not hold;
/// nothing else changes either.
fn check_decisions(
    rules: &GameRuleset,
    (before, after): (&GameState, &GameState),
    actor: PlayerId,
    events: &[GameEvent],
) -> Result<(), TestCaseError> {
    let affords = |state: &GameState, price: &[(ResourceId, u32)]| {
        let held = state.treasury(actor).unwrap();
        price
            .iter()
            .all(|&(resource, amount)| held[usize::from(resource.raw())] >= amount)
    };
    let mut decision = before.decision(actor).cloned();
    let mut bought: Vec<_> = before.adventure().bought(actor).collect();
    let mut keys = before.adventure().keys(actor);
    let mut passed = None;
    for event in events {
        match *event {
            GameEvent::OfferOpened {
                player,
                hero,
                object_type,
                anchor,
            }
            | GameEvent::OfferRefused {
                player,
                hero,
                object_type,
                anchor,
            } => {
                let opened = matches!(event, GameEvent::OfferOpened { .. });
                prop_assert_eq!(player, actor);
                prop_assert!(decision.is_none());
                prop_assert_eq!(after.adventure().hero(hero).map(|h| h.owner), Some(actor));
                let kind = &rules.adventure().object_type(object_type).unwrap().kind;
                let kind = match kind {
                    ObjectKind::Monster { .. } => {
                        prop_assert!(opened, "{:?}: a monster's offer is never refused", event);
                        let open = after.decision(actor).map(|o| o.kind.clone());
                        prop_assert!(
                            matches!(
                                open,
                                Some(
                                    OfferKind::Join
                                        | OfferKind::JoinForGold { .. }
                                        | OfferKind::Pursue
                                )
                            ),
                            "{:?}: {:?}",
                            event,
                            open
                        );
                        open.unwrap()
                    }
                    ObjectKind::RevealTerrain { price, .. } => {
                        prop_assert!(!bought.contains(&object_type));
                        OfferKind::Reveal {
                            price: price.clone(),
                        }
                    }
                    ObjectKind::KeyGuard { key } => {
                        prop_assert!(keys >> key & 1 == 1, "{:?} without the key", event);
                        OfferKind::Pass
                    }
                    // E08 T21a: a guarded pile asks while its guards stand, never refused.
                    ObjectKind::Pickup(_) => {
                        prop_assert!(opened, "{:?}: a pile's question is never refused", event);
                        let map = before.adventure().map();
                        let asks = map.objects().iter().enumerate().any(|(i, o)| {
                            (o.object_type, o.anchor) == (Some(object_type), anchor)
                                && !before.adventure().is_removed(i)
                                && o.guards.as_ref().is_some_and(|g| g.question)
                                && !before.adventure().guards_left(i).is_empty()
                        });
                        prop_assert!(asks, "{:?} without guards that ask", event);
                        OfferKind::Fight
                    }
                    _ => return Err(fail(format!("{event:?} offers no offer"))),
                };
                let offer = Offer {
                    hero,
                    object_type,
                    anchor,
                    kind,
                };
                prop_assert_eq!(affords(after, offer.price()), opened, "{:?}", event);
                if opened {
                    decision = Some(offer);
                }
            }
            GameEvent::OfferAnswered {
                player,
                object_type,
                anchor,
                accepted,
            } => {
                prop_assert_eq!(player, actor);
                let open = decision.take().unwrap();
                prop_assert_eq!((open.object_type, open.anchor), (object_type, anchor));
                match (accepted, open.kind) {
                    (true, OfferKind::Reveal { .. }) => {
                        bought.push(object_type);
                        bought.sort();
                    }
                    (true, OfferKind::Pass) => passed = Some((object_type, anchor)),
                    // E08 T20b3b1: the group joins and leaves the map, a declined flight lets the
                    // monster leave; an accepted pursuit is its battle.
                    (true, OfferKind::Join | OfferKind::JoinForGold { .. }) => {
                        let joined = events.iter().any(|e| {
                            matches!(
                                e,
                                GameEvent::Adventure(AdventureEvent {
                                    kind: AdventureEventKind::CreaturesJoined { .. },
                                    ..
                                })
                            )
                        });
                        prop_assert!(joined, "{:?} without the creatures joining", event);
                        passed = Some((object_type, anchor));
                    }
                    (false, OfferKind::Pursue) => passed = Some((object_type, anchor)),
                    // E08 T21a: an accepted question of a guarded pile is its battle (its guards
                    // stand: nobody but the visitor's owner acts while it is open).
                    (true, OfferKind::Pursue | OfferKind::Fight) => {
                        let fought = events
                            .iter()
                            .any(|e| matches!(e, GameEvent::BattleStarted { .. }));
                        prop_assert!(fought, "{:?} without its battle", event);
                    }
                    (false, _) => {}
                }
            }
            GameEvent::Adventure(AdventureEvent {
                kind: AdventureEventKind::KeyGained { key },
                seen_by,
            }) => {
                prop_assert_eq!(seen_by, Viewers::NONE.with(actor));
                prop_assert!(keys >> key & 1 == 0, "{:?} held", event);
                keys |= 1 << key;
            }
            GameEvent::Adventure(AdventureEvent {
                kind:
                    AdventureEventKind::ObjectRemoved {
                        object_type,
                        anchor,
                    },
                ..
            }) if passed.is_some() => {
                prop_assert_eq!(passed.take(), Some((object_type, anchor)));
            }
            _ => {}
        }
    }
    prop_assert_eq!(passed, None);
    prop_assert_eq!(after.decision(actor), decision.as_ref());
    prop_assert!(after.adventure().bought(actor).eq(bought));
    prop_assert_eq!(after.adventure().keys(actor), keys);
    Ok(())
}

/// Ownership isolation: a command of `actor` moves no other player's hero, changes no other
/// player's army, explored tiles, treasury, purchases, open decision (E08 T14b) or keys (T15a),
/// and changes another player's heroes' points only when that player's turn starts
/// (ADV-MOV-002); but a battle (E08 T19a) the actor owns with another player: its commands play
/// the battle both own, and the one that ends it changes the other's hero (its army, or its
/// removal) and, by a surrender, its treasury ([`check_battle_end`]).
fn check_isolation(
    before: &GameState,
    after: &GameState,
    actor: PlayerId,
    events: &[GameEvent],
) -> Result<(), TestCaseError> {
    let fought: Vec<HeroId> = ended(events)
        .iter()
        .flat_map(|(heroes, _)| heroes.iter().flatten().copied())
        .collect();
    let (b, a) = (before.adventure().heroes(), after.adventure().heroes());
    for b in b {
        let a = a.iter().find(|a| a.id == b.id);
        if fought.contains(&b.id) {
            continue;
        }
        let a = a.ok_or_else(|| fail(format!("{:?} left the map", b.id)))?;
        prop_assert_eq!((b.id, b.owner), (a.id, a.owner));
        if b.owner == actor {
            continue;
        }
        prop_assert_eq!((b.pos, &b.army), (a.pos, &a.army));
        let started = events.contains(&GameEvent::TurnStarted { player: b.owner });
        prop_assert!(started || b.movement == a.movement);
    }
    prop_assert!(a.iter().all(|a| b.iter().any(|b| b.id == a.id)));
    // The other owner of the actor's battle, before or after the command.
    let partner = |state: &GameState| {
        state
            .battle(actor)
            .map(|battle| battle.owners().into_iter().flatten().collect::<Vec<_>>())
            .unwrap_or_default()
    };
    let (mut partners, after_partners) = (partner(before), partner(after));
    partners.extend(after_partners);
    let surrendered = ended(events)
        .iter()
        .any(|(_, result)| matches!(result, BattleResult::Surrender(_)));
    for player in (0..before.players()).map(|p| PlayerId::new(u8::try_from(p).unwrap())) {
        if player != actor {
            prop_assert!(before.adventure().explored(player) == after.adventure().explored(player));
            if !partners.contains(&player) {
                prop_assert_eq!(before.battle(player), after.battle(player));
            }
            if !(surrendered && partners.contains(&player)) {
                prop_assert_eq!(before.treasury(player), after.treasury(player));
            }
            prop_assert!(
                before
                    .adventure()
                    .bought(player)
                    .eq(after.adventure().bought(player))
            );
            prop_assert_eq!(before.decision(player), after.decision(player));
            prop_assert_eq!(
                before.adventure().keys(player),
                after.adventure().keys(player)
            );
        }
    }
    Ok(())
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 48, ..ProptestConfig::default() })]

    /// Atomicity, legality consistency, ownership isolation and conservation at every step of
    /// random play on imported maps, in both profiles.
    #[test]
    fn every_step_keeps_the_commands_invariants(
        plan in plan(),
        strict in any::<bool>(),
        seed in any::<u64>(),
        steps in steps(),
    ) {
        let rules = if strict { &*STRICT } else { &*FAITHFUL };
        let mut state = start(&plan, rules, seed);
        for step in &steps {
            let legal = options(&state, rules);
            let reaches = reaches(&state, rules);
            check_legal(&state, rules, &legal, &reaches)?;
            let (actor, command) = resolve(&state, rules, &legal, step);
            let listed = actor == self::actor(&state) && legal.contains(&command);
            let before = state.clone();
            let hash = state.state_hash();
            let mut events = Vec::new();
            let result = apply(&mut state, rules, actor, command.clone(), &mut events);
            if result.is_err() {
                // Atomicity: a rejected command changes nothing and emits nothing.
                prop_assert_eq!(&state, &before);
                prop_assert_eq!(state.state_hash(), hash);
                prop_assert!(events.is_empty());
                prop_assert!(!listed, "legal {:?} rejected", command);
                continue;
            }
            prop_assert_eq!(actor, self::actor(&before));
            if !listed {
                check_unlisted(rules, (&before, &state), actor, &command, &reaches)?;
            }
            if let Some((hero, path)) = as_move(&command) {
                let start = before.adventure().hero(hero).unwrap().pos;
                let (at, _) = landing(&before, start, path, &events)?;
                prop_assert_eq!(state.adventure().hero(hero).map(|h| h.pos), Some(at));
            }
            let player = player_of(&before, actor);
            check_decisions(rules, (&before, &state), player, &events)?;
            check_isolation(&before, &state, player, &events)?;
            check_conservation(rules, &before, &state, &events)?;
        }
    }

    /// Clone equivalence: a fork playing other commands leaves its parent as it was, and a fork
    /// and its parent evolve identically under the same commands.
    #[test]
    fn forks_evolve_as_their_parent(
        plan in plan(),
        seed in any::<u64>(),
        steps in steps().prop_filter("a step", |steps| !steps.is_empty()),
        split in any::<proptest::sample::Index>(),
    ) {
        let rules = &*FAITHFUL;
        let (prefix, rest) = steps.split_at(split.index(steps.len()));
        let (mut state, _, _) = play(&plan, rules, seed, prefix);
        let hash = state.state_hash();
        let mut wild = state.clone();
        for step in rest.iter().rev() {
            let _ = advance(&mut wild, rules, step, &mut Vec::new());
        }
        prop_assert_eq!(state.state_hash(), hash);
        prop_assert_eq!(&state, &play(&plan, rules, seed, prefix).0);
        let mut fork = state.clone();
        for step in rest {
            let (mut a, mut b) = (Vec::new(), Vec::new());
            let ra = advance(&mut state, rules, step, &mut a);
            let rb = advance(&mut fork, rules, step, &mut b);
            prop_assert_eq!(ra, rb);
            prop_assert_eq!(a, b);
            prop_assert_eq!(&state, &fork);
            prop_assert_eq!(state.state_hash(), fork.state_hash());
        }
    }

    /// Determinism: the same map file, seed and commands give the same states, events and hashes,
    /// also with the rules compiled again.
    #[test]
    fn replays_are_deterministic(
        plan in plan(),
        seed in any::<u64>(),
        steps in steps(),
    ) {
        let m = lay_out(&plan);
        prop_assert_eq!(parse(&encode(&m), &Limits::default()), Ok(m));
        let again = play(&plan, &FAITHFUL_AGAIN, seed, &steps);
        prop_assert_eq!(play(&plan, &FAITHFUL, seed, &steps), again);
    }
}

/// What a set of games played: the battles started and how they ended, and the heroes' part of
/// their outcomes (E08 T19b2); (E08 T20b3b1) the monsters' battles, those on a special ground,
/// how they ended for the visitor, the monsters' offers by kind, and the groups that joined; (E08
/// T21a) the guarded piles' questions and battles, and how those ended.
#[derive(Debug, Default)]
struct Fought {
    started: usize,
    victories: usize,
    draws: usize,
    retreats: usize,
    surrenders: usize,
    commands: usize,
    captured: usize,
    raised: usize,
    learned: usize,
    casts: usize,
    monster_battles: usize,
    on_ground: usize,
    monsters_beaten: usize,
    visitors_lost: usize,
    system_commands: usize,
    joins_offered: usize,
    gold_offered: usize,
    flights_offered: usize,
    joined: usize,
    questions: usize,
    guard_battles: usize,
    piles_won: usize,
    guards_held: usize,
    /// The visitors of the guarded piles' battles in progress.
    at_piles: Vec<HeroId>,
}

impl Fought {
    /// Counts `events`, which a command emitted on its way to `after`.
    fn count(&mut self, rules: &GameRuleset, after: &GameState, events: &[GameEvent]) {
        let part = rules.adventure();
        for event in events {
            match *event {
                GameEvent::BattleStarted { heroes, tile, .. }
                    if heroes[1].is_none() && Self::at_pile(after, tile) =>
                {
                    self.started += 1;
                    self.guard_battles += 1;
                    self.at_piles.extend(heroes[0]);
                }
                GameEvent::BattleEnded { heroes, result, .. }
                    if heroes[0].is_some_and(|h| self.at_piles.contains(&h)) =>
                {
                    self.count_result(result);
                    self.at_piles.retain(|&h| Some(h) != heroes[0]);
                    if result.winner() == Some(BattleSide::Attacker) {
                        self.piles_won += 1;
                    } else {
                        self.guards_held += 1;
                    }
                }
                GameEvent::BattleStarted { heroes, tile, .. } if heroes[1].is_none() => {
                    self.started += 1;
                    self.monster_battles += 1;
                    let ground = |o: &h3_adventure::MapObject| {
                        o.lies_on(tile)
                            && o.object_type
                                .and_then(|id| part.object_type(id))
                                .is_some_and(|ty| matches!(ty.kind, ObjectKind::Ground { .. }))
                    };
                    if after.adventure().map().objects().iter().any(ground) {
                        self.on_ground += 1;
                    }
                }
                GameEvent::BattleEnded { heroes, result, .. } if heroes[1].is_none() => {
                    self.count_result(result);
                    if result.winner() == Some(BattleSide::Attacker) {
                        self.monsters_beaten += 1;
                    } else {
                        self.visitors_lost += 1;
                    }
                }
                GameEvent::OfferOpened {
                    player,
                    object_type,
                    ..
                } if matches!(
                    part.object_type(object_type).map(|t| &t.kind),
                    Some(ObjectKind::Monster { .. })
                ) =>
                {
                    match after.decision(player).map(|o| &o.kind) {
                        Some(OfferKind::Join) => self.joins_offered += 1,
                        Some(OfferKind::JoinForGold { .. }) => self.gold_offered += 1,
                        Some(OfferKind::Pursue) => self.flights_offered += 1,
                        _ => {}
                    }
                }
                GameEvent::OfferOpened { player, .. }
                    if after.decision(player).map(|o| &o.kind) == Some(&OfferKind::Fight) =>
                {
                    self.questions += 1;
                }
                GameEvent::Adventure(AdventureEvent {
                    kind: AdventureEventKind::CreaturesJoined { .. },
                    ..
                }) => self.joined += 1,
                GameEvent::BattleStarted { .. } => self.started += 1,
                GameEvent::BattleEnded { result, .. } => self.count_result(result),
                GameEvent::HeroChanged { change, .. } => match change {
                    HeroChange::AddArtifact(_) => self.captured += 1,
                    HeroChange::LearnSpell(_) => self.learned += 1,
                    _ => {}
                },
                GameEvent::CreaturesRaised { .. } => self.raised += 1,
                GameEvent::Battle {
                    event:
                        Event {
                            kind: EventKind::SpellCast { .. },
                            ..
                        },
                    ..
                } => self.casts += 1,
                _ => {}
            }
        }
    }

    /// Whether the battle fought at `tile` is against a guarded pile's guards.
    fn at_pile(state: &GameState, tile: TilePos) -> bool {
        let map = state.adventure().map();
        map.visited_from(tile)
            .iter()
            .rev()
            .find(|&&i| !state.adventure().is_removed(i))
            .is_some_and(|&i| map.objects()[i].guards.is_some())
    }

    fn count_result(&mut self, result: BattleResult) {
        match result {
            BattleResult::Victory(_) => self.victories += 1,
            BattleResult::Draw => self.draws += 1,
            BattleResult::Retreat(_) => self.retreats += 1,
            BattleResult::Surrender(_) => self.surrenders += 1,
        }
    }
}

/// The plans play battles to a victory (E08 T19b2): over a fixed set of cases (proptest's
/// deterministic runner), random play with the fighting runs ([`steps`]) starts battles that end
/// by victories as well as by retreats and surrenders, and their winners capture artifacts, raise
/// creatures and learn Eagle Eye's picks, so the properties above see each. Since E08 T20b3b1 it
/// meets monsters too: their battles, the system playing their side, some on a special ground,
/// won and lost; their offers to join, to join for gold and to flee, and groups that join. Since
/// E08 T21a guarded piles too: their questions, and their battles won and lost.
#[test]
fn the_plans_play_battles_to_a_victory() {
    use proptest::strategy::ValueTree;
    let mut runner = proptest::test_runner::TestRunner::deterministic();
    let strategy = (plan(), any::<bool>(), any::<u64>(), steps());
    let mut fought = Fought::default();
    for _ in 0..192 {
        let (plan, strict, seed, steps) = strategy.new_tree(&mut runner).unwrap().current();
        let rules = if strict { &*STRICT } else { &*FAITHFUL };
        let mut state = start(&plan, rules, seed);
        for step in &steps {
            let mut events = Vec::new();
            let fighting = state.battle(acting(&state)).is_some();
            let system = actor(&state) == Actor::System;
            if advance(&mut state, rules, step, &mut events).is_ok() && fighting {
                fought.commands += 1;
                fought.system_commands += usize::from(system);
            }
            fought.count(rules, &state, &events);
        }
    }
    let f = &fought;
    assert!(f.started > 0 && f.commands > 0 && f.casts > 0, "{f:?}");
    assert!(f.victories > f.retreats + f.surrenders + f.draws, "{f:?}");
    assert!(f.retreats > 0 && f.surrenders > 0, "{f:?}");
    assert!(f.captured > 0 && f.raised > 0 && f.learned > 0, "{f:?}");
    assert!(
        f.monster_battles > 0 && f.system_commands > 0 && f.on_ground > 0,
        "{f:?}"
    );
    assert!(f.monsters_beaten > 0 && f.visitors_lost > 0, "{f:?}");
    assert!(
        f.joins_offered > 0 && f.flights_offered > 0 && f.joined > 0,
        "{f:?}"
    );
    assert!(
        f.questions > 0 && f.guard_battles > 0 && f.piles_won > 0 && f.guards_held > 0,
        "{f:?}"
    );
    assert!(f.gold_offered > 0, "{f:?}");
}
