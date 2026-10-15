//! Text in the original's bitmap fonts (E17 M3 T4a, `docs/epics/E17-design.md` §3.6): the
//! executable's text drawer `TextDraw` (0x4B5490) with its line breaking, its line drawer
//! (0x4B5260), its glyph drawer (0x4B51A0) and the line count it centres by (0x4B5820), ported as
//! read, quirks included.
//!
//! A text is bytes in the game's code page, ending at its first NUL. `{` and `}` switch the
//! highlight on and off: a highlighted glyph takes the next colour entry; neither is drawn nor
//! counted in a line's width. A line ends at a newline, at the text's end or where the next glyph
//! would pass the box's width: then it breaks after the last space that lets it fit, else (one long
//! word) where the first glyph from its end lets it fit, and that glyph is lost; the box's last
//! line breaks at that glyph whatever the spaces. A line whose glyphs would pass the box's bottom
//! is not drawn, and drawing stops at the first line that does not fit below one already drawn.
//!
//! What the executable does that this does not: a caret (`_`) for an input field (the drawer's
//! last argument, −1 in the battle); writing past the screen's edge (the glyph drawer does not
//! clip, so its callers keep texts inside it; here the screen clips); and a line that starts with
//! a glyph of 128 or above with a negative left space, or ends too wide with one with a negative
//! right space: there the line's width takes that space at the glyph's value minus 256 (a
//! sign-extended index, before the metrics: no such line is drawn yet; here the glyph's own). A
//! font of height 0 draws nothing (the executable would loop for ever on a text of more lines
//! than its box holds).

use h3_assets::fnt::{Font, INK};

use crate::palette::ScreenPalette;
use crate::screen::Screen;

/// The colour entry a glyph pixel other than 0 and [`INK`] takes (the font's colours at 0x1098,
/// entry 32 of `game.pal`: black).
pub const SHADOW: usize = 32;

/// What `TextDraw`'s colour argument adds to reach its entry of `game.pal`, unless it carries
/// [`RAW_ENTRY`] (0x4B54C6).
pub const FIRST_COLOUR: u32 = 9;

/// A colour argument with this bit names its entry itself, the bit cleared.
pub const RAW_ENTRY: u32 = 0x100;

/// The box a text is drawn in: its top-left corner and size, as `TextDraw`'s arguments.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextBox {
    /// The left edge.
    pub x: i32,
    /// The top edge.
    pub y: i32,
    /// The width the lines break at and are aligned in.
    pub width: i32,
    /// The height lines must fit in.
    pub height: i32,
}

/// `TextDraw`'s alignment bits: the rest of the value aligns each line (0 left, 1 centred, 2
/// right, any other left).
pub mod align {
    /// The lines centred in the box's height.
    pub const MIDDLE: u32 = 4;
    /// The lines at the box's bottom.
    pub const BOTTOM: u32 = 8;
    /// Each line centred in the box's width.
    pub const CENTRE: u32 = 1;
    /// Each line at the box's right edge.
    pub const RIGHT: u32 = 2;
}

/// The entry of `game.pal` a colour argument draws in (0x4B54C6).
#[must_use]
pub fn colour_entry(colour: u32) -> u32 {
    if colour & RAW_ENTRY == 0 {
        colour.wrapping_add(FIRST_COLOUR)
    } else {
        colour & !RAW_ENTRY
    }
}

