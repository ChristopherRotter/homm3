//! Movement (E08): a day's movement points (T6b; the hero's bonuses since T40); the army's native
//! terrain, the price of a step and the points left after it (T7b, survey
//! `E08-adventure-core.md` §1.2–1.3; the hero's Pathfinding, Nomads and Cursed Ground since T40,
//! survey `E08-hero-movement.md` §2.1). `MoveHero` is in `command.rs`, the reachable-tiles query
//! in `reach.rs`.

use alloc::sync::Arc;
use alloc::vec::Vec;

use h3_content::{
    DIAGONAL_CHECK_IGNORES_NATIVE, DIAGONAL_CHECK_IGNORES_ROADS, NATIVE_SKIPS_LEADING_NEUTRALS,
    Ruleset, TerrainId,
};
use h3_contracts::{ArmyStack, HeroId};
use h3_heroes::{HeroDetails, Movement, land_points, movement_speed, scouting_radius};

use crate::ruleset::{AdventureRuleset, Passage, RoadId};
use crate::setup::Pos;
use crate::state::StaticTile;

/// A hero's details in the game's store of heroes by [`HeroId`] (ADR-0016 point 3), if the store
/// holds them. An adventure played without the game (this crate's own tests) passes an empty
/// store: its heroes count without skills, artifacts or a specialty.
pub(crate) fn details_of(heroes: &[Arc<HeroDetails>], hero: HeroId) -> Option<&HeroDetails> {
    heroes
        .get(usize::from(hero.raw()))
        .map(|details| &**details)
}

/// The land table's entry for a hero's army (ADV-MOV-001, E08 T40): the entry for the lowest
/// speed among its creatures as the hero counts them (`h3_heroes::movement_speed`, HER-ADV-004: a
/// creature specialist's creature and its upgrades one faster; the creature table's speed
/// without `details`). The original (0x4E4990) starts from speed 20 and takes the slowest of the
/// seven slots; the table's last entry stands for every faster speed, one a specialty lifts past
/// the table's end included. An army without stacks, or a creature the core lacks, counts as
/// fastest; a validated army has neither.
// rule: ADV-MOV-001
pub fn land_table_points(
    core: &Ruleset,
    part: &AdventureRuleset,
    details: Option<&HeroDetails>,
    army: &[ArmyStack],
) -> u32 {
    let speed = |stack: &ArmyStack| match details {
        Some(details) => movement_speed(details, core, stack.creature),
        None => core.creature(stack.creature).map_or(u8::MAX, |c| c.speed),
    };
    let slowest = army.iter().map(speed).min().unwrap_or(u8::MAX);
    part.params().land_points(slowest)
}

/// A hero's day's movement points on land (ADV-MOV-001, E08 T40): [`land_table_points`], then
/// the hero's bonuses on it (`h3_heroes::land_points`, HER-ADV-003: Logistics, a Logistics
/// specialist's growth under Q-0005, the worn Boots of Speed and Equestrian's Gloves). Without
/// `details` the table's entry alone. Stables, the sea and a computer player's bonus come later
/// (survey `E08-hero-movement.md` §6).
// rule: ADV-MOV-001
pub fn day_points(
    core: &Ruleset,
    part: &AdventureRuleset,
    details: Option<&HeroDetails>,
    army: &[ArmyStack],
) -> u32 {
    let base = land_table_points(core, part, details, army);
    details.map_or(base, |details| land_points(details, core, base))
}

/// A hero's scouting radius (ADV-FOG-001, E08 T40): `h3_heroes::scouting_radius` (HER-ADV-006:
/// Scouting, the worn Speculum and Spyglass) on the part's base; the base without `details`.
// rule: ADV-FOG-001
pub(crate) fn hero_radius(
    core: &Ruleset,
    part: &AdventureRuleset,
    details: Option<&HeroDetails>,
) -> u8 {
    let base = part.params().scouting_radius();
    details.map_or(base, |details| scouting_radius(details, core, base))
}

