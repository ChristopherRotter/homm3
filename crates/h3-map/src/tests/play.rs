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
//! that winners capture, raise and learn (`the_plans_play_battles_to_a_victory`); since T20b3b1
//! with wandering monsters and special grounds (their offers to join, to join for gold and to
//! flee, groups joining, and their battles, whose unowned side the driver plays as the system
//! actor); since T21a with guarded piles (their questions, and their battles won and lost); since
//! T52 with management (increment 6): a mate of the first hero beside it with Scholar, spells and
//! Elixir of Life's parts, a full army whose join opens the army dialog, the meetings, the
//! management commands listed and arbitrary ones, the invariants of every state, management's
//! effects and a move's inverse (`the_plans_meet_and_manage`). What a viewer may learn is T8b2b's
//! ([`super::view`]), on the same maps and driver.

use std::ops::RangeInclusive;
use std::sync::{Arc, LazyLock};

use h3_adventure::{
    AdventureCommand, AdventureCommandError, AdventureEvent, AdventureEventKind, ObjectKind, Offer,
    OfferKind, Passage, Pos as TilePos, Reach, Viewers,
};
use h3_battle::{
    CastTargets, Command, Event, EventKind, cast_targets, castable_spells, second_targets,
};
use h3_content::{ArtifactTypeId, CreatureTypeId, Formation, ResourceId};
use h3_contracts::{BattleResult, BattleSide, HeroId, PlayerId};
use h3_game::{
    Actor, Answer, ArmyCommand, ArmyRef, ArmySlot, ArtifactPlace, GameCommand, GameCommandError,
    GameEvent, GameRuleset, GameState, HeroChange, Manage, OpenBattle, Position, Sink, apply,
    legal_commands, legal_management, legal_moves, split_range,
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
    Artifacts, Bits, Body, HeroBody, Message, MonsterBody, MonsterTreasure, RawMap, Stack,
    Template, Text, Tile, Version,
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
pub(super) const GUARDED: [(usize, bool, Guard); 2] = [
    (27, true, [(0, 28, 8), (3, 98, 2)]),
    (28, false, [(2, 56, 20), (2, 56, 0)]),
];
/// A guarded pile's two stacks as slot, creature code and count (a count of 0: no stack).
pub(super) type Guard = [(u8, u16, u16); 2];
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
    /// Each hero's movement and scouting kit, by the hero's index (none or empty: nothing of its
    /// own; E08 T42): its worn artifacts join its backpack's, its skills join its own skills (a
    /// hero with skills of its own wears the Spell Book as above).
    pub(super) kits: Vec<Kit>,
    /// A stack of Nomads (count) in the army of the hero of this index (modulo the heroes) on an
    /// AB or SoD map (E08 T42: the `ab` pack's terrain walker), its first slot's or its only one.
    pub(super) nomads: Option<(u8, u16)>,
    /// Each object's anchor (modulo the map) and object (shape and kind, [`OBJECTS`]; since E08
    /// T20b3b1 also the monsters' and grounds', [`MONSTER_OBJECTS`], [`GROUND_OBJECTS`]).
    pub(super) blockers: Vec<(Spot, usize)>,
    /// The game's difficulty (E08 T14b), written as the map's (the import's own default is
    /// normal; the tests take the map's): impossible, with no gold to start, in half the cases,
    /// so that Cartographers are refused as well as offered.
    pub(super) difficulty: u8,
    /// (E08 T52) The first hero's exchanges: a mate of its owner beside it, and its full army;
    /// none: neither.
    pub(super) exchange: Option<Exchange>,
}

/// Elixir of Life's three parts (E08 T52; HER-ART-006: a SoD combination of artifacts every map
/// version holds), as code and position in a map's worn slots: Ring of Vitality (94) and Ring of
/// Life (95) on the rings, Vial of Lifeblood (96) in the third misc slot.
pub(super) const ELIXIR_PARTS: [(u16, usize); 3] = [(94, 6), (95, 7), (96, 11)];
/// Scholar's and Wisdom's skill codes (E08 T52; HER-SPL-007, HER-SPL-005).
const SCHOLAR: u8 = 18;
const WISDOM: u8 = 7;
/// The spells a mate may know of its own on a SoD map (E08 T52; the map's custom spells, which
/// only SoD stores): Magic Arrow, Bless, Haste and Slow (level 1), Ice Bolt and Lightning Bolt
/// (2), Fireball and Air Shield (3), Implosion (5).
const MATE_SPELLS: [u8; 9] = [15, 41, 53, 54, 16, 17, 21, 28, 18];
/// A full army's seven creatures (E08 T52), none a plan's monster's ([`MONSTERS`]), so that a
/// group joining it finds no room: archers, griffins, swordsmen, monks, cavaliers, dwarves and
/// wood elves.
const FULL_ARMY: [u16; 7] = [2, 4, 6, 8, 10, 16, 18];

/// The first hero's exchanges (E08 T52; increment 6's design note §12 in `E08-log.md`): a mate,
/// another hero of its owner on a tile beside it, so that a step meets it (GAM-MET-001), with a
/// Spell Book, Scholar and Wisdom of its own (the first hero wearing a book too, so that a meeting
/// teaches either, HER-SPL-007), spells of its own on a SoD map, and Elixir of Life's parts worn
/// (an assembly, HER-ART-006); and in some plans the first hero's army full, seven stacks, with six
/// compliant goblins near it, so that their join finds no room and opens the army dialog
/// (GAM-DEC-004).
#[derive(Debug, Clone)]
pub(super) struct Exchange {
    /// The mate's direction from the first hero ([`DIRECTIONS`]).
    pub(super) mate: u8,
    /// The mate's Scholar level (0: none).
    pub(super) scholar: u8,
    /// The mate's Wisdom level (0: none).
    pub(super) wisdom: u8,
    /// The mate's spells of its own on a SoD map, as indices into [`MATE_SPELLS`].
    pub(super) spells: Vec<usize>,
    /// Whether the mate wears Elixir of Life's parts ([`ELIXIR_PARTS`]).
    pub(super) parts: bool,
    /// The first hero's full army ([`FULL_ARMY`]'s counts), the goblins on the tile beside it
    /// opposite the mate's; none: neither.
    pub(super) full: Option<Vec<u16>>,
}

