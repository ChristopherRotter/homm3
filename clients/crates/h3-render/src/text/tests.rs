use super::*;
use crate::screen::PixelFormat;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use h3_assets::fnt::Glyph;

/// The screen colours of the entries the tests draw in.
const PLAIN: u16 = 0x1111;
const BRIGHT: u16 = 0x2222;
const SHADE: u16 = 0x3333;

/// `game.pal` as the tests see it: entry 13 (colour 4, the count box's) plain, 14 bright, 32 the
/// shadow.
fn colours() -> ScreenPalette {
    let mut p = [0u16; 256];
    (p[13], p[14], p[SHADOW]) = (PLAIN, BRIGHT, SHADE);
    ScreenPalette(p)
}

fn glyph(left: i32, width: i32, right: i32, pixels: &[u8]) -> Glyph {
    Glyph {
        left,
        width,
        right,
        pixels: pixels.to_vec(),
    }
}

/// A font one row high: `x` one inked pixel, `w` ink and shadow then a gap (advance 3), a space
/// of one blank pixel, `n` two inked pixels from one left of the pen (advance 1), `r` one inked
/// pixel with the pen moved back (advance 0).
fn font() -> Font {
    let mut glyphs = vec![Glyph::default(); 256];
    glyphs[usize::from(b'x')] = glyph(0, 1, 0, &[INK]);
    glyphs[usize::from(b'w')] = glyph(0, 2, 1, &[INK, 1]);
    glyphs[usize::from(b' ')] = glyph(0, 1, 0, &[0]);
    glyphs[usize::from(b'n')] = glyph(-1, 2, 0, &[INK, INK]);
    glyphs[usize::from(b'r')] = glyph(0, 1, -1, &[INK]);
    Font {
        height: 1,
        y_offset: 0,
        glyphs,
    }
}

/// The same `x` two rows high.
fn tall() -> Font {
    let mut f = font();
    f.height = 2;
    f.glyphs[usize::from(b'x')] = glyph(0, 1, 0, &[INK, INK]);
    f
}

/// The screen's rows after drawing: `A` plain, `H` bright, `s` shadow, `.` nothing.
fn rows(screen: &Screen) -> Vec<String> {
    let width = usize::try_from(screen.width()).unwrap();
    screen
        .pixels()
        .chunks(width)
        .map(|row| {
            row.iter()
                .map(|&p| match p {
                    PLAIN => 'A',
                    BRIGHT => 'H',
                    SHADE => 's',
                    _ => '.',
                })
                .collect()
        })
        .collect()
}

/// `text` drawn in `font` into a `w` × `h` box at the top-left of a screen of that size, with the
/// executable's bugs (the tests below describe the executable, but where they say otherwise).
fn drawn(font: &Font, text: &str, size: [i32; 2], colour: u32, alignment: u32) -> Vec<String> {
    drawn_with(Bugs::Original, font, text, size, colour, alignment)
}

/// As [`drawn`], the long word's bug fixed.
fn fixed(font: &Font, text: &str, size: [i32; 2], alignment: u32) -> Vec<String> {
    drawn_with(Bugs::Fixed, font, text, size, 4, alignment)
}

fn drawn_with(
    bugs: Bugs,
    font: &Font,
    text: &str,
    [w, h]: [i32; 2],
    colour: u32,
    alignment: u32,
) -> Vec<String> {
    let size = |v: i32| u32::try_from(v).unwrap();
    let mut screen = Screen::new(size(w), size(h), PixelFormat::RGB565).unwrap();
    let area = TextBox {
        x: 0,
        y: 0,
        width: w,
        height: h,
    };
    let style = TextStyle {
        font,
        colours: &colours(),
        colour,
        alignment,
        bugs,
    };
    draw_text(&mut screen, text.as_bytes(), area, style);
    rows(&screen)
}

#[test]
fn a_line_is_placed_by_its_alignment() {
    let f = font();
    assert_eq!(drawn(&f, "xwx", [10, 1], 4, 0), ["AAs.A....."]);
    assert_eq!(drawn(&f, "xwx", [10, 1], 4, align::CENTRE), ["..AAs.A..."]);
    assert_eq!(drawn(&f, "xwx", [10, 1], 4, align::RIGHT), [".....AAs.A"]);
    // Any other horizontal value is left.
    assert_eq!(drawn(&f, "xwx", [10, 1], 4, 3), ["AAs.A....."]);
    // The text ends at its first NUL.
    assert_eq!(drawn(&f, "x\0x", [4, 1], 4, 0), ["A..."]);
    assert_eq!(drawn(&f, "", [4, 1], 4, 0), ["...."]);
}

