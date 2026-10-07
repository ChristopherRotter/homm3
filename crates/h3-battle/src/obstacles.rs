//! Spell obstacles on the battlefield (E04b T5a, T5c1, T5c2; ADR-0014 points 5 and 7): what an
//! obstacle spell places (BAT-MAG-027: Force Field blocks its hexes), Fire Wall's damage to a
//! stack that enters one of its hexes or acts on one (BAT-MAG-028), the rounds they last, Expert
//! Dispel's clearing (BAT-MAG-030), who sees a hidden obstacle (BAT-MAG-031), Quicksand's pits,
//! which stop a walker (BAT-MAG-032), and Land Mine's mines, which go off under an enemy stack
//! (BAT-MAG-033). The original's objects and their triggers: `docs/formats/installation.md`
//! §7.15 to §7.17.
//!
//! The obstacles live in one growable list per battle in the order they were placed (ADR-0014
//! point 5). Most battles have none: the list reserves its room when the first obstacle is
//! placed, and a fork reserves it only if it has obstacles, so a fork of a battle without them
//! still costs one allocation (the effect list's) and a battle with them allocates once. The
//! hexes they block and those that burn are kept as sets beside the list, so the hot paths
//! (movement, a move without spell obstacles) read a set and never the list.

use alloc::vec::Vec;

use h3_content::{
    DRAWS_AVOID_HIDDEN, MAX_FIELD_HEXES, ObstacleKind, ObstacleShape, Ruleset, SpellId, SpellMagic,
};
use h3_contracts::BattleSide;
use h3_kernel::{CanonicalHash, FixedVec, StateHasher};

use crate::combat::check_end;
use crate::event::{Event, EventKind, Sink, Visibility};
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
    /// Rounds left, the current one included; 0: the whole battle (Quicksand).
    pub rounds: u8,
    /// Fire Wall's or a mine's value at the cast: `power × power factor + effect[mastery]`, before
    /// any bonus (BAT-MAG-028, BAT-MAG-033); 0 for other kinds.
    pub value: u64,
    /// Both sides see it (BAT-MAG-031): a kind that is not hidden, a hidden one placed while the
    /// other side had natives at the battle's start (quirk BAT-QUIRK-1), or one a stack of the
    /// side that did not see it ran into (Quicksand, BAT-MAG-032; a mine goes off and is gone,
    /// BAT-MAG-033).
    pub revealed: bool,
}

impl CanonicalHash for SpellObstacle {
    /// Spell, kind, side, hexes, rounds, value, revealed (state hash v27).
    fn hash_into(&self, h: &mut StateHasher) {
        self.spell.hash_into(h);
        h.write_u8(self.kind as u8);
        h.write_u8(side_code(self.side));
        self.hexes.hash_into(h);
        h.write_u8(self.rounds);
        h.write_u64(self.value);
        h.write_bool(self.revealed);
    }
}

/// The battle's spell obstacles, in the order they were placed, with the hexes Force Fields
/// block, those Fire Walls burn and those Quicksand's pits and the mines cover, and the sides'
/// sight of hidden obstacles (BAT-MAG-031).
#[derive(Debug, PartialEq, Eq)]
pub struct SpellObstacles {
    list: Vec<SpellObstacle>,
    walls: HexSet,
    fire: HexSet,
    pits: HexSet,
    mines: HexSet,
    /// Whether each side had a stack on its native terrain at the battle's start.
    natives: [bool; 2],
    /// Quirk BAT-QUIRK-1 fixed: a side sees hidden obstacles while it has a living native stack,
    /// instead of by `natives` when they are placed.
    live_sight: bool,
}

impl SpellObstacles {
    /// None, nothing reserved yet.
    pub(crate) const fn new() -> Self {
        Self {
            list: Vec::new(),
            walls: HexSet::new(),
            fire: HexSet::new(),
            pits: HexSet::new(),
            mines: HexSet::new(),
            natives: [false; 2],
            live_sight: false,
        }
    }

