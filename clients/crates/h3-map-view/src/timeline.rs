//! The map over time (E24 M2 T2 and T3a, *provisional*): the original's adventure loop on a clock
//! of whole milliseconds, for the map standing still, the view scrolling and the selected hero
//! walking (`docs/epics/E24-design.md` §3.1, §3.4, §3.5), from a sequence's start and the player's
//! actions at their milliseconds. Each millisecond gives a [`State`]: what the original's memory
//! holds (the view's corner, the objects' frame steps, the palette steps, the cursor, the selected
//! hero, its walk's drawing and the path's arrows; since E24 M3 T4b its movement points) and what
//! its screen shows (the view and, since E24 M3 T3b, the minimap as last drawn, since T4b the hero
//! and town lists as last shown, and the cursor); [`lines`] writes the recorded timeline's lines
//! from them (design §3.2).
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
//!
//! The status window (E24 M3 T5b2, design §4.9) is chosen again at the end of every view's draw
//! (`status_window_choose` 0x415DE0 from `adventure_view_draw` 0x40F861) and, forced, at a hero's
//! selection and after a key's step: a kind asked for a time ([`Start::asked`]) gives way to the
//! selection's window once its end has come; the date's window steps its picture's frame at a draw
//! at least the tables' step after the last, up to its last frame, and plays the day's sound when
//! built.

use alloc::collections::{BTreeMap, VecDeque};
use alloc::format;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::fmt;

use h3_map::model::Pos;
use h3_render::Bugs;
use h3_render::screen::Screen;

use crate::hover::{self, Hint, Hover, PanelItems, Selection};
use crate::layout::{AdventureLayout, DIRECTIONS};
use crate::lists::{SLOTS, top_showing};
use crate::live::{
    Arrow, Buttons, Chosen, ChosenHero, ChosenTown, DateStatus, Kingdom, KingdomStatus, Selected,
    Status,
};
use crate::minimap::{MinimapArt, divided};
use crate::panels;
use crate::scene::ViewAt;
use crate::sequence::{DEFAULT_HERO_SPEED, DEFAULT_SCROLL_SPEED};
use crate::sound::{Cue, Sound};
use crate::status::DateArt;
use crate::tables::{Tables, WalkArt};

/// The mouse manager's type of the adventure cursor (`cursor_defs` 0x67FF38: 1, `cradvntr.def`),
/// the timeline's `cursor` lines' first number.
pub const CURSOR_TYPE: u32 = 1;

/// The last green arrow (1 to 25 green, a step the hero pays today; 26 to 50 red: design §2.10).
pub use crate::live::GREEN;

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
    /// The left button clicked at a point (the pointer moved there): pressed and released at
    /// once ([`Action::Press`], [`Action::Release`]).
    Click([i32; 2]),
    /// The left button pressed at a point (the pointer moved there; E24 M3 T6b1, design §4.13):
    /// on the view a tile clicked shows the selected hero's path to it, its destination clicked
    /// again the hero walks it (design §3.5); on a panel's button the button held down until the
    /// release; on a list's slot its hero or town chosen.
    Press([i32; 2]),
    /// The left button released at a point (the pointer moved there): a button held clicks when
    /// the mouse is over it.
    Release([i32; 2]),
    /// A panel button's hotkey pressed (E24 M3 T6b1): the button pressed.
    Key(Hotkey),
    /// A panel button's hotkey released: the button it pressed clicks.
    KeyUp(Hotkey),
    /// Space (E24 M3 T6b3, design §4.15): the selected hero chosen again when it stands on its
    /// tile (the view centred on it), then the object it stands on visited.
    Visit,
    /// Another key released (a direction key's, Space's; E24 M3 T6b3): a message in the input
    /// queue (`input_key_message` 0x4EC0E0, kind 2) that does nothing on the map.
    Up,
}

impl Action {
    /// The input messages it is (`input_key_message` 0x4EC0E0, `input_mouse_message` 0x4EC290;
    /// design §4.15): a click two, a button's down and up; everything else one (*provisional*: a
    /// scroll's Control key, its own down and up, is not counted).
    #[must_use]
    pub const fn messages(self) -> usize {
        match self {
            Self::Click(_) => 2,
            _ => 1,
        }
    }

    /// Whether its message ends a path's walk after the step under way (`hero_move_loop`
    /// 0x407FA7: a key down, kind 1; the left button's or the right one's down, 8 and 0x20; a
    /// control's, 0x200).
    #[must_use]
    pub const fn ends_walk(self) -> bool {
        matches!(
            self,
            Self::Step(_)
                | Self::Scroll(_)
                | Self::Click(_)
                | Self::Press(_)
                | Self::Key(_)
                | Self::Visit
        )
    }
}

/// The input manager's ring holds 64 messages, of which 63 wait: the 64th written drops the
/// oldest (`input_key_message` 0x4EC1D9, `input_mouse_message` 0x4EC3F1; design §3.4, §4.15).
pub const QUEUED: usize = 63;

/// The actions waiting kept to the ring's [`QUEUED`] messages, the oldest dropped (a click's down
/// before its up).
fn keep_ring(queued: &mut Vec<Action>) {
    let mut n: usize = queued.iter().map(|a| a.messages()).sum();
    while n > QUEUED {
        match queued.first().copied() {
            Some(Action::Click(at)) => queued[0] = Action::Release(at),
            Some(_) => {
                queued.remove(0);
            }
            None => break,
        }
        n -= 1;
    }
}

/// A panel button's hotkey (the dialog's +0x48, scan codes `adventure_dialog_create` 0x401510
/// gives; E24 M3 T6b1, design §4.1, §4.13): its key down presses the button, its key up clicks it
/// (`def_button_message` 0x456190). Sleep's button takes Z while the selected hero is awake and W
/// while it sleeps (`sleep_button_def` 0x403CC0). The adventure map's own keys (0x408C40) take
/// none of these letters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Hotkey {
    /// K: the kingdom overview.
    Kingdom,
    /// U: the underground.
    Underground,
    /// Q: the quest log.
    Quests,
    /// Z: sleep.
    Sleep,
    /// W: wake (sleep's button while the hero sleeps).
    Wake,
    /// M: move hero.
    MoveHero,
    /// C: cast a spell.
    Cast,
    /// A: the adventure options.
    Adventure,
    /// O: the system options.
    System,
    /// H: next hero.
    NextHero,
    /// E: end turn.
    EndTurn,
}

impl Hotkey {
    /// Every hotkey, in the buttons' order.
    pub const ALL: [Self; 11] = [
        Self::Kingdom,
        Self::Underground,
        Self::Quests,
        Self::Sleep,
        Self::Wake,
        Self::MoveHero,
        Self::Cast,
        Self::Adventure,
        Self::System,
        Self::NextHero,
        Self::EndTurn,
    ];

    /// The button it presses (its control id).
    #[must_use]
    pub const fn button(self) -> u16 {
        match self {
            Self::Kingdom => 3,
            Self::Underground => 4,
            Self::Quests => 5,
            Self::Sleep | Self::Wake => 6,
            Self::MoveHero => 7,
            Self::Cast => 8,
            Self::Adventure => 9,
            Self::System => 10,
            Self::NextHero => 11,
            Self::EndTurn => 12,
        }
    }

    /// The original's key: the letter of its scan code (0x25, 0x16, 0x10, 0x2C, 0x11, 0x32, 0x2E,
    /// 0x1E, 0x18, 0x23, 0x12), as the adapter names it.
    #[must_use]
    pub const fn letter(self) -> &'static str {
        match self {
            Self::Kingdom => "K",
            Self::Underground => "U",
            Self::Quests => "Q",
            Self::Sleep => "Z",
            Self::Wake => "W",
            Self::MoveHero => "M",
            Self::Cast => "C",
            Self::Adventure => "A",
            Self::System => "O",
            Self::NextHero => "H",
            Self::EndTurn => "E",
        }
    }
}

/// Why the timeline cannot play an action yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unplayable {
    /// The pointer over the view: the cursor by the tile under it, M3's.
    OverView([i32; 2]),
    /// A click on a panel's item whose action is not built: the selected hero's or town's screen
    /// (M4, M5), a hero or town whose choice the live part does not give.
    Panel([i32; 2]),
    /// A click on a tile no given path leads to from the selected hero (or with none): M2 walks
    /// the paths recorded, the engine's come with M4.
    NoPath([i32; 3]),
    /// A button's command that opens a dialog or is the engine's action (M4, M5), by the
    /// button's id.
    Command(u16),
    /// Space with the selected hero on an object's entrance (E24 M3 T6b3): its visit is the
    /// engine's (M4, M5).
    Visit([i32; 3]),
}

impl fmt::Display for Unplayable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OverView([x, y]) => write!(f, "the pointer over the view at ({x}, {y}) (M3)"),
            Self::Panel([x, y]) => write!(f, "a click on a panel at ({x}, {y}) not built"),
            Self::NoPath([x, y, z]) => {
                write!(f, "no path given from the selected hero to ({x}, {y}, {z})")
            }
            Self::Command(id) => write!(f, "button {id}'s command (a dialog or the engine's)"),
            Self::Visit([x, y, z]) => {
                write!(f, "a visit of the object at ({x}, {y}, {z}) (the engine's)")
            }
        }
    }
}

