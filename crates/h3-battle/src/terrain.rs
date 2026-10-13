//! Terrain: the native-terrain bonus (BAT-TER-001) and random obstacles (BAT-GEO-004).
//!
//! Obstacle generation runs *before* the battle starts (the caller puts the hexes into
//! `BattleSetup::obstacles`, 03-domain "Seams"), with a random stream the caller chooses
//! (`StreamId::OBSTACLES` in the tools). The algorithm is ported from VCMI
//! (`BattleInfo::setupBattle`, 1.7.5), which follows the original's; the original seeds it from
//! the map tile, we from the stream (fidelity is about the algorithm, not the seed).

use alloc::vec::Vec;

use h3_content::{CreatureTypeId, TerrainId};
use h3_content_battle::{BattleRuleset, ObstacleType};
use h3_contracts::Obstacle;
use h3_kernel::Rng;

use crate::geometry::{Field, Hex, HexSet};

/// Whether creatures of type `creature` fight on their native terrain: their faction's native
/// terrain is the battle's (neutral creatures have none) (BAT-TER-001).
// rule: BAT-TER-001
pub(crate) fn is_native(
    ruleset: &BattleRuleset,
    creature: CreatureTypeId,
    terrain: Option<TerrainId>,
) -> bool {
    let native = ruleset
        .creature_info(creature)
        .and_then(|i| i.faction)
        .and_then(|f| ruleset.factions().get(f.index()))
        .and_then(|f| f.native_terrain);
    native.is_some() && native == terrain
}

/// One of `items`, uniformly, or `None` if there are none.
fn pick<T: Copy>(rng: &mut Rng, items: &[T]) -> Option<T> {
    let n = u32::try_from(items.len()).unwrap_or(u32::MAX);
    if n == 0 {
        return None;
    }
    items
        .get(usize::try_from(rng.below(n)).unwrap_or(0))
        .copied()
}

/// The hexes obstacle `o` blocks with its anchor at `(x, y)` (bottom-left hex), or `None` if the
/// placement is not allowed: the anchor needs `height` rows above it, a standable hex (column ≥ 1
/// with the original's one side column) and `width` columns to its right within the first
/// `field width − 2` columns; every blocked hex must be
/// on the field, free, and in columns 3 to `field width − 4` (the two columns next to each
/// army's starting column stay free). From an odd row, blocked hexes in even rows shift one
/// column left, so that `(1, -1)` is always the top-right neighbour.
fn placement(
    field: Field,
    width: i32,
    o: &ObstacleType,
    anchor: Hex,
    blocked: HexSet,
) -> Option<HexSet> {
    let (x, y) = field.coords(anchor);
    if y <= i32::from(o.height)
        || !field.is_standable(anchor)
        || x + i32::from(o.width) > width - 2
        || blocked.contains(anchor)
    {
        return None;
    }
    let mut hexes = HexSet::new();
    for &(dx, dy) in &o.blocked {
        let ty = y + i32::from(dy);
        let shift = i32::from(y % 2 == 1 && ty % 2 == 0);
        let tx = x + i32::from(dx) - shift;
        let hex = field
            .hex(tx, ty)
            .filter(|h| !blocked.contains(*h) && (3..=width - 4).contains(&tx))?;
        hexes.insert(hex);
    }
    Some(hexes)
}

/// Generates the obstacles of a battle on `terrain` (BAT-GEO-004), in the order placed, each
/// with its hexes in hex order: block 5–12 hexes; with 40 % chance first place one absolute
/// obstacle of the terrain (it counts half its hexes; no anchor); then, while hexes remain to be
/// blocked, place a uniformly chosen usual obstacle of the terrain at a uniformly chosen allowed
/// anchor (rows 1 to `height − 2`; the obstacle keeps it), stopping early if no obstacle or no
/// anchor is left.
// rule: BAT-GEO-004
pub fn generate_obstacles(
    ruleset: &BattleRuleset,
    terrain: TerrainId,
    rng: &mut Rng,
) -> Vec<Obstacle> {
    let params = ruleset.battle().field;
    let field = Field::new(params);
    let (width, height) = (i32::from(params.width), i32::from(params.height));
    let of_terrain = |absolute: bool| -> Vec<&ObstacleType> {
        ruleset
            .obstacles()
            .iter()
            .filter(|o| o.absolute == absolute && o.terrains.contains(&terrain))
            .collect()
    };
    let mut blocked = HexSet::new();
    let mut out = Vec::new();
    let piece = |hexes: HexSet, anchor: Option<Hex>| Obstacle {
        hexes: hexes.iter().map(|h| h.0).collect(),
        anchor: anchor.map(|a| a.0),
    };
    // 5..=12 always fits in an i32.
    let mut to_block = i32::try_from(rng.range_inclusive(5, 12)).unwrap_or(0);
    if rng.range_inclusive(1, 100) <= 40
        && let Some(o) = pick(rng, &of_terrain(true))
    {
        // The compiler checked that absolute obstacles block hexes of the field.
        let hexes = o
            .blocked
            .iter()
            .filter_map(|&(x, y)| field.hex(i32::from(x), i32::from(y)))
            .fold(HexSet::new(), |mut set, h| {
                set.insert(h);
                set
            });
        blocked = blocked.union(hexes);
        out.push(piece(hexes, None));
        to_block -= i32::try_from(o.blocked.len() / 2).unwrap_or(i32::MAX);
    }
    let usual = of_terrain(false);
    while to_block > 0 {
        let Some(o) = pick(rng, &usual) else {
            break;
        };
        // Anchors from row 1, column 1 to row `height − 2`, column `width − 2`, in hex order.
        let first = u8::try_from(width + 1).unwrap_or(u8::MAX);
        let last = u8::try_from((height - 1) * width - 2).unwrap_or(0);
        let anchors: Vec<(Hex, HexSet)> = (first..=last)
            .filter_map(|i| placement(field, width, o, Hex(i), blocked).map(|s| (Hex(i), s)))
            .collect();
        let Some((anchor, hexes)) = pick(rng, &anchors) else {
            break;
        };
        blocked = blocked.union(hexes);
        out.push(piece(hexes, Some(anchor)));
        to_block -= i32::try_from(o.blocked.len()).unwrap_or(i32::MAX);
    }
    out
}

/// The hexes obstacle `o` blocks at `anchor` on an empty field (tests only).
#[cfg(test)]
pub(crate) fn placement_for_tests(field: Field, o: &ObstacleType, anchor: Hex) -> Option<Vec<u8>> {
    placement(field, 17, o, anchor, HexSet::new()).map(|s| s.iter().map(|h| h.0).collect())
}
