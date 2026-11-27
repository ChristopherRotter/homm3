//! The jumps of the computer's search (CPU-PTH-007): mode 4's Dimension Door legs with
//! `pathfinder_relax` 0x4B1A70's rules for them, its Fly stretch and sea spells, the boat node of
//! Summon Boat and of the caller's boat marks (in every mode), and from mode 4 the boarding's and
//! the landing's day's rest, as `pathfinder_expand` 0x4B2300 makes them. The spells are the
//! caller's ([`super::SearchJumps`]): the search reads no spell points.

use super::marks::{BOAT, DOOR, DOOR_OFFSET_SHIFT, FLY, FROM_PLACE, NO_MAGIC, PLACE, WATER_WALK};
use super::{DOOR_OFFSET_MASK, Frame, Search, SearchAsks, SearchMode, SearchNode};
use crate::state::StaticTile;

/// The columns and the rows a Door leg goes at most from its start (0x4B1B7D, 0x4B1B8D).
const LEG_COLUMNS: i32 = 9;
const LEG_ROWS: i32 = 8;

/// The second cost a jump's start adds (0x4B297E, 0x4B29ED, 0x4B2B48, 0x4B2BFD, 0x4B2C48).
const JUMP_SURCHARGE: u16 = 500;

/// `pathfinder_relax` 0x4B1A70's rules for a Door leg's `node` relaxed from `source` with the
/// stop flag `stop` (0x4B1B21..0x4B1BAF; its source and [`FROM_PLACE`] already the source's):
/// refused when it lands (a place) on an entrance; from a pass node of the leg, the leg's start
/// (the source's own source) as its source, with [`FROM_PLACE`] (0x4B1B50); refused more than 9
/// columns or 8 rows from that start, else its offset from it in bits 16 to 25, the columns' in
/// the low 5 bits and the rows' in the high 5, each two's complement.
// rule: CPU-PTH-007
pub(super) fn leap(source: &SearchNode, node: &mut SearchNode, stop: bool) -> bool {
    if node.marks & PLACE != 0 && stop {
        return false;
    }
    if source.marks & PLACE == 0 {
        node.from = source.from;
        node.marks |= FROM_PLACE;
    }
    let dx = i32::from(node.pos.x) - i32::from(node.from.x);
    let dy = i32::from(node.pos.y) - i32::from(node.from.y);
    if dx.abs() > LEG_COLUMNS || dy.abs() > LEG_ROWS {
        return false;
    }
    let offset = ((dy & 0x1F) << 5 | (dx & 0x1F)) as u32;
    node.marks = node.marks & !DOOR_OFFSET_MASK | offset << DOOR_OFFSET_SHIFT;
    true
}

/// Whether a spell may be cast at `source`: a place without the no-magic mark (0x4B2964..0x4B2975,
/// 0x4B29C1..0x4B29D2, 0x4B2AD6, 0x4B2BC3).
pub(super) const fn may_cast(source: &SearchNode) -> bool {
    source.marks & PLACE != 0 && source.marks & NO_MAGIC == 0
}

/// The price of a step that takes the day's rest (`pathfinder_landing_day` 0x4B28A6,
/// `pathfinder_boat_day` 0x4B2B8B): the node's `points` left when they pay its `price`, else
/// those and another day's `maximum`.
const fn rest(points: u16, price: u32, maximum: u32) -> u32 {
    let points = points as u32;
    if points >= price {
        points
    } else {
        maximum.wrapping_add(points)
    }
}

