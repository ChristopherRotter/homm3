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
//!
//! The turn's end and the next day (E24 M4 T1a, design §5.1, *provisional*): End Turn's command
//! asks Move Reminder's question while a hero can still move (a message box, `h3_ui::message`,
//! laid out by the adapter: [`Start::question`]); the turn's change is the next of
//! [`Start::turns`] (a recording's `[[turn]]`, from Part B the engine's events), drawn in that
//! millisecond as `player_turn_start` 0x4CC7D0 draws the next day, with the week's or the month's
//! message over it at a week's start. While a box is open the map stands still (the tick runs, the
//! week's box steps the date's window behind it); its buttons take the mouse and its keys.
//!
//! A computer player's turn in the viewer's sight (E24 M4 T1b2, design §5.2, *provisional*): a turn
//! whose status window is another player's (kind 5) is a computer's, played from its facts
//! ([`NextTurn::walks`]: its heroes' steps as a recording shows them; Part B the engine's). Its
//! start disables four buttons and shows the shield in the minimap's place, the lists staying the
//! viewer's; each hero is selected (the view centred on it when the viewer sees it) and walks its
//! steps, a step the viewer sees drawn as a human's at Computer Walk Speed and one it does not see
//! taking no time; kind 5 steps its sand and glass. Its end starts the next turn. The single draws
//! of a step at speed 4 take no time either: the loop keeps them ([`Loop::take_held`]) for the
//! window, which shows each on its own clock.
//!
//! The finds told (E24 M4 T2a2, design §5.3, *provisional*): a walk's end on the entrance of a
//! find ([`Start::finds`], laid out by the adapter from `crate::finds`) opens its box in that
//! millisecond, the input queue emptied; told again, the find's later box. The scenario's test at
//! the viewer's turn's end (`scenario_condition_met` 0x4F2CE0): heroes but no town, the days
//! without a town become the tables' count with its warning ([`Start::town`]), the turn's change
//! going on at the box's close; counted down at each turn's end, they are told at each later
//! human day's start. Part B: a key's step onto a pile or an artifact visits it from beside
//! ([`Found::taken`]): a pile's kind 6 asked for the tables' time, an artifact's box; then the
//! object taken off the map, the view's middle dissolved into the view without it in eight
//! steps ([`Draw::dissolve`]), the map stopped meanwhile; an event's boxes one after another, the
//! resources shown as their box opens.
//!
//! The questions (E24 M4 T2b2 part A1, design §5.4, *provisional*): a find's [`Question`] (laid
//! out by the adapter from `crate::asks`) opens after its first boxes, on its entrance or from
//! beside ([`Found::blocks`]: a guarded pile, an artifact), the status window built again first
//! where its handler does; its Cancel ends the visit, its OK shows its [`Answer`]'s boxes and then
//! what the panels show, or is refused where the answer is not built ([`Unplayable::Answer`]).
//! Since part A2 the Cancel's [`Question::no`] is an answer too (a monster's flight asked next, the
//! monster taken), and an answer may take its object off the map with its sound
//! ([`Answer::taken`]: the dissolve), build the status window again and reveal terrains
//! ([`Draw::revealed`]: the view and the minimap drawn again).
//!
//! A level-up's choice (E24 M4 T2c, design §5.5, *provisional*): the experience a visit gives
//! ([`Gain::experience`]: the Learning Stone's, a chest's, an event's rewards) takes the selected
//! hero to the levels it reaches (the tables' thresholds, from [`Start::experience`]); each
//! level's dialog ([`Start::level_ups`], laid out by the adapter from `crate::levels`: the
//! recording's facts, Part B the engine's) opens with its sound where the visit's handler calls
//! `hero_level_up` 0x4DA720 (after the visit's boxes; a chest's after its dissolve), the next in
//! the millisecond the last closes; its keys 1 and 2 ([`Action::Offer`]) or a click on a picture
//! or its frame choose an offer, its OK (Enter and Escape) closes it; after the last the primary
//! skills raised are shown and the mouse looked at again.
//!
//! A flag raised (E24 M4 T2d1, design §5.6, *provisional*): a find's [`Found::flag`] makes its
//! object the visitor's in memory ([`Owners`]: a mine's and a Lighthouse's after their box, a
//! dwelling's before its hire), drawing nothing; the visit's end draws the minimap
//! (`visit_and_update` 0x4AAAA0) with the object's new colour ([`Minimap::owners`]), the view's
//! next draw its flag ([`Draw::owners`]: the idle step's at the box's close). *Not built:* the
//! mouse-over's owners (the live part's at 0: a hint or a cursor over the object after its visit)
//! and the rest of `visit_and_update`'s redraws (the lists, the status window, the resource bar:
//! nothing they show changes in the recordings). On a record (E24 M4 T4b2) the flag is the
//! record's (`ObjectFlagged`, in its colour).
//!
//! A boat boarded and left (E24 M4 T2d2, design §5.7, *provisional*): a human's step onto an
//! empty boat ([`Boats::empty`]) dissolves the hero off the land before its animation
//! (`hero_loss_effect` 0x4ACBB0: the view drawn without it, [`Draw::hidden`]), the step then
//! drawn with nothing lifted; at its end the boat is taken off the map and lifted as the hero's
//! picture ([`Boats::sailed`]: its group by the boat's facing, mirrored by the hero's, MCL-Q35). A
//! step from a boat onto land ([`Start::water`]) puts the boat back on the water first, steps with
//! nothing lifted and at its end dissolves the hero in on the land. Both visits end with
//! `visit_and_update`'s redraws (the minimap, the lists, the status window, the buttons). On a
//! record (E24 M4 T4b2, design §5.15) the record's boarding or landing ([`Move::sea`]) decides in
//! place of the boats and the water; the dissolve's view draw shows the status window it chose at
//! once (the date's step). *Not built:* the Admiral's Hat's points at sea, the mouse-over's boats
//! (the live part's at 0), a computer's boarding or landing, a find on the tile landed on.
//!
//! A spell cast on the map (E24 M4 T2d3, design §5.8, *provisional*): the cast button's command
//! lifts and centres the selected hero, shows the view and opens the spell book ([`State::book`];
//! its screens are M5's), during which only the tick runs; a click closes it and casts the next of
//! [`Start::casts`] (a recording's facts, from Part B the engine's). Summon Boat: the boat moved
//! taken off its tile, its box dissolved when it lay in the view; the boat put beside the hero
//! with the spell's sound, its box dissolved in from the back buffer's picture
//! ([`Dissolve::from`]); at the end the cost spent (the hero list's slot drawn, MCL-Q14 in both
//! profiles), the status window built again and the map's redraw due since the book opened. *Not
//! built:* the book's own screens and inputs, the other spells, Summon Boat's refusals and its
//! chance, the wait for the spell's sound with sound on (the original's ends the cast after it),
//! the cast in the window (live: the button does nothing).

use alloc::boxed::Box;
use alloc::collections::{BTreeMap, VecDeque};
use alloc::format;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;
use core::fmt;

use h3_map::model::Pos;
use h3_render::Bugs;
use h3_render::screen::Screen;
use h3_ui::message::{self as boxes, Key, MessageBox, Open, Outcome};
use h3_ui::ui::Item;

use crate::asks::Fade;
use crate::finds::{FindArt, Gain, pickup};
use crate::hover::{self, Hint, Hover, PanelItems, Selection};
use crate::layout::{AdventureLayout, DIRECTIONS};
use crate::levels::{self, LevelArt};
use crate::lists::{SLOTS, top_showing};
use crate::live::{
    Arrow, Attacking, Boat, Buttons, Cast, Chosen, ChosenHero, ChosenTown, Date, DateStatus,
    Facing, FoundStatus, HeroStatus, Kingdom, KingdomStatus, Live, LiveBoat, Selected, Stack,
    Status, Summoned, Turn, TurnStatus,
};
use crate::minimap::{MinimapArt, divided};
use crate::panels;
use crate::reach::{Hosted, Reach, Stand, WalkAsked};
use crate::scene::ViewAt;
use crate::sequence::{DEFAULT_COMPUTER_SPEED, DEFAULT_HERO_SPEED, DEFAULT_SCROLL_SPEED};
use crate::sound::{Cue, Sound};
use crate::status::{DateArt, OtherArt};
use crate::tables::{CastArt, Tables, WalkArt};
use crate::turn::{self, TurnArt};

/// The mouse manager's type of the adventure cursor (`cursor_defs` 0x67FF38: 1, `cradvntr.def`),
/// the timeline's `cursor` lines' first number.
pub const CURSOR_TYPE: u32 = 1;

/// The mouse manager's type of the default cursor (0, `crdeflt.def`): a message box's (E24 M4
/// T1a, `mouse_set_cursor(0, 0)` at 0x4F74AC).
pub const DEFAULT_CURSOR: u32 = 0;

/// The mouse manager's type of the combat cursor (2, `crcombat.def`; E24 M4 T5b2, design §5.21):
/// `battle_start` 0x462A20 sets it at the black before the battle's fade in.
pub const BATTLE_CURSOR: u32 = 2;

/// A fade's step, ms (`screen_fade_out` 0x6030E0: each step 50 ms of the clock after the last
/// began; the fade in, `screen_fade_in` 0x6032E0 not read, observed so: E27 design §32.2).
pub const FADE_MS: u32 = 50;

/// A [`Switch`]'s shift that shows black (E24 M4 T5b2).
pub const FADE_BLACK: u8 = u8::MAX;

/// The palettes' next cycle after the map comes back from a battle, ms (E24 M4 T5b2,
/// *provisional*: as recorded, `battle-won` at 730, `battle-mine` at 1730; the harness writes the
/// pending step back across the battle, design §5.19).
const BACK_CYCLE_MS: u32 = 20;

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
    /// A direction key's repeat, the system's while the key is held (E24 M3 T7b1, MCL-Q25): a
    /// step as [`Action::Step`], but dropped while a key's step waits, so that the hero stops at
    /// most a step after the key's release; with the original's bugs (the comparisons) queued as
    /// a step, as the original's ring keeps every WM_KEYDOWN (design §4.15). A recording has none
    /// (the harness turns the display's repeat off).
    Repeat(Direction),
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
    /// The right button pressed (E24 M3 T7b1, the review's F4-5): a message in the input queue
    /// (kind 0x20) that ends a path's walk (0x407FB1); nothing else on the map in M3 (the
    /// original's right click shows a help text while held: M4's).
    RightPress,
    /// The right button released (kind 0x40): a message in the input queue; a button the mouse
    /// holds down takes it as the left one's release (`def_button_message` 0x456338: its command
    /// when it shows pressed, 0x456451).
    RightRelease,
    /// A message box's hotkey pressed (E24 M4 T1a: Enter the OK's, Escape the Cancel's,
    /// `message_box_build` 0x4F6990): its button pressed; on the map with no box open nothing.
    BoxKey(Key),
    /// Its key released: the button it pressed clicks.
    BoxKeyUp(Key),
    /// The key 1 or 2 (scan codes 2 and 3; E24 M4 T2c, design §5.5): a level-up's dialog's left
    /// or right offer chosen (`level_up_dialog_proc` 0x4F9780); elsewhere nothing
    /// (*provisional*: the map's own keys, `adventure_key_down` 0x408C40, not read for them).
    /// Its release is [`Action::Up`].
    Offer(usize),
    /// The kingdom's hero `i` chosen as its hero list's slot's click chooses it (E24 M4 T4c, the
    /// record player's, `crate::director`: its slot need not show in the list); the selected one
    /// nothing.
    Choose(usize),
    /// The selected hero's path to the tile clicked, then clicked again (E24 M4 T4c, the record
    /// player's: the tile need not show on the view): the path among the ones the map knows that
    /// leads there shown, then walked; none, nothing.
    Walk(Pos),
    /// The battle's result window closed (E24 M4 T5b2, design §5.21: the battle client's element
    /// run to its OK in the window; since E24 M4 T6, R5-3, a record player's input, given where
    /// the switch shows the battle whole, `crate::director`): the battle's switch back to the map
    /// goes on; elsewhere nothing.
    BattleClosed,
}

impl Action {
    /// The mouse's point it gives (a move's, a click's, a press's, a release's).
    #[must_use]
    pub const fn point(self) -> Option<[i32; 2]> {
        match self {
            Self::Pointer(at) | Self::Click(at) | Self::Press(at) | Self::Release(at) => Some(at),
            _ => None,
        }
    }

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
                | Self::Repeat(_)
                | Self::Scroll(_)
                | Self::Click(_)
                | Self::Press(_)
                | Self::RightPress
                | Self::Key(_)
                | Self::BoxKey(_)
                | Self::Offer(_)
                | Self::Visit
        )
    }
}

/// The input manager's ring holds 64 messages, of which 63 wait: the 64th written drops the
/// oldest (`input_key_message` 0x4EC1D9, `input_mouse_message` 0x4EC3F1; design §3.4, §4.15).
pub const QUEUED: usize = 63;

/// An action given during a walk, waiting in the ring ([`keep_ring`]); a key's repeat dropped while
/// a key's step waits, the client's fix (MCL-Q25), else a step as the original's every WM_KEYDOWN.
fn wait_in(moving: &mut Moving, action: Action, bugs: Bugs) {
    let action = match action {
        Action::Repeat(_) if moving.key && bugs == Bugs::Fixed => return,
        Action::Repeat(d) if bugs == Bugs::Original => Action::Step(d),
        action => action,
    };
    moving.queued.push(action);
    keep_ring(&mut moving.queued);
}

/// The actions waiting kept to the ring's [`QUEUED`] messages, the oldest dropped (a click's down
/// before its up).
fn keep_ring(queued: &mut Vec<Action>) {
    let mut n: usize = queued.iter().map(|a| a.messages()).sum();
    while n > QUEUED {
        // More than the ring holds: never empty.
        if let Some(Action::Click(at)) = queued.first().copied() {
            queued[0] = Action::Release(at);
        } else {
            queued.remove(0);
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
    /// A question's OK whose answer is not built (E24 M4 T2b2: a battle, a level-up, the recruit
    /// dialog; the engine's, M4 T5 and M5), by its find's tile.
    Answer([i32; 3]),
    /// A level the selected hero's experience reaches whose dialog the facts lack (E24 M4 T2c: a
    /// recording's `[[level_up]]` tables; the engine's from Part B), or since E24 M4 T4b1 one
    /// closed with an offer its record's answer did not take, by the level.
    LevelUp(u8),
    /// A spell cast through the book whose facts are not given (E24 M4 T2d3: a recording's
    /// `[[cast]]` table at the book's close, its cost spent at the cast's end; the engine's from
    /// Part B), by the millisecond.
    Cast(u32),
    /// A spell's cast on the map that is not built (E24 M4 T2d3: Summon Boat's alone, with one
    /// boat moved or made), by the spell.
    Spell(u8),
    /// A walk's step the engine's record does not take (E24 M4 T4a2: a tile off the move's, or
    /// a walk past the record's last), by the tile stepped onto.
    Moved([i32; 3]),
    /// A record's landing whose boat's facing the record gives past 7 (E24 M4 T6, the reviewer's
    /// R3-6: [`Sea::Unfaced`]), by the tile stepped onto.
    Landing([i32; 3]),
    /// A record played without a recording (E24 M4 T4c, `crate::director`): a walk to the tile
    /// whose hero the kingdom's list does not hold, or (the tile `[-1, -1, -1]`) the record not
    /// at its end by the limit.
    Record([i32; 3]),
    /// A record played without a recording at its end with this many of its walks not taken
    /// (E24 M4 T4c).
    Untaken(usize),
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
            Self::Answer([x, y, z]) => {
                write!(
                    f,
                    "the answer of the question at ({x}, {y}, {z}) (the engine's)"
                )
            }
            Self::LevelUp(level) => {
                write!(f, "the level-up to level {level}: its facts not given")
            }
            Self::Cast(t) => write!(f, "the cast at {t} ms: its facts not given"),
            Self::Spell(spell) => write!(f, "spell {spell}'s cast on the map (not built)"),
            Self::Moved([x, y, z]) => {
                write!(
                    f,
                    "a step onto ({x}, {y}, {z}) the record's walks do not take"
                )
            }
            Self::Landing([x, y, z]) => {
                write!(
                    f,
                    "a landing onto ({x}, {y}, {z}) whose boat's facing the record gives past 7"
                )
            }
            Self::Record([-1, -1, -1]) => write!(f, "the record not at its end by the limit"),
            Self::Untaken(n) => write!(f, "the record at its end with {n} walks not taken"),
            Self::Record([x, y, z]) => {
                write!(f, "a record's walk to ({x}, {y}, {z}) of a hero not listed")
            }
        }
    }
}

/// The numbers the loop runs by: the tables' (`[map]`, its heroes' standing groups, `[map.walk]`),
/// the layout's (the frame, the view's rectangle, origin and tile, `[scroll]`, `[cursor]`) and
/// whether the original's bugs are reproduced (the edge zones of MCL-Q6, the cursor's even x: E17
/// design §9.9; the mouse's tile 8 pixels off, MCL-Q18; the hint a redraw hides, MCL-Q22; a
/// hotkey's key up a hold drops, MCL-Q24 and MCL-Q28; for the comparisons only).
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
    /// The turn's numbers and sounds (E24 M4 T1a, design §5.1).
    pub turn: TurnArt,
    /// Another player's turn's status window's step (E24 M4 T1b2, design §5.2).
    pub other: OtherArt,
    /// The default cursor's frame a computer player's turn sets (E24 M4 T1b2).
    pub busy: u32,
    /// The finds' numbers (E24 M4 T2a2 part B): kind 6's time, the dissolve's rectangle and step,
    /// the pickup's sounds.
    pub finds: FindArt,
    /// A level-up's numbers (E24 M4 T2c): the experience each level starts at, the dialog's
    /// sound.
    pub level_up: LevelArt,
    /// A cast's numbers (E24 M4 T2d3): Summon Boat's dissolves and sound.
    pub cast: CastArt,
    /// The default cursor's frame the spell book leaves (E24 M4 T2d3: 1, over its place).
    pub book: u32,
    /// The combat cursor's frame a battle's switch sets (E24 M4 T5b2, design §5.21: 6).
    pub battle_cursor: u32,
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
            turn: tables.map.turn.clone(),
            other: tables.map.status.other,
            busy: layout.cursor.busy,
            finds: tables.map.finds.clone(),
            level_up: tables.map.level_up.clone(),
            cast: tables.map.cast.clone(),
            book: layout.cursor.book,
            battle_cursor: layout.cursor.battle_frame,
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
        if !self.on_frame([x, y]) {
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

    /// Whether a point lies on the frame: the window's points always do but the one it gives at
    /// the focus's loss, which stops the edge scroll ([`Rules::edge_at`]: none off the frame).
    #[must_use]
    pub fn on_frame(&self, [x, y]: [i32; 2]) -> bool {
        let [width, height] = self.frame;
        (0..width).contains(&x) && (0..height).contains(&y)
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

/// The boats on the map (E24 M4 T2d2, design §5.7, *provisional*: the live part's at 0, from Part
/// B the engine's): the empty ones on their tiles and the one each of the map's heroes sails.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Boats {
    /// The boats without a hero, in order: one boarded taken off its tile (`hero_hide`), one left
    /// put back last (`hero_show`).
    pub empty: Vec<LiveBoat>,
    /// The boat each of the map's heroes sails ([`Start::heroes`]' order), none on land: a boarded
    /// one's type and its captain's colour.
    pub sailed: Vec<Option<Boat>>,
}

impl Boats {
    /// A live part's boats: its empty ones and its heroes' (E24 M4 T2d2).
    #[must_use]
    pub fn of(live: &Live) -> Self {
        Self {
            empty: live.boats.clone(),
            sailed: live.heroes.iter().map(|h| h.boat).collect(),
        }
    }
}

/// The destination's arrow of a path the hero pays today (`path_arrows` frame 0, design §2.10).
pub const DESTINATION: u16 = 1;

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
    /// The cursor's frames over the view after the selected hero walked, each with its
    /// millisecond (E24 M4 T2b2 part B, *provisional*): the reach there is the engine's pathfinder's
    /// (the mouse-over's facts are the hero's where it stood at 0), so a recording gives them (its
    /// `cursor` lines of the adventure cursor, `Sequence::reached`: `choose-arena`'s at 1460,
    /// `choose-chest`'s at 2380), as [`Start::points`] the points. Empty: the arrow (M3's).
    pub reached: Vec<(u32, u32)>,
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
    /// The current player's colour (E24 M4 T1a: the viewer's at 0; the timeline's `player`
    /// lines).
    pub player: u8,
    /// Move Reminder's setting (E24 M4 T1a, design §5.1): End Turn asks its question while a hero
    /// can still move.
    pub move_reminder: bool,
    /// End Turn's question laid out (GENRLTXT 56, style 2; the adapter's, with the fonts); none:
    /// End Turn with a hero to go to is not built (refused in a recording).
    pub question: Option<Arc<MessageBox>>,
    /// The turns' starts End Turn plays, in order (E24 M4 T1a: a recording's `[[turn]]`; Part B
    /// the engine's events); past the last End Turn's change is not built.
    pub turns: Vec<NextTurn>,
    /// A computer player's turn's options (E24 M4 T1b2).
    pub computer: Computer,
    /// The finds a walk's end tells (E24 M4 T2a2, design §5.3; the adapter's, laid out), by
    /// their entrance tiles.
    pub finds: Vec<Found>,
    /// The viewer's boxes of a turn without a town (E24 M4 T2a2); none: the scenario's test is
    /// not played.
    pub town: Option<TownBoxes>,
    /// The selected hero's experience at 0 (E24 M4 T2c, design §5.5, *provisional*: a
    /// situation's, which the map gives it): the levels a visit's experience reaches from it.
    /// None: not known (a new game's draw; since E24 M4 T6, R2-3, a record's, whose finds carry
    /// their own level-ups, [`Found::levels`]), and no level is reached.
    pub experience: Option<u32>,
    /// The selected hero's level-ups' dialogs, by their level (E24 M4 T2c: the adapter's, laid
    /// out from a recording's facts; none for a record's, R2-3).
    pub level_ups: Vec<LevelBox>,
    /// The boats at 0 (E24 M4 T2d2, design §5.7: the live part's, [`Boats::of`]).
    pub boats: Boats,
    /// The map's water (E24 M4 T2d2: a hero in a boat lands on a tile that is not), each tile's by
    /// level, row and column (`(z × size + y) × size + x`, the map art's order); a tile past its
    /// end is water, so that none known lands nowhere.
    pub water: Arc<Vec<bool>>,
    /// The spells cast through the spell book, in order (E24 M4 T2d3, design §5.8,
    /// *provisional*: a recording's `[[cast]]` tables; the engine's from Part B): the book's
    /// clicks cast them one after another; past the last the cast button's command is not built.
    pub casts: Vec<Cast>,
    /// The engine's walks of the viewer's heroes, in order (E24 M4 T4a2, design §5.13,
    /// *provisional*: a whole game's record's, `crate::engine::played`): each walk the selected
    /// hero starts takes the next one's facts, its steps checked against the move's tiles
    /// ([`Unplayable::Moved`]), its points ([`Start::points`] not read) and the tiles each step
    /// reveals. None: a recording's walks (the points from [`Start::points`], nothing revealed).
    pub moves: Option<Vec<Move>>,
    /// The hot seat's host (E24 M5 T4a, design §5.26, *provisional*): live, a walk past the
    /// engine's known ones and the selected hero's reach at a stand not given are asked of it
    /// ([`Loop::walk_asked`], [`Loop::reach_asked`]), and what it gave so far. None: neither is
    /// asked (a recording's, a record's walks).
    pub host: Option<Hosted>,
}

/// One of the engine's walks (E24 M4 T4a2, design §5.13, *provisional*): a `MoveHero`'s tiles
/// in order, without the hero's own, and per step the points memory holds after it (none: as
/// they were) and the tiles it reveals at its start, before its animation (`hero_move_step`
/// 0x4805E0: `reveal_disk` at 0x48106B; design §5.12). A record tells neither per step
/// (MCL-Q39): `crate::engine::played` gives the walk's points at its last step and its whole
/// reveal at its first (MCL-Q39 (c)).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Move {
    /// The path's tiles, in order: those stepped onto (up to [`Move::stops`]).
    pub tiles: Vec<Pos>,
    /// The points after each step (none: unchanged).
    pub points: Vec<Option<i32>>,
    /// The tiles each step reveals at its start.
    pub revealed: Vec<Vec<Pos>>,
    /// What its last step does with a boat (E24 M4 T4b2, design §5.15): the record's boarding or
    /// landing; none, nothing (on a record the boats' rules of Part A are not asked).
    pub sea: Option<Sea>,
    /// The battle the walk began (E24 M4 T5b1, design §5.20, *provisional*): the record's number
    /// of the `GameEvents` whose `BattleStarted` began it; at the walk's end the map hands the
    /// battle over ([`Loop::battles`]). None: no battle.
    pub battle: Option<u32>,
    /// The empty boats the viewer is told of since the walk before (E24 M4 T6, the reviewer's
    /// R3-3, *provisional*: a record's `GameReveal`'s boats, its walk's own, and a `BoatBuilt`'s),
    /// put on the map at its first step with its reveal, before a boarding there; one on a tile
    /// with an empty boat replaces it.
    pub boats: Vec<LiveBoat>,
    /// The steps the record's walk takes when fewer than its tiles (E24 M4 T6, *provisional*: the
    /// record's `HeroStepped`): all but the last, a visit from beside (a find's, else one the map
    /// does not show: [`Loop::pass_by`]); fewer, a walk stopped on the way (an event, a guard's
    /// battle) after that many, the rest of its path left shown. None: every tile stepped onto.
    pub stops: Option<usize>,
}

/// A record's walk's last step with a boat (E24 M4 T4b2, design §5.15, *provisional*): the
/// record's `Boarded` or `Landed`, in place of Part A's empty boats and water ([`Boats`],
/// [`Start::water`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sea {
    /// The empty boat of this type on the step's tile boarded (`Boarded`).
    Board(u8),
    /// The hero's boat left on its water tile, of its type and facing the record's way
    /// (`Landed`).
    Land(LiveBoat),
    /// A `Landed` whose facing the record gives past 7 (E24 M4 T6, the reviewer's R3-6): no boat
    /// the map can leave, so the step is refused where it is played ([`Unplayable::Landing`]), as
    /// a boarding with no boat is.
    Unfaced(u8),
}

/// The tiles the viewer's walks explored since the sequence's 0 (E24 M4 T4a2, design §5.13: a
/// record's [`Move::revealed`]), in the order revealed: the live part at 0's explored tiles with
/// these (each level's index built again).
pub type Walked = Arc<Vec<Pos>>;

/// A level-up's dialog (E24 M4 T2c, design §5.5, *provisional*: laid out by the adapter,
/// `crate::levels::dialog`, from the facts `crate::levels::LevelUp`): the level it shows, the
/// dialog, its offers and the primary skill it raises.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LevelBox {
    /// The new level.
    pub level: u8,
    /// The dialog.
    pub dialog: Arc<MessageBox>,
    /// Its offers, the left first (none to two).
    pub offers: Vec<levels::Offer>,
    /// The primary skill raised (0 Attack to 3 Knowledge).
    pub primary: u8,
    /// The offer a record's answer took (E24 M4 T4b1, design §5.14): the OK closing it with
    /// another is refused ([`Unplayable::LevelUp`]); none: any (a recording's).
    pub chosen: Option<usize>,
}

