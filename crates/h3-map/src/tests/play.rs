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
//! effects and a move's inverse (`the_plans_meet_and_manage`); since T64b with increment 7's
//! objects, one of each class and guarded ones ([`REWARDS`]): their decisions answered, their
//! gains conserved, and the invariants of the visits' state ([`check_rewards`],
//! `the_plans_visit_every_kind_and_answer_every_decision`); since T86b1 with increment 9's objects
//! ([`ECONOMY`]) and weeks and months of play: the flags, the mines' questions, battles and Leave
//! Guards, the boats bought and built, the weekly visits, the week's step and a creature month's
//! monsters met as placed ones (`the_plans_flag_guard_build_and_grow`); since T96b1 with increment
//! 10's dwellings and hiring (`the_plans_visit_dwellings_and_hire`); since E09 T9b1 with towns
//! ([`TownPlan`], [`Step::Town`]): own towns entered, undefended ones captured, sieges against a
//! garrison and against a visitor with the garrison merged, won and lost, the Capitol's limit, and
//! the towns' invariants ([`check_towns`], the entrances in [`check_invariants`], the store of
//! towns in the forks; `the_plans_capture_and_besiege_towns`). What a viewer may learn is
//! T8b2b's ([`super::view`]), on the same maps and driver (without the towns until E09 T9b2).

use std::ops::RangeInclusive;
use std::sync::{Arc, LazyLock};

use h3_adventure::{
    AdventureCommand, AdventureCommandError, AdventureEvent, AdventureEventKind, Gift, Holds,
    ObjectKind, ObjectTypeId, Offer, OfferKind, Passage, Pos as TilePos, Reach, Resolved,
    RewardRecord, Turn, Viewers,
};
use h3_battle::{
    CastTargets, Command, Event, EventKind, cast_targets, castable_spells, second_targets,
};
use h3_content::{ArtifactTypeId, CreatureFlag, CreatureTypeId, Formation, ResourceId};
use h3_contracts::{BattleResult, BattleSide, HeroId, PlayerId, TownId};
use h3_game::{
    Actor, Answer, ArenaChoice, ArmyCommand, ArmyRef, ArmySlot, ArtifactPlace, ChestChoice,
    ChoiceKind, Consent, DecisionView, GameCommand, GameCommandError, GameEvent, GameRuleset,
    GameState, HeroChange, HireAnswer, HireTarget, HireView, Manage, OpenBattle, Position,
    SchoolChoice, Sink, SkillOffer, SlotState, UniversityAnswer, Viewer, apply, hire_range,
    legal_commands, legal_management, legal_moves, observe, split_range,
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
    Artifacts, Bits, Body, Buildings, EventBody, HeroBody, Message, MonsterBody, MonsterTreasure,
    RandomDwelling, RawMap, Rewards, Stack, Template, Text, Tile, TimedEvent, TownBody, Version,
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
    let towns = h3_pack_towns::compile(&h3_pack_towns::ORIGINAL_PACKS, profile).unwrap();
    GameRuleset::new(
        Arc::new(core),
        Arc::new(part),
        Arc::new(battle),
        Arc::new(towns),
    )
    .unwrap()
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
/// Blockers' object class: any the import keeps as a blocker (not a hero or a town): a crater
/// (`AVLc*`; E08 T72: the hole, 124 until then, is typed now).
pub(super) const BLOCKER_CLASS: u32 = 118;
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
/// first and four crystals whose guards fight at once. Then (E08 T64b, increment 7's design note
/// §16) increment 7's kinds ([`REWARDS`]): one of each of its 31 classes, a guarded box, a guarded
/// event that stays, a guarded artifact and a monster with a treasure artifact. Then (E08 T75b,
/// increment 8's design note §15) the sea's kinds ([`SEA`]): the three boats, the four pickups and
/// Favorable Winds. Then (E08 T86b1, increment 9's design note §15) increment 9's kinds
/// ([`ECONOMY`]): the seven mines, the abandoned mine, the Lighthouse, the Shipyard, the three
/// weekly objects and gremlins that never grow. Then (E08 T96b1, increment 10's design note §14)
/// increment 10's kinds ([`DWELLINGS`]): six dwellings, the three random dwellings and the Refugee
/// Camp.
pub(super) const KINDS: [(u32, u32, Option<u32>); 96] = [
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
    (4, 0, None),
    (5, 7, None),
    (6, 0, None),
    (22, 0, None),
    (23, 0, None),
    (26, 0, None),
    (32, 0, None),
    (39, 0, None),
    (41, 0, None),
    (47, 0, None),
    (51, 0, None),
    (61, 0, None),
    (63, 0, None),
    (65, 0, None),
    (66, 0, None),
    (67, 0, None),
    (68, 0, None),
    (69, 0, None),
    (81, 0, None),
    (88, 0, None),
    (89, 0, None),
    (90, 0, None),
    (93, 0, None),
    (100, 0, None),
    (101, 0, None),
    (102, 0, None),
    (104, 0, None),
    (105, 0, None),
    (107, 0, None),
    (108, 0, None),
    (113, 0, None),
    (6, 0, None),
    (26, 0, None),
    (5, 8, None),
    (54, 98, None),
    (8, 0, None),
    (8, 1, None),
    (8, 2, None),
    (29, 0, None),
    (82, 0, None),
    (86, 0, None),
    (59, 0, None),
    (225, 0, None),
    (53, 0, None),
    (53, 1, None),
    (53, 2, None),
    (53, 3, None),
    (53, 4, None),
    (53, 5, None),
    (53, 6, None),
    (53, 7, None),
    (42, 0, None),
    (87, 0, None),
    (112, 0, None),
    (109, 0, None),
    (55, 0, None),
    (54, 28, None),
    (17, 56, None),
    (17, 29, None),
    (17, 12, None),
    (17, 58, None),
    (17, 35, None),
    (20, 0, None),
    (216, 0, None),
    (217, 1, None),
    (218, 6, None),
    (78, 0, None),
];
/// Increment 7's kinds in [`KINDS`] (E08 T64b, increment 7's design note §16): the first of them,
/// and how many. In order: the Arena, Centaur's Axe, a Pandora's Box (experience, gold, Magic Arrow
/// and four pikemen, which open the army dialog for a full army), a Corpse, Marletto Tower, an
/// event (experience and gold for every colour, computers too, removed once it fires), the Garden
/// of Revelation, a Lean To, the Library of Enlightenment, the School of Magic, the Mercenary Camp,
/// the Star Axis, the Pyramid, the five random artifacts, a Scholar of a drawn gift, the three
/// Shrines of a drawn spell, a Magic Arrow scroll, a Learning Stone, a treasure chest, a Tree of
/// Knowledge, a University, a Wagon, the School of War, a Warrior's Tomb and a Witch Hut of every
/// skill; then a box whose two gnolls fight at once, which gives a level's experience, Bless and
/// Prayer (its level-up before its spells, E08 T63b); an event two gnolls guard, which stays and
/// gives Attack and luck; Blackshard of the Dead Knight, whose gnolls ask first; and three savage
/// gnolls with Centaur's Axe as their treasure (ADV-MON-011).
pub(super) const REWARDS: (usize, usize) = (29, 35);
/// The kinds of [`REWARDS`] whose object has one tile, blocked and visited (as [`masks`] gives
/// them): the two events and the monster.
const ONE_TILE_REWARDS: [usize; 3] = [34, 61, 63];
/// The objects of [`REWARDS`] a plan draws: each kind's of a shape with a visitable tile, the
/// events' and the monster's of their one tile.
pub(super) static REWARD_OBJECTS: LazyLock<Vec<usize>> = LazyLock::new(|| {
    (REWARDS.0..REWARDS.0 + REWARDS.1)
        .flat_map(|kind| {
            let one = ONE_TILE_REWARDS.contains(&kind);
            let shapes: &[usize] = if one { &[0] } else { &[1, 2] };
            shapes
                .iter()
                .map(move |shape| BLOCKERS.len() * kind + shape)
        })
        .collect()
});
/// The sea's kinds in [`KINDS`] (E08 T75b, increment 8's design note §15): the first of them, and
/// how many. In order: the boats of the three types (class 8; no objects once imported, the map's
/// empty boats, ADV-SEA-001), Flotsam, the Sea Chest, the Shipwreck Survivor and the Ocean Bottle
/// (ADV-PIK-014 to -017; each one tile, blocked and visited from a neighbour), and Favorable Winds
/// (ADV-SEA-004; a ground, the import's one tile for a picture it does not know).
pub(super) const SEA: (usize, usize) = (64, 8);
/// The boats' objects, by type (one shape: [`masks`] gives a boat its one tile; the import takes
/// the template's visitable tile as the boat's).
pub(super) const BOAT_OBJECTS: [usize; 3] = [
    BLOCKERS.len() * SEA.0,
    BLOCKERS.len() * (SEA.0 + 1),
    BLOCKERS.len() * (SEA.0 + 2),
];
/// The sea's four pickups' objects (Flotsam, the Sea Chest, the Shipwreck Survivor, the Ocean
/// Bottle), each of its one tile.
pub(super) const PICKUP_OBJECTS: [usize; 4] = [
    BLOCKERS.len() * (SEA.0 + 3),
    BLOCKERS.len() * (SEA.0 + 4),
    BLOCKERS.len() * (SEA.0 + 5),
    BLOCKERS.len() * (SEA.0 + 6),
];
/// Favorable Winds' object (no tile blocked or visited, as a special ground's).
pub(super) const WINDS_OBJECT: usize = BLOCKERS.len() * (SEA.0 + 7);
/// Increment 9's kinds in [`KINDS`] (E08 T86b1, increment 9's design note §15): the first of them,
/// and how many. In order: the mines of the seven resources (class 53 by subclass; ADV-OWN-001,
/// ADV-MIN-002, GAM-RES-003), the abandoned mine (53.7, of a set [`body`] draws from its
/// identifier; ADV-MIN-003), the Lighthouse (ADV-OWN-005), the Shipyard (ADV-SHY-001 to -003), the
/// Windmill, the Water Wheel and the Mystical Garden (ADV-WEK-001 to -003), and twelve hostile
/// gremlins that never grow (ADV-MON-013's exception).
pub(super) const ECONOMY: (usize, usize) = (72, 14);
/// The ownable kinds of [`ECONOMY`] (the mines, the abandoned mine, the Lighthouse, the
/// Shipyard): their owners at load come from the plan's [`Plan::owners`].
const OWNABLE: RangeInclusive<usize> = ECONOMY.0..=ECONOMY.0 + 9;
/// The gremlins' kind, the last of [`ECONOMY`].
const STILL_MONSTER: usize = ECONOMY.0 + ECONOMY.1 - 1;
/// The objects of [`ECONOMY`] a plan draws: each kind's of a shape with a visitable tile, the
/// gremlins' of their one tile.
pub(super) static ECONOMY_OBJECTS: LazyLock<Vec<usize>> = LazyLock::new(|| {
    (ECONOMY.0..ECONOMY.0 + ECONOMY.1)
        .flat_map(|kind| {
            let shapes: &[usize] = if kind == STILL_MONSTER { &[0] } else { &[1, 2] };
            shapes
                .iter()
                .map(move |shape| BLOCKERS.len() * kind + shape)
        })
        .collect()
});
/// The Shipyard's object visited one column left of its anchor ([`BLOCKERS`]' second shape): a
/// plan's strait puts one on its coast, so that its first offset east of the entrance, two
/// columns, is the strait's water (ADV-SHY-001).
pub(super) const SHIPYARD_OBJECT: usize = BLOCKERS.len() * (ECONOMY.0 + 9) + 1;
/// Increment 10's kinds in [`KINDS`] (E08 T96b1, increment 10's design note §14): the first of
/// them, and how many. In order: the one-creature dwellings (class 17; ADV-DWL-001 to -004) of the
/// Guardhouse (Castle's level 1, Pikemen: free to their owner's visitor, no dialog), the Imp
/// Crucible (Inferno's level 1), the Dwarf Cottage (Rampart's level 2), the Barracks (Castle's
/// level 4) and the Monastery (Castle's level 5, guarded by nine Monks), the Elemental Conflux
/// (class 20, four creatures, guarded by twelve Earth Elementals), the random dwellings not tied to
/// a town (ADV-DWL-005: 216 of a faction set and a level range, 217 of level 2 and a set, 218 of
/// Stronghold and a range, each drawn by [`dwelling_body`] from its identifier) and the Refugee
/// Camp (78; ADV-RFG-001).
pub(super) const DWELLINGS: (usize, usize) = (86, 10);
/// The Refugee Camp's kind, the last of [`DWELLINGS`]: owned by nobody (increment 10's design note
/// §1), its every other kind ownable.
const CAMP: usize = DWELLINGS.0 + DWELLINGS.1 - 1;
/// The objects of [`DWELLINGS`] a plan draws: each kind's of a shape with a visitable tile.
pub(super) static DWELLING_OBJECTS: LazyLock<Vec<usize>> = LazyLock::new(|| {
    (DWELLINGS.0..DWELLINGS.0 + DWELLINGS.1)
        .flat_map(|kind| [1, 2].map(|shape| BLOCKERS.len() * kind + shape))
        .collect()
});

/// Whether an object of kind `kind` is ownable, its owner at load from the plan's
/// [`Plan::owners`]: [`OWNABLE`]'s (E08 T86b1) and (E08 T96b1) a dwelling or a random dwelling of
/// [`DWELLINGS`] (not the camp), as the import's ownable kinds (ADV-OWN-001).
fn ownable(kind: usize) -> bool {
    OWNABLE.contains(&kind) || (DWELLINGS.0..CAMP).contains(&kind)
}
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
/// ground's, which neither blocks nor is visited (E08 T20b3b1); an event's one tile too (E08
/// T64b: as the shipped maps' events, which the import keeps apart from the movement's masks);
/// (E08 T75b) a boat's and a sea pickup's one tile, and nothing of Favorable Winds'.
pub(super) fn masks(object: usize) -> (Mask, Mask) {
    match KINDS[object / BLOCKERS.len()].0 {
        54 | 26 | 8 | 29 | 82 | 86 | 59 => (&[(0, 0)], &[(0, 0)]),
        21 | 46 | 225 => (&[], &[]),
        _ => BLOCKERS[object % BLOCKERS.len()],
    }
}

/// The body of an object of kind `kind`: a pile's amount (and since E08 T21a its message with
/// guards, [`GUARDED`]), a monster's data ([`MONSTERS`]; its identifier, which AB and SoD store,
/// `identifier`), (E08 T64b) increment 7's ([`reward_body`]), (E08 T75b) the Ocean Bottle's
/// message, else none.
fn body(kind: usize, v: Version, identifier: u32) -> Body {
    if (REWARDS.0..REWARDS.0 + REWARDS.1).contains(&kind) {
        return reward_body(kind, v, identifier);
    }
    if (ECONOMY.0..ECONOMY.0 + ECONOMY.1).contains(&kind) {
        return economy_body(kind, v, identifier);
    }
    if (DWELLINGS.0..DWELLINGS.0 + DWELLINGS.1).contains(&kind) {
        return dwelling_body(kind, v, identifier);
    }
    if kind == SEA.0 + 6 {
        return Body::Sign(Text(b"Ahoy".to_vec()));
    }
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
/// The body of an object of increment 7's kind `kind` ([`REWARDS`]; E08 T64b), of a map of
/// version `v`, with the monster's `identifier`.
fn reward_body(kind: usize, v: Version, identifier: u32) -> Body {
    let gnolls = |count| {
        let mut guards = vec![None; 7];
        guards[0] = Some(Stack {
            creature: 98,
            count,
        });
        guards
    };
    let gold = |amount| [0, 0, 0, 0, 0, 0, amount];
    match kind {
        30 | 42..=46 => Body::Artifact(None),
        31 => Body::Pandora {
            message: None,
            rewards: Rewards {
                experience: 500,
                resources: gold(500),
                spells: vec![15],
                creatures: vec![Stack {
                    creature: 0,
                    count: 4,
                }],
                ..Rewards::default()
            },
        },
        34 => Body::Event(EventBody {
            message: None,
            rewards: Rewards {
                experience: 300,
                resources: gold(300),
                ..Rewards::default()
            },
            players: u8::MAX,
            computer: true,
            remove_after_visit: true,
        }),
        47 => Body::Scholar { kind: -1, code: 0 },
        48..=50 => Body::Shrine {
            spell: u32::from(u8::MAX),
        },
        51 => Body::Scroll {
            message: None,
            spell: 15,
        },
        59 => Body::WitchHut(Bits(if v == Version::RoE {
            Vec::new()
        } else {
            vec![0xFF, 0xFF, 0xFF, 0x0F]
        })),
        60 => Body::Pandora {
            message: Some(Message {
                text: Text(Vec::new()),
                guards: Some(gnolls(2)),
            }),
            rewards: Rewards {
                experience: 1500,
                spells: vec![41, 48],
                ..Rewards::default()
            },
        },
        61 => Body::Event(EventBody {
            message: Some(Message {
                text: Text(Vec::new()),
                guards: Some(gnolls(2)),
            }),
            rewards: Rewards {
                luck: 1,
                primary: [1, 0, 0, 0],
                ..Rewards::default()
            },
            players: u8::MAX,
            computer: true,
            remove_after_visit: false,
        }),
        62 => Body::Artifact(Some(Message {
            text: Text(b"Fight?".to_vec()),
            guards: Some(gnolls(2)),
        })),
        63 => Body::Monster(MonsterBody {
            identifier: (v != Version::RoE).then_some(identifier),
            count: 3,
            disposition: 4,
            treasure: Some(MonsterTreasure {
                artifact: Some(7),
                ..MonsterTreasure::default()
            }),
            never_flees: true,
            no_growth: false,
        }),
        _ => Body::None,
    }
}

/// The body of an object of increment 9's kind `kind` ([`ECONOMY`]; E08 T86b1), neutral: a mine's
/// and an ownable's owner (set by [`lay_out`] from [`Plan::owners`]), an abandoned mine's set (one
/// to all seven resources, by its identifier), the gremlins' data (their identifier, which AB and
/// SoD store, `identifier`); the weekly objects' none.
fn economy_body(kind: usize, v: Version, identifier: u32) -> Body {
    match kind - ECONOMY.0 {
        0..=6 => Body::Mine {
            owner: NEUTRAL_OWNER,
        },
        7 => Body::AbandonedMine(Bits(vec![
            u8::try_from(identifier % 127).unwrap() + 1,
            0,
            0,
            0,
        ])),
        8 | 9 => Body::Owned {
            owner: NEUTRAL_OWNER,
        },
        13 => Body::Monster(MonsterBody {
            identifier: (v != Version::RoE).then_some(identifier),
            count: 12,
            disposition: 3,
            treasure: None,
            never_flees: false,
            no_growth: true,
        }),
        _ => Body::None,
    }
}

/// The body of an object of increment 10's kind `kind` ([`DWELLINGS`]; E08 T96b1), neutral: a
/// dwelling's owner (set by [`lay_out`] from [`Plan::owners`]); a random dwelling's owner, faction
/// set (216 and 217: one of the eight base factions, codes 0 to 7, in a map's set of its version's
/// bytes) and level range (216 and 218: one level, codes 0 to 6), by its `identifier`; the camp's
/// none. One outcome each, varied over the objects: the dwelling a random one becomes is a draw of
/// the setup stream that every viewer of its tiles sees (its type), which the hygiene property's
/// hidden differences (the seed, the other players' armies drawn before it) would move
/// (`view.rs`).
fn dwelling_body(kind: usize, v: Version, identifier: u32) -> Body {
    let mut set = vec![0; v.faction_bytes()];
    set[0] = 1 << (identifier % 8);
    let level = u8::try_from(identifier / 8 % 7).unwrap();
    let faction = Some((0, Some(Bits(set))));
    let (low, high) = (level, level);
    let random = |faction, levels| {
        Body::RandomDwelling(RandomDwelling {
            owner: NEUTRAL_OWNER,
            faction,
            levels,
        })
    };
    match kind - DWELLINGS.0 {
        0..=5 => Body::Owned {
            owner: NEUTRAL_OWNER,
        },
        6 => random(faction, Some((low, high))),
        7 => random(faction, None),
        8 => random(None, Some((low, high))),
        _ => Body::None,
    }
}

/// A random dwelling's outcome ([`dwelling_outcomes`]): the dwelling type, its blocked and
/// visitable cells as `(column, row)` left of and above the anchor, and its shift.
type Outcome = (ObjectTypeId, Vec<(u8, u8)>, u8);

/// The tiles and names a random dwelling of kind `kind` (216 to 218 of [`DWELLINGS`]) with `body` may
/// take on the build (E08 T96b1; ADV-DWL-005, increment 10's design note §5): for each faction of
/// its set and each level of its range (the subclass giving 217's level and 218's faction, as the
/// import reads them), the dwelling type its faction's table gives, that type's blocked and
/// visitable cells and its shift. The tables are the faithful rules' (the strict profile's are the
/// same packs').
fn dwelling_outcomes(kind: usize, body: &Body) -> Vec<Outcome> {
    let Body::RandomDwelling(d) = body else {
        return Vec::new();
    };
    let subclass = u8::try_from(KINDS[kind].1).unwrap();
    let factions: Vec<usize> = match &d.faction {
        Some((_, Some(set))) => (0..8).filter(|&b| set.bytes()[0] >> b & 1 == 1).collect(),
        _ => vec![usize::from(subclass)],
    };
    let (low, high) = d.levels.unwrap_or((subclass, subclass));
    let (core, part) = (FAITHFUL.core(), FAITHFUL.adventure());
    let mut outcomes = Vec::new();
    for faction in factions {
        let faction = core.faction_by_key(codes::FACTIONS[faction]).unwrap();
        let table = part.random_dwelling(faction).unwrap();
        for &object_type in &table[usize::from(low)..=usize::from(high)] {
            let kind = &part.object_type(object_type).unwrap().kind;
            let ObjectKind::Dwelling {
                blocked,
                visitable,
                shift,
                ..
            } = *kind
            else {
                unreachable!("a random dwelling's table names a dwelling");
            };
            let cells = blocked.offsets().chain(visitable.offsets()).collect();
            outcomes.push((object_type, cells, shift));
        }
    }
    outcomes
}

/// The tiles of the map of side `side` that the random dwelling with this kind, body and anchor
/// may cover once the build resolves it (E08 T96b1; ADV-DWL-005): every outcome's
/// ([`dwelling_outcomes`]) blocked and visitable cells from the anchor moved by its shift; none for
/// an object of another kind. No hero may stand there (`SetupError::RandomDwellingTile`) and no
/// boat lie there (`SetupError::Boat`).
fn dwelling_reach(kind: usize, body: &Body, (ax, ay, az): Spot, side: u8) -> Vec<Spot> {
    dwelling_outcomes(kind, body)
        .iter()
        .flat_map(|(_, cells, shift)| {
            cells.iter().filter_map(move |&(column, row)| {
                let x = (ax + shift).checked_sub(column).filter(|&x| x < side)?;
                Some((x, ay.checked_sub(row)?, az))
            })
        })
        .collect()
}

/// [`dwelling_reach`] of an object of a laid-out map of side `side` (E08 T96b1), for the hygiene
/// property's changes (`view.rs`).
pub(super) fn object_reach(o: &crate::model::Object, side: u8) -> Vec<Spot> {
    let kind = usize::try_from(o.template.saturating_sub(1)).unwrap() / BLOCKERS.len();
    dwelling_reach(kind, &o.body, (o.pos.x, o.pos.y, o.pos.z), side)
}

/// A neutral ownable object's owner in a map's body.
const NEUTRAL_OWNER: u32 = 0xFF;

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
    /// T20b3b1 also the monsters' and grounds', [`MONSTER_OBJECTS`], [`GROUND_OBJECTS`]; since
    /// E08 T64b increment 7's, [`REWARD_OBJECTS`]).
    pub(super) blockers: Vec<(Spot, usize)>,
    /// The game's difficulty (E08 T14b), written as the map's (the import's own default is
    /// normal; the tests take the map's): impossible, with no gold to start, in half the cases,
    /// so that Cartographers are refused as well as offered.
    pub(super) difficulty: u8,
    /// (E08 T52) The first hero's exchanges: a mate of its owner beside it, and its full army;
    /// none: neither.
    pub(super) exchange: Option<Exchange>,
    /// (E08 T85b) The map's timed events ([`timed`]); before SoD the map writes no human flag, so
    /// [`lay_out`] sets it.
    pub(super) events: Vec<TimedEvent>,
    /// (E08 T86b1) The owners at load of the ownable objects ([`ownable`]; since E08 T96b1 the
    /// dwellings and random dwellings too): the `n`-th such object of [`Plan::blockers`] the
    /// `n`-th's (cyclically), a playing colour by its index (modulo the players) or none
    /// (neutral); empty: every one neutral.
    pub(super) owners: Vec<Option<u8>>,
    /// (E09 T9b1; the town epic's design note §18) The towns ([`TownPlan`], [`towns`]), laid
    /// out after every other object.
    pub(super) towns: Vec<TownPlan>,
}

/// A plan's town (E09 T9b1; the town epic's design note §18), laid out as the import's tests
/// build one (`towns.rs`: the shipped towns' masks, five tiles of its row and three above, the
/// entrance in the middle of its row, so that it is entered from the row below).
#[derive(Debug, Clone)]
pub(super) struct TownPlan {
    /// Its entrance's offset from a hero's tile, two columns left of it to two right and one to
    /// four rows above, and the hero (modulo the plan's).
    pub(super) at: ((u8, u8), u8),
    /// A town of this faction's code (class 98; modulo the version's factions), or none: a random
    /// town (class 77).
    pub(super) faction: Option<u8>,
    /// The owner at load: a playing colour by its index (modulo the players), or none (neutral).
    pub(super) owner: Option<u8>,
    /// A random town's alignment on a SoD map: a playing colour by its index (modulo the
    /// players), or none (the owner's).
    pub(super) alignment: Option<u8>,
    /// The garrison: none the editor's (an owned town's empty, a neutral town's drawn,
    /// TWN-SET-004), or seven slots as creature code and count (all empty: none at all).
    pub(super) garrison: Option<Vec<Option<(u16, u16)>>>,
    /// The buildings ([`TOWN_BUILDINGS`]).
    pub(super) buildings: usize,
    /// The garrison's formation is tight.
    pub(super) tight: bool,
    /// Its entrance on the hero's tile instead, the town the hero's owner's: the hero its visitor
    /// at load (TWN-VIS-002), so that sieges meet visitors with garrisons to merge (TWN-SIE-002).
    pub(super) visited: bool,
}

/// The plans' towns' buildings (E09 T9b1; building bits, `docs/formats/h3m.md`, "Town
/// buildings"): the standard set without a Fort and with one, and custom sets of the Town Hall,
/// the City Hall and the Capitol (TWN-CAP-002: one Capitol per player after a capture) with a
/// Fort and without one.
const TOWN_BUILDINGS: [Buildings; 4] = [
    Buildings::Standard { fort: false },
    Buildings::Standard { fort: true },
    Buildings::Custom {
        built: [0b1111, 0, 0, 0, 0, 0],
        forbidden: [0; 6],
    },
    Buildings::Custom {
        built: [0b0111, 0, 0, 0, 0, 0],
        forbidden: [0; 6],
    },
];
/// The level-1 creatures of the eight base factions (Pikemen, Centaurs, Gremlins, Imps,
/// Skeletons, Troglodytes, Goblins and Gnolls): a plan's weak garrisons (E09 T9b1).
const LEVEL_1: [u16; 8] = [0, 14, 28, 42, 56, 70, 84, 98];

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
                // E08 T64b, drawn last for the same reason.
                rewards(),
                // E08 T75b, drawn last for the same reason.
                sea(),
                // E08 T85b, drawn last for the same reason.
                timed(),
                // E08 T86b1, drawn last for the same reason.
                economy(),
                // E08 T96b1, drawn last for the same reason.
                dwellings(),
                // E09 T9b1, drawn last for the same reason.
                towns(),
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
                (
                    backpacks,
                    mut skills,
                    beside,
                    pile,
                    (
                        mut kits,
                        lifts,
                        nomads,
                        exchange,
                        rewards,
                        sea,
                        events,
                        economy,
                        dwellings,
                        towns,
                    ),
                ),
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
                // E08 T64b: increment 7's objects within two tiles of a hero, so that random
                // play visits them and answers what they ask.
                for ((dx, dy), hero, object) in rewards {
                    let ((hx, hy, hz), _) = heroes[usize::from(hero) % heroes.len()];
                    let levels = 1 + u8::from(two_levels);
                    let at = |h: u8, d: u8| (h % side + side + d - 2) % side;
                    blockers.push(((at(hx, dx), at(hy, dy), hz % levels), object));
                }
                // E08 T86b1: increment 9's objects within two tiles of a hero, so that random
                // play flags, guards and visits them ([`Economy`]).
                for &((dx, dy), hero, object) in &economy.objects {
                    let ((hx, hy, hz), _) = heroes[usize::from(hero) % heroes.len()];
                    let levels = 1 + u8::from(two_levels);
                    let at = |h: u8, d: u8| (h % side + side + d - 2) % side;
                    blockers.push(((at(hx, dx), at(hy, dy), hz % levels), object));
                }
                // E08 T75b: a strait beside a hero, its boats, pickups, winds and monster, and
                // the hero's sources of sea points ([`Sea`]).
                if let Some(sea) = sea {
                    let index = usize::from(sea.hero) % heroes.len();
                    let ((hx, hy, hz), _) = heroes[index];
                    let (hx, hy, z) = (hx % side, hy % side, hz % (1 + u8::from(two_levels)));
                    let column = |c: u8| (hx + sea.gap + c % sea.width) % side;
                    let row = |r: u8| (hy + side + r - 2) % side;
                    for c in 0..sea.width {
                        for y in 0..side {
                            let i = (usize::from(z) * usize::from(side) + usize::from(y))
                                * usize::from(side)
                                + usize::from(column(c));
                            tiles[i].0 = 8;
                        }
                    }
                    for &((c, r), object) in sea.boats.iter().chain(&sea.objects) {
                        blockers.push(((column(c), row(r), z), object));
                    }
                    // E08 T86b1: a Shipyard on the strait's coast, its entrance two columns west
                    // of the strait ([`SHIPYARD_OBJECT`]), so that it has a boat tile.
                    if let Some(r) = economy.coast {
                        let x = (hx + sea.gap + side - 1) % side;
                        blockers.push(((x, row(r), z), SHIPYARD_OBJECT));
                    }
                    let worn = sea.worn.map(|w| SEAFARERS[w].0);
                    if let Some(code) =
                        worn.filter(|&c| c != ADMIRALS_HAT || version == Version::Sod)
                    {
                        kits.resize_with(kits.len().max(index + 1), Kit::default);
                        kits[index].worn.push(code);
                    }
                    if sea.navigation > 0 {
                        skills.resize(skills.len().max(index + 1), None);
                        let own = skills[index].get_or_insert_with(Vec::new);
                        own.push((NAVIGATION, sea.navigation));
                    }
                }
                // E08 T96b1: increment 10's objects within two tiles of a hero, so that random
                // play visits them, answers their questions, fights their guards and hires
                // ([`dwellings`]); after every other object, so that a plan without them lays out
                // as before.
                for &((dx, dy), hero, object) in &dwellings {
                    let ((hx, hy, hz), _) = heroes[usize::from(hero) % heroes.len()];
                    let levels = 1 + u8::from(two_levels);
                    let at = |h: u8, d: u8| (h % side + side + d - 2) % side;
                    blockers.push(((at(hx, dx), at(hy, dy), hz % levels), object));
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
                    events,
                    owners: economy.owners,
                    towns,
                }
            },
        )
}

