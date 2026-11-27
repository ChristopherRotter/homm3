//! The guards of the computer's search (CPU-PTH-004): `find_guarding_monster` 0x41A460 with its
//! exclusion, the guarded node below mode 4, and from mode 4 `pathfinder_target_weigh` 0x56AAD0
//! and `pathfinder_entrance_guarded` 0x56A360 with the caller's fight answer.

use super::{Class, Ground, REFUSED, Search, SearchAsks, SearchMode, SearchNode};
use crate::setup::Pos;

impl<A: SearchAsks> Search<'_, A> {
    /// The monster guarding `centre`, if one does, but the one at `excluded`
    /// (`find_guarding_monster` 0x41A460, read whole, R3 §6): the 3 × 3 block around it on its
    /// level, clipped to the map, columns from the left and each column's rows from the top,
    /// the row above left out when `centre` is the entrance of an object not visited from above
    /// (0x41A55B); the first tile that is a monster's entrance (class 54) on water as `centre`
    /// is (0x41A5A9), but `excluded`'s tile, which the scan goes past (0x41A5CA..0x41A5F7). The
    /// engine's [`crate::AdventureState::guard_of`] scans the same, without the exclusion.
    // rule: CPU-PTH-004
    pub(super) fn guarding(&self, centre: Pos, excluded: Option<Pos>) -> Option<Pos> {
        let spot = self.spot(centre);
        let water = spot.ground == Ground::Water;
        let first = if spot.closed {
            centre.y
        } else {
            centre.y.saturating_sub(1)
        };
        let rows = first..centre.y.saturating_add(2).min(self.map.height());
        let columns = centre.x.saturating_sub(1)..centre.x.saturating_add(2).min(self.map.width());
        columns
            .flat_map(|x| rows.clone().map(move |y| Pos::new(x, y, centre.level)))
            .find(|&tile| {
                let spot = self.spot(tile);
                (spot.ground == Ground::Water) == water
                    && excluded != Some(tile)
                    && (spot.tile.entrance).is_some_and(|entrance| entrance.class == Class::Monster)
            })
    }

    /// `pathfinder_target_weigh` 0x56AAD0 (from mode 4, a step onto another hero, a garrison or
    /// a monster, 0x4B2932): yes when `node`'s path met its tile's guard already; else the
    /// caller's fight answer at the tile, no at or below −500 000 000, below 0 the tile the
    /// node's guard and the answer added to its key.
    // rule: CPU-PTH-004
    pub(super) fn target_weigh(&self, node: &mut SearchNode) -> bool {
        if node.guard == Some(node.pos) {
            return true;
        }
        let answer = self.asks.fight_answer(node.pos);
        if answer <= REFUSED {
            return false;
        }
        if answer < 0 {
            node.guard = Some(node.pos);
            node.key = node.key.wrapping_add(answer);
        }
        true
    }

    /// `pathfinder_entrance_guarded` 0x56A360 (from mode 4 every step neither only flown over
    /// nor a jump's, 0x4B2A51; a town's passage from mode 3, 0x56AEF1): whether a guard stops
    /// `node`: a tile a monster may guard (`tile_vision_word`'s guard flag, 0x56A397) with a
    /// guard other than the one its path met (0x56A3AD); in mode 3 yes; else the caller's fight
    /// answer at the guard, yes at or below −500 000 000, below 0 the guard the node's and the
    /// answer added to its key.
    // rule: CPU-PTH-004
    pub(super) fn entrance_guarded(&self, node: &mut SearchNode) -> bool {
        let guard = if self.state.may_guard(node.pos) {
            self.guarding(node.pos, node.guard)
        } else {
            None
        };
        guard.is_some_and(|guard| {
            if self.start.mode == SearchMode::Threat {
                return true;
            }
            let answer = self.asks.fight_answer(guard);
            if answer <= REFUSED {
                return true;
            }
            if answer < 0 {
                node.guard = Some(guard);
                node.key = node.key.wrapping_add(answer);
            }
            false
        })
    }
}
