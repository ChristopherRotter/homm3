//! Spell obstacles on the battlefield (E04b T5a; ADR-0014 points 5 and 7): what an obstacle
//! spell places (BAT-MAG-027: Force Field blocks its hexes), Fire Wall's damage to a stack that
//! enters one of its hexes or acts on one (BAT-MAG-028), the rounds they last, and Expert
//! Dispel's clearing (BAT-MAG-030). The original's objects and their triggers:
//! `docs/formats/installation.md` §7.15.
//!
//! The obstacles live in one growable list per battle in the order they were placed (ADR-0014
//! point 5). Most battles have none: the list reserves its room when the first obstacle is
//! placed, and a fork reserves it only if it has obstacles, so a fork of a battle without them
//! still costs one allocation (the effect list's) and a battle with them allocates once. The
//! hexes they block and those that burn are kept as sets beside the list, so the hot paths
//! (movement, a move without spell obstacles) read a set and never the list.

use alloc::vec::Vec;

use h3_content::{ObstacleKind, ObstacleShape, Ruleset, SpellId};
use h3_contracts::BattleSide;
use h3_kernel::{CanonicalHash, FixedVec, StateHasher};

use crate::combat::check_end;
use crate::event::{Event, EventKind, Sink};
use crate::geometry::{Direction, Field, Hex, HexSet};
use crate::magic;
use crate::movement;
use crate::state::{BattleState, StackId, StackTraits, side_code};

/// Obstacles reserved with the first one and on every fork that has any (ADR-0014 point 5).
pub const OBSTACLE_CAPACITY: usize = 8;

/// One spell obstacle (ADR-0014 point 7; the original's battlefield object, `installation.md`
/// §7.15).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpellObstacle {
    /// The spell that placed it.
    pub spell: SpellId,
    /// What it does.
    pub kind: ObstacleKind,
    /// The side whose hero cast it.
    pub side: BattleSide,
    /// Its hexes.
    pub hexes: HexSet,
    /// Rounds left, the current one included.
    pub rounds: u8,
    /// Fire Wall's value at the cast: `power × power factor + effect[mastery]`, before any bonus
    /// (BAT-MAG-028); 0 for other kinds.
    pub value: u64,
}

impl CanonicalHash for SpellObstacle {
    /// Spell, kind, side, hexes, rounds, value.
    fn hash_into(&self, h: &mut StateHasher) {
        self.spell.hash_into(h);
        h.write_u8(self.kind as u8);
        h.write_u8(side_code(self.side));
        self.hexes.hash_into(h);
        h.write_u8(self.rounds);
        h.write_u64(self.value);
    }
}

/// The battle's spell obstacles, in the order they were placed, with the hexes Force Fields
/// block and those Fire Walls burn.
#[derive(Debug, PartialEq, Eq)]
pub struct SpellObstacles {
    list: Vec<SpellObstacle>,
    walls: HexSet,
    fire: HexSet,
}

impl SpellObstacles {
    /// None, nothing reserved yet.
    pub(crate) const fn new() -> Self {
        Self {
            list: Vec::new(),
            walls: HexSet::new(),
            fire: HexSet::new(),
        }
    }

    /// Every obstacle, in the order they were placed.
    pub fn as_slice(&self) -> &[SpellObstacle] {
        &self.list
    }

    /// The hexes the obstacles block (Force Field, BAT-MAG-027).
    pub const fn walls(&self) -> HexSet {
        self.walls
    }

    /// The hexes that burn (Fire Wall, BAT-MAG-028).
    pub const fn fire(&self) -> HexSet {
        self.fire
    }

    /// Every hex an obstacle covers.
    pub fn hexes(&self) -> HexSet {
        self.list
            .iter()
            .fold(HexSet::new(), |set, o| set.union(o.hexes))
    }

    /// Recomputes the blocked and burning hexes from the list.
    fn refresh(&mut self) {
        let of = |kind| {
            self.list
                .iter()
                .filter(|o| o.kind == kind)
                .fold(HexSet::new(), |set, o| set.union(o.hexes))
        };
        self.walls = of(ObstacleKind::Wall);
        self.fire = of(ObstacleKind::Fire);
    }

    /// The first Fire Wall obstacle on `hex`.
    fn fire_at(&self, hex: Hex) -> Option<SpellObstacle> {
        self.list
            .iter()
            .find(|o| o.kind == ObstacleKind::Fire && o.hexes.contains(hex))
            .copied()
    }
}

impl Clone for SpellObstacles {
    /// A fork keeps the reserve if it has obstacles (one allocation), else allocates nothing.
    fn clone(&self) -> Self {
        let mut list = Vec::new();
        if !self.list.is_empty() {
            list.reserve_exact(self.list.len().max(OBSTACLE_CAPACITY));
            list.extend_from_slice(&self.list);
        }
        Self {
            list,
            walls: self.walls,
            fire: self.fire,
        }
    }
}

