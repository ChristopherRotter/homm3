//! The map over time (E24 M2 T2 and T3a, *provisional*): the original's adventure loop on a clock
//! of whole milliseconds, for the map standing still, the view scrolling and the selected hero
//! walking (`docs/epics/E24-design.md` §3.1, §3.4, §3.5), from a sequence's start and the player's
//! actions at their milliseconds. Each millisecond gives a [`State`]: what the original's memory
//! holds (the view's corner, the objects' frame steps, the palette steps, the cursor, the selected
//! hero, its walk's drawing and the path's arrows) and what its screen shows (the view as last
//! drawn and the cursor); [`lines`] writes the recorded timeline's lines from them (design §3.2).
//! The cursor over the view is M3's: [`Unplayable`].
//!
//! Per millisecond, as the managers' loop takes it (§3.1): the palette tick when due, the mouse
//! poll when due (the cursor's point), then the idle message (the edge scroll, then the idle
//! step's redraw when due), then each action given at that millisecond as its message (the edge
//! scroll before each, §3.4). The comparisons give the actions after the millisecond's idle turns,
//! as the harness gives its inputs (`h3map.py frames`); the window (T4) takes them the same way.
//!
//! A walk (§3.5, M2 T3a) takes time inside one action: a direction key steps the selected hero
//! (`adventure_key_down` 0x408E97), a second click on the path's destination walks the path
//! (`hero_move_loop` 0x407B80). Its frames wait for their times (`wait_until` 0x4F82F0), whose
//! turns run only the tick (the poll and the palettes): no idle redraw, no edge scroll, the actions
//! given meanwhile waiting. The hero walks given tiles (M2: the path the original showed, from the
//! recording; M4 and M5: the engine's path and move), never a path of the client's own.

use alloc::collections::{BTreeMap, VecDeque};
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

use h3_map::model::Pos;
use h3_render::Bugs;
use h3_render::screen::Screen;

use crate::layout::{AdventureLayout, DIRECTIONS};
use crate::live::Arrow;
use crate::scene::ViewAt;
use crate::sound::{Cue, Sound};
use crate::tables::{Tables, WalkArt};

/// The mouse manager's type of the adventure cursor (`cursor_defs` 0x67FF38: 1, `cradvntr.def`),
/// the timeline's `cursor` lines' first number.
pub const CURSOR_TYPE: u32 = 1;

/// The last green arrow (1 to 25 green, a step the hero pays today; 26 to 50 red: design §2.10).
pub const GREEN: u16 = 25;

/// The scroll's and the hero's eight directions, clockwise from north (the original's 0 to 7,
/// `view_scroll` 0x4195C0; a hero's facing, its record's +0x47).
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

    /// The direction of a step from one tile to a neighbour on its level, if `to` is one.
    #[must_use]
    pub fn between(from: Pos, to: Pos) -> Option<Self> {
        let delta = [
            i32::from(to.x) - i32::from(from.x),
            i32::from(to.y) - i32::from(from.y),
        ];
        Self::ALL
            .into_iter()
            .find(|d| d.delta() == delta && from.z == to.z)
    }
}

/// What the player does (the window's actions: T4's key table binds the keys to them; the
/// original's defaults in design §3.4). A key's release does nothing on the map.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// Ctrl and a direction key: the view one scroll step that way (`adventure_key_down`
    /// 0x408C40), without the edge cursor.
    Scroll(Direction),
    /// A direction key alone: the selected hero one step that way (design §3.5).
    Step(Direction),
    /// The pointer moved to a point of the frame.
    Pointer([i32; 2]),
    /// The left button clicked at a point (the pointer moved there): a tile clicked shows the
    /// selected hero's path to it, its destination clicked again the hero walks it (design §3.5).
    Click([i32; 2]),
}

/// Why the timeline cannot play an action yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unplayable {
    /// The pointer over the view: the cursor by the tile under it, M3's.
    OverView([i32; 2]),
    /// A click off the view: the panels', M3's.
    Panel([i32; 2]),
    /// A click on a tile no given path leads to from the selected hero (or with none): M2 walks
    /// the paths recorded, the engine's come with M4.
    NoPath([i32; 3]),
}

