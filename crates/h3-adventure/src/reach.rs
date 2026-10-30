//! The reachable-tiles query (E08 T7b; a first-class core query, ADR-0006 guardrail; the env's
//! action is a destination among its tiles, ADR-0017 point 8): every tile a hero can reach today
//! with the points it would have left there, and the path chosen to it.
//!
//! The search is Dijkstra's by points left, the most first: each step is priced at the points the
//! search has for its source tile, the most it found (ties keep the path found first, in the
//! order of tiles and of [`DIRECTIONS`]). It keeps one path per tile, as the original's pathfinder
//! does (0x56B440: one node per tile, each step priced at the node's points left, a node replaced
//! only by a strictly better one) and VCMI's. That is exact only while more points on a tile are
//! never worse, which the diagonal exception (ADV-MOV-006) breaks: a diagonal from swamp costs 175
//! with 175 to 247 points but 247 with more, so 257 points on the swamp leave 10 after it and 245
//! leave 70. Then a path that reaches a tile on the way with fewer points can end with more, or
//! reach a tile the search does not (rare, within the last ~250 points of a day; measured in
//! `E08-log.md`). Without the exception the search matches an exhaustive one exactly. Every path
//! it reports is one `MoveHero` accepts, and it reports every tile one step away
//! (property-tested); a client may send any other payable path.
//!
//! A tile an object is visited from (ADV-OBJ-003) is a destination, never a tile on the way: the
//! search reaches it and goes no further from it, with the points the hero keeps after the visit
//! (a pickup's on the tile before, E08 T13b; a Cartographer's on its tile, T14a). Neither such a
//! tile of an object not visited from above is reached from the three tiles above it, nor are
//! those tiles reached from it when the hero stands there (ADV-OBJ-004, T14a). A border gate whose
//! key the hero's owner holds is not visited: its tile is one like any other, on the way or at
//! the end (ADV-VIS-005, T15b); a gate is visited from above (the ruleset part requires it), so
//! the search goes on from it in every direction. Another hero's tile is a destination too, its
//! visit a battle with another player's (GAM-BAT-004, E08 T19a) or a meeting with an own one
//! (GAM-MET-001, T50), from any direction, whatever object lies under it. A tile a monster guards (ADV-MON-005) is a destination as well:
//! the search does not go on from it, but from the hero's own tile (ADV-MON-006, E08 T20a2b; the
//! original's pathfinder, 0x56BC05).
//!
//! A flying or water-walking hero (E08 T41; ADV-MOV-009, ADV-MOV-010) is searched over two kinds
//! of node per tile, as the original's pathfinder keeps two (flags bit 10, 0x4B1C4B): a stop node,
//! where a move may end and a visit be made from, and a pass node, over which a flight or a water
//! walk only goes on (`AdventureState::passage`'s table). Each node is replaced only by a strictly
//! better one of its kind, and records the kind of the node it was entered from (the original's
//! bit 11, 0x4B1B15), so that a path follows each node's predecessor of its kind. A flying hero's
//! search goes on over guarded tiles (0x56BBCE). Its destinations are the stop nodes. A walking
//! hero's search keeps its one node per tile and its own loop (`bench-check`).
//!
//! Boats (E08 T73b; ADV-SEA-002 to -005, increment 8's design note §9). A hero in a boat is
//! searched over one node per tile, every water tile a stop, by `AdventureState::passage`'s rows of
//! a boat and a boat's prices (`Pricing::sail`, the drop of `Pricing::left_by`): a landing onto a
//! free coastal tile, a visit from the boat's tile (an object's, another hero's) and a water tile a
//! monster guards are destinations it goes no further from, a landing with the points its
//! conversion leaves (`converted_points` to the land's maximum, as the move's). A walking hero's
//! water neighbour is tested out of its loop, only where another hero stands in a boat or an empty
//! boat lies: the hero is visited, the boat boarded (with a boat's drop and the conversion to the
//! sea's maximum, as a flyer's and a water walker's boarding), each a destination. Events stay
//! unseen, at sea as on land (decision 131).

