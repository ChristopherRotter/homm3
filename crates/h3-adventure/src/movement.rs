//! Movement (E08): a day's movement points (T6b; the hero's bonuses since T40); the army's native
//! terrain, the price of a step and the points left after it (T7b, survey
//! `E08-adventure-core.md` §1.2–1.3; the hero's Pathfinding, Nomads and Cursed Ground since T40,
//! its flying and water walking since T41, survey `E08-hero-movement.md` §2; a boat's since T73,
//! survey `E08-sea.md` §4.1). `MoveHero` is in `command.rs`, the passage of a flying,
//! water-walking or sailing hero in `state/passage.rs`, the reachable-tiles query in `reach.rs`.

use alloc::sync::Arc;
use alloc::vec::Vec;

use h3_content::{
    DIAGONAL_CHECK_IGNORES_NATIVE, DIAGONAL_CHECK_IGNORES_ROADS, DROP_IGNORES_ARTIFACTS,
    NATIVE_SKIPS_LEADING_NEUTRALS, Ruleset, TerrainId,
};
use h3_contracts::{ArmyStack, HeroId};
use h3_heroes::{HeroDetails, Movement, land_points, movement_speed, scouting_radius, sea_points};

use crate::ruleset::{AdventureRuleset, Passage, RoadId};
use crate::setup::Pos;
use crate::state::{HeroOnMap, Meets, Mode, StaticTile};

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

/// A hero's day's movement points at sea (ADV-MOV-001, HER-ADV-009; E08 T72):
/// `h3_heroes::sea_points` on the part's sea table (Navigation's level, its specialists' growth,
/// the Necklace of Ocean Guidance and the two Hats; the army's speeds do not count, survey
/// `E08-sea.md` §4.2); without `details` the table's first entry.
// rule: ADV-MOV-001
pub fn sea_day_points(
    core: &Ruleset,
    part: &AdventureRuleset,
    details: Option<&HeroDetails>,
) -> u32 {
    let table = part.params().sea_points();
    details.map_or(table[0], |details| sea_points(details, core, table))
}

/// A hero's day's maximum on its surface (ADV-MOV-002, E08 T72; ADR-0016 point 3): its
/// [`sea_day_points`] in a boat, else its [`day_points`] for its army.
pub(crate) fn day_maximum(
    core: &Ruleset,
    part: &AdventureRuleset,
    details: Option<&HeroDetails>,
    hero: &HeroOnMap,
) -> u32 {
    if hero.boat.is_some() {
        sea_day_points(core, part, details)
    } else {
        day_points(core, part, details, &hero.army)
    }
}

/// The points a hero keeps when a boarding or a landing changes its surface (ADV-SEA-002,
/// ADV-SEA-003; ADR-0016 point 3, D-41, E08 T73): with free boarding (ADR-0013 point 13.7, the
/// Admiral's Hat) its points `points`, after the step's drop to 0, × the `new` maximum ÷ the
/// `old` one, truncated (a 64-bit product; an old maximum of 0 gives 0); without it none. The
/// original's order, confirmed by batch E8 (E8-1: 75 left after the step onto the boat drop to 0
/// first; 425 become 637 = 425 × 3 000 ÷ 2 000; E8-2: 500 become 333); VCMI converts before the
/// drop.
// rule: ADV-SEA-002, ADV-SEA-003
pub(crate) fn converted_points(points: u32, (old, new): (u32, u32), free: bool) -> u32 {
    if !free || old == 0 {
        return 0;
    }
    u32::try_from(u64::from(points) * u64::from(new) / u64::from(old)).unwrap_or(u32::MAX)
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
/// none without `details`. Its flying and water-walking levels price its steps and decide its
/// passage since T41 (ADV-MOV-009, ADV-MOV-010).
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
    /// Whether the terrain is water (E08 T41: the fly and water-walk prices, ADV-MOV-009,
    /// ADV-MOV-010).
    water: bool,
}

impl TerrainPrice {
    /// The price of a terrain the part lacks (a tile off the map).
    const UNKNOWN: Self = Self {
        walk: 0xFFFF,
        native: false,
        water: false,
    };
}

/// The prices that replace a step's walking price when the hero flies or water-walks
/// (ADV-MOV-009, ADV-MOV-010; E08 T41): the fly price at its flying level and the water-walk price
/// at its water-walking level, each if it has the level.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct Lifts {
    fly: Option<u32>,
    water_walk: Option<u32>,
}