/// Increment 9's objects of a plan (E08 T86b1, increment 9's design note §15), drawn as
/// [`economy`] draws them.
#[derive(Debug, Clone)]
pub(super) struct Economy {
    /// The objects ([`ECONOMY_OBJECTS`]), each at an offset from a hero's tile (two columns and rows
    /// left of it to two right) and the hero (modulo the plan's).
    objects: Vec<((u8, u8), u8, usize)>,
    /// The ownable objects' owners at load ([`Plan::owners`]).
    owners: Vec<Option<u8>>,
    /// On a plan with a strait ([`Sea`]): a Shipyard on its coast ([`SHIPYARD_OBJECT`]) at a row
    /// from two above the strait's hero to two below (0 to 4).
    coast: Option<u8>,
}

/// A plan's increment 9 objects (E08 T86b1): in seven plans of eight, one to six; up to four
/// owners, each neutral in one case of two; in three plans of four with a strait, a Shipyard on
/// its coast.
fn economy() -> impl Strategy<Value = Economy> {
    let object = (0..ECONOMY_OBJECTS.len()).prop_map(|i| ECONOMY_OBJECTS[i]);
    (
        prop_oneof![
            1 => Just(Vec::new()),
            7 => vec(((0u8..5, 0u8..5), any::<u8>(), object), 1..=6),
        ],
        vec(proptest::option::of(any::<u8>()), 0..=4),
        proptest::option::weighted(0.75, 0u8..5),
    )
        .prop_map(|(objects, owners, coast)| Economy {
            objects,
            owners,
            coast,
        })
}

/// Increment 10's objects of a plan (E08 T96b1, increment 10's design note §14): each one's
/// offset from a hero's tile (two columns and rows left of it to two right), the hero (modulo the
/// plan's), and the object ([`DWELLING_OBJECTS`]); in seven plans of eight, one to six; their
/// owners at load are the ownable objects' ([`Plan::owners`]).
fn dwellings() -> impl Strategy<Value = Vec<((u8, u8), u8, usize)>> {
    let object = (0..DWELLING_OBJECTS.len()).prop_map(|i| DWELLING_OBJECTS[i]);
    prop_oneof![
        1 => Just(Vec::new()),
        7 => vec(((0u8..5, 0u8..5), any::<u8>(), object), 1..=6),
    ]
}

/// A plan's towns (E09 T9b1; the town epic's design note §18): in three plans of four, one to
/// four ([`TownPlan`]); each of a fixed faction in three of four, owned in two of three, a random
/// town's alignment in one of two (SoD), visited at load in one of three; its garrison the
/// editor's in one of six, none at all in two, one weak stack of a level-1 creature in two, seven
/// slots of any base creature in one ([`slots`]), so that visits capture, sieges are won and lost
/// and garrisons merge.
fn towns() -> impl Strategy<Value = Vec<TownPlan>> {
    prop_oneof![
        1 => Just(Vec::new()),
        3 => vec(town(), 1..=4),
    ]
}

/// A plan's town ([`towns`]).
fn town() -> impl Strategy<Value = TownPlan> {
    let weak = (proptest::sample::select(&LEVEL_1[..]), 1u16..=10, 0usize..7).prop_map(
        |(creature, count, slot)| {
            let mut slots = vec![None; 7];
            slots[slot] = Some((creature, count));
            Some(slots)
        },
    );
    let garrison = prop_oneof![
        1 => Just(None),
        2 => Just(Some(vec![None; 7])),
        2 => weak,
        1 => slots().prop_map(Some),
    ];
    (
        ((0u8..5, 0u8..4), any::<u8>()),
        proptest::option::weighted(0.75, any::<u8>()),
        proptest::option::weighted(0.67, any::<u8>()),
        proptest::option::weighted(0.5, any::<u8>()),
        garrison,
        0..TOWN_BUILDINGS.len(),
        any::<bool>(),
        proptest::option::weighted(0.33, Just(())),
    )
        .prop_map(
            |(at, faction, owner, alignment, garrison, buildings, tight, visited)| TownPlan {
                at,
                faction,
                owner,
                alignment,
                garrison,
                buildings,
                tight,
                visited: visited.is_some(),
            },
        )
}

/// A plan's timed events (E08 T85b; GAM-CAL-004): none to three, each for some colours (those
/// nobody plays left out by the import), humans and computers as drawn, from the file's day 0 to
/// 6 (the game's 1 to 7) once or every one to three days, each resource in four given or taken
/// (up to 3 000), so that takes meet empty treasuries (an impossible game's).
fn timed() -> impl Strategy<Value = Vec<TimedEvent>> {
    let amount = prop_oneof![3 => Just(0i32), 1 => -3000i32..=3000];
    let event = (
        any::<u8>(),
        any::<(bool, bool)>(),
        0u16..=6,
        prop_oneof![2 => Just(0u16), 1 => 1u16..=3],
        vec(amount, 7),
    )
        .prop_map(
            |(players, (human, computer), first_day, interval, amounts)| TimedEvent {
                resources: amounts.try_into().unwrap(),
                players,
                human,
                computer,
                first_day,
                interval,
                ..TimedEvent::default()
            },
        );
    vec(event, 0..=3)
}

/// Increment 7's objects of a plan (E08 T64b): each one's offset from a hero's tile (two
/// columns and rows left of it to two right), the hero (modulo the plan's), and the object
/// ([`REWARD_OBJECTS`]); in seven plans of eight, up to six.
fn rewards() -> impl Strategy<Value = Vec<((u8, u8), u8, usize)>> {
    let object = (0..REWARD_OBJECTS.len()).prop_map(|i| REWARD_OBJECTS[i]);
    prop_oneof![
        1 => Just(Vec::new()),
        7 => vec(((0u8..5, 0u8..5), any::<u8>(), object), 1..=6),
    ]
}

/// The sea's sources of a hero's points at sea (E08 T75b; HER-ADV-009), as code and position in a
/// map's worn slots: the Necklace of Ocean Guidance (71) on the neck, the Sea Captain's Hat (123)
/// and the Admiral's Hat (136, SoD's combination of the two, free boarding) on the head.
pub(super) const SEAFARERS: [(u16, usize); 3] = [(71, 2), (123, 0), (136, 0)];
/// The Admiral's Hat's code ([`SEAFARERS`]).
const ADMIRALS_HAT: u16 = 136;
/// Navigation's skill code (`SSTRAITS.TXT` order).
const NAVIGATION: u8 = 5;

/// A strait beside a hero (E08 T75b, increment 8's design note §15): water over the whole height of
/// the hero's level, with boats on it, mostly on its coast, the sea's pickups, Favorable Winds and
/// monsters on it, and the hero's sources of sea points; so that random play boards, sails, lands,
/// takes the pickups and fights at sea.
#[derive(Debug, Clone)]
pub(super) struct Sea {
    /// The hero it runs beside (modulo the plan's heroes).
    hero: u8,
    /// Its first column's distance east of the hero's: 1 (the hero on its coast) or 2.
    gap: u8,
    /// Its width in columns, 2 to 4.
    width: u8,
    /// The boats ([`BOAT_OBJECTS`]), each at a column of the strait (modulo its width: 0, the
    /// coast's, in two cases of three) and a row from two above the hero's to two below (0 to 4).
    boats: Vec<((u8, u8), usize)>,
    /// The pickups ([`PICKUP_OBJECTS`]), Favorable Winds ([`WINDS_OBJECT`]) and monsters
    /// ([`MONSTER_OBJECTS`]), placed as the boats.
    objects: Vec<((u8, u8), usize)>,
    /// The hero's Navigation level (0: none of its own).
    navigation: u8,
    /// What it wears of [`SEAFARERS`] (the Admiral's Hat, half the time, on a SoD map only).
    worn: Option<usize>,
}

