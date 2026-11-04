//! The army info windows on the mouse's move (E17 M6 T4b2b1, `docs/epics/E17-design.md` §12.15):
//! the battle dialog's six windows (`combat_dialog_create` 0x472483 to 0x4725D4, each with its
//! shown flag), shown by the mouse entering a new hex at Creature Info's levels 1 and 2 and by the
//! right press over a hero, hidden by a new hex, a left press over the battlefield, the key T and
//! every action's start, drawn at the end of every `battle_redraw` while the level is not 0.
//!
//! [`Windows`] is their state at a time: which are shown, what filled them and the level. The
//! battle's mouse over time ([`crate::mouse::track`]) changes it at the inputs and the log's
//! entries; a still ([`Hover`]) at the harness's moves and presses at the opening. A hero's
//! window is filled as the right press fills it ([`crate::hero_info`]). The stacks' four windows
//! (kind 1 "all statistics" at level 1, kind 2 "spells only" at level 2, by the stack's side) are
//! T4b2b2's: not built; their place is [`Windows::enter`]'s hex arm.

use h3_protocol::wire::{Observation, Side};
use h3_render::Bugs;

use crate::layout::BattleLayout;
use crate::mouse::{Spot, spot};

/// The army info windows at a time: those shown (their flags, each with what filled it) and
/// Creature Info's level ("Combat Army Info Level" 0x6987C8), which draws the shown ones while
/// not 0 (§12.14). Only the heroes' two are built; the stacks' four are T4b2b2's.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Windows {
    /// Each side's hero's window while shown (the attacker's first: +0x74 at (1, 135), +0x78 at
    /// (721, 135)): the turn whose battle filled it (0 the opening's).
    pub heroes: [Option<usize>; 2],
    /// Creature Info's level, 0 to 2.
    pub level: u8,
}

impl Windows {
    /// None shown, at `level`.
    #[must_use]
    pub const fn at_level(level: u8) -> Self {
        Self {
            heroes: [None; 2],
            level,
        }
    }

    /// Every window hidden (`army_info_hide` 0x46CE70 and `army_info_stack_hide` 0x46DF80 on the
    /// six); the level kept.
    pub const fn hide(&mut self) {
        self.heroes = [None; 2];
    }

    /// The mouse entering a new hex `spot` (0x47557D to 0x475879; [`Spot::Nothing`] for −1) in
    /// turn `turn`'s battle `observation`: every window hidden, then at most one shown while the
    /// level is not 0: over a side's hero's rectangle (0xFC, 0xFD) with a hero there, its window,
    /// filled by the turn's battle (`army_info_hero_set`, then `army_info_show`).
    pub fn enter(&mut self, spot: Spot, observation: &Observation, turn: usize) {
        self.hide();
        if self.level == 0 {
            return;
        }
        match spot {
            Spot::Hero(side) if observation.sides[index(side)].hero.is_some() => {
                self.heroes[index(side)] = Some(turn);
            }
            // A stack's hex (`hex_stack`): its window by its owner's side, kind 1 at level 1 and
            // kind 2 at level 2 (`army_info_stack_set`, `army_info_stack_show`): T4b2b2's, not
            // built; none shown meanwhile.
            Spot::Hex(_) => {}
            // Anything else: none.
            Spot::Hero(_) | Spot::Keep | Spot::Tower | Spot::Nothing => {}
        }
    }

    /// The right button pressed over side `side`'s hero's rectangle in turn `turn` and released
    /// (`battle_right_click` 0x476A0B, 0x476A81, then 0x476A1D): its window filled and shown while
    /// held, then hidden at level 0; at 1 and 2 it stays shown, as a hover's (oddity 6: one shown
    /// already is filled again, nothing drawn anew).
    pub fn held(&mut self, side: Side, turn: usize) {
        self.heroes[index(side)] = (self.level != 0).then_some(turn);
    }

    /// Whether the right button held over side `side`'s hero shows its window in profile `bugs`
    /// (CLT-Q50, design §12.15's oddity 4): the original's shows nothing when the window is shown
    /// already at level 0 (still flagged, `army_info_show` does nothing and the redraws at 0 skip
    /// it); the client's shows it whatever (fixed, the user's answer of 2026-11-04).
    #[must_use]
    pub const fn held_shows(&self, side: Side, bugs: Bugs) -> bool {
        matches!(bugs, Bugs::Fixed) || self.level != 0 || self.heroes[index(side)].is_none()
    }