use alloc::collections::BinaryHeap;
use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;
use core::cmp::Reverse;

use h3_content::Ruleset;
use h3_contracts::{ArmyStack, HeroId, PlayerId};
use h3_heroes::{HeroDetails, free_boarding};

use crate::movement::{
    DIRECTIONS, Pricing, converted_points, day_points, details_of, hero_movement, sea_day_points,
};
use crate::ruleset::AdventureRuleset;
use crate::setup::Pos;
use crate::state::{
    AdventureState, HeroOnMap, Meets, Mode, Node, StaticMap, StaticTile, enterable_terrain,
};

/// What a search reads besides its window: the state, the rules, the hero's pricing and owner,
/// and whether another hero stands, or an empty boat lies, on each of the window's tiles.
struct Search<'s> {
    state: &'s AdventureState,
    part: &'s AdventureRuleset,
    map: &'s StaticMap,
    pricing: Pricing<'s>,
    hero: HeroId,
    owner: PlayerId,
    /// Whether another hero stands on the slot's tile (on land, or on water in a boat) or an
    /// empty boat lies there (E08 T73b: on water only, so a land tile's flag is a hero's).
    occupied: Vec<bool>,
    /// Whether the map holds a monster, so that a tile may be guarded (E08 T20a2b): the search
    /// asks [`Search::guarded`] only then (`bench-check`).
    guards: bool,
    /// What a boarding's and a landing's conversion reads (E08 T73; ADV-SEA-002, ADV-SEA-003):
    /// the core, the hero's details, its day's maximum and its army; derived only at a boarding
    /// or a landing met (`bench-check`).
    surface: (&'s Ruleset, Option<&'s HeroDetails>, u32, &'s [ArmyStack]),
}

impl Search<'_> {
    /// The points a boarding keeps of `left` on the boat's tile `tile` (E08 T73; ADV-SEA-002):
    /// the boat's drop, then the conversion to the sea's maximum (`converted_points`), as the
    /// move leaves them. Out of the search's hot loop: boats are rare.
    #[cold]
    fn boarded(&self, tile: &StaticTile, left: u32) -> u32 {
        let (core, details, _, _) = self.surface;
        self.converted(
            self.pricing.sail_left_after(tile, left),
            sea_day_points(core, self.part, details),
        )
    }

    /// The points a landing keeps of `left`, the points after its drop (E08 T73b; ADV-SEA-003):
    /// the conversion to the land's maximum (`day_points` for the hero's army), as the move's.
    #[cold]
    fn landed(&self, left: u32) -> u32 {
        let (core, details, _, army) = self.surface;
        self.converted(left, day_points(core, self.part, details, army))
    }

    /// [`converted_points`] of `points` from the hero's day's maximum to `new`, with its free
    /// boarding.
    fn converted(&self, points: u32, new: u32) -> u32 {
        let (core, details, old, _) = self.surface;
        let free = details.is_some_and(|details| free_boarding(details, core));
        converted_points(points, (old, new), free)
    }
}

impl Search<'_> {
    /// Whether a monster guards `pos` (ADV-MON-005), out of the search's hot loop.
    #[cold]
    #[inline(never)]
    fn guarded(&self, pos: Pos) -> bool {
        self.map.near_monster(pos) && self.state.guard_of(self.part, pos).is_some()
    }
}

/// The bit of a node's direction byte that says its predecessor is a pass node (a flying or
/// water-walking hero's search, E08 T41); the low three bits are the direction.
const FROM_PASS: u8 = 8;

/// The tiles a hero can reach today ([`AdventureState::reach`]): a window of the map around the
/// hero, as far as its points can take it, with the points left on each tile reached and the
/// direction of the step the chosen path enters it by.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reach {
    origin: Pos,
    start: usize,
    left: u16,
    top: u16,
    width: u16,
    height: u16,
    points: Vec<Option<u32>>,
    via: Vec<u8>,
    /// A flying or water-walking hero's pass nodes, by slot (E08 T41): the points left on each
    /// and its direction byte; empty for a walking hero.
    passing: Vec<Option<u32>>,
    passing_via: Vec<u8>,
}