/// The first hero's exchanges (E08 T52): the mate in any direction, Scholar and Wisdom at a level
/// or none, up to five spells, the parts in one case of two, and a full army in one case of two.
fn exchange() -> impl Strategy<Value = Exchange> {
    (
        0u8..8,
        0u8..=3,
        0u8..=3,
        subsequence((0..MATE_SPELLS.len()).collect::<Vec<_>>(), 0..=5),
        any::<bool>(),
        proptest::option::weighted(0.5, vec(1u16..=40, FULL_ARMY.len())),
    )
        .prop_map(|(mate, scholar, wisdom, spells, parts, full)| Exchange {
            mate,
            scholar,
            wisdom,
            spells,
            parts,
            full,
        })
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

/// The six artifacts of a hero's movement and scouting (E08 T42; HER-ADV-003 to -006), as code and
/// position in a map's worn slots (the editor's, which the import keeps since v9, ADR-0013 point
/// 1): the Speculum (52) and the Spyglass (53) in the misc slots, Equestrian's Gloves (70) on a
/// ring, Angel Wings (72) on the shoulders (flying, ADV-MOV-009), Boots of Levitation (90, water
/// walking, ADV-MOV-010) and Boots of Speed (98) on the feet, one at a time.
pub(super) const MOVERS: [(u16, usize); 6] =
    [(52, 9), (53, 10), (70, 6), (72, 1), (90, 8), (98, 8)];
/// Angel Wings' code ([`MOVERS`]).
pub(super) const WINGS: u16 = 72;
/// Boots of Levitation's code ([`MOVERS`]).
pub(super) const LEVITATION: u16 = 90;
/// Nomads' creature code (AB and SoD maps; the `ab` pack's terrain walker: sand as dirt, ADV-MOV-004).
pub(super) const NOMADS: u16 = 142;

/// A hero's movement and scouting kit of its own (E08 T42), beside its army and skills: what it
/// wears of [`MOVERS`] and its Pathfinding (code 0), Logistics (2) and Scouting (3).
#[derive(Debug, Clone, Default)]
pub(super) struct Kit {
    /// Worn artifacts of [`MOVERS`], at most one on the feet.
    pub(super) worn: Vec<u16>,
    /// Skill code and level (1 to 3).
    pub(super) skills: Vec<(u8, u8)>,
}

/// A kit (E08 T42): in two heroes of five some of the Speculum, the Spyglass, Equestrian's Gloves
/// and Boots of Speed, in two of five some of the three skills at a level each. Angel Wings and
/// Boots of Levitation come with the plan's flyer and water walker ([`plan`]).
fn kit() -> impl Strategy<Value = Kit> {
    (
        proptest::option::weighted(0.4, subsequence(vec![52u16, 53, 70, 98], 1..=4)),
        proptest::option::weighted(0.4, (subsequence(vec![0u8, 2, 3], 1..=3), vec(1u8..=3, 3))),
    )
        .prop_map(|(worn, skills)| Kit {
            worn: worn.unwrap_or_default(),
            skills: skills.map_or_else(Vec::new, |(skills, levels)| {
                skills.into_iter().zip(levels).collect()
            }),
        })
}

/// A hero's custom experience (E08 T31): mostly up to 300 below the threshold of a level 2 to 20,
/// so that its levels at load stop one short of that level and its first battle gives the next
/// ones; sometimes any amount, the bound and past it (its levels at load then reach the highest).
pub(super) fn experience() -> impl Strategy<Value = u32> {
    prop_oneof![
        4 => (2usize..=20, 1u32..=300).prop_map(|(level, below)| {
            let thresholds = &FAITHFUL.core().hero_params().unwrap().experience;
            thresholds[level - 1] - below
        }),
        1 => any::<u32>(),
    ]
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
            // E08 T42, drawn last so that the draws before stay as they were (the regressions
            // files' cases keep their maps).
            (
                vec(kit(), 0..=6),
                proptest::option::weighted(0.75, (any::<bool>(), 0u8..4)),
                proptest::option::weighted(0.5, (any::<u8>(), 1u16..=20)),
                // E08 T52, drawn last for the same reason.
                proptest::option::weighted(0.5, exchange()),
            ),
        ),
    )
        .prop_map(
            |(
                version,
                side,
                two_levels,
                mut tiles,
                colours,
                humans,
                heroes,
                hero_types,
                mut blockers,
                difficulty,
                keyring,
                (backpacks, mut skills, beside, pile, (mut kits, lifts, nomads, exchange)),
            )| {
                // In three plans of four (E08 T42) a flyer and a water walker: the first two
                // heroes in either order wear Angel Wings and Boots of Levitation (in place of
                // Boots of Speed), in one of those cases of four the flyer both; and a strip of
                // three water tiles two columns east of each, on its level, so that the flight
                // and the water walk pass over water near the objects beside the first hero.
                if let Some((swap, both)) = lifts {
                    let levels = 1 + u8::from(two_levels);
                    let (flyer, walker) = if swap { (1, 0) } else { (0, 1) };
                    kits.resize_with(kits.len().max(2), Kit::default);
                    for (index, lift) in [(flyer, WINGS), (walker, LEVITATION)] {
                        let worn = &mut kits[index].worn;
                        worn.retain(|&a| a != 98);
                        worn.push(lift);
                        if lift == WINGS && both == 0 {
                            worn.push(LEVITATION);
                        }
                        let ((hx, hy, hz), _) = heroes[index];
                        let (x, z) = ((hx % side + 2) % side, hz % levels);
                        for dy in [side - 1, 0, 1] {
                            let y = (hy % side + dy) % side;
                            let i = (usize::from(z) * usize::from(side) + usize::from(y))
                                * usize::from(side)
                                + usize::from(x);
                            tiles[i].0 = 8;
                        }
                    }
                }
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
                    kits,
                    nomads,
                    blockers,
                    difficulty,
                    exchange,
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
    // E08 T52: the goblins of a full army ([`Exchange`]) too.
    let goblins = plan
        .exchange
        .as_ref()
        .and_then(|e| e.full.as_ref())
        .map(|_| MONSTER_OBJECTS[0]);
    let used = plan.blockers.iter().map(|&(_, o)| o).chain(goblins);
    let used = used.map(|o| o + 1).max().unwrap_or(0);
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
    // E08 T52: the first hero's full army, the goblins near it and its mate on the tile beside
    // it (none on a tile another hero holds), the last hero.
    let exchange = plan.exchange.as_ref();
    let full: Slots = exchange.and_then(|e| e.full.as_ref()).map(|counts| {
        FULL_ARMY
            .iter()
            .zip(counts)
            .map(|(&creature, &count)| Some((creature, count)))
            .collect()
    });
    let none: Slots = None;
    let mut goblins = None;
    let mut mate = None;
    if let (Some(exchange), Some(&((hx, hy, hz), _))) = (exchange, heroes.first()) {
        let beside = |direction: u8| {
            let (dx, dy) = DIRECTIONS[usize::from(direction % 8)];
            let shift = |h: u8, d: i16| {
                u8::try_from((i16::from(h) + d + i16::from(side)) % i16::from(side)).unwrap()
            };
            (shift(hx, dx), shift(hy, dy), hz)
        };
        if exchange.full.is_some() {
            heroes[0].1 = &full;
            goblins = Some((beside(exchange.mate + 4), MONSTER_OBJECTS[0]));
        }
        let at = beside(exchange.mate);
        if heroes.iter().all(|&(taken, _)| taken != at) {
            mate = Some(heroes.len());
            heroes.push((at, &none));
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
    for &((x, y, z), shape) in plan.blockers.iter().chain(&goblins) {
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
    // E08 T42: the hero with Nomads (AB and SoD know the creature).
    let nomads = plan
        .nomads
        .filter(|_| v != Version::RoE)
        .map(|(n, count)| (usize::from(n) % heroes.len(), count));
    for (index, &((x, y, z), army)) in (0u8..).zip(&heroes) {
        let mut army: Option<Vec<Option<Stack>>> = army.as_ref().map(|slots| {
            slots
                .iter()
                .map(|slot| slot.map(|(creature, count)| Stack { creature, count }))
                .collect()
        });
        if let Some((_, count)) = nomads.filter(|&(hero, _)| hero == usize::from(index)) {
            let slots = army.get_or_insert_with(|| vec![None; 7]);
            slots[0] = Some(Stack {
                creature: NOMADS,
                count,
            });
        }
        let hero_type = (u16::from(plan.hero_types) + 7 * u16::from(index)) % 128;
        let backpack = plan
            .backpacks
            .get(usize::from(index))
            .filter(|b| !b.is_empty());
        let kit = plan.kits.get(usize::from(index));
        let worn_kit = kit.map_or(&[][..], |kit| &kit.worn[..]);
        // A hero with skills of its own wears a Spell Book (artifact 0, on the map's book entry
        // 17; E08 T19b2, until T49a in the head's): Eagle Eye's picks are learned only with it
        // (HER-SPL-006), and its type may have none. Since E08 T42 its kit's skills are its own
        // too.
        let mut skills = plan.skills.get(usize::from(index)).cloned().flatten();
        if let Some(kit) = kit.filter(|kit| !kit.skills.is_empty()) {
            skills.get_or_insert_with(Vec::new).extend(&kit.skills);
        }
        // E08 T52: the mate's Scholar and Wisdom are its skills of its own; it and the first hero
        // wear a Spell Book (HER-SPL-007: a meeting teaches only between two books), the mate
        // Elixir of Life's parts, and on a SoD map its spells of its own.
        let mated = exchange.filter(|_| Some(usize::from(index)) == mate);
        if let Some(exchange) = mated {
            let levels = [(SCHOLAR, exchange.scholar), (WISDOM, exchange.wisdom)];
            skills = Some(levels.into_iter().filter(|&(_, l)| l > 0).collect());
        }
        let parts = mated
            .filter(|e| e.parts)
            .map_or(&[][..], |_| &ELIXIR_PARTS[..]);
        let book = skills.is_some() || (index == 0 && exchange.is_some());
        let own = backpack.is_some() || book || !worn_kit.is_empty();
        let artifacts = own.then(|| {
            let mut worn = vec![None; v.artifact_slots()];
            if book {
                worn[17] = Some(0);
            }
            for &artifact in worn_kit {
                let (_, slot) = MOVERS.iter().find(|&&(a, _)| a == artifact).unwrap();
                worn[*slot] = Some(artifact);
            }
            for &(artifact, slot) in parts {
                worn[slot] = Some(artifact);
            }
            Artifacts {
                worn,
                backpack: backpack.into_iter().flatten().copied().map(Some).collect(),
            }
        });
        let spells = mated.filter(|e| v == Version::Sod && !e.spells.is_empty());
        let spells = spells.map(|exchange| {
            let mut bits = vec![0u8; 9];
            for &i in &exchange.spells {
                let spell = MATE_SPELLS[i];
                bits[usize::from(spell / 8)] |= 1 << (spell % 8);
            }
            Bits(bits)
        });
        let owner = if mated.is_some() {
            0
        } else {
            usize::from(index)
        };
        let body = HeroBody {
            identifier: (v != Version::RoE).then_some(u32::from(index)),
            owner: colours[owner % colours.len()],
            hero_type: u8::try_from(hero_type).unwrap(),
            experience: (v != Version::Sod).then_some(0),
            army,
            patrol: u8::MAX,
            sex: u8::MAX,
            spell: (v == Version::Ab).then_some(u8::MAX),
            artifacts,
            secondary_skills: skills,
            spells,
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

/// The plan's map with `experience`, if any, as its heroes' custom experience, by the hero's index
/// in the map's order (E08 T31: their levels at load, GAM-SET-002).
pub(super) fn experienced(plan: &Plan, experience: Option<&[u32]>) -> RawMap {
    let mut m = lay_out(plan);
    let heroes = m.objects.iter_mut().filter_map(|o| match &mut o.body {
        Body::Hero(hero) => Some(hero),
        _ => None,
    });
    for (hero, &amount) in heroes.zip(experience.unwrap_or_default()) {
        hero.experience = Some(amount);
    }
    m
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
    /// (E08 T52) The acting player's `n`-th management command ([`managing`], modulo their
    /// number), or as `Legal(n)` if it has none (a battle, an offer or a level-up open).
    Manage(u16),
    /// (E08 T52) On the map the acting player's hero seeks another of its own ([`meet`]); with a
    /// meeting or an army dialog open, as `Manage(n)`.
    Meet(u16),
    /// (E08 T52) The acting player's open offer to join accepted; on the map its `n`-th option
    /// that ends on a wandering monster's tile ([`join`]); else as `Meet(n)`.
    Join(u16),
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
    /// (E08 T52) A management command of arbitrary heroes, slots, places and counts.
    Manage(Arbitrary),
}

/// An arbitrary management command (E08 T52; GAM-MAN-001): its kind (modulo 9: a move, a split
/// and a dismissal of a stack, a formation, a flag, an artifact's move, an assembly, a
/// disassembly, a move onto the same slot), two heroes (or the group, `None`), two slots (up to
/// one past the army's), a count, and two places (worn or in the backpack, up to a few past the
/// last).
#[derive(Debug, Clone)]
pub(super) struct Arbitrary {
    kind: u8,
    heroes: [Option<Who>; 2],
    slots: [u8; 2],
    count: u32,
    places: [(bool, u8); 2],
}

impl Arbitrary {
    /// The command, its heroes resolved for `actor`.
    fn command(&self, state: &GameState, actor: PlayerId) -> Manage {
        let army = |who: Option<Who>| {
            who.map_or(ArmyRef::Group, |who| ArmyRef::Hero(who.hero(state, actor)))
        };
        let [a, b] = self.heroes.map(army);
        let [sa, sb] = self.slots;
        let from = ArmySlot { army: a, slot: sa };
        let to = ArmySlot { army: b, slot: sb };
        let hero = |who: Option<Who>| who.map_or(HeroId::new(0), |w| w.hero(state, actor));
        let place = |(worn, n): (bool, u8)| {
            if worn {
                ArtifactPlace::Worn(Position(usize::from(n)))
            } else {
                ArtifactPlace::Backpack(usize::from(n))
            }
        };
        let (h, at) = (
            hero(self.heroes[0]),
            Position(usize::from(self.places[0].1)),
        );
        match self.kind % 9 {
            0 => Manage::Army(ArmyCommand::Move { from, to }),
            1 => Manage::Army(ArmyCommand::Split {
                from,
                to,
                count: self.count,
            }),
            2 => Manage::Army(ArmyCommand::Dismiss { at: from }),
            3 => Manage::Formation {
                hero: h,
                formation: if self.count.is_multiple_of(2) {
                    Formation::Loose
                } else {
                    Formation::Tight
                },
            },
            4 => Manage::Tactics {
                hero: h,
                on: self.count.is_multiple_of(2),
            },
            5 => Manage::Artifact {
                from: (h, place(self.places[0])),
                to: (hero(self.heroes[1]), place(self.places[1])),
            },
            6 => Manage::Assemble { hero: h, at },
            7 => Manage::Disassemble { hero: h, at },
            _ => Manage::Army(ArmyCommand::Move { from, to: from }),
        }
    }
}

fn arbitrary() -> impl Strategy<Value = Arbitrary> {
    (
        any::<u8>(),
        [
            proptest::option::weighted(0.9, who()),
            proptest::option::weighted(0.9, who()),
        ],
        [0u8..9, 0u8..9],
        prop_oneof![3 => 0u32..50, 1 => any::<u32>()],
        [(any::<bool>(), 0u8..24), (any::<bool>(), 0u8..24)],
    )
        .prop_map(|(kind, heroes, slots, count, places)| Arbitrary {
            kind,
            heroes,
            slots,
            count,
            places,
        })
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
        arbitrary().prop_map(Free::Manage),
    ];
    prop_oneof![
        6 => any::<u16>().prop_map(Step::Legal),
        2 => any::<u16>().prop_map(Step::Visit),
        1 => Just(Step::EndTurn),
        2 => (any::<u16>(), edit).prop_map(|(n, edit)| Step::Edited(n, edit)),
        2 => (proptest::option::weighted(0.33, any::<u8>()), free)
            .prop_map(|(player, command)| Step::Free { player, command }),
        2 => any::<u16>().prop_map(Step::Manage),
    ]
}

/// Steps of random play (E08 T19b2): single steps and runs of [`Step::Fight`], long enough to
/// bring two players' heroes together and play their battle to its end; since E08 T52 runs of
/// management, a [`Step::Meet`] and then [`Step::Manage`]s, which meet a hero's mate and
/// exchange with it, and runs of joins, two [`Step::Join`]s (a visit to a monster and the
/// join's acceptance) and then [`Step::Manage`]s, which manage an army dialog.
pub(super) fn steps() -> impl Strategy<Value = Vec<Step>> {
    let run = vec(any::<u16>(), 16..96).prop_map(|ns| ns.into_iter().map(Step::Fight).collect());
    let manage = (any::<u16>(), vec(any::<u16>(), 2..16)).prop_map(|(n, ns)| {
        core::iter::once(Step::Meet(n))
            .chain(ns.into_iter().map(Step::Manage))
            .collect()
    });
    let join = (any::<u16>(), vec(any::<u16>(), 2..16)).prop_map(|(n, ns)| {
        [Step::Join(n), Step::Join(n)]
            .into_iter()
            .chain(ns.into_iter().map(Step::Manage))
            .collect()
    });
    vec(
        prop_oneof![12 => step().prop_map(|step| vec![step]), 1 => run, 2 => manage, 1 => join],
        0..40,
    )
    .prop_map(|runs: Vec<Vec<Step>>| runs.into_iter().flatten().collect())
}

/// On the map (E08 T19b2): the acting player's open offer, meeting or army dialog declined (E08
/// T51b), its open level-up's first offer taken (E08 T30); else the `n`-th of its
/// options that end nearest to another player's hero on the hero's level, nearer than the hero
/// stands (an attack, on the other hero's tile, is nearest of all; the heroes seek each other as
/// the fork benchmark's do: random play seldom meets); `EndTurn` when none gets nearer.
fn seek(state: &GameState, legal: &[GameCommand], n: u16) -> GameCommand {
    let actor = acting(state);
    let exchanging = state.meeting(actor).is_some() || state.army_dialog(actor).is_some();
    if state.decision(actor).is_some() || exchanging {
        return GameCommand::Answer(Answer::Decline);
    }
    if state.level_up(actor).is_some() {
        return GameCommand::Answer(Answer::First);
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

/// The armies the acting player may manage now (E08 T52; ADR-0017 point 6): a meeting's two
/// heroes, an army dialog's hero and group, else each of its heroes alone; none while it may
/// not manage (a battle, an offer or a level-up open, the game waiting for another player).
fn managed(state: &GameState) -> Vec<Vec<ArmyRef>> {
    let player = acting(state);
    if actor(state) != Actor::Player(player)
        || state.battle(player).is_some()
        || state.decision(player).is_some()
        || state.waiting_for().is_some()
    {
        return Vec::new();
    }
    if let Some(heroes) = state.meeting(player) {
        return vec![heroes.map(ArmyRef::Hero).to_vec()];
    }
    if let Some((hero, _)) = state.army_dialog(player) {
        return vec![vec![ArmyRef::Hero(hero), ArmyRef::Group]];
    }
    let heroes = state.adventure().heroes().iter();
    heroes
        .filter(|h| h.owner == player)
        .map(|h| vec![ArmyRef::Hero(h.id)])
        .collect()
}

/// The stack at `at` (E08 T52): in its hero's army on the map, or in the acting player's army
/// dialog's group.
fn stack_at(state: &GameState, at: ArmySlot) -> Option<h3_contracts::ArmyStack> {
    let army = match at.army {
        ArmyRef::Hero(hero) => state.adventure().hero(hero).map(|h| &h.army[..]),
        ArmyRef::Group => state.army_dialog(acting(state)).map(|(_, group)| group),
    };
    army.and_then(|army| army.iter().find(|s| s.slot == at.slot).copied())
}

/// The splits the acting player may give now (E08 T52; GAM-MAN-002, ADV-ARM-003): of each stack
/// of the armies it manages ([`managed`]) with each other slot of those armies, each pair with
/// its [`split_range`].
fn splits(
    state: &GameState,
    rules: &GameRuleset,
) -> Vec<(ArmySlot, ArmySlot, RangeInclusive<u32>)> {
    let actor = actor(state);
    let mut splits = Vec::new();
    for armies in managed(state) {
        let slots: Vec<ArmySlot> = armies
            .iter()
            .flat_map(|&army| (0..7).map(move |slot| ArmySlot { army, slot }))
            .collect();
        for &from in slots.iter().filter(|&&at| stack_at(state, at).is_some()) {
            for &to in &slots {
                if let Some(range) = split_range(state, rules, actor, (from, to)) {
                    splits.push((from, to, range));
                }
            }
        }
    }
    splits
}

/// The acting player's management commands (E08 T52; GAM-MAN-002): each of its heroes'
/// [`legal_management`] by id, then its [`splits`], each to its range's ends and the count
/// between them.
pub(super) fn managing(state: &GameState, rules: &GameRuleset) -> Vec<GameCommand> {
    let actor = actor(state);
    let mut commands: Vec<GameCommand> = state
        .adventure()
        .heroes()
        .iter()
        .flat_map(|h| legal_management(state, rules, actor, h.id))
        .map(GameCommand::Manage)
        .collect();
    for (from, to, range) in splits(state, rules) {
        let (low, high) = (*range.start(), *range.end());
        let mut counts = vec![low, low + (high - low) / 2, high];
        counts.dedup();
        commands.extend(counts.into_iter().map(|count| {
            GameCommand::Manage(Manage::Army(ArmyCommand::Split { from, to, count }))
        }));
    }
    commands
}

/// A management command's kind (E08 T52): a stack's move, split or dismissal, a formation or a
/// flag, an artifact's move, an assembly or a disassembly.
fn kind(command: &GameCommand) -> usize {
    match command {
        GameCommand::Manage(Manage::Army(ArmyCommand::Move { .. })) => 0,
        GameCommand::Manage(Manage::Army(ArmyCommand::Split { .. })) => 1,
        GameCommand::Manage(Manage::Army(ArmyCommand::Dismiss { .. })) => 2,
        GameCommand::Manage(Manage::Formation { .. } | Manage::Tactics { .. }) => 3,
        GameCommand::Manage(Manage::Artifact { .. }) => 4,
        _ => 5,
    }
}

/// The `n`-th of `commands` (E08 T52), a kind ([`kind`]) chosen first among those listed, then a
/// command of it, so that a rare kind (an assembly among hundreds of moves) is played as often
/// as the others; none if there are none.
fn pick(commands: &[GameCommand], n: u16) -> Option<GameCommand> {
    let mut kinds: Vec<usize> = commands.iter().map(kind).collect();
    kinds.sort_unstable();
    kinds.dedup();
    let n = usize::from(n);
    let chosen = *kinds.get(n % kinds.len().max(1))?;
    let of: Vec<&GameCommand> = commands.iter().filter(|c| kind(c) == chosen).collect();
    Some(of[(n / kinds.len()) % of.len()].clone())
}

/// On the map (E08 T52): the acting player's open offer to join (for free or for gold) accepted,
/// another offer declined, its level-up's first offer taken; else the `n`-th of its options that
/// end on a wandering monster's tile (a visit: its offer, or its battle), those of a hero whose
/// army is full first; else as [`meet`].
fn join(state: &GameState, rules: &GameRuleset, legal: &[GameCommand], n: u16) -> GameCommand {
    let actor = acting(state);
    match state.decision(actor).map(|offer| &offer.kind) {
        Some(OfferKind::Join | OfferKind::JoinForGold { .. }) => {
            return GameCommand::Answer(Answer::Accept);
        }
        Some(_) => return GameCommand::Answer(Answer::Decline),
        None => {}
    }
    if battle_of(state, self::actor(state)).is_none()
        && state.level_up(actor).is_none()
        && state.meeting(actor).is_none()
        && state.army_dialog(actor).is_none()
    {
        let adventure = state.adventure();
        let monster = |pos: TilePos| {
            adventure.map().visited_from(pos).iter().any(|&i| {
                !adventure.is_removed(i) && adventure.map().objects()[i].monster.is_some()
            })
        };
        let visits: Vec<(HeroId, &GameCommand)> = legal
            .iter()
            .filter_map(|c| {
                let (hero, path) = as_move(c)?;
                path.last()
                    .is_some_and(|&to| monster(to))
                    .then_some((hero, c))
            })
            .collect();
        // A full army's visits first: their joins find no room.
        let slots = usize::from(rules.adventure().params().army_slots());
        let full = |hero: HeroId| adventure.hero(hero).is_some_and(|h| h.army.len() == slots);
        let first: Vec<&GameCommand> = visits
            .iter()
            .filter(|(h, _)| full(*h))
            .map(|&(_, c)| c)
            .collect();
        let all: Vec<&GameCommand> = visits.iter().map(|&(_, c)| c).collect();
        let visits = if first.is_empty() { all } else { first };
        if !visits.is_empty() {
            return visits[usize::from(n) % visits.len()].clone();
        }
    }
    meet(state, rules, legal, n)
}

/// On the map (E08 T52): the acting player's open offer declined, its level-up's first offer
/// taken, its meeting or army dialog managed (the `n`-th of [`managing`], `Decline` if none);
/// else the `n`-th of its options that end nearest to another of its own heroes on the hero's
/// level, nearer than the hero stands (a step onto the other's tile meets it, nearest of all);
/// `EndTurn` when none gets nearer. In a battle, as `Legal(n)`.
fn meet(state: &GameState, rules: &GameRuleset, legal: &[GameCommand], n: u16) -> GameCommand {
    let actor = acting(state);
    if battle_of(state, self::actor(state)).is_some() {
        return legal[usize::from(n) % legal.len()].clone();
    }
    if state.decision(actor).is_some() {
        return GameCommand::Answer(Answer::Decline);
    }
    if state.level_up(actor).is_some() {
        return GameCommand::Answer(Answer::First);
    }
    if state.meeting(actor).is_some() || state.army_dialog(actor).is_some() {
        return pick(&managing(state, rules), n).unwrap_or(GameCommand::Answer(Answer::Decline));
    }
    let heroes = state.adventure().heroes();
    let distance = |mover: HeroId, pos: TilePos| {
        heroes
            .iter()
            .filter(|h| h.owner == actor && h.id != mover && h.pos.level == pos.level)
            .map(|h| h.pos.x.abs_diff(pos.x).max(h.pos.y.abs_diff(pos.y)))
            .min()
    };
    let mut best: Vec<&GameCommand> = Vec::new();
    let mut nearest = u16::MAX;
    for command in legal {
        let Some((hero, path)) = as_move(command) else {
            continue;
        };
        let here = state
            .adventure()
            .hero(hero)
            .and_then(|h| distance(hero, h.pos));
        let there = path.last().and_then(|&to| distance(hero, to));
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
        GameCommand::EndTurn
        | GameCommand::Answer(_)
        | GameCommand::Battle(_)
        | GameCommand::Manage(_) => None,
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
        // Another hero is visited from the tile before: a battle or, an own one, a meeting
        // (GAM-MET-001, E08 T50).
        let heroes = before.adventure().heroes();
        let mover = heroes.iter().find(|h| h.pos == start);
        let other = mover.is_some_and(|m| heroes.iter().any(|h| h.pos == to && h.id != m.id));
        prop_assert!(visitable || other);
    }
    let walked = &path[..steps];
    Ok((walked.last().copied().unwrap_or(start), stays))
}

/// The live object tile bits of `pos` in `state`: whether an object not removed blocks it, and
/// whether one is visited on it (`MapObject::covers`).
fn live_bits(state: &GameState, pos: TilePos) -> (bool, bool) {
    let adventure = state.adventure();
    let objects = adventure.map().objects().iter().enumerate();
    objects
        .filter(|&(i, _)| !adventure.is_removed(i))
        .map(|(_, o)| o.covers(pos))
        .fold((false, false), |(b, v), (ob, ov)| (b || ob, v || ov))
}

/// Whether `pos` is water in `state`'s map.
fn is_water(state: &GameState, rules: &GameRuleset, pos: TilePos) -> bool {
    let tile = state.adventure().map().tile(pos);
    let passage = tile.and_then(|t| rules.adventure().passage(t.terrain));
    matches!(passage, Some(Passage::Water(_)))
}

/// Whether a move that ended on `pos` from `before` ended where a hero may stop (E08 T42;
/// ADV-MOV-007, ADV-MOV-009, ADV-MOV-010): land, and no object blocks it but one visited on it (an
/// entered object, a border gate its key's holder crosses).
fn stops_on(before: &GameState, rules: &GameRuleset, pos: TilePos) -> bool {
    let tile = before.adventure().map().tile(pos);
    let land = tile.and_then(|t| rules.adventure().passage(t.terrain));
    let (blocked, visited) = live_bits(before, pos);
    matches!(land, Some(Passage::Land(_))) && (!blocked || visited)
}

/// The player who acts: the one whose turn is running (sequential play: one at a time), or, while
/// its battle's other side is to act, that side's owner (E08 T19a); while the other side is a
/// monster's, still the player whose battle it is ([`actor`] is then the system); while the game
/// waits for a level-up, its owner (E08 T30).
pub(super) fn acting(state: &GameState) -> PlayerId {
    let player = state.turn_group()[0];
    state.waiting_for().unwrap_or_else(|| {
        state
            .battle(player)
            .and_then(OpenBattle::owner_to_act)
            .unwrap_or(player)
    })
}

/// Who acts (E08 T20b3b1): the system actor while a battle's side to act is a monster's (no
/// player owns it: `GameState::unowned_battle`), else the acting player ([`acting`]).
pub(super) fn actor(state: &GameState) -> Actor {
    if state.waiting_for().is_none() && state.unowned_battle().is_some() {
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
        Step::Manage(n) => (
            who,
            pick(&managing(state, rules), *n)
                .unwrap_or_else(|| legal[usize::from(*n) % legal.len()].clone()),
        ),
        Step::Meet(n) => (who, meet(state, rules, legal, *n)),
        Step::Join(n) => (who, join(state, rules, legal, *n)),
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
                Free::Manage(arbitrary) => {
                    (who, GameCommand::Manage(arbitrary.command(state, actor)))
                }
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

/// The answers to a level-up, as `legal_commands` lists them (GAM-DEC-002, E08 T30).
const LEVEL_ANSWERS: [GameCommand; 2] = [
    GameCommand::Answer(Answer::First),
    GameCommand::Answer(Answer::Second),
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
/// visit from a neighbour) with the points the query reports there, one event per step. A
/// level-up (E08 T30) is a decision too, its answers `First` and `Second`, each of which applies;
/// it holds the game, so its owner acts (in its turn or not: a battle's defender that won) and no
/// one else lists a command.
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
    let levelling = state.level_up(actor).is_some();
    // E08 T51b: a meeting or an army dialog, closed by `Decline` alone.
    let exchanging = state.meeting(actor).is_some() || state.army_dialog(actor).is_some();
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
            let expected = if state.level_up(player).is_some() {
                LEVEL_ANSWERS.to_vec()
            } else if !me {
                Vec::new()
            } else if decided {
                answers.to_vec()
            } else if exchanging {
                vec![GameCommand::Answer(Answer::Decline)]
            } else {
                vec![GameCommand::EndTurn]
            };
            prop_assert_eq!(listed, expected);
        }
        for (hero, reach) in reaches {
            let moves = legal_moves(state, rules, Actor::Player(player), *hero);
            let own = me
                && !decided
                && !exchanging
                && !levelling
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
    if levelling {
        prop_assert_eq!(legal, &LEVEL_ANSWERS[..]);
        for answer in &LEVEL_ANSWERS {
            let mut probe = state.clone();
            let result = apply(
                &mut probe,
                rules,
                Actor::Player(actor),
                answer.clone(),
                &mut Vec::new(),
            );
            prop_assert!(result.is_ok(), "legal {:?} rejected: {:?}", answer, result);
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
    if exchanging {
        let decline = GameCommand::Answer(Answer::Decline);
        prop_assert_eq!(legal, &[decline.clone()][..]);
        let mut probe = state.clone();
        let result = apply(
            &mut probe,
            rules,
            Actor::Player(actor),
            decline,
            &mut Vec::new(),
        );
        prop_assert!(result.is_ok(), "Decline rejected: {:?}", result);
        prop_assert!(!probe.has_decision(actor));
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
    // E08 T52: a management command `apply` accepts is one its hero lists, or a split to a count
    // in its pair's range (GAM-MAN-002).
    if let GameCommand::Manage(manage) = *command {
        let player = Actor::Player(actor);
        let heroes = before.adventure().heroes().iter();
        let listed = heroes
            .clone()
            .any(|h| legal_management(before, rules, player, h.id).contains(&manage));
        let split = match manage {
            Manage::Army(ArmyCommand::Split { from, to, count }) => {
                split_range(before, rules, player, (from, to)).is_some_and(|r| r.contains(&count))
            }
            _ => false,
        };
        prop_assert!(listed || split, "unlisted {:?} accepted", command);
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
/// heroes' details (E08 T19a2a) change only by a battle's end, and (E08 T30) a hero's by the
/// levels it is told it gained answering its level-up ([`check_levels`]: its level and skills,
/// nothing else but the offers' levels). Since E08 T20b3b1 a wandering
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
    for (i, (b, a)) in before.heroes().iter().zip(after.heroes()).enumerate() {
        let id = HeroId::new(u16::try_from(i).unwrap());
        let levelled = events
            .iter()
            .any(|e| matches!(e, GameEvent::LevelGained { hero, .. } if *hero == id));
        // E08 T52: outside a battle a hero's details change by the changes its owner is told: a
        // meeting's spells (HER-SPL-007), an artifact's move, an assembly and a disassembly
        // (HER-ART-005, -006), each applied in order.
        let mut told = (**b).clone();
        for event in events {
            if let GameEvent::HeroChanged {
                player,
                hero,
                change,
            } = *event
                && hero == id
            {
                prop_assert_eq!(Some(player), before.adventure().hero(id).map(|h| h.owner));
                prop_assert!(!levelled, "{:?} levelled and changed", id);
                h3_heroes::apply_change(&mut told, rules.core(), change);
            }
        }
        if levelled {
            check_levels(rules, (before, after), events, id)?;
            prop_assert_eq!(
                (a.experience, a.mana, &a.worn, &a.backpack, &a.spells),
                (b.experience, b.mana, &b.worn, &b.backpack, &b.spells)
            );
        } else {
            prop_assert_eq!(&told, &**a);
        }
    }
    prop_assert_eq!(before.heroes().len(), after.heroes().len());
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
    // E08 T52: a group that finds no room joins nothing and is held whole in the army dialog
    // (ADV-MON-008, GAM-DEC-004), its monster leaving as one that joins.
    let held: Vec<(HeroId, CreatureTypeId, u32)> = events
        .iter()
        .filter_map(|e| match *e {
            GameEvent::ArmyDialogOpened { hero, stack, .. } => {
                Some((hero, stack.creature, stack.count))
            }
            _ => None,
        })
        .collect();
    prop_assert!(joined.len() + held.len() <= 1, "{:?} {:?}", joined, held);
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
                    && let Some(&(_, creature, count)) = joined.first().or(held.first())
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
    // E08 T52: a full army: the group held (the army as it was), none of its creature's stacks
    // able to take it within a stack's most (ADV-ARM-007).
    for &(hero, creature, count) in &held {
        let (_, army) = expected.iter().find(|(id, _)| *id == hero).unwrap();
        let most = rules.adventure().params().max_stack_count();
        let slots = rules.adventure().params().army_slots();
        prop_assert_eq!(army.len(), usize::from(slots), "{:?} held with room", hero);
        prop_assert!(
            army.iter()
                .all(|s| s.creature != creature || s.count.saturating_add(count) > most),
            "{:?} held a group its stack takes",
            hero
        );
    }
    // E08 T52: every slot a management command names, told with what it holds after it
    // (GAM-MAN-001).
    for event in events {
        if let GameEvent::StackChanged {
            at:
                ArmySlot {
                    army: ArmyRef::Hero(hero),
                    slot,
                },
            stack,
            ..
        } = *event
        {
            let (_, army) = expected
                .iter_mut()
                .find(|(id, _)| *id == hero)
                .ok_or_else(|| fail(format!("{event:?} of a hero off the map")))?;
            army.retain(|s| s.slot != slot);
            if let Some(stack) = stack {
                prop_assert_eq!(stack.slot, slot);
                let at = army.iter().position(|s| s.slot > slot);
                army.insert(at.unwrap_or(army.len()), stack);
            }
        }
    }
    prop_assert_eq!(expected, armies(after));
    check_groups(before, after, events)
}

/// The army dialogs' groups (E08 T52; GAM-DEC-004): a dialog opens with the group whole in its
/// slot 0, as told; a slot of a group changes as told (`StackChanged`); a closed dialog's group
/// is lost, each stack it held told in slot order after the dialog's close; nothing else
/// changes a group.
fn check_groups(
    before: &GameState,
    after: &GameState,
    events: &[GameEvent],
) -> Result<(), TestCaseError> {
    for player in (0..before.players()).map(|p| PlayerId::new(u8::try_from(p).unwrap())) {
        let group = |state: &GameState| {
            state
                .army_dialog(player)
                .map(|(hero, group)| (hero, group.to_vec()))
        };
        let mut expected = group(before);
        let mut lost = Vec::new();
        for event in events {
            match *event {
                GameEvent::ArmyDialogOpened {
                    player: owner,
                    hero,
                    stack,
                } if owner == player => {
                    prop_assert!(expected.is_none(), "{:?} with a dialog open", event);
                    prop_assert_eq!(stack.slot, 0);
                    expected = Some((hero, vec![stack]));
                }
                GameEvent::StackChanged {
                    player: owner,
                    at:
                        ArmySlot {
                            army: ArmyRef::Group,
                            slot,
                        },
                    stack,
                } if owner == player => {
                    let (_, group) = expected
                        .as_mut()
                        .ok_or_else(|| fail(format!("{event:?} without a dialog")))?;
                    group.retain(|s| s.slot != slot);
                    if let Some(stack) = stack {
                        prop_assert_eq!(stack.slot, slot);
                        let at = group.iter().position(|s| s.slot > slot);
                        group.insert(at.unwrap_or(group.len()), stack);
                    }
                }
                GameEvent::DecisionClosed { player: owner } if owner == player => {
                    if let Some((_, group)) = expected.take() {
                        lost = group;
                    }
                }
                GameEvent::GroupLost {
                    player: owner,
                    stack,
                    ..
                } if owner == player => {
                    prop_assert!(!lost.is_empty(), "{:?} without a group", event);
                    prop_assert_eq!(lost.remove(0), stack);
                }
                _ => {}
            }
        }
        prop_assert!(
            lost.is_empty(),
            "{:?}'s group not told lost: {:?}",
            player,
            lost
        );
        prop_assert_eq!(group(after), expected);
    }
    Ok(())
}

/// The creatures of `armies`, by creature, as `state` has them (E08 T52).
fn creatures(state: &GameState, armies: &[ArmyRef]) -> Vec<(CreatureTypeId, u64)> {
    let mut all: Vec<(CreatureTypeId, u64)> = Vec::new();
    for &army in armies {
        for slot in 0..7 {
            if let Some(stack) = stack_at(state, ArmySlot { army, slot }) {
                match all.iter_mut().find(|(c, _)| *c == stack.creature) {
                    Some((_, n)) => *n += u64::from(stack.count),
                    None => all.push((stack.creature, u64::from(stack.count))),
                }
            }
        }
    }
    all.sort_unstable();
    all
}

/// The artifacts `heroes` wear and carry, as type and scroll spell, sorted (E08 T52).
fn artifacts(state: &GameState, heroes: &[HeroId]) -> Vec<(u16, Option<u16>)> {
    let mut all: Vec<_> = heroes
        .iter()
        .flat_map(|&id| {
            let d = state.hero(id).unwrap();
            d.worn
                .iter()
                .flatten()
                .chain(&d.backpack)
                .map(|item| (item.artifact.raw(), item.spell.map(|s| s.raw())))
                .collect::<Vec<_>>()
        })
        .collect();
    all.sort_unstable();
    all
}

/// The armies an army command names (E08 T52): the stack's, and the target's if another.
fn named(command: &ArmyCommand) -> Vec<ArmyRef> {
    let (from, to) = match *command {
        ArmyCommand::Move { from, to } | ArmyCommand::Split { from, to, .. } => (from, Some(to)),
        ArmyCommand::Dismiss { at } => (at, None),
    };
    let mut armies = vec![from.army];
    armies.extend(to.map(|to| to.army).filter(|&army| army != from.army));
    armies
}

/// A management command's effects (E08 T52; GAM-MAN-001, ADV-ARM-001 to -006, HER-ART-005,
/// HER-ART-006), beside the told slots and hero changes [`check_conservation`] checks: it opens
/// and closes no decision; no hero moves or loses points; an army command changes only the armies
/// it names, which hold the same creatures but a dismissed stack, which leaves; a formation or a
/// flag changes only its hero's, as told; an artifact's move keeps the artifacts of the heroes it
/// names, an assembly turns its combination's parts into it, a disassembly the combination into
/// its parts.
fn check_management(
    rules: &GameRuleset,
    (before, after): (&GameState, &GameState),
    player: PlayerId,
    command: &Manage,
    events: &[GameEvent],
) -> Result<(), TestCaseError> {
    prop_assert_eq!(before.meeting(player), after.meeting(player));
    prop_assert_eq!(
        before.army_dialog(player).map(|(hero, _)| hero),
        after.army_dialog(player).map(|(hero, _)| hero)
    );
    let armies = match command {
        Manage::Army(army) => named(army),
        _ => Vec::new(),
    };
    for (b, a) in before
        .adventure()
        .heroes()
        .iter()
        .zip(after.adventure().heroes())
    {
        prop_assert_eq!(
            (b.id, b.owner, b.pos, b.movement),
            (a.id, a.owner, a.pos, a.movement)
        );
        if !armies.contains(&ArmyRef::Hero(b.id)) {
            prop_assert_eq!(&b.army, &a.army);
        }
        let formation = events.iter().rev().find_map(|e| match *e {
            GameEvent::FormationSet {
                hero, formation, ..
            } if hero == b.id => Some(formation),
            _ => None,
        });
        let tactics = events.iter().rev().find_map(|e| match *e {
            GameEvent::TacticsSet { hero, on, .. } if hero == b.id => Some(on),
            _ => None,
        });
        prop_assert_eq!(a.formation, formation.unwrap_or(b.formation));
        prop_assert_eq!(a.tactics, tactics.unwrap_or(b.tactics));
    }
    match *command {
        Manage::Army(army) => {
            let mut expected = creatures(before, &armies);
            if let ArmyCommand::Dismiss { at } = army {
                let stack = stack_at(before, at).unwrap();
                let (_, n) = expected
                    .iter_mut()
                    .find(|(c, _)| *c == stack.creature)
                    .unwrap();
                *n -= u64::from(stack.count);
                expected.retain(|&(_, n)| n > 0);
            }
            prop_assert_eq!(creatures(after, &armies), expected);
        }
        Manage::Formation { hero, formation } => {
            let told = GameEvent::FormationSet {
                player,
                hero,
                formation,
            };
            prop_assert_eq!(events, &[told][..]);
        }
        Manage::Tactics { hero, on } => {
            prop_assert_eq!(events, &[GameEvent::TacticsSet { player, hero, on }][..]);
        }
        Manage::Artifact { from, to } => {
            let heroes = if from.0 == to.0 {
                vec![from.0]
            } else {
                vec![from.0, to.0]
            };
            prop_assert_eq!(artifacts(after, &heroes), artifacts(before, &heroes));
        }
        Manage::Assemble { hero, .. } | Manage::Disassemble { hero, .. } => {
            let assembled = matches!(command, Manage::Assemble { .. });
            let (whole, parted) = if assembled {
                (after, before)
            } else {
                (before, after)
            };
            // The one artifact the whole side has more: the combination, whose parts the parted
            // side has instead.
            let mut gained = artifacts(whole, &[hero]);
            let mut expected = artifacts(parted, &[hero]);
            for item in artifacts(parted, &[hero]) {
                if let Some(i) = gained.iter().position(|&i| i == item) {
                    gained.remove(i);
                }
            }
            prop_assert_eq!(gained.len(), 1, "{:?}", command);
            let combination = ArtifactTypeId::from_raw(gained[0].0);
            let parts = &rules.core().artifact(combination).unwrap().parts;
            prop_assert!(!parts.is_empty(), "{:?}", combination);
            for part in parts {
                let i = expected.iter().position(|&(a, _)| a == part.raw());
                expected.remove(i.ok_or_else(|| fail(format!("{part:?} missing")))?);
            }
            expected.push(gained[0]);
            expected.sort_unstable();
            prop_assert_eq!(artifacts(whole, &[hero]), expected);
        }
    }
    Ok(())
}

/// A move and its inverse (E08 T52; increment 6's design note §12 in `E08-log.md`): a stack moved
/// onto an empty slot or swapped with another creature's stack, or an artifact moved onto an empty
/// place, goes back by the inverse move, which `apply` accepts, leaving the armies and the heroes
/// as they were. A merge's inverse is a split, and a swap of artifacts out of a backpack returns
/// the other artifact to the backpack: those have none.
fn check_inverse(
    rules: &GameRuleset,
    (before, after): (&GameState, &GameState),
    player: PlayerId,
    command: &Manage,
) -> Result<bool, TestCaseError> {
    let inverse = match *command {
        Manage::Army(ArmyCommand::Move { from, to }) => {
            let (moved, there) = (stack_at(before, from), stack_at(before, to));
            if there.is_some_and(|t| moved.is_some_and(|m| m.creature == t.creature)) {
                return Ok(false);
            }
            Manage::Army(ArmyCommand::Move { from: to, to: from })
        }
        Manage::Artifact { from, to } => {
            let target = before.hero(to.0).unwrap();
            if let ArtifactPlace::Worn(q) = to.1
                && target.worn[q.0].is_some()
            {
                return Ok(false);
            }
            Manage::Artifact { from: to, to: from }
        }
        _ => return Ok(false),
    };
    let mut back = after.clone();
    let result = apply(
        &mut back,
        rules,
        Actor::Player(player),
        GameCommand::Manage(inverse),
        &mut Vec::new(),
    );
    prop_assert!(
        result.is_ok(),
        "{:?} undone by {:?}: {:?}",
        command,
        inverse,
        result
    );
    prop_assert_eq!(armies(&back), armies(before));
    prop_assert_eq!(back.heroes(), before.heroes());
    prop_assert_eq!(
        back.army_dialog(player).map(|(_, g)| g.to_vec()),
        before.army_dialog(player).map(|(_, g)| g.to_vec())
    );
    Ok(true)
}

/// The invariants of every state (E08 T52; ADV-ARM-002, -004, -007, HER-ART-001): every hero on
/// the map has one to seven stacks, in distinct slots in order, each of at least one creature and
/// at most a stack's most; every hero's details are valid; an army dialog's group has at most
/// seven stacks alike (none: all taken) and holds a hero of its player on the map; a meeting holds
/// two of the player's heroes on the map, neighbours on one level.
fn check_invariants(rules: &GameRuleset, state: &GameState) -> Result<(), TestCaseError> {
    let params = rules.adventure().params();
    let (slots, most) = (params.army_slots(), params.max_stack_count());
    let valid = |army: &[h3_contracts::ArmyStack]| {
        army.windows(2).all(|w| w[0].slot < w[1].slot)
            && army
                .iter()
                .all(|s| s.slot < slots && (1..=most).contains(&s.count))
    };
    for hero in state.adventure().heroes() {
        prop_assert!(!hero.army.is_empty() && valid(&hero.army), "{:?}", hero);
        let details = state.hero(hero.id).unwrap();
        let checked = h3_heroes::validate(details, rules.core());
        prop_assert!(checked.is_ok(), "{:?}: {:?}", hero.id, checked);
    }
    for player in (0..state.players()).map(|p| PlayerId::new(u8::try_from(p).unwrap())) {
        let own = |hero: HeroId| {
            state
                .adventure()
                .hero(hero)
                .filter(|h| h.owner == player)
                .map(|h| h.pos)
        };
        if let Some((hero, group)) = state.army_dialog(player) {
            prop_assert!(valid(group), "{:?}", group);
            prop_assert!(own(hero).is_some());
        }
        if let Some([a, b]) = state.meeting(player) {
            let (a, b) = (own(a).unwrap(), own(b).unwrap());
            prop_assert_eq!(a.level, b.level);
            prop_assert_eq!(a.x.abs_diff(b.x).max(a.y.abs_diff(b.y)), 1);
        }
    }
    Ok(())
}

/// Up to `most` of `items`, evenly spread from an offset `seed` picks (E08 T52: the probes of a
/// long listing, a different sample at every state).
fn spread<T>(items: &[T], most: usize, seed: u64) -> impl Iterator<Item = &T> {
    let stride = items.len().div_ceil(most).max(1);
    let offset = usize::try_from(seed % u64::try_from(stride).unwrap()).unwrap();
    items.iter().skip(offset).step_by(stride)
}

/// Management's legality consistency, before a step (E08 T52; GAM-MAN-002, ADR-0017 point 6):
/// only the acting player lists management commands, and only while it may manage ([`managed`]):
/// for each of its heroes, or the heroes its meeting or army dialog holds; each hero lists both
/// formations and both values of the flag and no command twice, and every command listed applies
/// (a sample of a long listing). A hero not on the map, another player's, and the system list
/// none. A pair of slots of the armies managed together has a split range exactly when a split
/// applies there: its ends apply, a count past either is refused, and a pair without one refuses
/// a split of one creature; another player has none.
fn check_managing(state: &GameState, rules: &GameRuleset) -> Result<(), TestCaseError> {
    let player = acting(state);
    let managed = managed(state);
    let held: Vec<HeroId> = managed
        .iter()
        .flatten()
        .filter_map(|&army| match army {
            ArmyRef::Hero(hero) => Some(hero),
            ArmyRef::Group => None,
        })
        .collect();
    let seed = state.state_hash();
    let probe = |command: Manage| {
        let mut probe = state.clone();
        let command = GameCommand::Manage(command);
        apply(
            &mut probe,
            rules,
            Actor::Player(player),
            command,
            &mut Vec::new(),
        )
    };
    let ids = (0..=u16::try_from(state.heroes().len()).unwrap()).map(HeroId::new);
    for id in ids {
        prop_assert!(legal_management(state, rules, Actor::System, id).is_empty());
        for p in (0..=state.players()).map(|p| PlayerId::new(u8::try_from(p).unwrap())) {
            let listed = legal_management(state, rules, Actor::Player(p), id);
            let lists = p == player && held.contains(&id);
            prop_assert_eq!(
                !listed.is_empty(),
                lists,
                "{:?} for {:?}: {:?}",
                id,
                p,
                listed
            );
            if !lists {
                continue;
            }
            for formation in [Formation::Loose, Formation::Tight] {
                let set = Manage::Formation {
                    hero: id,
                    formation,
                };
                prop_assert!(listed.contains(&set), "{:?} unlisted", set);
            }
            for on in [true, false] {
                let set = Manage::Tactics { hero: id, on };
                prop_assert!(listed.contains(&set), "{:?} unlisted", set);
            }
            for (i, command) in listed.iter().enumerate() {
                prop_assert!(!listed[..i].contains(command), "{:?} twice", command);
            }
            for &command in spread(&listed, 24, seed) {
                let result = probe(command);
                prop_assert!(
                    result.is_ok(),
                    "listed {:?} rejected: {:?}",
                    command,
                    result
                );
            }
        }
    }
    let other = PlayerId::new((player.raw() + 1) % u8::try_from(state.players()).unwrap());
    let mut pairs = Vec::new();
    for armies in &managed {
        let slots: Vec<ArmySlot> = armies
            .iter()
            .flat_map(|&army| (0..7).map(move |slot| ArmySlot { army, slot }))
            .collect();
        for &from in slots.iter().filter(|&&at| stack_at(state, at).is_some()) {
            pairs.extend(slots.iter().map(|&to| (from, to)));
        }
    }
    for &(from, to) in spread(&pairs, 16, seed) {
        let range = split_range(state, rules, Actor::Player(player), (from, to));
        if other != player {
            prop_assert_eq!(
                split_range(state, rules, Actor::Player(other), (from, to)),
                None
            );
        }
        let split = |count| probe(Manage::Army(ArmyCommand::Split { from, to, count })).is_ok();
        match range {
            Some(range) => {
                let (low, high) = (*range.start(), *range.end());
                prop_assert!(split(low) && split(high), "{:?} {:?} {:?}", from, to, range);
                prop_assert!(
                    low == 0 || !split(low - 1),
                    "{:?} {:?} {:?}",
                    from,
                    to,
                    range
                );
                prop_assert!(!split(high + 1), "{:?} {:?} {:?}", from, to, range);
            }
            None => prop_assert!(!split(1), "{:?} {:?} split without a range", from, to),
        }
    }
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
/// received and lost, its level, primary and secondary skills by the levels it is told it gained
/// (E08 T30: only one that stays on the map, each the next level), and nothing else of it
/// changes; the two heroes' artifacts only move between them or leave with a destroyed war
/// machine.
/// The levels `hero` is told it gained (E08 T30, GAM-DEC-002): each by its owner, of a hero on
/// the map, the next level, raising the primary skill told (up to the ruleset's maximum) and
/// learning or advancing the skill told; its level, primary and secondary skills are what they
/// give, its type as before.
fn check_levels(
    rules: &GameRuleset,
    (before, after): (&GameState, &GameState),
    events: &[GameEvent],
    id: HeroId,
) -> Result<(), TestCaseError> {
    let (b, a) = (before.hero(id).unwrap(), after.hero(id).unwrap());
    let (mut level, mut primary, mut skills) = (b.level, b.primary, b.skills.clone());
    let max = rules.core().hero_params().unwrap().primary_max;
    for event in events {
        if let GameEvent::LevelGained {
            player,
            hero,
            level: gained,
            primary: raised,
            skill,
        } = *event
        {
            if hero != id {
                continue;
            }
            prop_assert!(after.adventure().hero(id).is_some(), "{:?} levelled", id);
            prop_assert_eq!(player, before.adventure().hero(id).unwrap().owner);
            prop_assert_eq!(gained, level + 1);
            level = gained;
            let base = &mut primary[raised.index()];
            *base = base.saturating_add(1).min(max);
            if let Some(skill) = skill {
                match skills.iter_mut().find(|(s, _)| *s == skill) {
                    Some((_, l)) => *l += 1,
                    None => skills.push((skill, 1)),
                }
            }
        }
    }
    prop_assert_eq!(
        (a.hero_type, a.level, a.primary, &a.skills),
        (b.hero_type, level, primary, &skills)
    );
    Ok(())
}

/// The heroes' levels by their experience, in every state the plans reach (E08 T31, design §14;
/// HER-EXP-002, HER-EXP-003, GAM-DEC-002), against an oracle of the ruleset's thresholds (a
/// level is the number of thresholds at or below the experience, at most the highest level): no
/// hero's level is above its experience's, nor its experience above the bound (the threshold after
/// the highest level, less one) or the scenario's level limit's threshold (or its own level's,
/// if higher); a hero has a pending level only while a level-up of its owner is open for it, of
/// the level after its own. At most one level-up is open, its hero on the map, and the game waits
/// for its owner (ADR-0017 point 6) and for no one else.
fn check_progression(rules: &GameRuleset, state: &GameState) -> Result<(), TestCaseError> {
    let params = rules.core().hero_params().unwrap();
    let thresholds = &params.experience;
    let max = params.max_level;
    let waiting = state.waiting_for();
    let open = waiting.and_then(|player| state.level_up(player));
    prop_assert_eq!(
        waiting.is_some(),
        open.is_some(),
        "waiting for {:?}",
        waiting
    );
    for player in (0..state.players()).map(|p| PlayerId::new(u8::try_from(p).unwrap())) {
        prop_assert!(Some(player) == waiting || state.level_up(player).is_none());
    }
    if let Some(view) = open {
        let owner = state.adventure().hero(view.hero).map(|h| h.owner);
        prop_assert_eq!(owner, waiting, "{:?}", view);
    }
    for (index, details) in state.heroes().iter().enumerate() {
        let id = HeroId::new(u16::try_from(index).unwrap());
        let (level, experience) = (details.level, details.experience);
        let reached = thresholds.iter().filter(|&&t| t <= experience).count();
        let reached = u8::try_from(reached).unwrap().min(max);
        prop_assert!(
            experience <= experience_cap(rules, state, level),
            "{:?}: {} experience",
            id,
            experience
        );
        prop_assert!(
            level <= reached,
            "{:?}: level {} at {} experience",
            id,
            level,
            experience
        );
        match open.filter(|view| view.hero == id) {
            Some(view) => {
                prop_assert!(reached > level, "{:?} without a pending level", view);
                prop_assert_eq!(view.level, level + 1);
            }
            None => prop_assert_eq!(
                reached,
                level,
                "{:?}: a pending level at {} experience",
                id,
                experience
            ),
        }
    }
    Ok(())
}

/// The most experience a hero at `level` may have in `state` (HER-EXP-002, HER-EXP-003; E08 T31,
/// an oracle of the ruleset's thresholds): the threshold of the scenario's level limit, or without
/// one the bound (the threshold after the highest level, less one); its own level's if higher.
fn experience_cap(rules: &GameRuleset, state: &GameState, level: u8) -> u32 {
    let params = rules.core().hero_params().unwrap();
    let threshold = |level: u8| params.experience[usize::from(level.max(1)) - 1];
    let (limit, max) = (state.level_limit(), params.max_level);
    let cap = if (1..=max).contains(&limit) {
        threshold(limit)
    } else {
        threshold(max + 1) - 1
    };
    cap.max(threshold(level))
}

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
        let (mut experience, mut mana, mut items) = (
            b.experience,
            b.mana,
            b.worn.iter().flatten().count() + b.backpack.len(),
        );
        let mut spells = b.spells.clone();
        for change in told {
            match change {
                // E08 T31: under the bound and the scenario's level limit (HER-EXP-002, -003).
                HeroChange::GainExperience(points) => {
                    prop_assert!(after.adventure().hero(id).is_some(), "{:?} gained", id);
                    let cap = experience_cap(rules, before, b.level);
                    experience = experience.saturating_add(points).min(cap);
                }
                HeroChange::SetSpellPoints(points) => mana = points,
                HeroChange::AddArtifact(_) => items += 1,
                HeroChange::RemoveArtifact(_) => items -= 1,
                // E08 T19a2b: only the winner that stays learns (Eagle Eye's picks).
                HeroChange::LearnSpell(spell) => {
                    prop_assert!(after.adventure().hero(id).is_some(), "{:?} learned", id);
                    spells.push(spell);
                }
                // E08 T27: a battle tells no level; the game gives them (T30).
                HeroChange::GainLevel { .. } => prop_assert!(false, "{:?} told a level", id),
                // E08 T49a: nor a placed artifact (a move's).
                HeroChange::PlaceArtifact { .. } => prop_assert!(false, "{:?} told a place", id),
            }
        }
        prop_assert_eq!((a.experience, a.mana), (experience, mana));
        prop_assert_eq!(a.worn.iter().flatten().count() + a.backpack.len(), items);
        prop_assert_eq!(&a.spells, &spells);
        check_levels(rules, (before, after), events, id)?;
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
                    .flatten()
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
                    // E08 T52: or, without room, the group held in the army dialog.
                    (true, OfferKind::Join | OfferKind::JoinForGold { .. }) => {
                        let joined = events.iter().any(|e| {
                            matches!(
                                e,
                                GameEvent::Adventure(AdventureEvent {
                                    kind: AdventureEventKind::CreaturesJoined { .. },
                                    ..
                                }) | GameEvent::ArmyDialogOpened { .. }
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
    // E08 T52: a meeting of two of the actor's heroes opens as told (a step toward another of its
    // heroes, GAM-MET-001), an army dialog as told (an accepted join without room), each when
    // neither is open, and each closes as told (its `Decline`; GAM-DEC-003, GAM-DEC-004).
    let dialog = |state: &GameState| state.army_dialog(actor).map(|(hero, _)| hero);
    let mut exchange = (before.meeting(actor), dialog(before));
    for event in events {
        match *event {
            GameEvent::MeetingOpened { player, heroes } => {
                prop_assert_eq!((player, exchange), (actor, (None, None)));
                for hero in heroes {
                    let owner = after.adventure().hero(hero).map(|h| h.owner);
                    prop_assert_eq!(owner, Some(actor));
                }
                exchange.0 = Some(heroes);
            }
            GameEvent::ArmyDialogOpened { player, hero, .. } => {
                prop_assert_eq!((player, exchange), (actor, (None, None)));
                exchange.1 = Some(hero);
            }
            GameEvent::DecisionClosed { player } => {
                prop_assert_eq!(player, actor);
                prop_assert!(exchange != (None, None), "{:?} with nothing open", event);
                exchange = (None, None);
            }
            _ => {}
        }
    }
    prop_assert_eq!((after.meeting(actor), dialog(after)), exchange);
    prop_assert_eq!(after.decision(actor), decision.as_ref());
    prop_assert!(after.adventure().bought(actor).eq(bought));
    prop_assert_eq!(after.adventure().keys(actor), keys);
    Ok(())
}

/// Ownership isolation: a command of `actor` moves no other player's hero, changes no other
/// player's army, explored tiles (but for a new day's reveal in strict-fixed, which only adds,
/// E08 T40), treasury, purchases, open decision (E08 T14b) or keys (T15a),
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
    // A new day's start (E08 T40): in strict-fixed (`ADV-QUIRK-18` off) every hero reveals around
    // it to its owner, so another player's explored tiles may grow then, never shrink.
    let new_day = events
        .iter()
        .any(|e| matches!(e, GameEvent::DayStarted { .. }));
    let kept = |was: &h3_adventure::BitGrid, is: &h3_adventure::BitGrid| {
        (0..was.levels()).all(|z| {
            (0..was.height()).all(|y| {
                (0..was.width()).all(|x| {
                    let pos = TilePos::new(x, y, z);
                    !was.get(pos) || is.get(pos)
                })
            })
        })
    };
    for player in (0..before.players()).map(|p| PlayerId::new(u8::try_from(p).unwrap())) {
        if player != actor {
            let (was, is) = (
                before.adventure().explored(player),
                after.adventure().explored(player),
            );
            match (was, is) {
                (Some(was), Some(is)) if new_day => prop_assert!(kept(was, is)),
                _ => prop_assert!(was == is),
            }
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
            // E08 T52: nor its meeting or army dialog.
            prop_assert_eq!(before.meeting(player), after.meeting(player));
            prop_assert_eq!(before.army_dialog(player), after.army_dialog(player));
            // E08 T30: a level-up opens for the other owner of the battle the command ended.
            if !partners.contains(&player) {
                prop_assert_eq!(before.level_up(player), after.level_up(player));
            }
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
    /// random play on imported maps, in both profiles; since E08 T31 the heroes' levels by their
    /// experience in every state ([`check_progression`]), in half the cases with heroes of a
    /// custom experience ([`experience`]: levels at load, and level-ups in play from the first
    /// battle).
    #[test]
    fn every_step_keeps_the_commands_invariants(
        plan in plan(),
        strict in any::<bool>(),
        seed in any::<u64>(),
        steps in steps(),
        experience in proptest::option::weighted(0.5, vec(experience(), 6)),
    ) {
        let rules = if strict { &*STRICT } else { &*FAITHFUL };
        let mut state = start_on(&experienced(&plan, experience.as_deref()), rules, seed);
        for step in &steps {
            check_progression(rules, &state)?;
            check_invariants(rules, &state)?;
            let legal = options(&state, rules);
            let reaches = reaches(&state, rules);
            check_legal(&state, rules, &legal, &reaches)?;
            check_managing(&state, rules)?;
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
                // E08 T42 (increment 5's exit criterion 3): a flight or a water walk passes over
                // water and blocked tiles but ends where a hero may stop, as a walk does.
                prop_assert!(stops_on(&before, rules, at), "{:?} ends on {:?}", hero, at);
            }
            let player = player_of(&before, actor);
            check_decisions(rules, (&before, &state), player, &events)?;
            check_isolation(&before, &state, player, &events)?;
            check_conservation(rules, &before, &state, &events)?;
            // E08 T52: management (GAM-MAN-001), and a move's inverse.
            if let GameCommand::Manage(manage) = &command {
                check_management(rules, (&before, &state), player, manage, &events)?;
                check_inverse(rules, (&before, &state), player, manage)?;
            }
        }
        check_progression(rules, &state)?;
        check_invariants(rules, &state)?;
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
/// T21a) the guarded piles' questions and battles, and how those ended; (E08 T31) the level-ups
/// opened and the levels gained in play.
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
    level_ups: usize,
    levels: usize,
    /// The visitors of the guarded piles' battles in progress.
    at_piles: Vec<HeroId>,
}

impl Fought {
    /// Counts `events`, which a command emitted on its way to `after`.
    fn count(&mut self, rules: &GameRuleset, after: &GameState, events: &[GameEvent]) {
        let part = rules.adventure();
        for event in events {
            match *event {
                // E08 T31: the levels in play, opened as a decision or given at once.
                GameEvent::LevelUpOpened { .. } => self.level_ups += 1,
                GameEvent::LevelGained { .. } => self.levels += 1,
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
/// E08 T21a guarded piles too: their questions, and their battles won and lost. Since E08 T31 the
/// winners' level-ups too, opened and answered, and levels given at once, the heroes' levels
/// checked after every command ([`check_progression`]).
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
            check_progression(rules, &state).unwrap();
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
    assert!(f.level_ups > 0 && f.levels > 0, "{f:?}");
}

/// What a set of games moved (E08 T42): the games with a hero wearing each of [`MOVERS`] and with
/// each of Pathfinding, Logistics and Scouting at the start; the accepted moves of a flyer over
/// water and over a tile an object blocks, of a water walker over water, and of a hero whose army
/// holds Nomads; the moves refused with `NoStop` (a path that ends where its hero may only pass).
#[derive(Debug, Default)]
struct Moved {
    wearing: [usize; 6],
    skilled: [usize; 3],
    flown_over_water: usize,
    flown_over_blocked: usize,
    walked_on_water: usize,
    with_nomads: usize,
    no_stop: usize,
}

impl Moved {
    /// Counts what the heroes of a game's first state wear and know.
    fn start(&mut self, rules: &GameRuleset, state: &GameState) {
        let core = rules.core();
        for (i, &(code, _)) in MOVERS.iter().enumerate() {
            let key = codes::ARTIFACTS[usize::from(code)];
            let worn = |d: &h3_heroes::HeroDetails| {
                d.worn.iter().flatten().any(|a| {
                    core.artifact(a.artifact)
                        .is_some_and(|ty| ty.key.as_str() == key)
                })
            };
            self.wearing[i] += usize::from(state.heroes().iter().any(|d| worn(d)));
        }
        for (i, code) in [0usize, 2, 3].into_iter().enumerate() {
            let skill = core.skill_by_key(codes::SKILLS[code]);
            let known =
                |d: &h3_heroes::HeroDetails| d.skills.iter().any(|&(s, _)| Some(s) == skill);
            self.skilled[i] += usize::from(state.heroes().iter().any(|d| known(d)));
        }
    }

    /// Counts the command `command` of `before`, accepted or refused (`result`).
    fn count(
        &mut self,
        rules: &GameRuleset,
        before: &GameState,
        command: &GameCommand,
        result: &Result<(), GameCommandError>,
    ) {
        let Some((hero, path)) = as_move(command) else {
            return;
        };
        if let Err(GameCommandError::Adventure(AdventureCommandError::NoStop { .. })) = result {
            self.no_stop += 1;
        }
        let (Ok(()), Some(details), Some(on_map)) =
            (result, before.hero(hero), before.adventure().hero(hero))
        else {
            return;
        };
        let movement = h3_heroes::movement(details, rules.core());
        let passed = &path[..path.len().saturating_sub(1)];
        let over_water = passed.iter().any(|&pos| is_water(before, rules, pos));
        if movement.flying.is_some() {
            self.flown_over_water += usize::from(over_water);
            let blocked = passed.iter().any(|&pos| live_bits(before, pos).0);
            self.flown_over_blocked += usize::from(blocked);
        } else if movement.water_walking.is_some() {
            self.walked_on_water += usize::from(over_water);
        }
        let nomads = codes::CREATURES
            .iter()
            .find(|&&(code, _)| code == NOMADS)
            .and_then(|&(_, key)| rules.core().creature_by_key(key));
        let holds = on_map
            .army
            .iter()
            .any(|stack| Some(stack.creature) == nomads);
        self.with_nomads += usize::from(holds);
    }
}

/// The plans fly and walk on water (E08 T42, increment 5's exit criterion 3): over a fixed set of
/// cases (proptest's deterministic runner), heroes wear each of the six artifacts and know each of
/// the three skills, and random play moves flyers over water and over tiles objects block, water
/// walkers over water and heroes with Nomads, and refuses paths that end where their heroes may
/// only pass (`NoStop`); every accepted move ends where its hero may stop
/// (`every_step_keeps_the_commands_invariants`).
// covers: ADV-MOV-007, ADV-MOV-009, ADV-MOV-010
#[test]
fn the_plans_fly_and_walk_on_water() {
    use proptest::strategy::ValueTree;
    let mut runner = proptest::test_runner::TestRunner::deterministic();
    let strategy = (plan(), any::<bool>(), any::<u64>(), steps());
    let mut moved = Moved::default();
    for _ in 0..64 {
        let (plan, strict, seed, steps) = strategy.new_tree(&mut runner).unwrap().current();
        let rules = if strict { &*STRICT } else { &*FAITHFUL };
        let mut state = start(&plan, rules, seed);
        moved.start(rules, &state);
        for step in &steps {
            let legal = options(&state, rules);
            let (actor, command) = resolve(&state, rules, &legal, step);
            let before = state.clone();
            let result = apply(&mut state, rules, actor, command.clone(), &mut Vec::new());
            moved.count(rules, &before, &command, &result);
        }
    }
    let m = &moved;
    assert!(m.wearing.iter().chain(&m.skilled).all(|&n| n > 0), "{m:?}");
    assert!(m.flown_over_water > 0 && m.flown_over_blocked > 0, "{m:?}");
    assert!(
        m.walked_on_water > 0 && m.with_nomads > 0 && m.no_stop > 0,
        "{m:?}"
    );
}

/// What a set of games managed (E08 T52): the meetings opened and the spells Scholar taught at
/// them; the accepted management commands by kind, those between a meeting's two heroes (stacks
/// and artifacts), the group's stacks taken into a hero's army; the army dialogs opened and the
/// stacks lost with their groups; the management commands refused; the moves undone by their
/// inverse ([`check_inverse`]).
#[derive(Debug, Default)]
struct Managed {
    meetings: usize,
    taught: usize,
    moves: usize,
    splits: usize,
    dismissals: usize,
    formations: usize,
    flags: usize,
    artifacts: usize,
    assemblies: usize,
    disassemblies: usize,
    stacks_exchanged: usize,
    artifacts_exchanged: usize,
    dialogs: usize,
    taken: usize,
    lost: usize,
    refused: usize,
    undone: usize,
}

impl Managed {
    /// Counts an applied command and its events.
    fn count(&mut self, command: &GameCommand, accepted: bool, events: &[GameEvent]) {
        let meeting = events
            .iter()
            .any(|e| matches!(e, GameEvent::MeetingOpened { .. }));
        for event in events {
            match *event {
                GameEvent::MeetingOpened { .. } => self.meetings += 1,
                GameEvent::HeroChanged {
                    change: HeroChange::LearnSpell(_),
                    ..
                } if meeting => self.taught += 1,
                GameEvent::ArmyDialogOpened { .. } => self.dialogs += 1,
                GameEvent::GroupLost { .. } => self.lost += 1,
                _ => {}
            }
        }
        let GameCommand::Manage(manage) = command else {
            return;
        };
        if !accepted {
            self.refused += 1;
            return;
        }
        let hero = |army: ArmyRef| matches!(army, ArmyRef::Hero(_));
        match *manage {
            Manage::Army(army) => {
                match army {
                    ArmyCommand::Move { .. } => self.moves += 1,
                    ArmyCommand::Split { .. } => self.splits += 1,
                    ArmyCommand::Dismiss { .. } => self.dismissals += 1,
                }
                let armies = named(&army);
                self.stacks_exchanged +=
                    usize::from(armies.len() == 2 && armies.iter().all(|&a| hero(a)));
                self.taken += usize::from(armies.len() == 2 && armies[0] == ArmyRef::Group);
            }
            Manage::Formation { .. } => self.formations += 1,
            Manage::Tactics { .. } => self.flags += 1,
            Manage::Artifact { from, to } => {
                self.artifacts += 1;
                self.artifacts_exchanged += usize::from(from.0 != to.0);
            }
            Manage::Assemble { .. } => self.assemblies += 1,
            Manage::Disassemble { .. } => self.disassemblies += 1,
        }
    }
}

/// The plans meet and manage (E08 T52, increment 6's exit criteria 1 to 4): over a fixed set of
/// cases (proptest's deterministic runner), random play with the management runs ([`steps`])
/// meets a hero's mate and learns Scholar's spells there, moves, splits and dismisses stacks,
/// exchanges stacks and artifacts between the two heroes, sets formations and flags, assembles
/// and disassembles Elixir of Life, opens army dialogs for a full army's join, takes the group's
/// stacks and loses the rest, refuses arbitrary management commands and undoes moves by their
/// inverse; the properties above check each (`every_step_keeps_the_commands_invariants`).
// covers: GAM-MAN-001, GAM-MAN-002, GAM-MET-001, GAM-DEC-003, GAM-DEC-004, HER-SPL-007
#[test]
fn the_plans_meet_and_manage() {
    use proptest::strategy::ValueTree;
    let mut runner = proptest::test_runner::TestRunner::deterministic();
    let strategy = (plan(), any::<bool>(), any::<u64>(), steps());
    let mut managed = Managed::default();
    for _ in 0..96 {
        let (plan, strict, seed, steps) = strategy.new_tree(&mut runner).unwrap().current();
        let rules = if strict { &*STRICT } else { &*FAITHFUL };
        let mut state = start(&plan, rules, seed);
        for step in &steps {
            let legal = options(&state, rules);
            let (actor, command) = resolve(&state, rules, &legal, step);
            let before = state.clone();
            let mut events = Vec::new();
            let result = apply(&mut state, rules, actor, command.clone(), &mut events);
            managed.count(&command, result.is_ok(), &events);
            if let (Ok(()), GameCommand::Manage(manage)) = (result, &command) {
                let player = player_of(&before, actor);
                let undone = check_inverse(rules, (&before, &state), player, manage).unwrap();
                managed.undone += usize::from(undone);
            }
        }
    }
    let m = &managed;
    assert!(m.meetings > 0 && m.taught > 0, "{m:?}");
    assert!(m.moves > 0 && m.splits > 0 && m.dismissals > 0, "{m:?}");
    assert!(m.formations > 0 && m.flags > 0 && m.artifacts > 0, "{m:?}");
    assert!(m.assemblies > 0 && m.disassemblies > 0, "{m:?}");
    assert!(m.stacks_exchanged > 0 && m.artifacts_exchanged > 0, "{m:?}");
    assert!(m.dialogs > 0 && m.taken > 0 && m.lost > 0, "{m:?}");
    assert!(m.refused > 0 && m.undone > 0, "{m:?}");
}

/// The draws' source by the profile, on the original packs (E08 T31, design §14; GAM-HER-002,
/// GAM-SET-002, `ADV-QUIRK-9`): in `faithful` a hero's levels at load come from its own level seed
/// and the level (the original's generator), so the levels another hero gains at load before it
/// (in hero id order) leave its own as they are; in `strict-fixed` they come from the setup stream,
/// which the other hero's levels draw from first, so they change with those for some seeds.
// covers: GAM-HER-002, GAM-SET-002
#[test]
fn the_levels_at_load_follow_the_seed_in_faithful_and_the_stream_in_strict_fixed() {
    let plan = Plan {
        version: Version::RoE,
        side: 8,
        two_levels: false,
        tiles: vec![(0, 0, 0); 64],
        players: vec![(0, true), (1, true)],
        heroes: vec![((1, 1, 0), None), ((6, 6, 0), None)],
        hero_types: 0,
        backpacks: vec![],
        skills: vec![],
        kits: vec![],
        nomads: None,
        blockers: vec![],
        difficulty: 1,
        exchange: None,
    };
    // The second hero's levels at load, at 12 000 experience (level 8), after the first hero's
    // at `first` experience.
    let second = |rules: &GameRuleset, seed: u64, first: u32| {
        let game = start_on(&experienced(&plan, Some(&[first, 12_000])), rules, seed);
        let details = game.hero(HeroId::new(1)).unwrap();
        assert_eq!(details.level, 8);
        (details.primary, details.skills.clone())
    };
    let mut changed = 0;
    for seed in 0..16 {
        // At 1 experience the first hero gains no level (0 would be RoE's default, drawn from
        // the setup stream before the seeds), at 20 000 levels 2 to 11.
        assert_eq!(
            second(&FAITHFUL, seed, 1),
            second(&FAITHFUL, seed, 20_000),
            "seed {seed}"
        );
        changed += usize::from(second(&STRICT, seed, 1) != second(&STRICT, seed, 20_000));
    }
    assert!(changed > 0);
}
