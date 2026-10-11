//! Movement (E08): a day's movement points (T6b); the army's native terrain, the price of a step
//! and the points left after it (T7b, survey `E08-adventure-core.md` §1.2–1.3). `MoveHero` is in
//! `command.rs`, the reachable-tiles query in `reach.rs`.

use h3_content::{
    DIAGONAL_CHECK_IGNORES_NATIVE, DIAGONAL_CHECK_IGNORES_ROADS, NATIVE_SKIPS_LEADING_NEUTRALS,
    Ruleset, TerrainId,
};
use h3_contracts::ArmyStack;

use crate::ruleset::{AdventureRuleset, Passage, RoadId};
use crate::setup::Pos;
use crate::state::StaticTile;

/// A day's movement points on land for an army (ADV-MOV-001): the land table's entry for the
/// lowest speed among its creatures (the original, 0x4E4990, starts from speed 20 and takes the
/// slowest of the seven slots; the table's last entry stands for every faster speed). An army
/// without stacks, or a creature the core lacks, counts as fastest; a validated army has neither.
/// Bonuses (Logistics, artifacts, a specialty's speed) come with the adventure profile.
// rule: ADV-MOV-001
pub fn day_points(core: &Ruleset, part: &AdventureRuleset, army: &[ArmyStack]) -> u32 {
    let slowest = army
        .iter()
        .map(|stack| core.creature(stack.creature).map_or(u8::MAX, |c| c.speed))
        .min()
        .unwrap_or(u8::MAX);
    part.params().land_points(slowest)
}

/// The terrain an army is native to (ADV-MOV-004): the native terrain of its creatures' factions
/// when every stack has the same one, else none (a neutral creature, or a faction without one,
/// has none). With `ADV-QUIRK-4` on (`quirk.adventure.native_skips_leading_neutrals`) the
/// original's walk (0x44C590): the stacks in slot order, where stacks without a native terrain
/// before the first one that has it are skipped. `army` is in slot order, as a hero's is.
// rule: ADV-MOV-004
pub fn native_terrain(core: &Ruleset, army: &[ArmyStack]) -> Option<TerrainId> {
    let mut natives = army.iter().map(|stack| {
        core.creature_info(stack.creature)
            .and_then(|info| info.faction)
            .and_then(|faction| core.factions().get(faction.index()))
            .and_then(|faction| faction.native_terrain)
    });
    let first = if core.quirks().is_enabled(NATIVE_SKIPS_LEADING_NEUTRALS) {
        natives.find(Option::is_some)
    } else {
        natives.next()
    };
    let first = first.flatten()?;
    natives.all(|native| native == Some(first)).then_some(first)
}

/// The cost of a diagonal step whose straight cost is `straight` (ADV-MOV-005): ⌊straight × √2⌋,
/// in integers `isqrt(2 · straight²)` (the original multiplies by the f32 1.41421354 and
/// truncates, which agrees for every cost below 5741; costs here are at most `u16::MAX`).
// rule: ADV-MOV-005
pub fn diagonal_cost(straight: u32) -> u32 {
    let straight = u64::from(straight.min(u32::from(u16::MAX)));
    u32::try_from((2 * straight * straight).isqrt()).unwrap_or(u32::MAX)
}

/// The eight steps as `(columns, rows)`, clockwise from up, in the original's order (0x678150);
/// the odd ones are diagonal.
pub(crate) const DIRECTIONS: [(i8, i8); 8] = [
    (0, -1),
    (1, -1),
    (1, 0),
    (1, 1),
    (0, 1),
    (-1, 1),
    (-1, 0),
    (-1, -1),
];

/// The direction of a step from `from` to `to` (ADV-MOV-007), an index into [`DIRECTIONS`]: `to`
/// is one of the eight tiles around `from` on its level.
// rule: ADV-MOV-007
pub(crate) fn direction(from: Pos, to: Pos) -> Option<usize> {
    let columns = i32::from(to.x) - i32::from(from.x);
    let rows = i32::from(to.y) - i32::from(from.y);
    DIRECTIONS
        .iter()
        .position(|&(x, y)| i32::from(x) == columns && i32::from(y) == rows)
        .filter(|_| from.level == to.level)
}

/// What prices one hero's steps (ADV-MOV-003 to ADV-MOV-006; the original's 0x4B1740 without
/// Pathfinding, Nomads, flying, water walking and Cursed Ground, which come with later
/// increments): the adventure rules, the army's native terrain and the profile's quirks of the
/// diagonal exception.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Pricing<'a> {
    part: &'a AdventureRuleset,
    native: Option<TerrainId>,
    /// `ADV-QUIRK-2`: the exception's check ignores roads.
    check_ignores_roads: bool,
    /// `ADV-QUIRK-3`: the exception's check ignores the native cost.
    check_ignores_native: bool,
}

