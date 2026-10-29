//! The screen the compositor draws into: 16-bit pixels in the format the executable gets from its
//! display surface (`docs/epics/E17-design.md` §3.2), the size a parameter (800 × 600 natively).
//!
//! The executable sets an 800 × 600 mode at 16 bits per pixel and takes each channel's width and
//! position from the surface's masks (0x55A6B0), packing red high and blue low; 565 on the hardware
//! of the last two decades. Colours reach the screen truncated to those widths (0x5226D0), and the
//! shadows work on the 16-bit pixel (0x47C460): this module holds those operations.

use alloc::vec;
use alloc::vec::Vec;

use h3_assets::palette::Rgb;

/// The longest side of a screen (*provisional*): bounds the memory a size can ask for.
pub const MAX_SIDE: u32 = 1 << 14;

/// A 16-bit pixel format: the widths of red, green and blue, packed red high, blue low.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PixelFormat {
    red: u8,
    green: u8,
    blue: u8,
}

impl PixelFormat {
    /// Five bits of red, six of green, five of blue: what the client models (*provisional*; the
    /// reference captures confirm what the user's setup reports, design §3.2).
    pub const RGB565: Self = Self {
        red: 5,
        green: 6,
        blue: 5,
    };

    /// Five bits each, as some early cards reported.
    pub const RGB555: Self = Self {
        red: 5,
        green: 5,
        blue: 5,
    };

    /// A format with these widths, each `1..=8`, together at most 16; `None` otherwise.
    #[must_use]
    pub fn new(red: u8, green: u8, blue: u8) -> Option<Self> {
        let fits = [red, green, blue].iter().all(|w| (1..=8).contains(w))
            && u32::from(red) + u32::from(green) + u32::from(blue) <= 16;
        fits.then_some(Self { red, green, blue })
    }

    /// The widths of red, green and blue.
    #[must_use]
    pub fn widths(self) -> (u8, u8, u8) {
        (self.red, self.green, self.blue)
    }

    /// A colour as a pixel: each channel's top bits, the rest dropped (0x5226D0, 0x44F010).
    #[must_use]
    pub fn pack(self, colour: Rgb) -> u16 {
        let top = |value: u8, width: u8| u16::from(value) >> (8 - width);
        top(colour.r, self.red) << (self.green + self.blue)
            | top(colour.g, self.green) << self.blue
            | top(colour.b, self.blue)
    }

    /// A colour scaled into the masks: each channel's value × its mask / 255, rounded down, kept
    /// to the mask (`DrawRay`'s colours, 0x5A55BF; E17 design §7.12). It can differ from
    /// [`PixelFormat::pack`] in the lowest bit.
    #[must_use]
    pub fn scale(self, colour: Rgb) -> u16 {
        let channel = |value: u8, width: u8, shift: u8| {
            let mask = ((1u32 << width) - 1) << shift;
            (u32::from(value) * mask / 255) & mask
        };
        let pixel = channel(colour.r, self.red, self.green + self.blue)
            | channel(colour.g, self.green, self.blue)
            | channel(colour.b, self.blue, 0);
        u16::try_from(pixel).unwrap_or(u16::MAX)
    }

    /// A pixel's colour with 8-bit channels, each channel's bits repeated downwards (a 5-bit 31
    /// becomes 255, 0 stays 0). For showing a screen (PNG) only: comparisons with the original
    /// stay in this format (design §3.2). *Provisional*.
    #[must_use]
    pub fn unpack(self, pixel: u16) -> Rgb {
        let channel = |shift: u8, width: u8| widen((pixel >> shift) & ((1 << width) - 1), width);
        Rgb {
            r: channel(self.green + self.blue, self.red),
            g: channel(self.blue, self.green),
            b: channel(0, self.blue),
        }
    }

    /// The red, green and blue masks (0xF800, 0x07E0, 0x001F in 565): the surface's, which the
    /// executable keeps at 0x694D68, 0x694D60 and 0x694D64 (E17 M3 T4a).
    #[must_use]
    pub fn channel_masks(self) -> [u32; 3] {
        let ones = |width: u8| (1u32 << width) - 1;
        [
            ones(self.red) << (self.green + self.blue),
            ones(self.green) << self.blue,
            ones(self.blue),
        ]
    }

