//! Credits (E27 M4 T2; `docs/epics/E27-design.md` §2.9, §36): `Credits.txt`'s text scrolled up
//! through a window over the main menu's background, its closing text near the end, a key or a
//! press ending it.
//!
//! - **The screen** (`game_main`'s Credits case 0x4EF12D, `credits_screen` 0x4EDDA0): the main
//!   menu's dialog closed (the screen shown without its buttons), `mainmenu.pcx` copied over
//!   where the buttons were (opaque, at (460, 0): the same pixels as `GamSelBk.pcx`'s there in the
//!   installation); `Credits.txt`'s record 1 drawn whole into a picture of its own the window's
//!   width (328) wide and its lines' height and a line high, in `Credits.fnt`, colour 0x11C,
//!   alignment 5 (so a half line above its first line), the picture filled first with the 16-bit
//!   colour its copies skip (`h3_ui::ui::Draw::Part`). The window: (460, 10, 328, 580).
//! - **The steps** (*recorded*, §36.2: every frame of `credits-scroll` and `credits-end` as the
//!   original's): one every 100 ms of the clock (CLT-Q88 (a): the menus' placeholder clip's ten
//!   frames a second, no clip), the first 100 ms after the screen opens. Each puts the window's
//!   background back and moves the text [`CreditsLook::step`] (2) pixels: first in from the
//!   window's bottom (580 rows to go: the picture's top rows shown at the window's bottom), then
//!   up (its first row shown moving on) while that row is above the text's height less the
//!   window's, then out at the top: the rows shown fewer by the step each time, the closing text
//!   (record 2, `smalfont.fnt`, colour 0xD, alignment 8: its lines at the window's bottom, left
//!   aligned) drawn over the window once they are under [`CreditsLook::closing_below`] (435;
//!   first at the 73rd step of the end); the step after the rows went below 0 ends the screen
//!   (*recorded*: the main menu shows at that step, `credits-scroll`'s `menu`).
//! - **The inputs** (the loop's `input_next_message`, its queue flushed as it starts): a key
//!   down but F4 (`super::start::key_skips`), a left press and a right press end it at once; a
//!   move, a release and a key up do nothing. The main menu then opens anew, its last items as
//!   the credits found them.
//!
//! **The background's key** (*recorded*, §36.3): the original saves the window's background
//! from the screen once into a black picture with its copies' key (the 16-bit colour 1, skipped),
//! and puts it back at each step, so a pixel of that colour shows black from the first step:
//! one of `mainmenu.pcx`'s, (463, 553). Fixed (a drawing bug: E17's binding decisions, as
//! CLT-Q138's): the client's profile draws the background whole; the original's
//! ([`Bugs::Original`]) for the comparisons only (`h3_ui::ui::Draw::Keyed`).

use alloc::string::String;
use alloc::vec::Vec;

use h3_assets::fnt::Font;
use h3_render::Bugs;
use h3_ui::ui::{Draw, Label};
use serde::Deserialize;

use super::Ended;
use super::keys::Context;
use super::menu::{Did, Hovers};

/// The credits' look (the shell's layout `[credits]`).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreditsLook {
    /// The picture copied over the main menu's buttons as the credits open (`main_menu_picture`
    /// 0x699514: `mainmenu.pcx`), at [`CreditsLook::at`].
    pub picture: String,
    /// Its corner: (460, 0).
    pub at: [i32; 2],
    /// The window the text moves through (left, top, width, height): (460, 10, 328, 580); its
    /// width the text's.
    pub window: [i32; 4],
    /// The text's font: `Credits.fnt`.
    pub font: String,
    /// The palette the fonts' colours come from: `game.pal`.
    pub font_palette: String,
    /// The text drawer's colour: 0x11C (`game.pal`'s entry 28).
    pub colour: u32,
    /// Its alignment: 5 (each line centred, the lines in the middle of the text's picture).
    pub align: u32,
    /// The closing text's font: `smalfont.fnt` (`small_font` 0x698A08).
    pub closing_font: String,
    /// Its colour: 0xD.
    pub closing_colour: u32,
    /// Its alignment: 8 (the lines at the window's bottom).
    pub closing_align: u32,
    /// The closing text shows once the leaving text's rows are fewer: 435 (0x1B3).
    pub closing_below: i32,
    /// The pixels the text moves each step: 2.
    pub step: i32,
    /// The milliseconds from a step to the next, and from the opening to the first: 100.
    pub every: u32,
}

/// The texts (the user's tables', the catalogue's `shell.credits.*`): `Credits.txt`'s record 1
/// and record 2.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Texts {
    /// The credits.
    pub text: String,
    /// The closing text.
    pub closing: String,
}

/// The credits: the text's place in the window, the clock's next step.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Credits {
    look: CreditsLook,
    texts: Texts,
    /// The text's lines' height (`text_line_count` at the window's width × the font's height).
    height: i32,
    /// The font's line height: the text's picture is a line more.
    line: i32,
    /// The rows still below the window while the text comes in (the window's height at the
    /// start, `credits_screen`'s +0x1C).
    entering: i32,
    /// The text's first row shown (+0x14).
    top: i32,
    /// The rows shown while it leaves (the window's height at the start, +0x24).
    leaving: i32,
    /// The clock's time of the next step, from its first reading.
    due: Option<u32>,
    /// The menus' last items, as the credits found them.
    hovers: Hovers,
    /// The original's background's key or the client's whole one.
    bugs: Bugs,
}

