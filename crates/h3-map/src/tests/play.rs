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
//! to act (a battle's outcome changes armies, removes heroes and moves a surrender's gold). What
//! a viewer may learn is T8b2b's ([`super::view`]), on the same maps and driver.

use std::sync::{Arc, LazyLock};

use h3_adventure::{
    AdventureCommand, AdventureEvent, AdventureEventKind, ObjectKind, Offer, OfferKind,
    Pos as TilePos, Reach, Viewers,
};
use h3_content::{ArtifactTypeId, ResourceId};
use h3_contracts::{BattleResult, HeroId, PlayerId};
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
use crate::model::{Artifacts, Body, HeroBody, RawMap, Stack, Template, Tile, Version};
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
/// drawn amount, a random resource, the Pillar of Fire, and the green tent, guard and gate.
pub(super) const KINDS: [(u32, u32, Option<u32>); 20] = [
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
    /// defaults); a hero with a backpack wears nothing (E08 T19a2a: a battle's capture).
    pub(super) backpacks: Vec<Vec<u16>>,
    /// Each object's anchor (modulo the map) and object (shape and kind, [`OBJECTS`]).
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
                ],
            ),
            0..=8,
        ),
        prop_oneof![Just(4), 0u8..4],
        proptest::option::weighted(0.75, ((0u8..7, 0u8..7), 0u8..3)),
        vec(vec(7u16..=40, 0..=3), 0..=6),
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
                backpacks,
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
                Plan {
                    version,
                    side,
                    two_levels,
                    tiles,
                    players: colours.into_iter().zip(humans).collect(),
                    heroes,
                    hero_types,
                    backpacks,
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
        let (blocked, visitable) = BLOCKERS[o % BLOCKERS.len()];
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
    // pile's, E08 T16a) and anchor.
    let mut typed: Vec<((u32, u32), Spot)> = Vec::new();
    for &((x, y, z), shape) in &plan.blockers {
        let (ax, ay, az) = (x % side, y % side, z % levels);
        let (blocked, visitable) = BLOCKERS[shape % BLOCKERS.len()];
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
        let twin = kind > 0 && typed.contains(&((class, subclass), (ax, ay, az)));
        if !twin && !heroes.iter().any(covers) {
            if kind > 0 {
                typed.push(((class, subclass), (ax, ay, az)));
            }
            let template = 1 + u32::try_from(shape).unwrap();
            let body = KINDS[shape / BLOCKERS.len()]
                .2
                .map_or(Body::None, |amount| Body::Resource {
                    message: None,
                    amount,
                });
            let mut o = object(ax, ay, template, body);
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
        let artifacts = backpack.map(|codes| Artifacts {
            worn: vec![None; v.artifact_slots()],
            backpack: codes.iter().copied().map(Some).collect(),
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
/// its battle's other side is to act, that side's owner (E08 T19a).
pub(super) fn acting(state: &GameState) -> PlayerId {
    let player = state.turn_group()[0];
    state
        .battle(player)
        .and_then(OpenBattle::owner_to_act)
        .unwrap_or(player)
}

/// The commands the driver picks from: the acting player's legal commands, then each of its legal
/// moves with the query's path, by hero and row by row. The game does not list moves one by one
/// (a hero with many points reaches thousands of tiles); the small maps here allow it.
pub(super) fn options(state: &GameState, rules: &GameRuleset) -> Vec<GameCommand> {
    let actor = Actor::Player(acting(state));
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

/// A step's actor and command, given the acting player's [`options`].
pub(super) fn resolve(
    state: &GameState,
    legal: &[GameCommand],
    step: &Step,
) -> (PlayerId, GameCommand) {
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
        Step::Legal(n) => (actor, legal[usize::from(*n) % legal.len()].clone()),
        Step::Visit(n) => {
            let visits: Vec<&GameCommand> = legal.iter().filter(visits).collect();
            let command = match visits.len() {
                0 => &legal[usize::from(*n) % legal.len()],
                len => visits[usize::from(*n) % len],
            };
            (actor, command.clone())
        }
        Step::EndTurn => (actor, GameCommand::EndTurn),
        Step::Edited(n, edit) => {
            let moves: Vec<(HeroId, &[TilePos])> = legal.iter().filter_map(as_move).collect();
            let Some(&(mut hero, path)) = moves.get(usize::from(*n) % moves.len().max(1)) else {
                return (actor, move_hero(HeroId::new(0), Vec::new()));
            };
            let origin = state.adventure().hero(hero).unwrap().pos;
            let mut path = path.to_vec();
            let mut actor = actor;
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
                Edit::OtherPlayer(id) => actor = PlayerId::new(id % (players + 1)),
            }
            (actor, move_hero(hero, path))
        }
        Step::Free { player, command } => {
            let actor = player.map_or(actor, |player| PlayerId::new(player % (players + 1)));
            match command {
                Free::EndTurn => (actor, GameCommand::EndTurn),
                Free::Answer(answer) => (actor, GameCommand::Answer(*answer)),
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
                    (actor, move_hero(hero, path))
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
                    (actor, move_hero(hero, path))
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
    let (actor, command) = resolve(state, &options, step);
    apply(state, rules, Actor::Player(actor), command, sink)
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
            let reach = state
                .adventure()
                .reach(rules.core(), rules.adventure(), hero.id);
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

/// Legality consistency, before a step: only the acting player has legal commands, `EndTurn`, or
/// with an open decision (only ever the acting player's in sequential play) its answers and
/// nothing else (GAM-DEC-001, E08 T14b), `Decline` alone if the offer no longer stands (T15a;
/// never in sequential play), or in a battle (T19a) the battle's commands while its side is to
/// act, each of which applies; its legal moves are the query of each of its own heroes while no
/// decision is open, and nobody has any for another player's; every listed answer applies and
/// closes the decision; every option's move applies and leaves the hero on its last tile (on the
/// tile before for a visit from a neighbour) with the points the query reports there, one event
/// per step.
fn check_legal(
    state: &GameState,
    rules: &GameRuleset,
    legal: &[GameCommand],
    reaches: &[(HeroId, Reach)],
) -> Result<(), TestCaseError> {
    let actor = acting(state);
    let fighting = state.battle(actor).is_some();
    let decided = state.decision(actor).is_some();
    let answers = match state.decision(actor) {
        Some(offer) if !state.adventure().offer_stands(offer) => &ANSWERS[1..],
        _ => &ANSWERS[..],
    };
    for player in (0..=state.players()).map(|p| PlayerId::new(u8::try_from(p).unwrap())) {
        let listed = legal_commands(state, rules, Actor::Player(player));
        prop_assert!(player == actor || state.decision(player).is_none());
        if player == actor && fighting {
            prop_assert!(!listed.is_empty());
            prop_assert!(listed.iter().all(|c| matches!(c, GameCommand::Battle(_))));
        } else {
            let expected = if player != actor {
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
            let own = player == actor
                && !decided
                && !fighting
                && state.adventure().hero(*hero).map(|h| h.owner) == Some(actor);
            prop_assert_eq!(moves.as_ref(), own.then_some(reach));
        }
    }
    if fighting {
        prop_assert_eq!(
            legal,
            &legal_commands(state, rules, Actor::Player(actor))[..]
        );
        for command in legal {
            let mut probe = state.clone();
            let result = apply(
                &mut probe,
                rules,
                Actor::Player(actor),
                command.clone(),
                &mut Vec::new(),
            );
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
            prop_assert!(probe.decision(actor).is_none());
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
/// legal moves, with at least the points the move left (every tile one step away is).
fn check_unlisted(
    (before, after): (&GameState, &GameState),
    actor: PlayerId,
    command: &GameCommand,
    reaches: &[(HeroId, Reach)],
) -> Result<(), TestCaseError> {
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
/// heroes' details (E08 T19a2a) change only by a battle's end.
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
            contents.extend(left.flat_map(|(_, o)| o.contents.iter().copied()));
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
    prop_assert_eq!(armies(before), armies(after));
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
/// destroyed war machine.
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
    for (side, hero) in heroes.iter().enumerate() {
        let hero = hero.unwrap();
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
        if paid.is_none() {
            prop_assert_eq!(b, a);
        }
    }
    check_heroes_after_battle(rules, (before, after), events, heroes)
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
/// (ADV-VIS-003), after an accepted command of `actor`: an offer opens, as the actor's decision,
/// only when it was told so and could pay the price, the object's type's (the actor's treasury
/// after the command holds it), or is refused when it could not; a Cartographer's offers its
/// reveal for its price to a player that has not bought its type, a border guard's its passage
/// without a price to a holder of its key; an answer closes the decision it names; only an
/// accepted Cartographer's adds the type to the actor's purchases, and only an accepted guard's
/// removes the guard, told as its removal; the actor's keys grow only by the keys it is told it
/// gained, each one it did not hold; nothing else changes either.
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
        steps in vec(step(), 0..40),
    ) {
        let rules = if strict { &*STRICT } else { &*FAITHFUL };
        let mut state = start(&plan, rules, seed);
        for step in &steps {
            let legal = options(&state, rules);
            let reaches = reaches(&state, rules);
            check_legal(&state, rules, &legal, &reaches)?;
            let (actor, command) = resolve(&state, &legal, step);
            let listed = actor == acting(&state) && legal.contains(&command);
            let before = state.clone();
            let hash = state.state_hash();
            let mut events = Vec::new();
            let result = apply(&mut state, rules, Actor::Player(actor), command.clone(), &mut events);
            if result.is_err() {
                // Atomicity: a rejected command changes nothing and emits nothing.
                prop_assert_eq!(&state, &before);
                prop_assert_eq!(state.state_hash(), hash);
                prop_assert!(events.is_empty());
                prop_assert!(!listed, "legal {:?} rejected", command);
                continue;
            }
            prop_assert_eq!(actor, acting(&before));
            if !listed {
                check_unlisted((&before, &state), actor, &command, &reaches)?;
            }
            if let Some((hero, path)) = as_move(&command) {
                let start = before.adventure().hero(hero).unwrap().pos;
                let (at, _) = landing(&before, start, path, &events)?;
                prop_assert_eq!(state.adventure().hero(hero).map(|h| h.pos), Some(at));
            }
            check_decisions(rules, (&before, &state), actor, &events)?;
            check_isolation(&before, &state, actor, &events)?;
            check_conservation(rules, &before, &state, &events)?;
        }
    }

    /// Clone equivalence: a fork playing other commands leaves its parent as it was, and a fork
    /// and its parent evolve identically under the same commands.
    #[test]
    fn forks_evolve_as_their_parent(
        plan in plan(),
        seed in any::<u64>(),
        steps in vec(step(), 1..40),
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
        steps in vec(step(), 0..40),
    ) {
        let m = lay_out(&plan);
        prop_assert_eq!(parse(&encode(&m), &Limits::default()), Ok(m));
        let again = play(&plan, &FAITHFUL_AGAIN, seed, &steps);
        prop_assert_eq!(play(&plan, &FAITHFUL, seed, &steps), again);
    }
}
