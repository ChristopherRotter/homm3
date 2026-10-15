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
            .map(|&p| {
                let c = self.format.unpack(p);
                u32::from(c.r) << 16 | u32::from(c.g) << 8 | u32::from(c.b)
            })
            .collect()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(non_snake_case)]
    fn RGB(r: u8, g: u8, b: u8) -> Rgb {
        Rgb { r, g, b }
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
}