    /// Sets the sides' sight of hidden obstacles at the battle's start (BAT-MAG-031): which
    /// sides had natives, and whether the living natives decide instead (BAT-QUIRK-1 fixed).
    pub(crate) const fn set_sight(&mut self, natives: [bool; 2], live: bool) {
        self.natives = natives;
        self.live_sight = live;
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

    /// The hexes with a pit, seen or not (Quicksand, BAT-MAG-032).
    pub const fn pits(&self) -> HexSet {
        self.pits
    }

    /// The hexes with a mine, seen or not (Land Mine, BAT-MAG-033).
    pub const fn mines(&self) -> HexSet {
        self.mines
    }

    /// Whether a hidden obstacle lies on the battlefield, seen or not (BAT-MAG-031).
    const fn has_hidden(&self) -> bool {
        !self.pits.is_empty() || !self.mines.is_empty()
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
        self.pits = of(ObstacleKind::Pit);
        self.mines = of(ObstacleKind::Mine);
    }

    /// Keeps only the obstacles `keep` picks (tests: a twin without what a side does not see).
    #[cfg(test)]
    pub(crate) fn retain(&mut self, keep: impl FnMut(&SpellObstacle) -> bool) {
        self.list.retain(keep);
        self.refresh();
    }

    /// Reveals the obstacles on `hex` to both sides (tests).
    #[cfg(test)]
    pub(crate) fn reveal_for_tests(&mut self, hex: Hex) {
        self.list
            .iter_mut()
            .filter(|o| o.hexes.contains(hex))
            .for_each(|o| o.revealed = true);
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
            pits: self.pits,
            mines: self.mines,
            natives: self.natives,
            live_sight: self.live_sight,
        }
    }
}

impl CanonicalHash for SpellObstacles {
    /// Count, then the obstacles in order (the hex sets follow from them), then the sight: the
    /// natives of each side and the live sight (state hash v27).
    fn hash_into(&self, h: &mut StateHasher) {
        h.write_len(self.list.len());
        for o in &self.list {
            o.hash_into(h);
        }
        h.write_bool(self.natives[0]);
        h.write_bool(self.natives[1]);
        h.write_bool(self.live_sight);
    }
}

/// Whether `viewer` sees every hidden obstacle because of its natives now: with quirk
/// BAT-QUIRK-1 fixed, while it has a living stack on its native terrain (BAT-MAG-031).
fn natives_see(state: &BattleState, viewer: BattleSide) -> bool {
    state.spell_obstacles.live_sight
        && state
            .stacks
            .iter()
            .any(|s| s.side == viewer && s.is_alive() && s.has(StackTraits::NATIVE))
}

/// Whether `viewer` would see a hidden obstacle the other side places now because of its natives
/// (BAT-MAG-031): it had natives at the battle's start (quirk BAT-QUIRK-1), or, with the quirk
/// fixed, it has a living stack on its native terrain.
fn natives_sight(state: &BattleState, viewer: BattleSide) -> bool {
    let o = &state.spell_obstacles;
    if o.live_sight {
        natives_see(state, viewer)
    } else {
        o.natives[viewer.index()]
    }
}

/// Whether `side`'s hero may place obstacles of kind `kind` now (BAT-MAG-033; the original's
/// spell book, 0x59ECEE, `installation.md` §7.17): no mines while the other side's natives would
/// see them; any other kind.
// rule: BAT-MAG-033
pub(crate) fn may_place(state: &BattleState, side: BattleSide, kind: ObstacleKind) -> bool {
    kind != ObstacleKind::Mine || !natives_sight(state, side.opponent())
}

/// Whether `viewer` sees obstacle `o` (BAT-MAG-031; ADR-0014 point 7): its own, a revealed one,
/// or any while its natives see (quirk BAT-QUIRK-1 fixed).
// rule: BAT-MAG-031
pub(crate) fn sees(state: &BattleState, viewer: BattleSide, o: &SpellObstacle) -> bool {
    o.side == viewer || o.revealed || natives_see(state, viewer)
}