    /// Each channel shifted down one, as the half mask leaves it (0x47C460: `0x7BEF` in 565).
    #[must_use]
    pub fn half_mask(self) -> u16 {
        self.mask(1)
    }

    /// Each channel shifted down two, as the quarter mask leaves it (`0x39E7` in 565).
    #[must_use]
    pub fn quarter_mask(self) -> u16 {
        self.mask(2)
    }

    fn mask(self, shift: u8) -> u16 {
        let ones = |width: u8| ((1u16 << width) - 1) >> shift;
        ones(self.red) << (self.green + self.blue) | ones(self.green) << self.blue | ones(self.blue)
    }

    /// A shadow's body over `pixel`: each channel halved, rounded down.
    #[must_use]
    pub fn shadow_body(self, pixel: u16) -> u16 {
        (pixel >> 1) & self.half_mask()
    }

    /// A shadow's border over `pixel`: each channel's quarter plus its half, each rounded down.
    #[must_use]
    pub fn shadow_border(self, pixel: u16) -> u16 {
        ((pixel >> 2) & self.quarter_mask()) + ((pixel >> 1) & self.half_mask())
    }
}

/// A `width`-bit value as eight bits, its bits repeated from the top.
fn widen(value: u16, width: u8) -> u8 {
    let mut out = 0u16;
    let mut filled = 0;
    while filled < 8 {
        out |= value << 8 >> (width + filled);
        filled += width;
    }
    u8::try_from(out & 0xFF).unwrap_or(u8::MAX)
}

/// The pixels the compositor draws into, row-major, top row first.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Screen {
    width: u32,
    height: u32,
    format: PixelFormat,
    pixels: Vec<u16>,
}

impl Screen {
    /// A black screen; `None` if a side exceeds [`MAX_SIDE`].
    #[must_use]
    pub fn new(width: u32, height: u32, format: PixelFormat) -> Option<Self> {
        (width <= MAX_SIDE && height <= MAX_SIDE).then(|| Self {
            width,
            height,
            format,
            pixels: vec![0; index(width) * index(height)],
        })
    }

    /// The width in pixels.
    #[must_use]
    pub fn width(&self) -> u32 {
        self.width
    }

    /// The height in pixels.
    #[must_use]
    pub fn height(&self) -> u32 {
        self.height
    }

    /// The pixel format.
    #[must_use]
    pub fn format(&self) -> PixelFormat {
        self.format
    }

    /// Every pixel, row-major, top row first.
    #[must_use]
    pub fn pixels(&self) -> &[u16] {
        &self.pixels
    }

    /// Every pixel for writing, row-major, top row first: for a drawer that walks the rows itself
    /// (the adventure map's, E24 M2 T4b3).
    pub fn pixels_mut(&mut self) -> &mut [u16] {
        &mut self.pixels
    }

    /// The pixel at `(x, y)`, if on the screen.
    #[must_use]
    pub fn pixel(&self, x: i64, y: i64) -> Option<u16> {
        self.at(x, y).and_then(|i| self.pixels.get(i).copied())
    }

    /// Every pixel widened to `0x00RRGGBB` by [`PixelFormat::unpack`], as the PNG shows it: what
    /// the window's texture takes (E17 M2 T5a, ADR-0024 point 3).
    #[must_use]
    pub fn xrgb(&self) -> Vec<u32> {
        self.pixels
            .iter()
            .map(|&p| xrgb_of(self.format, p))
            .collect()
    }

