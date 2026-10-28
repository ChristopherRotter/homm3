//! The scene: from a record's snapshot and the asset tables to the draw list of one frame (E17 M1
//! T7: the battle at time 0, `docs/epics/E17-design.md` §4; animations come with M2).
//!
//! The list names what is drawn (a picture, an obstacle's sprite, a creature's sprite with the
//! shadow its frame holds and its outline, a hero and its flag) in the executable's order; [`crate::compose`] decides
//! how each is drawn, so a later graphics option changes only the compositor (the user,
//! 2026-10-13).

use alloc::boxed::Box;
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt;

use h3_protocol::wire::{Event, Manifest, Observation, Side, Stack};

use crate::anim::{
    Blast, DRAWBRIDGE_RECT, Field, GONE, Magic, Object, Pose, Projectile, Ray, Spot, Timeline,
    TurretPose,
};
use crate::auto::Auto;
use crate::hero::{FLAG_START, HeroPose};
use crate::layout::BattleLayout;
use crate::panel::Autoplay;
use crate::siege::{AFTER_ROWS, BEFORE_ROWS, Context, GATE_ROW, SiegeLook, Walls};
use crate::tables::{ObstacleArt, Tables};

/// The row passes of the battle's redraw (0x494440, §4.6): for each row, passes 0–7, each over
/// the row's columns left to right.
const PASSES: u8 = 8;
/// The pass that draws the layer-0 obstacle anchored on a hex.
const OBSTACLE_PASS: u8 = 2;
/// A standing stack's draw layer, the pass it is drawn in (+0x8 = 4 from the setup, 0x43D750);
/// a pose names its own (`crate::anim`: walks, strikes, the corpses' pass 1).
const STANDING: u8 = 4;

/// The battle at a moment of its timeline (`crate::anim`): each stack's pose, the active stack,
/// the idle steps taken and the projectile in flight.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Moment {
    /// The stacks' poses, by id; a living stack without one stands on its hex.
    pub poses: BTreeMap<u8, Pose>,
    /// The active stack (its outline).
    pub active: Option<u8>,
    /// The idle steps taken: the outline's colour rotates with them (§4.2).
    pub pulse: u32,
    /// The projectile in flight, drawn over everything (§7.9).
    pub projectile: Option<Projectile>,
    /// The ray drawn so far, over everything (§7.12).
    pub ray: Option<Ray>,
    /// The attacker's and the defender's hero and flag (E17 M3 T3a; none: the hero stands, its
    /// flag at its start).
    pub heroes: [Option<HeroPose>; 2],
    /// The magic animation drawn over stacks (E17 M3 T3b).
    pub magic: Option<Magic>,
    /// The magic animation at a hex, over everything (E17 M3 T7c-1).
    pub spot: Option<Spot>,
    /// The battlefield objects spells placed, by anchor (E17 M3 T7c-2).
    pub objects: BTreeMap<u8, Object>,
    /// The magic animation over the whole battlefield, tiled over everything on it (E17 M3
    /// T7c-2).
    pub field: Option<Field>,
    /// The counts the boxes show, by id, of the stacks whose count changed since the snapshot
    /// (E17 M3 T5b); the others show the snapshot's.
    pub counts: BTreeMap<u8, u32>,
    /// The hues of the count boxes of the stacks with active spells, by id, as the bits of the
    /// f32 values ([`spell_hue`]; E17 M3 T7b-2a); the other boxes take the layout's tint.
    pub hues: BTreeMap<u8, u32>,
    /// The stack the mouse marks (E17 M3 T6c-1, `crate::mouse`): its outline the mouse's unless
    /// it is the active stack (0x43E23A to 0x43E261, §4.2).
    pub marked: Option<u8>,
    /// The stacks an area spell's aim marks (E17 M5 T3b, +0x4F1): their outline the mouse's, the
    /// active stack's too (0x43E268).
    pub area: Vec<u8>,
    /// A siege's picture states, when they are not the record's own (E17 M4 T1b: a hit part's;
    /// *provisional*, [`crate::siege::Walls::of`]; E17 M4 T2b: the timeline's).
    pub walls: Option<Walls>,
    /// A siege's turret records' poses (E17 M4 T2b; none: standing, facing left).
    pub turrets: [Option<TurretPose>; 3],
    /// A wall shot's explosion (E17 M4 T2b).
    pub blast: Option<Blast>,
    /// While the drawbridge's redraws show its rectangle alone (E17 M4 T2c-1, the original's:
    /// CLT-Q23), the battle the screen shows beyond it.
    pub bridge: Option<Box<Moment>>,
    /// While Earthquake shakes the screen (E17 M4 T2c-2), how many of its steps have shown: the
    /// frame's assembler shifts the whole screen ([`Draw::Shaken`]).
    pub shake: Option<u32>,
}

impl Moment {
    /// The battle at `t` of `timeline` (E17 M2 T4b-2a).
    pub fn at(timeline: &Timeline, t: u32) -> Self {
        Self {
            poses: timeline.at(t),
            active: timeline.outlined(t),
            pulse: timeline.pulse(t),
            projectile: timeline.projectile(t),
            ray: timeline.ray(t),
            heroes: timeline
                .heroes
                .as_ref()
                .map_or([None; 2], |h| h.at(timeline, t)),
            magic: timeline.magic(t),
            spot: timeline.spot(t),
            objects: timeline.objects(t),
            field: timeline.field(t),
            counts: timeline.counts(t),
            hues: timeline.hues(t),
            marked: None,
            area: Vec::new(),
            walls: timeline.walls(t),
            turrets: timeline.turrets(t),
            blast: timeline.blast(t),
            bridge: timeline
                .bridge(t)
                .map(|before| Box::new(Self::at(timeline, before))),
            shake: timeline.shake(t),
        }
    }
}