impl Reach {
    /// The search from `hero`'s tile with its points, its steps priced by its details in the
    /// game's store (E08 T40).
    fn search(
        state: &AdventureState,
        core: &Ruleset,
        part: &AdventureRuleset,
        heroes: &[Arc<HeroDetails>],
        hero: &HeroOnMap,
    ) -> Self {
        let map = state.map();
        let movement = hero_movement(core, details_of(heroes, hero.id));
        let mode = Mode::of(&movement, hero.boat);
        let pricing = Pricing::new(core, part, &hero.army, &movement);
        // No step costs less than the cheapest straight step, so none goes farther than this: a
        // boat's every step is one at sea (a landing's priced from its water tile, E08 T73b).
        let cheapest = match mode {
            Mode::Sail(_) => pricing.sail_cheapest(),
            _ => pricing.cheapest(),
        };
        let radius = u16::try_from(hero.movement / cheapest).unwrap_or(u16::MAX);
        let origin = hero.pos;
        let left = origin.x.saturating_sub(radius);
        let top = origin.y.saturating_sub(radius);
        let right = origin.x.saturating_add(radius).min(map.width() - 1);
        let bottom = origin.y.saturating_add(radius).min(map.height() - 1);
        let (width, height) = (right - left + 1, bottom - top + 1);
        let size = usize::from(width) * usize::from(height);
        let mut reach = Self {
            origin,
            start: usize::from(origin.y - top) * usize::from(width) + usize::from(origin.x - left),
            left,
            top,
            width,
            height,
            points: vec![None; size],
            via: vec![0; size],
            passing: Vec::new(),
            passing_via: Vec::new(),
        };
        // Another hero's tile: a visit, which ends the path (GAM-BAT-004, GAM-MET-001); an empty
        // boat's: a boarding, which does too (ADV-SEA-002, E08 T73b). Whether one of them lies on
        // the window's water (another hero in a boat, an empty boat) decides whether the walking
        // search looks at water at all (`bench-check`).
        let mut occupied = vec![false; size];
        let mut afloat = false;
        for other in state.heroes().iter().filter(|h| h.id != hero.id) {
            if let Some(slot) = reach.slot(other.pos) {
                occupied[slot] = true;
                afloat |= other.boat.is_some();
            }
        }
        let boats = state.boats().map(|(_, boat)| boat);
        for boat in boats.filter(|boat| !boat.aboard && !boat.gone) {
            if let Some(slot) = reach.slot(boat.tile) {
                occupied[slot] = true;
                afloat = true;
            }
        }
        let details = details_of(heroes, hero.id);
        let search = Search {
            state,
            part,
            map,
            pricing,
            hero: hero.id,
            owner: hero.owner,
            occupied,
            guards: map.any_monster(),
            surface: (core, details, hero.max_movement, &hero.army),
        };
        reach.points[reach.start] = Some(hero.movement);
        if let Mode::Sail(boat) = mode {
            reach.search_sail(&search, boat, hero.movement);
            return reach;
        }
        if mode != Mode::Walk {
            reach.search_layered(&search, mode, hero.movement);
            return reach;
        }
        if afloat {
            reach.search_walking::<true>(&search, hero.movement);
        } else {
            reach.search_walking::<false>(&search, hero.movement);
        }
        reach
    }

    /// A walking hero's search: Dijkstra's over one node per tile, the most points first (ties by
    /// slot), from the hero's tile with `points`. `AFLOAT`: the window holds an empty boat or
    /// another hero in a boat, whose water tiles the search then looks at (E08 T73b); without, its
    /// loop is the one before boats (`bench-check`).
    fn search_walking<const AFLOAT: bool>(&mut self, search: &Search<'_>, points: u32) {
        let mut queue = BinaryHeap::new();
        // ADV-OBJ-004: a hero standing on the tile of an object not visited from above leaves it
        // toward none of the three tiles above (E08 T14a); the start's own expansion only, so the
        // search's hot loop has no test for it.
        let closed = search.state.closed_above(search.part, self.origin);
        self.expand::<true, AFLOAT>(search, &mut queue, self.start, points, closed);
        while let Some((points, Reverse(slot))) = queue.pop() {
            if self.points[slot] != Some(points) {
                continue;
            }
            self.expand::<false, AFLOAT>(search, &mut queue, slot, points, false);
        }
    }