/// Draws `text` in `font` into `area` of `screen`: `TextDraw` (0x4B5490) with the colour argument
/// `colour` ([`colour_entry`]) and the alignment `alignment` ([`align`]); `colours` is `game.pal`
/// in the screen's format (the font's copy, 0x55B817).
pub fn draw_text(
    screen: &mut Screen,
    font: &Font,
    colours: &ScreenPalette,
    text: &[u8],
    area: TextBox,
    colour: u32,
    alignment: u32,
) {
    let text = text.split(|&b| b == 0).next().unwrap_or_default();
    let len = to_i64(text.len());
    let at = |i: i64| byte(text, i);
    let advance = |c: u8| i64::from(font.glyph(c).advance());
    let fh = i64::from(font.height);
    let (w, h) = (i64::from(area.width), i64::from(area.height));
    let mut alignment = alignment;
    let mut top = 0;
    if alignment & align::MIDDLE != 0 {
        alignment &= !align::MIDDLE;
        let total = line_count(font, text, w) * fh;
        if total < h {
            top = (h - total) / 2;
        } else if h < 2 * fh {
            top = (h - fh) / 2;
        }
    }
    if alignment & align::BOTTOM != 0 {
        alignment &= !align::BOTTOM;
        let total = line_count(font, text, w) * fh;
        if total < h {
            top = h - total;
        }
    }
    let colour = colour_entry(colour);
    let screen_bottom = i64::from(screen.height()) - i64::from(area.y) - i64::from(font.y_offset);
    let mut i = 0;
    // Each line (0x4B5598): it stops at the text's end or below the box. The screen's bottom
    // stops it too: lines only move down, and none past it shows.
    while fh > 0 && i < len && at(i) != 0 && !(top + fh > h && top != 0) && top < screen_bottom {
        let start = i;
        let mut width = 0;
        while matches!(at(i), b'{' | b'}') {
            i += 1;
        }
        let first = at(i);
        if first != 0 {
            let left = i64::from(font.glyph(first).left);
            if left < 0 {
                width = -left;
            }
            let mut c = first;
            while c != 0 && c != b'\n' && width <= w {
                if !matches!(c, b'{' | b'}') {
                    width += advance(c);
                }
                i += 1;
                c = at(i);
            }
        }
        // The last glyph measured, before any braces (0x4B5631).
        let mut last = i - 1;
        while matches!(at(last), b'{' | b'}') && last > start {
            last -= 1;
        }
        let right = i64::from(font.glyph(at(last)).right);
        if i > 0 && right < 0 {
            width -= right;
        }
        if width > w {
            // Too wide (0x4B5666): back to a space where it fits.
            let whole = width;
            let mut fits = 0;
            let mut k = last;
            if right < 0 {
                width += right;
            }
            if at(k) != b' ' {
                while k >= start {
                    let c = at(k);
                    if !matches!(c, b'{' | b'}') {
                        width -= advance(c);
                        if top + 2 * fh > h && width < w {
                            // The box's last line: cut at this glyph (0x4B56D5).
                            break;
                        }
                        if fits == 0 && width < w {
                            fits = k;
                        }
                    }
                    k -= 1;
                    if at(k) == b' ' {
                        break;
                    }
                }
            }
            if k <= start {
                // No space: the first glyph from the end that lets it fit (0x4B56F7).
                k = fits;
                width = whole;
            }
            if at(k) == b' ' {
                width -= advance(b' ');
            }
            i = k;
        }
        let offset = match alignment {
            align::CENTRE => (w - width) / 2,
            align::RIGHT => w - width,
            _ => 0,
        };
        let line = Line {
            from: start,
            len: i - start,
            x: i64::from(area.x) + offset,
            y: i64::from(area.y) + top,
        };
        draw_line(screen, font, colours, text, line, area, colour);
        top += fh;
        i += 1;
    }
}

/// A line's place in its text and on the screen.
#[derive(Debug, Clone, Copy)]
struct Line {
    from: i64,
    len: i64,
    x: i64,
    y: i64,
}

