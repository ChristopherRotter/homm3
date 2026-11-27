//! The computer's scouting value of a circle (E31 T2h; survey
//! `docs/rules/surveys/E31-computer-player.md` §2.4): `ai_vision_area_value` 0x432220 and its
//! class values (`ai_class_scout_values` 0x6925AC, filled at start-up by `ai_class_tables_init`
//! 0x428070 from `ai_class_scout_value_list` 0x6605D8). The object value asks it for the Redwood
//! Observatory and the Pillar of Fire (radius 20); the turn start's Eye value would ask it for
//! each Eye of the Magi (radius 10), which the engine does not have (`AI-STUB-10`). It reads the
//! objects under the fog (survey §5.2's hidden reads). Its tests are the object value's
//! (`objects/tests.rs`).

use h3_adventure::Pos;
use h3_contracts::PlayerId;

use super::objects::at;
use crate::{GameRuleset, GameState};

/// The scouting value of an unexplored entrance by its class (`ai_class_scout_value_list`
/// 0x6605D8: 75 pairs of a class and its value, in the list's order; a class it does not list
/// 0). Among them classes the engine does not have: the Cover of Darkness (15), the Grail (36),
/// the Hut of the Magi (37), the Obelisk (57) and the Seer's Hut (83).
pub(super) const SCOUT: [(u8, i32); 75] = [
    (2, 1),
    (4, 100),
    (5, 200),
    (6, 400),
    (8, 100),
    (10, 500),
    (11, 1),
    (12, 10),
    (13, 1000),
    (14, 1),
    (15, 1),
    (16, 10),
    (17, 10),
    (20, 10),
    (22, 1),
    (23, 100),
    (24, 10),
    (25, 10),
    (28, 1),
    (29, 5),
    (30, 1),
    (31, 1),
    (32, 100),
    (35, 1),
    (36, 1000),
    (37, 200),
    (38, 1),
    (39, 1),
    (41, 400),
    (42, 10),
    (43, 50),
    (45, 100),
    (47, 50),
    (48, 10),
    (49, 10),
    (51, 100),
    (52, 1),
    (53, 20),
    (55, 10),
    (56, 1),
    (57, 10),
    (58, 100),
    (60, 100),
    (61, 100),
    (62, 200),
    (63, 10),
    (64, 1),
    (78, 10),
    (79, 10),
    (81, 20),
    (82, 10),
    (83, 10),
    (84, 10),
    (85, 10),
    (86, 10),
    (88, 10),
    (89, 10),
    (90, 10),
    (93, 10),
    (94, 10),
    (96, 1),
    (98, 200),
    (100, 50),
    (101, 20),
    (102, 100),
    (104, 50),
    (105, 1),
    (106, 1),
    (107, 50),
    (108, 10),
    (109, 10),
    (110, 1),
    (111, 50),
    (112, 10),
    (113, 50),
];

/// The scouting value of an unexplored entrance of class number `class` ([`SCOUT`], 0 for a class
/// it does not list).
fn scout(class: u8) -> i32 {
    SCOUT
        .iter()
        .find(|&&(c, _)| c == class)
        .map_or(0, |&(_, value)| value)
}

/// `player`'s scouting value of the circle of radius `radius` around `pos`
/// (`ai_vision_area_value` 0x432220; CPU-OBJ-012): over the tiles of `pos`'s level within
/// `radius` columns and rows (clipped to the map) whose distance from it is at most `radius` +
/// 0.5 (the original's `sqrt(dx² + dy²)` in double precision against `r + 0.5`: exactly
/// `dx² + dy² ≤ r² + r`), each tile `player` has not explored counts 1, plus its class's scouting
/// value ([`SCOUT`]) when it is an entrance (`super::objects::at`: a hero's tile the class 34, an
/// empty boat's 8).
// rule: CPU-OBJ-012
pub(super) fn value(
    state: &GameState,
    rules: &GameRuleset,
    player: PlayerId,
    radius: i32,
    pos: Pos,
) -> i32 {
    let adventure = state.adventure();
    let map = adventure.map();
    let (width, height) = (i32::from(map.width()), i32::from(map.height()));
    let (x, y) = (i32::from(pos.x), i32::from(pos.y));
    let within = radius.wrapping_mul(radius).wrapping_add(radius);
    let mut value = 0_i32;
    for row in (y - radius).max(0)..(y + radius + 1).min(height) {
        for column in (x - radius).max(0)..(x + radius + 1).min(width) {
            let (dx, dy) = (column - x, row - y);
            if dx * dx + dy * dy > within {
                continue;
            }
            let tile = Pos::new(
                u16::try_from(column).unwrap_or(0),
                u16::try_from(row).unwrap_or(0),
                pos.level,
            );
            if adventure.is_explored(player, tile) {
                continue;
            }
            let class = at(state, rules, tile).map_or(0, |at| at.class().number());
            value = value.wrapping_add(1).wrapping_add(scout(class));
        }
    }
    value
}
