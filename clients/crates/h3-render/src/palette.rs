//! Palettes in the screen's format, and the player colours (`docs/epics/E17-design.md` §3.3).
//!
//! The executable converts a sprite's or an image's palette into the screen's format once, when it
//! loads it (0x5226D0); the drawers then look indices up in that converted palette. Player colours
//! are a palette change made before drawing: indices 224–255 take the player's 32 colours
//! (0x5FFE40 for the 24-bit palette, 0x5FFE20 for the converted one); the drawers know nothing of
//! players.

use core::ops::RangeInclusive;

use h3_assets::palette::{Palette, Rgb};

use crate::screen::PixelFormat;

/// The indices that take a player's colours.
pub const PLAYER_COLOURS: RangeInclusive<usize> = 224..=255;

/// A palette converted into a screen format.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScreenPalette(pub [u16; 256]);

impl ScreenPalette {
    /// `palette` in `format`, each colour truncated (0x5226D0).
    #[must_use]
    pub fn new(palette: &Palette, format: PixelFormat) -> Self {
        Self(palette.0.map(|c| format.pack(c)))
    }

    /// The pixel for `index`.
    #[must_use]
    pub fn get(&self, index: u8) -> u16 {
        self.0[usize::from(index)]
    }
}

/// `palette` with indices 224–255 replaced by `colours` (a player's 32; fewer replace fewer, more
/// are ignored): `PLAYERS.PAL`'s entries from 32 × the player's colour (0x5FFE20 copies them from
/// its 16-bit conversion, 0x6AACA8; read at E17 M3 T3a for the heroes' flags).
#[must_use]
pub fn with_player_colours(palette: &Palette, colours: &[Rgb]) -> Palette {
    let mut out = palette.clone();
    for (slot, &colour) in out.0.iter_mut().skip(*PLAYER_COLOURS.start()).zip(colours) {
        *slot = colour;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_palette_is_converted_entry_by_entry() {
        let mut p = Palette::default();
        p.0[1] = Rgb { r: 255, g: 0, b: 0 };
        p.0[255] = Rgb { r: 0, g: 255, b: 7 };
        let s = ScreenPalette::new(&p, PixelFormat::RGB565);
        assert_eq!((s.get(0), s.get(1), s.get(255)), (0, 0xF800, 0x07E0));
    }

    #[test]
    fn player_colours_replace_the_last_32_indices() {
        let red = Rgb { r: 200, g: 0, b: 0 };
        let p = with_player_colours(&Palette::default(), &[red; 40]);
        assert_eq!(p.0[223], Rgb::default());
        assert!(p.0[PLAYER_COLOURS].iter().all(|&c| c == red));
        let p = with_player_colours(&Palette::default(), &[red; 2]);
        assert_eq!((p.0[224], p.0[225], p.0[226]), (red, red, Rgb::default()));
    }
}