/// One thing drawn, in the order of the list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Draw {
    /// The battlefield's picture, opaque, its top-left corner at (`x`, `y`) (§4.4).
    Background {
        /// The picture.
        picture: String,
        /// Its left edge.
        x: i32,
        /// Its top edge.
        y: i32,
    },
    /// The absolute obstacle's picture, index 0 transparent, into the background (§4.4).
    AbsoluteObstacle {
        /// The picture.
        picture: String,
        /// Its left edge.
        x: i32,
        /// Its top edge.
        y: i32,
    },
    /// A usual obstacle's sprite: never mirrored, without a keyed shadow (§4.4).
    Obstacle {
        /// The DEF.
        def: String,
        /// The DEF group.
        group: u32,
        /// The frame in the group.
        frame: u32,
        /// The canvas's left edge.
        x: i32,
        /// The canvas's top edge.
        y: i32,
    },
    /// A projectile's frame, drawn plain over the screen as it stands, runs skipped (§7.9).
    Projectile {
        /// The DEF.
        def: String,
        /// The frame in group 0.
        frame: u32,
        /// The canvas's left edge.
        x: i32,
        /// The canvas's top edge.
        y: i32,
        /// Reflected across its canvas.
        mirrored: bool,
    },
    /// A wall shot's explosion frame (E17 M4 T2b, §10.2): drawn plain like a projectile; in the
    /// original's profile only its canvas's top `rows` rows, the height of its rectangle clipped
    /// to the battlefield (0x446453 passes the clipped size with the canvas's own place: CLT-Q22).
    Explosion {
        /// The DEF.
        def: String,
        /// The frame in group 0.
        frame: u32,
        /// The canvas's left edge.
        x: i32,
        /// The canvas's top edge.
        y: i32,
        /// The canvas's rows drawn from its top.
        rows: u32,
    },
    /// A ray's strokes, each pixel in its colour straight into the screen (§7.12).
    Ray(Ray),
    /// A hero's flag beside the battlefield, in its player's colours, with the shadow its frame
    /// holds (§4.5; E17 M3 T3a).
    Flag {
        /// The DEF.
        def: String,
        /// The frame in group 0.
        frame: u32,
        /// The canvas's left edge.
        x: i32,
        /// The canvas's top edge.
        y: i32,
        /// The palette of the players' colours.
        palette: String,
        /// The player's colour (0 red to 7 pink): its 32 entries of that palette.
        player: u8,
    },
    /// A hero beside the battlefield, with the shadow its frame holds, no outline (§4.5).
    Hero {
        /// The DEF.
        def: String,
        /// The DEF group.
        group: u32,
        /// The frame in the group.
        frame: u32,
        /// The canvas's left edge.
        x: i32,
        /// The canvas's top edge.
        y: i32,
        /// Reflected across its canvas (the right side's).
        mirrored: bool,
    },
    /// A magic animation's frame over a stack, right after its sprite (`draw_battle_stack`'s
    /// tail 0x43E795, §7.15; E17 M3 T3b): group 0, by the plain drawer with runs skipped or the
    /// translucent one (0x4953B0, which clips it to the battlefield, 0x694F18: the panel below,
    /// not drawn yet, is the only part of the screen outside it).
    Magic {
        /// The DEF.
        def: String,
        /// The frame in group 0.
        frame: u32,
        /// The canvas's left edge.
        x: i32,
        /// The canvas's top edge.
        y: i32,
        /// Reflected across its canvas (the stack faces left).
        mirrored: bool,
        /// Drawn half and half with what is under it.
        translucent: bool,
    },
    /// A magic animation's frame tiled over an area from its top-left corner (Armageddon's,
    /// `mass_damage_handler` 0x5A4FEF to 0x5A50AD; E17 M3 T7c-2): ceil(800 / W) columns and
    /// ceil(600 / H) rows of its canvas, each cut to what is left of the area, plain.
    Tiled {
        /// The DEF.
        def: String,
        /// The frame in group 0.
        frame: u32,
        /// The area's width and height (the battlefield's).
        area: [i32; 2],
    },
    /// A stack's count box, right after its sprite and any magic animation over it
    /// (`draw_battle_stack`'s part 1 0x43E568, §4.3; E17 M3 T4a): the layout's picture with index
    /// 0 transparent, its inside tinted, the count written over it in the layout's font.
    CountBox {
        /// Whose box it is (the stack's id).
        stack: u8,
        /// The box's left edge.
        x: i32,
        /// Its top edge.
        y: i32,
        /// The count written (`%d` of the stack's +0x50 while it is not −1, else +0x4C: §4.3).
        count: u32,
        /// The tint's hue and saturation, the bits of the f32 values: the layout's without
        /// spells, else the hue by the stack's spells ([`spell_hue`], E17 M3 T7b-2a).
        tint: [u32; 2],
    },
    /// A creature's sprite with the shadow its frame holds, standing on a hex (§4.2).
    Creature {
        /// The DEF.
        def: String,
        /// The DEF group.
        group: u32,
        /// The frame in the group.
        frame: u32,
        /// Where it stands: its position hex's anchor.
        anchor: [i32; 2],
        /// It faces left (the defender): drawn mirrored.
        mirrored: bool,
        /// Its outline: the layout's outline palette's entry, none for no outline.
        outline: Option<u8>,
    },
    /// A siege's part (E17 M4 T1b, design §10.1): its picture with index 0 transparent, its
    /// top-left corner at (`x`, `y`), only its pixels inside `clip` (`battle_draw_pcx_rect`
    /// 0x4957B0's strips, `moat_over_stack` 0x495A10's: left, top, right and bottom edges, the
    /// last two excluded; none: whole).
    Wall {
        /// The picture.
        picture: String,
        /// Its left edge.
        x: i32,
        /// Its top edge.
        y: i32,
        /// The screen's rectangle it is cut to.
        clip: Option<[i32; 4]>,
    },
    /// A turret's shooter (E17 M4 T1b, `draw_turret_shooter` 0x495090): drawn as a creature
    /// ([`Draw::Creature`]: its canvas placed by facing from `anchor`), but only its canvas's top
    /// `rows` rows.
    Shooter {
        /// The DEF.
        def: String,
        /// The DEF group.
        group: u32,
        /// The frame in the group.
        frame: u32,
        /// Its point, the shift of a two-hex creature included.
        anchor: [i32; 2],
        /// Drawn mirrored (facing left).
        mirrored: bool,
        /// Its outline's palette entry, if any.
        outline: Option<u8>,
        /// The canvas's rows drawn from its top.
        rows: u32,
    },
    /// A draw list composed on its own frame, of which only the rectangle `rect` (left, top,
    /// right, bottom, exclusive) is drawn here (E17 M4 T2b: a wall shot's explosion shown in its
    /// rectangle alone, `screen_show_rect` 0x446485).
    Within {
        /// The rectangle.
        rect: [i32; 4],
        /// The list.
        list: Vec<Draw>,
    },
    /// Earthquake's shake (E17 M4 T2c-2, `earthquake` 0x5A7C80, design §10.2): the list composed
    /// on a frame of its own (the screen saved), copied onto a copy of itself at each offset in
    /// turn (what an offset uncovers keeps the step before's pixels: CLT-Q19 (6)); that copy's
    /// rectangle `rect` (the battlefield shown) is drawn here, the saved frame elsewhere.
    Shaken {
        /// The offsets, in order.
        offsets: Vec<[i32; 2]>,
        /// The rectangle shown (left, top, right, bottom, exclusive).
        rect: [i32; 4],
        /// The list.
        list: Vec<Draw>,
    },
    /// What a dialog draws (E17 M3 T6a; §9.7): its picture, button, DEF item, text over a picture,
    /// text alone or shadow, the dialog's own draw items since E24 M3 T1a (`h3_ui::ui::Draw`).
    Dialog(h3_ui::ui::Draw),
    /// The result window's clip (E17 M5 T4b2b, §7.21): the frame on the screen `since` ms after
    /// the window opened (`crate::clip::shown`), the start's and then its loop's, its pixels
    /// straight onto the screen with its corner at (`x`, `y`) (`BinkCopyToBuffer`).
    Video {
        /// The start's file (in the video archives).
        start: String,
        /// The loop's, if any.
        looped: Option<String>,
        /// How long the window has been open.
        since: u32,
        /// The left edge.
        x: i32,
        /// The top edge.
        y: i32,
    },
    /// The mouse's cursor, over everything (E17 M3 T6c-1, design §9.9): a frame of its DEF's group
    /// 0, its canvas's corner at the mouse less the frame's hot spot, by the plain drawer with runs
    /// skipped (0x50D8B0 → 0x47BEB0 → 0x47C570). The original's drawing bug, the mouse's x
    /// rounded down to even first (0x50CE04), is the compositor's (`h3_render::Bugs`).
    Cursor {
        /// The DEF.
        def: String,
        /// The frame in group 0.
        frame: u32,
        /// The mouse's x.
        x: i32,
        /// The mouse's y.
        y: i32,
        /// The frame's hot spot in its canvas.
        hot: [i32; 2],
    },
}

