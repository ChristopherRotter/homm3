//! The step of the computer's search (CPU-PTH-003): `pathfinder_expand` 0x4B2300, a node's eight
//! steps with their prices, the node's points and the new day, the tile cases and the fog.

use super::marks::{BOAT, DOOR, FLY, PLACE, WATER_WALK};
use super::{Class, Frame, Ground, Search, SearchAsks, SearchFog, SearchMode, SearchNode};
use crate::movement::DIRECTIONS;
use crate::setup::Pos;

impl<A: SearchAsks> Search<'_, A> {
    /// The tile one step `dx`, `dy` from `at`, if it lies on the map (0x4B23C3..0x4B23FB).
    fn neighbour(&self, at: Pos, dx: i8, dy: i8) -> Option<Pos> {
        let x =
            at.x.checked_add_signed(i16::from(dx))
                .filter(|&x| x < self.map.width())?;
        let y =
            at.y.checked_add_signed(i16::from(dy))
                .filter(|&y| y < self.map.height())?;
        Some(Pos::new(x, y, at.level))
    }

    /// Whether the tile at column `x`, row `y` of `level` is water (a boat's diagonal's side
    /// tiles, 0x4B2726..0x4B27F3).
    fn wet(&self, x: u16, y: u16, level: u8) -> bool {
        self.ground_at(Pos::new(x, y, level)) == Ground::Water
    }

    /// Expands `source`, a node popped with `left` of the day's points after its cost, in the
    /// eight directions (`pathfinder_expand` 0x4B2300); a node a monster guards (below mode 4,
    /// `guarded`) steps only onto its guard's tile (0x4B2401). Each step's node is the source's
    /// copy on the new tile, a place (`pathfinder_step_stop_place` 0x4B2453), priced:
    ///
    /// - onto rock nothing, its second cost 100 more and no place, a Door leg's only (0x4B248B);
    ///   from a boat node a boat's step ([`crate::movement`]'s `sail`: `step_cost` 0x4B1740
    ///   without a road, a native terrain, flying or water walking, 0x4B24B7); else the step at
    ///   the node's flying and water walking (0x4B24C9); the diagonal window is `left`'s, none
    ///   when negative (0x4B2500);
    /// - the node's points pay the step (0x4B2512); else, from a place only, the step starts a
    ///   new day (`pathfinder_new_day` 0x4B252A): the node's points the day's whole maximum, at
    ///   sea from a boat node, the step priced again without flying or water walking (0x4B257F),
    ///   which the node loses (0x4B2591). The relax takes the bare price, the day's rest not
    ///   counted.
    ///
    /// Then the tile: blocked, only flown over without a place (0x4B2596); unexplored, outside
    /// mode 3, by [`SearchFog`] (`pathfinder_fog_test` 0x4B25D4); up off the source's entrance
    /// of an object not visited from above, only flown over, its place kept
    /// (`pathfinder_up_off_entrance` 0x4B2693); down onto such an entrance, never (0x4B26C0).
    /// Water from a boat node: onto an empty boat a jump's only (0x4B26EB), a diagonal with a side
    /// tile not water a jump's only, its place kept (0x4B2703). Water from a node not in a boat
    /// (`pathfinder_onto_water` 0x4B27FB): another hero's, an empty boat's and a coast bank's
    /// entrances a step as on land, any other entrance only flown over, plain water a step at
    /// sea only, both without a place. Land from a boat node: but onto a free coastal tile (the
    /// original's class 3), a jump's only (0x4B2882); onto one a step kept in the boat, a leaf.
    /// From mode 4 a step onto another hero, a garrison or a monster is weighed
    /// (`pathfinder_target_weigh_call` 0x4B2932) and refused one only flown over; then any other
    /// step's guard (`pathfinder_guard_call` 0x4B2A51), a refused one losing its place and only
    /// flown over. A jump's step, one only flown over that does not fly and one at sea that
    /// neither flies nor water-walks go no further. The rest is relaxed with the tile's entrance
    /// as its stop, but from a pass node onto the entrance of a class visited from the
    /// neighbouring tile or of a town (`pathfinder_stop_relax` 0x4B2CE5); a flying step onto an
    /// entrance as a place is relaxed again as a pass node, no stop (`pathfinder_flying_copy`
    /// 0x4B2D3D).
    // rule: CPU-PTH-003
    pub(super) fn expand(
        &self,
        frame: &mut Frame,
        source: &SearchNode,
        left: i32,
        guarded: Option<Pos>,
    ) {
        let (state, part, mode) = (self.state, self.part, self.start.mode);
        let at = source.pos;
        let from = state.live(at, self.map.at(at));
        let closed = state.closed_above(part, at);
        let window = u32::try_from(left).unwrap_or(0);
        for (direction, &(dx, dy)) in DIRECTIONS.iter().enumerate() {
            let Some(pos) = self.neighbour(at, dx, dy) else {
                continue;
            };
            if guarded.is_some_and(|guard| guard != pos) {
                continue;
            }
            let diagonal = direction % 2 == 1;
            let mut node = SearchNode {
                pos,
                marks: source.marks | PLACE,
                ..*source
            };
            // A Door leg's node from a place loses its leg (0x4B247C): CPU-PTH-007.
            let tile = self.read(pos);
            let ground = self.ground(&tile.live);
            let (mut fly_only, mut jump_only, mut sea) = (false, false, false);
            // A Door leg's step costs nothing (0x4B24A6): CPU-PTH-007.
            let mut price = if ground == Ground::Rock {
                node.marks &= !PLACE;
                node.cost2 = node.cost2.wrapping_add(100);
                jump_only = true;
                0
            } else if source.marks & BOAT != 0 {
                self.pricings[0].sail(&from, diagonal, window)
            } else {
                self.pricing(node.marks)
                    .step(&from, &tile.live, diagonal, window)
            };
            if price <= u32::from(source.points) {
                node.points = source.points - price as u16;
            } else {
                if source.marks & PLACE == 0 {
                    continue;
                }
                node.points = (if source.marks & BOAT != 0 {
                    self.sea
                } else {
                    self.land
                }) as u16;
                if node.marks & (FLY | WATER_WALK) != 0 {
                    price = self.pricings[0].step(&from, &tile.live, diagonal, window);
                }
                node.marks &= !(FLY | WATER_WALK);
            }
            if tile.blocked {
                fly_only = true;
                node.marks &= !PLACE;
            }
            if mode != SearchMode::Threat
                && !self.explored(pos)
                && match self.start.fog {
                    SearchFog::Human => true,
                    SearchFog::Computer => !self.explored(at) && self.towns,
                }
            {
                fly_only = true;
                node.marks &= !PLACE;
            }
            // Mode 3 onto a Sanctuary's entrance, only flown over (0x4B263F): E12.
            // stub: AI-STUB-23
            if dy < 0 && closed {
                fly_only = true;
            }
            if dy > 0 && state.closed_above(part, pos) {
                continue;
            }
            match ground {
                Ground::Water if source.marks & BOAT != 0 => {
                    if tile
                        .entrance
                        .is_some_and(|entrance| entrance.class == Class::Boat)
                    {
                        jump_only = true;
                        node.marks &= !PLACE;
                    }
                    // The source, a boat node the run expands, lies on water (0x4B2703).
                    if diagonal
                        && !(self.wet(pos.x, at.y, at.level) && self.wet(at.x, pos.y, at.level))
                    {
                        jump_only = true;
                    }
                }
                Ground::Water => match tile.entrance.map(|entrance| entrance.class) {
                    Some(Class::Hero(_) | Class::Boat | Class::Coast) => {}
                    Some(_) => {
                        fly_only = true;
                        node.marks &= !PLACE;
                    }
                    None => {
                        sea = true;
                        node.marks &= !PLACE;
                    }
                },
                // From mode 4 a landing onto a free coastal tile (0x4B2890): CPU-PTH-007.
                Ground::Land
                    if source.marks & BOAT != 0
                        && !(tile.entrance.is_none() && state.free_coast(pos, &tile.live)) =>
                {
                    jump_only = true;
                    node.marks &= !PLACE;
                }
                _ => {}
            }
            // Another player's hero on a Sanctuary, only flown over (0x4B2842): E12.
            // stub: AI-STUB-23
            let entrance = tile.entrance;
            if mode >= SearchMode::Plan
                && entrance.is_some_and(|entrance| {
                    matches!(
                        entrance.class,
                        Class::Hero(_) | Class::Garrison(_) | Class::Monster
                    )
                })
                && !self.target_weigh(&mut node)
            {
                fly_only = true;
                node.marks &= !PLACE;
            }
            // A Door leg from here (`pathfinder_door_conditions` 0x4B2953): CPU-PTH-007.
            if jump_only {
                continue;
            }
            // The Fly stretch and a Door leg (0x4B29B7, `pathfinder_fly_stretch` 0x4B2A73):
            // CPU-PTH-007.
            if fly_only {
                if node.marks & (FLY | DOOR) == 0 {
                    continue;
                }
            } else if mode >= SearchMode::Plan && self.entrance_guarded(&mut node) {
                node.marks &= !PLACE;
                if node.marks & (FLY | DOOR) == 0 {
                    continue;
                }
            }
            // The boat node and the sea spells (`pathfinder_sea_step` 0x4B2AC1): CPU-PTH-007.
            if sea && node.marks & (FLY | WATER_WALK | DOOR) == 0 {
                continue;
            }
            let stop = entrance.is_some();
            let (key, guard) = (node.key, node.guard);
            let step = (direction as u32, price);
            if source.marks & PLACE != 0
                || !entrance
                    .is_some_and(|entrance| entrance.not_entered || entrance.class == Class::Town)
            {
                self.relax(frame, source, &mut node, step, (key, guard, stop));
            }
            if node.marks & (FLY | DOOR) != 0 && stop && node.marks & PLACE != 0 {
                node.marks &= !PLACE;
                self.relax(frame, source, &mut node, step, (key, guard, false));
            }
        }
    }
}