#[test]
fn braces_highlight_and_colours_name_their_entries() {
    let f = font();
    assert_eq!(drawn(&f, "x{x}x", [5, 1], 4, 0), ["AHA.."]);
    assert_eq!(drawn(&f, "{x", [3, 1], 4, align::CENTRE), [".H."]);
    assert_eq!(drawn(&f, "x", [2, 1], RAW_ENTRY | 14, 0), ["H."]);
    assert_eq!(colour_entry(4), 13);
    assert_eq!(colour_entry(RAW_ENTRY | 7), 7);
    // An entry past the palette draws its ink in nothing; the shadow still shows.
    assert_eq!(drawn(&f, "w", [3, 1], 255, 0), [".s."]);
}

#[test]
fn lines_break_at_newlines_and_spaces() {
    let f = font();
    assert_eq!(drawn(&f, "x\nxx", [4, 3], 4, 0), ["A...", "AA..", "...."]);
    assert_eq!(
        drawn(&f, "xxx xxx", [5, 3], 4, 0),
        ["AAA..", "AAA..", "....."]
    );
    // At the box's last line the break comes at the glyph that lets it fit, spaces or not.
    assert_eq!(
        drawn(&f, "xx xxxx", [5, 3], 4, 0),
        ["AA...", "AAAA.", "....."]
    );
    assert_eq!(drawn(&f, "xx xxxx", [5, 1], 4, 0), ["AA.A."]);
    // A long word breaks where the first glyph from its end lets it fit; that glyph is lost
    // and the line keeps the whole width (centred: (5 - 6) / 2, toward zero).
    assert_eq!(drawn(&f, "xxxxxxxx", [5, 2], 4, 0), ["AAAA.", "AAA.."]);
    assert_eq!(
        drawn(&f, "xxxxxxxx", [5, 2], 4, align::CENTRE),
        ["AAAA.", ".AAA."]
    );
    // Its first glyph too wide alone: nothing of it, the rest on the next line.
    assert_eq!(drawn(&f, "wx", [2, 2], 4, 0), ["..", "A."]);
    // A space last when it passes the box: the line ends before it.
    assert_eq!(drawn(&f, "xxx xx", [3, 3], 4, 0), ["AAA", "AA.", "..."]);
    // Braces count for nothing on the way back (the glyph lost is the third `x`).
    assert_eq!(drawn(&f, "xx{x}xx", [3, 3], 4, 0), ["AA.", "AA.", "..."]);
    // A text of braces only draws nothing.
    assert_eq!(drawn(&f, "{}", [3, 1], 4, 0), ["..."]);
}

#[test]
fn the_long_words_bug_is_fixed_but_for_comparisons() {
    // The user (2026-10-15): the glyph at a long word's break starts the next line, and the line
    // is aligned by its own width; the executable's loss only in the comparisons' mode.
    let f = font();
    let word = "xxxxxxxx";
    assert_eq!(
        drawn(&f, word, [6, 2], 4, align::CENTRE),
        ["AAAAA.", "..AA.."]
    );
    assert_eq!(fixed(&f, word, [6, 2], align::CENTRE), ["AAAAA.", ".AAA.."]);
    assert_eq!(
        drawn(&f, word, [6, 2], 4, align::RIGHT),
        ["AAAA..", "....AA"]
    );
    assert_eq!(fixed(&f, word, [6, 2], align::RIGHT), [".AAAAA", "...AAA"]);
    let ten = "xxxxxxxxxx";
    assert_eq!(drawn(&f, ten, [5, 3], 4, 0), ["AAAA.", "AAAAA", "....."]);
    assert_eq!(fixed(&f, ten, [5, 3], 0), ["AAAA.", "AAAA.", "AA..."]);
    // A first glyph wider than the box takes its line alone (not drawn: it passes the edge).
    assert_eq!(fixed(&f, "wx", [2, 2], 0), ["..", "A."]);
    // Spaces break as before.
    assert_eq!(fixed(&f, "xx xxxx", [5, 3], 0), ["AA...", "AAAA.", "....."]);
    assert_eq!(Bugs::default(), Bugs::Fixed);
}

#[test]
fn negative_spaces_shift_and_widen_lines() {
    let f = font();
    // The first glyph's negative left space: the line starts that much to the right.
    assert_eq!(drawn(&f, "nx", [10, 1], 4, 0), ["AAA......."]);
    assert_eq!(drawn(&f, "nx", [10, 1], 4, align::CENTRE), ["...AAA...."]);
    // The last glyph's negative right space widens the line: centred at (9 - 2) / 2.
    assert_eq!(drawn(&f, "xr", [9, 1], 4, align::CENTRE), ["...AA...."]);
    // Too wide with it: the line breaks as if it were not there.
    assert_eq!(drawn(&f, "xxr", [2, 3], 4, 0), ["A.", "A.", ".."]);
    assert_eq!(drawn(&f, "xxr", [2, 1], 4, 0), ["A."]);
}

