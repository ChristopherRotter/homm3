//! The game's mouse cursor (E17 M2 T5a; common to both viewers since E24 M3 T1a, the map's edge
//! cursors since E24 M2 T2): a DEF frame drawn into the frame at the pointer (the drawn cursor) or
//! handed to the window as an image (the hardware cursor, ADR-0024 point 4).
//!
//! Which frame, its hot spot and its drawer are *provisional*: the executable's cursor code is not
//! read yet (E17 M3, the battle interface's cursors). As E17 M2 T2's spikes showed it:
//! [`DEFAULT`]'s first frame, its hot spot at the canvas's top-left corner, index 0 and the keyed
//! indices not drawn (here: the battle's drawer without an outline, so a shadow would darken what
//! is under it).

use alloc::vec::Vec;

use h3_assets::def::Frame;
use h3_assets::palette::Rgb;
use h3_render::blit::{Drawer, Placement, draw_frame};
use h3_render::palette::ScreenPalette;
use h3_render::screen::{PixelFormat, Screen};

use crate::sprite::Sprite;

/// The default cursor's DEF (in `H3sprite.lod`).
pub const DEFAULT: &str = "CRDEFLT.DEF";

/// The cursor's drawer (*provisional*).
const DRAWER: Drawer = Drawer::Shadowed { colour: None };

/// A cursor: one DEF frame, its palette in the screen's format and its hot spot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cursor {
    frame: Frame,
    palette: ScreenPalette,
    hot: (i32, i32),
    drawer: Drawer,
}

/// A cursor as an image over nothing, for the window: straight RGBA, 8 bits per channel, row by
/// row from the top, each colour widened as the frame's pixels are
/// ([`PixelFormat::unpack`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Image {
    /// Width in pixels (the frame's canvas).
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// The hot spot in its pixels.
    pub hot: (u32, u32),
    /// `width × height × 4` bytes.
    pub rgba: Vec<u8>,
}

impl Cursor {
    /// Frame `index` of `sprite`'s group 0, its hot spot at `hot` in the frame's canvas, its
    /// palette packed into `format`; `None` if the DEF has no such frame.
    #[must_use]
    pub fn new(sprite: &Sprite, index: u32, hot: (u32, u32), format: PixelFormat) -> Option<Self> {
        let frame = sprite.frame(0, index)?.clone();
        let coordinate = |v: u32| i32::try_from(v).unwrap_or(i32::MAX);
        Some(Self {
            frame,
            palette: ScreenPalette::new(&sprite.def.palette, format),
            hot: (coordinate(hot.0), coordinate(hot.1)),
            drawer: DRAWER,
        })
    }

    /// As [`Cursor::new`], drawn as the mouse manager draws a cursor's frame (0x50D8B0: the plain
    /// drawer with runs skipped, E17 M3 T6c-1): the battle's cursors (`crcombat.def`, T6c-2), the
    /// map's (`cradvntr.def`'s frames and hot spots from its layout's `[cursor]`, E24 M2 T2).
    #[must_use]
    pub fn read(sprite: &Sprite, index: u32, hot: (u32, u32), format: PixelFormat) -> Option<Self> {
        Some(Self {
            drawer: Drawer::Plain {
                skip_runs: true,
                colour: None,
            },
            ..Self::new(sprite, index, hot, format)?
        })
    }

    /// Draws the cursor with its hot spot at `(x, y)`, clipped by the screen.
    pub fn draw(&self, screen: &mut Screen, x: i32, y: i32) {
        let at = Placement {
            x: x.saturating_sub(self.hot.0),
            y: y.saturating_sub(self.hot.1),
            mirrored: false,
        };
        draw_frame(screen, &self.frame, &self.palette, at, self.drawer);
    }

    /// The rectangle (left, top, right, bottom; the last two exclusive) its frame's canvas covers
    /// with its hot spot at `(x, y)`: all [`Cursor::draw`] can change (E24 M3 T2b1).
    #[must_use]
    pub fn rect(&self, x: i32, y: i32) -> [i64; 4] {
        let (left, top) = (
            i64::from(x) - i64::from(self.hot.0),
            i64::from(y) - i64::from(self.hot.1),
        );
        [
            left,
            top,
            left + i64::from(self.frame.full_width),
            top + i64::from(self.frame.full_height),
        ]
    }

