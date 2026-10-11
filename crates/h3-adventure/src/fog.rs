//! Fog of war (ADR-0016 point 5, E08 T8a): each player's explored tiles, one [`BitGrid`] per
//! player in the state, and the shape of the area a hero reveals. A tile once explored stays
//! explored (no rule clears one in increment 1; darkness comes with the towns' buildings). The
//! fringe, the hidden tiles touching an explored one, and a player's sight as planes of words for
//! the observation (E08 T8b1, ADV-FOG-005).

use alloc::vec;
use alloc::vec::Vec;

use crate::movement::DIRECTIONS;
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

/// Whether the tile at `pos`, a tile of the grid, is on the fringe of `explored` (ADV-FOG-005):
/// not explored, and one of the eight tiles around it on its level is.
// rule: ADV-FOG-005
pub(crate) fn on_fringe(explored: &BitGrid, pos: Pos) -> bool {
    !explored.get(pos)
        && DIRECTIONS.iter().any(|&(dx, dy)| {
            pos.x
                .checked_add_signed(i16::from(dx))
                .zip(pos.y.checked_add_signed(i16::from(dy)))
                .is_some_and(|(x, y)| explored.get(Pos::new(x, y, pos.level)))
        })
}

/// A player's sight as planes of words (E08 T8b1): level by level, row by row, `stride` words a
/// row, the tile of column `x` as bit `x % 64` of word `x / 64`; bits past the map's width are
/// clear.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Sight {
    /// Words a row: the map's width divided by 64, rounded up.
    pub(crate) stride: usize,
    /// The explored tiles.
    pub(crate) explored: Vec<u64>,
    /// The fringe: the tiles not explored with an explored one among the eight around them on
    /// their level.
    pub(crate) fringe: Vec<u64>,
}

impl Sight {
    /// No tile seen on a map of these dimensions (a viewer that is not a player of the game).
    pub(crate) fn none(width: u16, height: u16, levels: u8) -> Self {
        let stride = usize::from(width.div_ceil(64));
        let words = stride * usize::from(height) * usize::from(levels);
        Self {
            stride,
            explored: vec![0; words],
            fringe: vec![0; words],
        }
    }

    /// The sight of a player who explored `explored`, built word by word: each row of the grid
    /// read once ([`BitGrid::read_row`]), the fringe as the explored tiles spread to the rows
    /// above and below and one column either way, less the explored tiles (ADV-FOG-005).
    // rule: ADV-FOG-005
    pub(crate) fn of(explored: &BitGrid) -> Self {
        let mut sight = Self::none(explored.width(), explored.height(), explored.levels());
        let (stride, height) = (sight.stride, usize::from(explored.height()));
        for (row, words) in sight.explored.chunks_exact_mut(stride).enumerate() {
            let level = u8::try_from(row / height).unwrap_or(u8::MAX);
            let y = u16::try_from(row % height).unwrap_or(u16::MAX);
            explored.read_row(level, y, words);
        }
        let last = match explored.width() % 64 {
            0 => u64::MAX,
            columns => u64::MAX >> (64 - columns),
        };
        let seen = &sight.explored;
        let mut spread = vec![0; stride];
        for (row, fringe) in sight.fringe.chunks_exact_mut(stride).enumerate() {
            let y = row % height;
            let at = row * stride;
            for (w, word) in spread.iter_mut().enumerate() {
                let above = if y > 0 { seen[at - stride + w] } else { 0 };
                let below = if y + 1 < height {
                    seen[at + stride + w]
                } else {
                    0
                };
                *word = above | seen[at + w] | below;
            }
            for (w, word) in fringe.iter_mut().enumerate() {
                let v = spread[w];
                // The column left of a word's first is the last of the word before; the column
                // right of its last is the first of the word after.
                let from_before = if w > 0 { spread[w - 1] >> 63 } else { 0 };
                let from_after = if w + 1 < stride {
                    spread[w + 1] << 63
                } else {
                    0
                };
                let valid = if w + 1 == stride { last } else { u64::MAX };
                *word = (v | v << 1 | from_before | v >> 1 | from_after) & !seen[at + w] & valid;
            }
        }
        sight
    }
}

#[cfg(test)]
mod tests;
