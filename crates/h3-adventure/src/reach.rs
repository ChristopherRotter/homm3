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

use alloc::collections::BinaryHeap;
use alloc::vec;
use alloc::vec::Vec;
use core::cmp::Reverse;

use h3_content::Ruleset;
use h3_contracts::HeroId;

use crate::movement::{DIRECTIONS, Pricing};
use crate::ruleset::AdventureRuleset;
use crate::setup::Pos;
use crate::state::{AdventureState, HeroOnMap, enterable_terrain};

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
    /// The search from `hero`'s tile with its points.
    fn search(
        state: &AdventureState,
        core: &Ruleset,
        part: &AdventureRuleset,
        hero: &HeroOnMap,
    ) -> Self {
        let map = state.map();
        let pricing = Pricing::new(core, part, &hero.army);
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
        let mut occupied = vec![false; size];
        for other in state.heroes().iter().filter(|h| h.id != hero.id) {
            if let Some(slot) = reach.slot(other.pos) {
                occupied[slot] = true;
            }
        }
        reach.points[reach.start] = Some(hero.movement);
        let mut queue = BinaryHeap::from([(hero.movement, Reverse(reach.start))]);
        while let Some((points, Reverse(slot))) = queue.pop() {
            if reach.points[slot] != Some(points) {
                continue;
            }
            let at = reach.pos(slot);
            let from = map.at(at);
            for (dir, &(dx, dy)) in DIRECTIONS.iter().enumerate() {
                let Some(next) = reach.neighbour(slot, dx, dy) else {
                    continue;
                };
                let pos = reach.pos(next);
                let to = map.at(pos);
                // The same tile checks as MoveHero's (ADV-MOV-007, ADV-MOV-008, ADR-0016 point 4).
                if !state.is_explored(hero.owner, pos)
                    || occupied[next]
                    || !enterable_terrain(part, &to)
                    || ((to.blocked || to.visitable) && {
                        // An object still on the map covers it (the occupancy layer).
                        let live = state.live(pos, to);
                        live.blocked || live.visitable
                    })
                {
                    continue;
                }
                let Some(left) = points.checked_sub(pricing.step(&from, &to, dir % 2 == 1, points))
                else {
                    continue;
                };
                let left = pricing.left_after(&to, left);
                if reach.points[next].is_none_or(|best| left > best) {
                    reach.points[next] = Some(left);
                    reach.via[next] = dir as u8;
                    queue.push((left, Reverse(next)));
                }
            }
        }
        reach
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
    /// reach it today.
    pub fn points_left(&self, pos: Pos) -> Option<u32> {
        self.slot(pos).and_then(|slot| self.points[slot])
    }

    /// The chosen path to `pos`, if the hero can reach it: the tiles it steps onto, in order,
    /// without its own (empty for its own tile), as `MoveHero` takes them.
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
    /// is on the map (see [`Reach`]). The caller decides whose hero it may ask about.
    // rule: ADV-MOV-008
    pub fn reach(&self, core: &Ruleset, part: &AdventureRuleset, hero: HeroId) -> Option<Reach> {
        self.hero(hero)
            .map(|hero| Reach::search(self, core, part, hero))
    }
}

#[cfg(test)]
mod tests;
