//! The map over time (E24 M2 T2, *provisional*): the original's adventure loop on a clock of whole
//! milliseconds, for the map standing still and the view scrolling (`docs/epics/E24-design.md`
//! §3.1, §3.4), from a sequence's start and the player's actions at their milliseconds. Each
//! millisecond gives a [`State`]: what the original's memory holds (the view's corner, the objects'
//! frame steps, the palette steps, the cursor) and what its screen shows (the view as last drawn
//! and the cursor); [`lines`] writes the recorded timeline's lines from them (design §3.2). The
//! walk (a hero's step, a click) is M2 T3's, the cursor over the view M3's: [`Unplayable`].
//!
//! Per millisecond, as the managers' loop takes it (§3.1): the palette tick when due, the mouse
//! poll when due (the cursor's point), then the idle message (the edge scroll, then the idle
//! step's redraw when due), then each action given at that millisecond as its message (the edge
//! scroll before each, §3.4). The comparisons give the actions after the millisecond's idle turns,
//! as the harness gives its inputs (`h3map.py frames`); the window (T4) takes them the same way.

use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

use h3_render::Bugs;

use crate::layout::{AdventureLayout, DIRECTIONS};
use crate::scene::ViewAt;
use crate::tables::Tables;

/// The mouse manager's type of the adventure cursor (`cursor_defs` 0x67FF38: 1, `cradvntr.def`),
/// the timeline's `cursor` lines' first number.
pub const CURSOR_TYPE: u32 = 1;

/// The scroll's and the hero's eight directions, clockwise from north (the original's 0 to 7,
/// `view_scroll` 0x4195C0).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Direction {
    /// Up: y less.
    North,
    /// x more, y less.
    NorthEast,
    /// x more.
    East,
    /// x and y more.
    SouthEast,
    /// y more.
    South,
    /// x less, y more.
    SouthWest,
    /// x less.
    West,
    /// x and y less.
    NorthWest,
}

impl Direction {
    /// The directions in the original's order.
    pub const ALL: [Self; 8] = [
        Self::North,
        Self::NorthEast,
        Self::East,
        Self::SouthEast,
        Self::South,
        Self::SouthWest,
        Self::West,
        Self::NorthWest,
    ];

    /// The original's number (0 north to 7 north-west).
    #[must_use]
    pub fn number(self) -> u32 {
        match self {
            Self::North => 0,
            Self::NorthEast => 1,
            Self::East => 2,
            Self::SouthEast => 3,
            Self::South => 4,
            Self::SouthWest => 5,
            Self::West => 6,
            Self::NorthWest => 7,
        }
    }

    /// The step of one tile that way: x grows east, y south.
    #[must_use]
    pub fn delta(self) -> [i32; 2] {
        match self {
            Self::North => [0, -1],
            Self::NorthEast => [1, -1],
            Self::East => [1, 0],
            Self::SouthEast => [1, 1],
            Self::South => [0, 1],
            Self::SouthWest => [-1, 1],
            Self::West => [-1, 0],
            Self::NorthWest => [-1, -1],
        }
    }
}

/// What the player does (the window's actions: T4's key table binds the keys to them; the
/// original's defaults in design §3.4). A key's release does nothing on the map.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// Ctrl and a direction key: the view one scroll step that way (`adventure_key_down`
    /// 0x408C40), without the edge cursor.
    Scroll(Direction),
    /// A direction key alone: the selected hero one step that way (M2 T3).
    Step(Direction),
    /// The pointer moved to a point of the frame.
    Pointer([i32; 2]),
    /// The left button clicked at a point (a tile clicked shows the selected hero's path to it,
    /// clicked again the hero walks it: M2 T3).
    Click([i32; 2]),
}

/// Why the timeline cannot play an action yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unplayable {
    /// A hero's step or a click: the walk, M2 T3's (MCL-STUB-1).
    Walk,
    /// The pointer over the view: the cursor by the tile under it, M3's.
    OverView([i32; 2]),
}

impl fmt::Display for Unplayable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Walk => write!(f, "a hero's walk (M2 T3, MCL-STUB-1)"),
            Self::OverView([x, y]) => write!(f, "the pointer over the view at ({x}, {y}) (M3)"),
        }
    }
}