impl CanonicalHash for SpellObstacles {
    /// Count, then the obstacles in order (the hex sets follow from them).
    fn hash_into(&self, h: &mut StateHasher) {
        h.write_len(self.list.len());
        for o in &self.list {
            o.hash_into(h);
        }
    }
}

/// The hexes of `shape` cast by `side`'s hero at `mastery` on `target`, in order: the target hex,
/// then each step's neighbour, mirrored for the defender if the shape says so (BAT-MAG-027);
/// `None` if one leaves the field.
pub(crate) fn shape_hexes(
    field: Field,
    shape: &ObstacleShape,
    (side, mastery): (BattleSide, u8),
    target: Hex,
) -> Option<FixedVec<Hex, { h3_content::MAX_OBSTACLE_HEXES }>> {
    let count = usize::from(shape.hexes[usize::from(mastery)]);
    let mut out = FixedVec::new();
    let mut at = target;
    // The compiler checked that `count` is at most the steps plus one, at most 8.
    let _ = out.push(at);
    for &step in shape.steps.iter().take(count.saturating_sub(1)) {
        let step = if shape.mirrored && side == BattleSide::Defender {
            step.mirrored()
        } else {
            step
        };
        at = field.neighbor(at, Direction::from_step(step))?;
        let _ = out.push(at);
    }
    Some(out)
}

/// Whether an obstacle may go on every hex of `hexes` (BAT-MAG-027; the original's target test,
/// `installation.md` §7.15): each outside the side columns, without an obstacle of either kind
/// and without a living stack.
pub(crate) fn free(state: &BattleState, field: Field, hexes: &[Hex]) -> bool {
    let taken = state
        .obstacles
        .union(state.spell_obstacles.hexes())
        .union(state.living_hexes(field));
    hexes
        .iter()
        .all(|&h| field.is_standable(h) && !taken.contains(h))
}

/// Places the obstacles of kind `kind` and shape `shape` of `side`'s cast of `id` on `hexes` with
/// the value `value` (BAT-MAG-027): one per hex for a separate shape (Fire Wall), else one over
/// all; each reported.
pub(crate) fn place<S: Sink>(
    state: &mut BattleState,
    (id, kind, shape): (SpellId, ObstacleKind, &ObstacleShape),
    side: BattleSide,
    (hexes, value): (&[Hex], u64),
    sink: &mut S,
) {
    let value = if kind == ObstacleKind::Fire { value } else { 0 };
    let make = |set: HexSet| SpellObstacle {
        spell: id,
        kind,
        side,
        hexes: set,
        rounds: shape.rounds,
        value,
    };
    let mut add = |o: SpellObstacle| {
        let list = &mut state.spell_obstacles.list;
        if list.capacity() == 0 {
            list.reserve_exact(OBSTACLE_CAPACITY);
        }
        list.push(o);
        sink.emit(Event::public(EventKind::ObstaclePlaced {
            side,
            spell: id,
            hexes: o.hexes,
        }));
    };
    if shape.separate {
        for &h in hexes {
            let mut set = HexSet::new();
            set.insert(h);
            add(make(set));
        }
    } else {
        add(make(hexes.iter().fold(HexSet::new(), |mut set, &h| {
            set.insert(h);
            set
        })));
    }
    state.spell_obstacles.refresh();
}

/// Removes the obstacles `gone` picks, each reported.
fn remove<S: Sink>(
    state: &mut BattleState,
    gone: impl Fn(&mut SpellObstacle) -> bool,
    sink: &mut S,
) {
    let before = state.spell_obstacles.list.len();
    state.spell_obstacles.list.retain_mut(|o| {
        let removed = gone(o);
        if removed {
            sink.emit(Event::public(EventKind::ObstacleRemoved {
                spell: o.spell,
                hexes: o.hexes,
            }));
        }
        !removed
    });
    if state.spell_obstacles.list.len() != before {
        state.spell_obstacles.refresh();
    }
}

/// Starts a round for the spell obstacles (BAT-MAG-027): each either goes (its last round has
/// passed) or loses a round; after the timed effects (`installation.md` §7.15).
// rule: BAT-MAG-027
pub(crate) fn count_down<S: Sink>(state: &mut BattleState, sink: &mut S) {
    if state.spell_obstacles.list.is_empty() {
        return;
    }
    remove(
        state,
        |o| {
            o.rounds = o.rounds.saturating_sub(1);
            o.rounds == 0
        },
        sink,
    );
}

/// Removes every spell obstacle, whoever cast it (Expert Dispel; BAT-MAG-030).
// rule: BAT-MAG-030
pub(crate) fn clear<S: Sink>(state: &mut BattleState, sink: &mut S) {
    remove(state, |_| true, sink);
}

/// The burning hexes stack `s` enters, `(hex, its Fire Wall)`: those of `hexes` a Fire Wall
/// covers, in order.
fn burning(state: &BattleState, hexes: [Option<Hex>; 2]) -> [Option<(Hex, SpellObstacle)>; 2] {
    hexes.map(|h| h.and_then(|h| state.spell_obstacles.fire_at(h).map(|o| (h, o))))
}