    /// The pixels of `rect` (left, top, right, bottom; the last two exclusive) as a screen of
    /// their own; `None` for a rectangle that shares no pixel with this screen. The rectangle is
    /// cut to the screen first (E24 M3 T2b1: what a window's cursor is drawn over).
    #[must_use]
    pub fn cut(&self, rect: [i64; 4]) -> Option<Self> {
        let [left, top, right, bottom] = self.inside(rect)?;
        let (width, height) = (right - left, bottom - top);
        let mut out = Self {
            width,
            height,
            format: self.format,
            pixels: vec![0; index(width) * index(height)],
        };
        let (w, from) = (index(width), index(self.width));
        for (row, y) in out.pixels.chunks_mut(w.max(1)).zip(top..bottom) {
            let start = index(y) * from + index(left);
            row.copy_from_slice(self.pixels.get(start..start + w).unwrap_or_default());
        }
        Some(out)
    }

    /// Copies `from`'s pixels in `rect` (left, top, right, bottom; the last two exclusive) to the
    /// same place here, the rectangle cut to both screens (E24 M3 T2b1: a kept frame's view
    /// restored from what lies under it). Nothing for screens of other formats.
    pub fn copy_rect(&mut self, from: &Self, rect: [i64; 4]) {
        if from.format != self.format {
            return;
        }
        let Some([left, top, right, bottom]) = self
            .inside(rect)
            .and_then(|r| from.inside(r.map(i64::from)))
        else {
            return;
        };
        let w = index(right - left);
        for y in top..bottom {
            let (to, at) = (
                index(y) * index(self.width) + index(left),
                index(y) * index(from.width) + index(left),
            );
            let src = from.pixels.get(at..at + w).unwrap_or_default();
            let dst = self.pixels.get_mut(to..to + src.len()).unwrap_or_default();
            dst.copy_from_slice(src.get(..dst.len()).unwrap_or_default());
        }
    }

    /// One step of a dissolve (`screen_dissolve` 0x602DC0, E17 M5 T5a; `docs/epics/E17-design.md`
    /// §7.25): in `rect` (left, top, right, bottom; the last two exclusive, cut to both screens)
    /// each pixel here, the picture saved before, moves `step` eighths of the way to `from`'s,
    /// the new picture: per channel `old + ((new - old) × step × 8 192 >> 16)` in 32-bit
    /// arithmetic (the product wraps, the shift is arithmetic: a step down rounds down), masked
    /// to the channel. Step 0 leaves the old picture, 8 or more gives the new. Nothing for screens
    /// of other formats.
    pub fn dissolve_rect(&mut self, from: &Self, rect: [i64; 4], step: u8) {
        if from.format != self.format {
            return;
        }
        let Some([left, top, right, bottom]) = self
            .inside(rect)
            .and_then(|r| from.inside(r.map(i64::from)))
        else {
            return;
        };
        // The weight in 1/65 536 (0x602ECB: the step shifted 16 left, then divided by 8).
        let weight = i32::from(step.min(8)) << 13;
        let masks = self
            .format
            .channel_masks()
            .map(|m| i32::try_from(m).unwrap_or(0));
        let w = index(right - left);
        for y in top..bottom {
            let (to, at) = (
                index(y) * index(self.width) + index(left),
                index(y) * index(from.width) + index(left),
            );
            let src = from.pixels.get(at..at + w).unwrap_or_default();
            let dst = self.pixels.get_mut(to..to + src.len()).unwrap_or_default();
            for (old, &new) in dst.iter_mut().zip(src) {
                let (o, n) = (i32::from(*old), i32::from(new));
                let blended = masks.iter().fold(0, |out, &m| {
                    let (o, n) = (o & m, n & m);
                    out | ((((n - o).wrapping_mul(weight)) >> 16) + o) & m
                });
                *old = u16::try_from(blended).unwrap_or_default();
            }
        }
    }

    /// `rect` cut to the screen, if anything of it is on it.
    fn inside(&self, [left, top, right, bottom]: [i64; 4]) -> Option<[u32; 4]> {
        let side = |v: i64, max: u32| u32::try_from(v.clamp(0, i64::from(max))).unwrap_or(0);
        let r = [
            side(left, self.width),
            side(top, self.height),
            side(right, self.width),
            side(bottom, self.height),
        ];
        (r[0] < r[2] && r[1] < r[3]).then_some(r)
    }