impl fmt::Display for Unplayable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OverView([x, y]) => write!(f, "the pointer over the view at ({x}, {y}) (M3)"),
            Self::Panel([x, y]) => write!(f, "a click off the view at ({x}, {y}) (M3)"),
            Self::NoPath([x, y, z]) => {
                write!(f, "no path given from the selected hero to ({x}, {y}, {z})")
            }
        }
    }
}

/// The numbers the loop runs by: the tables' (`[map]`, its heroes' standing groups, `[map.walk]`),
/// the layout's (the frame, the view's rectangle and tile, `[scroll]`, `[cursor]`) and whether the
/// original's bugs are reproduced (the edge zones of MCL-Q6, the cursor's even x: E17 design §9.9;
/// for the comparisons only).
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
    /// A tile's side (32): a step's slide, and the mouse's tile.
    pub tile: i32,
    /// The view's middle tile (9, 8): the tile the map's first and last column and row stop at,
    /// and the one a step centres the hero on (`hero_walk_animation` 0x48041D, `hero_select`
    /// 0x417E44).
    pub middle: [i32; 2],
    /// The edge zones' width.
    pub edge: i32,
    /// The arrow's frame.
    pub arrow: u32,
    /// The first edge cursor's frame.
    pub edges: u32,
    /// The hero's standing group by its facing (`hero_map_group` 0x4D9110).
    pub standing: Vec<u32>,
    /// The walk's groups, frames and times.
    pub walk: WalkArt,
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
            tile: layout.view.tile,
            middle: layout.scroll.middle,
            edge: layout.scroll.edge,
            arrow: layout.cursor.arrow,
            edges: layout.cursor.edges,
            standing: tables.map.heroes.groups.clone(),
            walk: tables.map.walk.clone(),
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

    /// The tile under a point of the view as the original finds it (`view_mouse_tile` 0x40DEB0,
    /// `adventure_mouse_move` 0x40E3D2): the view's corner plus the point's x and y divided by a
    /// tile's side. The tiles are drawn 8 pixels lower (§1.2): whether the point is the screen's,
    /// so that a row's last 8 pixels give the tile below, is M3's open point; the sequences click
    /// tiles' middles.
    #[must_use]
    pub fn tile_at(&self, view: ViewAt, [x, y]: [i32; 2]) -> [i32; 3] {
        [
            view.x + x / self.tile.max(1),
            view.y + y / self.tile.max(1),
            i32::from(view.z),
        ]
    }
}

/// The selected hero: its tile and facing (the timeline's `hero` lines).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hero {
    /// The tile it stands on.
    pub at: Pos,
    /// Where it faces (its record's +0x47).
    pub facing: Direction,
}

/// Where a sequence starts (its 0: the palette tick, the redraw, the mouse poll and the scroll's
/// last time all at 0, as the harness sets them).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Start {
    /// The view's corner.
    pub view: ViewAt,
    /// The map's tiles a side.
    pub size: i32,
    /// The scroll speed (0 to 2).
    pub scroll_speed: usize,
    /// The hero speed (1 to 4).
    pub hero_speed: usize,
    /// The pointer's point.
    pub pointer: [i32; 2],
    /// The current player's selected hero, standing on its tile (the harness sets a sequence's
    /// view through the minimap, which puts back the hero a turn's start lifted: design §3.5);
    /// without one the direction keys do nothing.
    pub hero: Option<Hero>,
    /// The paths a click on a tile can show, each its arrows in any order (M2: the original's,
    /// recorded; the engine's from M4): the one leading from the hero, neighbour by neighbour, to
    /// the tile clicked.
    pub paths: Vec<Vec<Arrow>>,
    /// The path shown at 0, its arrows in any order (a situation's kept still: the selected
    /// hero's; E24 M2 T4b): shown when it leads from the hero, one of the paths a click shows then.
    pub path: Vec<Arrow>,
    /// The window's live input (E24 M2 T4b, *provisional*): the pointer over the view shows the
    /// arrow (M3: the cursor by the tile under it), and a click off the view or on a tile no path
    /// leads to only moves the pointer (M3: the panels; M4: the engine's paths). A recorded
    /// sequence refuses them ([`Unplayable`]), so that its frames never show what M3 draws.
    pub live: bool,
    /// The map's sound (E24 M2 T5, design §3.11): the original's sound state before the map's
    /// screen starts, which the loop starts at 0 and drives; none for a map without the tables'
    /// sounds, which gives no cues.
    pub sound: Option<Sound>,
}

