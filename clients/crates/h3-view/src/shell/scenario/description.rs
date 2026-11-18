//! The scenario screen's description (E27 M3 T2c; `docs/epics/E27-design.md` §22): a scrolling
//! text (`H3DlgScrollableText`, `scrollable_text_create` 0x5B9FB0 at (422, 155, 319, 115),
//! `smalfont.fnt`, colour 4, a blue bar) whose text `scenario_to_game` sets at every select
//! (`scrollable_text_set` 0x5BA6E0), from the list's record (`scenario_header_read` 0x583170:
//! its first 300 bytes, [`DESCRIPTION_BYTES`]).
//!
//! - **The split** (`h3_render::text::split_lines`, 0x4B5B90): at the box's width − 24; more
//!   lines than the box's height ÷ the font's height (7) scroll, else the text is split again at
//!   width − 3. Each line is a text item of its own (alignment 0, the font's height apart), so a
//!   highlight (`{`) ends with its line: the client draws the lines shown as one label, a line
//!   left highlighted closed by a `}` (which the drawer neither draws nor counts).
//! - **The bar** (`scrollbar_create` 0x5960D0 at the box's right edge, 16 wide, its height; the
//!   ticks the lines less the shown plus one, a page of the shown, its arrows' keys): shown only
//!   while the text scrolls (0x5BA2E0, 0x5BA82C); a set resets it to 0. Its value is the first
//!   line shown: a change redraws the lines (the subclass's vtable 0x642CC8 +0x40, 0x5B9FA0 →
//!   0x5BA600: the box's background less the bar's column put back from its copy, kept at the
//!   first redraw (0x5BA571), the lines drawn, the box shown). The mouse as on the list's bar
//!   (`h3_ui::scrollbar`); its keys at their release ([`super::keys::Scroll`]: the list's keys
//!   act at their press, so in the original's profile Down onto a long description shows it from
//!   its second line; in the client's the bar takes no list key while the list shows, Left and
//!   Right always: CLT-Q140 (b), decision 52).
//! - **What the screen last drew** (the original's profile, CLT-Q138's partial redraw): the
//!   select of the row already selected resets the text and the bar (`scenario_select` 0x585500
//!   calls `scenario_to_game` whatever the change) but the partial redraw draws neither: the
//!   screen keeps the lines and the bar as last drawn (by a whole redraw, a change of value, the
//!   bar's own draw at a press or a key's step) until either draws again.

use alloc::string::String;
use alloc::vec::Vec;

use h3_assets::fnt::Font;
use h3_render::text::split_lines;
use h3_ui::scrollbar::{self, KeyStep, Scrollbar};
use h3_ui::text::{decode_1252, encode_1252};
use h3_ui::ui::Draw;
use serde::Deserialize;

use super::super::keys::Scroll;

/// The bytes of a description the list's record keeps (`scenario_header_read` 0x583170:
/// `strncpy` of 0x12C into +0x5C9; a longer text is not read past them, *provisional*: the
/// record's next byte, 0 for Jihad's 300 in `scn-long-text`).
pub const DESCRIPTION_BYTES: usize = 300;

/// What the split leaves of the box's width while the text scrolls (0x5BA037: − 0x18) and while
/// it fits (0x5BA07E: − 3).
const SCROLLING_MARGIN: i32 = 24;
const FITTING_MARGIN: i32 = 3;

/// The description's look (the layout's `[scenario.description]`): the scrolling text's font and
/// box's size (its corner the label's, [`super::DESCRIPTION`]) and its bar.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DescriptionLook {
    /// The font the text splits in (`smalfont.fnt`, the label's).
    pub font: String,
    /// The box's width and height (319 × 115).
    pub size: [i32; 2],
    /// The bar (`SlideBuV.def`, `sliderV.pcx`, 16-pixel buttons, a page of the lines shown).
    pub bar: scrollbar::Look,
}

/// Every asset the description names (the adapter's batch at the shell's start): its font, the
/// bar's DEF and track.
#[must_use]
pub fn description_names(look: &DescriptionLook) -> Vec<String> {
    alloc::vec![
        look.font.clone(),
        look.bar.def.clone(),
        look.bar.track.clone()
    ]
}

/// The description as it is and as the screen last drew it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Description {
    look: DescriptionLook,
    /// The font's metrics (`smalfont.fnt`); none: the text one label, never scrolling.
    font: Option<Font>,
    /// The lines of the text set, as split.
    lines: Vec<String>,
    /// More lines than the box shows: the bar shows.
    scrolls: bool,
    bar: Scrollbar,
    /// The lines' text and the bar (when shown) as the screen last drew them.
    drawn_text: String,
    drawn_bar: Option<Scrollbar>,
    /// A key down whose release steps the bar: its step and its name.
    key: Option<(KeyStep, String)>,
}

impl Description {
    /// The description of `look` in the font `font`, with no text.
    pub(super) fn new(look: DescriptionLook, font: Option<Font>) -> Self {
        let bar = Scrollbar::new(look.bar.clone());
        Self {
            look,
            font,
            lines: Vec::new(),
            scrolls: false,
            bar,
            drawn_text: String::new(),
            drawn_bar: None,
            key: None,
        }
    }

    /// Its look.
    pub(super) const fn look(&self) -> &DescriptionLook {
        &self.look
    }

