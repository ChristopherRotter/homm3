//! Stacks a battle creates (E04b T6b; the original's code, `docs/formats/installation.md` §7.19):
//! Clone copies a stack of the caster's side for some rounds (BAT-MAG-038), Summon Elemental
//! summons a stack of the spell's creature (BAT-MAG-039). Both take a dead created stack's slot of
//! the side or a new one, while the side has fewer stacks than the limit (BAT-MAG-037).

use h3_content::{
    CLONE_ON_ITS_ORIGINAL, CreatureTypeId, MAX_FIELD_HEXES, MAX_SUMMON_SPELLS, Ruleset, Spell,
};
use h3_contracts::BattleSide;
use h3_kernel::{CanonicalHash, FixedVec, StateHasher};

use crate::effects::{self, SpellValues};
use crate::event::{Event, EventKind, Sink};
use crate::geometry::{Direction, Field, Hex, HexSet};
use crate::morale;
use crate::state::{
    BattleState, CREATED_SLOT, NO_CORPSE, STACK_CAPACITY, Stack, StackFlags, StackId, StackTraits,
};

/// [`Created::link`] of a stack without one.
const NO_LINK: u8 = u8::MAX;

/// Rings around the original that Clone searches for a place (the original's 0x5A6C70).
const CLONE_RINGS: u32 = 10;

/// What the battle keeps about the stacks it creates (BAT-MAG-037 to BAT-MAG-039).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Created {
    /// Per stack: for a clone its original, for a stack with a living clone that clone,
    /// [`NO_LINK`] otherwise (the original's +0x24 and +0x28).
    link: [u8; STACK_CAPACITY],
    /// Per clone: the rounds left, of which each round's start takes one; it vanishes at the
    /// start that leaves none (the original's +0x2C).
    pub(crate) rounds: [u16; STACK_CAPACITY],
    /// Per stack that vanished when its rounds ran out: the creatures it vanished with, which
    /// were not killed (the original's death keeps its count, 0x444120; BAT-Q13); 0 otherwise.
    pub(crate) vanished: [u32; STACK_CAPACITY],
    /// Per side: the creature its Summon Elemental casts are bound to, once it cast one (the
    /// original's +0x132A8).
    summoned: [Option<CreatureTypeId>; 2],
    /// Per side: a stack of each creature its hero's spells summon, as at the battle's start.
    templates: [FixedVec<Stack, MAX_SUMMON_SPELLS>; 2],
}

impl Created {
    /// Nothing created yet; the sides' summon `templates`.
    pub(crate) const fn new(templates: [FixedVec<Stack, MAX_SUMMON_SPELLS>; 2]) -> Self {
        Self {
            link: [NO_LINK; STACK_CAPACITY],
            rounds: [0; STACK_CAPACITY],
            vanished: [0; STACK_CAPACITY],
            summoned: [None; 2],
            templates,
        }
    }
}

impl CanonicalHash for Created {
    fn hash_into(&self, h: &mut StateHasher) {
        h.write_bytes(&self.link);
        for &r in &self.rounds {
            h.write_u16(r);
        }
        for &v in &self.vanished {
            h.write_u32(v);
        }
        self.summoned.hash_into(h);
        for t in &self.templates {
            t.hash_into(h);
        }
    }
}

/// Whether `side` may create a stack now (BAT-MAG-037; the original's spell book, §7.7): it has
/// fewer stacks than the battle's side stack limit, dead ones included.
fn has_room(state: &BattleState, ruleset: &Ruleset, side: BattleSide) -> bool {
    let stacks = state.stacks.iter().filter(|s| s.side == side).count();
    stacks < usize::from(ruleset.battle().side_stack_limit)
}

/// Whether `side`'s cast of the Clone `spell` at `mastery` may copy stack `s`, which the cast may
/// reach (BAT-MAG-038; the original's 0x5A8090, case 0x5A841A): the side has room for a stack, and
/// `s` is no clone, has no clone and its creature's level is at most the spell's
/// effect[mastery].
pub(crate) fn clonable(
    state: &BattleState,
    ruleset: &Ruleset,
    side: BattleSide,
    (spell, mastery): (&Spell, u8),
    s: StackId,
) -> bool {
    let stack = state.at(s);
    let limit = spell.effect[usize::from(mastery)];
    !stack.has(StackTraits::CLONE)
        && state.created.link[s.index()] == NO_LINK
        && ruleset
            .creature_info(stack.creature)
            .is_some_and(|c| u16::from(c.level) <= limit)
        && has_room(state, ruleset, side)
}