    /// Relaxes the steps from `slot`, reached with `points` left, in the eight directions; a
    /// tile reached with more points than before goes on the queue unless a visit or a guard ends
    /// the path there. `START` is the hero's own tile, whose steps up are skipped when `closed`;
    /// `AFLOAT` as [`Self::search_walking`]'s.
    // rule: ADV-OBJ-003, ADV-OBJ-004, ADV-VIS-005, GAM-BAT-004, ADV-MON-006
    #[inline(always)]
    fn expand<const START: bool, const AFLOAT: bool>(
        &mut self,
        search: &Search<'_>,
        queue: &mut BinaryHeap<(u32, Reverse<usize>)>,
        slot: usize,
        points: u32,
        closed: bool,
    ) {
        let (state, part, pricing) = (search.state, search.part, &search.pricing);
        let at = self.pos(slot);
        let from = search.map.at(at);
        for (dir, &(dx, dy)) in DIRECTIONS.iter().enumerate() {
            if START && closed && dy < 0 {
                continue;
            }
            let Some(next) = self.neighbour(slot, dx, dy) else {
                continue;
            };
            let pos = self.pos(next);
            let to = search.map.at(pos);
            // The same tile checks as MoveHero's (ADV-MOV-007, ADV-MOV-008, ADR-0016 point 4).
            if !state.is_explored(search.owner, pos) {
                continue;
            }
            if !enterable_terrain(part, &to) {
                // Water: a hero in a boat visited, an empty boat boarded (E08 T73b), out of the
                // loop, only where one is and in a window holding one (`bench-check`).
                if AFLOAT && search.occupied[next] {
                    self.afloat(search, &from, (next, dir), points);
                }
                continue;
            }
            // Another hero is a visit (GAM-BAT-004, GAM-MET-001), but not from above onto the
            // tile of an object not visited from above (ADV-OBJ-004: the object under it decides,
            // E08 T33). Else what objects still on the map cover there (the
            // occupancy layer): a visit ends the path (ADV-OBJ-003), a blocked tile, a visit not
            // built yet or one entered from above against ADV-OBJ-004 stops it, and a gate the
            // owner holds the key of is crossed (ADV-VIS-005): `visit_at` with the owner, as
            // `MoveHero`'s step. Directions 3 to 5 step down (DIRECTIONS).
            let mut visit = false;
            if search.occupied[next] {
                if (3..=5).contains(&dir) && state.closed_above(part, pos) {
                    continue;
                }
                visit = true;
            } else if to.blocked || to.visitable {
                let live = state.live(pos, to);
                if live.visitable {
                    match state.visit_at(part, search.owner, pos) {
                        Some(v) if v.from_above || !(3..=5).contains(&dir) => visit = !v.pass,
                        _ => continue,
                    }
                } else if live.blocked {
                    continue;
                }
            }
            let price = pricing.walking_step(&from, &to, dir % 2 == 1, points);
            let Some(left) = points.checked_sub(price) else {
                continue;
            };
            let left = pricing.walking_left_after(&to, left);
            if self.points[next].is_none_or(|best| left > best) {
                self.points[next] = Some(left);
                self.via[next] = dir as u8;
                // A step onto a guarded tile ends the path there too (ADV-MON-006), asked only
                // of a tile the search improves, on a map with monsters (`bench-check`).
                if !visit && !(search.guards && search.guarded(pos)) {
                    queue.push((left, Reverse(next)));
                }
            }
        }
    }