    /// Sets every pixel.
    pub fn fill(&mut self, pixel: u16) {
        self.pixels.fill(pixel);
    }

    /// Sets the pixel at `(x, y)`, if on the screen (a ray's pixel, written straight into the
    /// screen: E17 design §7.12).
    pub fn set(&mut self, x: i64, y: i64, pixel: u16) {
        if let Some(p) = self.pixel_mut(x, y) {
            *p = pixel;
        }
    }

    /// The pixel at `(x, y)` for writing, if on the screen.
    pub(crate) fn pixel_mut(&mut self, x: i64, y: i64) -> Option<&mut u16> {
        self.at(x, y).and_then(|i| self.pixels.get_mut(i))
    }

    fn at(&self, x: i64, y: i64) -> Option<usize> {
        let x = u32::try_from(x).ok().filter(|&x| x < self.width)?;
        let y = u32::try_from(y).ok().filter(|&y| y < self.height)?;
        Some(index(y) * index(self.width) + index(x))
    }
}

/// A `u32` as an index (the client's targets have at least 32-bit `usize`).
pub(crate) fn index(value: u32) -> usize {
    usize::try_from(value).unwrap_or(usize::MAX)
}

/// A pixel of `format` widened to `0x00RRGGBB` ([`PixelFormat::unpack`]).
fn xrgb_of(format: PixelFormat, pixel: u16) -> u32 {
    let c = format.unpack(pixel);
    u32::from(c.r) << 16 | u32::from(c.g) << 8 | u32::from(c.b)
}

/// Every pixel of a format widened to `0x00RRGGBB`, worked out once (E24 M3 T2b1,
/// *provisional*): what [`Screen::xrgb`] gives pixel by pixel, for a window that widens each
/// frame it shows (the per-channel loops of [`PixelFormat::unpack`] were most of a frame's cost).
/// 65 536 entries, 256 KiB.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Widening {
    format: PixelFormat,
    table: Vec<u32>,
}

impl Widening {
    /// The table of `format`.
    #[must_use]
    pub fn new(format: PixelFormat) -> Self {
        Self {
            format,
            table: (0..=u16::MAX).map(|p| xrgb_of(format, p)).collect(),
        }
    }

    /// The format it widens.
    #[must_use]
    pub fn format(&self) -> PixelFormat {
        self.format
    }

    /// A pixel widened.
    #[must_use]
    pub fn pixel(&self, pixel: u16) -> u32 {
        self.table.get(usize::from(pixel)).copied().unwrap_or(0)
    }

    /// `screen` widened into `out`, which is made its size: [`Screen::xrgb`] without a new
    /// vector.
    pub fn widen(&self, screen: &Screen, out: &mut Vec<u32>) {
        out.resize(screen.pixels.len(), 0);
        self.widen_onto(screen, [0, 0], out, screen.width);
    }

    /// `rect` of `screen` (cut to it) widened onto `out`, a widened frame of the screen's size
    /// (E24 M3 T2b2: only what changed since the frame `out` holds); nothing for an `out` of
    /// another size.
    pub fn widen_rect(&self, screen: &Screen, rect: [i64; 4], out: &mut [u32]) {
        if out.len() != screen.pixels.len() {
            return;
        }
        let Some([left, top, right, bottom]) = screen.inside(rect) else {
            return;
        };
        let (w, from, to) = (index(screen.width), index(left), index(right));
        for y in index(top)..index(bottom) {
            let src = screen
                .pixels
                .get(y * w + from..y * w + to)
                .unwrap_or_default();
            let dst = out.get_mut(y * w + from..y * w + to).unwrap_or_default();
            if screen.format == self.format {
                for (d, &p) in dst.iter_mut().zip(src) {
                    *d = self.pixel(p);
                }
            } else {
                for (d, &p) in dst.iter_mut().zip(src) {
                    *d = xrgb_of(screen.format, p);
                }
            }
        }
    }