/// A plan's strait (E08 T75b): in three plans of four, one to three boats and up to five objects.
fn sea() -> impl Strategy<Value = Option<Sea>> {
    let column = prop_oneof![2 => Just(0u8), 1 => any::<u8>()];
    let at = (column, 0u8..5);
    let object = prop_oneof![
        4 => proptest::sample::select(&PICKUP_OBJECTS[..]),
        1 => Just(WINDS_OBJECT),
        1 => proptest::sample::select(&MONSTER_OBJECTS[..]),
    ];
    proptest::option::weighted(
        0.75,
        (
            any::<u8>(),
            1u8..=2,
            2u8..=4,
            vec((at.clone(), proptest::sample::select(&BOAT_OBJECTS[..])), 1..=3),
            vec((at, object), 0..=5),
            0u8..=3,
            prop_oneof![1 => Just(None), 1 => Just(Some(0)), 1 => Just(Some(1)), 3 => Just(Some(2))],
        )
            .prop_map(
                |(hero, gap, width, boats, objects, navigation, worn)| Sea {
                    hero,
                    gap,
                    width,
                    boats,
                    objects,
                    navigation,
                    worn,
                },
            ),
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
    // E08 T85b: before SoD an event fires for humans (the reader's, `docs/formats/h3m.md`).
    m.events = plan
        .events
        .iter()
        .map(|e| TimedEvent {
            human: e.human || v != Version::Sod,
            ..e.clone()
        })
        .collect();
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
    // E08 T75b: the tiles the objects laid out block or are visited on, and the boats, laid out
    // after them ([`boats_laid_out`]).
    let mut covered: Vec<Spot> = Vec::new();
    let mut boats: Vec<(Spot, usize)> = Vec::new();
    let colours: Vec<u8> = plan.players.iter().map(|&(colour, _)| colour).collect();
    // E08 T86b1: the ownable objects met so far, for their owners ([`Plan::owners`]).
    let mut ownables = 0;
    // E08 T96b1: the names the dwellings laid out may take, as type and anchor.
    let mut named: Vec<(ObjectTypeId, Spot)> = Vec::new();
    for &((x, y, z), shape) in plan.blockers.iter().chain(&goblins) {
        let (ax, ay, az) = (x % side, y % side, z % levels);
        if BOAT_OBJECTS.contains(&shape) {
            boats.push(((ax, ay, az), shape));
            continue;
        }
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
        // Every monster has one type, whatever its creature (E08 T20b3b1); so has every placed
        // artifact, whatever its artifact (E08 T64b).
        let subclass = if class == 54 || class == 5 {
            0
        } else {
            subclass
        };
        let twin = kind > 0 && typed.contains(&((class, subclass), (ax, ay, az)));
        // E08 T96b1: a dwelling's name, a random dwelling's every outcome's (its type and moved
        // anchor, `dwelling_outcomes`), which no other dwelling may share (the build's name check,
        // `SetupError::SharedAnchor`); a random dwelling's outcomes' tiles, on which no hero may
        // stand (`SetupError::RandomDwellingTile`) and no boat lie (`SetupError::Boat`).
        let identifier = u32::try_from(m.objects.len()).unwrap();
        let body = body(kind, v, identifier);
        let outcomes = dwelling_outcomes(kind, &body);
        let names: Vec<(ObjectTypeId, Spot)> = match class {
            17 | 20 => {
                let key = codes::object_type(class, subclass).flatten().unwrap();
                let id = FAITHFUL.adventure().object_type_by_key(key).unwrap();
                vec![(id, (ax, ay, az))]
            }
            _ => outcomes
                .iter()
                .map(|&(id, _, shift)| (id, (ax + shift, ay, az)))
                .collect(),
        };
        let reach = dwelling_reach(kind, &body, (ax, ay, az), side);
        let shared = names.iter().any(|name| named.contains(name));
        let refused = heroes.iter().any(|&(at, _)| reach.contains(&at));
        // E08 T86b1: an ownable object takes the next owner ((E08 T96b1) a dwelling and a random
        // dwelling too); one of [`OWNABLE`] entered on water is left out (the import refuses it:
        // `Unsupported::OwnedOnWater`, ADV-OWN-002). (E08 T97b1) A dwelling or a random dwelling
        // entered on water is laid out, as the import builds it: a hero in a boat visits it as on
        // land (survey `E08-dwellings.md` §17, ADV-DWL-004).
        let owner = ownable(kind).then(|| {
            ownables += 1;
            let drawn = plan.owners.get((ownables - 1) % plan.owners.len().max(1));
            drawn.copied().flatten().map_or(NEUTRAL_OWNER, |n| {
                u32::from(colours[usize::from(n) % colours.len()])
            })
        });
        let afloat = OWNABLE.contains(&kind)
            && visitable.iter().any(|&(column, row)| {
                let (Some(ex), Some(ey)) = (ax.checked_sub(column), ay.checked_sub(row)) else {
                    return false;
                };
                let index = (usize::from(az) * usize::from(side) + usize::from(ey))
                    * usize::from(side)
                    + usize::from(ex);
                m.terrain.tiles[index].terrain == 8
            });
        if !twin && !afloat && !shared && !refused && !heroes.iter().any(covers) {
            if kind > 0 {
                typed.push(((class, subclass), (ax, ay, az)));
            }
            named.extend(names);
            covered.extend(reach);
            let template = 1 + u32::try_from(shape).unwrap();
            let mut o = object(ax, ay, template, body);
            o.pos.z = az;
            match (owner, &mut o.body) {
                (Some(colour), Body::Mine { owner } | Body::Owned { owner }) => *owner = colour,
                (Some(colour), Body::RandomDwelling(d)) => d.owner = colour,
                _ => {}
            }
            m.objects.push(o);
            covered.extend(
                blocked
                    .iter()
                    .chain(visitable)
                    .filter_map(|&(column, row)| {
                        Some((ax.checked_sub(column)?, ay.checked_sub(row)?, az))
                    }),
            );
        }
    }
    // E09 T9b1: each hero's owner, as the heroes are laid out below.
    let owners: Vec<u8> = (0..heroes.len())
        .map(|i| colours[if Some(i) == mate { 0 } else { i } % colours.len()])
        .collect();
    towns_laid_out(&mut m, plan, &heroes, &owners, &mut covered);
    boats_laid_out(&mut m, &boats, &mut covered);
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
                let movers = MOVERS.iter().chain(&SEAFARERS);
                let (_, slot) = movers.into_iter().find(|&&(a, _)| a == artifact).unwrap();
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

/// The tiles a town blocks (its entrance among them), as `(column, row)` left of and above its
/// anchor ([`super::towns::town_template`]'s), and its entrance.
const TOWN_BLOCKED: Mask = &[
    (0, 0),
    (1, 0),
    (2, 0),
    (3, 0),
    (4, 0),
    (1, 1),
    (2, 1),
    (3, 1),
];
const TOWN_ENTRANCE: (u8, u8) = (2, 0);

/// The plan's towns laid out after its other objects but the boats (E09 T9b1; [`TownPlan`]), each
/// with a template of its own after the others: its anchor on the map, its entrance on the map's
/// land, none of its tiles a hero's (the import refuses a hero in a town, a hero on its entrance
/// would be its visitor at load) or another object's (`covered`, which takes its tiles); a town
/// anywhere else is left out. Plans without towns keep their maps (the committed ones,
/// `fixtures.rs`).
fn towns_laid_out(
    m: &mut RawMap,
    plan: &Plan,
    heroes: &[(Spot, &Slots)],
    owners: &[u8],
    covered: &mut Vec<Spot>,
) {
    let (v, side) = (plan.version, plan.side);
    let colours: Vec<u8> = plan.players.iter().map(|&(colour, _)| colour).collect();
    let colour = |n: u8| colours[usize::from(n) % colours.len()];
    for town in &plan.towns {
        let ((dx, dy), hero) = town.at;
        let hero = usize::from(hero) % heroes.len();
        let ((hx, hy, hz), _) = heroes[hero];
        let (ex, ey) = if town.visited {
            (hx, hy)
        } else {
            ((hx + side + dx - 2) % side, (hy + side - 1 - dy) % side)
        };
        let (ax, ay) = (ex + TOWN_ENTRANCE.0, ey + TOWN_ENTRANCE.1);
        let tiles: Vec<Spot> = TOWN_BLOCKED
            .iter()
            .filter_map(|&(column, row)| Some((ax.checked_sub(column)?, ay.checked_sub(row)?, hz)))
            .collect();
        let index = (usize::from(hz) * usize::from(side) + usize::from(ey)) * usize::from(side)
            + usize::from(ex);
        let visitor = |t: &Spot| town.visited && *t == (ex, ey, hz);
        let taken =
            |t: &Spot| covered.contains(t) || heroes.iter().any(|&(at, _)| at == *t && !visitor(t));
        if ax >= side || m.terrain.tiles[index].terrain >= 8 || tiles.iter().any(taken) {
            continue;
        }
        let factions = if v == Version::RoE { 8 } else { 9 };
        let (class, subclass) = town
            .faction
            .map_or((77, 0), |f| (98, u32::from(f % factions)));
        let alignment = town
            .alignment
            .filter(|_| class == 77 && v == Version::Sod)
            .map_or(u8::MAX, colour);
        let garrison = town.garrison.as_ref().map(|slots| {
            slots
                .iter()
                .map(|slot| slot.map(|(creature, count)| Stack { creature, count }))
                .collect()
        });
        let identifier = u32::try_from(m.objects.len()).unwrap();
        let body = TownBody {
            identifier: (v != Version::RoE).then_some(identifier),
            owner: if town.visited {
                owners[hero]
            } else {
                town.owner.map_or(u8::MAX, colour)
            },
            garrison,
            formation: u8::from(town.tight),
            buildings: TOWN_BUILDINGS[town.buildings].clone(),
            required_spells: (v != Version::RoE).then(|| Bits(vec![0; 9])),
            banned_spells: Bits(vec![0; 9]),
            alignment,
            ..TownBody::default()
        };
        let template = u32::try_from(m.templates.len()).unwrap();
        m.templates
            .push(super::towns::town_template(class, subclass));
        let mut o = object(ax, ay, template, Body::Town(Box::new(body)));
        o.pos.z = hz;
        m.objects.push(o);
        covered.extend(tiles);
    }
}

/// The plan's boats laid out after its other objects (E08 T75b), each where the setup takes it
/// (ADV-SEA-001): on water, on no tile an object blocks or is visited on (`covered`), and on no
/// earlier boat's tile (the heroes stand on land, [`lay_out`]); a boat anywhere else is left out.
/// A map with boats has its coast marked as the game's maps have (the flag of every tile but
/// water with water on a neighbour, `h3_map::writer`'s): a boat lands only on the coast
/// (ADV-SEA-003, D-42). Plans without boats keep their maps (the committed ones, `fixtures.rs`).
fn boats_laid_out(m: &mut RawMap, boats: &[(Spot, usize)], covered: &mut Vec<Spot>) {
    let side = m.terrain.size;
    if !boats.is_empty() {
        let water: Vec<bool> = m.terrain.tiles.iter().map(|t| t.terrain == 8).collect();
        for (i, tile) in m.terrain.tiles.iter_mut().enumerate() {
            let (x, y, z) = (i % side, i / side % side, i / side / side);
            let near = |c: usize| c.saturating_sub(1)..=(c + 1).min(side - 1);
            let coast = near(y).any(|ny| near(x).any(|nx| water[(z * side + ny) * side + nx]));
            if !water[i] && coast {
                tile.flags |= codes::COAST;
            }
        }
    }
    for &((x, y, z), shape) in boats {
        let index = (usize::from(z) * side + usize::from(y)) * side + usize::from(x);
        if m.terrain.tiles[index].terrain == 8 && !covered.contains(&(x, y, z)) {
            covered.push((x, y, z));
            let template = 1 + u32::try_from(shape).unwrap();
            let mut o = object(x, y, template, Body::None);
            o.pos.z = z;
            m.objects.push(o);
        }
    }
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

/// The random dwellings of the map `m` as the game built on it with `rules` resolved them (E08
/// T96b1; ADV-DWL-005, increment 10's design note §5, §14), read apart from the build: each random
/// dwelling of the import's setup (its holds a faction set and a level range) is on `state`'s map
/// a dwelling of a type its holds name (a faction of the set, the dwelling of a level of the range
/// in that faction's table), with that type's blocked and visitable cells as its masks, its anchor
/// the setup's moved right by the type's shift, holding nothing and owned as the setup says; and
/// its type one of the outcomes the map's bytes name (its class, subclass and body read back,
/// [`dwelling_outcomes`], apart from the import's holds), its anchor the map's moved by that
/// outcome's shift. The number of them.
fn check_resolved(
    rules: &GameRuleset,
    m: &RawMap,
    state: &GameState,
) -> Result<usize, TestCaseError> {
    let m = parse(&encode(m), &Limits::default()).unwrap();
    let setup = import(&m, rules).unwrap();
    let (part, adventure) = (rules.adventure(), state.adventure());
    // The map's random dwellings in its order, which the import keeps (it leaves out none: a
    // plan lays out none entered on water): each one's outcomes read from its bytes read back
    // (its kind by its template, its faction set and level range by its body;
    // [`dwelling_outcomes`]), apart from the import's holds.
    let mut raw = m
        .objects
        .iter()
        .filter(|o| matches!(o.body, Body::RandomDwelling(_)));
    let mut resolved = 0;
    for (i, placed) in setup.map.objects.iter().enumerate() {
        let Holds::RandomDwelling { factions, levels } = &placed.holds else {
            continue;
        };
        resolved += 1;
        // A random dwelling is an object of the static map, which the build resolves in place.
        let object = &adventure.map().objects()[i];
        let Some(o) = raw.next() else {
            return Err(fail(format!("random dwelling {i} not in the map")));
        };
        let kind = usize::try_from(o.template - 1).unwrap() / BLOCKERS.len();
        let read = dwelling_outcomes(kind, &o.body);
        let ty = object.object_type.unwrap();
        let Some(&(_, _, shift)) = read.iter().find(|&&(named, _, _)| named == ty) else {
            return Err(fail(format!(
                "random dwelling {i} resolved to {ty:?}, not one of the map's {read:?}"
            )));
        };
        let at = TilePos::new(
            u16::from(o.pos.x) + u16::from(shift),
            u16::from(o.pos.y),
            o.pos.z,
        );
        prop_assert_eq!(object.anchor, at, "random dwelling {}", i);
        let outcomes: Vec<ObjectTypeId> = factions
            .iter()
            .flat_map(|&faction| part.random_dwelling(faction).unwrap())
            .zip((1..=7).cycle())
            .filter(|&(_, level)| (levels.0..=levels.1).contains(&level))
            .map(|(&ty, _)| ty)
            .collect();
        prop_assert!(outcomes.contains(&ty), "random dwelling {}: {:?}", i, ty);
        let Some(&ObjectKind::Dwelling {
            blocked,
            visitable,
            shift,
            ..
        }) = part.object_type(ty).map(|t| &t.kind)
        else {
            return Err(fail(format!("random dwelling {i} resolved to no dwelling")));
        };
        prop_assert_eq!((object.blocked, object.visitable), (blocked, visitable));
        let moved = TilePos::new(
            placed.anchor.x + u16::from(shift),
            placed.anchor.y,
            placed.anchor.level,
        );
        prop_assert_eq!(object.anchor, moved);
        prop_assert_eq!(&object.holds, &Holds::Nothing);
        prop_assert_eq!(adventure.owner(i), placed.owner);
    }
    prop_assert!(
        raw.next().is_none(),
        "a random dwelling of the map not imported"
    );
    Ok(resolved)
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
    /// (E08 T86b1; increment 9's design note §15) With an army dialog open (a Leave Guards
    /// dialog's group is its mine's guards), as `Manage(n)`; its open offer (a mine's question)
    /// accepted; on the map the acting player's `n`-th option that enters a mine of its own (which
    /// opens Leave Guards), else one that enters a mine or another ownable object not its own
    /// (which flags it or asks first), else one that enters a weekly object ([`garrison`]); else
    /// as `Join(n)`.
    Garrison(u16),
    /// (E08 T86b1) The acting player's `n`-th listed `BuildBoat` (ADV-SHY-003; modulo their
    /// number), or as `Legal(n)` if it lists none.
    BuildBoat(u16),
    /// (E08 T96b1; increment 10's design note §14) Its open offer (a dwelling's guards' question
    /// or a hire question) accepted, declined in one step of four; its hire dialog answered with
    /// the `n`-th of its legal answers (a `Buy` or `Close`); on the map the acting player's `n`-th
    /// option that enters a dwelling or a Refugee Camp ([`hire`]); else as `Garrison(n)`.
    Hire(u16),
    /// (E09 T9b1; the town epic's design note §18) In a battle as `Fight(n)`; on the map the
    /// acting player's `n`-th option that enters a town, else one that gets nearer to a town not
    /// its own ([`visit_town`]); as `Hire(n)` on a map without towns.
    Town(u16),
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
    /// An answer to a pending decision (E08 T14b), of any kind (T63a: [`ANY_ANSWERS`]).
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
        (0..ANY_ANSWERS.len()).prop_map(|i| Free::Answer(ANY_ANSWERS[i])),
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
    // E08 T86b1: runs of garrisons, three `Step::Garrison`s (a mine's flag or its guards' battle,
    // then an own mine's visit) and then `Step::Garrison`s again, which manage its Leave Guards
    // dialog; single `Step::BuildBoat`s; and runs of 64 ends of turn, so that games reach weeks'
    // and months' starts (16 to 32 days with two to four players), a creature month's monsters
    // among them (increment 9's design note §19).
    let garrison = (any::<u16>(), vec(any::<u16>(), 2..12)).prop_map(|(n, ns)| {
        [Step::Garrison(n), Step::Garrison(n), Step::Garrison(n)]
            .into_iter()
            .chain(ns.into_iter().map(Step::Garrison))
            .collect()
    });
    let boat = any::<u16>().prop_map(|n| vec![Step::BuildBoat(n)]);
    let days = Just(vec![Step::EndTurn; 64]);
    // E08 T96b1: runs of hires, three `Step::Hire`s (a dwelling's or a camp's visit, its question
    // answered, a guards' battle's first command or a dialog's answer) and then `Step::Hire`s
    // again, which play the battle on or answer the hire question and the dialog.
    let hire = (any::<u16>(), vec(any::<u16>(), 2..12)).prop_map(|(n, ns)| {
        [Step::Hire(n), Step::Hire(n), Step::Hire(n)]
            .into_iter()
            .chain(ns.into_iter().map(Step::Hire))
            .collect()
    });
    // E09 T9b1: runs of towns, `Step::Town`s, which walk heroes to the towns' entrances, enter,
    // capture and besiege them, and play the sieges' battles.
    let town = vec(any::<u16>(), 16..64).prop_map(|ns| ns.into_iter().map(Step::Town).collect());
    vec(
        prop_oneof![
            12 => step().prop_map(|step| vec![step]),
            1 => run,
            2 => manage,
            1 => join,
            1 => garrison,
            1 => boat,
            1 => days,
            1 => hire,
            1 => town,
        ],
        0..40,
    )
    .prop_map(|runs: Vec<Vec<Step>>| runs.into_iter().flatten().collect())
}

/// On the map (E08 T19b2): the acting player's open offer declined, its meeting or army dialog
/// closed (E08 T51b), its open level-up's first offer taken (E08 T30), its choice or University
/// answered with the `n`-th of its answers (E08 T63a); else the `n`-th of its
/// options that end nearest to another player's hero on the hero's level, nearer than the hero
/// stands (an attack, on the other hero's tile, is nearest of all; the heroes seek each other as
/// the fork benchmark's do: random play seldom meets); `EndTurn` when none gets nearer.
/// The acting player's answer to its open decision but a battle, if it has one (E08 T63a): an
/// offer declined, a level-up's first offer taken, a meeting or an army dialog closed, a choice,
/// a University or (E08 T95b) a hire dialog answered with the `n`-th of its legal answers (a hire
/// dialog's: a `Buy` of each creature's most, then `Close`).
fn settle(state: &GameState, legal: &[GameCommand], n: u16) -> Option<GameCommand> {
    let actor = acting(state);
    if state.decision(actor).is_some() {
        return Some(ANSWERS[1].clone());
    }
    if state.level_up(actor).is_some() {
        return Some(LEVEL_ANSWERS[0].clone());
    }
    if state.meeting(actor).is_some() || state.army_dialog(actor).is_some() {
        return Some(CLOSE);
    }
    (state.choice(actor).is_some() || state.university(actor).is_some() || hiring(state, actor))
        .then(|| legal[usize::from(n) % legal.len()].clone())
}

/// Whether `player`'s first open decision is a hire dialog (GAM-DEC-008, E08 T95b): an open
/// decision of none of the other kinds, as `GameState` shows a hire dialog only through its
/// owner's observation ([`hire_view`]).
fn hiring(state: &GameState, player: PlayerId) -> bool {
    state.has_decision(player)
        && state.battle(player).is_none()
        && state.decision(player).is_none()
        && state.level_up(player).is_none()
        && state.meeting(player).is_none()
        && state.army_dialog(player).is_none()
        && state.choice(player).is_none()
        && state.university(player).is_none()
}

/// `player`'s open hire dialog as its observation shows it (GAM-DEC-008, E08 T95b): its army, its
/// source by type and anchor, and the source's creatures with their counts and the cost of one;
/// none without one.
fn hire_view(state: &GameState, rules: &GameRuleset, player: PlayerId) -> Option<HireView> {
    if !hiring(state, player) {
        return None;
    }
    match observe(state, rules, Viewer::Player(player)).decision? {
        DecisionView::Hire(view) => Some(view),
        _ => None,
    }
}

fn seek(state: &GameState, legal: &[GameCommand], n: u16) -> GameCommand {
    let actor = acting(state);
    if let Some(answer) = settle(state, legal, n) {
        return answer;
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
/// not manage (a battle, an offer or a level-up open, the game waiting for another player; since
/// E08 T64b's plans a choice or a University open, GAM-DEC-005, GAM-DEC-006; since E08 T95b a
/// hire dialog, GAM-DEC-008).
fn managed(state: &GameState) -> Vec<Vec<ArmyRef>> {
    let player = acting(state);
    if actor(state) != Actor::Player(player)
        || state.battle(player).is_some()
        || state.decision(player).is_some()
        || state.choice(player).is_some()
        || state.university(player).is_some()
        || hiring(state, player)
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

/// The stack at `at` (E08 T52): in its hero's army on the map, in the acting player's army
/// dialog's group, or in a town's garrison (E09 T8b2b).
fn stack_at(state: &GameState, at: ArmySlot) -> Option<h3_contracts::ArmyStack> {
    let adventure = state.adventure();
    let army = match at.army {
        ArmyRef::Hero(hero) => adventure.hero(hero).map(|h| &h.army[..]),
        ArmyRef::Group => state.army_dialog(acting(state)).map(|(_, group)| group),
        ArmyRef::Garrison(town) => adventure
            .map()
            .town_object(town)
            .and_then(|index| adventure.town_garrison(index)),
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
            return ANSWERS[0].clone();
        }
        Some(_) => return ANSWERS[1].clone(),
        None => {}
    }
    if battle_of(state, self::actor(state)).is_none() && !state.has_decision(actor) {
        let adventure = state.adventure();
        // E08 T86b1: a monster added in play too.
        let monster = |pos: TilePos| {
            visited_at(state, pos).is_some_and(|i| adventure.object(i).unwrap().monster.is_some())
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

/// [`Step::Garrison`] (E08 T86b1): with an army dialog open, the `n`-th of [`managing`] (`Close` if
/// none, and in one step of eight); its open offer accepted (a mine's guards fought); on the map
/// the `n`-th of its options that enter a mine of its own, else one that enters an ownable object
/// not its own, else one that enters a weekly object (a second visit in a week finds it empty);
/// else as [`join`].
fn garrison(state: &GameState, rules: &GameRuleset, legal: &[GameCommand], n: u16) -> GameCommand {
    let actor = acting(state);
    if battle_of(state, self::actor(state)).is_none() {
        // One step of eight closes the dialog, so that a mine keeps the group as its guards.
        if state.army_dialog(actor).is_some() {
            let managed = pick(&managing(state, rules), n).filter(|_| n % 8 != 7);
            return managed.unwrap_or(CLOSE);
        }
        if state.decision(actor).is_some() {
            return ANSWERS[0].clone();
        }
    }
    if battle_of(state, self::actor(state)).is_none() && !state.has_decision(actor) {
        let (adventure, part) = (state.adventure(), rules.adventure());
        // 0: an own mine; 1: an ownable object not its own; 2: a weekly object.
        let entered = |aim: u8| -> Vec<&GameCommand> {
            let wanted = |pos: TilePos| {
                visited_at(state, pos).is_some_and(|i| {
                    let ty = adventure.object(i).and_then(|o| o.object_type);
                    let kind = ty.and_then(|ty| part.object_type(ty)).map(|t| &t.kind);
                    let mine = matches!(
                        kind,
                        Some(ObjectKind::Mine { .. } | ObjectKind::AbandonedMine { .. })
                    );
                    let other = matches!(
                        kind,
                        Some(ObjectKind::Lighthouse { .. } | ObjectKind::Shipyard { .. })
                    );
                    let owned = adventure.owner(i) == Some(actor);
                    match aim {
                        0 => mine && owned,
                        1 => (mine || other) && !owned,
                        _ => adventure.weekly(i).is_some(),
                    }
                })
            };
            legal
                .iter()
                .filter(|c| {
                    as_move(c)
                        .and_then(|(_, path)| path.last())
                        .is_some_and(|&to| wanted(to))
                })
                .collect()
        };
        if let Some(visits) = (0..3).map(entered).find(|v| !v.is_empty()) {
            return visits[usize::from(n) % visits.len()].clone();
        }
    }
    join(state, rules, legal, n)
}

/// [`Step::Hire`] (E08 T96b1; increment 10's design note §14): not in a battle, its open offer
/// accepted, declined when `n % 4 == 3` (a guards' question or a hire question, both ways); its
/// hire dialog answered with the `n`-th of its legal answers (each creature's most, `Close`); on the
/// map the `n`-th of its options that enter a dwelling (any owner's: a flag, a guards' question
/// or a hire question) or a Refugee Camp; else as [`garrison`].
fn hire(state: &GameState, rules: &GameRuleset, legal: &[GameCommand], n: u16) -> GameCommand {
    let actor = acting(state);
    if battle_of(state, self::actor(state)).is_none() {
        if state.decision(actor).is_some() {
            return ANSWERS[usize::from(n % 4 == 3)].clone();
        }
        if hiring(state, actor) {
            return legal[usize::from(n) % legal.len()].clone();
        }
        if !state.has_decision(actor) {
            let (adventure, part) = (state.adventure(), rules.adventure());
            let wanted = |pos: TilePos| {
                visited_at(state, pos).is_some_and(|i| {
                    let ty = adventure.object(i).and_then(|o| o.object_type);
                    matches!(
                        ty.and_then(|ty| part.object_type(ty)).map(|t| &t.kind),
                        Some(ObjectKind::Dwelling { .. } | ObjectKind::RefugeeCamp { .. })
                    )
                })
            };
            let visits: Vec<&GameCommand> = legal
                .iter()
                .filter(|c| {
                    as_move(c)
                        .and_then(|(_, path)| path.last())
                        .is_some_and(|&to| wanted(to))
                })
                .collect();
            if !visits.is_empty() {
                return visits[usize::from(n) % visits.len()].clone();
            }
        }
    }
    garrison(state, rules, legal, n)
}

/// [`Step::Town`] (E09 T9b1; the town epic's design note §18): in a battle as [`fight`] (a
/// siege's too, either side's); an open decision settled as [`seek`] settles it; on the map the
/// `n`-th of its options that end on a town's entrance (its own entered, TWN-VIS-001; another
/// player's or a neutral one captured or besieged, TWN-CAP-001, TWN-SIE-001; another team's
/// visitor besieged with the garrison merged, TWN-SIE-002), in three steps of four of a town not
/// its own if it has such options; else the `n`-th of those that end nearer to the entrance of a
/// town not its own on the hero's level than the hero stands; else `EndTurn`, so that the turns
/// pass until a hero gets there. On a map without towns, as [`hire`].
fn visit_town(
    state: &GameState,
    rules: &GameRuleset,
    legal: &[GameCommand],
    n: u16,
) -> GameCommand {
    let (adventure, player) = (state.adventure(), acting(state));
    let map = adventure.map();
    if map.towns().is_empty() {
        return hire(state, rules, legal, n);
    }
    if battle_of(state, actor(state)).is_some() {
        return fight(state, rules, legal, n);
    }
    if let Some(answer) = settle(state, legal, n) {
        return answer;
    }
    let towns: Vec<(TilePos, bool)> = map
        .towns()
        .iter()
        .filter_map(|&i| {
            let entrance = map.objects()[i].entrance(map)?;
            Some((entrance, adventure.owner(i) == Some(player)))
        })
        .collect();
    let moves: Vec<(&GameCommand, HeroId, &[TilePos])> = legal
        .iter()
        .filter_map(|c| as_move(c).map(|(hero, path)| (c, hero, path)))
        .collect();
    let entering = |own: bool| -> Vec<&GameCommand> {
        moves
            .iter()
            .filter(|(_, _, path)| {
                path.last()
                    .is_some_and(|to| towns.iter().any(|&(e, o)| e == *to && (own || !o)))
            })
            .map(|&(c, _, _)| c)
            .collect()
    };
    let mut visits = entering(n % 4 == 3);
    if visits.is_empty() {
        visits = entering(true);
    }
    if !visits.is_empty() {
        return visits[usize::from(n) % visits.len()].clone();
    }
    let distance = |pos: TilePos| {
        towns
            .iter()
            .filter(|&&(e, own)| !own && e.level == pos.level)
            .map(|(e, _)| e.x.abs_diff(pos.x).max(e.y.abs_diff(pos.y)))
            .min()
    };
    let mut best: Vec<&GameCommand> = Vec::new();
    let mut nearest = u16::MAX;
    for &(command, hero, path) in &moves {
        let here = adventure.hero(hero).and_then(|h| distance(h.pos));
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

/// On the map (E08 T52): the acting player's open offer declined, its level-up's first offer
/// taken, its meeting or army dialog managed (the `n`-th of [`managing`], `Close` if none), its
/// choice or University answered as [`settle`] does (E08 T63a);
/// else the `n`-th of its options that end nearest to another of its own heroes on the hero's
/// level, nearer than the hero stands (a step onto the other's tile meets it, nearest of all);
/// `EndTurn` when none gets nearer. In a battle, as `Legal(n)`.
fn meet(state: &GameState, rules: &GameRuleset, legal: &[GameCommand], n: u16) -> GameCommand {
    let actor = acting(state);
    if battle_of(state, self::actor(state)).is_some() {
        return legal[usize::from(n) % legal.len()].clone();
    }
    if state.meeting(actor).is_some() || state.army_dialog(actor).is_some() {
        return pick(&managing(state, rules), n).unwrap_or(CLOSE);
    }
    if let Some(answer) = settle(state, legal, n) {
        return answer;
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
        GameCommand::Adventure(AdventureCommand::BuildBoat { .. })
        | GameCommand::EndTurn
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
/// start). Since E08 T64b an event stops the move on its tile (ADV-MOV-011), fired or held for
/// the guard of its tile: the rest of the path is dropped.
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
    // (E08 T75b) A sea pickup leaves at its visit (ADV-PIK-014 to -017): its removal follows.
    let leaves = steps + 2 == kinds.len()
        && matches!(kinds.last(), Some(AdventureEventKind::ObjectRemoved { .. }));
    // (E08 T86b1) A flag reveals around its object (ADV-OWN-001): its `TilesRevealed` follows,
    // then a Shipyard's refusal (ADV-SHY-002) or a mine's guards changed by the visit; a weekly
    // object's mark comes before its emptiness (ADV-VIS-006).
    let visit = &kinds[steps..];
    let flagged = matches!(
        visit,
        [
            AdventureEventKind::ObjectFlagged { .. },
            AdventureEventKind::TilesRevealed { .. },
            rest @ ..
        ] if rest.len() <= 1 && rest.iter().all(|k| matches!(
            k,
            AdventureEventKind::ShipyardBlocked { .. } | AdventureEventKind::GuardsChanged { .. }
        ))
    );
    let marked = matches!(
        visit,
        [
            AdventureEventKind::ObjectMarked { .. },
            AdventureEventKind::ObjectEmpty { .. }
        ]
    );
    // (E09 T9b1) An undefended town's capture reveals around it too (TWN-CAP-001): its
    // `TilesRevealed` follows.
    let captured = matches!(
        visit,
        [
            AdventureEventKind::TownCaptured { .. },
            AdventureEventKind::TilesRevealed { .. }
        ]
    );
    prop_assert!(
        kinds.len() <= steps + 1 || leaves || flagged || marked || captured,
        "{:?}",
        kinds
    );
    // A live event on the last tile stepped onto stopped the move: fired, or held on a tile a
    // monster guards, whose visit comes first (E08 T63b, GAM-DEC-007; the plans' events fire for
    // every colour, computers too).
    let adventure = before.adventure();
    let event = |at: TilePos| {
        let events = adventure.map().events_at(at);
        events.iter().any(|&i| !adventure.is_removed(i))
    };
    let fired = matches!(
        kinds.get(steps),
        Some(AdventureEventKind::EventFired { .. })
    );
    let on_event = steps > 0 && event(path[steps - 1]);
    prop_assert!(!fired || on_event, "{:?}", kinds);
    let stopped = on_event && steps < path.len();
    let stays = steps + 1 == path.len() && !stopped;
    prop_assert!(
        steps == path.len() || stays || stopped,
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
            .is_some_and(|tile| tile.visitable)
            || added_at(before, to);
        // Another hero is visited from the tile before: a battle or, an own one, a meeting
        // (GAM-MET-001, E08 T50).
        let heroes = before.adventure().heroes();
        let mover = heroes.iter().find(|h| h.pos == start);
        let other = mover.is_some_and(|m| heroes.iter().any(|h| h.pos == to && h.id != m.id));
        prop_assert!(visitable || other, "{:?} {:?}", path, events);
    }
    let walked = &path[..steps];
    Ok((walked.last().copied().unwrap_or(start), stays))
}

/// The tile of the event that `events` tell fired, if one did (E08 T64b; ADV-OBJ-007).
fn fired_at(events: &[GameEvent]) -> Option<TilePos> {
    events.iter().find_map(|e| match *e {
        GameEvent::Adventure(AdventureEvent {
            kind: AdventureEventKind::EventFired { tile, .. },
            ..
        }) => Some(tile),
        _ => None,
    })
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

/// A move's rules at sea (E08 T75b; increment 8's design note §7, §8, §15): a hero not in a boat
/// steps onto no water tile an object still on the map is visited on, nor visits one from the shore
/// (ADV-PIK-018, D-46: only a boat visits the sea's objects, and no flight passes one), but where a
/// hero in a boat stands, who is met there (ADV-SEA-005); a boarding or a landing converts the
/// hero's points (ADV-SEA-002, -003; D-41): to at most the new maximum with free boarding
/// (HER-ADV-009, the Admiral's Hat), to 0 without.
fn check_sea_move(
    rules: &GameRuleset,
    (before, after): (&GameState, &GameState),
    hero: HeroId,
    path: &[TilePos],
    events: &[GameEvent],
) -> Result<(), TestCaseError> {
    let afloat = before
        .adventure()
        .hero(hero)
        .is_some_and(|h| h.boat.is_some());
    let heroes = before.adventure().heroes();
    for &tile in path.iter().filter(|_| !afloat) {
        // A hero in a boat on the tile is met there, whatever lies under it (ADV-SEA-005: from
        // the shore too; D-46 leaves such a tile open), and an empty boat on it is boarded
        // (ADV-SEA-002; D-46 leaves a boat's tile open too): a key holder's boat may lie on its
        // border gate's water tile, which the gate lets it end on (ADV-VIS-005), and a walker
        // boards it there from the shore (E08 decision 216).
        let met = heroes.iter().any(|h| h.pos == tile && h.id != hero)
            || before.adventure().is_empty_boat(tile);
        let entrance = is_water(before, rules, tile) && live_bits(before, tile).1 && !met;
        let adventure = before.adventure();
        let objects: Vec<_> = (adventure.map().objects().iter().enumerate())
            .filter(|&(i, o)| !adventure.is_removed(i) && o.covers(tile).1)
            .map(|(_, o)| {
                o.object_type
                    .and_then(|ty| rules.adventure().object_type(ty))
                    .map(|ty| ty.key.to_string())
            })
            .collect();
        let movement = h3_heroes::movement(before.hero(hero).unwrap(), rules.core());
        prop_assert!(
            !entrance,
            "{:?} onto {:?} by {:?}: {:?} {:?}",
            hero,
            tile,
            path,
            objects,
            movement
        );
    }
    let converted = events.iter().any(|e| {
        matches!(e, GameEvent::Adventure(AdventureEvent {
            kind: AdventureEventKind::Boarded { hero: h, .. }
                | AdventureEventKind::Landed { hero: h, .. },
            ..
        }) if *h == hero)
    });
    if let Some(on) = after.adventure().hero(hero).filter(|_| converted) {
        let details = after.hero(hero).unwrap();
        if h3_heroes::free_boarding(details, rules.core()) {
            prop_assert!(on.movement <= on.max_movement, "{:?}", on);
        } else {
            prop_assert_eq!(on.movement, 0, "{:?}", on);
        }
    }
    Ok(())
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
        Step::Garrison(n) => (who, garrison(state, rules, legal, *n)),
        Step::Hire(n) => (who, hire(state, rules, legal, *n)),
        Step::Town(n) => (who, visit_town(state, rules, legal, *n)),
        Step::BuildBoat(n) => {
            let boats: Vec<&GameCommand> = legal
                .iter()
                .filter(|c| {
                    matches!(
                        c,
                        GameCommand::Adventure(AdventureCommand::BuildBoat { .. })
                    )
                })
                .collect();
            let command = match boats.len() {
                0 => &legal[usize::from(*n) % legal.len()],
                len => boats[usize::from(*n) % len],
            };
            (who, command.clone())
        }
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

/// The answers to a pending offer, as `legal_commands` lists them (GAM-DEC-001).
const ANSWERS: [GameCommand; 2] = [
    GameCommand::Answer(Answer::Offer(Consent::Accept)),
    GameCommand::Answer(Answer::Offer(Consent::Decline)),
];

/// The answers to a level-up, as `legal_commands` lists them (GAM-DEC-002, E08 T30).
const LEVEL_ANSWERS: [GameCommand; 2] = [
    GameCommand::Answer(Answer::LevelUp(SkillOffer::First)),
    GameCommand::Answer(Answer::LevelUp(SkillOffer::Second)),
];

/// The answer that closes a meeting or an army dialog (GAM-DEC-003, GAM-DEC-004; E08 T63a).
const CLOSE: GameCommand = GameCommand::Answer(Answer::Close);

/// An answer of every kind (E08 T63a), the University's purchases of the slots a University may
/// have and one beyond: the free answers' pool.
const ANY_ANSWERS: [Answer; 17] = [
    Answer::Offer(Consent::Accept),
    Answer::Offer(Consent::Decline),
    Answer::LevelUp(SkillOffer::First),
    Answer::LevelUp(SkillOffer::Second),
    Answer::Chest(ChestChoice::Gold),
    Answer::Chest(ChestChoice::Experience),
    Answer::Arena(ArenaChoice::Attack),
    Answer::Arena(ArenaChoice::Defense),
    Answer::School(SchoolChoice::First),
    Answer::School(SchoolChoice::Second),
    Answer::School(SchoolChoice::Leave),
    Answer::University(UniversityAnswer::Buy(0)),
    Answer::University(UniversityAnswer::Buy(1)),
    Answer::University(UniversityAnswer::Buy(3)),
    Answer::University(UniversityAnswer::Buy(4)),
    Answer::University(UniversityAnswer::Leave),
    Answer::Close,
];

/// The answers to the acting player's open choice or University as `legal_commands` must list
/// them (GAM-DEC-005, GAM-DEC-006; E08 T63a), from the decision and the slots' states read
/// independently: a chest's gold then experience, the Arena's two skills, a School's two and
/// leaving, a University's purchase of each slot open for the hero while the treasury holds the
/// price, then leaving. None without such a decision.
fn choice_answers(state: &GameState, rules: &GameRuleset, player: PlayerId) -> Vec<GameCommand> {
    let answers: Vec<Answer> = if let Some(choice) = state.choice(player) {
        match choice.kind {
            ChoiceKind::Chest { .. } => vec![
                Answer::Chest(ChestChoice::Gold),
                Answer::Chest(ChestChoice::Experience),
            ],
            ChoiceKind::Primary { leave: false, .. } => vec![
                Answer::Arena(ArenaChoice::Attack),
                Answer::Arena(ArenaChoice::Defense),
            ],
            ChoiceKind::Primary { leave: true, .. } => vec![
                Answer::School(SchoolChoice::First),
                Answer::School(SchoolChoice::Second),
                Answer::School(SchoolChoice::Leave),
            ],
        }
    } else if let Some(visit) = state.university(player) {
        let treasury = state.treasury(player).unwrap();
        let pays = visit
            .price
            .iter()
            .all(|&(r, amount)| treasury[usize::from(r.raw())] >= amount);
        let hero = state.hero(visit.hero).unwrap();
        (0u8..)
            .zip(&visit.skills)
            .filter(|&(_, &skill)| {
                pays && h3_heroes::university_slot(hero, rules.core(), skill) == SlotState::Open
            })
            .map(|(slot, _)| Answer::University(UniversityAnswer::Buy(slot)))
            .chain([Answer::University(UniversityAnswer::Leave)])
            .collect()
    } else {
        Vec::new()
    };
    answers.into_iter().map(GameCommand::Answer).collect()
}

/// The creatures the hire dialog's source of `object_type` on `anchor` sells on `state`, with
/// their counts as they stand (ADV-HIR-002, E08 T95b), read apart from the dialog: a dwelling's
/// kind's creatures in its order with the dwelling's counts, a Refugee Camp's week's creature and
/// count.
fn hire_stock(
    rules: &GameRuleset,
    state: &GameState,
    (object_type, anchor): (ObjectTypeId, TilePos),
) -> Vec<(CreatureTypeId, u32)> {
    let index = object_index(state, object_type, anchor).unwrap();
    let adventure = state.adventure();
    match rules.adventure().object_type(object_type).map(|t| &t.kind) {
        Some(ObjectKind::Dwelling { creatures, .. }) => {
            let counts = adventure.dwelling_counts(index).unwrap();
            creatures
                .iter()
                .copied()
                .zip(counts.iter().copied())
                .collect()
        }
        Some(ObjectKind::RefugeeCamp { .. }) => adventure.camp(index).into_iter().collect(),
        _ => Vec::new(),
    }
}

/// The cost of one `creature` (ADV-HIR-002, E08 T95b): its core cost's resources of more than 0,
/// in the core's order.
fn unit_cost(rules: &GameRuleset, creature: CreatureTypeId) -> Vec<(ResourceId, u32)> {
    let cost = &rules.core().creature(creature).unwrap().cost;
    let mut cost: Vec<(ResourceId, u32)> = cost.iter().copied().filter(|&(_, n)| n > 0).collect();
    cost.sort_unstable();
    cost
}

/// The most of `creature` of a source holding `count` of it that `player` may buy now into
/// `hero`'s army (GAM-DEC-008, ADV-ARM-007; E08 T95b), by an independent reading: the least of
/// the count, the treasury divided by the cost of one for each resource it costs, and the room, a
/// stack's most with a slot free, else the most less the least count among the army's stacks of
/// the creature, else 0.
fn hire_most(
    rules: &GameRuleset,
    state: &GameState,
    (player, hero): (PlayerId, HeroId),
    (creature, count): (CreatureTypeId, u32),
) -> u32 {
    let params = rules.adventure().params();
    let (slots, most) = (params.army_slots(), params.max_stack_count());
    let army = &state.adventure().hero(hero).unwrap().army;
    let room = if army.len() < usize::from(slots) {
        most
    } else {
        army.iter()
            .filter(|s| s.creature == creature)
            .map(|s| most.saturating_sub(s.count))
            .max()
            .unwrap_or(0)
    };
    let treasury = state.treasury(player).unwrap();
    unit_cost(rules, creature)
        .iter()
        .map(|&(resource, each)| treasury[usize::from(resource.raw())] / each)
        .fold(count.min(room), u32::min)
}

/// The answers to `player`'s open hire dialog as `legal_commands` must list them (GAM-DEC-008,
/// E08 T95b), from the source's stock, the treasury and the army read independently
/// ([`hire_stock`], [`hire_most`]): a `Buy` of the most of each creature in the source's order
/// whose most is at least 1, then `Close`. None without a hire dialog.
fn hire_answers(state: &GameState, rules: &GameRuleset, player: PlayerId) -> Vec<GameCommand> {
    let Some(view) = hire_view(state, rules, player) else {
        return Vec::new();
    };
    let HireTarget::Hero(hero) = view.target;
    hire_stock(rules, state, (view.object_type, view.anchor))
        .into_iter()
        .map(|(creature, count)| {
            (
                creature,
                hire_most(rules, state, (player, hero), (creature, count)),
            )
        })
        .filter(|&(_, most)| most > 0)
        .map(|(creature, amount)| Answer::Hire(HireAnswer::Buy { creature, amount }))
        .chain([Answer::Hire(HireAnswer::Close)])
        .map(GameCommand::Answer)
        .collect()
}

/// An open hire dialog's answers (GAM-DEC-008, ADV-HIR-002, ADV-ARM-007; E08 T95b, increment 10's
/// design note §14), `view` the actor's: the view shows the source's stock as it stands, each
/// creature with the cost of one; [`hire_range`] is 1 to each creature's most ([`hire_most`]),
/// empty for a creature the source does not sell; a `Buy` of 0 or of one past the range's end, and
/// one of a creature the source does not sell, is rejected with the state unchanged; every listed
/// `Buy` applies, pays its price (the cost of one times the amount, `ResourcePaid` per resource),
/// joins the amount to the army where ADV-ARM-007's room is (its first stack of the creature that
/// takes them within a stack's most, else its first free slot; `CreaturesJoined` to the owner
/// alone) and lowers the source's count by it, the dialog closing (`DecisionClosed`) exactly when
/// its source sells one creature; `Close` closes it and does nothing else. What follows a close
/// (E08 T96b1) comes from the visit's held rests: on a tile a monster guards, its visit, starting
/// with its offer or its battle (GAM-DEC-007, ADV-MON-006), which leave the treasury and the army
/// as the close left them; on any other tile, nothing.
fn check_hire(
    state: &GameState,
    rules: &GameRuleset,
    (actor, view): (PlayerId, &HireView),
    legal: &[GameCommand],
) -> Result<(), TestCaseError> {
    let HireTarget::Hero(hero) = view.target;
    let source = (view.object_type, view.anchor);
    let stock = hire_stock(rules, state, source);
    let shown: Vec<(CreatureTypeId, u32)> = view
        .creatures
        .iter()
        .map(|c| (c.creature, c.count))
        .collect();
    prop_assert_eq!(&shown, &stock);
    for shown in &view.creatures {
        prop_assert_eq!(&shown.cost, &unit_cost(rules, shown.creature));
    }
    let probe = |command: &GameCommand| {
        let mut probe = state.clone();
        let mut events = Vec::new();
        let command = command.clone();
        let result = apply(
            &mut probe,
            rules,
            Actor::Player(actor),
            command,
            &mut events,
        );
        (probe, events, result)
    };
    let hash = state.state_hash();
    let buy =
        |creature, amount| GameCommand::Answer(Answer::Hire(HireAnswer::Buy { creature, amount }));
    let mut refused = Vec::new();
    for &(creature, count) in &stock {
        let most = hire_most(rules, state, (actor, hero), (creature, count));
        let range = hire_range(state, rules, actor, creature);
        prop_assert_eq!(range.clone(), 1..=most, "{:?}", creature);
        refused.extend([buy(creature, 0), buy(creature, range.end() + 1)]);
    }
    let other = (0..rules.core().creature_count())
        .filter_map(CreatureTypeId::from_index)
        .find(|&c| stock.iter().all(|&(sold, _)| sold != c));
    if let Some(other) = other {
        prop_assert!(hire_range(state, rules, actor, other).is_empty());
        refused.push(buy(other, 1));
    }
    for command in &refused {
        let (after, events, result) = probe(command);
        prop_assert!(result.is_err(), "{:?} accepted", command);
        prop_assert_eq!(&after, state);
        prop_assert_eq!(after.state_hash(), hash);
        prop_assert!(events.is_empty());
    }
    let single = stock.len() == 1;
    let (slots, most) = {
        let params = rules.adventure().params();
        (params.army_slots(), params.max_stack_count())
    };
    // E08 T96b1: what may follow the dialog's close, the visit's held rests (GAM-DEC-007,
    // ADV-MON-006): the visit of the monster guarding the hero's tile (an added one's too), its
    // offer to the hero (to join, to flee) or its battle with the hero first; nothing on a tile
    // no monster guards.
    let guard = state.adventure().hero(hero).and_then(|h| {
        let index = state.adventure().guard_of(rules.adventure(), h.pos)?;
        state.adventure().object(index)
    });
    let held = |rest: &[GameEvent]| match (guard, rest.first()) {
        (_, None) => true,
        (
            Some(o),
            Some(&GameEvent::OfferOpened {
                player,
                hero: by,
                object_type,
                anchor,
            }),
        ) => (player, by, Some(object_type), anchor) == (actor, hero, o.object_type, o.anchor),
        (Some(o), Some(&GameEvent::BattleStarted { heroes, tile, .. })) => {
            let (blocks, visited) = o.covers(tile);
            heroes == [Some(hero), None] && (blocks || visited)
        }
        _ => false,
    };
    for command in legal {
        let (after, events, result) = probe(command);
        prop_assert!(result.is_ok(), "legal {:?} rejected: {:?}", command, result);
        let GameCommand::Answer(Answer::Hire(HireAnswer::Buy { creature, amount })) = *command
        else {
            prop_assert_eq!(
                command,
                &GameCommand::Answer(Answer::Hire(HireAnswer::Close))
            );
            // E08 T96b1: what follows the close comes from the visit's held rests (GAM-DEC-007,
            // ADV-MON-006: the visit of a monster guarding the dwelling's entrance, held until the
            // dialog closes), as after a meeting's or an army dialog's ([`check_legal`]).
            prop_assert_eq!(
                events.first(),
                Some(&GameEvent::DecisionClosed { player: actor })
            );
            prop_assert!(held(&events[1..]), "{:?} after {:?}", events, command);
            prop_assert!(hire_view(&after, rules, actor).is_none());
            continue;
        };
        let price: Vec<(ResourceId, u32)> = unit_cost(rules, creature)
            .into_iter()
            .map(|(resource, each)| (resource, each * amount))
            .collect();
        let mut expected: Vec<GameEvent> = price
            .iter()
            .map(|&(resource, amount)| GameEvent::ResourcePaid {
                player: actor,
                resource,
                amount,
            })
            .collect();
        expected.push(GameEvent::Adventure(AdventureEvent {
            kind: AdventureEventKind::CreaturesJoined {
                hero,
                creature,
                count: amount,
            },
            seen_by: Viewers::NONE.with(actor),
        }));
        expected.extend(single.then_some(GameEvent::DecisionClosed { player: actor }));
        // E08 T96b1: after a purchase that closes the dialog, its visit's held rests (a guarding
        // monster's visit, GAM-DEC-007, ADV-MON-006: `held`), which start with an offer or a
        // battle and so leave the treasury and the army as the purchase left them.
        prop_assert_eq!(events.get(..expected.len()), Some(&expected[..]));
        let rest = &events[expected.len()..];
        prop_assert!(single || rest.is_empty(), "{:?}", events);
        prop_assert!(held(rest), "{:?} after {:?}", events, command);
        let mut left = stock.clone();
        let place = left.iter().position(|&(sold, _)| sold == creature).unwrap();
        left[place].1 -= amount;
        prop_assert_eq!(hire_stock(rules, &after, source), left);
        let open = hire_view(&after, rules, actor);
        prop_assert_eq!(open.is_none(), single, "{:?}", command);
        if let Some(open) = open {
            prop_assert_eq!(
                (open.target, open.object_type, open.anchor),
                (view.target, source.0, source.1)
            );
        }
        let mut treasury = state.treasury(actor).unwrap().to_vec();
        for &(resource, paid) in &price {
            treasury[usize::from(resource.raw())] -= paid;
        }
        prop_assert_eq!(after.treasury(actor).unwrap(), &treasury[..]);
        let mut army = state.adventure().hero(hero).unwrap().army.clone();
        let fits = |s: &h3_contracts::ArmyStack| {
            s.creature == creature && s.count.saturating_add(amount) <= most
        };
        if let Some(at) = army.iter().position(fits) {
            army[at].count += amount;
        } else {
            let slot = (0..slots).find(|&slot| army.iter().all(|s| s.slot != slot));
            let slot = slot.ok_or_else(|| fail(format!("{command:?} bought without room")))?;
            let at = army
                .iter()
                .position(|s| s.slot > slot)
                .unwrap_or(army.len());
            army.insert(
                at,
                h3_contracts::ArmyStack {
                    slot,
                    creature,
                    count: amount,
                },
            );
        }
        prop_assert_eq!(&after.adventure().hero(hero).unwrap().army, &army);
    }
    Ok(())
}

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
/// one else lists a command. A hire dialog (E08 T95b) lists its answers ([`hire_answers`]), each
/// checked by [`check_hire`].
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
    // E08 T51b: a meeting or an army dialog, closed by `Close` alone (T63a).
    let exchanging = state.meeting(actor).is_some() || state.army_dialog(actor).is_some();
    // E08 T63a: a choice or a University.
    let choosing = state.choice(actor).is_some() || state.university(actor).is_some();
    // E08 T95b: a hire dialog, as its owner sees it.
    let hiring = hire_view(state, rules, actor);
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
                vec![CLOSE]
            } else if choosing {
                choice_answers(state, rules, player)
            } else if hiring.is_some() {
                hire_answers(state, rules, player)
            } else {
                // E08 T86b1: then the Shipyards' boats it may build (ADV-SHY-003).
                core::iter::once(GameCommand::EndTurn)
                    .chain(buildable(state, rules, player))
                    .collect()
            };
            prop_assert_eq!(listed, expected);
        }
        for (hero, reach) in reaches {
            let moves = legal_moves(state, rules, Actor::Player(player), *hero);
            let own = me
                && !decided
                && !exchanging
                && !choosing
                && hiring.is_none()
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
            let flees = *answer == ANSWERS[1]
                && matches!(open.kind, OfferKind::Join | OfferKind::JoinForGold { .. })
                && next.is_some_and(|n| {
                    (n.object_type, n.anchor, &n.kind)
                        == (open.object_type, open.anchor, &OfferKind::Pursue)
                });
            let guard = state.adventure().hero(open.hero).and_then(|hero| {
                let index = state.adventure().guard_of(rules.adventure(), hero.pos)?;
                // E08 T85b: a monster added in play guards too.
                let o = state.adventure().object(index)?;
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
        prop_assert_eq!(legal, &[CLOSE][..]);
        let mut probe = state.clone();
        let mut events = Vec::new();
        let result = apply(&mut probe, rules, Actor::Player(actor), CLOSE, &mut events);
        prop_assert!(result.is_ok(), "Close rejected: {:?}", result);
        // The meeting or the army dialog closes, the dialog's group lost stack by stack
        // (GAM-DEC-003, GAM-DEC-004); what follows comes from the visit's held rests and may open
        // another decision (E08 T63b, GAM-DEC-007: the guard's visit, a record's experience with
        // its levels, an event on the hero's tile with its guards' battle; E08 T71: a guard's join
        // without room, then the event it guarded and its guards' battle).
        // E08 T86b1: a Leave Guards dialog's group is kept as the mine's guards, lost never
        // ([`check_mine_guards`]).
        let lost = state
            .army_dialog(actor)
            .filter(|_| state.leave_guards(actor).is_none());
        let closing: Vec<GameEvent> = core::iter::once(GameEvent::DecisionClosed { player: actor })
            .chain(lost.into_iter().flat_map(|(hero, group)| {
                group.iter().map(move |&stack| GameEvent::GroupLost {
                    player: actor,
                    hero,
                    stack,
                })
            }))
            .collect();
        prop_assert_eq!(events.get(..closing.len()), Some(&closing[..]));
        prop_assert!(!probe.has_decision(actor) || events.len() > closing.len());
        return Ok(());
    }
    if choosing {
        // E08 T63a: every listed answer applies; a purchase keeps the University open, every
        // other answer closes the decision (what follows may open another: a level-up, a guard's
        // offer or battle).
        prop_assert_eq!(legal, &choice_answers(state, rules, actor)[..]);
        for answer in legal {
            let mut probe = state.clone();
            let result = apply(
                &mut probe,
                rules,
                Actor::Player(actor),
                answer.clone(),
                &mut Vec::new(),
            );
            prop_assert!(result.is_ok(), "legal {:?} rejected: {:?}", answer, result);
            let buys = matches!(
                answer,
                GameCommand::Answer(Answer::University(UniversityAnswer::Buy(_)))
            );
            prop_assert_eq!(probe.university(actor).is_some(), buys);
            prop_assert!(probe.choice(actor).is_none());
        }
        return Ok(());
    }
    // E08 T95b: a hire dialog's answers, each applied ([`check_hire`]).
    if let Some(view) = &hiring {
        prop_assert_eq!(legal, &hire_answers(state, rules, actor)[..]);
        return check_hire(state, rules, (actor, view), legal);
    }
    prop_assert_eq!(legal.first(), Some(&GameCommand::EndTurn));
    check_build_boat(state, rules, actor, legal)?;
    for command in &legal[1..] {
        if matches!(
            command,
            GameCommand::Adventure(AdventureCommand::BuildBoat { .. })
        ) {
            continue;
        }
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
        // E08 T64b: an event on the way stops the move on its tile (fired, or held where a
        // monster guards it: `Reach` never reads events, increment 7's design note §12), with the
        // points the query reports there (ADV-MOV-011).
        let stopped = !visited && at != to;
        prop_assert!(!stopped || !state.adventure().map().events_at(at).is_empty());
        let end = if stopped { at } else { to };
        prop_assert_eq!(
            at,
            if visited {
                path.len().checked_sub(2).map_or(start, |i| path[i])
            } else {
                end
            }
        );
        let moved = probe.adventure().hero(hero).unwrap();
        let points = reach_of(reaches, hero).points_left(end);
        prop_assert_eq!((moved.pos, Some(moved.movement)), (at, points));
    }
    Ok(())
}

/// The `BuildBoat`s `player` may issue now, by an oracle of ADV-SHY-003 (E08 T86b1; increment 9's
/// design note §15): one for each Shipyard in object order that is its own (an ally's too: the
/// player alone until E10), has a tile the player explored, a boat tile (ADV-SHY-001) with neither
/// an empty boat nor a hero on it, while the treasury pays the kind's price.
fn buildable(state: &GameState, rules: &GameRuleset, player: PlayerId) -> Vec<GameCommand> {
    let adventure = state.adventure();
    let map = adventure.map();
    let treasury = state.treasury(player).unwrap_or_default();
    let objects = map.objects().iter().enumerate();
    objects
        .filter_map(|(i, o)| {
            let object_type = o.object_type?;
            let kind = rules.adventure().object_type(object_type).map(|t| &t.kind);
            let Some(ObjectKind::Shipyard { price, .. }) = kind else {
                return None;
            };
            let tile = o.resolved.boat_tile()?;
            let taken =
                adventure.is_empty_boat(tile) || adventure.heroes().iter().any(|h| h.pos == tile);
            let seen = o.tiles(map).any(|pos| adventure.is_explored(player, pos));
            let pays = price
                .iter()
                .all(|&(r, amount)| treasury.get(r.index()).is_some_and(|&t| t >= amount));
            (adventure.owner(i) == Some(player) && seen && !taken && pays).then_some(
                GameCommand::Adventure(AdventureCommand::BuildBoat {
                    object_type,
                    anchor: o.anchor,
                }),
            )
        })
        .collect()
}

/// Every `BuildBoat` (E08 T86b1; ADV-SHY-003, increment 9's design note §15): each one the actor
/// lists applies, builds the boat on its Shipyard's tile told as `BoatBuilt`, and pays the price;
/// a `BuildBoat` naming any other typed object, a Shipyard it may not build at among them, is
/// rejected with the state unchanged.
fn check_build_boat(
    state: &GameState,
    rules: &GameRuleset,
    actor: PlayerId,
    legal: &[GameCommand],
) -> Result<(), TestCaseError> {
    let map = state.adventure().map();
    for (i, o) in map.objects().iter().enumerate() {
        let Some(object_type) = o.object_type else {
            continue;
        };
        let command = GameCommand::Adventure(AdventureCommand::BuildBoat {
            object_type,
            anchor: o.anchor,
        });
        let mut probe = state.clone();
        let mut events = Vec::new();
        let result = apply(
            &mut probe,
            rules,
            Actor::Player(actor),
            command.clone(),
            &mut events,
        );
        if !legal.contains(&command) {
            prop_assert!(result.is_err(), "unlisted {:?} accepted", command);
            prop_assert_eq!(&probe, state);
            prop_assert!(events.is_empty());
            continue;
        }
        prop_assert!(result.is_ok(), "legal {:?} rejected: {:?}", command, result);
        let tile = o.resolved.boat_tile().unwrap();
        let built = events.iter().any(|e| {
            matches!(e, GameEvent::Adventure(AdventureEvent {
                kind: AdventureEventKind::BoatBuilt { tile: t, .. },
                ..
            }) if *t == tile)
        });
        prop_assert!(built, "{:?}", events);
        prop_assert!(probe.adventure().is_empty_boat(tile));
        let Some(ObjectKind::Shipyard { price, .. }) =
            rules.adventure().object_type(object_type).map(|t| &t.kind)
        else {
            return Err(fail(format!("{command:?} listed for object {i}")));
        };
        let mut treasury = state.treasury(actor).unwrap().to_vec();
        for &(r, amount) in price {
            treasury[r.index()] -= amount;
        }
        prop_assert_eq!(probe.treasury(actor).unwrap(), &treasury[..]);
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

/// Whether a monster added in play (a creature month's, E08 T85b) still stands at `pos`: visited
/// from its neighbours there, as a placed one.
fn added_at(state: &GameState, pos: TilePos) -> bool {
    let adventure = state.adventure();
    let placed = adventure.map().objects().len();
    adventure
        .added_objects()
        .iter()
        .enumerate()
        .any(|(i, o)| o.anchor == pos && !adventure.is_removed(placed + i))
}

/// Every object of `state` with its index (E08 T86b1): the static map's, then those added in play
/// (a creature month's monsters), on the map or not. (E08 T96b1) `view.rs`'s too.
pub(super) fn all_objects(
    state: &GameState,
) -> impl Iterator<Item = (usize, &h3_adventure::MapObject)> {
    let adventure = state.adventure();
    let added = adventure.added_objects().iter();
    adventure.map().objects().iter().chain(added).enumerate()
}

/// The object a visit of `pos` concerns, by index (as the engine's, ADV-OBJ-003): the last object
/// added in play still on the map that is visited from it (E08 T86b1: a creature month's monster
/// comes after the static map's in map order), else the static map's last still on the map.
fn visited_at(state: &GameState, pos: TilePos) -> Option<usize> {
    let adventure = state.adventure();
    let placed = adventure.map().objects().len();
    let live = |&i: &usize| !adventure.is_removed(i);
    let added = (placed..)
        .zip(adventure.added_objects())
        .filter(|&(i, o)| live(&i) && o.covers(pos).1)
        .map(|(i, _)| i)
        .last();
    added.or_else(|| {
        let statics = adventure.map().visited_from(pos).iter().copied();
        statics.rev().find(live)
    })
}

/// The index of the object of type `object_type` anchored at `anchor` in `state`, the objects
/// added in play included. (E08 T96b1) `view.rs`'s too.
pub(super) fn object_index(
    state: &GameState,
    object_type: ObjectTypeId,
    anchor: TilePos,
) -> Option<usize> {
    all_objects(state)
        .find(|(_, o)| (o.object_type, o.anchor) == (Some(object_type), anchor))
        .map(|(i, _)| i)
}

/// What an accepted hire question `offer` (its hero, object type and anchor) naming `creatures`
/// tells `actor` on `before` (ADV-HIR-001, ADV-ARM-007; E08 T95a1): nothing but on a dwelling
/// `actor` owns, where each of its creatures of the part's `free_level` or less in turn, by an
/// independent reading of the rule: none here at a count of 0; else all of the count joins where
/// the army has room (a stack of the creature that takes it within a stack's most, or a free
/// slot), the army taking it for the next; else refused, the army as it was.
fn free_creatures(
    rules: &GameRuleset,
    before: &GameState,
    actor: PlayerId,
    (hero, object_type, anchor): (HeroId, ObjectTypeId, TilePos),
    creatures: &[CreatureTypeId],
) -> Vec<AdventureEventKind> {
    let (part, adventure) = (rules.adventure(), before.adventure());
    let index = object_index(before, object_type, anchor).unwrap();
    let dwelling = matches!(
        part.object_type(object_type).map(|t| &t.kind),
        Some(ObjectKind::Dwelling { .. })
    );
    if !dwelling || adventure.owner(index) != Some(actor) {
        return Vec::new();
    }
    let free = part.params().dwellings().map_or(0, |p| p.free_level);
    let (most, slots) = (part.params().max_stack_count(), part.params().army_slots());
    let counts = adventure.dwelling_counts(index).unwrap();
    let mut army = adventure.hero(hero).unwrap().army.clone();
    let mut told = Vec::new();
    for (&creature, &count) in creatures.iter().zip(counts) {
        if rules.core().creature(creature).unwrap().level > free {
            continue;
        }
        let stack = army
            .iter()
            .position(|s| s.creature == creature && s.count.saturating_add(count) <= most);
        let slot = (0..slots).find(|&slot| army.iter().all(|s| s.slot != slot));
        told.push(if count == 0 {
            AdventureEventKind::NoneToRecruit {
                object_type,
                anchor,
                creature,
            }
        } else if let Some(i) = stack {
            army[i].count += count;
            AdventureEventKind::CreaturesJoined {
                hero,
                creature,
                count,
            }
        } else if let Some(slot) = slot {
            army.push(h3_contracts::ArmyStack {
                slot,
                creature,
                count,
            });
            army.sort_by_key(|s| s.slot);
            AdventureEventKind::CreaturesJoined {
                hero,
                creature,
                count,
            }
        } else {
            AdventureEventKind::JoinRefused {
                hero,
                creature,
                count,
            }
        });
    }
    told
}

/// A new day's income (GAM-RES-003 to -005, E08 T85b; increment 9's design note §14), from the
/// day's events (its `DayStarted` on): every player in id order is told, as one gain per resource
/// with an amount in the core's order, exactly what an independent sum of its parts gives on the
/// state before the day: the production of the mines it owns, (E09 T9b1) the day's income of the
/// towns it owns (TWN-INC-001, -002: `h3_towns::daily_income` of each town's record), its heroes'
/// `daily_income`, and on a week's first day each creature type with a weekly resource in its
/// heroes' armies once.
fn check_income(
    rules: &GameRuleset,
    before: &GameState,
    day: &[GameEvent],
) -> Result<(), TestCaseError> {
    let gains: Vec<(PlayerId, ResourceId, u32)> = day
        .iter()
        .filter_map(|e| match *e {
            GameEvent::ResourceGained {
                player,
                resource,
                amount,
            } => Some((player, resource, amount)),
            _ => None,
        })
        .collect();
    if day.is_empty() {
        prop_assert!(gains.is_empty());
        return Ok(());
    }
    let week = day
        .iter()
        .any(|e| matches!(e, GameEvent::WeekStarted { .. }));
    let (core, part, adventure) = (rules.core(), rules.adventure(), before.adventure());
    let mut expected = Vec::new();
    for player in (0..before.players()).map(|p| PlayerId::new(u8::try_from(p).unwrap())) {
        let mut income = vec![0_u32; core.resources().len()];
        for i in 0..adventure.map().objects().len() {
            if adventure.owner(i) == Some(player)
                && let Some((resource, amount)) = adventure.production(part, i)
            {
                income[resource.index()] += amount;
            }
        }
        for (place, &i) in adventure.map().towns().iter().enumerate() {
            if adventure.owner(i) == Some(player) {
                let town = before.towns().get(town_id(place)).unwrap();
                for (resource, amount) in h3_towns::daily_income(town, rules.towns()) {
                    income[resource.index()] += amount;
                }
            }
        }
        let heroes: Vec<_> = adventure
            .heroes()
            .iter()
            .filter(|h| h.owner == player)
            .collect();
        for hero in &heroes {
            let daily = h3_heroes::daily_income(before.hero(hero.id).unwrap(), core);
            for (entry, amount) in income.iter_mut().zip(daily) {
                *entry += amount;
            }
        }
        let mut seen = Vec::new();
        for stack in heroes.iter().flat_map(|h| &h.army).filter(|_| week) {
            if !seen.contains(&stack.creature) {
                seen.push(stack.creature);
                for ability in &core.creature(stack.creature).unwrap().abilities {
                    let h3_content::CreatureAbility::WeeklyResource { resource, amount } = *ability;
                    income[resource.index()] += amount;
                }
            }
        }
        let owed = income
            .into_iter()
            .enumerate()
            .filter(|&(_, amount)| amount > 0);
        expected
            .extend(owed.map(|(r, amount)| (player, ResourceId::from_index(r).unwrap(), amount)));
    }
    prop_assert_eq!(gains, expected);
    Ok(())
}

/// The `TownId` of the town at `place` among the map's towns (E09 T9b1).
fn town_id(place: usize) -> TownId {
    TownId::new(u16::try_from(place).unwrap())
}

/// The map's timed events at the turn starts among a command's events (GAM-CAL-004, E08 T85b),
/// from the first `TurnStarted` on (`turn`): after each player's `TurnStarted`, exactly what each
/// event that fires for it on `day` does, in the map's order: its gains, its losses down to 0 at
/// most from the treasury as it stands then (the state's before the command, then the changes told
/// before), then `TimedEventFired` with its message. The days read on their own: the first day,
/// or every interval after it.
fn check_timed(
    before: &GameState,
    (events, turn): (&[GameEvent], usize),
    day: u32,
) -> Result<(), TestCaseError> {
    let players = (0..before.players()).map(|p| PlayerId::new(u8::try_from(p).unwrap()));
    let mut treasuries: Vec<Vec<u32>> = players
        .map(|p| before.treasury(p).unwrap().to_vec())
        .collect();
    for event in &events[..turn] {
        match *event {
            GameEvent::ResourceGained {
                player,
                resource,
                amount,
            } => treasuries[player.index()][resource.index()] += amount,
            GameEvent::ResourceLost {
                player,
                resource,
                amount,
            }
            | GameEvent::ResourcePaid {
                player,
                resource,
                amount,
            } => treasuries[player.index()][resource.index()] -= amount,
            _ => {}
        }
    }
    let mut expected = Vec::new();
    for event in &events[turn..] {
        let GameEvent::TurnStarted { player } = *event else {
            continue;
        };
        expected.push(*event);
        let human = before.human(player).unwrap();
        let treasury = &mut treasuries[player.index()];
        for timed in before.events() {
            let (first, every) = (u32::from(timed.first_day), u32::from(timed.interval));
            let today =
                day == first || every > 0 && day > first && (day - first).is_multiple_of(every);
            let kind = if human { timed.human } else { timed.computer };
            if !(today && kind && timed.players.contains(&player)) {
                continue;
            }
            for &(resource, amount) in timed.resources.iter().filter(|&&(_, a)| a > 0) {
                let amount = amount.unsigned_abs();
                treasury[resource.index()] += amount;
                expected.push(GameEvent::ResourceGained {
                    player,
                    resource,
                    amount,
                });
            }
            for &(resource, amount) in timed.resources.iter().filter(|&&(_, a)| a < 0) {
                let amount = amount.unsigned_abs().min(treasury[resource.index()]);
                treasury[resource.index()] -= amount;
                expected.push(GameEvent::ResourceLost {
                    player,
                    resource,
                    amount,
                });
            }
            expected.push(GameEvent::TimedEventFired {
                player,
                event: timed.message,
            });
        }
    }
    prop_assert_eq!(&events[turn..], &expected[..]);
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
/// levels it is told it gained answering its level-up (its level and skills, nothing else but
/// the offers' levels). Since E08 T20b3b1 a wandering
/// monster is a source of creatures and resources: a group that joins goes into its visitor's
/// army (into the stack of its creature, else the first free slot), all of it, told to the
/// visitor's owner as the creatures joining, and its treasure is the owner's gains as it leaves
/// the map; a monster that flees leaves without its treasure. Since E08 T64b increment 7's
/// sources and sinks ([`visit_gains`], [`records`]): a chest's gold when its gold is chosen, a box's and a fired
/// event's record (a positive amount a gain, a negative one a loss the treasury is told it lost,
/// never below 0), a find's contents at the visit that empties it, and the prices of a School's
/// choice and a University's purchases; a hero's details change by its changes and its levels
/// in the order it is told them (a reward's rest after a level-up, GAM-DEC-007). Since E08 T95b a
/// hire dialog's purchase pays the cost of one times the amount and joins the amount as a join does.
fn check_conservation(
    rules: &GameRuleset,
    (before, after): (&GameState, &GameState),
    command: &GameCommand,
    events: &[GameEvent],
) -> Result<(), TestCaseError> {
    let battles = ended(events);
    if !battles.is_empty() {
        return check_battle_end(rules, (before, after), events, &battles);
    }
    for (i, (b, a)) in before.heroes().iter().zip(after.heroes()).enumerate() {
        let id = HeroId::new(u16::try_from(i).unwrap());
        // E08 T52: outside a battle a hero's details change by the changes its owner is told: a
        // meeting's spells (HER-SPL-007), an artifact's move, an assembly and a disassembly
        // (HER-ART-005, -006), each applied in order; (E08 T30) by the levels it is told it
        // gained answering its level-up, each the next level (GAM-DEC-002, HER-LVL-005).
        let owner = before.adventure().hero(id).map(|h| h.owner);
        let mut told = (**b).clone();
        for event in events {
            match *event {
                GameEvent::HeroChanged {
                    player,
                    hero,
                    change,
                } if hero == id => {
                    prop_assert_eq!(Some(player), owner);
                    h3_heroes::apply_change(&mut told, rules.core(), change);
                }
                GameEvent::LevelGained {
                    player,
                    hero,
                    level,
                    primary,
                    skill,
                } if hero == id => {
                    prop_assert_eq!(Some(player), owner);
                    prop_assert!(after.adventure().hero(id).is_some(), "{:?} levelled", id);
                    prop_assert_eq!(level, told.level + 1);
                    // The offers' levels after the last level told are the hero's (one command
                    // may give several levels).
                    let change = HeroChange::GainLevel {
                        primary,
                        learned: skill,
                        offered: if level == a.level {
                            a.offered
                        } else {
                            told.offered
                        },
                    };
                    let checked = h3_heroes::check_change(&told, rules.core(), change);
                    prop_assert!(checked.is_ok(), "{:?}: {:?}", change, checked);
                    h3_heroes::apply_change(&mut told, rules.core(), change);
                }
                // E08 T85b: a new day's regain (HER-MAN-002), which no event tells.
                GameEvent::DayStarted { .. } => {
                    told.mana = h3_heroes::day_mana(&told, rules.core());
                }
                _ => {}
            }
        }
        prop_assert_eq!(&told, &**a);
    }
    prop_assert_eq!(before.heroes().len(), after.heroes().len());
    let adventure = after.adventure();
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
    // E08 T86b1: a Leave Guards dialog's stacks are its mine's guards ([`check_mine_guards`]).
    let guards = events
        .iter()
        .any(|e| matches!(e, GameEvent::LeaveGuardsOpened { .. }));
    let held: Vec<(HeroId, CreatureTypeId, u32)> = events
        .iter()
        .filter(|_| !guards)
        .filter_map(|e| match *e {
            GameEvent::ArmyDialogOpened { hero, stack, .. } => {
                Some((hero, stack.creature, stack.count))
            }
            _ => None,
        })
        .collect();
    // E08 T95a1: an accepted hire question's free creatures, one join per creature, come from its
    // dwelling, whose count of each that joined drops to 0, the others kept, as a camp's
    // (ADV-HIR-001; their events: [`free_creatures`]).
    let hired = events.iter().find_map(|e| match *e {
        GameEvent::OfferAnswered {
            player,
            accepted: true,
            ..
        } => before
            .decision(player)
            .filter(|o| matches!(o.kind, OfferKind::Hire { .. })),
        _ => None,
    });
    if let Some(offer) = hired {
        let OfferKind::Hire { creatures } = &offer.kind else {
            unreachable!()
        };
        let index = object_index(before, offer.object_type, offer.anchor).unwrap();
        let (was, now) = (before.adventure(), after.adventure());
        let mut counts = was.dwelling_counts(index).map(<[u32]>::to_vec);
        for &(hero, creature, count) in &joined {
            prop_assert_eq!(hero, offer.hero);
            let at = creatures.iter().position(|&c| c == creature);
            let left = counts.as_mut().zip(at).map(|(counts, at)| &mut counts[at]);
            prop_assert!(left.is_some(), "{:?} joined from {:?}", creature, offer);
            let left = left.unwrap();
            prop_assert_eq!(core::mem::take(left), count);
        }
        prop_assert_eq!(now.dwelling_counts(index).map(<[u32]>::to_vec), counts);
        prop_assert_eq!(now.camp(index), was.camp(index));
        prop_assert!(held.is_empty(), "{:?} held at {:?}", held, offer);
    } else {
        prop_assert!(joined.len() + held.len() <= 1, "{:?} {:?}", joined, held);
    }
    let mut contents = visit_gains(rules, (before, after), command, events);
    let mut losses = Vec::new();
    for event in events {
        // E08 T64b: a box's record as it leaves, an event's as it fires, in the events' order.
        records((before, after), event, (&mut contents, &mut losses));
        if let GameEvent::Adventure(AdventureEvent {
            kind:
                AdventureEventKind::ObjectRemoved {
                    object_type,
                    anchor,
                },
            ..
        }) = *event
        {
            // E08 T86b1: a monster added in play leaves as a placed one.
            let left = all_objects(before).filter(|&(i, o)| {
                (o.object_type, o.anchor) == (Some(object_type), anchor)
                    && !before.adventure().is_removed(i)
                    && adventure.is_removed(i)
            });
            for (i, o) in left {
                // E08 T64b: a chest leaves the map with the visit, its gold given only when its
                // choice takes it ([`visit_gains`]; ADV-PIK-005); (E08 T75b) a find that leaves
                // gives what its artifact and turn say, [`visit_gains`]' too (ADV-PIK-014 to -016).
                let apart = matches!(
                    rules.adventure().object_type(object_type).map(|t| &t.kind),
                    Some(ObjectKind::Chest(_) | ObjectKind::Find(_))
                );
                if !apart {
                    contents.extend(o.contents.iter().copied());
                }
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
    // E08 T85b: a new day's income comes after its `DayStarted`, apart from the visits' gains,
    // and the turn starts' timed events after the first `TurnStarted` ([`check_timed`]).
    let turn_at = events
        .iter()
        .position(|e| matches!(e, GameEvent::TurnStarted { .. }))
        .unwrap_or(events.len());
    let day_at = events
        .iter()
        .position(|e| matches!(e, GameEvent::DayStarted { .. }))
        .unwrap_or(turn_at);
    let gains: Vec<(ResourceId, u32)> = events[..day_at]
        .iter()
        .filter_map(|e| match *e {
            GameEvent::ResourceGained {
                resource, amount, ..
            } => Some((resource, amount)),
            _ => None,
        })
        .collect();
    prop_assert_eq!(&gains, &contents, "{:?} {:?}", command, events);
    check_income(rules, before, &events[day_at..turn_at])?;
    check_timed(before, (events, turn_at), after.day())?;
    let lost: Vec<(ResourceId, u32)> = events[..day_at]
        .iter()
        .filter_map(|e| match *e {
            GameEvent::ResourceLost {
                resource, amount, ..
            } => Some((resource, amount)),
            _ => None,
        })
        .collect();
    // A loss takes what the treasury holds, never more (ADV-RWD-001).
    prop_assert_eq!(lost.len(), losses.len());
    for (&(resource, amount), &(r, most)) in lost.iter().zip(&losses) {
        prop_assert!(
            resource == r && amount <= most,
            "{:?} of {:?}",
            lost,
            losses
        );
    }
    // E08 T64b: a visit's gains come with an object leaving the map, a find emptied or an event
    // fired.
    let removed = events.iter().any(|e| {
        matches!(
            e,
            GameEvent::Adventure(AdventureEvent {
                kind: AdventureEventKind::ObjectRemoved { .. }
                    | AdventureEventKind::EventFired { .. },
                ..
            })
        )
    }) || emptied(before, after).next().is_some()
        || weekly_emptied(before, after).next().is_some()
        || matches!(
            command,
            GameCommand::Answer(Answer::Chest(ChestChoice::Gold))
        );
    // E08 T86b1: `BuildBoat` pays its Shipyard's price (ADV-SHY-003).
    let boat = match *command {
        GameCommand::Adventure(AdventureCommand::BuildBoat { object_type, .. }) => {
            match rules.adventure().object_type(object_type).map(|t| &t.kind) {
                Some(ObjectKind::Shipyard { price, .. }) => Some(price),
                _ => None,
            }
        }
        _ => None,
    };
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
        for (at, event) in events.iter().enumerate() {
            match *event {
                GameEvent::ResourceGained {
                    player: owner,
                    resource,
                    amount,
                } => {
                    prop_assert!(removed || at > day_at, "{:?} without a visit", event);
                    if owner == player {
                        treasury[usize::from(resource.raw())] += amount;
                    }
                }
                GameEvent::ResourcePaid {
                    player: owner,
                    resource,
                    amount,
                } => {
                    // E08 T64b: or a School's choice (GAM-DEC-005) or a University's purchase
                    // (GAM-DEC-006) answered; (E08 T95b) or a hire dialog's purchase (GAM-DEC-008).
                    let answered = matches!(command, GameCommand::Answer(_))
                        && (before.choice(owner).is_some()
                            || before.university(owner).is_some()
                            || hiring(before, owner));
                    prop_assert!(
                        accepted == Some(owner) || answered || boat.is_some(),
                        "{:?} without an offer",
                        event
                    );
                    if owner == player {
                        treasury[usize::from(resource.raw())] -= amount;
                        paid.push((resource, amount));
                    }
                }
                GameEvent::ResourceLost {
                    player: owner,
                    resource,
                    amount,
                } if owner == player => treasury[usize::from(resource.raw())] -= amount,
                _ => {}
            }
        }
        prop_assert_eq!(after.treasury(player).unwrap(), &treasury[..]);
        if accepted == Some(player) {
            prop_assert_eq!(&paid[..], before.decision(player).unwrap().price());
        } else if let Some(price) = boat.filter(|_| !paid.is_empty()) {
            prop_assert_eq!(&paid[..], &price[..]);
        } else if let Some(choice) = before.choice(player).filter(|_| !paid.is_empty()) {
            prop_assert_eq!(&paid[..], choice.price());
        } else if let Some(visit) = before.university(player).filter(|_| !paid.is_empty()) {
            prop_assert_eq!(&paid[..], &visit.price[..]);
        } else if let GameCommand::Answer(Answer::Hire(HireAnswer::Buy { creature, amount })) =
            *command
            && hiring(before, player)
            && !paid.is_empty()
        {
            // E08 T95b: a purchase's price, the cost of one times the amount (ADV-HIR-002).
            let price: Vec<(ResourceId, u32)> = unit_cost(rules, creature)
                .into_iter()
                .map(|(resource, each)| (resource, each * amount))
                .collect();
            prop_assert_eq!(&paid[..], &price[..]);
        }
    }
    let mut expected = armies(before);
    // E08 T95b: into the first stack of the creature that takes them within a stack's most, else
    // the first free slot (ADV-ARM-007; a purchase may not fit the first stack).
    let most = rules.adventure().params().max_stack_count();
    for &(hero, creature, count) in &joined {
        let (_, army) = expected.iter_mut().find(|(id, _)| *id == hero).unwrap();
        let fits = |s: &&mut h3_contracts::ArmyStack| {
            s.creature == creature && s.count.saturating_add(count) <= most
        };
        if let Some(stack) = army.iter_mut().find(fits) {
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

/// The object a step visited (E08 T64b), by index: a move's, the live typed object visited from
/// its last tile when it stops before it (ADV-OBJ-003) or from the tile it ends on, the event
/// that fired there; an answer's, the object of the offer, choice or University it answers.
fn visited(before: &GameState, command: &GameCommand, events: &[GameEvent]) -> Option<usize> {
    let adventure = before.adventure();
    let map = adventure.map();
    let live = |indices: &[usize]| {
        indices
            .iter()
            .rev()
            .copied()
            .find(|&i| !adventure.is_removed(i))
    };
    let named = |(object_type, anchor): (ObjectTypeId, TilePos)| {
        let objects = map.objects().iter().enumerate().rev();
        objects
            .filter(|&(i, o)| {
                (o.object_type, o.anchor) == (Some(object_type), anchor) && !adventure.is_removed(i)
            })
            .map(|(i, _)| i)
            .next()
    };
    match command {
        GameCommand::Adventure(AdventureCommand::MoveHero { hero, path }) => {
            if let Some(tile) = fired_at(events) {
                return live(map.events_at(tile));
            }
            let start = adventure.hero(*hero)?.pos;
            let steps = events
                .iter()
                .filter(|e| {
                    matches!(
                        e,
                        GameEvent::Adventure(AdventureEvent {
                            kind: AdventureEventKind::HeroStepped { .. },
                            ..
                        })
                    )
                })
                .count();
            // An event held for its tile's guard stopped the move ([`landing`]): the guard's
            // visit, not an object's.
            if steps > 0 && steps < path.len() && live(map.events_at(path[steps - 1])).is_some() {
                return None;
            }
            let to = if steps < path.len() {
                *path.get(steps)?
            } else {
                path.last().copied().unwrap_or(start)
            };
            // A step toward a hero meets it or fights it: no object is visited there.
            let met = adventure
                .heroes()
                .iter()
                .any(|h| h.pos == to && h.id != *hero);
            if met {
                return None;
            }
            live(map.visited_from(to))
        }
        GameCommand::Answer(_) => {
            let player = acting(before);
            if let Some(offer) = before.decision(player) {
                named((offer.object_type, offer.anchor))
            } else if let Some(choice) = before.choice(player) {
                named((choice.object_type, choice.anchor))
            } else {
                let visit = before.university(player)?;
                named((visit.object_type, visit.anchor))
            }
        }
        _ => None,
    }
}

/// Increment 7's invariants after every command (E08 T64b, increment 7's design note §16): a
/// removed object never returns, a hero's once-per-hero marks (of a hero still on the map), a
/// player's knowledge and the emptied objects only grow (ADV-VIS-006, ADV-PIK-009 to -012), and an
/// emptied find, or a looted Pyramid, gives nothing at a later visit: no resource, artifact or
/// spell.
fn check_rewards(
    rules: &GameRuleset,
    (before, after): (&GameState, &GameState),
    command: &GameCommand,
    events: &[GameEvent],
) -> Result<(), TestCaseError> {
    let (b, a) = (before.adventure(), after.adventure());
    let objects = 0..b.map().objects().len();
    let players = 0..u8::try_from(before.players()).unwrap();
    for i in objects {
        prop_assert!(!b.is_removed(i) || a.is_removed(i), "object {} returned", i);
        let (Some(was), Some(is)) = (b.visits(), a.visits()) else {
            prop_assert!(b.visits().is_none(), "the visits block went");
            continue;
        };
        prop_assert!(
            !was.is_emptied(i) || is.is_emptied(i),
            "object {} refilled",
            i
        );
        for player in players.clone().map(PlayerId::new) {
            prop_assert!(
                !was.knows(player, i) || is.knows(player, i),
                "{:?} forgot",
                player
            );
        }
        for hero in a.heroes().iter().map(|h| h.id) {
            prop_assert!(
                !was.is_marked(hero, i) || is.is_marked(hero, i),
                "{:?} unmarked",
                hero
            );
        }
    }
    let Some(index) = visited(before, command, events) else {
        return Ok(());
    };
    let finds = matches!(
        b.object(index)
            .and_then(|o| o.object_type)
            .and_then(|ty| rules.adventure().object_type(ty))
            .map(|ty| &ty.kind),
        Some(ObjectKind::Find(_) | ObjectKind::Pyramid(_))
    );
    if finds && b.visits().is_some_and(|v| v.is_emptied(index)) {
        let gives = events.iter().any(|e| {
            matches!(
                e,
                GameEvent::ResourceGained { .. }
                    | GameEvent::HeroChanged {
                        change: HeroChange::AddArtifact(_) | HeroChange::LearnSpell(_),
                        ..
                    }
            )
        });
        prop_assert!(!gives, "emptied object {} gave {:?}", index, events);
    }
    Ok(())
}

/// The objects a step emptied (E08 T64b; ADV-PIK-009 to -012, ADV-TCH-008): a find searched,
/// the Pyramid looted, by index.
fn emptied<'s>(before: &'s GameState, after: &'s GameState) -> impl Iterator<Item = usize> + 's {
    let is = |state: &GameState, i| state.adventure().visits().is_some_and(|v| v.is_emptied(i));
    (0..before.adventure().map().objects().len()).filter(move |&i| !is(before, i) && is(after, i))
}

/// The weekly objects a visit emptied between `before` and `after` (E08 T86b1; ADV-WEK-001 to
/// -003): full before, empty after.
fn weekly_emptied<'s>(
    before: &'s GameState,
    after: &'s GameState,
) -> impl Iterator<Item = usize> + 's {
    let amount = |state: &GameState, i| state.adventure().weekly(i).map_or(0, |w| w.amount);
    let weeklies = before.adventure().map().weeklies().iter().copied();
    weeklies.filter(move |&i| amount(before, i) > 0 && amount(after, i) == 0)
}

/// The finds that left the map between `before` and `after` (E08 T75b; ADV-PIK-014 to -016): a
/// sea pickup leaves at its visit instead of being emptied, so its gains come at its removal.
fn left<'s>(
    rules: &'s GameRuleset,
    before: &'s GameState,
    after: &'s GameState,
) -> impl Iterator<Item = usize> + 's {
    let map = before.adventure().map();
    (0..map.objects().len()).filter(move |&i| {
        let ty = map.objects()[i].object_type;
        let kind = ty
            .and_then(|ty| rules.adventure().object_type(ty))
            .map(|ty| &ty.kind);
        !before.adventure().is_removed(i)
            && after.adventure().is_removed(i)
            && matches!(kind, Some(ObjectKind::Find(find)) if find.leaves)
    })
}

/// Resources and their amounts, in order.
type Amounts = Vec<(ResourceId, u32)>;

/// Increment 7's resource gains of a visit outside a battle (E08 T64b; ADV-PIK-005, ADV-PIK-009
/// to -012): a chest's gold when its choice takes it, a find's at the visit that empties it; the
/// records' are [`records`]', the piles' and monsters' [`check_conservation`]'s.
fn visit_gains(
    rules: &GameRuleset,
    (before, after): (&GameState, &GameState),
    command: &GameCommand,
    events: &[GameEvent],
) -> Amounts {
    let map = before.adventure().map();
    let chest = (0..before.players()).find_map(|p| {
        let player = PlayerId::new(u8::try_from(p).unwrap());
        match before.choice(player).map(|c| &c.kind) {
            Some(&ChoiceKind::Chest { gold, .. }) if after.choice(player).is_none() => Some(gold),
            _ => None,
        }
    });
    let gold = matches!(
        command,
        GameCommand::Answer(Answer::Chest(ChestChoice::Gold))
    );
    let mut gains: Amounts = chest.filter(|_| gold).into_iter().collect();
    // E08 T86b1: a weekly object's resource and amount at the visit that empties it (ADV-WEK-001
    // to -003).
    for i in weekly_emptied(before, after) {
        let weekly = before.adventure().weekly(i).unwrap();
        gains.push((weekly.resource, weekly.amount));
    }
    // A find's artifact when its hero receives it, with a fixed entry's resources (E08 T72), else
    // what the artifact turns into (the Corpse's gold, the Wagon's own resources, nothing when it
    // is lost), or its resources; (E08 T75b) a find that leaves, at its removal.
    for i in emptied(before, after).chain(left(rules, before, after)) {
        let o = &map.objects()[i];
        let ty = o
            .object_type
            .and_then(|ty| rules.adventure().object_type(ty));
        let turn = match ty.map(|ty| &ty.kind) {
            Some(ObjectKind::Find(find)) => find.full_backpack.as_ref(),
            _ => None,
        };
        let received = |artifact: ArtifactTypeId| {
            events.iter().any(|e| {
                matches!(e, GameEvent::HeroChanged {
                    change: HeroChange::AddArtifact(item), ..
                } if item.artifact == artifact)
            })
        };
        match (o.resolved.artifact(), turn) {
            (Some(artifact), Some(Turn::Drawn)) if received(artifact) => {}
            (Some(artifact), _) if received(artifact) => gains.extend(o.contents.iter().copied()),
            (Some(_), Some(Turn::Give(resources))) => gains.extend(resources.iter().copied()),
            (Some(_), Some(Turn::Stays | Turn::Lost) | None) => {}
            (Some(_), Some(Turn::Drawn)) | (None, _) => gains.extend(o.contents.iter().copied()),
        }
    }
    gains
}

/// A reward record's resources that `event` gives (E08 T64b; ADV-RWD-001, ADV-PIK-013,
/// ADV-OBJ-007), added to `gains` (its positive amounts) and `losses` (its negative ones): a
/// box's as it leaves the map, an event's as it fires unless its guards stand (their battle gives
/// it).
fn records(
    (before, after): (&GameState, &GameState),
    event: &GameEvent,
    (gains, losses): (&mut Amounts, &mut Amounts),
) {
    let map = before.adventure().map();
    let mut give = |record: &RewardRecord| {
        for &(resource, amount) in &record.resources {
            let into = if amount < 0 {
                &mut *losses
            } else {
                &mut *gains
            };
            into.push((resource, amount.unsigned_abs()));
        }
    };
    let GameEvent::Adventure(AdventureEvent { kind, .. }) = *event else {
        return;
    };
    match kind {
        AdventureEventKind::ObjectRemoved {
            object_type,
            anchor,
        } => {
            let left = map.objects().iter().enumerate().filter(|&(i, o)| {
                (o.object_type, o.anchor) == (Some(object_type), anchor)
                    && !before.adventure().is_removed(i)
                    && after.adventure().is_removed(i)
            });
            for (_, o) in left {
                if let Holds::Record(record) = &o.holds {
                    give(record);
                }
            }
        }
        AdventureEventKind::EventFired { tile, .. } => {
            let live = map.events_at(tile).iter().rev();
            let fired = live.copied().find(|&i| !before.adventure().is_removed(i));
            let fired = fired.map(|i| (i, &map.objects()[i]));
            if let Some((i, o)) = fired
                && let Holds::Event(event) = &o.holds
                && before.adventure().guards_left(i).is_empty()
            {
                give(&event.record);
            }
        }
        _ => {}
    }
}

/// The army dialogs' groups (E08 T52; GAM-DEC-004): a dialog opens with the group whole in its
/// slot 0, as told; a slot of a group changes as told (`StackChanged`); a closed dialog's group
/// is lost, each stack it held told in slot order after the dialog's close; nothing else
/// changes a group. (E08 T86b1) An own mine's Leave Guards dialog opens empty, then with each of
/// the mine's guard stacks as told, and its close keeps the group as the mine's guards
/// (ADV-MIN-002, GAM-DEC-004 amended; [`check_mine_guards`]): nothing is lost.
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
        let mut mine = before.leave_guards(player).is_some();
        for event in events {
            match *event {
                GameEvent::LeaveGuardsOpened {
                    player: owner,
                    hero,
                    ..
                } if owner == player => {
                    prop_assert!(expected.is_none(), "{:?} with a dialog open", event);
                    expected = Some((hero, Vec::new()));
                    mine = true;
                }
                // E08 T62b1: one event per stack of the group at its opening, in slot order (a
                // join's one stack in slot 0; a reward record's at their slots).
                GameEvent::ArmyDialogOpened {
                    player: owner,
                    hero,
                    stack,
                } if owner == player => match &mut expected {
                    Some((open, group))
                        if *open == hero && group.last().map_or(mine, |s| s.slot < stack.slot) =>
                    {
                        group.push(stack);
                    }
                    _ => {
                        prop_assert!(expected.is_none(), "{:?} with a dialog open", event);
                        expected = Some((hero, vec![stack]));
                    }
                },
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
                    if let Some((_, group)) = expected.take()
                        && !mine
                    {
                        lost = group;
                    }
                    mine = false;
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

/// A week's step (E08 T86b1; ADV-WEK-001 to -003, ADV-MON-013; increment 9's design note §15):
/// after a `WeekStarted` every weekly object is full, its marks kept (a Windmill one of its
/// resources and an amount within its bounds, a Water Wheel its later weeks' amount, a Garden one
/// of its gifts), (E08 T96b1) every dwelling full and every camp drawn anew
/// ([`check_dwellings_week`]), and every wandering monster that stood before grew: one that grows to
/// `v + ⌊v × growth_bp ÷ 10 000⌋`, at most `growth_cap` creatures (`v` its count in growth units
/// with its fraction), as count and fraction; one that does not grow keeps its count.
fn check_week(
    rules: &GameRuleset,
    (before, after): (&GameState, &GameState),
    events: &[GameEvent],
) -> Result<(), TestCaseError> {
    if !events
        .iter()
        .any(|e| matches!(e, GameEvent::WeekStarted { .. }))
    {
        return Ok(());
    }
    let (part, b, a) = (rules.adventure(), before.adventure(), after.adventure());
    for &i in b.map().weeklies() {
        let (was, now) = (b.weekly(i).unwrap(), a.weekly(i).unwrap());
        prop_assert_eq!(now.marks, was.marks);
        let ty = b.map().objects()[i].object_type.unwrap();
        let full = match &part.object_type(ty).unwrap().kind {
            ObjectKind::Windmill { resources, amount } => {
                resources.contains(&now.resource) && (amount[0]..=amount[1]).contains(&now.amount)
            }
            ObjectKind::WaterWheel {
                resource, later, ..
            } => (now.resource, now.amount) == (*resource, *later),
            ObjectKind::Garden { gifts } => gifts.contains(&(now.resource, now.amount)),
            _ => false,
        };
        prop_assert!(full, "weekly object {} after a week's start: {:?}", i, now);
    }
    check_dwellings_week(rules, after)?;
    let Some(params) = part.params().monsters() else {
        return Ok(());
    };
    let unit = u64::from(params.growth_unit());
    let (most, rate) = (
        u64::from(params.growth_cap()) * unit,
        u64::from(params.growth_bp()),
    );
    for (i, o) in all_objects(before) {
        let Some(monster) = o.monster.as_ref().filter(|_| !b.is_removed(i)) else {
            continue;
        };
        let count = u64::from(b.monster_count(i).unwrap());
        let now = (
            u64::from(a.monster_count(i).unwrap()),
            u64::from(a.fraction(i)),
        );
        if monster.no_growth {
            prop_assert_eq!(now.0, count);
            continue;
        }
        let value = count * unit + u64::from(b.fraction(i));
        let grown = (value + value * rate / 10_000).min(most);
        prop_assert_eq!(now, (grown / unit, grown % unit), "monster {}", i);
    }
    Ok(())
}

/// After a week's step (E08 T96b1; ADV-DWL-003, ADV-RFG-001, increment 10's design note §9, §14):
/// every dwelling full, owned or not, read apart from the engine's week: each of its kind's
/// creatures its growth, its guards one stack by slot from 0 of each creature of the part's
/// `guard_level` or more, `guard_factor` times its growth, in its kind's order; every Refugee
/// Camp's creature a random monster of its kind's levels (ADV-MON-003's set, read from the core:
/// its level within them, no war machine, none of [`CAMP_NOT_RANDOM`]), its count that creature's
/// growth.
pub(super) fn check_dwellings_week(
    rules: &GameRuleset,
    after: &GameState,
) -> Result<(), TestCaseError> {
    let (core, part, a) = (rules.core(), rules.adventure(), after.adventure());
    let growth = |creature: CreatureTypeId| u32::from(core.creature(creature).unwrap().growth);
    for &i in a.map().dwellings() {
        // A dwelling is an object of the static map (`StaticMap::dwellings`), never one added in
        // play.
        let ty = a.map().objects()[i].object_type.unwrap();
        let Some(ObjectKind::Dwelling { creatures, .. }) = part.object_type(ty).map(|t| &t.kind)
        else {
            return Err(fail(format!("dwelling {i} of no dwelling's type")));
        };
        let counts: Vec<u32> = creatures.iter().map(|&c| growth(c)).collect();
        prop_assert_eq!(a.dwelling_counts(i).unwrap(), &counts[..], "dwelling {}", i);
        let params = part.params().dwellings().unwrap();
        let guards: Vec<h3_contracts::ArmyStack> = (0..)
            .zip(
                creatures
                    .iter()
                    .filter(|&&c| core.creature(c).unwrap().level >= params.guard_level),
            )
            .map(|(slot, &creature)| h3_contracts::ArmyStack {
                slot,
                creature,
                count: u32::from(params.guard_factor.get()) * growth(creature),
            })
            .collect();
        prop_assert_eq!(a.dwelling_guards(i), guards, "dwelling {}", i);
    }
    // The six creatures the original's draw leaves out (its ids 132 to 137, ADV-MON-003), named
    // here apart from the part's `not_random_monsters`, which the engine's draw reads.
    let not_random = CAMP_NOT_RANDOM.map(|key| core.creature_by_key(key).unwrap());
    for &i in a.map().camps() {
        // A camp is an object of the static map (`StaticMap::camps`), never one added in play.
        let ty = a.map().objects()[i].object_type.unwrap();
        let Some(&ObjectKind::RefugeeCamp { levels }) = part.object_type(ty).map(|t| &t.kind)
        else {
            return Err(fail(format!("camp {i} of no camp's type")));
        };
        let (creature, count) = a.camp(i).unwrap();
        // ADV-RFG-001's set read from the core, not from `random_monsters` (the pool the engine
        // draws from, which `check_camps` already reads after every command).
        let info = core.creature(creature).unwrap();
        prop_assert!(
            (levels[0]..=levels[1]).contains(&info.level)
                && !info.flags.contains(CreatureFlag::WarMachine)
                && !not_random.contains(&creature),
            "camp {}: {:?} of level {} not of the camp's set, levels {:?}",
            i,
            creature,
            info.level,
            levels
        );
        prop_assert_eq!(count, growth(creature), "camp {}", i);
    }
    Ok(())
}

/// The creatures a Refugee Camp never holds besides the war machines (ADV-RFG-001, ADV-MON-003:
/// the original's `random_creature_by_level` draws among its ids 0 to 144 without 132 to 137), by
/// key, for [`check_dwellings_week`].
const CAMP_NOT_RANDOM: [&str; 6] = [
    "ab:azure_dragon",
    "ab:crystal_dragon",
    "ab:faerie_dragon",
    "ab:rust_dragon",
    "ab:enchanter",
    "ab:sharpshooter",
];

/// The towns across a command (E09 T9b1; the town epic's design note §18): every town has one
/// owner, the adventure's (the store of towns holds none, ADR-0028), a player of the game or
/// none; it changes only by the town's capture, told by one `TownCaptured` naming the new owner
/// (TWN-CAP-001, -003), and a captured town's garrison is empty (its leftovers lost with it,
/// design §11); the store keeps the number of towns and each record's faction, and a record's
/// buildings change only by the Capitol's limit at a capture, told by `TownChanged` with the
/// town's Capitol as the store has it after (TWN-CAP-002).
fn check_towns(
    rules: &GameRuleset,
    (before, after): (&GameState, &GameState),
    events: &[GameEvent],
) -> Result<(), TestCaseError> {
    let (adventure, part) = (after.adventure(), rules.towns());
    let players = after.players();
    prop_assert_eq!(after.towns().len(), adventure.map().towns().len());
    prop_assert_eq!(after.towns().len(), before.towns().len());
    for (place, &i) in adventure.map().towns().iter().enumerate() {
        let town = town_id(place);
        let (was, now) = (before.adventure().owner(i), adventure.owner(i));
        prop_assert!(
            now.is_none_or(|p| p.index() < players),
            "{:?}: {:?}",
            town,
            now
        );
        let captured: Vec<PlayerId> = events
            .iter()
            .filter_map(|e| match e {
                GameEvent::Adventure(AdventureEvent {
                    kind: AdventureEventKind::TownCaptured { town: t, owner, .. },
                    ..
                }) if *t == town => Some(*owner),
                _ => None,
            })
            .collect();
        if captured.is_empty() {
            prop_assert_eq!(was, now, "{:?} changed owner untold", town);
        } else {
            prop_assert_eq!(&captured, &[now.unwrap()], "{:?}", town);
            prop_assert_ne!(was, now, "{:?} captured by its owner", town);
            prop_assert_eq!(adventure.town_garrison(i), Some(&[][..]), "{:?}", town);
        }
        let (old, new) = (before.towns().get(town), after.towns().get(town));
        let (old, new) = (old.unwrap(), new.unwrap());
        prop_assert_eq!(old.faction(), new.faction());
        let changed: Vec<bool> = events
            .iter()
            .filter_map(|e| match *e {
                GameEvent::TownChanged {
                    town: t, capitol, ..
                } if t == town => Some(capitol),
                _ => None,
            })
            .collect();
        if old == new {
            prop_assert!(changed.is_empty(), "{:?}: {:?}", town, changed);
        } else {
            prop_assert!(!captured.is_empty(), "{:?} rebuilt without a capture", town);
            prop_assert_eq!(&changed, &[after.towns().capitol(town, part)], "{:?}", town);
        }
    }
    Ok(())
}

/// A Leave Guards dialog's close (E08 T86b1; ADV-MIN-002, GAM-DEC-004 amended, increment 9's
/// design note §15's `Step::Garrison`): the group as it stood becomes the mine's guards, by slot
/// (empty: none), and the mine stays its owner's.
fn check_mine_guards(
    rules: &GameRuleset,
    (before, after): (&GameState, &GameState),
) -> Result<(), TestCaseError> {
    for player in (0..before.players()).map(|p| PlayerId::new(u8::try_from(p).unwrap())) {
        let (Some((object_type, anchor)), Some((_, group))) =
            (before.leave_guards(player), before.army_dialog(player))
        else {
            continue;
        };
        if after.army_dialog(player).is_some() {
            continue;
        }
        let map = before.adventure().map();
        let index = map
            .objects()
            .iter()
            .position(|o| (o.object_type, o.anchor) == (Some(object_type), anchor))
            .unwrap();
        prop_assert_eq!(
            after.adventure().mine_guards(rules.adventure(), index),
            group.to_vec()
        );
        prop_assert_eq!(after.adventure().owner(index), Some(player));
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
    // E08 T75b: the boats' invariants (increment 8's design note §6; ADV-SEA-001).
    let adventure = state.adventure();
    prop_assert!(
        adventure.check_boats(rules.adventure()),
        "{:?}",
        adventure.boats().collect::<Vec<_>>()
    );
    // E08 T83b1: the owners' and the weekly objects' invariants (increment 9's design note §6).
    prop_assert!(adventure.check_owners(rules.adventure()));
    prop_assert!(adventure.check_weekly(rules.adventure()));
    // E08 T83b2a: the monsters' fractions' and the objects added in play's invariants.
    prop_assert!(adventure.check_added(rules.adventure()));
    // E08 T96b1: the dwellings' and the camps' invariants (increment 10's design note §6): no
    // week's block while every dwelling is full, each count at most its growth, each guards' stack
    // within the full guards; each camp's creature of its levels, its count at most its growth.
    prop_assert!(adventure.check_dwellings());
    prop_assert!(adventure.check_camps(rules.core(), rules.adventure()));
    // E09 T9b1: no hero stands on the entrance of a town of another team or of none (TWN-VIS-002:
    // a town's visitor is of its owner's team; another team's hero besieges it from the tile
    // before, an undefended town is captured as it is entered).
    let map = adventure.map();
    for &i in map.towns() {
        let entrance = map.objects()[i].entrance(map);
        for hero in adventure
            .heroes()
            .iter()
            .filter(|h| Some(h.pos) == entrance)
        {
            let owner = adventure.owner(i);
            prop_assert!(
                owner.is_some_and(|o| h3_adventure::allied(o, hero.owner)),
                "{:?} on town {} of {:?}",
                hero.id,
                i,
                owner
            );
        }
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
            ArmyRef::Group | ArmyRef::Garrison(_) => None,
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
    (before, after): (&GameState, &GameState),
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
        check_unowned_battle_end(rules, (before, after), events, (heroes, result))?;
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
/// ADV-PIK-004), or (E08 T64b) a guarded artifact, scroll or box, the Pyramid, or a guarded
/// event lying there. A monster: a victory of the visitor's side removes it, told as its removal,
/// and gives the visitor's owner exactly its treasure; a draw removes it without one; else it
/// stays with between one and the creatures it had. A guarded object: a victory gives it after
/// the winner's levels (E08 T63b, GAM-DEC-007): in the battle's last command, told as its removal
/// (an event only with its flag; the Pyramid stays, looted) and giving the visitor's owner exactly
/// its contents (a box's and an event's record: its gains, and its losses as lost), unless the
/// winner's level-up is left open, its guards beaten meanwhile; else it stays, its guards'
/// survivors by slot each of its stack's creature and at most its count (none after a draw). An
/// event held on the tile fires after the guard's visit ([`records`]). No treasury changes
/// otherwise; no side surrenders to them or pays them. A mine's guards (E08 T86b1:
/// [`check_mine_battle_end`]) and a dwelling's (E08 T95a2: [`check_dwelling_battle_end`]) have
/// their own.
fn check_unowned_battle_end(
    rules: &GameRuleset,
    (before, after): (&GameState, &GameState),
    events: &[GameEvent],
    (heroes, result): ([Option<HeroId>; 2], BattleResult),
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
    let (adventure, map) = (before.adventure(), before.adventure().map());
    // E08 T86b1: a mine's guards ([`check_mine_battle_end`]); E08 T95a2: a dwelling's
    // ([`check_dwelling_battle_end`]).
    let guarded = visited_at(before, tile).and_then(|i| {
        let ty = adventure.object(i).and_then(|o| o.object_type);
        let kind = ty.and_then(|ty| rules.adventure().object_type(ty));
        kind.map(|t| (i, &t.kind))
    });
    match guarded {
        Some((index, ObjectKind::Mine { .. } | ObjectKind::AbandonedMine { .. })) => {
            return check_mine_battle_end(rules, (before, after), events, (visitor, index, result));
        }
        Some((index, ObjectKind::Dwelling { .. })) => {
            return check_dwelling_battle_end((before, after), events, (visitor, index, result));
        }
        Some((index, ObjectKind::Town)) => {
            return check_town_battle_end((before, after), events, (visitor, index, result));
        }
        _ => {}
    }
    // E08 T86b1: a found index may be a monster's added in play (a creature month's), read
    // through `object` (`docs/lanes/adventure/TRAPS.md`, objects added in play).
    let at = |i: usize| adventure.object(i).unwrap();
    let kind = |i: usize| {
        let ty = at(i).object_type;
        ty.and_then(|ty| rules.adventure().object_type(ty))
            .map(|t| &t.kind)
    };
    let pyramid = |i: usize| matches!(kind(i), Some(ObjectKind::Pyramid(_)));
    let live = |&i: &usize| !adventure.is_removed(i);
    let index = visited_at(before, tile)
        .filter(|&i| {
            let o = at(i);
            o.monster.is_some() || o.guards.is_some() || pyramid(i)
        })
        .or_else(|| {
            let events = map.events_at(tile).iter().copied().rev();
            events.filter(live).find(|&i| at(i).guards.is_some())
        })
        .ok_or_else(|| fail(format!("no monster or guards fought at {tile:?}")))?;
    let object = at(index);
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
        if !won && !pyramid(index) {
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
        // A level-up the battle's own experience opens holds the gift (E08 T63b, GAM-DEC-007):
        // opened before anything of the gift is told (its removal, its changes to the winner,
        // its gains); levels given at once do not hold it, nor does a level-up the gift's record
        // opens after it (a box's spells wait for that one, the box has left).
        let ended = events
            .iter()
            .position(|e| matches!(e, GameEvent::BattleEnded { .. }))
            .unwrap_or(0);
        let gift = |e: &GameEvent| match *e {
            GameEvent::Adventure(AdventureEvent {
                kind: AdventureEventKind::ObjectRemoved { anchor, .. },
                ..
            }) => anchor == object.anchor,
            GameEvent::HeroChanged { hero, change, .. } => {
                hero == visitor
                    && matches!(
                        change,
                        HeroChange::GainPrimary(_)
                            | HeroChange::RaiseSkill { .. }
                            | HeroChange::Bonus { .. }
                            | HeroChange::LearnSpell(_)
                            | HeroChange::AddArtifact(_)
                    )
            }
            GameEvent::ResourceGained { .. } => true,
            _ => false,
        };
        let after_battle = &events[ended..];
        let opened = after_battle
            .iter()
            .position(|e| matches!(e, GameEvent::LevelUpOpened { hero, .. } if *hero == visitor));
        let levels = opened.is_some_and(|p| !after_battle[..p].iter().any(gift));
        let picked = won && !levels;
        if won && !picked && !pyramid(index) {
            prop_assert!(after.adventure().guards_left(index).is_empty());
        }
        // E08 T64b: a pile's contents; an event's record (it leaves only with its flag); a
        // box's record comes with its removal ([`rewarded`]); the Pyramid stays, looted.
        let (gains, removed) = match &object.holds {
            Holds::Record(_) => (Vec::new(), picked),
            Holds::Event(event) => {
                let positive = event.record.resources.iter().filter(|r| r.1 > 0);
                let gains = positive.map(|&(r, a)| (r, a.unsigned_abs())).collect();
                (gains, picked && event.remove)
            }
            _ if pyramid(index) => (Vec::new(), false),
            _ => (object.contents.clone(), picked),
        };
        (removed, picked.then_some(gains))
    };
    prop_assert_eq!(after.adventure().is_removed(index), removed, "{:?}", result);
    // An event is in no view: its removal is told to no one (ADV-FOG-007).
    let event = matches!(object.holds, Holds::Event(_));
    prop_assert_eq!(removals, usize::from(removed && !event));
    prop_assert!(
        !matches!(result, BattleResult::Surrender(_)),
        "{:?}",
        result
    );
    // E08 T64b: then a box's record as it leaves, and an event held on the guard's tile, which
    // fires after its visit ([`records`]).
    let mut gains = gains.unwrap_or_default();
    for event in events {
        records((before, after), event, (&mut gains, &mut Vec::new()));
    }
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
            GameEvent::ResourceLost {
                player,
                resource,
                amount,
            } => {
                prop_assert_eq!(player, owner);
                treasury[usize::from(resource.raw())] -= amount;
            }
            GameEvent::ResourcePaid { .. } => {
                return Err(fail(format!("{event:?} in a battle without an enemy hero")));
            }
            _ => {}
        }
    }
    prop_assert_eq!(gained, gains, "{:?}", events);
    prop_assert_eq!(after.treasury(owner).unwrap(), &treasury[..]);
    for player in (0..before.players()).map(|p| PlayerId::new(u8::try_from(p).unwrap())) {
        if player != owner {
            prop_assert_eq!(before.treasury(player), after.treasury(player));
        }
    }
    Ok(())
}

/// The end of a siege against a town's garrison without a hero (E09 T9b1; TWN-CAP-003, design
/// §11), a neutral town's (the system played it) or another player's (its owner did): won, the
/// garrison is empty and the town is captured for the attacker's owner in the battle's last
/// command, told by its `TownCaptured` ([`check_towns`]), unless the attacker left the map (a win
/// with only created stacks) or its owner's decision (the winner's level-up) holds the capture,
/// which keeps the owner; else the owner is kept and the garrison holds survivors, each of a
/// stack it had by slot and creature with at most its count (none after a draw). No object leaves
/// the map; no treasury changes.
fn check_town_battle_end(
    (before, after): (&GameState, &GameState),
    events: &[GameEvent],
    (visitor, index, result): (HeroId, usize, BattleResult),
) -> Result<(), TestCaseError> {
    let (b, a) = (before.adventure(), after.adventure());
    let owner = b.hero(visitor).unwrap().owner;
    let (had, left) = (
        b.town_garrison(index).unwrap(),
        a.town_garrison(index).unwrap(),
    );
    let captured = events.iter().any(|e| {
        matches!(
            e,
            GameEvent::Adventure(AdventureEvent {
                kind: AdventureEventKind::TownCaptured { .. },
                ..
            })
        )
    });
    if result.winner() == Some(BattleSide::Attacker) {
        prop_assert!(left.is_empty(), "{:?} after a win", left);
        let stays = a.hero(visitor).is_some();
        if captured {
            prop_assert!(stays);
            prop_assert_eq!(a.owner(index), Some(owner));
        } else {
            prop_assert!(
                !stays || after.has_decision(owner),
                "{:?} not captured",
                result
            );
            prop_assert_eq!(a.owner(index), b.owner(index));
        }
    } else {
        prop_assert!(!captured);
        prop_assert_eq!(a.owner(index), b.owner(index));
        prop_assert!(
            left.iter().all(|s| had
                .iter()
                .any(|h| (h.slot, h.creature) == (s.slot, s.creature) && h.count >= s.count)),
            "{:?} of {:?}",
            left,
            had
        );
        prop_assert_eq!(
            left.is_empty(),
            result == BattleResult::Draw,
            "{:?}",
            result
        );
    }
    let removed = events.iter().any(|e| {
        matches!(
            e,
            GameEvent::Adventure(AdventureEvent {
                kind: AdventureEventKind::ObjectRemoved { .. },
                ..
            })
        )
    });
    prop_assert!(!removed);
    for player in (0..before.players()).map(|p| PlayerId::new(u8::try_from(p).unwrap())) {
        prop_assert_eq!(before.treasury(player), after.treasury(player));
    }
    Ok(())
}

/// The end of a battle against a mine's guards (E08 T86b1; ADV-MIN-002, GAM-BAT-001 amended,
/// increment 9's design note §13), a neutral mine's or another player's (whose owner played their
/// side): won, the guards are gone and the mine is flagged for the visitor's owner, told as its
/// flag, unless the winner's level-up is left open (the flag comes after it, GAM-DEC-007); else
/// its owner is kept and its guards are survivors, each of a stack it had by slot and creature
/// with at most its count (none after a draw). No object leaves the map; no treasury changes.
fn check_mine_battle_end(
    rules: &GameRuleset,
    (before, after): (&GameState, &GameState),
    events: &[GameEvent],
    (visitor, index, result): (HeroId, usize, BattleResult),
) -> Result<(), TestCaseError> {
    let part = rules.adventure();
    let (b, a) = (before.adventure(), after.adventure());
    let owner = b.hero(visitor).unwrap().owner;
    let (had, left) = (b.mine_guards(part, index), a.mine_guards(part, index));
    let flagged = events.iter().any(|e| {
        matches!(
            e,
            GameEvent::Adventure(AdventureEvent {
                kind: AdventureEventKind::ObjectFlagged { owner: o, .. },
                ..
            }) if *o == owner
        )
    });
    if result.winner() == Some(BattleSide::Attacker) {
        prop_assert!(left.is_empty(), "{:?} left after a win", left);
        let levels = events
            .iter()
            .any(|e| matches!(e, GameEvent::LevelUpOpened { hero, .. } if *hero == visitor));
        prop_assert_eq!(flagged, !levels);
        let expected = if levels { b.owner(index) } else { Some(owner) };
        prop_assert_eq!(a.owner(index), expected);
    } else {
        prop_assert!(!flagged);
        prop_assert_eq!(a.owner(index), b.owner(index));
        prop_assert!(
            left.iter().all(|s| had
                .iter()
                .any(|h| (h.slot, h.creature) == (s.slot, s.creature) && h.count >= s.count)),
            "{:?} of {:?}",
            left,
            had
        );
        if result == BattleResult::Draw {
            prop_assert!(left.is_empty());
        }
    }
    let removed = events.iter().any(|e| {
        matches!(
            e,
            GameEvent::Adventure(AdventureEvent {
                kind: AdventureEventKind::ObjectRemoved { .. },
                ..
            })
        )
    });
    prop_assert!(!removed, "{:?}", events);
    for player in (0..before.players()).map(|p| PlayerId::new(u8::try_from(p).unwrap())) {
        prop_assert_eq!(before.treasury(player), after.treasury(player));
    }
    Ok(())
}

/// The end of a battle against a dwelling's guards (E08 T95a2; ADV-DWL-004, GAM-BAT-001 amended,
/// increment 10's design note §7 step 2, §12), a neutral dwelling's or another player's (the
/// system played its guards, unsplit, whoever its owner): its creatures' counts are kept, no
/// object leaves the map, no treasury changes, and nothing tells its guards or its emptiness. Won,
/// the guards are gone; after the winner's levels (GAM-DEC-007) the visit goes on for a visitor
/// still on the map: the dwelling is flagged for the visitor's owner, told as its flag, then asks
/// that player's hero the hire question (its kind: [`check_decisions`]), in this command unless the
/// winner's level-up is left open, its owner kept meanwhile. Else its owner is kept, nothing is
/// flagged or asked, and its guards are survivors, each of a stack it had by slot and creature
/// with at most its count, some unless after a draw (none).
fn check_dwelling_battle_end(
    (before, after): (&GameState, &GameState),
    events: &[GameEvent],
    (visitor, index, result): (HeroId, usize, BattleResult),
) -> Result<(), TestCaseError> {
    let (b, a) = (before.adventure(), after.adventure());
    let owner = b.hero(visitor).unwrap().owner;
    // A found index read through `object` (`docs/lanes/adventure/TRAPS.md`, objects added in
    // play).
    let object = b.object(index).unwrap();
    let (object_type, anchor) = (object.object_type.unwrap(), object.anchor);
    let (had, left) = (b.dwelling_guards(index), a.dwelling_guards(index));
    prop_assert_eq!(a.dwelling_counts(index), b.dwelling_counts(index));
    let flagged = events.iter().position(|e| {
        matches!(
            e,
            GameEvent::Adventure(AdventureEvent {
                kind: AdventureEventKind::ObjectFlagged {
                    object_type: t,
                    anchor: at,
                    ..
                },
                ..
            }) if (*t, *at) == (object_type, anchor)
        )
    });
    let asked = events.iter().position(|e| {
        matches!(
            e,
            GameEvent::OfferOpened {
                player,
                hero,
                object_type: t,
                anchor: at,
            } if (*player, *hero, *t, *at) == (owner, visitor, object_type, anchor)
        )
    });
    let told = events.iter().any(|e| {
        matches!(
            e,
            GameEvent::Adventure(AdventureEvent {
                kind: AdventureEventKind::ObjectRemoved { .. }
                    | AdventureEventKind::ObjectEmpty { .. }
                    | AdventureEventKind::GuardsChanged { .. },
                ..
            })
        )
    });
    prop_assert!(!told, "{:?}", events);
    if result.winner() == Some(BattleSide::Attacker) {
        prop_assert!(left.is_empty(), "{:?} left after a win", left);
        let levels = events
            .iter()
            .any(|e| matches!(e, GameEvent::LevelUpOpened { hero, .. } if *hero == visitor));
        let rest = !levels && a.hero(visitor).is_some();
        prop_assert_eq!(flagged.is_some(), rest, "{:?}", events);
        prop_assert_eq!(asked.is_some(), rest, "{:?}", events);
        prop_assert!(
            flagged <= asked,
            "the flag after the question: {:?}",
            events
        );
        let expected = if rest { Some(owner) } else { b.owner(index) };
        prop_assert_eq!(a.owner(index), expected);
    } else {
        prop_assert_eq!((flagged, asked), (None, None), "{:?}", events);
        prop_assert_eq!(a.owner(index), b.owner(index));
        prop_assert!(
            left.iter().all(|s| had
                .iter()
                .any(|h| (h.slot, h.creature) == (s.slot, s.creature) && h.count >= s.count)),
            "{:?} of {:?}",
            left,
            had
        );
        prop_assert_eq!(left.is_empty(), result == BattleResult::Draw, "{:?}", left);
    }
    for player in (0..before.players()).map(|p| PlayerId::new(u8::try_from(p).unwrap())) {
        prop_assert_eq!(before.treasury(player), after.treasury(player));
    }
    Ok(())
}

/// The heroes' levels by their experience, in every state the plans reach (E08 T31, increment 4's
/// design note §14; HER-EXP-002, HER-EXP-003, GAM-DEC-002), against an oracle of the ruleset's
/// thresholds (a level is the number of thresholds at or below the experience, at most the highest
/// level): no hero's level is above its experience's, nor its experience above the bound (the
/// threshold after the highest level, less one) or the scenario's level limit's threshold (or its
/// own level's, if higher); a hero has a pending level only while a level-up of its owner is open
/// for it, of the level after its own. At most one level-up is open, its hero on the map, and the
/// game waits for its owner (ADR-0017 point 6) and for no one else.
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

/// The heroes' part of [`check_battle_end`] (E08 T19a2a): each change goes to one of the
/// battle's heroes and is told to its owner; every other hero's details stay; a battle hero's
/// experience grows by the gains it is told (only one that stays on the map gains), its spell
/// points become the last it is told, its artifacts change in number by the ones it is told it
/// received and lost, its level, primary and secondary skills by the levels it is told it gained
/// (E08 T30: only one that stays on the map, each the next level), and nothing else of it
/// changes; the two heroes' artifacts only move between them or leave with a destroyed war
/// machine. Since E08 T64b the winner that stays also changes by a reward's changes after the
/// battle (a guarded object's, an event's; GAM-DEC-007), applied in the order told with its
/// levels, every battle hero's bonuses end (GAM-BAT-014), and an artifact appears only as one an
/// object that left the map held (a guarded artifact's, a monster's treasure, ADV-MON-011).
fn check_heroes_after_battle(
    rules: &GameRuleset,
    (before, after): (&GameState, &GameState),
    events: &[GameEvent],
    heroes: [Option<HeroId>; 2],
) -> Result<(), TestCaseError> {
    let fought: Vec<HeroId> = heroes.iter().flatten().copied().collect();
    for event in events {
        if let GameEvent::HeroChanged { player, hero, .. } = *event {
            prop_assert!(fought.contains(&hero), "{:?} changed", hero);
            prop_assert_eq!(player, before.adventure().hero(hero).unwrap().owner);
        }
    }
    for (i, (b, a)) in before.heroes().iter().zip(after.heroes()).enumerate() {
        let id = HeroId::new(u16::try_from(i).unwrap());
        if !fought.contains(&id) {
            prop_assert_eq!(b, a);
            continue;
        }
        let (mut experience, mut mana, mut items) = (
            b.experience,
            b.mana,
            b.worn.iter().flatten().count() + b.backpack.len(),
        );
        // The rest of the details (level, primary and secondary skills, spells, bonuses) by the
        // changes and levels told, in order.
        let mut told = (**b).clone();
        let stays = after.adventure().hero(id).is_some();
        for event in events {
            match *event {
                GameEvent::HeroChanged { hero, change, .. } if hero == id => match change {
                    // E08 T31: under the bound and the scenario's level limit (HER-EXP-002, -003).
                    HeroChange::GainExperience(points) => {
                        prop_assert!(stays, "{:?} gained", id);
                        let cap = experience_cap(rules, before, b.level);
                        experience = experience.saturating_add(points).min(cap);
                    }
                    HeroChange::SetSpellPoints(points) => mana = points,
                    HeroChange::AddArtifact(_) => items += 1,
                    HeroChange::RemoveArtifact(_) => items -= 1,
                    // E08 T19a2b: only the winner that stays learns (Eagle Eye's picks; since
                    // E08 T64b a reward's spells after the battle too).
                    HeroChange::LearnSpell(_) => {
                        prop_assert!(stays, "{:?} learned", id);
                        h3_heroes::apply_change(&mut told, rules.core(), change);
                    }
                    // E08 T27: a battle tells no level; the game gives them (T30).
                    HeroChange::GainLevel { .. } => prop_assert!(false, "{:?} told a level", id),
                    // E08 T49a: nor a placed artifact (a move's).
                    HeroChange::PlaceArtifact { .. } => {
                        prop_assert!(false, "{:?} told a place", id);
                    }
                    // E08 T64b: a reward's changes after the battle, the winner's that stays (a
                    // guarded object's, an event's held on the tile; GAM-DEC-007), and the
                    // bonuses' end (GAM-BAT-014).
                    HeroChange::GainPrimary(_)
                    | HeroChange::RaiseSkill { .. }
                    | HeroChange::Bonus { .. }
                    | HeroChange::ClearBonuses => {
                        let bonuses = matches!(change, HeroChange::ClearBonuses);
                        prop_assert!(stays || bonuses, "{:?} told a reward", id);
                        h3_heroes::apply_change(&mut told, rules.core(), change);
                    }
                },
                // E08 T30: the levels it is told it gained, each the next level, only one that
                // stays on the map (GAM-DEC-002, HER-LVL-005).
                GameEvent::LevelGained {
                    player,
                    hero,
                    level,
                    primary,
                    skill,
                } if hero == id => {
                    prop_assert!(stays, "{:?} levelled", id);
                    prop_assert_eq!(player, before.adventure().hero(id).unwrap().owner);
                    prop_assert_eq!(level, told.level + 1);
                    // The offers' levels after the last level told are the hero's (one command
                    // may give several levels).
                    let change = HeroChange::GainLevel {
                        primary,
                        learned: skill,
                        offered: if level == a.level {
                            a.offered
                        } else {
                            told.offered
                        },
                    };
                    told.experience = experience;
                    let checked = h3_heroes::check_change(&told, rules.core(), change);
                    prop_assert!(checked.is_ok(), "{:?}: {:?}", change, checked);
                    h3_heroes::apply_change(&mut told, rules.core(), change);
                }
                _ => {}
            }
        }
        prop_assert_eq!((a.experience, a.mana), (experience, mana));
        prop_assert_eq!(a.worn.iter().flatten().count() + a.backpack.len(), items);
        prop_assert_eq!(
            (
                a.hero_type,
                a.level,
                a.primary,
                &a.skills,
                &a.spells,
                a.bonuses
            ),
            (
                told.hero_type,
                told.level,
                told.primary,
                &told.skills,
                &told.spells,
                told.bonuses
            )
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
    let mut held = left_with(before, after);
    for item in items(after) {
        if let Some(at) = gone.iter().position(|&i| i == item) {
            gone.remove(at);
            continue;
        }
        let at = held.iter().position(|&i| i == item);
        prop_assert!(at.is_some(), "{:?} appeared", item);
        held.remove(at.unwrap());
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

/// The artifacts the objects that left the map in a step held, as artifact and spell codes (E08
/// T64b): a placed or guarded artifact's or a scroll's, a random artifact's drawn one, a
/// monster's treasure artifact (ADV-MON-011).
fn left_with(before: &GameState, after: &GameState) -> Vec<(u16, Option<u16>)> {
    let map = before.adventure().map();
    let left = map
        .objects()
        .iter()
        .enumerate()
        .filter(|&(i, _)| !before.adventure().is_removed(i) && after.adventure().is_removed(i));
    left.filter_map(|(_, o)| match &o.holds {
        Holds::Artifact(item) => Some((item.artifact.raw(), item.spell.map(|s| s.raw()))),
        _ => o
            .resolved
            .artifact()
            .or_else(|| o.monster.as_ref().and_then(|m| m.artifact))
            .map(|artifact| (artifact.raw(), None)),
    })
    .collect()
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
/// nothing else changes either. Since E08 T64b increment 7's offers too: a guarded artifact's or
/// scroll's question as a pile's, a gift's price (the Tree of Knowledge's drawn one; a School's
/// choice refused so when the treasury cannot pay), the Warrior's Tomb's, a box's and the
/// Pyramid's questions, never refused; a choice and a University open and close as told. Since
/// E08 T95b an accepted hire question's dialog opens where [`hire_opens`] says, and closes as told.
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
                    // E08 T21a: a guarded pile asks while its guards stand, never refused; (E08
                    // T64b) a guarded artifact (ADV-PIK-007) and a guarded scroll with a text
                    // (ADV-PIK-008) too.
                    ObjectKind::Pickup(_) | ObjectKind::Artifact { .. } | ObjectKind::Scroll => {
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
                    // E08 T64b (T62a, T63a): a gift's price, the Tree of Knowledge's drawn one,
                    // refused when the treasury cannot pay it; a School's choice is refused so
                    // too (its price; ADV-TCH-002), and opens as a choice otherwise.
                    ObjectKind::Gift(gift) => OfferKind::Gift {
                        price: gift_price(before, gift, (object_type, anchor)),
                    },
                    // E08 T64b (T62a, T62b1): the Warrior's Tomb's, a box's and the Pyramid's
                    // questions, never refused.
                    ObjectKind::Find(_) | ObjectKind::PandoraBox | ObjectKind::Pyramid(_) => {
                        prop_assert!(opened, "{:?}: a question is never refused", event);
                        match kind {
                            ObjectKind::Find(_) => OfferKind::Search,
                            ObjectKind::PandoraBox => OfferKind::OpenBox,
                            _ => OfferKind::SearchPyramid,
                        }
                    }
                    // E08 T86b1: a mine neutral or another's asks while its guards stand, never
                    // refused (ADV-MIN-002).
                    ObjectKind::Mine { .. } | ObjectKind::AbandonedMine { .. } => {
                        prop_assert!(opened, "{:?}: a mine's question is never refused", event);
                        let map = before.adventure().map();
                        let (part, adventure) = (rules.adventure(), before.adventure());
                        let asks = map.objects().iter().enumerate().any(|(i, o)| {
                            (o.object_type, o.anchor) == (Some(object_type), anchor)
                                && adventure.owner(i) != Some(actor)
                                && !adventure.mine_guards(part, i).is_empty()
                        });
                        prop_assert!(asks, "{:?} without guards", event);
                        OfferKind::Fight
                    }
                    // E08 T86b1: a Shipyard's boat while its boat tile is free (ADV-SHY-002),
                    // refused when the treasury cannot pay the price.
                    ObjectKind::Shipyard { price, .. } => {
                        let map = before.adventure().map();
                        let free = map.objects().iter().any(|o| {
                            (o.object_type, o.anchor) == (Some(object_type), anchor)
                                && o.resolved.boat_tile().is_some_and(|tile| {
                                    !before.adventure().is_empty_boat(tile)
                                        && after.adventure().heroes().iter().all(|h| h.pos != tile)
                                })
                        });
                        prop_assert!(free, "{:?} without a free boat tile", event);
                        OfferKind::Boat {
                            price: price.clone(),
                        }
                    }
                    // E08 T95a1: a dwelling's hire question, of its kind's creatures, never
                    // refused (no price): its visitor's own at once, unflagged, else flagged for
                    // the visitor first, told as its flag (ADV-DWL-004, ADV-OWN-001). E08 T95a2
                    // (increment 10's design note §7 step 2, §12): one not its own whose guards
                    // stand asks the guards' question instead, never refused, its owner kept and
                    // nothing flagged: the creature of their first occupied slot and the word of
                    // their total count; once the visitor's hero beat them, in this command's
                    // battle after the winner's levels (GAM-DEC-007), it is flagged, then asks
                    // the hire question.
                    ObjectKind::Dwelling { creatures, .. } => {
                        prop_assert!(
                            opened,
                            "{:?}: a dwelling's question is never refused",
                            event
                        );
                        let index = object_index(before, object_type, anchor).unwrap();
                        let (b, a) = (before.adventure(), after.adventure());
                        let (was, guards) = (b.owner(index), b.dwelling_guards(index));
                        let won = events.iter().any(|e| {
                            matches!(
                                e,
                                GameEvent::BattleEnded { heroes, result, .. }
                                    if heroes[BattleSide::Attacker.index()] == Some(hero)
                                        && result.winner() == Some(BattleSide::Attacker)
                            )
                        });
                        let flagged = events.iter().any(|e| {
                            matches!(
                                e,
                                GameEvent::Adventure(AdventureEvent {
                                    kind: AdventureEventKind::ObjectFlagged {
                                        object_type: t,
                                        anchor: at,
                                        owner,
                                    },
                                    ..
                                }) if (*t, *at, *owner) == (object_type, anchor, actor)
                            )
                        });
                        if was != Some(actor) && !guards.is_empty() && !won {
                            prop_assert!(!flagged, "{:?} flagged with guards standing", event);
                            prop_assert_eq!(a.owner(index), was);
                            prop_assert_eq!(a.dwelling_guards(index), guards.clone());
                            let total = guards
                                .iter()
                                .fold(0, |sum: u32, s| sum.saturating_add(s.count));
                            let words = rules.adventure().params().monsters();
                            OfferKind::Guarded {
                                creature: guards[0].creature,
                                count_word: words.map_or(0, |m| m.count_word(total)),
                            }
                        } else {
                            prop_assert_eq!(flagged, was != Some(actor), "{:?}", event);
                            prop_assert_eq!(a.owner(index), Some(actor));
                            prop_assert!(
                                was == Some(actor) || a.dwelling_guards(index).is_empty(),
                                "{:?} flagged with guards standing",
                                event
                            );
                            OfferKind::Hire {
                                creatures: creatures.clone(),
                            }
                        }
                    }
                    // E08 T95a1: a Refugee Camp's, naming its week's creature while it has some
                    // left (ADV-RFG-001); never refused, never flagged.
                    ObjectKind::RefugeeCamp { .. } => {
                        prop_assert!(opened, "{:?}: a hire question is never refused", event);
                        let index = object_index(before, object_type, anchor).unwrap();
                        let camp = before.adventure().camp(index);
                        prop_assert!(camp.is_some_and(|(_, count)| count > 0), "{:?}", event);
                        prop_assert_eq!(after.adventure().owner(index), None);
                        OfferKind::Hire {
                            creatures: vec![camp.unwrap().0],
                        }
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
                    // E08 T95a2: an accepted guards' question of a dwelling is their battle
                    // (ADV-DWL-004, GAM-BAT-001 amended; increment 10's design note §7 step 2,
                    // §12), on the tile its visitor stands on, without a defending hero; while it
                    // is open, unowned whoever owns the dwelling (the system plays it), its
                    // defenders the guards as they stood, unsplit, each by slot.
                    (true, OfferKind::Guarded { .. }) => {
                        let tile = before.adventure().hero(open.hero).unwrap().pos;
                        let fought = events.iter().any(|e| {
                            matches!(
                                e,
                                GameEvent::BattleStarted { heroes, tile: at, .. }
                                    if *heroes == [Some(open.hero), None] && *at == tile
                            )
                        });
                        prop_assert!(fought, "{:?} without its battle", event);
                        if let Some(battle) = after.battle(actor) {
                            prop_assert_eq!(battle.owners(), [Some(actor), None]);
                            prop_assert_eq!(battle.tile(), tile);
                            let mut defending: Vec<_> = battle
                                .state()
                                .stacks()
                                .iter()
                                .filter(|s| s.side == BattleSide::Defender)
                                .map(|s| (s.slot, s.creature, s.count))
                                .collect();
                            defending.sort_by_key(|&(slot, ..)| slot);
                            let index = object_index(before, object_type, anchor).unwrap();
                            let guards: Vec<_> = before
                                .adventure()
                                .dwelling_guards(index)
                                .iter()
                                .map(|s| (s.slot, s.creature, s.count))
                                .collect();
                            prop_assert_eq!(defending, guards);
                        }
                    }
                    // E08 T62a, T62b1: a gift's price, a find's, a box's and the Pyramid's
                    // questions (their gains: `check_rewards`, E08 T64b).
                    (
                        true,
                        OfferKind::Gift { .. }
                        | OfferKind::Search
                        | OfferKind::OpenBox
                        | OfferKind::SearchPyramid,
                    ) => {}
                    // E08 T84b2: an accepted boat offer builds its boat (the offer stood: its
                    // tile was free; the plans place Shipyards from T86).
                    (true, OfferKind::Boat { .. }) => {
                        let built = events.iter().any(|e| {
                            matches!(
                                e,
                                GameEvent::Adventure(AdventureEvent {
                                    kind: AdventureEventKind::BoatBuilt { .. },
                                    ..
                                })
                            )
                        });
                        prop_assert!(built, "{:?} without its boat", event);
                    }
                    // E08 T95a1: an accepted hire question gives exactly its free creatures
                    // ([`free_creatures`]); (E08 T95b) after them the hire dialog opens, told to
                    // the actor as `HireOpened` with the visitor's army and the source, where
                    // [`hire_opens`] says, nowhere else (ADV-HIR-002, GAM-DEC-008).
                    (true, OfferKind::Hire { creatures }) => {
                        let told: Vec<AdventureEventKind> = events
                            .iter()
                            .filter_map(|e| match *e {
                                GameEvent::Adventure(AdventureEvent { kind, seen_by })
                                    if matches!(
                                        kind,
                                        AdventureEventKind::CreaturesJoined { .. }
                                            | AdventureEventKind::JoinRefused { .. }
                                            | AdventureEventKind::NoneToRecruit { .. }
                                    ) =>
                                {
                                    Some((kind, seen_by))
                                }
                                _ => None,
                            })
                            .map(|(kind, seen_by)| {
                                prop_assert_eq!(seen_by, Viewers::NONE.with(actor));
                                Ok(kind)
                            })
                            .collect::<Result<_, TestCaseError>>()?;
                        let offer = (open.hero, object_type, anchor);
                        let expected = free_creatures(rules, before, actor, offer, &creatures);
                        prop_assert_eq!(told, expected);
                        let opened: Vec<GameEvent> = events
                            .iter()
                            .filter(|e| matches!(e, GameEvent::HireOpened { .. }))
                            .copied()
                            .collect();
                        let source = (object_type, anchor);
                        let opens = hire_opens(rules, before, actor, source, &creatures);
                        let expected = GameEvent::HireOpened {
                            player: actor,
                            target: HireTarget::Hero(open.hero),
                            object_type,
                            anchor,
                        };
                        prop_assert_eq!(
                            opened,
                            opens.then_some(expected).into_iter().collect::<Vec<_>>()
                        );
                        let at = events.iter().position(|e| *e == expected);
                        let late = at.is_some_and(|at| {
                            events[at..].iter().any(|e| {
                                matches!(
                                    e,
                                    GameEvent::Adventure(AdventureEvent {
                                        kind: AdventureEventKind::CreaturesJoined { .. }
                                            | AdventureEventKind::JoinRefused { .. }
                                            | AdventureEventKind::NoneToRecruit { .. },
                                        ..
                                    })
                                )
                            })
                        });
                        prop_assert!(!late, "a free creature after the dialog: {:?}", events);
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
            // E08 T95a1: a visit found empty is a weekly object's (ADV-WEK-001 to -003) or a
            // Refugee Camp's without creatures (ADV-RFG-001), told to its visitor's owner alone.
            GameEvent::Adventure(AdventureEvent {
                kind:
                    AdventureEventKind::ObjectEmpty {
                        object_type,
                        anchor,
                    },
                seen_by,
            }) => {
                prop_assert_eq!(seen_by, Viewers::NONE.with(actor));
                let index = object_index(before, object_type, anchor).unwrap();
                let adventure = before.adventure();
                let weekly = adventure.map().weeklies().contains(&index);
                let deserted = adventure.camp(index).is_some_and(|(_, count)| count == 0);
                prop_assert!(weekly || deserted, "{:?}", event);
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
    // E08 T64b (T63a): a choice and a University open as told, each when nothing of these is
    // open, and close as told (GAM-DEC-005, GAM-DEC-006); (E08 T95b) a hire dialog too, which a
    // purchase of a single creature's closes (GAM-DEC-008; its purchases: [`check_hire`]).
    let dialog = |state: &GameState| state.army_dialog(actor).map(|(hero, _)| hero);
    let choosing = |state: &GameState| {
        let hero = state.choice(actor).map(|c| c.hero);
        let hero = hero.or_else(|| state.university(actor).map(|u| u.hero));
        // E08 T95b: or a hire dialog, by its army's hero.
        hero.or_else(|| {
            let HireTarget::Hero(hero) = hire_view(state, rules, actor)?.target;
            Some(hero)
        })
    };
    let mut exchange = (before.meeting(actor), dialog(before), choosing(before));
    let mut guards = false;
    for event in events {
        match *event {
            GameEvent::MeetingOpened { player, heroes } => {
                prop_assert_eq!((player, exchange), (actor, (None, None, None)));
                for hero in heroes {
                    let owner = after.adventure().hero(hero).map(|h| h.owner);
                    prop_assert_eq!(owner, Some(actor));
                }
                exchange.0 = Some(heroes);
            }
            // E08 T86b1: an own mine's Leave Guards dialog (ADV-MIN-002, GAM-DEC-004 amended)
            // opens as an army dialog, then tells each guard stack as its group's.
            GameEvent::LeaveGuardsOpened {
                player,
                hero,
                object_type,
                anchor,
            } => {
                prop_assert_eq!((player, exchange), (actor, (None, None, None)));
                let map = after.adventure().map();
                let own = map.objects().iter().enumerate().any(|(i, o)| {
                    (o.object_type, o.anchor) == (Some(object_type), anchor)
                        && after.adventure().owner(i) == Some(actor)
                });
                prop_assert!(own, "{:?} of a mine not the actor's", event);
                exchange.1 = Some(hero);
                guards = true;
            }
            GameEvent::ArmyDialogOpened { player, hero, .. } if guards => {
                prop_assert_eq!((player, exchange), (actor, (None, Some(hero), None)));
            }
            GameEvent::ArmyDialogOpened { player, hero, .. } => {
                prop_assert_eq!((player, exchange), (actor, (None, None, None)));
                exchange.1 = Some(hero);
            }
            GameEvent::ChoiceOpened { player, hero, .. }
            | GameEvent::UniversityOpened { player, hero, .. }
            | GameEvent::HireOpened {
                player,
                target: HireTarget::Hero(hero),
                ..
            } => {
                prop_assert_eq!((player, exchange), (actor, (None, None, None)));
                prop_assert_eq!(after.adventure().hero(hero).map(|h| h.owner), Some(actor));
                exchange.2 = Some(hero);
            }
            GameEvent::DecisionClosed { player } => {
                prop_assert_eq!(player, actor);
                prop_assert!(
                    exchange != (None, None, None),
                    "{:?} with nothing open",
                    event
                );
                exchange = (None, None, None);
                guards = false;
            }
            _ => {}
        }
    }
    let open = (after.meeting(actor), dialog(after), choosing(after));
    prop_assert_eq!(open, exchange);
    prop_assert_eq!(after.decision(actor), decision.as_ref());
    prop_assert!(after.adventure().bought(actor).eq(bought));
    prop_assert_eq!(after.adventure().keys(actor), keys);
    Ok(())
}

/// Whether an accepted hire question of `actor` naming `creatures` at the object of `object_type`
/// on `anchor` opens the hire dialog on `before`, the state it is accepted on (ADV-HIR-002; E08
/// T95b, increment 10's design note §13): at a dwelling `actor` owns, when one of its creatures'
/// levels is above the part's `free_level`; at a Refugee Camp, always.
fn hire_opens(
    rules: &GameRuleset,
    before: &GameState,
    actor: PlayerId,
    (object_type, anchor): (ObjectTypeId, TilePos),
    creatures: &[CreatureTypeId],
) -> bool {
    let part = rules.adventure();
    let free = part.params().dwellings().map_or(0, |p| p.free_level);
    let index = object_index(before, object_type, anchor).unwrap();
    match part.object_type(object_type).map(|t| &t.kind) {
        Some(ObjectKind::Dwelling { .. }) => {
            before.adventure().owner(index) == Some(actor)
                && creatures
                    .iter()
                    .any(|&c| rules.core().creature(c).unwrap().level > free)
        }
        Some(ObjectKind::RefugeeCamp { .. }) => true,
        _ => false,
    }
}

/// The price of the gift `gift` of the object of `object_type` on `anchor` in `state` (E08
/// T64b; ADV-TCH-003): its one price, or the one the build drew among several (the Tree of
/// Knowledge's, [`Resolved::Price`]); empty for a free one.
fn gift_price(
    state: &GameState,
    gift: &Gift,
    (object_type, anchor): (ObjectTypeId, TilePos),
) -> Vec<(ResourceId, u32)> {
    let object = state
        .adventure()
        .map()
        .objects()
        .iter()
        .find(|o| (o.object_type, o.anchor) == (Some(object_type), anchor));
    let drawn = match object.map(|o| &o.resolved) {
        Some(Resolved::Price(drawn)) => usize::from(*drawn),
        _ => 0,
    };
    gift.prices.get(drawn).cloned().unwrap_or_default()
}

/// `army`, the army of `hero` before a command, with each slot its `StackChanged`s in `events`
/// tell (E09 T9b1): a town's garrison merged into its visitor's army before a siege
/// (TWN-SIE-002), told to the visitor's owner.
fn told_army(
    army: &[h3_contracts::ArmyStack],
    hero: HeroId,
    events: &[GameEvent],
) -> Vec<h3_contracts::ArmyStack> {
    let mut army = army.to_vec();
    for event in events {
        if let GameEvent::StackChanged {
            at:
                ArmySlot {
                    army: ArmyRef::Hero(h),
                    slot,
                },
            stack,
            ..
        } = event
            && *h == hero
        {
            army.retain(|s| s.slot != *slot);
            army.extend(stack);
            army.sort_by_key(|s| s.slot);
        }
    }
    army
}

/// Ownership isolation: a command of `actor` moves no other player's hero, changes no other
/// player's army (but for a siege's merge into a town's visitor's army, E09 T9b1,
/// [`told_army`]), explored tiles (but for a new day's reveal in strict-fixed, which only adds,
/// E08 T40), treasury (but for a new day's income and the timed events at its turn's start, E08
/// T85b), purchases, open decision (E08 T14b)
/// or keys (T15a),
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
        // E09 T9b1: a siege against a town's visitor merges the garrison into its army first
        // (TWN-SIE-002), as the `StackChanged`s of its slots tell its owner.
        let besieged = events.iter().any(
            |e| matches!(e, GameEvent::BattleStarted { heroes: [_, Some(h)], .. } if *h == b.id),
        );
        let army = if besieged {
            told_army(&b.army, b.id, events)
        } else {
            b.army.clone()
        };
        prop_assert_eq!((b.pos, &army), (a.pos, &a.army));
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
            // E08 T85b: a new day pays every player its income ([`check_income`]), and a turn's
            // start fires its player's timed events ([`check_timed`]).
            let started = events.contains(&GameEvent::TurnStarted { player });
            if !(surrendered && partners.contains(&player)) && !new_day && !started {
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
    /// battle); since E09 T9b1 the towns' owners, garrisons and records ([`check_towns`]).
    #[test]
    fn every_step_keeps_the_commands_invariants(
        plan in plan(),
        strict in any::<bool>(),
        seed in any::<u64>(),
        steps in steps(),
        experience in proptest::option::weighted(0.5, vec(experience(), 6)),
    ) {
        let rules = if strict { &*STRICT } else { &*FAITHFUL };
        let m = experienced(&plan, experience.as_deref());
        let mut state = start_on(&m, rules, seed);
        // E08 T96b1: the random dwellings as the build resolved them.
        check_resolved(rules, &m, &state)?;
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
                // water and blocked tiles but ends where a hero may stop, as a walk does; (E08
                // T75b) a boat's hero at sea.
                let afloat = state.adventure().hero(hero).is_some_and(|h| h.boat.is_some());
                prop_assert!(
                    stops_on(&before, rules, at) || (afloat && is_water(&before, rules, at)),
                    "{:?} ends on {:?}",
                    hero,
                    at
                );
                check_sea_move(rules, (&before, &state), hero, path, &events)?;
            }
            let player = player_of(&before, actor);
            check_rewards(rules, (&before, &state), &command, &events)?;
            check_decisions(rules, (&before, &state), player, &events)?;
            check_isolation(&before, &state, player, &events)?;
            check_conservation(rules, (&before, &state), &command, &events)?;
            check_mine_guards(rules, (&before, &state))?;
            check_week(rules, (&before, &state), &events)?;
            check_towns(rules, (&before, &state), &events)?;
            // E08 T52: management (GAM-MAN-001), and a move's inverse.
            if let GameCommand::Manage(manage) = &command {
                check_management(rules, (&before, &state), player, manage, &events)?;
                check_inverse(rules, (&before, &state), player, manage)?;
            }
        }
        check_progression(rules, &state)?;
        check_invariants(rules, &state)?;
    }

    /// Clone equivalence: a fork playing other commands leaves its parent as it was (since E09
    /// T9b1 its store of towns too, shared until the fork's first write), and a fork and its
    /// parent evolve identically under the same commands.
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
        // E09 T9b1: the store of towns, shared by the fork and copied on its first write.
        let towns = state.towns().clone();
        let mut wild = state.clone();
        for step in rest.iter().rev() {
            let _ = advance(&mut wild, rules, step, &mut Vec::new());
        }
        prop_assert_eq!(state.state_hash(), hash);
        prop_assert_eq!(state.towns(), &towns);
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
        visited_at(state, tile)
            .is_some_and(|i| state.adventure().object(i).unwrap().guards.is_some())
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
/// checked after every command ([`check_progression`]). E08 T97b2: 320 cases (192 before): the
/// dwellings laid out on water changed the games, and a guarded pile was no longer won.
#[test]
fn the_plans_play_battles_to_a_victory() {
    use proptest::strategy::ValueTree;
    let mut runner = proptest::test_runner::TestRunner::deterministic();
    let strategy = (plan(), any::<bool>(), any::<u64>(), steps());
    let mut fought = Fought::default();
    for _ in 0..320 {
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
/// (`every_step_keeps_the_commands_invariants`). E08 T97b2: 96 cases (64 before): the dwellings
/// laid out on water changed the games, and no `NoStop` was left.
// covers: ADV-MOV-007, ADV-MOV-009, ADV-MOV-010
#[test]
fn the_plans_fly_and_walk_on_water() {
    use proptest::strategy::ValueTree;
    let mut runner = proptest::test_runner::TestRunner::deterministic();
    let strategy = (plan(), any::<bool>(), any::<u64>(), steps());
    let mut moved = Moved::default();
    for _ in 0..96 {
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

/// What a set of games visited and answered of increment 7 (E08 T64b): the object types visited
/// ([`visited`]), by key, and the answers applied, by the decision they answered and the answer.
#[derive(Debug, Default)]
struct Rewarded {
    visited: Vec<(String, usize)>,
    answered: Vec<(&'static str, Answer, usize)>,
}

impl Rewarded {
    /// Counts an applied command of `before` and its events.
    fn count(
        &mut self,
        rules: &GameRuleset,
        before: &GameState,
        command: &GameCommand,
        events: &[GameEvent],
    ) {
        if let Some(index) = visited(before, command, events) {
            let ty = before.adventure().object(index).and_then(|o| o.object_type);
            let ty = ty.and_then(|ty| rules.adventure().object_type(ty));
            if let Some(key) = ty.map(|ty| ty.key.as_str().to_owned()) {
                match self.visited.iter_mut().find(|(k, _)| *k == key) {
                    Some((_, n)) => *n += 1,
                    None => self.visited.push((key, 1)),
                }
            }
        }
        let GameCommand::Answer(answer) = *command else {
            return;
        };
        let player = acting(before);
        let decision = match before.decision(player).map(|offer| &offer.kind) {
            Some(OfferKind::Gift { .. }) => "gift",
            Some(OfferKind::Search) => "search",
            Some(OfferKind::OpenBox) => "box",
            Some(OfferKind::SearchPyramid) => "pyramid",
            Some(OfferKind::Fight) => "fight",
            Some(_) => "offer",
            None if before.choice(player).is_some() => "choice",
            None if before.university(player).is_some() => "university",
            // E08 T95b: a hire dialog's purchase or close.
            None if hiring(before, player) => "hire",
            None if before.army_dialog(player).is_some() => "dialog",
            None if before.level_up(player).is_some() => "level-up",
            None => "other",
        };
        match self
            .answered
            .iter_mut()
            .find(|(d, a, _)| (*d, *a) == (decision, answer))
        {
            Some((_, _, n)) => *n += 1,
            None => self.answered.push((decision, answer, 1)),
        }
    }
}

/// The plans visit every kind and answer every decision (E08 T64b, increment 7's design note §16;
/// increment 7's exit criteria): over a fixed set of cases (proptest's deterministic runner),
/// random play visits an object of each of increment 7's 31 classes
/// ([`super::rewards_map::CLASSES`]) and answers each of its decisions both ways: a Tree's price, a
/// Warrior's Tomb's, a box's and the Pyramid's questions and a guarded artifact's accepted and
/// declined, a chest's gold and experience, the Arena's two skills, a School's two and leaving, a
/// University's purchase and leaving, and a box's army dialog closed; the properties above check
/// each (`every_step_keeps_the_commands_invariants`, `check_rewards`). Since E08 T85b the plans'
/// timed events fire, give, and take less than they would at the floor ([`check_timed`]).
// covers: ADV-PIK-005, ADV-PIK-013, ADV-TCH-001, ADV-TCH-002, ADV-TCH-003, ADV-TCH-007
// covers: ADV-TCH-008, GAM-DEC-005, GAM-DEC-006, GAM-CAL-004
#[test]
fn the_plans_visit_every_kind_and_answer_every_decision() {
    use proptest::strategy::ValueTree;
    let mut runner = proptest::test_runner::TestRunner::deterministic();
    // Visits and legal commands (their answers among them) more often than `steps` plays them,
    // and moves across an event's tile (`None`: a step never visits one, `Step::Visit`).
    let steps = vec(
        prop_oneof![
            4 => any::<u16>().prop_map(|n| (Step::Visit(n), false)),
            4 => any::<u16>().prop_map(|n| (Step::Legal(n), false)),
            1 => any::<u16>().prop_map(|n| (Step::Legal(n), true)),
            1 => Just((Step::EndTurn, false)),
        ],
        0..120,
    );
    let strategy = (plan(), any::<bool>(), any::<u64>(), steps);
    let mut rewarded = Rewarded::default();
    let mut timed = [0; 3];
    for _ in 0..1024 {
        let (plan, strict, seed, steps) = strategy.new_tree(&mut runner).unwrap().current();
        let rules = if strict { &*STRICT } else { &*FAITHFUL };
        let mut state = start(&plan, rules, seed);
        for (step, event) in &steps {
            let mut legal = options(&state, rules);
            if *event {
                let adventure = state.adventure();
                let live = |&i: &usize| !adventure.is_removed(i);
                let crosses = |path: &[TilePos]| {
                    let map = adventure.map();
                    path.iter().any(|&at| map.events_at(at).iter().any(live))
                };
                let across: Vec<GameCommand> = legal
                    .iter()
                    .filter(|&c| as_move(c).is_some_and(|(_, path)| crosses(path)))
                    .cloned()
                    .collect();
                if !across.is_empty() {
                    legal = across;
                }
            }
            let (actor, command) = resolve(&state, rules, &legal, step);
            let before = state.clone();
            let mut events = Vec::new();
            if apply(&mut state, rules, actor, command.clone(), &mut events).is_ok() {
                rewarded.count(rules, &before, &command, &events);
                count_timed(&before, &events, &mut timed);
            }
        }
    }
    // E08 T85b: the plans' timed events fire, give and take less than they would (the floor).
    assert!(timed.iter().all(|&n| n > 0), "timed events {timed:?}");
    let r = &rewarded;
    let missed: Vec<_> = super::rewards_map::CLASSES
        .iter()
        .filter(|&&key| !r.visited.iter().any(|(v, _)| v == key))
        .collect();
    assert!(
        missed.is_empty(),
        "never visited: {missed:?} ({:?})",
        r.visited
    );
    let yes = Answer::Offer(Consent::Accept);
    let no = Answer::Offer(Consent::Decline);
    let wanted = [
        ("gift", yes),
        ("gift", no),
        ("search", yes),
        ("search", no),
        ("box", yes),
        ("box", no),
        ("pyramid", yes),
        ("pyramid", no),
        ("fight", yes),
        ("fight", no),
        ("choice", Answer::Chest(ChestChoice::Gold)),
        ("choice", Answer::Chest(ChestChoice::Experience)),
        ("choice", Answer::Arena(ArenaChoice::Attack)),
        ("choice", Answer::Arena(ArenaChoice::Defense)),
        ("choice", Answer::School(SchoolChoice::First)),
        ("choice", Answer::School(SchoolChoice::Second)),
        ("choice", Answer::School(SchoolChoice::Leave)),
        ("university", Answer::University(UniversityAnswer::Leave)),
        ("dialog", Answer::Close),
    ];
    let missed: Vec<_> = wanted
        .iter()
        .filter(|&&w| !r.answered.iter().any(|&(d, a, _)| (d, a) == w))
        .collect();
    assert!(
        missed.is_empty(),
        "never answered: {missed:?} ({:?})",
        r.answered
    );
    let bought = r.answered.iter().any(|(d, a, _)| {
        *d == "university" && matches!(a, Answer::University(UniversityAnswer::Buy(_)))
    });
    assert!(bought, "no University purchase: {:?}", r.answered);
}

/// Counts the timed events among a command's events (E08 T85b, [`check_timed`]): those that fired,
/// their gains, and their losses that took less than the event's amount (the floor at 0).
fn count_timed(before: &GameState, events: &[GameEvent], timed: &mut [usize; 3]) {
    for (at, event) in events.iter().enumerate() {
        match *event {
            GameEvent::TimedEventFired { .. } => timed[0] += 1,
            GameEvent::ResourceGained { .. } | GameEvent::ResourceLost { .. }
                if !events[..at]
                    .iter()
                    .any(|e| matches!(e, GameEvent::TurnStarted { .. })) => {}
            GameEvent::ResourceGained { .. } => timed[1] += 1,
            GameEvent::ResourceLost {
                resource, amount, ..
            } => {
                let fired = events[at..].iter().find_map(|e| match *e {
                    GameEvent::TimedEventFired { event, .. } => Some(event),
                    _ => None,
                });
                let take = before
                    .events()
                    .iter()
                    .filter(|e| Some(e.message) == fired)
                    .flat_map(|e| &e.resources)
                    .find(|&&(r, _)| r == resource)
                    .map_or(0, |&(_, amount)| amount.unsigned_abs());
                timed[2] += usize::from(amount < take);
            }
            _ => {}
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
    for _ in 0..320 {
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

/// The draws' source by the profile, on the original packs (E08 T31, increment 4's design note §14;
/// GAM-HER-002, GAM-SET-002, `ADV-QUIRK-9`): in `faithful` a hero's levels at load come from its
/// own level seed and the level (the original's generator), so the levels another hero gains at
/// load before it (in hero id order) leave its own as they are; in `strict-fixed` they come from
/// the setup stream, which the other hero's levels draw from first, so they change with those for
/// some seeds.
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
        events: Vec::new(),
        owners: Vec::new(),
        towns: Vec::new(),
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

/// What a set of games did at sea (E08 T75b): the boardings, those with free boarding (the
/// Admiral's Hat's conversion), the landings, the accepted moves of a hero in a boat and those
/// over Favorable Winds, the sea's pickups taken (Flotsam, the Sea Chest, the Shipwreck Survivor,
/// the Ocean Bottle), the battles with a hero in a boat (on the deck: against a monster or a
/// hero; on the two boats) and the boats that left the map with their heroes.
#[derive(Debug, Default)]
struct Sailed {
    boardings: usize,
    free: usize,
    landings: usize,
    sails: usize,
    winds: usize,
    pickups: [usize; 4],
    monsters: usize,
    deck: usize,
    two_boats: usize,
    boats_removed: usize,
}

impl Sailed {
    /// Counts a command `apply` accepted on `before`, leaving `after`, and its events.
    fn count(
        &mut self,
        rules: &GameRuleset,
        (before, after): (&GameState, &GameState),
        command: &GameCommand,
        events: &[GameEvent],
    ) {
        let part = rules.adventure();
        let key = |ty| part.object_type(ty).map(|ty| ty.key.to_string());
        let afloat = |state: &GameState, hero| {
            state
                .adventure()
                .hero(hero)
                .is_some_and(|h| h.boat.is_some())
        };
        if let Some((hero, path)) = as_move(command).filter(|&(hero, _)| afloat(before, hero)) {
            self.sails += 1;
            let objects = before.adventure().map().objects();
            let winds = |&at: &TilePos| {
                objects.iter().any(|o| {
                    o.lies_on(at)
                        && o.object_type.and_then(key).as_deref() == Some("sod:favorable_winds")
                })
            };
            self.winds += usize::from(path.iter().any(winds));
            let _ = hero;
        }
        for event in events {
            match event {
                GameEvent::Adventure(AdventureEvent { kind, .. }) => match *kind {
                    AdventureEventKind::Boarded { hero, .. } => {
                        self.boardings += 1;
                        let details = after.hero(hero).unwrap();
                        self.free += usize::from(h3_heroes::free_boarding(details, rules.core()));
                    }
                    AdventureEventKind::Landed { .. } => self.landings += 1,
                    AdventureEventKind::BoatRemoved { .. } => self.boats_removed += 1,
                    AdventureEventKind::ObjectRemoved { object_type, .. } => {
                        let keys = [
                            "base:flotsam",
                            "base:sea_chest",
                            "base:shipwreck_survivor",
                            "base:ocean_bottle",
                        ];
                        let found = key(object_type);
                        if let Some(i) = keys.iter().position(|&k| Some(k) == found.as_deref()) {
                            self.pickups[i] += 1;
                        }
                    }
                    _ => {}
                },
                GameEvent::BattleStarted { heroes, tile, .. } => {
                    let boats = heroes
                        .iter()
                        .flatten()
                        .filter(|&&h| afloat(after, h))
                        .count();
                    let monster = heroes[1].is_none() && is_water(after, rules, *tile);
                    self.monsters += usize::from(boats > 0 && monster);
                    self.deck += usize::from(boats == 1 && !monster);
                    self.two_boats += usize::from(boats == 2);
                }
                _ => {}
            }
        }
    }
}

/// The moves of `legal` that board an empty boat or move a hero in a boat (E08 T75b), or `legal`
/// if none does: random play's preference at sea ([`the_plans_sail`], the sea XL map's test).
pub(super) fn at_sea(state: &GameState, legal: Vec<GameCommand>) -> Vec<GameCommand> {
    let adventure = state.adventure();
    let afloat = |hero| adventure.hero(hero).is_some_and(|h| h.boat.is_some());
    let sails = |c: &&GameCommand| {
        as_move(c).is_some_and(|(hero, path)| {
            afloat(hero) || path.last().is_some_and(|&t| adventure.is_empty_boat(t))
        })
    };
    let moves: Vec<GameCommand> = legal.iter().filter(sails).cloned().collect();
    if moves.is_empty() { legal } else { moves }
}

/// The plans sail (E08 T75b, increment 8's design note §15; increment 8's exit criteria 2 to 6):
/// over a fixed set of cases (proptest's deterministic runner), with steps that prefer a boarding
/// or a move of a hero in a boat when one is legal, random play boards with and without free
/// boarding, sails, over Favorable Winds too, lands, takes each of the sea's four pickups, fights
/// at sea (a monster on water, a hero on the deck from the shore, the two boats) and loses a boat
/// with its beaten hero; every command keeps the boats' invariants and the moves the sea's rules
/// (`every_step_keeps_the_commands_invariants`: `check_invariants`, [`check_sea_move`]).
// covers: ADV-SEA-001, ADV-SEA-002, ADV-SEA-003, ADV-SEA-004, ADV-SEA-005, ADV-SEA-006
#[test]
fn the_plans_sail() {
    use proptest::strategy::ValueTree;
    let mut runner = proptest::test_runner::TestRunner::deterministic();
    let steps = vec(
        prop_oneof![
            3 => any::<u16>().prop_map(|n| (Step::Visit(n), true)),
            4 => any::<u16>().prop_map(|n| (Step::Legal(n), true)),
            1 => any::<u16>().prop_map(|n| (Step::Legal(n), false)),
            2 => any::<u16>().prop_map(|n| (Step::Fight(n), false)),
            1 => Just((Step::EndTurn, false)),
        ],
        0..120,
    );
    let strategy = (plan(), any::<bool>(), any::<u64>(), steps);
    let mut sailed = Sailed::default();
    for _ in 0..512 {
        let (plan, strict, seed, steps) = strategy.new_tree(&mut runner).unwrap().current();
        let rules = if strict { &*STRICT } else { &*FAITHFUL };
        let mut state = start(&plan, rules, seed);
        for (step, sea) in &steps {
            let mut legal = options(&state, rules);
            if *sea {
                legal = at_sea(&state, legal);
            }
            let (actor, command) = resolve(&state, rules, &legal, step);
            let before = state.clone();
            let mut events = Vec::new();
            if apply(&mut state, rules, actor, command.clone(), &mut events).is_ok() {
                sailed.count(rules, (&before, &state), &command, &events);
            }
        }
    }
    let s = &sailed;
    eprintln!("{s:?}");
    assert!(s.boardings > 0 && s.free > 0 && s.landings > 0, "{s:?}");
    assert!(
        s.sails > 0 && s.winds > 0 && s.pickups.iter().all(|&n| n > 0),
        "{s:?}"
    );
    assert!(
        s.monsters > 0 && s.deck > 0 && s.two_boats > 0 && s.boats_removed > 0,
        "{s:?}"
    );
}

/// What the plans' games did with increment 9's objects (E08 T86b1, increment 9's design note
/// §15): the flags of neutral and other players' objects, the mines' questions and their guards'
/// battles, the Leave Guards dialogs and the guards they left, the boats built at a Shipyard's
/// offer and by `BuildBoat`, a Shipyard's refusals, the weekly objects' gains and empty visits,
/// the days a mine's income was paid, the weeks' starts, the monsters a creature month added and
/// those of them visited (an offer, a battle or a join, as a placed one's).
#[derive(Debug, Default)]
struct Owned {
    flags: usize,
    taken: usize,
    questions: usize,
    mine_battles: usize,
    guards_left: usize,
    guards_kept: usize,
    offered_boats: usize,
    built_boats: usize,
    blocked: usize,
    weekly_gains: usize,
    weekly_empty: usize,
    income: usize,
    weeks: usize,
    added: usize,
    added_visited: usize,
}

impl Owned {
    fn count(
        &mut self,
        rules: &GameRuleset,
        (before, after): (&GameState, &GameState),
        command: &GameCommand,
        events: &[GameEvent],
    ) {
        let (part, adventure) = (rules.adventure(), before.adventure());
        let index_of = |object_type: ObjectTypeId, anchor: TilePos| {
            all_objects(before)
                .find(|(_, o)| (o.object_type, o.anchor) == (Some(object_type), anchor))
                .map(|(i, _)| i)
        };
        let mine = |i: usize| {
            let kind = adventure.object(i).and_then(|o| o.object_type);
            matches!(
                kind.and_then(|ty| part.object_type(ty)).map(|t| &t.kind),
                Some(ObjectKind::Mine { .. } | ObjectKind::AbandonedMine { .. })
            )
        };
        let placed = adventure.map().objects().len();
        for event in events {
            match *event {
                GameEvent::Adventure(AdventureEvent { kind, .. }) => match kind {
                    AdventureEventKind::ObjectFlagged {
                        object_type,
                        anchor,
                        ..
                    } => {
                        self.flags += 1;
                        let was = index_of(object_type, anchor).and_then(|i| adventure.owner(i));
                        self.taken += usize::from(was.is_some());
                    }
                    AdventureEventKind::BoatBuilt { .. } => {
                        if matches!(
                            command,
                            GameCommand::Adventure(AdventureCommand::BuildBoat { .. })
                        ) {
                            self.built_boats += 1;
                        } else {
                            self.offered_boats += 1;
                        }
                    }
                    AdventureEventKind::ShipyardBlocked { .. } => self.blocked += 1,
                    // E08 T95a1: a deserted Refugee Camp's is not a weekly object's.
                    AdventureEventKind::ObjectEmpty {
                        object_type,
                        anchor,
                    } => {
                        let index = index_of(object_type, anchor);
                        let weekly = |i| adventure.map().weeklies().contains(&i);
                        self.weekly_empty += usize::from(index.is_some_and(weekly));
                    }
                    AdventureEventKind::MonsterAdded { .. } => self.added += 1,
                    _ => {}
                },
                GameEvent::OfferOpened {
                    object_type,
                    anchor,
                    ..
                } => {
                    let index = index_of(object_type, anchor);
                    self.questions += usize::from(index.is_some_and(mine));
                    self.added_visited += usize::from(index.is_some_and(|i| i >= placed));
                }
                GameEvent::BattleStarted { tile, .. } => {
                    let index = visited_at(before, tile);
                    self.mine_battles += usize::from(index.is_some_and(mine));
                    self.added_visited += usize::from(index.is_some_and(|i| i >= placed));
                }
                GameEvent::DayStarted { .. } => {
                    let owned = adventure.map().ownables().iter().any(|&i| {
                        adventure.owner(i).is_some() && adventure.production(part, i).is_some()
                    });
                    self.income += usize::from(owned);
                }
                GameEvent::WeekStarted { .. } => self.weeks += 1,
                _ => {}
            }
        }
        self.weekly_gains += weekly_emptied(before, after).count();
        // A Leave Guards dialog closed: the guards it left, an empty group or not.
        for player in (0..before.players()).map(|p| PlayerId::new(u8::try_from(p).unwrap())) {
            if let (Some(_), Some((_, group)), None) = (
                before.leave_guards(player),
                before.army_dialog(player),
                after.army_dialog(player),
            ) {
                if group.is_empty() {
                    self.guards_left += 1;
                } else {
                    self.guards_kept += 1;
                }
            }
        }
        // A join of a monster added in play.
        let joined = events.iter().any(|e| {
            matches!(
                e,
                GameEvent::Adventure(AdventureEvent {
                    kind: AdventureEventKind::CreaturesJoined { .. },
                    ..
                })
            )
        });
        let gone = (placed..placed + adventure.added_objects().len())
            .any(|i| !adventure.is_removed(i) && after.adventure().is_removed(i));
        self.added_visited += usize::from(joined && gone);
    }
}

/// The plans' games flag, guard, fight for mines, build boats, visit the weekly objects, earn a
/// mine's income and meet a creature month's monsters (E08 T86b1, increment 9's design note §15):
/// every count of [`Owned`] above zero, so that the properties' checks run on them
/// ([`check_decisions`], [`check_groups`], [`check_mine_guards`], [`check_build_boat`],
/// [`check_income`], [`check_week`], [`check_unowned_battle_end`]).
#[test]
fn the_plans_flag_guard_build_and_grow() {
    use proptest::strategy::ValueTree;
    let mut runner = proptest::test_runner::TestRunner::deterministic();
    let strategy = (plan(), any::<bool>(), any::<u64>(), steps());
    let mut owned = Owned::default();
    for _ in 0..256 {
        let (plan, strict, seed, steps) = strategy.new_tree(&mut runner).unwrap().current();
        let rules = if strict { &*STRICT } else { &*FAITHFUL };
        let mut state = start(&plan, rules, seed);
        for step in &steps {
            let legal = options(&state, rules);
            let (actor, command) = resolve(&state, rules, &legal, step);
            let before = state.clone();
            let mut events = Vec::new();
            if apply(&mut state, rules, actor, command.clone(), &mut events).is_ok() {
                owned.count(rules, (&before, &state), &command, &events);
            }
        }
    }
    let o = &owned;
    eprintln!("{o:?}");
    assert!(o.flags > 0 && o.taken > 0 && o.questions > 0, "{o:?}");
    assert!(
        o.mine_battles > 0 && o.guards_left > 0 && o.guards_kept > 0,
        "{o:?}"
    );
    assert!(
        o.offered_boats > 0 && o.built_boats > 0 && o.blocked > 0,
        "{o:?}"
    );
    assert!(o.weekly_gains > 0 && o.weekly_empty > 0, "{o:?}");
    assert!(o.income > 0 && o.weeks > 0, "{o:?}");
    assert!(o.added > 0 && o.added_visited > 0, "{o:?}");
}

/// What the plans' games did with increment 10's objects (E08 T96b1, increment 10's design note
/// §14): the visits of each kind of [`DWELLINGS`] (a question opened there, a camp found deserted;
/// a random dwelling's after the build resolved it), the hire questions and the guards' questions
/// accepted and declined, the guards' battles and those won, the free creatures joined, the
/// purchases in a dwelling's and in a camp's dialog, the dialogs closed, the weeks that refilled a
/// dwelling, and the random dwellings resolved at the builds; the free creatures missing
/// (`NoneToRecruit`), the camps found deserted (`ObjectEmpty`) and the weeks started with a camp on
/// the map ([`check_dwellings_week`]'s camps).
#[derive(Debug, Default)]
struct Hired {
    visits: [usize; DWELLINGS.1],
    hire_accepted: usize,
    hire_declined: usize,
    guards_accepted: usize,
    guards_declined: usize,
    guard_battles: usize,
    guards_beaten: usize,
    free_joined: usize,
    dwelling_buys: usize,
    camp_buys: usize,
    closes: usize,
    refills: usize,
    resolved: usize,
    none_to_recruit: usize,
    camps_deserted: usize,
    camp_weeks: usize,
    /// The visitors of the guards' battles in progress.
    fighting: Vec<HeroId>,
}

impl Hired {
    /// Counts what `command` did on its way from `before` to `after`, `kinds` the plan's kind of
    /// each object of the static map, by index.
    fn count(
        &mut self,
        rules: &GameRuleset,
        (before, after): (&GameState, &GameState),
        (command, events): (&GameCommand, &[GameEvent]),
        kinds: &[usize],
    ) {
        let part = rules.adventure();
        let player = acting(before);
        let open = before.decision(player).map(|o| &o.kind);
        let mut visit = |object_type, anchor| {
            let kind = object_index(before, object_type, anchor).and_then(|i| kinds.get(i));
            if let Some(&kind) = kind.filter(|&&k| (DWELLINGS.0..CAMP + 1).contains(&k)) {
                self.visits[kind - DWELLINGS.0] += 1;
            }
        };
        for event in events {
            match *event {
                GameEvent::OfferOpened {
                    object_type,
                    anchor,
                    ..
                }
                | GameEvent::Adventure(AdventureEvent {
                    kind:
                        AdventureEventKind::ObjectEmpty {
                            object_type,
                            anchor,
                        },
                    ..
                }) => visit(object_type, anchor),
                _ => {}
            }
        }
        for event in events {
            match *event {
                GameEvent::OfferAnswered { accepted, .. } => match open {
                    Some(OfferKind::Hire { .. }) if accepted => self.hire_accepted += 1,
                    Some(OfferKind::Hire { .. }) => self.hire_declined += 1,
                    Some(OfferKind::Guarded { .. }) if accepted => self.guards_accepted += 1,
                    Some(OfferKind::Guarded { .. }) => self.guards_declined += 1,
                    _ => {}
                },
                GameEvent::BattleStarted { heroes, .. }
                    if matches!(open, Some(OfferKind::Guarded { .. })) =>
                {
                    self.guard_battles += 1;
                    self.fighting.extend(heroes[0]);
                }
                GameEvent::BattleEnded { heroes, result, .. }
                    if heroes[0].is_some_and(|h| self.fighting.contains(&h)) =>
                {
                    self.fighting.retain(|&h| Some(h) != heroes[0]);
                    if result.winner() == Some(BattleSide::Attacker) {
                        self.guards_beaten += 1;
                    }
                }
                GameEvent::Adventure(AdventureEvent {
                    kind: AdventureEventKind::CreaturesJoined { .. },
                    ..
                }) if matches!(open, Some(OfferKind::Hire { .. })) => self.free_joined += 1,
                GameEvent::WeekStarted { .. } => {
                    let (b, a) = (before.adventure(), after.adventure());
                    let refilled = b.map().dwellings().iter().any(|&i| {
                        (b.dwelling_counts(i), b.dwelling_guards(i))
                            != (a.dwelling_counts(i), a.dwelling_guards(i))
                    });
                    self.refills += usize::from(refilled);
                    self.camp_weeks += usize::from(!a.map().camps().is_empty());
                }
                GameEvent::Adventure(AdventureEvent {
                    kind: AdventureEventKind::NoneToRecruit { .. },
                    ..
                }) => self.none_to_recruit += 1,
                GameEvent::Adventure(AdventureEvent {
                    kind: AdventureEventKind::ObjectEmpty { object_type, .. },
                    ..
                }) if matches!(
                    part.object_type(object_type).map(|t| &t.kind),
                    Some(ObjectKind::RefugeeCamp { .. })
                ) =>
                {
                    self.camps_deserted += 1;
                }
                _ => {}
            }
        }
        if let Some(view) = hire_view(before, rules, player) {
            let camp = matches!(
                part.object_type(view.object_type).map(|t| &t.kind),
                Some(ObjectKind::RefugeeCamp { .. })
            );
            match command {
                GameCommand::Answer(Answer::Hire(HireAnswer::Buy { .. })) if camp => {
                    self.camp_buys += 1;
                }
                GameCommand::Answer(Answer::Hire(HireAnswer::Buy { .. })) => {
                    self.dwelling_buys += 1;
                }
                GameCommand::Answer(Answer::Hire(HireAnswer::Close)) => self.closes += 1,
                _ => {}
            }
        }
    }
}

/// The plans' games visit dwellings, random dwellings and Refugee Camps, answer their questions,
/// fight their guards and hire (E08 T96b1, increment 10's design note §14): every count of
/// [`Hired`] above zero, so that the properties' checks run on them ([`check_decisions`]' hire
/// and guards' questions and [`free_creatures`], [`check_dwelling_battle_end`], [`check_hire`],
/// [`check_conservation`]'s purchases, [`check_week`]'s refill and camps, [`check_resolved`];
/// `view.rs`'s `check_hire_sources` on `NoneToRecruit` and a deserted camp's `ObjectEmpty`).
// covers: ADV-DWL-001, ADV-DWL-002, ADV-DWL-003, ADV-DWL-004, ADV-DWL-005
// covers: ADV-HIR-001, ADV-HIR-002, ADV-RFG-001, GAM-DEC-008
#[test]
fn the_plans_visit_dwellings_and_hire() {
    use proptest::strategy::ValueTree;
    let mut runner = proptest::test_runner::TestRunner::deterministic();
    let strategy = (plan(), any::<bool>(), any::<u64>(), steps());
    let mut hired = Hired::default();
    for _ in 0..256 {
        let (plan, strict, seed, steps) = strategy.new_tree(&mut runner).unwrap().current();
        let rules = if strict { &*STRICT } else { &*FAITHFUL };
        let m = lay_out(&plan);
        let mut state = start_on(&m, rules, seed);
        hired.resolved += check_resolved(rules, &m, &state).unwrap();
        // The plan's kind of each object of the static map (the import keeps every object but
        // the heroes and the boats, in the map's order); (E09 T9b1) the towns, laid out after
        // the other objects, have none.
        let kinds: Vec<usize> = m
            .objects
            .iter()
            .filter(|o| o.template != HERO_TEMPLATE && !matches!(o.body, Body::Town(_)))
            .map(|o| usize::try_from(o.template - 1).unwrap() / BLOCKERS.len())
            .filter(|&kind| KINDS[kind].0 != 8)
            .collect();
        for step in &steps {
            let legal = options(&state, rules);
            let (actor, command) = resolve(&state, rules, &legal, step);
            let before = state.clone();
            let mut events = Vec::new();
            if apply(&mut state, rules, actor, command.clone(), &mut events).is_ok() {
                hired.count(rules, (&before, &state), (&command, &events), &kinds);
            }
        }
    }
    let h = &hired;
    eprintln!("{h:?}");
    assert!(h.visits.iter().all(|&n| n > 0), "{h:?}");
    assert!(h.hire_accepted > 0 && h.hire_declined > 0, "{h:?}");
    assert!(h.guards_accepted > 0 && h.guards_declined > 0, "{h:?}");
    assert!(h.guard_battles > 0 && h.guards_beaten > 0, "{h:?}");
    assert!(
        h.free_joined > 0 && h.dwelling_buys > 0 && h.camp_buys > 0,
        "{h:?}"
    );
    assert!(h.closes > 0 && h.refills > 0 && h.resolved > 0, "{h:?}");
    assert!(
        h.none_to_recruit > 0 && h.camps_deserted > 0 && h.camp_weeks > 0,
        "{h:?}"
    );
}

/// What the plans' games did at a dwelling entered from water (E08 T97b2; survey
/// `E08-dwellings.md` §17): the questions a dwelling (a random one as the build resolved it) asked
/// a hero in a boat (its guards' question or its hire question), the flags it took for that hero's
/// owner, the guards' battles fought on the deck at its entrance (the visitor in its boat, no
/// defending hero), and the creatures that joined a hero in a boat there (free, or bought in the
/// hire dialog).
#[derive(Debug, Default)]
struct Afloat {
    questions: usize,
    flags: usize,
    deck: usize,
    joined: usize,
}

impl Afloat {
    /// Counts what `command` did on its way from `before` to `after`.
    fn count(
        &mut self,
        rules: &GameRuleset,
        (before, after): (&GameState, &GameState),
        command: &GameCommand,
        events: &[GameEvent],
    ) {
        let part = rules.adventure();
        let player = acting(before);
        let in_boat = |state: &GameState, hero| {
            state
                .adventure()
                .hero(hero)
                .is_some_and(|h| h.boat.is_some())
        };
        let dwelling = |object_type| {
            matches!(
                part.object_type(object_type).map(|t| &t.kind),
                Some(ObjectKind::Dwelling { .. })
            )
        };
        // The visitor of the command: a hero in a boat moving, or the attacker of a battle the
        // command ended.
        let sailor = as_move(command)
            .map(|(hero, _)| hero)
            .or_else(|| ended(events).first().and_then(|(heroes, _)| heroes[0]))
            .filter(|&hero| in_boat(before, hero));
        let hiring = matches!(
            before.decision(player).map(|o| &o.kind),
            Some(OfferKind::Hire { .. })
        ) || hire_view(before, rules, player).is_some();
        for event in events {
            match *event {
                GameEvent::OfferOpened {
                    hero, object_type, ..
                } if dwelling(object_type) && in_boat(after, hero) => self.questions += 1,
                GameEvent::Adventure(AdventureEvent {
                    kind: AdventureEventKind::ObjectFlagged { object_type, .. },
                    ..
                }) if dwelling(object_type) && sailor.is_some() => self.flags += 1,
                GameEvent::BattleStarted {
                    heroes: [Some(hero), None],
                    tile,
                    ..
                } if in_boat(after, hero) => {
                    let ty = visited_at(before, tile)
                        .and_then(|i| before.adventure().object(i))
                        .and_then(|o| o.object_type);
                    self.deck += usize::from(ty.is_some_and(dwelling));
                }
                GameEvent::Adventure(AdventureEvent {
                    kind: AdventureEventKind::CreaturesJoined { hero, .. },
                    ..
                }) if hiring && in_boat(after, hero) => self.joined += 1,
                _ => {}
            }
        }
    }
}

/// The boat plans (E08 T97b2): a plan, the profile (strict or not), the seed and up to 119 steps
/// that prefer a boarding or a move of a hero in a boat when one is legal (the step's flag:
/// [`at_sea`]) and enter a dwelling or a camp ([`Step::Hire`]); what
/// [`the_plans_visit_dwellings_from_a_boat`] counts and `view.rs`'s
/// `boat_visits_are_told_as_they_happen` checks, over proptest's deterministic runner.
pub(super) fn afloat_plans() -> impl Strategy<Value = (Plan, bool, u64, Vec<(Step, bool)>)> {
    let steps = vec(
        prop_oneof![
            3 => any::<u16>().prop_map(|n| (Step::Visit(n), true)),
            3 => any::<u16>().prop_map(|n| (Step::Legal(n), true)),
            4 => any::<u16>().prop_map(|n| (Step::Hire(n), true)),
            1 => any::<u16>().prop_map(|n| (Step::Legal(n), false)),
            1 => Just((Step::EndTurn, false)),
        ],
        0..120,
    );
    (plan(), any::<bool>(), any::<u64>(), steps)
}

/// The plans' games visit dwellings entered from water from a boat (E08 T97b2; survey
/// `E08-dwellings.md` §17, increment 10's exit criteria): over a fixed set of cases (proptest's
/// deterministic runner), with steps that prefer a boarding or a move of a hero in a boat when one
/// is legal and steps that enter a dwelling ([`afloat_plans`]), a hero in a boat is
/// asked a dwelling's questions, flags one, fights its guards on the deck at the entrance and takes
/// its creatures: every count of [`Afloat`] above zero, so that the properties' checks run on such
/// visits ([`check_decisions`]' guards' and hire questions and [`free_creatures`],
/// [`check_dwelling_battle_end`], [`check_hire`], [`check_sea_move`]; `view.rs`'s
/// `check_hire_sources`).
// covers: ADV-DWL-004, ADV-OWN-002, ADV-SEA-005
#[test]
fn the_plans_visit_dwellings_from_a_boat() {
    use proptest::strategy::ValueTree;
    let mut runner = proptest::test_runner::TestRunner::deterministic();
    let strategy = afloat_plans();
    let mut afloat = Afloat::default();
    for _ in 0..512 {
        let (plan, strict, seed, steps) = strategy.new_tree(&mut runner).unwrap().current();
        let rules = if strict { &*STRICT } else { &*FAITHFUL };
        let mut state = start(&plan, rules, seed);
        for (step, sea) in &steps {
            let mut legal = options(&state, rules);
            if *sea {
                legal = at_sea(&state, legal);
            }
            let (actor, command) = resolve(&state, rules, &legal, step);
            let before = state.clone();
            let mut events = Vec::new();
            if apply(&mut state, rules, actor, command.clone(), &mut events).is_ok() {
                afloat.count(rules, (&before, &state), &command, &events);
            }
        }
    }
    let a = &afloat;
    eprintln!("{a:?}");
    assert!(a.questions > 0 && a.flags > 0, "{a:?}");
    assert!(a.deck > 0 && a.joined > 0, "{a:?}");
}

/// What the plans' games did with their towns (E09 T9b1; the town epic's design note §18): the
/// towns the builds made; the own towns entered (TWN-VIS-001); the undefended towns captured by a
/// step, neutral and another player's (TWN-CAP-001); the sieges against a garrison alone and
/// against a visitor (TWN-SIE-001, -002), each won (the town captured after, TWN-CAP-003) or not;
/// the merges that changed a garrison's slots before a siege (TWN-SIE-002); the captures, every
/// way; and the Capitols the limit took (TWN-CAP-002).
#[derive(Debug, Default)]
struct Besieged {
    towns: usize,
    entered: usize,
    undefended: [usize; 2],
    sieges: [usize; 2],
    won: [usize; 2],
    lost: [usize; 2],
    merges: usize,
    captures: usize,
    limited: usize,
    /// The sieges in progress: the attacking hero, and whether a visitor defends.
    open: Vec<(HeroId, usize)>,
}

impl Besieged {
    /// Counts what `command` did on its way from `before` to `after`.
    fn count(
        &mut self,
        (before, after): (&GameState, &GameState),
        command: &GameCommand,
        events: &[GameEvent],
    ) {
        let map = before.adventure().map();
        let entrances: Vec<(TilePos, usize)> = map
            .towns()
            .iter()
            .filter_map(|&i| Some((map.objects()[i].entrance(map)?, i)))
            .collect();
        let town_at = |pos: TilePos| entrances.iter().find(|&&(e, _)| e == pos).map(|&(_, i)| i);
        let mut captured = false;
        for event in events {
            match event {
                GameEvent::Adventure(AdventureEvent {
                    kind: AdventureEventKind::TownCaptured { town, .. },
                    ..
                }) => {
                    self.captures += 1;
                    captured = true;
                    if as_move(command).is_some() {
                        let i = map.town_object(*town).unwrap();
                        self.undefended[usize::from(before.adventure().owner(i).is_some())] += 1;
                    }
                }
                GameEvent::BattleStarted { heroes, tile, .. } if town_at(*tile).is_some() => {
                    let visitor = usize::from(heroes[1].is_some());
                    self.sieges[visitor] += 1;
                    self.open.push((heroes[0].unwrap(), visitor));
                }
                GameEvent::BattleEnded { heroes, result, .. } => {
                    let siege = self.open.iter().position(|&(h, _)| Some(h) == heroes[0]);
                    if let Some(siege) = siege {
                        let (_, visitor) = self.open.swap_remove(siege);
                        let won = matches!(
                            result,
                            BattleResult::Victory(BattleSide::Attacker)
                                | BattleResult::Retreat(BattleSide::Defender)
                                | BattleResult::Surrender(BattleSide::Defender)
                        );
                        if won {
                            self.won[visitor] += 1;
                        } else {
                            self.lost[visitor] += 1;
                        }
                    }
                }
                GameEvent::TownChanged { capitol: false, .. } => self.limited += 1,
                _ => {}
            }
        }
        let merged = events.iter().any(|e| {
            matches!(
                e,
                GameEvent::StackChanged {
                    at: ArmySlot {
                        army: ArmyRef::Garrison(_),
                        ..
                    },
                    ..
                }
            )
        });
        self.merges += usize::from(merged);
        if let Some((hero, _)) = as_move(command)
            && !captured
            && let Some(h) = after.adventure().hero(hero)
            && let Some(i) = town_at(h.pos)
            && before.adventure().owner(i) == Some(h.owner)
            && before
                .adventure()
                .hero(hero)
                .is_some_and(|b| b.pos != h.pos)
        {
            self.entered += 1;
        }
    }
}

/// The plans play towns (E09 T9b1; the town epic's design note §18, exit criterion 5): over a
/// fixed set of cases (proptest's deterministic runner), random play with the towns' runs
/// ([`steps`], [`Step::Town`]) enters own towns, captures undefended ones (neutral and another
/// player's), besieges towns defended by their garrison alone and by a visitor with the garrison
/// merged, and wins and loses both, and the Capitol's limit takes a Capitol, so the properties
/// above ([`check_towns`], the entrances in [`check_invariants`], the forks' store) see each.
#[test]
fn the_plans_capture_and_besiege_towns() {
    use proptest::strategy::ValueTree;
    let mut runner = proptest::test_runner::TestRunner::deterministic();
    // More towns than the properties' plans have, so that their games meet each case.
    let towns = (plan(), vec(town(), 3..=6)).prop_map(|(plan, towns)| Plan { towns, ..plan });
    let strategy = (towns, any::<bool>(), any::<u64>(), steps());
    let mut besieged = Besieged::default();
    for _ in 0..512 {
        let (plan, strict, seed, steps) = strategy.new_tree(&mut runner).unwrap().current();
        let rules = if strict { &*STRICT } else { &*FAITHFUL };
        let mut state = start(&plan, rules, seed);
        besieged.towns += state.towns().len();
        besieged.open.clear();
        for step in &steps {
            let legal = options(&state, rules);
            let (actor, command) = resolve(&state, rules, &legal, step);
            let before = state.clone();
            let mut events = Vec::new();
            if apply(&mut state, rules, actor, command.clone(), &mut events).is_ok() {
                besieged.count((&before, &state), &command, &events);
            }
        }
    }
    let b = &besieged;
    eprintln!("{b:?}");
    assert!(b.towns > 0 && b.entered > 0, "{b:?}");
    assert!(b.undefended.iter().all(|&n| n > 0), "{b:?}");
    assert!(b.sieges.iter().all(|&n| n > 0), "{b:?}");
    assert!(
        b.won.iter().all(|&n| n > 0) && b.lost.iter().all(|&n| n > 0),
        "{b:?}"
    );
    assert!(b.merges > 0 && b.limited > 0, "{b:?}");
    assert!(b.captures > b.undefended.iter().sum::<usize>(), "{b:?}");
}