/// The cursor drawn: a frame of the layout's cursor and the point its hot spot is at.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cursor {
    /// The frame.
    pub frame: u32,
    /// The point.
    pub at: [i32; 2],
    /// Shown: a walk hides it from its first step to its end (`mouse_hide` 0x50D540: a key's
    /// 0x409013, the path's 0x407E7C).
    pub shown: bool,
}

/// The walk's drawing in memory (the timeline's `walk` lines): the lifted hero's group (the
/// adventure manager's +0x1FC), the frame pass 9 draws next (+0x200) and the view's pixel offset
/// (+0xF4, +0xF8), which every pass adds to every tile's place.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Walk {
    /// The group.
    pub group: u32,
    /// The frame.
    pub frame: u32,
    /// The view's offset, x and y.
    pub offset: [i32; 2],
}

/// A draw of the view: its corner, the objects' frame steps and the palette steps since the
/// sequence's 0 (the draw at 0 shows 0 of each: the still the live part at 0 gives), and the walk's
/// part: the view's offset, the selected hero, its lifted picture and the path's arrows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Draw {
    /// The view's corner.
    pub view: ViewAt,
    /// The objects' frame steps (the manager's +0x100 less its value at 0's draw).
    pub frames: u32,
    /// The palette steps (both kinds').
    pub cycles: u32,
    /// The view's pixel offset (a walk's frames slide the map under the lifted hero).
    pub offset: [i32; 2],
    /// The selected hero.
    pub hero: Option<Hero>,
    /// The lifted hero's group and frame (drawn at the view's middle by passes 5, 7 and 9, not on
    /// its tile; design §3.5); none while it stands on its tile.
    pub lifted: Option<[u32; 2]>,
    /// The path's arrows shown (memory's order).
    pub path: Vec<Arrow>,
}

/// The state after a millisecond.
#[derive(Debug, Clone, PartialEq, Eq)]
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
    /// The selected hero.
    pub hero: Option<Hero>,
    /// The walk's drawing in memory.
    pub walk: Walk,
    /// The path's arrows left, in memory's order (level, row, column: `Mem.path_arrows`).
    pub arrows: Vec<Arrow>,
}

/// A walk under way: inside a key's step or the path's walk, which wait for their frames' times.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Moving {
    /// A key's step (`adventure_key_down`: the messages given meanwhile wait for the loop), or the
    /// path's walk (`hero_move_loop`: it drains them after each step, 0x407FA7).
    key: bool,
    /// The step under way.
    direction: Direction,
    /// Whether it is the walk's last.
    last: bool,
    /// The steps after it.
    rest: VecDeque<(Direction, Pos)>,
    /// Its next frame.
    frame: usize,
    /// When the wait before that frame, or the step's end, ends.
    until: u32,
    /// The actions given meanwhile.
    queued: Vec<Action>,
}

/// The original's adventure loop (§3.1, §3.4, §3.5), one millisecond at a time.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Loop {
    rules: Rules,
    /// The map's last column and row the view's middle may show.
    last: i32,
    /// The scroll step's tiles.
    step: i32,
    /// The hero speed's frames: the pixels each slides the view by (none: one draw).
    pixels: Vec<i32>,
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
    hero: Option<Hero>,
    /// The selected hero lifted off its tile (the manager's +0x1EC, +0x20C).
    lifted: bool,
    walk: Walk,
    /// The path shown, in its order from the hero (its last arrow the destination), each arrow with
    /// the step onto it.
    path: Vec<(Direction, Arrow)>,
    paths: Vec<Vec<Arrow>>,
    moving: Option<Moving>,
    /// The window's live input ([`Start::live`]).
    live: bool,
    /// The map's sound and its cues not taken yet ([`Start::sound`]).
    sound: Option<Sound>,
}