/// Who may see an event about obstacle `o`: both sides if the other side sees it, else its side.
fn visibility(state: &BattleState, o: &SpellObstacle) -> Visibility {
    if sees(state, o.side.opponent(), o) {
        Visibility::Public
    } else {
        Visibility::Side(o.side)
    }
}

/// The spell obstacles `viewer` sees, in the order they were placed (BAT-MAG-031).
pub(crate) fn seen_by(state: &BattleState, viewer: BattleSide) -> Vec<SpellObstacle> {
    state
        .spell_obstacles
        .list
        .iter()
        .filter(|o| sees(state, viewer, o))
        .copied()
        .collect()
}

/// The hexes of the pits `viewer` sees (BAT-MAG-032): where its walkers stop.
pub(crate) fn seen_pits(state: &BattleState, viewer: BattleSide) -> HexSet {
    if state.spell_obstacles.pits.is_empty() {
        return HexSet::new();
    }
    state
        .spell_obstacles
        .list
        .iter()
        .filter(|o| o.kind == ObstacleKind::Pit && sees(state, viewer, o))
        .fold(HexSet::new(), |set, o| set.union(o.hexes))
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

/// The hexes of the spell obstacles `side` sees (BAT-MAG-031).
fn seen_hexes(state: &BattleState, side: BattleSide) -> HexSet {
    let o = &state.spell_obstacles;
    if o.has_hidden() {
        o.list
            .iter()
            .filter(|o| sees(state, side, o))
            .fold(HexSet::new(), |set, o| set.union(o.hexes))
    } else {
        o.hexes()
    }
}

/// Whether `side`'s hero may place an obstacle on every hex of `hexes` (BAT-MAG-027; the
/// original's target test, `installation.md` §7.15): each outside the side columns, without a
/// natural obstacle, an anchor hex (quirk BAT-QUIRK-3, BAT-GEO-003), a spell obstacle the side
/// sees or a living stack. The original also refuses the other side's hidden obstacles, which
/// gives them away (Q-0002, BAT-MAG-031): here the new obstacle goes over them.
pub(crate) fn free(state: &BattleState, field: Field, side: BattleSide, hexes: &[Hex]) -> bool {
    let taken = state
        .natural
        .blocked()
        .union(state.natural.anchors())
        .union(seen_hexes(state, side))
        .union(state.living_hexes(field));
    hexes
        .iter()
        .all(|&h| field.is_standable(h) && !taken.contains(h))
}

/// Up to `count` random hexes for `side`'s random obstacles (Quicksand, Land Mine; BAT-MAG-032;
/// the original's 0x5A0354, `installation.md` §7.16), in the order drawn: each drawn with equal
/// chances among the hexes not yet drawn that are outside the side columns, without a natural
/// obstacle, an anchor hex (quirk BAT-QUIRK-3), a spell obstacle and a stack, living or dead. With
/// quirk BAT-QUIRK-4 every spell obstacle counts, the other side's hidden ones too (which tells
/// the caster where none lies); without it only those `side` sees, and a new one may go over a
/// hidden one of the other side (Q-0002). The original draws from every hex and skips the
/// unsuitable ones, which gives the same chances.
// rule: BAT-MAG-032
pub(crate) fn random_hexes(
    state: &mut BattleState,
    ruleset: &Ruleset,
    side: BattleSide,
    count: u8,
) -> FixedVec<Hex, { h3_content::MAX_OBSTACLE_HEXES }> {
    let field = Field::new(ruleset.battle().field);
    let spells = if ruleset.quirks().is_enabled(DRAWS_AVOID_HIDDEN) {
        state.spell_obstacles.hexes()
    } else {
        seen_hexes(state, side)
    };
    let mut taken = state
        .natural
        .blocked()
        .union(state.natural.anchors())
        .union(spells);
    for s in state.stacks.iter() {
        s.footprint(field).hexes().for_each(|h| taken.insert(h));
    }
    let mut candidates = [Hex(0); MAX_FIELD_HEXES];
    let mut n = 0;
    for h in (0u8..).take(field.hex_count()).map(Hex) {
        if field.is_standable(h) && !taken.contains(h) {
            candidates[n] = h;
            n += 1;
        }
    }
    let mut out = FixedVec::new();
    for _ in 0..count {
        let Some(len) = u32::try_from(n).ok().filter(|&len| len > 0) else {
            break;
        };
        let i = usize::try_from(state.rng.below(len)).unwrap_or(0);
        // At most `MAX_OBSTACLE_HEXES` by the content's check.
        let _ = out.push(candidates[i]);
        n -= 1;
        candidates[i] = candidates[n];
    }
    out
}

/// Places the obstacles of kind `kind` and shape `shape` of `side`'s cast of `id` on `hexes` with
/// the value `value` (BAT-MAG-027): one per hex for a separate shape (Fire Wall, Quicksand, Land
/// Mine), else one over all; each reported to the sides that see it (BAT-MAG-031). A hidden kind
/// is revealed at once if the other side had natives at the battle's start (quirk BAT-QUIRK-1;
/// the original's 0x5A0442, `installation.md` §7.16; mines are never cast then, BAT-MAG-033).
pub(crate) fn place<S: Sink>(
    state: &mut BattleState,
    (id, kind, shape): (SpellId, ObstacleKind, &ObstacleShape),
    side: BattleSide,
    (hexes, value): (&[Hex], u64),
    sink: &mut S,
) {
    let value = if kind.has_value() { value } else { 0 };
    let sight = &state.spell_obstacles;
    // rule: BAT-MAG-031
    let revealed =
        !kind.is_hidden() || (!sight.live_sight && sight.natives[side.opponent().index()]);
    let make = |set: HexSet| SpellObstacle {
        spell: id,
        kind,
        side,
        hexes: set,
        rounds: shape.rounds,
        value,
        revealed,
    };
    let mut add = |o: SpellObstacle| {
        let list = &mut state.spell_obstacles.list;
        if list.capacity() == 0 {
            list.reserve_exact(OBSTACLE_CAPACITY);
        }
        list.push(o);
        sink.emit(Event {
            visibility: visibility(state, &o),
            kind: EventKind::ObstaclePlaced {
                side,
                spell: id,
                hexes: o.hexes,
            },
        });
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

/// Removes the obstacles `gone` picks, each reported to the sides that saw it (BAT-MAG-031).
fn remove<S: Sink>(
    state: &mut BattleState,
    gone: impl Fn(&mut SpellObstacle) -> bool,
    sink: &mut S,
) {
    let before = state.spell_obstacles.list.len();
    let natives = BattleSide::BOTH.map(|side| natives_see(state, side));
    state.spell_obstacles.list.retain_mut(|o| {
        let removed = gone(o);
        if removed {
            let other = o.side.opponent();
            let visibility = if o.revealed || natives[other.index()] {
                Visibility::Public
            } else {
                Visibility::Side(o.side)
            };
            sink.emit(Event {
                visibility,
                kind: EventKind::ObstacleRemoved {
                    spell: o.spell,
                    hexes: o.hexes,
                },
            });
        }
        !removed
    });
    if state.spell_obstacles.list.len() != before {
        state.spell_obstacles.refresh();
    }
}

/// Starts a round for the spell obstacles (BAT-MAG-027): each either goes (its last round has
/// passed) or loses a round, but one for the whole battle (rounds 0, Quicksand) stays; after the
/// timed effects (`installation.md` §7.15).
// rule: BAT-MAG-027
pub(crate) fn count_down<S: Sink>(state: &mut BattleState, sink: &mut S) {
    if state.spell_obstacles.list.is_empty() {
        return;
    }
    remove(
        state,
        |o| {
            if o.rounds == 0 {
                return false;
            }
            o.rounds -= 1;
            o.rounds == 0
        },
        sink,
    );
}

/// Whether `side`'s hero casting a remove effect `magic` at `mastery` finds an obstacle on `hex`
/// (BAT-MAG-034; the original's target test, `installation.md` §7.15): a removable natural
/// obstacle there (its blocked hexes, and its anchor hex under quirk BAT-QUIRK-3), or a spell
/// obstacle there that the side sees and that `magic` removes at `mastery`. The original's
/// Expert cursor also finds the other side's hidden obstacles, which gives them away (Q-0002,
/// the wiki's Remove Obstacle page): here they are no target.
// rule: BAT-MAG-034
pub(crate) fn removable(
    state: &BattleState,
    side: BattleSide,
    (magic, mastery): (&SpellMagic, u8),
    hex: Hex,
) -> bool {
    state.natural.removable_at(hex).is_some()
        || state.spell_obstacles.list.iter().any(|o| {
            o.hexes.contains(hex) && magic.removes_at(o.kind, mastery) && sees(state, side, o)
        })
}

/// Removes what `side`'s validated remove cast `magic` at `mastery` finds on `hex`
/// (BAT-MAG-034; the original's 0x5A1C04 removes the whole object, `installation.md` §7.15): the
/// removable natural obstacle there, reported to both sides; else every spell obstacle there that
/// the side sees and `magic` removes at `mastery` (one Fire Wall hex is an obstacle of its own),
/// each reported to the sides that saw it (BAT-MAG-031).
// rule: BAT-MAG-034
pub(crate) fn remove_at<S: Sink>(
    state: &mut BattleState,
    side: BattleSide,
    (magic, mastery): (&SpellMagic, u8),
    hex: Hex,
    sink: &mut S,
) {
    if let Some(i) = state.natural.removable_at(hex) {
        let gone = state.natural.remove(i);
        sink.emit(Event::public(EventKind::NaturalObstacleRemoved {
            anchor: gone.anchor(),
            hexes: gone.hex_set(),
        }));
        return;
    }
    let natives = natives_see(state, side);
    remove(
        state,
        |o| {
            o.hexes.contains(hex)
                && magic.removes_at(o.kind, mastery)
                && (o.side == side || o.revealed || natives)
        },
        sink,
    );
}

/// Removes every spell obstacle, whoever cast it (Expert Dispel; BAT-MAG-030).
// rule: BAT-MAG-030
pub(crate) fn clear<S: Sink>(state: &mut BattleState, sink: &mut S) {
    remove(state, |_| true, sink);
}

/// The index of the mine on `hex` that goes off under stack `s` (BAT-MAG-033; the original's
/// 0x59FB40, `installation.md` §7.17): one its side does not see (not its own, not revealed, not
/// seen by its natives; the stack's own side even when it is hypnotized) and whose spell hurts it
/// (an immune stack crosses and the mine stays as it was).
fn mine_under(state: &BattleState, ruleset: &Ruleset, s: StackId, hex: Hex) -> Option<usize> {
    let side = state.at(s).side;
    state.spell_obstacles.list.iter().position(|o| {
        o.kind == ObstacleKind::Mine
            && o.hexes.contains(hex)
            && !sees(state, side, o)
            && magic::hurts(state, ruleset, o, s)
    })
}

/// Whether entering `hex` acts on stack `s`: a Fire Wall burns there (BAT-MAG-028) or a mine
/// goes off under it (BAT-MAG-033). A mine that does not go off changes nothing, not even how the
/// move is reported.
fn acts_on(state: &BattleState, ruleset: &Ruleset, s: StackId, hex: Hex) -> bool {
    let o = &state.spell_obstacles;
    o.fire.contains(hex) || (o.mines.contains(hex) && mine_under(state, ruleset, s, hex).is_some())
}

/// Stack `s` enters each of `hexes` in turn, while it lives: on each a Fire Wall burns it
/// (BAT-MAG-028), then a mine may go off under it (BAT-MAG-033), as the original's 0x46A570
/// (`installation.md` §7.15, §7.17); the battle may end. Returns whether it still lives.
fn enter<S: Sink>(
    state: &mut BattleState,
    ruleset: &Ruleset,
    s: StackId,
    hexes: [Option<Hex>; 2],
    sink: &mut S,
) -> bool {
    for hex in hexes.into_iter().flatten() {
        if !state.at(s).is_alive() {
            break;
        }
        if let Some(fire) = state.spell_obstacles.fire_at(hex) {
            sink.emit(Event::public(EventKind::ObstacleTriggered {
                stack: s,
                spell: fire.spell,
                hex,
            }));
            magic::burn(state, ruleset, &fire, s, sink);
            // Only this stack takes damage: the battle was not over before this burn.
            check_end(state, sink);
        }
        if state.at(s).is_alive()
            && let Some(i) = mine_under(state, ruleset, s, hex)
        {
            explode(state, ruleset, (s, hex), i, sink);
        }
    }
    state.at(s).is_alive()
}

/// The mine `i` goes off under stack `s` on `hex` (BAT-MAG-033): it is revealed and reported to
/// both sides, does its spell's damage as a Fire Wall does (BAT-MAG-028) and is gone; the battle
/// may end.
// rule: BAT-MAG-033
fn explode<S: Sink>(
    state: &mut BattleState,
    ruleset: &Ruleset,
    (s, hex): (StackId, Hex),
    i: usize,
    sink: &mut S,
) {
    let mine = state.spell_obstacles.list.remove(i);
    state.spell_obstacles.refresh();
    sink.emit(Event::public(EventKind::ObstacleRevealed {
        side: mine.side,
        spell: mine.spell,
        hexes: mine.hexes,
    }));
    sink.emit(Event::public(EventKind::ObstacleTriggered {
        stack: s,
        spell: mine.spell,
        hex,
    }));
    magic::burn(state, ruleset, &mine, s, sink);
    sink.emit(Event::public(EventKind::ObstacleRemoved {
        spell: mine.spell,
        hexes: mine.hexes,
    }));
    // Only this stack takes damage: the battle was not over before.
    check_end(state, sink);
}

/// Moves stack `mover` from `from` to another free position `to` (BAT-MOV-001 to 003), reported
/// by `Moved` events. Every hex of its footprint it enters on the way, each once (not those it
/// starts on), may burn it (BAT-MAG-028), stop it (a pit, BAT-MAG-032) or blow up under it (a
/// mine, BAT-MAG-033): a walker enters the hexes of each position of its path in turn, stops at
/// the first position where it enters a pit (revealing a pit its controller did not see) and
/// stops where it dies, but walks on after a mine; a flyer (and a teleporting creature) enters
/// only those it lands on and no pit stops it. Returns whether the stack arrived at `to` alive
/// (an attack ordered with the move happens only then).
#[inline]
pub(crate) fn travel<S: Sink>(
    state: &mut BattleState,
    ruleset: &Ruleset,
    mover: StackId,
    (from, to): (Hex, Hex),
    sink: &mut S,
) -> bool {
    let o = &state.spell_obstacles;
    if o.fire().is_empty() && !o.has_hidden() {
        moved(state, mover, (from, to), sink);
        return true;
    }
    travel_through(state, ruleset, mover, (from, to), sink)
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

/// [`travel`] while a Fire Wall, a pit or a mine lies on the battlefield.
#[inline(never)]
fn travel_through<S: Sink>(
    state: &mut BattleState,
    ruleset: &Ruleset,
    mover: StackId,
    (from, to): (Hex, Hex),
    sink: &mut S,
) -> bool {
    let field = Field::new(ruleset.battle().field);
    let stack = *state.at(mover);
    let walks = !stack.has(StackTraits::FLIES);
    let pits = if walks {
        state.spell_obstacles.pits()
    } else {
        HexSet::new()
    };
    // rule: BAT-MAG-028, BAT-MAG-033
    let mut visited = stack
        .footprint(field)
        .hexes()
        .fold(HexSet::new(), |mut v, h| {
            v.insert(h);
            v
        });
    let (path, len) = if walks {
        movement::path(state, field, mover, &stack, to)
    } else {
        let mut one = [Hex(0); movement::MAX_PATH];
        one[0] = to;
        (one, 1)
    };
    let mut start = from;
    for &at in path.iter().take(len) {
        let footprint = field.footprint(at, stack.is_two_hex(), stack.side);
        let mut new = [None; 2];
        let mut in_pit = [None; 2];
        for ((slot, pit), hex) in new
            .iter_mut()
            .zip(in_pit.iter_mut())
            .zip(footprint.into_iter().flat_map(|f| f.hexes()))
        {
            if !visited.contains(hex) {
                visited.insert(hex);
                *slot = Some(hex).filter(|&h| acts_on(state, ruleset, mover, h));
                *pit = Some(hex).filter(|&h| pits.contains(h));
            }
        }
        let stops = in_pit.iter().any(Option::is_some);
        if stops || new.iter().any(Option::is_some) {
            moved(state, mover, (start, at), sink);
            start = at;
            if stops {
                // rule: BAT-MAG-032
                for hex in in_pit.into_iter().flatten() {
                    stuck(state, mover, hex, sink);
                }
                enter(state, ruleset, mover, new, sink);
                return false;
            }
            if !enter(state, ruleset, mover, new, sink) {
                return false;
            }
        }
    }
    if start != to {
        moved(state, mover, (start, to), sink);
    }
    true
}

/// Walker `mover` entered a pit on `hex` and stops there (BAT-MAG-032): the pit is revealed if
/// its controller did not see it (the original's walker 0x441FA0, `installation.md` §7.16), then
/// reported (`ObstacleRevealed`, public; `ObstacleTriggered`, to the sides that see the pit).
fn stuck<S: Sink>(state: &mut BattleState, mover: StackId, hex: Hex, sink: &mut S) {
    let viewer = state.at(mover).controller();
    for i in 0..state.spell_obstacles.list.len() {
        let pit = state.spell_obstacles.list[i];
        if pit.kind != ObstacleKind::Pit || !pit.hexes.contains(hex) {
            continue;
        }
        if !sees(state, viewer, &pit) {
            state.spell_obstacles.list[i].revealed = true;
            sink.emit(Event::public(EventKind::ObstacleRevealed {
                side: pit.side,
                spell: pit.spell,
                hexes: pit.hexes,
            }));
        }
        let pit = state.spell_obstacles.list[i];
        sink.emit(Event {
            visibility: visibility(state, &pit),
            kind: EventKind::ObstacleTriggered {
                stack: mover,
                spell: pit.spell,
                hex,
            },
        });
    }
}

/// After stack `s` acted without moving (it defended, waited, shot, or attacked from where it
/// stands), it enters the hexes it stands on again (the original's 0x46A570): a Fire Wall there
/// burns it (BAT-MAG-028) and a mine there that it did not set off yet goes off (it was placed
/// there by Teleport, or it was immune when it came; BAT-MAG-033), unless the battle is over.
// rule: BAT-MAG-028, BAT-MAG-033
#[inline]
pub(crate) fn stand<S: Sink>(state: &mut BattleState, ruleset: &Ruleset, s: StackId, sink: &mut S) {
    let o = &state.spell_obstacles;
    if !o.fire().is_empty() || !o.mines().is_empty() {
        stand_on(state, ruleset, s, sink);
    }
}

/// [`stand`] while a Fire Wall or a mine lies on the battlefield.
#[inline(never)]
fn stand_on<S: Sink>(state: &mut BattleState, ruleset: &Ruleset, s: StackId, sink: &mut S) {
    if state.result.is_some() || !state.at(s).is_alive() {
        return;
    }
    let field = Field::new(ruleset.battle().field);
    let footprint = state.at(s).footprint(field);
    let mut on = [None; 2];
    for (slot, hex) in on.iter_mut().zip(footprint.hexes()) {
        *slot = Some(hex);
    }
    enter(state, ruleset, s, on, sink);
}