impl Lifts {
    /// Whether neither price applies: a walking step.
    const fn none(self) -> bool {
        self.fly.is_none() && self.water_walk.is_none()
    }
}

/// What prices one hero's steps (ADV-MOV-003 to ADV-MOV-006, ADV-MOV-009, ADV-MOV-010; the
/// original's `step_cost` 0x4B1740 with Pathfinding, Nomads and Cursed Ground since E08 T40,
/// flying and water walking since T41): the adventure rules, each terrain's walking price for the
/// hero and its army, the native cost, the hero's fly and water-walk prices (for the step, and for
/// the drop to 0 as `ADV-QUIRK-17` has them) and the profile's quirks of the diagonal exception.
/// Built once per search or move, so the step's price is one load per terrain.
#[derive(Debug, Clone)]
pub(crate) struct Pricing<'a> {
    part: &'a AdventureRuleset,
    /// By core terrain.
    prices: Vec<TerrainPrice>,
    native_cost: u32,
    /// The hero's fly and water-walk prices for its steps.
    lifts: Lifts,
    /// The ones the drop to 0 counts: none under `ADV-QUIRK-17`.
    drop_lifts: Lifts,
    /// `ADV-QUIRK-2`: the exception's check ignores roads.
    check_ignores_roads: bool,
    /// `ADV-QUIRK-3`: the exception's check ignores the native cost.
    check_ignores_native: bool,
    /// The share of a boat's step from a tile a ground of the sea tops (ADV-SEA-004), as
    /// numerator and denominator (the part checks 1 <= numerator <= denominator).
    sea_ground: [u32; 2],
}