    /// A walking hero's step from `from` onto the water tile of `next` in `dir` with `points`
    /// (E08 T73b; increment 8's design note §9), where another hero stands in a boat or an empty
    /// boat lies: as `MoveHero`'s `check_step`, not down across the top of an object's tile
    /// (ADV-OBJ-004), a hero visited from `from` (GAM-BAT-004, GAM-MET-001) with the walker's drop,
    /// else the boat boarded (ADV-SEA-002) with a boat's drop and the conversion
    /// ([`Search::boarded`]); each a destination the search goes no further from. The step is
    /// priced as the walker's from its land tile.
    // rule: ADV-SEA-002, GAM-BAT-004, GAM-MET-001
    #[cold]
    #[inline(never)]
    fn afloat(
        &mut self,
        search: &Search<'_>,
        from: &StaticTile,
        (next, dir): (usize, usize),
        points: u32,
    ) {
        let (state, pricing) = (search.state, &search.pricing);
        let pos = self.pos(next);
        if (3..=5).contains(&dir) && state.closed_above(search.part, pos) {
            return;
        }
        let to = search.map.at(pos);
        let price = pricing.walking_step(from, &to, dir % 2 == 1, points);
        let Some(left) = points.checked_sub(price) else {
            return;
        };
        let hero = state
            .heroes()
            .iter()
            .any(|h| h.pos == pos && h.id != search.hero);
        let left = if hero {
            pricing.walking_left_after(&to, left)
        } else {
            search.boarded(&to, left)
        };
        self.improve((next, Node::Stop), left, dir as u8);
    }

    /// A hero in a boat's search (E08 T73b; increment 8's design note §9; ADV-SEA-003 to -005):
    /// Dijkstra's over one node per tile, the most points first (ties by slot), from the boat's
    /// tile with `points`, every step one [`AdventureState::passage`] allows the boat `boat` (each
    /// a stop), priced from the water tile ([`Pricing::sail`]) with the drop of
    /// [`Pricing::left_by`]. A landing, a visit and a water tile a monster guards are destinations
    /// the search goes no further from; a landing keeps the points of its conversion
    /// ([`Search::landed`]).
    // rule: ADV-SEA-003, ADV-SEA-004, ADV-SEA-005, ADV-MON-006
    fn search_sail(&mut self, search: &Search<'_>, boat: u32, points: u32) {
        let mut queue = BinaryHeap::new();
        self.expand_sail(search, &mut queue, boat, self.start, points);
        while let Some((points, Reverse(slot))) = queue.pop() {
            if self.points[slot] != Some(points) {
                continue;
            }
            self.expand_sail(search, &mut queue, boat, slot, points);
        }
    }

    /// Relaxes the steps of a hero in the boat `boat` from `slot`, reached with `points` left, in
    /// the eight directions ([`Self::search_sail`]).
    fn expand_sail(
        &mut self,
        search: &Search<'_>,
        queue: &mut BinaryHeap<(u32, Reverse<usize>)>,
        boat: u32,
        slot: usize,
        points: u32,
    ) {
        let (state, part, pricing) = (search.state, search.part, &search.pricing);
        let mode = Mode::Sail(boat);
        let at = self.pos(slot);
        let from = search.map.at(at);
        let mover = (search.owner, search.hero, mode);
        for (dir, &(dx, dy)) in DIRECTIONS.iter().enumerate() {
            let Some(next) = self.neighbour(slot, dx, dy) else {
                continue;
            };
            let pos = self.pos(next);
            let Ok(entry) = state.passage(part, mover, (at, Node::Stop), pos, dir, 0) else {
                continue;
            };
            // A boat's every entry is a stop (`sail_entry`): what it meets, if anything.
            let met = entry.stop.flatten();
            let Some(left) = points.checked_sub(pricing.sail(&from, dir % 2 == 1, points)) else {
                continue;
            };
            let left = pricing.left_by(mode, met, &entry.tile, left);
            let left = match met {
                Some(Meets::Land(_)) => search.landed(left),
                _ => left,
            };
            if self.improve((next, Node::Stop), left, dir as u8)
                && met.is_none()
                && !(search.guards && search.guarded(pos))
            {
                queue.push((left, Reverse(next)));
            }
        }
    }