    /// The windows drawn (each `battle_redraw`'s end): the shown ones while the level is not 0,
    /// each hero's side with the turn whose battle filled it; none at level 0.
    pub fn drawn(&self) -> impl Iterator<Item = (Side, usize)> + '_ {
        [Side::Attacker, Side::Defender]
            .into_iter()
            .zip(self.heroes)
            .filter_map(|(side, turn)| Some((side, turn?)))
            .filter(|_| self.level != 0)
    }
}

/// The side's index in the battle's pairs (the attacker's first).
const fn index(side: Side) -> usize {
    match side {
        Side::Attacker => 0,
        Side::Defender => 1,
    }
}

/// Whether `point` of the screen is over the battlefield (x 0 to 799, y 0 to 555: the move's
/// test, 0x475402; the left press's, 0x4752BB).
#[must_use]
pub fn on_field(layout: &BattleLayout, [x, y]: [i32; 2]) -> bool {
    let [width, height] = layout.mouse.field;
    (0..width).contains(&x) && (0..height).contains(&y)
}

/// The battle's mouse as the army info windows follow it in a still (E17 M6 T4b2b1,
/// *provisional*): the harness's moves and presses at the opening, its battle the opening's (turn
/// 0), its level the still's (`frames.toml`'s `army_info`; the key F5 cycles it). The window's
/// battle follows the same rules over time in [`crate::mouse::track`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Hover {
    /// The mouse's hex (+0x132D4; none: −1).
    hex: Option<Spot>,
    /// The windows.
    pub windows: Windows,
}

impl Hover {
    /// The mouse nowhere yet, no window shown, at `level`.
    #[must_use]
    pub const fn new(level: u8) -> Self {
        Self {
            hex: None,
            windows: Windows::at_level(level),
        }
    }

    /// The battle takes a move to `point` over `observation`'s battle, the keep's and the upper
    /// tower's rectangles found with `rects` (CLT-Q21): out of the battlefield every window
    /// hidden and the hex forgotten (0x475402); over the hex it is over already nothing changes
    /// (0x475544); over a new one [`Windows::enter`].
    pub fn moved(
        &mut self,
        layout: &BattleLayout,
        observation: &Observation,
        (point, rects): ([i32; 2], bool),
    ) {
        if !on_field(layout, point) {
            self.windows.hide();
            self.hex = None;
            return;
        }
        let at = match spot(layout, point, rects) {
            Spot::Nothing => None,
            other => Some(other),
        };
        if at.is_some() && at == self.hex {
            return;
        }
        self.windows
            .enter(at.unwrap_or(Spot::Nothing), observation, 0);
        self.hex = at;
    }

    /// The mouse's hex forgotten (a stats dialog's close, a human's turn's start: the move they
    /// post then counts as a new hex's).
    pub const fn forget(&mut self) {
        self.hex = None;
    }

    /// A left press at `point` (0x4752BB): over the battlefield every window hidden, the mouse's
    /// hex kept (the windows come back only with another hex).
    pub fn pressed(&mut self, layout: &BattleLayout, point: [i32; 2]) {
        if on_field(layout, point) {
            self.windows.hide();
        }
    }

    /// One of the keys C, F, S, R and O pressed with the mouse at `point` (CLT-Q49, design
    /// §12.15's oddity 3): the client's profile hides the windows as a left press there does
    /// (fixed, the user's answer of 2026-11-04); the original's keeps them through the key's book,
    /// aim or box.
    pub fn key(&mut self, layout: &BattleLayout, point: [i32; 2], bugs: Bugs) {
        if bugs == Bugs::Fixed {
            self.pressed(layout, point);
        }
    }

    /// The key F5 (§12.14): the level cycled 0, 1, 2, 0; no window shown or hidden (oddity 6).
    pub const fn cycle(&mut self) {
        self.windows.level = (self.windows.level + 1) % 3;
    }
}

#[cfg(test)]
mod tests;
