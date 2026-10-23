//! The original's battle pathfinder (0x4B3400; `docs/formats/installation.md` §7.40, §7.41):
//! the walks' paths (BAT-MOV-003) and the berserk turn's candidates and walk (BAT-MAG-025).
//!
//! A uniform-cost search from the stack's position. The queue takes the cheapest entry and,
//! among equal costs, the one queued first; a taken position looks at its neighbours from top
//! right clockwise (top right, right, bottom right, bottom left, left, top left). A step costs 1,
//! or, when it enters a stop (a hex the stack covers after the step and not before it: a pit its
//! controller sees, a town's moat; BAT-MAG-032, BAT-SIE-004), the rest of the turn: up to the
//! next multiple of the stack's speed. A position keeps the first step that reached it at its
//! least cost (only a strictly cheaper step replaces it), and a path is traced back along those
//! steps. Where the stack does not fit, or the computer keeps it out of a stop (BAT-SIE-022), the
//! stack on that neighbour, and for a two-hex walker the stack on the hex beside it, are marked
//! attackable at the step's cost, a whole move more from a stop; a flyer (or a teleporter) passes
//! over such a neighbour while it has passed over fewer than its speed less one in a row. Draws:
//! none.
//!
//! A search may leave the stacks of one side unmarked (the walker's map of the original's battle
//! AI asks with its own side, 0x4217BE; the berserk turn with none, side −1).
//!
//! A search may count with a turn's speed other than the stack's (the walker's hex scores ask
//! each enemy's reach with its speed + 1 as range and speed, 0x420260): a stop then costs up to
//! that speed's next multiple and a mark from a stop that speed more; with the range within the
//! speed nothing is marked from a stop (0x4B378D to 0x4B37D1). A flyer passes over hexes by its
//! own speed (0x4B39E7).
//!
//! In the tactics phase the computer's searches take the original's tactics flag (0x4B34DE; E23
//! T6): the range and the turn's speed are both [`TACTICS_RANGE`], whatever the stack's speed or
//! Bind, so a stop's step costs the rest of that range and nothing is reached or marked from it.
//! Such a stop is reached but not queued (its cost lies beyond the queue's), and a walk's search
//! to it ends on it after the others, as the original takes it last.
//!
//! Not modelled, as no caller of the engine meets it: the original's cap of 500 queued entries (a
//! position is queued at most twice, below); the tactics phase's range in the engine's own
//! searches (no walk is searched there: a placement is one move); the computer's stop avoidance in
//! a command's own walk (0x4B3420: for a stack the computer plays at a town; the engine's walks
//! are commands, whose legality reads no player, and within one move a stop is an end that no
//! path to another position passes, so no such path changes; the battle AI's own searches ask
//! for it, `computer`).

use h3_content_battle::MAX_FIELD_HEXES;

use crate::binds;
use crate::geometry::{Field, Hex, HexSet};
use crate::movement::{MAX_PATH, fits};
use crate::obstacles;
use crate::siege;
use crate::state::{BattleState, Stack, StackId, StackTraits};
use h3_contracts::BattleSide;

/// The range of the computer's searches over the whole field (the berserk turn's 127: 0x445582,
/// 0x42232C).
pub(crate) const AI_RANGE: u16 = 127;

/// The range and the turn's speed of the computer's searches in the tactics phase (0x4B34DE).
pub(crate) const TACTICS_RANGE: u16 = 1000;

/// Positions: every hex index.
const HEXES: usize = MAX_FIELD_HEXES + 1;

/// The cost buckets: every cost up to the largest range (a speed is a `u8`).
const BUCKETS: usize = 256;

/// The queue's entries. A step's cost is 1, or a stop's rest of the turn, which never falls as
/// the cost it starts from grows: so a position is queued first at its least cost unless that
/// first step entered a stop and a later one does not (a two-hex walker that already covers the
/// stop), and then once more. At most twice per position, the start once.
const ENTRIES: usize = 2 * HEXES;

/// No entry.
const NONE: u16 = u16::MAX;

/// The cost of a cell neither reached nor marked (above every range).
const UNTOUCHED: u16 = u16::MAX;

/// The neighbours' order (0x523D90's directions 0 to 5: top right, right, bottom right, bottom
/// left, left, top left), as indices into [`Field::neighbors`] ([`crate::geometry::Direction::ALL`]: right first).
const ORDER: [usize; 6] = [5, 0, 1, 2, 3, 4];

