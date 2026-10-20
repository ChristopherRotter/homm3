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
//! Not modelled, as no caller of the engine meets it: the original's cap of 500 queued entries (a
//! position is queued at most twice, below); its refusal to mark from a stop when the range is
//! within one move (0x4B378D: the walks' searches, which read no marks); the side whose stacks
//! are no targets (every caller here asks with none, as the berserk turn's side −1); the tactics
//! phase's range (no walk is searched there: a placement is one move); the computer's stop
//! avoidance in a walk's own search (0x4B3420: for a stack the computer plays at a town; the
//! engine's walks are commands, whose legality reads no player, and within one move a stop is
//! an end that no path to another position passes, so no such path changes).

use h3_content_battle::MAX_FIELD_HEXES;

use crate::binds;
use crate::geometry::{Field, Hex, HexSet};
use crate::movement::{MAX_PATH, fits};
use crate::obstacles;
use crate::siege;
use crate::state::{BattleState, Stack, StackId, StackTraits};

/// The range of the computer's searches over the whole field (the berserk turn's 127: 0x445582,
/// 0x42232C).
pub(crate) const AI_RANGE: u16 = 127;

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
        let speed = u16::from(mover.speed_now());
        let range = if speed == 0 || binds::is_bound(state, id) {
            0
        } else {
            query.range
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
                // The attack marks (0x4B3772): from a stop a whole move more.
                let a = c + if stops.contains(at) { speed } else { 0 };
                if let Some(occupant) = occupant.as_ref().filter(|_| a <= range) {
                    for hit in [Some(n), beside(n)].into_iter().flatten() {
                        if let Some(t) = occupant[hit.index()] {
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
                if flies && u16::from(flown) + 1 < speed {
                    search.push(&mut queue, (n, at), c + 1, flown + 1, range);
                }
            }
        }
        search
    }

    /// Reaches `n` from `at` at `cost` unless that is above `range` or `n` was reached at no more.
    fn push(&mut self, queue: &mut Queue, (n, at): (Hex, Hex), cost: u16, flown: u8, range: u16) {
        let i = n.index();
        if cost > range || (self.reached.contains(n) && self.cost[i] <= cost) {
            return;
        }
        self.reached.insert(n);
        self.cost[i] = cost;
        self.from[i] = at;
        self.flown[i] = flown;
        queue.push(n, cost, flown);
    }

    /// Marks `hex` attackable at `a` unless it is marked at no more.
    fn mark(&mut self, hex: Hex, a: u16) {
        if !(self.marked.contains(hex) && self.cost[hex.index()] <= a) {
            self.marked.insert(hex);
            self.cost[hex.index()] = a;
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

/// The footprint of the living stack on each hex of the field (turrets stand off it).
fn occupants(state: &BattleState, field: Field) -> [Option<crate::geometry::Footprint>; HEXES] {
    let mut on = [None; HEXES];
    for s in state.stacks.iter() {
        if s.is_alive() && !s.is_turret() {
            let fp = s.footprint(field);
            for h in fp.hexes() {
                on[h.index()] = Some(fp);
            }
        }
    }
    on
}