/// The numbers the loop runs by: the tables' (`[map]`), the layout's (the frame, the view's
/// rectangle, `[scroll]`, `[cursor]`) and whether the original's bugs are reproduced (the edge
/// zones of MCL-Q6, the cursor's even x: E17 design §9.9; for the comparisons only).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rules {
    /// A palette step's time (200 ms).
    pub cycle_ms: u32,
    /// An idle redraw's (180 ms).
    pub frame_ms: u32,
    /// The edge scroll's time between steps (70 ms).
    pub scroll_ms: u32,
    /// The mouse poll's (33 ms).
    pub poll_ms: u32,
    /// A scroll step's tiles by the scroll speed.
    pub steps: Vec<i32>,
    /// The frame's width and height.
    pub frame: [i32; 2],
    /// The view's rectangle (its clip: left, top, right, bottom, the last two exclusive): the
    /// pointer over it is M3's.
    pub view: [i32; 4],
    /// The view's tile the map's first and last column and row stop at.
    pub middle: [i32; 2],
    /// The edge zones' width.
    pub edge: i32,
    /// The arrow's frame.
    pub arrow: u32,
    /// The first edge cursor's frame.
    pub edges: u32,
    /// The original's bugs or the client's fixes.
    pub bugs: Bugs,
}

impl Rules {
    /// The rules of these tables and layout.
    #[must_use]
    pub fn new(tables: &Tables, layout: &AdventureLayout, bugs: Bugs) -> Self {
        let side = |v: u32| i32::try_from(v).unwrap_or(i32::MAX);
        Self {
            cycle_ms: tables.map.cycle_ms,
            frame_ms: tables.map.frame_ms,
            scroll_ms: tables.map.scroll_ms,
            poll_ms: tables.map.poll_ms,
            steps: tables.map.scroll_steps.clone(),
            frame: layout.frame.map(side),
            view: layout.view.clip,
            middle: layout.scroll.middle,
            edge: layout.scroll.edge,
            arrow: layout.cursor.arrow,
            edges: layout.cursor.edges,
            bugs,
        }
    }

    /// The edge zone a point lies in (`edge_scroll` 0x419820): none off the frame. The client's
    /// zones are `edge` pixels wide on every side; the original's test the mouse's x rounded down
    /// to even against x < 16 and x > 784, y against y < 16 and y > 584, so its zones are 16, 14,
    /// 16 and 15 pixels wide (MCL-Q6).
    #[must_use]
    pub fn edge_at(&self, [x, y]: [i32; 2]) -> Option<Direction> {
        let [width, height] = self.frame;
        if !(0..width).contains(&x) || !(0..height).contains(&y) {
            return None;
        }
        let (west, east, north, south) = match self.bugs {
            Bugs::Fixed => (
                x < self.edge,
                x >= width - self.edge,
                y < self.edge,
                y >= height - self.edge,
            ),
            Bugs::Original => {
                let x = x & !1;
                (
                    x < self.edge,
                    x > width - self.edge,
                    y < self.edge,
                    y > height - self.edge,
                )
            }
        };
        match (north, east, south, west) {
            (true, true, _, _) => Some(Direction::NorthEast),
            (_, true, true, _) => Some(Direction::SouthEast),
            (_, _, true, true) => Some(Direction::SouthWest),
            (true, _, _, true) => Some(Direction::NorthWest),
            (true, ..) => Some(Direction::North),
            (_, true, ..) => Some(Direction::East),
            (_, _, true, _) => Some(Direction::South),
            (.., true) => Some(Direction::West),
            _ => None,
        }
    }

    /// Where the cursor is drawn for the mouse at `at`: there, or (the original's drawing bug,
    /// E17 design §9.9) its x rounded down to even.
    #[must_use]
    pub fn drawn_at(&self, [x, y]: [i32; 2]) -> [i32; 2] {
        match self.bugs {
            Bugs::Fixed => [x, y],
            Bugs::Original => [x & !1, y],
        }
    }

    /// Whether a point lies over the view.
    #[must_use]
    pub fn over_view(&self, [x, y]: [i32; 2]) -> bool {
        let [left, top, right, bottom] = self.view;
        (left..right).contains(&x) && (top..bottom).contains(&y)
    }
}

/// Where a sequence starts (its 0: the palette tick, the redraw, the mouse poll and the scroll's
/// last time all at 0, as the harness sets them).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Start {
    /// The view's corner.
    pub view: ViewAt,
    /// The map's tiles a side.
    pub size: i32,
    /// The scroll speed (0 to 2).
    pub scroll_speed: usize,
    /// The pointer's point.
    pub pointer: [i32; 2],
}

/// The cursor drawn: a frame of the layout's cursor and the point its hot spot is at.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cursor {
    /// The frame.
    pub frame: u32,
    /// The point.
    pub at: [i32; 2],
}

/// A draw of the view: its corner, and the objects' frame steps and the palette steps since the
/// sequence's 0 (the draw at 0 shows 0 of each: the still the live part at 0 gives).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Draw {
    /// The view's corner.
    pub view: ViewAt,
    /// The objects' frame steps (the manager's +0x100 less its value at 0's draw).
    pub frames: u32,
    /// The palette steps (both kinds').
    pub cycles: u32,
}