impl<'a> Pricing<'a> {
    /// The pricing of the steps of a hero with these levels (`h3_heroes::movement`) and army
    /// under `core`'s profile: each terrain counts as a terrain walker of a creature in the army
    /// makes it count (ADV-MOV-003, Nomads), then costs the step's price of that terrain at the
    /// hero's Pathfinding level (the part's [`AdventureRuleset::step_cost`]), and is native when
    /// that terrain is the army's ([`native_terrain`]). A flying level brings the part's fly
    /// price at that level, a water-walking level its water-walk price (ADV-MOV-009, ADV-MOV-010);
    /// the drop to 0 counts them only with `ADV-QUIRK-17` off (the original's `cheapest_step_from`
    /// 0x4B1880 reads the hero's spell levels, not the artifacts': none in increment 5).
    // rule: ADV-MOV-003, ADV-MOV-004, ADV-MOV-006, ADV-MOV-009, ADV-MOV-010
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
                    water: matches!(
                        part.passage(TerrainId::from_raw(raw)),
                        Some(Passage::Water(_))
                    ),
                }
            })
            .collect();
        let params = part.params();
        let lifts = Lifts {
            fly: movement.flying.map(|level| params.fly_cost(level)),
            water_walk: movement
                .water_walking
                .map(|level| params.water_walk_cost(level)),
        };
        Self {
            part,
            prices,
            native_cost: u32::from(params.native_cost().get()),
            lifts,
            drop_lifts: if quirks.is_enabled(DROP_IGNORES_ARTIFACTS) {
                Lifts::default()
            } else {
                lifts
            },
            check_ignores_roads: quirks.is_enabled(DIAGONAL_CHECK_IGNORES_ROADS),
            check_ignores_native: quirks.is_enabled(DIAGONAL_CHECK_IGNORES_NATIVE),
            sea_ground: params.sea_ground().map(u32::from),
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

    /// A step's price from `from` after the hero's flying and water walking (ADV-MOV-009,
    /// ADV-MOV-010; survey `E08-hero-movement.md` §2.1 points 4, 5): from water the water-walk
    /// price replaces `walk`; then the fly price replaces it from water and from a tile an object
    /// blocks (an obstacle flown over), and from a land tile no object blocks (an object's
    /// visitable tile included: the original's tile bit +0xC 0x40, ADV-Q22) the lesser of the two
    /// applies. `from` is the tile as it stands now (the live occupancy).
    #[inline]
    fn lifted(&self, from: &StaticTile, walk: u32, lifts: Lifts) -> u32 {
        if lifts.none() {
            return walk;
        }
        let water = self.price(from.terrain).water;
        let price = match lifts.water_walk {
            Some(water_walk) if water => water_walk,
            _ => walk,
        };
        match lifts.fly {
            None => price,
            Some(fly) if water || (from.blocked && !from.visitable) => fly,
            Some(fly) => price.min(fly),
        }
    }

    /// The cheapest straight step the hero makes (every step costs at least this): the least of
    /// the native cost, the roads' costs, the land terrains' walking prices and the hero's fly
    /// and water-walk prices (E08 T41).
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
        roads
            .chain(lands)
            .chain(self.lifts.fly)
            .chain(self.lifts.water_walk)
            .fold(self.native_cost, u32::min)
    }

    /// The price of a step from `from` to `to` with `points` left (ADV-MOV-003 to ADV-MOV-006,
    /// ADV-MOV-009, ADV-MOV-010). Straight: the road's cost when both tiles have a road, else the
    /// native cost when the source's terrain counts as the army's and is not under Cursed Ground,
    /// else the source terrain's walking price (by the hero's Pathfinding level, Nomads' sand as
    /// dirt): only the source tile prices a step (D-2); then the hero's flying and water walking
    /// ([`Self::lifted`], E08 T41). Diagonal: ⌊straight × √2⌋, except that with points from a check
    /// cost C up to ⌊C × √2⌋ it costs only the straight cost (the diagonal exception). C is the
    /// step's own straight cost, whatever priced it (the fly or water-walk price included); under
    /// `ADV-QUIRK-2` or `ADV-QUIRK-3` it is the walking check, never the fly or water-walk price,
    /// which under `ADV-QUIRK-2` leaves out the road and under `ADV-QUIRK-3` the native cost (the
    /// original compares with the source terrain's walking price, 0x4B1740). `from` and `to` are
    /// the tiles as they stand now. A price above `points` means the step cannot be made.
    // rule: ADV-MOV-003, ADV-MOV-004, ADV-MOV-005, ADV-MOV-006, ADV-MOV-009, ADV-MOV-010
    pub(crate) fn step(
        &self,
        from: &StaticTile,
        to: &StaticTile,
        diagonal: bool,
        points: u32,
    ) -> u32 {
        self.price_step::<true>(from, to, diagonal, points)
    }

    /// [`Self::step`] for a hero that neither flies nor water-walks, without the test of its
    /// fly and water-walk prices: the walking search's hot loop (`bench-check`, E08 T41). The
    /// same price as [`Self::step`] for such a hero. Not inlined: the search's loop measures
    /// fewer instructions with the call (`bench-check`, E08 T41).
    #[inline(never)]
    pub(crate) fn walking_step(
        &self,
        from: &StaticTile,
        to: &StaticTile,
        diagonal: bool,
        points: u32,
    ) -> u32 {
        self.price_step::<false>(from, to, diagonal, points)
    }

    /// [`Self::step`], with the hero's fly and water-walk prices when `LIFTED`.
    #[inline(always)]
    fn price_step<const LIFTED: bool>(
        &self,
        from: &StaticTile,
        to: &StaticTile,
        diagonal: bool,
        points: u32,
    ) -> u32 {
        let road = self.road(from, to);
        let walk = self.base(from, road, true);
        let straight = if LIFTED {
            self.lifted(from, walk, self.lifts)
        } else {
            walk
        };
        if !diagonal {
            return straight;
        }
        // Clean, a walking step's check is its walking price: the same as the lifted one's.
        let check = if !LIFTED || self.check_ignores_roads || self.check_ignores_native {
            self.base(
                from,
                road.filter(|_| !self.check_ignores_roads),
                !self.check_ignores_native,
            )
        } else {
            straight
        };
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
    /// Nomads); then the hero's water walking and flying as for a step ([`Self::lifted`]: on water
    /// the water-walk price, then the fly price from water or a blocked tile and the lesser from a
    /// free land tile), without them under `ADV-QUIRK-17` (E08 T41: the original passes the
    /// hero's spell levels, not the artifacts'). The original's `cheapest_step_from` 0x4B1880
    /// after each step of a move, in 0x4805E0. `to` is the tile as it stands now.
    // rule: ADV-MOV-006
    pub(crate) fn left_after(&self, to: &StaticTile, left: u32) -> u32 {
        let cheapest = self.lifted(to, self.walking_cheapest(to), self.drop_lifts);
        if left < cheapest { 0 } else { left }
    }

    /// [`Self::left_after`] for a hero that neither flies nor water-walks: the walking search's
    /// hot loop (E08 T41), the same points as [`Self::left_after`] for such a hero.
    #[inline]
    pub(crate) fn walking_left_after(&self, to: &StaticTile, left: u32) -> u32 {
        if left < self.walking_cheapest(to) {
            0
        } else {
            left
        }
    }

    /// A boat's straight step from `from` (ADV-SEA-004, E08 T73): water's price and the step's.
    /// Water's is its terrain's table price, the same at every Pathfinding level (the part's
    /// `Passage::Water`, 100; the original's `step_cost` for a boat, 0x4B24B7); the step's is
    /// water's, or on a tile a ground of the sea tops (Favorable Winds) the part's `sea_ground`
    /// share of it, truncated (66; 0x4B17B3).
    #[inline]
    fn sea_straight(&self, from: &StaticTile) -> (u32, u32) {
        let water = self.price(from.terrain).walk;
        let [numerator, denominator] = self.sea_ground;
        let straight = if from.winds {
            water * numerator / denominator
        } else {
            water
        };
        (water, straight)
    }

    /// The price of a boat's step from the water tile `from` with `points` left (ADV-SEA-004, E08
    /// T73; survey `E08-sea.md` §4.1): the straight step of [`Self::sea_straight`], with neither
    /// roads, nor the native cost, nor flying or water walking (a hero in a boat has neither); a
    /// diagonal ⌊straight × √2⌋ (141, 93 from Favorable Winds), except that with points from
    /// water's price C up to ⌊C × √2⌋ it costs only the straight step (ADV-MOV-005's exception,
    /// compared with water's price, never the winds' step: D-43, confirmed by batch E8's E8-4; in
    /// both profiles). A landing's step is priced so, from its water tile.
    // rule: ADV-SEA-004, ADV-MOV-005
    pub(crate) fn sail(&self, from: &StaticTile, diagonal: bool, points: u32) -> u32 {
        let (water, straight) = self.sea_straight(from);
        if !diagonal || (water..=diagonal_cost(water)).contains(&points) {
            straight
        } else {
            diagonal_cost(straight)
        }
    }

    /// The price of a step of a hero moving by `mode` (E08 T73): a boat's ([`Self::sail`]),
    /// else [`Self::step`] (a boarding's step is priced by the hero's mode on land).
    pub(crate) fn step_by(
        &self,
        mode: Mode,
        (from, to): (&StaticTile, &StaticTile),
        diagonal: bool,
        points: u32,
    ) -> u32 {
        match mode {
            Mode::Sail(_) => self.sail(from, diagonal, points),
            _ => self.step(from, to, diagonal, points),
        }
    }

    /// The points left on `to` after a step that left `left` and met `met` (ADV-MOV-006's drop,
    /// E08 T73; design §8): a step that ends at sea or boards drops to 0 below a boat's cheapest
    /// step from `to` (the straight step of [`Self::sea_straight`]: 100, 66 on Favorable Winds;
    /// the original's `cheapest_step_from` 0x4B1880 for a boat); a landing as a walker without
    /// its levels ([`Self::walking_left_after`]: the coast tile's road price, else the native
    /// price, Cursed Ground's terrain price under it), in both profiles; any other step
    /// [`Self::left_after`].
    // rule: ADV-MOV-006, ADV-SEA-002, ADV-SEA-003, ADV-SEA-004
    pub(crate) fn left_by(
        &self,
        mode: Mode,
        met: Option<Meets<'_>>,
        to: &StaticTile,
        left: u32,
    ) -> u32 {
        match (mode, met) {
            (Mode::Sail(_), Some(Meets::Land(_))) => self.walking_left_after(to, left),
            (Mode::Sail(_), _) | (_, Some(Meets::Board(_))) => self.sail_left_after(to, left),
            _ => self.left_after(to, left),
        }
    }

    /// The points a boat has left on `to`, or a hero that boarded there, after a step that left
    /// `left`: none below a boat's cheapest step from `to` (100, 66 on Favorable Winds).
    pub(crate) fn sail_left_after(&self, to: &StaticTile, left: u32) -> u32 {
        if left < self.sea_straight(to).1 {
            0
        } else {
            left
        }
    }

    /// The cheapest walking step from `to`, priced against itself: its road's cost, else the
    /// native cost, or under Cursed Ground the terrain's walking price.
    #[inline(always)]
    fn walking_cheapest(&self, to: &StaticTile) -> u32 {
        to.road
            .and_then(|road| self.road_cost(road))
            .unwrap_or_else(|| {
                if to.no_native {
                    self.price(to.terrain).walk
                } else {
                    self.native_cost
                }
            })
    }
}

#[cfg(test)]
mod tests;