    /// A flying or water-walking hero's search (E08 T41): Dijkstra's over the nodes (tile, kind),
    /// the most points first (ties by slot, then the stop node first), from the hero's tile as a
    /// stop node with `points`.
    fn search_layered(&mut self, search: &Search<'_>, mode: Mode, points: u32) {
        let size = self.points.len();
        self.passing = vec![None; size];
        self.passing_via = vec![0; size];
        let mut queue = BinaryHeap::new();
        self.expand_layered(search, &mut queue, mode, (self.start, Node::Stop), points);
        while let Some((points, Reverse(index))) = queue.pop() {
            let node = (
                index / 2,
                if index % 2 == 0 {
                    Node::Stop
                } else {
                    Node::Pass
                },
            );
            if self.node_points(node) != Some(points) {
                continue;
            }
            self.expand_layered(search, &mut queue, mode, node, points);
        }
    }

    /// The points left on a node, if the search reached it.
    fn node_points(&self, (slot, kind): (usize, Node)) -> Option<u32> {
        match kind {
            Node::Stop => self.points[slot],
            Node::Pass => self.passing[slot],
        }
    }

    /// Relaxes the steps from the node `(slot, kind)`, reached with `points` left, in the eight
    /// directions (a flying or water-walking hero, E08 T41): each step [`AdventureState::passage`]
    /// allows, priced from and to the tiles as they stand now, makes the step's stop node, its
    /// pass node or both; a node reached with more points than before goes on the queue unless a
    /// visit ends the path there, or a guard does for a hero that does not fly (ADV-MON-006: a
    /// flight goes on over guarded tiles, 0x56BBCE).
    // rule: ADV-MOV-009, ADV-MOV-010, ADV-MON-006
    fn expand_layered(
        &mut self,
        search: &Search<'_>,
        queue: &mut BinaryHeap<(u32, Reverse<usize>)>,
        mode: Mode,
        (slot, kind): (usize, Node),
        points: u32,
    ) {
        let (state, part, pricing) = (search.state, search.part, &search.pricing);
        let at = self.pos(slot);
        let from = state.live(at, search.map.at(at));
        let mover = (search.owner, search.hero, mode);
        for (dir, &(dx, dy)) in DIRECTIONS.iter().enumerate() {
            let Some(next) = self.neighbour(slot, dx, dy) else {
                continue;
            };
            let pos = self.pos(next);
            let Ok(entry) = state.passage(part, mover, (at, kind), pos, dir, 0) else {
                continue;
            };
            let price = pricing.step(&from, &entry.tile, dir % 2 == 1, points);
            let Some(price_left) = points.checked_sub(price) else {
                continue;
            };
            let left = pricing.left_after(&entry.tile, price_left);
            let via = dir as u8 | if kind == Node::Pass { FROM_PASS } else { 0 };
            let guarded = || mode != Mode::Fly && search.guards && search.guarded(pos);
            // A boarding keeps the points a boat's drop and the conversion leave (E08 T73,
            // ADV-SEA-002; the move's `Pricing::left_by` and `converted_points`).
            let stop = match entry.stop {
                Some(Some(Meets::Board(_))) => search.boarded(&entry.tile, price_left),
                _ => left,
            };
            if let Some(met) = entry.stop
                && self.improve((next, Node::Stop), stop, via)
                && met.is_none()
                && !guarded()
            {
                queue.push((left, Reverse(next * 2)));
            }
            if entry.pass && self.improve((next, Node::Pass), left, via) && !guarded() {
                queue.push((left, Reverse(next * 2 + 1)));
            }
        }
    }

    /// Records `left` points on a node entered by `via` when they are more than it had; whether
    /// they were.
    fn improve(&mut self, (slot, kind): (usize, Node), left: u32, via: u8) -> bool {
        let (points, vias) = match kind {
            Node::Stop => (&mut self.points, &mut self.via),
            Node::Pass => (&mut self.passing, &mut self.passing_via),
        };
        if points[slot].is_some_and(|best| left <= best) {
            return false;
        }
        points[slot] = Some(left);
        vias[slot] = via;
        true
    }