/// Whether `side` may cast the summon `spell` now (BAT-MAG-039; the original's spell book, §7.7):
/// the side has room for a stack and has summoned no other creature in this battle.
pub(crate) fn may_summon(
    state: &BattleState,
    ruleset: &Ruleset,
    side: BattleSide,
    spell: &Spell,
) -> bool {
    spell.summons.is_some_and(|c| {
        state.created.summoned[side.index()].is_none_or(|bound| bound == c)
            && has_room(state, ruleset, side)
    })
}

/// Where Clone puts the clone of a stack ([`clone_place`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ClonePlace {
    /// A place for the clone, by its head.
    At(Hex),
    /// The original's own hexes (quirk BAT-QUIRK-7): the original becomes the clone.
    Original,
}

/// Clones stack `s` for `rounds` rounds (BAT-MAG-038; the original's 0x5A6C70): a fresh stack of
/// its side, creature and statistics with its count, placed by [`clone_place`]; without a place
/// nothing happens. With quirk BAT-QUIRK-7 a place on the original's own hexes makes no clone and
/// turns the original into one ([`turn`]).
pub(crate) fn clone<S: Sink>(
    state: &mut BattleState,
    ruleset: &Ruleset,
    (s, rounds): (StackId, u16),
    sink: &mut S,
) {
    let field = Field::new(ruleset.battle().field);
    let on_original = ruleset.quirks().is_enabled(CLONE_ON_ITS_ORIGINAL);
    let head = match clone_place(state, field, s, on_original) {
        Some(ClonePlace::At(head)) => head,
        // rule: BAT-MAG-038
        Some(ClonePlace::Original) => return turn(state, (s, rounds), sink),
        None => return,
    };
    let original = *state.at(s);
    let shots = ruleset.creature(original.creature).map_or(0, |c| c.shots);
    // Set up as a new stack, without its original's Poison and Age (BAT-MAG-044).
    let max_hp = crate::vitality::start_hit_points(state, s);
    let stack = Stack {
        hex: head,
        slot: CREATED_SLOT,
        base_count: original.count,
        max_hp,
        top_hp: max_hp,
        shots,
        retaliations_left: original.retaliations,
        traits: original
            .traits
            .with(StackTraits::SUMMONED, true)
            .with(StackTraits::CLONE, true),
        flags: StackFlags::default(),
        spells: SpellValues::new(original.speed),
        ..original
    };
    let id = add(state, ruleset, stack);
    state.created.link[id.index()] = s.0;
    state.created.link[s.index()] = id.0;
    state.created.rounds[id.index()] = rounds;
    created(state, id, Some(s), sink);
}

/// Stack `s` becomes its own clone for `rounds` rounds (quirk BAT-QUIRK-7; the original's
/// 0x5A6C70 when the clone's place is the original's hex, where 0x47A100 creates nothing and the
/// cast flags the stack on it): a summoned stack, linked to itself, that vanishes when its rounds
/// run out and is no longer of its side's army (BAT-MAG-038). It is no clone: damage does not
/// kill all its creatures.
fn turn<S: Sink>(state: &mut BattleState, (s, rounds): (StackId, u16), sink: &mut S) {
    let stack = state.at_mut(s);
    stack.traits = stack.traits.with(StackTraits::SUMMONED, true);
    state.created.link[s.index()] = s.0;
    state.created.rounds[s.index()] = rounds;
    sink.emit(Event::public(EventKind::TurnedSummoned {
        stack: s,
        rounds,
    }));
}