/// The numbers the loop runs by: the tables' (`[map]`, its heroes' standing groups, `[map.walk]`),
/// the layout's (the frame, the view's rectangle, origin and tile, `[scroll]`, `[cursor]`) and
/// whether the original's bugs are reproduced (the edge zones of MCL-Q6, the cursor's even x: E17
/// design §9.9; the mouse's tile 8 pixels off, MCL-Q18; for the comparisons only).
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
    /// Where the view's tile (vx, vy) has its top-left corner on the screen (0, 8): the client's
    /// mouse's tile (MCL-Q18).
    pub origin: [i32; 2],
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
    /// The date's status window's pictures and step (E24 M3 T5b2, design §4.9).
    pub date: DateArt,
    /// The original's bugs or the client's fixes.
    pub bugs: Bugs,
    /// The panels' items the mouse finds (E24 M3 T6a1, design §4.10).
    pub panels: PanelItems,
    /// The minimap's art: a click's divisor by the map's side (E24 M3 T6b2, design §4.14).
    pub minimap: MinimapArt,
    /// How long a click on the status window asks the kingdom's or the date's window for, ms
    /// (E24 M3 T6b2).
    pub asked_ms: u32,
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
            origin: layout.view.origin,
            middle: layout.scroll.middle,
            edge: layout.scroll.edge,
            arrow: layout.cursor.arrow,
            edges: layout.cursor.edges,
            standing: tables.map.heroes.groups.clone(),
            walk: tables.map.walk.clone(),
            date: tables.map.status.date.clone(),
            bugs,
            panels: PanelItems::new(layout),
            minimap: tables.map.minimap.clone(),
            asked_ms: tables.map.status.asked,
        }
    }

    /// Whether an edge cursor stays with the mouse at a point outside the view
    /// (`adventure_mouse_move` 0x40EB7D): on the frame and within the edge zones' width of a side
    /// (the original's test of the mouse's x as it is, x < 16 or x > 784, y < 16 or y > 584; the
    /// client's: its zones, [`Rules::edge_at`]).
    #[must_use]
    pub fn keeps_edge(&self, point: [i32; 2]) -> bool {
        let [x, y] = point;
        let [width, height] = self.frame;
        match self.bugs {
            Bugs::Fixed => self.edge_at(point).is_some(),
            Bugs::Original => {
                (0..width).contains(&x)
                    && (0..height).contains(&y)
                    && (x < self.edge
                        || x > width - self.edge
                        || y < self.edge
                        || y > height - self.edge)
            }
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

    /// The tile under a point of the view: the view's corner plus the view's column and row under
    /// it ([`Rules::column_at`]).
    #[must_use]
    pub fn tile_at(&self, view: ViewAt, point: [i32; 2]) -> [i32; 3] {
        let [column, row] = self.column_at(point);
        [
            view.x.saturating_add(column),
            view.y.saturating_add(row),
            i32::from(view.z),
        ]
    }

    /// The view's column and row under a point (the manager's +0xEC, +0xF0): the drawn tile's,
    /// the point less the view's origin divided by a tile's side. The original's (`view_mouse_tile`
    /// 0x40DEB0, `adventure_mouse_move` 0x40E3D2) divide the screen's point itself, though the
    /// tiles are drawn 8 pixels lower (§1.2; E24 M3 T6a1, observed: design §4.10), so a drawn
    /// row's last 8 pixels give the tile below: the client's fix, the original's under
    /// [`Bugs::Original`] (MCL-Q18).
    #[must_use]
    pub fn column_at(&self, [x, y]: [i32; 2]) -> [i32; 2] {
        let side = self.tile.max(1);
        match self.bugs {
            Bugs::Fixed => {
                let [ox, oy] = self.origin;
                [
                    x.saturating_sub(ox).div_euclid(side),
                    y.saturating_sub(oy).div_euclid(side),
                ]
            }
            Bugs::Original => [x / side, y / side],
        }
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
    /// The viewer's kingdom at 0 as the panels show it (E24 M3 T2a, design §4.1; since T4b its
    /// heroes' points as the hero list last drew them, which a recording's write of the selected
    /// hero's points leaves: `points`).
    pub kingdom: Kingdom,
    /// The selected hero's movement points memory holds, each from its millisecond on (E24 M3
    /// T4b, design §4.5; *provisional*): M3's walks are scripted, so a recording gives them (its
    /// `points` lines, `Sequence::points`): the first at 0, the rest what the steps left (a step's
    /// cost is the engine's, M4). A step's end takes the one at its millisecond. Empty: the
    /// kingdom's, which a walk leaves.
    pub points: Vec<(u32, i32)>,
    /// Move hero's and next hero's states at 0 as the original last set them (a recording's, read
    /// from its memory); none: as the kingdom and the path give them ([`Buttons::of`]).
    pub buttons: Option<Buttons>,
    /// The status window's kind asked for a time at 0 (E24 M3 T5b2, design §4.9; a recording's
    /// `status`: the harness asks it so that 0's redraw builds it, the kingdom's window at 0 the
    /// window it built). None: the kingdom's date's or kingdom's window at 0 asked until no end
    /// (a situation's still), another the selection's (kind 0).
    pub asked: Option<Asked>,
    /// The mouse-over (E24 M3 T6a1, design §4.10): the map's tiles and the game's facts; none
    /// (a recording without them) refuses the pointer over the view but live, where it keeps the
    /// arrow.
    pub hover: Option<Arc<Hover>>,
    /// The map's levels (the underground's button: a panel's item taking the mouse).
    pub levels: usize,
    /// The map's heroes on their tiles and facings at 0, in the live part's order (E24 M3 T6b1:
    /// a hero chosen is one of them, [`Chosen`]'s index).
    pub heroes: Vec<Hero>,
    /// The selected hero's index among them ([`Start::hero`] its tile and facing).
    pub index: Option<usize>,
    /// What choosing each hero and town of the kingdom's shows (E24 M3 T6b1).
    pub chosen: Chosen,
}

/// A status window's kind asked for a time (`status_window_set_kind` 0x415D60: the manager's
/// +0x398 and +0x39C; E24 M3 T5b2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Asked {
    /// The kind: 1 the date's, 2 the kingdom's.
    pub kind: u8,
    /// The millisecond from which a view's draw shows the selection's window again.
    pub end: u32,
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

/// The minimap's last draw (`minimap_draw` 0x412C40; E24 M3 T3b, design §4.4): the view's corner
/// its box shows and the selected hero's tile, as they were then. A scroll that moves the view
/// draws it again (`view_scroll` 0x41973E), so does a hero's selection that centres the view
/// (`hero_select` 0x418046: a step lifting a hero not lifted) and each step's end
/// (`after_step_guard_check` 0x4800A5); through a step's frames its box and hero stay where they
/// were.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Minimap {
    /// The view's corner.
    pub view: ViewAt,
    /// The selected hero's tile.
    pub hero: Option<Pos>,
    /// The selected hero's index among the map's heroes ([`Start::heroes`]).
    pub index: Option<usize>,
}

/// A draw of the view: its corner, the objects' frame steps and the palette steps since the
/// sequence's 0 (the draw at 0 shows 0 of each: the still the live part at 0 gives), and the walk's
/// part: the view's offset, the selected hero, its lifted picture and the path's arrows; the
/// minimap as last drawn.
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
    /// The minimap (E24 M3 T3b).
    pub minimap: Minimap,
    /// The kingdom as the panels last showed it (E24 M3 T4b, design §4.5): the hero and town
    /// lists' slots as last drawn and shown (a step's end shows its hero's movement bar, a lift
    /// that centres the view and a walk's end that leaves the hero unable to move every slot).
    pub kingdom: Arc<Kingdom>,
    /// The status bar's hint as last shown (E24 M3 T6a1, design §4.10: each change drawn and
    /// shown at once).
    pub hint: Hint,
    /// The selected hero's index among the map's heroes ([`Start::heroes`]; E24 M3 T6b1: a
    /// hero chosen is another), none without one.
    pub selected: Option<usize>,
    /// The panels' buttons shown pressed, by id in order (E24 M3 T6b1: each press and release
    /// drawn and shown at once).
    pub pressed: Vec<u16>,
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
    /// Move hero's and next hero's states as the screen shows them (E24 M3 T2a, design §4.1:
    /// each change drawn and shown at once).
    pub buttons: Buttons,
    /// The selected hero's movement points (its +0x4D; E24 M3 T4b): the timeline's `points`.
    pub points: Option<i32>,
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
    /// The minimap's last draw.
    minimap: Minimap,
    walk: Walk,
    /// The path shown, in its order from the hero (its last arrow the destination), each arrow with
    /// the step onto it.
    path: Vec<(Direction, Arrow)>,
    paths: Vec<Vec<Arrow>>,
    moving: Option<Moving>,
    /// A path's walk started since the flag was last taken: it empties the input queue at its
    /// start (0x407E87), the messages behind its click gone ([`Loop::resume`]).
    flushed: bool,
    /// The window's live input ([`Start::live`]).
    live: bool,
    /// The map's sound and its cues not taken yet ([`Start::sound`]).
    sound: Option<Sound>,
    /// The viewer's kingdom in memory ([`Start::kingdom`], its selected hero's points
    /// [`Start::points`]'): a walk the points end leaves its hero unable to move.
    kingdom: Kingdom,
    /// The selected hero's points by millisecond ([`Start::points`]).
    points: Vec<(u32, i32)>,
    /// Move hero's and next hero's states.
    buttons: Buttons,
    /// The status window's kind asked (the manager's +0x398; 0 the selection's) and its end
    /// (+0x39C; E24 M3 T5b2).
    asked: u8,
    end: u32,
    /// When the date's window stepped its frame last, or was built (its panel's +0x44).
    stepped: u32,
    /// The kingdom's window, as the live part at 0 shows it.
    realm: Option<KingdomStatus>,
    /// The mouse-over ([`Start::hover`]) and the map's levels.
    hover: Option<Arc<Hover>>,
    levels: usize,
    /// The selected hero's record index (the mouse-over's).
    selected_id: Option<u8>,
    /// The selected hero walked since 0: the facts' reach is no longer its.
    walked: bool,
    /// The view's column and row the mouse was over at the last look (the manager's +0xEC,
    /// +0xF0); none after a walk's end or a key's step, and at 0.
    column: Option<[i32; 2]>,
    /// The panel's item the hint looked at last (0x65F228; none: -1).
    last_item: Option<u16>,
    /// What a click on the view would do (the manager's +0x40; T6b's).
    action: hover::Action,
    /// The map's heroes on their tiles and facings: the selected one's as it stood when last
    /// chosen away from (E24 M3 T6b1).
    heroes: Vec<Hero>,
    /// The selected hero's index among them.
    index: Option<usize>,
    /// What choosing each hero and town shows: a hero's path as it was when chosen away from.
    chosen: Chosen,
    /// The button the mouse holds down (`def_button_message`'s loop, 0x45641C: only the tick
    /// runs, the messages go to the button): its id and whether the mouse is over it.
    held: Option<(u16, bool)>,
    /// The buttons a hotkey holds pressed, with the key.
    keyed: Vec<(Hotkey, u16)>,
    /// The mouse holds the minimap down (`minimap_click`'s own loop from 0x40A344: the tick by
    /// each message's wait, every message but the mouse's moves and its release dropped; E24 M3
    /// T6b2).
    dragging: bool,
    /// A click's divisor on this map's minimap (the f32's bits: [`MinimapArt::click`]).
    scale: u32,
    /// An item of the status window holds the press (its state's bit 1, `sub_44FF60` 0x450030):
    /// the left button's next release, wherever, is its command (0x409FD8); a window built again
    /// meanwhile takes it with its items.
    status_held: bool,
}