/// A hero's levels for its steps (E08 T40): `h3_heroes::movement` (HER-ADV-005, HER-ADV-007);
/// none without `details`.
pub(crate) fn hero_movement(core: &Ruleset, details: Option<&HeroDetails>) -> Movement {
    details.map_or_else(Movement::default, |details| {
        h3_heroes::movement(details, core)
    })
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

/// ⌊√2 · 2⁶⁴⌋, the fixed-point √2 of [`diagonal_cost`].
const SQRT_2_Q64: u128 = 0x1_6A09_E667_F3BC_C908;

/// The cost of a diagonal step whose straight cost is `straight` (ADV-MOV-005): ⌊straight × √2⌋,
/// in integers `isqrt(2 · straight²)` (the original multiplies by the f32 1.41421354 and
/// truncates, which agrees for every cost below 5741; costs here are at most `u16::MAX`). Computed
/// as ⌊straight · ⌊√2 · 2⁶⁴⌋ / 2⁶⁴⌋, one multiplication, which equals `isqrt(2 · straight²)`
/// for every cost up to `u16::MAX` (tested exhaustively; E08 T13b: the hot path of the
/// reachable-tiles query no longer depends on the compiler inlining `isqrt`).
// rule: ADV-MOV-005
pub fn diagonal_cost(straight: u32) -> u32 {
    let straight = u128::from(straight.min(u32::from(u16::MAX)));
    u32::try_from((straight * SQRT_2_Q64) >> 64).unwrap_or(u32::MAX)
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

/// What a step from a terrain costs one hero's army ([`Pricing`], E08 T40), precomputed per
/// terrain when the pricing is built.
#[derive(Debug, Clone, Copy)]
struct TerrainPrice {
    /// The walking price of a step from the terrain (ADV-MOV-003): the price at the hero's
    /// Pathfinding level of the terrain it counts as for the army (a terrain walker's, Nomads'
    /// sand as dirt); the dearest a cost can be for one no hero stands on.
    walk: u32,
    /// Whether the terrain, as it counts for the army, is the army's native terrain
    /// (ADV-MOV-004).
    native: bool,
}

impl TerrainPrice {
    /// The price of a terrain the part lacks (a tile off the map).
    const UNKNOWN: Self = Self {
        walk: 0xFFFF,
        native: false,
    };
}

/// What prices one hero's steps (ADV-MOV-003 to ADV-MOV-006; the original's `step_cost`
/// 0x4B1740 with Pathfinding, Nomads and Cursed Ground since E08 T40, without flying and water
/// walking, which come with T41): the adventure rules, each terrain's walking price for the hero
/// and its army, the native cost and the profile's quirks of the diagonal exception. Built once
/// per search or move, so the step's price is one load per terrain.
#[derive(Debug, Clone)]
pub(crate) struct Pricing<'a> {
    part: &'a AdventureRuleset,
    /// By core terrain.
    prices: Vec<TerrainPrice>,
    native_cost: u32,
    /// `ADV-QUIRK-2`: the exception's check ignores roads.
    check_ignores_roads: bool,
    /// `ADV-QUIRK-3`: the exception's check ignores the native cost.
    check_ignores_native: bool,
}

impl<'a> Pricing<'a> {
    /// The pricing of the steps of a hero with these levels (`h3_heroes::movement`) and army
    /// under `core`'s profile: each terrain counts as a terrain walker of a creature in the army
    /// makes it count (ADV-MOV-003, Nomads), then costs the step's price of that terrain at the
    /// hero's Pathfinding level (the part's [`AdventureRuleset::step_cost`]), and is native when
    /// that terrain is the army's ([`native_terrain`]).
    // rule: ADV-MOV-003, ADV-MOV-004
    pub(crate) fn new(
        core: &Ruleset,
        part: &'a AdventureRuleset,
        army: &[ArmyStack],
        movement: &Movement,
    ) -> Self {
        let quirks = core.quirks();
        let native = native_terrain(core, army);
        let counts_as = |terrain: TerrainId| {
            part.terrain_walkers()
                .iter()
                .find(|w| w.terrain == terrain && army.iter().any(|s| s.creature == w.creature))
                .map_or(terrain, |walker| walker.counts_as)
        };
        let prices = (0..=u16::MAX)
            .take(part.terrains().len())
            .map(|raw| {
                let terrain = counts_as(TerrainId::from_raw(raw));
                TerrainPrice {
                    walk: part
                        .step_cost(terrain, movement.pathfinding)
                        .map_or(TerrainPrice::UNKNOWN.walk, |cost| u32::from(cost.get())),
                    native: native == Some(terrain),
                }
            })
            .collect();
        Self {
            part,
            prices,
            native_cost: u32::from(part.params().native_cost().get()),
            check_ignores_roads: quirks.is_enabled(DIAGONAL_CHECK_IGNORES_ROADS),
            check_ignores_native: quirks.is_enabled(DIAGONAL_CHECK_IGNORES_NATIVE),
        }
    }

    /// The army's price of a step from a terrain.
    fn price(&self, terrain: TerrainId) -> TerrainPrice {
        self.prices
            .get(usize::from(terrain.raw()))
            .copied()
            .unwrap_or(TerrainPrice::UNKNOWN)
    }

    /// The straight cost of a step along a road (ADV-MOV-003).
    fn road_cost(&self, road: RoadId) -> Option<u32> {
        self.part.road(road).map(|kind| u32::from(kind.cost.get()))
    }

    /// The road's cost of a step from `from` to `to`, when both tiles have a road: the source's
    /// road prices it (ADV-MOV-003, D-2).
    fn road(&self, from: &StaticTile, to: &StaticTile) -> Option<u32> {
        from.road
            .filter(|_| to.road.is_some())
            .and_then(|road| self.road_cost(road))
    }

    /// A step's cost before the diagonal, from the source tile alone: `road` when given, else the
    /// native cost when `native`, the source's terrain counts as the army's and no ground on the
    /// source takes the native cost away (Cursed Ground, ADV-MOV-004), else the source terrain's
    /// walking price.
    fn base(&self, from: &StaticTile, road: Option<u32>, native: bool) -> u32 {
        road.unwrap_or_else(|| {
            let price = self.price(from.terrain);
            if native && price.native && !from.no_native {
                self.native_cost
            } else {
                price.walk
            }
        })
    }

    /// The cheapest straight step the hero makes (every step costs at least this): the least of
    /// the native cost, the roads' costs and the land terrains' walking prices.
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
            .zip(&self.prices)
            .filter(|(terrain, _)| matches!(terrain.passage, Passage::Land(_)))
            .map(|(_, price)| price.walk);
        roads.chain(lands).fold(self.native_cost, u32::min)
    }

    /// The price of a step from `from` to `to` with `points` left (ADV-MOV-003 to ADV-MOV-006).
    /// Straight: the road's cost when both tiles have a road, else the native cost when the
    /// source's terrain counts as the army's and is not under Cursed Ground, else the source
    /// terrain's walking price (by the hero's Pathfinding level, Nomads' sand as dirt): only the
    /// source tile prices a step (D-2). Diagonal: ⌊straight × √2⌋, except that with points from a
    /// check cost C up to ⌊C × √2⌋ it costs only the straight cost (the diagonal exception). C is
    /// the step's own straight cost; under `ADV-QUIRK-2` it leaves out the road, under
    /// `ADV-QUIRK-3` the native cost (the original compares with the source terrain's walking
    /// price, 0x4B1740). A price above `points` means the step cannot be made.
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
    /// none when fewer remain than the cheapest straight step from `to`, priced against itself
    /// with its own terrain as native: its road's cost, else the native cost, or under Cursed
    /// Ground the terrain's walking price (E08 T40: by the hero's Pathfinding level, after
    /// Nomads). The original's `cheapest_step_from` 0x4B1880 after each step of a move, in
    /// 0x4805E0 (the walking case: flying and water walking with T41, `ADV-QUIRK-17`).
    // rule: ADV-MOV-006
    pub(crate) fn left_after(&self, to: &StaticTile, left: u32) -> u32 {
        let cheapest = to
            .road
            .and_then(|road| self.road_cost(road))
            .unwrap_or_else(|| {
                if to.no_native {
                    self.price(to.terrain).walk
                } else {
                    self.native_cost
                }
            });
        if left < cheapest { 0 } else { left }
    }
}

#[cfg(test)]
mod tests;