/// The line drawer (0x4B5260): down by the font's y offset, right by a negative left space of its
/// first glyph; not drawn unless it lies inside the box's height; glyphs left of the box skipped;
/// it ends before the first glyph that would pass the box's right edge.
fn draw_line(
    screen: &mut Screen,
    font: &Font,
    colours: &ScreenPalette,
    text: &[u8],
    line: Line,
    area: TextBox,
    colour: u32,
) {
    let at = |i: i64| byte(text, line.from + i);
    let (mut x, y) = (line.x, line.y + i64::from(font.y_offset));
    let first = font.glyph(at(0));
    if at(0) != 0 && first.left < 0 {
        x -= i64::from(first.left);
    }
    let (left, top) = (i64::from(area.x), i64::from(area.y));
    let (right, bottom) = (left + i64::from(area.width), top + i64::from(area.height));
    if y < top || y + i64::from(font.height) > bottom {
        return;
    }
    let mut n = 0;
    while n < line.len && x + i64::from(font.glyph(at(n)).left) < left {
        x += i64::from(font.glyph(at(n)).advance());
        n += 1;
    }
    let mut highlight = 0;
    while n < line.len {
        let c = at(n);
        match c {
            b'{' => highlight = 1,
            b'}' => highlight = 0,
            _ => {
                let advance = i64::from(font.glyph(c).advance());
                if x + advance > right {
                    return;
                }
                glyph(
                    screen,
                    font,
                    colours,
                    c,
                    [x, y],
                    colour.wrapping_add(highlight),
                );
                x += advance;
            }
        }
        n += 1;
    }
}

/// The glyph drawer (0x4B51A0): its pixels at `x` plus its left space, 0 not drawn, [`INK`] in
/// `entry`, any other in [`SHADOW`]'s colour. An entry beyond the palette draws nothing (the
/// executable reads the font's next fields; no caller passes one).
fn glyph(
    screen: &mut Screen,
    font: &Font,
    colours: &ScreenPalette,
    c: u8,
    [x, y]: [i64; 2],
    entry: u32,
) {
    let g = font.glyph(c);
    let width = usize::try_from(g.width).unwrap_or(0).max(1);
    let ink = usize::try_from(entry)
        .ok()
        .and_then(|e| colours.0.get(e))
        .copied();
    let x = x + i64::from(g.left);
    for (i, &v) in g.pixels.iter().enumerate() {
        let pixel = match v {
            0 => None,
            INK => ink,
            _ => colours.0.get(SHADOW).copied(),
        };
        if let Some(p) = pixel {
            screen.set(x + to_i64(i % width), y + to_i64(i / width), p);
        }
    }
}

/// How many lines `text` takes at `width` (0x4B5820): its own breaking, simpler than the
/// drawer's (no negative spaces, no last line).
fn line_count(font: &Font, text: &[u8], width: i64) -> i64 {
    let len = to_i64(text.len());
    let at = |i: i64| byte(text, i);
    let advance = |c: u8| i64::from(font.glyph(c).advance());
    let (mut count, mut i) = (0, 0);
    while i < len && at(i) != 0 {
        let start = i;
        let mut used = 0;
        let mut c = at(i);
        while c != b'\n' && used <= width {
            if !matches!(c, b'{' | b'}') {
                used += advance(c);
            }
            i += 1;
            c = at(i);
            if c == 0 {
                break;
            }
        }
        if used > width {
            let mut fits = 0;
            i -= 1;
            if at(i) != b' ' {
                while i >= start {
                    let c = at(i);
                    if !matches!(c, b'{' | b'}') {
                        used -= advance(c);
                        if fits == 0 && used < width {
                            fits = i;
                        }
                    }
                    i -= 1;
                    if at(i) == b' ' {
                        break;
                    }
                }
            }
            if i <= start {
                i = fits;
            }
        }
        count += 1;
        i += 1;
    }
    count
}

/// The byte at `i`, 0 outside the text (where the executable reads its neighbours: the text's
/// NUL after it; before it, only when a line starts with a newline, whose width is 0 anyway).
fn byte(text: &[u8], i: i64) -> u8 {
    usize::try_from(i)
        .ok()
        .and_then(|i| text.get(i))
        .copied()
        .unwrap_or(0)
}

fn to_i64(value: usize) -> i64 {
    i64::try_from(value).unwrap_or(i64::MAX)
}

#[cfg(test)]
mod tests;
