//! Screen layouts as data (E17 M1 T7; ADR-0020 point 3: the frame size and every layout are data,
//! never literals in code). One file per screen and resolution under `clients/layouts/`; the
//! native battle screen is `battle.toml` (`docs/epics/E17-design.md` §4). An HD layout is another
//! file of the same format. The format is *provisional* until the M1 review.

use alloc::string::{String, ToString};
use serde::Deserialize;

/// The battle screen's layout: the frame, the hex grid and where sprites stand.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BattleLayout {
    /// The frame's width and height (800 × 600).
    pub frame: [u32; 2],
    /// The top-left corner of the box of the hex in row 0, column 0, before an even row's shift
    /// (14, 86; §4.1).
    pub hex_origin: [i32; 2],
    /// A column's width and a row's height (44, 42).
    pub hex_step: [i32; 2],
    /// What an even row adds to x (22: half a hex; rows count from 0).
    pub even_row_shift: i32,
    /// Where a sprite stands, from its hex box's top-left corner (22, 42; the hex record's anchor).
    pub anchor: [i32; 2],
    /// A creature's canvas facing right has its top-left corner at the anchor minus this
    /// (196, 267); facing left it is mirrored, its left edge at the anchor's x minus the DEF's
    /// width plus this x (§4.2).
    pub stack_canvas: [i32; 2],
    /// The first and last column the pass of the layer-1 obstacles reads (1, 15; §4.6).
    pub before_stacks_columns: [u8; 2],
    /// The palette whose entries colour the outlines (`game.pal`, §4.2).
    pub outline_palette: String,
    /// The active stack's outline: that palette's entry (0x60) before its range rotates.
    pub outline_active: u8,
}

impl BattleLayout {
    /// The top-left corner of the box of the hex in `row` and `column`.
    pub fn hex_box(&self, row: u8, column: u8) -> [i32; 2] {
        let shift = if row.is_multiple_of(2) {
            self.even_row_shift
        } else {
            0
        };
        [
            self.hex_origin[0] + self.hex_step[0] * i32::from(column) + shift,
            self.hex_origin[1] + self.hex_step[1] * i32::from(row),
        ]
    }

    /// Where a sprite on the hex in `row` and `column` stands (the hex record's anchor).
    pub fn anchor(&self, row: u8, column: u8) -> [i32; 2] {
        let [x, y] = self.hex_box(row, column);
        [x + self.anchor[0], y + self.anchor[1]]
    }
}

/// Parses a battle layout file.
///
/// # Errors
/// The parser's message: not TOML, an unknown, missing or mistyped field.
pub fn parse_battle(text: &str) -> Result<BattleLayout, String> {
    toml::from_str(text).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The committed native layout.
    const NATIVE: &str = include_str!("../../../layouts/battle.toml");

    #[test]
    fn the_native_layout_gives_the_executables_hex_boxes() {
        let layout = parse_battle(NATIVE).unwrap();
        assert_eq!(layout.frame, [800, 600]);
        // §4.1: left = 14 + 44c + 22e, top = 86 + 42r, e = 1 on even rows; anchor (+22, +42).
        assert_eq!(layout.hex_box(0, 0), [36, 86]);
        assert_eq!(layout.hex_box(1, 0), [14, 128]);
        assert_eq!(layout.hex_box(10, 16), [14 + 44 * 16 + 22, 86 + 420]);
        assert_eq!(layout.anchor(5, 3), [14 + 132 + 22, 86 + 210 + 42]);
        assert_eq!(layout.outline_palette, "game.pal");
        assert_eq!(layout.outline_active, 0x60);
    }

    #[test]
    fn a_bad_layout_is_an_error() {
        assert!(parse_battle("frame = [1, 2]\nwhat = 3\n").is_err());
    }
}