/// A find's boxes (E24 M4 T2a2, design §5.3, *provisional*: the viewer's facts at 0,
/// `crate::finds::Find`): its entrance tile, the first visit's boxes and every later one's (the
/// find as the visit leaves it, [`crate::finds::Find::visited`]), what the panels show, and for
/// part B a pile's kind 6, the object taken and the scenario's test after an event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Found {
    /// The entrance tile a walk ends on, or a key's step visits from beside ([`Found::taken`]).
    pub at: Pos,
    /// The first visit's boxes, one after another, each with what the panels show as it opens
    /// (an event's resources); none: a pile's.
    pub first: Vec<(Arc<MessageBox>, Gain)>,
    /// A later visit's; none: nothing told again (an object taken, an event removed).
    pub again: Option<Arc<MessageBox>>,
    /// What the first visit gives that the panels show, at its last box's close (a pile's at
    /// once).
    pub gain: Gain,
    /// Kind 6 the visit asks (a pile's, `status_resource_set` 0x416060).
    pub found: Option<FoundStatus>,
    /// The object the visit takes off the map (its index in the map's order): a pile's, an
    /// artifact's, which block their tile; a key's step onto it leaves the hero beside it.
    pub taken: Option<usize>,
    /// An event (`event_fire` 0x4A0ED0, fired inside the step onto its tile): its boxes before
    /// the step's end draws the hero's slot (recorded, `find-event`: the movement bar's points at
    /// 2260), the scenario's test after them (*provisional*: its place not read).
    pub event: bool,
    /// The object whose entrance blocks its tile (E24 M4 T2b2: its index in the map's order): a
    /// key's step onto it visits it from beside ([`Loop::visit_beside`]) until it is taken; a
    /// pile's and an artifact's, [`Found::taken`] too when the visit takes it.
    pub blocks: Option<usize>,
    /// The question the visit asks after its first boxes (E24 M4 T2b2 part A1, design §5.4): asked
    /// at every visit until its OK is taken; after that the find tells nothing (*provisional*: the
    /// find as its answer leaves it is the engine's facts).
    pub ask: Option<Question>,
    /// The flag the visit raises (E24 M4 T2d1, design §5.6): the object (its index in the map's
    /// order) and the visitor's colour, its owner after its first boxes and before its question
    /// (a mine's and a Lighthouse's after their box, `set_mine_owner` 0x4C66E0; a dwelling's
    /// before its hire, `set_dwelling_owner` 0x4C67B0). Nothing is drawn by it: the visit's end
    /// draws the minimap with it, the view's next draw its flag.
    pub flag: Option<(usize, u8)>,
    /// The battle the visit begins at once (E24 M4 T5b2, design §5.21: a savage monster's from
    /// beside): the attack drawn, the battle's switch, then its result on the map.
    pub fight: Option<Fight>,
    /// A record's visit (E24 M4 T6, the reviewer's R2-1): the record's walk (its place among
    /// [`Start::moves`]) whose end visits it, told at that visit alone (each of a record's visits
    /// to a tile is a find of its own, with its own boxes, gain, flag and level-ups); none: a
    /// recording's, told at every visit to its tile (its first boxes, then [`Found::again`]).
    pub walk: Option<usize>,
    /// A record's visit's level-ups (E24 M4 T6, the reviewer's R2-3: the record's own, any
    /// hero's), in order: due once the visit's experience is shown ([`Loop::gain`]), opened where
    /// the visit's handler calls `hero_level_up` as [`Start::level_ups`]' are; none for a
    /// recording's (the tables' thresholds).
    pub levels: Vec<LevelBox>,
}

impl Found {
    /// The battles its visit may begin (E24 M4 T5b2): its own fight's and its question's
    /// answers', by their record's numbers.
    #[must_use]
    pub fn battles(&self) -> Vec<u32> {
        let answers = self
            .ask
            .iter()
            .flat_map(|q| [&q.yes, &q.second, &q.no])
            .flatten()
            .filter_map(|a| a.fight.as_ref());
        self.fight.iter().chain(answers).map(|f| f.battle).collect()
    }
}

/// A battle a visit begins (E24 M4 T5b2, design §5.21, *provisional*: a record's): a monster's
/// attack drawn, the battle's winner and what the map shows after it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fight {
    /// The monster's picture's frame in the attack DEF (as [`Question::attacker`]: twice its
    /// creature; plus 1 with the hero left of it, added at the visit).
    pub attacker: u32,
    /// The battle: the record's number of the `GameEvents` whose `BattleStarted` began it.
    pub battle: u32,
    /// The winner: 0 the attacker, 1 the defender (an attacker's retreat too).
    pub winner: i32,
    /// What follows the map's fade in.
    pub after: AfterBattle,
    /// The experience the hero gains (the record's), which nothing shows (its levels).
    pub experience: i32,
}

/// What the map shows after a battle's switch (E24 M4 T5b2, design §5.19's rest).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AfterBattle {
    /// The monster beaten and taken off the map (`monster_fight` 0x4A6C60's removal:
    /// `killfade.82M`, the view's middle dissolved).
    Taken,
    /// The hero beaten or retreated: it leaves the map (`adventure_battle`'s `hero_loss_effect`
    /// with 0, after `battle_aftermath` 0x4AC930's way: the hero gone first, then dissolved), then
    /// the player's next hero or town selected (`mobilize_hero(0, 0, 1)`). A draw's hero the same
    /// (E24 M5 T1b, design §5.23: the draw's `battle_aftermath` with 0, 0x4AE868, runs the
    /// beaten one's steps, 0x4AE6B4 to 0x4AE6FD, one for one).
    Lost,
    /// A draw (E24 M4 T6, the reviewer's R5-4: the record takes both off the map): the hero
    /// leaves as [`AfterBattle::Lost`]'s (`battle_aftermath`'s way, the hero's leaving first:
    /// read, E24 M5 T1b), then the monster is taken off as [`AfterBattle::Taken`]'s
    /// (`monster_fight`'s rest: none left), *provisional*: not recorded.
    Both,
    /// Nothing more (the monster stands: a defender beaten elsewhere, not built; since E24 M4 T6,
    /// R5-4, a mine's guards' battle lost by a hero the record keeps).
    Stays,
    /// The answer's boxes that began it, one after another (`visit_mine` 0x4A39A0: ADVEVENT 85,
    /// then MINEEVNT's), and its flag.
    Boxes,
}

impl AfterBattle {
    /// What a wandering monster's battle leaves by what the record's end of it took off the map
    /// (E24 M4 T5b2; since E24 M4 T6, the reviewer's R5-4, a draw's both): the monster `taken`,
    /// the hero `lost`.
    #[must_use]
    pub const fn of_fight(taken: bool, lost: bool) -> Self {
        match (taken, lost) {
            (true, true) => Self::Both,
            (true, false) => Self::Taken,
            (false, true) => Self::Lost,
            (false, false) => Self::Stays,
        }
    }

    /// What a mine's guards' battle leaves (E24 M4 T6, the reviewers' R5-4 and R3-2): the
    /// answer's boxes after the hero's win (`winner` 0); else the hero gone where the record
    /// takes it off the map (`lost`: beaten, retreated or a draw), or nothing. No box and no flag
    /// either way (E24 M5 T1b, MCL-Q51, design §5.23: `visit_mine` 0x4A39A0 ends the visit at a
    /// winner not the hero, `monster_battle`'s at 0x4A3AE5, `adventure_battle`'s at 0x4A3B59;
    /// recorded, `battle-mine-retreat`).
    #[must_use]
    pub const fn of_guards(winner: i32, lost: bool) -> Self {
        match (winner, lost) {
            (0, _) => Self::Boxes,
            (_, true) => Self::Lost,
            (_, false) => Self::Stays,
        }
    }
}

/// A visit's question (E24 M4 T2b2 part A1, design §5.4, *provisional*: the viewer's facts at 0,
/// `crate::asks::Ask`): its box of style 2, whether the status window is built again before it
/// (`status_window_choose` 0x415DE0 with 0, 1, 1) and what its OK and its Cancel show (none: not
/// built, refused in a recording; A1's every Cancel the default answer, the visit ended).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Question {
    /// The question's box.
    pub asked: Arc<MessageBox>,
    /// The status window chosen again before it.
    pub chooses: bool,
    /// What its OK shows.
    pub yes: Option<Answer>,
    /// What its Cancel shows (E24 M4 T2b2 part A2).
    pub no: Option<Answer>,
    /// A monster's question (E24 M4 T2b2 part A2): twice its creature, its picture's frame in the
    /// attack DEF from the visit's start ([`crate::live::Attacking`]; plus 1 with the hero left
    /// of it), none else.
    pub attacker: Option<u32>,
    /// A choice's (E24 M4 T2b2 part B: style 7 or 10, its OK closing with the picture chosen):
    /// what its second picture shows (0x780A); its first's is [`Question::yes`], its Cancel's
    /// [`Question::no`].
    pub second: Option<Answer>,
}

/// What a question's answer shows (E24 M4 T2b2 part A1): its boxes one after another, each with
/// what the panels show as it opens, and what they show at the last one's close (recorded:
/// `ask-tomb`'s morale, `ask-hire`'s army at 1660); since part A2 (*provisional*, `crate::asks::Reply`)
/// then the status window built again (a monster's join: `ask-join`'s army at 1060), the question
/// asked next (`ask-join-no`'s flight at 1060), the object taken off the map (its index in the
/// map's order) with its sound ([`Loop::take`]: `ask-join`, `ask-guard`, `ask-pandora`) or the
/// terrains revealed with the view and the minimap drawn again (`ask-cartographer` at 1060); the
/// default ends the visit.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Answer {
    /// The boxes.
    pub boxes: Vec<(Arc<MessageBox>, Gain)>,
    /// What the panels show at the end.
    pub gain: Gain,
    /// The status window built again after the gain (`status_window_choose` 0x415DE0 with 1, 1, 1).
    pub chooses: bool,
    /// The question asked next.
    pub then: Option<Box<Question>>,
    /// The object taken off the map, and its sound.
    pub taken: Option<(usize, Fade)>,
    /// What the panels show at the taken object's dissolve's end (E24 M4 T2c: a chest's
    /// experience, its levels after the chest's removal).
    pub after: Gain,
    /// The terrains revealed (a bit per terrain, `reveal_terrain_mask` 0x4CA240).
    pub reveal: u32,
    /// A Cancel's answer marks the find answered as an OK's does (E24 M4 T2b2 part B: the
    /// Arena's, MCL-Q33).
    pub marks: bool,
    /// The battle the answer begins (E24 M4 T5b2, design §5.21: a mine's guards'): at its box's
    /// close the battle's switch, the answer's boxes after it.
    pub fight: Option<Fight>,
    /// The flag raised past the answer's last box (E24 M4 T5b2: a guarded mine's, after
    /// MINEEVNT's box: the object and the colour).
    pub flag: Option<(usize, u8)>,
}

/// The viewer's boxes of a turn without a town (E24 M4 T2a2, design §5.3): the warning at its
/// turn's end (GENRLTXT 7, [`crate::finds::town_lost`]) and the days left at each later day's
/// start ([`crate::finds::days_left`]), `left[d - 1]` the box of `d` days, as many as the tables'
/// `[map.turn]` `town_days`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TownBoxes {
    /// The warning.
    pub lost: Arc<MessageBox>,
    /// The days left's.
    pub left: Vec<Arc<MessageBox>>,
}

/// A computer player's turn's options (E24 M4 T1b2, design §5.2): Computer Walk Speed
/// (`option_computer_walk_speed` 0x698758: a step's frames as the hero speed's of that number,
/// 3 the harness's settings') and Blackout Computer (0x698790: no step seen).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Computer {
    /// Computer Walk Speed (2 to 4 drawn; 5, which the system options set with Blackout, lies
    /// past the tables: one draw as 4's).
    pub speed: usize,
    /// Blackout Computer.
    pub blackout: bool,
}

impl Default for Computer {
    fn default() -> Self {
        Self {
            speed: DEFAULT_COMPUTER_SPEED,
            blackout: false,
        }
    }
}

/// A turn's start End Turn plays (E24 M4 T1a, design §5.1, *provisional*): the live part's facts
/// and, at a week's start, the week's or the month's message laid out (the adapter's: the text,
/// [`turn::message`], in a box of style 1); since E24 M4 T1b2 the map's heroes where the turn's
/// start finds them and, for a computer player's turn in sight, its heroes' walks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NextTurn {
    /// The facts.
    pub turn: Turn,
    /// The week's or the month's message; none on another day.
    pub message: Option<Arc<MessageBox>>,
    /// The map's heroes the facts hold standing ([`Turn::heroes`], [`turn::placed`]), each by
    /// its index among [`Start::heroes`] with its tile and facing: placed there as the turn
    /// starts (a computer's moves out of sight).
    pub heroes: Vec<(usize, Hero)>,
    /// A computer player's turn's heroes' walks, in order (E24 M4 T1b2: [`turn::walks`]); none
    /// for a human's.
    pub walks: Vec<Steps>,
    /// The map's objects (their indexes among the map's objects) taken off as the turn starts
    /// (E24 M4 T4c2: a record's other player's pickups without a step, MCL-Q46 (c); since E24 M4
    /// T6, R1-5, any object a stepless command removed).
    pub taken: Vec<usize>,
    /// The objects flagged as the turn starts, each by its index and its owner's colour (E24 M4
    /// T6, the reviewer's R1-5: a record's other player's stepless command's, *provisional*:
    /// nothing shown).
    pub flags: Vec<(usize, u8)>,
    /// The map's heroes taken off the map as the turn starts, by their indexes (as
    /// [`NextTurn::flags`]).
    pub lost: Vec<usize>,
    /// The empty boats put on the map as the turn starts (E24 M4 T6, the reviewer's R3-3,
    /// *provisional*: a record's boats told since the table before and before any walk of the
    /// viewer's, a boat built; as [`NextTurn::flags`], nothing shown).
    pub boats: Vec<LiveBoat>,
    /// The map's hero of each of the kingdom's heroes, slot by slot, none for one no map hero is
    /// (E24 M4 T6, the reviewer's R1-4: a record's, its list at the turn's start, a hero lost or
    /// hired since); none: the start's choices by their slots (a recording's).
    pub list: Option<Vec<Option<usize>>>,
    /// Played at the loop's start in place of after End Turn (E24 M4 T6, the reviewer's R1-1: a
    /// record whose viewer does not act first starts in another player's turn): the first of
    /// [`Start::turns`] only.
    pub first: bool,
    /// A computer's turn the record ends in (E24 M4 T6, R1-1, *provisional*): played to its last
    /// walk, the map then left in its screen (its end is not in the record).
    pub open: bool,
}

/// A computer player's hero's walk in its turn (E24 M4 T1b2, *provisional*): the hero (its index
/// among [`Start::heroes`]), its place in the computer's kingdom's list, and its steps as a
/// recording shows them (the timeline's `hero` lines in the turn: each step's destination and the
/// way it faces; the steps it does not show, unseen ones, lie between two of them); Part B the
/// engine's moves.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Steps {
    /// The map's hero.
    pub hero: usize,
    /// Its place in the computer's kingdom's hero list.
    pub slot: usize,
    /// Its steps' destinations and facings, in order.
    pub steps: Vec<Hero>,
    /// The steps taken unseen whatever the viewer sees (E24 M4 T4c: a record's step into sight,
    /// whose start it does not give, MCL-Q44), by their places among [`Steps::steps`].
    pub unseen: Vec<usize>,
    /// The steps that board an empty boat or land from one (E24 M4 T4c2: a record's `Boarded`,
    /// `Landed`), each by its place among [`Steps::steps`].
    pub sea: Vec<(usize, Sea)>,
    /// Its moves' ends, in order (E24 M4 T6, the reviewer's R1-3: a record's, each command's
    /// steps a move; since T4c the objects a record's hero picks up from beside at its end); none:
    /// one move to its last step, nothing taken (a recording's).
    pub ends: Vec<End>,
    /// The record's judgement of whether the viewer sees the hero's tile as it is chosen (E24 M4
    /// T6, the reviewer's R1-2: the walk's first step's start told), each step seen but the
    /// [`Steps::unseen`] ones (Blackout apart) and each end by its [`End::seen`]; none: the
    /// viewer's explored tiles decide ([`Loop::sees`], a recording's).
    pub seen: Option<bool>,
}

/// A computer's move's end (E24 M4 T6, the reviewer's R1-3, R1-5, *provisional*: a record's, one
/// command's steps): the hero standing, the view drawn at its last step (`hero_move_step`'s fifth
/// argument 1, design §5.18), then what the command changes on the map.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct End {
    /// The walk's steps up to it.
    pub after: usize,
    /// The objects taken from beside, each by its anchor and its index among the map's objects:
    /// the hero turned to the first, its dissolve with the pickup's sound where the viewer sees
    /// it ([`Loop::ai_take`]).
    pub taken: Vec<(Pos, usize)>,
    /// The objects off the map at once, nothing dissolved (a monster beaten), by their indexes.
    pub removed: Vec<usize>,
    /// The objects flagged, each by its index and its owner's colour ([`Loop::raise`]).
    pub flags: Vec<(usize, u8)>,
    /// The map's heroes off the map, drawn nowhere from then on (a hero lost in sight), by their
    /// indexes.
    pub lost: Vec<usize>,
    /// The empty boats put on the map (E24 M4 T6, the reviewer's R3-3: a boat the command built
    /// in sight, `BoatBuilt`).
    pub boats: Vec<LiveBoat>,
    /// Whether the viewer sees the hero's tile there, by the record's judgement ([`Steps::seen`]:
    /// the move's last step's end told).
    pub seen: bool,
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
    /// The mouse manager's type (E24 M4 T1a): [`CURSOR_TYPE`] the adventure cursor's,
    /// [`DEFAULT_CURSOR`] the default one's (a message box's).
    pub kind: u32,
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
/// were. Since E24 M4 T2d1 the objects' owners changed as they were then (a visit's end draws it,
/// `visit_and_update` 0x4AAAA0: a flag raised shows on it before the view's next draw).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Minimap {
    /// The view's corner.
    pub view: ViewAt,
    /// The selected hero's tile.
    pub hero: Option<Pos>,
    /// The selected hero's index among the map's heroes ([`Start::heroes`]).
    pub index: Option<usize>,
    /// The objects' owners changed since 0 (E24 M4 T2d1).
    pub owners: Owners,
    /// The tiles the walks explored since 0 (E24 M4 T4a2): a step's reveal shows on the minimap
    /// at its next draw, the step's end (`reveal_disk` 0x49D160 draws it at once only in a network
    /// game; design §5.12).
    pub walked: Walked,
}

/// The objects whose owner changed since the sequence's 0 (E24 M4 T2d1, design §5.6: a flag
/// raised), each by its index in the map's order with its owner's colour now, in the order first
/// changed: the live part at 0's owners with these over them (each level's index built again).
pub type Owners = Arc<Vec<(usize, u8)>>;

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
    /// The message box over the map, its buttons' states (E24 M4 T1a); none without one.
    pub message: Option<Arc<MessageBox>>,
    /// A computer player's turn shown (E24 M4 T1b2, design §5.2: four buttons disabled, the
    /// minimap's place the shield; [`crate::live::Live::computer`]).
    pub computer: bool,
    /// The map's heroes on their tiles and facings ([`Start::heroes`]; the selected one's as it
    /// stood when last chosen away from: [`Draw::hero`] gives it): E24 M4 T1b2, a computer's moves.
    pub heroes: Arc<Vec<Hero>>,
    /// The objects taken off the map (E24 M4 T2a2 part B: a pile, an artifact), by their index in
    /// the map's order, in order: the view draws without them (the level's index built again).
    pub gone: Arc<Vec<usize>>,
    /// A dissolve's step shown (E24 M4 T2a2 part B, design §5.3): over this draw, its rectangle
    /// the picture before blended towards this one.
    pub dissolve: Option<Dissolve>,
    /// The terrains a Cartographer revealed to the viewer since 0 (E24 M4 T2b2 part A2: a bit per
    /// terrain, `reveal_terrain_mask` 0x4CA240): every tile of them on both levels explored (each
    /// level's index built again).
    pub revealed: u32,
    /// The monster a hero visits, drawn as its attack (E24 M4 T2b2 part A2).
    pub attacking: Option<Attacking>,
    /// The objects' owners changed since 0 as the view was last drawn (E24 M4 T2d1, design §5.6:
    /// the flags' colours; the minimap's are [`Minimap::owners`]).
    pub owners: Owners,
    /// The selected hero off its tile and not lifted (E24 M4 T2d2, design §5.7: +0x1EC 0 after
    /// `hero_select` took it off its tile, +0x20C; a boarding's dissolve and step, a landing's
    /// step): drawn nowhere.
    pub hidden: bool,
    /// The boats as the view was last drawn (E24 M4 T2d2).
    pub boats: Arc<Boats>,
    /// The tiles the walks explored since 0 as the view was last drawn (E24 M4 T4a2, design
    /// §5.13: a step's reveal shows from its first frame; the minimap's are
    /// [`Minimap::walked`]).
    pub walked: Walked,
    /// A battle's switch shown instead of this draw as it is (E24 M4 T5b2, design §5.21): the
    /// map's fade out and in, the battle's fade in and out.
    pub switch: Option<Switch>,
    /// The heroes gone off the map (E24 M4 T5b2: a battle lost; since E24 M4 T6, R1-5, another
    /// player's hero lost in sight), by their indexes among the map's heroes: drawn nowhere.
    pub left: Vec<usize>,
}

/// What a battle's switch shows (E24 M4 T5b2, design §5.21, *provisional*: the host draws it,
/// the battle's frames the battle client's): the draw it belongs to faded, or the battle's frame
/// faded.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Switch {
    /// The battle's frame instead of the map's (the record's number of the `GameEvents` whose
    /// `BattleStarted` began it: its first frame, or with [`Switch::ended`] its last).
    pub battle: Option<u32>,
    /// The fade's step: each channel shifted down so many bits (0 to 2); [`FADE_BLACK`] black.
    pub shift: u8,
    /// The battle's last frame under its result window, from its result's OK (its fade out):
    /// the harness's battle step keeps no screen there (`clients/capture/README.md`, "A battle in
    /// a sequence"), so a sequence's run draws nothing of it.
    pub ended: bool,
}

/// A dissolve's step (`screen_dissolve` 0x602DC0; E17 design §7.25, E24 design §5.3): the
/// screen as the draw before showed it, its rectangle (left, top, right, bottom) `step` eighths
/// of the way to the draw's (`h3_render::Screen::dissolve_rect`); the rest of the screen the one
/// before's.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dissolve {
    /// What the screen showed before: outside the rectangle, and in it the picture blended from
    /// unless [`Dissolve::from`] gives another. Since E24 M4 T2d3 it may show a dissolve of its
    /// own, an earlier one ended (its step 8: the screen as that one left it, Summon Boat's
    /// first box).
    pub before: Arc<Draw>,
    /// The picture the rectangle blends from when not the screen's (E24 M4 T2d3: the back
    /// buffer's rectangle `screen_save_rect` 0x602CC0 saved, the view a first dissolve drew
    /// without the boat; its rest never shown); none: [`Dissolve::before`]'s.
    pub from: Option<Arc<Draw>>,
    /// The rectangle (left, top, right, bottom, the last two exclusive).
    pub rect: [i64; 4],
    /// The step, 0 (the picture before) to 7; 8 the dissolve ended, its rectangle the draw's.
    pub step: u8,
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
    /// The game's date (E24 M4 T1a): the timeline's `date`.
    pub date: Date,
    /// The current player's colour: the timeline's `player`.
    pub player: u8,
    /// Another player's turn's status window shown (E24 M4 T1b2): its crest's, sand's and glass's
    /// frames, the timeline's `other`; none under another kind.
    pub other: Option<[u32; 3]>,
    /// The spell book open over the map (E24 M4 T2d3, design §5.8): the timeline's `book`; its
    /// screens are M5's.
    pub book: bool,
    /// The battle shown whole at this millisecond (E24 M4 T5b2, design §5.21): its winner, the
    /// timeline's `battle` (the harness's `<ms> battle <winner>`).
    pub battle: Option<i32>,
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
    /// The selected hero off its tile and not lifted (E24 M4 T2d2: [`Draw::hidden`]).
    hidden: bool,
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
    /// The cursor's frames after a walk by millisecond ([`Start::reached`]) and the millisecond
    /// run now.
    reached: Vec<(u32, u32)>,
    now: u32,
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
    /// A redraw of every dialog item hid the status bar's hint this turn: the client's fix draws
    /// the hint of the mouse's item again at the turn's end (MCL-Q22, [`Loop::hint_again`]).
    rehint: bool,
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
    /// T6b2), with the point of the last message its loop took (the press's, then each move's).
    dragging: Option<[i32; 2]>,
    /// A click's divisor on this map's minimap (the f32's bits: [`MinimapArt::click`]).
    scale: u32,
    /// An item of the status window holds the press (its state's bit 1, `sub_44FF60` 0x450030):
    /// the left button's next release, wherever, is its command (0x409FD8); a window built again
    /// meanwhile takes it with its items.
    status_held: bool,
    /// The current player ([`Start::player`]).
    player: u8,
    /// Move Reminder's setting and End Turn's question ([`Start::move_reminder`],
    /// [`Start::question`]).
    move_reminder: bool,
    question: Option<Arc<MessageBox>>,
    /// The turns' starts ([`Start::turns`]) and the next one's place among them.
    turns: Arc<Vec<NextTurn>>,
    next_turn: usize,
    /// The millisecond End Turn's change found no next turn while live (E24 M5 T3b): the host's
    /// session plays the End Turn and gives the turns after it ([`Loop::add_turns`]).
    turns_asked: Option<u32>,
    /// The message box open over the map (E24 M4 T1a), and the boxes waiting to open as it closes
    /// (E24 M4 T2a2: the days left's, then the week's).
    boxed: Option<Boxed>,
    waiting: VecDeque<(Arc<MessageBox>, Told)>,
    /// The finds ([`Start::finds`]) and those told once, by their place among them; those whose
    /// question's OK was taken (E24 M4 T2b2).
    finds: Arc<Vec<Found>>,
    told: Vec<usize>,
    answered: Vec<usize>,
    /// The objects' owners changed since 0 in memory (E24 M4 T2d1: the flags raised), which the
    /// view and the minimap take at their draws.
    owners: Owners,
    /// The viewer's boxes of a turn without a town ([`Start::town`]) and its days without a town
    /// (the player's +0x3D: −1 not counting, *provisional*: a new game's at 0).
    town: Option<Arc<TownBoxes>>,
    days: i32,
    /// Kind 6's window a pile's visit asked (E24 M4 T2a2 part B: the chooser's kind 6 builds it).
    found: Option<FoundStatus>,
    /// An object's dissolve under way (E24 M4 T2a2 part B): the map stopped until its end.
    dissolving: Option<Dissolving>,
    /// A battle's switch under way (E24 M4 T5b2, design §5.21): the map stopped until its end.
    switching: Option<Switching>,
    /// The millisecond the last battle showed whole and its winner (the timeline's `battle`).
    whole: Option<(u32, i32)>,
    /// The battles a switch handed over (their record's numbers): a walk's end lists none of
    /// them again ([`Loop::battles`]).
    switched: Vec<u32>,
    /// The heroes off the map (E24 M4 T5b2: a battle lost; since E24 M4 T6, R1-5, another
    /// player's hero lost in sight), by their indexes among the map's heroes.
    left: Vec<usize>,
    /// The kingdom's slots of the heroes battles took off the map, in order (E24 M4 T6:
    /// [`Loop::slots_left`]).
    slots_left: Vec<usize>,
    /// The input queue emptied (`input_flush` 0x4EC570: a box's opening, a human's turn's
    /// start): the actions after this one in its millisecond are dropped.
    flush: bool,
    /// Computer Walk Speed's frames: the pixels each slides the view by (none: one draw); Blackout
    /// Computer (E24 M4 T1b2, [`Start::computer`]).
    computer_pixels: Vec<i32>,
    blackout: bool,
    /// A computer player's turn under way (E24 M4 T1b2), and its step under way: its direction,
    /// its next frame and when that frame, or its end, comes.
    ai: Option<AiTurn>,
    ai_step: Option<(Direction, usize, u32)>,
    /// The single draws of a computer's step at speed 4 in this millisecond, each as the screen
    /// showed it ([`Loop::take_held`]).
    within: Vec<State>,
    /// The selected hero's experience ([`Start::experience`]) and level (E24 M4 T2c), the map's
    /// hero they are of (the selected one at 0) and its level-ups' dialogs
    /// ([`Start::level_ups`]).
    experience: Option<u32>,
    hero_level: u8,
    experienced: Option<usize>,
    level_ups: Arc<Vec<LevelBox>>,
    /// A record's find whose visit is under way and whose level-ups wait for its experience, and
    /// the level-ups due (E24 M4 T6, the reviewer's R2-3: [`Found::levels`]).
    leveling: Option<usize>,
    due_levels: VecDeque<LevelBox>,
    /// The boats in memory and the map's water ([`Start::boats`], [`Start::water`]), and the boat's
    /// visit the step under way ends with (E24 M4 T2d2).
    boats: Boats,
    water: Arc<Vec<bool>>,
    aboard: Option<Aboard>,
    /// The spells cast through the book ([`Start::casts`]) and the next one's place among them,
    /// and the book open (E24 M4 T2d3).
    casts: Arc<Vec<Cast>>,
    next_cast: usize,
    book: bool,
    /// The engine's walks ([`Start::moves`]), the next one's place among them, the one under way
    /// and its step, the tiles the walks explored and the step off the record found in this
    /// millisecond (E24 M4 T4a2).
    moves: Option<Arc<Vec<Move>>>,
    /// The hot seat's host's facts ([`Start::host`]), the walk asked of it and the stand whose
    /// reach is asked (E24 M5 T4a): each kept until the host answers.
    host: Option<Arc<Hosted>>,
    walk_asked: Option<WalkAsked>,
    reach_asked: Option<(u32, Stand)>,
    next_move: usize,
    walking: Option<(usize, usize)>,
    /// The engine's walk taken last (E24 M4 T6, R2-1: a record's find is its end's,
    /// [`Found::walk`]); none before the first or past the last.
    last_move: Option<usize>,
    walked_tiles: Walked,
    off_record: Option<Unplayable>,
    /// The battles handed over so far (E24 M4 T5b1): each walk's end that began one, its
    /// millisecond and [`Move::battle`]; since E24 M4 T5b2 each switch's where it shows the
    /// battle whole ([`Loop::battles`]).
    battles: Vec<(u32, u32)>,
}