impl Loop {
    /// The loop after the sequence's 0: the tick, the poll and the redraw there.
    ///
    /// # Errors
    /// [`Unplayable::OverView`] for a pointer over the view (but live).
    pub fn start(rules: Rules, start: &Start) -> Result<Self, Unplayable> {
        // A speed out of range is the original's default (`settings_check` 0x50B260: the scroll
        // speed 1, the hero speed 2; M2 T6a, the reviewer's R1-7).
        let step = rules
            .steps
            .get(start.scroll_speed)
            .or_else(|| rules.steps.get(DEFAULT_SCROLL_SPEED))
            .copied()
            .unwrap_or(1);
        let pixels = start
            .hero_speed
            .checked_sub(1)
            .and_then(|i| rules.walk.pixels.get(i))
            .or_else(|| rules.walk.pixels.get(DEFAULT_HERO_SPEED - 1))
            .cloned()
            .unwrap_or_default();
        let cursor = Cursor {
            frame: rules.arrow,
            at: rules.drawn_at(start.pointer),
            shown: true,
        };
        // The minimap at 0 as the still shows it (the harness set the view through it).
        let minimap = Minimap {
            view: start.view,
            hero: start.hero.map(|h| h.at),
            index: start.index,
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
                minimap,
                kingdom: Arc::new(start.kingdom.clone()),
                hint: Hint::Empty,
                selected: start.index,
                pressed: Vec::new(),
            },
            hero: start.hero,
            lifted: false,
            minimap,
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
            flushed: false,
            live: start.live,
            sound: start.sound.clone(),
            kingdom: start.kingdom.clone(),
            points: start.points.clone(),
            buttons: start
                .buttons
                .unwrap_or_else(|| Buttons::of(&start.kingdom, &start.path)),
            asked: 0,
            end: 0,
            stepped: 0,
            realm: None,
            selected_id: start
                .hero
                .zip(start.hover.as_ref())
                .and_then(|(h, hover)| hover.hero_at(h.at)),
            hover: start.hover.clone(),
            levels: start.levels,
            walked: false,
            column: None,
            last_item: None,
            action: -1,
            heroes: start.heroes.clone(),
            index: start.index,
            chosen: start.chosen.clone(),
            held: None,
            keyed: Vec::new(),
            dragging: false,
            scale: rules
                .minimap
                .click(usize::try_from(start.size).unwrap_or(usize::MAX)),
            status_held: false,
            rules,
        };
        // The status window at 0 (E24 M3 T5b2): the kind asked, else the date's or the kingdom's
        // shown asked until no end; the date's at its first frame built at 0, with its sound.
        let shown = start.kingdom.status.as_ref();
        (l.asked, l.end) = match (start.asked, shown.map(Status::kind)) {
            (Some(asked), _) => (asked.kind, asked.end),
            (None, Some(kind @ (1 | 2))) => (kind, u32::MAX),
            _ => (0, 0),
        };
        // The kingdom's window: as memory would build it (E24 M3 T6b2: a click's), else the one
        // shown.
        l.realm = start.kingdom.realm.clone().or_else(|| match shown {
            Some(Status::Kingdom(realm)) => Some(realm.clone()),
            _ => None,
        });
        // Memory's points at 0: a recording's write, which the hero list does not show yet.
        l.spend(0);
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
            let built = matches!(shown, Some(Status::Date(DateStatus { frame: 0 })));
            if built && !l.rules.date.shows(start.kingdom.date).week {
                sound.effect(0, &l.rules.date.sound);
            }
        }
        if !l.live && l.hover.is_none() && l.rules.over_view(start.pointer) {
            return Err(Unplayable::OverView(start.pointer));
        }
        // The status bar at 0 (E24 M3 T6a1): the hint of the pointer's place, as the staging's
        // parking left it (the panel's item looked at last its item).
        l.moved();
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
            buttons: self.buttons,
            points: self
                .selected_hero()
                .and_then(|i| self.kingdom.heroes.get(i))
                .map(|h| h.points),
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

    /// Move hero's and next hero's states (E24 M3 T2a).
    #[must_use]
    pub fn buttons(&self) -> Buttons {
        self.buttons
    }

    /// The sound's cues given since the last take (E24 M2 T5), with their milliseconds.
    pub fn take_cues(&mut self) -> Vec<(u32, Cue)> {
        self.sound.as_mut().map(Sound::take).unwrap_or_default()
    }

    /// The millisecond `t` (after the last one run) with the actions given at it, one after
    /// another, each handled before the next is given (the harness's way: a click's walk at hero
    /// speed 4 ends before the next input, a later one waits behind a walk under way).
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
                    // waits in the input ring, the oldest dropped past its 63 (design §4.15).
                    if let Action::Pointer(at)
                    | Action::Click(at)
                    | Action::Press(at)
                    | Action::Release(at) = action
                    {
                        self.pointer = at;
                    }
                    moving.queued.push(action);
                    keep_ring(&mut moving.queued);
                }
                None => {
                    self.act(t, action, true)?;
                    // Given after the click was handled: behind the walk's emptying of the queue.
                    self.flushed = false;
                }
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
        // The mouse holding a button down runs the button's loop: the tick only (0x45641C); so
        // does the minimap's (0x40A344, E24 M3 T6b2).
        if self.held.is_some() || self.dragging {
            return;
        }
        self.edge(t);
        // Each redraw steps the objects once (0x408B26); one later than a step leaves the deadline
        // at the clock, so the loop's next turn, in the same millisecond, redraws again: after a
        // walk the screen shows the last of them (M2 T3b, design §3.8).
        while t >= self.frame_at {
            self.draw(t);
            self.step_objects(t);
        }
    }

    /// One action's message at `t`, the edge scroll before it: `fresh`, given now; else given
    /// during a walk and taken after it, whose point the mouse has left (the edge scroll and the
    /// cursor read the mouse's point now, GetCursorPos: the latest given, M2 T6a, the reviewer's
    /// R1-3; a click's tile is its message's).
    fn act(&mut self, t: u32, action: Action, fresh: bool) -> Result<(), Unplayable> {
        if self.held.is_some() || self.dragging {
            return self.holding(t, action);
        }
        if fresh
            && let Action::Pointer(at) | Action::Click(at) | Action::Press(at) | Action::Release(at) =
                action
        {
            // The move's message: GetCursorPos (the edge's) already reads the new point.
            self.pointer = at;
        }
        self.edge(t);
        match action {
            Action::Scroll(direction) => {
                self.scroll(t, direction, false);
            }
            Action::Pointer(at) => {
                if !self.live && self.hover.is_none() && self.rules.over_view(at) {
                    return Err(Unplayable::OverView(at));
                }
                self.moved();
            }
            Action::Step(direction) => self.key_step(t, direction),
            Action::Click(at) | Action::Press(at) | Action::Release(at) => {
                // The move's message comes first (E24 M3 T6a1): with the mouse-over, the tile's
                // cursor and hint; off the view the panel's hint.
                if self.hover.is_some() || !self.rules.over_view(at) {
                    self.moved();
                }
                match action {
                    Action::Release(_) => self.release(t),
                    Action::Press(_) => self.press(t, at)?,
                    _ => {
                        self.press(t, at)?;
                        if self.held.is_some() || self.dragging {
                            self.holding(t, Action::Release(at))?;
                        } else {
                            self.release(t);
                        }
                    }
                }
            }
            Action::Key(key) => self.key(t, key),
            Action::KeyUp(key) => self.key_up(t, key)?,
            Action::Visit => self.visit(t)?,
            Action::Up => {}
        }
        Ok(())
    }

    /// An action while the mouse holds a button down (`def_button_message`'s own loop from
    /// 0x456338: the tick only, every message the button's): a move over it presses it again,
    /// with its sound (0x4563A2), one off it releases it (0x4563B5), each drawn and shown at once;
    /// the left button's release ends the hold, a click when the button shows pressed (0x456451:
    /// its command, `adventure_command` 0x409A70); every other message is taken and dropped.
    fn holding(&mut self, t: u32, action: Action) -> Result<(), Unplayable> {
        if self.dragging {
            self.drag(t, action);
            return Ok(());
        }
        let (Some((id, over)), Action::Pointer(at) | Action::Release(at)) = (self.held, action)
        else {
            return Ok(());
        };
        self.pointer = at;
        let inside = self.rules.panels.holds(id, at);
        if inside != over {
            self.held = Some((id, inside));
            self.show_pressed(id, inside);
            if inside {
                self.press_sound(t);
            }
        }
        if matches!(action, Action::Release(_)) {
            self.held = None;
            // The mouse looked at again where the hold left it (recorded: released off a
            // button, the hint the point's, `press-buttons`; *provisional*, not read).
            self.moved();
            if inside {
                self.show_pressed(id, false);
                self.command(t, id)?;
            }
            // The loop's next turn in the same millisecond: the redraw the hold kept from its
            // deadline.
            self.idle_message(t);
        }
        Ok(())
    }

    /// The left button pressed at `at` (`adventure_click` 0x4097E0 for the item's message, E24
    /// M3 T6b1, design §4.13): the item under the mouse that takes it ([`hover::takes`]); a
    /// button held down (`def_button_message` 0x456328; not the quest log without a quest, whose
    /// state 0x20 the button skips: 0x456287), the view's area and the text over it a tile's
    /// click, a hero list's slot its hero chosen, a town list's its town, the minimap its own
    /// loop ([`Loop::minimap_press`]); every other item nothing (the bars, the status bar). The
    /// status window's items, the dialog's newest (`dialog_send_to_items` 0x5FEDE0 from the
    /// list's head), take the press first: the item holds it (E24 M3 T6b2, design §4.14).
    fn press(&mut self, t: u32, at: [i32; 2]) -> Result<(), Unplayable> {
        if self.kingdom.status.is_some() && self.rules.panels.in_status(at) {
            self.status_held = true;
            return Ok(());
        }
        let (kingdom, buttons, levels) = (&self.kingdom, self.buttons, self.levels);
        let item = self
            .rules
            .panels
            .at(at, |i| hover::takes(i, kingdom, buttons, levels));
        match item {
            Some(id) if is_button(id) => {
                if id != panels::id::QUESTS || self.kingdom.quests {
                    self.held = Some((id, true));
                    self.show_pressed(id, true);
                    self.press_sound(t);
                }
                Ok(())
            }
            Some(panels::id::VIEW | panels::id::TEXT) => self.click(t, at),
            Some(panels::id::MINIMAP) => {
                self.minimap_press(t, at);
                Ok(())
            }
            Some(id) => match list_slot(id) {
                Some((false, slot)) => self.hero_slot(t, slot, at),
                Some((true, slot)) => self.town_slot(t, slot, at),
                None => Ok(()),
            },
            None => Ok(()),
        }
    }

    /// A hotkey's key down (`def_button_message` 0x45659F): the button it names presses when it
    /// takes the key (shown, not disabled, not skipped as the quest log without a quest; sleep's
    /// by the selected hero's sleep), with its sound; else the key goes on to the map's keys,
    /// which take none of the letters (0x408C40).
    fn key(&mut self, t: u32, key: Hotkey) {
        let id = key.button();
        let asleep = self
            .selected_hero()
            .and_then(|i| self.kingdom.heroes.get(i))
            .is_some_and(|h| h.asleep);
        let (wake, sleep) = (key == Hotkey::Wake, key == Hotkey::Sleep);
        let takes = !panels::disabled(&self.kingdom, self.buttons, self.levels)
            .iter()
            .any(|&(d, off)| d == id && off)
            && (id != panels::id::QUESTS || self.kingdom.quests)
            && !(wake && !asleep)
            && !(sleep && asleep);
        if takes {
            if !self.keyed.contains(&(key, id)) {
                self.keyed.push((key, id));
            }
            self.show_pressed(id, true);
            self.press_sound(t);
        }
    }

    /// A hotkey's key up (0x4564D6): the button its key pressed released, drawn and shown, and
    /// its command; nothing for a key that pressed none.
    fn key_up(&mut self, t: u32, key: Hotkey) -> Result<(), Unplayable> {
        let Some(at) = self.keyed.iter().position(|&(k, _)| k == key) else {
            return Ok(());
        };
        let (_, id) = self.keyed.remove(at);
        self.show_pressed(id, false);
        self.command(t, id)
    }

    /// A button pressed or released, drawn and shown at once.
    fn show_pressed(&mut self, id: u16, pressed: bool) {
        let shown = &mut self.drawn.pressed;
        match (shown.binary_search(&id), pressed) {
            (Err(at), true) => shown.insert(at, id),
            (Ok(at), false) => {
                shown.remove(at);
            }
            _ => {}
        }
    }

    /// A press's sound (`button.wav`, 0x4568B8).
    fn press_sound(&mut self, t: u32) {
        if let Some(sound) = &mut self.sound {
            sound.press(t);
        }
    }

    /// A button's command (`adventure_command` 0x409A70, its switch 0x409AB4): the
    /// underground's (0x409F6B), next hero's (0x409D89), the hero list's arrows (0x403220) and
    /// the town list's (0x403280); the rest a dialog's or the engine's (M4, M5).
    fn command(&mut self, t: u32, id: u16) -> Result<(), Unplayable> {
        match id {
            panels::id::UNDERGROUND => {
                self.underground(t);
                Ok(())
            }
            panels::id::NEXT_HERO => {
                self.next_hero(t);
                Ok(())
            }
            id if panels::id::HEROES.contains(&id) || panels::id::TOWNS.contains(&id) => {
                let towns = panels::id::TOWNS.contains(&id);
                let up = id == panels::id::HEROES[0] || id == panels::id::TOWNS[0];
                self.scroll_list(towns, up);
                Ok(())
            }
            id => self.unbuilt(Unplayable::Command(id)),
        }
    }

    /// What is not built yet: refused in a recording (its frames would show what M3 does not
    /// draw), nothing live.
    fn unbuilt(&self, why: Unplayable) -> Result<(), Unplayable> {
        if self.live { Ok(()) } else { Err(why) }
    }

    /// The underground's command (`adventure_command` 0x409F6B; E24 M3 T6b2, design §4.14): on
    /// a map of two levels the selected hero put back on its tile (0x409F84), the view on the
    /// other level at the same corner (0x409FAE), the button's picture by it (0x403C40) and the
    /// whole screen updated and shown (`adventure_full_update` 0x417420 with 1: every item drawn,
    /// so the status bar's hint hidden, the status window built again, the view and the
    /// minimap); no sound, no mouse move.
    fn underground(&mut self, t: u32) {
        if self.levels < 2 {
            return;
        }
        self.lifted = false;
        self.view.z = u8::from(self.view.z == 0);
        self.draw_lists();
        self.choose(t, true);
        self.draw_minimap();
        self.draw(t);
    }

    /// The left button pressed on the minimap (`minimap_click` 0x40A0C0, the left button's;
    /// E24 M3 T6b2, design §4.14): the selected hero put back on its tile (0x40A112), the view's
    /// corner the tile under the point less the view's middle, held within the scroll's limits
    /// (0x40A1D7 to 0x40A267), the minimap drawn and shown (0x40A280) and the view (0x40A2BB,
    /// shown 0x40A2D3) with the objects' step when due; then its own loop holds the mouse
    /// ([`Loop::drag`]).
    fn minimap_press(&mut self, t: u32, at: [i32; 2]) {
        self.lifted = false;
        let Some([left, top, ..]) = self.rules.panels.area(panels::id::MINIMAP) else {
            return;
        };
        let [mx, my] = self.rules.middle;
        let tile = |v: i32, from: i32| divided(v.saturating_sub(from).unsigned_abs(), self.scale);
        self.view.x = tile(at[0], left)
            .saturating_sub(mx)
            .clamp(-mx, self.last.saturating_sub(mx).max(-mx));
        self.view.y = tile(at[1], top)
            .saturating_sub(my)
            .clamp(-my, self.last.saturating_sub(my).max(-my));
        self.dragging = true;
        self.minimap_drawn(t);
    }

    /// An action while the mouse holds the minimap down (its loop, 0x40A344 to 0x40A5BE: each
    /// turn the messages waiting, up to the release, the last move kept, the rest dropped): a move
    /// (the latest point, GetCursorPos) held inside the area (0x40A3C4 to 0x40A41B; its height by
    /// the width, harmless on the square area), its tile within the map (0x40A47B to 0x40A4BC: by
    /// the width) less the view's middle the corner, never otherwise held, the minimap and the
    /// view drawn and shown with the objects' step when due, the corner changed or not; the
    /// release (after the move the harness gives it at another point) ends the loop, the
    /// manager's next turn in the same millisecond (the redraw the hold kept from its deadline).
    /// The mouse manager's move (0x40A42A) draws nothing (its message handler 0x4EC560 returns
    /// at once); no hint, no cursor change, no edge scroll.
    fn drag(&mut self, t: u32, action: Action) {
        let (Action::Pointer(at) | Action::Release(at)) = action else {
            return;
        };
        let moved = at != self.pointer;
        self.pointer = at;
        if matches!(action, Action::Pointer(_)) || moved {
            if let Some([left, top, right, _]) = self.rules.panels.area(panels::id::MINIMAP) {
                let last = right.saturating_sub(1);
                let bottom = top
                    .saturating_add(right.saturating_sub(left))
                    .saturating_sub(1);
                let (x, y) = (
                    at[0].clamp(left, last.max(left)),
                    at[1].clamp(top, bottom.max(top)),
                );
                let tile = |v: i32, from: i32| {
                    divided(v.saturating_sub(from).unsigned_abs(), self.scale)
                        .clamp(0, self.last.max(0))
                };
                let [mx, my] = self.rules.middle;
                self.view.x = tile(x, left).saturating_sub(mx);
                self.view.y = tile(y, top).saturating_sub(my);
            }
            self.minimap_drawn(t);
        }
        if matches!(action, Action::Release(_)) {
            self.dragging = false;
            self.idle_message(t);
        }
    }

    /// The minimap and the view drawn and shown at the corner, the objects stepped when due (a
    /// minimap's click and its moves, 0x40A2D8 and 0x40A561: as the idle step steps them).
    fn minimap_drawn(&mut self, t: u32) {
        self.draw_minimap();
        self.draw(t);
        self.step_objects(t);
    }

    /// The left button released with no loop of its own holding it (`dialog_send_to_items`
    /// 0x5FEDE0: the first item from the list's head whose state holds the press, `sub_44FF60`
    /// 0x450079; E24 M3 T6b2, design §4.14): a status window's item gives its command
    /// ([`Loop::status_click`]); the others' commands do nothing (the view's, the minimap's: its
    /// own loop took its release, so its state still holds the press, harmlessly).
    fn release(&mut self, t: u32) {
        if core::mem::take(&mut self.status_held) {
            self.status_click(t);
        }
    }

    /// The status window's click (`adventure_command` 0x409FD8, its items 2000 to 2200; design
    /// §4.8, §4.14): the kingdom's asked gives the date's, another kind asked but the selection's
    /// gives the selection's; the selection's gives the kingdom's, or the date's while the
    /// kingdom's shows; the date's and the kingdom's for the tables' time from now (0x40A030);
    /// then the window built again, drawn and shown (`status_window_choose` with 1, 1, 1).
    fn status_click(&mut self, t: u32) {
        let shown = self.kingdom.status.as_ref().map(Status::kind);
        match self.asked {
            2 => {
                self.asked = 1;
                self.end = t.saturating_add(self.rules.asked_ms);
            }
            0 => {
                self.asked = if shown == Some(2) { 1 } else { 2 };
                self.end = t.saturating_add(self.rules.asked_ms);
            }
            _ => self.asked = 0,
        }
        self.choose(t, true);
    }

    /// A list's arrow (`hero_list_scroll` 0x403220, `town_list_scroll` 0x403280): its first slot
    /// one up (not below 0) or one down (while more than its slots lie below it), the list
    /// updated without a place (its arrows' states by the new first slot), drawn and shown.
    fn scroll_list(&mut self, towns: bool, up: bool) {
        let list = usize::from(towns);
        let len = if towns {
            self.kingdom.towns.len()
        } else {
            self.kingdom.heroes.len()
        };
        let top = &mut self.kingdom.tops[list];
        if up {
            *top = top.saturating_sub(1);
        } else if len.saturating_sub(*top) > SLOTS {
            *top = top.saturating_add(1);
        }
        self.draw_lists();
    }

    /// A hero list's slot clicked (`adventure_click` 0x409829): the hero in it chosen, unless the
    /// slot lies past the list (the slot, not the top plus it, is tested: 0x409844) or holds the
    /// selected hero, whose screen it opens (M4, M5: refused).
    fn hero_slot(&mut self, t: u32, slot: usize, at: [i32; 2]) -> Result<(), Unplayable> {
        let i = self.kingdom.tops[0].saturating_add(slot);
        if slot >= self.kingdom.heroes.len() || i >= self.kingdom.heroes.len() {
            return Ok(());
        }
        if self.kingdom.selected == Selected::Hero(i) {
            return self.unbuilt(Unplayable::Panel(at));
        }
        let Some(chosen) = self.chosen_hero(i).cloned() else {
            return self.unbuilt(Unplayable::Panel(at));
        };
        self.keep_path();
        self.choose_hero(t, i, chosen);
        Ok(())
    }

    /// A town list's slot clicked (`adventure_click` 0x40987D, a slot holding a town: the empty
    /// ones take no click): move hero disabled and a path shown cleared, the view drawn and
    /// shown with the objects' step when due (0x4098BB to 0x40998A); then the town chosen, or its
    /// screen opened when it is the selected one (M4, M5: refused).
    fn town_slot(&mut self, t: u32, slot: usize, at: [i32; 2]) -> Result<(), Unplayable> {
        let i = self.kingdom.tops[1].saturating_add(slot);
        let chosen = self.chosen.towns.get(i).cloned().flatten();
        let (Some(chosen), false) = (chosen, self.kingdom.selected == Selected::Town(i)) else {
            return self.unbuilt(Unplayable::Panel(at));
        };
        self.buttons.move_hero = false;
        self.keep_path();
        if !self.path.is_empty() {
            self.path.clear();
            self.draw(t);
            self.step_objects(t);
        }
        self.choose_town(t, i, chosen);
        Ok(())
    }

    /// Next hero's command (0x409D89): a path shown cleared, the view drawn and shown with the
    /// objects' step when due (0x409DF3, 0x409DFE); then the player's next hero chosen
    /// (`next_hero_find` 0x4BAA40: from the one after the selected, or the first, to the list's
    /// end, then from the first to the selected, the first awake that can still move), none
    /// found nothing.
    fn next_hero(&mut self, t: u32) {
        self.keep_path();
        if !self.path.is_empty() {
            self.path.clear();
            self.draw(t);
            self.step_objects(t);
        }
        let heroes = &self.kingdom.heroes;
        let from = self.selected_hero().map_or(0, |i| i.saturating_add(1));
        let next = (from..heroes.len())
            .chain(0..from.min(heroes.len()))
            .find(|&i| heroes.get(i).is_some_and(|h| h.movable && !h.asleep));
        if let Some((i, chosen)) = next.and_then(|i| Some((i, self.chosen_hero(i)?.clone()))) {
            self.choose_hero(t, i, chosen);
        }
    }

    /// Space (`adventure_key_down`'s case of scan code 57, 0x408CC4; design §4.15): nothing
    /// without a selected hero. A hero standing on its tile (+0x20C clear) is chosen again: move
    /// hero disabled (0x408D06), a path shown cleared with the view drawn and shown and the
    /// objects' step when due (0x408D31 to 0x408D6F), then `hero_select` with 0, 0, 1 (0x408D83:
    /// [`Loop::reselect`]). A hero lifted (after a step, a walk, a choice) is not. Then the
    /// object on the hero's tile is visited when the tile is an entrance (map_item +0xD bit 0x10,
    /// its class not 3: `visit_and_update` 0x4AAAA0 at 0x408E65): the engine's (M4, M5).
    fn visit(&mut self, t: u32) -> Result<(), Unplayable> {
        if self.hero.is_none() {
            return Ok(());
        }
        if !self.lifted {
            self.buttons.move_hero = false;
            if !self.path.is_empty() {
                let path = core::mem::take(&mut self.path);
                self.draw(t);
                self.step_objects(t);
                self.path = path;
            }
            self.reselect(t);
        }
        // The hero's tile as the hero's lift leaves it (`hero_select`: the object under it, its
        // own class and entrance gone; observed in memory). Without the mouse-over's facts it
        // is not known: nothing (*provisional*).
        let (Some(hover), Some(selection)) = (&self.hover, self.selection()) else {
            return Ok(());
        };
        let at = [
            i32::from(selection.at.x),
            i32::from(selection.at.y),
            i32::from(selection.at.z),
        ];
        match hover.current(at, Some(selection)) {
            Some(tile) if tile.entrance && tile.class != 3 => self.unbuilt(Unplayable::Visit(at)),
            _ => Ok(()),
        }
    }

    /// The selected hero chosen again (`hero_select` 0x417B20 with 0, 0, 1, as
    /// [`Loop::choose_hero`] but with what it shows now): put back on its tile and lifted, the
    /// view centred on it, the lists, the status window and the minimap ([`Loop::lift`]); its
    /// path made again from its destination (0x417FE2: the arrows it shows, *provisional*: the
    /// engine's path from M4; move hero by its green arrows); the view drawn and the whole screen
    /// shown; the mouse's move posted (0x418146).
    fn reselect(&mut self, t: u32) {
        self.lifted = false;
        self.lift(t, true);
        self.buttons.move_hero = self.green();
        self.draw(t);
        self.look_again();
    }

    /// What choosing the kingdom's hero `i` shows, when the live part gives it and its index
    /// names a map's hero.
    fn chosen_hero(&self, i: usize) -> Option<&ChosenHero> {
        self.chosen
            .heroes
            .get(i)?
            .as_ref()
            .filter(|c| c.hero < self.heroes.len())
    }

    /// The selected hero's path kept as its choice shows it again (its destination stays: the
    /// original makes the path again from it), and its tile and facing; then none shown.
    fn keep_path(&mut self) {
        let arrows = self.arrows();
        if let (Some(i), Some(index), Some(hero)) = (self.selected_hero(), self.index, self.hero) {
            if let Some(Some(chosen)) = self.chosen.heroes.get_mut(i) {
                chosen.path = arrows;
            }
            if let Some(h) = self.heroes.get_mut(index) {
                *h = hero;
            }
        }
    }

    /// The kingdom's hero `i` chosen (`hero_select` 0x417B20 with 0, 0, 1; design §4.13; the
    /// selected hero's path kept first, [`Loop::keep_path`]): move hero disabled and a path
    /// shown cleared (0x417B80, 0x417BA0), the selected hero put back
    /// on its tile (0x417BAC); the hero selected, lifted off its tile and the view centred on it
    /// (a hero of the viewer's always sees its tile: 0x417DAE), the lists updated, the status
    /// window built (0x41800B) and the minimap drawn ([`Loop::lift`]); its path made again
    /// (0x417FE2: the path its choice shows, move hero by its green arrows); the view drawn and
    /// the whole screen shown (0x418082); the mouse's move posted with the view's column
    /// forgotten (0x418146: +0xEC 0, the row kept) and taken at once.
    fn choose_hero(&mut self, t: u32, i: usize, chosen: ChosenHero) {
        self.buttons.move_hero = false;
        self.path.clear();
        self.lifted = false;
        let hero = self.heroes.get(chosen.hero).copied();
        self.kingdom.selected = Selected::Hero(i);
        self.index = Some(chosen.hero);
        self.hero = hero;
        self.selected_id = self
            .hover
            .as_ref()
            .and_then(|h| h.facts().hero_list.get(i))
            .map(|h| h.0);
        // The facts' reach and the hero's own marks were the staged hero's (design §4.13: the
        // engine's from M4).
        self.walked = true;
        self.kingdom.selection = Some(Status::Hero(chosen.status));
        self.path = hero
            .and_then(|h| chain(h.at, &chosen.path))
            .unwrap_or_default();
        self.lift(t, true);
        self.buttons.move_hero = self.green();
        self.draw(t);
        self.look_again();
    }

    /// The kingdom's town `i` chosen (`town_select` 0x417830 with 0, 1; design §4.13): the
    /// selected hero put back and none selected, the town selected, the view centred on its
    /// place (never clamped, 0x4178BD), the town list updated to show it and the hero list
    /// (0x417953, 0x417961), move hero disabled and a path shown cleared (0x4179AF), the whole
    /// screen updated (`adventure_full_update` 0x417420: the status window built, the minimap and
    /// the view drawn); the music of the terrain at the view's middle and its ambient sounds
    /// (0x417A33, 0x417AF8); the mouse's move posted as after a hero's choice.
    fn choose_town(&mut self, t: u32, i: usize, chosen: ChosenTown) {
        self.lifted = false;
        self.hero = None;
        self.index = None;
        self.selected_id = None;
        self.kingdom.selected = Selected::Town(i);
        let top = &mut self.kingdom.tops[1];
        *top = top_showing(*top, i, self.kingdom.towns.len());
        let [mx, my] = self.rules.middle;
        self.view = ViewAt {
            x: i32::from(chosen.at.x) - mx,
            y: i32::from(chosen.at.y) - my,
            z: chosen.at.z,
        };
        self.buttons.move_hero = false;
        self.path.clear();
        self.kingdom.selection = Some(Status::Town(chosen.status));
        self.draw_lists();
        self.choose(t, true);
        self.draw_minimap();
        self.draw(t);
        let middle = self.middle();
        if let Some(sound) = &mut self.sound {
            sound.select(t, middle, chosen.at, true);
        }
        self.look_again();
    }

    /// The mouse's move a choice posts (`input_post_mouse_move` 0x4ECC00) with the view's
    /// column forgotten (+0xEC 0, its row kept), taken at once.
    fn look_again(&mut self) {
        let row = self.column.map_or(-1, |[_, y]| y);
        self.column = Some([0, row]);
        self.moved();
    }

    /// The mouse's move (`adventure_mouse_move` 0x40E360, on the viewer's turn; design §4.10).
    /// Over the view, with the mouse-over, when the view's column or row under the mouse changed:
    /// off the map or on a tile the viewer has not explored the arrow; else the tile's hint
    /// ([`Hover::hint`]) and its cursor ([`Hover::cursor`]). Without the mouse-over the cursor is
    /// kept (a recording's click, which the sequences follow at once by the pointer parked; a
    /// walk's end, where `walk-4` recorded the arrow it had), live the arrow. Outside the view
    /// (0x40EB7D) an edge cursor stays while the mouse lies within the edge zones' width of a
    /// side ([`Rules::keeps_edge`]), any other the arrow; then the panel's hint
    /// ([`Loop::panel_hint`]).
    fn moved(&mut self) {
        if self.rules.over_view(self.pointer) {
            match self.hover.clone() {
                Some(hover) => self.over_map(&hover),
                None if self.live => self.cursor.frame = self.rules.arrow,
                None => {}
            }
            return;
        }
        if !(self.edge_cursor() && self.rules.keeps_edge(self.pointer)) {
            self.cursor.frame = self.rules.arrow;
        }
        self.panel_hint();
    }

    /// The mouse over the view's tile under it, when its column or row changed (0x40E3E6).
    fn over_map(&mut self, hover: &Hover) {
        let column = self.rules.column_at(self.pointer);
        if self.column == Some(column) {
            return;
        }
        self.column = Some(column);
        self.action = -1;
        let tile = self.rules.tile_at(self.view, self.pointer);
        if !hover.explored(tile) {
            self.cursor.frame = self.rules.arrow;
            return;
        }
        let selection = self.selection();
        let middle = column == self.rules.middle;
        self.show_hint(hover.hint(tile, middle, selection));
        let cursor = hover.cursor(tile, selection, !self.walked);
        self.cursor.frame = cursor.frame;
        self.action = cursor.action;
    }

    /// The panel's hint (`adventure_panel_hint` 0x403010): the item under the mouse, when it is
    /// not the one looked at last: none sets the arrow; its hint shown at once.
    fn panel_hint(&mut self) {
        let (kingdom, buttons, levels) = (&self.kingdom, self.buttons, self.levels);
        let item = self
            .rules
            .panels
            .at(self.pointer, |i| hover::takes(i, kingdom, buttons, levels));
        if item == self.last_item {
            return;
        }
        self.last_item = item;
        if item.is_none() {
            self.cursor.frame = self.rules.arrow;
        }
        let facts = self.hover.as_ref().map(|h| h.facts());
        let hint = hover::panel_hint(item, &self.kingdom, facts);
        self.show_hint(hint);
    }

    /// The status bar's text set, drawn and shown at once.
    fn show_hint(&mut self, hint: Hint) {
        self.drawn.hint = hint;
    }

    /// The selected hero as the mouse-over reads it.
    fn selection(&self) -> Option<Selection> {
        let (hero, id) = (self.hero?, self.selected_id?);
        let points = self
            .selected_hero()
            .and_then(|i| self.kingdom.heroes.get(i))
            .map_or(0, |h| h.points);
        Some(Selection {
            id,
            at: hero.at,
            lifted: self.lifted,
            points,
        })
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
    /// on its tile (`adventure_hero_stop` 0x417680), draws the minimap (0x41973E, shown at once)
    /// and the view, shown at once, with the objects' step when the redraw is due. Whether it
    /// moved.
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
        self.draw_minimap();
        self.draw(t);
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
        // Move hero (design §4.1): disabled at the step's start (0x408F0B), enabled again when the
        // step lifts a hero not lifted, whose selection makes the step's own path (`hero_select`
        // through `mobilize_hero`, `make_hero_path` with its third argument; *provisional*: as
        // the recordings show it).
        self.buttons.move_hero = !self.lifted;
        if !self.path.is_empty() {
            self.path.clear();
            self.draw(t);
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

    /// A click on the view (the press on its area, E24 M3 T6b1): on the path's destination the
    /// hero walks the path, on another tile the path there is shown (`make_hero_path` 0x418DD0,
    /// which draws the view). Live, a click on a tile no path leads to is only the pointer's
    /// move.
    fn click(&mut self, t: u32, at: [i32; 2]) -> Result<(), Unplayable> {
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
        self.draw(t);
        // Move hero by the path's green arrows (`make_hero_path` from 0x40AA17, its third argument
        // set: 0x4191A2; design §4.1).
        self.buttons.move_hero = self.green();
        Ok(())
    }

    /// Whether the path shown holds a green arrow, a step the hero pays today.
    fn green(&self) -> bool {
        self.path
            .iter()
            .any(|(_, a)| (1..=GREEN).contains(&a.arrow))
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
        // The input queue emptied after the lift (`mouse_hide`, 0x4EC570 at 0x407E87; M2 T6a, the
        // reviewer's R1-5): the messages behind the click are gone.
        self.flushed = true;
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
        self.walked = true;
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
            self.draw(t);
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
    /// 0x481423), the path's walk taking the actions given meanwhile (0x407F80 to 0x407FFA: up
    /// to the first key-down or button, a step, a scroll or a click, which ends it after the step;
    /// those before it dropped, those behind it waiting), the next step at once; the walk's end.
    /// The walk left under way waits; one that ended gives the actions waiting for the loop's
    /// turn.
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
            // The points the step left (0x48118C, after its frames).
            self.spend(t);
            if moving.last {
                self.stand();
            }
            // The step's end's sounds (0x481216 to 0x48141B): the view's middle is the hero's.
            let middle = self.middle();
            if let (Some(sound), Some(hero)) = (&mut self.sound, self.hero) {
                sound.step_end(t, middle, hero.at, moving.last, self.walk.frame);
            }
            self.walk.offset = [0, 0];
            // The guard's test after the step (`after_step_guard_check` 0x480090, from 0x48169D)
            // draws the minimap first, shown at once (0x4800A5).
            self.draw_minimap();
            // Back from the step, its hero's slot drawn and shown (`hero_list_slot` with -1, 1, 1:
            // the path's 0x407EEF, a key's 0x409069).
            self.draw_slot();
            if !moving.key {
                // The messages' kinds 1, 8, 0x20 and 0x200 end the walk (0x407FA7): a key-down,
                // Ctrl's too (M2 T6a, the reviewer's R1-1), a hotkey's and Space's (E24 M3 T6b3),
                // and a button's down; the loop reads up to the first of them, taken and not
                // handled, the rest stays in the queue for the main loop (R1-2): a click's up
                // among them. A key without a binding (Ctrl alone) is no action: the adapter's
                // keys only.
                let ends = moving.queued.iter().position(|a| a.ends_walk());
                match ends {
                    Some(i) => {
                        moving.rest.clear();
                        if let Some(Action::Click(at)) = moving.queued.get(i).copied() {
                            moving.queued.drain(..i);
                            moving.queued[0] = Action::Release(at);
                        } else {
                            moving.queued.drain(..=i);
                        }
                    }
                    None => moving.queued.clear(),
                }
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

    /// The walk's end: `hero_walk_end` 0x47F7D0 (the standing group, frame 0; for a walk a
    /// message ended at 0x408002, before the path's redraw: M2 T6a, the reviewer's R1-4); the
    /// path's walk clears the path once the hero stands on its destination, else shows it again
    /// for the rest and draws the view (`make_hero_path` 0x418DD0 at 0x4080C9, with the option
    /// Show Route on, 0x698774; M2: the arrows left, as `walk-points` recorded; the engine's path
    /// from M4); the cursor shown again where the mouse is now,
    /// by its move with its tile forgotten (`mouse_position_now` 0x50D500, its x even;
    /// `adventure_mouse_move` 0x40E360: a key's 0x4090F1, the path's 0x40812F).
    fn finish(&mut self, t: u32, key: bool) {
        self.stand();
        if let Some(sound) = &mut self.sound {
            sound.walk_end(t);
        }
        if !key && self.path.is_empty() {
            // On its destination the path is cleared: move hero off (design §4.1; the walk keeps
            // the state its click set until here, *provisional*: as the recordings show it).
            self.buttons.move_hero = false;
        }
        if !key && !self.path.is_empty() {
            self.draw(t);
            // The rest's path made again with its third argument (0x4080C9): move hero by its
            // green arrows. With none left the points did not pay the next step, which set them
            // to 0 (0x480864): the hero cannot move on (*provisional*: the engine's from M4), and
            // next hero follows the kingdom (0x408193 to 0x4081C4; design §4.1).
            self.buttons.move_hero = self.green();
            if !self.buttons.move_hero {
                if let Selected::Hero(i) = self.kingdom.selected
                    && let Some(hero) = self.kingdom.heroes.get_mut(i)
                {
                    hero.movable = false;
                }
                // The hero list drawn and the whole screen shown (`hero_list_update` with -1, 1,
                // 1: 0x4081B9).
                self.draw_lists();
                self.buttons.next_hero = self.kingdom.next_hero();
            }
        }
        self.cursor.at = self.rules.drawn_at(self.pointer);
        self.cursor.shown = true;
        // The mouse looked at again (+0xEC -1: 0x40812F, 0x4090F1).
        self.column = None;
        self.moved();
        if key {
            // The status window built again (0x4090FE, E24 M3 T5b2).
            self.choose(t, true);
        }
    }

    /// The loop's turn after a walk at its last millisecond: the actions given during it and
    /// left in the queue (a key's step leaves all; the path's walk those behind the one that
    /// ended it) as their messages, then the idle message. One that walks again queues the rest;
    /// a path's walk empties the queue at its start (0x407E87: M2 T6a, the reviewer's R1-5).
    fn resume(&mut self, t: u32, queued: Vec<Action>) -> Result<(), Unplayable> {
        let mut queued = queued.into_iter();
        while let Some(action) = queued.next() {
            self.act(t, action, false)?;
            if core::mem::take(&mut self.flushed) {
                queued.by_ref().for_each(drop);
            }
            if let Some(moving) = &mut self.moving {
                moving.queued.extend(queued);
                return Ok(());
            }
        }
        self.idle_message(t);
        Ok(())
    }

    /// The selected hero lifted off its tile (`hero_select` with b clear: +0x20C, the standing
    /// group by its facing, frame 0); with `centred` (its c) the lists drawn, the hero list
    /// scrolled to show the hero (`hero_list_update` with its place, 1, 0 and `town_list_update`
    /// with -1, 1, 0: 0x417F09, 0x417F19), the view centred on it and the minimap drawn (0x418046;
    /// the whole screen shown at once, 0x41809B).
    fn lift(&mut self, t: u32, centred: bool) {
        self.lifted = true;
        self.stand();
        if centred {
            if let Some(i) = self.selected_hero() {
                let top = &mut self.kingdom.tops[0];
                *top = top_showing(*top, i, self.kingdom.heroes.len());
            }
            self.draw_lists();
            // The status window built again (0x41800B with its c set, E24 M3 T5b2).
            self.choose(t, true);
            self.centre();
            self.draw_minimap();
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

    /// The selected hero's place in the kingdom's heroes, if any.
    fn selected_hero(&self) -> Option<usize> {
        match self.kingdom.selected {
            Selected::Hero(i) if i < self.kingdom.heroes.len() => Some(i),
            _ => None,
        }
    }

    /// The selected hero's points memory holds at `t` as [`Start::points`] gives them (none
    /// given: as they are).
    fn spend(&mut self, t: u32) {
        let given = self.points.iter().take_while(|(at, _)| *at <= t).last();
        if let (Some(&(_, points)), Some(i)) = (given, self.selected_hero())
            && let Some(hero) = self.kingdom.heroes.get_mut(i)
        {
            hero.points = points;
        }
    }

    /// The selected hero's slot drawn and shown (`hero_list_slot` 0x403560 with -1, 1, 1; design
    /// §4.5), when the list shows it: the portrait, both bars and the frame drawn, the rectangle
    /// (609, 212 + 32 i), 48 × 32 shown, which holds the movement bar and leaves out the mana bar
    /// (and the portrait's last 8 columns): the hero's points shown, its mana as last shown (the
    /// slot's hero and portrait change only by an update, which shows the whole screen).
    fn draw_slot(&mut self) {
        let Some(i) = self.selected_hero() else {
            return;
        };
        let top = self.kingdom.tops[0];
        let points = self.kingdom.heroes.get(i).map(|h| h.points);
        let shown = i.checked_sub(top).is_some_and(|slot| slot < SLOTS);
        if let Some(points) = points.filter(|_| shown)
            && self
                .drawn
                .kingdom
                .heroes
                .get(i)
                .is_some_and(|h| h.points != points)
            && let Some(hero) = Arc::make_mut(&mut self.drawn.kingdom).heroes.get_mut(i)
        {
            hero.points = points;
        }
    }

    /// Both lists drawn as memory holds them, shown with the whole screen (`hero_list_update`
    /// 0x4032E0, `town_list_update` 0x403420 with their redraws: every item of the dialog drawn).
    fn draw_lists(&mut self) {
        if *self.drawn.kingdom != self.kingdom {
            self.drawn.kingdom = Arc::new(self.kingdom.clone());
        }
        // Every item drawn (`dialog_redraw` 0x5FF020 with the whole range): the chat's edit 38,
        // drawn after the status bar over its box, paints its picture with no text over the
        // hint (`edit_draw` 0x5BB400; E24 M3 T6b1, design §4.13). Memory keeps the hint and the
        // item looked at last: only another item's or the map's next hint shows a text again.
        self.drawn.hint = Hint::Empty;
    }

    /// The minimap drawn (`minimap_draw` 0x412C40) and shown: the view's corner and the selected
    /// hero's tile now.
    fn draw_minimap(&mut self) {
        self.minimap = Minimap {
            view: self.view,
            hero: self.hero.map(|h| h.at),
            index: self.index,
        };
        self.drawn.minimap = self.minimap;
    }

    /// The view drawn and shown: at the objects' step and the palettes now, the walk's offset, the
    /// hero and the arrows; the minimap as last drawn; then the status window chosen (0x40F861,
    /// E24 M3 T5b2).
    fn draw(&mut self, t: u32) {
        self.drawn = Draw {
            view: self.view,
            frames: self.steps,
            cycles: self.ticks.saturating_sub(1),
            offset: self.walk.offset,
            hero: self.hero,
            lifted: self.lifted.then_some([self.walk.group, self.walk.frame]),
            path: self.arrows(),
            minimap: self.minimap,
            kingdom: Arc::clone(&self.drawn.kingdom),
            hint: self.drawn.hint.clone(),
            selected: self.index,
            pressed: core::mem::take(&mut self.drawn.pressed),
        };
        self.choose(t, false);
    }

    /// The status window chosen (`status_window_choose` 0x415DE0; design §4.8, §4.9): a kind
    /// asked whose end has come back to the selection's (0); then the kind asked built (1 the
    /// date's, 2 the kingdom's) or for 0 the selection's window (the hero's, else the town's, else
    /// the date's), drawn and shown at once. Without `force` (a view's draw, 0x40F861) the kind
    /// shown builds nothing, but the date's steps its frame ([`Loop::date_window`]); with it (a
    /// hero's selection 0x41800B, a key's step 0x4090FE) the window is built again. A window the
    /// live part does not give (none shown, or the selection's unknown) is left as it is.
    fn choose(&mut self, t: u32, force: bool) {
        if self.kingdom.status.is_none() {
            return;
        }
        if self.asked != 0 && t >= self.end {
            self.asked = 0;
        }
        let shown = self.kingdom.status.as_ref().map(Status::kind);
        let selection = self.kingdom.selection.clone().or_else(|| {
            self.kingdom
                .status
                .clone()
                .filter(|s| matches!(s, Status::Hero(_) | Status::Town(_)))
        });
        let window = match (self.asked, self.kingdom.selected, selection) {
            (1, _, _) | (0, Selected::None, _) => None,
            (2, _, _) => match &self.realm {
                Some(realm) => Some(Status::Kingdom(realm.clone())),
                None => return,
            },
            (0, Selected::Hero(_), Some(s @ Status::Hero(_)))
            | (0, Selected::Town(_), Some(s @ Status::Town(_))) => Some(s),
            _ => return,
        };
        let (next, built) = match window {
            None => {
                let date = match self.kingdom.status {
                    Some(Status::Date(date)) if !force => Some(date),
                    _ => None,
                };
                (self.date_window(t, date), date.is_none())
            }
            Some(s) => {
                let built = force || shown != Some(s.kind());
                (built.then_some(s), built)
            }
        };
        // A window built anew drops the old one's items, a press they held with them (E24 M3
        // T6b2: `status_panel_set` 0x402C10).
        if built {
            self.status_held = false;
        }
        if let Some(status) = next.filter(|s| self.kingdom.status.as_ref() != Some(s)) {
            // The selection's window kept as another takes its place (E24 M3 T6b2: a click's),
            // for the chooser's kind 0 to build again.
            let selection = |s: &Status| matches!(s, Status::Hero(_) | Status::Town(_));
            if self.kingdom.selection.is_none()
                && !selection(&status)
                && self.kingdom.status.as_ref().is_some_and(selection)
            {
                self.kingdom.selection.clone_from(&self.kingdom.status);
            }
            self.kingdom.status = Some(status.clone());
            Arc::make_mut(&mut self.drawn.kingdom).status = Some(status);
        }
    }

    /// The date's window (E24 M3 T5b2, design §4.9): the one `shown` its frame stepped once the
    /// tables' step has passed since its last step and while it is short of its picture's last
    /// (`status_date_update` 0x4511A0: the frame drawn and its rectangle shown at once); else
    /// built at its first frame (0x450DD0), the day's playing its sound.
    fn date_window(&mut self, t: u32, shown: Option<DateStatus>) -> Option<Status> {
        let shows = self.rules.date.shows(self.kingdom.date);
        if let Some(date) = shown {
            let next = date.frame.saturating_add(1);
            if next >= shows.frames || t.saturating_sub(self.stepped) < self.rules.date.step {
                return None;
            }
            self.stepped = t;
            return Some(Status::Date(DateStatus { frame: next }));
        }
        self.stepped = t;
        if !shows.week
            && let Some(sound) = &mut self.sound
        {
            sound.effect(t, &self.rules.date.sound);
        }
        Some(Status::Date(DateStatus { frame: 0 }))
    }

    /// A walk's frame drawn: pass 9 steps the lifted hero's frame after it (0x47FF6D).
    fn frame_drawn(&mut self, t: u32) {
        self.draw(t);
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

/// Whether the dialog's item `id` is a button (`def_button_message`'s): the buttons 3 to 12 and
/// the lists' arrows.
const fn is_button(id: u16) -> bool {
    matches!(id, 3..=14 | 30 | 31)
}

/// The list and slot of a list's item (`adventure_click`'s table 0x409A44): a hero list's
/// selection frame (false; over its portrait, which the original's table names too, and taking
/// the mouse in every slot: the portrait is never the item under it), a town list's picture
/// (true); none for another item (the bars take no click).
const fn list_slot(id: u16) -> Option<(bool, usize)> {
    match id {
        39..=43 => Some((false, (id - 39) as usize)),
        32..=36 => Some((true, (id - 32) as usize)),
        _ => None,
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

/// The timeline's lines (design §3.2): at 0 the view, the objects, the cycles, the cursor, the
/// buttons (move hero's and next hero's states, 1 enabled; E24 M3 T2a) and the selected hero's
/// points (E24 M3 T4b; none without one), then each one where it changed, in that order within a
/// millisecond; after 0 the walk's (§3.5) where
/// they changed: the selected hero, the walk's drawing, the path's arrows; and a `screen` line
/// where `screens` gives the box the screen changed in (never at 0, whose screen is always kept).
#[must_use]
pub fn lines(states: &[State], screens: &BTreeMap<u32, [i32; 4]>) -> Vec<String> {
    let mut out = Vec::new();
    let mut last: Option<&State> = None;
    // The last points written: a state without a selected hero writes none and keeps them (E24
    // M3 T6b1: a town chosen, then a hero of the same points, writes no line).
    let mut points_written = None;
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
        let buttons = |s: &State| {
            let b = s.buttons;
            format!(
                "buttons {} {}",
                u8::from(b.move_hero),
                u8::from(b.next_hero)
            )
        };
        for f in [view, objects, cycles, cursor, buttons] {
            if changed(f) {
                out.push(format!("{} {}", s.t, f(s)));
            }
        }
        if let Some(points) = s.points.filter(|p| points_written != Some(*p)) {
            out.push(format!("{} points {points}", s.t));
            points_written = Some(points);
        }
        if let Some(l) = last {
            if s.hero != l.hero {
                // None selected (a town chosen): the line without numbers.
                out.push(s.hero.map_or_else(
                    || format!("{} hero", s.t),
                    |h| {
                        let (at, facing) = (h.at, h.facing.number());
                        format!("{} hero {} {} {} {facing}", s.t, at.x, at.y, at.z)
                    },
                ));
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

/// The box two frames differ in outside the `masked` rectangles (left, top, right, bottom, the last
/// two exclusive), as the timeline's `screen` lines give it (the harness's `changed_box` over the
/// screen outside the panels' mask, `clients/map/masks.toml`; E24 M3 T2a: the whole screen but the
/// mask); `None` when nothing there differs. Frames of other sizes differ in all of `after`.
#[must_use]
pub fn changed(before: &Screen, after: &Screen, masked: &[[i32; 4]]) -> Option<[i32; 4]> {
    let [right, bottom] = [after.width(), after.height()].map(|v| i32::try_from(v).unwrap_or(0));
    if (before.width(), before.height()) != (after.width(), after.height()) {
        return Some([0, 0, right, bottom]);
    }
    let hidden = |x: i32, y: i32| {
        masked
            .iter()
            .any(|&[x0, y0, x1, y1]| (x0..x1).contains(&x) && (y0..y1).contains(&y))
    };
    let mut found: Option<[i32; 4]> = None;
    for y in 0..bottom {
        for x in 0..right {
            let (x64, y64) = (i64::from(x), i64::from(y));
            if before.pixel(x64, y64) != after.pixel(x64, y64) && !hidden(x, y) {
                let [x0, y0, x1, y1] = found.unwrap_or([x, y, x + 1, y + 1]);
                found = Some([x0.min(x), y0.min(y), x1.max(x + 1), y1.max(y + 1)]);
            }
        }
    }
    found
}

#[cfg(test)]
pub(crate) mod tests;