/// What the panel below the battlefield draws, after the battlefield (E17 M3 T4b-2b-1, T6a; §9.7):
/// the layout's panel as the stacks of `side` (the active stack's controller; none before a turn)
/// begin their turn in the battle `observation` shows (`crate::panel`), with what it `kept` from
/// the turns before, its log showing `log`.
#[must_use]
pub fn panel(
    observation: &Observation,
    side: Option<Side>,
    kept: crate::panel::Kept,
    log: &str,
    layout: &BattleLayout,
) -> Vec<Draw> {
    pressed_panel(
        observation,
        side,
        (kept, Autoplay::default()),
        log,
        layout,
        None,
    )
}

/// As [`panel`], the button `pressed` (the mouse's or a hotkey's, E17 M3 T6c-2) showing its
/// click frame unless it is disabled (a disabled button takes no press, 0x4562DD).
fn pressed_panel(
    observation: &Observation,
    side: Option<Side>,
    kept: (crate::panel::Kept, Autoplay),
    log: &str,
    layout: &BattleLayout,
    pressed: Option<u16>,
) -> Vec<Draw> {
    let mut shown = crate::panel::shown(observation, side, kept, log, PLAYERS);
    if let Some(id) = pressed {
        let state = shown.states.entry(id).or_default();
        state.pressed = !state.disabled;
    }
    layout
        .panel
        .draws(&shown, &layout.player_palette)
        .into_iter()
        .map(Draw::Dialog)
        .collect()
}

/// The panel over a battle (E17 M3 T6b, design §9.8): the battle as each turn's start found it
/// (`crate::panel::turns`) and the text its log's item shows from each time on
/// (`crate::battle_log::texts`), drawn at any time of the battle by [`Panels::at`].
///
/// A frame reads it without going over the battle (E17 M5 T1b): what the panel keeps after each
/// number of turns started, and the texts by time, are made once.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Panels {
    /// The battle at each `TurnStarted`'s turn, in order.
    turns: Vec<Observation>,
    /// The log item's text from each time on.
    texts: Vec<(u32, String)>,
    /// When each hero's cast started, and its side (E17 M3 T7b-1), in time order.
    casts: Vec<(u32, Side)>,
    /// What the panel keeps after each number of turns started (`crate::panel::kept`), from
    /// none: the opening's, then each turn's.
    kept: Vec<crate::panel::Kept>,
    /// The texts' times in order, each with the last text of the list shown by then (the list
    /// need not be in time order: a shot's line has its start's time).
    by_time: Vec<(u32, usize)>,
    /// When Auto Combat plays the turns (E17 M5 T4c1), and before each turn (from none: the
    /// opening's) whether its button is selected from the human players' last turn.
    auto: (Auto, Vec<bool>),
}

impl Panels {
    /// The panel over the battle the record's opening `snapshot` and its `events` (in order)
    /// show, its log's item showing `texts`.
    #[must_use]
    pub fn new(snapshot: &Observation, events: &[Event], texts: Vec<(u32, String)>) -> Self {
        let turns = crate::panel::turns(snapshot, events);
        // The opening's side the panel was built for (*provisional*), then each turn's.
        let built = controller(snapshot, snapshot).unwrap_or(Side::Attacker);
        let mut kept = alloc::vec![crate::panel::Kept::built(built)];
        for o in core::iter::once(snapshot).chain(&turns) {
            let last = kept[kept.len() - 1];
            let now = controller(snapshot, o)
                .map_or(last, |s| last.then(snapshot, (s, o.tactics.is_some())));
            kept.push(now);
        }
        // The opening's own controller comes with the opening: none before it.
        kept.remove(0);
        let mut by_time: Vec<(u32, usize)> = texts.iter().map(|(at, _)| *at).zip(0..).collect();
        by_time.sort_unstable();
        let mut last = 0;
        for (_, i) in &mut by_time {
            last = last.max(*i);
            *i = last;
        }
        Self {
            turns,
            texts,
            casts: Vec::new(),
            kept,
            by_time,
            auto: (Auto::default(), Vec::new()),
        }
    }

    /// The panel with Auto Combat playing as `auto` says (E17 M5 T4c1, design §7.22): a turn it
    /// plays dims the panel, its button enabled and selected; the button stays selected through
    /// the other sides' turns until a human's turn it does not play.
    #[must_use]
    pub fn with_auto(self, snapshot: &Observation, auto: Auto) -> Self {
        let mut selected = alloc::vec![false];
        for (k, o) in core::iter::once(snapshot).chain(&self.turns).enumerate() {
            let last = selected[k];
            let human = controller(snapshot, o)
                .is_some_and(|s| snapshot.sides[usize::from(s == Side::Defender)].human);
            selected.push(if human { auto.played(k) } else { last });
        }
        Self {
            auto: (auto, selected),
            ..self
        }
    }