impl<'a> Pricing<'a> {
    /// The pricing of an army's steps under `core`'s profile.
    pub(crate) fn new(core: &Ruleset, part: &'a AdventureRuleset, army: &[ArmyStack]) -> Self {
        let quirks = core.quirks();
        Self {
            part,
            native: native_terrain(core, army),
            check_ignores_roads: quirks.is_enabled(DIAGONAL_CHECK_IGNORES_ROADS),
            check_ignores_native: quirks.is_enabled(DIAGONAL_CHECK_IGNORES_NATIVE),
        }
    }

    /// The straight cost of a step on a terrain (ADV-MOV-003); one no hero stands on (rock, or a
    /// terrain the part lacks) as the dearest a cost can be.
    fn terrain_cost(&self, terrain: TerrainId) -> u32 {
        match self.part.passage(terrain) {
            Some(Passage::Land(cost) | Passage::Water(cost)) => u32::from(cost.get()),
            Some(Passage::Impassable) | None => u32::from(u16::MAX),
        }
    }

    /// The straight cost of a step along a road (ADV-MOV-003).
    fn road_cost(&self, road: RoadId) -> Option<u32> {
        self.part.road(road).map(|kind| u32::from(kind.cost.get()))
    }

    /// The native cost (ADV-MOV-004).
    fn native_cost(&self) -> u32 {
        u32::from(self.part.params().native_cost().get())
    }

    /// The road's cost of a step from `from` to `to`, when both tiles have a road: the source's
    /// road prices it (ADV-MOV-003, D-2).
    fn road(&self, from: &StaticTile, to: &StaticTile) -> Option<u32> {
        from.road
            .filter(|_| to.road.is_some())
            .and_then(|road| self.road_cost(road))
    }

    /// A step's cost before the diagonal, from the source tile alone: `road` when given, else the
    /// native cost when `native` and the source's terrain is the army's, else the source terrain's.
    fn base(&self, from: &StaticTile, road: Option<u32>, native: bool) -> u32 {
        road.unwrap_or_else(|| {
            if native && self.native == Some(from.terrain) {
                self.native_cost()
            } else {
                self.terrain_cost(from.terrain)
            }
        })
    }

    /// The cheapest straight step any army makes (every step costs at least this): the least of
    /// the native cost, the roads' and the land terrains' costs.
    pub(crate) fn cheapest(&self) -> u32 {
        let roads = self
            .part
            .roads()
            .iter()
            .map(|road| u32::from(road.cost.get()));
        let lands = self
            .part
            .terrains()
            .iter()
            .map(|terrain| match terrain.passage {
                Passage::Land(cost) => u32::from(cost.get()),
                Passage::Water(_) | Passage::Impassable => u32::MAX,
            });
        roads.chain(lands).fold(self.native_cost(), u32::min)
    }

    /// The price of a step from `from` to `to` with `points` left (ADV-MOV-003 to ADV-MOV-006).
    /// Straight: the road's cost when both tiles have a road, else the native cost when the
    /// source's terrain is the army's, else the source terrain's: only the source tile prices a
    /// step (D-2). Diagonal: ⌊straight × √2⌋, except that with points from a check cost C up to
    /// ⌊C × √2⌋ it costs only the straight cost (the diagonal exception). C is the step's own
    /// straight cost; under `ADV-QUIRK-2` it leaves out the road, under `ADV-QUIRK-3` the native
    /// cost (the original compares with the source terrain's plain cost, 0x4B1740). A price above
    /// `points` means the step cannot be made.
    // rule: ADV-MOV-003, ADV-MOV-004, ADV-MOV-005, ADV-MOV-006
    pub(crate) fn step(
        &self,
        from: &StaticTile,
        to: &StaticTile,
        diagonal: bool,
        points: u32,
    ) -> u32 {
        let road = self.road(from, to);
        let straight = self.base(from, road, true);
        if !diagonal {
            return straight;
        }
        let check = self.base(
            from,
            road.filter(|_| !self.check_ignores_roads),
            !self.check_ignores_native,
        );
        if (check..=diagonal_cost(check)).contains(&points) {
            straight
        } else {
            diagonal_cost(straight)
        }
    }

    /// The points a hero has left on `to` after a step that left it `left` (ADV-MOV-006, D-3):
    /// none when fewer remain than the cheapest straight step from `to`, its road's cost or else
    /// the native cost (the original's 0x4B1880 after each step of a move, in 0x4805E0).
    // rule: ADV-MOV-006
    pub(crate) fn left_after(&self, to: &StaticTile, left: u32) -> u32 {
        let cheapest = to
            .road
            .and_then(|road| self.road_cost(road))
            .unwrap_or_else(|| self.native_cost());
        if left < cheapest { 0 } else { left }
    }
}

#[cfg(test)]
mod tests;
