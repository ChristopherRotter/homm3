//! Stacks a battle creates (E04b T6b; the original's code, `docs/formats/installation.md` §7.19):
//! Clone copies a stack of the caster's side for some rounds (BAT-MAG-038), Summon Elemental
//! summons a stack of the spell's creature (BAT-MAG-039). Both take the slot of a killed created
//! stack of the side or a new one, while the side has fewer stacks than the limit (BAT-MAG-037).

use h3_content::{CLONE_ON_ITS_ORIGINAL, CreatureTypeId};
use h3_content_battle::BattleSpell;
use h3_content_battle::{BattleRuleset, MAX_FIELD_HEXES, MAX_RAISED_CREATURES, MAX_SUMMON_SPELLS};
use h3_contracts::BattleSide;
use h3_kernel::{CanonicalHash, FixedVec, StateHasher};

use crate::effects::{self, SpellValues};
use crate::event::{Event, EventKind, Sink};
use crate::geometry::{Direction, Field, Footprint, Hex, HexSet};
use crate::morale;
use crate::state::{
    BattleState, CREATED_SLOT, NO_CORPSE, STACK_CAPACITY, Stack, StackFlags, StackId, StackTraits,
};

/// [`Created::link`] of a stack without one.
const NO_LINK: u8 = u8::MAX;

/// Most stacks of a side's [`Created`] templates: one per summon spell and per raised creature.
pub(crate) const TEMPLATES: usize = MAX_SUMMON_SPELLS + MAX_RAISED_CREATURES;

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
    /// Per side: a stack of each creature its hero's spells summon and of each creature the
    /// corpse raisers raise, as at the battle's start.
    templates: [FixedVec<Stack, TEMPLATES>; 2],
}

impl Created {
    /// Per stack, by stack id, the stack linked to it: a clone's original, a stack's living clone
    /// (observations, BAT-MAG-038).
    pub(crate) fn links(&self) -> [Option<StackId>; STACK_CAPACITY] {
        self.link.map(|l| (l != NO_LINK).then_some(StackId(l)))
    }

    /// Per side, the creature its Summon Elemental casts are bound to (observations, BAT-MAG-039).
    pub(crate) const fn summoned(&self) -> [Option<CreatureTypeId>; 2] {
        self.summoned
    }