    /// The panel with the heroes' casts, each its start and its side (E17 M3 T7b-1): from a
    /// cast's start the spell book's button of its side's turn is disabled, as the hero has cast
    /// this round (the recorded `cast`: at the click).
    #[must_use]
    pub fn with_casts(self, casts: Vec<(u32, Side)>) -> Self {
        Self { casts, ..self }
    }

    /// The text the log's item shows at `t` (`crate::battle_log::text_at`'s: the last of the list
    /// shown by then).
    #[must_use]
    pub fn text(&self, t: u32) -> &str {
        let shown = self.by_time.partition_point(|(at, _)| *at <= t);
        shown
            .checked_sub(1)
            .and_then(|k| self.texts.get(self.by_time[k].1))
            .map_or("", |(_, text)| text.as_str())
    }

    /// What the panel draws at `t` of the battle `snapshot` opens, whose turns started at the
    /// times of `started` (a timeline's `turns`, in the order of the `TurnStarted`s): as its last
    /// turn's start by then set it (the opening's before the first), with what it kept from the
    /// turns so far (`crate::panel::kept`, the opening's side the one it was built for:
    /// *provisional*).
    #[must_use]
    pub fn at(
        &self,
        snapshot: &Observation,
        started: &[(u32, u8)],
        t: u32,
        layout: &BattleLayout,
    ) -> Vec<Draw> {
        self.at_showing(snapshot, started, t, layout, self.text(t))
    }

    /// As [`Panels::at`], its log's item showing `text` (the status line's hint, E17 M3 T6c-1).
    #[must_use]
    pub fn at_showing(
        &self,
        snapshot: &Observation,
        started: &[(u32, u8)],
        t: u32,
        layout: &BattleLayout,
        text: &str,
    ) -> Vec<Draw> {
        self.at_pressed(snapshot, started, t, layout, text, None)
    }

    /// As [`Panels::at_showing`], the button `pressed` showing its click frame (E17 M3 T6c-2).
    #[must_use]
    pub fn at_pressed(
        &self,
        snapshot: &Observation,
        started: &[(u32, u8)],
        t: u32,
        layout: &BattleLayout,
        text: &str,
        pressed: Option<u16>,
    ) -> Vec<Draw> {
        let (kept, turns) = self.kept_at(started, t);
        let since = started
            .get(..started.partition_point(|(at, _)| *at <= t))
            .and_then(<[_]>::last)
            .map_or(0, |(at, _)| *at);
        let mut now = turns.last().unwrap_or(snapshot).clone();
        let from = self.casts.partition_point(|(at, _)| *at < since);
        let to = self.casts.partition_point(|(at, _)| *at <= t);
        for &(_, side) in self.casts.get(from..to).unwrap_or_default() {
            if let Some(hero) = now.sides[usize::from(side == Side::Defender)].hero.as_mut() {
                hero.casts = hero.casts.max(1);
            }
        }
        // Auto Combat on the turn under way (E17 M5 T4c1).
        let (auto, selected) = &self.auto;
        let k = turns.len();
        let autoplay = Autoplay {
            plays: auto.plays(k, t),
            selected: selected.get(k).copied().unwrap_or_default(),
        };
        pressed_panel(
            &now,
            controller(snapshot, &now),
            (kept, autoplay),
            text,
            layout,
            pressed,
        )
    }

    /// Whether the log's scroll buttons are enabled at `t` ([`Panels::at`]'s arguments): a click
    /// on a disabled one scrolls nothing (design §9.10, 0x4562DD; E17 M3 T9b, the reviewer).
    #[must_use]
    pub fn scrolls(&self, started: &[(u32, u8)], t: u32) -> bool {
        self.kept_at(started, t).0.scroll
    }

    /// What the panel kept from the turns started by `t` (`crate::panel::kept`), and those turns;
    /// `started` in time order (a timeline's turns are).
    fn kept_at(&self, started: &[(u32, u8)], t: u32) -> (crate::panel::Kept, &[Observation]) {
        let started = started.partition_point(|(at, _)| *at <= t);
        let turns = self.turns.get(..started).unwrap_or(&self.turns);
        (self.kept[turns.len()], turns)
    }
}

/// The controller of the battle `o`'s active stack, by the stacks of the opening `snapshot`
/// (*provisional*: a controller Hypnotize changes and a stack the battle creates are not
/// followed, as created stacks are not drawn; the queue's inbox).
fn controller(snapshot: &Observation, o: &Observation) -> Option<Side> {
    o.active
        .and_then(|id| snapshot.stacks.iter().find(|s| s.id == id))
        .map(|s| s.controller)
}

/// Why a snapshot has no draw list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SceneError {
    /// A handle the manifest's key tables do not hold: the table and the handle.
    Handle(&'static str, u16),
    /// A content key the asset tables have no entry for.
    Key(String),
    /// An obstacle type drawn the other way: a usual one as the absolute obstacle, or an absolute
    /// one among the usual ones.
    Art(String),
}

impl fmt::Display for SceneError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Handle(table, h) => write!(f, "the manifest's {table} have no handle {h}"),
            Self::Key(key) => write!(f, "the asset tables have no entry for `{key}`"),
            Self::Art(key) => write!(f, "`{key}` is not drawn that way"),
        }
    }
}

/// A usual obstacle or a spell's battlefield object, placed, with the frame it shows.
struct Placed {
    def: String,
    frame: u32,
    before_stacks: bool,
    at: (u8, u8),
    x: i32,
    y: i32,
}

/// The players' colours of the attacker's and the defender's flags: red and blue, as the capture
/// set's maps give them (*provisional*: a battle's record names no player colours, CLT-Q6).
const PLAYERS: [u8; 2] = [0, 1];

/// The background variant 0x41ADB0 returns when no tree, mountain or dead vegetation stands
/// around the battle (§4.4).
const EMPTY_SURROUNDINGS: usize = 2;