    /// The lines the box shows (its height ÷ the font's height: 7).
    fn shown(&self) -> usize {
        let height = self.font.as_ref().map_or(0, |f| i32::from(f.height));
        usize::try_from(self.look.size[1].checked_div(height).unwrap_or(0)).unwrap_or(0)
    }

    /// The text `text` set (`scrollable_text_set` 0x5BA6E0): split at the width less 24; more
    /// lines than shown keep that split, the bar shown with a tick for each first line, at 0;
    /// else split at the width less 3, the bar hidden.
    pub(super) fn set(&mut self, text: &str) {
        let Some(font) = &self.font else {
            self.lines = alloc::vec![String::from(text)];
            self.scrolls = false;
            return;
        };
        let bytes = encode_1252(text).0;
        let split = |margin: i32| {
            split_lines(font, &bytes, i64::from(self.look.size[0] - margin))
                .iter()
                .map(|line| decode_1252(line))
                .collect::<Vec<_>>()
        };
        let lines = split(SCROLLING_MARGIN);
        let shown = self.shown();
        self.scrolls = lines.len() > shown;
        if self.scrolls {
            let ticks = i32::try_from(lines.len() - shown + 1).unwrap_or(i32::MAX);
            self.bar.reset(ticks);
            self.lines = lines;
        } else {
            self.bar.reset(1);
            self.lines = split(FITTING_MARGIN);
        }
        self.bar.set_value(0);
    }

    /// The lines shown now, as the label's text: from the bar's value, as many as the box shows
    /// (all while the text fits), a line left highlighted closed.
    pub(super) fn text(&self) -> String {
        let first = usize::try_from(self.bar.value()).unwrap_or(0);
        let count = if self.scrolls {
            self.shown()
        } else {
            self.lines.len()
        };
        let mut text = String::new();
        for (k, line) in self.lines.iter().skip(first).take(count).enumerate() {
            if k > 0 {
                text.push('\n');
            }
            text.push_str(line);
            let lit = line.chars().fold(false, |lit, c| match c {
                '{' => true,
                '}' => false,
                _ => lit,
            });
            if lit {
                text.push('}');
            }
        }
        text
    }

    /// The text as the screen last drew it (the original's profile after the partial redraw).
    pub(super) fn drawn_text(&self) -> &str {
        &self.drawn_text
    }

    /// The screen drew the description whole: as it is now.
    pub(super) fn drew(&mut self) {
        self.drawn_text = self.text();
        self.drew_bar();
    }

    /// The bar drew itself (a press, a move, a release, a key's step).
    fn drew_bar(&mut self) {
        self.drawn_bar = self.scrolls.then(|| self.bar.clone());
    }

    /// The bar's value changed to `value`: the lines drawn again (0x5BA600). Whether it did.
    fn scrolled(&mut self, value: Option<i32>) -> bool {
        if value.is_some() {
            self.drawn_text = self.text();
        }
        value.is_some()
    }

    /// Whether the mouse holds the bar.
    pub(super) fn held(&self) -> bool {
        self.bar.held()
    }

    /// The left button pressed at `inside` (from the dialog's corner): on the bar while it shows,
    /// as the list's bar ([`Scrollbar::press`]). Whether the bar took it.
    pub(super) fn press(&mut self, inside: [i32; 2]) -> bool {
        if !self.scrolls || !self.bar.contains(inside) {
            return false;
        }
        let value = self.bar.press(inside);
        self.drew_bar();
        self.scrolled(value);
        true
    }

    /// The mouse moved to `inside` while it holds the bar.
    pub(super) fn moved(&mut self, inside: [i32; 2]) {
        let value = self.bar.moved(inside);
        self.drew_bar();
        self.scrolled(value);
    }

    /// The left (or right) button released while it holds the bar.
    pub(super) fn release(&mut self) {
        let value = self.bar.release();
        self.drew_bar();
        self.scrolled(value);
    }

    /// A key down of the list's command `scroll` named `by`: its step kept for its release.
    pub(super) fn key_down(&mut self, scroll: Scroll, by: &str) {
        let step = match scroll {
            Scroll::Up | Scroll::Left => KeyStep::Back,
            Scroll::Down | Scroll::Right => KeyStep::On,
            Scroll::PageUp => KeyStep::PageBack,
            Scroll::PageDown => KeyStep::PageOn,
        };
        self.key = Some((step, String::from(by)));
    }

    /// The key `by` released: the bar's step of its key down when `free` (nothing over the
    /// screen) and the bar shows (`scrollbar_key_step` 0x596230: the bar drawn, the lines when the
    /// value changed). Whether the bar stepped.
    pub(super) fn key_up(&mut self, by: &str, free: bool) -> bool {
        let Some((step, _)) = self.key.take_if(|(_, name)| name == by) else {
            return false;
        };
        if !free || !self.scrolls || !self.bar.takes_key(step) {
            return false;
        }
        let value = self.bar.key(step);
        self.drew_bar();
        self.scrolled(value);
        true
    }

    /// The bar's draw list with the dialog's corner at `place`: as it is, or as last drawn
    /// (`stale`: the original's profile after the partial redraw); none while hidden.
    pub(super) fn bar_draws(&self, place: [i32; 2], stale: bool) -> Vec<Draw> {
        let bar = if stale {
            self.drawn_bar.as_ref()
        } else {
            self.scrolls.then_some(&self.bar)
        };
        bar.map_or_else(Vec::new, |bar| bar.draws(place))
    }
}

#[cfg(test)]
mod tests;