/// The state after a millisecond.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct State {
    /// The millisecond.
    pub t: u32,
    /// The view's corner in memory.
    pub view: ViewAt,
    /// The objects' frame steps since 0's redraw stepped them (the timeline's `objects`).
    pub objects: u32,
    /// The palette steps since 0's (the timeline's `cycles`, both kinds').
    pub cycles: u32,
    /// The cursor.
    pub cursor: Cursor,
    /// What the screen's view shows: its last draw.
    pub drawn: Draw,
}

/// The original's adventure loop (§3.1, §3.4), one millisecond at a time.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Loop {
    rules: Rules,
    /// The map's last column and row the view's middle may show.
    last: i32,
    /// The scroll step's tiles.
    step: i32,
    /// The palette tick's next time (0x6989B4) and the steps since before 0's.
    palette_at: u32,
    ticks: u32,
    /// The idle redraw's next time (`frame_deadline` 0x698998) and the steps since before 0's
    /// redraw (+0x100 less its value then).
    frame_at: u32,
    steps: u32,
    /// The mouse poll's next time (0x69CA18).
    poll_at: u32,
    /// The scroll's last time (`scroll_time` 0x691674).
    scroll_at: u32,
    view: ViewAt,
    pointer: [i32; 2],
    cursor: Cursor,
    drawn: Draw,
}

impl Loop {
    /// The loop after the sequence's 0: the tick, the poll and the redraw there.
    ///
    /// # Errors
    /// [`Unplayable::OverView`] for a pointer over the view.
    pub fn start(rules: Rules, start: &Start) -> Result<Self, Unplayable> {
        let step = rules
            .steps
            .get(start.scroll_speed)
            .copied()
            .unwrap_or(1);
        let cursor = Cursor {
            frame: rules.arrow,
            at: rules.drawn_at(start.pointer),
        };
        let mut l = Self {
            last: start.size.saturating_sub(1),
            step,
            palette_at: 0,
            ticks: 0,
            frame_at: 0,
            steps: 0,
            poll_at: 0,
            scroll_at: 0,
            view: start.view,
            pointer: start.pointer,
            cursor,
            drawn: Draw {
                view: start.view,
                frames: 0,
                cycles: 0,
            },
            rules,
        };
        if l.rules.over_view(start.pointer) {
            return Err(Unplayable::OverView(start.pointer));
        }
        l.run(0, &[])?;
        Ok(l)
    }

    /// The state memory holds and the screen shows now.
    #[must_use]
    pub fn state(&self, t: u32) -> State {
        State {
            t,
            view: self.view,
            objects: self.steps.saturating_sub(1),
            cycles: self.ticks.saturating_sub(1),
            cursor: self.cursor,
            drawn: self.drawn,
        }
    }

    /// The millisecond `t` (after the last one run) with the actions given at it.
    ///
    /// # Errors
    /// An action the loop cannot play yet ([`Unplayable`]).
    pub fn run(&mut self, t: u32, actions: &[Action]) -> Result<(), Unplayable> {
        // The loop's turn: the tick before its message (0x4ED490 from input_next_message), the
        // mouse poll (0x50D680): the cursor drawn where it reads the mouse.
        if t >= self.palette_at {
            self.ticks = self.ticks.saturating_add(1);
            self.palette_at = t.saturating_add(self.rules.cycle_ms);
        }
        if t >= self.poll_at {
            self.cursor.at = self.rules.drawn_at(self.pointer);
            self.poll_at = self
                .poll_at
                .saturating_add(self.rules.poll_ms.max(t - self.poll_at));
        }
        // The idle message: the edge scroll before it, then the idle step (0x408A88).
        self.edge(t);
        if t >= self.frame_at {
            self.draw();
            self.step_objects(t);
        }
        for &action in actions {
            if let Action::Pointer(at) = action {
                // The move's message: GetCursorPos (the edge's) already reads the new point.
                self.pointer = at;
            }
            self.edge(t);
            match action {
                Action::Scroll(direction) => {
                    self.scroll(t, direction, false);
                }
                Action::Pointer(at) => {
                    // A move outside the view (adventure_mouse_move 0x40EB7D) keeps an edge cursor
                    // while the mouse lies in a zone, the arrow otherwise.
                    if self.rules.over_view(at) {
                        return Err(Unplayable::OverView(at));
                    }
                    if self.rules.edge_at(at).is_none() {
                        self.cursor.frame = self.rules.arrow;
                    }
                }
                Action::Step(_) | Action::Click(_) => return Err(Unplayable::Walk),
            }
        }
        Ok(())
    }