/// The draw list of the battle `observation` shows, at time 0: the background with the absolute
/// obstacle, the layer-1 obstacles, each side's flag and hero (E17 M3 T3a), then each row's
/// passes with its layer-0 obstacles and its living stacks, each with its count box (§4.6; M3
/// T4a). A natural obstacle without a type is not drawn; a removed one is no longer in the
/// observation's list (the engine drops it), so every listed one is drawn, whatever hexes a
/// removal cleared. The spells' battlefield objects come with a moment of the timeline
/// ([`battle_at`], E17 M3 T7c-2), whose stage holds their looks. Left out until later tasks:
/// corpses, the grid.
///
/// # Errors
/// A handle the manifest does not resolve, a key the tables lack, an obstacle type of the wrong
/// kind.
pub fn battle(
    observation: &Observation,
    manifest: &Manifest,
    tables: &Tables,
    layout: &BattleLayout,
) -> Result<Vec<Draw>, SceneError> {
    let moment = Moment {
        active: observation.active,
        ..Moment::default()
    };
    battle_at(observation, &moment, manifest, tables, layout)
}

/// The draw list of the battle `observation` shows at `moment` (E17 M2 T4a): as [`battle`],
/// each stack in its pose, facing its way, in its layer's pass of its position's row (T4b-2a:
/// a walk's, a strike's; a dead stack's is the corpses' pass), a flier between its hexes after
/// every row (T5a, `anim::FLYING`); the outline on the moment's active stack in the colour its
/// idle steps rotated to; the projectile last.
///
/// # Errors
/// As [`battle`].
pub fn battle_at(
    observation: &Observation,
    moment: &Moment,
    manifest: &Manifest,
    tables: &Tables,
    layout: &BattleLayout,
) -> Result<Vec<Draw>, SceneError> {
    // The drawbridge's redraws (E17 M4 T2c-1, CLT-Q23): the battle as the screen showed it
    // before, the moment's inside the drawbridge's rectangle.
    if let Some(before) = &moment.bridge {
        let now = Moment {
            bridge: None,
            ..moment.clone()
        };
        let [shown, drawn] =
            [before.as_ref(), &now].map(|m| battle_at(observation, m, manifest, tables, layout));
        return shown.and_then(|mut list| {
            drawn.map(|inner| {
                list.push(Draw::Within {
                    rect: DRAWBRIDGE_RECT,
                    list: inner,
                });
                list
            })
        });
    }
    // A wall shot's explosion (E17 M4 T2b, design §10.2): in its rectangle each frame drawn over a
    // redraw of the battle with its walls' states, without the background restored after the
    // first (the original's, CLT-Q19 (4)); with the slip fixed, the last alone. The battle's own
    // list first, then each frame's redraw.
    let frames = moment.blast.as_ref().map_or(&[][..], |b| &b.frames[..]);
    let layers = core::iter::once((None, moment.walls)).chain(
        frames
            .iter()
            .map(|&(frame, walls)| (Some(frame), Some(walls))),
    );
    let (mut list, mut inner) = (Vec::new(), Vec::new());
    for (i, (frame, walls)) in layers.enumerate() {
        let layer = Moment {
            walls,
            blast: None,
            ..moment.clone()
        };
        let (background, mut drawn) = drawn(observation, &layer, manifest, tables, layout)?;
        let Some((frame, blast)) = frame.zip(moment.blast.as_ref()) else {
            list = drawn;
            continue;
        };
        if i > 1 {
            drawn.drain(..background);
        }
        inner.extend(drawn);
        // 0x446453 draws the canvas's top as many rows as the clipped rectangle has from the
        // canvas's own top, and the frame drawer clips the rows above the battlefield off them
        // again: an explosion over the top edge loses as many rows at its bottom (CLT-Q22: the
        // compose's original profile alone). Its columns alike at the left edge, which no wall's
        // target reaches (the canvas 254 wide around x 400 or more). Earthquake's the same way
        // over every target hit, in turn (0x5A7FDC; E17 M4 T2c-2).
        for landing in &blast.landings {
            let rows = u32::try_from(landing.rect[3] - landing.rect[1]).unwrap_or(0);
            inner.push(Draw::Explosion {
                def: blast.def.clone(),
                frame,
                x: landing.at[0],
                y: landing.at[1],
                rows,
            });
        }
    }
    // Each landing's rectangle shown (0x446485, 0x5A7FFC): the back buffer's in all of them.
    for landing in moment.blast.iter().flat_map(|b| &b.landings) {
        list.push(Draw::Within {
            rect: landing.rect,
            list: inner.clone(),
        });
    }
    Ok(list)
}

/// A frame's whole list (the battle and the panel, no cursor) as Earthquake's shake shows it after
/// `steps` of its steps (E17 M4 T2c-2, design §10.2): the screen saved at its start shifted by
/// each offset in turn, the battlefield shown (the layout's mouse field), the panel as it was.
#[must_use]
pub fn shaken(list: Vec<Draw>, steps: u32, layout: &BattleLayout) -> Draw {
    let [width, height] = layout.mouse.field;
    Draw::Shaken {
        offsets: crate::anim::shake_offsets(steps),
        rect: [0, 0, width, height],
        list,
    }
}