impl<A: SearchAsks> Search<'_, A> {
    /// A Dimension Door leg's start on `node` (`pathfinder_door_start` 0x4B2984, 0x4B29FC,
    /// 0x4B2C03): its second cost 500 more, the leg's mark, and the spell's price, which the
    /// relax takes; its points stay the step's.
    // rule: CPU-PTH-007
    pub(super) fn door_leg(&self, node: &mut SearchNode) -> u32 {
        node.cost2 = node.cost2.wrapping_add(JUMP_SURCHARGE);
        node.marks |= DOOR;
        self.cast.door_price
    }

    /// The price of `node`'s step between `tiles` by its flying and water walking, its points the
    /// day's maximum less it, as at a cast at the day's start (0x4B2AB4..0x4B2ABD,
    /// 0x4B2CD8..0x4B2CE1).
    fn lifted(
        &self,
        node: &mut SearchNode,
        (from, to): (&StaticTile, &StaticTile),
        diagonal: bool,
        window: u32,
    ) -> u32 {
        let price = self.pricing(node.marks).step(from, to, diagonal, window);
        node.points = (self.land as u16).wrapping_sub(price as u16);
        price
    }

    /// The Fly stretch on `node`, a step only flown over (`pathfinder_fly_stretch` 0x4B2A73):
    /// its second cost 500 more (0x4B29ED), flying, priced at the fly level ([`Self::lifted`]).
    // rule: CPU-PTH-007
    pub(super) fn fly_stretch(
        &self,
        node: &mut SearchNode,
        tiles: (&StaticTile, &StaticTile),
        diagonal: bool,
        window: u32,
    ) -> u32 {
        node.cost2 = node.cost2.wrapping_add(JUMP_SURCHARGE);
        node.marks |= FLY;
        self.lifted(node, tiles, diagonal, window)
    }

    /// The sea spell on `node`, a step from a place onto plain water (`pathfinder_sea_spell_choice`
    /// 0x4B2C31): Fly when the fly level is above the water-walk level, else Water Walk, whichever
    /// of the two is castable (none a level below any); its second cost 500 more unless the hero
    /// flies or water-walks so on its own (Angel Wings, the Boots of Levitation: 0x4B2C3F,
    /// 0x4B2C85), priced at its level ([`Self::lifted`]).
    // rule: CPU-PTH-007
    pub(super) fn sea_spell(
        &self,
        node: &mut SearchNode,
        tiles: (&StaticTile, &StaticTile),
        diagonal: bool,
        window: u32,
    ) -> u32 {
        let (fly, walk) = self.levels;
        let (lift, own) = if fly > walk {
            (FLY, self.wings)
        } else {
            (WATER_WALK, self.boots)
        };
        if !own {
            node.cost2 = node.cost2.wrapping_add(JUMP_SURCHARGE);
        }
        node.marks |= lift;
        self.lifted(node, tiles, diagonal, window)
    }

    /// The boat node of `node`, a step from `source` onto plain water in `direction`
    /// (0x4B2AE5..0x4B2BB8, in every mode), from a place only (0x4B2ADC: the search's every pass
    /// node flies, water-walks or leaps, whose sea steps go on before, so the test always holds;
    /// and the step's tile is no entrance, 0x4B2AEC, as plain water's): when Summon Boat is
    /// useful and `source` has no no-magic mark, else at a boat mark of the caller's
    /// ([`SearchAsks::boat_mark`],
    /// `pathfinder_boat_mark_read` 0x4B2B01), whose purchase lowers its key (0x4B2B67). A copy of
    /// `node` in a boat and a place, its second cost 500 more, priced as a boat's step from
    /// `from` (0x4B2B40); from mode 4 the day's rest (the source's points when they pay it, else
    /// those and the land's maximum, `pathfinder_boat_day` 0x4B2B8B), its flying and water walking
    /// cleared and its points the sea's maximum; below, the bare price and `node`'s points. Relaxed
    /// with its key and guard, no stop.
    // rule: CPU-PTH-007
    pub(super) fn boat_node(
        &self,
        frame: &mut Frame,
        (source, node): (&SearchNode, &SearchNode),
        direction: u32,
        (from, diagonal, window): (&StaticTile, bool, u32),
    ) {
        let summoned = self.cast.summon_boat && source.marks & NO_MAGIC == 0;
        let change = if summoned {
            Some(0)
        } else {
            self.asks.boat_mark(node.pos)
        };
        let place = source.marks & PLACE != 0;
        change.filter(|_| place).into_iter().for_each(|change| {
            let mut boat = SearchNode {
                marks: node.marks | BOAT | PLACE,
                key: node.key.wrapping_add(change),
                cost2: node.cost2.wrapping_add(JUMP_SURCHARGE),
                ..*node
            };
            let mut price = self.pricings[0].sail(from, diagonal, window);
            if self.start.mode >= SearchMode::Plan {
                price = rest(source.points, price, self.land);
                boat.marks &= !(FLY | WATER_WALK);
                boat.points = self.sea as u16;
            }
            let (key, guard) = (boat.key, boat.guard);
            self.relax(
                frame,
                source,
                &mut boat,
                (direction, price),
                (key, guard, false),
            );
        });
    }

    /// The landing of `node` from the boat node `source` onto a free coastal tile, from mode 4
    /// (0x4B2890..0x4B28B6): the step priced `price` takes the day's rest (the source's points when
    /// they pay it, else those and the sea's maximum, `pathfinder_landing_day` 0x4B28A6), the node
    /// leaves the boat and its flying and water walking, its points the land's maximum.
    // rule: CPU-PTH-007
    pub(super) fn landing(&self, source: &SearchNode, node: &mut SearchNode, price: u32) -> u32 {
        node.marks &= !(BOAT | FLY | WATER_WALK);
        node.points = self.land as u16;
        rest(source.points, price, self.sea)
    }
}