    /// The edge scroll (`edge_scroll` 0x419820, before every message): outside the zones the
    /// scroll's time is now; inside, a step once `scroll_ms` passed since it, with the edge
    /// cursor, and the arrow when that step moved nothing (a limit).
    fn edge(&mut self, t: u32) {
        match self.rules.edge_at(self.pointer) {
            None => self.scroll_at = t,
            Some(direction) => {
                if t.saturating_sub(self.scroll_at) >= self.rules.scroll_ms
                    && !self.scroll(t, direction, true)
                    && self.edge_cursor()
                {
                    self.cursor.frame = self.rules.arrow;
                }
            }
        }
    }

    /// A scroll step (`view_scroll` 0x4195C0): the scroll's time now, the edge cursor with `edge`,
    /// the corner moved and held within the limits; a corner that moved is drawn and shown at
    /// once, with the objects' step when the redraw is due. Whether it moved.
    fn scroll(&mut self, t: u32, direction: Direction, edge: bool) -> bool {
        self.scroll_at = t;
        if edge {
            self.cursor.frame = self.rules.edges.saturating_add(direction.number());
        }
        let [dx, dy] = direction.delta();
        let [mx, my] = self.rules.middle;
        let x = self
            .view
            .x
            .saturating_add(dx.saturating_mul(self.step))
            .clamp(-mx, self.last.saturating_sub(mx).max(-mx));
        let y = self
            .view
            .y
            .saturating_add(dy.saturating_mul(self.step))
            .clamp(-my, self.last.saturating_sub(my).max(-my));
        if (x, y) == (self.view.x, self.view.y) {
            return false;
        }
        self.view.x = x;
        self.view.y = y;
        self.draw();
        self.step_objects(t);
        true
    }

    /// The view drawn and shown: at the objects' step and the palettes now.
    fn draw(&mut self) {
        self.drawn = Draw {
            view: self.view,
            frames: self.steps,
            cycles: self.ticks.saturating_sub(1),
        };
    }

    /// The objects' step after a redraw (0x408B26): while the clock has reached the deadline,
    /// +0x100 grows by one and the deadline moves on by the larger of the step and its lateness.
    fn step_objects(&mut self, t: u32) {
        while t >= self.frame_at {
            self.steps = self.steps.saturating_add(1);
            let late = t - self.frame_at;
            self.frame_at = self
                .frame_at
                .saturating_add(self.rules.frame_ms.max(late).max(1));
        }
    }

    /// Whether the cursor is one of the edge cursors.
    fn edge_cursor(&self) -> bool {
        (self.rules.edges..self.rules.edges.saturating_add(DIRECTIONS)).contains(&self.cursor.frame)
    }
}

/// A sequence played: the state after each millisecond from 0 to `until`, the actions at their
/// milliseconds (from 1; one at 0 or after `until` is not played).
///
/// # Errors
/// The millisecond of the first action the loop cannot play yet, and why.
pub fn play(
    rules: Rules,
    start: &Start,
    actions: &BTreeMap<u32, Vec<Action>>,
    until: u32,
) -> Result<Vec<State>, (u32, Unplayable)> {
    let mut l = Loop::start(rules, start).map_err(|why| (0, why))?;
    let mut states = Vec::with_capacity(usize::try_from(until).unwrap_or(0).saturating_add(1));
    states.push(l.state(0));
    for t in 1..=until {
        let given = actions.get(&t).map_or(&[][..], Vec::as_slice);
        l.run(t, given).map_err(|why| (t, why))?;
        states.push(l.state(t));
    }
    Ok(states)
}

/// The timeline's lines (design §3.2): at 0 the view, the objects, the cycles and the cursor, then
/// each one where it changed, in that order within a millisecond, and a `screen` line where
/// `screens` gives the box the screen changed in (never at 0, whose screen is always kept).
#[must_use]
pub fn lines(states: &[State], screens: &BTreeMap<u32, [i32; 4]>) -> Vec<String> {
    let mut out = Vec::new();
    let mut last: Option<&State> = None;
    for s in states {
        let changed = |f: fn(&State) -> String| last.is_none_or(|l| f(l) != f(s));
        let view = |s: &State| format!("view {} {} {}", s.view.x, s.view.y, s.view.z);
        let objects = |s: &State| format!("objects {}", s.objects);
        let cycles = |s: &State| format!("cycles {} {}", s.cycles, s.cycles);
        let cursor = |s: &State| {
            format!(
                "cursor {CURSOR_TYPE} {} {} {}",
                s.cursor.frame, s.cursor.at[0], s.cursor.at[1]
            )
        };
        for f in [view, objects, cycles, cursor] {
            if changed(f) {
                out.push(format!("{} {}", s.t, f(s)));
            }
        }
        if s.t > 0
            && let Some([x0, y0, x1, y1]) = screens.get(&s.t)
        {
            out.push(format!("{} screen {x0},{y0},{x1},{y1}", s.t));
        }
        last = Some(s);
    }
    out
}

#[cfg(test)]
mod tests;