/// [`battle_at`] without a wall shot's explosion, and how many of its first draws are the
/// background buffer's (§4.4: the battlefield's picture and the absolute obstacle; a siege's
/// background, moat and lip).
fn drawn(
    observation: &Observation,
    moment: &Moment,
    manifest: &Manifest,
    tables: &Tables,
    layout: &BattleLayout,
) -> Result<(usize, Vec<Draw>), SceneError> {
    let keys = &manifest.keys;
    let (width, height) = (manifest.field.width, manifest.field.height);
    let at = |hex: u8| (hex / width.max(1), hex % width.max(1));
    let mut list = Vec::new();
    // The background buffer (§4.4): the battlefield's picture, then the absolute obstacle. The
    // picture (0x4646D0, §5): a special battlefield's own (the shore, a special ground's field, a
    // ship's deck, two boats; E17 M3 T2); else the terrain's. Which of a terrain's three pictures
    // a battle gets depends on the adventure map around it (0x41ADB0, §4.4), which a battle's
    // record does not carry: an empty surrounding gives the third, which the reference captures
    // confirm (*provisional*, E17 M2 T1); a field with one picture has it. Water and rock have
    // none (the original fights on water on a deck or between boats only): a key the tables lack.
    let field = match (observation.battlefield, observation.terrain) {
        (Some(handle), _) => Some(key(&keys.battlefields, "battlefields", handle)?),
        (None, Some(handle)) => Some(key(&keys.terrains, "terrains", handle)?),
        (None, None) => None,
    };
    // A siege (E17 M4 T1b, design §10.1): with a fort the town's background, walls and turrets.
    let siege = match &observation.siege {
        Some(siege) => {
            let faction = key(&keys.factions, "factions", siege.faction)?;
            let context = Context {
                walls: moment.walls.unwrap_or_else(|| Walls::of(siege)),
                stacks: &observation.stacks,
                active: moment
                    .active
                    .map(|id| (id, layout.outline_after(moment.pulse))),
                turrets: moment.turrets,
            };
            SiegeLook::new(siege, faction, tables, context, width)?
        }
        None => None,
    };
    if let Some(siege) = &siege {
        list.extend(siege.background());
    } else if let Some(key) = field {
        let field = tables
            .battlefields
            .get(key)
            .ok_or_else(|| SceneError::Key(key.to_string()))?;
        let variant = EMPTY_SURROUNDINGS.min(field.backgrounds.len().saturating_sub(1));
        list.push(Draw::Background {
            picture: field.backgrounds[variant].clone(),
            x: 0,
            y: 0,
        });
    }
    if let Some(handle) = observation.absolute_obstacle {
        let (key, art) = obstacle(manifest, tables, handle)?;
        let ObstacleArt::Absolute { pcx, at: point } = art else {
            return Err(SceneError::Art(key));
        };
        list.push(Draw::AbsoluteObstacle {
            picture: pcx.clone(),
            x: i32::from(point[0]),
            y: i32::from(point[1]),
        });
    }
    let background = list.len();
    let mut placed = Vec::new();
    for o in &observation.removable_obstacles {
        let Some(handle) = o.kind else {
            continue;
        };
        let (key, art) = obstacle(manifest, tables, handle)?;
        let ObstacleArt::Usual {
            def,
            rows,
            before_stacks,
        } = art
        else {
            return Err(SceneError::Art(key));
        };
        // The canvas's top-left: the anchor hex's box, raised by the rows above it (§4.4).
        let (row, column) = at(o.anchor);
        let [x, y] = layout.hex_box(row, column);
        placed.push(Placed {
            def: def.clone(),
            frame: 0,
            before_stacks: *before_stacks,
            at: (row, column),
            x,
            y: y - layout.hex_step[1] * (i32::from(*rows) - 1),
        });
    }
    // The spells' battlefield objects go through the same passes, their frame by the idle steps
    // (`battle_redraw` 0x494680: +0x13FFC modulo the group's frames; E17 M3 T7c-2). The original
    // draws one its owner placed hidden only when it shows to the side whose view the screen
    // shows (0x49460A to 0x49463A); a record holds those its viewer sees.
    for (&anchor, o) in &moment.objects {
        let (row, column) = at(anchor);
        let [x, y] = layout.hex_box(row, column);
        placed.push(Placed {
            def: o.def.clone(),
            frame: moment.pulse % o.frames.max(1),
            before_stacks: o.before_stacks,
            at: (row, column),
            x,
            y: y - layout.hex_step[1] * (i32::from(o.rows) - 1),
        });
    }
    let obstacle_draw = |o: &Placed| Draw::Obstacle {
        def: o.def.clone(),
        group: 0,
        frame: o.frame,
        x: o.x,
        y: o.y,
    };
    // Layer-1 obstacles, by anchor: rows top to bottom, the layout's columns left to right.
    let [first, last] = layout.before_stacks_columns;
    for row in 0..height {
        for column in first..=last {
            let here = placed
                .iter()
                .filter(|o| o.before_stacks && o.at == (row, column));
            list.extend(here.map(obstacle_draw));
        }
    }
    // A siege's back wall (0x4946BB).
    list.extend(siege.as_ref().and_then(SiegeLook::back_wall));
    // The heroes (§4.5, `battle_redraw` 0x494701, after the siege's back wall): each side's flag,
    // then its hero, the left side first; the right hero mirrored in its canvas.
    let standing = u32::from(tables.battle.hero_groups.standing);
    for (side, view) in observation.sides.iter().enumerate() {
        let Some(hero) = &view.hero else {
            continue;
        };
        let key = key(&keys.hero_types, "hero types", hero.hero_type)?;
        let def = tables
            .heroes
            .get(key)
            .ok_or_else(|| SceneError::Key(key.to_string()))?;
        let pose = moment.heroes[side].unwrap_or(HeroPose {
            group: standing,
            frame: 0,
            flag: FLAG_START[side],
        });
        let ([fx, fy], [hx, hy]) = (layout.flag_canvas[side], layout.hero_canvas[side]);
        list.push(Draw::Flag {
            def: tables.battle.flags[side].clone(),
            frame: pose.flag,
            x: fx,
            y: fy,
            palette: layout.player_palette.clone(),
            player: PLAYERS[side],
        });
        list.push(Draw::Hero {
            def: def.def.clone(),
            group: pose.group,
            frame: pose.frame,
            x: hx,
            y: hy,
            mirrored: side == 1,
        });
    }
    // A siege's parts on hex 255: the upper tower, its shooter and cover (0x4947AB).
    if let Some(siege) = &siege {
        list.extend(siege.parts_at(BEFORE_ROWS.0, BEFORE_ROWS.1, layout));
    }
    // The hexes a count box cannot take (0x43E605): a living stack's (each of its poses' hexes)
    // while it holds them (not dead since, its box not hidden: E17 M3 T5a), and the blocked ones.
    let mut taken: BTreeSet<u8> = observation.obstacles.iter().copied().collect();
    taken.extend(
        moment
            .objects
            .values()
            .flat_map(|o| o.blocked.iter().copied()),
    );
    // A siege's standing walls (E17 M4 T2c-1).
    if let Some(s) = &observation.siege {
        taken.extend(moment.walls.unwrap_or_else(|| Walls::of(s)).blocking(s));
    }
    for s in observation.stacks.iter().filter(|s| s.alive) {
        let pose = pose_of(s, moment, tables);
        if !pose.holds_hex() {
            continue;
        }
        taken.insert(pose.hex);
        if s.two_hex {
            taken.extend(ahead(pose.hex, !pose.mirrored, 0));
        }
    }
    // A stack gone without a corpse (`anim::GONE`, E17 M3 T9d) is not drawn, nor one off the
    // field: a siege's turrets (their shooters are the town's, `draw_hex_stack` 0x494F4F).
    let cells = u16::from(width) * u16::from(height);
    let stacks = observation
        .stacks
        .iter()
        .filter(|s| u16::from(pose_of(s, moment, tables).hex) < cells)
        .filter(|s| match moment.poses.get(&s.id) {
            Some(pose) => pose.layer != GONE,
            None => s.alive,
        })
        .map(|s| {
            let pose = pose_of(s, moment, tables);
            let sprite = creature(s, pose, moment, manifest, tables, layout, at)?;
            // A magic animation over the stack is drawn after it and its count box (E17 M3 T3b,
            // T5b-2b: `draw_battle_stack`'s part 0 runs on from the sprite into the box,
            // 0x43E568, then draws the animation, 0x43E795; a siege's second call, part 1 after
            // the moat, draws the box again: not built).
            let magic = moment.magic.as_ref().and_then(|m| {
                let &(_, [x, y], mirrored) = m.over.iter().find(|o| o.0 == s.id)?;
                Some(Draw::Magic {
                    def: m.def.clone(),
                    frame: m.frame,
                    x,
                    y,
                    mirrored,
                    translucent: m.translucent,
                })
            });
            let shown = moment.counts.get(&s.id).copied().unwrap_or(s.count);
            let hue = moment.hues.get(&s.id).copied();
            let count = count_box(s, pose, (shown, hue), &sprite, &taken, layout);
            // In a siege the moat over the stack and its box again (E17 M4 T1b, 0x494FCE).
            let over = siege.as_ref().map_or_else(Vec::new, |siege| {
                let stack = (pose.hex, s.two_hex, pose.mirrored);
                siege.over_stack(stack, pose.layer, layout, count.as_ref())
            });
            let draws: Vec<Draw> = [Some(sprite.draw), count, magic]
                .into_iter()
                .flatten()
                .chain(over)
                .collect();
            Ok((sprite.at, pose.layer, draws))
        })
        .collect::<Result<Vec<_>, SceneError>>()?;
    for row in 0..height {
        // A siege's columns run right to left from row 6, its gate row's drawbridge before the
        // passes and its rope after them (0x4947C8 to 0x494A5D, design §10.1).
        let (columns, d) = siege
            .as_ref()
            .map_or_else(|| ((0..width).collect(), 1), |s| s.columns(row));
        if row == GATE_ROW {
            list.extend(siege.as_ref().and_then(SiegeLook::drawbridge));
        }
        for pass in 0..PASSES {
            for &column in &columns {
                if pass == 0
                    && let Some(siege) = &siege
                {
                    list.extend(siege.parts_at(row * width + column, d, layout));
                }
                if pass == OBSTACLE_PASS {
                    let here = placed
                        .iter()
                        .filter(|o| !o.before_stacks && o.at == (row, column));
                    list.extend(here.map(obstacle_draw));
                }
                let here = stacks
                    .iter()
                    .filter(|(at, layer, _)| *layer == pass && *at == (row, column));
                list.extend(here.flat_map(|(_, _, draws)| draws.iter().cloned()));
            }
        }
        if row == GATE_ROW {
            list.extend(siege.as_ref().and_then(SiegeLook::rope));
        }
    }
    // A siege's parts on hexes 200 and 251: the lower wall's last strip, the lower tower, its
    // shooter and cover (0x494A74).
    if let Some(siege) = &siege {
        for (hex, d) in AFTER_ROWS {
            list.extend(siege.parts_at(hex, d, layout));
        }
    }
    // A flier between its hexes: off them, drawn by itself over everything (anim::FLYING).
    list.extend(
        stacks
            .iter()
            .filter(|(_, layer, _)| *layer >= PASSES)
            .flat_map(|(_, _, draws)| draws.iter().cloned()),
    );
    if let Some(p) = &moment.projectile {
        list.push(Draw::Projectile {
            def: p.def.clone(),
            frame: p.frame,
            x: p.at[0],
            y: p.at[1],
            mirrored: p.mirrored,
        });
    }
    if let Some(ray) = &moment.ray {
        list.push(Draw::Ray(ray.clone()));
    }
    // A magic animation at a hex, over everything on the screen (0x496A10 through 0x47C170, E17
    // M3 T7c-1).
    if let Some(spot) = &moment.spot {
        list.push(Draw::Magic {
            def: spot.def.clone(),
            frame: spot.frame,
            x: spot.at[0],
            y: spot.at[1],
            mirrored: false,
            translucent: spot.translucent,
        });
    }
    // The magic animation over the whole battlefield, tiled over it after the redraw (Armageddon's
    // 0x5A4FEF to 0x5A50AD; E17 M3 T7c-2).
    if let Some(f) = &moment.field {
        list.push(Draw::Tiled {
            def: f.def.clone(),
            frame: f.frame,
            area: layout.mouse.field,
        });
    }
    Ok((background, list))
}

