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
//! the search goes on from it in every direction. Another player's hero's tile is a destination
//! too, its visit a battle (GAM-BAT-004, E08 T19a), from any direction, whatever object lies under
//! it; an own hero's tile blocks. A tile a monster guards (ADV-MON-005) is a destination as well:
//! the search does not go on from it, but from the hero's own tile (ADV-MON-006, E08 T20a2b; the
//! original's pathfinder, 0x56BC05).

use alloc::collections::BinaryHeap;
use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;
use core::cmp::Reverse;

use h3_content::Ruleset;
use h3_contracts::{HeroId, PlayerId};
use h3_heroes::HeroDetails;

use crate::movement::{DIRECTIONS, Pricing, details_of, hero_movement};
use crate::ruleset::AdventureRuleset;
use crate::setup::Pos;
use crate::state::{AdventureState, HeroOnMap, StaticMap, enterable_terrain};

/// What a search reads besides its window: the state, the rules, the hero's pricing and owner,
/// and the window's tiles other heroes stand on ([`FREE`], [`OWN`] or [`ENEMY`]).
struct Search<'s> {
    state: &'s AdventureState,
    part: &'s AdventureRuleset,
    map: &'s StaticMap,
    pricing: Pricing<'s>,
    owner: PlayerId,
    occupied: Vec<u8>,
    /// Whether the map holds a monster, so that a tile may be guarded (E08 T20a2b): the search
    /// asks [`Search::guarded`] only then (`bench-check`).
    guards: bool,
}

impl Search<'_> {
    /// Whether a monster guards `pos` (ADV-MON-005), out of the search's hot loop.
    #[cold]
    #[inline(never)]
    fn guarded(&self, pos: Pos) -> bool {
        self.map.near_monster(pos) && self.state.guard_of(self.part, pos).is_some()
    }
}

/// No other hero stands on the tile.
const FREE: u8 = 0;
/// Another hero of the same owner stands there: the tile blocks.
const OWN: u8 = 1;
/// Another player's hero stands there: a visit, which ends the path (GAM-BAT-004).
const ENEMY: u8 = 2;

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
        let pricing = Pricing::new(core, part, &hero.army, &movement);
        // No step costs less than the cheapest straight step, so none goes farther than this.
        let radius = u16::try_from(hero.movement / pricing.cheapest()).unwrap_or(u16::MAX);
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
        };
        let mut occupied = vec![FREE; size];
        for other in state.heroes().iter().filter(|h| h.id != hero.id) {
            if let Some(slot) = reach.slot(other.pos) {
                occupied[slot] = if other.owner == hero.owner {
                    OWN
                } else {
                    ENEMY
                };
            }
        }
        let search = Search {
            state,
            part,
            map,
            pricing,
            owner: hero.owner,
            occupied,
            guards: map.any_monster(),
        };
        reach.points[reach.start] = Some(hero.movement);
        let mut queue = BinaryHeap::new();
        // ADV-OBJ-004: a hero standing on the tile of an object not visited from above leaves it
        // toward none of the three tiles above (E08 T14a); the start's own expansion only, so the
        // search's hot loop has no test for it.
        let closed = state.closed_above(part, origin);
        reach.expand::<true>(&search, &mut queue, reach.start, hero.movement, closed);
        while let Some((points, Reverse(slot))) = queue.pop() {
            if reach.points[slot] != Some(points) {
                continue;
            }
            reach.expand::<false>(&search, &mut queue, slot, points, false);
        }
        reach
    }

    /// Relaxes the steps from `slot`, reached with `points` left, in the eight directions; a
    /// tile reached with more points than before goes on the queue unless a visit or a guard ends
    /// the path there. `START` is the hero's own tile, whose steps up are skipped when `closed`.
    // rule: ADV-OBJ-003, ADV-OBJ-004, ADV-VIS-005, GAM-BAT-004, ADV-MON-006
    #[inline(always)]
    fn expand<const START: bool>(
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
            if !state.is_explored(search.owner, pos) || !enterable_terrain(part, &to) {
                continue;
            }
            // Another player's hero is a visit (GAM-BAT-004), but not from above onto the tile of
            // an object not visited from above (ADV-OBJ-004: the object under it decides, E08
            // T33); an own hero blocks. Else what objects still on the map cover there (the
            // occupancy layer): a visit ends the path (ADV-OBJ-003), a blocked tile, a visit not
            // built yet or one entered from above against ADV-OBJ-004 stops it, and a gate the
            // owner holds the key of is crossed (ADV-VIS-005): `visit_at` with the owner, as
            // `MoveHero`'s step. Directions 3 to 5 step down (DIRECTIONS).
            let mut visit = false;
            let occupied = search.occupied[next];
            if occupied != FREE {
                if occupied == OWN || ((3..=5).contains(&dir) && state.closed_above(part, pos)) {
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
            let Some(left) = points.checked_sub(pricing.step(&from, &to, dir % 2 == 1, points))
            else {
                continue;
            };
            let left = pricing.left_after(&to, left);
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
    pub fn path(&self, pos: Pos) -> Option<Vec<Pos>> {
        let mut slot = self.slot(pos)?;
        self.points[slot]?;
        let width = isize::try_from(self.width).unwrap_or(isize::MAX);
        let mut path = Vec::new();
        while slot != self.start {
            path.push(self.pos(slot));
            // Back along the step that entered the slot: the chain ends at the start.
            let (dx, dy) = DIRECTIONS[usize::from(self.via[slot])];
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
    /// Pathfinding prices its steps (E08 T40; its flying and water walking with T41).
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