#[test]
fn lines_outside_the_box_are_not_drawn() {
    let t = tall();
    // Centred in the height, or at the bottom.
    let empty = || vec![String::from("..."); 7];
    let mut middle = empty();
    (middle[2], middle[3]) = ("A..".into(), "A..".into());
    assert_eq!(drawn(&t, "x", [3, 7], 4, align::MIDDLE), middle);
    let mut bottom = empty();
    (bottom[5], bottom[6]) = ("A..".into(), "A..".into());
    assert_eq!(drawn(&t, "x", [3, 7], 4, align::BOTTOM), bottom);
    // Too many lines for the box: the first at the top, the next stops the drawing.
    assert_eq!(
        drawn(&t, "x\nx", [3, 3], 4, align::MIDDLE),
        ["A..", "A..", "..."]
    );
    // More lines than fit but a box at least two lines high: from the top.
    assert_eq!(
        drawn(&t, "x\nx\nx", [3, 4], 4, align::MIDDLE),
        ["A..", "A..", "A..", "A.."]
    );
    assert_eq!(
        drawn(&t, "x\nx\nx", [3, 4], 4, align::BOTTOM),
        ["A..", "A..", "A..", "A.."]
    );
    // A box lower than a line: nothing.
    assert_eq!(drawn(&t, "x", [3, 1], 4, align::MIDDLE), ["..."]);
    // A font with no rows draws nothing, and stops.
    let mut flat = font();
    flat.height = 0;
    assert_eq!(drawn(&flat, "x\nx", [3, 2], 4, 0), ["...", "..."]);
}

#[test]
fn the_y_offset_moves_lines_down() {
    let mut f = font();
    f.y_offset = 1;
    assert_eq!(drawn(&f, "x\nx", [2, 3], 4, 0), ["..", "A.", "A."]);
}

#[test]
fn a_box_taller_than_the_screen_stops_at_its_bottom() {
    let f = font();
    let mut screen = Screen::new(2, 2, PixelFormat::RGB565).unwrap();
    let area = TextBox {
        x: 0,
        y: 0,
        width: 2,
        height: i32::MAX,
    };
    let style = TextStyle {
        font: &f,
        colours: &colours(),
        colour: 4,
        alignment: 0,
        bugs: Bugs::Fixed,
    };
    draw_text(&mut screen, b"x\nx\nx\nx", area, style);
    assert_eq!(rows(&screen), ["A.", "A."]);
}

#[test]
fn the_line_drawer_skips_glyphs_left_of_the_box_and_stops_at_its_right() {
    let f = font();
    let mut screen = Screen::new(6, 1, PixelFormat::RGB565).unwrap();
    let area = TextBox {
        x: 1,
        y: 0,
        width: 3,
        height: 1,
    };
    let text = b"xxxxxx";
    let line = Line {
        from: 0,
        len: 6,
        x: -1,
        y: 0,
    };
    draw_line(&mut screen, &f, &colours(), text, line, area, 13);
    assert_eq!(rows(&screen), [".AAA.."]);
    // A line above the box is not drawn.
    let mut screen = Screen::new(6, 2, PixelFormat::RGB565).unwrap();
    let low = TextBox { y: 1, ..area };
    draw_line(&mut screen, &f, &colours(), text, line, low, 13);
    assert_eq!(rows(&screen), ["......", "......"]);
}

#[test]
fn lines_are_counted_as_the_executable_counts_them() {
    let f = font();
    assert_eq!(line_count(&f, b"", 5, Bugs::Original), 0);
    assert_eq!(line_count(&f, b"xx xxxx", 5, Bugs::Original), 2);
    assert_eq!(line_count(&f, b"xxxxxxxx", 5, Bugs::Original), 2);
    assert_eq!(line_count(&f, b"x\n\nx", 5, Bugs::Original), 3);
    assert_eq!(line_count(&f, b"x{x}x", 5, Bugs::Original), 1);
    assert_eq!(line_count(&f, b"xxx xx", 5, Bugs::Original), 2);
    assert_eq!(line_count(&f, b"xxxxx ", 5, Bugs::Original), 1);
    assert_eq!(line_count(&f, b"{xxxxxxx", 5, Bugs::Original), 2);
    // The long word fixed: the glyph at the break starts the next line.
    let ten = b"xxxxxxxxxx";
    assert_eq!(line_count(&f, ten, 5, Bugs::Original), 2);
    assert_eq!(line_count(&f, ten, 5, Bugs::Fixed), 3);
    assert_eq!(line_count(&f, b"wx", 2, Bugs::Fixed), 2);
    assert_eq!(line_count(&f, b"xx xxxx", 5, Bugs::Fixed), 2);
    assert_eq!(byte(b"ab", -1), 0);
    assert_eq!(byte(b"ab", 2), 0);
}