/// A computer player's turn under way (E24 M4 T1b2, design §5.2, *provisional*): its facts, kind
/// 5's window, its heroes' walks left and the step under way, and what the viewer's turn's end
/// left for its next day.
#[derive(Debug, Clone, PartialEq, Eq)]
struct AiTurn {
    /// The computer's kingdom (the turn's facts): its heroes' points, its selection.
    kingdom: Kingdom,
    /// Kind 5's window and the millisecond it last stepped or was built (the panel's +0x40).
    status: TurnStatus,
    stepped: u32,
    /// The heroes' walks left.
    walks: VecDeque<Steps>,
    /// The walking hero's place in the computer's kingdom (the current player's selected hero:
    /// the timeline's `points`) and its steps left, each with whether it is taken unseen and
    /// what it does with a boat (E24 M4 T4c2).
    slot: usize,
    steps: VecDeque<(Hero, bool, Option<Sea>)>,
    /// The steps of the walk under way taken so far, its moves' ends left (E24 M4 T6, R1-3: each
    /// command's move; since T4c the objects it takes at its end) and the record's judgement of
    /// what the viewer sees of it (R1-2, [`Steps::seen`]).
    done: usize,
    ends: VecDeque<End>,
    judged: Option<bool>,
    /// Whether a hero of the turn was chosen (E24 M4 T4c2: the next one's choice puts it back).
    chosen: bool,
    /// The record ends in this turn (E24 M4 T6, R1-1, [`NextTurn::open`]).
    open: bool,
    /// The viewer's turn's end.
    ended: Ended,
}

/// What the viewer's turn's end leaves for its next day (E24 M4 T1a's turn's change): its
/// selection, the selection's window memory kept and the path shown.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct Ended {
    before: Selected,
    window: Option<Status>,
    path: Vec<(Direction, Arrow)>,
}

/// The next day's selection as the turn's facts give it (E24 M4 T1a): the kingdom's hero `i` and
/// its map hero, or the town `i` and what choosing it shows.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Choice {
    hero: Option<(usize, usize)>,
    town: Option<(usize, ChosenTown)>,
}

/// A message box open over the map (E24 M4 T1a, design §5.1): what its OK does, the box and the
/// button it holds, the cursor before it (restored at its end).
#[derive(Debug, Clone, PartialEq, Eq)]
struct Boxed {
    /// What the box is.
    told: Told,
    open: Open,
    /// The cursor's type and frame before the box.
    cursor: [u32; 2],
}

/// Level-ups under way (E24 M4 T2c, design §5.5): the dialog open, those after it, what they
/// raised so far (shown at the last's close: the primary skills, a Leadership's morale, a Luck's
/// luck) and what follows the last.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Leveling {
    open: LevelBox,
    rest: VecDeque<LevelBox>,
    raised: Gain,
    after: After,
}

/// What follows a visit's level-ups (E24 M4 T2c): where its handler called `hero_level_up`.
#[derive(Debug, Clone, PartialEq, Eq)]
enum After {
    /// A find's first visit's end (its place among [`Start::finds`]): an event's slot and the
    /// scenario's test, then the idle step ([`Loop::found_end`]).
    Found(usize),
    /// An answer's end (its find's place): the status window, its next question, its object
    /// taken or its terrains revealed ([`Loop::answer_end`]).
    Answer(usize, Box<Answer>),
    /// A dissolve's end: the step's end, then the actions given meanwhile.
    Step(Vec<Action>),
    /// A visit's end with nothing more to show (E24 M4 T6, the reviewer's R2-3: a battle whose
    /// monster stands, [`Loop::visited`]).
    Visited(usize),
}

/// An object's dissolve under way (E24 M4 T2a2 part B, design §5.3): its start, the draw before
/// it and the actions given meanwhile (taken after it as a walk's are).
#[derive(Debug, Clone, PartialEq, Eq)]
struct Dissolving {
    start: u32,
    before: Arc<Draw>,
    /// The picture its rectangle blends from when not the screen's ([`Dissolve::from`]), the
    /// rectangle and its steps' time (E24 M4 T2d3: an object's and a boat's the tables' middle
    /// at 65 ms, Summon Boat's a boat's box at 33 ms).
    from: Option<Arc<Draw>>,
    rect: [i64; 4],
    step_ms: u32,
    queued: Vec<Action>,
    /// What the panels show at its end (an artifact's primary skills: `find-artifact`'s Attack at
    /// 1580, the hero's window built again by the step's end).
    gain: Gain,
    /// The palettes' cycling held meanwhile: an object's removal's (`visit_resource_pile`, the
    /// pickups' and the monsters' handlers), not a boat's (E24 M4 T2d2: `hero_loss_effect`
    /// 0x4ACBB0 alone, `board`'s cycles at 200, 400 and 600).
    holds: bool,
    /// What follows its end.
    then: Then,
}

/// A battle's switch under way (E24 M4 T5b2, design §5.21): its phase and when it began, the
/// screen as the map last showed it, the find whose visit began it and its battle.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Switching {
    phase: Phase,
    start: u32,
    before: Arc<Draw>,
    find: usize,
    fight: Fight,
    /// The answer that began it, its boxes shown after it (E24 M4 T5b2: a mine's guards').
    rest: Option<Box<Answer>>,
}

/// A battle's switch's phases (E24 M4 T5b2, design §5.19, §5.21).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Phase {
    /// The map's fade out (`battle_start` 0x462A20: `screen_fade_out` 0x6030E0's shifts 0, 1, 2
    /// of the screen as it showed, then black).
    Out,
    /// The battle's fade in at the black (three steps, the last the battle whole).
    In,
    /// The battle under way: its result window's OK (Enter's release, the result's command
    /// 0x200/0xD; the host's [`Action::BattleClosed`]) ends it.
    Battle,
    /// The battle's fade out with its result window (`battle_manager_exit` 0x463260).
    Back,
    /// The map back (`manager_run` 0x4B0C70: the whole screen drawn, the mouse-over again) and
    /// faded in from the black.
    Map,
}

/// What follows a dissolve's end (E24 M4 T2d2).
#[derive(Debug, Clone, PartialEq, Eq)]
enum Then {
    /// An object taken (E24 M4 T2a2 part B): the gain shown, the levels reached, the key's step's
    /// end.
    Taken,
    /// A boarding's dissolve: the step's animation onto the boat, the walk going on.
    Board(Box<Boarding>),
    /// A landing's: `visit_and_update`'s redraws, then the walk's end (a key's or not).
    Landed(bool),
    /// Summon Boat's boat taken off its tile in sight (E24 M4 T2d3): its arrival dissolved next.
    Summoned(Box<Cast>, Summoned),
    /// Its arrival: the cost spent, the cast's end.
    Cast(Box<Cast>),
    /// A computer's pickup (E24 M4 T4c): its turn going on.
    Ai,
    /// A computer's hero's landing (E24 M4 T4c2): its turn going on.
    AiLanded,
    /// A computer's boarding (E24 M4 T4c2): the step onto the boat and whether it is one draw,
    /// its animation next, then its turn going on.
    AiBoard(Hero, bool),
    /// A hero a battle took off the map (E24 M4 T5b2): its slot left out of the kingdom's list,
    /// then the player's next hero or town; since E24 M4 T6 (R5-4, a draw) the monster to take
    /// off after it, if any.
    Lost(Option<usize>),
    /// The actions given meanwhile, then the idle step (E24 M4 T6: a draw's monster taken).
    Resume,
}

/// A step onto a boat held by its dissolve (E24 M4 T2d2): the walk, the boat's tile and whether
/// the step is one draw.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Boarding {
    moving: Moving,
    next: Pos,
    single: bool,
}

/// The visit a step ends with by its tile's class (`visit_object`'s cases 8 and 3, E24 M4 T2d2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Aboard {
    /// The empty boat boarded, as it lay.
    Board(LiveBoat),
    /// The land stepped onto from the boat.
    Land,
}

/// What a path's last node takes in place of a step (E24 M4 T6, [`Loop::last_node`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Beside {
    /// The find's visit from beside, by its place among the finds.
    Find(usize),
    /// A record's visit from beside the map does not show ([`Loop::pass_by`]).
    Unshown,
}

/// The selected hero's reach in play (E24 M5 T4a, [`Loop::hosted_reach`]).
#[derive(Debug, Clone, PartialEq, Eq)]
enum Reached {
    /// No host, or no hero selected: the start's facts.
    Unhosted,
    /// The host's for the stand.
    Given(Arc<Reach>),
    /// Asked of the host: the action waits.
    Asked,
}

/// A dissolve's steps (`screen_dissolve` 0x602DC0: eight, the blend's eighths).
const DISSOLVE_STEPS: u32 = 8;

/// A dissolve's step once it ended: its rectangle the new picture (E24 M4 T2d3).
const DISSOLVE_DONE: u8 = 8;

/// Summon Boat's spell (the spell table's 0; E24 M4 T2d3).
const SUMMON_BOAT: u8 = 0;

/// Kind 6, a resource found (the status window's kind a pile's visit asks).
const KIND_FOUND: u8 = 6;

/// A creature `stack` joining `army`: the slot of its kind, else the first empty one; none with
/// no room (*provisional*: `army_dialog_with` not built).
fn joined(army: &mut [Option<Stack>; 7], stack: Stack) {
    let slot = army
        .iter()
        .position(|s| s.is_some_and(|s| s.creature == stack.creature))
        .or_else(|| army.iter().position(Option::is_none));
    if let Some(slot) = slot.and_then(|i| army.get_mut(i)) {
        match slot {
            Some(have) => have.count = have.count.saturating_add(stack.count),
            None => *slot = Some(stack),
        }
    }
}

