//! A dialog's scroll bar (`H3DlgScrollbar`, vtable 0x641D50; E27 M3 T1b1, read for the scenario
//! list's, design §19.2): two arrows, a track and a thumb, the value a tick of `ticks`. Only the
//! vertical bar is built (a bar taller than wide; *provisional*: the horizontal one's art,
//! `SlideBuH.def` and `iGPCrDiv.def`, and its x-axis twins of the rules below, wait for a screen
//! that has one).
//!
//! The original's rules:
//! - **Its art** (`H3DlgScrollbar::Create` 0x5960D0, `loadButton` 0x596050): the DEF's frames 0
//!   and 1 the top arrow up and pressed, 2 and 3 the bottom arrow's, 4 the thumb; the track the
//!   picture (`sliderV.pcx`) cut to the bar's width and its height less both arrows. `button` is the
//!   DEF's height (+0x54, +0x40 the thumb's offset from the bar's top, at least `button`); the
//!   thumb travels the height less three buttons (+0x44).
//! - **Drawn** (0x596C40): the top arrow at the bar's corner, pressed while the bar is held with
//!   the mouse less than a button below its top; the bottom arrow a button above its foot, pressed
//!   while held with the mouse more than a button above the foot; the track; the thumb.
//! - **The value** (+0x3C, `set_value` 0x5961E0): clamped to `0..ticks`, the thumb at `button +
//!   travel × value ÷ (ticks − 1)` (at `button` for a single tick). A new count of ticks (0x5970B0)
//!   keeps the value clamped; a reset (0x5970F0) sets it 0, the thumb at the top.
//! - **A press** (0x596980, from the procedure 0x5964E0): on an arrow, nothing but the arrow
//!   drawn pressed; in the track above the thumb the value less a page (`step`, +0x50: the
//!   scenario list's rows), below it the value more a page; on the thumb (or with no page) the
//!   thumb dragged to the mouse (0x597020: its middle at the mouse, the value the nearest tick).
//!   The bar runs its own loop until a release: each move records the mouse (its x taken as the
//!   bar's within 40 pixels of either side) and, over the track, drags the thumb there (0x5965F7
//!   with its argument 1: from an arrow too).
//! - **The release** (0x596AF0): with the mouse last recorded on the top arrow (no test of x) the
//!   value less one, on the bottom arrow more one, within the ticks. No repeat while an arrow is
//!   held: the loop reads no clock (the procedure's timer at 0x5964F9 runs only outside it).
//! - **The callback** (+0x64): after a press, a move or a release that changed the value since it
//!   was last called, with the value (the caller's [`Scrollbar::press`], [`Scrollbar::moved`] and
//!   [`Scrollbar::release`] return it).
//!
//! The keys of a bar with its arrows' keys (+0x5C, 0x596230) are not built: the scenario list's
//! bar has none (its keys are the dialog's).

use alloc::string::String;
use alloc::vec::Vec;

use serde::Deserialize;

use crate::ui::Draw;

/// A scroll bar's look: where it is and its art.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Look {
    /// The control id.
    pub id: u16,
    /// Its top-left corner from the dialog's.
    pub at: [i32; 2],
    /// Its width and height (taller than wide: a vertical bar).
    pub size: [i32; 2],
    /// The DEF of its arrows and thumb (`SlideBuV.def`, the blue one).
    pub def: String,
    /// The track's picture (`sliderV.pcx`).
    pub track: String,
    /// The DEF's height: an arrow's and the thumb's.
    pub button: i32,
    /// A page: the change of a press in the track (+0x50; none below 1).
    pub step: i32,
}

/// A vertical scroll bar's state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scrollbar {
    look: Look,
    ticks: i32,
    value: i32,
    /// The value last reported to the caller (+0x38).
    reported: i32,
    /// The thumb's offset from the bar's top (+0x40).
    thumb: i32,
    /// Held by the mouse (state bit 1).
    held: bool,
    /// The mouse last recorded, from the dialog's corner (+0x58, +0x5A).
    mouse: [i32; 2],
}

/// How far either side of the bar a held mouse still counts as over it (0x596656: 0x28).
const NEAR: i32 = 40;

impl Scrollbar {
    /// The bar of `look`, one tick, its value 0.
    #[must_use]
    pub const fn new(look: Look) -> Self {
        let thumb = look.button;
        Self {
            look,
            ticks: 1,
            value: 0,
            reported: 0,
            thumb,
            held: false,
            mouse: [0, 0],
        }
    }

    /// Its look.
    #[must_use]
    pub const fn look(&self) -> &Look {
        &self.look
    }

    /// The value (0 to `ticks` − 1).
    #[must_use]
    pub const fn value(&self) -> i32 {
        self.value
    }

    /// The count of ticks (at least 1).
    #[must_use]
    pub const fn ticks(&self) -> i32 {
        self.ticks
    }

    /// Whether the mouse holds it.
    #[must_use]
    pub const fn held(&self) -> bool {
        self.held
    }

    /// The thumb's travel: the height less three buttons (+0x44).
    const fn travel(&self) -> i32 {
        self.look.size[1] - 3 * self.look.button
    }

    /// Reset (0x5970F0): `ticks` ticks (at least 1), the value 0, the thumb at the top.
    pub const fn reset(&mut self, ticks: i32) {
        self.thumb = self.look.button;
        self.value = 0;
        self.reported = 0;
        self.ticks = if ticks > 0 { ticks } else { 1 };
    }

