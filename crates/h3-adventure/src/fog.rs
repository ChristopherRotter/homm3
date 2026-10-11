//! Fog of war (ADR-0016 point 5, E08 T8a): each player's explored tiles, one [`BitGrid`] per
//! player in the state, and the shape of the area a hero reveals. A tile once explored stays
//! explored (no rule clears one in increment 1; darkness comes with the towns' buildings). What a
//! viewer sees of a tile, the fringe included, is the projection's (T8b).

use crate::persistent::BitGrid;
use crate::setup::Pos;

/// Reveals on `grid` the tiles around `center` within `radius` (ADV-FOG-002): the tiles on its
/// level, on the map, with dx² + dy² ≤ r² + r, a disk row by row, each row one run of
/// [`BitGrid::set_run`].
///
/// The original reveals a tile when √(dx² + dy²) ≤ r + 0.5 (0x49D160, `sqrt` at 0x617B94 against
/// the radius plus 0.5); VCMI when round(√(dx² + dy²)) ≤ r (`getTilesInRange`, `DIST_2D`). Both
/// are the integer test above: dx² + dy² is an integer and (r + ½)² = r² + r + ¼.
// rule: ADV-FOG-002
pub(crate) fn reveal(grid: &mut BitGrid, center: Pos, radius: u8) {
    let r = u32::from(radius);
    let bound = r * r + r;
    let reach = u16::from(radius);
    for y in center.y.saturating_sub(reach)..=center.y.saturating_add(reach) {
        let dy = u32::from(y.abs_diff(center.y));
        // dy ≤ r, so the difference is at least r and the root at most r: it fits.
        let half = u16::try_from((bound - dy * dy).isqrt()).unwrap_or(u16::MAX);
        let left = center.x.saturating_sub(half);
        let len = (center.x.saturating_add(half) - left).saturating_add(1);
        grid.set_run(Pos::new(left, y, center.level), len, true);
    }
}

#[cfg(test)]
mod tests;