    /// The window's slot of `pos`, if the window holds it.
    fn slot(&self, pos: Pos) -> Option<usize> {
        let column = pos.x.checked_sub(self.left).filter(|&c| c < self.width)?;
        let row = pos.y.checked_sub(self.top).filter(|&r| r < self.height)?;
        (pos.level == self.origin.level)
            .then(|| usize::from(row) * usize::from(self.width) + usize::from(column))
    }

    /// The tile of a slot.
    fn pos(&self, slot: usize) -> Pos {
        let width = usize::from(self.width);
        let column = u16::try_from(slot % width).unwrap_or(u16::MAX);
        let row = u16::try_from(slot / width).unwrap_or(u16::MAX);
        Pos::new(self.left + column, self.top + row, self.origin.level)
    }

    /// The slot one step `dx`, `dy` from `slot`, if the window holds it.
    fn neighbour(&self, slot: usize, dx: i8, dy: i8) -> Option<usize> {
        let width = usize::from(self.width);
        let column = (slot % width).checked_add_signed(isize::from(dx))?;
        let row = (slot / width).checked_add_signed(isize::from(dy))?;
        (column < width && row < usize::from(self.height)).then(|| row * width + column)
    }

    /// The tile the hero stands on.
    pub const fn origin(&self) -> Pos {
        self.origin
    }

    /// The points the hero would have left on `pos` (on its own tile: its points now), if it can
    /// reach it today; for a tile it visits a pickup from, the points it keeps after the visit on
    /// the tile before.
    pub fn points_left(&self, pos: Pos) -> Option<u32> {
        self.slot(pos).and_then(|slot| self.points[slot])
    }

    /// The chosen path to `pos`, if the hero can reach it: the tiles it steps onto, in order,
    /// without its own (empty for its own tile), as `MoveHero` takes them (a visit's tile last).
    /// A flying or water-walking hero's path follows each node's predecessor of the kind it
    /// recorded (E08 T41).
    pub fn path(&self, pos: Pos) -> Option<Vec<Pos>> {
        let mut slot = self.slot(pos)?;
        self.points[slot]?;
        let width = isize::try_from(self.width).unwrap_or(isize::MAX);
        let mut path = Vec::new();
        let mut pass = false;
        while pass || slot != self.start {
            path.push(self.pos(slot));
            // Back along the step that entered the node: the chain ends at the start's stop node.
            let via = if pass {
                self.passing_via[slot]
            } else {
                self.via[slot]
            };
            let (dx, dy) = DIRECTIONS[usize::from(via & !FROM_PASS)];
            pass = via & FROM_PASS != 0;
            slot = slot.wrapping_add_signed(-(isize::from(dy) * width + isize::from(dx)));
        }
        path.reverse();
        Some(path)
    }

    /// Every tile the hero can reach today but its own, row by row, with the points it would have
    /// left there.
    pub fn tiles(&self) -> impl Iterator<Item = (Pos, u32)> + '_ {
        self.points
            .iter()
            .enumerate()
            .filter_map(|(slot, points)| points.map(|points| (self.pos(slot), points)))
            .filter(|&(pos, _)| pos != self.origin)
    }
}

impl AdventureState {
    /// The tiles `hero` can reach today over the tiles its owner has explored (ADV-MOV-008), if it
    /// is on the map (see [`Reach`]). The caller decides whose hero it may ask about. `heroes` is
    /// the game's store of heroes' details, as for [`Self::apply`] (E08 T20a2b): the hero's
    /// Pathfinding prices its steps (E08 T40), its flying and water walking price them and decide
    /// where it passes and stops (T41).
    // rule: ADV-MOV-008
    pub fn reach(
        &self,
        core: &Ruleset,
        part: &AdventureRuleset,
        heroes: &[Arc<HeroDetails>],
        hero: HeroId,
    ) -> Option<Reach> {
        self.hero(hero)
            .map(|hero| Reach::search(self, core, part, heroes, hero))
    }
}

#[cfg(test)]
mod tests;