    /// The cursor as an image over nothing: drawn once over black and once over white. A pixel
    /// drawn alike over both is opaque; one that kept both backgrounds is transparent; one a
    /// shadow darkened is black, as opaque as the shadow made white dark (an approximation: the
    /// window blends where the frame halves or quarters).
    #[must_use]
    pub fn image(&self, format: PixelFormat) -> Image {
        let (width, height) = (self.frame.full_width, self.frame.full_height);
        let white = format.pack(Rgb {
            r: 255,
            g: 255,
            b: 255,
        });
        let over = |background: u16| {
            Screen::new(width, height, format).map(|mut s| {
                s.fill(background);
                self.draw(&mut s, self.hot.0, self.hot.1);
                s
            })
        };
        let mut rgba = Vec::new();
        if let (Some(dark), Some(light)) = (over(0), over(white)) {
            rgba.reserve(dark.pixels().len() * 4);
            for (&d, &l) in dark.pixels().iter().zip(light.pixels()) {
                let c = format.unpack(d);
                rgba.extend_from_slice(&match (d == l, d == 0 && l == white) {
                    (true, _) => [c.r, c.g, c.b, 255],
                    (false, true) => [0, 0, 0, 0],
                    (false, false) => [0, 0, 0, 255 - format.unpack(l).g],
                });
            }
        }
        let corner = |v: i32| u32::try_from(v).unwrap_or(0);
        Image {
            width,
            height,
            hot: (corner(self.hot.0), corner(self.hot.1)),
            rgba,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::collections::BTreeMap;
    use alloc::vec;
    use h3_assets::def::{Def, Encoding, FrameRef, Group};
    use h3_assets::palette::Palette;

    /// A 3 × 2 canvas with a 3 × 1 frame on its second row: a literal colour 9, a run of 0
    /// (transparent) and a run of 4 (a shadow's body).
    fn sprite() -> Sprite {
        let mut palette = Palette::default();
        palette.0[9] = Rgb { r: 255, g: 0, b: 0 };
        let frame = Frame {
            encoding: Encoding::Rows,
            full_width: 3,
            full_height: 2,
            left: 0,
            top: 1,
            width: 3,
            height: 1,
            pixels: vec![9, 0, 4],
            runs: vec![false, true, true],
            short_header: false,
        };
        Sprite {
            def: Def {
                kind: 0x46,
                width: 3,
                height: 2,
                palette,
                groups: vec![Group {
                    id: 0,
                    frames: vec![FrameRef {
                        name: "CURSOR".into(),
                        offset: 100,
                    }],
                }],
            },
            frames: BTreeMap::from([(100, frame)]),
        }
    }

    #[test]
    fn a_cursor_is_drawn_at_its_hot_spot() {
        let format = PixelFormat::RGB565;
        assert_eq!(Cursor::new(&sprite(), 1, (0, 0), format), None);
        let cursor = Cursor::new(&sprite(), 0, (1, 1), format).unwrap();
        let mut screen = Screen::new(4, 3, format).unwrap();
        screen.fill(0xFFFF);
        cursor.draw(&mut screen, 2, 2);
        // The canvas's top-left corner at (1, 1): its second row is the screen's third.
        assert_eq!(cursor.rect(2, 2), [1, 1, 4, 3]);
        assert_eq!(screen.pixel(1, 2), Some(0xF800));
        assert_eq!(screen.pixel(2, 2), Some(0xFFFF));
        assert_eq!(screen.pixel(3, 2), Some(format.shadow_body(0xFFFF)));
        assert_eq!(screen.pixels().iter().filter(|&&p| p != 0xFFFF).count(), 2);
        // Clipped at the edges.
        cursor.draw(&mut screen, -5, 9);
        cursor.draw(&mut screen, i32::MIN, i32::MAX);
        assert_eq!(screen.pixel(1, 2), Some(0xF800));
    }

    #[test]
    fn its_image_is_opaque_transparent_or_a_shadow() {
        let format = PixelFormat::RGB565;
        let image = Cursor::new(&sprite(), 0, (2, 1), format)
            .unwrap()
            .image(format);
        assert_eq!((image.width, image.height, image.hot), (3, 2, (2, 1)));
        let pixel = |i: usize| &image.rgba[i * 4..i * 4 + 4];
        // The first row: nothing drawn.
        assert_eq!(pixel(0), [0, 0, 0, 0]);
        assert_eq!(pixel(3), [255, 0, 0, 255]);
        assert_eq!(pixel(4), [0, 0, 0, 0]);
        // The shadow's body halves white: 0x7BEF, green 31 of 63 widened to 125.
        assert_eq!(pixel(5), [0, 0, 0, 130]);
        assert_eq!(image.rgba.len(), 24);
    }

    #[test]
    fn the_read_drawer_skips_the_runs() {
        let format = PixelFormat::RGB565;
        let cursor = Cursor::read(&sprite(), 0, (0, 0), format).unwrap();
        let image = cursor.image(format);
        // The literal pixel opaque, both runs skipped (transparent).
        assert_eq!(image.rgba[12..16], [255, 0, 0, 255]);
        assert_eq!(image.rgba[16..24], [0; 8]);
        assert!(Cursor::read(&sprite(), 5, (0, 0), format).is_none());
    }

    #[test]
    fn a_canvas_too_large_gives_an_empty_image() {
        let format = PixelFormat::RGB565;
        let mut big = sprite();
        big.frames
            .values_mut()
            .for_each(|f| f.full_width = u32::MAX);
        let image = Cursor::new(&big, 0, (u32::MAX, 0), format)
            .unwrap()
            .image(format);
        assert!(image.rgba.is_empty());
        assert_eq!(image.hot, (i32::MAX as u32, 0));
    }
}
