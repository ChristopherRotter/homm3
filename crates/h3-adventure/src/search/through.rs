//! The passage of the computer's search past a stop (CPU-PTH-005): `pathfinder_pass_through`
//! 0x56AB40, by the class of the stop's tile.

use super::marks::{BOAT, FLY, WATER_WALK};
use super::{Class, Frame, Search, SearchAsks, SearchMode, SearchNode};
use crate::state::allied;

impl<A: SearchAsks> Search<'_, A> {
    /// Whether the search goes on from `node`, a stop popped from `cell` (`pathfinder_pass_through`
    /// 0x56AB40, from `pathfinder_run` 0x56BB7C), by its tile's class (the modes are 1 to 5: mode
    /// 0's test before the switch, 0x56AB61, is the human's):
    ///
    /// - 8, an empty boat (`pathfinder_boat_case` 0x56AD01): from mode 4 and not already in a
    ///   boat, the boarding: the node's points, the day's rest, added to its cost, its points the
    ///   sea's maximum, its flying and water walking cleared, it and its stored cell in a boat
    ///   (0x56AD6F; the cell keeps its cost and points);
    /// - 9, a border guard (0x56ABA1): from mode 4, as 212;
    /// - 212, a border gate (`pathfinder_key_case` 0x56ABB1): the current player holds its key,
    ///   in every mode (not the hero owner's: a mode-3 search of an enemy hero tests the
    ///   computer's keys);
    /// - 33, a garrison (`pathfinder_garrison_case` 0x56ACAE): one of the hero's team, in every
    ///   mode; any other from mode 4;
    /// - 34, another hero (`pathfinder_hero_case` 0x56AC5F): one of another team, from mode 3;
    /// - 54, a monster (`pathfinder_monster_case` 0x56ACF2): from mode 4;
    /// - 98, a town (`pathfinder_town_case` 0x56AEDB): from mode 3 unless a guard stops the node
    ///   (`pathfinder_entrance_guarded` 0x56A360), whatever its owner or the node's cost;
    /// - 43, 45, 111 and 103, the teleporters (0x56AE14, 0x56AE59, 0x56AE9F,
    ///   `pathfinder_gate_case` 0x56ADCB): no, from mode 3 after their exits are relaxed
    ///   (CPU-PTH-006);
    /// - any other (`pathfinder_not_entered_default` 0x56AF27): a class entered, not visited from
    ///   the neighbouring tile (the settings' byte `+0` whole, 0x56AF37), a one-way exit (44)
    ///   among them.
    // rule: CPU-PTH-005, CPU-PTH-006
    pub(super) fn pass_through(
        &self,
        frame: &mut Frame,
        cell: usize,
        node: &mut SearchNode,
    ) -> bool {
        let mode = self.start.mode;
        let entrance = self.read(node.pos).entrance;
        // The Quest Guard's case 215 (`pathfinder_quest_guard_case` 0x56ABD3): E12.
        // stub: AI-STUB-23
        match entrance.map_or(Class::Other, |entrance| entrance.class) {
            Class::Boat => {
                if node.marks & BOAT != 0 || mode < SearchMode::Plan {
                    return false;
                }
                node.cost = node.cost.wrapping_add(node.points);
                node.points = self.sea as u16;
                node.marks = node.marks & !(FLY | WATER_WALK) | BOAT;
                frame.cells[cell].marks |= BOAT;
                true
            }
            Class::Guard(key) => {
                mode >= SearchMode::Plan && self.state.has_key(self.start.player, key)
            }
            Class::Gate(key) => self.state.has_key(self.start.player, key),
            Class::Garrison(owner) => {
                owner.is_some_and(|owner| allied(owner, self.owner)) || mode >= SearchMode::Plan
            }
            Class::Hero(owner) => !allied(owner, self.owner) && mode >= SearchMode::Threat,
            Class::Monster => mode >= SearchMode::Plan,
            // Then the Castle Gate from the town (`pathfinder_castle_gate` 0x56A850, at
            // 0x56AF17), and yes.
            // stub: AI-STUB-22
            Class::Town => mode >= SearchMode::Threat && !self.entrance_guarded(node),
            Class::Teleporter(teleporter) => {
                if mode >= SearchMode::Threat {
                    self.teleport(frame, node, teleporter);
                }
                false
            }
            Class::Exit | Class::Coast | Class::Other => {
                !entrance.is_some_and(|entrance| entrance.not_entered)
            }
        }
    }
}