/// What an open box is, which says what its close does (E24 M4 T2a2).
#[derive(Debug, Clone, PartialEq, Eq)]
enum Told {
    /// End Turn's question: its OK ends the turn.
    Question,
    /// The week's or the month's message: the date's window steps behind it (the dialog's +0x6C).
    Week,
    /// The scenario's warning at the turn's end (GENRLTXT 7): its close goes on with the turn's
    /// change.
    Warning,
    /// A find's first visit's box (the find's place among [`Start::finds`], the box's among its
    /// first ones): its close opens the next with what it shows, after the last shows the gain,
    /// takes the object or tests the scenario, then the idle step.
    Found(usize, usize),
    /// A find's question (E24 M4 T2b2 part A1; the find's place among [`Start::finds`], since
    /// part A2 the question itself: a first one or an answer's next): its OK and its Cancel show
    /// their answers.
    Asked(usize, Box<Question>),
    /// A question's answer's box (its find's place, the box's among the answer's, the answer):
    /// its close opens the next, after the last the answer's end ([`Loop::answered`]).
    Answer(usize, usize, Box<Answer>),
    /// A level-up's dialog (E24 M4 T2c): its close raises its primary skill and opens the next
    /// level's, after the last [`Loop::leveled`].
    Level(Box<Leveling>),
    /// The days left to take a town: at its close the turn's selection's window built again
    /// (*provisional*: `turn-no-town` at 2060 shows the date's window unstepped, as `hero_select`
    /// after the box would build it; not read), then the idle step.
    DaysLeft,
    /// Any other (a find told again): its close goes to the idle step.
    Plain,
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
        // Computer Walk Speed by the hero speeds' frames (design §5.2: `walk_frame_pixels` by
        // the option's number); 5, past the tables, one draw (*provisional*: not recorded alone).
        let computer_pixels = start
            .computer
            .speed
            .checked_sub(1)
            .and_then(|i| rules.walk.pixels.get(i))
            .cloned()
            .unwrap_or_default();
        let cursor = Cursor {
            kind: CURSOR_TYPE,
            frame: rules.arrow,
            at: rules.drawn_at(start.pointer),
            shown: true,
        };
        // The minimap at 0 as the still shows it (the harness set the view through it).
        let minimap = Minimap {
            view: start.view,
            hero: start.hero.map(|h| h.at),
            index: start.index,
            owners: Arc::new(Vec::new()),
            walked: Arc::new(Vec::new()),
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
                message: None,
                computer: false,
                heroes: Arc::new(start.heroes.clone()),
                gone: Arc::new(Vec::new()),
                dissolve: None,
                revealed: 0,
                attacking: None,
                owners: Arc::new(Vec::new()),
                hidden: false,
                boats: Arc::new(start.boats.clone()),
                walked: Arc::new(Vec::new()),
                switch: None,
                left: Vec::new(),
            },
            hero: start.hero,
            lifted: false,
            hidden: false,
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
            reached: start.reached.clone(),
            now: 0,
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
            rehint: false,
            action: -1,
            heroes: start.heroes.clone(),
            index: start.index,
            chosen: start.chosen.clone(),
            held: None,
            keyed: Vec::new(),
            dragging: None,
            scale: rules
                .minimap
                .click(usize::try_from(start.size).unwrap_or(usize::MAX)),
            status_held: false,
            player: start.player,
            move_reminder: start.move_reminder,
            question: start.question.clone(),
            turns: Arc::new(start.turns.clone()),
            next_turn: 0,
            turns_asked: None,
            boxed: None,
            waiting: VecDeque::new(),
            finds: Arc::new(start.finds.clone()),
            told: Vec::new(),
            answered: Vec::new(),
            owners: Arc::new(Vec::new()),
            town: start.town.clone().map(Arc::new),
            days: -1,
            found: None,
            dissolving: None,
            switching: None,
            whole: None,
            switched: Vec::new(),
            left: Vec::new(),
            slots_left: Vec::new(),
            flush: false,
            computer_pixels,
            blackout: start.computer.blackout,
            ai: None,
            ai_step: None,
            within: Vec::new(),
            experience: start.experience,
            hero_level: start.experience.map_or(1, |e| rules.level_up.level_of(e)),
            experienced: start.index,
            level_ups: Arc::new(start.level_ups.clone()),
            leveling: None,
            due_levels: VecDeque::new(),
            boats: start.boats.clone(),
            water: Arc::clone(&start.water),
            aboard: None,
            casts: Arc::new(start.casts.clone()),
            next_cast: 0,
            book: false,
            moves: start.moves.clone().map(Arc::new),
            host: start.host.clone().map(Arc::new),
            walk_asked: None,
            reach_asked: None,
            next_move: 0,
            walking: None,
            last_move: None,
            battles: Vec::new(),
            walked_tiles: Arc::new(Vec::new()),
            off_record: None,
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
        // A record that starts in another player's turn (E24 M4 T6, the reviewer's R1-1): its
        // first turn played at once, as after End Turn.
        if start.turns.first().is_some_and(|n| n.first) {
            l.turn_proceed(0)?;
        }
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
            // The current player's selected hero's (a computer's during its turn).
            points: match &self.ai {
                Some(ai) => ai.kingdom.heroes.get(ai.slot).map(|h| h.points),
                None => self
                    .selected_hero()
                    .and_then(|i| self.kingdom.heroes.get(i))
                    .map(|h| h.points),
            },
            date: self.kingdom.date,
            player: self.player,
            other: match &self.kingdom.status {
                Some(Status::Turn(s)) => Some([s.crest, s.sand, s.glass]),
                _ => None,
            },
            book: self.book,
            battle: self
                .whole
                .filter(|(at, _)| *at == t)
                .map(|(_, winner)| winner),
        }
    }

    /// The states the screen showed inside the last millisecond run before its last (E24 M4
    /// T1b2: a computer's steps at speed 4 seen, each drawn once with no wait), in order; taken.
    pub fn take_held(&mut self) -> Vec<State> {
        core::mem::take(&mut self.within)
    }

    /// The millisecond End Turn's change asked the host for the turns after it (E24 M5 T3b): live,
    /// with no next turn known; none since [`Loop::add_turns`] gave them.
    #[must_use]
    pub fn turns_asked(&self) -> Option<u32> {
        self.turns_asked
    }

    /// The turns' starts after the known ones, given by the host (E24 M5 T3b: the hot seat's
    /// session played End Turn); the question cleared.
    pub fn add_turns(&mut self, turns: &[NextTurn]) {
        Arc::make_mut(&mut self.turns).extend_from_slice(turns);
        self.turns_asked = None;
    }

    /// The walk asked of the host (E24 M5 T4a, [`Start::host`]): a path's walk or a key's step
    /// past the engine's known walks; none since [`Loop::add_walks`] or [`Loop::refuse_walk`].
    #[must_use]
    pub fn walk_asked(&self) -> Option<&WalkAsked> {
        self.walk_asked.as_ref()
    }

    /// The engine's walks after the known ones and the finds their ends tell, given by the host
    /// (E24 M5 T4a: it played the walk asked); the question cleared.
    pub fn add_walks(&mut self, moves: &[Move], finds: &[Found]) {
        if let Some(known) = &mut self.moves {
            Arc::make_mut(known).extend_from_slice(moves);
        }
        Arc::make_mut(&mut self.finds).extend_from_slice(finds);
        self.walk_asked = None;
    }

    /// The walk `asked` refused by the engine (E24 M5 T4a): played again from its stand it changes
    /// nothing; the question cleared.
    pub fn refuse_walk(&mut self, asked: &WalkAsked) {
        if let Some(host) = &mut self.host {
            let refused = (asked.stand, asked.tiles.clone());
            Arc::make_mut(host).refused.push(refused);
        }
        self.walk_asked = None;
    }

    /// The millisecond and the stand whose reach is asked of the host (E24 M5 T4a): the selected
    /// hero's where it stands, at a click on the view or the mouse over it; none since
    /// [`Loop::add_reach`].
    #[must_use]
    pub fn reach_asked(&self) -> Option<(u32, Stand)> {
        self.reach_asked
    }

    /// The reach of `stand` given by the host (E24 M5 T4a); the question cleared.
    pub fn add_reach(&mut self, stand: Stand, reach: &Arc<Reach>) {
        if let Some(host) = &mut self.host {
            Arc::make_mut(host).reaches.push((stand, Arc::clone(reach)));
        }
        self.reach_asked = None;
    }

    /// The map's heroes ([`Start::heroes`]' order) where they stand and face now, the selected
    /// one's where its walk left it (E24 M5 T3b: what the host keeps of a view it leaves).
    #[must_use]
    pub fn heroes(&self) -> Vec<Hero> {
        let mut heroes = self.heroes.clone();
        if let (Some(index), Some(hero)) = (self.index, self.hero)
            && let Some(h) = heroes.get_mut(index)
        {
            *h = hero;
        }
        heroes
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
        self.now = t;
        self.within.clear();
        // The loop's turn and a walk's wait alike run the tick (0x4ED490: the poll, the palettes).
        self.tick(t);
        // A battle's switch (E24 M4 T5b2): the map stopped until its end, then the battle's
        // result on the map.
        if self.switching.is_some() {
            self.switch_step(t)?;
        } else if self.dissolving.is_some() {
            // An object's dissolve (E24 M4 T2a2 part B): the map stopped until its end, then the
            // step's end, the actions given meanwhile and the idle step.
            if self.dissolve_step(t)
                && let Some(dissolving) = self.dissolving.take()
            {
                self.dissolved(t, dissolving)?;
            }
        } else {
            self.turn_of_loop(t)?;
        }
        // A box opened by the loop's turn (a walk's end) emptied the queue then: the actions given
        // after it are not behind it (E24 M4 T4b1: `game-asks`' click and pointer at 1000 after
        // the Arena's box at 250).
        self.flush = false;
        for &action in actions {
            // During a battle's switch only its result's OK counts (*provisional*: the fades'
            // and the battle's own input the battle manager's); the mouse's point is read.
            if let Some(switching) = &self.switching {
                self.pointer = action.point().unwrap_or(self.pointer);
                if switching.phase == Phase::Battle
                    && matches!(action, Action::BoxKeyUp(Key::Ok) | Action::BattleClosed)
                {
                    self.switch_back(t);
                }
                continue;
            }
            if let Some(dissolving) = &mut self.dissolving {
                // The mouse's point is read at once; the message waits as during a walk.
                self.pointer = action.point().unwrap_or(self.pointer);
                dissolving.queued.push(action);
                continue;
            }
            if self.ai.is_some() {
                // During a computer's turn the messages wait in the queue, which the next human's
                // start empties (*provisional*); the mouse's point is read at once.
                if let Action::Pointer(at)
                | Action::Click(at)
                | Action::Press(at)
                | Action::Release(at) = action
                {
                    self.pointer = at;
                }
                continue;
            }
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
                    wait_in(moving, action, self.rules.bugs);
                }
                None => {
                    self.act(t, action, true)?;
                    // Given after the click was handled: behind the walk's emptying of the queue.
                    self.flushed = false;
                }
            }
            // A box opened or a turn started: the queue emptied, the rest of the millisecond's
            // input with it (E24 M4 T1a, *provisional*: the window's pass's).
            if core::mem::take(&mut self.flush) {
                break;
            }
        }
        self.hint_again();
        match self.off_record.take() {
            Some(why) => Err(why),
            None => Ok(()),
        }
    }

    /// The loop's turn at `t` outside a dissolve: a computer player's turn (E24 M4 T1b2: its walk's
    /// waits; at its end the next turn, then the manager's next message, the redraws due since run
    /// then), a walk's wait or its end, else the idle message.
    fn turn_of_loop(&mut self, t: u32) -> Result<(), Unplayable> {
        match self.ai_on(t)? {
            Some(true) => self.idle_message(t),
            Some(false) => {}
            None => match self.moving.take() {
                Some(moving) => {
                    if let Some(queued) = self.carry_on(t, moving) {
                        self.resume(t, queued)?;
                    }
                }
                None => self.idle_message(t),
            },
        }
        Ok(())
    }

    /// The client's fix of MCL-Q22 (the user, 2026-10-31; [`Bugs::Fixed`]): after a redraw of
    /// every dialog item ([`Loop::draw_lists`]) the status bar shows the hint of the mouse's item
    /// again, where the original's chat edit leaves it empty until another item's hint. Off the
    /// view the panel's item under the mouse now and its hint now (a list's slot names the hero or
    /// town it holds now), looked at as the last; over the view the tile's hint memory holds (the
    /// mouse-over gives a new one only when the mouse's column or row changes, as after a
    /// scroll). Once per turn, after its actions: what the screen shows then (*provisional*).
    fn hint_again(&mut self) {
        if !core::mem::take(&mut self.rehint) || self.rules.over_view(self.pointer) {
            return;
        }
        let (kingdom, buttons, levels) = (&self.kingdom, self.buttons, self.levels);
        let item = self
            .rules
            .panels
            .at(self.pointer, |i| hover::takes(i, kingdom, buttons, levels));
        self.last_item = item;
        let facts = self.hover.as_ref().map(|h| h.facts());
        self.drawn.hint = hover::panel_hint(item, &self.kingdom, facts);
    }

    /// The tick (0x4ED490), each turn of the loop and of a walk's wait: the mouse poll when due
    /// (0x50D680: the cursor drawn where it reads the mouse), the palettes' step when due.
    fn tick(&mut self, t: u32) {
        if t >= self.palette_at {
            // An object's dissolve holds the cycling: the deadline moves on, the palettes stay
            // (E24 M4 T2a2 part B: `find-pile`'s due 200 at 165, 400 at 230, its cycles 0 to
            // 800); a boat's does not (E24 M4 T2d2).
            // So does a battle from its black until the map is back (E24 M4 T5b2, design §5.21:
            // recorded, `battle-won` none from 250 to 710).
            let battle = self
                .switching
                .as_ref()
                .is_some_and(|s| matches!(s.phase, Phase::In | Phase::Battle | Phase::Back));
            if !self.dissolving.as_ref().is_some_and(|d| d.holds) && !battle {
                self.ticks = self.ticks.saturating_add(1);
            }
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
        // A message box's own loop (E24 M4 T1a): the map stands still.
        if self.boxed.is_some() {
            self.box_step(t);
            return;
        }
        // The spell book's (E24 M4 T2d3): the map stands still, the tick runs.
        if self.book {
            return;
        }
        // The mouse holding a button down runs the button's loop: the tick only (0x45641C); so
        // does the minimap's (0x40A344, E24 M3 T6b2).
        if self.held.is_some() || self.dragging.is_some() {
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
        if self.boxed.is_some() {
            return self.box_act(t, action, fresh);
        }
        if self.book {
            return self.book_act(t, action, fresh);
        }
        if self.held.is_some() || self.dragging.is_some() {
            return self.holding(t, action, fresh);
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
            Action::Step(direction) | Action::Repeat(direction) => self.key_step(t, direction),
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
                        if self.held.is_some() || self.dragging.is_some() {
                            self.holding(t, Action::Release(at), fresh)?;
                        } else {
                            self.release(t);
                        }
                    }
                }
            }
            Action::Key(key) => self.key(t, key),
            Action::KeyUp(key) => self.key_up(t, key)?,
            Action::Visit => self.visit(t)?,
            // A box's keys with no box open: nothing (*provisional*: the map's own keys,
            // `adventure_key_down` 0x408C40, were not read for Enter and Escape).
            Action::Up
            | Action::RightPress
            | Action::RightRelease
            | Action::BoxKey(_)
            | Action::BoxKeyUp(_)
            | Action::Offer(_)
            | Action::BattleClosed => {}
            Action::Choose(i) => self.choose_slot(t, i)?,
            Action::Walk(to) => self.walk_to(t, to)?,
        }
        Ok(())
    }

    /// The kingdom's hero `i` chosen (E24 M4 T4c, [`Action::Choose`]): as its list's slot's click;
    /// the selected one nothing, one not known refused ([`Unplayable::Record`]).
    fn choose_slot(&mut self, t: u32, i: usize) -> Result<(), Unplayable> {
        if self.kingdom.selected == Selected::Hero(i) {
            return Ok(());
        }
        let Some(chosen) = self.chosen_hero(i).cloned() else {
            return self.unbuilt(Unplayable::Record([-1, -1, -1]));
        };
        self.keep_path();
        self.choose_hero(t, i, chosen);
        Ok(())
    }

    /// The selected hero's path to `to` shown and walked (E24 M4 T4c, [`Action::Walk`]): as its
    /// destination's two clicks, the path's arrows taken in their order (a record's walk may
    /// pass a tile beside its last: [`chain`] would find two ways on); no such path refused
    /// ([`Unplayable::NoPath`]).
    fn walk_to(&mut self, t: u32, to: Pos) -> Result<(), Unplayable> {
        let in_order = |from: Pos, arrows: &[Arrow]| -> Option<Vec<(Direction, Arrow)>> {
            let mut at = from;
            arrows
                .iter()
                .map(|a| {
                    let direction = Direction::between(at, a.at)?;
                    at = a.at;
                    Some((direction, *a))
                })
                .collect()
        };
        // The record's next walk's path first (E24 M4 T6: an earlier walk's to the same tile may
        // start beside the hero too).
        let (earlier, later) = self.paths.split_at(self.next_move.min(self.paths.len()));
        let path = self.hero.and_then(|hero| {
            later
                .iter()
                .chain(earlier)
                .filter_map(|p| in_order(hero.at, p))
                .find(|c| c.last().is_some_and(|(_, a)| a.at == to))
        });
        let Some(path) = path else {
            let tile = [i32::from(to.x), i32::from(to.y), i32::from(to.z)];
            return self.refused(Unplayable::NoPath(tile));
        };
        self.path = path;
        self.draw(t);
        self.buttons.move_hero = self.green();
        self.walk_path(t);
        Ok(())
    }

    /// Whether the map waits for input (E24 M4 T4c: the record player's, `crate::director`): no
    /// walk, computer's turn, dissolve, battle's switch (E24 M4 T6, the reviewer's R5-3), box,
    /// spell book or held button under way; a computer's turn the record ends in once its walks
    /// are played (E24 M4 T6, R1-1) waits for nothing more.
    #[must_use]
    pub fn idle(&self) -> bool {
        self.moving.is_none()
            && (self.ai.is_none() || self.ai_waits())
            && self.dissolving.is_none()
            && self.switching.is_none()
            && self.boxed.is_none()
            && self.waiting.is_empty()
            && !self.book
            && self.held.is_none()
            && self.dragging.is_none()
    }

    /// How many of a record's walks are not taken yet (E24 M4 T4c: the record player's); none
    /// without a record.
    #[must_use]
    pub fn walks_left(&self) -> usize {
        self.moves
            .as_ref()
            .map_or(0, |m| m.len().saturating_sub(self.next_move))
    }

    /// The battles the map handed over so far (E24 M4 T5b1, design §5.20, *provisional*), in
    /// order: each at the millisecond its walk ended (a plain cut) or, since E24 M4 T5b2, its
    /// switch showed it whole (a visit's fight, an answer's), with the record's number of the
    /// `GameEvents` that began it. The battle takes none of the map's time: a host shows it before
    /// that millisecond's frame and goes on from there.
    #[must_use]
    pub fn battles(&self) -> &[(u32, u32)] {
        &self.battles
    }

    /// The millisecond the battle's switch under way shows the battle whole and hands it over
    /// (E24 M4 T6, the reviewer's R5-3 and R5-7: the record player closes it there, so that the
    /// battle takes none of the map's time and that millisecond's change is its fade out's
    /// first), once the battle's fade in has begun; none otherwise.
    #[must_use]
    pub fn battle_whole_at(&self) -> Option<u32> {
        self.switching
            .as_ref()
            .filter(|s| s.phase == Phase::In)
            .map(|s| s.start.saturating_add(2 * FADE_MS))
    }

    /// The kingdom's slots the heroes a battle took off the map left, in order (E24 M4 T6, the
    /// reviewer's R5-6: each later slot one less from then on).
    #[must_use]
    pub fn slots_left(&self) -> &[usize] {
        &self.slots_left
    }

    /// The slot of the map's hero `index` in the kingdom's list as the map holds it now (E24 M4
    /// T6, the reviewer's R1-4: the record player's choice; a hero a battle took off the map gone
    /// from it, each day's list the record's): its choice's; on a map that knows no choices the
    /// selected hero's own (unless it is another map hero); none for a hero the list does not
    /// hold.
    #[must_use]
    pub fn slot_of(&self, index: usize) -> Option<usize> {
        let bare = self.chosen.heroes.is_empty() && self.index.is_none_or(|i| i == index);
        self.chosen
            .heroes
            .iter()
            .position(|c| c.as_ref().is_some_and(|c| c.hero == index))
            .or(match self.kingdom.selected {
                Selected::Hero(i) if bare => Some(i),
                _ => None,
            })
    }

    /// Whether a computer's turn the record ends in has played its walks (E24 M4 T6, R1-1): the
    /// map left in its screen.
    fn ai_waits(&self) -> bool {
        self.ai_step.is_none()
            && self.ai.as_ref().is_some_and(|ai| {
                ai.open && ai.walks.is_empty() && ai.steps.is_empty() && ai.ends.is_empty()
            })
    }

    /// Whether a box is open (E24 M4 T4c: the record player's): a message box, a question or a
    /// level-up's dialog.
    #[must_use]
    pub fn boxed(&self) -> bool {
        self.boxed.is_some()
    }

    /// Whether the open box takes an answer (E24 M4 T6, the reviewer's R2-2: the record player's):
    /// a question or a choice, a level-up's dialog, End Turn's question; not a find's box, an
    /// answer's, the week's message (each closed by Enter, the record's next answer kept for the
    /// box it answers: Pandora's Box's map text before its question).
    #[must_use]
    pub fn asks(&self) -> bool {
        self.boxed
            .as_ref()
            .is_some_and(|b| matches!(b.told, Told::Asked(..) | Told::Level(_) | Told::Question))
    }

    /// The middle of the open box's choice frame `n` (E24 M4 T6, the reviewer's R2-2: where the
    /// record player clicks the picture its answer chose, as a player's mouse would; a chest's,
    /// the Arena's, a School's, `CHOICE + n`); none without a choice's box open.
    #[must_use]
    pub fn choice_at(&self, n: usize) -> Option<[i32; 2]> {
        let dialog = &self.boxed.as_ref()?.open.message.dialog;
        let id = boxes::CHOICE.checked_add(u16::try_from(n).ok()?)?;
        dialog.items.iter().find_map(|item| match item {
            Item::Area(a) if a.id == id => Some([
                dialog.place[0] + a.at[0] + a.size[0] / 2,
                dialog.place[1] + a.at[1] + a.size[1] / 2,
            ]),
            _ => None,
        })
    }

    /// An action while the mouse holds a button down (`def_button_message`'s own loop from
    /// 0x456338: the tick only, every message the button's): a move over it presses it again,
    /// with its sound (0x4563A2), one off it releases it (0x4563B5), each drawn and shown at once;
    /// the left button's release ends the hold (0x10), so does the right one's (0x40, 0x456338),
    /// a click when the button shows pressed (0x456451: its command, `adventure_command`
    /// 0x409A70); every other message is taken and dropped. A message's own point decides (0x456359: `[esi + 4]`, `[esi + 8]`); the pointer
    /// (GetCursorPos) is set only by a `fresh` one, never by one taken from the queue after a
    /// step, which the mouse has left (E24 M3 T7's review, F4-3). The window's point off the
    /// frame (its focus lost: no mouse message in the original, WM_ACTIVATEAPP) leaves the hold
    /// as it is (F4-2). A hotkey's key up is dropped by both loops (a button's: its message to
    /// the mouse manager, 0x456341, then the next, 0x456350; the minimap's, design §4.14), so
    /// the button its key pressed stays drawn pressed with its command lost; the client's fix
    /// releases it without its command ([`Loop::key_released`]; MCL-Q24, MCL-Q28).
    fn holding(&mut self, t: u32, action: Action, fresh: bool) -> Result<(), Unplayable> {
        if let Action::Pointer(at) = action
            && !self.rules.on_frame(at)
        {
            return Ok(());
        }
        if let Action::KeyUp(key) = action {
            if self.rules.bugs == Bugs::Fixed {
                self.key_released(key);
            }
            return Ok(());
        }
        if self.dragging.is_some() {
            self.drag(t, action, fresh);
            return Ok(());
        }
        let (Some((id, over)), Action::Pointer(_) | Action::Release(_) | Action::RightRelease) =
            (self.held, action)
        else {
            return Ok(());
        };
        let inside = match action {
            Action::Pointer(at) | Action::Release(at) => {
                if fresh {
                    self.pointer = at;
                }
                let inside = self.rules.panels.holds(id, at);
                if inside != over {
                    self.held = Some((id, inside));
                    self.show_pressed(id, inside);
                    if inside {
                        self.press_sound(t);
                    }
                }
                inside
            }
            // The right button's up ends the hold where the last move left it (F4-5).
            _ => over,
        };
        if matches!(action, Action::Release(_) | Action::RightRelease) {
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
    /// its command; nothing for a key that pressed none, nor when the button no longer shows
    /// pressed (design §4.15: a mouse's hold released it meanwhile, its click given or not; E24
    /// M3 T7's review, F4-4).
    fn key_up(&mut self, t: u32, key: Hotkey) -> Result<(), Unplayable> {
        let Some(at) = self.keyed.iter().position(|&(k, _)| k == key) else {
            return Ok(());
        };
        let (_, id) = self.keyed.remove(at);
        if self.drawn.pressed.binary_search(&id).is_err() {
            return Ok(());
        }
        self.show_pressed(id, false);
        self.command(t, id)
    }

    /// The client's fix of MCL-Q24 and MCL-Q28 (the user, 2026-10-31; [`Bugs::Fixed`]): a
    /// hotkey's key up while the mouse holds the minimap or a button down releases the button its
    /// key pressed, drawn and shown, without its command and without a sound (as a release off a
    /// button); the button the mouse holds stays pressed, its hold deciding (its key's later up
    /// would find it released: F4-4).
    fn key_released(&mut self, key: Hotkey) {
        let Some(at) = self.keyed.iter().position(|&(k, _)| k == key) else {
            return;
        };
        let (_, id) = self.keyed.remove(at);
        if self.held.is_none_or(|(held, _)| held != id) {
            self.show_pressed(id, false);
        }
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
    /// underground's (0x409F6B), next hero's (0x409D89), end turn's (0x409D12, E24 M4 T1a), cast a
    /// spell's (E24 M4 T2d3), the hero list's arrows (0x403220) and the town list's (0x403280); the
    /// rest a dialog's or the engine's (M4, M5).
    fn command(&mut self, t: u32, id: u16) -> Result<(), Unplayable> {
        match id {
            panels::id::UNDERGROUND => {
                self.underground(t);
                Ok(())
            }
            panels::id::NEXT_HERO => self.next_hero(t),
            panels::id::END_TURN => self.end_turn(t),
            panels::id::CAST => self.cast_open(t),
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

    /// End Turn's command (`adventure_command` 0x409D12 to 0x409D84; E24 M4 T1a, design §5.1):
    /// outside the tutorial (game +0x1F69D: the viewer plays none), with Move Reminder set and a
    /// hero to go to (`next_hero_exists` 0x4BAB90: one awake that can move) the question first
    /// (style 2, [`Loop::open_box`]: its OK goes on to the turn's change, anything else ends the
    /// command there); else the turn's change at once ([`Loop::turn_change`]).
    fn end_turn(&mut self, t: u32) -> Result<(), Unplayable> {
        if self.move_reminder && self.kingdom.next_hero() {
            let Some(question) = self.question.clone() else {
                return self.unbuilt(Unplayable::Command(panels::id::END_TURN));
            };
            self.open_box(question, Told::Question);
            return Ok(());
        }
        self.turn_change(t)
    }

    /// The scenario's test of the viewer's turn's end (`scenario_condition_met` 0x4F2CE0 from
    /// `next_player_turn`; E24 M4 T2a2, design §5.3): with heroes but no town, the days without a
    /// town not counting become the tables' count and the warning's box is given; none without
    /// [`Start::town`], nor on a record (E24 M4 T6, *provisional*: the engine's rules, which have no
    /// days without a town yet, E10; MCL-STUB-4).
    ///
    /// # Errors
    /// The days down to their last (their end at the next day's start: the player's elimination,
    /// not built).
    fn town_test(&mut self) -> Result<Option<Arc<MessageBox>>, Unplayable> {
        // stub: MCL-STUB-4 (a record's days without a town: the engine tells none, E10)
        let Some(town) = self.town.as_ref().filter(|_| self.moves.is_none()) else {
            return Ok(None);
        };
        if !self.kingdom.towns.is_empty() || self.kingdom.heroes.is_empty() {
            return Ok(None);
        }
        match self.days {
            -1 => {
                self.days = i32::from(self.rules.turn.town_days);
                Ok(Some(town.lost.clone()))
            }
            ..=1 => Err(Unplayable::Command(panels::id::END_TURN)),
            _ => Ok(None),
        }
    }

    /// The turn's change (`next_player_turn` 0x4C6FE0, `player_turn_start` 0x4CC7D0; design
    /// §5.1), all in End Turn's millisecond (a computer's turn out of sight takes no virtual time):
    /// the next of [`Start::turns`] (none: not built, refused in a recording). The viewer's turn's
    /// end ([`Loop::turn_end`]), then a computer player's turn in sight when the facts give one
    /// (E24 M4 T1b2: [`Loop::computer_start`]), else the next day ([`Loop::day_start`]).
    fn turn_change(&mut self, t: u32) -> Result<(), Unplayable> {
        // The scenario's tests first (E24 M4 T2a2): a warning's box, the change at its close.
        if let Some(warning) = self.town_test()? {
            self.open_box(warning, Told::Warning);
            return Ok(());
        }
        self.turn_proceed(t)
    }

    /// The turn's change after the scenario's tests ([`Loop::turn_change`]).
    fn turn_proceed(&mut self, t: u32) -> Result<(), Unplayable> {
        let refused = Unplayable::Command(panels::id::END_TURN);
        let Some(next) = self.turns.get(self.next_turn).cloned() else {
            // Live, the host's session is asked (E24 M5 T3b): the change waits for its turns.
            if self.live {
                self.turns_asked.get_or_insert(t);
            }
            return self.unbuilt(refused);
        };
        if let Status::Turn(status) = next.turn.status {
            if !self.computer_playable(&next) {
                return self.unbuilt(refused);
            }
            let ended = self.turn_end(t);
            return self.computer_start(t, next, status, ended);
        }
        // The selection's map hero or town, known before anything changes.
        let Some(choice) = self.turn_choice(&next) else {
            return self.unbuilt(refused);
        };
        let ended = self.turn_end(t);
        self.day_start(t, next, choice, ended);
        Ok(())
    }

    /// The viewer's turn's end (E24 M4 T1a, design §5.1): the selected hero put back, its path
    /// kept for its choice; the selection's window memory kept (none under a kind asked: the
    /// selection's next). A redraw's deadline already passed moves to the tables' step from now
    /// without a step (*provisional*: recorded, `turn-question` at 1060, not read; E24 M4 T1b2:
    /// here, before a computer's turn in sight too, whose end then steps the objects once,
    /// [`Loop::ai_end`]).
    fn turn_end(&mut self, t: u32) -> Ended {
        // The days without a town counted down (`next_player_turn`: +0x3D above 0; E24 M4 T2a2).
        if self.days > 0 {
            self.days -= 1;
        }
        self.keep_path();
        let shown = |s: &Status| matches!(s, Status::Hero(_) | Status::Town(_));
        let window = self
            .kingdom
            .selection
            .take()
            .or_else(|| self.kingdom.status.clone().filter(shown));
        self.lifted = false;
        if self.frame_at <= t {
            self.frame_at = t.saturating_add(self.rules.frame_ms);
        }
        Ended {
            before: self.kingdom.selected,
            window,
            path: core::mem::take(&mut self.path),
        }
    }

    /// The next day's selection as `next` gives it: a hero's map hero (by the turn's list, E24 M4
    /// T6, R1-4; else by the mouse-over's facts, else its choice's) or a town's choice, none when
    /// neither is known.
    fn turn_choice(&self, next: &NextTurn) -> Option<Choice> {
        let selected = next.turn.kingdom.selected;
        let town = match selected {
            Selected::Town(i) => Some((i, self.chosen.towns.get(i).cloned().flatten()?)),
            _ => None,
        };
        let hero = match (selected, &next.list) {
            (Selected::Hero(i), Some(list)) => Some((i, (*list.get(i)?)?)),
            (Selected::Hero(i), None) => Some((i, self.map_hero(i)?)),
            _ => None,
        };
        Some(Choice { hero, town })
    }

    /// A human player's turn's start (`player_turn_start` 0x4CC7D0; design §5.1): the map's heroes
    /// where the facts have them (E24 M4 T1b2); the current player and the kingdom the turn's (the
    /// date, the resources, the heroes' rows and points, the towns, the selection, the lists'
    /// first slots), the status window shown kept until it is built again, the selection's window
    /// the one before for the same hero or town, else its choice's (*provisional*: the engine's
    /// from Part B); the input queue emptied (`input_flush` 0x4EC570); a computer's screen undone
    /// (the buttons enabled, the minimap drawn again: `adventure_full_update` 0x417478). The screen
    /// redrawn whole (`adventure_full_update` 0x417420) with the date's window asked for the
    /// tables' time and built, its sound with it; then the turn's hero chosen (`hero_select` with
    /// 0, 0, 1: lifted, the view centred on it, the date's window built again, its path made again
    /// from its destination), else its town (`town_select`); next hero's state (0x419450). At a
    /// week's start but the game's first its sound (0x4CCAE5) and its message
    /// ([`Loop::open_box`]).
    fn day_start(&mut self, t: u32, next: NextTurn, choice: Choice, ended: Ended) {
        self.next_turn += 1;
        let NextTurn {
            turn,
            message,
            heroes,
            taken,
            flags,
            lost,
            boats,
            list,
            ..
        } = next;
        let Ended {
            before,
            window,
            path,
        } = ended;
        self.place(&heroes);
        self.changed(&taken, &flags, &lost, &boats);
        // The choices by the day's list (E24 M4 T6, R1-4): each hero's as it was, a hired one's
        // none.
        if let Some(list) = list {
            let before = core::mem::take(&mut self.chosen.heroes);
            self.chosen.heroes = list
                .iter()
                .map(|index| {
                    let index = (*index)?;
                    before.iter().flatten().find(|c| c.hero == index).cloned()
                })
                .collect();
        }
        self.player = turn.player;
        let selected = turn.kingdom.selected;
        let mut kingdom = turn.kingdom;
        kingdom.status = self.kingdom.status.take();
        kingdom.realm = self.kingdom.realm.take();
        // The window memory built for the same hero or town first: a map's start's choices
        // (`Live::at_start`) are the map's, not the game's.
        kingdom.selection = window
            .filter(|_| selected == before)
            .or_else(|| match selected {
                Selected::Hero(i) => self.chosen_hero(i).map(|c| Status::Hero(c.status.clone())),
                Selected::Town(_) => choice
                    .town
                    .as_ref()
                    .map(|(_, c)| Status::Town(c.status.clone())),
                Selected::None => None,
            });
        self.kingdom = kingdom;
        self.flush = true;
        self.drawn.computer = false;
        // The screen redrawn whole, the date's window asked and built.
        self.asked = 1;
        self.end = t.saturating_add(self.rules.turn.asked);
        self.draw_lists();
        self.choose(t, true);
        self.draw_minimap();
        self.draw(t);
        // The turn's selection.
        match (choice.hero, choice.town) {
            (Some((i, index)), _) => {
                self.buttons.move_hero = false;
                self.hero = self.heroes.get(index).copied();
                self.index = Some(index);
                self.selected_id = self.record_index(index);
                self.walked = true;
                self.path = if before == Selected::Hero(i) {
                    path
                } else {
                    let arrows = self.chosen_hero(i).map(|c| c.path.clone());
                    self.hero
                        .zip(arrows)
                        .and_then(|(h, a)| chain(h.at, &a))
                        .unwrap_or_default()
                };
                self.lift(t, true);
                self.buttons.move_hero = self.green();
                self.draw(t);
                self.look_again();
            }
            (None, Some((i, chosen))) => self.choose_town(t, i, chosen),
            (None, None) => {
                (self.hero, self.index, self.selected_id) = (None, None, None);
                self.buttons.move_hero = false;
            }
        }
        self.buttons.next_hero = self.kingdom.next_hero();
        // The days left to take a town (E24 M4 T2a2, design §5.3: the player's +0x3D at 0 or more;
        // recorded over the day's whole redraw, `turn-no-town` at 1060), then the week's message.
        let mut boxes = Vec::new();
        if let Some(town) = &self.town
            && let Some(left) = usize::try_from(self.days - 1)
                .ok()
                .and_then(|d| town.left.get(d))
        {
            boxes.push((left.clone(), Told::DaysLeft));
        }
        let date = self.kingdom.date;
        if turn::starts_week(date) {
            if let Some(sound) = &mut self.sound {
                sound.effect(t, turn::sound(&self.rules.turn, date));
            }
            if let Some(message) = message {
                boxes.push((message, Told::Week));
            }
        }
        let mut boxes = boxes.into_iter();
        if let Some((first, told)) = boxes.next() {
            self.open_box(first, told);
            self.waiting.extend(boxes);
        }
    }

    /// The map's heroes the facts give, each on its tile and facing (E24 M4 T1b2).
    fn place(&mut self, heroes: &[(usize, Hero)]) {
        for &(index, hero) in heroes {
            if let Some(h) = self.heroes.get_mut(index) {
                *h = hero;
            }
        }
    }

    /// What a turn's start changes at once ([`NextTurn::taken`], [`NextTurn::flags`],
    /// [`NextTurn::lost`]; E24 M4 T4c2, MCL-Q46 (c); E24 M4 T6, R1-5), also a computer's move's
    /// end's ([`End`]): the objects taken off, the flags raised, the heroes off the map and (E24
    /// M4 T6, R3-3) the boats told put on it; the next draw shows them.
    fn changed(
        &mut self,
        taken: &[usize],
        flags: &[(usize, u8)],
        lost: &[usize],
        boats: &[LiveBoat],
    ) {
        self.take_off(taken);
        self.put_boats(boats);
        for &flag in flags {
            self.raise(Some(flag));
        }
        for &index in lost {
            if !self.left.contains(&index) {
                self.left.push(index);
            }
        }
    }

    /// The map's hero `index`'s record index by the mouse-over's facts (E24 M4 T6, R1-4: by the map
    /// hero, not by a list's slot that a hero lost or hired moves).
    fn record_index(&self, index: usize) -> Option<u8> {
        self.hover
            .as_ref()
            .and_then(|h| h.facts().heroes.get(index))
            .map(|h| h.0)
    }

    /// The map's objects `taken` off the map at once as a turn starts ([`NextTurn::taken`]: E24
    /// M4 T4c2, MCL-Q46 (c)): the next draw shows them gone.
    fn take_off(&mut self, taken: &[usize]) {
        if !taken.is_empty() {
            let gone = Arc::make_mut(&mut self.drawn.gone);
            gone.extend(taken);
            gone.sort_unstable();
            gone.dedup();
        }
    }

    /// Whether the loop plays a computer's turn in sight from `next` (E24 M4 T1b2): the viewer's
    /// explored tiles known (the mouse-over's facts) and each walk's hero on the map.
    fn computer_playable(&self, next: &NextTurn) -> bool {
        self.hover.is_some() && next.walks.iter().all(|w| w.hero < self.heroes.len())
    }

    /// A computer player's turn's start (E24 M4 T1b2, design §5.1, §5.2): the map's heroes where
    /// the facts have them; the computer the current player; the busy cursor (`mouse_set_cursor`
    /// 0x4C74FE), a shown path cleared and move hero disabled (`path_cancel` 0x419300), the
    /// viewer's selection cleared at its turn's end (0x4175A0: the lists show none);
    /// `covered_view_redraw` 0x4CA5B0: four buttons disabled and the lists, the quest log, the
    /// resource bar and every item drawn for the viewer (its kingdom, not the computer's), the
    /// shield in the minimap's place (every minimap draw returning at once until the next human's
    /// start); under Blackout Computer the view drawn covered (not built: the turn falls in End
    /// Turn's millisecond). Then `ai_player_turn` 0x525E80: kind 5 built (its value the facts')
    /// and its heroes' turns ([`Loop::ai_on`]).
    fn computer_start(
        &mut self,
        t: u32,
        next: NextTurn,
        status: TurnStatus,
        ended: Ended,
    ) -> Result<(), Unplayable> {
        self.next_turn += 1;
        let NextTurn {
            turn,
            heroes,
            walks,
            taken,
            flags,
            lost,
            boats,
            open,
            ..
        } = next;
        self.place(&heroes);
        self.changed(&taken, &flags, &lost, &boats);
        self.player = turn.player;
        // Its music (design §5.1; E24 M4 T4c: the tables' tracks in turn, *provisional*).
        let tracks = &self.rules.turn.computer;
        if let (Some(sound), false) = (&mut self.sound, tracks.is_empty()) {
            sound.computer(t, &tracks[self.next_turn % tracks.len()]);
        }
        self.set_cursor(DEFAULT_CURSOR, self.rules.busy);
        self.buttons.move_hero = false;
        self.kingdom.selected = Selected::None;
        self.drawn.computer = true;
        self.draw_lists();
        self.asked = 0;
        let status = TurnStatus {
            crest: u32::from(turn.player),
            sand: 0,
            glass: 0,
            ..status
        };
        self.show_status(Status::Turn(status));
        self.ai = Some(AiTurn {
            kingdom: turn.kingdom,
            status,
            stepped: t,
            slot: walks.first().map_or(0, |w| w.slot),
            walks: walks.into(),
            steps: VecDeque::new(),
            done: 0,
            ends: VecDeque::new(),
            judged: None,
            chosen: false,
            open,
            ended,
        });
        self.ai_on(t)?;
        Ok(())
    }

    /// A computer player's turn from `t` (E24 M4 T1b2, design §5.2): its step's frames as their
    /// times come, each step's end (the points taken after the animation: the recording's, the
    /// standing group at a move's last step, the offset 0); a move's end's changes (E24 M4 T6,
    /// R1-3, R1-5: [`End`]); its next step, its next hero's turn, its end (none for a turn the
    /// record ends in, R1-1). Whether it ended; none without a computer's turn under way.
    fn ai_on(&mut self, t: u32) -> Result<Option<bool>, Unplayable> {
        loop {
            let Some(ai) = &mut self.ai else {
                return Ok(None);
            };
            if let Some((direction, frame, until)) = self.ai_step {
                if t < until {
                    return Ok(Some(false));
                }
                // A move's last step (E24 M4 T6, R1-3: each command's move ends at its own).
                let end = ai.ends.front().filter(|e| e.after == ai.done);
                let last = ai.steps.is_empty() || end.is_some();
                // A move that ends taking an object from beside: the object's node is the move's
                // last, not this step (E24 M4 T4c2: `game-ai` at 660, its draw at 1180).
                let beside = end.is_some_and(|e| !e.taken.is_empty());
                if let Some(&pixels) = self.computer_pixels.get(frame) {
                    self.ai_step = Some((
                        direction,
                        frame.saturating_add(1),
                        t.saturating_add(self.rules.walk.frame_ms),
                    ));
                    let [dx, dy] = direction.delta();
                    self.walk.offset[0] -= pixels * dx;
                    self.walk.offset[1] -= pixels * dy;
                    self.frame_drawn(t);
                    continue;
                }
                self.ai_step = None;
                self.ai_spend(t);
                if last {
                    self.stand();
                    if !beside {
                        self.last_drawn(t);
                    }
                }
                // The step's end's sounds (0x481216 to 0x48141B; E24 M4 T4c): a new terrain at the
                // view's middle asks for its music, its step sound played or queued.
                let middle = self.middle();
                if let (Some(sound), Some(hero)) = (&mut self.sound, self.hero) {
                    sound.step_end(t, middle, hero.at, last, self.walk.frame);
                }
                self.walk.offset = [0, 0];
                // The step's visit of a boat (`visit_object`'s cases 8 and 3; E24 M4 T4c2): the
                // landing's dissolve holds the turn.
                match self.aboard.take() {
                    Some(Aboard::Board(boat)) => self.ai_board(boat),
                    Some(Aboard::Land) => {
                        self.ai_land(t);
                        return Ok(Some(false));
                    }
                    None => {}
                }
                continue;
            }
            // A move's end (E24 M4 T6, R1-3, R1-5): its command's changes, its pickup last.
            if ai.ends.front().is_some_and(|e| e.after <= ai.done) {
                let end = ai.ends.pop_front().unwrap_or_default();
                let seen = ai.judged.map(|_| end.seen);
                self.changed(&end.removed, &end.flags, &end.lost, &end.boats);
                if !end.taken.is_empty() && self.ai_take(t, &end.taken, seen) {
                    return Ok(Some(false));
                }
                continue;
            }
            if let Some((to, unseen, sea)) = ai.steps.pop_front() {
                ai.done += 1;
                if self.computer_step(t, to, unseen, sea) {
                    return Ok(Some(false));
                }
                continue;
            }
            if let Some(walk) = ai.walks.pop_front() {
                let again = core::mem::replace(&mut ai.chosen, true);
                ai.slot = walk.slot;
                ai.ends = walk.ends.into();
                ai.done = 0;
                ai.judged = walk.seen;
                ai.steps = walk
                    .steps
                    .iter()
                    .enumerate()
                    .map(|(i, &h)| {
                        let sea = walk.sea.iter().find(|(s, _)| *s == i).map(|&(_, sea)| sea);
                        (h, walk.unseen.contains(&i), sea)
                    })
                    .collect();
                self.ai_hero(t, walk.hero, again, walk.seen);
                continue;
            }
            // A turn the record ends in (E24 M4 T6, R1-1): the map left in its screen.
            if ai.open {
                return Ok(Some(false));
            }
            let ended = core::mem::take(&mut ai.ended);
            self.ai = None;
            self.ai_end(t, ended)?;
            return Ok(Some(true));
        }
    }

    /// A computer's hero's turn (`ai_hero_turn` 0x5261F0, design §5.2): kind 5's step; the
    /// selected one put back (`adventure_hero_stop`: shown on its tile; `again`, a hero of this
    /// turn before, its redraw's objects' step when due, 0x4177D3, *provisional*: E24 M4 T4c2,
    /// `game-ai`'s second hero chosen at 1180) and this one selected (`hero_select` with 1, 0, 1:
    /// lifted, standing; the view's corner its tile less the middle, never clamped, drawn and
    /// shown when the viewer sees its tile or a neighbour, never under Blackout Computer; no list,
    /// status window or minimap drawn). `seen`: the record's judgement of that tile (E24 M4 T6,
    /// R1-2, [`Steps::seen`]), else the viewer's explored tiles'.
    fn ai_hero(&mut self, t: u32, index: usize, again: bool, seen: Option<bool>) {
        self.ai_status_step(t);
        self.keep_hero();
        self.hidden = false;
        if again {
            self.step_objects(t);
        }
        self.index = Some(index);
        self.hero = self.heroes.get(index).copied();
        self.lifted = true;
        self.stand();
        let seen = seen.unwrap_or_else(|| self.hero.is_some_and(|h| self.sees(h.at)));
        if seen {
            self.centre();
            self.draw(t);
            // `hero_select`'s sounds (0x4180A0; E24 M4 T4c): the terrain's music when it differs.
            let middle = self.middle();
            if let (Some(sound), Some(hero)) = (&mut self.sound, self.hero) {
                sound.select(t, middle, hero.at, true);
            }
        }
    }

    /// A computer's hero's step onto `to` (`ai_hero_step` 0x42FC50, `hero_move_step` 0x4805E0;
    /// design §5.2; `unseen` a record's step whose start it does not give, E24 M4 T4c; `sea` what
    /// it does with a boat, E24 M4 T4c2, design §5.18): kind 5's step (*provisional*: at every
    /// step's start, its 50 ms deciding; which call steps it is not read); the steps the facts do
    /// not show before it (unseen: no time, nothing drawn) bring the hero to its start, their
    /// points the recording's. A step the viewer sees (`step_seen` 0x480000: its start or its end
    /// tile or a neighbour of either explored; never under Blackout Computer) hides the cursor
    /// until the turn's end (`mouse_hide`) and is a human's step (the view centred on the hero's
    /// new tile, never clamped: MCL-Q31's view under the fog as the original's), its frames at
    /// Computer Walk Speed; at speed 4 its one draw kept for the window ([`Loop::take_held`]); its
    /// boarding's start dissolves the hero off the land before them, the step's animation after
    /// it ([`Then::AiBoard`]), its landing's start puts the boat back first, as a human's
    /// ([`Loop::board_start`], [`Loop::land_start`]: the class switch is the same code). One it
    /// does not see moves the hero and takes no time, its boat's visit made at once
    /// ([`Loop::sea_unseen`]: `view_shown` 0, so nothing is drawn). Whether the step holds the
    /// turn (the boarding's dissolve).
    fn computer_step(&mut self, t: u32, to: Hero, unseen: bool, sea: Option<Sea>) -> bool {
        self.ai_status_step(t);
        let number = usize::try_from(to.facing.number()).unwrap_or(0);
        let back = Direction::ALL[number.saturating_add(4) % Direction::ALL.len()];
        let at = self.hero.map(|h| h.at);
        let from = self.next(to.at, back).or(at).unwrap_or(to.at);
        self.hero = self.hero.map(|h| Hero { at: from, ..h });
        self.ai_spend(t);
        // A record's step is one the viewer sees (E24 M4 T6, R1-2: the engine's `step_seen`).
        let judged = self.ai.as_ref().is_some_and(|ai| ai.judged.is_some());
        let seen = !unseen && !self.blackout && (judged || self.sees(from) || self.sees(to.at));
        if !seen {
            // One into sight faces as the hero did (E24 M4 T6, R1-8: MCL-Q44, its way not told).
            let facing = self.hero.map_or(to.facing, |h| h.facing);
            self.hero = Some(Hero {
                facing: if unseen { facing } else { to.facing },
                ..to
            });
            self.sea_unseen(to.at, sea);
            return false;
        }
        self.cursor.shown = false;
        let single = self.computer_pixels.is_empty();
        if !self.lifted {
            // A step after the hero's boarding in its walk (the Admiral's Hat's points; not
            // recorded, *provisional*): `mobilize_hero` 0x480914 lifts it and shows the view,
            // without a human's path.
            self.lifted = true;
            self.hidden = false;
            self.stand();
            self.draw(t);
        }
        match sea {
            Some(Sea::Board(kind)) => {
                if let Some(boat) = self.boarded(to.at, kind) {
                    self.board_start(t, to.facing, boat);
                    self.lose(t, Then::AiBoard(to, single));
                    return true;
                }
            }
            Some(Sea::Land(boat)) => self.land_start(t, to.facing, boat),
            Some(Sea::Unfaced(_)) => self.refuse(to.at, Unplayable::Landing),
            None => {}
        }
        self.ai_animate(t, to, single);
        false
    }

    /// A computer's step's animation onto `to` ([`Loop::animate`]; `single` at speed 4: its one
    /// draw kept for the window), its frames from `t` ([`Loop::ai_on`]).
    fn ai_animate(&mut self, t: u32, to: Hero, single: bool) {
        self.animate(t, to.facing, to.at, single);
        if single {
            let state = self.state(t);
            self.within.push(state);
        }
        self.ai_step = Some((to.facing, 0, t));
    }

    /// A computer's hero's boat boarded at its step's end (`visit_object`'s case 8,
    /// `hero_boards_boat` 0x4A1010; E24 M4 T4c2, design §5.18): the boat taken off its tile, the
    /// hero in it with its type and its owner's colour; its points 0 (the record holds none:
    /// MCL-Q43) and the hero list's slot −1 (the viewer's, nothing selected in a computer's turn:
    /// nothing drawn). For a player not human here (0x4A10C7) the boat's picture is not lifted nor
    /// the view drawn: the hero stays off its tile and not lifted, so that nothing draws it until
    /// it is put back ([`Loop::ai_hero`], [`Loop::ai_end`]: `adventure_hero_stop`), the screen
    /// the animation's last frame (the boat on its tile, no hero; `game-ai-boat` at 1430).
    fn ai_board(&mut self, boat: LiveBoat) {
        self.boats.empty.retain(|b| b.at != boat.at);
        self.sail(Some(Boat {
            kind: boat.kind,
            colour: self.player,
        }));
    }

    /// A computer's hero's land stepped onto from its boat at its step's end (`visit_object`'s case
    /// 3, `hero_lands_from_boat` 0x49E670, no test of the player there; E24 M4 T4c2, design
    /// §5.18): out of its boat, standing, lifted, then [`Loop::lose`]: the hero dissolved in on
    /// the land (`killfade.82M`), its turn going on at its end ([`Then::AiLanded`]); the points and
    /// the slot as [`Loop::ai_board`]'s.
    fn ai_land(&mut self, t: u32) {
        self.sail(None);
        self.stand();
        self.lifted = true;
        self.hidden = false;
        self.lose(t, Then::AiLanded);
    }

    /// A computer's move's last step's end (`hero_move_step` with `ai_hero_step`'s fifth argument
    /// 1, 0x42FDF6, and the view shown: 0x481237 to 0x481264; E24 M4 T4c2): the view drawn and the
    /// objects' step when due (`game-ai-boat`'s landing at 2260). A walk that ends taking an object
    /// from beside has the object's node last, which takes no step: none (`game-ai` at 660).
    fn last_drawn(&mut self, t: u32) {
        self.draw(t);
        self.step_objects(t);
    }

    /// A computer's step's boat visit out of the viewer's sight (E24 M4 T4c2): its boarding or
    /// landing made at once, nothing drawn (`hero_move_step` sets `view_shown` to whether the
    /// viewer sees the step, 0x48071F, and the dissolves and draws test it).
    fn sea_unseen(&mut self, at: Pos, sea: Option<Sea>) {
        match sea {
            Some(Sea::Board(kind)) => {
                if let Some(boat) = self.boarded(at, kind) {
                    self.ai_board(boat);
                }
            }
            Some(Sea::Land(boat)) => {
                self.boats.empty.push(boat);
                self.sail(None);
            }
            Some(Sea::Unfaced(_)) => self.refuse(at, Unplayable::Landing),
            None => {}
        }
    }

    /// The objects `taken` (anchors, indexes) a computer's hero takes from beside at its move's
    /// end (E24 M4 T4c, design §5.16: `visit_resource_pile` 0x4A4BE0 as the viewer's own, §5.3):
    /// the hero turned to the first, standing; where the viewer sees the hero's tile (`seen`: the
    /// record's judgement, E24 M4 T6, R1-2; else the explored tiles') its removal dissolved with
    /// the pickup's sound ([`Loop::take`]: the computer's turn going on at its end), else taken
    /// off at once. Whether a dissolve holds the turn.
    fn ai_take(&mut self, t: u32, taken: &[(Pos, usize)], seen: Option<bool>) -> bool {
        let objects: Vec<usize> = taken.iter().map(|&(_, o)| o).collect();
        // Kind 5's step at the visit (*provisional*: recorded, `game-ai` at 660; its caller not
        // read).
        self.ai_status_step(t);
        let at = self.hero.map(|h| h.at);
        let towards = at
            .zip(taken.first())
            .and_then(|(h, &(a, _))| Direction::between(h, a));
        if let Some(direction) = towards {
            self.turn_to(direction);
        }
        self.stand();
        let seen = !self.blackout && seen.unwrap_or_else(|| at.is_some_and(|a| self.sees(a)));
        let Some((&object, rest)) = objects.split_first().filter(|_| seen) else {
            self.take_off(&objects);
            return false;
        };
        Arc::make_mut(&mut self.drawn.gone).extend(rest);
        self.take_then(t, object, Gain::default(), Fade::Pickup, Then::Ai);
        true
    }

    /// A computer player's turn's end (`ai_player_turn`'s, design §5.2): the last hero put back on
    /// its tile (`adventure_hero_stop` with 0, 1), the cursor shown again (`mouse_show` 0x50D5B0)
    /// as the adventure cursor's arrow (*provisional*: where its type is set back is not read),
    /// the objects stepped once (E24 M4 T4c2: the turn's end's redraw); then the next turn:
    /// another computer's, or the next day (refused in a recording when the facts give none or
    /// its selection is not known).
    fn ai_end(&mut self, t: u32, ended: Ended) -> Result<(), Unplayable> {
        self.lifted = false;
        self.keep_hero();
        self.hidden = false;
        self.set_cursor(CURSOR_TYPE, self.rules.arrow);
        self.cursor.shown = true;
        // The turn's end's redraw (`adventure_hero_stop`'s, 0x4177D3), the objects stepped when
        // due, then the deadline at least a step on from the clock (*provisional*: recorded, not
        // read; E24 M4 T4c2, after the move's last step's own step in [`Loop::ai_on`]: `ai-walk`'s
        // two at 1660, the next at 1840; `game-ai-boat`'s two at 1480, the next at 1660, and its
        // one at 2780 after the landing's dissolve, the next at 2960, not at the late deadline).
        self.step_objects(t);
        self.frame_at = self
            .frame_at
            .max(t.saturating_add(self.rules.frame_ms.max(1)));
        let refused = Unplayable::Command(panels::id::END_TURN);
        let Some(next) = self.turns.get(self.next_turn).cloned() else {
            return self.unbuilt(refused);
        };
        if let Status::Turn(status) = next.turn.status {
            if !self.computer_playable(&next) {
                return self.unbuilt(refused);
            }
            return self.computer_start(t, next, status, ended);
        }
        let Some(choice) = self.turn_choice(&next) else {
            return self.unbuilt(refused);
        };
        self.day_start(t, next, choice, ended);
        Ok(())
    }

    /// The selected hero kept among the map's heroes on its tile and facing (E24 M4 T1b2: a
    /// computer's, put back).
    fn keep_hero(&mut self) {
        if let (Some(index), Some(hero)) = (self.index, self.hero)
            && let Some(h) = self.heroes.get_mut(index)
        {
            *h = hero;
        }
    }

    /// The computer's walking hero's points memory holds at `t` as [`Start::points`] gives them
    /// (the recording's `points` lines follow the current player's selected hero).
    fn ai_spend(&mut self, t: u32) {
        let given = self.points.iter().take_while(|(at, _)| *at <= t).last();
        if let (Some(&(_, points)), Some(ai)) = (given, &mut self.ai)
            && let Some(hero) = ai.kingdom.heroes.get_mut(ai.slot)
        {
            hero.points = points;
        }
    }

    /// Kind 5's step (`status_other_turn_step` 0x4536F0, design §5.2): once its step's time has
    /// passed since the last, the glass and the sand stepped by the computer's heroes' points
    /// left ([`OtherArt::stepped`]), drawn and shown at once.
    fn ai_status_step(&mut self, t: u32) {
        let other = self.rules.other;
        let stepped = match &mut self.ai {
            Some(ai) if t.saturating_sub(ai.stepped) >= other.step => {
                ai.stepped = t;
                let left = ai
                    .kingdom
                    .heroes
                    .iter()
                    .fold(0_i32, |sum, h| sum.saturating_add(h.points));
                ai.status = other.stepped(ai.status, left);
                Some(ai.status)
            }
            _ => None,
        };
        if let Some(status) = stepped {
            self.show_status(Status::Turn(status));
        }
    }

    /// A status window shown at once, memory's and the screen's.
    fn show_status(&mut self, status: Status) {
        self.kingdom.status = Some(status.clone());
        Arc::make_mut(&mut self.drawn.kingdom).status = Some(status);
    }

    /// Whether the viewer sees `at`: the tile or one of its neighbours inside the map explored
    /// (`tile_or_neighbour_seen` 0x41A750); nothing without the mouse-over's facts.
    fn sees(&self, at: Pos) -> bool {
        let [x, y, z] = [i32::from(at.x), i32::from(at.y), i32::from(at.z)];
        self.hover.as_ref().is_some_and(|hover| {
            (-1..=1).any(|dy| (-1..=1).any(|dx| hover.explored([x + dx, y + dy, z])))
        })
    }

    /// The map hero (its index among [`Start::heroes`]) of the kingdom's hero `i`: the one of its
    /// record index by the mouse-over's facts (memory's), else its choice's (a recording's
    /// `[chosen]`, or a map's start's).
    fn map_hero(&self, i: usize) -> Option<usize> {
        let by_facts = self.hover.as_ref().and_then(|hover| {
            let facts = hover.facts();
            let id = facts.hero_list.get(i)?.0;
            facts
                .heroes
                .iter()
                .position(|h| h.0 == id)
                .filter(|&index| index < self.heroes.len())
        });
        by_facts.or_else(|| self.chosen_hero(i).map(|c| c.hero))
    }

    /// A message box opened (`message_box` 0x4F6570; E17 design §7.19): the default cursor's frame 0
    /// at the mouse (`mouse_set_cursor(0, 0)` at 0x4F74AC), the box drawn and shown over the map,
    /// the input queue emptied as it opens; `question` End Turn's (its OK ends the turn), else the
    /// week's (the date's window stepping behind it).
    fn open_box(&mut self, message: Arc<MessageBox>, told: Told) {
        let cursor = [self.cursor.kind, self.cursor.frame];
        self.set_cursor(DEFAULT_CURSOR, 0);
        self.boxed = Some(Boxed {
            told,
            open: Open::new((*message).clone()),
            cursor,
        });
        self.drawn.message = Some(message);
        self.flush = true;
    }

    /// The cursor set (`mouse_set_cursor`): its type and frame, drawn at the mouse now.
    fn set_cursor(&mut self, kind: u32, frame: u32) {
        self.cursor.kind = kind;
        self.cursor.frame = frame;
        self.cursor.at = self.rules.drawn_at(self.pointer);
    }

    /// A message box's loop's turn (0x602520 with `message_box_proc` 0x4F08D0): the week's box
    /// calls `status_panel_update` with 1, the adventure dialog's +0x6C set (0x4CCAE5), so the
    /// date's window steps behind it (§4.9); none while the mouse holds one of its buttons (the
    /// button's own loop), none behind End Turn's question.
    fn box_step(&mut self, t: u32) {
        let steps = self
            .boxed
            .as_ref()
            .is_some_and(|b| b.told == Told::Week && !b.open.holds_mouse());
        if let (true, Some(Status::Date(date))) = (steps, self.kingdom.status.clone())
            && let Some(status) = self.date_window(t, Some(date))
        {
            self.kingdom.status = Some(status.clone());
            Arc::make_mut(&mut self.drawn.kingdom).status = Some(status);
        }
    }

    /// An action while a message box is open (its modal loop; E17 design §7.19): the mouse's
    /// moves, presses and releases and the box's keys go to the box ([`Open`]); a button pressed
    /// is drawn and shown with its sound (`button.wav`: `def_button_message`, as the panels'), a
    /// click closes the box ([`Loop::close_box`]); every other message is dropped (no edge scroll,
    /// no hint: the map stands still). The mouse's point is the latest given.
    fn box_act(&mut self, t: u32, action: Action, fresh: bool) -> Result<(), Unplayable> {
        if fresh
            && let Action::Pointer(at) | Action::Click(at) | Action::Press(at) | Action::Release(at) =
                action
        {
            self.pointer = at;
        }
        // A click a press and a release (a press never closes the box).
        let closed = match action {
            Action::Click(at) => {
                self.box_input(t, Action::Press(at));
                self.box_input(t, Action::Release(at))
            }
            _ => self.box_input(t, action),
        };
        match closed {
            Some(id) => self.close_box(t, id),
            None => Ok(()),
        }
    }

    /// An input's message to the open box ([`Loop::box_act`]): a button pressed or released
    /// drawn and shown, a press with its sound; the id of the button whose click closes it. A
    /// level-up's dialog (E24 M4 T2c, design §5.5) takes its keys 1 and 2 and a press on an
    /// offer's picture or frame as the offer's choice ([`levels::choose`]: the dialog drawn again)
    /// and Escape as its OK's hotkey.
    fn box_input(&mut self, t: u32, action: Action) -> Option<u16> {
        let boxed = self.boxed.as_mut()?;
        let before = pressed(&boxed.open);
        let offers = match &boxed.told {
            Told::Level(leveling) => Some(leveling.open.offers.len()),
            _ => None,
        };
        let message = &mut boxed.open.message;
        let choose = |message: &mut MessageBox, n: usize, offers: usize| {
            if levels::choose(message, n, offers) {
                Outcome::Changed
            } else {
                Outcome::Nothing
            }
        };
        let outcome = match action {
            Action::Pointer(at) => boxed.open.moved(at),
            Action::Press(at) => match offers.zip(levels::offer_at(message, at)) {
                Some((offers, n)) => choose(message, n, offers),
                None => boxed.open.press(at),
            },
            Action::Release(_) | Action::RightRelease => boxed.open.release(),
            Action::BoxKey(Key::Cancel) if offers.is_some() => {
                boxed.open.key(Key::Ok, key_name(Key::Cancel))
            }
            Action::BoxKey(key) => boxed.open.key(key, key_name(key)),
            Action::BoxKeyUp(key) => boxed.open.key_up(key_name(key)),
            Action::Offer(n) => match offers {
                Some(offers) => choose(message, n, offers),
                None => Outcome::Nothing,
            },
            _ => Outcome::Nothing,
        };
        match outcome {
            Outcome::Nothing => None,
            Outcome::Changed => {
                let now = pressed(&boxed.open);
                self.drawn.message = Some(Arc::new(boxed.open.message.clone()));
                if now.is_some() && now != before {
                    self.press_sound(t);
                }
                None
            }
            Outcome::Closed(id) => Some(id),
        }
    }

    /// The box closed by its button `id`'s click: the cursor before it set again, the screen
    /// under it shown as it was (the map stood still); End Turn's question's OK goes on to the
    /// turn's change, anything else to the map's idle step in that millisecond (recorded: its
    /// redraws due since run then).
    fn close_box(&mut self, t: u32, id: u16) -> Result<(), Unplayable> {
        let Some(boxed) = self.boxed.take() else {
            return Ok(());
        };
        self.drawn.message = None;
        // The cursor before it set again where the mouse manager last drew it (the poll moves
        // it: recorded, `level-up-right`'s OK clicked at 2000, the cursor at its old place until
        // 2013; E24 M4 T2c).
        [self.cursor.kind, self.cursor.frame] = boxed.cursor;
        match boxed.told {
            Told::Found(i, k) => return self.found_closed(t, i, k),
            Told::Asked(i, question) => {
                // A choice's OK closes with the picture chosen (E24 M4 T2b2 part B).
                let id = match (id, boxed.open.chosen()) {
                    (boxes::OK, Some(chosen)) => chosen,
                    _ => id,
                };
                return self.answer(t, i, *question, id);
            }
            Told::Answer(i, k, answer) => return self.answered(t, i, k + 1, *answer),
            Told::Level(leveling) => {
                let chosen = levels::chosen(&boxed.open.message, leveling.open.offers.len());
                return self.leveled(t, *leveling, chosen);
            }
            Told::DaysLeft => self.choose(t, true),
            _ => {}
        }
        // A box waiting opens in this millisecond (E24 M4 T2a2).
        if let Some((next, told)) = self.waiting.pop_front() {
            self.open_box(next, told);
            return Ok(());
        }
        match (boxed.told, id) {
            (Told::Question, boxes::YES) => self.turn_change(t),
            (Told::Warning, _) => self.turn_proceed(t),
            _ => {
                self.idle_message(t);
                Ok(())
            }
        }
    }

    /// A find's first visit's box `k` closed (E24 M4 T2a2): the next opens in that millisecond,
    /// what it shows shown first (an event's resources: `find-event`'s gold at 1060); after the
    /// last the object taken (an artifact's: its dissolve, the gain shown at its end), else the
    /// gain shown, then the scenario's test (an event's: its warning a box of its own) or the
    /// idle step.
    fn found_closed(&mut self, t: u32, i: usize, k: usize) -> Result<(), Unplayable> {
        let found = Arc::clone(&self.finds);
        let Some(found) = found.get(i) else {
            self.idle_message(t);
            return Ok(());
        };
        if let Some((next, shows)) = found.first.get(k + 1) {
            self.gain(*shows);
            self.open_box(Arc::clone(next), Told::Found(i, k + 1));
            return Ok(());
        }
        // Its flag raised after its first boxes (E24 M4 T2d1: a mine's, a Lighthouse's).
        self.raise(found.flag);
        // Its question after its first boxes (E24 M4 T2b2).
        if self.ask(t, i) {
            return Ok(());
        }
        if let Some(object) = found.taken {
            self.take(t, object, found.gain, Fade::Pickup);
            return Ok(());
        }
        self.gain(found.gain);
        // The levels its experience reaches (E24 M4 T2c: the Learning Stone's after its box,
        // `level-up` at 1060; an event's after its rewards' boxes, `grant_rewards`).
        if let Some(After::Found(i)) = self.level_up(t, After::Found(i))? {
            return self.found_end(t, i);
        }
        Ok(())
    }

    /// Find `i`'s first visit's end after its gain (and since E24 M4 T2c its level-ups): an
    /// event's hero's slot drawn and the scenario's test (its warning a box of its own), else
    /// (E24 M4 T2d1) the minimap drawn (`visit_and_update` 0x4AAAA0 after the visit: a flag
    /// raised shows there, `flag-mine`'s at 1060); then the idle step.
    fn found_end(&mut self, t: u32, i: usize) -> Result<(), Unplayable> {
        let event = self.finds.get(i).is_some_and(|f| f.event);
        if event {
            self.draw_slot();
        } else {
            self.draw_minimap();
        }
        if event && let Some(warning) = self.town_test()? {
            self.open_box(warning, Told::Plain);
            return Ok(());
        }
        self.idle_message(t);
        Ok(())
    }

    /// The selected hero's levels its experience now reaches (E24 M4 T2c, design §5.5:
    /// `hero_level_up` 0x4DA720, a level at a time), each with its level's dialog
    /// ([`Start::level_ups`]); none while its experience is not known (a hero's other than the
    /// one at 0, *provisional*). A level whose dialog the facts lack is refused in a recording
    /// ([`Unplayable::LevelUp`]) and passed over live.
    fn levels_due(&mut self) -> Result<VecDeque<LevelBox>, Unplayable> {
        // A record's visit's own, due since its experience was shown (E24 M4 T6, R2-3).
        let mut due = core::mem::take(&mut self.due_levels);
        let Some(experience) = self.experience.filter(|_| self.index == self.experienced) else {
            return Ok(due);
        };
        let reached = self.rules.level_up.level_of(experience);
        while self.hero_level < reached {
            self.hero_level += 1;
            let level = self.hero_level;
            match self.level_ups.iter().find(|b| b.level == level) {
                Some(up) => due.push_back(up.clone()),
                None => self.unbuilt(Unplayable::LevelUp(level))?,
            }
        }
        Ok(due)
    }

    /// The level-ups due now shown one after another (E24 M4 T2c), `after` following the last;
    /// `after` given back when none is due.
    fn level_up(&mut self, t: u32, after: After) -> Result<Option<After>, Unplayable> {
        let mut due = self.levels_due()?;
        let Some(open) = due.pop_front() else {
            return Ok(Some(after));
        };
        self.open_level(
            t,
            Leveling {
                open,
                rest: due,
                raised: Gain::default(),
                after,
            },
        );
        Ok(None)
    }

    /// The dialog of `leveling`'s open level built (`hero_level_up` 0x4DA9F2 on): its sound
    /// (`nwherolv.82m`, 0x4DAA26), then the dialog, modal, the mouse shown with the arrow
    /// (`mouse_set_cursor` (0, 1): recorded, `level-up`'s cursor at 1060).
    fn open_level(&mut self, t: u32, leveling: Leveling) {
        if let Some(sound) = &mut self.sound
            && !self.rules.level_up.sound.is_empty()
        {
            sound.effect(t, &self.rules.level_up.sound);
        }
        let dialog = Arc::clone(&leveling.open.dialog);
        self.open_box(dialog, Told::Level(Box::new(leveling)));
        self.set_cursor(CURSOR_TYPE, self.rules.arrow);
        self.cursor.shown = true;
    }

    /// A level-up's dialog closed by its OK with the offer `chosen` (E24 M4 T2c, design §5.5): its
    /// primary skill raised, the offer learned, the next level's dialog built in that millisecond
    /// (recorded: `level-up-twice`'s at 4260, the status window as it was); after the last what
    /// they raised shown (`level-up`'s Defense at 2260, `level-up-twice`'s Knowledge and its
    /// Advanced Leadership's morale at 5260), the mouse looked at again (the dialog's arrow gives
    /// way to the tile's cursor, `level-up`'s at 2260), then what follows the visit's level-ups
    /// ([`After`]). *Provisional:* the offer learned is the engine's (Part B); the viewer shows
    /// only a Leadership's morale and a Luck's luck ([`LevelArt`]).
    fn leveled(
        &mut self,
        t: u32,
        mut leveling: Leveling,
        chosen: Option<usize>,
    ) -> Result<(), Unplayable> {
        // A record's level-up closed with the offer its answer took (E24 M4 T4b1).
        if leveling
            .open
            .chosen
            .is_some_and(|want| chosen != Some(want))
        {
            self.unbuilt(Unplayable::LevelUp(leveling.open.level))?;
        }
        let raised = &mut leveling.raised;
        if let Some(skill) = raised.skills.get_mut(usize::from(leveling.open.primary)) {
            *skill = skill.saturating_add(1);
        }
        let learned = chosen.and_then(|n| leveling.open.offers.get(n));
        let art = &self.rules.level_up;
        if let Some(offer) = learned {
            if art.morale == Some(offer.skill) {
                raised.morale = raised.morale.map(|m| m.saturating_add(1));
            }
            if art.luck == Some(offer.skill) {
                raised.luck = raised.luck.saturating_add(1);
            }
        }
        if let Some(next) = leveling.rest.pop_front() {
            leveling.open = next;
            self.open_level(t, leveling);
            return Ok(());
        }
        self.gain(leveling.raised);
        self.column = None;
        self.moved();
        match leveling.after {
            After::Found(i) => self.found_end(t, i),
            After::Answer(i, answer) => self.answer_end(t, i, *answer),
            After::Step(queued) => {
                self.finish(t, true);
                self.resume(t, queued)
            }
            After::Visited(i) => {
                self.visited(t, i);
                Ok(())
            }
        }
    }

    /// A find on the tile a walk ends on told (E24 M4 T2a2, design §5.3: the step and its box in
    /// one millisecond, recorded): its first boxes, told again its later one; whether one opened.
    /// A find whose object blocks its tile ([`Found::blocks`]) is a key's step's
    /// ([`Loop::visit_beside`]). Since E24 M4 T2b2 a find with a question asks it after its first
    /// boxes at every visit, until its OK is taken; then it tells nothing.
    fn tell_find(&mut self, t: u32) -> bool {
        let Some(at) = self.hero.map(|h| h.at) else {
            return false;
        };
        // A record's find is its walk's end's (E24 M4 T6, R2-1): the walk just ended.
        let walk = self.last_move;
        let Some(i) = self.finds.iter().position(|f| {
            f.at == at && f.blocks.is_none() && f.walk.is_none_or(|w| Some(w) == walk)
        }) else {
            return false;
        };
        self.visiting(i);
        let found = Arc::clone(&self.finds);
        let found = &found[i];
        if self.answered.contains(&i) {
            return false;
        }
        if self.told.contains(&i) && found.ask.is_none() {
            return found
                .again
                .clone()
                .map(|again| self.open_box(again, Told::Plain))
                .is_some();
        }
        if !self.told.contains(&i) {
            self.told.push(i);
        }
        match found.first.first() {
            Some((first, shows)) => {
                self.gain(*shows);
                self.open_box(Arc::clone(first), Told::Found(i, 0));
                true
            }
            None => {
                // Its flag raised before its question (E24 M4 T2d1: a dwelling's).
                self.raise(found.flag);
                if self.ask(t, i) {
                    return true;
                }
                // A record's flag raised alone (E24 M4 T6, the reviewers' R3-4: what the visit
                // shows besides is not built): the minimap drawn after the visit
                // (`visit_and_update` 0x4AAAA0), the view at its next draw.
                if found.flag.is_some() {
                    self.draw_minimap();
                }
                false
            }
        }
    }

    /// Find `i`'s question asked (E24 M4 T2b2 part A1, design §5.4; recorded in the visit's
    /// millisecond, [`Loop::pose`]); whether it has one.
    fn ask(&mut self, t: u32, i: usize) -> bool {
        let finds = Arc::clone(&self.finds);
        let Some(question) = finds.get(i).and_then(|f| f.ask.as_ref()) else {
            return false;
        };
        self.pose(t, i, question.clone());
        true
    }

    /// Find `i`'s `question` asked: the status window built again first when its handler does
    /// (`status_window_choose` 0x415DE0 with 0, 1, 1), then its box (E24 M4 T2b2 part A2: kept as
    /// the one open, a first question's or an answer's next).
    fn pose(&mut self, t: u32, i: usize, question: Question) {
        if question.chooses {
            self.choose(t, true);
        }
        let asked = Arc::clone(&question.asked);
        self.open_box(asked, Told::Asked(i, Box::new(question)));
    }

    /// Find `i`'s question closed by its button `id` (E24 M4 T2b2 part A1): its OK's answer or
    /// its Cancel's shown ([`Loop::answered`]; an OK's marks the find answered), refused where
    /// the answer is not built (live: the visit ended, [`Loop::visited`]; recorded, `ask-pile`'s
    /// and `ask-mine`'s Cancel: the idle step at 1060). Since part B a choice's `id` the picture
    /// chosen: the first's answer [`Question::yes`] (a plain OK's too), the second's
    /// [`Question::second`], its Cancel's [`Question::no`] (marking the find where it
    /// [`Answer::marks`]: the Arena's).
    fn answer(&mut self, t: u32, i: usize, question: Question, id: u16) -> Result<(), Unplayable> {
        let reply = match id {
            boxes::YES | boxes::OK | boxes::CHOICE => question.yes,
            _ if id == boxes::CHOICE + 1 => question.second,
            _ => question.no,
        };
        if let Some(reply) = reply {
            if !matches!(id, boxes::NO | boxes::CANCEL) || reply.marks {
                self.answered.push(i);
            }
            return self.answered(t, i, 0, reply);
        }
        let at = self.finds[i].at;
        self.unbuilt(Unplayable::Answer([
            i32::from(at.x),
            i32::from(at.y),
            i32::from(at.z),
        ]))?;
        self.visited(t, i);
        Ok(())
    }

    /// Find `i`'s answer's box `k` opened, what it shows shown first (E24 M4 T2b2 part A1:
    /// `ask-tomb`'s 162, `ask-hire`'s GENRLTXT 424 in the OK's millisecond); past the last the
    /// answer's gain shown, then (part A2) the status window built again where the answer does,
    /// its next question asked, its object taken ([`Loop::take`]: the step's end after the
    /// dissolve) or its terrains revealed (the view and the minimap drawn again: `visit_object`'s
    /// common end 0x4A9AD3), and the visit ended.
    fn answered(
        &mut self,
        t: u32,
        i: usize,
        k: usize,
        mut answer: Answer,
    ) -> Result<(), Unplayable> {
        // Its battle first (E24 M4 T5b2, design §5.21: a mine's guards'), the hero put back on
        // its tile before it (E24 M5 T1a, design §5.22: the view drawn, then the objects stepped
        // when due, recorded, `battle-mine` at 1060), its boxes after it.
        if let Some(fight) = answer.fight.take() {
            self.battle_stop(t);
            self.switch_out(t, i, fight, Some(Box::new(answer)));
            return Ok(());
        }
        if let Some((next, shows)) = answer.boxes.get(k) {
            self.gain(*shows);
            let next = Arc::clone(next);
            self.open_box(next, Told::Answer(i, k, Box::new(answer)));
            return Ok(());
        }
        self.gain(answer.gain);
        // Its flag past its boxes (E24 M4 T5b2: a guarded mine's, `set_mine_owner` 0x4C66E0
        // after MINEEVNT's box): drawn at the view's next draw.
        self.raise(answer.flag.take());
        // The levels its experience reaches (E24 M4 T2c: Pandora's Box's after its rewards'
        // boxes, `grant_rewards`).
        if let Some(After::Answer(i, answer)) =
            self.level_up(t, After::Answer(i, Box::new(answer)))?
        {
            return self.answer_end(t, i, *answer);
        }
        Ok(())
    }

    /// Find `i`'s answer's end after its gain (and since E24 M4 T2c its level-ups): the status
    /// window built again where the answer does, its next question asked, its object taken
    /// ([`Loop::take`], what [`Answer::after`] gives shown at the dissolve's end) or its terrains
    /// revealed, and the visit ended.
    fn answer_end(&mut self, t: u32, i: usize, answer: Answer) -> Result<(), Unplayable> {
        if answer.chooses {
            self.choose(t, true);
        }
        if let Some(then) = answer.then {
            self.pose(t, i, *then);
            return Ok(());
        }
        if let Some((object, fade)) = answer.taken {
            self.take(t, object, answer.after, fade);
            return Ok(());
        }
        if answer.reveal != 0 {
            self.drawn.revealed |= answer.reveal;
            self.draw(t);
            self.draw_minimap();
        }
        self.visited(t, i);
        Ok(())
    }

    /// Find `i`'s visit ended with nothing more to show (E24 M4 T2b2 part A1): one from beside
    /// ends its step first ([`Loop::finish`], as after a dissolve), one on its entrance looks at
    /// the mouse again as the step's end does (part B: `choose-arena`'s cursor over the view at
    /// 1460, the mouse moved there during the box; *provisional*: the visit's place in the step,
    /// `hero_move_step` 0x4805E0, not read), then the idle step.
    fn visited(&mut self, t: u32, i: usize) {
        self.drawn.attacking = None;
        if self.finds.get(i).is_some_and(|f| f.blocks.is_some()) {
            self.finish(t, true);
        } else {
            // The minimap drawn after the visit (`visit_and_update` 0x4AAAA0; E24 M4 T2d1: a
            // dwelling's flag raised before its hire, `flag-dwelling`'s at 1660).
            self.draw_minimap();
            self.column = None;
            self.moved();
        }
        self.idle_message(t);
    }

    /// The flag `flag` raised (E24 M4 T2d1, design §5.6): the object's owner the colour in memory
    /// ([`Loop::owners`]), nothing drawn: the minimap shows it at the visit's end, the view at its
    /// next draw. A flag already raised changes nothing.
    fn raise(&mut self, flag: Option<(usize, u8)>) {
        let Some((object, colour)) = flag else {
            return;
        };
        if self.owners.contains(&(object, colour)) {
            return;
        }
        let owners = Arc::make_mut(&mut self.owners);
        match owners.iter_mut().find(|(o, _)| *o == object) {
            Some(owner) => owner.1 = colour,
            None => owners.push((object, colour)),
        }
    }

    /// Whether the selected hero stands on an event not told yet (E24 M4 T2a2 part B).
    fn event_due(&self) -> bool {
        let at = self.hero.map(|h| h.at);
        self.finds
            .iter()
            .enumerate()
            .any(|(i, f)| f.event && Some(f.at) == at && !self.told.contains(&i))
    }

    /// The find a key's step onto `at` visits from beside (E24 M4 T2a2 part B): one whose object
    /// blocks its tile (since T2b2 [`Found::blocks`]: a guarded pile's too), not taken yet; its
    /// place among the finds.
    fn beside(&self, at: Pos) -> Option<usize> {
        // A record's find is its walk's (E24 M4 T6, R2-1): the one under way, else the next.
        let walk = self.walking.map_or(self.next_move, |(m, _)| m);
        self.finds.iter().position(|f| {
            f.blocks
                .is_some_and(|o| f.at == at && !self.drawn.gone.contains(&o))
                && f.walk.is_none_or(|w| w == walk)
        })
    }

    /// Find `i`'s visit begun (E24 M4 T6, the reviewer's R2-3): a record's level-ups wait for its
    /// experience ([`Found::levels`]), none due from an earlier visit.
    fn visiting(&mut self, i: usize) {
        self.leveling = Some(i);
        self.due_levels.clear();
    }

    /// A key's step onto a pile or an artifact (E24 M4 T2a2 part B, design §5.3; recorded,
    /// `find-pile`, `find-artifact`; *provisional*: `hero_move_step` 0x4805E0's blocked visit not
    /// read): the mouse hidden, the hero lifted (the view centred on it and drawn) and turned that
    /// way but left on its tile, its points spent, its slot and the minimap drawn; then the visit
    /// (`visit_resource_pile` 0x4A4BE0, `visit_artifact` 0x49F7E0): a pile's resources shown on
    /// the resource bar and its kind 6 asked, chosen and drawn, then the pile taken; an
    /// artifact's box (the mouse shown over it), the artifact taken at its close. The step's end
    /// ([`Loop::finish`]) comes after the dissolve. Since E24 M4 T2b2 a find with a question
    /// asks it (the mouse shown over it; recorded, `ask-pile`, `ask-artifact`), its visit's end
    /// the step's ([`Loop::visited`]).
    fn visit_beside(&mut self, t: u32, direction: Direction, at: Pos, i: usize) {
        self.beside_start(t, direction, at);
        self.told.push(i);
        self.visiting(i);
        let finds = Arc::clone(&self.finds);
        let found = &finds[i];
        if let Some((first, shows)) = found.first.first() {
            self.gain(*shows);
            self.cursor.shown = true;
            self.open_box(Arc::clone(first), Told::Found(i, 0));
            return;
        }
        if let (Some(fight), Some(object), Some(hero)) = (&found.fight, found.blocks, self.hero) {
            // A savage monster fought at once (E24 M4 T5b2, design §5.21): drawn as its attack,
            // the view drawn and shown (as a question's), the hero put back on its tile (E24 M5
            // T1a, design §5.22), then the battle's switch.
            let left = u32::from(hero.at.x < found.at.x);
            self.drawn.attacking = Some(Attacking {
                object,
                frame: fight.attacker + left,
            });
            self.draw(t);
            self.step_objects(t);
            self.battle_stop(t);
            self.switch_out(t, i, fight.clone(), None);
            return;
        }
        if let Some(question) = &found.ask {
            // A monster drawn as its attack (its frame + 1 with the hero left of it), the view
            // drawn and shown (`visit_object` 0x4A97B2 to 0x4A97F4; E24 M4 T2b2 part A2).
            if let (Some(frame), Some(object), Some(hero)) =
                (question.attacker, found.blocks, self.hero)
            {
                let left = u32::from(hero.at.x < found.at.x);
                self.drawn.attacking = Some(Attacking {
                    object,
                    frame: frame + left,
                });
                self.draw(t);
                self.step_objects(t);
            }
            self.cursor.shown = true;
            self.ask(t, i);
            return;
        }
        self.gain(found.gain);
        if let Some(status) = &found.found {
            // Kind 6 asked until the tables' time from now, the chooser at once (1, 1, 1).
            self.found = Some(status.clone());
            self.asked = KIND_FOUND;
            self.end = t.saturating_add(self.rules.finds.found_ms);
            self.choose(t, true);
        }
        match found.taken {
            Some(object) => self.take(t, object, Gain::default(), Fade::Pickup),
            None => self.visited(t, i),
        }
    }

    /// A path's last node onto find `i`'s object, which blocks its tile `next` (E24 M4 T6, the
    /// reviewer's R5-1, *provisional*: `hero_move_step` 0x4805E0's blocked visit not read; design
    /// §5.18, a computer's walk that ends taking an object from beside has the object's node
    /// last, which takes no step): the node's arrow zeroed as a step's start does
    /// ([`Loop::mobilize`], 0x480B30), then the visit as a key's step's ([`Loop::visit_beside`]:
    /// a record's monster fought, a pile or an artifact taken).
    /// Since E24 M4 T6 (*provisional*) a record's node its walk visits from beside with nothing the
    /// map shows ([`Beside::Unshown`]: [`Loop::pass_by`]).
    fn last_node(&mut self, t: u32, direction: Direction, next: Pos, node: Beside) {
        self.path.retain(|(_, a)| a.at != next);
        match node {
            Beside::Find(i) => self.visit_beside(t, direction, next, i),
            Beside::Unshown => self.pass_by(t, direction, next),
        }
    }

    /// What a path's last node onto `next` takes in place of a step (E24 M4 T6): a find's visit
    /// from beside ([`Loop::beside`]), else on a record whose walk stops one short of its path
    /// ([`Move::stops`]) a visit the map does not show; none: a step.
    fn node(&self, next: Pos) -> Option<Beside> {
        if let Some(found) = self.beside(next) {
            return Some(Beside::Find(found));
        }
        self.record_walk()
            .is_some_and(|w| w.stops.is_some_and(|n| n + 1 == w.tiles.len()))
            .then_some(Beside::Unshown)
    }

    /// The record's walk under way, else its next (E24 M4 T6).
    fn record_walk(&self) -> Option<&Move> {
        let m = self.walking.map_or(self.next_move, |(m, _)| m);
        self.moves.as_ref()?.get(m)
    }

    /// A record's path's last node its walk visits from beside with nothing the map shows (E24 M4
    /// T6, *provisional*: a meeting, M5's screen; an object whose visit tells nothing Part A
    /// builds): as a find's visit from beside ([`Loop::visit_beside`]) the hero turned to it, its
    /// points spent, its slot and the minimap drawn; then the visit's end with nothing shown
    /// ([`Loop::visited`]'s for an object that blocks its tile), its events folded.
    fn pass_by(&mut self, t: u32, direction: Direction, next: Pos) {
        self.beside_start(t, direction, next);
        self.finish(t, true);
        self.idle_message(t);
    }

    /// A visit from beside of the object on `at` begun ([`Loop::visit_beside`], [`Loop::pass_by`]):
    /// the mouse hidden; on one of the engine's walks (E24 M4 T4b1, design §5.14) a move onto the
    /// object's tile with no step, its points the move's (since E24 M4 T6, R5-1, a path's last
    /// node too, its walk under way: [`Loop::last_node`]); the hero lifted (the view centred on
    /// it and drawn) and turned `direction` but left on its tile, its points spent, its slot and
    /// the minimap drawn.
    fn beside_start(&mut self, t: u32, direction: Direction, at: Pos) {
        self.cursor.shown = false;
        self.walked = true;
        if self.walking.is_none() {
            self.take_move(at);
        }
        self.step_on_record(at);
        if !self.lifted {
            self.lift(t, true);
            self.draw(t);
        }
        self.hero = self.hero.map(|hero| Hero {
            facing: direction,
            ..hero
        });
        self.stand();
        self.spend(t);
        self.draw_minimap();
        self.draw_slot();
    }

    /// The object `object` taken off the map (`remove_map_object` 0x4AABB0 between the view's
    /// draws of `visit_resource_pile` 0x4A4BE0 and `artifact_pickup_remove` 0x49E170; design
    /// §5.3): the palettes' cycling held, the view drawn and shown with it, the object removed,
    /// the pickup's sound (the viewer's own, [`pickup`]; since E24 M4 T2b2 part A2 by `fade`,
    /// `hero_loss_effect` 0x4ACBB0's: a monster's `killfade.82M`), the mouse hidden, the view
    /// drawn without it and the tables' rectangle dissolved into that in eight steps from now
    /// ([`Loop::dissolve_step`]), `gain` shown at its end.
    fn take(&mut self, t: u32, object: usize, gain: Gain, fade: Fade) {
        self.take_then(t, object, gain, fade, Then::Taken);
    }

    /// [`Loop::take`] with `then` after the dissolve (E24 M4 T4c: a computer's pickup).
    fn take_then(&mut self, t: u32, object: usize, gain: Gain, fade: Fade, then: Then) {
        self.draw(t);
        let before = Arc::new(self.drawn.clone());
        self.drawn.attacking = None;
        let taken = self.drawn.gone.len();
        let gone = Arc::make_mut(&mut self.drawn.gone);
        gone.push(object);
        gone.sort_unstable();
        gone.dedup();
        let name = match fade {
            Fade::Pickup => pickup(&self.rules.finds, taken),
            Fade::Killfade => Some(self.rules.finds.killfade.as_str()),
        };
        if let (Some(sound), Some(name)) = (&mut self.sound, name) {
            sound.effect(t, name);
        }
        self.dissolve(t, before, gain, true, then);
    }

    /// The view's middle dissolved from `before` into the view as last drawn, from `t`
    /// (`screen_dissolve` 0x602DC0, [`Loop::dissolve_step`]): the mouse hidden, the palettes'
    /// cycling held when it `holds`, `gain` shown at its end and `then` after it.
    fn dissolve(&mut self, t: u32, before: Arc<Draw>, gain: Gain, holds: bool, then: Then) {
        self.cursor.shown = false;
        let [x, y, w, h] = self.rules.finds.dissolve.map(i64::from);
        self.dissolve_over(
            t,
            Dissolving {
                start: t,
                before,
                from: None,
                rect: [x, y, x + w, y + h],
                step_ms: self.rules.finds.dissolve_ms,
                queued: Vec::new(),
                gain,
                holds,
                then,
            },
        );
    }

    /// A dissolve under way from `t` ([`Loop::dissolve_step`]): its first step shown at once.
    fn dissolve_over(&mut self, t: u32, dissolving: Dissolving) {
        self.dissolving = Some(dissolving);
        self.dissolve_step(t);
    }

    /// A dissolve's end (E24 M4 T2a2 part B, T2d2): an object taken shows its gain, the levels its
    /// experience reaches, then the key's step's end; a boarding's step goes on (its animation onto
    /// the boat, the actions given meanwhile waiting in the walk's queue); a landing's visit ends
    /// with `visit_and_update`'s redraws, then the walk's end; then the actions given meanwhile.
    fn dissolved(&mut self, t: u32, dissolving: Dissolving) -> Result<(), Unplayable> {
        let Dissolving {
            before,
            from,
            rect,
            queued,
            gain,
            then,
            ..
        } = dissolving;
        match then {
            // A computer's pickup or landing: its turn on in the dissolve's last millisecond.
            Then::Ai | Then::AiLanded => return self.ai_on(t).map(|_| ()),
            // A computer's boarding: its step's animation onto the boat, its turn on.
            Then::AiBoard(to, single) => {
                self.ai_animate(t, to, single);
                return self.ai_on(t).map(|_| ());
            }
            Then::Lost(monster) => self.left_map(t, queued, monster)?,
            Then::Resume => {
                self.cursor.shown = true;
                self.resume(t, queued)?;
            }
            Then::Taken => {
                self.gain(gain);
                // The levels its experience reaches (E24 M4 T2c: a chest's after its removal,
                // `level-up-twice` at 1980), the step's end after them.
                if let Some(After::Step(queued)) = self.level_up(t, After::Step(queued))? {
                    self.finish(t, true);
                    self.resume(t, queued)?;
                }
            }
            Then::Board(boarding) => {
                let Boarding {
                    mut moving,
                    next,
                    single,
                } = *boarding;
                for action in queued {
                    wait_in(&mut moving, action, self.rules.bugs);
                }
                self.animate(t, moving.direction, next, single);
                if let Some(queued) = self.carry_on(t, moving) {
                    self.resume(t, queued)?;
                }
            }
            Then::Landed(key) => {
                self.updated(t);
                self.walk_done(t, key);
                if key {
                    self.stuck(t);
                }
                self.resume(t, queued)?;
            }
            Then::Summoned(cast, boat) => {
                // The new picture copied back, its rectangle shown (E17 design §7.25): the screen
                // the view without the boat there, as shown before elsewhere; the back buffer that
                // view whole.
                let back = Arc::new(self.drawn.clone());
                self.drawn.dissolve = Some(Dissolve {
                    before,
                    from,
                    rect,
                    step: DISSOLVE_DONE,
                });
                self.arrive(t, cast, boat, Some(back), queued);
            }
            Then::Cast(cast) => self.cast_end(t, &cast, queued)?,
        }
        Ok(())
    }

    /// A battle's start at `t` (E24 M5 T1a, design §5.22): `adventure_hero_stop` with 0, 1
    /// (`monster_battle` 0x4AC593, `adventure_battle` 0x4AD54E): a lifted hero put back on its
    /// tile (its standing group, frame 0), the view drawn and shown at once, then the objects
    /// stepped when due (0x4177D3). The original does nothing for a hero not lifted; here every
    /// battle comes from a visit from beside, which lifted it ([`Loop::beside_start`]). The hero
    /// stands on its tile, drawn without pass 9's blend, until the map's return lifts it
    /// ([`Loop::after_battle`]).
    fn battle_stop(&mut self, t: u32) {
        self.lifted = false;
        self.stand();
        self.keep_hero();
        self.draw(t);
        self.step_objects(t);
    }

    /// A battle's switch begun at `t` by find `i`'s visit (E24 M4 T5b2, design §5.19, §5.21,
    /// *provisional*): the busy cursor, the screen as it shows faded out from now
    /// ([`Loop::switch_step`]), the battle listed as handed over by the switch.
    fn switch_out(&mut self, t: u32, i: usize, fight: Fight, rest: Option<Box<Answer>>) {
        self.set_cursor(DEFAULT_CURSOR, self.rules.busy);
        self.cursor.shown = false;
        self.switched.push(fight.battle);
        self.switching = Some(Switching {
            phase: Phase::Out,
            start: t,
            before: Arc::new(self.drawn.clone()),
            find: i,
            fight,
            rest,
        });
    }

    /// A battle's switch's turn at `t` (E24 M4 T5b2, design §5.19, §5.21; recorded, `battle-*`):
    /// the map's fade out's shifts 1 and 2 of the screen as it showed, 50 ms apart, then at the
    /// black the combat cursor and the battle's fade in at once, its shifts 2, 1 and the battle
    /// whole 50 ms apart (the battle handed over then: [`Loop::battles`]); from its result's OK the
    /// battle's fade out likewise, then at the black the map back (the whole screen drawn, the
    /// mouse-over again, the adventure cursor) and faded in likewise; at its end the battle's
    /// result on the map ([`Loop::after_battle`]).
    fn switch_step(&mut self, t: u32) -> Result<(), Unplayable> {
        let Some(switching) = &self.switching else {
            return Ok(());
        };
        let since = t.saturating_sub(switching.start);
        let steps = since / FADE_MS;
        if since == 0 || since % FADE_MS != 0 {
            return Ok(());
        }
        let battle = Some(switching.fight.battle);
        let shift = |s: u32| u8::try_from(s).unwrap_or(FADE_BLACK);
        match switching.phase {
            Phase::Out if steps < 3 => {
                let mut faded = (*switching.before).clone();
                faded.switch = Some(Switch {
                    battle: None,
                    shift: shift(steps),
                    ended: false,
                });
                self.drawn = faded;
            }
            Phase::Out => {
                self.set_cursor(BATTLE_CURSOR, self.rules.battle_cursor);
                self.cursor.shown = true;
                self.drawn.switch = Some(Switch {
                    battle,
                    shift: 2,
                    ended: false,
                });
                self.rephase(t, Phase::In);
            }
            Phase::In if steps < 2 => {
                self.drawn.switch = Some(Switch {
                    battle,
                    shift: shift(2 - steps),
                    ended: false,
                });
            }
            Phase::In => {
                self.drawn.switch = Some(Switch {
                    battle,
                    shift: 0,
                    ended: false,
                });
                let fight = &switching.fight;
                self.whole = Some((t, fight.winner));
                self.battles.push((t, fight.battle));
                self.rephase(t, Phase::Battle);
            }
            Phase::Battle => {}
            Phase::Back if steps < 3 => {
                self.drawn.switch = Some(Switch {
                    battle,
                    shift: shift(steps),
                    ended: true,
                });
            }
            Phase::Back => self.map_back(t),
            Phase::Map if steps < 2 => {
                self.drawn.switch = Some(Switch {
                    battle: None,
                    shift: shift(2 - steps),
                    ended: false,
                });
            }
            Phase::Map => {
                self.drawn.switch = None;
                // The switch under way (always: this is its turn) ends.
                self.switching
                    .take()
                    .map(|switching| self.after_battle(t, switching))
                    .transpose()?;
            }
        }
        Ok(())
    }

    /// The switch's next phase from `t`.
    fn rephase(&mut self, t: u32, phase: Phase) {
        if let Some(switching) = &mut self.switching {
            switching.phase = phase;
            switching.start = t;
        }
    }

    /// The battle's result window's OK at `t` (E24 M4 T5b2): its fade out from now.
    fn switch_back(&mut self, t: u32) {
        let battle = self.switching.as_ref().map(|s| s.fight.battle);
        self.drawn.switch = Some(Switch {
            battle,
            shift: 0,
            ended: true,
        });
        self.rephase(t, Phase::Back);
    }

    /// The map back at the black after the battle (`manager_run` 0x4B0C70, design §5.19): its
    /// palettes' cycling and objects' steps on again (*provisional*: the next cycle 20 ms on, as
    /// recorded), the whole screen drawn with the attack's picture cleared, the adventure cursor
    /// set but the mouse hidden until the fade in's end (E24 M5 T1a, design §5.22: shown after
    /// `manager_run`, 0x4AE30F), the hero on its tile as the battle's start left it, the fade
    /// in's first step at once.
    fn map_back(&mut self, t: u32) {
        self.palette_at = t.saturating_add(BACK_CYCLE_MS);
        self.set_cursor(CURSOR_TYPE, self.rules.arrow);
        self.cursor.shown = false;
        self.draw(t);
        self.drawn.switch = Some(Switch {
            battle: None,
            shift: 2,
            ended: false,
        });
        self.rephase(t, Phase::Map);
    }

    /// A battle's result on the map at the end of its switch at `t` (E24 M4 T5b2, design §5.19's
    /// rest, *provisional*): a beaten monster taken off the map with `killfade.82M` (its dissolve
    /// holding the cycling, the experience gained at its end, the visit's step's end after it).
    fn after_battle(&mut self, t: u32, switching: Switching) -> Result<(), Unplayable> {
        let Switching {
            find, fight, rest, ..
        } = switching;
        // The selected hero's path made again (`mobilize_hero(0, 0, 1)`, design §5.19): none to
        // the tile it fought for, so move hero is disabled (recorded, `battle-won` at 810).
        self.path.clear();
        self.buttons.move_hero = false;
        // The mouse shown after `manager_run` (0x4AE30F) and the hero lifted again, the view
        // drawn (E24 M5 T1a, design §5.22, *provisional*: recorded, every `battle-*` at the fade
        // in's end; `mobilize_hero` lifts it after a won battle, `adventure_battle`'s at its end
        // 0x4AE8F3 (E24 M5 T1c: 0x4AD62E is a battle without a human's), `monster_battle`'s
        // 0x4AC917 then finding it lifted, but a retreated hero shows lifted in
        // `adventure_battle`'s draw before it leaves, 0x4AE6BB, by a lift not read). The battle's visitor is the selected hero ([`Loop::visit_beside`],
        // [`Loop::answered`]).
        self.cursor.shown = true;
        self.lifted = true;
        self.stand();
        self.draw(t);
        // Its sounds when it stays on the map (E24 M5 T1c, MCL-Q52 (a), design §5.24):
        // `adventure_battle`'s end lifts it through `mobilize_hero(0, 0, 1)` (0x4AE8F3, after
        // every outcome), whose `hero_select` restarts the ambient sounds around the view's middle
        // (0x4180FF: each slot stopped and emptied, then found again), so the loops the
        // hand-over's `effects_stop_all` ended play again, and asks for the terrain's music only
        // where it differs from the one asked for last (MCL-Q47 (a): none on the same terrain). A
        // hero that leaves restarts none here: the next choice's lift does ([`Loop::left_map`]).
        // *Provisional:* a level-up's dialogs come first in the original (`hero_level_up`
        // 0x4AE324), so with one the restart waits for their close; here it comes with the lift.
        if !matches!(fight.after, AfterBattle::Lost | AfterBattle::Both) {
            let middle = self.middle();
            if let (Some(sound), Some(hero)) = (&mut self.sound, self.hero) {
                sound.select(t, middle, hero.at, true);
            }
        }
        let gain = Gain {
            experience: fight.experience,
            ..Gain::default()
        };
        let object = self.finds.get(find).and_then(|f| f.blocks);
        match (fight.after, object) {
            // The answer's boxes after the battle, its flag past them (E24 M4 T5b2: a mine's).
            (AfterBattle::Boxes, _) => {
                if let Some(answer) = rest {
                    self.gain(gain);
                    return self.answered(t, find, 0, *answer);
                }
                self.visited(t, find);
            }
            (AfterBattle::Taken, Some(object)) => {
                self.take(t, object, gain, Fade::Killfade);
            }
            (AfterBattle::Lost | AfterBattle::Both, _) => {
                // `battle_aftermath` 0x4AC930's way: the view shown with the hero, its objects
                // stepped (`view_show_step`: recorded, `battle-retreat`'s at 810, the flag's frame
                // the step's), the hero leaving the game (none selected, the lists drawn so), then
                // its dissolve (`killfade.82M`, the palettes cycling on) into the view without it.
                // The view's draw shows the hero lifted, the panel's sleep and cast a spell as its
                // selection left them until the next choice (E24 M5 T1a, design §5.22: recorded,
                // `battle-retreat` 810 to 1265).
                self.step_objects(t);
                // Its tile and facing kept in the map's heroes (as a choice away from it does).
                self.keep_path();
                let (slot, index) = (self.selected_hero(), self.index);
                // The engine's walk under way ends here (E24 M4 T6: the next visit from beside
                // takes the next), the slot left told ([`Loop::slots_left`]).
                self.walking = None;
                self.slots_left.extend(slot);
                self.kingdom.selected = Selected::None;
                self.buttons.left = true;
                // Its slot out of the kingdom's list at once (`hero_leave_game` 0x4D9EC0:
                // recorded, `battle-retreat`'s list at 810).
                let mut i = 0;
                self.kingdom.heroes.retain(|_| {
                    i += 1;
                    Some(i - 1) != slot
                });
                let mut i = 0;
                self.chosen.heroes.retain(|_| {
                    i += 1;
                    Some(i - 1) != slot
                });
                self.draw_lists();
                self.draw(t);
                self.lifted = false;
                self.hero = None;
                self.index = None;
                self.selected_id = None;
                self.left.extend(index);
                // A draw's monster taken off after the hero (E24 M4 T6, R5-4).
                let monster = object.filter(|_| fight.after == AfterBattle::Both);
                self.lose(t, Then::Lost(monster));
            }
            _ => {
                self.gain(gain);
                self.draw(t);
                // Its levels (E24 M4 T6, R2-3: a record's, none dropped), then the visit's end.
                if let Some(After::Visited(find)) = self.level_up(t, After::Visited(find))? {
                    self.visited(t, find);
                }
            }
        }
        Ok(())
    }

    /// A hero's dissolve off the map ended at `t` (E24 M4 T5b2, design §5.19): the lists and the
    /// minimap drawn (`hero_list_update`, `minimap_redraw`), then the player's next hero or town
    /// chosen as `mobilize_hero(0, 0, 1)` does with none selected (*provisional*: the first hero,
    /// else the first town), the actions given meanwhile, the idle step; since E24 M4 T6 (R5-4,
    /// a draw) `monster` taken off the map first if any (`monster_fight`'s rest after
    /// `battle_aftermath`: `killfade.82M`, the actions given meanwhile after its dissolve).
    fn left_map(
        &mut self,
        t: u32,
        queued: Vec<Action>,
        monster: Option<usize>,
    ) -> Result<(), Unplayable> {
        self.draw_lists();
        self.draw_minimap();
        // The next choice draws sleep and cast a spell again ([`Buttons::left`]).
        self.buttons.left = false;
        if self.kingdom.heroes.is_empty() {
            if let Some(chosen) = self.chosen.towns.first().cloned().flatten() {
                self.choose_town(t, 0, chosen);
            }
        } else {
            self.choose_slot(t, 0)?;
        }
        if let Some(object) = monster {
            self.take_then(t, object, Gain::default(), Fade::Killfade, Then::Resume);
            self.dissolving = self.dissolving.take().map(|d| Dissolving { queued, ..d });
            return Ok(());
        }
        self.cursor.shown = true;
        self.resume(t, queued)
    }

    /// `hero_loss_effect` 0x4ACBB0 with 0 (E24 M4 T2d2, design §5.7: a boat boarded or left):
    /// `killfade.82M`, the mouse hidden, the view's middle as the screen last showed it dissolved
    /// into the view drawn now, the palettes cycling on; `then` at its end. The view's draw
    /// chooses the status window, shown at once (E24 M4 T4b2: the date's frame stepped).
    fn lose(&mut self, t: u32, then: Then) {
        let before = self.drawn_over(t);
        if let Some(sound) = &mut self.sound {
            sound.effect(t, &self.rules.finds.killfade);
        }
        self.dissolve(t, before, Gain::default(), false, then);
    }

    /// The view drawn for a dissolve, and the screen it dissolves from: the screen as last shown
    /// with the status window that draw chose. The draw (`view_draw_here` 0x40F870, as
    /// `hero_loss_effect` 0x4ACBB0 and `summon_boat_cast` 0x41C8A0 call it) runs
    /// `adventure_view_draw`'s `status_window_choose` 0x40F861, and `status_date_update` 0x4511A0
    /// shows the date's step at once, outside the box dissolved (E24 M4 T4b2: `game-boats`'
    /// landing at 4250; since E24 M4 T6, the reviewer's R3-5, Summon Boat's two dissolves too).
    fn drawn_over(&mut self, t: u32) -> Arc<Draw> {
        let mut before = self.drawn.clone();
        self.draw(t);
        if before.kingdom.status != self.drawn.kingdom.status {
            Arc::make_mut(&mut before.kingdom)
                .status
                .clone_from(&self.drawn.kingdom.status);
        }
        Arc::new(before)
    }

    /// A dissolve's turn at `t` (`screen_dissolve` 0x602DC0; E17 design §7.25): step k shown at
    /// the start plus k times its step's time (0 the picture before, 7 the last before the
    /// view without the object); eight steps on, the view without it shown, the mouse shown, the
    /// step's end ([`Loop::finish`]) and the actions given meanwhile, then the idle step. Whether
    /// the dissolve is over.
    fn dissolve_step(&mut self, t: u32) -> bool {
        let Some(dissolving) = &self.dissolving else {
            return true;
        };
        let since = t.saturating_sub(dissolving.start);
        let step_ms = dissolving.step_ms.max(1);
        if since >= step_ms.saturating_mul(DISSOLVE_STEPS) {
            self.drawn.dissolve = None;
            return true;
        }
        if since % step_ms == 0 {
            self.drawn.dissolve = Some(Dissolve {
                before: Arc::clone(&dissolving.before),
                from: dissolving.from.clone(),
                rect: dissolving.rect,
                step: u8::try_from(since / step_ms).unwrap_or(0),
            });
        }
        false
    }

    /// A find's gain shown (E24 M4 T2a2, *provisional*: the engine's in time): the kingdom's
    /// resources, the resource bar drawn; the selected hero's primary skills, morale (−3 to 3; the
    /// week's seventh day's on that day) and luck (−3 to 3) in its windows and its army's new
    /// creatures (each the slot of its kind, else the first empty one; none with no room), its
    /// spell points in the kingdom's list, the window shown drawn; since E24 M4 T2c its
    /// experience, which nothing shows (its levels: [`Loop::levels_due`]).
    fn gain(&mut self, gain: Gain) {
        // A record's visit's level-ups due with its experience (E24 M4 T6, the reviewer's R2-3:
        // the record's, any hero's).
        if gain.experience != 0
            && let Some(i) = self.leveling.take()
            && let Some(found) = self.finds.get(i)
        {
            self.due_levels.extend(found.levels.iter().cloned());
        }
        // The experience (E24 M4 T2c): the selected hero's when it is the one known.
        if gain.experience != 0 && self.index == self.experienced {
            self.experience = self
                .experience
                .map(|e| e.saturating_add_signed(gain.experience));
        }
        if gain.resources != [0; 7] {
            for (have, more) in self.kingdom.resources.iter_mut().zip(gain.resources) {
                *have = have.saturating_add(more);
            }
            Arc::make_mut(&mut self.drawn.kingdom).resources = self.kingdom.resources;
        }
        if gain.mana != 0
            && let Some(i) = self.selected_hero()
            && let Some(hero) = self.kingdom.heroes.get_mut(i)
        {
            hero.mana = hero.mana.saturating_add(gain.mana);
            let mana = hero.mana;
            if let Some(drawn) = Arc::make_mut(&mut self.drawn.kingdom).heroes.get_mut(i) {
                drawn.mana = mana;
            }
        }
        let morale = gain.morale[usize::from(self.kingdom.date.day == 7)];
        let army = gain.army.iter().flatten().count() > 0;
        if gain.skills == [0; 4] && morale == 0 && gain.luck == 0 && !army {
            return;
        }
        let raise = |hero: &mut HeroStatus| {
            for (skill, more) in hero.skills.iter_mut().zip(gain.skills) {
                *skill = skill.saturating_add(more);
            }
            hero.morale = hero.morale.saturating_add(morale).clamp(-3, 3);
            hero.luck = hero.luck.saturating_add(gain.luck).clamp(-3, 3);
            for stack in gain.army.iter().flatten() {
                joined(&mut hero.army, *stack);
            }
        };
        let kingdom = &mut self.kingdom;
        for status in [&mut kingdom.status, &mut kingdom.selection] {
            if let Some(Status::Hero(hero)) = status {
                raise(hero);
            }
        }
        if let Some(Some(chosen)) = self
            .selected_hero()
            .and_then(|i| self.chosen.heroes.get_mut(i))
        {
            raise(&mut chosen.status);
        }
        Arc::make_mut(&mut self.drawn.kingdom)
            .status
            .clone_from(&self.kingdom.status);
    }

    /// The cast button's command (`adventure_cast_open` 0x41C2F0; E24 M4 T2d3, design §5.8,
    /// *provisional*): the selected hero lifted and centred, the whole screen shown when it was
    /// not (`mobilize_hero` with 0, 0, 1: its lists' redraw leaving the status bar empty,
    /// MCL-Q22), the view drawn and shown with the objects' step when due (`view_show_step`), the
    /// adventure cursor; then the spell book, modal ([`Loop::book_act`]; its screens are M5's).
    /// Not built past the casts a recording gives ([`Start::casts`]); live (the window) nothing.
    fn cast_open(&mut self, t: u32) -> Result<(), Unplayable> {
        if self.live || self.next_cast >= self.casts.len() {
            return self.unbuilt(Unplayable::Command(panels::id::CAST));
        }
        if !self.lifted {
            self.lift(t, true);
        }
        self.draw(t);
        self.step_objects(t);
        self.set_cursor(CURSOR_TYPE, self.rules.arrow);
        self.book = true;
        Ok(())
    }

    /// An action while the spell book is open (`spell_book_proc` 0x59D429 with the adventure
    /// bookmark; E24 M4 T2d3, *provisional*: the book is M5's): a click on a spell's place closes
    /// it with that spell and casts it, the next of [`Start::casts`], which must be given at the
    /// click's millisecond; every other input is dropped, the mouse's point read.
    fn book_act(&mut self, t: u32, action: Action, fresh: bool) -> Result<(), Unplayable> {
        if fresh && let Some(at) = action.point() {
            self.pointer = at;
        }
        if !matches!(action, Action::Click(_)) {
            return Ok(());
        }
        let cast = self
            .casts
            .get(self.next_cast)
            .filter(|c| c.at == t)
            .cloned()
            .ok_or(Unplayable::Cast(t))?;
        self.next_cast += 1;
        self.book = false;
        // The book's last cursor stays until the cast's end: the default one's frame over its
        // place, where the mouse manager last drew it.
        [self.cursor.kind, self.cursor.frame] = [DEFAULT_CURSOR, self.rules.book];
        self.summon(t, cast)
    }

    /// Summon Boat cast (`adventure_spell` 0x41C490's case 0, `summon_boat_cast` 0x41C8A0; design
    /// §5.8, *provisional*: its boat the facts', its chance and refusals the engine's from Part B):
    /// the boat moved taken off its old tile; when that tile lies in the view on its level, its box
    /// (the tables', [`Loop::boat_box`]) saved, the view drawn without it (the status window it
    /// chooses shown at once, [`Loop::drawn_over`]) and dissolved into it in eight steps, the
    /// palettes cycling on; then its arrival ([`Loop::arrive`]). Only Summon Boat with one boat
    /// moved or made is built.
    fn summon(&mut self, t: u32, cast: Cast) -> Result<(), Unplayable> {
        if cast.spell != SUMMON_BOAT {
            return Err(Unplayable::Spell(cast.spell));
        }
        let &[boat] = cast.boats.as_slice() else {
            return Err(Unplayable::Spell(cast.spell));
        };
        let cast = Box::new(cast);
        if let Some(from) = boat.from {
            self.boats.empty.retain(|b| b.at != from);
            if self.in_view(from) {
                let before = self.drawn_over(t);
                let then = Then::Summoned(cast, boat);
                let dissolving = self.cast_dissolve(t, before, None, from, then);
                self.dissolve_over(t, dissolving);
                return Ok(());
            }
        }
        self.arrive(t, cast, boat, None, Vec::new());
        Ok(())
    }

    /// Summon Boat's boat put on its new tile, its type and facing the facts' (a boat made, type
    /// 0 facing right, likewise), the spell's sound (the spell table's +0x4), the new tile's box
    /// saved (from the back buffer: `back`, the view a first dissolve drew, else the screen), the
    /// view drawn with the boat (as [`Loop::summon`]'s) and dissolved into it; the cast's end at
    /// the dissolve's
    /// ([`Loop::cast_end`]), the actions `queued` and those given meanwhile after it.
    fn arrive(
        &mut self,
        t: u32,
        cast: Box<Cast>,
        boat: Summoned,
        back: Option<Arc<Draw>>,
        queued: Vec<Action>,
    ) {
        self.boats.empty.push(boat.to);
        if let Some(sound) = &mut self.sound {
            sound.effect(t, &self.rules.cast.summon_boat);
        }
        let before = self.drawn_over(t);
        let mut dissolving = self.cast_dissolve(t, before, back, boat.to.at, Then::Cast(cast));
        dissolving.queued = queued;
        self.dissolve_over(t, dissolving);
    }

    /// A Summon Boat's dissolve from `t` over the box about the boat's tile `at`, from `before`
    /// (and `from`, [`Dissolve::from`]), at the tables' step, the palettes cycling on and the
    /// mouse shown (*provisional*); `then` at its end.
    fn cast_dissolve(
        &self,
        t: u32,
        before: Arc<Draw>,
        from: Option<Arc<Draw>>,
        at: Pos,
        then: Then,
    ) -> Dissolving {
        Dissolving {
            start: t,
            before,
            from,
            rect: self.boat_box(at),
            step_ms: self.rules.cast.dissolve_ms,
            queued: Vec::new(),
            gain: Gain::default(),
            holds: false,
            then,
        }
    }

    /// Whether the tile `at` lies in the view's columns and rows on its level (the tables' 19 ×
    /// 17 from the corner).
    fn in_view(&self, at: Pos) -> bool {
        let [columns, rows] = self.rules.cast.view;
        let (dx, dy) = (i32::from(at.x) - self.view.x, i32::from(at.y) - self.view.y);
        at.z == self.view.z && (0..columns).contains(&dx) && (0..rows).contains(&dy)
    }

    /// The box about the boat's tile `at` dissolved (`summon_boat_cast`; the tables' `boat_box`
    /// in tiles from it and `clip`, both ends inclusive): ((dx − 1) · 32, (dy − 1) · 32) to ((dx +
    /// 3) · 32, (dy + 2) · 32), dx and dy from the view's corner, cut to (8, 8) to (615, 551); as
    /// left, top, right, bottom, the last two exclusive.
    fn boat_box(&self, at: Pos) -> [i64; 4] {
        let [left, top, right, bottom] = self.rules.cast.boat_box.map(i64::from);
        let [cl, ct, cr, cb] = self.rules.cast.clip.map(i64::from);
        let side = i64::from(self.rules.tile);
        let (dx, dy) = (
            i64::from(at.x) - i64::from(self.view.x),
            i64::from(at.y) - i64::from(self.view.y),
        );
        [
            ((dx + left) * side).max(cl),
            ((dy + top) * side).max(ct),
            ((dx + right) * side).min(cr) + 1,
            ((dy + bottom) * side).min(cb) + 1,
        ]
    }

    /// Summon Boat's end at its arrival's dissolve's (design §5.8): the view shown with the
    /// objects' step when due (`view_show_step`), the cost spent (`hero_spend_mana` 0x4D92D0:
    /// the facts' mana, at their millisecond, else refused; the hero list's slot drawn,
    /// [`Loop::draw_slot`], MCL-Q14), the status window built again with it (the opening's
    /// `status_window_choose` with 1, 1, 1), the adventure cursor; then the actions given
    /// meanwhile and the loop's turn, whose redraw has been due since the book opened.
    fn cast_end(&mut self, t: u32, cast: &Cast, queued: Vec<Action>) -> Result<(), Unplayable> {
        if cast.spent != Some(t) {
            return Err(Unplayable::Cast(t));
        }
        self.step_objects(t);
        if let Some(i) = self.selected_hero()
            && let Some(hero) = self.kingdom.heroes.get_mut(i)
        {
            hero.mana = cast.mana;
        }
        self.draw_slot();
        // The window built with the hero's mana now (MCL-Q14: the slot may show another).
        let kingdom = &mut self.kingdom;
        for status in [&mut kingdom.status, &mut kingdom.selection] {
            if let Some(Status::Hero(hero)) = status {
                hero.mana = Some(cast.mana);
            }
        }
        if let Some(Some(chosen)) = self
            .selected_hero()
            .and_then(|i| self.chosen.heroes.get_mut(i))
        {
            chosen.status.mana = Some(cast.mana);
        }
        Arc::make_mut(&mut self.drawn.kingdom)
            .status
            .clone_from(&self.kingdom.status);
        self.choose(t, true);
        self.set_cursor(CURSOR_TYPE, self.rules.arrow);
        self.resume(t, queued)
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
        self.dragging = Some(at);
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
    /// at once); no hint, no cursor change, no edge scroll. The message's own point decides
    /// (0x40A3C7: `[ebp - 0x28]`, its copy's), the pointer set only by a `fresh` one (F4-3).
    fn drag(&mut self, t: u32, action: Action, fresh: bool) {
        let (Action::Pointer(at) | Action::Release(at)) = action else {
            return;
        };
        let moved = self.dragging != Some(at);
        self.dragging = Some(at);
        if fresh {
            self.pointer = at;
        }
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
            self.dragging = None;
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
    /// found nothing. A hero found whose choice the live part does not give is refused in a
    /// recording, as its slot's click (E24 M3 T7's review, F4-6).
    fn next_hero(&mut self, t: u32) -> Result<(), Unplayable> {
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
        let Some(i) = next else {
            return Ok(());
        };
        let Some(chosen) = self.chosen_hero(i).cloned() else {
            return self.unbuilt(Unplayable::Command(panels::id::NEXT_HERO));
        };
        self.choose_hero(t, i, chosen);
        Ok(())
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
        self.selected_id = self.record_index(chosen.hero);
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
        // With the hot seat's host the reach is the engine's where the hero stands (E24 M5 T4a),
        // asked of it first: the move waits.
        let costs = match self.hosted_reach(self.now) {
            Reached::Asked => return,
            Reached::Given(reach) => Some(reach),
            Reached::Unhosted => None,
        };
        let selection = self.selection();
        let middle = column == self.rules.middle;
        self.show_hint(hover.hint(tile, middle, selection));
        let cursor = match &costs {
            Some(reach) => hover.cursor_by(tile, selection, Some(&reach.costs)),
            None => hover.cursor(tile, selection, !self.walked),
        };
        self.cursor.frame = cursor.frame;
        self.action = cursor.action;
        // After a walk the reach is the engine's: the recording's frame at this millisecond, if
        // any (E24 M4 T2b2 part B, *provisional*; a click there is not built).
        if self.walked
            && cursor.frame == hover::frame::ARROW
            && let Some(&(_, frame)) = self.reached.iter().find(|&&(at, _)| at == self.now)
        {
            self.cursor.frame = frame;
            self.action = -1;
        }
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
        self.rehint = false;
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
            lifted: self.lifted || self.hidden,
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
        // Live with the hot seat's host (E24 M5 T4a): the engine's step asked first, nothing if it
        // refuses it (`hero_step_allowed`'s place).
        if self.hosted_walk(t, vec![next]) {
            return;
        }
        // Move hero (design §4.1): disabled at the step's start (0x408F0B), enabled again when the
        // step lifts a hero not lifted, whose selection makes the step's own path (`hero_select`
        // through `mobilize_hero`, `make_hero_path` with its third argument; *provisional*: as
        // the recordings show it).
        self.buttons.move_hero = !self.lifted;
        if !self.path.is_empty() {
            self.path.clear();
            self.draw(t);
            self.step_objects(t);
        }
        // A pile or an artifact visited from beside (E24 M4 T2a2 part B); since E24 M5 T4a a
        // record's visit from beside the map does not show too ([`Loop::node`]).
        if let Some(node) = self.node(next) {
            self.last_node(t, direction, next, node);
            return;
        }
        self.walk_off(t, true, direction, next, VecDeque::new());
    }

    /// A click on the view (the press on its area, E24 M3 T6b1): on the path's destination the
    /// hero walks the path, on another tile the path there is shown (`make_hero_path` 0x418DD0,
    /// which draws the view). Live, a click on a tile no path leads to is only the pointer's
    /// move. With the hot seat's host (E24 M5 T4a) the paths are the engine's reach where the
    /// hero stands, asked of it first.
    fn click(&mut self, t: u32, at: [i32; 2]) -> Result<(), Unplayable> {
        let tile = self.rules.tile_at(self.view, at);
        let pos = |a: &Arrow| [i32::from(a.at.x), i32::from(a.at.y), i32::from(a.at.z)];
        if self.path.last().is_some_and(|(_, a)| pos(a) == tile) {
            self.walk_path(t);
            return Ok(());
        }
        let reach = match self.hosted_reach(t) {
            Reached::Asked => return Ok(()),
            Reached::Given(reach) => Some(reach),
            Reached::Unhosted => None,
        };
        let paths = reach.as_ref().map_or(&self.paths, |r| &r.paths);
        let path = self.hero.and_then(|hero| {
            paths
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

    /// The selected hero's reach where it stands now, with a host (E24 M5 T4a, `crate::reach`):
    /// the host's, asked at `t` when it gave none for the stand yet.
    fn hosted_reach(&mut self, t: u32) -> Reached {
        let (Some(host), Some(hero)) = (&self.host, self.index) else {
            return Reached::Unhosted;
        };
        let stand = self.stand_of(hero);
        if let Some(reach) = host.reach(stand) {
            return Reached::Given(Arc::clone(reach));
        }
        self.reach_asked.get_or_insert((t, stand));
        Reached::Asked
    }

    /// The selected hero `hero`'s stand now (E24 M5 T4a): the walks and turns taken so far.
    fn stand_of(&self, hero: usize) -> Stand {
        Stand {
            hero,
            walks: self.next_move,
            turns: self.next_turn,
        }
    }

    /// Whether the walk along `tiles` at `t` is the host's to answer (E24 M5 T4a): with one, a
    /// walk it refused from the same stand is nothing (played again, before a later walk it gave
    /// too; a held key's repeats against a wall), one past the engine's known walks is asked of
    /// it; the action changes nothing until it answers.
    fn hosted_walk(&mut self, t: u32, tiles: Vec<Pos>) -> bool {
        let Some(host) = self.host.as_ref().filter(|_| !tiles.is_empty()) else {
            return false;
        };
        let stand = self.stand_of(self.index.unwrap_or_default());
        if host.refuses(stand, &tiles) {
            return true;
        }
        if self
            .moves
            .as_ref()
            .is_some_and(|m| self.next_move < m.len())
        {
            return false;
        }
        self.walk_asked.get_or_insert(WalkAsked {
            at: t,
            stand,
            tiles,
        });
        true
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
        // Live with the hot seat's host (E24 M5 T4a): the engine's walk asked first.
        let tiles = self.path_steps().iter().map(|&(_, at)| at).collect();
        if self.hosted_walk(t, tiles) {
            return;
        }
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
        let mut steps = self.path_steps();
        // A record's walk stopped on the way (E24 M4 T6, [`Move::stops`]): the record's steps.
        if let Some(n) = self
            .record_walk()
            .and_then(|w| w.stops)
            .filter(|&n| n + 1 < steps.len())
        {
            steps.truncate(n.max(1));
        }
        match steps.pop_front() {
            Some((direction, next)) => self.walk_off(t, false, direction, next, steps),
            None => self.finish(t, false),
        }
    }

    /// The path's steps a walk takes: onto its green arrows, in order.
    fn path_steps(&self) -> VecDeque<(Direction, Pos)> {
        self.path
            .iter()
            .take_while(|(_, a)| a.arrow <= GREEN)
            .map(|&(d, a)| (d, a.at))
            .collect()
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
        // A path's one step onto an object visited from beside: no step (E24 M4 T6, the
        // reviewer's R5-1; [`Loop::last_node`]).
        if !key
            && rest.is_empty()
            && let Some(node) = self.node(next)
        {
            self.last_node(t, direction, next, node);
            return;
        }
        self.cursor.shown = false;
        self.walked = true;
        self.take_move(next);
        let moving = Moving {
            key,
            direction,
            last: rest.is_empty(),
            rest,
            frame: 0,
            until: t,
            queued: Vec::new(),
        };
        if let Some(moving) = self.stride(t, moving, next) {
            self.carry_on(t, moving);
        }
    }

    /// A human's step onto `next` (E24 M4 T2d2, design §5.7): its start ([`Loop::mobilize`]), then
    /// by the tile's class (`hero_move_step`'s switch, 0x480D05): onto an empty boat the
    /// boarding's start, the walk held by its dissolve (none returned); from a boat onto land the
    /// landing's start; then the animation ([`Loop::animate`]). On a record (E24 M4 T4b2, design
    /// §5.15) the record's boarding or landing ([`Move::sea`]) in place of the boats and the water.
    fn stride(&mut self, t: u32, moving: Moving, next: Pos) -> Option<Moving> {
        let (direction, single) = (moving.direction, self.pixels.is_empty());
        self.mobilize(t, direction, next);
        self.step_on_record(next);
        let record = self.sea_on_record(next);
        let boat = match record {
            None => self.boat_on(next),
            Some(Some(Sea::Board(kind))) => self.boarded(next, kind),
            Some(_) => None,
        };
        if let Some(boat) = boat {
            self.board_start(t, direction, boat);
            let boarding = Boarding {
                moving,
                next,
                single,
            };
            self.lose(t, Then::Board(Box::new(boarding)));
            return None;
        }
        let landing = match record {
            None => self.landing(next, direction),
            Some(Some(Sea::Land(boat))) => Some(boat),
            Some(Some(Sea::Unfaced(_))) => {
                self.refuse(next, Unplayable::Landing);
                None
            }
            Some(_) => None,
        };
        if let Some(boat) = landing {
            self.land_start(t, direction, boat);
        }
        self.animate(t, direction, next, single);
        Some(moving)
    }

    /// A step's start before its tile's class (`hero_move_step` 0x4805E0): a hero not lifted (after
    /// a scroll) lifted, the view centred on it, drawn and shown (`mobilize_hero` with 0, 0, 1,
    /// 0x480914), its selection's path (`make_hero_path` from `hero_select`: a key's step's own,
    /// one arrow on `next`, the destination's, *provisional*: a step the points pay; E24 M4 T2d2,
    /// `board`'s screen at 100 shows it on the boat); the next tile's arrow zeroed (0x480B30: the
    /// next draw shows it gone).
    fn mobilize(&mut self, t: u32, direction: Direction, next: Pos) {
        if !self.lifted {
            self.lift(t, true);
            self.path = vec![(
                direction,
                Arrow {
                    at: next,
                    arrow: DESTINATION,
                },
            )];
            self.draw(t);
        }
        self.path.retain(|(_, a)| a.at != next);
    }

    /// A step's animation (`hero_walk_animation` 0x480380): the offset 0, the walking group, the
    /// facing, the hero on the next tile at once and the view centred on it (0x48041D, never
    /// clamped); at speed 4 (`single`: the hero speed's or, E24 M4 T1b2, Computer Walk Speed's)
    /// the view drawn once (0x480483), else the offset a tile back (`walk_start_offsets`
    /// 0x63D6F0) for the frames.
    fn animate(&mut self, t: u32, direction: Direction, next: Pos, single: bool) {
        let n = usize::try_from(direction.number()).unwrap_or(0);
        self.walk.offset = [0, 0];
        self.walk.group = self.rules.walk.groups.get(n).copied().unwrap_or(0);
        self.hero = self.hero.map(|_| Hero {
            at: next,
            facing: direction,
        });
        self.centre();
        if single {
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
            // the path's 0x407EEF, a key's 0x409069): after an event's boxes on the last tile
            // (E24 M4 T2a2 part B, [`Found::event`]).
            if !(moving.last && self.event_due()) {
                self.draw_slot();
            }
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
                // A landing's dissolve (E24 M4 T2d2): the actions waiting wait for its end.
                if let Some(dissolving) = &mut self.dissolving {
                    dissolving.queued.append(&mut moving.queued);
                    return None;
                }
                // A find on its last tile told (E24 M4 T2a2): its box empties the queue.
                if self.tell_find(t) {
                    moving.queued.clear();
                }
                return Some(moving.queued);
            };
            // The path's last node onto an object visited from beside: no step (E24 M4 T6, the
            // reviewer's R5-1); the input waiting behind its dissolve, emptied by its box
            // (`input_flush`), dropped by its switch.
            if !moving.key
                && moving.rest.is_empty()
                && let Some(node) = self.node(next)
            {
                self.last_node(t, direction, next, node);
                if let Some(dissolving) = &mut self.dissolving {
                    dissolving.queued.append(&mut moving.queued);
                    return None;
                }
                return self.boxed.is_some().then(Vec::new);
            }
            moving.direction = direction;
            moving.last = moving.rest.is_empty();
            moving.frame = 0;
            moving = self.stride(t, moving, next)?;
        }
    }

    /// The walk's end: `hero_walk_end` 0x47F7D0 (the standing group, frame 0; for a walk a
    /// message ended at 0x408002, before the path's redraw: M2 T6a, the reviewer's R1-4); the
    /// path's walk clears the path once the hero stands on its destination, else shows it again
    /// for the rest and draws the view (`make_hero_path` 0x418DD0 at 0x4080C9, with the option
    /// Show Route on, 0x698774; M2: the arrows left, as `walk-points` recorded; the engine's path
    /// from M4); then the boat's visit, if any (E24 M4 T2d2: `visit_and_update` after
    /// `hero_walk_end`, the path's 0x4080E9, a key's 0x4090A9; a landing's ends after its
    /// dissolve), and [`Loop::walk_done`].
    fn finish(&mut self, t: u32, key: bool) {
        // One of the engine's walks ends whole (E24 M4 T4a2): a step it does not take is off the
        // record (a walk a message ended).
        if let (Some(moves), Some((m, k))) = (&self.moves, self.walking.take()) {
            // The battle the walk began handed over at its end (E24 M4 T5b1, design §5.20).
            if let Some(n) = moves
                .get(m)
                .and_then(|w| w.battle)
                .filter(|n| !self.switched.contains(n))
            {
                self.battles.push((t, n));
            }
            // A walk the record stops on the way ends there (E24 M4 T6, [`Move::stops`]).
            if let Some(&left) = moves
                .get(m)
                .filter(|w| w.stops != Some(k))
                .and_then(|w| w.tiles.get(k))
            {
                self.off(left);
            }
        }
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
            // green arrows. With none left the points did not pay the next step: the hero cannot
            // move on (*provisional*: the engine's from M4), and next hero follows the kingdom
            // (0x408193 to 0x4081C4; design §4.1). The points drawn are the recording's, as the
            // last step's `spend` took them: the original's refused step set them to 0
            // (0x480864) in the step's own call chain, so its line there reads 0 (M3 T4b's
            // checks; T7b2, F1-3); nothing here sets them.
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
        let stuck = match self.aboard.take() {
            Some(Aboard::Board(boat)) => {
                self.board(t, boat);
                self.updated(t);
                key
            }
            Some(Aboard::Land) => return self.land(t, key),
            None => false,
        };
        self.walk_done(t, key);
        if stuck {
            self.stuck(t);
        }
    }

    /// A key's step's end with the hero unable to move (`adventure_key_down` from 0x40913E, after
    /// the status window; E24 M4 T2d2, *provisional*: built after a boat's visit, whose points 0
    /// the loop knows; another step's end leaves points the engine's from Part B): the step's own
    /// path cleared (`make_hero_path` with 1, 0, 0: +0x4C) and the view drawn and shown with the
    /// objects' step when due (`view_show_step`), then both lists and every item of the dialog
    /// drawn, the view again (`board`'s boat's flag at 770 a frame step on), and the whole screen
    /// shown (`hero_list_update` with −1, 1, 1, 0x409167); Next Hero by the kingdom (0x40918B).
    fn stuck(&mut self, t: u32) {
        self.draw(t);
        self.step_objects(t);
        self.draw_lists();
        self.draw(t);
        self.buttons.next_hero = self.kingdom.next_hero();
    }

    /// The walk's end after the visit: the cursor shown again where the mouse is now, by its move
    /// with its tile forgotten (`mouse_position_now` 0x50D500, its x even; `adventure_mouse_move`
    /// 0x40E360: a key's 0x4090F1, the path's 0x40812F); a key's step builds the status window
    /// again.
    fn walk_done(&mut self, t: u32, key: bool) {
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

    /// The empty boat on `at` the selected hero on land steps onto (E24 M4 T2d2: its tile's class
    /// 8); none for a hero in a boat.
    fn boat_on(&self, at: Pos) -> Option<LiveBoat> {
        let boat = self.boats.empty.iter().find(|b| b.at == at).copied();
        boat.filter(|_| self.sailed().is_none())
    }

    /// What the record says the step onto `next` does with a boat (E24 M4 T4b2, design §5.15):
    /// the walk's [`Move::sea`] at its last step, nothing at another; none off a record (Part A's
    /// boats and water decide).
    fn sea_on_record(&self, next: Pos) -> Option<Option<Sea>> {
        let moves = self.moves.as_ref()?;
        let walk = self.walking.and_then(|(m, k)| {
            moves
                .get(m)
                .filter(|w| k + 1 == w.tiles.len() && w.tiles.get(k) == Some(&next))
        });
        Some(walk.and_then(|w| w.sea))
    }

    /// The empty boat on `next` the record's boarding takes, of its type `kind` (E24 M4 T4b2);
    /// none there: the step is off the record ([`Unplayable::Moved`]), no boarding.
    fn boarded(&mut self, next: Pos, kind: u8) -> Option<LiveBoat> {
        let boat = self.boats.empty.iter().find(|b| b.at == next).copied();
        if boat.is_none() {
            self.off(next);
        }
        boat.map(|b| LiveBoat { kind, ..b })
    }

    /// The boat the selected hero sails ([`Boats::sailed`]), if any.
    fn sailed(&self) -> Option<Boat> {
        self.index
            .and_then(|i| self.boats.sailed.get(i).copied())
            .flatten()
    }

    /// The boat a step of the selected hero's from its boat onto the land `at` puts back on its
    /// water tile, facing the step's way `direction` (E24 M4 T2d2): none at sea, none on land.
    fn landing(&self, at: Pos, direction: Direction) -> Option<LiveBoat> {
        let (Some(boat), Some(hero)) = (self.sailed(), self.hero) else {
            return None;
        };
        let facing = Facing::ALL[usize::try_from(direction.number()).unwrap_or(0) % 8];
        (!self.water(at)).then_some(LiveBoat {
            at: hero.at,
            kind: boat.kind,
            facing,
        })
    }

    /// Whether the tile `at` is water ([`Start::water`]: past its end it is).
    fn water(&self, at: Pos) -> bool {
        let side = usize::try_from(self.last).map_or(0, |last| last + 1);
        let tile = (usize::from(at.z) * side + usize::from(at.y)) * side + usize::from(at.x);
        self.water.get(tile).copied().unwrap_or(true)
    }

    /// The selected hero turned `direction`'s way.
    fn turn_to(&mut self, direction: Direction) {
        self.hero = self.hero.map(|hero| Hero {
            facing: direction,
            ..hero
        });
    }

    /// The boarding's start (`hero_move_step`'s class 8, 0x480C19 to 0x480D84; design §5.7): the
    /// hero turned the step's way (*provisional*: both recordings' heroes faced it already), its
    /// standing group and frame 0 (`hero_map_group`), the step sound stopped, no longer lifted
    /// (+0x1EC 0) but off its tile, so that nothing draws it; the boat boarded at the step's end.
    /// [`Loop::lose`] follows: the hero dissolved off the land.
    fn board_start(&mut self, t: u32, direction: Direction, boat: LiveBoat) {
        self.turn_to(direction);
        self.stand();
        if let Some(sound) = &mut self.sound {
            sound.walk_end(t);
        }
        self.lifted = false;
        self.hidden = true;
        self.aboard = Some(Aboard::Board(boat));
    }

    /// The landing's start (`hero_move_step` before the animation, 0x480949 to 0x480B0C; design
    /// §5.7): the boat put back on the hero's water tile facing the step's way (`hero_show` with
    /// class 8, 0x480A71: `boat`, aboard 0), the hero turned that way, no longer lifted but off its
    /// tile, its standing group and frame 0, the step sound stopped, the view drawn and shown at
    /// once (0x480AFE) and the objects' step when due (`view_show_step`); the land visited at the
    /// step's end.
    fn land_start(&mut self, t: u32, direction: Direction, boat: LiveBoat) {
        self.boats.empty.push(boat);
        self.turn_to(direction);
        self.lifted = false;
        self.hidden = true;
        self.stand();
        if let Some(sound) = &mut self.sound {
            sound.walk_end(t);
        }
        self.draw(t);
        self.step_objects(t);
        self.aboard = Some(Aboard::Land);
    }

    /// The boat boarded (`visit_object`'s case 8, `hero_boards_boat` 0x4A1010; design §5.7): taken
    /// off its tile (`hero_hide`), the hero in it with its type and the hero's owner's colour (the
    /// boat's captain, aboard 1), the points 0 ([`Loop::moored`]); for a human (0x4A10D0) the boat
    /// lifted as the hero's picture: the group `boat_map_group` 0x4D9160 by the boat's own facing
    /// (+0x1B), which passes 7 and 9 mirror by the hero's, the step's way (MCL-Q35, the original's
    /// in both modes, *provisional*: `board`'s boat facing down-left shows down-right), the view
    /// drawn and shown at once with the objects' step when due (`view_draw_here`,
    /// `view_show_step` 0x4A1111). The group stays until the hero's next step.
    fn board(&mut self, t: u32, boat: LiveBoat) {
        self.boats.empty.retain(|b| b.at != boat.at);
        self.sail(Some(Boat {
            kind: boat.kind,
            colour: self.player,
        }));
        self.moored();
        self.walk.group = self.group(u32::from(boat.facing.number()));
        self.lifted = true;
        self.hidden = false;
        self.draw(t);
        self.step_objects(t);
    }

    /// The land stepped onto from the boat (`visit_object`'s case 3, `hero_lands_from_boat`
    /// 0x49E670; design §5.7): the hero out of its boat, the points 0 ([`Loop::moored`]); its
    /// facing the step's way (+0x47 from +0x1F4), its standing group, lifted (+0x1F0 34, +0x1EC 1);
    /// then [`Loop::lose`]: the hero dissolved in on the land, `visit_and_update`'s redraws and the
    /// walk's end after it (`guard_attack`: none built).
    fn land(&mut self, t: u32, key: bool) {
        self.sail(None);
        self.moored();
        self.stand();
        self.lifted = true;
        self.hidden = false;
        self.lose(t, Then::Landed(key));
    }

    /// The boat the selected hero sails set ([`Boats::sailed`]: a boarded one's, none at a landing).
    fn sail(&mut self, boat: Option<Boat>) {
        if let Some(sailed) = self.index.and_then(|i| self.boats.sailed.get_mut(i)) {
            *sailed = boat;
        }
    }

    /// The selected hero's points 0 after a boat's visit (`hero_boards_boat`,
    /// `hero_lands_from_boat`; *provisional*: the Admiral's Hat's share at sea not built), so that
    /// it cannot move on (`hero_can_move`, the engine's from Part B), and its slot drawn and shown
    /// at once (`hero_list_slot` with −1, 1, 1).
    fn moored(&mut self) {
        if let Some(i) = self.selected_hero()
            && let Some(hero) = self.kingdom.heroes.get_mut(i)
        {
            hero.points = 0;
            hero.movable = false;
        }
        self.draw_slot();
    }

    /// `visit_and_update` 0x4AAAA0's redraws after a boat's visit (E24 M4 T2d2, design §5.6,
    /// §5.7): the minimap at once, both lists and the resource bar ([`Loop::draw_lists`]), the
    /// status window built again (`status_window_choose` with 1, 1, 1); Move Hero and Next Hero
    /// as the kingdom and the path give them (none: Move Hero off, the hero unable to move).
    fn updated(&mut self, t: u32) {
        self.draw_minimap();
        self.draw_lists();
        self.choose(t, true);
        self.buttons = Buttons::of(&self.kingdom, &self.arrows());
    }

    /// The loop's turn after a walk at its last millisecond: the actions given during it and
    /// left in the queue (a key's step leaves all; the path's walk those behind the one that
    /// ended it) as their messages, then the idle message. One that walks again queues the rest;
    /// a path's walk empties the queue at its start (0x407E87: M2 T6a, the reviewer's R1-5).
    fn resume(&mut self, t: u32, queued: Vec<Action>) -> Result<(), Unplayable> {
        let mut queued = queued.into_iter();
        while let Some(action) = queued.next() {
            self.act(t, action, false)?;
            if core::mem::take(&mut self.flushed) || core::mem::take(&mut self.flush) {
                queued.by_ref().for_each(drop);
            }
            if let Some(moving) = &mut self.moving {
                for action in queued {
                    wait_in(moving, action, self.rules.bugs);
                }
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
        self.hero
            .filter(|_| !self.lifted && !self.hidden)
            .map(|h| h.at)
    }

    /// The standing group by the hero's facing and frame 0 (`hero_map_group` 0x4D9110).
    fn stand(&mut self) {
        let facing = self.hero.map_or(0, |h| h.facing.number());
        self.walk.group = self.group(facing);
        self.walk.frame = 0;
    }

    /// The group a facing draws (`hero_map_group` 0x4D9110, `boat_map_group` 0x4D9160: one table).
    fn group(&self, facing: u32) -> u32 {
        let n = usize::try_from(facing).unwrap_or(0);
        self.rules.standing.get(n).copied().unwrap_or(0)
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
    /// given: as they are); during one of the engine's walks (E24 M4 T4a2) its step's
    /// ([`Move::points`]), the walk then at its next step.
    fn spend(&mut self, t: u32) {
        let given = match (&self.moves, self.walking) {
            (Some(moves), Some((m, k))) => {
                self.walking = Some((m, k.saturating_add(1)));
                moves
                    .get(m)
                    .and_then(|w| w.points.get(k))
                    .copied()
                    .flatten()
            }
            (Some(_), None) => None,
            (None, _) => self
                .points
                .iter()
                .take_while(|(at, _)| *at <= t)
                .last()
                .map(|&(_, points)| points),
        };
        if let (Some(points), Some(i)) = (given, self.selected_hero())
            && let Some(hero) = self.kingdom.heroes.get_mut(i)
        {
            hero.points = points;
        }
    }

    /// A walk's start (E24 M4 T4a2): the next of the engine's walks under way, from its first
    /// step; past the last the step onto `next` is off the record.
    fn take_move(&mut self, next: Pos) {
        let Some(moves) = &self.moves else {
            return;
        };
        if self.next_move < moves.len() {
            self.walking = Some((self.next_move, 0));
            self.last_move = Some(self.next_move);
            self.next_move += 1;
        } else {
            self.walking = None;
            self.last_move = None;
            self.off(next);
        }
    }

    /// A human's step onto `next` during one of the engine's walks (E24 M4 T4a2, design §5.13):
    /// off the record unless it is the move's step; its tiles revealed at once, before the step's
    /// animation (`reveal_disk` at 0x48106B; *provisional*: a boarding's before its dissolve, not
    /// read), which the view's next draw shows and the minimap's at the step's end.
    fn step_on_record(&mut self, next: Pos) {
        let Some((moves, (m, k))) = self.moves.clone().zip(self.walking) else {
            return;
        };
        let walk = moves.get(m);
        if walk.and_then(|w| w.tiles.get(k)) != Some(&next) {
            self.off(next);
            return;
        }
        let revealed = walk.and_then(|w| w.revealed.get(k));
        if let Some(tiles) = revealed.filter(|tiles| !tiles.is_empty()) {
            let mut walked = (*self.walked_tiles).clone();
            walked.extend(tiles.iter().copied());
            self.walked_tiles = Arc::new(walked);
        }
        // The boats told since the walk before, with its first step's reveal (E24 M4 T6, R3-3).
        if k == 0
            && let Some(w) = walk
        {
            self.put_boats(&w.boats);
        }
    }

    /// The empty boats `boats` put on the map (E24 M4 T6, the reviewer's R3-3: a record's boats
    /// told after its start, [`Move::boats`], [`NextTurn::boats`], [`End::boats`]), each in place
    /// of an empty one on its tile: the view's next draw shows them.
    fn put_boats(&mut self, boats: &[LiveBoat]) {
        for &boat in boats {
            self.boats.empty.retain(|b| b.at != boat.at);
            self.boats.empty.push(boat);
        }
    }

    /// A step off the engine's record (E24 M4 T4a2): the first of the millisecond refuses it
    /// when the millisecond ends ([`Loop::run`]).
    fn off(&mut self, next: Pos) {
        self.refuse(next, Unplayable::Moved);
    }

    /// A step onto `next` the record refuses (E24 M4 T4a2; since E24 M4 T6 any refusal by the
    /// tile, `why`: R3-6's landing): the first of the millisecond refuses it when the millisecond
    /// ends ([`Loop::run`]).
    fn refuse(&mut self, next: Pos, why: fn([i32; 3]) -> Unplayable) {
        if self.off_record.is_none() {
            let at = [i32::from(next.x), i32::from(next.y), i32::from(next.z)];
            self.off_record = Some(why(at));
        }
    }

    /// The selected hero's slot drawn and shown (`hero_list_slot` 0x403560 with -1, 1, 1; design
    /// §4.5), when the list shows it: the portrait, both bars and the frame drawn, the rectangle
    /// (609, 212 + 32 i), 48 × 32 shown, which holds the movement bar and leaves out the mana bar
    /// (and the portrait's last 8 columns): the hero's points shown, its mana as last shown (the
    /// slot's hero and portrait change only by an update, which shows the whole screen; seen at
    /// E24 M4 T2d3, MCL-Q14: the original's, [`Bugs::Original`]); the client's shows its mana too.
    fn draw_slot(&mut self) {
        let Some(i) = self.selected_hero() else {
            return;
        };
        let top = self.kingdom.tops[0];
        let hero = self.kingdom.heroes.get(i).copied();
        let shown = i.checked_sub(top).is_some_and(|slot| slot < SLOTS);
        if let Some(hero) = hero.filter(|_| shown)
            && let Some(drawn) = self.drawn.kingdom.heroes.get(i)
        {
            // The client's fix of MCL-Q14 (the user, 2026-10-31; E24 M4 T2d3, design §5.8): the
            // slot shown whole, its mana bar too (a cast's cost).
            let mana = match self.rules.bugs {
                Bugs::Original => drawn.mana,
                Bugs::Fixed => hero.mana,
            };
            if (drawn.points, drawn.mana) != (hero.points, mana)
                && let Some(slot) = Arc::make_mut(&mut self.drawn.kingdom).heroes.get_mut(i)
            {
                slot.points = hero.points;
                slot.mana = mana;
            }
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
        // The client draws the mouse's item's hint again at the turn's end (MCL-Q22).
        match self.rules.bugs {
            Bugs::Original => self.drawn.hint = Hint::Empty,
            Bugs::Fixed => self.rehint = true,
        }
    }

    /// The minimap drawn (`minimap_draw` 0x412C40) and shown: the view's corner, the selected
    /// hero's tile and (E24 M4 T2d1) the objects' owners now; in a computer player's turn nothing (E24 M4 T1b2, design §5.2: the draw
    /// returns at once, the shield standing).
    fn draw_minimap(&mut self) {
        if self.ai.is_some() {
            return;
        }
        self.drawn.minimap = Minimap {
            view: self.view,
            hero: self.hero.map(|h| h.at),
            index: self.index,
            owners: Arc::clone(&self.owners),
            walked: Arc::clone(&self.walked_tiles),
        };
    }

    /// The view drawn and shown: at the objects' step and the palettes now, the walk's offset, the
    /// hero and the arrows; the minimap as last drawn (`drawn.minimap`, which only
    /// [`Loop::draw_minimap`] writes); then the status window chosen (0x40F861,
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
            minimap: self.drawn.minimap.clone(),
            kingdom: Arc::clone(&self.drawn.kingdom),
            hint: self.drawn.hint.clone(),
            selected: self.index,
            pressed: core::mem::take(&mut self.drawn.pressed),
            message: self.drawn.message.take(),
            computer: self.drawn.computer,
            heroes: if *self.drawn.heroes == self.heroes {
                Arc::clone(&self.drawn.heroes)
            } else {
                Arc::new(self.heroes.clone())
            },
            gone: Arc::clone(&self.drawn.gone),
            dissolve: None,
            revealed: self.drawn.revealed,
            attacking: self.drawn.attacking,
            owners: Arc::clone(&self.owners),
            hidden: self.hidden,
            boats: if *self.drawn.boats == self.boats {
                Arc::clone(&self.drawn.boats)
            } else {
                Arc::new(self.boats.clone())
            },
            walked: Arc::clone(&self.walked_tiles),
            switch: None,
            left: self.left.clone(),
        };
        self.choose(t, false);
    }

    /// The status window chosen (`status_window_choose` 0x415DE0; design §4.8, §4.9): a kind
    /// asked whose end has come back to the selection's (0); then the kind asked built (1 the
    /// date's, 2 the kingdom's, 6 a pile's: E24 M4 T2a2) or for 0 the selection's window (the hero's, else the town's, else
    /// the date's), drawn and shown at once. Without `force` (a view's draw, 0x40F861) the kind
    /// shown builds nothing, but the date's steps its frame ([`Loop::date_window`]); with it (a
    /// hero's selection 0x41800B, a key's step 0x4090FE) the window is built again. A window the
    /// live part does not give (none shown, or the selection's unknown) is left as it is.
    fn choose(&mut self, t: u32, force: bool) {
        // A computer player's turn shows kind 5, which its own steps draw (E24 M4 T1b2).
        if self.kingdom.status.is_none() || self.ai.is_some() {
            return;
        }
        if self.asked != 0 && t >= self.end {
            self.asked = 0;
        }
        let shown = self.kingdom.status.as_ref().map(Status::kind);
        // The kind asked compared first, its window cloned only when it is built (T7b2, F1-5:
        // an unforced draw of the kind shown builds nothing).
        let builds = |kind: u8| force || shown != Some(kind);
        let selection = self.kingdom.selection.as_ref().or_else(|| {
            self.kingdom
                .status
                .as_ref()
                .filter(|s| matches!(s, Status::Hero(_) | Status::Town(_)))
        });
        let window = match (self.asked, self.kingdom.selected, selection) {
            (1, _, _) | (0, Selected::None, _) => None,
            (2, _, _) => match &self.realm {
                Some(realm) => Some(builds(2).then(|| Status::Kingdom(realm.clone()))),
                None => return,
            },
            // A pile's kind 6 (E24 M4 T2a2 part B).
            (KIND_FOUND, _, _) => match &self.found {
                Some(found) => Some(builds(KIND_FOUND).then(|| Status::Found(found.clone()))),
                None => return,
            },
            (0, Selected::Hero(_), Some(s @ Status::Hero(_)))
            | (0, Selected::Town(_), Some(s @ Status::Town(_))) => {
                Some(builds(s.kind()).then(|| s.clone()))
            }
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
            Some(next) => {
                let built = next.is_some();
                (next, built)
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

    /// A walk's frame drawn: pass 9 steps the lifted hero's frame after it (0x47FF6D); with none
    /// lifted it does not run (E24 M4 T2d2: a boarding's and a landing's step keep frame 0).
    fn frame_drawn(&mut self, t: u32) {
        self.draw(t);
        if !self.lifted {
            return;
        }
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

/// The button an open box shows pressed, if any.
fn pressed(open: &Open) -> Option<u16> {
    open.message
        .shown
        .states
        .iter()
        .find(|(_, s)| s.pressed)
        .map(|(&id, _)| id)
}

/// A box's key's name, which its key's release matches ([`Open::key`]).
const fn key_name(key: Key) -> &'static str {
    match key {
        Key::Ok => "Return",
        Key::Cancel => "Escape",
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

/// The arrows of a path the hero walks whole today from `from` along `tiles` (E24 M4 T4a2,
/// design §5.12, *provisional*: a record's `MoveHero`, which the engine walks whole that day,
/// ADV-MOV-006): each tile it passes `table` (`path_arrow_frames` 0x6782AC: by the step into it
/// plus 8 × the step out of it, [`Direction::number`]) plus 2, the destination
/// [`DESTINATION`]; all green (`make_hero_path` 0x418DD0's red twin is for a step the points do
/// not pay). `None` when two tiles in a row are no neighbours on one level, or the table holds
/// no arrow for a pair of steps.
#[must_use]
pub fn arrows(table: &[u8], from: Pos, tiles: &[Pos]) -> Option<Vec<Arrow>> {
    let mut at = from;
    let mut steps = Vec::with_capacity(tiles.len());
    for &tile in tiles {
        steps.push((Direction::between(at, tile)?, tile));
        at = tile;
    }
    steps
        .iter()
        .enumerate()
        .map(|(i, &(into, tile))| {
            let arrow = match steps.get(i + 1) {
                None => DESTINATION,
                Some(&(out, _)) => {
                    let n = into.number() + 8 * out.number();
                    let frame = table.get(usize::try_from(n).unwrap_or(usize::MAX))?;
                    u16::from(*frame) + 2
                }
            };
            Some(Arrow { at: tile, arrow })
        })
        .collect()
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
    play_held(rules, start, actions, until).map(|(states, _)| states)
}

/// The states each millisecond showed before its last, by millisecond (E24 M4 T1b2: [`play_held`]).
pub type Held = BTreeMap<u32, Vec<State>>;

/// [`play`] with the states each millisecond showed before its last (E24 M4 T1b2: a computer's
/// single draws at speed 4, [`Loop::take_held`]), by millisecond where there are any.
///
/// # Errors
/// As [`play`].
pub fn play_held(
    rules: Rules,
    start: &Start,
    actions: &BTreeMap<u32, Vec<Action>>,
    until: u32,
) -> Result<(Vec<State>, Held), (u32, Unplayable)> {
    let mut l = Loop::start(rules, start).map_err(|why| (0, why))?;
    let mut states = Vec::with_capacity(usize::try_from(until).unwrap_or(0).saturating_add(1));
    let mut held = BTreeMap::new();
    states.push(l.state(0));
    for t in 1..=until {
        let given = actions.get(&t).map_or(&[][..], Vec::as_slice);
        l.run(t, given).map_err(|why| (t, why))?;
        let shown = l.take_held();
        if !shown.is_empty() {
            held.insert(t, shown);
        }
        states.push(l.state(t));
    }
    Ok((states, held))
}

/// The timeline's lines (design §3.2): at 0 the view, the objects, the cycles, the cursor, the
/// buttons (move hero's and next hero's states, 1 enabled; E24 M3 T2a) and the selected hero's
/// points (E24 M3 T4b; none without one), then each one where it changed, in that order within a
/// millisecond; after 0 the walk's (§3.5) where
/// they changed: the selected hero, the walk's drawing, since E24 M4 T1a the date and the current
/// player, since T1b2 another player's turn's window, the path's arrows; and a `screen` line
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
                "cursor {} {} {} {}",
                s.cursor.kind, s.cursor.frame, s.cursor.at[0], s.cursor.at[1]
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
            // The date and the current player where they change (E24 M4 T1a).
            if s.date != l.date {
                let Date { month, week, day } = s.date;
                out.push(format!("{} date {month} {week} {day}", s.t));
            }
            if s.player != l.player {
                out.push(format!("{} player {}", s.t, s.player));
            }
            // Another player's turn's window where its frames change, the line alone once another
            // kind shows (E24 M4 T1b2).
            if s.other != l.other {
                out.push(s.other.map_or_else(
                    || format!("{} other", s.t),
                    |[crest, sand, glass]| format!("{} other {crest} {sand} {glass}", s.t),
                ));
            }
            // The spell book where it opens and leaves (E24 M4 T2d3).
            if s.book != l.book {
                out.push(format!("{} book {}", s.t, u8::from(s.book)));
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
        // The battle shown whole (E24 M4 T5b2): its winner.
        if let Some(winner) = s.battle {
            out.push(format!("{} battle {winner}", s.t));
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