impl Loop {
    /// The loop after the sequence's 0: the tick, the poll and the redraw there.
    ///
    /// # Errors
    /// [`Unplayable::OverView`] for a pointer over the view (but live).
    pub fn start(rules: Rules, start: &Start) -> Result<Self, Unplayable> {
        let step = rules.steps.get(start.scroll_speed).copied().unwrap_or(1);
        let pixels = start
            .hero_speed
            .checked_sub(1)
            .and_then(|i| rules.walk.pixels.get(i))
            .cloned()
            .unwrap_or_default();
        let cursor = Cursor {
            frame: rules.arrow,
            at: rules.drawn_at(start.pointer),
            shown: true,
        };
        let mut l = Self {
            last: start.size.saturating_sub(1),
            step,
            pixels,
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
                offset: [0, 0],
                hero: start.hero,
                lifted: None,
                path: Vec::new(),
            },
            hero: start.hero,
            lifted: false,
            walk: Walk {
                group: 0,
                frame: 0,
                offset: [0, 0],
            },
            path: start
                .hero
                .and_then(|h| chain(h.at, &start.path))
                .unwrap_or_default(),
            paths: start.paths.clone(),
            moving: None,
            live: start.live,
            sound: start.sound.clone(),
            rules,
        };
        if !l.path.is_empty() {
            l.paths.push(start.path.clone());
            l.drawn.path = l.arrows();
        }
        // The turn's start left the standing group by the facing and frame 0 (`hero_select`).
        l.stand();
        // The map's screen started before 0: its music and ambient sounds (0x419B3F), the hero
        // on its tile.
        let (middle, standing) = (l.middle(), l.standing());
        if let Some(sound) = &mut l.sound {
            sound.start(0, middle, standing);
        }
        if !l.live && l.rules.over_view(start.pointer) {
            return Err(Unplayable::OverView(start.pointer));
        }
        l.tick(0);
        l.idle_message(0);
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
            drawn: self.drawn.clone(),
            hero: self.hero,
            walk: self.walk,
            arrows: self.arrows(),
        }
    }

    /// What the screen's view shows: its last draw.
    #[must_use]
    pub fn drawn(&self) -> &Draw {
        &self.drawn
    }

    /// The cursor.
    #[must_use]
    pub fn cursor(&self) -> Cursor {
        self.cursor
    }

    /// The sound's cues given since the last take (E24 M2 T5), with their milliseconds.
    pub fn take_cues(&mut self) -> Vec<(u32, Cue)> {
        self.sound.as_mut().map(Sound::take).unwrap_or_default()
    }

    /// The millisecond `t` (after the last one run) with the actions given at it.
    ///
    /// # Errors
    /// An action the loop cannot play yet ([`Unplayable`]).
    pub fn run(&mut self, t: u32, actions: &[Action]) -> Result<(), Unplayable> {
        // The loop's turn and a walk's wait alike run the tick (0x4ED490: the poll, the palettes).
        self.tick(t);
        match self.moving.take() {
            Some(moving) => {
                if let Some(queued) = self.carry_on(t, moving) {
                    self.resume(t, queued)?;
                }
            }
            None => self.idle_message(t),
        }
        for &action in actions {
            match &mut self.moving {
                Some(moving) => {
                    // During a walk the mouse's point is read at once (GetCursorPos); the message
                    // waits.
                    if let Action::Pointer(at) | Action::Click(at) = action {
                        self.pointer = at;
                    }
                    moving.queued.push(action);
                }
                None => self.act(t, action)?,
            }
        }
        Ok(())
    }

    /// The tick (0x4ED490), each turn of the loop and of a walk's wait: the mouse poll when due
    /// (0x50D680: the cursor drawn where it reads the mouse), the palettes' step when due.
    fn tick(&mut self, t: u32) {
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
    }

    /// The idle message: the edge scroll before it, then the idle step (0x408A88).
    fn idle_message(&mut self, t: u32) {
        self.edge(t);
        // Each redraw steps the objects once (0x408B26); one later than a step leaves the deadline
        // at the clock, so the loop's next turn, in the same millisecond, redraws again: after a
        // walk the screen shows the last of them (M2 T3b, design §3.8).
        while t >= self.frame_at {
            self.draw();
            self.step_objects(t);
        }
    }

    /// One action given at `t`: its message, the edge scroll before it.
    fn act(&mut self, t: u32, action: Action) -> Result<(), Unplayable> {
        if let Action::Pointer(at) | Action::Click(at) = action {
            // The move's message: GetCursorPos (the edge's) already reads the new point.
            self.pointer = at;
        }
        self.edge(t);
        match action {
            Action::Scroll(direction) => {
                self.scroll(t, direction, false);
            }
            Action::Pointer(at) => {
                if !self.live && self.rules.over_view(at) {
                    return Err(Unplayable::OverView(at));
                }
                self.moved();
            }
            Action::Step(direction) => self.key_step(t, direction),
            Action::Click(at) => self.click(t, at)?,
        }
        Ok(())
    }

    /// The mouse's move (`adventure_mouse_move` 0x40E360): outside the view (0x40EB7D) an edge
    /// cursor stays while the mouse lies in a zone, the arrow otherwise. Over the view the cursor
    /// by the tile is M3's: kept (a click's own move, which the sequences follow at once by the
    /// pointer parked; a walk's end, where `walk-4` recorded the arrow it had); live, as outside.
    fn moved(&mut self) {
        if (self.live || !self.rules.over_view(self.pointer))
            && self.rules.edge_at(self.pointer).is_none()
        {
            self.cursor.frame = self.rules.arrow;
        }
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
    /// the corner moved and held within the limits; a corner that moved puts the lifted hero back
    /// on its tile (`adventure_hero_stop` 0x417680) and is drawn and shown at once, with the
    /// objects' step when the redraw is due. Whether it moved.
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
        self.lifted = false;
        self.draw();
        self.step_objects(t);
        true
    }

    /// A direction key alone (`adventure_key_down` 0x408E97): the selected hero one step that way
    /// if it may go there (the map's side here; the engine's test from M4). A path shown is
    /// cleared first, the view drawn and shown, the objects stepped once if due (0x408F52 to
    /// 0x409002).
    fn key_step(&mut self, t: u32, direction: Direction) {
        let Some(next) = self.hero.and_then(|h| self.next(h.at, direction)) else {
            return;
        };
        if !self.path.is_empty() {
            self.path.clear();
            self.draw();
            if t >= self.frame_at {
                self.steps = self.steps.saturating_add(1);
                let late = t - self.frame_at;
                self.frame_at = self
                    .frame_at
                    .saturating_add(self.rules.frame_ms.max(late).max(1));
            }
        }
        self.walk_off(t, true, direction, next, VecDeque::new());
    }

    /// A click on the view (M3 the rest): on the path's destination the hero walks the path, on
    /// another tile the path there is shown (`make_hero_path` 0x418DD0, which draws the view).
    /// Live, a click off the view or on a tile no path leads to is only the pointer's move.
    fn click(&mut self, t: u32, at: [i32; 2]) -> Result<(), Unplayable> {
        if !self.rules.over_view(at) {
            return self.refused(Unplayable::Panel(at));
        }
        let tile = self.rules.tile_at(self.view, at);
        let pos = |a: &Arrow| [i32::from(a.at.x), i32::from(a.at.y), i32::from(a.at.z)];
        if self.path.last().is_some_and(|(_, a)| pos(a) == tile) {
            self.walk_path(t);
            return Ok(());
        }
        let path = self.hero.and_then(|hero| {
            self.paths
                .iter()
                .filter_map(|p| chain(hero.at, p))
                .find(|c| c.last().is_some_and(|(_, a)| pos(a) == tile))
        });
        let Some(path) = path else {
            return self.refused(Unplayable::NoPath(tile));
        };
        self.path = path;
        self.draw();
        Ok(())
    }

    /// A click the loop cannot play: refused, or live the pointer's move only.
    fn refused(&mut self, why: Unplayable) -> Result<(), Unplayable> {
        if !self.live {
            return Err(why);
        }
        self.moved();
        Ok(())
    }

    /// The path's walk (`hero_move_loop` 0x407B80): the hero lifted without centring
    /// (`mobilize_hero` with 1, 0, 0, 0x407DEC); its steps onto the path's green arrows, with no
    /// wait between them (the first red one a step the points do not pay, which ends the walk:
    /// 0x480864; the engine's move from M4); none, and the walk ends at once.
    fn walk_path(&mut self, t: u32) {
        // The start tile's step sound (0x407D03), then the lift (0x407DEC).
        if let (Some(sound), Some(hero)) = (&mut self.sound, self.hero) {
            sound.walk_start(t, hero.at);
        }
        if !self.lifted {
            self.lift(t, false);
        }
        let mut steps: VecDeque<(Direction, Pos)> = self
            .path
            .iter()
            .take_while(|(_, a)| a.arrow <= GREEN)
            .map(|&(d, a)| (d, a.at))
            .collect();
        match steps.pop_front() {
            Some((direction, next)) => self.walk_off(t, false, direction, next, steps),
            None => self.finish(t, false),
        }
    }

    /// A walk's start: the cursor hidden, the first step taken, its frames until the first wait (a
    /// walk that ends at once, at speed 4, has no actions waiting, and the loop's turn after it
    /// changes nothing: its idle message's edge scroll ran before the action).
    fn walk_off(
        &mut self,
        t: u32,
        key: bool,
        direction: Direction,
        next: Pos,
        rest: VecDeque<(Direction, Pos)>,
    ) {
        self.cursor.shown = false;
        let moving = Moving {
            key,
            direction,
            last: rest.is_empty(),
            rest,
            frame: 0,
            until: t,
            queued: Vec::new(),
        };
        self.step(t, direction, next);
        self.carry_on(t, moving);
    }

    /// A step's start (`hero_move_step` 0x4805E0, then `hero_walk_animation` 0x480380): a hero not
    /// lifted (after a scroll) lifted, the view centred on it, drawn and shown (`mobilize_hero`
    /// with 0, 0, 1, 0x480914); the next tile's arrow zeroed (0x480B30: the next frame shows it
    /// gone); the offset 0, the walking group, the facing, the hero on the next tile at once and
    /// the view centred on it (0x48041D, never clamped); at speed 4 the view drawn once (0x480483),
    /// else the offset a tile back (`walk_start_offsets` 0x63D6F0) for the frames.
    fn step(&mut self, t: u32, direction: Direction, next: Pos) {
        if !self.lifted {
            self.lift(t, true);
            self.draw();
        }
        self.path.retain(|(_, a)| a.at != next);
        let n = usize::try_from(direction.number()).unwrap_or(0);
        self.walk.offset = [0, 0];
        self.walk.group = self.rules.walk.groups.get(n).copied().unwrap_or(0);
        self.hero = self.hero.map(|_| Hero {
            at: next,
            facing: direction,
        });
        self.centre();
        if self.pixels.is_empty() {
            self.frame_drawn(t);
        } else {
            let [dx, dy] = direction.delta();
            self.walk.offset = [self.rules.tile * dx, self.rules.tile * dy];
        }
    }

    /// The walk from `t`: each frame when its time comes (0x4804F0: the offset less its pixels, the
    /// view drawn and shown, its end its start plus the frame's time), each step's end (0x48117A
    /// on: at the walk's last step the standing group and frame 0, 0x4811CD; the offset 0,
    /// 0x481423), the path's walk draining the actions given meanwhile (a key or a click ends it
    /// after the step, the rest dropped: 0x407FA7), the next step at once; the walk's end. The
    /// walk left under way waits; one that ended gives the actions waiting for the loop's turn.
    fn carry_on(&mut self, t: u32, mut moving: Moving) -> Option<Vec<Action>> {
        loop {
            if t < moving.until {
                self.moving = Some(moving);
                return None;
            }
            if let Some(&pixels) = self.pixels.get(moving.frame) {
                let [dx, dy] = moving.direction.delta();
                self.walk.offset[0] -= pixels * dx;
                self.walk.offset[1] -= pixels * dy;
                self.frame_drawn(t);
                moving.frame += 1;
                moving.until = t.saturating_add(self.rules.walk.frame_ms);
                continue;
            }
            if moving.last {
                self.stand();
            }
            // The step's end's sounds (0x481216 to 0x48141B): the view's middle is the hero's.
            let middle = self.middle();
            if let (Some(sound), Some(hero)) = (&mut self.sound, self.hero) {
                sound.step_end(t, middle, hero.at, moving.last, self.walk.frame);
            }
            self.walk.offset = [0, 0];
            if !moving.key {
                if moving
                    .queued
                    .iter()
                    .any(|a| matches!(a, Action::Step(_) | Action::Click(_)))
                {
                    moving.rest.clear();
                }
                moving.queued.clear();
            }
            let Some((direction, next)) = moving.rest.pop_front() else {
                self.finish(t, moving.key);
                return Some(moving.queued);
            };
            moving.direction = direction;
            moving.last = moving.rest.is_empty();
            moving.frame = 0;
            self.step(t, direction, next);
        }
    }

    /// The walk's end: the path's walk clears the path once the hero stands on its destination,
    /// else shows it again for the rest and draws the view (`make_hero_path` 0x418DD0 at 0x4080C9;
    /// M2: the arrows left, as `walk-points` recorded; the engine's path from M4); `hero_walk_end`
    /// 0x47F7D0 (the standing group, frame 0); the cursor shown again where the mouse is now,
    /// by its move with its tile forgotten (`mouse_position_now` 0x50D500, its x even;
    /// `adventure_mouse_move` 0x40E360: a key's 0x4090F1, the path's 0x40812F).
    fn finish(&mut self, t: u32, key: bool) {
        if !key && !self.path.is_empty() {
            self.draw();
        }
        self.stand();
        if let Some(sound) = &mut self.sound {
            sound.walk_end(t);
        }
        self.cursor.at = self.rules.drawn_at(self.pointer);
        self.cursor.shown = true;
        self.moved();
    }

    /// The loop's turn after a walk at its last millisecond: the actions given during it (a key's
    /// step only: the path's walk dropped them) as their messages, then the idle message. One that
    /// walks again queues the rest.
    fn resume(&mut self, t: u32, queued: Vec<Action>) -> Result<(), Unplayable> {
        let mut queued = queued.into_iter();
        while let Some(action) = queued.next() {
            self.act(t, action)?;
            if let Some(moving) = &mut self.moving {
                moving.queued.extend(queued);
                return Ok(());
            }
        }
        self.idle_message(t);
        Ok(())
    }

    /// The selected hero lifted off its tile (`hero_select` with b clear: +0x20C, the standing
    /// group by its facing, frame 0).
    fn lift(&mut self, t: u32, centred: bool) {
        self.lifted = true;
        self.stand();
        if centred {
            self.centre();
        }
        // Its sounds (0x4180A0): the ambient sounds around the view's middle, restarted; the
        // music of the hero's terrain when it centred the view.
        let middle = self.middle();
        if let (Some(sound), Some(hero)) = (&mut self.sound, self.hero) {
            sound.select(t, middle, hero.at, centred);
        }
    }

    /// The tile at the view's middle (the corner plus the layout's middle: (9, 8)).
    fn middle(&self) -> [i32; 3] {
        let [mx, my] = self.rules.middle;
        [
            self.view.x.saturating_add(mx),
            self.view.y.saturating_add(my),
            i32::from(self.view.z),
        ]
    }

    /// The selected hero standing on its tile (not lifted).
    fn standing(&self) -> Option<Pos> {
        self.hero.filter(|_| !self.lifted).map(|h| h.at)
    }

    /// The standing group by the hero's facing and frame 0 (`hero_map_group` 0x4D9110).
    fn stand(&mut self) {
        let facing = self.hero.map_or(0, |h| h.facing.number());
        let n = usize::try_from(facing).unwrap_or(0);
        self.walk.group = self.rules.standing.get(n).copied().unwrap_or(0);
        self.walk.frame = 0;
    }

    /// The view centred on the selected hero: its tile less the view's middle (never clamped).
    fn centre(&mut self) {
        let [mx, my] = self.rules.middle;
        self.view = self.hero.map_or(self.view, |hero| ViewAt {
            x: i32::from(hero.at.x) - mx,
            y: i32::from(hero.at.y) - my,
            z: hero.at.z,
        });
    }

    /// The tile one step from `at` that way, if it lies on the map.
    fn next(&self, at: Pos, direction: Direction) -> Option<Pos> {
        let [dx, dy] = direction.delta();
        let on = |v: u8, d: i32| {
            u8::try_from(i32::from(v) + d)
                .ok()
                .filter(|&v| i32::from(v) <= self.last)
        };
        match (on(at.x, dx), on(at.y, dy)) {
            (Some(x), Some(y)) => Some(Pos { x, y, z: at.z }),
            _ => None,
        }
    }

    /// The path's arrows left in memory's order.
    fn arrows(&self) -> Vec<Arrow> {
        let mut arrows: Vec<Arrow> = self.path.iter().map(|&(_, a)| a).collect();
        arrows.sort_by_key(|a| (a.at.z, a.at.y, a.at.x));
        arrows
    }

    /// The view drawn and shown: at the objects' step and the palettes now, the walk's offset, the
    /// hero and the arrows.
    fn draw(&mut self) {
        self.drawn = Draw {
            view: self.view,
            frames: self.steps,
            cycles: self.ticks.saturating_sub(1),
            offset: self.walk.offset,
            hero: self.hero,
            lifted: self.lifted.then_some([self.walk.group, self.walk.frame]),
            path: self.arrows(),
        };
    }

    /// A walk's frame drawn: pass 9 steps the lifted hero's frame after it (0x47FF6D).
    fn frame_drawn(&mut self, t: u32) {
        self.draw();
        self.walk.frame = (self.walk.frame + 1)
            .checked_rem(self.rules.walk.frames)
            .unwrap_or(0);
        // At the frame's wrap to 0 the queued step sound (0x47FF88).
        if self.walk.frame == 0
            && let Some(sound) = &mut self.sound
        {
            sound.wrap(t);
        }
    }

    /// The objects' step after a redraw (0x408B26, `view_show_step` 0x40F270): once the clock has
    /// reached the deadline, +0x100 grows by one and the deadline moves on by the larger of the
    /// step and its lateness (a late one to the clock: the next idle message redraws again).
    fn step_objects(&mut self, t: u32) {
        if t >= self.frame_at {
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

/// A path's arrows in its order from the hero's tile, each with the step onto it: each the one
/// arrow left that neighbours the last on its level; `None` when one has none or several.
#[must_use]
pub fn chain(from: Pos, arrows: &[Arrow]) -> Option<Vec<(Direction, Arrow)>> {
    let mut left = arrows.to_vec();
    let mut at = from;
    let mut out = Vec::with_capacity(left.len());
    while !left.is_empty() {
        let near: Vec<(usize, Direction)> = left
            .iter()
            .enumerate()
            .filter_map(|(i, a)| Direction::between(at, a.at).map(|d| (i, d)))
            .collect();
        let [(i, direction)] = near[..] else {
            return None;
        };
        let arrow = left.remove(i);
        out.push((direction, arrow));
        at = arrow.at;
    }
    Some(out)
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
/// each one where it changed, in that order within a millisecond; after 0 the walk's (§3.5) where
/// they changed: the selected hero, the walk's drawing, the path's arrows; and a `screen` line
/// where `screens` gives the box the screen changed in (never at 0, whose screen is always kept).
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
        if let Some(l) = last {
            if let Some(h) = s.hero.filter(|_| s.hero != l.hero) {
                let at = h.at;
                let facing = h.facing.number();
                out.push(format!("{} hero {} {} {} {facing}", s.t, at.x, at.y, at.z));
            }
            if s.walk != l.walk {
                let Walk {
                    group,
                    frame,
                    offset,
                } = s.walk;
                let [x, y] = offset;
                out.push(format!("{} walk {group} {frame} {x} {y}", s.t));
            }
            if s.arrows != l.arrows {
                let arrows: String = s
                    .arrows
                    .iter()
                    .map(|a| format!(" {},{},{},{}", a.at.x, a.at.y, a.at.z, a.arrow))
                    .collect();
                out.push(format!("{} arrows{arrows}", s.t));
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

/// The box two frames differ in within `area` (left, top, right, bottom, the last two exclusive),
/// as the timeline's `screen` lines give it (the harness's `changed_box` over the screen outside
/// the panels); `None` when nothing in it differs. Frames of other sizes differ in all of it.
#[must_use]
pub fn changed(before: &Screen, after: &Screen, area: [i32; 4]) -> Option<[i32; 4]> {
    let [left, top, right, bottom] = area;
    if (before.width(), before.height()) != (after.width(), after.height()) {
        return Some(area);
    }
    let mut found: Option<[i32; 4]> = None;
    for y in top..bottom {
        for x in left..right {
            let (x64, y64) = (i64::from(x), i64::from(y));
            if before.pixel(x64, y64) != after.pixel(x64, y64) {
                let [x0, y0, x1, y1] = found.unwrap_or([x, y, x + 1, y + 1]);
                found = Some([x0.min(x), y0.min(y), x1.max(x + 1), y1.max(y + 1)]);
            }
        }
    }
    found
}

#[cfg(test)]
pub(crate) mod tests;
