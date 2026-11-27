//! The teleporters of the computer's search (CPU-PTH-006): `pathfinder_teleport_exits` 0x56A400
//! for the monoliths and the Whirlpools, `pathfinder_gate_exit` 0x56A730 for the Subterranean
//! Gates, from `pathfinder_pass_through` 0x56AB40's cases 43, 45, 111 and 103 and from the start
//! (`pathfinder_visit_here_mark` 0x56B9CF). The original's lists by the tile's subclass and word
//! are the engine's channels ([`crate::state::Channels`], in the same walk's order); its "word"
//! that excludes a tile is the entrance's tile for a two-way monolith and its object for a
//! Whirlpool, as the jump's draw reads them (`state/jumps.rs`).

use super::marks::DIRECTION_SHIFT;
use super::{Class, Frame, REFUSED, Search, SearchAsks, SearchMode, SearchNode, Teleporter};
use crate::setup::Pos;
use crate::state::allied;

/// The list a teleporter's exits come from, as `pathfinder_teleport_exits` 0x56A400 tells them
/// by its class argument (44, 45, 111).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum List {
    /// A one-way entrance's (43): its channel's exits, class 44.
    Exits,
    /// A two-way monolith's (45): its channel's members, class 45.
    Members,
    /// A Whirlpool's (111): every Whirlpool's tiles, class 111.
    Whirlpools,
}

impl List {
    /// Whether an entrance of `class` is an exit of this list (the class argument).
    const fn fits(self, class: Class) -> bool {
        matches!(
            (self, class),
            (Self::Exits, Class::Exit)
                | (Self::Members, Class::Teleporter(Teleporter::TwoWay(_)))
                | (
                    Self::Whirlpools,
                    Class::Teleporter(Teleporter::Whirlpool(_))
                )
        )
    }
}