/// A stack's pose at `moment`: its own, else standing on its hex, facing its side's way.
fn pose_of(stack: &Stack, moment: &Moment, tables: &Tables) -> Pose {
    moment.poses.get(&stack.id).copied().unwrap_or(Pose {
        group: u32::from(tables.battle.groups.standing),
        frame: 0,
        hex: stack.hex,
        offset: [0, 0],
        mirrored: stack.side == Side::Defender,
        layer: STANDING,
        boxless: false,
    })
}

/// The hex `1 + extra` ahead of `hex` facing right or left, if there is one.
fn ahead(hex: u8, right: bool, extra: u8) -> Option<u8> {
    let step = 1 + extra;
    if right {
        hex.checked_add(step)
    } else {
        hex.checked_sub(step)
    }
}

/// A stack's count box, if it shows one (§4.3, 0x43E568): a living stack, not a war machine,
/// standing or under the mouse (groups 2 and 1), its box not hidden by a walk or a flight (its
/// +0x30, [`Pose::boxless`]: E17 M3 T5a). Beside its sprite's anchor by its facing, its width and
/// whether the hex ahead is `taken`; its number the count `shown` (E17 M3 T5b: the snapshot's,
/// then each strike's from its end), its tint's hue the `hue` of its spells if it has any (E17
/// M3 T7b-2a), else the layout's.
fn count_box(
    stack: &Stack,
    pose: Pose,
    (shown, hue): (u32, Option<u32>),
    sprite: &Sprite,
    taken: &BTreeSet<u8>,
    layout: &BattleLayout,
) -> Option<Draw> {
    // A stack among the corpses shows none, even standing as a raise lifts it (E17 M3 T7c-3,
    // `raise`: its dead bit is cleared only at the raise's end, 0x5A7867).
    if !stack.alive || stack.war_machine || !sprite.boxed || !pose.holds_hex() {
        return None;
    }
    let right = !pose.mirrored;
    let wide = stack.two_hex;
    let blocked = ahead(pose.hex, right, u8::from(wide)).is_some_and(|h| taken.contains(&h));
    let [x, y] = layout
        .count_box
        .corner(sprite.anchor, right, wide, blocked, sprite.troop_offset);
    Some(Draw::CountBox {
        stack: stack.id,
        x,
        y,
        count: shown,
        tint: [
            hue.unwrap_or(layout.count_box.tint[0]),
            layout.count_box.tint[1],
        ],
    })
}