    /// Nothing created yet; the sides' summon `templates`.
    pub(crate) const fn new(templates: [FixedVec<Stack, TEMPLATES>; 2]) -> Self {
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
fn has_room(state: &BattleState, ruleset: &BattleRuleset, side: BattleSide) -> bool {
    let stacks = state.stacks.iter().filter(|s| s.side == side).count();
    stacks < usize::from(ruleset.battle().side_stack_limit)
}

/// Whether a stack of `creature` may rise for `side` from a corpse now (BAT-MAG-052): the side
/// has its template ([`Created::templates`]), and a killed created stack of the side leaves its
/// slot ([`reusable`]), else the side has room for a stack. (The original's 0x47A100 finds no
/// record without a slot, and the raise 0x5A7390 crashes; the engine refuses the raise,
/// BAT-QUIRK-12.)
pub(crate) fn has_slot(
    state: &BattleState,
    ruleset: &BattleRuleset,
    (side, creature): (BattleSide, CreatureTypeId),
) -> bool {
    state.created.templates[side.index()]
        .iter()
        .any(|t| t.creature == creature)
        && (has_room(state, ruleset, side) || reusable(state, side).is_some())
}

/// The record a new stack of `side` takes over, if any (BAT-MAG-037; the original's 0x47A100:
/// the first record of the side that is empty or holds a dead summoned stack with no creatures,
/// `installation.md` §7.19): its first created stack that was killed. A created stack that
/// vanished when its rounds ran out keeps its creatures in the original (its death leaves the
/// count, BAT-END-003), so its record stays in use for the rest of the battle: it is passed
/// over (E04b T11b review).
// rule: BAT-MAG-037
fn reusable(state: &BattleState, side: BattleSide) -> Option<StackId> {
    (0u8..)
        .zip(state.stacks.iter())
        .zip(state.created.vanished)
        .find(|&((_, s), vanished)| {
            s.side == side && s.is_created() && !s.is_alive() && vanished == 0
        })
        .map(|((i, _), _)| StackId(i))
}

/// A stack of `count` creatures of `creature` rises for `side` from the corpse of stack `corpse`
/// (BAT-MAG-052; the original's 0x5A7390, `installation.md` §7.21): the corpse is gone, and the
/// new stack, as `side` would have it at the battle's start, takes a killed created stack's slot
/// or a new one (BAT-MAG-037) and the corpse's army slot, on the corpse's position (BAT-GEO-002).
/// It is an army stack of that slot ([`StackTraits::RAISED`]), no created one. The
/// side has a slot for it ([`has_slot`]), and its template ([`Created::templates`]: a corpse
/// raiser of either army raises the creature).
pub(crate) fn raise_corpse<S: Sink>(
    state: &mut BattleState,
    ruleset: &BattleRuleset,
    (side, corpse): (BattleSide, StackId),
    (creature, count): (CreatureTypeId, u32),
    sink: &mut S,
) {
    let dead = *state.at(corpse);
    let hex = dead.hex;
    state.raised.corpse[corpse.index()] = NO_CORPSE;
    let template = state.created.templates[side.index()]
        .iter()
        .find(|t| t.creature == creature)
        .copied()
        .unwrap_or_default();
    let stack = Stack {
        hex,
        slot: dead.slot,
        count,
        base_count: count,
        traits: template.traits.with(StackTraits::RAISED, true),
        ..template
    };
    let id = add(state, ruleset, stack);
    sink.emit(Event::public(EventKind::RoseFromCorpse {
        stack: id,
        corpse,
        creature,
        count,
        hex,
    }));
}

/// Whether `side`'s cast of the Clone `spell` at `mastery` may copy stack `s`, which the cast may
/// reach (BAT-MAG-038; the original's 0x5A8090, case 0x5A841A): the side has room for a stack, and
/// `s` is no clone, has no clone and its creature's level is at most the spell's
/// effect[mastery].
pub(crate) fn clonable(
    state: &BattleState,
    ruleset: &BattleRuleset,
    side: BattleSide,
    (spell, mastery): (&BattleSpell, u8),
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
    ruleset: &BattleRuleset,
    side: BattleSide,
    spell: &BattleSpell,
) -> bool {
    spell.summons.is_some_and(|c| {
        state.created.summoned[side.index()].is_none_or(|bound| bound == c)
            && has_room(state, ruleset, side)
    })
}

/// Where Clone puts the clone of a stack ([`clone_place`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ClonePlace {
    /// A place for the clone, by its position.
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
    ruleset: &BattleRuleset,
    (s, rounds): (StackId, u16),
    sink: &mut S,
) {
    let field = Field::new(ruleset.battle().field);
    let on_original = ruleset.quirks().is_enabled(CLONE_ON_ITS_ORIGINAL);
    let position = match clone_place(state, field, s, on_original) {
        Some(ClonePlace::At(position)) => position,
        // rule: BAT-MAG-038
        Some(ClonePlace::Original) => return turn(state, (s, rounds), sink),
        None => return,
    };
    let original = *state.at(s);
    let shots = ruleset.creature(original.creature).map_or(0, |c| c.shots);
    // Set up as a new stack, without its original's Poison and Age (BAT-MAG-044).
    let max_hp = crate::vitality::start_hit_points(state, s);
    let stack = Stack {
        hex: position,
        slot: CREATED_SLOT,
        base_count: original.count,
        max_hp,
        top_hp: max_hp,
        shots,
        retaliations_left: original.retaliations,
        // A clone of a stack raised from a corpse (BAT-MAG-052) is not one itself.
        traits: original
            .traits
            .with(StackTraits::SUMMONED, true)
            .with(StackTraits::CLONE, true)
            .without(StackTraits::RAISED),
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

/// Where the clone of stack `s` goes (BAT-MAG-038; the original's 0x5A6C70, §7.19): in rings 1 to
/// 10, from the stack's position (BAT-GEO-002) and, for a two-hex stack, from its other hex, along
/// the six directions (from top right clockwise for the attacker, from top left anticlockwise for
/// the defender), the hexes up to the ring's number away; the first where the clone fits with its
/// position there. Skipped: right and left of the position in ring 1, and from the other hex
/// towards the position in rings 1 and 2. With quirk BAT-QUIRK-7 (`on_original`) the stack's own
/// hexes count as free, as in the original: a place whose position is one of them is
/// [`ClonePlace::Original`]; without it they are taken. A place whose other hex is one of them,
/// where the original would put both stacks on one hex, is skipped either way (BAT-QUIRK-8).
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
    let starts = [fp.position(), fp.front];
    let taken = state.blocked_for(field, if on_original { s } else { StackId(u8::MAX) });
    let own = |h: Hex| fp.hexes().any(|o| o == h);
    let (order, backwards) = match stack.side {
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
        for (from_other, &start) in starts.iter().take(1 + usize::from(two_hex)).enumerate() {
            for d in order {
                let skipped = if from_other == 0 {
                    ring == 1 && matches!(d, Left | Right)
                } else {
                    ring <= 2 && d == backwards
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
                        Some(fp) if !own(fp.front) => return Some(ClonePlace::At(hex)),
                        _ => {}
                    }
                }
            }
        }
    }
    None
}

/// The footprint of a stack of `side` whose position is `at`, if it fits (the original's
/// 0x446C40): its hexes are standable and none is in `taken`.
fn fits(
    field: Field,
    taken: HexSet,
    (at, two_hex): (Hex, bool),
    side: BattleSide,
) -> Option<Footprint> {
    field.footprint(at, two_hex, side).filter(|fp| {
        fp.hexes()
            .all(|h| field.is_standable(h) && !taken.contains(h))
    })
}

/// Summons `count` creatures of `creature` for `side` (BAT-MAG-039; the original's 0x5A7080):
/// the side's summons are bound to the creature from now on; a stack of it, as the side would have
/// at the battle's start, goes to a random row where it fits of the first column with one, from
/// the side's edge inwards (the original's sampler draws rows until one fits); without one (or
/// without creatures) nothing happens. The side's hero's book has a spell that summons the
/// creature, so the side has its template.
pub(crate) fn summon<S: Sink>(
    state: &mut BattleState,
    ruleset: &BattleRuleset,
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
        let mut positions: FixedVec<Hex, { MAX_FIELD_HEXES }> = FixedVec::new();
        for position in (0..rows).filter_map(|y| {
            field
                .hex(x, y)
                .filter(|&h| fits(field, taken, (h, two_hex), side).is_some())
        }) {
            // At most one per hex of the field.
            let _ = positions.push(position);
        }
        if positions.is_empty() || count == 0 {
            continue;
        }
        // At most the field's hexes, within `u32`.
        let len = u32::try_from(positions.len()).unwrap_or(u32::MAX);
        let pick = usize::try_from(state.rng.below(len)).unwrap_or(0);
        let stack = Stack {
            hex: positions.as_slice()[pick],
            count,
            base_count: count,
            traits: template.traits.with(StackTraits::SUMMONED, true),
            ..template
        };
        let id = add(state, ruleset, stack);
        created(state, id, None, sink);
        return;
    }
}

/// Puts the created `stack` into the battle (BAT-MAG-037; the original's 0x47A100): in the slot of
/// its side's first killed created stack ([`reusable`]), else in a new one; morale and luck as
/// its side's stacks have them. Returns its id.
fn add(state: &mut BattleState, ruleset: &BattleRuleset, stack: Stack) -> StackId {
    let id = if let Some(id) = reusable(state, stack.side) {
        // A reused record is set up from scratch (the original's 0x47A100): nothing of the dead
        // stack's effects stays, neither the timed ones the command's end has not removed yet nor
        // the permanent ones a dead stack keeps (BAT-MAG-010).
        effects::forget(state, id);
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
pub(crate) fn died<S: Sink>(
    state: &mut BattleState,
    ruleset: &BattleRuleset,
    s: StackId,
    sink: &mut S,
) {
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
fn vanish<S: Sink>(state: &mut BattleState, ruleset: &BattleRuleset, c: StackId, sink: &mut S) {
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
pub(crate) fn count_down<S: Sink>(state: &mut BattleState, ruleset: &BattleRuleset, sink: &mut S) {
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
