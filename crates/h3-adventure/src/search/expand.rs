//! The step of the computer's search (CPU-PTH-003): `pathfinder_expand` 0x4B2300, a node's eight
//! steps with their prices, the node's points and the new day, the tile cases and the fog.

use super::jumps::may_cast;
use super::marks::{BOAT, DOOR, FLY, PLACE, WATER_WALK};
use super::{
    Class, Frame, Ground, Search, SearchAsks, SearchFog, SearchJumps, SearchMode, SearchNode,
};
use crate::movement::DIRECTIONS;
use crate::setup::Pos;

impl<A: SearchAsks> Search<'_, A> {
    /// The tile one step `dx`, `dy` from `at`, a tile of the map, if it lies on the map
    /// (0x4B23C3..0x4B23FB): a step off the left or top edge wraps past the right or bottom.
    fn neighbour(&self, at: Pos, dx: i8, dy: i8) -> Option<Pos> {
        let x = at.x.wrapping_add_signed(i16::from(dx));
        let y = at.y.wrapping_add_signed(i16::from(dy));
        (x < self.map.width() && y < self.map.height()).then_some(Pos::new(x, y, at.level))
    }

    /// Whether the tile at column `x`, row `y` of `level` is water (a boat's diagonal's side
    /// tiles, 0x4B2726..0x4B27F3).
    fn wet(&self, x: u16, y: u16, level: u8) -> bool {
        self.ground_at(Pos::new(x, y, level)) == Ground::Water
    }

    /// Expands `source`, a node popped with `left` of the day's points after its cost, in the
    /// eight directions (`pathfinder_expand` 0x4B2300); a node a monster guards (below mode 4,
    /// `guarded`) steps only onto its guard's tile (0x4B2401). Each step's node is the source's
    /// copy on the new tile, a place (`pathfinder_step_stop_place` 0x4B2453), no Door leg when
    /// the source is a place, its leg landed (0x4B247C), priced:
    ///
    /// - onto rock nothing, its second cost 100 more and no place, a Door leg's only (0x4B248B);
    ///   a leg's step nothing, its second cost 100 more (0x4B24A6); from a boat node a boat's
    ///   step ([`crate::movement`]'s `sail`: `step_cost` 0x4B1740 without a road, a native
    ///   terrain, flying or water walking, 0x4B24B7); else the step at the node's flying and
    ///   water walking (0x4B24C9); the diagonal window is `left`'s, none when negative
    ///   (0x4B2500);
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
    /// sea only, both without a place; from a leg's node a jump's only (0x4B2830). Land from a
    /// boat node: from a leg or but onto a free coastal tile (the original's class 3), a jump's
    /// only (0x4B2882); onto one a place below mode 4 a step kept in the boat, a leaf, from mode
    /// 4 the landing with the day's rest ([`Self::landing`], `pathfinder_landing_day` 0x4B28A6).
    /// From mode 4 a step onto another hero, a garrison or a monster is weighed
    /// (`pathfinder_target_weigh_call` 0x4B2932) and refused one only flown over; then any other
    /// step's guard (`pathfinder_guard_call` 0x4B2A51), a refused one losing its place and only
    /// flown over. Mode 4's jumps (CPU-PTH-007), each from a place without the no-magic mark: a
    /// jump's step goes on as a leg, else starts one with Dimension Door ([`Self::door_leg`],
    /// `pathfinder_door_conditions` 0x4B2953); a step only flown over goes on flying or as a leg,
    /// else starts a leg or, not from a boat, a Fly stretch ([`Self::fly_stretch`], 0x4B29B7); a
    /// step at sea goes on flying, water walking or as a leg, else from a place makes the boat
    /// node ([`Self::boat_node`], in every mode) and starts a leg or a sea spell
    /// ([`Self::sea_spell`], `pathfinder_sea_step` 0x4B2AC1); any other such step goes no
    /// further. The rest is relaxed with the tile's entrance as its stop, but from a pass node
    /// onto the entrance of a class visited from the neighbouring tile or of a town
    /// (`pathfinder_stop_relax` 0x4B2CE5); a flying or leaping step onto an entrance as a place is
    /// relaxed again as a pass node, no stop (`pathfinder_flying_copy` 0x4B2D3D).
    // rule: CPU-PTH-003, CPU-PTH-007
    pub(super) fn expand<const THREAT: bool>(
        &self,
        frame: &mut Frame,
        source: &SearchNode,
        left: i32,
        guarded: Option<Pos>,
    ) {
        let (state, mode) = (self.state, self.mode::<THREAT>());
        // Mode 3's search has no jumps.
        let cast = if THREAT {
            SearchJumps::default()
        } else {
            self.cast
        };
        let at = source.pos;
        let spots = &self.spots;
        let here = spots.at(at);
        let (from, closed) = (&spots.spots[here].tile.live, spots.spots[here].closed);
        let window = u32::try_from(left).unwrap_or(0);
        // The steps' prices by the node's flying and water walking (its steps keep the source's),
        // and the last straight and diagonal step's price with the tile it entered, kept from
        // the last node expanded when its prices, its tile's word and its window were the same:
        // a step onto a tile like it costs the same.
        let pricing = self.pricing(source.marks);
        let by = (source.marks >> 4 & 3, spots.spots[here].key, window);
        let mut priced = if frame.priced.0 == by {
            frame.priced.1
        } else {
            [(u64::MAX, 0); 2]
        };
        for (direction, &(dx, dy)) in DIRECTIONS.iter().enumerate() {
            let Some(pos) = self.neighbour(at, dx, dy) else {
                continue;
            };
            if guarded.is_some_and(|guard| guard != pos) {
                continue;
            }
            let diagonal = direction % 2 == 1;
            let mut node = *source;
            node.pos = pos;
            node.marks |= PLACE;
            // A landed leg's steps are no leg's (0x4B247C).
            if source.marks & PLACE != 0 {
                node.marks &= !DOOR;
            }
            // The source lies in the box, its neighbour on the map in the spots' region.
            let spot = &spots.spots[here.wrapping_add_signed(spots.steps[direction])];
            let (tile, ground) = (&spot.tile, spot.ground);
            let (mut fly_only, mut jump_only, mut sea) = (false, false, false);
            let mut price = if ground == Ground::Rock {
                node.marks &= !PLACE;
                node.cost2 = node.cost2.wrapping_add(100);
                jump_only = true;
                0
            } else if node.marks & DOOR != 0 {
                node.cost2 = node.cost2.wrapping_add(100);
                0
            } else if source.marks & BOAT != 0 {
                self.pricings[0].sail(from, diagonal, window)
            } else {
                let last = &mut priced[usize::from(diagonal)];
                if last.0 != spot.key {
                    *last = (spot.key, pricing.step(from, &tile.live, diagonal, window));
                }
                last.1
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
                    price = self.pricings[0].step(from, &tile.live, diagonal, window);
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
            if dy > 0 && spot.closed {
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
                Ground::Water => {
                    match tile.entrance.map(|entrance| entrance.class) {
                        Some(Class::Hero(_) | Class::Boat | Class::Coast) => {}
                        Some(_) => {
                            fly_only = true;
                            node.marks &= !PLACE;
                        }
                        None => {
                            sea = true;
                            node.marks &= !PLACE;
                        }
                    }
                    // A leg's step onto water is a jump's only (0x4B2830).
                    if source.marks & DOOR != 0 {
                        jump_only = true;
                        node.marks &= !PLACE;
                    }
                }
                Ground::Land if source.marks & BOAT != 0 => {
                    if source.marks & DOOR != 0
                        || !(tile.entrance.is_none() && state.free_coast(pos, &tile.live))
                    {
                        jump_only = true;
                        node.marks &= !PLACE;
                    } else if node.marks & PLACE != 0 && mode >= SearchMode::Plan {
                        price = self.landing(source, &mut node, price);
                    }
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
            if !fly_only
                && !jump_only
                && mode >= SearchMode::Plan
                && self.entrance_guarded(&mut node)
            {
                node.marks &= !PLACE;
                fly_only = true;
            }
            if jump_only && node.marks & DOOR == 0 {
                if !(cast.door && may_cast(source)) {
                    continue;
                }
                price = self.door_leg(&mut node);
            }
            let tiles = (from, &tile.live);
            if fly_only && node.marks & (FLY | DOOR) == 0 {
                if !((cast.fly || cast.door) && may_cast(source)) {
                    continue;
                }
                if cast.door {
                    price = self.door_leg(&mut node);
                } else if source.marks & BOAT != 0 {
                    continue;
                } else {
                    price = self.fly_stretch(&mut node, tiles, diagonal, window);
                }
            }
            if sea && node.marks & (FLY | WATER_WALK | DOOR) == 0 {
                let sail = (from, diagonal, window);
                self.boat_node(frame, (source, &node), direction as u32, sail);
                if !may_cast(source) {
                    continue;
                }
                // A step at sea comes from no boat: the boat test holds.
                if cast.door {
                    price = self.door_leg(&mut node);
                } else if (cast.fly || cast.water_walk) && source.marks & BOAT == 0 {
                    price = self.sea_spell(&mut node, tiles, diagonal, window);
                } else {
                    continue;
                }
            }
            let stop = entrance.is_some();
            let (key, guard) = (node.key, node.guard);
            let step = (direction as u32, price);
            let place = (spot.cell != u32::MAX).then_some(spot.cell as usize);
            if source.marks & PLACE != 0
                || !entrance
                    .is_some_and(|entrance| entrance.not_entered || entrance.class == Class::Town)
            {
                self.relax_in(frame, source, (&mut node, place), step, (key, guard, stop));
            }
            if node.marks & (FLY | DOOR) != 0 && stop && node.marks & PLACE != 0 {
                node.marks &= !PLACE;
                self.relax_in(frame, source, (&mut node, place), step, (key, guard, false));
            }
        }
        frame.priced = (by, priced);
    }
}