/// What a search ends on.
#[derive(Debug, Clone, Copy)]
pub(crate) enum Goal {
    /// Nothing: the whole field within the range (the berserk candidates' map, 0x4B2DA0).
    Map,
    /// A position the stack reaches without passing over it (a walk's path, 0x5239D0).
    Reach(Hex),
    /// A stack marked from a step onto one of these hexes, or for a two-hex walker onto the hex
    /// beside the step's (the berserk turn's walk, 0x42232C, with the target in +0x10; the
    /// original's test is the target's position hex alone, BAT-QUIRK-24).
    Attack(HexSet),
}

/// A search's question.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Query {
    /// Where it ends.
    pub(crate) goal: Goal,
    /// The most a position may cost.
    pub(crate) range: u16,
    /// Whether the computer keeps the stack out of its stops (BAT-SIE-022): a step that enters
    /// one is treated as blocked.
    pub(crate) avoid: bool,
    /// The side whose stacks are not marked (0x4B3801: a stack's own side, +0xF4), or none.
    pub(crate) spare: Option<BattleSide>,
    /// The turn's speed the search counts with (the original's last argument), or the stack's
    /// own (−1): a stop costs up to its next multiple, a mark from a stop one more of it, and a
    /// range within it marks nothing from a stop. A flyer passes over hexes by its own speed.
    pub(crate) speed: Option<u16>,
    /// The original's tactics flag (0x4B34DE): the range and the turn's speed are both
    /// [`TACTICS_RANGE`], for a bound stack or one without speed too.
    pub(crate) tactics: bool,
}

/// The queue: a bucket per cost, first in first out within one.
struct Queue {
    first: [u16; BUCKETS],
    last: [u16; BUCKETS],
    hex: [Hex; ENTRIES],
    flown: [u8; ENTRIES],
    next: [u16; ENTRIES],
    len: usize,
    cost: usize,
}

impl Queue {
    fn new() -> Self {
        Self {
            first: [NONE; BUCKETS],
            last: [NONE; BUCKETS],
            hex: [Hex(0); ENTRIES],
            flown: [0; ENTRIES],
            next: [NONE; ENTRIES],
            len: 0,
            cost: 0,
        }
    }

    /// Queues `hex` at `cost` behind the entries of that cost. A step costs at least 1, so a
    /// bucket is never filled again once it was emptied.
    fn push(&mut self, hex: Hex, cost: u16, flown: u8) {
        let i = self.len;
        self.len += 1;
        self.hex[i] = hex;
        self.flown[i] = flown;
        let i = u16::try_from(i).unwrap_or(NONE);
        let c = usize::from(cost);
        if self.first[c] == NONE {
            self.first[c] = i;
        } else {
            self.next[usize::from(self.last[c])] = i;
        }
        self.last[c] = i;
    }

    /// Takes the cheapest entry queued first: its hex, cost and the hexes passed over before it.
    fn pop(&mut self) -> Option<(Hex, u16, u8)> {
        while let Some(&i) = self.first.get(self.cost) {
            if let Some(&hex) = self.hex.get(usize::from(i)) {
                self.first[self.cost] = self.next[usize::from(i)];
                let cost = u16::try_from(self.cost).unwrap_or(u16::MAX);
                return Some((hex, cost, self.flown[usize::from(i)]));
            }
            self.cost += 1;
        }
        None
    }
}

/// A finished search: each position's least cost and the step that reached it, the marks, and
/// where it ended.
#[derive(Debug, Clone)]
pub(crate) struct Search {
    start: Hex,
    /// A reached position's cost, or a marked hex's mark (the original's cells share it).
    cost: [u16; HEXES],
    from: [Hex; HEXES],
    flown: [u8; HEXES],
    reached: HexSet,
    marked: HexSet,
    stops: HexSet,
    end: Option<Hex>,
}