/// Burns stack `s` on each of `entered`, its head's hex first, while it lives (BAT-MAG-028); the
/// battle may end. Returns whether it still lives.
fn burn_hexes<S: Sink>(
    state: &mut BattleState,
    ruleset: &Ruleset,
    s: StackId,
    entered: [Option<(Hex, SpellObstacle)>; 2],
    sink: &mut S,
) -> bool {
    for (hex, fire) in entered.into_iter().flatten() {
        if !state.at(s).is_alive() {
            break;
        }
        sink.emit(Event::public(EventKind::ObstacleTriggered {
            stack: s,
            spell: fire.spell,
            hex,
        }));
        magic::burn(state, ruleset, &fire, s, sink);
        // Only this stack takes damage: the battle was not over before this burn.
        check_end(state, sink);
    }
    state.at(s).is_alive()
}

/// Moves stack `mover` from `from` to another free position `to` (BAT-MOV-001 to 003), reported
/// by `Moved` events. Every hex of its footprint it enters on the way, each once (not those it
/// starts on), may burn it (BAT-MAG-028): a walker enters the hexes of each position of its path
/// in turn and stops where it dies; a flyer (and a teleporting creature) enters only those it
/// lands on. Returns whether the stack still lives.
#[inline]
pub(crate) fn travel<S: Sink>(
    state: &mut BattleState,
    ruleset: &Ruleset,
    mover: StackId,
    (from, to): (Hex, Hex),
    sink: &mut S,
) -> bool {
    if state.spell_obstacles.fire().is_empty() {
        moved(state, mover, (from, to), sink);
        return true;
    }
    travel_burning(state, ruleset, mover, (from, to), sink)
}

/// Moves stack `mover` from `a` to `b`, reported.
fn moved<S: Sink>(state: &mut BattleState, mover: StackId, (a, b): (Hex, Hex), sink: &mut S) {
    state.at_mut(mover).hex = b;
    sink.emit(Event::public(EventKind::Moved {
        stack: mover,
        from: a,
        to: b,
    }));
}

/// [`travel`] while a Fire Wall stands.
#[inline(never)]
fn travel_burning<S: Sink>(
    state: &mut BattleState,
    ruleset: &Ruleset,
    mover: StackId,
    (from, to): (Hex, Hex),
    sink: &mut S,
) -> bool {
    let fire = state.spell_obstacles.fire();
    let field = Field::new(ruleset.battle().field);
    let stack = *state.at(mover);
    // rule: BAT-MAG-028
    let mut visited = stack
        .footprint(field)
        .hexes()
        .fold(HexSet::new(), |mut v, h| {
            v.insert(h);
            v
        });
    let (path, len) = if stack.has(StackTraits::FLIES) {
        let mut one = [Hex(0); movement::MAX_PATH];
        one[0] = to;
        (one, 1)
    } else {
        movement::path(state, field, mover, &stack, to)
    };
    let mut start = from;
    for &at in path.iter().take(len) {
        let footprint = field.footprint(at, stack.is_two_hex(), stack.side);
        let mut new = [None; 2];
        for (slot, hex) in new
            .iter_mut()
            .zip(footprint.into_iter().flat_map(|f| f.hexes()))
        {
            if !visited.contains(hex) {
                visited.insert(hex);
                *slot = Some(hex).filter(|&h| fire.contains(h));
            }
        }
        let entered = burning(state, new);
        if entered.iter().any(Option::is_some) {
            moved(state, mover, (start, at), sink);
            start = at;
            if !burn_hexes(state, ruleset, mover, entered, sink) {
                return false;
            }
        }
    }
    if start != to {
        moved(state, mover, (start, to), sink);
    }
    true
}

/// After stack `s` acted without moving (it defended, waited, shot, or attacked from where it
/// stands), each hex it stands on that a Fire Wall covers burns it (BAT-MAG-028; the original
/// enters them again), unless the battle is over.
// rule: BAT-MAG-028
#[inline]
pub(crate) fn stand<S: Sink>(state: &mut BattleState, ruleset: &Ruleset, s: StackId, sink: &mut S) {
    if !state.spell_obstacles.fire().is_empty() {
        stand_burning(state, ruleset, s, sink);
    }
}

/// [`stand`] while a Fire Wall stands.
#[inline(never)]
fn stand_burning<S: Sink>(state: &mut BattleState, ruleset: &Ruleset, s: StackId, sink: &mut S) {
    if state.result.is_some() || !state.at(s).is_alive() {
        return;
    }
    let field = Field::new(ruleset.battle().field);
    let footprint = state.at(s).footprint(field);
    let mut on = [None; 2];
    for (slot, hex) in on.iter_mut().zip(footprint.hexes()) {
        *slot = Some(hex);
    }
    let entered = burning(state, on);
    burn_hexes(state, ruleset, s, entered, sink);
}