impl Credits {
    /// The credits as they open, `look` the layout's, the texts' `height` their lines' (at the
    /// window's width) and `line` the font's line height, in `bugs`' profile.
    #[must_use]
    pub fn new(look: CreditsLook, texts: Texts, (height, line): (i32, i32), bugs: Bugs) -> Self {
        let rows = look.window[3];
        Self {
            look,
            texts,
            height,
            line,
            entering: rows,
            top: 0,
            leaving: rows,
            due: None,
            hovers: Hovers::default(),
            bugs,
        }
    }

    /// The credits as they open with the text's lines counted in `font` (the look's: `Credits.fnt`)
    /// at the window's width (`text_line_count` 0x4B5820, `h3_render::text::line_count`), the
    /// text in the game's code page as the drawer reads it.
    #[must_use]
    pub fn laid(look: CreditsLook, texts: Texts, font: &Font, bugs: Bugs) -> Self {
        let bytes = h3_ui::text::encode_1252(&texts.text).0;
        let lines = h3_render::text::line_count(font, &bytes, i64::from(look.window[2]), bugs);
        let line = i32::from(font.height);
        let height = i32::try_from(lines.saturating_mul(i64::from(line))).unwrap_or(i32::MAX);
        Self::new(look, texts, (height, line), bugs)
    }

    /// These credits opened anew, the menus' last items `hovers`.
    #[must_use]
    pub fn opened(&self, hovers: Hovers) -> Self {
        let mut credits = Self::new(
            self.look.clone(),
            self.texts.clone(),
            (self.height, self.line),
            self.bugs,
        );
        credits.hovers = hovers;
        credits
    }

    /// The menus' last items: as the credits found them.
    #[must_use]
    pub const fn hovers(&self) -> Hovers {
        self.hovers
    }

    /// Where it takes its keys: none of the key table's (any key but F4 ends it).
    #[must_use]
    pub const fn context(&self) -> Context {
        Context::Credits
    }

    /// The shell's clock is `now`: the first reading times the first step, a step due runs (one
    /// a reading, the next [`CreditsLook::every`] later). The step after the text has left ends
    /// the credits.
    pub fn clock(&mut self, now: u32) -> Did {
        let Some(due) = self.due else {
            self.due = Some(now.wrapping_add(self.look.every));
            return Did::default();
        };
        if (now.wrapping_sub(due) as i32) < 0 {
            return Did::default();
        }
        self.due = Some(now.wrapping_add(self.look.every));
        if self.step() {
            Did::CHANGED
        } else {
            Did {
                ended: Some(Ended::Done),
                ..Did::CHANGED
            }
        }
    }

    /// One step (`credits_screen` from 0x4EE022): in, up or out; false once the text has left.
    fn step(&mut self) -> bool {
        let [_, _, _, rows] = self.look.window;
        let step = self.look.step;
        if self.entering > 0 {
            self.entering -= step;
        } else if self.top < self.height - rows {
            self.top += step;
        } else if self.leaving >= 0 {
            self.leaving -= step;
            self.top += step;
        } else {
            return false;
        }
        true
    }

    /// What it draws over the main menu's background: `mainmenu.pcx` (from the first step the
    /// window's key pixels black in the original's profile), the text's part in the window, the
    /// closing text once due.
    #[must_use]
    pub fn draws(&self) -> Vec<Draw> {
        let look = &self.look;
        let [x, y, width, rows] = look.window;
        let mut draws = alloc::vec![Draw::Picture {
            picture: look.picture.clone(),
            x: look.at[0],
            y: look.at[1],
            player: None,
            size: None,
        }];
        if self.bugs == Bugs::Original && self.entering < rows {
            draws.push(Draw::Keyed { rect: look.window });
        }
        let shown = if self.entering > 0 {
            rows - self.entering
        } else {
            self.leaving
        };
        let item = |font: &str, size, colour, align| Label {
            id: 0,
            at: [0, 0],
            size,
            font: String::from(font),
            font_palette: look.font_palette.clone(),
            colour,
            align,
            hint: None,
            repeat: None,
        };
        if shown > 0 {
            let size = [width, self.height.saturating_add(self.line)];
            draws.push(Draw::Part {
                item: item(&look.font, size, look.colour, look.align),
                text: self.texts.text.clone(),
                part: [0, self.top, width, shown],
                x,
                y: y + self.entering.max(0),
            });
        }
        if self.entering <= 0 && self.leaving < look.closing_below {
            draws.push(Draw::Label {
                item: item(
                    &look.closing_font,
                    [width, rows],
                    look.closing_colour,
                    look.closing_align,
                ),
                x,
                y,
                text: self.texts.closing.clone(),
                disabled: false,
            });
        }
        draws
    }

    /// A key down by the name `by`: any but F4 ends the credits.
    #[must_use]
    pub fn key(&self, by: &str) -> Did {
        if super::start::key_skips(by) {
            Self::done()
        } else {
            Did::default()
        }
    }

    /// A left or a right press: the credits' end.
    #[must_use]
    pub fn press(&self) -> Did {
        Self::done()
    }

    const fn done() -> Did {
        Did {
            changed: false,
            sample: false,
            ended: Some(Ended::Done),
        }
    }
}

#[cfg(test)]
pub(crate) mod tests;