/// Where the clone of stack `s` goes (BAT-MAG-038; the original's 0x5A6C70, §7.19): in rings 1
/// to 10, from the stack's position (its tail, or its only hex) and, for a two-hex stack, from its
/// head, along the six directions (from top right clockwise for the attacker, from top left
/// anticlockwise for the defender), the hexes up to the ring's number away; the first where the
/// clone fits. Skipped: right and left of the position in ring 1, and from the head towards the
/// tail in rings 1 and 2. With quirk BAT-QUIRK-7 (`on_original`) the stack's own hexes count as
/// free, as in the original: a place whose position is one of them is [`ClonePlace::Original`];
/// without it they are taken. A place whose other hex is one of them, where the original would
/// put both stacks on one hex, is skipped either way (BAT-QUIRK-8).
fn clone_place(
    state: &BattleState,
    field: Field,
    s: StackId,
    on_original: bool,
) -> Option<ClonePlace> {
    use Direction::{BottomLeft, BottomRight, Left, Right, TopLeft, TopRight};
    let stack = state.at(s);
    let two_hex = stack.has(StackTraits::TWO_HEX);
    let fp = stack.footprint(field);
    let starts = [fp.tail.unwrap_or(fp.head), fp.head];
    let taken = state.blocked_for(field, if on_original { s } else { StackId(u8::MAX) });
    let own = |h: Hex| fp.hexes().any(|o| o == h);
    let (order, toward_tail) = match stack.side {
        BattleSide::Attacker => (
            [TopRight, Right, BottomRight, BottomLeft, Left, TopLeft],
            Left,
        ),
        BattleSide::Defender => (
            [TopLeft, Left, BottomLeft, BottomRight, Right, TopRight],
            Right,
        ),
    };
    for ring in 1..=CLONE_RINGS {
        for (from_head, &start) in starts.iter().take(1 + usize::from(two_hex)).enumerate() {
            for d in order {
                let skipped = if from_head == 0 {
                    ring == 1 && matches!(d, Left | Right)
                } else {
                    ring <= 2 && d == toward_tail
                };
                if skipped {
                    continue;
                }
                let mut hex = start;
                for _ in 0..ring {
                    let Some(next) = field.neighbor(hex, d) else {
                        break;
                    };
                    hex = next;
                    match fits(field, taken, (hex, two_hex), stack.side) {
                        // rule: BAT-MAG-038
                        Some(_) if own(hex) => return Some(ClonePlace::Original),
                        Some(head) if !own(head) => return Some(ClonePlace::At(head)),
                        _ => {}
                    }
                }
            }
        }
    }
    None
}

/// The head of a stack of `side` whose position (its tail, or its only hex) is `at`, if it fits
/// (the original's 0x446C40): its hexes are standable and none is in `taken`.
fn fits(field: Field, taken: HexSet, (at, two_hex): (Hex, bool), side: BattleSide) -> Option<Hex> {
    let head = if two_hex {
        field.neighbor(at, Field::head_direction(side))?
    } else {
        at
    };
    field
        .footprint(head, two_hex, side)
        .filter(|fp| {
            fp.hexes()
                .all(|h| field.is_standable(h) && !taken.contains(h))
        })
        .map(|fp| fp.head)
}

/// Summons `count` creatures of `creature` for `side` (BAT-MAG-039; the original's 0x5A7080):
/// the side's summons are bound to the creature from now on; a stack of it, as the side would have
/// at the battle's start, goes to a random row where it fits of the first column with one, from
/// the side's edge inwards (the original's sampler draws rows until one fits); without one (or
/// without creatures) nothing happens. The side's hero's book has a spell that summons the
/// creature, so the side has its template.
pub(crate) fn summon<S: Sink>(
    state: &mut BattleState,
    ruleset: &Ruleset,
    (side, creature): (BattleSide, CreatureTypeId),
    count: u32,
    sink: &mut S,
) {
    state.created.summoned[side.index()] = Some(creature);
    let templates = &state.created.templates[side.index()];
    let template = templates
        .iter()
        .find(|t| t.creature == creature)
        .copied()
        .unwrap_or_default();
    let field = Field::new(ruleset.battle().field);
    let taken = state.blocked_for(field, StackId(u8::MAX));
    let two_hex = template.has(StackTraits::TWO_HEX);
    let (first, last) = (
        field.first_standable_column(),
        field.last_standable_column(),
    );
    let rows = i32::from(ruleset.battle().field.height);
    for i in 0..=last - first {
        let x = match side {
            BattleSide::Attacker => first + i,
            BattleSide::Defender => last - i,
        };
        let mut heads: FixedVec<Hex, { MAX_FIELD_HEXES }> = FixedVec::new();
        for head in (0..rows).filter_map(|y| {
            field
                .hex(x, y)
                .and_then(|h| fits(field, taken, (h, two_hex), side))
        }) {
            // At most one per hex of the field.
            let _ = heads.push(head);
        }
        if heads.is_empty() || count == 0 {
            continue;
        }
        // At most the field's hexes, within `u32`.
        let len = u32::try_from(heads.len()).unwrap_or(u32::MAX);
        let pick = usize::try_from(state.rng.below(len)).unwrap_or(0);
        let stack = Stack {
            hex: heads.as_slice()[pick],
            count,
            base_count: count,
            ..template
        };
        let id = add(state, ruleset, stack);
        created(state, id, None, sink);
        return;
    }
}