impl Search {
    /// Searches from the position of `mover` (stack `id`) as the original's 0x4B3400 does with
    /// `query`; a bound stack or one without speed has a range of 0.
    pub(crate) fn run(
        state: &BattleState,
        field: Field,
        (id, mover): (StackId, &Stack),
        query: Query,
    ) -> Self {
        let own_speed = u16::from(mover.speed_now());
        let (range, speed) = if query.tactics {
            (TACTICS_RANGE, TACTICS_RANGE)
        } else {
            let speed = query.speed.unwrap_or(own_speed);
            let range = if speed == 0 || binds::is_bound(state, id) {
                0
            } else {
                query.range
            };
            (range, speed)
        };
        let blocked = state.blocked_for(field, id);
        let flies = mover.has(StackTraits::FLIES);
        let beside = |hex: Hex| {
            mover
                .is_two_hex()
                .then(|| field.neighbor(hex, Field::front_direction(mover.side)))
                .flatten()
        };
        // rule: BAT-MAG-032, BAT-SIE-004
        // [`stops`]'s code, kept inline: a call costs the computer's searches 5 % (E23 T5b2a).
        let stops = if flies {
            HexSet::new()
        } else {
            let side = mover.controller();
            let own = mover
                .footprint(field)
                .hexes()
                .fold(HexSet::new(), |mut own, h| {
                    own.insert(h);
                    own
                });
            obstacles::seen_pits(state, side)
                .union(siege::stops(state, side))
                .without(own)
        };
        // A walk's search reads no marks (its target is a free position): none are made.
        let (ends, occupant) = match query.goal {
            Goal::Attack(ends) => (ends, Some(occupants(state, field))),
            Goal::Map => (HexSet::new(), Some(occupants(state, field))),
            Goal::Reach(_) => (HexSet::new(), None),
        };
        let mut search = Self {
            start: mover.hex,
            cost: [UNTOUCHED; HEXES],
            from: [mover.hex; HEXES],
            flown: [0; HEXES],
            reached: HexSet::new(),
            marked: HexSet::new(),
            stops,
            end: None,
        };
        search.reached.insert(mover.hex);
        search.cost[mover.hex.index()] = 0;
        let mut queue = Queue::new();
        queue.push(mover.hex, 0, 0);
        'search: while let Some((at, c, flown)) = queue.pop() {
            if matches!(query.goal, Goal::Reach(to) if to == at && flown == 0) {
                search.end = Some(at);
                break;
            }
            let around = field.neighbors(at);
            // The hex beside a position is its neighbour towards the front (`Direction::ALL`'s
            // order is `neighbors`').
            let at_beside =
                around[Field::front_direction(mover.side) as usize].filter(|_| mover.is_two_hex());
            for i in ORDER {
                let Some(n) = around[i] else {
                    continue;
                };
                // A walk's search (no marks: a cell's cost is its least cost, or none) passes
                // over a position it reached at no more than a step from here: no step improves
                // it (the original tests it and pushes none).
                if occupant.is_none() && search.cost[n.index()] <= c + 1 {
                    continue;
                }
                let free = fits(field, mover, n, blocked);
                // A hex the stack covers before the step is entered by none (0x4B3701–0x4B3725).
                let enters = free
                    && !stops.is_empty()
                    && ((stops.contains(n) && Some(n) != at_beside)
                        || beside(n).is_some_and(|b| b != at && stops.contains(b)));
                if free && !(query.avoid && enters) {
                    // Without speed (range 0) a step costs 1: the original divides only by a
                    // speed above 0 (0x4B373A).
                    let turn = speed.max(1);
                    let step = if enters { turn - c % turn } else { 1 };
                    search.push(&mut queue, (n, at), c + step, 0, range);
                    continue;
                }
                // The attack marks (0x4B3772): from a stop a whole move more, and none from a
                // stop (or a two-hex walker's hex beside it on one) when the range is within the
                // speed (0x4B378D to 0x4B37D1).
                let from_stop = stops.contains(at);
                let a = c + if from_stop { speed } else { 0 };
                let refused =
                    range <= speed && (from_stop || at_beside.is_some_and(|b| stops.contains(b)));
                if let Some(occupant) = occupant.as_ref().filter(|_| a <= range && !refused) {
                    for hit in [Some(n), beside(n)].into_iter().flatten() {
                        if let Some((t, side)) = occupant[hit.index()] {
                            if Some(side) == query.spare {
                                continue;
                            }
                            for h in t.hexes() {
                                search.mark(h, a);
                            }
                            if ends.contains(hit) {
                                search.end = Some(at);
                                break 'search;
                            }
                        }
                    }
                }
                // A flyer has no stops: its step costs 1.
                // rule: BAT-MOV-002
                if flies && u16::from(flown) + 1 < own_speed {
                    search.push(&mut queue, (n, at), c + 1, flown + 1, range);
                }
            }
        }
        // A stop in the tactics phase was reached but not queued: a walk to it ends on it last.
        if let Goal::Reach(to) = query.goal
            && search.end.is_none()
            && search.reached.contains(to)
            && usize::from(search.cost[to.index()]) >= BUCKETS
        {
            search.end = Some(to);
        }
        search
    }

    /// Reaches `n` from `at` at `cost` unless that is above `range` or `n` was reached at no more.
    /// A cost beyond the queue's (a stop in the tactics phase, the whole [`TACTICS_RANGE`]) is
    /// reached but not queued: no step or mark comes from it.
    fn push(&mut self, queue: &mut Queue, (n, at): (Hex, Hex), cost: u16, flown: u8, range: u16) {
        let i = n.index();
        if cost > range || (self.reached.contains(n) && self.cost[i] <= cost) {
            return;
        }
        self.reached.insert(n);
        self.cost[i] = cost;
        self.from[i] = at;
        self.flown[i] = flown;
        if usize::from(cost) < BUCKETS {
            queue.push(n, cost, flown);
        }
    }

    /// Marks `hex` attackable at `a` unless it is marked at no more.
    fn mark(&mut self, hex: Hex, a: u16) {
        if !(self.marked.contains(hex) && self.cost[hex.index()] <= a) {
            self.marked.insert(hex);
            self.cost[hex.index()] = a;
        }
    }

    /// A map set by hand (the computer's destination map, 0x4B2FF0): every hex of `reached` a
    /// position at cost 1 with nothing passed over, every hex of `marked` marked at cost 1, the
    /// walk's stops `stops`, no end; `start` the stack's position, not reached.
    pub(crate) fn placed(start: Hex, (reached, marked): (HexSet, HexSet), stops: HexSet) -> Self {
        let mut cost = [UNTOUCHED; HEXES];
        for h in reached.union(marked).iter() {
            cost[h.index()] = 1;
        }
        Self {
            start,
            cost,
            from: [start; HEXES],
            flown: [0; HEXES],
            reached,
            marked,
            stops,
            end: None,
        }
    }

    /// The mark of `hex`, if it is marked: a stack's position hex says whether it can be attacked
    /// and at what cost (the berserk candidates, 0x445490).
    pub(crate) fn mark_of(&self, hex: Hex) -> Option<u16> {
        self.marked.contains(hex).then(|| self.cost[hex.index()])
    }

    /// The cost the original's cell of `hex` holds: its mark, or its least cost when reached
    /// after it (a flyer passing over it); `u16::MAX` if neither (the original's 0: no caller
    /// reads such a cell).
    pub(crate) fn cell_cost(&self, hex: Hex) -> u16 {
        self.cost[hex.index()]
    }

    /// The hexes passed over in a row before the position `hex` was reached (0 for one the stack
    /// stands on).
    pub(crate) fn flown(&self, hex: Hex) -> u8 {
        self.flown[hex.index()]
    }

    /// Whether the search reached `hex` (the cell's bit 0), a position or a hex flown over.
    pub(crate) const fn reached(&self, hex: Hex) -> bool {
        self.reached.contains(hex)
    }

    /// Whether `hex` is one of the walk's stops (the pathfinder's +0x68, 0x4B3290): a pit its
    /// controller sees or a town's moat, not under the stack at its start; none for a flyer.
    pub(crate) const fn is_stop(&self, hex: Hex) -> bool {
        self.stops.contains(hex)
    }

    /// The stops the search marked (+0x68).
    pub(crate) const fn stops(&self) -> HexSet {
        self.stops
    }

    /// Where the search ended: the position reached ([`Goal::Reach`]) or the one the stack attacks
    /// from ([`Goal::Attack`]).
    pub(crate) const fn end(&self) -> Option<Hex> {
        self.end
    }

    /// The positions from the first step to the end, in order, the start left out; empty without
    /// an end or for one at the start.
    pub(crate) fn path(&self) -> ([Hex; MAX_PATH], usize) {
        let mut out = [Hex(0); MAX_PATH];
        let mut len = 0;
        let mut at = self.end.unwrap_or(self.start);
        while at != self.start && len < MAX_PATH {
            out[len] = at;
            len += 1;
            at = self.from[at.index()];
        }
        out[..len].reverse();
        (out, len)
    }
}