/// The fractional bits of [`spell_hue`]'s fixed point: every value it rounds is a multiple of
/// 2^−86 below 2.
const HUE_FRAC: u32 = 96;

/// The hue of the count box of a stack with active spells (`draw_battle_stack` 0x43E6AA to
/// 0x43E742, design §4.3; E17 M3 T7b-2a), as the bits of the f32 the executable passes: r = Σ k
/// / Σ |k| over the `signs` k of the stack's active spells (the spell records' +0: 1 positive,
/// −1 negative, 0 neither; r = 0 when every one is 0), then f32((r + 1) × 0.1667), the constant
/// f32(0.1667) widened, at the x87's 53 bits: r, r + 1 and the product each rounded to 53
/// significant bits, the last to 24 by the store, ties to even. All positive spells give green
/// (2 × 0.1667), all negative red (0), a mix the hues between. Computed exactly in integers (no
/// float arithmetic in the client, ADR-0003 §3).
pub fn spell_hue(signs: impl IntoIterator<Item = i8>) -> u32 {
    /// f32(0.1667): its 24-bit mantissa; the value is it × 2^−26.
    const SIXTH: u128 = 0xAA_B368;
    const SIXTH_SHIFT: u32 = 26;
    let one: u128 = 1 << HUE_FRAC;
    let (sum, magnitude) = signs.into_iter().fold((0i32, 0u32), |(s, m), k| {
        (s + i32::from(k), m + u32::from(k.unsigned_abs()))
    });
    // r rounded to 53 bits (its quotient's remainder decides a tie), then r + 1, exact.
    let r = if magnitude == 0 {
        0
    } else {
        let scaled = u128::from(sum.unsigned_abs()) << HUE_FRAC;
        let den = u128::from(magnitude);
        round_significant(scaled / den, 53, !scaled.is_multiple_of(den))
    };
    let v = round_significant(if sum < 0 { one - r } else { one + r }, 53, false);
    let w = round_significant((v * SIXTH) >> SIXTH_SHIFT, 53, false);
    let h = round_significant(w, 24, false);
    if h == 0 {
        return 0;
    }
    // h = 1.f × 2^e: its exponent and its 23 fraction bits.
    let width = 128 - h.leading_zeros();
    let exponent = i64::from(width) - 1 - i64::from(HUE_FRAC) + 127;
    let fraction = (h >> (width - 24)) & 0x7F_FFFF;
    u32::try_from(exponent).unwrap_or(0) << 23 | u32::try_from(fraction).unwrap_or(0)
}

/// `v` rounded to `bits` significant bits, ties to even; `sticky`: `v` was cut below its last
/// bit, so a half is more than half.
fn round_significant(v: u128, bits: u32, sticky: bool) -> u128 {
    let width = 128 - v.leading_zeros();
    if width <= bits {
        return v;
    }
    let shift = width - bits;
    let (kept, rest) = (v >> shift, v & ((1 << shift) - 1));
    let half = 1 << (shift - 1);
    let up = rest > half || (rest == half && (sticky || kept & 1 == 1));
    (kept + u128::from(up)) << shift
}

/// A stack's sprite as the scene places it.
struct Sprite {
    /// Its position (row, column).
    at: (u8, u8),
    /// Where it stands: its anchor moved by its pose's offset.
    anchor: [i32; 2],
    /// Its creature's troop-count offset (§4.3).
    troop_offset: i32,
    /// Its pose's group shows a count box: standing or under the mouse (groups 2 and 1).
    boxed: bool,
    /// Its draw.
    draw: Draw,
}

/// A stack's sprite in its pose at `moment`.
fn creature(
    stack: &Stack,
    pose: Pose,
    moment: &Moment,
    manifest: &Manifest,
    tables: &Tables,
    layout: &BattleLayout,
    at: impl Fn(u8) -> (u8, u8),
) -> Result<Sprite, SceneError> {
    let key = key(&manifest.keys.creatures, "creatures", stack.creature)?;
    let entry = tables
        .creatures
        .get(key)
        .ok_or_else(|| SceneError::Key(key.to_string()))?;
    // Drawn once, from its position (§4.2): the engine's position is the executable's, a two-hex
    // stack's rear hex (BAT-GEO-002 since PR #150, protocol v23); moved by its pose's offset.
    let (row, column) = at(pose.hex);
    let [x, y] = layout.anchor(row, column);
    let anchor = [x + pose.offset[0], y + pose.offset[1]];
    let draw = Draw::Creature {
        def: entry.def.clone(),
        group: pose.group,
        frame: pose.frame,
        anchor,
        mirrored: pose.mirrored,
        outline: if moment.area.contains(&stack.id) {
            Some(layout.mouse_outline_after(moment.pulse))
        } else if moment.active == Some(stack.id) {
            Some(layout.outline_after(moment.pulse))
        } else {
            (moment.marked == Some(stack.id)).then(|| layout.mouse_outline_after(moment.pulse))
        },
    };
    let groups = tables.battle.groups;
    Ok(Sprite {
        at: (row, column),
        anchor,
        troop_offset: entry.troop_count_offset,
        boxed: [groups.standing, groups.mouse_over]
            .map(u32::from)
            .contains(&pose.group),
        draw,
    })
}

/// The key of `handle` in a manifest's key table.
fn key<'a>(table: &'a [String], name: &'static str, handle: u16) -> Result<&'a str, SceneError> {
    table
        .get(usize::from(handle))
        .map(String::as_str)
        .ok_or(SceneError::Handle(name, handle))
}

/// An obstacle type's key and graphics.
fn obstacle<'t>(
    manifest: &Manifest,
    tables: &'t Tables,
    handle: u16,
) -> Result<(String, &'t ObstacleArt), SceneError> {
    let key = key(&manifest.keys.obstacles, "obstacles", handle)?;
    let art = tables
        .obstacles
        .get(key)
        .ok_or_else(|| SceneError::Key(key.to_string()))?;
    Ok((key.to_string(), art))
}

#[cfg(test)]
pub(crate) mod tests;