/// Puts the created `stack` into the battle (BAT-MAG-037; the original's 0x47A100): in the slot of
/// its side's first dead created stack, else in a new one; morale and luck as its side's stacks
/// have them. Returns its id.
fn add(state: &mut BattleState, ruleset: &Ruleset, stack: Stack) -> StackId {
    // A dead stack's effects are gone after every command; a reused slot starts without them.
    effects::remove_dead(state, ruleset);
    let reused = (0u8..)
        .zip(state.stacks.iter())
        .find(|(_, s)| s.side == stack.side && s.is_created() && !s.is_alive())
        .map(|(i, _)| StackId(i));
    let id = if let Some(id) = reused {
        *state.at_mut(id) = stack;
        id
    } else {
        // The side has fewer stacks than the limit, at most its slots (BAT-MAG-037), so the
        // battle's capacity has room.
        let id = StackId(u8::try_from(state.stacks.len()).unwrap_or(u8::MAX));
        let _ = state.stacks.push(stack);
        id
    };
    state.raised.temporary[id.index()] = 0;
    state.raised.corpse[id.index()] = 0;
    state.created.link[id.index()] = NO_LINK;
    state.created.rounds[id.index()] = 0;
    state.created.vanished[id.index()] = 0;
    crate::vitality::reset(state, id);
    // rule: BAT-MAG-048
    state.casts.left[id.index()] = crate::casters::per_battle(ruleset, state.at(id).creature);
    morale::materialize(state, ruleset);
    id
}

/// Reports the created stack `id`.
fn created<S: Sink>(state: &BattleState, id: StackId, original: Option<StackId>, sink: &mut S) {
    let stack = state.at(id);
    sink.emit(Event::public(EventKind::StackCreated {
        stack: id,
        creature: stack.creature,
        count: stack.count,
        hex: stack.hex,
        original,
    }));
}

/// After stack `s` took damage or lost creatures, if it died (the original's 0x444120): the stacks
/// it binds are released (BAT-MAG-045); a dead clone leaves its original free to be cloned again,
/// a dead original takes its clone with it (BAT-MAG-038).
pub(crate) fn died<S: Sink>(state: &mut BattleState, ruleset: &Ruleset, s: StackId, sink: &mut S) {
    if state.at(s).is_alive() {
        return;
    }
    // rule: BAT-MAG-045
    crate::binds::died(state, ruleset, s, sink);
    let link = state.created.link[s.index()];
    if link == NO_LINK {
        return;
    }
    state.created.link[s.index()] = NO_LINK;
    state.created.link[usize::from(link)] = NO_LINK;
    if state.at(s).has(StackTraits::CLONE) || link == s.0 {
        state.created.rounds[s.index()] = 0;
    } else {
        vanish(state, ruleset, StackId(link), sink);
    }
}

/// Clone `c` vanishes (BAT-MAG-038): all its creatures are gone, without a corpse; the stacks it
/// binds are released (BAT-MAG-045).
fn vanish<S: Sink>(state: &mut BattleState, ruleset: &Ruleset, c: StackId, sink: &mut S) {
    let clone = state.at_mut(c);
    let creatures = clone.count;
    clone.count = 0;
    clone.top_hp = 0;
    state.raised.corpse[c.index()] = NO_CORPSE;
    state.created.rounds[c.index()] = 0;
    sink.emit(Event::public(EventKind::Vanished {
        stack: c,
        creatures,
    }));
    crate::binds::died(state, ruleset, c, sink);
}

/// At a round's start, after the timed effects (BAT-MAG-038; the original's 0x447120): every
/// living clone (one with rounds left) loses a round, and one left with none vanishes, freeing its
/// original; it keeps the creatures it vanished with as not killed (BAT-Q13).
pub(crate) fn count_down<S: Sink>(state: &mut BattleState, ruleset: &Ruleset, sink: &mut S) {
    for i in 0..state.stacks.len() {
        let rounds = &mut state.created.rounds[i];
        if *rounds == 0 {
            continue;
        }
        *rounds -= 1;
        if *rounds == 0 {
            let c = StackId(u8::try_from(i).unwrap_or(u8::MAX));
            state.created.vanished[i] = state.at(c).count;
            vanish(state, ruleset, c, sink);
            died(state, ruleset, c, sink);
        }
    }
}