    /// `screen` widened onto `out`, a frame `width` pixels wide, its top-left corner at `at`, cut
    /// to `out`'s rows and columns (a screen of another format by [`PixelFormat::unpack`]).
    pub fn widen_onto(&self, screen: &Screen, at: [i64; 2], out: &mut [u32], width: u32) {
        let (w, sw) = (i64::from(width), index(screen.width).max(1));
        let rows = i64::try_from(out.len()).unwrap_or(i64::MAX) / w.max(1);
        let (left, right) = (at[0].max(0), (at[0] + i64::from(screen.width)).min(w));
        if left >= right {
            return;
        }
        let from = usize::try_from(left - at[0]).unwrap_or(0);
        let n = usize::try_from(right - left).unwrap_or(0);
        for (r, row) in screen.pixels.chunks(sw).enumerate() {
            let y = at[1] + i64::try_from(r).unwrap_or(i64::MAX);
            if !(0..rows).contains(&y) {
                continue;
            }
            let start = usize::try_from(y * w + left).unwrap_or(0);
            let dst = out.get_mut(start..start + n).unwrap_or_default();
            let src = row.get(from..from + n).unwrap_or_default();
            if screen.format == self.format {
                for (d, &p) in dst.iter_mut().zip(src) {
                    *d = self.pixel(p);
                }
            } else {
                for (d, &p) in dst.iter_mut().zip(src) {
                    *d = xrgb_of(screen.format, p);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(non_snake_case)]
    fn RGB(r: u8, g: u8, b: u8) -> Rgb {
        Rgb { r, g, b }
    }

    #[test]
    fn a_dissolve_moves_each_channel_an_eighth_a_step_rounding_down() {
        let f = PixelFormat::RGB565;
        let screen = |pixels: &[u16]| {
            let mut s = Screen::new(2, 1, f).unwrap();
            s.pixels_mut().copy_from_slice(pixels);
            s
        };
        // Old white and black, new black and white: red 31 down to 0 and up from 0, and so on.
        let (old, new) = (screen(&[0xFFFF, 0]), screen(&[0, 0xFFFF]));
        let at = |step: u8| {
            let mut s = old.clone();
            s.dissolve_rect(&new, [0, 0, 2, 1], step);
            [s.pixels()[0], s.pixels()[1]]
        };
        assert_eq!(at(0), [0xFFFF, 0]);
        // Step 1: 31 − 3.875 rounds down to 27, 63 − 7.875 to 55; up 3.875 to 3, 7.875 to 7.
        assert_eq!(at(1), [27 << 11 | 55 << 5 | 27, 3 << 11 | 7 << 5 | 3]);
        // Step 7: red's product wraps in 32 bits (31 × 0x800 × 57 344 > 2³¹); its mask keeps
        // the channel exact: 31 × 7 / 8 = 27.125 up to 27, down to 3.875 rounding to 3.
        assert_eq!(at(7), [3 << 11 | 7 << 5 | 3, 27 << 11 | 55 << 5 | 27]);
        assert_eq!(at(8), [0, 0xFFFF]);
        assert_eq!(at(9), [0, 0xFFFF]);
        // Only the rectangle, cut to both screens; another format: nothing.
        let mut s = old.clone();
        s.dissolve_rect(&new, [1, 0, 5, 3], 8);
        assert_eq!(s.pixels(), &[0xFFFF, 0xFFFF]);
        let mut s = old.clone();
        s.dissolve_rect(&new, [2, 0, 5, 3], 8);
        assert_eq!(s.pixels(), old.pixels());
        let other = Screen::new(2, 1, PixelFormat::RGB555).unwrap();
        s.dissolve_rect(&other, [0, 0, 2, 1], 8);
        assert_eq!(s.pixels(), old.pixels());
    }

    #[test]
    fn formats_check_their_widths() {
        assert_eq!(PixelFormat::new(5, 6, 5), Some(PixelFormat::RGB565));
        assert_eq!(PixelFormat::new(5, 5, 5), Some(PixelFormat::RGB555));
        assert_eq!(PixelFormat::RGB565.widths(), (5, 6, 5));
        assert_eq!(PixelFormat::new(0, 8, 8), None);
        assert_eq!(PixelFormat::new(9, 1, 1), None);
        assert_eq!(PixelFormat::new(8, 8, 1), None);
        assert!(PixelFormat::new(8, 4, 4).is_some());
    }

    #[test]
    fn colours_are_truncated_into_the_format() {
        let f = PixelFormat::RGB565;
        assert_eq!(f.pack(RGB(255, 255, 255)), 0xFFFF);
        assert_eq!(f.pack(RGB(0, 0, 0)), 0);
        // The low bits go: 7 of red and blue, 3 of green.
        assert_eq!(f.pack(RGB(7, 3, 7)), 0);
        assert_eq!(f.pack(RGB(8, 4, 8)), 1 << 11 | 1 << 5 | 1);
        assert_eq!(f.pack(RGB(0xF8, 0, 0)), 0xF800);
        assert_eq!(PixelFormat::RGB555.pack(RGB(0, 0xF8, 0)), 0x03E0);
        assert_eq!(
            PixelFormat::new(8, 4, 4)
                .unwrap()
                .pack(RGB(0xAB, 0xCD, 0xEF)),
            0xABCE
        );
    }

    #[test]
    fn colours_are_scaled_into_the_masks() {
        let f = PixelFormat::RGB565;
        // The Arch Magi's ray (E17 design §7.12): as the original's screen shows it.
        assert_eq!(f.scale(RGB(152, 188, 24)), 0x95C2);
        assert_eq!(f.scale(RGB(124, 216, 124)), 0x7EAF);
        assert_eq!(f.scale(RGB(36, 180, 36)), 0x2584);
        assert_eq!(f.scale(RGB(255, 255, 255)), 0xFFFF);
        assert_eq!(f.scale(RGB(0, 0, 0)), 0);
        // Not the top bits: 152 packs to 0x9800 in red.
        assert_eq!(f.pack(RGB(152, 0, 0)), 0x9800);
        assert_eq!(f.scale(RGB(152, 0, 0)), 0x9000);
    }

    #[test]
    fn pixels_widen_by_repeating_their_bits() {
        let f = PixelFormat::RGB565;
        assert_eq!(f.unpack(0xFFFF), RGB(255, 255, 255));
        assert_eq!(f.unpack(0), RGB(0, 0, 0));
        // 5-bit 0b10000 -> 0b10000100, 6-bit 0b100000 -> 0b10000010.
        assert_eq!(f.unpack(0x8410), RGB(0x84, 0x82, 0x84));
        // A 1-bit channel fills all eight bits; a 3-bit one repeats twice and a bit.
        let odd = PixelFormat::new(1, 3, 8).unwrap();
        assert_eq!(
            odd.unpack(1 << 11 | 0b101 << 8 | 0x5A),
            RGB(0xFF, 0b1011_0110, 0x5A)
        );
        // Packing what unpacking gave returns the pixel.
        for pixel in [0x1234, 0xFFFF, 0x0841, 0xF81F] {
            assert_eq!(f.pack(f.unpack(pixel)), pixel);
        }
    }

    #[test]
    fn shadows_halve_and_quarter_each_channel() {
        let f = PixelFormat::RGB565;
        assert_eq!(f.half_mask(), 0x7BEF);
        assert_eq!(f.quarter_mask(), 0x39E7);
        assert_eq!(PixelFormat::RGB555.half_mask(), 0x3DEF);
        assert_eq!(PixelFormat::RGB555.quarter_mask(), 0x1CE7);
        assert_eq!(f.channel_masks(), [0xF800, 0x07E0, 0x001F]);
        assert_eq!(
            PixelFormat::RGB555.channel_masks(),
            [0x7C00, 0x03E0, 0x001F]
        );
        let white = 0xFFFF;
        // Body: 31 -> 15, 63 -> 31. Border: 31 -> 7 + 15 = 22, 63 -> 15 + 31 = 46.
        assert_eq!(f.shadow_body(white), 15 << 11 | 31 << 5 | 15);
        assert_eq!(f.shadow_border(white), 22 << 11 | 46 << 5 | 22);
        // Shadows compound and never carry into the next channel.
        assert_eq!(f.shadow_body(f.shadow_body(white)), 7 << 11 | 15 << 5 | 7);
        assert_eq!(f.shadow_border(1 << 11 | 1 << 5 | 1), 0);
    }

    #[test]
    fn a_screen_is_bounded_and_addressed() {
        assert!(Screen::new(MAX_SIDE + 1, 1, PixelFormat::RGB565).is_none());
        assert!(Screen::new(1, MAX_SIDE + 1, PixelFormat::RGB565).is_none());
        let mut s = Screen::new(3, 2, PixelFormat::RGB555).unwrap();
        assert_eq!(
            (s.width(), s.height(), s.format()),
            (3, 2, PixelFormat::RGB555)
        );
        assert_eq!(s.pixels(), [0; 6]);
        s.fill(7);
        *s.pixel_mut(2, 1).unwrap() = 9;
        assert_eq!(s.pixel(2, 1), Some(9));
        assert_eq!(s.pixel(0, 0), Some(7));
        for (x, y) in [(-1, 0), (3, 0), (0, -1), (0, 2)] {
            assert_eq!(s.pixel(x, y), None);
            assert!(s.pixel_mut(x, y).is_none());
            s.set(x, y, 5);
        }
        s.set(1, 0, 5);
        assert_eq!(s.pixel(1, 0), Some(5));
        assert_eq!(s.pixels().iter().filter(|p| **p == 5).count(), 1);
    }

    #[test]
    fn a_screen_widens_for_the_window() {
        let mut s = Screen::new(3, 1, PixelFormat::RGB565).unwrap();
        *s.pixel_mut(0, 0).unwrap() = 0xFFFF;
        *s.pixel_mut(1, 0).unwrap() = 0xF800;
        *s.pixel_mut(2, 0).unwrap() = 0x0841;
        // As the PNG widens: each channel's bits repeated downwards.
        assert_eq!(s.xrgb(), [0x00FF_FFFF, 0x00FF_0000, 0x0008_0808]);
    }

    #[test]
    fn the_widening_table_is_the_screens_own() {
        for format in [PixelFormat::RGB565, PixelFormat::RGB555] {
            let table = Widening::new(format);
            assert_eq!(table.format(), format);
            let mut every = Screen::new(256, 256, format).unwrap();
            for (i, p) in every.pixels_mut().iter_mut().enumerate() {
                *p = u16::try_from(i).unwrap();
            }
            // A vector of another size is made the screen's.
            let mut out = vec![7; 3];
            table.widen(&every, &mut out);
            assert_eq!(out, every.xrgb());
            assert_eq!(table.pixel(0xFFFF), 0x00FF_FFFF);
        }
    }

    #[test]
    fn a_screen_widens_onto_a_larger_frame_cut_to_it() {
        let table = Widening::new(PixelFormat::RGB565);
        let mut s = Screen::new(2, 2, PixelFormat::RGB565).unwrap();
        s.pixels_mut()
            .copy_from_slice(&[0xFFFF, 0xF800, 0x001F, 0x07E0]);
        let (white, red, blue, green) = (0x00FF_FFFF, 0x00FF_0000, 0x0000_00FF, 0x0000_FF00);
        // A 3 x 2 frame: the screen at (2, 1) keeps its top-left pixel only.
        let mut out = vec![0; 6];
        table.widen_onto(&s, [2, 1], &mut out, 3);
        assert_eq!(out, [0, 0, 0, 0, 0, white]);
        // At (-1, -1) only its bottom-right one.
        let mut out = vec![0; 6];
        table.widen_onto(&s, [-1, -1], &mut out, 3);
        assert_eq!(out, [green, 0, 0, 0, 0, 0]);
        // Wholly off the frame's sides, above or below it: nothing.
        let mut out = vec![0; 6];
        for at in [[3, 0], [-2, 0], [0, -2], [0, 2]] {
            table.widen_onto(&s, at, &mut out, 3);
        }
        assert_eq!(out, [0; 6]);
        // Inside, every pixel.
        table.widen_onto(&s, [0, 0], &mut out, 3);
        assert_eq!(out, [white, red, 0, blue, green, 0]);
        // A screen of another format widens as its own.
        let mut other = Screen::new(1, 1, PixelFormat::RGB555).unwrap();
        other.fill(0x7C00);
        table.widen_onto(&other, [2, 0], &mut out, 3);
        assert_eq!(out[2], red);
    }

    #[test]
    fn a_rectangle_of_a_screen_widens_in_place() {
        // E24 M3 T2b2: only the rectangle (cut to the screen), onto a frame of the screen's size.
        let table = Widening::new(PixelFormat::RGB565);
        let mut s = Screen::new(3, 2, PixelFormat::RGB565).unwrap();
        s.pixels_mut()
            .copy_from_slice(&[0xFFFF, 0xF800, 0x001F, 0x07E0, 0xFFFF, 0]);
        let (white, red, blue, green) = (0x00FF_FFFF, 0x00FF_0000, 0x0000_00FF, 0x0000_FF00);
        let mut out = vec![7; 6];
        table.widen_rect(&s, [1, -4, 9, 1], &mut out);
        assert_eq!(out, [7, red, blue, 7, 7, 7]);
        table.widen_rect(&s, [0, 1, 1, 2], &mut out);
        assert_eq!(out, [7, red, blue, green, 7, 7]);
        // Off the screen, or onto a frame of another size: nothing.
        table.widen_rect(&s, [3, 0, 5, 2], &mut out);
        let mut short = vec![7; 5];
        table.widen_rect(&s, [0, 0, 3, 2], &mut short);
        assert_eq!((out[4], short), (7, vec![7; 5]));
        // A screen of another format widens as its own.
        let mut other = Screen::new(3, 2, PixelFormat::RGB555).unwrap();
        other.fill(0x7C00);
        table.widen_rect(&other, [0, 0, 1, 1], &mut out);
        assert_eq!(out[0], red);
        table.widen(&s, &mut out);
        assert_eq!(out[4], white);
    }

    #[test]
    fn rectangles_are_cut_out_and_copied_in_place() {
        let mut s = Screen::new(3, 2, PixelFormat::RGB565).unwrap();
        s.pixels_mut().copy_from_slice(&[1, 2, 3, 4, 5, 6]);
        let cut = s.cut([1, 0, 9, 1]).unwrap();
        assert_eq!(
            (cut.width(), cut.height(), cut.pixels()),
            (2, 1, &[2, 3][..])
        );
        assert_eq!(s.cut([-5, -5, 1, 1]).unwrap().pixels(), [1]);
        for off in [[3, 0, 5, 2], [0, 2, 3, 4], [2, 0, 2, 2], [-3, -3, 0, 0]] {
            assert!(s.cut(off).is_none());
        }
        let mut to = Screen::new(3, 2, PixelFormat::RGB565).unwrap();
        to.copy_rect(&s, [1, 1, 3, 2]);
        assert_eq!(to.pixels(), [0, 0, 0, 0, 5, 6]);
        // Cut to the smaller screen.
        to.copy_rect(&cut, [0, 0, 3, 2]);
        assert_eq!(to.pixels(), [2, 3, 0, 0, 5, 6]);
        // Off both, or from another format: nothing.
        to.copy_rect(&s, [3, 0, 4, 2]);
        let other = Screen::new(3, 2, PixelFormat::RGB555).unwrap();
        to.copy_rect(&other, [0, 0, 3, 2]);
        assert_eq!(to.pixels(), [2, 3, 0, 0, 5, 6]);
    }

    #[test]
    fn the_pixels_are_written_row_by_row() {
        let mut s = Screen::new(3, 2, PixelFormat::RGB565).unwrap();
        s.pixels_mut()[4] = 9;
        assert_eq!(s.pixel(1, 1), Some(9));
        assert_eq!(s.pixels().iter().filter(|&&p| p != 0).count(), 1);
    }
}