/// The stops of a search for `mover` (the pathfinder's +0x68, which 0x4B3290 marks at every
/// search's start for its stack): none for a flyer; else the pits its controller sees and a town's
/// moat (BAT-MAG-032, BAT-SIE-004), not under the stack at its start. [`Search::run`] keeps its
/// own copy of this code (a call there costs its searches 5 %); `h3-battle-ai`'s test
/// `the_search_stops_are_the_searchs_own` holds the two together.
// rule: BAT-MAG-032, BAT-SIE-004
pub(crate) fn stops(state: &BattleState, field: Field, mover: &Stack) -> HexSet {
    if mover.has(StackTraits::FLIES) {
        return HexSet::new();
    }
    let side = mover.controller();
    let own = mover
        .footprint(field)
        .hexes()
        .fold(HexSet::new(), |mut own, h| {
            own.insert(h);
            own
        });
    obstacles::seen_pits(state, side)
        .union(siege::stops(state, side))
        .without(own)
}

/// The footprint and side of the living stack on each hex of the field (turrets stand off it).
fn occupants(
    state: &BattleState,
    field: Field,
) -> [Option<(crate::geometry::Footprint, BattleSide)>; HEXES] {
    let mut on = [None; HEXES];
    for s in state.stacks.iter() {
        if s.is_alive() && !s.is_turret() {
            let fp = s.footprint(field);
            for h in fp.hexes() {
                on[h.index()] = Some((fp, s.side));
            }
        }
    }
    on
}