    /// The value `value` clamped to the ticks (`set_value` 0x5961E0), the thumb with it. The value
    /// last reported becomes the one before (+0x38), as the original's: the next press, move or
    /// release reports the value even when it leaves it unchanged.
    pub fn set_value(&mut self, value: i32) {
        self.reported = self.value;
        self.value = value.clamp(0, self.ticks - 1);
        self.place_thumb();
    }

    /// The thumb's place for the value.
    fn place_thumb(&mut self) {
        self.thumb = if self.ticks > 1 {
            self.travel() * self.value / (self.ticks - 1) + self.look.button
        } else {
            self.look.button
        };
    }

    /// Whether `point` (from the dialog's corner) is on the bar.
    #[must_use]
    pub fn contains(&self, point: [i32; 2]) -> bool {
        let ([x, y], [w, h]) = (self.look.at, self.look.size);
        (x..x + w).contains(&point[0]) && (y..y + h).contains(&point[1])
    }

    /// The left button pressed at `point` on the bar (from the dialog's corner): held, then as the
    /// module's rules say; the value when it changed.
    pub fn press(&mut self, point: [i32; 2]) -> Option<i32> {
        self.held = true;
        self.mouse = point;
        let offset = point[1] - self.look.at[1];
        let button = self.look.button;
        let on_track = (button..self.look.size[1] - button).contains(&offset);
        if on_track {
            if self.look.step > 0 {
                if offset < self.thumb {
                    self.set_raw(self.value - self.look.step);
                } else if offset >= self.thumb + button {
                    self.set_raw(self.value + self.look.step);
                } else {
                    self.drag(point[1]);
                }
            } else {
                self.drag(point[1]);
            }
        }
        self.report()
    }

    /// The mouse moved to `point` (from the dialog's corner) while it holds the bar: recorded, its
    /// x the bar's within [`NEAR`] of either side; over the track the thumb dragged there. The
    /// value when it changed.
    pub fn moved(&mut self, point: [i32; 2]) -> Option<i32> {
        if !self.held {
            return None;
        }
        let [x, w] = [self.look.at[0], self.look.size[0]];
        let mut mouse = point;
        let left = x - mouse[0];
        if left > 0 && left < NEAR {
            mouse[0] = x;
        }
        let right = mouse[0] - x - w;
        if right > 0 && right < NEAR {
            mouse[0] = x;
        }
        self.mouse = mouse;
        let [y, h, button] = [self.look.at[1], self.look.size[1], self.look.button];
        let over_track =
            (x..x + w).contains(&mouse[0]) && (y + button..y + h - button).contains(&mouse[1]);
        if over_track {
            self.drag(mouse[1]);
        }
        self.report()
    }

    /// The left button released: let go, the value stepped by the arrow the mouse was last on.
    /// The value when it changed.
    pub fn release(&mut self) -> Option<i32> {
        if !self.held {
            return None;
        }
        self.held = false;
        let offset = self.mouse[1] - self.look.at[1];
        let button = self.look.button;
        if offset < button {
            if self.value > 0 {
                self.value -= 1;
                self.place_thumb();
            }
        } else if offset > self.look.size[1] - button && self.value < self.ticks - 1 {
            self.value += 1;
            self.place_thumb();
        }
        self.report()
    }

    /// The value set by a press (the vtable's +0x38: clamped, the thumb placed), not reported.
    fn set_raw(&mut self, value: i32) {
        let reported = self.reported;
        self.set_value(value);
        self.reported = reported;
    }

    /// The thumb dragged so that its middle is at `y` (0x597020): the nearest tick.
    fn drag(&mut self, y: i32) {
        let button = self.look.button;
        let travel = self.travel();
        let at = (y - self.look.at[1] - button - button / 2).clamp(0, travel.max(0));
        if travel > 0 {
            self.value = ((self.ticks - 1) * at + travel / 2) / travel;
        }
        self.place_thumb();
    }

    /// The value when it changed since last reported (the callback's call, 0x596ABA).
    fn report(&mut self) -> Option<i32> {
        (self.value != self.reported).then(|| {
            self.reported = self.value;
            self.value
        })
    }

    /// Its draw list with the dialog's corner at `place`: the arrows, the track, the thumb.
    #[must_use]
    pub fn draws(&self, place: [i32; 2]) -> Vec<Draw> {
        let [x, y] = [place[0] + self.look.at[0], place[1] + self.look.at[1]];
        let [w, h] = self.look.size;
        let button = self.look.button;
        let offset = self.mouse[1] - self.look.at[1];
        let frame = |frame: u32, y: i32| Draw::Button {
            def: self.look.def.clone(),
            frame,
            x,
            y,
            mirrored: false,
            player: None,
        };
        let top = u32::from(self.held && offset < button);
        let bottom = 2 + u32::from(self.held && offset > h - button);
        alloc::vec![
            frame(top, y),
            frame(bottom, y + h - button),
            Draw::Picture {
                picture: self.look.track.clone(),
                x,
                y: y + button,
                player: None,
                size: Some([w, h - 2 * button]),
            },
            frame(4, y + self.thumb),
        ]
    }
}

#[cfg(test)]
mod tests;