impl<A: SearchAsks> Search<'_, A> {
    /// The passage through the teleporter of `node`'s tile (from mode 3: `pathfinder_pass_through`
    /// 0x56AB40's cases 43 at 0x56AE14, 45 at 0x56AE59, 111 at 0x56AE9F and 103 at 0x56ADCB, and
    /// the start's copy): nothing when the node has a cost and a guard stops it
    /// (`pathfinder_entrance_guarded` 0x56A360 at 0x56A411 and 0x56ADDB, whose answer may lower the
    /// node's key first; in mode 3 any guard stops it); else a one-way entrance's channel's exits,
    /// a two-way monolith's channel's members but itself, every Whirlpool's tiles but its own
    /// Whirlpool's ([`Self::exits`]), a gate's pair ([`Self::gate_exit`]). The passage answers no
    /// after it: the node itself is a leaf.
    // rule: CPU-PTH-006
    pub(super) fn teleport(
        &self,
        frame: &mut Frame,
        node: &mut SearchNode,
        teleporter: Teleporter,
    ) {
        if node.cost > 0 && self.entrance_guarded(node) {
            return;
        }
        let channels = self.map.channels();
        let (node, at) = (&*node, node.pos);
        match teleporter {
            Teleporter::OneWay(channel) => {
                let listed = channels.exits(channel).iter().map(|&tile| (tile, false));
                self.exits(frame, node, List::Exits, listed);
            }
            Teleporter::TwoWay(channel) => {
                let listed = channels
                    .two_way(channel)
                    .iter()
                    .map(|&tile| (tile, tile == at));
                self.exits(frame, node, List::Members, listed);
            }
            Teleporter::Whirlpool(object) => {
                let listed = (channels.whirlpools().iter())
                    .map(|whirlpool| (whirlpool.tile, whirlpool.object == object));
                self.exits(frame, node, List::Whirlpools, listed);
            }
            Teleporter::Gate => self.gate_exit(frame, node),
        }
    }

    /// `pathfinder_teleport_exits` 0x56A400 from `entrance` over `listed`, its list's tiles in
    /// order, each with whether the entrance's word excludes it. From mode 4 and but for a two-way
    /// monolith a pre-pass (`teleport_guard_prepass` 0x56A447): each tile of the list's class, not
    /// excluded, with a guard (`find_guarding_monster` 0x41A460, the entrance's guard left out, no
    /// guard flag asked) is weighed by the caller's fight answer at its guard; one at or below
    /// −500 000 000 drops the whole channel (0x56A504), the lowest (from 0) lowers every exit's
    /// key. Then each exit in the list's order (0x56A545): a tile with another hero is one when the
    /// hero is not of the searching hero's team, whatever lies there (`teleport_exit_hero_test`
    /// 0x56A59C: not the current player's, unlike the jump's draw); any other when it is of the
    /// list's class and not excluded (an empty boat's tile is none). Its node is a copy of the
    /// entrance's, one copy for every exit as the original's (0x56A524): its marks and points the
    /// entrance's, its tile the exit's, its second cost the entrance's plus 100 for each tile of the
    /// list past the first (16 for the Whirlpools, `teleport_whirlpool_key` 0x56A6A3), relaxed from
    /// the entrance in the entrance's direction at no price, with the key the entrance's plus the
    /// lowest answer, a Whirlpool's k-th exit 500 × k lower still (0x56A6AF), the guard the exit's
    /// (from mode 4, but for a two-way monolith: 0x56A65A) or the entrance's, a stop when a hero
    /// stands on it (0x56A6E3).
    // rule: CPU-PTH-006
    fn exits(
        &self,
        frame: &mut Frame,
        entrance: &SearchNode,
        list: List,
        listed: impl Iterator<Item = (Pos, bool)> + Clone,
    ) {
        let guarded = self.start.mode >= SearchMode::Plan && list != List::Members;
        let mut lowest = 0_i32;
        if guarded {
            let candidates = listed.clone().filter(|&(tile, excluded)| {
                !excluded && (self.read(tile).entrance).is_some_and(|exit| list.fits(exit.class))
            });
            for (tile, _) in candidates {
                if let Some(guard) = self.guarding(tile, entrance.guard) {
                    let answer = self.asks.fight_answer(guard);
                    if answer <= REFUSED {
                        return;
                    }
                    lowest = lowest.min(answer);
                }
            }
        }
        let count = listed.clone().count() as u16;
        let spread: u16 = if list == List::Whirlpools { 16 } else { 100 };
        let cost2 = (entrance.cost2).wrapping_add(spread.wrapping_mul(count.wrapping_sub(1)));
        let direction = entrance.marks >> DIRECTION_SHIFT & 0xF;
        let mut copy = *entrance;
        for (tile, excluded) in listed {
            let class = self.read(tile).entrance.map(|exit| exit.class);
            let taken = class.is_some_and(|class| match class {
                Class::Hero(owner) => !allied(owner, self.owner),
                class => !excluded && list.fits(class),
            });
            if !taken {
                continue;
            }
            let guard = if guarded {
                self.guarding(tile, entrance.guard).or(entrance.guard)
            } else {
                entrance.guard
            };
            if list == List::Whirlpools {
                lowest = lowest.wrapping_sub(500);
            }
            let hero = matches!(class, Some(Class::Hero(_)));
            copy.pos = tile;
            copy.cost2 = cost2;
            let key = entrance.key.wrapping_add(lowest);
            self.relax(
                frame,
                entrance,
                &mut copy,
                (direction, 0),
                (key, guard, hero),
            );
        }
    }

    /// `pathfinder_gate_exit` 0x56A730 from `node` on a Subterranean Gate: its pair (`gate_exit`
    /// 0x4CDE40; none: nothing), a copy of the node on it relaxed from the node in its direction at
    /// no price, with its key and guard, its second cost unchanged and no guard weighed there, a
    /// stop when a hero, of any team, stands on it (0x56A809). The original's pair is none too when
    /// its tile holds neither a gate nor a hero, which the engine's static pair always does.
    // rule: CPU-PTH-006
    fn gate_exit(&self, frame: &mut Frame, node: &SearchNode) {
        let direction = node.marks >> DIRECTION_SHIFT & 0xF;
        let pair = self.map.channels().gate_pair(node.pos);
        pair.into_iter().for_each(|pair| {
            let hero =
                (self.read(pair).entrance).is_some_and(|exit| matches!(exit.class, Class::Hero(_)));
            let mut copy = SearchNode { pos: pair, ..*node };
            self.relax(
                frame,
                node,
                &mut copy,
                (direction, 0),
                (node.key, node.guard, hero),
            );
        });
    }
}
